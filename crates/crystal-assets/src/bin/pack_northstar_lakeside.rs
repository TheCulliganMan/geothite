//! Build the Minnesota Lakeside revision from the released Crossings pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 5,
        "usage: pack_northstar_lakeside RAW_CROSSINGS HOSTED_CROSSINGS POKEMON_ROOT NATIVE_ROOT FRESH_OUTPUT"
    );
    let output = &args[4];
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
    let candidate = crystal_assets::revise_northstar_lakeside(&raw, &hosted, &args[2], &args[3])?;
    candidate.write_preserving_storage(output)?;
    let readback = crystal_assets::read_verified_compiled_game_pack(output)?;
    ensure!(readback == candidate, "Lakeside readback differs");
    println!("{} {}", output.display(), readback.identity()?.content_hash);
    Ok(())
}
