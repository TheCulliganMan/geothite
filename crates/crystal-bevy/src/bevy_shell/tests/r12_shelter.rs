//! Explicit local R12 fixtures, then only production controller inputs.
//! Initial low PP is setup, not earned battles or regional SQ03 progress.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk, neighbors};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};

const ROOM: &str = "MnR12Cave1F";
const OUTSIDE: &str = "MnR12C01R00";
const EVENT: &str = "EVENT_MN_R12_CAVE_ETHER";
fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_R12_SHELTER_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../planning/minnesota/caves/MnR12Cave1F.candidate.json")
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("same frozen source as publication"))
        .expect("R12 source")
}
fn point(v: &Value) -> Point {
    (
        v[0].as_i64().expect("core x") as i16,
        v[1].as_i64().expect("core y") as i16,
    )
}
fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R12_SHELTER_CONTROLLER_PACK")
                    .expect("actual revised R12 native pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict revised pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum PpSetup {
    Empty,
    OneLowMove,
    NativeItemReview,
}
fn fixture(
    map_name: &str,
    at: Point,
    pp_setup: PpSetup,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: map_name.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local shelter setup");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    if pp_setup != PpSetup::Empty {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 30, Dv::default())
            .expect("native legal learned moves");
        assert!(!pokemon.moves.is_empty());
        let max = runtime.data().moves[&pokemon.moves[0].name].pp;
        assert!(max > 1);
        if pp_setup == PpSetup::NativeItemReview {
            assert!(pokemon.moves.len() >= 2);
            for learned in &mut pokemon.moves {
                learned.current_pp = 1;
            }
        } else {
            pokemon.moves[0].current_pp = 1;
        }
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
    }
    let state = c.shell.shell.session_mut().state_mut();
    if pp_setup == PpSetup::NativeItemReview {
        for id in ["ETHER", "MAX_ETHER", "ELIXER", "MAX_ELIXER"] {
            assert!(
                state
                    .bag
                    .add_item(&runtime.data().items[id], 2)
                    .expect("initial native PP item review inventory")
            );
        }
    }
    state.money = 1000;
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // No party/move/bag/flag/position/RNG writes after this fixture boundary.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(tile(&mut c), at);
    assert!(!collected(&c));
    let s = source();
    let m = &runtime.data().maps[ROOM];
    assert_eq!([m.attributes.width, m.attributes.height], [16, 12]);
    let blocks = s["rows"]
        .as_array()
        .expect("frozen rows")
        .iter()
        .flat_map(|r| r.as_str().expect("row").split_whitespace())
        .map(|t| u16::from_str_radix(t, 16).expect("cave block"))
        .collect::<Vec<_>>();
    assert_eq!(
        m.blocks, blocks,
        "actual installed geometry must match this frozen candidate"
    );
    (root, runtime, c)
}
fn collected(c: &VisibleShellController) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(EVENT)
        .expect("original cache event")
}
fn quantity(c: &VisibleShellController, runtime: &CrystalRuntime) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items["MAX_ETHER"])
}
fn kind(runtime: &CrystalRuntime, p: Point) -> &str {
    let m = &runtime.data().maps[ROOM];
    let b = m.blocks[p.1 as usize / 2 * 16 + p.0 as usize / 2];
    &runtime.data().tilesets["cave"].collision[&format!("{b:02x}")]
        [p.1 as usize % 2 * 2 + p.0 as usize % 2]
}
fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .into_iter()
        .find_map(|(p, b)| (p == to).then_some(b))
        .expect("one adjacent native step")
}
fn step(c: &mut VisibleShellController, b: GameButton, to: Point) {
    for _ in 0..8 {
        if tile(c) == to {
            return;
        }
        press(c, b);
    }
    assert_eq!(tile(c), to);
}
fn blocked(c: &mut VisibleShellController, b: GameButton) {
    let before = tile(c);
    for _ in 0..4 {
        press(c, b);
        assert_eq!(tile(c), before);
    }
}
fn outside(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR12",
            columns: 2,
            rows: 3,
            tileset: "mn_gunflint",
            start: (66, 24),
            doors: BTreeMap::from([((66, 23), ROOM), ((85, 83), "MnR12RangerHut")]),
        },
    )
}
fn enter(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    outside(runtime).walk(c, (66, 23));
    assert_eq!(map(c), ROOM);
    assert_eq!(tile(c), (17, 23));
    indoor_walk(c, runtime, (17, 22));
    assert_eq!(tile(c), (17, 22));
}
fn leave(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (17, 22));
    indoor_walk(c, runtime, (17, 23));
    assert_eq!(map(c), OUTSIDE);
    let mut arrivals = vec![tile(c)];
    // The data still targets active CAVE2,23. Different native presentation
    // stages can expose that tile or its automatic downward Floor step.
    for _ in 0..8 {
        let at = tile(c);
        assert!(
            at == (2, 23) || at == (2, 24),
            "unexpected return: {arrivals:?}"
        );
        if at == (2, 24) {
            break;
        }
        c.wait_frames(8).expect("bounded native cave return frames");
        assert_eq!(
            map(c),
            OUTSIDE,
            "return cannot accidentally reenter: {arrivals:?}"
        );
        arrivals.push(tile(c));
    }
    if tile(c) == (2, 23) {
        assert!(outside(runtime).floor.contains(&(66, 23)));
        step(c, GameButton::Down, (2, 24));
        arrivals.push(tile(c));
    }
    assert_eq!(
        tile(c),
        (2, 24),
        "active warp must permit its clear step: {arrivals:?}"
    );
    assert_eq!(map(c), OUTSIDE);
    c.wait_frames(60)
        .expect("native cave exit stable on safe Floor");
    assert_eq!(map(c), OUTSIDE, "safe Floor must not reenter: {arrivals:?}");
    assert_eq!(tile(c), (2, 24));
    assert!(outside(runtime).floor.contains(&(66, 24)));
}
fn ordinary_path(runtime: &CrystalRuntime, from: Point, to: Point) -> Vec<Point> {
    let m = &runtime.data().maps[ROOM];
    let occupied = m
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(from, from)]);
    let mut todo = VecDeque::from([from]);
    while let Some(at) = todo.pop_front() {
        if at == to {
            break;
        }
        for (next, _) in neighbors(at) {
            if next.0 < 0
                || next.1 < 0
                || next.0 >= 32
                || next.1 >= 24
                || kind(runtime, next) != "FLOOR"
                || occupied.contains(&next)
                || parents.contains_key(&next)
            {
                continue;
            }
            parents.insert(next, at);
            todo.push_back(next);
        }
    }
    assert!(
        parents.contains_key(&to),
        "ordinary ascent {from:?}->{to:?}"
    );
    let mut path = Vec::new();
    let mut p = to;
    while p != from {
        path.push(p);
        p = parents[&p];
    }
    path.reverse();
    path
}

