//! Compact-map review fixture; the earned regional journey remains separate.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, indoor_walk, position};

#[test]
#[ignore = "requires the external compact Central Lakes pack"]
fn central_lakes_controller_walks_bridge_dock_barriers_and_cabin_returns() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_CENTRAL_LAKES_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("published pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR15C00R00".into(),
            tile_x: 60,
            tile_y: 24,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit review placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    controller.set_runtime_journal_enabled(false);
    let grid = RouteGrid::new(
        &runtime,
        RouteSpec {
            prefix: "MnR15",
            columns: 1,
            rows: 1,
            tileset: "kanto",
            start: (60, 24),
            doors: BTreeMap::from([((53, 11), "MnR15LakeCabin")]),
        },
    );
    // The bridge is physical floor, traversable both ways on both rows.
    for y in [28, 29] {
        for x in 28..=38 {
            grid.walk(&mut controller, (x, y));
        }
        for x in (28..=38).rev() {
            grid.walk(&mut controller, (x, y));
        }
    }
    for (at, button) in [
        ((35, 15), GameButton::Left),
        ((38, 13), GameButton::Up),
        ((38, 16), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        for _ in 0..3 {
            controller.press(button).expect("try dock rail");
        }
        assert_eq!(position(&mut controller), at, "dock barrier failed");
    }
    for x in [35, 36, 38, 39, 40, 41, 42, 43] {
        grid.walk(&mut controller, (x, 15));
    }
    assert!(grid.talk(&mut controller, (37, 15)).contains("put the"));
    assert!(
        controller
            .snapshot()
            .expect("dock cache")
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "REVIVE" && s.quantity == 1)
    );
    assert!(grid.talk(&mut controller, (45, 17)).contains("rail ends"));
    for exit in [(5, 7), (6, 7)] {
        grid.walk(&mut controller, (53, 11));
        assert_eq!(
            controller
                .snapshot()
                .expect("cabin entry")
                .overworld
                .map_name,
            "MnR15LakeCabin"
        );
        indoor_walk(&mut controller, &runtime, exit);
        // Native downward carpets warp when leaving the carpet, rather than
        // immediately on entering it. Use the same departure input as play.
        for _ in 0..4 {
            if controller
                .snapshot()
                .expect("carpet departure")
                .overworld
                .map_name
                != "MnR15LakeCabin"
            {
                break;
            }
            controller
                .press(GameButton::Down)
                .expect("leave cabin carpet");
        }
        assert_eq!(
            controller
                .snapshot()
                .expect("cabin return")
                .overworld
                .map_name,
            "MnR15C00R00"
        );
        assert_eq!(position(&mut controller), (53, 12));
    }
    grid.walk(&mut controller, (60, 24));
    controller.press(GameButton::Start).expect("Start");
    assert!(controller.snapshot().expect("menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close menu");
    if let Some(save) = std::env::var_os("CRYSTAL_CENTRAL_LAKES_FIXTURE_SAVE") {
        controller.save(PathBuf::from(save)).expect("review save");
    }
}

#[test]
#[ignore = "requires the external compact Central Lakes pack"]
fn central_lakes_controller_hops_meadow_ledge_and_rejects_reverse() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_CENTRAL_LAKES_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("published pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR15C00R00".into(),
            tile_x: 22,
            tile_y: 17,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit upper ledge placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    controller.set_runtime_journal_enabled(false);
    let mut traversed_hop = false;
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native downhill input");
        let at = position(&mut controller);
        assert_ne!(at, (22, 19), "walked through the solid ledge face");
        traversed_hop |= at == (22, 18);
        if at == (22, 20) {
            break;
        }
    }
    assert!(traversed_hop, "did not enter the native HOP_DOWN quadrant");
    assert_eq!(
        position(&mut controller),
        (22, 20),
        "native hop did not land below the ledge"
    );
    for _ in 0..4 {
        controller
            .press(GameButton::Up)
            .expect("try reverse ascent");
    }
    assert_eq!(
        position(&mut controller),
        (22, 20),
        "ledge permits reverse ascent"
    );
}
