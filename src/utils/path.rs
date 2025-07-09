use std::path::{Path, PathBuf};
use anyhow::Result;

pub fn relative_path(path: &Path, base: &Path) -> Result<PathBuf> {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    
    match path.strip_prefix(&base) {
        Ok(relative) => Ok(relative.to_path_buf()),
        Err(_) => Ok(path),
    }
}