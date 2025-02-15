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
        /// Directory to scan for duplicates
        #[arg(short = 'i', long = "input")]
        input: PathBuf,

        /// Directory to move duplicates to
        #[arg(short = 'o', long = "output")]
        output: PathBuf,

        /// Only detect duplicates without moving files
        #[arg(short = 'd', long)]
        dry_run: bool,

        /// Recursively scan subdirectories
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,
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