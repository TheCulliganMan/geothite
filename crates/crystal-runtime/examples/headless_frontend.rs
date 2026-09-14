//! A frontend entry point that needs neither Bevy nor an audio device.
use anyhow::{Context, Result};
use crystal_runtime::{
    CrystalRuntime, RuntimeGameShell,
    assets::{AssetRoot, read_loaded_verified_compiled_game_pack},
};
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    let pack_path = PathBuf::from(
        env::args()
            .nth(1)
            .context("usage: headless_frontend <game.crystalpack>")?,
    );
    let pack_path = pack_path.canonicalize().context("resolve game pack")?;
    let root = AssetRoot::new(pack_path.parent().context("pack parent")?);
    let loaded = read_loaded_verified_compiled_game_pack(&pack_path)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    let spawn = runtime.title_new_game_spawn_identifier()?;
    let mut game = RuntimeGameShell::new_game(root, runtime, spawn)?;
    // The host maps its own input to GameButton and supplies the simulation clock.
    // This single empty input frame exercises the backend without a window.
    game.tick([])?;
    let snapshot = game.snapshot()?;
    println!(
        "phase={:?} overworld={:?}",
        snapshot.phase, snapshot.overworld
    );
    for cue in game.drain_resolved_audio_events()?.events {
        println!("audio={cue:?}");
    }
    // Device adapters can send these exact samples to their own output API.
    if let Some((id, program)) = game.runtime().audio().sound_effects().first_key_value() {
        let pcm = crystal_runtime::audio::pcm::decode_program_source(program.source.clone())?;
        println!(
            "{id}: {} stereo frames, loop={:?}",
            pcm.samples.len() / 2,
            pcm.loop_range
        );
    }
    Ok(())
}
