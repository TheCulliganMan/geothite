//! Explicit geometry fixture; earned Winona/Rochester progression is separate.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde::Deserialize;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::authored_route_fixture::{
    RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, opposite, position,
};
use super::r10_gallery::completed_visible_page;

#[test]
#[ignore = "requires the external Driftless pack"]
fn driftless_controller_walks_ramp_ledge_dock_dialogue_pickups_and_both_rooms() {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_DRIFTLESS_TEST_PACK").expect("external pack"));
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
            map_name: "MnR01C00R00".into(),
            tile_x: 52,
            tile_y: 24,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit post-grass placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("name");
    controller.set_runtime_journal_enabled(false);
    // This fixture has no party and checks geometry only. Cross the common
    // grass with production inputs to reach the entrance sign; wild encounters
    // and starter battles require the separate populated-party journey.
    for (target, button) in [((60, 24), GameButton::Right), ((60, 21), GameButton::Up)] {
        for _ in 0..32 {
            if position(&mut controller) == target {
                break;
            }
            controller.press(button).expect("entrance geometry input");
            assert!(
                controller
                    .snapshot()
                    .expect("empty-party preview")
                    .battle
                    .is_none()
            );
        }
        assert_eq!(position(&mut controller), target);
    }
    controller
        .press(GameButton::Up)
        .expect("face entrance sign");
    controller.press(GameButton::A).expect("entrance sign");
    assert!(
        controller
            .snapshot()
            .expect("entrance directions")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("DRIFTLESS"))
    );
    finish_text(&mut controller);
    for (target, button) in [((60, 24), GameButton::Down), ((52, 24), GameButton::Left)] {
        for _ in 0..32 {
            if position(&mut controller) == target {
                break;
            }
            controller
                .press(button)
                .expect("return through grass geometry");
        }
        assert_eq!(position(&mut controller), target);
    }
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR01",
            columns: 1,
            rows: 1,
            tileset: "johto",
            start: (52, 24),
            doors: BTreeMap::from([((18, 17), "MnR01Cave1F"), ((49, 15), "MnR01Farmhouse")]),
        },
    );
    for point in [(2, 24), (49, 17), (19, 19), (27, 25), (17, 35), (7, 25)] {
        grid.walk(&mut controller, point);
    }
    // The narrow opening is a native bidirectional ramp, not a painted wall cut.
    for y in [20, 19, 18, 17, 18, 19, 20] {
        grid.walk(&mut controller, (41, y));
    }
    grid.walk(&mut controller, (29, 15));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native downhill input");
        assert_ne!(
            position(&mut controller),
            (29, 17),
            "walked through ledge wall"
        );
        if position(&mut controller) == (29, 18) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (29, 18));
    for _ in 0..4 {
        controller.press(GameButton::Up).expect("try reverse ledge");
    }
    assert_eq!(position(&mut controller), (29, 18));
    // The uphill return remains playable after the one-way drop.
    for at in [(41, 20), (41, 19), (41, 18), (41, 17), (29, 15)] {
        grid.walk(&mut controller, at);
    }
    for (tile, text) in [
        ((41, 21), "ramp climbs"),
        ((23, 35), "fence keeps"),
        ((52, 16), "NELSON FARM"),
        ((16, 20), "LIMESTONE"),
        ((4, 26), "ROCHESTER"),
        ((32, 14), "ONE WAY"),
    ] {
        assert!(
            grid.talk(&mut controller, tile).contains(text),
            "interaction {tile:?}"
        );
    }
    for (tile, item, event) in [
        ((19, 11), "POTION", "EVENT_MN_R01_ITEM_01"),
        ((13, 35), "POKE_BALL", "EVENT_MN_R01_ITEM_02"),
    ] {
        collect_original_pickup(&grid, &mut controller, &runtime, tile, item, event);
    }
    retained_four_hops(&grid, &mut controller);
    // Closed dock end and both rail faces stop the player at their real walls.
    for (at, button) in [
        ((11, 35), GameButton::Left),
        ((16, 33), GameButton::Up),
        ((16, 36), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        for _ in 0..3 {
            controller.press(button).expect("try native rail");
        }
        assert_eq!(position(&mut controller), at);
    }
    grid.walk(&mut controller, (18, 17));
    assert_eq!(
        controller
            .snapshot()
            .expect("cave entry")
            .overworld
            .map_name,
        "MnR01Cave1F"
    );
    for target in [(5, 6), (5, 4), (11, 4), (11, 6)] {
        indoor_walk(&mut controller, &runtime, target);
    }
    controller.press(GameButton::Up).expect("face cave cache");
    controller.press(GameButton::A).expect("cave cache");
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (5, 6));
    controller.press(GameButton::Up).expect("face guide");
    controller.press(GameButton::A).expect("guide");
    assert!(
        controller
            .snapshot()
            .expect("guide text")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("Two small hollows"))
    );
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (9, 11));
    assert_eq!(
        controller
            .snapshot()
            .expect("cave return")
            .overworld
            .map_name,
        "MnR01C00R00"
    );
    assert_eq!(position(&mut controller), (18, 18));
    for exit in [(4, 9), (5, 9)] {
        grid.walk(&mut controller, (49, 15));
        assert_eq!(
            controller
                .snapshot()
                .expect("farm entry")
                .overworld
                .map_name,
            "MnR01Farmhouse"
        );
        if exit == (4, 9) {
            indoor_walk(&mut controller, &runtime, (4, 3));
            controller.press(GameButton::Right).expect("face Nelson");
            controller.press(GameButton::A).expect("tutorial");
            assert!(
                controller
                    .snapshot()
                    .expect("tutorial text")
                    .ui
                    .text
                    .and_then(|t| t.asm_text)
                    .is_some_and(|t| t.contains("Wild partners"))
            );
            finish_text(&mut controller);
            indoor_walk(&mut controller, &runtime, (4, 1));
            controller.press(GameButton::Up).expect("face notice");
            controller.press(GameButton::A).expect("notice");
            assert!(
                controller
                    .snapshot()
                    .expect("notice text")
                    .ui
                    .text
                    .and_then(|t| t.asm_text)
                    .is_some_and(|t| t.contains("TAMARACK"))
            );
            finish_text(&mut controller);
            indoor_walk(&mut controller, &runtime, (8, 8));
            controller.press(GameButton::Up).expect("face Berry");
            controller.press(GameButton::A).expect("Berry");
            finish_text(&mut controller);
        }
        indoor_walk(&mut controller, &runtime, exit);
        for _ in 0..4 {
            if controller
                .snapshot()
                .expect("carpet departure")
                .overworld
                .map_name
                != "MnR01Farmhouse"
            {
                break;
            }
            controller.press(GameButton::Down).expect("leave carpet");
        }
        assert_eq!(
            controller
                .snapshot()
                .expect("farm return")
                .overworld
                .map_name,
            "MnR01C00R00"
        );
        assert_eq!(position(&mut controller), (49, 16));
    }
    for flag in [
        "EVENT_MN_R01_ITEM_01",
        "EVENT_MN_R01_ITEM_02",
        "EVENT_MN_R01_ITEM_03",
        "EVENT_MN_R01_ITEM_04",
    ] {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("pickup flag")
        );
    }
    grid.walk(&mut controller, (52, 24));
    controller.press(GameButton::Start).expect("Start");
    assert!(controller.snapshot().expect("menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close menu");
    if let Some(path) = std::env::var_os("CRYSTAL_DRIFTLESS_FIXTURE_SAVE") {
        controller
            .save(PathBuf::from(path))
            .expect("geometry fixture save");
    }
}

