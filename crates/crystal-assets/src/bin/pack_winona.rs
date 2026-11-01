use anyhow::{Context, Result, ensure};
use crystal_assets::{build_winona_opening_modpack, read_verified_compiled_game_pack};
use std::path::PathBuf;

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: pack_winona BASE TOWN LAYOUT STORY OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[4].canonicalize().unwrap_or_else(|_| paths[4].clone()),
        "output must differ from base"
    );
    let result = build_winona_opening_modpack(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
        &std::fs::read_to_string(&paths[3])?,
    )?;
    result.write_preserving_storage(&paths[4])?;
    let verified =
        read_verified_compiled_game_pack(&paths[4]).context("strict opening pack readback")?;
    println!(
        "published {} opening maps in {} ({} total)",
        6,
        paths[4].display(),
        verified.data().maps.len()
    );
    Ok(())
}
