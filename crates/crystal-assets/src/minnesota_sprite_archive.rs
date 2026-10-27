//! Immutable copies of every converter input and output, alongside latest views.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn retain(path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    let directory = path.parent().context("render directory")?.join("versions");
    fs::create_dir_all(&directory)?;
    let name = path
        .file_name()
        .context("render filename")?
        .to_string_lossy();
    let archived = directory.join(format!("{:x}.{name}", Sha256::digest(bytes)));
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&archived)
    {
        Ok(mut file) => file.write_all(bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            ensure!(fs::read(&archived)? == bytes, "archive object mismatch");
        }
        Err(error) => return Err(error.into()),
    }
    Ok(archived)
}

/// Preserve both the previous and new bytes before updating a latest filename.
/// Returns the immutable filename for the new bytes. Errors leave any retained
/// versions intact, including when writing the latest filename fails.
pub fn write_render(path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    if path.try_exists()? {
        retain(path, &fs::read(path)?)?;
    }
    let archived = retain(path, bytes)?;
    fs::write(path, bytes)?;
    Ok(archived)
}