fn collect_original_pickup(
    grid: &RouteGrid,
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    tile: (i16, i16),
    item: &str,
    event: &str,
) {
    let quantity = |c: &VisibleShellController| {
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items[item])
    };
    let collected = |c: &VisibleShellController| {
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(event)
            .expect("original native pickup event")
    };
    assert!(!collected(c));
    let before = quantity(c);
    let (approach, direction) = neighbors(tile)
        .into_iter()
        .find(|(p, _)| {
            grid.floor.contains(p) && !grid.blocked.contains(p) && !grid.hazards.contains(p)
        })
        .expect("native original pickup approach");
    grid.walk(c, approach);
    c.press(opposite(direction)).expect("face original item");
    assert_eq!(position(c), approach, "uncollected item owns its tile");
    let history_start = c.shell.dialogue_log_events.len();
    c.press(GameButton::A)
        .expect("collect original item through production A");
    let name = runtime.data().items[item].name.replace('_', " ");
    let found = format!("CHRIS found\n{name}!");
    let notice = c
        .shell
        .visible_field_item_notice
        .as_ref()
        .expect("native retained successful item notice");
    assert_eq!(
        notice.sound_trigger_text, found,
        "original Found/name payload"
    );
    assert_eq!(
        notice.phase,
        super::super::VisibleFieldItemPhase::PocketText
    );
    assert!(matches!(
        notice.presentation,
        super::super::VisibleFieldItemPresentation::ItemBall
    ));
    // event=show is printer provenance, not evidence of completed Found glyphs.
    let shown = format!("text={found:?}");
    assert_eq!(
        c.shell
            .dialogue_log_events
            .iter()
            .skip(history_start)
            .filter(|line| line.contains("event=show ") && line.contains(&shown))
            .count(),
        1,
        "new original Found/name presentation: {:?}",
        c.shell.dialogue_log_events
    );
    assert!(collected(c));
    assert_eq!(quantity(c), before + 1);
    let mut pages = BTreeMap::new();
    for _ in 0..32 {
        if let Some((index, page)) = completed_visible_page(c) {
            pages.insert(index, page);
            c.press(GameButton::Down)
                .expect("printed reward movement lock");
            assert_eq!(position(c), approach);
            c.press(GameButton::A)
                .expect("acknowledge printed reward page");
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        c.wait_frames(240).expect("native current page printer");
    }
    assert_eq!(pages.get(&0), Some(&format!("CHRIS put the\n{name} in")));
    let pocket = if runtime.data().items[item].pocket == "BALL" {
        "BALL POCKET"
    } else {
        "ITEM POCKET"
    };
    assert_eq!(pages.get(&1), Some(&format!("{name} in\nthe {pocket}.")));
    finish_text(c);
    assert_eq!(quantity(c), before + 1);
    // Same facing, same original item tile, actual repeated A: no second grant.
    c.press(GameButton::A)
        .expect("retry collected original item");
    finish_text(c);
    assert_eq!(quantity(c), before + 1);
    assert!(collected(c));
    assert_eq!(
        c.shell
            .dialogue_log_events
            .iter()
            .skip(history_start)
            .filter(|line| line.contains("event=show ") && line.contains(&shown))
            .count(),
        1
    );
}

