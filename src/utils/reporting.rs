use std::path::Path;
use csv::Writer;
use crate::analyzers::bitrate::{BitrateStats, BitrateCategory};
use crate::analyzers::duplicate::DuplicateResults;
use crate::AudioFile;
use crate::Result;

pub struct Reporter;

impl Reporter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_bitrate_report(&self, stats: &BitrateStats, files: &[AudioFile], output_path: impl AsRef<Path>) -> Result<()> {
        let output_path_ref = output_path.as_ref();
        let mut summary_path = output_path_ref.to_path_buf();
        let mut detailed_path = output_path_ref.to_path_buf();
        
        // Create summary and detailed report paths
        let file_stem = output_path_ref.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("bitrate");
        
        summary_path.set_file_name(format!("{}_summary.csv", file_stem));
        detailed_path.set_file_name(format!("{}_detailed.csv", file_stem));

        // Generate summary report
        self.generate_summary_report(stats, &summary_path)?;
        
        // Generate detailed report
        self.generate_detailed_report(files, &detailed_path)?;

        Ok(())
    }

    fn generate_summary_report(&self, stats: &BitrateStats, path: &Path) -> Result<()> {
        let mut writer = Writer::from_path(path)?;
        
        // Write header
        writer.write_record(&["Category", "File Count", "Percentage"])?;

        let total_files: usize = stats.category_distribution.values().sum();

        // Sort categories for consistent output (highest to lowest quality)
        let mut categories: Vec<_> = stats.category_distribution.iter().collect();
        categories.sort_by(|a, b| b.0.cmp(a.0));

        // Write distribution by category
        for (category, count) in categories {
            let percentage = ((*count as f64 / total_files as f64) * 100.0).round();
            writer.write_record(&[
                category.to_string(),
                count.to_string(),
                format!("{:.1}%", percentage),
            ])?;
        }

        // Write summary
        writer.write_record(&["", "", ""])?;
        writer.write_record(&["Summary Statistics", "", ""])?;
        writer.write_record(&["Total Files", &stats.file_count.to_string(), ""])?;
        writer.write_record(&["Average Bitrate", &format!("{:.1} kbps", stats.average_bitrate), ""])?;
        writer.write_record(&["Min Bitrate", &format!("{} kbps", stats.min_bitrate), ""])?;
        writer.write_record(&["Max Bitrate", &format!("{} kbps", stats.max_bitrate), ""])?;

        writer.flush()?;
        println!("Summary report generated: {}", path.display());
        Ok(())
    }

    fn generate_detailed_report(&self, files: &[AudioFile], path: &Path) -> Result<()> {
        let mut writer = Writer::from_path(path)?;
        
        // Write header
        writer.write_record(&[
            "File Name",
            "Bitrate (kbps)",
            "Quality Category",
            "Size (MB)",
            "Artist",
            "Title",
            "Album"
        ])?;

        // Sort files by bitrate (highest to lowest)
        let mut sorted_files: Vec<&AudioFile> = files.iter().collect();
        sorted_files.sort_by(|a, b| {
            b.bitrate.unwrap_or(0).cmp(&a.bitrate.unwrap_or(0))
        });

        // Write file details
        for file in sorted_files {
            if let Some(bitrate) = file.bitrate {
                let category = BitrateCategory::from_bitrate(bitrate);
                let size_mb = file.size_bytes as f64 / 1_048_576.0; // Convert bytes to MB
                
                writer.write_record(&[
                    &file.file_name,
                    &bitrate.to_string(),
                    category.as_str(),
                    &format!("{:.2}", size_mb),
                    file.artist.as_deref().unwrap_or("Unknown"),
                    file.title.as_deref().unwrap_or("Unknown"),
                    file.album.as_deref().unwrap_or("Unknown"),
                ])?;
            }
        }

        writer.flush()?;
        println!("Detailed report generated: {}", path.display());
        Ok(())
    }

    pub fn generate_duplicate_report(&self, results: &DuplicateResults, output_path: impl AsRef<Path>) -> Result<()> {
        let output_path_ref = output_path.as_ref();
        let mut writer = Writer::from_path(output_path_ref)?;
        
        writer.write_record(&[
            "Higher Quality File",
            "Higher Quality Size (MB)",
            "Higher Quality Bitrate",
            "Lower Quality File",
            "Lower Quality Size (MB)",
            "Lower Quality Bitrate",
            "Match Reason",
            "Quality Difference"
        ])?;

        for dup_match in &results.matches {
            let higher_size_mb = dup_match.higher_quality.size_bytes as f64 / 1_048_576.0;
            let lower_size_mb = dup_match.lower_quality.size_bytes as f64 / 1_048_576.0;

            writer.write_record(&[
                &dup_match.higher_quality.file_name,
                &format!("{:.2}", higher_size_mb),
                &dup_match.higher_quality.bitrate.map_or("Unknown".to_string(), |b| format!("{} kbps", b)),
                &dup_match.lower_quality.file_name,
                &format!("{:.2}", lower_size_mb),
                &dup_match.lower_quality.bitrate.map_or("Unknown".to_string(), |b| format!("{} kbps", b)),
                &dup_match.match_reason,
                &dup_match.quality_difference,
            ])?;
        }

        writer.flush()?;
        println!("Duplicate report generated: {}", output_path_ref.display());
        Ok(())
    }
}