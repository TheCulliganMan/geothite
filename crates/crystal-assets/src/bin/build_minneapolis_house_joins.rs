//! Publish the reviewed native Minneapolis endcap/foundation correction.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_minneapolis_house_joins_pack, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: build_minneapolis_house_joins BASE CONTRACT NEW_OUTPUT"
    );
    ensure!(!paths[2].exists(), "house join output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?))
            == "57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62",
        "exact published first river-bend parent required"
    );
    let contract = std::fs::read_to_string(&paths[1]).context("house join contract")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let output = build_minneapolis_house_joins_pack(&base, &contract)?;
    output.write_preserving_storage(&paths[2])?;
    let written = read_verified_compiled_game_pack(&paths[2])
        .context("strict written house join readback")?;
    ensure!(
        written == output,
        "written house join pack differs from validated output"
    );
    println!(
        "Published Minneapolis native facade joins: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
