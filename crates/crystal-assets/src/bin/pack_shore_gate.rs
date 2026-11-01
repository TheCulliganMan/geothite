use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_shore_gate_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: pack_shore_gate BASE SOURCE OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "preserve existing packs; choose a new output"
    );
    let pack = build_shore_gate_modpack(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
    )?;
    pack.write_preserving_storage(&paths[2])?;
    let written = read_verified_compiled_game_pack(&paths[2])?;
    println!(
        "Verified {}: {} maps, {} species",
        paths[2].display(),
        written.data().maps.len(),
        written.data().pokemon.len()
    );
    Ok(())
}
