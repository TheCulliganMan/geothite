use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_rochester_gates};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_rochester_gates BASE HISTORICAL CANDIDATE OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[3].canonicalize().unwrap_or_else(|_| paths[3].clone()),
        "output must differ from input"
    );
    let result = revise_rochester_gates(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
    )?;
    result.write_preserving_storage(&paths[3])?;
    let checked = read_verified_compiled_game_pack(&paths[3]).context("written gate revision")?;
    println!(
        "published Rochester gates: {} maps; {}",
        checked.data().maps.len(),
        checked.identity().context("identity")?.content_hash
    );
    Ok(())
}
