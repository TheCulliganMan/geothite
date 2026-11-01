//! Publish a guarded native survey cave exploration child to fresh ignored output.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("PACK SHA256 BEFORE AFTER CONTRACT TRAINERS OUTPUT")?,
    );
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let before = std::fs::read_to_string(args.next().context("before terrain")?)?;
    let after = std::fs::read_to_string(args.next().context("after terrain")?)?;
    let contract = std::fs::read_to_string(args.next().context("native room contract")?)?;
    let trainers = std::fs::read_to_string(args.next().context("survey trainers")?)?;
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and exact arguments required"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "native pack must remain ignored"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent SHA drift"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&parent)?;
    let pack =
        crystal_assets::rebuild_r10_exploration(&base, &before, &after, &contract, &trainers)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&parent)?)) == expected,
        "parent changed"
    );
    pack.write_preserving_storage(&output)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        reopened.identity()? == pack.identity()?,
        "native survey cave readback mismatch"
    );
    println!("{}", output.display());
    Ok(())
}
