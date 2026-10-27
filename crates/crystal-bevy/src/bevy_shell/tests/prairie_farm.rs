//! Explicit empty-party geometry fixture; earned Q02 progression is separate.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, finish_text, indoor_walk, position};

#[test]
#[ignore = "requires the external Prairie Farm pack"]
fn prairie_farm_controller_walks_west_ledge_bridge_dialogue_rewards_and_both_rooms() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_PRAIRIE_FARM_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("native pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR02C00R00".into(),
            tile_x: 54,
            tile_y: 14,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit first-badge route placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    controller.set_runtime_journal_enabled(false);
    // Empty-party geometry only. This permits checking every grass tile with
    // native inputs, but does not establish encounters or starter-team balance.
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR02",
            columns: 1,
            rows: 1,
            tileset: if runtime.data().tilesets.contains_key("mn_prairie_kanto") {
                "mn_prairie_kanto"
            } else {
                "kanto"
            },
            start: (54, 14),
            doors: BTreeMap::from([((11, 11), "MnR02Farmhouse"), ((52, 33), "MnR02ToolShed")]),
        },
    );
    for point in [(2, 30), (11, 13), (40, 14), (49, 35), (23, 34)] {
        grid.walk(&mut controller, point);
    }
    grid.walk(&mut controller, (48, 15));
    for _ in 0..8 {
        controller
            .press(GameButton::Left)
            .expect("native westward drop");
        assert_ne!(
            position(&mut controller),
            (46, 15),
            "walked through bank wall"
        );
        if position(&mut controller) == (45, 15) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (45, 15));
    for _ in 0..4 {
        controller
            .press(GameButton::Right)
            .expect("try reverse bank");
    }
    assert_eq!(position(&mut controller), (45, 15));
    // The uphill return remains playable after the one-way drop.
    for at in [(41, 26), (52, 26), (52, 24), (48, 15)] {
        grid.walk(&mut controller, at);
    }
    // Both deck rows work both ways; the rail's inner strips remain walkable.
    for y in [24, 25] {
        for x in [26, 37, 26] {
            grid.walk(&mut controller, (x, y));
        }
    }
    for (at, button) in [
        ((32, 23), GameButton::Up),
        ((32, 26), GameButton::Down),
        ((28, 19), GameButton::Right),
        ((35, 19), GameButton::Left),
    ] {
        grid.walk(&mut controller, at);
        for _ in 0..3 {
            controller.press(button).expect("native channel barrier");
        }
        assert_eq!(position(&mut controller), at);
    }
    for (at, text) in [
        ((56, 13), "PRAIRIE"),
        ((14, 15), "LIND FARM"),
        ((50, 19), "NURSERY"),
        ((48, 37), "TOOL SHED"),
        ((4, 33), "MANKATO"),
        ((37, 23), "stakes"),
        ((43, 11), "short"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "interaction {at:?}"
        );
    }
    for at in [(25, 35), (19, 11)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    for (door, name, arrival, resident, resident_face, notice, item, item_approach, exit) in [
        (
            (11, 11),
            "MnR02Farmhouse",
            (11, 12),
            (5, 3),
            GameButton::Right,
            (4, 1),
            (7, 7),
            (7, 8),
            (4, 9),
        ),
        (
            (52, 33),
            "MnR02ToolShed",
            (52, 34),
            (5, 3),
            GameButton::Right,
            (4, 1),
            (7, 5),
            (7, 6),
            (5, 7),
        ),
    ] {
        grid.walk(&mut controller, door);
        assert_eq!(
            controller
                .snapshot()
                .expect("native entry")
                .overworld
                .map_name,
            name
        );
        indoor_walk(&mut controller, &runtime, resident);
        controller.press(resident_face).expect("face keeper");
        controller.press(GameButton::A).expect("keeper");
        assert!(
            controller
                .snapshot()
                .expect("keeper text")
                .ui
                .text
                .is_some()
        );
        finish_text(&mut controller);
        indoor_walk(&mut controller, &runtime, notice);
        controller.press(GameButton::Up).expect("face wall note");
        controller.press(GameButton::A).expect("wall note");
        assert!(
            controller
                .snapshot()
                .expect("wall note text")
                .ui
                .text
                .is_some()
        );
        finish_text(&mut controller);
        indoor_walk(&mut controller, &runtime, item_approach);
        controller
            .press(GameButton::Up)
            .expect("face indoor reward");
        controller.press(GameButton::A).expect("indoor reward");
        assert!(
            controller
                .snapshot()
                .expect("reward text")
                .ui
                .text
                .is_some(),
            "item {item:?}"
        );
        finish_text(&mut controller);
        indoor_walk(&mut controller, &runtime, exit);
        for _ in 0..4 {
            if controller
                .snapshot()
                .expect("carpet departure")
                .overworld
                .map_name
                != name
            {
                break;
            }
            controller.press(GameButton::Down).expect("leave carpet");
        }
        assert_eq!(
            controller
                .snapshot()
                .expect("native return")
                .overworld
                .map_name,
            "MnR02C00R00"
        );
        assert_eq!(position(&mut controller), arrival);
        // Reenter and exercise the second carpet without changing fixture state.
        grid.walk(&mut controller, door);
        indoor_walk(&mut controller, &runtime, (exit.0 + 1, exit.1));
        for _ in 0..4 {
            if controller
                .snapshot()
                .expect("other carpet")
                .overworld
                .map_name
                != name
            {
                break;
            }
            controller
                .press(GameButton::Down)
                .expect("leave other carpet");
        }
        assert_eq!(position(&mut controller), arrival);
    }
    let s = controller.snapshot().expect("rewards");
    for item in ["POTION", "POKE_BALL", "BERRY", "PARLYZ_HEAL"] {
        assert!(
            s.bag
                .items
                .iter()
                .chain(&s.bag.balls)
                .any(|slot| slot.item_id == item && slot.quantity == 1),
            "reward {item}"
        );
    }
    for n in 1..=4 {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_R02_ITEM_{n:02}"))
                .expect("pickup flag")
        );
    }
    grid.walk(&mut controller, (54, 14));
    controller.press(GameButton::Start).expect("Start");
    assert!(controller.snapshot().expect("menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close menu");
    if let Some(save) = std::env::var_os("CRYSTAL_PRAIRIE_FARM_FIXTURE_SAVE") {
        controller.save(PathBuf::from(save)).expect("fixture save");
    }
}
