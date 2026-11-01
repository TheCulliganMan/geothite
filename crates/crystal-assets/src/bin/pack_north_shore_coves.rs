//! Publish the reviewed warm Superior cape overlay to a fresh ignored pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    ensure!(
        args.len() == 3,
        "usage: pack_north_shore_coves BASE DELTA OUTPUT"
    );
    ensure!(!args[2].exists(), "retain all previous publications");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(&args[2])
            .status()?
            .success(),
        "coastal output must remain ignored"
    );
    ensure!(
        !std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", "--"])
            .arg(&args[2])
            .output()?
            .status
            .success(),
        "coastal output must not be tracked"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let pack =
        crystal_assets::build_north_shore_coves_pack(&base, &std::fs::read_to_string(&args[1])?)?;
    pack.write_preserving_storage(&args[2])?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&args[2])? == pack,
        "written coastal readback differs"
    );
    println!("Published native coastal capes: {}", args[2].display());
    Ok(())
}
