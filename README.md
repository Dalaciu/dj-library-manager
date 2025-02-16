# DJ Library Manager

This project was driven by my own need. If you like to keep your music library clean, you might want to give it a try. A high-performance DJ music library management tool written in Rust that helps you organize, analyze and deduplicate your music collection. Perfect for DJs managing large music collections across different formats and versions.

## Features

### 1. Duplicate Detection

- Intelligent duplicate detection specifically designed for DJ music libraries
- Smart version detection for DJ-specific formats (Radio Edits, Club Mixes, Extended Versions)
- Quality-aware selection (keeps the highest quality version)
- Handles artist collaborations and DJ aliases
- Supports FLAC, MP3, and WAV formats

### 2. Bitrate Analysis

- Complete bitrate analysis of your DJ library
- Categorization into quality tiers (High-Res, Lossless, High, Standard, Low)
- Detailed CSV reports with comprehensive file information
- Format-aware analysis (special handling for FLAC vs MP3)

## Installation

```bash
# Build from source
git clone https://github.com/yourusername/dj-library-manager
cd dj-library-manager
cargo build --release
```

## Usage

### Duplicate Detection

```bash
dj-library-manager duplicates --input <INPUT_DIR> --output <OUTPUT_DIR> [--dry-run]

Options:
  -i, --input   Directory to scan for duplicates
  -o, --output  Directory to move duplicates to
  -d, --dry-run Only detect duplicates without moving files
```

### Bitrate Analysis

```bash
dj-library-manager bitrate --input <INPUT_DIR> --output <OUTPUT_FILE>

Options:
  -i, --input   Directory to scan for audio files
  -o, --output  Output CSV file path
```

## How It Works

### Duplicate Detection Algorithm

1. **DJ-Specific Title Parsing**

   - Splits filenames into artist, title, and version components
   - Handles various artist collaboration formats (feat., ft., x)
   - Normalizes and sorts artist names for consistent matching
   - Special handling for DJ aliases and remixer names

2. **Version Detection**

   - Identifies common DJ formats (Club Mix, Radio Edit, Extended Mix)
   - Smart handling of remixer names and DJ edits
   - Version comparison considering DJ-specific patterns
   - Special handling for remastered versions and special editions

3. **Quality Comparison**
   - Prioritizes lossless formats (FLAC) for highest quality playback
   - Compares bitrates for same-format files
   - Falls back to file size comparison when needed

### Bitrate Analysis

1. **Quality Categories**

   - High-Resolution: 1500+ kbps (Perfect for large venue sound systems)
   - Lossless: 700-1499 kbps (Ideal for professional DJ use)
   - High Quality: 256-400 kbps (Suitable for most DJ setups)
   - Standard Quality: 160-255 kbps (Acceptable for casual use)
   - Low Quality: 64-159 kbps (Not recommended for professional use)

2. **Analysis Process**
   - Extracts audio metadata using symphonia
   - Calculates accurate bitrates
   - Generates detailed CSV reports

## Performance

- Multi-threaded processing for handling large DJ libraries
- Efficient parallel file scanning
- Smart progress tracking
- Memory-efficient processing of extensive collections

## Example Output

### Duplicate Detection

```
Found duplicate:
  Higher quality: track.flac (1411 kbps)
  Would move: track.mp3 (320 kbps)
  Reason: Exact title match: 'artist - track (club mix)'
  Quality difference: Bitrate difference: 1411 vs 320 kbps
```

### Bitrate Analysis

```
Bitrate Analysis Summary:
Total files: 1000
Average bitrate: 320.5 kbps
Min bitrate: 128 kbps
Max bitrate: 1411 kbps

Bitrate Distribution:
High-Resolution: 100 files (10.0%)
Lossless: 200 files (20.0%)
High Quality: 500 files (50.0%)
Standard Quality: 150 files (15.0%)
Low Quality: 50 files (5.0%)
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.

## Acknowledgments

- [rayon](https://github.com/rayon-rs/rayon) for parallel processing
- [symphonia](https://github.com/pdeljanov/Symphonia) for audio metadata extraction
- [clap](https://github.com/clap-rs/clap) for CLI argument parsing
