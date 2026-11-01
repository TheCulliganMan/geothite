use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_prairie_story_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 4,
        "usage: pack_prairie_story BASE CHAPTER STORY OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[3].canonicalize().unwrap_or_else(|_| p[3].clone()),
        "output must differ from base"
    );
    let pack = build_prairie_story_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
    )?;
    pack.write_preserving_storage(&p[3])?;
    let checked = read_verified_compiled_game_pack(&p[3]).context("written prairie readback")?;
    println!(
        "published Prairie Road and Mankato notebook: {} maps in {}",
        checked.data().maps.len(),
        p[3].display()
    );
    Ok(())
}
