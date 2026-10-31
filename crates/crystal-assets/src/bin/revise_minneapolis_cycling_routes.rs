//! Publish a fresh, strictly verified Minneapolis cycling revision.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(args.next().context("PACK SHA256 ROUTES OUTPUT")?).canonicalize()?;
    let expected = args
        .next()
        .context("parent SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let source = std::fs::read_to_string(args.next().context("routes JSON")?)?;
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
        "pack must remain ignored"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent SHA drift"
    );
    let parent = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let child = crystal_assets::revise_minneapolis_cycling_routes(&parent, &source)?;
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&input)?)) == expected,
        "parent changed"
    );
    child.write_preserving_storage(&output)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        reopened.identity()? == child.identity()?,
        "cycling readback mismatch"
    );
    println!("{}", output.display());
    Ok(())
}
