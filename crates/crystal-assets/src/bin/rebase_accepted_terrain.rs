//! Rebase an explicit accepted terrain delta onto an ignored room composition.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, rebase_accepted_terrain};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: rebase_accepted_terrain CURRENT ACCEPTED_BASE ACCEPTED_CHILD CONTRACT FRESH_OUTPUT"
    );
    ensure!(
        !paths[4].exists(),
        "preserve completed packs; choose a fresh output"
    );
    let revised = rebase_accepted_terrain(
        &read_verified_compiled_game_pack(&paths[0])?,
        &read_verified_compiled_game_pack(&paths[1])?,
        &read_verified_compiled_game_pack(&paths[2])?,
        &std::fs::read_to_string(&paths[3])?,
    )?;
    revised.write_preserving_storage(&paths[4])?;
    ensure!(
        read_verified_compiled_game_pack(&paths[4])? == revised,
        "terrain synchronization readback differs"
    );
    println!(
        "Published accepted terrain synchronization: {}",
        paths[4].display()
    );
    Ok(())
}
