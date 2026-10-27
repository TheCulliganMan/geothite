use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, refresh_installed_minnesota_gyms};
use std::path::PathBuf;

fn main() -> Result<()> {
    let p = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        p.len() == 4,
        "usage: pack_gym_refresh STORY_BASE REVIEWED_GYMS SOURCE OUTPUT"
    );
    ensure!(
        p[0].canonicalize()? != p[3].canonicalize().unwrap_or_else(|_| p[3].clone()),
        "output must differ from base"
    );
    let pack = refresh_installed_minnesota_gyms(
        &read_verified_compiled_game_pack(&p[0])?,
        &read_verified_compiled_game_pack(&p[1])?,
        &std::fs::read_to_string(&p[2])?,
    )?;
    pack.write_preserving_storage(&p[3])?;
    let checked =
        read_verified_compiled_game_pack(&p[3]).context("written refreshed story pack")?;
    println!(
        "published current eight-gym story pack: {} maps in {}",
        checked.data().maps.len(),
        p[3].display()
    );
    Ok(())
}
