//! Build the authored adventure revision without selecting it or touching saves.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: pack_northstar_adventure CURRENT STORY_JSON DISCOVERIES_JSON FRESH_OUTPUT"
    );
    let output = &paths[3];
    ensure!(!output.exists(), "choose a fresh candidate path");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated candidate must remain ignored"
    );
    let current = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let story = std::fs::read_to_string(&paths[1]).context("canonical story")?;
    let discoveries = std::fs::read_to_string(&paths[2]).context("hidden discoveries")?;
    let candidate = crystal_assets::improve_northstar_adventure(&current, &story, &discoveries)?;
    candidate.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(written == candidate, "adventure pack readback differs");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
