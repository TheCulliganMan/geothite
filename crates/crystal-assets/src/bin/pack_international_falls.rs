use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_international_falls_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: pack_international_falls BASE TOWN LAYOUT OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[3].canonicalize().unwrap_or_else(|_| paths[3].clone()),
        "output must differ from input"
    );
    ensure!(
        !paths[3].exists(),
        "preserve completed publications; choose a new output path"
    );
    let pack = build_international_falls_modpack(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
    )?;
    pack.write_preserving_storage(&paths[3])?;
    let written = read_verified_compiled_game_pack(&paths[3])?;
    println!(
        "Verified {}: {} maps, {} species",
        paths[3].display(),
        written.data().maps.len(),
        written.data().pokemon.len()
    );
    Ok(())
}
