use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_gym_four_approach, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 5,
        "usage: pack_gym_four_approach BASE CONTRACT R04 STORY OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[4].canonicalize().unwrap_or_else(|_| paths[4].clone()),
        "output must differ from base"
    );
    let pack = build_gym_four_approach(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
        &std::fs::read_to_string(&paths[3])?,
    )?;
    pack.write_preserving_storage(&paths[4])?;
    let checked = read_verified_compiled_game_pack(&paths[4])?;
    println!(
        "published fourth-gym river approach: {} maps in {}",
        checked.data().maps.len(),
        paths[4].display()
    );
    Ok(())
}
