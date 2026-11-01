//! Publish an unselected, exact-parent consolidation of the full v27 story.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{consolidate_northstar_story, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: pack_northstar_story_consolidation CURRENT COMMON V27 WRITING_JSON FRESH_OUTPUT"
    );
    let output = &paths[4];
    ensure!(
        !output.exists(),
        "preserve completed candidates; output must be fresh"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "pack output must remain ignored"
    );
    let current = read_verified_compiled_game_pack(&paths[0])?;
    let common = read_verified_compiled_game_pack(&paths[1])?;
    let donor = read_verified_compiled_game_pack(&paths[2])?;
    let writing =
        std::fs::read_to_string(&paths[3]).context("reviewed conversation replacements")?;
    let revised = consolidate_northstar_story(&current, &common, &donor, &writing)?;
    revised.write_preserving_storage(output)?;
    let written = read_verified_compiled_game_pack(output)?;
    ensure!(
        written.identity()? == revised.identity()?,
        "consolidated pack readback differs"
    );
    println!(
        "Verified {}: {} maps; {}",
        output.display(),
        written.data().maps.len(),
        written.identity()?.content_hash
    );
    Ok(())
}
