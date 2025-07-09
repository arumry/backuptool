use crate::common::*;
use std::fs;

#[test]
fn test_basic_snapshot() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success(), "Snapshot command failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_snapshot_with_binary_files() {
    let env = TestEnvironment::new();
    
    let binary_content = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    create_binary_file(&env.test_data_dir, "binary.dat", &binary_content).unwrap();
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
}

#[test]
fn test_snapshot_deduplication() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("file1.txt"), "Same content").unwrap();
    fs::write(env.test_data_dir.join("file2.txt"), "Same content").unwrap();
    fs::write(env.test_data_dir.join("file3.txt"), "Different content").unwrap();
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
}

#[test]
fn test_snapshot_unchanged_directory() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    let output1 = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output1.status.success());
    
    let output2 = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output2.status.success(), "Second snapshot of unchanged directory failed");
}

#[test]
fn test_snapshot_incremental() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    let output1 = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output1.status.success());
    
    fs::write(env.test_data_dir.join("file1.txt"), "Modified content").unwrap();
    fs::write(env.test_data_dir.join("new_file.txt"), "New content").unwrap();
    
    let output2 = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output2.status.success());
}

#[test]
fn test_snapshot_with_absolute_path() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    let absolute_path = env.test_data_dir.canonicalize().unwrap();
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", absolute_path.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success(), "Snapshot with absolute path failed");
}

#[test]
fn test_snapshot_with_relative_path() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    let current_dir = std::env::current_dir().unwrap();
    let relative_path = env.test_data_dir.strip_prefix(&current_dir)
        .unwrap_or(&env.test_data_dir);
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", relative_path.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success(), "Snapshot with relative path failed");
}

#[test]
fn test_snapshot_empty_directory() {
    let env = TestEnvironment::new();
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success(), "Snapshot of empty directory failed");
}

#[test]
fn test_snapshot_deeply_nested_structure() {
    let env = TestEnvironment::new();
    
    let deep_path = env.test_data_dir.join("a/b/c/d/e/f");
    fs::create_dir_all(&deep_path).unwrap();
    fs::write(deep_path.join("deep_file.txt"), "Deep content").unwrap();
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
}