#[test]
#[ignore = "requires actual revised R12 shelter pack and matching frozen source"]
fn r12_shelter_controller_original_mouth_three_stairs_two_hop_lanes_and_ordinary_returns() {
    let (_, runtime, mut c) = fixture(OUTSIDE, (2, 24), PpSetup::Empty);
    enter(&mut c, &runtime);
    let s = source();
    let probes = &s["review_contract"]["controller_probes"];
    let stairs = probes["stairs"].as_array().expect("three authored stairs");
    assert_eq!(stairs.len(), 3);
    for stair in stairs {
        let path = std::iter::once(point(&stair["before"]))
            .chain(
                stair["lane"]
                    .as_array()
                    .expect("native lane")
                    .iter()
                    .map(point),
            )
            .chain(std::iter::once(point(&stair["after"])))
            .collect::<Vec<_>>();
        for &p in &path {
            assert_eq!(kind(&runtime, p), "FLOOR");
            indoor_walk(&mut c, &runtime, p);
            assert_eq!(tile(&mut c), p);
        }
        let from = point(&stair["blocked_from"]);
        let target = point(&stair["blocked_target"]);
        assert_eq!(kind(&runtime, target), "WALL");
        indoor_walk(&mut c, &runtime, from);
        blocked(&mut c, direction(from, target));
        indoor_walk(&mut c, &runtime, *path.last().expect("far landing"));
        for &p in path.iter().rev().skip(1) {
            indoor_walk(&mut c, &runtime, p);
            assert_eq!(tile(&mut c), p);
        }
    }
    for lane in probes["ledge_lanes"].as_array().expect("both hop lanes") {
        let upper = point(&lane["upper_approach"]);
        let from = point(&lane["hop_from"]);
        let landing = point(&lane["landing"]);
        let reverse = point(&lane["reverse_target"]);
        indoor_walk(&mut c, &runtime, upper);
        assert_eq!(kind(&runtime, from), "HOP_DOWN");
        step(&mut c, GameButton::Down, from);
        for _ in 0..8 {
            if tile(&mut c) == landing {
                break;
            }
            press(&mut c, GameButton::Down);
            let actual = tile(&mut c);
            assert_ne!(actual, reverse, "jump must cross the blocking quadrant");
            assert!(actual == from || actual == landing);
        }
        assert_eq!(tile(&mut c), landing);
        blocked(&mut c, GameButton::Up);
        let path = ordinary_path(&runtime, landing, upper);
        assert_eq!(path.len(), if upper.0 == 14 { 21 } else { 19 });
        for p in path {
            let before = tile(&mut c);
            step(&mut c, direction(before, p), p);
            assert_eq!(map(&mut c), ROOM);
        }
        assert_eq!(tile(&mut c), upper);
    }
    for shoulder in probes["ledge_shoulders"]
        .as_array()
        .expect("real shoulders")
    {
        assert_eq!(kind(&runtime, point(shoulder)), "WALL");
    }
    leave(&mut c, &runtime);
    enter(&mut c, &runtime);
    assert!(!collected(&c));
}

