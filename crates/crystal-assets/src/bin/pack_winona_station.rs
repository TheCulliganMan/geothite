use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_winona_station_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: pack_winona_station BASE TOWN STATION OUTPUT"
    );
    ensure!(
        !paths[3].exists(),
        "preserve existing packs; choose a new output path"
    );
    let pack = build_winona_station_modpack(
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
