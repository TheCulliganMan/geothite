//! Replay the preserved story source against a reviewed compact parent.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_compact_interior_story_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: pack_compact_interior_stories BASE STORIES SUPPLEMENT FRESH_OUTPUT"
    );
    ensure!(
        !paths[3].exists(),
        "preserve existing packs; choose a fresh output"
    );
    let pack = build_compact_interior_story_pack(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
    )?;
    pack.write_preserving_storage(&paths[3])?;
    ensure!(
        read_verified_compiled_game_pack(&paths[3])? == pack,
        "strict written story replay differs"
    );
    println!("Published compact room stories: {}", paths[3].display());
    Ok(())
}
