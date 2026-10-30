//! Publish the native League turnout at canonical outdoor connection offsets.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let parent = PathBuf::from(
        args.next()
            .context("usage: align_northstar_league_entrance PARENT EXPECTED_SHA256 OUTPUT")?,
    )
    .canonicalize()?;
    let expected = args
        .next()
        .context("expected SHA256")?
        .into_string()
        .map_err(|_| anyhow::anyhow!("SHA256 must be UTF-8"))?;
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output required"
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
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "pack must remain untracked"
    );
    let hash =
        || -> Result<String> { Ok(format!("{:x}", Sha256::digest(std::fs::read(&parent)?))) };
    ensure!(hash()? == expected, "parent SHA256 differs");
    let base = crystal_assets::read_verified_compiled_game_pack(&parent)?;
    let pack = crystal_assets::align_northstar_league_entrance(&base)?;
    ensure!(hash()? == expected, "parent changed during publication");
    pack.write_preserving_storage(&output)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        reopened.identity()? == pack.identity()?,
        "readback identity differs"
    );
    println!("Verified canonical League entrance in {}", output.display());
    Ok(())
}
