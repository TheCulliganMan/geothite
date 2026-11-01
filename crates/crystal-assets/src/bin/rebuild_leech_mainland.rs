//! Publish a guarded mainland revision to fresh ignored output.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(args.next().context("PACK SHA256 SOURCE CONTRACT OUTPUT")?);
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let source = std::fs::read_to_string(args.next().context("mainland source")?)?;
    let contract = std::fs::read_to_string(args.next().context("native town contract")?)?;
    let binding: serde_json::Value = serde_json::from_str(&contract)?;
    ensure!(
        binding["parent_pack_sha256"] == expected,
        "contract must bind the supplied publication parent"
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
    let pack = crystal_assets::rebuild_leech_lake_mainland(&base, &source, &contract)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent changed during publication"
    );
    pack.write_preserving_storage(&output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&output)? == pack,
        "strict native mainland readback differs"
    );
    println!("{}", output.display());
    Ok(())
}
