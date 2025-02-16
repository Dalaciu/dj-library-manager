use std::path::{Path, PathBuf};
use std::fs;
use crate::Result;

pub struct FileManager {
    duplicate_dir: PathBuf,
}

impl FileManager {
    pub fn new(duplicate_dir: impl Into<PathBuf>) -> Self {
        Self {
            duplicate_dir: duplicate_dir.into(),
        }
    }

    pub fn move_duplicate(&self, file_path: impl AsRef<Path>) -> Result<PathBuf> {
        let file_path = file_path.as_ref();
        let file_name = file_path.file_name()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid file path"
            ))?;

        // Create duplicate directory if it doesn't exist
        fs::create_dir_all(&self.duplicate_dir)?;

        let destination = self.duplicate_dir.join(file_name);
        
        // Handle case where file already exists in destination
        let final_destination = if destination.exists() {
            let mut counter = 1;
            let file_stem = file_name.to_str().unwrap_or("duplicate");
            let extension = file_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
                
            loop {
                let new_name = format!("{}_{}_{}.{}", file_stem, "duplicate", counter, extension);
                let new_path = self.duplicate_dir.join(new_name);
                if !new_path.exists() {
                    break new_path;
                }
                counter += 1;
            }
        } else {
            destination
        };

        // Try to move the file first
        match fs::rename(file_path, &final_destination) {
            Ok(_) => Ok(final_destination),
            Err(e) => {
                // Check if error is about cross-device move
                if e.to_string().contains("cannot move") || 
                   e.to_string().contains("different disk drive") {
                    // Fall back to copy + delete
                    fs::copy(file_path, &final_destination)?;
                    fs::remove_file(file_path)?;
                    Ok(final_destination)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    pub fn ensure_directory(&self, path: impl AsRef<Path>) -> Result<()> {
        fs::create_dir_all(path.as_ref())?;
        Ok(())
    }
}