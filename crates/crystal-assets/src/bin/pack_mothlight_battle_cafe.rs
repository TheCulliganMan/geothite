//! Build the Mothlight Lodge from the selected northern trails pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 2,
        "usage: pack_mothlight_battle_cafe NORTHERN_TRAILS FRESH_OUTPUT"
    );
    let output = &args[1];
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
    let candidate = crystal_assets::build_mothlight_battle_cafe(&raw)?;
    candidate.write_preserving_storage(output)?;
    let readback = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(readback == candidate, "Mothlight readback differs");
    println!("{} {}", output.display(), readback.identity()?.content_hash);
    Ok(())
}
