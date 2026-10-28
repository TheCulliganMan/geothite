use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{
    read_minnesota_gym_design, read_verified_compiled_game_pack, refresh_current_minnesota_gyms,
};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: pack_current_gyms BASE SELECTION OUTPUT"
    );
    ensure!(
        paths[0].canonicalize()? != paths[2].canonicalize().unwrap_or_else(|_| paths[2].clone()),
        "output must differ from input"
    );
    let pack = refresh_current_minnesota_gyms(
        &read_verified_compiled_game_pack(&paths[0])?,
        &read_minnesota_gym_design(&paths[1])?,
    )?;
    pack.write_preserving_storage(&paths[2])?;
    let written = read_verified_compiled_game_pack(&paths[2]).context("written current gyms")?;
    println!(
        "published eight current gyms: {} maps; {}",
        written.data().maps.len(),
        written.identity().context("identity")?.content_hash
    );
    Ok(())
}
