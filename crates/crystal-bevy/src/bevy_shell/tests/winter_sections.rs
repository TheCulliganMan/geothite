//! Explicit empty-party geometry fixtures, followed only by production input.
//! Local placement/old trainer flags do not prove earned regional progression.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::systems::script_flags::{ScriptFlagCommand, apply_script_flag_mutation};
use crystal_core::world::map::Direction;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, indoor_walk, position};
use super::north_shore::{climb_added_cascade, mount_surf, settle_water, water_step};
use super::r10_gallery::{completed_visible_page, continue_save, flags, press, settle};

type Point = [u16; 2];
const CONTRACT: &str = include_str!(
    "../../../../../planning/minnesota/regions/ice/sections/source-v03/integration.contract.json"
);
const PICKUP_EVENT: &str = "EVENT_MN_R11_ITEM_02";

fn contract() -> &'static Value {
    static PLAN: OnceLock<Value> = OnceLock::new();
    PLAN.get_or_init(|| serde_json::from_str(CONTRACT).expect("frozen winter-section contract"))
}

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_WINTER_TEST_PACK")
                    .expect("actual completed winter sections pack"),
            )
            .canonicalize()
            .expect("completed external pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict native winter pack"),
            )
            .expect("native winter runtime");
            (root, runtime)
        })
        .clone()
}

fn point(value: &Value) -> Point {
    [
        u16::try_from(value[0].as_u64().expect("x")).expect("u16 x"),
        u16::try_from(value[1].as_u64().expect("y")).expect("u16 y"),
    ]
}

fn input(value: &Value) -> GameButton {
    match (
        value[0].as_i64().expect("dx"),
        value[1].as_i64().expect("dy"),
    ) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("non-cardinal authored input: {other:?}"),
    }
}

fn opposite(button: GameButton) -> GameButton {
    match button {
        GameButton::Left => GameButton::Right,
        GameButton::Right => GameButton::Left,
        GameButton::Up => GameButton::Down,
        GameButton::Down => GameButton::Up,
        _ => unreachable!("only cardinal winter-course inputs"),
    }
}

fn resolve(route: &Value, core: Point) -> (String, Point) {
    let column = core[0] / 64;
    let row = core[1] / 48;
    let source = route["native_modules"]
        .as_array()
        .expect("original chunks")
        .iter()
        .find(|v| v["column"] == column && v["row"] == row)
        .expect("core inside original source chunk");
    (
        source["map"].as_str().expect("native map").into(),
        [core[0] % 64, core[1] % 48],
    )
}

fn at(c: &mut VisibleShellController, route: &Value) -> Point {
    let snapshot = c.snapshot().expect("authoritative winter location");
    let source = route["native_modules"]
        .as_array()
        .expect("chunks")
        .iter()
        .find(|v| v["map"] == snapshot.overworld.map_name)
        .expect("no hidden map/warp transition");
    [
        u16::try_from(snapshot.overworld.tile.x).expect("nonnegative x")
            + u16::try_from(source["column"].as_u64().expect("column")).expect("column u16") * 64,
        u16::try_from(snapshot.overworld.tile.y).expect("nonnegative y")
            + u16::try_from(source["row"].as_u64().expect("row")).expect("row u16") * 48,
    ]
}

fn collision(runtime: &CrystalRuntime, route: &Value, core: Point) -> String {
    let (name, local) = resolve(route, core);
    let map = &runtime.data().maps[&name];
    let block = map.blocks
        [usize::from(local[1] / 2) * usize::from(map.attributes.width) + usize::from(local[0] / 2)];
    runtime.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
        [usize::from(local[1] % 2 * 2 + local[0] % 2)]
    .clone()
}

fn fixture(route: &Value, core: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    fixture_with_moves(route, core, &[], false, false)
}

