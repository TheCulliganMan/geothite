//! Publish the reviewed two-cell cold-roof canopy correction to a fresh pack.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_winter_roof_fidelity_pack, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: build_winter_roof_fidelity BASE CONTRACT NEW_OUTPUT"
    );
    ensure!(!paths[2].exists(), "winter roof output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?))
            == "aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a",
        "exact published northern-winter parent file required"
    );
    let contract = std::fs::read_to_string(&paths[1]).context("roof contract")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let output = build_winter_roof_fidelity_pack(&base, &contract)?;
    output.write_preserving_storage(&paths[2])?;
    let written =
        read_verified_compiled_game_pack(&paths[2]).context("strict written roof pack readback")?;
    ensure!(
        written == output,
        "written roof pack differs from validated output"
    );
    println!(
        "Published two cold roof cells: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
