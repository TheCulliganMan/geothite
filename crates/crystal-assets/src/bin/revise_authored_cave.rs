//! Publish one current Minnesota cave terrain revision to a fresh ignored file.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_assets::{read_verified_compiled_game_pack, revise_authored_cave_terrain};

fn main() -> Result<()> {
    let paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        paths.len() == 4,
        "usage: revise_authored_cave BASE PREVIOUS_ROOM REVISED_ROOM OUTPUT"
    );
    ensure!(
        !paths[3].exists(),
        "retain previous cave publications; choose a fresh output"
    );
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&paths[3])
            .status()?
            .success(),
        "cave output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&paths[3])
            .output()?
            .status
            .success(),
        "cave output must not be tracked"
    );
    let old = std::fs::read_to_string(&paths[1]).context("capture previous cave source")?;
    let new = std::fs::read_to_string(&paths[2]).context("capture revised cave source")?;
    let base = read_verified_compiled_game_pack(&paths[0])?;
    crystal_assets::validate_compact_superior_geography(&base)?;
    let result = revise_authored_cave_terrain(&base, &old, &new)?;
    crystal_assets::validate_compact_superior_geography(&result)?;
    result.write_preserving_storage(&paths[3])?;
    let written =
        read_verified_compiled_game_pack(&paths[3]).context("strict written cave revision")?;
    crystal_assets::validate_compact_superior_geography(&written)?;
    ensure!(written == result, "strict written cave readback differs");
    println!("Revised cave terrain in {}", paths[3].display());
    Ok(())
}
