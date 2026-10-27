//! Publish the bounded R09 terrain revision and exact historical split repair.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_r09_mine_terrain};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: revise_r09_mine BASE PREVIOUS_ROOM REVISED_ROOM STORY OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[4].canonicalize().unwrap_or_else(|_| paths[4].clone()),
        "mine output must differ from its completed base"
    );
    let old = std::fs::read_to_string(&paths[1]).context("capture previous mine source")?;
    let new = std::fs::read_to_string(&paths[2]).context("capture revised mine source")?;
    let story = std::fs::read_to_string(&paths[3]).context("capture Q06 story authority")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let result = revise_r09_mine_terrain(&base, &old, &new, &story)?;
    result.write_preserving_storage(&paths[4])?;
    read_verified_compiled_game_pack(&paths[4]).context("strict written Q06 mine revision")?;
    println!(
        "Revised Q06 mine terrain and certified mirrors in {}",
        paths[4].display()
    );
    Ok(())
}
