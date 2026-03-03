use std::path::{Path, PathBuf};
use ignore::WalkBuilder;
use std::fs;
use anyhow::{Result, Context};

pub struct FileOps;

impl FileOps {
    /// Lists all files in the given directory, respecting .gitignore and ignoring hidden files
    pub fn list_files<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(dir.as_ref())
            .hidden(true) // Ignore hidden files like .git
            .git_ignore(true) // Respect .gitignore
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        files.push(entry.into_path());
                    }
                }
                Err(err) => {
                    tracing::warn!("Error traversing directory: {}", err);
                }
            }
        }

        Ok(files)
    }

    /// Reads a specific file
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read file: {:?}", path.as_ref()))
    }

    /// Writes content to a specific file
    pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write file: {:?}", path.as_ref()))
    }
}
