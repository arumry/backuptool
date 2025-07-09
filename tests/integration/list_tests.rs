use crate::common::*;
use std::fs;

#[test]
fn test_list_empty_database() {
    let env = TestEnvironment::new();
    
    let output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SNAPSHOT"));
    assert!(stdout.contains("TIMESTAMP"));
}

#[test]
fn test_list_single_snapshot() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2025-"));
}

#[test]
fn test_list_multiple_snapshots() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    for i in 0..3 {
        fs::write(env.test_data_dir.join(format!("file_{}.txt", i)), format!("Content {}", i)).unwrap();
        
        run_backuptool(&[
            "snapshot",
            "--target-directory", env.test_data_dir.to_str().unwrap(),
            "--database", env.db_path.to_str().unwrap()
        ]);
        
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    let output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1"));
    assert!(stdout.contains("2"));
    assert!(stdout.contains("3"));
}

#[test]
fn test_list_with_size_metrics() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("file1.txt"), "Small content").unwrap();
    fs::write(env.test_data_dir.join("file2.txt"), "Another small content").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::write(env.test_data_dir.join("file3.txt"), "Additional content").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SIZE"));
    assert!(stdout.contains("DISTINCT_SIZE"));
    assert!(stdout.contains("total"));
}

#[test]
fn test_list_ordering() {
    let env = TestEnvironment::new();
    create_test_files(&env.test_data_dir).unwrap();
    
    for _ in 0..5 {
        run_backuptool(&[
            "snapshot",
            "--target-directory", env.test_data_dir.to_str().unwrap(),
            "--database", env.db_path.to_str().unwrap()
        ]);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    let output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let lines: Vec<&str> = stdout.lines().collect();
    let mut snapshot_numbers = vec![];
    
    for line in lines {
        if line.starts_with(|c: char| c.is_numeric()) {
            if let Some(num) = line.split_whitespace().next() {
                if let Ok(n) = num.parse::<u32>() {
                    snapshot_numbers.push(n);
                }
            }
        }
    }
    
    let mut sorted = snapshot_numbers.clone();
    sorted.sort();
    assert_eq!(snapshot_numbers, sorted, "Snapshots not in order");
}