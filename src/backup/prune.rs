use anyhow::{Result, bail};

use crate::storage::Database;

pub struct Prune {
    db: Database,
}

impl Prune {
    pub fn new(db: Database) -> Self {
        Prune { db }
    }

    pub fn prune_snapshot(&self, snapshot_id: u32) -> Result<()> {
        // Check if snapshot exists
        if !self.db.snapshot_exists(snapshot_id)? {
            bail!("Snapshot {} does not exist", snapshot_id);
        }

        println!("Pruning snapshot {}", snapshot_id);

        // Delete the snapshot
        self.db.delete_snapshot(snapshot_id)?;

        // Clean up orphaned content
        self.db.cleanup_orphaned_content()?;

        println!("Snapshot {} pruned successfully", snapshot_id);
        println!("Orphaned content cleaned up");

        Ok(())
    }
}