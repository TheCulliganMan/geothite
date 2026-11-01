//! Publish native Minnesota games and prize counters to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_minnesota_game_corner PACK SOURCE_JSON OUTPUT")?,
    );
    let source = std::fs::read_to_string(args.next().context("Game Corner contract")?)?;
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "three arguments and fresh output required"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let pack = crystal_assets::build_minnesota_game_corner(&base, &source)?;
    pack.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == pack.identity()?,
        "published identity differs"
    );
    println!(
        "Verified {} ({})",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
