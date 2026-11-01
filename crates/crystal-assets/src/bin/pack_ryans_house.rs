use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_ryans_house_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 7,
        "usage: pack_ryans_house PACK OLD_LAYOUT LAYOUT OLD_DOSSIER DOSSIER HOUSE OUTPUT"
    );
    ensure!(
        args[0].canonicalize()? != args[6].canonicalize().unwrap_or_else(|_| args[6].clone()),
        "output must differ from source pack"
    );
    let sources = args[1..6]
        .iter()
        .map(std::fs::read_to_string)
        .collect::<std::io::Result<Vec<_>>>()?;
    let pack = build_ryans_house_modpack(
        &read_verified_compiled_game_pack(&args[0])?,
        &sources[0],
        &sources[1],
        &sources[2],
        &sources[3],
        &sources[4],
    )?;
    pack.write_preserving_storage(&args[6])?;
    let verified = read_verified_compiled_game_pack(&args[6]).context("read back Ryan's house")?;
    println!(
        "exported {} with {} maps",
        args[6].display(),
        verified.data().maps.len()
    );
    Ok(())
}
