use rayon::prelude::*;
use crate::AudioFile;
use std::sync::Mutex;

pub struct DuplicateAnalyzer {
    similarity_threshold: f64,
}

#[derive(Debug)]
pub struct DuplicateGroup {
    pub original: AudioFile,
    pub duplicates: Vec<AudioFile>,
}

impl DuplicateAnalyzer {
    pub fn new(similarity_threshold: f64) -> Self {
        println!("Initializing DuplicateAnalyzer with threshold: {}", similarity_threshold);
        Self { similarity_threshold }
    }

    fn calculate_similarity(file1: &AudioFile, file2: &AudioFile) -> f64 {
        println!("Comparing:\n  - {}\n  - {}", file1.file_name, file2.file_name);
        let mut score = 0.0;
        let mut factors = 0.0;

        // Compare sizes (exact match gives high score)
        if file1.size_bytes == file2.size_bytes {
            score += 3.0;
            println!("  Size match ({})", file1.size_bytes);
        }
        factors += 3.0;

        // Compare durations if available
        if let (Some(d1), Some(d2)) = (file1.duration_secs, file2.duration_secs) {
            if (d1 - d2).abs() < 1.0 { // Within 1 second
                score += 2.0;
                println!("  Duration match ({:.2}s)", d1);
            }
        }
        factors += 2.0;

        // Compare metadata
        if let (Some(t1), Some(t2)) = (&file1.title, &file2.title) {
            if t1 == t2 {
                score += 2.0;
                println!("  Title match ({})", t1);
            }
        }
        if let (Some(a1), Some(a2)) = (&file1.artist, &file2.artist) {
            if a1 == a2 {
                score += 2.0;
                println!("  Artist match ({})", a1);
            }
        }
        factors += 4.0;

        let final_score = score / factors;
        println!("  Similarity score: {:.2}", final_score);
        final_score
    }

    pub fn find_duplicates(&self, files: Vec<AudioFile>) -> Vec<DuplicateGroup> {
        println!("Starting duplicate analysis with {} files", files.len());
        if files.is_empty() {
            println!("No files to analyze!");
            return Vec::new();
        }

        let processed = Mutex::new(vec![false; files.len()]);
        let groups = Mutex::new(Vec::new());

        (0..files.len()).into_par_iter().for_each(|i| {
            let mut should_process = false;
            {
                let mut processed = processed.lock().unwrap();
                if !processed[i] {
                    processed[i] = true;
                    should_process = true;
                }
            }

            if should_process {
                let mut current_duplicates = Vec::new();
                
                // Compare with remaining files
                for j in (i + 1)..files.len() {
                    let is_unprocessed = {
                        let processed = processed.lock().unwrap();
                        !processed[j]
                    };

                    if is_unprocessed {
                        let similarity = Self::calculate_similarity(&files[i], &files[j]);
                        if similarity >= self.similarity_threshold {
                            println!("Found duplicate: {} -> {}", files[i].file_name, files[j].file_name);
                            current_duplicates.push(files[j].clone());
                            let mut processed = processed.lock().unwrap();
                            processed[j] = true;
                        }
                    }
                }

                if !current_duplicates.is_empty() {
                    let mut groups = groups.lock().unwrap();
                    groups.push(DuplicateGroup {
                        original: files[i].clone(),
                        duplicates: current_duplicates,
                    });
                }
            }
        });

        let result = groups.into_inner().unwrap();
        println!("Found {} groups of duplicates", result.len());
        result
    }
}