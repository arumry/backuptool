use crate::common::*;
use std::fs;

#[test]
fn test_all_files_restored() {
    let env = TestEnvironment::new();
    
    let files = vec![
        "file1.txt",
        "file2.txt",
        "dir1/file3.txt",
        "dir1/dir2/file4.txt",
        "dir3/file5.txt",
    ];
    
    for file in &files {
        let path = env.test_data_dir.join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("Content of {}", file)).unwrap();
    }
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("all_files");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    for file in &files {
        let restored_path = restore_dir.join(file);
        verify_file_exists(&restored_path);
        verify_file_content(&restored_path, &format!("Content of {}", file));
    }
}

#[test]
fn test_restored_files_bit_identical() {
    let env = TestEnvironment::new();
    
    let binary_data = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0xFF, 0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8,
        0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86,
    ];
    
    fs::write(env.test_data_dir.join("binary.dat"), &binary_data).unwrap();
    fs::write(env.test_data_dir.join("text.txt"), "Exact text content\nWith newlines\n").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("bit_identical");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let original_binary = fs::read(env.test_data_dir.join("binary.dat")).unwrap();
    let restored_binary = fs::read(restore_dir.join("binary.dat")).unwrap();
    assert_eq!(original_binary, restored_binary, "Binary files not bit-identical");
    
    let original_text = fs::read(env.test_data_dir.join("text.txt")).unwrap();
    let restored_text = fs::read(restore_dir.join("text.txt")).unwrap();
    assert_eq!(original_text, restored_text, "Text files not bit-identical");
}

#[test]
fn test_prune_does_not_affect_other_snapshots() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("shared.txt"), "Shared content").unwrap();
    fs::write(env.test_data_dir.join("snap1.txt"), "Snapshot 1 only").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::remove_file(env.test_data_dir.join("snap1.txt")).unwrap();
    fs::write(env.test_data_dir.join("snap2.txt"), "Snapshot 2 only").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::remove_file(env.test_data_dir.join("snap2.txt")).unwrap();
    fs::write(env.test_data_dir.join("snap3.txt"), "Snapshot 3 only").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    run_backuptool(&[
        "prune",
        "--snapshot", "2",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir1 = env.restore_dir("after_prune_1");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir1.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    assert!(output.status.success());
    verify_file_content(&restore_dir1.join("shared.txt"), "Shared content");
    verify_file_content(&restore_dir1.join("snap1.txt"), "Snapshot 1 only");
    
    let restore_dir3 = env.restore_dir("after_prune_3");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "3",
        "--output-directory", restore_dir3.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    assert!(output.status.success());
    verify_file_content(&restore_dir3.join("shared.txt"), "Shared content");
    verify_file_content(&restore_dir3.join("snap3.txt"), "Snapshot 3 only");
}

#[test]
fn test_handles_arbitrary_binary_content() {
    let env = TestEnvironment::new();
    
    let mut binary_content = Vec::new();
    for i in 0..256 {
        binary_content.push(i as u8);
    }
    
    binary_content.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    binary_content.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
    
    fs::write(env.test_data_dir.join("all_bytes.bin"), &binary_content).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("binary_content");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_binary_content(&restore_dir.join("all_bytes.bin"), &binary_content);
}

#[test]
fn test_handles_relative_and_absolute_paths() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    let absolute_path = env.test_data_dir.canonicalize().unwrap();
    run_backuptool(&[
        "snapshot",
        "--target-directory", absolute_path.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let current_dir = std::env::current_dir().unwrap();
    if let Ok(relative_path) = env.test_data_dir.strip_prefix(&current_dir) {
        fs::write(env.test_data_dir.join("new_file.txt"), "New content").unwrap();
        
        run_backuptool(&[
            "snapshot",
            "--target-directory", relative_path.to_str().unwrap(),
            "--database", env.db_path.to_str().unwrap()
        ]);
    }
    
    let restore_dir = env.restore_dir("paths");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_exists(&restore_dir.join("file1.txt"));
}

#[test]
fn test_no_duplicate_storage_on_unchanged_snapshot() {
    let env = TestEnvironment::new();
    
    let content = "x".repeat(10000);
    fs::write(env.test_data_dir.join("large.txt"), &content).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let db_size_after_first = fs::metadata(&env.db_path).unwrap().len();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let db_size_after_second = fs::metadata(&env.db_path).unwrap().len();
    
    let size_increase = db_size_after_second - db_size_after_first;
    assert!(size_increase < 1000, "Database size increased too much ({} bytes) for unchanged snapshot", size_increase);
}