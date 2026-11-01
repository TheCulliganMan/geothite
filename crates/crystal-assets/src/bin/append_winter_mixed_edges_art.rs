//! Append reviewed mixed winter edges to existing private winter art.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{append_winter_mixed_edges_art, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: append_winter_mixed_edges_art BASE SPEC NEW_OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "winter mixed-edge art requires a fresh output path"
    );
    let source = std::fs::read_to_string(&paths[1]).context("mixed winter edge specification")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let output = append_winter_mixed_edges_art(&base, &source)?;
    output.write_preserving_storage(&paths[2])?;
    let written =
        read_verified_compiled_game_pack(&paths[2]).context("strict mixed winter edge readback")?;
    ensure!(written == output, "written mixed winter edge pack differs");
    println!(
        "Published mixed winter edges: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
