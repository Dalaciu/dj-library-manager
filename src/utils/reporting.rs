use std::path::Path;
use csv::Writer;
use crate::analyzers::bitrate::{BitrateStats, BitrateCategory};
use crate::analyzers::duplicate::DuplicateGroup;
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

    pub fn generate_duplicate_report(&self, groups: &[DuplicateGroup], output_path: impl AsRef<Path>) -> Result<()> {
        let output_path_ref = output_path.as_ref();
        let mut writer = Writer::from_path(output_path_ref)?;
        
        writer.write_record(&[
            "Original File",
            "Original Size (MB)",
            "Original Bitrate",
            "Duplicate Files",
            "Duplicate Sizes (MB)",
            "Duplicate Bitrates"
        ])?;

        for group in groups {
            let duplicates = group.duplicates.iter()
                .map(|f| f.file_name.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let duplicate_sizes = group.duplicates.iter()
                .map(|f| format!("{:.2}", f.size_bytes as f64 / 1_048_576.0))
                .collect::<Vec<_>>()
                .join(", ");

            let duplicate_bitrates = group.duplicates.iter()
                .map(|f| f.bitrate.map_or("Unknown".to_string(), |b| format!("{} kbps", b)))
                .collect::<Vec<_>>()
                .join(", ");

            let original_size_mb = group.original.size_bytes as f64 / 1_048_576.0;

            writer.write_record(&[
                &group.original.file_name,
                &format!("{:.2}", original_size_mb),
                &group.original.bitrate.map_or("Unknown".to_string(), |b| format!("{} kbps", b)),
                &duplicates,
                &duplicate_sizes,
                &duplicate_bitrates,
            ])?;
        }

        writer.flush()?;
        println!("Duplicate report generated: {}", output_path_ref.display());
        Ok(())
    }
}