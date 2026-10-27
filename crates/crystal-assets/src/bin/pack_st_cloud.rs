use anyhow::{Context, Result, ensure};
use crystal_assets::{build_st_cloud_modpack, read_verified_compiled_game_pack};
use std::path::PathBuf;

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 6,
        "usage: pack_st_cloud BASE TOWN LAYOUT STATION GYMS OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[5].canonicalize().unwrap_or_else(|_| p[5].clone()),
        "output must differ from base"
    );
    let pack = build_st_cloud_modpack(
        &read_verified_compiled_game_pack(&p[0])?,
        &std::fs::read_to_string(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
        &std::fs::read_to_string(&p[3])?,
        &std::fs::read_to_string(&p[4])?,
    )?;
    pack.write_preserving_storage(&p[5])?;
    let checked = read_verified_compiled_game_pack(&p[5]).context("St Cloud readback")?;
    println!(
        "published St Cloud, canonical Gym5 and reciprocal R05/R06 seams: {} maps in {}",
        checked.data().maps.len(),
        p[5].display()
    );
    Ok(())
}
