use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_itasca_story_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 7,
        "usage: pack_itasca_story BASE TOWN LAYOUT STORY R14 R09 OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[6].canonicalize().unwrap_or_else(|_| p[6].clone()),
        "output must differ from base"
    );
    let result = build_itasca_story_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
        &std::fs::read_to_string(&p[3])?,
        &std::fs::read_to_string(&p[4])?,
        &std::fs::read_to_string(&p[5])?,
    )?;
    result.write_preserving_storage(&p[6])?;
    let checked = read_verified_compiled_game_pack(&p[6]).context("written Itasca story")?;
    println!(
        "published Itasca reading/comparison chapter: {} maps in {}",
        checked.data().maps.len(),
        p[6].display()
    );
    Ok(())
}
