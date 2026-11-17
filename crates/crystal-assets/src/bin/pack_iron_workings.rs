//! Build the Iron Range and floorplan revision from the selected external parents.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 4,
        "usage: pack_iron_workings RAW_V04 HOSTED_V04 MINNESOTA_SOURCE FRESH_OUTPUT"
    );
    let output = &args[3];
    ensure!(
        !output.try_exists()?
            && std::process::Command::new("git")
                .args(["check-ignore", "--quiet", "--no-index", "--"])
                .arg(output)
                .status()?
                .success(),
        "choose a fresh ignored pack output"
    );
    let raw = crystal_assets::read_loaded_verified_compiled_game_pack(&args[0])?;
    let hosted = crystal_assets::read_loaded_verified_compiled_game_pack(&args[1])?;
    let candidate = crystal_assets::revise_iron_workings(&raw, &hosted, &args[2])?;
    candidate.write_preserving_storage(output)?;
    let readback = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(readback == candidate, "floorplan revision readback differs");
    println!("{} {}", output.display(), readback.identity()?.content_hash);
    Ok(())
}
