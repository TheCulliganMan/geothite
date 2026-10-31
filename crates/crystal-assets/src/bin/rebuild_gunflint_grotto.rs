//! Publish the complete native grotto against its exact current parent.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("PACK SHA256 BEFORE AFTER CONTRACT OUTPUT")?,
    );
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let before = std::fs::read_to_string(args.next().context("original terrain")?)?;
    let after = std::fs::read_to_string(args.next().context("grotto source")?)?;
    let contract = std::fs::read_to_string(args.next().context("native contract")?)?;
    let binding: serde_json::Value = serde_json::from_str(&contract)?;
    ensure!(
        binding["parent_pack_sha256"] == expected,
        "contract publication parent differs"
    );
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
    let child = crystal_assets::rebuild_gunflint_grotto(&base, &before, &after, &contract)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent changed"
    );
    child.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == child,
        "strict grotto readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
