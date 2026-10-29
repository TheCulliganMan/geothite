//! Publish Staples' native station to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_staples_station_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: pack_staples_station BASE SOURCE FRESH_OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "preserve existing packs; choose a fresh output"
    );
    let pack = build_staples_station_pack(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
    )?;
    pack.write_preserving_storage(&paths[2])?;
    ensure!(
        read_verified_compiled_game_pack(&paths[2])? == pack,
        "strict Staples readback differs"
    );
    println!("Published native Staples station: {}", paths[2].display());
    Ok(())
}
