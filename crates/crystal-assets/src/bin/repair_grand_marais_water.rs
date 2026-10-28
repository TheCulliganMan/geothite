//! Publish the isolated missing Grand Marais Water encounter content repair.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, repair_grand_marais_water_encounters};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: repair_grand_marais_water BASE MN_R11_SOURCE NEW_OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "Water repair requires a fresh output path"
    );
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?))
            == "58b91b1fb0a7e1a5abca317c2826ba03182e1fc46546ee8ace626448a9553846",
        "exact completed terrain parent file required"
    );
    let source =
        std::fs::read_to_string(&paths[1]).context("existing authored North Shore source")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let repaired = repair_grand_marais_water_encounters(&base, &source)?;
    repaired.write_preserving_storage(&paths[2])?;
    let written = read_verified_compiled_game_pack(&paths[2])
        .context("strict written Water repair readback")?;
    ensure!(written == repaired, "written Water repair content differs");
    println!(
        "Repaired Grand Marais Water encounter content: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
