use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use anyhow::{Result, Context};

use crate::storage::Database;
use crate::utils::{hash_content, relative_path};

pub struct Snapshot {
    db: Database,
}

impl Snapshot {
    pub fn new(db: Database) -> Self {
        Snapshot { db }
    }

    pub fn create(&self, target_directory: &Path) -> Result<u32> {
        let target_dir_str = target_directory.to_string_lossy().to_string();
        let snapshot_id = self.db.create_snapshot(&target_dir_str)?;

        println!("Creating snapshot {} for directory: {}", snapshot_id, target_dir_str);

        let mut file_count = 0;
        let mut total_size = 0;
        let mut deduplicated_files = 0;

        for entry in WalkDir::new(target_directory)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let file_path = entry.path();
                let relative_path = relative_path(file_path, target_directory)?;
                let relative_path_str = relative_path.to_string_lossy().to_string();

                match self.process_file(snapshot_id, file_path, &relative_path_str) {
                    Ok(ProcessResult { size, was_deduplicated }) => {
                        file_count += 1;
                        total_size += size;
                        if was_deduplicated {
                            deduplicated_files += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to process file {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        println!("Snapshot {} created successfully", snapshot_id);
        println!("  Files processed: {}", file_count);
        println!("  Total size: {} bytes", total_size);
        println!("  Deduplicated files: {}", deduplicated_files);

        Ok(snapshot_id)
    }

    fn process_file(&self, snapshot_id: u32, file_path: &Path, relative_path: &str) -> Result<ProcessResult> {
        let content = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        let size = content.len() as u64;
        let content_hash = hash_content(&content);

        // Try to store content - this will be skipped if content already exists
        let was_deduplicated = match self.db.store_content(&content_hash, &content) {
            Ok(_) => false, // New content was stored
            Err(_) => true, // Content already existed (deduplicated)
        };

        // Always store the file reference, even if content was deduplicated
        self.db.store_content(&content_hash, &content)?;
        self.db.add_file_to_snapshot(snapshot_id, relative_path, &content_hash, size)?;

        Ok(ProcessResult { size, was_deduplicated })
    }
}

struct ProcessResult {
    size: u64,
    was_deduplicated: bool,
}