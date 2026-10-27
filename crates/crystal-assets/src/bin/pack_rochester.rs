use anyhow::{Context, Result, ensure};
use crystal_assets::{
    build_rochester_modpack, connect_winona_rochester, read_verified_compiled_game_pack,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 7,
        "usage: pack_rochester BASE TOWN LAYOUT STORY GYM LINKS OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[6].canonicalize().unwrap_or_else(|_| p[6].clone()),
        "output must differ from base"
    );
    let pack = build_rochester_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
        &std::fs::read_to_string(&p[3])?,
        &std::fs::read_to_string(&p[4])?,
    )?;
    let pack = connect_winona_rochester(&pack, &std::fs::read_to_string(&p[5])?)?;
    pack.write_preserving_storage(&p[6])?;
    let checked = read_verified_compiled_game_pack(&p[6]).context("written Rochester readback")?;
    println!(
        "published Rochester and canonical Gym1: {} maps in {}",
        checked.data().maps.len(),
        p[6].display()
    );
    Ok(())
}
