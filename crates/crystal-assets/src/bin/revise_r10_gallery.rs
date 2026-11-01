//! Publish one reviewed cave-room revision while retaining the installed world.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_r10_gallery_modpack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_r10_gallery BASE PREVIOUS_ROOM REVISED_ROOM OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[3].canonicalize().unwrap_or_else(|_| paths[3].clone()),
        "gallery output must differ from completed base"
    );
    let old = std::fs::read_to_string(&paths[1]).context("capture original room authoring")?;
    let next = std::fs::read_to_string(&paths[2]).context("capture reviewed gallery authoring")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let result = revise_r10_gallery_modpack(&base, &old, &next)?;
    result.write_preserving_storage(&paths[3])?;
    read_verified_compiled_game_pack(&paths[3]).context("strict written gallery pack")?;
    println!("Published reviewed R10 gallery to {}", paths[3].display());
    Ok(())
}
