use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_leech_shore_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 6,
        "usage: pack_leech_shore BASE TOWN LAYOUT R07 R08 OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[5].canonicalize().unwrap_or_else(|_| p[5].clone()),
        "output must differ from base"
    );
    let result = build_leech_shore_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
        &std::fs::read_to_string(&p[3])?,
        &std::fs::read_to_string(&p[4])?,
    )?;
    result.write_preserving_storage(&p[5])?;
    let checked = read_verified_compiled_game_pack(&p[5]).context("written Leech Lake shore")?;
    println!(
        "published Leech Lake shore and land roads: {} maps in {}",
        checked.data().maps.len(),
        p[5].display()
    );
    Ok(())
}