fn otta(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (6, 5));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    let at = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), at, "original Otta dialogue owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        c.wait_frames(240).expect("print original Otta page");
        remember_dialogue(c, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(c) {
            assert_eq!(
                c.snapshot()
                    .expect("Otta identity")
                    .ui
                    .text
                    .as_ref()
                    .map(|t| t.label.as_str()),
                Some("MN_R12_OTTAText")
            );
            pages.insert(index, page);
        }
        let s = c.snapshot().expect("Otta owner");
        if !s.ui.text_window_open && !c.has_pending_script_work() {
            break;
        }
        press(c, GameButton::A);
    }
    assert_eq!(
        pages.get(&0).map(String::as_str),
        Some("A dry rock roof\nbeats a wet tent!"),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1).map(String::as_str),
        Some("Two paths curl\nround the stone."),
        "{diagnostics:#?}"
    );
    settle(c, "after original Otta pages");
}
fn collect(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(c, runtime, (26, 5));
    press(c, GameButton::Right);
    assert_eq!(tile(c), (26, 5));
    press(c, GameButton::A);
    assert!(collected(c));
    assert_eq!(quantity(c, runtime), 1);
    let at = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), at, "native item notification owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        c.wait_frames(240).expect("print actual pocket page");
        remember_dialogue(c, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(c) {
            assert_eq!(
                c.shell
                    .visible_field_item_notice
                    .as_ref()
                    .expect("native item phase")
                    .phase,
                super::super::VisibleFieldItemPhase::PocketText
            );
            pages.insert(index, page);
        }
        if c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
            && !c.has_pending_script_work()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let item = runtime.data().items["MAX_ETHER"].name.replace('_', " ");
    assert_eq!(
        pages.get(&0),
        Some(&format!("CHRIS put the\n{item} in")),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1),
        Some(&format!("{item} in\nthe ITEM POCKET.")),
        "{diagnostics:#?}"
    );
    settle(c, "after actual Max Ether pickup pages");
}
fn cursor(
    c: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
    forward: GameButton,
    backward: GameButton,
) {
    for _ in 0..32 {
        let n = index(c);
        if n == target {
            return;
        }
        press(c, if n < target { forward } else { backward });
    }
    panic!("native cursor did not reach {target}");
}
fn pp_observe(
    c: &mut VisibleShellController,
    item: &str,
    stage: &str,
    history: &mut VecDeque<String>,
) {
    let snapshot = c.snapshot().expect("native PP input diagnostic");
    let selected = super::super::selected_field_pack_item_id(&mut c.shell).ok();
    if history.len() == 16 {
        history.pop_front();
    }
    history.push_back(format!(
        "item={item} stage={stage} selected={selected:?} mode={:?} action={:?} party_cursor={} move_cursor={:?} notice={:?} wait={:?} phase={:?} pp={:?} recent={:?}",
        c.shell.field_pack_target_mode,c.shell.field_pack_action_cursor,c.shell.party_cursor,
        c.shell.party_move_cursor,c.shell.field_notice,snapshot.ui.pending_text_wait,snapshot.phase,
        snapshot.party.slots.first().map(|p|&p.pokemon.moves),
        c.shell.last_audio_events.iter().rev().take(6).collect::<Vec<_>>(),
    ));
}
pub(super) fn use_pp_item(
    c: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    item_id: &str,
    move_slot: Option<usize>,
) {
    settle(c, "before native PP item Pack input");
    let mut diagnostics = VecDeque::new();
    pp_observe(c, item_id, "before Start", &mut diagnostics);
    let before = c.snapshot().expect("actual source-created party");
    let item = &runtime.data().items[item_id];
    assert_eq!(
        item.pp_restore_scope.as_deref(),
        Some(if move_slot.is_some() {
            "MOVE"
        } else {
            "POKEMON"
        })
    );
    let carried = c.shell.shell.session().state().bag.quantity(item);
    assert!(carried > 0);
    let mut expected = before.party.clone();
    for (slot, learned) in expected.slots[0].pokemon.moves.iter_mut().enumerate() {
        if move_slot.is_some_and(|selected| selected != slot) {
            continue;
        }
        assert_eq!(
            learned.pp_ups, 0,
            "source-created native fixture has no synthetic PP Ups"
        );
        let max = runtime.data().moves[&learned.name].pp;
        learned.current_pp = match item.pp_restore_points {
            Some(amount) => learned.current_pp.saturating_add(amount).min(max),
            None => max,
        };
    }
    let changed = before.party != expected;
    press(c, GameButton::Start);
    let s = c.snapshot().expect("Start options");
    let target = super::super::visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == super::super::StartMenuOption::Pack)
        .expect("PACK");
    cursor(
        c,
        target,
        |c| {
            c.shell
                .start_menu_cursor
                .as_ref()
                .expect("Start cursor")
                .option_index
        },
        GameButton::Down,
        GameButton::Up,
    );
    press(c, GameButton::A);
    let target = c
        .snapshot()
        .expect("collected pocket")
        .bag
        .items
        .iter()
        .filter(|i| i.quantity > 0)
        .position(|i| i.item_id == item_id)
        .expect("carried native PP item");
    cursor(
        c,
        target,
        |c| {
            c.shell
                .bag_cursor
                .as_ref()
                .expect("Pack cursor")
                .option_index
        },
        GameButton::Down,
        GameButton::Up,
    );
    press(c, GameButton::A);
    pp_observe(c, item_id, "item action menu", &mut diagnostics);
    assert_eq!(
        super::super::selected_field_pack_item_id(&mut c.shell).expect("actual selected item"),
        item_id,
        "{diagnostics:#?}"
    );
    let s = c.snapshot().expect("native item actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &s,
        &c.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("native PP item actions");
    let target = actions
        .iter()
        .position(|a| *a == super::super::FieldPackAction::Use)
        .expect("USE");
    cursor(
        c,
        target,
        |c| {
            c.shell
                .field_pack_action_cursor
                .as_ref()
                .expect("action cursor")
                .option_index
        },
        GameButton::Down,
        GameButton::Up,
    );
    press(c, GameButton::A);
    assert_eq!(
        c.shell.field_pack_target_mode,
        Some(if move_slot.is_some() {
            super::super::FieldPackTargetMode::PartyMove
        } else {
            super::super::FieldPackTargetMode::PartyPokemon
        }),
        "native source scope must select its actual target menu"
    );
    cursor(
        c,
        0,
        |c| c.shell.party_cursor,
        GameButton::Down,
        GameButton::Up,
    );
    if let Some(slot) = move_slot {
        cursor(
            c,
            slot,
            |c| {
                c.shell
                    .party_move_cursor
                    .as_ref()
                    .expect("real learned move cursor")
                    .option_index
            },
            GameButton::Right,
            GameButton::Left,
        );
    }
    c.wait_frames(120)
        .expect("native target presentation before confirmation");
    pp_observe(c, item_id, "before target A", &mut diagnostics);
    assert_eq!(
        super::super::selected_field_pack_item_id(&mut c.shell)
            .expect("selected item at native target"),
        item_id,
        "{diagnostics:#?}"
    );
    press(c, GameButton::A);
    pp_observe(c, item_id, "after target A", &mut diagnostics);
    let actual = c.snapshot().expect("native PP item result").party;
    assert!(
        actual == expected,
        "{item_id} scope/amount must change only intended native PP; expected_pp={:?}; actual_pp={:?}; observations={diagnostics:#?}",
        expected.slots.first().map(|p| &p.pokemon.moves),
        actual.slots.first().map(|p| &p.pokemon.moves)
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        carried - if changed { 1 } else { 0 },
        "{item_id}: full-PP refusal must not consume the item; {diagnostics:#?}"
    );
    let expected_notice = if !changed {
        "It won't have any effect.".to_string()
    } else if let Some(slot) = move_slot {
        let name = super::super::battle_move_display_name(
            &before,
            &before.party.slots[0].pokemon.moves[slot].name,
        );
        format!("{name}'s PP was restored.")
    } else {
        "PP was restored.".to_string()
    };
    assert_eq!(
        c.shell.field_notice.as_deref(),
        Some(expected_notice.as_str()),
        "{item_id} actual native notice; {diagnostics:#?}"
    );
    c.wait_frames(240)
        .expect("print native PP success or refusal notice");
    assert_eq!(
        completed_visible_page(c).map(|(_, p)| p),
        Some(expected_notice)
    );
    let at = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), at, "PP success/refusal notice owns movement");
    for _ in 0..64 {
        pp_observe(c, item_id, "closing native PP menus", &mut diagnostics);
        let snapshot = c.snapshot().expect("close native PP menus");
        if c.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            c.wait_frames(60).expect("PP notice frames");
            press(c, GameButton::A);
        } else if snapshot.ui.menu.is_some()
            || c.shell.field_pack_target_mode.is_some()
            || c.shell.field_pack_action_cursor.is_some()
            || c.shell.bag_cursor.is_some()
            || c.shell.field_pack_pocket.is_some()
            || c.shell.start_menu_cursor.is_some()
        {
            press(c, GameButton::B);
        } else {
            settle(c, "after native PP menus close");
            return;
        }
    }
    panic!("native PP menus did not release input for {item_id}: {diagnostics:#?}");
}
#[test]
#[ignore = "requires actual revised R12 shelter pack and matching frozen source"]
fn r12_shelter_controller_original_otta_max_ether_restores_legal_pp_once_and_continues() {
    let (root, runtime, mut c) = fixture(ROOM, (17, 22), PpSetup::OneLowMove);
    let initial_flags = flags(&c);
    otta(&mut c, &runtime);
    assert_eq!(flags(&c), initial_flags);
    assert_eq!(quantity(&c, &runtime), 0);
    collect(&mut c, &runtime);
    use_pp_item(&mut c, &runtime, "MAX_ETHER", Some(0));
    assert_eq!(quantity(&c, &runtime), 0);
    leave(&mut c, &runtime);
    enter(&mut c, &runtime);
    indoor_walk(&mut c, &runtime, (26, 5));
    step(&mut c, GameButton::Right, (27, 5));
    press(&mut c, GameButton::A);
    settle(&mut c, "already collected native cache");
    assert_eq!(quantity(&c, &runtime), 0);
    assert!(collected(&c));
    c = continue_save(root, runtime.clone(), &mut c);
    assert!(collected(&c));
    assert_eq!(quantity(&c, &runtime), 0);
    otta(&mut c, &runtime);
    assert_eq!(c.shell.shell.session().state().money, 1000);
}

