use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;

// Find all .ksh files in the given path and its subdirectories
pub fn find_ksh_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().and_then(|s| s.to_str()) == Some("ksh")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    // sort files by path
    files.sort();

    Ok(files)
}
