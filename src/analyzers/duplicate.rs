use crate::AudioFile;
use crate::utils::parallel::ParallelProcessor;
use std::sync::atomic::Ordering;
use regex::Regex;
use std::sync::Arc;

#[derive(Debug)]
pub struct DuplicateMatch {
    pub higher_quality: AudioFile,
    pub lower_quality: AudioFile,
    pub match_reason: String,
    pub quality_difference: String
}

#[derive(Debug)]
pub struct DuplicateResults {
    pub matches: Vec<DuplicateMatch>,
    pub total_files_scanned: usize,
}

pub struct DuplicateAnalyzer {
    title_regex: Arc<Regex>,
}

impl ParallelProcessor for DuplicateAnalyzer {}

impl DuplicateAnalyzer {
    pub fn new(_threshold: f64) -> Self {
        Self::init_parallel_processing();
        println!("Initializing DuplicateAnalyzer");
        Self {
            title_regex: Arc::new(Regex::new(r"^\d+\.?\s*").unwrap()),
        }
    }

    fn clean_title(&self, filename: &str) -> (String, Option<String>, Option<String>) {
        // Remove file extension
        let without_ext = filename.rfind('.').map_or(filename, |i| &filename[..i]);

        // Remove track numbers in brackets or with dots
        let without_numbers = without_ext
            .replace(|c| c == '[' || c == ']', "")
            .replace(|c| c == '_', " ")
            .trim()
            .to_string();

        // Remove leading numbers and dots using pre-compiled regex
        let without_numbers = self.title_regex.replace(&without_numbers, "").to_string();

        // Split artist and title
        let parts: Vec<&str> = without_numbers.split(" - ").collect();
        if parts.len() < 2 {
            return (without_numbers, None, None);
        }

        let artist = parts[0].trim().to_lowercase();
        let mut title_parts = parts[1..].join(" - ");

        // Extract version info in parentheses
        let version = if let Some(paren_start) = title_parts.rfind('(') {
            if let Some(paren_end) = title_parts[paren_start..].find(')') {
                let version = title_parts[paren_start + 1..paren_start + paren_end].trim().to_lowercase();
                title_parts = title_parts[..paren_start].trim().to_string();
                Some(version)
            } else {
                None
            }
        } else {
            None
        };

        (artist, Some(title_parts.trim().to_lowercase()), version)
    }

    fn are_duplicates(&self, file1: &AudioFile, file2: &AudioFile) -> Option<DuplicateMatch> {
        let (artist1, title1, version1) = self.clean_title(&file1.file_name);
        let (artist2, title2, version2) = self.clean_title(&file2.file_name);

        // Must have exact artist match
        if artist1 != artist2 {
            return None;
        }

        // Must have title portion after the dash
        let (Some(title1), Some(title2)) = (title1, title2) else {
            return None;
        };

        // Must have exact main title match
        if title1 != title2 {
            return None;
        }

        // Different versions are not duplicates
        match (version1, version2) {
            (Some(v1), Some(v2)) if v1 != v2 => {
                // Check for significant version differences
                let version_keywords = [
                    "remix", "edit", "version", "mix", "remaster", 
                    "extended", "radio", "club", "instrumental", "dub",
                    "original", "rework", "reconstruction", "vip", "bootleg",
                    "mashup", "flip", "cut", "recut", "reprise"
                ];

                // If either version contains any of these keywords, they're different versions
                for keyword in version_keywords {
                    if (v1.contains(keyword) || v2.contains(keyword)) && v1 != v2 {
                        return None;
                    }
                }
            }
            _ => {}
        }

        let match_reason = format!("Exact title match: '{} - {}'", artist1, title1);
        let (file1_better, quality_difference) = self.determine_quality_difference(file1, file2);
        
        let (higher, lower) = if file1_better {
            (file1.clone(), file2.clone())
        } else {
            (file2.clone(), file1.clone())
        };

        Some(DuplicateMatch {
            higher_quality: higher,
            lower_quality: lower,
            match_reason,
            quality_difference,
        })
    }

    fn determine_quality_difference(
        &self,
        file1: &AudioFile,
        file2: &AudioFile
    ) -> (bool, String) {
        // Compare bitrates first
        match (file1.bitrate, file2.bitrate) {
            (Some(b1), Some(b2)) if b1 != b2 => {
                // Consider format differences (e.g., FLAC vs MP3)
                let file1_better = if file1.file_name.ends_with(".flac") && !file2.file_name.ends_with(".flac") {
                    true
                } else if !file1.file_name.ends_with(".flac") && file2.file_name.ends_with(".flac") {
                    false
                } else {
                    b1 > b2
                };
                return (file1_better, format!("Bitrate difference: {} vs {} kbps", b1, b2));
            }
            _ => {}
        }

        // If bitrates are same or unavailable, compare file sizes
        if file1.size_bytes != file2.size_bytes {
            let file1_better = file1.size_bytes > file2.size_bytes;
            let size1_mb = file1.size_bytes as f64 / 1_048_576.0;
            let size2_mb = file2.size_bytes as f64 / 1_048_576.0;
            return (file1_better, format!("Size difference: {:.2} MB vs {:.2} MB", size1_mb, size2_mb));
        }

        // If everything is equal, keep the first one
        (true, "Files are identical in size and bitrate".to_string())
    }

    pub fn find_duplicates(&self, files: Vec<AudioFile>) -> DuplicateResults {
        println!("Starting duplicate analysis with {} files using {} threads", 
            files.len(), 
            rayon::current_num_threads()
        );

        if files.is_empty() {
            println!("No files to analyze!");
            return DuplicateResults { matches: Vec::new(), total_files_scanned: 0 };
        }

        let progress = Self::get_progress_counter();
        let total_files = files.len();

        // Use parallel comparison for finding duplicates
        let matches = Self::parallel_compare(&files, |file1, file2| {
            let result = self.are_duplicates(file1, file2);
            
            if result.is_some() {
                let dup = result.as_ref().unwrap();
                println!("\nFound duplicate:");
                println!("  Higher quality: {} ({} kbps)", 
                    dup.higher_quality.file_name, 
                    dup.higher_quality.bitrate.unwrap_or(0));
                println!("  Lower quality: {} ({} kbps)", 
                    dup.lower_quality.file_name, 
                    dup.lower_quality.bitrate.unwrap_or(0));
                println!("  Reason: {}", dup.match_reason);
                println!("  Quality difference: {}", dup.quality_difference);
            }

            let processed = progress.fetch_add(1, Ordering::SeqCst) + 1;
            if processed % 1000 == 0 || processed == total_files {
                println!("Progress: processed {} file comparisons", processed);
            }

            result
        });

        println!("\nFound {} duplicate matches", matches.len());
        DuplicateResults {
            matches,
            total_files_scanned: total_files
        }
    }
}