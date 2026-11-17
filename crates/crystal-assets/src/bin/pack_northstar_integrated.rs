//! Assemble the reviewed gyms and current campaign writing on the selected world.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 8,
        "usage: pack_northstar_integrated CURRENT GYM_COMMON GYM_REVIEW STORY_COMMON STORY_REVIEW CONSOLIDATION_WRITING STORY_JSON FRESH_OUTPUT"
    );
    let output = &paths[7];
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
    let common = crystal_assets::read_verified_compiled_game_pack(&paths[1])?;
    let reviewed = crystal_assets::read_verified_compiled_game_pack(&paths[2])?;
    let story_common = crystal_assets::read_verified_compiled_game_pack(&paths[3])?;
    let story_review = crystal_assets::read_verified_compiled_game_pack(&paths[4])?;
    let writing = std::fs::read_to_string(&paths[5]).context("consolidation writing")?;
    let story = std::fs::read_to_string(&paths[6]).context("canonical story")?;
    let (candidate, deferred) = crystal_assets::integrate_northstar_campaign(
        &current,
        &common,
        &reviewed,
        &story_common,
        &story_review,
        &writing,
        &story,
    )?;
    ensure!(
        deferred.is_empty(),
        "integrated candidate has unstaged canonical dialogue:\n{}",
        deferred.join("\n")
    );
    candidate.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(written == candidate, "integrated pack readback differs");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
