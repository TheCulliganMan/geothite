//! Publish the two North Shore shops to a fresh ignored local pack.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let input = PathBuf::from(
        args.next()
            .context("usage: pack_north_shore_shops PACK SOURCE_JSON OUTPUT")?,
    )
    .canonicalize()?;
    let source = std::fs::read_to_string(args.next().context("shops source")?)?;
    let output = PathBuf::from(args.next().context("fresh ignored output")?);
    ensure!(
        args.next().is_none() && !output.exists(),
        "fresh output and three arguments required"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack output must be ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&input)?;
    let result = crystal_assets::build_north_shore_shops(&base, &source)?;
    result.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == result.identity()?,
        "written shop pack differs"
    );
    println!(
        "Verified {} ({})",
        output.display(),
        written.identity()?.content_hash
    );
    Ok(())
}
