use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

pub struct Discovery {
    pub files: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub input_is_file: bool,
}

/// Expand a file or directory argument into a deterministic list of candidate
/// files. Format probing happens later, per file.
pub fn discover(input: &Path, follow_links: bool) -> Result<Discovery> {
    let metadata =
        fs::metadata(input).with_context(|| format!("cannot access {}", input.display()))?;

    if metadata.is_file() {
        return Ok(Discovery {
            files: vec![input.to_path_buf()],
            warnings: Vec::new(),
            input_is_file: true,
        });
    }

    if !metadata.is_dir() {
        bail!(
            "{} is neither a regular file nor a directory",
            input.display()
        );
    }

    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for entry in WalkDir::new(input).follow_links(follow_links) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    files.push(entry.into_path());
                }
            }
            Err(error) => warnings.push(error.to_string()),
        }
    }

    files.sort();
    Ok(Discovery {
        files,
        warnings,
        input_is_file: false,
    })
}
