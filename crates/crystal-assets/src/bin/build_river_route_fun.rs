//! Publish grassy Mississippi banks to a fresh ignored pack.
use anyhow::{Context, Result, ensure};
use std::path::PathBuf;

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: build_river_route_fun BASE REVISION NEW_OUTPUT"
    );
    ensure!(!paths[2].exists(), "river output must be fresh");
    let base = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let output = crystal_assets::build_river_route_fun(
        &base,
        &std::fs::read_to_string(&paths[1]).context("river source")?,
    )?;
    output.write_preserving_storage(&paths[2])?;
    let written = crystal_assets::read_verified_compiled_game_pack(&paths[2])?;
    ensure!(written == output, "river written readback differs");
    println!(
        "Published {}: {}",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
