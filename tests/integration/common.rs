use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub test_data_dir: std::path::PathBuf,
    pub db_path: std::path::PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let test_data_dir = temp_dir.path().join("test_data");
        let db_path = temp_dir.path().join("test.db");
        
        fs::create_dir_all(&test_data_dir).unwrap();
        
        Self {
            temp_dir,
            test_data_dir,
            db_path,
        }
    }
    
    pub fn restore_dir(&self, suffix: &str) -> std::path::PathBuf {
        self.temp_dir.path().join(format!("restored_{}", suffix))
    }
}

pub fn run_backuptool(args: &[&str]) -> std::process::Output {
    Command::new("./target/debug/backuptool")
        .args(args)
        .output()
        .expect("Failed to execute backuptool command")
}

pub fn create_test_files(dir: &Path) -> std::io::Result<()> {
    fs::write(dir.join("file1.txt"), "Hello World")?;
    fs::write(dir.join("file2.txt"), "Another file")?;
    fs::create_dir_all(dir.join("subdir"))?;
    fs::write(dir.join("subdir/file3.txt"), "Nested file")?;
    Ok(())
}

pub fn create_binary_file(dir: &Path, name: &str, content: &[u8]) -> std::io::Result<()> {
    fs::write(dir.join(name), content)
}

pub fn verify_file_content(path: &Path, expected: &str) {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
    assert_eq!(content, expected, "File content mismatch for: {}", path.display());
}

pub fn verify_binary_content(path: &Path, expected: &[u8]) {
    let content = fs::read(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", path.display()));
    assert_eq!(content, expected, "Binary content mismatch for: {}", path.display());
}

pub fn verify_file_exists(path: &Path) {
    assert!(path.exists(), "File does not exist: {}", path.display());
}

pub fn verify_file_not_exists(path: &Path) {
    assert!(!path.exists(), "File should not exist: {}", path.display());
}