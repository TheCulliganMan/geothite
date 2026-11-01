//! Reproduce the web server's hosted overlays for an external review pack.
use std::path::Path;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: materialize_northstar_hosted BASE FRESH_OUTPUT"
    );
    let output = Path::new(&args[1]);
    ensure!(!output.exists(), "choose a fresh output");
    ensure!(
        std::process::Command::new("git")
            .args(["check-ignore", "--quiet", "--no-index"])
            .arg(output)
            .status()?
            .success(),
        "generated pack must remain ignored"
    );
    let base = crystal_assets::read_verified_compiled_game_pack(&args[0])?;
    let all = crystal_assets::build_all_pokemon_modpack(&base)?;
    let battles = crystal_assets::build_default_battle_modpack(&all)?;
    let clock = crystal_assets::build_realtime_clock_modpack(&battles)?;
    let hosted = crystal_assets::build_player_customization_modpack(&clock)?;
    hosted.write_preserving_storage(output)?;
    ensure!(
        crystal_assets::read_verified_compiled_game_pack(output)? == hosted,
        "hosted pack readback differs"
    );
    println!("{} {}", output.display(), hosted.identity()?.content_hash);
    Ok(())
}
