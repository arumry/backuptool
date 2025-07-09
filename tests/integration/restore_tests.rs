use crate::common::*;
use std::fs;

#[test]
fn test_restore_basic() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("basic");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success(), "Restore failed: {}", String::from_utf8_lossy(&output.stderr));
    
    verify_file_content(&restore_dir.join("file1.txt"), "Hello World");
    verify_file_content(&restore_dir.join("file2.txt"), "Another file");
    verify_file_content(&restore_dir.join("subdir/file3.txt"), "Nested file");
}

#[test]
fn test_restore_binary_files() {
    let env = TestEnvironment::new();
    
    let binary_content = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
    create_binary_file(&env.test_data_dir, "binary.dat", &binary_content).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("binary");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_binary_content(&restore_dir.join("binary.dat"), &binary_content);
}

#[test]
fn test_restore_specific_snapshot() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("file.txt"), "Version 1").unwrap();
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::write(env.test_data_dir.join("file.txt"), "Version 2").unwrap();
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::write(env.test_data_dir.join("file.txt"), "Version 3").unwrap();
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir2 = env.restore_dir("v2");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "2",
        "--output-directory", restore_dir2.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_content(&restore_dir2.join("file.txt"), "Version 2");
}

#[test]
fn test_restore_deeply_nested() {
    let env = TestEnvironment::new();
    
    let deep_path = env.test_data_dir.join("a/b/c/d/e/f");
    fs::create_dir_all(&deep_path).unwrap();
    fs::write(deep_path.join("deep_file.txt"), "Deep content").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("deep");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_content(&restore_dir.join("a/b/c/d/e/f/deep_file.txt"), "Deep content");
}

#[test]
fn test_restore_directory_with_file() {
    let env = TestEnvironment::new();
    
    fs::create_dir_all(env.test_data_dir.join("test_dir")).unwrap();
    fs::write(env.test_data_dir.join("test_dir/file.txt"), "Directory content").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("dir_with_file");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_exists(&restore_dir.join("test_dir"));
    verify_file_content(&restore_dir.join("test_dir/file.txt"), "Directory content");
}

#[test]
fn test_restore_nonexistent_snapshot() {
    let env = TestEnvironment::new();
    
    let restore_dir = env.restore_dir("nonexistent");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "999",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(!output.status.success(), "Restoring nonexistent snapshot should fail");
}

#[test]
fn test_restore_files_with_special_characters() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("file with spaces.txt"), "Content").unwrap();
    fs::write(env.test_data_dir.join("file-with-dashes.txt"), "Content").unwrap();
    fs::write(env.test_data_dir.join("file_with_underscores.txt"), "Content").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("special");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_exists(&restore_dir.join("file with spaces.txt"));
    verify_file_exists(&restore_dir.join("file-with-dashes.txt"));
    verify_file_exists(&restore_dir.join("file_with_underscores.txt"));
}