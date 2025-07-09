use crate::common::*;
use std::fs;
use std::io::Write;

#[test]
fn test_large_number_of_files() {
    let env = TestEnvironment::new();
    
    for i in 0..100 {
        fs::write(env.test_data_dir.join(format!("file_{}.txt", i)), format!("Content {}", i)).unwrap();
    }
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    
    let restore_dir = env.restore_dir("many_files");
    let output = run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
    
    for i in 0..100 {
        verify_file_content(&restore_dir.join(format!("file_{}.txt", i)), &format!("Content {}", i));
    }
}

#[test]
fn test_zero_byte_files() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("empty.txt"), "").unwrap();
    fs::write(env.test_data_dir.join("normal.txt"), "content").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("zero_byte");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_exists(&restore_dir.join("empty.txt"));
    verify_file_content(&restore_dir.join("empty.txt"), "");
    verify_file_content(&restore_dir.join("normal.txt"), "content");
}

#[test]
fn test_unicode_filenames() {
    let env = TestEnvironment::new();
    
    let unicode_names = vec![
        "файл.txt",
        "文件.txt",
        "αρχείο.txt",
        "🎉🎊.txt",
    ];
    
    for name in &unicode_names {
        fs::write(env.test_data_dir.join(name), format!("Content of {}", name)).unwrap();
    }
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir = env.restore_dir("unicode");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    for name in &unicode_names {
        verify_file_exists(&restore_dir.join(name));
    }
}

#[test]
fn test_file_deletion_tracking() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(env.test_data_dir.join("file2.txt"), "Content 2").unwrap();
    fs::write(env.test_data_dir.join("file3.txt"), "Content 3").unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    fs::remove_file(env.test_data_dir.join("file2.txt")).unwrap();
    
    run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let restore_dir1 = env.restore_dir("with_file2");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "1",
        "--output-directory", restore_dir1.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_exists(&restore_dir1.join("file2.txt"));
    
    let restore_dir2 = env.restore_dir("without_file2");
    run_backuptool(&[
        "restore",
        "--snapshot-number", "2",
        "--output-directory", restore_dir2.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    verify_file_not_exists(&restore_dir2.join("file2.txt"));
    verify_file_exists(&restore_dir2.join("file1.txt"));
    verify_file_exists(&restore_dir2.join("file3.txt"));
}

#[test]
fn test_symlink_handling() {
    let env = TestEnvironment::new();
    
    fs::write(env.test_data_dir.join("target.txt"), "Target content").unwrap();
    
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            env.test_data_dir.join("target.txt"),
            env.test_data_dir.join("link.txt")
        ).unwrap();
    }
    
    let output = run_backuptool(&[
        "snapshot",
        "--target-directory", env.test_data_dir.to_str().unwrap(),
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    assert!(output.status.success());
}

#[test]
fn test_concurrent_modifications() {
    let env = TestEnvironment::new();
    
    let file_path = env.test_data_dir.join("changing.txt");
    let mut file = fs::File::create(&file_path).unwrap();
    
    for i in 0..10 {
        writeln!(file, "Line {}", i).unwrap();
        file.flush().unwrap();
        
        run_backuptool(&[
            "snapshot",
            "--target-directory", env.test_data_dir.to_str().unwrap(),
            "--database", env.db_path.to_str().unwrap()
        ]);
    }
    
    let output = run_backuptool(&[
        "list",
        "--database", env.db_path.to_str().unwrap()
    ]);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches('\n').count() >= 10);
}