use crate::AudioFile;
use crate::utils::parallel::ParallelProcessor;
use crate::analyzers::bitrate::BitrateAnalyzer;
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

#[derive(Debug, PartialEq)]
enum VersionType {
    None,
    WithMarkers(Vec<String>),
}

impl VersionType {
    fn from_str(text: Option<&str>) -> Self {
        let markers = [
            // Remix and edit types
            "remix", "mix", "rmx", "rework", "edit", "reconstruction",
            "bootleg", "mashup", "flip", "recut", "reprise",
            // Version types
            "version", "radio", "club", "special", "extended",
            // DJ markers
            "dj", "vs", "presents", 
            // Release types
            "remaster", "master", "remastered",
            // Mix types
            "dub", "instrumental", "acapella", "acoustic", "live",
            // Length markers
            "long", "short", "full", "cut", "original",
            // Regional markers
            "us", "uk", "euro", "italian", "spanish", "dutch",
            // Special combinations
            "radio edit", "club mix", "dance mix", "extended mix"
        ];

        match text {
            None => Self::None,
            Some(text) => {
                let text_lower = text.to_lowercase();
                let found_markers: Vec<String> = markers.iter()
                    .filter(|&&m| text_lower.contains(m))
                    .map(|&s| s.to_string())
                    .collect();

                if found_markers.is_empty() && text_lower.chars()
                    .filter(|c| c.is_ascii_digit())
                    .count() >= 4 {
                    Self::WithMarkers(vec!["year".to_string()])
                } else if !found_markers.is_empty() {
                    Self::WithMarkers(found_markers)
                } else {
                    Self::None
                }
            }
        }
    }

    fn share_markers(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::WithMarkers(m1), Self::WithMarkers(m2)) => 
                m1.iter().any(|m| m2.contains(m)),
            _ => false
        }
    }
}

#[derive(Debug)]
struct ParsedTitle {
    artist: String,
    title: String,
    version: Option<String>,
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

    fn normalize_artist(artist: &str) -> String {
        let normalized = artist
            .to_lowercase()
            .replace("feat.", "featuring")
            .replace("ft.", "featuring")
            .replace(" x ", " featuring ");

        let mut artists: Vec<_> = normalized
            .split(',')
            .map(|s| {
                let artist_name = s.trim();
                // Remove any parenthetical content from artist names
                artist_name
                    .split('(')
                    .next()
                    .unwrap_or(artist_name)
                    .trim()
                    .to_string()
            })
            .collect();

        artists.sort();
        artists.join(", ")
    }

    fn extract_version(text: &str) -> (String, Option<String>) {
        match (text.rfind('('), text[..].rfind(')')) {
            (Some(start), Some(end)) if start < end => {
                let version_text = text[start + 1..end].trim();
                match VersionType::from_str(Some(version_text)) {
                    VersionType::None => (text.trim().to_string(), None),
                    VersionType::WithMarkers(_) => (
                        text[..start].trim().to_string(),
                        Some(version_text.to_lowercase())
                    ),
                }
            },
            _ => (text.trim().to_string(), None)
        }
    }

    fn clean_title(&self, filename: &str) -> ParsedTitle {
        let without_ext = filename.rfind('.').map_or(filename, |i| &filename[..i]);
        
        let clean_name = without_ext
            .replace(|c| c == '[' || c == ']', "")
            .replace('_', " ")
            .trim()
            .to_string();

        let without_numbers = self.title_regex.replace(&clean_name, "").to_string();

        let parts: Vec<&str> = without_numbers.split(" - ").collect();
        if parts.len() < 2 {
            return ParsedTitle {
                artist: without_numbers.clone(),
                title: without_numbers,
                version: None,
            };
        }

        let artist = Self::normalize_artist(parts[0].trim());
        let title_parts = parts[1..].join(" - ");
        let (clean_title, version) = Self::extract_version(&title_parts);

        ParsedTitle {
            artist,
            title: clean_title.to_lowercase(),
            version,
        }
    }

    fn are_different_versions(version1: Option<&str>, version2: Option<&str>) -> bool {
        let v1_type = VersionType::from_str(version1);
        let v2_type = VersionType::from_str(version2);

        match (v1_type, v2_type) {
            (VersionType::None, VersionType::None) => false,
            (VersionType::WithMarkers(_), VersionType::WithMarkers(_)) 
                if version1 == version2 => false,
            (v1, v2) => !v1.share_markers(&v2)
        }
    }

    fn get_formatted_reason(&self, parsed: &ParsedTitle, version: Option<&str>) -> String {
        let version_info = version.map_or(String::new(), |v| format!(" ({})", v));
        format!("Exact title match: '{} - {}{}'", parsed.artist, parsed.title, version_info)
    }

    fn are_duplicates(&self, file1: &AudioFile, file2: &AudioFile) -> Option<DuplicateMatch> {
        let parsed1 = self.clean_title(&file1.file_name);
        let parsed2 = self.clean_title(&file2.file_name);

        // Early returns for non-matches
        if parsed1.artist != parsed2.artist || parsed1.title != parsed2.title {
            return None;
        }

        // Check for different versions
        if Self::are_different_versions(parsed1.version.as_deref(), parsed2.version.as_deref()) {
            return None;
        }

        // Use BitrateAnalyzer for quality comparison
        let (file1_better, quality_difference) = BitrateAnalyzer::compare_quality(file1, file2);
        let match_reason = self.get_formatted_reason(&parsed1, parsed1.version.as_deref());
        
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