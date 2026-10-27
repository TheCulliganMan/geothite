//! Extend a supplied local species pack with the authored Northwoods move catalog.
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "usage: pack_northwoods_moves SPECIES_PACK OUTPUT_PACK"
    );
    let source = PathBuf::from(&args[0]).canonicalize()?;
    let output = PathBuf::from(&args[1]);
    let parent = output.parent().context("output parent")?;
    fs::create_dir_all(parent)?;
    let resolved = parent
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(resolved != source, "output must differ from source");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&resolved)
            .status()?
            .success(),
        "output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&resolved)
            .output()?
            .status
            .success(),
        "output must not be tracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&source)?;
    let result = crystal_assets::build_northwoods_moves_modpack(&base)?;
    result.write_preserving_storage(&resolved)?;
    let verified = crystal_assets::read_verified_compiled_game_pack(&resolved)?;
    ensure!(
        verified.identity()? == result.identity()?,
        "written Northwoods pack differs"
    );
    println!(
        "Verified {}: {} species, {} moves",
        resolved.display(),
        verified.data().pokemon.len(),
        verified.data().moves.len()
    );
    Ok(())
}
