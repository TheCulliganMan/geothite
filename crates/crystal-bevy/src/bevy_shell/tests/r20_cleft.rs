//! Explicit local position/party/RNG fixtures followed only by controller input.
//! These tests do not earn Voyageurs access, SQ03 reporting or regional badges.
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

const ROOM: &str = "MnR20Cave1F";
const EVENT: &str = "EVENT_MN_R20_ITEM_01";
const ITEM: &str = "HYPER_POTION";

fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_R20_CLEFT_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/renders/caves/r20-candidate/v02/MnR20Cave1F.candidate.json",
            )
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen R20 source"))
        .expect("R20 source JSON")
}

fn point(value: &Value) -> Point {
    (
        value[0].as_i64().expect("source x") as i16,
        value[1].as_i64().expect("source y") as i16,
    )
}

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R20_CLEFT_CONTROLLER_PACK")
                    .expect("actual revised R20 native pack"),
            )
            .canonicalize()
            .expect("published R20 pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict published pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}

fn fixture(
    name: &str,
    at: Point,
    injured_partner: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("retained HOME"),
            map_name: name.to_string(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".to_string()),
            ..Default::default()
        },
    )
    .expect("explicit local cleft fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if injured_partner {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 80, Dv::default())
            .expect("source-legal learned moves and HP");
        assert!(pokemon.max_hp > 210);
        pokemon.hp = pokemon.max_hp - 210;
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
    }
    let state = controller.shell.shell.session_mut().state_mut();
    state.money = 1000;
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Initial fixture setup ends here. No later party, flag, bag or position writes.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(tile(&mut controller), at);
    assert!(!collected(&controller));
    assert_eq!(quantity(&controller, &runtime), 0);
    let room = &runtime.data().maps[ROOM];
    assert_eq!([room.attributes.width, room.attributes.height], [9, 8]);
    let expected = source()["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|block| u16::from_str_radix(block, 16).expect("native block"))
        .collect::<Vec<_>>();
    assert_eq!(room.blocks, expected, "actual pack must match frozen v02");
    assert!(room.script_field_pickups.iter().any(|pickup| {
        pickup.source_script == "MN_R20_ITEM_01"
            && pickup.command_index == 0
            && pickup.item_id.as_deref() == Some(ITEM)
            && pickup.quantity == 1
            && pickup.event_flag.as_deref() == Some(EVENT)
    }));
    (root, runtime, controller)
}

fn collected(controller: &VisibleShellController) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(EVENT)
        .expect("original cave item flag")
}

fn quantity(controller: &VisibleShellController, runtime: &CrystalRuntime) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[ITEM])
}

fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let room = &runtime.data().maps[ROOM];
    let block = room.blocks[at.1 as usize / 2 * 9 + at.0 as usize / 2];
    &runtime.data().tilesets["cave"].collision[&format!("{block:02x}")]
        [at.1 as usize % 2 * 2 + at.0 as usize % 2]
}

fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .iter()
        .find_map(|&(at, button)| (at == to).then_some(button))
        .expect("adjacent native step")
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    for _ in 0..8 {
        if tile(controller) == target {
            return;
        }
        press(controller, button);
    }
    assert_eq!(tile(controller), target);
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = tile(controller);
    for _ in 0..4 {
        press(controller, button);
        assert_eq!(tile(controller), before, "native wall/reverse must block");
    }
}

fn outside(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR20",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start: (60, 34),
            doors: BTreeMap::from([((60, 33), ROOM), ((76, 31), ROOM)]),
        },
    )
}

fn enter(controller: &mut VisibleShellController, runtime: &CrystalRuntime, east: bool) {
    let (mouth, inside, clear) = if east {
        ((76, 31), (13, 13), (13, 12))
    } else {
        ((60, 33), (5, 13), (5, 12))
    };
    outside(runtime).walk(controller, mouth);
    assert_eq!(map(controller), ROOM);
    assert_eq!(tile(controller), inside, "original entry warp pairing");
    indoor_walk(controller, runtime, clear);
    assert_eq!(tile(controller), clear);
}

fn leave(controller: &mut VisibleShellController, runtime: &CrystalRuntime, east: bool) {
    let (clear, mouth, exterior, local, global) = if east {
        ((13, 12), (13, 13), "MnR20C01R00", (12, 32), (76, 32))
    } else {
        ((5, 12), (5, 13), "MnR20C00R00", (60, 34), (60, 34))
    };
    indoor_walk(controller, runtime, clear);
    indoor_walk(controller, runtime, mouth);
    assert_eq!(map(controller), exterior);
    assert_eq!(tile(controller), local);
    controller
        .wait_frames(60)
        .expect("true passive Floor arrival");
    assert_eq!(map(controller), exterior);
    assert_eq!(tile(controller), local);
    let grid = outside(runtime);
    grid.walk(controller, (global.0 + 1, global.1));
    grid.walk(controller, global);
    assert_eq!(
        map(controller),
        exterior,
        "revisiting passive Floor cannot warp"
    );
    assert_eq!(tile(controller), local);
}

