//! Publish only the two starter signatures over the completed Northwoods pack.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: pack_northwoods_starter_signatures BASE_PACK OUTPUT_PACK | --check BASE_PACK"
    );
    let check_only = args[0] == "--check";
    let source = Path::new(if check_only { &args[1] } else { &args[0] }).canonicalize()?;
    let base = crystal_assets::read_verified_compiled_game_pack(&source)?;
    let maximum = base
        .data()
        .moves
        .values()
        .map(|mov| mov.source_index)
        .max()
        .context("empty move table")?;
    println!(
        "Base: {} species, {} moves, maximum move index {maximum}",
        base.data().pokemon.len(),
        base.data().moves.len()
    );
    let installed = crystal_assets::build_northwoods_starter_signatures_modpack(&base)?;
    if check_only {
        println!(
            "Verified starter overlay in memory: 272 moves, two exclusive level 36 learnsets, eight original pose banks; no pack written."
        );
        return Ok(());
    }
    let output = PathBuf::from(&args[1]);
    ensure!(!output.exists(), "output pack must be fresh");
    let parent = output.parent().context("output parent")?;
    fs::create_dir_all(parent)?;
    let resolved = parent
        .canonicalize()?
        .join(output.file_name().context("output filename")?);
    ensure!(resolved != source, "output must differ from source");
    ensure!(
        Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&resolved)
            .status()?
            .success(),
        "output must remain ignored"
    );
    ensure!(
        !Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&resolved)
            .output()?
            .status
            .success(),
        "output must not be tracked"
    );
    installed.write_preserving_storage(&resolved)?;
    let reopened = crystal_assets::read_verified_compiled_game_pack(&resolved)?;
    crystal_assets::verify_northwoods_starter_signatures_preservation(&base, &reopened)?;
    ensure!(
        reopened.identity()? == installed.identity()?,
        "written starter pack differs"
    );
    println!(
        "Verified {}: {} species, {} moves; existing artwork and audio preserved",
        resolved.display(),
        reopened.data().pokemon.len(),
        reopened.data().moves.len()
    );
    Ok(())
}

#[cfg(test)]
#[path = "../northwoods_starter_signatures_tests.rs"]
mod tests;
