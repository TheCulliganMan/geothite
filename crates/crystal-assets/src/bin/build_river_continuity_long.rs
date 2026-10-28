//! Publish the frozen wooded river revision to a fresh ignored content pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: build_river_continuity_long BASE CONTRACT EXPECTED_PARENT_FILE_SHA NEW_OUTPUT"
    );
    ensure!(!paths[3].exists(), "long-river output must be fresh");
    let expected_sha = paths[2]
        .to_str()
        .context("actual parent file hash must be UTF-8")?;
    ensure!(
        expected_sha.len() == 64 && expected_sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "explicit actual parent SHA-256 required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?)) == expected_sha,
        "actual approved parent file hash differs"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let output = crystal_assets::build_river_continuity_long_pack(
        &base,
        &std::fs::read_to_string(&paths[1])?,
    )?;
    output.write_preserving_storage(&paths[3])?;
    let written = crystal_assets::read_verified_compiled_game_pack(&paths[3])?;
    ensure!(
        written == output,
        "written long-river pack differs from validated output"
    );
    println!(
        "Published long wooded rivers: {} ({})",
        paths[3].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
