use rusqlite::{Connection, params};
use std::path::Path;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};

pub struct Database {
    conn: Connection,
}

#[derive(Debug)]
pub struct SnapshotInfo {
    pub id: u32,
    pub timestamp: DateTime<Utc>,
    pub total_size: u64,
    pub distinct_size: u64,
}

#[derive(Debug)]
pub struct FileInfo {
    pub path: String,
    pub content_hash: String,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database connection")?;
        
        let db = Database { conn };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                target_directory TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS content_blocks (
                hash TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                content BLOB NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                FOREIGN KEY (content_hash) REFERENCES content_blocks (hash)
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshot_files (
                snapshot_id INTEGER NOT NULL,
                file_id INTEGER NOT NULL,
                PRIMARY KEY (snapshot_id, file_id),
                FOREIGN KEY (snapshot_id) REFERENCES snapshots (id),
                FOREIGN KEY (file_id) REFERENCES files (id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_snapshot(&self, target_directory: &str) -> Result<u32> {
        let timestamp = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO snapshots (timestamp, target_directory) VALUES (?1, ?2)",
            params![timestamp, target_directory],
        )?;

        let snapshot_id = self.conn.last_insert_rowid() as u32;
        Ok(snapshot_id)
    }

    pub fn store_content(&self, hash: &str, content: &[u8]) -> Result<bool> {
        // Only insert if content doesn't already exist
        let exists: bool = self.conn.query_row(
            "SELECT 1 FROM content_blocks WHERE hash = ?1",
            params![hash],
            |_| Ok(true),
        ).unwrap_or(false);

        if !exists {
            self.conn.execute(
                "INSERT INTO content_blocks (hash, size, content) VALUES (?1, ?2, ?3)",
                params![hash, content.len() as i64, content],
            )?;
        }

        Ok(exists)
    }

    pub fn add_file_to_snapshot(&self, snapshot_id: u32, path: &str, content_hash: &str, size: u64) -> Result<()> {
        // Check if file already exists
        let file_id: Option<i64> = self.conn.query_row(
            "SELECT id FROM files WHERE path = ?1 AND content_hash = ?2",
            params![path, content_hash],
            |row| Ok(row.get(0)?),
        ).ok();

        let file_id = match file_id {
            Some(id) => id,
            None => {
                self.conn.execute(
                    "INSERT INTO files (path, content_hash, size) VALUES (?1, ?2, ?3)",
                    params![path, content_hash, size as i64],
                )?;
                self.conn.last_insert_rowid()
            }
        };

        // Link file to snapshot
        self.conn.execute(
            "INSERT OR IGNORE INTO snapshot_files (snapshot_id, file_id) VALUES (?1, ?2)",
            params![snapshot_id, file_id],
        )?;

        Ok(())
    }

    pub fn list_snapshots(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.timestamp,
                    COALESCE(SUM(f.size), 0) as total_size,
                    COALESCE(SUM(CASE WHEN cnt.usage_count = 1 THEN f.size ELSE 0 END), 0) as distinct_size
             FROM snapshots s
             LEFT JOIN snapshot_files sf ON s.id = sf.snapshot_id
             LEFT JOIN files f ON sf.file_id = f.id
             LEFT JOIN (
                 SELECT content_hash, COUNT(*) as usage_count
                 FROM files f2
                 JOIN snapshot_files sf2 ON f2.id = sf2.file_id
                 GROUP BY content_hash
             ) cnt ON f.content_hash = cnt.content_hash
             GROUP BY s.id, s.timestamp
             ORDER BY s.id"
        )?;

        let snapshot_iter = stmt.query_map([], |row| {
            Ok(SnapshotInfo {
                id: row.get(0)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap().with_timezone(&Utc),
                total_size: row.get::<_, i64>(2)? as u64,
                distinct_size: row.get::<_, i64>(3)? as u64,
            })
        })?;

        println!("SNAPSHOT  TIMESTAMP            SIZE  DISTINCT_SIZE");
        let mut total_db_size = 0u64;
        
        for snapshot in snapshot_iter {
            let snapshot = snapshot?;
            total_db_size += snapshot.distinct_size;
            println!("{:<8}  {:<19}  {:<4}  {}", 
                     snapshot.id, 
                     snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                     snapshot.total_size,
                     snapshot.distinct_size);
        }
        
        println!("total                          {}", total_db_size);
        Ok(())
    }

    pub fn get_snapshot_files(&self, snapshot_id: u32) -> Result<Vec<FileInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, f.content_hash
             FROM files f
             JOIN snapshot_files sf ON f.id = sf.file_id
             WHERE sf.snapshot_id = ?1"
        )?;

        let file_iter = stmt.query_map(params![snapshot_id], |row| {
            Ok(FileInfo {
                path: row.get(0)?,
                content_hash: row.get(1)?,
            })
        })?;

        let mut files = Vec::new();
        for file in file_iter {
            files.push(file?);
        }

        Ok(files)
    }

    pub fn get_file_content(&self, content_hash: &str) -> Result<Vec<u8>> {
        let content: Vec<u8> = self.conn.query_row(
            "SELECT content FROM content_blocks WHERE hash = ?1",
            params![content_hash],
            |row| Ok(row.get(0)?),
        )?;

        Ok(content)
    }

    pub fn delete_snapshot(&self, snapshot_id: u32) -> Result<()> {
        // Delete snapshot-file relationships
        self.conn.execute(
            "DELETE FROM snapshot_files WHERE snapshot_id = ?1",
            params![snapshot_id],
        )?;

        // Delete the snapshot
        self.conn.execute(
            "DELETE FROM snapshots WHERE id = ?1",
            params![snapshot_id],
        )?;

        Ok(())
    }

    pub fn cleanup_orphaned_content(&self) -> Result<()> {
        // Delete files that are no longer referenced by any snapshot
        self.conn.execute(
            "DELETE FROM files WHERE id NOT IN (
                SELECT DISTINCT file_id FROM snapshot_files
            )",
            [],
        )?;

        // Delete content blocks that are no longer referenced by any file
        self.conn.execute(
            "DELETE FROM content_blocks WHERE hash NOT IN (
                SELECT DISTINCT content_hash FROM files
            )",
            [],
        )?;

        Ok(())
    }

    pub fn snapshot_exists(&self, snapshot_id: u32) -> Result<bool> {
        let exists: bool = self.conn.query_row(
            "SELECT 1 FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            |_| Ok(true),
        ).unwrap_or(false);

        Ok(exists)
    }
}