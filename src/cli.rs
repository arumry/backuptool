use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

use crate::storage::Database;
use crate::backup::{Snapshot, Restore, Prune};

#[derive(Parser)]
#[command(name = "backuptool")]
#[command(about = "A command line file backup tool with incremental snapshots")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Takes a snapshot of all files in the specified directory
    Snapshot {
        /// Directory to snapshot
        #[arg(long = "target-directory")]
        target_directory: PathBuf,
        /// Optional database path (default: ./backups.db)
        #[arg(long = "database", default_value = "backups.db")]
        database: PathBuf,
    },
    /// Lists snapshots stored in the database
    List {
        /// Optional database path (default: ./backups.db)
        #[arg(long = "database", default_value = "backups.db")]
        database: PathBuf,
    },
    /// Restores directory state from a snapshot
    Restore {
        /// Snapshot number to restore
        #[arg(long = "snapshot-number")]
        snapshot_number: u32,
        /// Output directory for restored files
        #[arg(long = "output-directory")]
        output_directory: PathBuf,
        /// Optional database path (default: ./backups.db)
        #[arg(long = "database", default_value = "backups.db")]
        database: PathBuf,
    },
    /// Removes old snapshots and unreferenced data
    Prune {
        /// Snapshot number to prune
        #[arg(long = "snapshot")]
        snapshot: u32,
        /// Optional database path (default: ./backups.db)
        #[arg(long = "database", default_value = "backups.db")]
        database: PathBuf,
    },
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Commands::Snapshot { target_directory, database } => {
                let db = Database::new(&database)?;
                let snapshot = Snapshot::new(db);
                snapshot.create(&target_directory)?;
                println!("Snapshot created successfully");
            }
            Commands::List { database } => {
                let db = Database::new(&database)?;
                db.list_snapshots()?;
            }
            Commands::Restore { snapshot_number, output_directory, database } => {
                let db = Database::new(&database)?;
                let restore = Restore::new(db);
                restore.restore_snapshot(snapshot_number, &output_directory)?;
                println!("Snapshot {} restored to {}", snapshot_number, output_directory.display());
            }
            Commands::Prune { snapshot, database } => {
                let db = Database::new(&database)?;
                let prune = Prune::new(db);
                prune.prune_snapshot(snapshot)?;
                println!("Snapshot {} pruned successfully", snapshot);
            }
        }
        Ok(())
    }
}