use std::collections::HashMap;
use crate::AudioFile;
use crate::utils::parallel::ParallelProcessor;
use serde::Serialize;
use std::fmt;
use rayon::prelude::*;

#[derive(Debug, Serialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BitrateCategory {
    HighRes,       // 1500+ kbps
    Lossless,      // 700-1499 kbps
    High,          // 256-400 kbps
    Standard,      // 160-255 kbps
    Low,           // 64-159 kbps
    Unknown        // Everything else
}

impl BitrateCategory {
    pub fn from_bitrate(bitrate: u32) -> Self {
        match bitrate {
            1500.. => BitrateCategory::HighRes,
            700..=1499 => BitrateCategory::Lossless,
            256..=400 => BitrateCategory::High,    // Extended range to catch VBR variations
            160..=255 => BitrateCategory::Standard,
            64..=159 => BitrateCategory::Low,
            _ => BitrateCategory::Unknown
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            BitrateCategory::HighRes => "High-Resolution (1500+ kbps)",
            BitrateCategory::Lossless => "Lossless (700-1499 kbps)",
            BitrateCategory::High => "High Bitrate (256-400 kbps)",
            BitrateCategory::Standard => "Standard Bitrate (160-255 kbps)",
            BitrateCategory::Low => "Low Bitrate (64-159 kbps)",
            BitrateCategory::Unknown => "Other"
        }
    }
}

impl fmt::Display for BitrateCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize)]
pub struct BitrateStats {
    pub file_count: usize,
    pub category_distribution: HashMap<BitrateCategory, usize>,
    pub average_bitrate: f64,
    pub min_bitrate: u32,
    pub max_bitrate: u32,
}

pub struct BitrateAnalyzer;

impl ParallelProcessor for BitrateAnalyzer {}

impl BitrateAnalyzer {
    pub fn new() -> Self {
        println!("Initializing BitrateAnalyzer");
        Self
    }

    // Reusable quality comparison function
    pub fn compare_quality(file1: &AudioFile, file2: &AudioFile) -> (bool, String) {
        match (file1.bitrate, file2.bitrate) {
            (Some(b1), Some(b2)) if b1 != b2 => {
                let file1_better = match (file1.file_name.ends_with(".flac"), 
                                        file2.file_name.ends_with(".flac")) {
                    (true, false) => true,
                    (false, true) => false,
                    _ => b1 > b2,
                };
                (file1_better, format!("Bitrate difference: {} vs {} kbps", b1, b2))
            },
            _ if file1.size_bytes != file2.size_bytes => {
                let file1_better = file1.size_bytes > file2.size_bytes;
                let size1_mb = file1.size_bytes as f64 / 1_048_576.0;
                let size2_mb = file2.size_bytes as f64 / 1_048_576.0;
                (file1_better, format!("Size difference: {:.2} MB vs {:.2} MB", size1_mb, size2_mb))
            },
            _ => (true, "Files are identical in size and bitrate".to_string())
        }
    }

    pub fn analyze(&self, files: &[AudioFile]) -> BitrateStats {
        println!("Starting bitrate analysis of {} files using {} threads", 
            files.len(), 
            rayon::current_num_threads()
        );
        
        let progress = Self::get_progress_counter();
        let total_files = files.len();

        // Process files in parallel
        let results: Vec<_> = files.par_iter()
            .filter_map(|file| file.bitrate.map(|b| (file, b)))
            .inspect(|(file, bitrate)| {
                let processed = progress.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if processed % 100 == 0 || processed == total_files {
                    println!("Progress: {}/{} files ({:.1}%)", 
                        processed, total_files,
                        (processed as f64 / total_files as f64) * 100.0
                    );
                }
                
                println!("Processed '{}' - {} kbps ({})", 
                    file.file_name, 
                    bitrate,
                    BitrateCategory::from_bitrate(*bitrate).as_str()
                );
            })
            .collect();

        // Calculate statistics
        let mut category_distribution: HashMap<BitrateCategory, usize> = HashMap::new();
        let mut total_bitrate = 0.0;
        let mut min_bitrate = u32::MAX;
        let mut max_bitrate = 0;

        for (_, bitrate) in &results {
            let category = BitrateCategory::from_bitrate(*bitrate);
            *category_distribution.entry(category).or_insert(0) += 1;
            total_bitrate += *bitrate as f64;
            min_bitrate = min_bitrate.min(*bitrate);
            max_bitrate = max_bitrate.max(*bitrate);
        }

        let stats = BitrateStats {
            file_count: files.len(),
            category_distribution,
            average_bitrate: if !results.is_empty() {
                total_bitrate / results.len() as f64
            } else {
                0.0
            },
            min_bitrate: if min_bitrate == u32::MAX { 0 } else { min_bitrate },
            max_bitrate,
        };

        Self::print_summary(&stats);
        stats
    }

    fn print_summary(stats: &BitrateStats) {
        println!("\nBitrate Analysis Summary:");
        println!("Total files: {}", stats.file_count);
        println!("Files with valid bitrate: {}", stats.category_distribution.values().sum::<usize>());
        println!("Average bitrate: {:.1} kbps", stats.average_bitrate);
        println!("Min bitrate: {} kbps", stats.min_bitrate);
        println!("Max bitrate: {} kbps", stats.max_bitrate);
        println!("\nBitrate Distribution:");
        
        let total_processed = stats.category_distribution.values().sum::<usize>();
        let mut categories: Vec<_> = stats.category_distribution.iter().collect();
        categories.sort_by(|a, b| b.0.cmp(a.0));
        
        for (category, count) in &categories {
            let percentage = (**count as f64 / total_processed as f64 * 100.0).round();
            println!("{}: {} files ({:.1}%)", category.as_str(), count, percentage);
        }
    }
}