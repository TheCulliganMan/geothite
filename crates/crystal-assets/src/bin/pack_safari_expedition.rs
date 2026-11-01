//! Publish an unselected expedition review pack with exact parent/source binding.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 5,
        "usage: pack_safari_expedition BASE BASE_SHA256 ZONE ZONE_SHA256 NEW_OUTPUT"
    );
    let base = PathBuf::from(&args[0]);
    let source = std::fs::read(PathBuf::from(&args[2]))?;
    let output = PathBuf::from(&args[4]);
    ensure!(
        !output.exists(),
        "preserve previous packs; output must be fresh"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&base)?))
            == args[1].to_str().context("parent hash")?,
        "parent SHA256 differs"
    );
    ensure!(
        format!("{:x}", Sha256::digest(&source)) == args[3].to_str().context("source hash")?,
        "zone SHA256 differs"
    );
    let pack = crystal_assets::read_verified_compiled_game_pack(&base)?
        .with_safari_zone(serde_json::from_slice(&source)?)?;
    pack.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == pack,
        "strict publication readback differs"
    );
    println!(
        "Published unselected Safari review pack: {}",
        output.display()
    );
    Ok(())
}
