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
        Commands::Duplicates { input, output, dry_run, recursive } => {
            println!("=== Starting Duplicate Analysis ===");
            println!("Input directory: {}", input.display());
            println!("Output directory: {}", output.display());
            println!("Dry run mode: {}", dry_run);
            println!("Recursive mode: {}", recursive);
            
            // Extract metadata from all audio files
            println!("\nScanning for audio files...");
            let files = match MetadataExtractor::process_directory(&input) {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("Error processing directory: {}", e);
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
            let analyzer = DuplicateAnalyzer::new(0.0);
            let results = analyzer.find_duplicates(files);

            println!("\nFound {} duplicate matches in {} scanned files", 
                results.matches.len(), 
                results.total_files_scanned
            );

            if results.matches.is_empty() {
                println!("No duplicates found.");
                return;
            }

            if dry_run {
                println!("\nDry run - no files will be moved");
                println!("The following actions would be taken:");
                for dup_match in &results.matches {
                    println!("\nDuplicate pair found:");
                    println!("  Will keep: {} ({} kbps)", 
                        dup_match.higher_quality.file_name,
                        dup_match.higher_quality.bitrate.unwrap_or(0));
                    println!("  Would move: {} ({} kbps)", 
                        dup_match.lower_quality.file_name,
                        dup_match.lower_quality.bitrate.unwrap_or(0));
                    println!("  Reason: {}", dup_match.match_reason);
                    println!("  Quality difference: {}", dup_match.quality_difference);
                }
            } else {
                // Create output directory if it doesn't exist
                println!("\nPreparing output directory...");
                let file_manager = FileManager::new(&output);
                file_manager.ensure_directory(&output)
                    .expect("Failed to create output directory");

                // Move duplicates
                println!("\nMoving duplicate files...");
                for dup_match in &results.matches {
                    println!("\nProcessing duplicate pair:");
                    println!("  Keeping: {} ({} kbps)", 
                        dup_match.higher_quality.file_name,
                        dup_match.higher_quality.bitrate.unwrap_or(0));
                    
                    match file_manager.move_duplicate(&dup_match.lower_quality.path) {
                        Ok(new_path) => println!("  Moved: {} ({} kbps) -> {}", 
                            dup_match.lower_quality.file_name,
                            dup_match.lower_quality.bitrate.unwrap_or(0),
                            new_path.file_name().unwrap_or_default().to_string_lossy()),
                        Err(e) => eprintln!("  Error moving file {}: {}", 
                            dup_match.lower_quality.file_name, e),
                    }
                }
            }

            // Generate report
            println!("\nGenerating report...");
            let reporter = Reporter::new();
            let report_path = output.join("duplicate_report.csv");
            match reporter.generate_duplicate_report(&results, &report_path) {
                Ok(_) => println!("Report saved to: {}", report_path.display()),
                Err(e) => eprintln!("Error generating report: {}", e),
            }

            println!("\n=== Duplicate Analysis Complete ===");
        }

        Commands::Bitrate { dir, output } => {
            // Bitrate command implementation remains unchanged
            println!("=== Starting Bitrate Analysis ===");
            println!("Analyzing bitrates in directory: {}", dir.display());
            
            let dirs = vec![dir];
            
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

            println!("\nAnalyzing bitrates...");
            let analyzer = BitrateAnalyzer::new();
            let stats = analyzer.analyze(&files);

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