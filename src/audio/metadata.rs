use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use crate::{AudioFile, Result, AudioError};
use rayon::prelude::*;

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract_metadata(path: impl AsRef<Path>) -> Result<AudioFile> {
        let path = path.as_ref();
        let file = std::fs::File::open(path)?;
        
        // Get basic file info
        let file_metadata = file.metadata()?;
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AudioError::Metadata("Invalid filename".into()))?
            .to_string();

        // Create media source stream
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        
        // Create hint to help with format detection
        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(extension);
        }

        // Probe the media source
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| AudioError::Metadata(e.to_string()))?;

        let mut format = probed.format;
        
        // Extract metadata
        let mut audio_file = AudioFile {
            path: path.to_path_buf(),
            file_name,
            size_bytes: file_metadata.len(),
            duration_secs: None,
            bitrate: None,
            artist: None,
            title: None,
            album: None,
        };

        // Try to get format info
        if let Some(track) = format.default_track() {
            let params = &track.codec_params;
            
            // Get duration if available
            if let Some(time_base) = params.time_base {
                if let Some(n_frames) = params.n_frames {
                    let time = time_base.calc_time(n_frames);
                    audio_file.duration_secs = Some(time.seconds as f64 + time.frac as f64 / 1_000_000_000.0);
                }
            }
            
            // Calculate bitrate from file size and duration
            if let Some(duration) = audio_file.duration_secs {
                if duration > 0.0 {
                    let bitrate = (file_metadata.len() * 8) as f64 / duration;
                    audio_file.bitrate = Some((bitrate / 1000.0) as u32); // Convert to kbps
                }
            }
        }

        // Get additional metadata if available
        if let Some(metadata) = format.metadata().current() {
            for tag in metadata.tags() {
                match tag.std_key {
                    Some(symphonia::core::meta::StandardTagKey::Artist) => {
                        audio_file.artist = Some(tag.value.to_string());
                    }
                    Some(symphonia::core::meta::StandardTagKey::TrackTitle) => {
                        audio_file.title = Some(tag.value.to_string());
                    }
                    Some(symphonia::core::meta::StandardTagKey::Album) => {
                        audio_file.album = Some(tag.value.to_string());
                    }
                    _ => {}
                }
            }
        }

        println!("Processed file: {} (Size: {} bytes, Duration: {:?}s, Bitrate: {:?}kbps)",
            audio_file.file_name,
            audio_file.size_bytes,
            audio_file.duration_secs,
            audio_file.bitrate
        );

        Ok(audio_file)
    }

    pub fn process_directories(dirs: &[impl AsRef<Path>]) -> Result<Vec<AudioFile>> {
        let mut all_files = Vec::new();
        
        for dir in dirs {
            println!("Processing directory: {}", dir.as_ref().display());
            let files = Self::process_directory(dir)?;
            println!("Found {} valid audio files in directory", files.len());
            all_files.extend(files);
        }
        
        println!("Total audio files found: {}", all_files.len());
        Ok(all_files)
    }

    pub fn process_directory(dir: impl AsRef<Path>) -> Result<Vec<AudioFile>> {
        let dir_ref = dir.as_ref();
        
        // Try to get canonical path
        let dir_path = if let Ok(canonical) = std::fs::canonicalize(dir_ref) {
            canonical
        } else {
            dir_ref.to_path_buf()
        };
        println!("Scanning directory structure: {}", dir_path.display());

        // First, scan for subdirectories
        let subdirs: Vec<_> = walkdir::WalkDir::new(&dir_path)
            .min_depth(1)  // Skip the root dir itself
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .collect();

        if !subdirs.is_empty() {
            println!("Found {} subdirectories:", subdirs.len());
            for subdir in &subdirs {
                println!("  - {}", subdir.path().display());
            }
        } else {
            println!("No subdirectories found, processing single directory");
        }

        // Now process all files in parallel
        println!("Collecting audio files...");
        let entries: Vec<_> = walkdir::WalkDir::new(&dir_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("Error accessing entry: {}", err);
                    None
                }
            })
            .filter(|e| {
                let is_file = e.file_type().is_file();
                let has_valid_ext = if let Some(ext) = e.path().extension().and_then(|e| e.to_str()) {
                    matches!(ext.to_lowercase().as_str(), "mp3" | "wav" | "flac")
                } else {
                    false
                };
                if is_file && !has_valid_ext {
                    println!("Skipping non-audio file: {}", e.path().display());
                }
                is_file && has_valid_ext
            })
            .collect();

        println!("Found {} potential audio files", entries.len());

        // Process files in parallel using rayon
        println!("Processing files using {} threads...", rayon::current_num_threads());
        let results: Vec<Result<AudioFile>> = entries.par_iter()
            .map(|entry| Self::extract_metadata(entry.path()))
            .collect();

        // Filter out errors and collect successful results
        let files: Vec<AudioFile> = results.into_iter()
            .filter_map(|r| match r {
                Ok(file) => Some(file),
                Err(e) => {
                    eprintln!("Error processing file: {}", e);
                    None
                }
            })
            .collect();

        Ok(files)
    }
}