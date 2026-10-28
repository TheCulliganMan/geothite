use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_voyageurs_ice_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_voyageurs_ice BASE SOURCE_JSON OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[2].canonicalize().unwrap_or_else(|_| args[2].clone()),
        "output must differ from input"
    );
    let pack = build_voyageurs_ice_modpack(
        &read_verified_compiled_game_pack(&args[0])?,
        &std::fs::read_to_string(&args[1])?,
    )?;
    pack.write_preserving_storage(&args[2])?;
    let checked = read_verified_compiled_game_pack(&args[2])?;
    ensure!(
        checked.identity()? == pack.identity()?,
        "strict ice pack readback"
    );
    println!(
        "published Voyageurs Ice Cavern: {} maps in {}",
        checked.data().maps.len(),
        args[2].display()
    );
    Ok(())
}
