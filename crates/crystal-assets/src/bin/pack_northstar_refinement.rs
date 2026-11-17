//! Build the refinement and checked save lineage without touching player saves.

use std::path::PathBuf;

use anyhow::{Context, Result, ensure};

fn main() -> Result<()> {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        paths.len() == 8,
        "usage: pack_northstar_refinement ADVENTURE STORY_JSON POKEMON_JSON DISCOVERIES_JSON FRESH_OUTPUT ADVENTURE_HOSTED INTEGRATED_HOSTED INTEGRATED_RAW"
    );
    let output = &paths[4];
    ensure!(!output.exists(), "choose a fresh candidate path");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated candidate must remain ignored"
    );
    let current = crystal_assets::read_loaded_verified_compiled_game_pack(&paths[0])?;
    let mut sources = vec![crystal_assets::northstar_refinement_save_source(
        current.pack(),
        &current,
    )?];
    for path in &paths[5..] {
        let source = crystal_assets::read_loaded_verified_compiled_game_pack(path)?;
        sources.push(crystal_assets::northstar_refinement_save_source(
            current.pack(),
            &source,
        )?);
    }
    let story = std::fs::read_to_string(&paths[1]).context("canonical story")?;
    let pokemon = std::fs::read_to_string(&paths[2]).context("canonical parties")?;
    let discoveries = std::fs::read_to_string(&paths[3]).context("discovery refinements")?;
    let candidate = crystal_assets::refine_northstar_adventure(
        current.pack(),
        &story,
        &pokemon,
        &discoveries,
        sources,
    )?;
    candidate.write_preserving_storage(output)?;
    let written = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(written == candidate, "refinement pack readback differs");
    println!("{} {}", output.display(), written.identity()?.content_hash);
    Ok(())
}
