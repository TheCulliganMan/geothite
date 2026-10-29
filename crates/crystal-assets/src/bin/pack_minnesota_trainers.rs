use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_minnesota_trainers_modpack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        paths.len() == 3,
        "usage: pack_minnesota_trainers BASE SOURCE OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "preserve existing packs; choose a fresh output"
    );
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let pack = build_minnesota_trainers_modpack(&base, &std::fs::read_to_string(&paths[1])?)?;
    pack.write_preserving_storage(&paths[2])?;
    let loaded = read_verified_compiled_game_pack(&paths[2])?;
    ensure!(
        pack.identity()? == loaded.identity()?,
        "pack identity changed on write"
    );
    println!(
        "Published {} additional Minnesota trainers to {}",
        loaded.data().trainers.trainers.len() - base.data().trainers.trainers.len(),
        paths[2].display()
    );
    Ok(())
}
