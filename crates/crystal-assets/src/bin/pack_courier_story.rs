use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_courier_story_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 5,
        "usage: pack_courier_story BASE CHAPTER STORY ROUTE OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[4].canonicalize().unwrap_or_else(|_| p[4].clone()),
        "output must differ from base"
    );
    let pack = build_courier_story_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
        &std::fs::read_to_string(&p[3])?,
    )?;
    pack.write_preserving_storage(&p[4])?;
    let checked = read_verified_compiled_game_pack(&p[4]).context("written courier story")?;
    println!(
        "published St Paul courier, manifest and R04 link: {} maps in {}",
        checked.data().maps.len(),
        p[4].display()
    );
    Ok(())
}
