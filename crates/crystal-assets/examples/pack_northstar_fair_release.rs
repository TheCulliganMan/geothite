//! Produce both installable editions from the selected artwork and last public archives.
use anyhow::{Result, ensure};
use crystal_assets::{
    CompiledGamePack, read_loaded_verified_compiled_game_pack, read_verified_compiled_game_pack,
};
use std::path::{Path, PathBuf};
fn write(pack: &CompiledGamePack, path: &Path) -> Result<()> {
    pack.write_preserving_storage(path)?;
    ensure!(
        &read_verified_compiled_game_pack(path)? == pack,
        "pack changed on readback"
    );
    println!("{} {}", path.display(), pack.identity()?.content_hash);
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [art, prior, live, out, hosted] = args.as_slice() else {
        anyhow::bail!(
            "usage: pack_northstar_fair_release ART LAKESIDE_RAW LAKESIDE_HOSTED RAW_OUTPUT HOSTED_OUTPUT"
        )
    };
    for path in [out, hosted] {
        ensure!(
            !path.exists()
                && std::process::Command::new("git")
                    .args(["check-ignore", "--quiet", "--no-index", "--"])
                    .arg(path)
                    .status()?
                    .success(),
            "choose fresh ignored output"
        );
    }
    ensure!(out != hosted, "separate output paths required");
    let raw = crystal_assets::build_northstar_fair_release(
        read_verified_compiled_game_pack(art)?,
        &read_loaded_verified_compiled_game_pack(prior)?,
        &read_loaded_verified_compiled_game_pack(live)?,
    )?;
    write(&raw, out)?;
    let all = crystal_assets::build_all_pokemon_modpack(&raw)?;
    let battles = crystal_assets::build_default_battle_modpack(&all)?;
    let clock = crystal_assets::build_realtime_clock_modpack(&battles)?;
    write(
        &crystal_assets::build_player_customization_modpack(&clock)?,
        hosted,
    )
}