fn ordinary_path(runtime: &CrystalRuntime, from: Point, target: Point) -> Vec<Point> {
    let occupied = runtime.data().maps[ROOM]
        .objects
        .iter()
        .map(|object| (object.x as i16, object.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(from, from)]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        if at == target {
            break;
        }
        for (next, _) in neighbors(at) {
            if next.0 < 0
                || next.1 < 0
                || next.0 >= 18
                || next.1 >= 16
                || kind(runtime, next) != "FLOOR"
                || occupied.contains(&next)
                || parents.contains_key(&next)
            {
                continue;
            }
            parents.insert(next, at);
            queue.push_back(next);
        }
    }
    assert!(parents.contains_key(&target), "ordinary return exists");
    let mut result = Vec::new();
    let mut at = target;
    while at != from {
        result.push(at);
        at = parents[&at];
    }
    result.reverse();
    result
}

#[test]
#[ignore = "requires published R20 cleft pack and frozen v02 source"]
fn r20_cleft_controller_both_mouths_passive_returns_and_two_stairs_bidirectional() {
    let (root, runtime, mut controller) = fixture("MnR20C00R00", (60, 34), false);
    let initial_flags = flags(&controller);
    enter(&mut controller, &runtime, false);
    let authored = source();
    let stairs = authored["review_contract"]["controller_probes"]["stairs"]
        .as_array()
        .expect("two native stair lanes");
    assert_eq!(stairs.len(), 2);
    for stair in stairs {
        let mut lane = vec![point(&stair["before"])];
        lane.extend(stair["lane"].as_array().expect("lane").iter().map(point));
        lane.push(point(&stair["after"]));
        for &at in &lane {
            assert_eq!(kind(&runtime, at), "FLOOR");
            indoor_walk(&mut controller, &runtime, at);
            assert_eq!(tile(&mut controller), at);
        }
        let from = point(&stair["blocked_from"]);
        let target = point(&stair["blocked_target"]);
        assert_eq!(kind(&runtime, target), "WALL");
        indoor_walk(&mut controller, &runtime, from);
        blocked(&mut controller, direction(from, target));
        indoor_walk(&mut controller, &runtime, *lane.last().expect("after"));
        for &at in lane.iter().rev().skip(1) {
            indoor_walk(&mut controller, &runtime, at);
            assert_eq!(tile(&mut controller), at);
        }
    }
    assert_eq!(kind(&runtime, (12, 11)), "WALL", "v01 toe bypass closed");
    leave(&mut controller, &runtime, true);
    controller = continue_save(root, runtime.clone(), &mut controller);
    enter(&mut controller, &runtime, true);
    leave(&mut controller, &runtime, false);
    enter(&mut controller, &runtime, false);
    leave(&mut controller, &runtime, false);
    assert_eq!(flags(&controller), initial_flags);
    assert!(!collected(&controller));
}

#[test]
#[ignore = "requires published native HOP_LEFT cleft geometry"]
fn r20_cleft_controller_both_left_hop_lanes_reverse_denial_and_ordinary_returns() {
    let (_, runtime, mut controller) = fixture(ROOM, (13, 12), false);
    let authored = source();
    let probes = &authored["review_contract"]["controller_probes"];
    for lane in probes["ledge_lanes"]
        .as_array()
        .expect("two left hop lanes")
    {
        let approach = point(&lane["approach"]);
        let trigger = point(&lane["hop_from"]);
        let landing = point(&lane["landing"]);
        let reverse = point(&lane["reverse_target"]);
        assert_eq!(kind(&runtime, trigger), "HOP_LEFT");
        assert_eq!(kind(&runtime, reverse), "WALL");
        indoor_walk(&mut controller, &runtime, approach);
        let mut observed = vec![approach];
        for _ in 0..8 {
            press(&mut controller, GameButton::Left);
            let at = tile(&mut controller);
            observed.push(at);
            assert!(
                at == approach || at == trigger || at == landing,
                "{observed:?}"
            );
            assert_ne!(at, reverse, "hop crosses the actual blocking quadrant");
            if at == landing {
                break;
            }
        }
        assert_eq!(tile(&mut controller), landing, "{observed:?}");
        blocked(&mut controller, GameButton::Right);
        let path = ordinary_path(&runtime, landing, approach);
        assert_eq!(path.len(), if landing.1 == 8 { 15 } else { 13 });
        for at in path {
            let before = tile(&mut controller);
            step(&mut controller, direction(before, at), at);
            assert_eq!(map(&mut controller), ROOM);
        }
    }
    for shoulder in probes["ledge_shoulders"]
        .as_array()
        .expect("four closed shoulders")
    {
        assert_eq!(kind(&runtime, point(shoulder)), "WALL");
    }
    assert!(!collected(&controller));
}

