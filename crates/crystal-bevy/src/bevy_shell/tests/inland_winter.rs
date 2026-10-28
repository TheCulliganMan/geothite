//! Explicit local placements, then production input through every inland ice lane.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};

fn position(controller: &mut VisibleShellController) -> (i16, i16) {
    let snapshot = controller.snapshot().expect("native winter snapshot");
    (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    map: &str,
    at: (i16, i16),
) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit winter lane fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    controller.set_runtime_journal_enabled(false);
    controller
}

#[test]
#[ignore = "requires completed winter pack in CRYSTAL_INLAND_WINTER_CONTROLLER_PACK"]
fn inland_winter_controller_all40_ice_quadrants_force_slides_and_stop_on_dry_land() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_INLAND_WINTER_CONTROLLER_PACK")
            .expect("published final winter pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(path)
            .expect("verified final winter pack"),
    )
    .expect("native winter runtime");
    // Each of the two gameplay lanes crosses all native ICE quadrants in its
    // frozen six-/eight-step course, with actual dry stops in both directions.
    for (map, lane, from, stop, direction, reverse) in [
        (
            "MnR12C00R01",
            32,
            (32, 23),
            (32, 30),
            GameButton::Down,
            GameButton::Up,
        ),
        (
            "MnR12C00R01",
            33,
            (33, 23),
            (33, 30),
            GameButton::Down,
            GameButton::Up,
        ),
        (
            "MnR12C01R00",
            12,
            (29, 12),
            (36, 12),
            GameButton::Right,
            GameButton::Left,
        ),
        (
            "MnR12C01R00",
            13,
            (29, 13),
            (36, 13),
            GameButton::Right,
            GameButton::Left,
        ),
        (
            "MnR13C00R00",
            46,
            (46, 7),
            (46, 16),
            GameButton::Down,
            GameButton::Up,
        ),
        (
            "MnR13C00R00",
            47,
            (47, 7),
            (47, 16),
            GameButton::Down,
            GameButton::Up,
        ),
    ] {
        let mut controller = fixture(&root, &runtime, map, from);
        let module = &runtime.data().maps[map];
        let collision = &runtime.data().tilesets[&module.attributes.tileset_name].collision;
        let kind = |at: (i16, i16)| {
            let block = module.blocks
                [(at.1 as usize / 2) * usize::from(module.attributes.width) + at.0 as usize / 2];
            collision[&format!("{block:02x}")][(at.1 as usize % 2) * 2 + at.0 as usize % 2].as_str()
        };
        assert_eq!(kind(from), "FLOOR");
        assert_eq!(kind(stop), "FLOOR");
        for (start, end, button, opposite) in [
            (from, stop, direction, reverse),
            (stop, from, reverse, direction),
        ] {
            assert_eq!(position(&mut controller), start);
            for _ in 0..4 {
                controller
                    .press(button)
                    .expect("turn/enter frozen native lane");
                controller.wait_frames(1).expect("native ice frame");
                if position(&mut controller) != start {
                    break;
                }
            }
            let entered = position(&mut controller);
            assert_ne!(entered, start, "lane {map} {lane}");
            if entered != end {
                assert_eq!(kind(entered), "ICE");
                controller
                    .press(opposite)
                    .expect("opposing input during forced slide");
                let after = position(&mut controller);
                match button {
                    GameButton::Down => assert!(after.1 >= entered.1),
                    GameButton::Up => assert!(after.1 <= entered.1),
                    GameButton::Right => assert!(after.0 >= entered.0),
                    GameButton::Left => assert!(after.0 <= entered.0),
                    _ => unreachable!("cardinal ice course"),
                }
            }
            controller.wait_frames(320).expect("native slide settles");
            assert_eq!(
                position(&mut controller),
                end,
                "native dry stop {map} {lane}"
            );
            controller.wait_frames(80).expect("stable dry stop");
            assert_eq!(position(&mut controller), end);
            assert_eq!(
                controller.snapshot().expect("dry stop phase").phase,
                RuntimeShellPhase::Overworld
            );
        }
        if let Some(output) = std::env::var_os("CRYSTAL_INLAND_WINTER_CAPTURE_SAVES_DIR") {
            let dir = PathBuf::from(output);
            std::fs::create_dir_all(&dir).expect("capture saves directory");
            let path = dir.join(format!("{map}-lane-{lane}-returned.crystalsave"));
            assert!(!path.exists(), "immutable reached winter save");
            controller.save(path).expect("native reached stop save");
        }
    }
}
