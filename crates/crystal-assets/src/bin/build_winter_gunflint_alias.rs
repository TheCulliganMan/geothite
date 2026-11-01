//! Publish the guarded private Gunflint winter art alias to a fresh pack.
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
        "usage: build_winter_gunflint_alias BASE CONTRACT EXPECTED_PARENT_FILE_SHA NEW_OUTPUT"
    );
    ensure!(!paths[3].exists(), "winter Gunflint output must be fresh");
    let expected = paths[2]
        .to_str()
        .context("actual parent hash must be UTF-8")?;
    ensure!(
        expected.len() == 64 && expected.bytes().all(|b| b.is_ascii_hexdigit()),
        "explicit actual parent SHA-256 required"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?)) == expected,
        "actual approved parent file hash differs"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let output = crystal_assets::build_winter_gunflint_alias_pack(
        &base,
        &std::fs::read_to_string(&paths[1])?,
    )?;
    output.write_preserving_storage(&paths[3])?;
    let written = crystal_assets::read_verified_compiled_game_pack(&paths[3])?;
    ensure!(
        written == output,
        "written winter Gunflint pack differs from validated output"
    );
    println!(
        "Published winter Gunflint art alias: {} ({})",
        paths[3].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