#[derive(Deserialize)]
struct RetainedHop {
    upper: [i16; 2],
    hop: [i16; 2],
    landing: [i16; 2],
    reverse: [i16; 2],
    ordinary_return_steps: usize,
    ordinary_return_path: Vec<[i16; 2]>,
}

fn retained_four_hops(grid: &RouteGrid, c: &mut VisibleShellController) {
    #[derive(Deserialize)]
    struct Plan {
        retained_four_hops: Vec<RetainedHop>,
    }
    let path = std::env::var_os("CRYSTAL_DRIFTLESS_LEDGE_PLAN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../planning/minnesota/renders/route-quality-next/r01/v05-cedar-ridge-c4816242/controller-input-plan.json",
        ));
    let plan: Plan = serde_json::from_str(
        &std::fs::read_to_string(path).expect("frozen R01 four-lane input plan"),
    )
    .expect("retained four native hops");
    assert_eq!(plan.retained_four_hops.len(), 4);
    let original = c.snapshot().expect("before retained native hop courses");
    let flags = c.shell.shell.session().state().flags.clone();
    let bag = c.shell.shell.session().state().bag.clone();
    for (lane, expected_steps) in plan.retained_four_hops.iter().zip([45, 43, 43, 43]) {
        let point = |p: [i16; 2]| (p[0], p[1]);
        assert_eq!(lane.ordinary_return_steps, expected_steps);
        assert_eq!(lane.ordinary_return_path.len(), expected_steps + 1);
        assert_eq!(lane.ordinary_return_path.first(), Some(&lane.landing));
        assert_eq!(lane.ordinary_return_path.last(), Some(&lane.upper));
        grid.walk(c, point(lane.upper));
        exact_native_step(c, GameButton::Down, point(lane.hop));
        exact_native_step(c, GameButton::Down, point(lane.landing));
        for _ in 0..3 {
            c.press(GameButton::Up).expect("native reverse hop denial");
            assert_eq!(
                position(c),
                point(lane.landing),
                "reverse must not enter native ledge wall {:?}",
                lane.reverse
            );
        }
        // Every declared adjacent ordinary step is applied directly; there is
        // no pathfinder between these inputs and no shortcut across the ridge.
        for pair in lane.ordinary_return_path.windows(2) {
            let from = point(pair[0]);
            let to = point(pair[1]);
            assert_eq!(position(c), from);
            let direction = match (to.0 - from.0, to.1 - from.1) {
                (-1, 0) => GameButton::Left,
                (1, 0) => GameButton::Right,
                (0, -1) => GameButton::Up,
                (0, 1) => GameButton::Down,
                delta => panic!("non-adjacent ordinary return {from:?}→{to:?}: {delta:?}"),
            };
            exact_native_step(c, direction, to);
            assert_eq!(
                c.snapshot()
                    .expect("ordinary return map")
                    .overworld
                    .map_name,
                original.overworld.map_name
            );
        }
        assert_eq!(position(c), point(lane.upper));
        assert_eq!(c.shell.shell.session().state().flags, flags);
        assert_eq!(c.shell.shell.session().state().bag, bag);
    }
}

fn exact_native_step(c: &mut VisibleShellController, button: GameButton, to: (i16, i16)) {
    let from = position(c);
    for _ in 0..4 {
        c.press(button)
            .expect("actual directional return or hop input");
        let now = position(c);
        if now == to {
            return;
        }
        assert_eq!(
            now, from,
            "unexpected native step {from:?}→{to:?}, button={button:?}"
        );
    }
    panic!("native step remained blocked {from:?}→{to:?}, button={button:?}");
}
