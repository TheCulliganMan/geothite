//! Publish private native winter art without changing any map terrain.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{build_winter_outdoor_art_pack, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: build_winter_outdoor_art BASE SPEC NEW_OUTPUT"
    );
    ensure!(
        !paths[2].exists(),
        "winter art requires a fresh output path"
    );
    let specification =
        std::fs::read_to_string(&paths[1]).context("authored winter art specification")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let output = build_winter_outdoor_art_pack(&base, &specification)?;
    output.write_preserving_storage(&paths[2])?;
    let written =
        read_verified_compiled_game_pack(&paths[2]).context("strict winter art readback")?;
    ensure!(written == output, "written winter art content differs");
    println!(
        "Published native winter art: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
