//! Publish renewable Minnesota berry and apricorn trees to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: pack_minnesota_fruit_groves BASE SOURCE NEW_OUTPUT"
    );
    ensure!(!paths[2].exists(), "fruit-grove output must be fresh");
    let base = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let output =
        crystal_assets::build_minnesota_fruit_groves(&base, &std::fs::read_to_string(&paths[1])?)?;
    output.write_preserving_storage(&paths[2])?;
    let written = crystal_assets::read_verified_compiled_game_pack(&paths[2])?;
    ensure!(written == output, "written fruit pack differs");
    println!(
        "Published {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