#[test]
#[ignore = "requires actual revised R12 shelter pack with native PP item catalogs"]
fn r12_shelter_controller_native_ether_elixers_and_full_pp_refusals_preserve_resources() {
    // Four actual native items (two each), legal learned moves at initial PP1.
    // All following inventory/PP changes come only from production menu input.
    let (root, runtime, mut c) = fixture(ROOM, (17, 22), PpSetup::NativeItemReview);
    let original_flags = flags(&c);
    assert_eq!(runtime.data().items["ETHER"].pp_restore_points, Some(10));
    assert_eq!(runtime.data().items["ELIXER"].pp_restore_points, Some(10));
    assert_eq!(runtime.data().items["MAX_ETHER"].pp_restore_points, None);
    assert_eq!(runtime.data().items["MAX_ELIXER"].pp_restore_points, None);
    use_pp_item(&mut c, &runtime, "ETHER", Some(0));
    use_pp_item(&mut c, &runtime, "MAX_ETHER", Some(1));
    // Selected move is full while other moves still have low PP. MOVE scope
    // must refuse rather than restoring another move or consuming the item.
    use_pp_item(&mut c, &runtime, "MAX_ETHER", Some(1));
    use_pp_item(&mut c, &runtime, "ELIXER", None);
    use_pp_item(&mut c, &runtime, "MAX_ELIXER", None);
    for (id, slot) in [
        ("ETHER", Some(0)),
        ("MAX_ETHER", Some(0)),
        ("ELIXER", None),
        ("MAX_ELIXER", None),
    ] {
        use_pp_item(&mut c, &runtime, id, slot);
    }
    assert_eq!(flags(&c), original_flags);
    assert!(!collected(&c));
    c = continue_save(root, runtime.clone(), &mut c);
    for id in ["ETHER", "MAX_ETHER", "ELIXER", "MAX_ELIXER"] {
        assert_eq!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&runtime.data().items[id]),
            1
        );
    }
    assert_eq!(c.shell.shell.session().state().money, 1000);
}
