//! Publish the native Split Rock clinic to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_split_rock_clinic_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: pack_split_rock_clinic BASE SOURCE FRESH_OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "preserve existing packs; choose a fresh output"
    );
    let pack = build_split_rock_clinic_pack(
        &read_verified_compiled_game_pack(&paths[0])?,
        &std::fs::read_to_string(&paths[1])?,
    )?;
    pack.write_preserving_storage(&paths[2])?;
    ensure!(
        read_verified_compiled_game_pack(&paths[2])? == pack,
        "strict clinic readback differs"
    );
    println!("Published native Split Rock clinic: {}", paths[2].display());
    Ok(())
}
