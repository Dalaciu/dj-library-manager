use std::path::Path;
use csv::Writer;
use crate::analyzers::bitrate::BitrateStats;
use crate::analyzers::duplicate::DuplicateGroup;
use crate::Result;

pub struct Reporter;

impl Reporter {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_bitrate_report(&self, stats: &BitrateStats, output_path: impl AsRef<Path>) -> Result<()> {
        let output_path_ref = output_path.as_ref();
        let mut writer = Writer::from_path(output_path_ref)?;
        
        // Write header
        writer.write_record(&["Category", "File Count", "Percentage"])?;

        let total_files: usize = stats.category_distribution.values().sum();

        // Write distribution by category
        for (category, count) in &stats.category_distribution {
            let percentage = ((*count as f64 / total_files as f64) * 100.0).round();
            writer.write_record(&[
                category.to_string(),
                count.to_string(),
                format!("{:.1}%", percentage),
            ])?;
        }

        // Write summary
        writer.write_record(&["", "", ""])?;
        writer.write_record(&["Summary", "", ""])?;
        writer.write_record(&["Total Files", &stats.file_count.to_string(), ""])?;
        writer.write_record(&["Average Bitrate", &format!("{:.1} kbps", stats.average_bitrate), ""])?;
        writer.write_record(&["Min Bitrate", &format!("{} kbps", stats.min_bitrate), ""])?;
        writer.write_record(&["Max Bitrate", &format!("{} kbps", stats.max_bitrate), ""])?;

        writer.flush()?;
        println!("Report generated: {}", output_path_ref.display());
        Ok(())
    }

    pub fn generate_duplicate_report(&self, groups: &[DuplicateGroup], output_path: impl AsRef<Path>) -> Result<()> {
        let output_path_ref = output_path.as_ref();
        let mut writer = Writer::from_path(output_path_ref)?;
        
        writer.write_record(&[
            "Original File",
            "Original Size (bytes)",
            "Original Bitrate",
            "Duplicate Files",
            "Duplicate Sizes",
            "Duplicate Bitrates"
        ])?;

        for group in groups {
            let duplicates = group.duplicates.iter()
                .map(|f| f.file_name.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            let duplicate_sizes = group.duplicates.iter()
                .map(|f| f.size_bytes.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            let duplicate_bitrates = group.duplicates.iter()
                .map(|f| f.bitrate.map_or("Unknown".to_string(), |b| format!("{} kbps", b)))
                .collect::<Vec<_>>()
                .join(", ");

            writer.write_record(&[
                &group.original.file_name,
                &group.original.size_bytes.to_string(),
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