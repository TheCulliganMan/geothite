//! Write a local Route 11 review pack from a verified Minnesota parent.
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_r11_bluff_and_shoal_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let parent = args
        .next()
        .context("usage: pack_r11_bluff_and_shoal BASE OUTPUT")?;
    let output = args.next().context("missing local output path")?;
    ensure!(args.next().is_none(), "expected only BASE and OUTPUT");
    ensure!(!output.exists(), "local output already exists");
    ensure!(
        Command::new("git")
            .args(["check-ignore", "--quiet", "--"])
            .arg(&output)
            .status()?
            .success(),
        "local output must be ignored by the repository"
    );
    let base = read_verified_compiled_game_pack(&parent)?;
    let candidate = build_r11_bluff_and_shoal_pack(&base)?;
    candidate.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == candidate,
        "local Route 11 pack differs on readback"
    );
    println!("{}", output.display());
    Ok(())
}
