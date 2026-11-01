//! Compose approved Duluth terrain/art without replacing released guard programs.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{compose_duluth_story_terrain, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: compose_duluth_story_terrain CURRENT ACCEPTED_BASE ACCEPTED_CHILD CONTRACT FRESH_OUTPUT"
    );
    let output = paths[4]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[4].file_name().context("output filename")?);
    ensure!(
        !output.exists(),
        "preserve completed candidates; output must be fresh"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&output)
            .status()?
            .success(),
        "pack output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&output)
            .output()?
            .status
            .success(),
        "pack output must remain untracked"
    );
    let current = read_verified_compiled_game_pack(&paths[0])?;
    let before = read_verified_compiled_game_pack(&paths[1])?;
    let accepted = read_verified_compiled_game_pack(&paths[2])?;
    let revised = compose_duluth_story_terrain(
        &current,
        &before,
        &accepted,
        &std::fs::read_to_string(&paths[3])?,
    )?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "Duluth composition readback differs"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        revised.data().maps.len()
    );
    Ok(())
}
