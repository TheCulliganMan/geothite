//! Repair reviewed stray Minnesota sprite pixels in a fresh external pack.
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    ensure!(
        args.len() == 2,
        "usage: pack_minnesota_sprite_edges BASE_PACK FRESH_OUTPUT"
    );
    ensure!(!args[1].exists(), "output must be fresh");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index", "--"])
            .arg(&args[1])
            .status()?
            .success(),
        "output must be ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let fixed = crystal_assets::repair_minnesota_sprite_edges(&base)?;
    ensure!(
        crystal_assets::repair_minnesota_sprite_edges(&fixed)? == fixed,
        "repair is not idempotent"
    );
    fixed.write_preserving_storage(&args[1])?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(&args[1])? == fixed,
        "pack readback differs"
    );
    println!("{} {}", args[1].display(), fixed.identity()?.content_hash);
    Ok(())
}
