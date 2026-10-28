//! Publish the reviewed native civic park pond, rails and shoulders.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_park_pond_route_pack, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: build_park_pond_route BASE CONTRACT NEW_OUTPUT"
    );
    ensure!(!paths[2].exists(), "park output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?))
            == "3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243",
        "exact published long-river parent required"
    );
    let contract = std::fs::read_to_string(&paths[1]).context("combined park contract")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let output = build_park_pond_route_pack(&base, &contract)?;
    output.write_preserving_storage(&paths[2])?;
    let written =
        read_verified_compiled_game_pack(&paths[2]).context("strict written park readback")?;
    ensure!(
        written == output,
        "written park differs from validated candidate"
    );
    println!(
        "Published civic park pond/bridge/shoulders: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
