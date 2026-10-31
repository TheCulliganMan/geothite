//! Publish the reviewed harbor onto an exact completed Minnesota parent.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_duluth_harbor};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: revise_duluth_harbor PACK SHA256 BEFORE AFTER OUTPUT")?,
    );
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let before = std::fs::read_to_string(args.next().context("before layout")?)?;
    let after = std::fs::read_to_string(args.next().context("after layout")?)?;
    let output = PathBuf::from(args.next().context("fresh output pack")?);
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
        "pack output must be ignored"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent SHA256 differs"
    );
    let base = read_verified_compiled_game_pack(&input)?;
    let revised = revise_duluth_harbor(&base, &before, &after)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed during publication"
    );
    revised.write_preserving_storage(&output)?;
    read_verified_compiled_game_pack(&output).context("harbor publication readback")?;
    println!("{}", output.display());
    Ok(())
}
