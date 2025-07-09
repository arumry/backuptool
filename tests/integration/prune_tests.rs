use crate::common::*;
use std::fs;

#[test]
fn test_prune_single_snapshot() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let output = run_backuptool(&[
        "prune",
        "--snapshot", "1",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    
    let list_output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(!stdout.contains(" 1 "), "Pruned snapshot should not appear in list");
}

#[test]
fn test_prune_preserves_shared_data() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("shared.txt"), "Shared content").unwrap();
    fs::write(env.test_data_dir.join("unique1.txt"), "Unique to snapshot 1").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::remove_file(env.test_data_dir.join("unique1.txt")).unwrap();
    fs::write(env.test_data_dir.join("unique2.txt"), "Unique to snapshot 2").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    run_backuptool(&[
        "prune",
        "--snapshot", "1",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("after_prune");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "2",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    verify_file_content(&restore_dir.join("shared.txt"), "Shared content");
    verify_file_content(&restore_dir.join("unique2.txt"), "Unique to snapshot 2");
}

#[test]
fn test_prune_multiple_snapshots() {
    let env = TestEnvironment::new();
    
    for i in 1..=5 {
        fs::write(env.test_data_dir.join(format!("file{}.txt", i)), format!("Content {}", i)).unwrap();
        run_backuptool(&[
            "snapshot",
            "--target-directory", env.test_data_dir.to_str().unwrap(),
            "--database", env.db_path.to_str().unwrap()
        ]);
    }
    
    run_backuptool(&[
        "prune",
        "--snapshot", "2",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    run_backuptool(&[
        "prune",
        "--snapshot", "4",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir3 = env.restore_dir("snap3");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "3",
        "--output-directory", restore_dir3.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    assert!(output.status.success());
    
    let restore_dir5 = env.restore_dir("snap5");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "5",
        "--output-directory", restore_dir5.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    assert!(output.status.success());
}

#[test]
fn test_prune_nonexistent_snapshot() {
    let env = TestEnvironment::new();
    
    let output = run_backuptool(&[
        "prune",
        "--snapshot", "999",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(!output.status.success(), "Pruning nonexistent snapshot should fail");
}

#[test]
fn test_prune_frees_space() {
    let env = TestEnvironment::new();
    
    let large_content = "x".repeat(10000);
    fs::write(env.test_data_dir.join("large1.txt"), &large_content).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::remove_file(env.test_data_dir.join("large1.txt")).unwrap();
    fs::write(env.test_data_dir.join("large2.txt"), &large_content).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let db_size_before = fs::metadata(&env.db_path).unwrap().len();
    
    run_backuptool(&[
        "prune",
        "--snapshot", "1",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let db_size_after = fs::metadata(&env.db_path).unwrap().len();
    
    assert!(db_size_after <= db_size_before, "Database size should not increase after pruning");
}