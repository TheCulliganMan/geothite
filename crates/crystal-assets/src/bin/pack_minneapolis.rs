use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_minneapolis_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 5,
        "usage: pack_minneapolis PACK TOWN LAYOUT STORY OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[4].canonicalize().unwrap_or_else(|_| args[4].clone()),
        "output must differ from input"
    );
    let town = std::fs::read_to_string(&args[1])?;
    let layout = std::fs::read_to_string(&args[2])?;
    let story = std::fs::read_to_string(&args[3])?;
    let pack = build_minneapolis_modpack(
        &read_verified_compiled_game_pack(&args[0])?,
        &town,
        &layout,
        &story,
    )?;
    pack.write_preserving_storage(&args[4])?;
    let verified =
        read_verified_compiled_game_pack(&args[4]).context("strict native Minneapolis readback")?;
    println!(
        "exported {} with {} maps",
        args[4].display(),
        verified.data().maps.len()
    );
    Ok(())
}
