use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bitrate_analyzer")]
#[command(author = "Dalac")]
#[command(version = "1.0")]
#[command(about = "Audio library manager for duplicate detection and bitrate analysis", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find and manage duplicate audio files
    Duplicates {
        /// Directories to scan for duplicates (can specify multiple)
        #[arg(short = 'i', long = "dir", num_args = 1.., value_delimiter = ',')]
        dirs: Vec<PathBuf>,

        /// Directory to move duplicates to
        #[arg(short = 'o', long)]
        output: PathBuf,

        /// Similarity threshold (0.0 - 1.0, default: 0.8)
        #[arg(short = 't', long, default_value_t = 0.8)]
        threshold: f64,

        /// Only detect duplicates without moving files
        #[arg(short = 'd', long)]
        dry_run: bool,
    },

    /// Analyze audio files bitrates
    Bitrate {
        /// Directory to scan for audio files
        #[arg(short = 'i', long)]
        dir: PathBuf,

        /// Output CSV file path
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
}