//! Publish the checked old-room to reviewed-room transition before city binding.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, replace_installed_minneapolis_gym};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: replace_minneapolis_gym BASE CANONICAL_GYM_PACK OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[2].canonicalize().unwrap_or_else(|_| paths[2].clone()),
        "output must differ from base"
    );
    ensure!(
        paths[1].canonicalize()? != paths[2].canonicalize().unwrap_or_else(|_| paths[2].clone()),
        "output must differ from canonical donor"
    );
    let result = replace_installed_minneapolis_gym(
        &read_verified_compiled_game_pack(&paths[0])?,
        &read_verified_compiled_game_pack(&paths[1])?,
    )?;
    result.write_preserving_storage(&paths[2])?;
    let verified =
        read_verified_compiled_game_pack(&paths[2]).context("strict gym replacement readback")?;
    println!(
        "replaced only the installed Gym3 in {} ({} maps)",
        paths[2].display(),
        verified.data().maps.len()
    );
    Ok(())
}
