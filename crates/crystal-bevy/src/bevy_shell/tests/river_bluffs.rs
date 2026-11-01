//! Explicit geometry fixture; connected Minneapolis/Saint Cloud progression is separate.
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
        PathBuf::from(std::env::var_os("CRYSTAL_RIVER_BLUFFS_TEST_PACK").expect("external pack"));
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
            map_name: "MnR05C00R02".into(),
            tile_x: 22,
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
#[ignore = "requires the external River Bluffs pack"]
fn river_bluffs_controller_walks_three_maps_two_drops_four_ramps_pier_and_both_cave_mouths() {
    let (_, runtime, mut controller) = fixture();
    // Empty-party geometry only; encounters and battle balance are separate.
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR05",
            columns: 1,
            rows: 3,
            tileset: "mn_headwaters",
            start: (22, 106),
            doors: BTreeMap::from([((16, 77), "MnR05RockCut"), ((30, 27), "MnR05RockCut")]),
        },
    );
    for at in [
        (22, 108),
        (24, 2),
        (23, 105),
        (12, 89),
        (29, 65),
        (16, 78),
        (37, 75),
        (29, 47),
        (15, 41),
        (30, 28),
        (51, 21),
        (37, 39),
        (55, 89),
        (25, 5),
    ] {
        grid.walk(&mut controller, at);
    }
    for (x, ys) in [
        (13, [78, 77, 76, 75, 76, 77, 78]),
        (29, [54, 53, 52, 51, 52, 53, 54]),
        (15, [42, 41, 40, 39, 40, 41, 42]),
        (25, [16, 15, 14, 13, 14, 15, 16]),
    ] {
        for y in ys {
            grid.walk(&mut controller, (x, y));
        }
    }
    for at in [
        (29, 47),
        (29, 48),
        (29, 47),
        (29, 48),
        (22, 95),
        (22, 96),
        (22, 95),
        (22, 96),
    ] {
        grid.walk(&mut controller, at);
    }
    for (y, foot) in [(39, 42), (75, 78)] {
        grid.walk(&mut controller, (37, y));
        for _ in 0..8 {
            controller.press(GameButton::Down).expect("native descent");
            assert_ne!(
                position(&mut controller),
                (37, foot - 1),
                "walked through the reverse wall"
            );
            if position(&mut controller) == (37, foot) {
                break;
            }
        }
        assert_eq!(position(&mut controller), (37, foot));
        for _ in 0..4 {
            controller.press(GameButton::Up).expect("try reverse drop");
        }
        assert_eq!(position(&mut controller), (37, foot));
        for at in if foot == 42 {
            vec![(15, 42), (15, 41), (15, 39), (37, 39)]
        } else {
            vec![(13, 78), (13, 77), (13, 75), (37, 75)]
        } {
            grid.walk(&mut controller, at);
        }
    }
    for at in [(47, 88), (60, 88), (47, 88), (60, 89), (47, 89)] {
        grid.walk(&mut controller, at);
    }
    for (at, button) in [
        ((51, 17), GameButton::Up),
        ((51, 22), GameButton::Down),
        ((54, 21), GameButton::Right),
        ((55, 87), GameButton::Up),
        ((55, 90), GameButton::Down),
        ((60, 89), GameButton::Right),
    ] {
        grid.walk(&mut controller, at);
        for _ in 0..3 {
            controller.press(button).expect("native pier barrier");
        }
        assert_eq!(position(&mut controller), at);
    }
    for (at, text) in [
        ((24, 106), "RIVER BLUFFS"),
        ((12, 78), "CLIMB"),
        ((18, 78), "DRY ROCK"),
        ((50, 18), "MISSISSIPPI"),
        ((46, 86), "RIVER PIER"),
        ((26, 4), "ST CLOUD"),
        ((19, 103), "city gyms"),
        ((57, 89), "river is fast"),
        ((53, 21), "old stone"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "interaction {at:?}"
        );
    }
    for at in [(9, 89), (37, 19), (41, 81)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    grid.walk(&mut controller, (16, 77));
    assert_eq!(
        controller
            .snapshot()
            .expect("lower entrance")
            .overworld
            .map_name,
        "MnR05RockCut"
    );
    indoor_walk(&mut controller, &runtime, (4, 13));
    controller.press(GameButton::Right).expect("face walker");
    controller.press(GameButton::A).expect("walker");
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (5, 6));
    controller.press(GameButton::Up).expect("face cave cache");
    controller.press(GameButton::A).expect("Repel cache");
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (11, 5));
    assert_eq!(
        position(&mut controller),
        (30, 28),
        "rock cut must reach the upper mouth"
    );
    for at in [(31, 28), (30, 28)] {
        grid.walk(&mut controller, at);
    }
    grid.walk(&mut controller, (30, 27));
    indoor_walk(&mut controller, &runtime, (5, 17));
    assert_eq!(
        position(&mut controller),
        (16, 78),
        "rock cut must return to the lower mouth"
    );
    for at in [(17, 78), (16, 78)] {
        grid.walk(&mut controller, at);
    }
    // Each mouth can also be left through its own ladder without a bounce.
    for (mouth, step, ladder, arrival) in [
        ((16, 77), (5, 16), (5, 17), (16, 78)),
        ((30, 27), (11, 4), (11, 5), (30, 28)),
    ] {
        grid.walk(&mut controller, mouth);
        indoor_walk(&mut controller, &runtime, step);
        indoor_walk(&mut controller, &runtime, ladder);
        assert_eq!(position(&mut controller), arrival);
    }
    for n in 1..=4 {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_R05_ITEM_{n:02}"))
                .expect("pickup flag")
        );
    }
    let snapshot = controller.snapshot().expect("all rewards");
    for item in ["SUPER_POTION", "GREAT_BALL", "REPEL", "ETHER"] {
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
    grid.walk(&mut controller, (22, 106));
    controller.press(GameButton::Start).expect("Start");
    assert!(
        controller
            .snapshot()
            .expect("native menu")
            .ui
            .menu
            .is_some()
    );
    controller.press(GameButton::B).expect("close menu");
    if let Some(path) = std::env::var_os("CRYSTAL_RIVER_BLUFFS_FIXTURE_SAVE") {
        controller.save(PathBuf::from(path)).expect("geometry save");
    }
}
