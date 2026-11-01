//! Publish the reviewed Rochester fossil laboratory.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: pack_minnesota_fossil_lab PACK PLAN FRESH_OUTPUT"
    );
    let input = PathBuf::from(&args[0]).canonicalize()?;
    let output = PathBuf::from(&args[2]);
    ensure!(!output.exists(), "output must be fresh");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(input)?;
    let pack = crystal_assets::build_minnesota_fossil_lab(
        &base,
        &std::fs::read_to_string(PathBuf::from(&args[1])).context("lab plan")?,
    )?;
    pack.write_preserving_storage(&output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(&output)?;
    ensure!(
        written.identity()? == pack.identity()?,
        "lab readback differs"
    );
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
