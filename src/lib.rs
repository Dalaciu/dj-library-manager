use std::path::PathBuf;
use serde::Serialize;

pub mod analyzers;
pub mod audio;
pub mod utils;
pub mod cli;

#[derive(Debug, Clone, Serialize)]
pub struct AudioFile {
    pub path: PathBuf,
    pub file_name: String,
    pub size_bytes: u64,
    pub duration_secs: Option<f64>,
    pub bitrate: Option<u32>,
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Metadata extraction error: {0}")]
    Metadata(String),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
}

pub type Result<T> = std::result::Result<T, AudioError>;

// Re-exports for convenience
pub use audio::metadata::MetadataExtractor;
pub use analyzers::duplicate::{DuplicateAnalyzer, DuplicateMatch, DuplicateResults};
pub use analyzers::bitrate::{BitrateAnalyzer, BitrateStats};