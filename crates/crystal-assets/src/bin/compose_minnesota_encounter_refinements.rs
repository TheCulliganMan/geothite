//! Apply the precise reviewed habitat delta to a fresh ignored current child.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: compose_minnesota_encounter_refinements CURRENT BEFORE AFTER FRESH_OUTPUT"
    );
    ensure!(!paths[3].exists(), "output must be fresh");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&paths[3])
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let current = crystal_assets::read_verified_compiled_game_pack(&paths[0])?;
    let before = crystal_assets::read_verified_compiled_game_pack(&paths[1])?;
    let after = crystal_assets::read_verified_compiled_game_pack(&paths[2])?;
    let result =
        crystal_assets::compose_minnesota_encounter_refinements(&current, &before, &after)?;
    result.write_preserving_storage(&paths[3])?;
    ensure!(
        result == crystal_assets::read_verified_compiled_game_pack(&paths[3])?,
        "written pack differs"
    );
    println!(
        "Verified {} ({})",
        paths[3].display(),
        result.identity()?.content_hash
    );
    Ok(())
}