fn collect(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (4, 5));
    press(controller, GameButton::Right);
    assert_eq!(tile(controller), (4, 5));
    press(controller, GameButton::A);
    assert!(collected(controller));
    assert_eq!(quantity(controller, runtime), 1);
    let before = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), before, "printed pickup notice owns input");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        controller
            .wait_frames(240)
            .expect("print native pickup page");
        remember_dialogue(controller, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(controller) {
            assert_eq!(
                controller
                    .shell
                    .visible_field_item_notice
                    .as_ref()
                    .expect("item notice")
                    .phase,
                super::super::VisibleFieldItemPhase::PocketText
            );
            pages.insert(index, page);
        }
        if controller.shell.field_notice.is_none()
            && controller.shell.visible_field_item_notice.is_none()
            && !controller.has_pending_script_work()
        {
            break;
        }
        press(controller, GameButton::A);
    }
    let item = runtime.data().items[ITEM].name.replace('_', " ");
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
    settle(controller, "after full original Hyper Potion pickup notice");
}

fn cursor(
    controller: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let selected = index(controller);
        if selected == target {
            return;
        }
        press(
            controller,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native cursor failed to reach {target}");
}

fn use_hyper_potion(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    settle(controller, "before native Hyper Potion Pack input");
    let before = controller.snapshot().expect("injured party").party;
    let mut expected = before.clone();
    let amount =
        u16::try_from(runtime.data().items[ITEM].parameter).expect("native healing amount");
    assert_eq!(amount, 200, "actual original item potency");
    let partner = &mut expected.slots[0].pokemon;
    partner.hp = partner.hp.saturating_add(amount).min(partner.max_hp);
    assert_ne!(before, expected);
    press(controller, GameButton::Start);
    let snapshot = controller.snapshot().expect("native Start");
    let target = super::super::visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|option| *option == super::super::StartMenuOption::Pack)
        .expect("PACK option");
    cursor(controller, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(controller, GameButton::A);
    let target = controller
        .snapshot()
        .expect("carried potion")
        .bag
        .items
        .iter()
        .filter(|item| item.quantity > 0)
        .position(|item| item.item_id == ITEM)
        .expect("actual collected Hyper Potion");
    cursor(controller, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("item actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("native potion actions");
    let target = actions
        .iter()
        .position(|action| *action == super::super::FieldPackAction::Use)
        .expect("USE option");
    cursor(controller, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    press(controller, GameButton::A);
    assert_eq!(
        controller.shell.field_pack_target_mode,
        Some(super::super::FieldPackTargetMode::PartyPokemon)
    );
    cursor(controller, 0, |c| c.shell.party_cursor);
    press(controller, GameButton::A);
    assert_eq!(
        controller.snapshot().expect("native item effect").party,
        expected,
        "only HP changes through the authoritative item engine"
    );
    assert_eq!(
        quantity(controller, runtime),
        0,
        "collected potion consumed once"
    );
    let before_position = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        tile(controller),
        before_position,
        "medicine notice owns movement"
    );
    for _ in 0..64 {
        let snapshot = controller.snapshot().expect("close medicine menu");
        if controller.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            controller.wait_frames(60).expect("medicine notice frames");
            press(controller, GameButton::A);
        } else if snapshot.ui.menu.is_some()
            || controller.shell.field_pack_target_mode.is_some()
            || controller.shell.field_pack_action_cursor.is_some()
            || controller.shell.bag_cursor.is_some()
            || controller.shell.field_pack_pocket.is_some()
            || controller.shell.start_menu_cursor.is_some()
        {
            press(controller, GameButton::B);
        } else {
            settle(controller, "after native medicine menus close");
            return;
        }
    }
    panic!("native Hyper Potion menus did not release input");
}

#[test]
#[ignore = "requires published R20 original Hyper Potion and native Pack effect"]
fn r20_cleft_controller_original_hyper_potion_prints_once_heals_and_continues() {
    let (root, runtime, mut controller) = fixture(ROOM, (5, 12), true);
    let initial_flags = flags(&controller);
    collect(&mut controller, &runtime);
    let mut expected_flags = initial_flags;
    expected_flags.0.insert(EVENT.to_string());
    assert_eq!(flags(&controller), expected_flags);
    use_hyper_potion(&mut controller, &runtime);
    leave(&mut controller, &runtime, true);
    enter(&mut controller, &runtime, true);
    indoor_walk(&mut controller, &runtime, (4, 5));
    step(&mut controller, GameButton::Right, (5, 5));
    press(&mut controller, GameButton::A);
    settle(&mut controller, "already collected original Hyper Potion");
    assert_eq!(quantity(&controller, &runtime), 0);
    assert!(collected(&controller));
    controller = continue_save(root, runtime.clone(), &mut controller);
    assert_eq!(quantity(&controller, &runtime), 0);
    assert!(collected(&controller));
    assert_eq!(flags(&controller), expected_flags);
    assert_eq!(controller.shell.shell.session().state().money, 1000);
}
