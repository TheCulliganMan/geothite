//! Measures the production controller without a window or audio device.
//!
//! Pack loading, new-game scripts, snapshots, and warmup are outside each
//! measurement. This measures controller time per input, not rendered FPS.

use std::env;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result, ensure};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let path = PathBuf::from(args.next().context("usage: input_timing <pack> [frames]")?)
        .canonicalize()
        .context("resolve pack")?;
    let frames: u32 = args
        .next()
        .map(|value| value.parse())
        .transpose()
        .context("parse frames")?
        .unwrap_or(1_024);
    ensure!(
        frames >= 64 && frames.is_multiple_of(64),
        "frames must be a positive multiple of 64"
    );
    let root = AssetRoot::new(path.parent().context("pack parent")?);
    let loaded = read_loaded_verified_compiled_game_pack(&path)?;
    let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)?;
    for scenario in ["idle", "turns", "walking"] {
        let mut controller =
            VisibleShellController::new_game(root.clone(), runtime.clone(), "CHRIS", None)?;
        controller.set_runtime_journal_enabled(false);
        let initial = controller.snapshot()?.overworld;
        let mut samples = Vec::new();
        for batch in 0..8 {
            let start = Instant::now();
            for frame in 0..frames {
                let button = match scenario {
                    "turns" if frame % 2 == 0 => Some(GameButton::Left),
                    "turns" => Some(GameButton::Right),
                    "walking" if frame % 64 < 32 => Some(GameButton::Left),
                    "walking" => Some(GameButton::Right),
                    _ => None,
                };
                controller.press_buttons(button)?;
            }
            let elapsed = start.elapsed().as_secs_f64();
            // Discard the first batch to warm the controller and allocator.
            if batch > 0 {
                samples.push(elapsed * 1e9 / f64::from(frames));
            }
        }
        let final_state = controller.snapshot()?;
        ensure!(
            final_state.overworld.map_name == initial.map_name,
            "scenario left its starting map"
        );
        ensure!(
            scenario != "walking" || final_state.overworld.tile != initial.tile,
            "walking inputs did not move the player; use at least 64 frames"
        );
        ensure!(
            scenario != "turns" || final_state.overworld.facing != initial.facing,
            "turning inputs did not change the player's facing"
        );
        samples.sort_by(f64::total_cmp);
        println!(
            "{}",
            serde_json::json!({
                "scenario": scenario,
                "frames_per_batch": frames,
                "ns_per_input": samples,
                "median_ns": samples[samples.len() / 2],
                "overworld": final_state.overworld,
                "pending_scripts": controller.has_pending_script_work(),
            })
        );
    }
    Ok(())
}
