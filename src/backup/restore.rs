use std::path::Path;
use std::fs;
use anyhow::{Result, Context, bail};

use crate::storage::Database;

pub struct Restore {
    db: Database,
}

impl Restore {
    pub fn new(db: Database) -> Self {
        Restore { db }
    }

    pub fn restore_snapshot(&self, snapshot_id: u32, output_directory: &Path) -> Result<()> {
        // Check if snapshot exists
        if !self.db.snapshot_exists(snapshot_id)? {
            bail!("Snapshot {} does not exist", snapshot_id);
        }

        // Create output directory if it doesn't exist
        if !output_directory.exists() {
            fs::create_dir_all(output_directory)
                .with_context(|| format!("Failed to create output directory: {}", output_directory.display()))?;
        }

        // Get all files in the snapshot
        let files = self.db.get_snapshot_files(snapshot_id)?;
        
        println!("Restoring snapshot {} to {}", snapshot_id, output_directory.display());
        println!("Files to restore: {}", files.len());

        let mut restored_count = 0;
        let mut total_size = 0;

        for file_info in files {
            match self.restore_file(&file_info.path, &file_info.content_hash, output_directory) {
                Ok(size) => {
                    restored_count += 1;
                    total_size += size;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to restore file {}: {}", file_info.path, e);
                }
            }
        }

        println!("Restore completed successfully");
        println!("  Files restored: {}", restored_count);
        println!("  Total size: {} bytes", total_size);

        Ok(())
    }

    fn restore_file(&self, relative_path: &str, content_hash: &str, output_directory: &Path) -> Result<u64> {
        let file_path = output_directory.join(relative_path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
        }

        // Get file content from database
        let content = self.db.get_file_content(content_hash)
            .with_context(|| format!("Failed to get content for hash: {}", content_hash))?;

        // Write content to file
        fs::write(&file_path, &content)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        Ok(content.len() as u64)
    }
}