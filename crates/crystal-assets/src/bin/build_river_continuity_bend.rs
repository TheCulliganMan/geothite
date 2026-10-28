//! Publish the frozen city river bend to a fresh ignored content pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{build_river_continuity_bend_pack, read_verified_compiled_game_pack};
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 3,
        "usage: build_river_continuity_bend BASE CONTRACT NEW_OUTPUT"
    );
    ensure!(!paths[2].exists(), "river bend output must be fresh");
    ensure!(
        format!("{:x}", Sha256::digest(std::fs::read(&paths[0])?))
            == "92bb645535db23c78491b6982ddf5fa17b72b1f0434dfb6456a395fbf8fdbd33",
        "exact reviewed winter parent file required"
    );
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let output = build_river_continuity_bend_pack(&base, &std::fs::read_to_string(&paths[1])?)?;
    output.write_preserving_storage(&paths[2])?;
    let written = read_verified_compiled_game_pack(&paths[2])?;
    ensure!(
        written == output,
        "written city river bend differs from validated pack"
    );
    println!(
        "Published city river bend: {} ({})",
        paths[2].display(),
        written.identity()?.content_hash
    );
    Ok(())
}
