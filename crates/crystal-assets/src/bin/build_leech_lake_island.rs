//! Publish a guarded native Leech Lake island to fresh ignored output.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("PACK SHA256 SOURCE CONTRACT FORCED_MOVEMENT OUTPUT")?,
    );
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let source = std::fs::read_to_string(args.next().context("island source")?)?;
    let contract = std::fs::read_to_string(args.next().context("town contract")?)?;
    let forced = std::fs::read_to_string(
        args.next()
            .context("pinned native forced-movement source")?,
    )?;
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "exact arguments and fresh output required"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "game content must remain ignored"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent SHA drift"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&parent)?;
    let pack = crystal_assets::build_leech_lake_island(&base, &source, &contract, &forced)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent changed during publication"
    );
    pack.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == pack,
        "strict native island readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