fn fixture_with_moves(
    route: &Value,
    core: Point,
    moves: &[&str],
    hive: bool,
    rising: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let (name, local) = resolve(route, core);
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: name,
            tile_x: local[0] as i16,
            tile_y: local[1] as i16,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit legal Floor/grass winter fixture");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture trainer name");
    let state = c.shell.shell.session_mut().state_mut();
    assert_eq!(
        state.storage.party.filled_slots(),
        0,
        "explicit empty-party geometry fixture"
    );
    // Initial defeated trainer flags keep geometry tests out of old battles.
    // Real populated resource/trainer journeys remain separate retained tests.
    for (prefix, count) in [("R17", 6), ("R11", 10)] {
        for n in 1..=count {
            apply_script_flag_mutation(
                state,
                ScriptFlagCommand {
                    command: "setevent".into(),
                    flag_id: format!("EVENT_MN_{prefix}_TRAINER_{n:02}"),
                    source_script: "WinterSectionsExplicitInitialFixture".into(),
                    command_index: n,
                },
            )
            .expect("registered original trainer event");
        }
    }
    if !moves.is_empty() {
        let mut partner = runtime
            .data()
            .create_pokemon("MEW", 60, Dv::default())
            .expect("native HM-compatible fixture partner");
        for name in moves {
            assert!(
                partner
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|learned| learned.as_str() == *name)
            );
        }
        partner.moves = moves
            .iter()
            .map(|name| LearnedMove {
                name: (*name).into(),
                current_pp: runtime.data().moves[*name].pp,
                pp_ups: 0,
            })
            .collect();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
    }
    for (index, (flag, enabled)) in [
        ("ENGINE_HIVEBADGE", hive),
        ("ENGINE_FOGBADGE", moves.contains(&"SURF")),
        ("ENGINE_RISINGBADGE", rising),
    ]
    .into_iter()
    .enumerate()
    {
        apply_script_flag_mutation(
            state,
            ScriptFlagCommand {
                command: if enabled { "setflag" } else { "clearflag" }.into(),
                flag_id: flag.into(),
                source_script: "WinterSectionsExplicitInitialFixture".into(),
                command_index: 32 + index,
            },
        )
        .expect("consistent native badge fixture");
    }
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Setup ends here: empty party or declared legal Mew60/HM/badges; no subsequent state/party/position/flag/RNG writes.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(at(&mut c, route), core);
    assert!(matches!(
        collision(&runtime, route, core).as_str(),
        "FLOOR" | "TALL_GRASS"
    ));
    (root, runtime, c)
}

fn slide(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    route: &Value,
    direction: GameButton,
    from: Point,
    stop: Point,
    length: usize,
) -> bool {
    assert_eq!(at(c, route), from);
    for _ in 0..4 {
        c.press(direction)
            .expect("turn/enter native sliding surface");
        c.wait_frames(1).expect("observe actual slide frame");
        if at(c, route) != from {
            break;
        }
    }
    let entered = at(c, route);
    assert_ne!(entered, from, "native ice entry must move");
    let observed_mid_slide = entered != stop;
    if observed_mid_slide {
        assert_eq!(collision(runtime, route, entered), "ICE");
        c.press(opposite(direction))
            .expect("opposing input during native slide");
        let now = at(c, route);
        match direction {
            GameButton::Right => {
                assert_eq!(now[1], entered[1]);
                assert!(now[0] >= entered[0]);
            }
            GameButton::Left => {
                assert_eq!(now[1], entered[1]);
                assert!(now[0] <= entered[0]);
            }
            GameButton::Down => {
                assert_eq!(now[0], entered[0]);
                assert!(now[1] >= entered[1]);
            }
            GameButton::Up => {
                assert_eq!(now[0], entered[0]);
                assert!(now[1] <= entered[1]);
            }
            _ => unreachable!("cardinal slide"),
        }
    }
    c.wait_frames((length + 4) * 24)
        .expect("native forced-movement clock to exact stop");
    assert_eq!(at(c, route), stop);
    c.wait_frames(80).expect("actual resting stop");
    assert_eq!(at(c, route), stop);
    let snapshot = c.snapshot().expect("stopped winter state");
    assert_eq!(snapshot.phase, RuntimeShellPhase::Overworld);
    assert!(snapshot.battle.is_none());
    assert!(
        snapshot.ui.menu.is_none() && !snapshot.ui.text_window_open && !c.has_pending_script_work()
    );
    assert!(c.shell.last_error.is_none());
    observed_mid_slide
}

fn capture(c: &mut VisibleShellController, id: &str) {
    if let Some(dir) = std::env::var_os("CRYSTAL_WINTER_SECTIONS_CAPTURE_SAVES_DIR") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("fresh capture directory");
        let path = dir.join(format!("{id}.crystalsave"));
        assert!(!path.exists(), "immutable native capture save");
        c.save(path)
            .expect("actual production save at reached native stop");
    }
}

