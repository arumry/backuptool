# Backup Tool

A command-line file backup tool that creates incremental snapshots of directories, supporting deduplication and efficient storage using SQLite.

## Features

- **Incremental Backups**: Only stores changed content between snapshots
- **Deduplication**: Identical files across snapshots share storage space
- **Bit-for-bit Restore**: Restored files are identical to originals
- **SQLite Storage**: Reliable database backend with efficient querying
- **Binary File Support**: Handles any file type, including binary data
- **Cross-platform**: Runs on Unix-like systems (Linux, macOS, etc.)

## Requirements

- Rust 1.70+ (2021 edition)
- SQLite (bundled with the application)

## Building

```bash
# Clone the repository
git clone <repository-url>
cd backuptool

# Build the project
cargo build --release

# The executable will be available at ./target/release/backuptool
```

## Usage

The tool provides four main operations:

### 1. Creating Snapshots

```bash
# Create a snapshot of a directory
backuptool snapshot --target-directory ~/my_important_files

# Use a custom database location
backuptool snapshot --target-directory ~/my_important_files --database ~/backups.db
```

### 2. Listing Snapshots

```bash
# List all snapshots
backuptool list

# Use a custom database location
backuptool list --database ~/backups.db
```

Example output:
```
SNAPSHOT  TIMESTAMP            SIZE  DISTINCT_SIZE
1         2024-09-01 14:35:22  432   42
2         2024-09-02 09:10:45  401   32
3         2024-09-03 16:22:10  305   37
total                          501
```

Where:
- **SIZE**: Total size of all files in the snapshot
- **DISTINCT_SIZE**: Space used by files unique to this snapshot
- **total**: Total database size

### 3. Restoring Snapshots

```bash
# Restore a snapshot to a directory
backuptool restore --snapshot-number 42 --output-directory ./restored

# Use a custom database location
backuptool restore --snapshot-number 42 --output-directory ./restored --database ~/backups.db
```

### 4. Pruning Snapshots

```bash
# Remove a snapshot and clean up unreferenced data
backuptool prune --snapshot 42

# Use a custom database location
backuptool prune --snapshot 42 --database ~/backups.db
```

## How It Works

### Storage Strategy

- **Content-based Deduplication**: Files are identified by SHA-256 hash
- **Incremental Storage**: Only new or changed files consume additional space
- **Efficient Database Schema**: SQLite tables optimize for storage and retrieval

### Database Schema

The tool uses four main tables:

1. **snapshots**: Metadata about each snapshot
2. **content_blocks**: Actual file content, indexed by hash
3. **files**: File path and metadata information
4. **snapshot_files**: Relationships between snapshots and files

### Safety Guarantees

- **Atomic Operations**: Database transactions ensure consistency
- **No Data Loss**: Pruning only removes unreferenced content
- **Bit-for-bit Accuracy**: Restored files are identical to originals

## Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests (single-threaded recommended for file operations)
cargo test --test integration -- --test-threads=1
```

### Test Coverage

- **Unit Tests**: Core functionality (hashing, database operations)
- **Integration Tests**: Full CLI workflow testing organized by operation type

## Examples

### Basic Backup Workflow

```bash
# Create initial snapshot
backuptool snapshot --target-directory ~/documents

# List snapshots
backuptool list
# Output: SNAPSHOT  TIMESTAMP            SIZE  DISTINCT_SIZE
#         1         2024-09-01 14:35:22  1024  1024

# Modify some files and create another snapshot
backuptool snapshot --target-directory ~/documents

# List snapshots to see incremental storage
backuptool list
# Output: SNAPSHOT  TIMESTAMP            SIZE  DISTINCT_SIZE
#         1         2024-09-01 14:35:22  1024  512
#         2         2024-09-01 14:40:10  1200  688
#         total                          1200

# Restore older snapshot
backuptool restore --snapshot-number 1 --output-directory ~/documents_backup

# Clean up old snapshot
backuptool prune --snapshot 1
```

### Advanced Usage

```bash
# Multiple backup targets with different databases
backuptool snapshot --target-directory ~/photos --database ~/photos.db
backuptool snapshot --target-directory ~/code --database ~/code.db

# Restore to verify backup integrity
backuptool restore --snapshot-number 1 --output-directory ~/verification --database ~/photos.db
diff -r ~/photos ~/verification  # Should show no differences
```

## Performance Considerations

- **Memory Usage**: Files are read entirely into memory (suitable for small-to-medium files)
- **Storage Efficiency**: Deduplication reduces storage requirements significantly
- **Query Performance**: SQLite indexes optimize snapshot listing and restoration

## Limitations

- **File Size**: Designed for small-to-medium files (loaded into memory)
- **Concurrency**: Single-threaded operation (one backup at a time)
- **Metadata**: Only stores file content and paths (no permissions, timestamps, etc.)
- **Empty Directories**: Empty directories are not preserved (only directories containing files)

## Error Handling

The tool provides clear error messages for common issues:

- **Missing directories**: Clear indication when target directory doesn't exist
- **Permission errors**: Informative messages for access issues
- **Database corruption**: Robust error handling for database problems
- **Disk space**: Warnings when storage is insufficient

## License

This project is licensed under the MIT License - see the LICENSE file for details.