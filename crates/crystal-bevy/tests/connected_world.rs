//! Controller regressions for the connected modeled-world renderer.
//!
//! These tests never install a renderer or alter scripts, flags, collisions,
//! warps, or a player's party. The location fixtures isolate map seams and
//! doorways; they are not evidence of completing the normal new-game journey.
//! The New Bark gate is also checked after an authoritative seam crossing.
#![cfg(feature = "bevy-shell")]

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_bevy::VisibleShellController;
use crystal_runtime::core::{input::GameButton, world::map::TilePosition};
use crystal_runtime::{CrystalRuntime, RuntimeGameShell, RuntimeShellPhase};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

fn external_runtime() -> (AssetRoot, CrystalRuntime) {
    let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/core-modular.browser.crystalpack")
        });
    let pack = pack.canonicalize().unwrap_or_else(|error| panic!(
        "connected-world regressions need an external compatible pack at {}: {error}; set CRYSTAL_RENDER_TEST_PACK", pack.display()));
    let root = AssetRoot::new(pack.parent().expect("pack directory"));
    let loaded = read_loaded_verified_compiled_game_pack(&pack).expect("verify external pack");
    let runtime =
        CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("load external pack");
    (root, runtime)
}

fn location_fixture(map: &str, x: i16, y: i16) -> VisibleShellController {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let (root, runtime) = external_runtime();
    let mut game = RuntimeGameShell::new_game_at_runtime_tile(
        root.clone(),
        runtime.clone(),
        runtime.title_new_game_spawn_identifier().unwrap(),
        map,
        x,
        y,
    )
    .expect("fresh location fixture");
    let save = std::env::temp_dir().join(format!(
        "geothite-connected-{}-{}.crystalsave",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    game.save(&save).expect("write transient location fixture");
    let result = VisibleShellController::load_save(root, runtime, save.clone(), None);
    std::fs::remove_file(&save).expect("remove transient location fixture");
    // Continue may normalize and rotate the save through its normal backup
    // policy. Both copies belong only to this location fixture.
    let backup = PathBuf::from(format!("{}.bak", save.display()));
    if backup.exists() {
        std::fs::remove_file(backup).expect("remove transient location fixture backup");
    }
    let mut controller = result.expect("load fixture into the production visible controller");
    assert!(
        controller.snapshot().unwrap().party.slots.is_empty(),
        "preview fixture must not seed a test party"
    );
    controller
}

fn assert_at(controller: &mut VisibleShellController, map: &str, x: i16, y: i16) {
    let snapshot = controller.snapshot().expect("controller snapshot");
    assert_eq!(snapshot.overworld.map_name, map);
    assert_eq!(snapshot.overworld.tile, TilePosition::new(x, y));
    assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
    assert!(
        snapshot.ui.text.is_none(),
        "unexpected input-owning dialogue"
    );
    assert!(
        snapshot.battle.is_none(),
        "unexpected battle on clear seam or doorway"
    );
}

fn move_tiles(controller: &mut VisibleShellController, button: GameButton, count: usize) {
    for _ in 0..count {
        let before = controller
            .snapshot()
            .expect("before directional press")
            .overworld;
        let mut moved = false;
        // Turning may consume one input. Every attempted step still travels
        // through VisibleShellController::press, as keyboard/MCP input does.
        for _ in 0..4 {
            controller
                .press(button)
                .expect("production directional input");
            let after = controller
                .snapshot()
                .expect("after directional press")
                .overworld;
            if after.map_name != before.map_name || after.tile != before.tile {
                moved = true;
                break;
            }
        }
        assert!(
            moved,
            "blocked {button:?} on {} {:?}",
            before.map_name, before.tile
        );
    }
}

#[test]
fn renderer_neutral_connected_world_east_seam_roundtrip_preserves_new_bark_gate() {
    let mut controller = location_fixture("Route29", 59, 8);
    assert_at(&mut controller, "Route29", 59, 8);
    move_tiles(&mut controller, GameButton::Right, 1);
    assert_at(&mut controller, "NewBarkTown", 0, 8);
    move_tiles(&mut controller, GameButton::Left, 1);
    assert_at(&mut controller, "Route29", 59, 8);
    move_tiles(&mut controller, GameButton::Right, 1);
    assert_at(&mut controller, "NewBarkTown", 0, 8);

    // The location fixture must not silently unlock the authored story gate.
    move_tiles(&mut controller, GameButton::Right, 1);
    let gate = controller.snapshot().unwrap();
    assert_eq!(gate.overworld.tile, TilePosition::new(1, 8));
    assert_eq!(gate.phase, RuntimeShellPhase::Text);
    assert!(
        gate.ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("Wait,")),
        "teacher gate must retain its dialogue"
    );
    controller.press(GameButton::Left).unwrap();
    assert_eq!(
        controller.snapshot().unwrap().overworld.tile,
        gate.overworld.tile,
        "the gate's dialogue owns movement"
    );
    for _ in 0..64 {
        if controller.snapshot().unwrap().phase == RuntimeShellPhase::Overworld {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    assert_at(&mut controller, "NewBarkTown", 5, 8);
}

#[test]
fn renderer_neutral_connected_world_west_seam_roundtrips() {
    let mut controller = location_fixture("Route29", 0, 6);
    assert_at(&mut controller, "Route29", 0, 6);
    for _ in 0..2 {
        move_tiles(&mut controller, GameButton::Left, 1);
        assert_at(&mut controller, "CherrygroveCity", 39, 6);
        move_tiles(&mut controller, GameButton::Right, 1);
        assert_at(&mut controller, "Route29", 0, 6);
    }
}

#[test]
fn renderer_neutral_connected_world_cherrygrove_center_and_mart_roundtrip() {
    let mut controller = location_fixture("CherrygroveCity", 29, 4);
    assert_at(&mut controller, "CherrygroveCity", 29, 4);
    for _ in 0..2 {
        move_tiles(&mut controller, GameButton::Up, 1);
        assert_at(&mut controller, "CherrygrovePokecenter1F", 3, 7);
        move_tiles(&mut controller, GameButton::Up, 1);
        assert_at(&mut controller, "CherrygrovePokecenter1F", 3, 6);
        move_tiles(&mut controller, GameButton::Down, 1);
        assert_at(&mut controller, "CherrygrovePokecenter1F", 3, 7);
        // Directional exit carpets fire when walking off the bottom edge.
        move_tiles(&mut controller, GameButton::Down, 1);
        assert_at(&mut controller, "CherrygroveCity", 29, 3);
        move_tiles(&mut controller, GameButton::Down, 1);
        move_tiles(&mut controller, GameButton::Left, 6);
        assert_at(&mut controller, "CherrygroveCity", 23, 4);
        move_tiles(&mut controller, GameButton::Up, 1);
        assert_at(&mut controller, "CherrygroveMart", 3, 7);
        move_tiles(&mut controller, GameButton::Up, 1);
        assert_at(&mut controller, "CherrygroveMart", 3, 6);
        move_tiles(&mut controller, GameButton::Down, 1);
        assert_at(&mut controller, "CherrygroveMart", 3, 7);
        move_tiles(&mut controller, GameButton::Down, 1);
        assert_at(&mut controller, "CherrygroveCity", 23, 3);
        move_tiles(&mut controller, GameButton::Down, 1);
        move_tiles(&mut controller, GameButton::Right, 6);
        assert_at(&mut controller, "CherrygroveCity", 29, 4);
    }
}

#[test]
fn renderer_neutral_connected_world_normal_new_game_keeps_mom_and_teacher_scripts() {
    let (root, runtime) = external_runtime();
    let mut controller =
        VisibleShellController::new_game(root, runtime, "CHRIS", None).expect("normal new game");
    assert_at(&mut controller, "PlayersHouse2F", 3, 3);
    controller.press(GameButton::Start).unwrap();
    assert!(
        controller
            .snapshot()
            .unwrap()
            .ui
            .menu
            .unwrap()
            .layout
            .vertical_menus[0]
            .options
            .iter()
            .any(|option| option.contains("PACK"))
    );
    controller.press(GameButton::B).unwrap();
    move_tiles(&mut controller, GameButton::Right, 4);
    move_tiles(&mut controller, GameButton::Up, 3);
    move_tiles(&mut controller, GameButton::Down, 4);
    let mom = controller.snapshot().unwrap();
    assert_eq!(mom.overworld.map_name, "PlayersHouse1F");
    assert_eq!(mom.overworld.tile, TilePosition::new(9, 4));
    assert_eq!(
        mom.ui.text.as_ref().map(|text| text.label.as_str()),
        Some("ElmsLookingForYouText")
    );
    controller.press(GameButton::Right).unwrap();
    assert_eq!(
        controller.snapshot().unwrap().overworld.tile,
        mom.overworld.tile
    );
    let first_page = controller
        .snapshot()
        .unwrap()
        .ui
        .text
        .unwrap()
        .asm_text
        .unwrap();
    controller.press(GameButton::A).unwrap();
    let next_page = controller
        .snapshot()
        .unwrap()
        .ui
        .text
        .unwrap()
        .asm_text
        .unwrap();
    assert_ne!(
        first_page, next_page,
        "A must advance the authored visible page"
    );
    for _ in 0..128 {
        let snapshot = controller.snapshot().unwrap();
        if snapshot.phase == RuntimeShellPhase::Overworld && snapshot.ui.text.is_none() {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    assert_at(&mut controller, "PlayersHouse1F", 9, 4);
    for (button, count) in [
        (GameButton::Down, 2),
        (GameButton::Left, 2),
        (GameButton::Down, 4),
        (GameButton::Left, 11),
        (GameButton::Down, 1),
        (GameButton::Left, 1),
    ] {
        move_tiles(&mut controller, button, count);
    }
    let gate = controller.snapshot().unwrap();
    assert_eq!(gate.overworld.map_name, "NewBarkTown");
    assert_eq!(gate.overworld.tile, TilePosition::new(1, 8));
    assert!(
        gate.ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_deref())
            .is_some_and(|text| text.contains("Wait, CHRIS!"))
    );
    for _ in 0..64 {
        if controller.snapshot().unwrap().phase == RuntimeShellPhase::Overworld {
            break;
        }
        controller.press(GameButton::A).unwrap();
    }
    assert_at(&mut controller, "NewBarkTown", 5, 8);
}

#[test]
fn renderer_neutral_connected_world_route30_offset_seam_roundtrips() {
    let mut controller = location_fixture("CherrygroveCity", 16, 0);
    assert_at(&mut controller, "CherrygroveCity", 16, 0);
    for _ in 0..2 {
        move_tiles(&mut controller, GameButton::Up, 1);
        assert_at(&mut controller, "Route30", 6, 53);
        move_tiles(&mut controller, GameButton::Down, 1);
        assert_at(&mut controller, "CherrygroveCity", 16, 0);
    }
}

#[test]
fn renderer_neutral_connected_world_violet_center_and_gym_roundtrip() {
    for (door_x, door_y, interior, arrival_x, arrival_y) in [
        (31, 25, "VioletPokecenter1F", 3, 7),
        (18, 17, "VioletGym", 4, 15),
    ] {
        let mut controller = location_fixture("VioletCity", door_x, door_y + 1);
        assert_at(&mut controller, "VioletCity", door_x, door_y + 1);
        for _ in 0..2 {
            move_tiles(&mut controller, GameButton::Up, 1);
            assert_at(&mut controller, interior, arrival_x, arrival_y);
            // Stay below all gym trainers' sightlines; no event is disabled.
            move_tiles(&mut controller, GameButton::Up, 1);
            assert_at(&mut controller, interior, arrival_x, arrival_y - 1);
            move_tiles(&mut controller, GameButton::Down, 1);
            assert_at(&mut controller, interior, arrival_x, arrival_y);
            move_tiles(&mut controller, GameButton::Down, 1);
            assert_at(&mut controller, "VioletCity", door_x, door_y);
            move_tiles(&mut controller, GameButton::Down, 1);
            assert_at(&mut controller, "VioletCity", door_x, door_y + 1);
        }
    }
}