fn all_courses(index: usize) {
    let route = &contract()["routes"][index];
    let mut mid_slides = 0;
    let mut continued_dry = false;
    let mut continued_wall = false;
    for (n, course) in route["slide_courses"]
        .as_array()
        .expect("complete frozen entry inventory")
        .iter()
        .enumerate()
    {
        let from = point(&course["from_core"]);
        let stop = point(&course["stop_core"]);
        let direction = input(&course["direction"]);
        let length = course["ice_path_core"]
            .as_array()
            .expect("native ice path")
            .len();
        let (root, runtime, mut c) = fixture(route, from);
        let before_flags = flags(&c);
        let before_bag = c.shell.shell.session().state().bag.clone();
        let before_storage = c.shell.shell.session().state().storage.clone();
        let money = c.shell.shell.session().state().money;
        mid_slides += usize::from(slide(
            &mut c, &runtime, route, direction, from, stop, length,
        ));
        let wall = course["stop_kind"] == "intentional_wall_stop_on_ICE";
        if (!wall && !continued_dry) || (wall && !continued_wall) {
            press(&mut c, GameButton::Start);
            assert!(c.snapshot().expect("production Start").ui.menu.is_some());
            press(&mut c, GameButton::B);
            assert_eq!(at(&mut c, route), stop);
            capture(
                &mut c,
                &format!(
                    "{}-course-{n:02}-{}",
                    route["id"].as_str().expect("id"),
                    if wall { "wall-stop" } else { "dry-stop" }
                ),
            );
            c = continue_save(root, runtime.clone(), &mut c);
            assert_eq!(at(&mut c, route), stop);
            if wall {
                continued_wall = true;
            } else {
                continued_dry = true;
            }
        }
        // Actual reversal from each reached resting state, including wall stops.
        slide(
            &mut c,
            &runtime,
            route,
            opposite(direction),
            stop,
            from,
            length,
        );
        assert_eq!(flags(&c), before_flags);
        assert_eq!(c.shell.shell.session().state().bag, before_bag);
        assert_eq!(c.shell.shell.session().state().storage, before_storage);
        assert_eq!(c.shell.shell.session().state().money, money);
        for turn in course["ordinary_turn_exits"]
            .as_array()
            .expect("wall-stop turns")
        {
            let (_, runtime, mut c) = fixture(route, from);
            let original = flags(&c);
            slide(&mut c, &runtime, route, direction, from, stop, length);
            slide(
                &mut c,
                &runtime,
                route,
                input(&turn["direction"]),
                stop,
                point(&turn["dry_stop_core"]),
                turn["ice_path_core"]
                    .as_array()
                    .expect("turn ice path")
                    .len(),
            );
            assert_eq!(flags(&c), original);
        }
    }
    assert!(
        mid_slides > 0,
        "observe actual opposing-input rejection while still sliding"
    );
    assert!(continued_dry);
    if index == 1 {
        assert!(continued_wall);
    }
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_TEST_PACK; local geometry fixture does not earn access"]
fn winter_sections_controller_r17_all56_entries_stops_returns_and_continue() {
    all_courses(0);
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_TEST_PACK; local geometry fixture does not earn access"]
fn winter_sections_controller_r11_all26_entries_six_wall_stops_turns_and_continue() {
    all_courses(1);
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_TEST_PACK; native old reward, not fixture-added inventory"]
fn winter_sections_controller_original_ultra_ball_printed_pocket_once_and_continue() {
    let route = &contract()["routes"][1];
    let (root, runtime, mut c) = fixture(route, [48, 33]);
    let item = &runtime.data().items["ULTRA_BALL"];
    let object = runtime.data().maps["MnR11C00R00"]
        .objects
        .iter()
        .find(|o| o.x == 49 && o.y == 33)
        .expect("original northern shore reward");
    assert_eq!(object.script, "MN_R11_ITEM_02");
    assert_eq!(object.event_flag, PICKUP_EVENT);
    let quantity = |c: &VisibleShellController| c.shell.shell.session().state().bag.quantity(item);
    let before = quantity(&c);
    let initial_flags = flags(&c);
    let money = c.shell.shell.session().state().money;
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(PICKUP_EVENT)
            .expect("native pickup event")
    );
    press(&mut c, GameButton::Right);
    assert_eq!(at(&mut c, route), [48, 33]);
    let history = c.shell.dialogue_log_events.len();
    c.press(GameButton::A).expect("native original pickup A");
    let name = item.name.replace('_', " ");
    let found = format!("CHRIS found\n{name}!");
    let notice = c
        .shell
        .visible_field_item_notice
        .as_ref()
        .expect("native retained successful field item notice");
    assert_eq!(notice.sound_trigger_text, found);
    assert_eq!(
        notice.phase,
        super::super::VisibleFieldItemPhase::PocketText
    );
    let shown = format!("text={found:?}");
    assert_eq!(
        c.shell
            .dialogue_log_events
            .iter()
            .skip(history)
            .filter(|line| line.contains("event=show ") && line.contains(&shown))
            .count(),
        1,
        "printer provenance for exact original Found/name payload, not a completed-glyph claim"
    );
    assert_eq!(quantity(&c), before + 1);
    let mut pages = BTreeMap::new();
    for _ in 0..32 {
        if let Some((index, page)) = completed_visible_page(&mut c) {
            pages.insert(index, page);
            c.press(GameButton::Up)
                .expect("printed Pocket page owns movement");
            assert_eq!(at(&mut c, route), [48, 33]);
            c.press(GameButton::A)
                .expect("acknowledge actual printed page");
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        c.wait_frames(240)
            .expect("native current Pocket page printer");
    }
    assert_eq!(pages.get(&0), Some(&format!("CHRIS put the\n{name} in")));
    assert_eq!(pages.get(&1), Some(&format!("{name} in\nthe BALL POCKET.")));
    settle(&mut c, "original shore reward releases input");
    press(&mut c, GameButton::A);
    settle(&mut c, "repeat collected reward");
    assert_eq!(quantity(&c), before + 1);
    let mut expected_flags = initial_flags;
    expected_flags.0.insert(PICKUP_EVENT.into());
    assert_eq!(flags(&c), expected_flags);
    for (button, to) in [(GameButton::Right, [49, 33]), (GameButton::Left, [48, 33])] {
        for _ in 0..4 {
            if at(&mut c, route) == to {
                break;
            }
            press(&mut c, button);
        }
        assert_eq!(at(&mut c, route), to);
    }
    capture(&mut c, "R11-original-ultra-ball-collected");
    c = continue_save(root, runtime.clone(), &mut c);
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    settle(&mut c, "continued collected reward stays gone");
    assert_eq!(quantity(&c), before + 1);
    assert_eq!(flags(&c), expected_flags);
    assert_eq!(c.shell.shell.session().state().money, money);
}

fn cedar_tree(c: &VisibleShellController) -> u16 {
    c.shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(18, 10)
        .expect("current original cedar Cut metatile")
}

fn cedar_grid(runtime: &CrystalRuntime, c: &mut VisibleShellController) -> RouteGrid {
    let mut grid = RouteGrid::new(
        runtime,
        RouteSpec {
            prefix: "MnR17",
            columns: 1,
            rows: 3,
            tileset: "mn_winter_outdoor",
            start: position(c),
            doors: BTreeMap::from([((10, 43), "MnR17Cave1F"), ((45, 49), "MnR17RangerHut")]),
        },
    );
    // Planner-only reflection of an actual native override; never changes game state.
    if cedar_tree(c) == 0x17 {
        let tokens = &runtime.data().tilesets["mn_winter_outdoor"].collision["17"];
        for dy in 0..2 {
            for dx in 0..2 {
                if tokens[dy * 2 + dx] == "FLOOR" {
                    grid.floor.insert((36 + dx as i16, 68 + dy as i16));
                }
            }
        }
    }
    grid
}

fn blocked(c: &mut VisibleShellController, route: &Value, direction: GameButton) {
    let before = at(c, route);
    for _ in 0..3 {
        press(c, direction);
        assert_eq!(at(c, route), before);
    }
}

fn native_cut(c: &mut VisibleShellController, route: &Value, direction: GameButton) {
    blocked(c, route, direction);
    for _ in 0..96 {
        if cedar_tree(c) == 0x17 {
            settle(c, "native Cut releases input");
            c.wait_frames(60).expect("Cut presentation");
            assert!(c.shell.flygon_field_actions.contains("cut"));
            return;
        }
        press(c, GameButton::A);
    }
    panic!("actual native winter Cut did not apply forest0f→17");
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_TEST_PACK; explicit HM/badge fixtures do not earn access"]
fn winter_sections_controller_native_cedar_cut_denial66_step_return6_step_open_and_reloads() {
    let route = &contract()["routes"][0];
    for (moves, hive) in [(["SWIFT"], true), (["CUT"], false)] {
        let (_, runtime, mut c) = fixture_with_moves(route, [36, 68], &moves, hive, false);
        let original = flags(&c);
        let bag = c.shell.shell.session().state().bag.clone();
        let before = c.snapshot().expect("denial partner").party;
        assert_eq!(cedar_tree(&c), 0x0f);
        blocked(&mut c, route, GameButton::Down);
        c.press(GameButton::A).expect("actual denied Cut");
        c.wait_frames(240)
            .expect("fully print tree/Cut explanation");
        let page =
            completed_visible_page(&mut c).expect("current completed native Cut denial page");
        assert!(page.1.contains("CUT"));
        c.press(GameButton::Down).expect("denial owns movement");
        assert_eq!(at(&mut c, route), [36, 68]);
        settle(&mut c, "denied Cut closes");
        assert_eq!(cedar_tree(&c), 0x0f);
        assert_eq!(flags(&c), original);
        assert_eq!(c.shell.shell.session().state().bag, bag);
        assert_eq!(
            c.snapshot().expect("denial preserved partner").party,
            before
        );
        assert_eq!(
            cedar_grid(&runtime, &mut c).path((36, 67), (36, 73)).len(),
            66
        );
    }
    let (root, runtime, mut c) = fixture_with_moves(route, [36, 67], &["CUT"], true, false);
    let original = flags(&c);
    let bag = c.shell.shell.session().state().bag.clone();
    let money = c.shell.shell.session().state().money;
    let grid = cedar_grid(&runtime, &mut c);
    assert_eq!(grid.path((36, 67), (36, 73)).len(), 66);
    grid.walk(&mut c, (36, 73));
    grid.walk(&mut c, (36, 67));
    grid.walk(&mut c, (36, 68));
    // Actual permanent east tree and west bank deny a walk-around at the original tree.
    blocked(&mut c, route, GameButton::Left);
    // Side denials change facing. Use the existing input-only walker so a
    // turn-first press is followed by the actual adjacent step.
    grid.walk(&mut c, (37, 68));
    assert_eq!(at(&mut c, route), [37, 68]);
    blocked(&mut c, route, GameButton::Down);
    grid.walk(&mut c, (36, 68));
    assert_eq!(at(&mut c, route), [36, 68]);
    native_cut(&mut c, route, GameButton::Down);
    let opened = cedar_grid(&runtime, &mut c);
    assert_eq!(opened.path((36, 67), (36, 73)).len(), 6);
    opened.walk(&mut c, (36, 67));
    opened.walk(&mut c, (36, 73));
    assert_eq!(flags(&c), original);
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    assert_eq!(
        cedar_tree(&c),
        0x0f,
        "native TILES reload resets vegetation on Continue"
    );
    let grid = cedar_grid(&runtime, &mut c);
    grid.walk(&mut c, (36, 70));
    native_cut(&mut c, route, GameButton::Up);
    let opened = cedar_grid(&runtime, &mut c);
    opened.walk(&mut c, (36, 67));
    opened.walk(&mut c, (45, 49));
    assert_eq!(
        c.snapshot()
            .expect("actual ranger entry")
            .overworld
            .map_name,
        "MnR17RangerHut"
    );
    indoor_walk(&mut c, &runtime, (5, 6));
    for _ in 0..8 {
        press(&mut c, GameButton::Down);
        if c.snapshot()
            .expect("actual paired return")
            .overworld
            .map_name
            == "MnR17C00R01"
        {
            break;
        }
    }
    assert_eq!(at(&mut c, route), [45, 50]);
    assert_eq!(
        cedar_tree(&c),
        0x0f,
        "ordinary room return reloads original vegetation"
    );
    assert_eq!(flags(&c), original);
    assert_eq!(c.shell.shell.session().state().bag, bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    capture(&mut c, "R17-cut-reset-after-native-hut-return");
}

#[test]
#[ignore = "requires completed CRYSTAL_WINTER_TEST_PACK; explicit Surf/Waterfall/badges are fixture only"]
fn winter_sections_controller_upper_gorge_native_waterfall_denial_ascent_current_bridge_and_continue()
 {
    let route = &contract()["routes"][1];
    for (moves, rising) in [(vec!["SURF"], true), (vec!["SURF", "WATERFALL"], false)] {
        let (_, runtime, mut c) = fixture_with_moves(route, [22, 27], &moves, false, rising);
        mount_surf(&mut c, GameButton::Left, (21, 27));
        for x in (18..21).rev() {
            water_step(&mut c, GameButton::Left, (x, 27));
        }
        water_step(&mut c, GameButton::Up, (18, 26));
        if c.snapshot()
            .expect("current native uphill facing")
            .overworld
            .facing
            != Direction::Up
        {
            c.press(GameButton::Up).expect("face waterfall");
            settle_water(&mut c);
        }
        assert_eq!(at(&mut c, route), [18, 26]);
        let before = c.snapshot().expect("native denial preimage");
        let expected = before
            .presentation
            .asm_text_for_label(".HugeWaterfallText@Script_CantDoWaterfall")
            .expect("native refusal")
            .to_owned();
        c.press(GameButton::A)
            .expect("unqualified native Waterfall A");
        assert_eq!(c.shell.field_notice.as_deref(), Some(expected.as_str()));
        c.press(GameButton::Down)
            .expect("denial text owns movement");
        assert_eq!(at(&mut c, route), [18, 26]);
        assert!(!c.shell.flygon_field_actions.contains("waterfall"));
        assert_eq!(c.snapshot().expect("denied partner").party, before.party);
        settle_water(&mut c);
        assert_eq!(collision(&runtime, route, [18, 25]), "WATERFALL");
    }
    let (root, runtime, mut c) =
        fixture_with_moves(route, [22, 27], &["SURF", "WATERFALL"], false, true);
    let original = flags(&c);
    let bag = c.shell.shell.session().state().bag.clone();
    mount_surf(&mut c, GameButton::Left, (21, 27));
    for x in (18..21).rev() {
        water_step(&mut c, GameButton::Left, (x, 27));
    }
    water_step(&mut c, GameButton::Up, (18, 26));
    climb_added_cascade(&mut c, 18, 26, 19);
    assert_eq!(collision(&runtime, route, [18, 19]), "WATER");
    assert_eq!(
        c.snapshot().expect("native ascent Surf").overworld.mode,
        MovementMode::Surf
    );
    for _ in 0..64 {
        if at(&mut c, route) == [18, 26] {
            break;
        }
        c.press(GameButton::Down)
            .expect("retained actual downward current");
        c.wait_frames(16).expect("cascade source steps");
        settle_water(&mut c);
    }
    assert_eq!(at(&mut c, route), [18, 26]);
    water_step(&mut c, GameButton::Down, (18, 27));
    for x in 19..=22 {
        water_step(&mut c, GameButton::Right, (x, 27));
    }
    assert_eq!(
        c.snapshot()
            .expect("native winter bank dismount")
            .overworld
            .mode,
        MovementMode::Normal
    );
    // Existing complete upper tributary bridge: native40/41,40/41 LADDER deck, Water alongside.
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR11",
            columns: 2,
            rows: 4,
            tileset: "mn_winter_north_shore",
            start: (22, 27),
            doors: BTreeMap::new(),
        },
    );
    for target in [
        (40, 39),
        (40, 40),
        (40, 41),
        (40, 42),
        (40, 41),
        (40, 40),
        (40, 39),
    ] {
        grid.walk(&mut c, target);
    }
    assert_eq!(flags(&c), original);
    assert_eq!(c.shell.shell.session().state().bag, bag);
    capture(&mut c, "R11-native-open-upper-gorge-after-climb-return");
    c = continue_save(root, runtime.clone(), &mut c);
    assert_eq!(at(&mut c, route), [40, 39]);
    assert_eq!(flags(&c), original);
}
