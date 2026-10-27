//! Publish one same-extent cave terrain revision from captured source inputs.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_authored_cave_terrain};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_authored_cave BASE PREVIOUS_ROOM REVISED_ROOM OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[3].canonicalize().unwrap_or_else(|_| paths[3].clone()),
        "cave output must differ from its completed base"
    );
    let old = std::fs::read_to_string(&paths[1]).context("capture previous cave source")?;
    let new = std::fs::read_to_string(&paths[2]).context("capture revised cave source")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let result = revise_authored_cave_terrain(&base, &old, &new)?;
    result.write_preserving_storage(&paths[3])?;
    read_verified_compiled_game_pack(&paths[3]).context("strict written cave revision")?;
    println!("Revised cave terrain in {}", paths[3].display());
    Ok(())
}
