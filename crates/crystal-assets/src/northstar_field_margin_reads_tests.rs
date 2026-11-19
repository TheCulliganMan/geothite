use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};

#[test]
#[ignore = "requires an external archive in MN_FEN_TRAIL_PACK"]
fn installed_field_resolves_the_visible_faces_and_all_four_approaches() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("MN_FEN_TRAIL_PACK").context("lake archive")?);
    let loaded = crate::read_loaded_verified_compiled_game_pack(path)?;
    let music = loaded.pack().audio_manifest.music.keys().cloned().collect();
    let (_, mut session) = loaded
        .pack()
        .data()
        .start_overworld_session_at_runtime_tile(
            "MnFieldMargin",
            TilePosition::new(16, 30),
            &music,
        )?;
    for (x, y, facing, label) in [
        (7, 6, Direction::Right, "MnKensigardEdge"),
        (9, 5, Direction::Down, "MnKensigardEdge"),
        (9, 8, Direction::Up, "MnKensigardEdge"),
        (8, 8, Direction::Up, "MnKensigardEdge"),
        (7, 10, Direction::Right, "MnKensigardBoundary"),
        (8, 12, Direction::Up, "MnKensigardBoundary"),
        (9, 12, Direction::Up, "MnKensigardBoundary"),
        (10, 10, Direction::Left, "MnKensigardBoundary"),
        (11, 8, Direction::Right, "MnKensigardSoil"),
        (11, 9, Direction::Right, "MnKensigardSoil"),
        (13, 22, Direction::Right, "MnKensigardGrooves"),
        (13, 23, Direction::Right, "MnKensigardGrooves"),
    ] {
        session.player.tile = TilePosition::new(x, y);
        session.player.facing = facing;
        let found = session.check_interaction_checked(1)?;
        ensure!(
            found.is_some_and(|e| e.script == label),
            "missing {label} from {x},{y}"
        );
    }
    for (x, y) in [(12, 5), (13, 4), (14, 5), (13, 6)] {
        session.player.tile = TilePosition::new(x, y);
        let trigger = session.check_coord_event_checked(None)?;
        ensure!(
            trigger.is_some_and(|e| e.script_name == "MnKensigardReveal"),
            "missing reveal at {x},{y}"
        );
    }
    Ok(())
}
