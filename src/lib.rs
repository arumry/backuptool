pub mod cli;
pub mod storage;
pub mod backup;
pub mod utils;

pub use cli::Cli;
pub use storage::Database;
pub use backup::{Snapshot, Restore, Prune};
pub use utils::hash_content;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_hash_content_consistency() {
        let content = b"Hello, World!";
        let hash1 = hash_content(content);
        let hash2 = hash_content(content);
        
        assert_eq!(hash1, hash2, "Hash should be consistent for same content");
    }

    #[test]
    fn test_hash_content_different_content() {
        let content1 = b"Hello, World!";
        let content2 = b"Hello, World?";
        
        let hash1 = hash_content(content1);
        let hash2 = hash_content(content2);
        
        assert_ne!(hash1, hash2, "Hash should be different for different content");
    }

    #[test]
    fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = Database::new(&db_path);
        assert!(db.is_ok(), "Database creation should succeed");
    }

    #[test]
    fn test_snapshot_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();
        
        let snapshot_id = db.create_snapshot("/test/path").unwrap();
        assert_eq!(snapshot_id, 1, "First snapshot should have ID 1");
        
        let snapshot_id2 = db.create_snapshot("/test/path2").unwrap();
        assert_eq!(snapshot_id2, 2, "Second snapshot should have ID 2");
    }

    #[test]
    fn test_content_storage_and_retrieval() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).unwrap();
        
        let content = b"Hello, World!";
        let hash = "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f";
        
        // Store content
        db.store_content(hash, content).unwrap();
        
        // Retrieve content
        let retrieved = db.get_file_content(hash).unwrap();
        assert_eq!(retrieved, content, "Retrieved content should match original");
    }
}