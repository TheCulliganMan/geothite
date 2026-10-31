//! Publish the bounded current-story correction as an unselected review child.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{complete_northstar_story_cast, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: complete_northstar_story_cast CURRENT DONOR FRESH_OUTPUT"
    );
    let output = paths[2]
        .parent()
        .context("output directory")?
        .canonicalize()?
        .join(paths[2].file_name().context("output filename")?);
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
    let donor = read_verified_compiled_game_pack(&paths[1])?;
    let revised = complete_northstar_story_cast(&current, &donor)?;
    revised.write_preserving_storage(&output)?;
    ensure!(
        read_verified_compiled_game_pack(&output)? == revised,
        "current story readback differs"
    );
    println!(
        "Verified {}: {} maps",
        output.display(),
        revised.data().maps.len()
    );
    Ok(())
}
