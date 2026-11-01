use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_mankato_terraces};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_mankato_terraces BASE HISTORICAL CANDIDATE OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[3].canonicalize().unwrap_or_else(|_| paths[3].clone()),
        "output must differ from input"
    );
    let pack = revise_mankato_terraces(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
        &std::fs::read_to_string(&paths[2])?,
    )?;
    pack.write_preserving_storage(&paths[3])?;
    let checked =
        read_verified_compiled_game_pack(&paths[3]).context("written terrace revision")?;
    println!(
        "published Mankato terraces: {} maps; {}",
        checked.data().maps.len(),
        checked.identity().context("identity")?.content_hash
    );
    Ok(())
}
