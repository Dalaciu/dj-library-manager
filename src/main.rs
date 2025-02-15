use clap::Parser;
use bitrate_analyzer::{
    MetadataExtractor,
    analyzers::{
        bitrate::BitrateAnalyzer,
        duplicate::DuplicateAnalyzer,
    },
    utils::{
        file_ops::FileManager,
        reporting::Reporter,
    },
};
use bitrate_analyzer::cli::commands::{Cli, Commands};

fn main() {
    env_logger::init();
    
    // Configure thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get()) // Use all available CPU cores
        .build_global()
        .unwrap();

    println!("Initialized with {} threads", rayon::current_num_threads());
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Duplicates { dirs, output, threshold, dry_run } => {
            println!("=== Starting Duplicate Analysis ===");
            println!("Scanning directories for duplicates:");
            for dir in &dirs {
                println!("  - {}", dir.display());
            }
            println!("Output directory: {}", output.display());
            println!("Similarity threshold: {}", threshold);
            println!("Dry run mode: {}", dry_run);
            
            // Extract metadata from all audio files
            println!("\nScanning for audio files...");
            let files = match MetadataExtractor::process_directories(&dirs) {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Error processing directories: {}", e);
                    return;
                }
            };

            println!("\nFound {} total audio files", files.len());

            if files.is_empty() {
                println!("No audio files found to analyze.");
                return;
            }

            // Find duplicates
            println!("\nAnalyzing for duplicates...");
            let analyzer = DuplicateAnalyzer::new(threshold);
            let duplicate_groups = analyzer.find_duplicates(files);

            println!("\nFound {} groups of duplicates", duplicate_groups.len());

            if duplicate_groups.is_empty() {
                println!("No duplicates found.");
                return;
            }

            if dry_run {
                println!("\nDry run - no files will be moved");
                println!("Would move the following duplicates:");
                for group in &duplicate_groups {
                    println!("\nOriginal: {}", group.original.file_name);
                    for duplicate in &group.duplicates {
                        println!("  Would move: {}", duplicate.file_name);
                    }
                }
            } else {
                // Create output directory if it doesn't exist
                println!("\nPreparing output directory...");
                let file_manager = FileManager::new(&output);
                file_manager.ensure_directory(&output)
                    .expect("Failed to create output directory");

                // Move duplicates
                println!("Moving duplicate files...");
                for group in &duplicate_groups {
                    println!("\nProcessing group with original: {}", group.original.file_name);
                    for duplicate in &group.duplicates {
                        match file_manager.move_duplicate(&duplicate.path) {
                            Ok(new_path) => println!("  Moved: {} -> {}", 
                                duplicate.file_name, 
                                new_path.file_name().unwrap_or_default().to_string_lossy()),
                            Err(e) => eprintln!("  Error moving duplicate {}: {}", duplicate.file_name, e),
                        }
                    }
                }
            }

            // Generate report
            println!("\nGenerating report...");
            let reporter = Reporter::new();
            let report_path = output.join("duplicate_report.csv");
            match reporter.generate_duplicate_report(&duplicate_groups, &report_path) {
                Ok(_) => println!("Report saved to: {}", report_path.display()),
                Err(e) => eprintln!("Error generating report: {}", e),
            }

            println!("\n=== Duplicate Analysis Complete ===");
        }

        Commands::Bitrate { dir, output } => {
            println!("=== Starting Bitrate Analysis ===");
            println!("Analyzing bitrates in directory: {}", dir.display());
            
            // Convert single dir to Vec for consistency
            let dirs = vec![dir];
            
            // Extract metadata from all audio files
            println!("\nScanning for audio files...");
            let files = match MetadataExtractor::process_directories(&dirs) {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Error processing directory: {}", e);
                    return;
                }
            };

            println!("\nFound {} audio files", files.len());

            if files.is_empty() {
                println!("No audio files found to analyze.");
                return;
            }

            // Analyze bitrates
            println!("\nAnalyzing bitrates...");
            let analyzer = BitrateAnalyzer::new();
            let stats = analyzer.analyze(&files);

            // Generate reports
            println!("\nGenerating reports...");
            let reporter = Reporter::new();
            match reporter.generate_bitrate_report(&stats, &files, &output) {
                Ok(_) => println!("Reports generated successfully."),
                Err(e) => eprintln!("Error generating reports: {}", e),
            }

            println!("\n=== Bitrate Analysis Complete ===");
        }
    }
}