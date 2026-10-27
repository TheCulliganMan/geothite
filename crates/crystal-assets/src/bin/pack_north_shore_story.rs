use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_north_shore_story_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 3,
        "usage: pack_north_shore_story BASE CONNECTIONS OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[2].canonicalize().unwrap_or_else(|_| p[2].clone()),
        "output must differ from input"
    );
    let pack = build_north_shore_story_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
    )?;
    pack.write_preserving_storage(&p[2])?;
    let checked = read_verified_compiled_game_pack(&p[2])?;
    println!(
        "published North Shore story: {} maps in {}",
        checked.data().maps.len(),
        p[2].display()
    );
    Ok(())
}
