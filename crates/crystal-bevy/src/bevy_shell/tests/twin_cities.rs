//! Isolated native park geometry; earned Twin Cities progression is separate.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, position};

pub(super) fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_TWIN_CITIES_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("native park pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR04C00R00".into(),
            tile_x: 4,
            tile_y: 18,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit western placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

#[test]
#[ignore = "requires the external Twin Cities pack"]
fn twin_cities_controller_walks_grass_alternative_bridge_rails_and_every_event() {
    let (_, runtime, mut controller) = fixture();
    // Empty party isolates collision and dialogue from the separate battle journey.
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR04",
            columns: 1,
            rows: 1,
            tileset: "mn_river_park",
            start: (4, 18),
            doors: BTreeMap::new(),
        },
    );
    for at in [
        (2, 18),
        (8, 13),
        (12, 12),
        (16, 18),
        (28, 19),
        (22, 23),
        (47, 17),
        (43, 27),
        (52, 18),
        (2, 18),
    ] {
        grid.walk(&mut controller, at);
    }
    // The north-connected elm tongue bends the east promenade around its tip.
    // Traverse the ordinary floor turn in both directions; the former straight
    // row is real canopy collision rather than a decorative painted island.
    for at in [
        (42, 18),
        (43, 20),
        (46, 20),
        (46, 18),
        (52, 18),
        (46, 18),
        (46, 20),
        (43, 20),
        (42, 18),
    ] {
        grid.walk(&mut controller, at);
    }
    for (at, button) in [((43, 18), GameButton::Right), ((46, 18), GameButton::Left)] {
        grid.walk(&mut controller, at);
        controller.press(button).expect("east promenade elm wall");
        assert_eq!(position(&mut controller), at);
    }
    for y in [18, 19] {
        for x in [22, 35, 22] {
            grid.walk(&mut controller, (x, y));
        }
    }
    for (at, button) in [
        ((28, 18), GameButton::Up),
        ((28, 19), GameButton::Down),
        ((22, 25), GameButton::Right),
        ((35, 25), GameButton::Left),
        ((10, 30), GameButton::Down),
        ((46, 22), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        for _ in 0..3 {
            controller.press(button).expect("native fence or bank");
        }
        assert_eq!(position(&mut controller), at);
    }
    for (at, text) in [
        ((2, 16), "MINNEAPOLIS"),
        ((52, 16), "ST PAUL"),
        ((12, 8), "ROSE WALK"),
        ((36, 14), "PARK POND BRIDGE"),
        ((42, 24), "RIVER PARK"),
        ((7, 25), "flowers"),
        ((23, 23), "stone banks"),
    ] {
        assert!(grid.talk(&mut controller, at).contains(text), "{at:?}");
    }
    for (at, item) in [
        ((7, 13), "SUPER POTION"),
        ((49, 27), "GREAT BALL"),
        ((9, 27), "ETHER"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(item),
            "native pickup {item}"
        );
    }
    let snapshot = controller.snapshot().expect("all park rewards");
    for item in ["SUPER_POTION", "GREAT_BALL", "ETHER"] {
        assert!(
            snapshot
                .bag
                .items
                .iter()
                .chain(&snapshot.bag.balls)
                .any(|s| s.item_id == item && s.quantity == 1),
            "{item}"
        );
    }
    for n in [1, 2, 4] {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_R04_ITEM_{n:02}"))
                .expect("pickup flag")
        );
    }
    for n in [1, 2] {
        assert!(
            !controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_R04_TRAINER_{n:02}"))
                .expect("unfought trainer")
        );
    }
    grid.walk(&mut controller, (4, 18));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
    if let Some(path) = std::env::var_os("CRYSTAL_TWIN_CITIES_FIXTURE_SAVE") {
        controller.save(PathBuf::from(path)).expect("geometry save");
    }
}
