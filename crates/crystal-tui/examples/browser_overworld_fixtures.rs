//! Art evaluation only. Production never imports these saves or moves players
//! to fixture coordinates; every screenshot still renders the real pack/controller.
use anyhow::{Context, Result};
use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_runtime::{CrystalRuntime, RuntimeGameShell};
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let pack = PathBuf::from(
        args.next()
            .context("usage: browser_overworld_fixtures PACK OUTPUT_DIRECTORY")?,
    );
    let output = PathBuf::from(args.next().context("missing output directory")?);
    let root = AssetRoot::new(pack.parent().context("pack parent")?);
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&pack)?,
    )?;
    let spawn = runtime.title_new_game_spawn_identifier()?;
    std::fs::create_dir_all(&output)?;
    for (map, x, y) in [
        ("NewBarkTown", 10, 8),
        ("VioletCity", 16, 17),
        ("GoldenrodCity", 15, 22),
        ("IlexForest", 6, 18),
        ("UnionCave1F", 8, 10),
        ("IcePath1F", 9, 10),
        ("TinTower1F", 8, 8),
        ("OlivineCity", 20, 20),
        ("Route40", 9, 10),
        ("VioletPokecenter1F", 5, 5),
    ] {
        let mut fixture = None;
        // Stay near the chosen landmark, but never spawn inside a source wall.
        // Construction validates traversal against the real map/collision data.
        'search: for radius in 0..=4i16 {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx.abs().max(dy.abs()) != radius {
                        continue;
                    }
                    if let Ok(game) = RuntimeGameShell::new_game_at_runtime_tile(
                        root.clone(),
                        runtime.clone(),
                        spawn,
                        map,
                        x + dx,
                        y + dy,
                    ) {
                        fixture = Some(game);
                        break 'search;
                    }
                }
            }
        }
        let mut fixture =
            fixture.with_context(|| format!("no valid art fixture near {map} ({x},{y})"))?;
        fixture.save(output.join(format!("{map}.crystalsave")))?;
    }
    Ok(())
}
