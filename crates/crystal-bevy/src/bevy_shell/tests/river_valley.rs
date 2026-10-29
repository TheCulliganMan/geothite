//! Explicit geometry fixture; connected Mankato/Minneapolis progression is separate.
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

pub(super) fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_RIVER_VALLEY_TEST_PACK").expect("external pack"));
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("native pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR03C00R01".into(),
            tile_x: 18,
            tile_y: 10,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit southern placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

#[test]
#[ignore = "requires the external River Valley pack"]
fn river_valley_controller_walks_both_bridges_seam_terrace_and_cave_events() {
    let (_, runtime, mut controller) = fixture();
    // An empty party keeps geometry and event checks separate from encounters.
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR03",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start: (18, 58),
            doors: BTreeMap::from([((8, 37), "MnR03Cave1F")]),
        },
    );
    for point in [(18, 60), (20, 2), (12, 14), (10, 38), (40, 55), (12, 55)] {
        grid.walk(&mut controller, point);
    }
    for p in [(38, 47), (38, 48), (38, 47), (39, 48), (39, 49), (38, 50)] {
        grid.walk(&mut controller, p);
    }
    for (left, right, ys) in [(16, 27, [28, 29]), (26, 37, [54, 55])] {
        for y in ys {
            for x in [left, right, left] {
                grid.walk(&mut controller, (x, y));
            }
        }
    }
    for (at, b) in [
        ((20, 27), GameButton::Up),
        ((20, 30), GameButton::Down),
        ((30, 53), GameButton::Up),
        ((30, 56), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        for _ in 0..3 {
            controller.press(b).expect("native bridge rail");
        }
        assert_eq!(position(&mut controller), at);
    }
    for y in [42, 41, 40, 39, 40, 41, 42] {
        grid.walk(&mut controller, (41, y));
    }
    grid.walk(&mut controller, (37, 39));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native optional descent");
        assert_ne!(
            position(&mut controller),
            (37, 41),
            "walked through ledge wall"
        );
        if position(&mut controller) == (37, 42) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (37, 42));
    for _ in 0..4 {
        controller.press(GameButton::Up).expect("try reverse ledge");
    }
    assert_eq!(position(&mut controller), (37, 42));
    // The uphill return remains playable after the one-way drop.
    for at in [(41, 42), (41, 41), (41, 40), (41, 39), (37, 39)] {
        grid.walk(&mut controller, at);
    }
    // Route30-style woodland drop: all four native lanes hop south, reject
    // reverse ascent, then return through the ordinary eastern meadow bend.
    for x in 10..14 {
        grid.walk(&mut controller, (x, 19));
        for _ in 0..8 {
            controller
                .press(GameButton::Down)
                .expect("bird meadow native descent");
            assert_ne!(
                position(&mut controller),
                (x, 21),
                "walked through reverse wall"
            );
            if position(&mut controller) == (x, 22) {
                break;
            }
        }
        assert_eq!(position(&mut controller), (x, 22));
        for _ in 0..4 {
            controller
                .press(GameButton::Up)
                .expect("try meadow reverse ascent");
        }
        assert_eq!(position(&mut controller), (x, 22));
        for at in [(16, 22), (16, 20), (16, 18), (x, 19)] {
            grid.walk(&mut controller, at);
        }
    }
    for (at, text) in [
        ((20, 60), "MINNESOTA"),
        ((22, 8), "MINNEAPOLIS"),
        ((12, 36), "RIVER POCKET"),
        ((28, 26), "UPPER"),
        ((40, 44), "TERRACE"),
        ((13, 13), "bridge"),
        ((25, 53), "rails"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "interaction {at:?}"
        );
    }
    for at in [(41, 55), (13, 55), (39, 35)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    grid.walk(&mut controller, (8, 37));
    assert_eq!(
        controller
            .snapshot()
            .expect("cave entry")
            .overworld
            .map_name,
        "MnR03Cave1F"
    );
    // The installed cave's cache is reached through its northern gallery;
    // the older southern waypoint (10,7) is now permanent rock.
    for at in [(5, 6), (4, 7), (5, 3), (11, 3), (11, 5)] {
        indoor_walk(&mut controller, &runtime, at);
    }
    controller.press(GameButton::Right).expect("face cache");
    controller.press(GameButton::A).expect("Great Ball");
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (5, 6));
    controller.press(GameButton::Up).expect("face walker");
    controller.press(GameButton::A).expect("walker");
    assert!(controller.snapshot().expect("cave text").ui.text.is_some());
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (7, 9));
    assert_eq!(
        controller
            .snapshot()
            .expect("cave return")
            .overworld
            .map_name,
        "MnR03C00R00"
    );
    assert_eq!(position(&mut controller), (8, 38));
    let s = controller.snapshot().expect("all rewards");
    for item in ["SUPER_POTION", "GREAT_BALL", "REPEL", "ETHER"] {
        assert!(
            s.bag
                .items
                .iter()
                .chain(&s.bag.balls)
                .any(|slot| slot.item_id == item && slot.quantity == 1),
            "{item}"
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
                .is_event_flag_set(&format!("EVENT_MN_R03_ITEM_{n:02}"))
                .expect("pickup flag")
        );
    }
    grid.walk(&mut controller, (18, 58));
    controller.press(GameButton::Start).expect("Start");
    assert!(controller.snapshot().expect("menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close menu");
    if let Some(save) = std::env::var_os("CRYSTAL_RIVER_VALLEY_FIXTURE_SAVE") {
        controller
            .save(PathBuf::from(save))
            .expect("geometry fixture save");
    }
}
