//! Publish the bounded public-road revision after full story consolidation.
use std::path::PathBuf;

use anyhow::{Result, ensure};
use crystal_assets::{open_northstar_public_roads, read_verified_compiled_game_pack};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 2,
        "usage: pack_northstar_public_roads CONSOLIDATED_PACK FRESH_OUTPUT"
    );
    ensure!(
        !paths[1].exists(),
        "preserve completed candidates; choose a fresh output"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&paths[1])
            .status()?
            .success(),
        "output pack must remain ignored"
    );
    let base = read_verified_compiled_game_pack(&paths[0])?;
    let pack = open_northstar_public_roads(&base)?;
    pack.write_preserving_storage(&paths[1])?;
    ensure!(
        read_verified_compiled_game_pack(&paths[1])?.identity()? == pack.identity()?,
        "public-roads readback differs"
    );
    println!(
        "Verified {}: {}",
        paths[1].display(),
        pack.identity()?.content_hash
    );
    Ok(())
}
