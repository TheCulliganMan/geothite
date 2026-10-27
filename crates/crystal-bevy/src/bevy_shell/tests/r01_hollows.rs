//! Explicit local position/legal party/status/RNG setup, then production input only.
//! These cases do not earn a starter, route access or regional progression.
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::world::map::Direction;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, FieldPackAction, FieldPackPocket, FieldPackTargetMode,
    StartMenuOption, VisibleFieldItemPhase, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty, selected_field_pack_item_id, visible_field_dialog_pages,
    visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk, neighbors};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};

const ROOM: &str = "MnR01Cave1F";
const OUTSIDE: &str = "MnR01C00R00";
const ITEM: &str = "ANTIDOTE";
const EVENT: &str = "EVENT_MN_R01_ITEM_03";

fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_R01_HOLLOWS_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../planning/minnesota/renders/caves/r01-candidate/214bd2388b04598e/MnR01Cave1F.candidate.json",
        ));
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen R01 source214"))
        .expect("R01 source JSON")
}

fn point(value: &Value) -> Point {
    (
        value[0].as_i64().expect("source x") as i16,
        value[1].as_i64().expect("source y") as i16,
    )
}

fn fixture(poisoned_partner: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    let (root, runtime) = LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R01_HOLLOWS_CONTROLLER_PACK")
                    .expect("actual published R01 hollows native pack"),
            )
            .canonicalize()
            .expect("published R01 pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict verified R01 pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("retained HOME"),
            map_name: OUTSIDE.to_string(),
            tile_x: 18,
            tile_y: 18,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".to_string()),
            ..Default::default()
        },
    )
    .expect("explicit local hollow fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if poisoned_partner {
        let mut pokemon = runtime
            .data()
            .create_pokemon("TOTODILE", 20, Dv::default())
            .expect("source-legal native partner and learned moves");
        pokemon.status = Some("POISON".to_string());
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.sync_party_from_storage();
    }
    controller
        .shell
        .shell
        .session_mut()
        .state_mut()
        .random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Initial fixture setup ends here. No later party/position/bag/flag mutations.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(tile(&mut controller), (18, 18));
    assert!(!collected(&controller));
    assert_eq!(quantity(&controller, &runtime), 0);
    let module = &runtime.data().maps[ROOM];
    assert_eq!([module.attributes.width, module.attributes.height], [8, 7]);
    assert_eq!(module.attributes.time_of_day.as_deref(), Some("nite"));
    let expected = source()["rows"]
        .as_array()
        .expect("frozen rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|value| u16::from_str_radix(value, 16).expect("native block"))
        .collect::<Vec<_>>();
    assert_eq!(
        module.blocks, expected,
        "actual pack matches frozen214 geometry"
    );
    assert!(
        module
            .script_field_pickups
            .iter()
            .any(|pickup| pickup.source_script == "MN_R01_ITEM_03"
                && pickup.command_index == 0
                && pickup.item_id.as_deref() == Some(ITEM)
                && pickup.quantity == 1
                && pickup.event_flag.as_deref() == Some(EVENT))
    );
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
        .expect("original Antidote event")
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
    let module = &runtime.data().maps[ROOM];
    let block = module.blocks[at.1 as usize / 2 * 8 + at.0 as usize / 2];
    &runtime.data().tilesets["cave"].collision[&format!("{block:02x}")]
        [at.1 as usize % 2 * 2 + at.0 as usize % 2]
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = tile(controller);
    for _ in 0..4 {
        press(controller, button);
        assert_eq!(tile(controller), before);
    }
}

fn outside(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new(
        runtime,
        RouteSpec {
            prefix: "MnR01",
            columns: 1,
            rows: 1,
            tileset: "johto",
            start: (18, 18),
            doors: BTreeMap::from([((18, 17), ROOM), ((49, 15), "MnR01Farmhouse")]),
        },
    )
}

fn enter(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    outside(runtime).walk(controller, (18, 17));
    assert_eq!(map(controller), ROOM);
    assert_eq!(tile(controller), (9, 11), "original warp1 arrival");
    indoor_walk(controller, runtime, (9, 10));
    assert_eq!(tile(controller), (9, 10));
}

fn leave(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (9, 10));
    indoor_walk(controller, runtime, (9, 11));
    assert_eq!(map(controller), OUTSIDE);
    assert_eq!(
        tile(controller),
        (18, 18),
        "original exterior passive warp2"
    );
    controller.wait_frames(60).expect("true passive return");
    assert_eq!(map(controller), OUTSIDE);
    assert_eq!(tile(controller), (18, 18));
    let grid = outside(runtime);
    grid.walk(controller, (18, 19));
    grid.walk(controller, (18, 18));
    assert_eq!(map(controller), OUTSIDE, "revisiting Floor cannot reenter");
    assert_eq!(tile(controller), (18, 18));
}

#[test]
#[ignore = "requires published frozen214 R01 hollows native pack"]
fn r01_hollows_controller_both_stairs_bidirectional_and_true_passive_return() {
    let (root, runtime, mut controller) = fixture(false);
    let initial_flags = flags(&controller);
    enter(&mut controller, &runtime);
    let authored = source();
    let stairs = authored["review_contract"]["controller_probes"]["stairs"]
        .as_array()
        .expect("two useful native stairs");
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
        let button = neighbors(from)
            .into_iter()
            .find_map(|(at, button)| (at == target).then_some(button))
            .expect("adjacent authentic side wall");
        blocked(&mut controller, button);
        indoor_walk(
            &mut controller,
            &runtime,
            *lane.last().expect("lower arrival"),
        );
        for &at in lane.iter().rev().skip(1) {
            indoor_walk(&mut controller, &runtime, at);
            assert_eq!(tile(&mut controller), at);
        }
    }
    leave(&mut controller, &runtime);
    controller = continue_save(root, runtime.clone(), &mut controller);
    enter(&mut controller, &runtime);
    leave(&mut controller, &runtime);
    assert_eq!(flags(&controller), initial_flags);
    assert!(!collected(&controller));
}

fn guide(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (4, 5));
    press(controller, GameButton::Right);
    assert_eq!(tile(controller), (4, 5));
    press(controller, GameButton::A);
    let before = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), before, "original guide owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        controller
            .wait_frames(240)
            .expect("fully print original guide page");
        remember_dialogue(controller, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(controller) {
            assert_eq!(
                controller
                    .snapshot()
                    .expect("guide text")
                    .ui
                    .text
                    .as_ref()
                    .map(|text| text.label.as_str()),
                Some("MN_R01_HOLLOW_GUIDEText")
            );
            pages.insert(index, page);
        }
        let snapshot = controller.snapshot().expect("guide printer ownership");
        if !snapshot.ui.text_window_open
            && visible_field_dialog_pages(&snapshot, &controller.shell).is_none()
            && snapshot.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
        {
            break;
        }
        press(controller, GameButton::A);
    }
    assert_eq!(
        pages,
        BTreeMap::from([
            (0, "Two small hollows\nmeet by the exit.".to_string()),
            (1, "Take either path.\nThe little ball".to_string()),
            (2, "holds an ANTIDOTE.".to_string())
        ]),
        "all3 fully printed original pages: {diagnostics:#?}"
    );
    settle(controller, "original guide released input");
}

fn collect(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (10, 5));
    press(controller, GameButton::Right);
    assert_eq!(tile(controller), (10, 5));
    press(controller, GameButton::A);
    assert!(collected(controller));
    assert_eq!(quantity(controller, runtime), 1);
    let before = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), before, "Antidote pickup text owns input");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        controller
            .wait_frames(240)
            .expect("fully print original item notice");
        remember_dialogue(controller, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(controller) {
            assert_eq!(
                controller
                    .shell
                    .visible_field_item_notice
                    .as_ref()
                    .expect("pickup notice")
                    .phase,
                VisibleFieldItemPhase::PocketText
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
    assert_eq!(
        pages.get(&0),
        Some(&"CHRIS put the\nANTIDOTE in".to_string()),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1),
        Some(&"ANTIDOTE in\nthe ITEM POCKET.".to_string()),
        "{diagnostics:#?}"
    );
    settle(controller, "original Antidote pickup released input");
}

fn cursor(
    controller: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let current = index(controller);
        if current == target {
            return;
        }
        press(
            controller,
            if current < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    panic!("native cursor failed to reach {target}");
}

fn use_antidote(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    settle(controller, "before actual Antidote input");
    let before = controller.snapshot().expect("poisoned legal partner").party;
    assert_eq!(before.slots.len(), 1);
    assert_eq!(before.slots[0].pokemon.status.as_deref(), Some("POISON"));
    assert!(before.slots[0].pokemon.hp > 0);
    let mut expected = before.clone();
    expected.slots[0].pokemon.status = None;
    press(controller, GameButton::Start);
    let snapshot = controller.snapshot().expect("production Start");
    let target = visible_start_menu_options(&controller.shell, &snapshot)
        .iter()
        .position(|option| *option == StartMenuOption::Pack)
        .expect("PACK");
    cursor(controller, target, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    press(controller, GameButton::A);
    assert_eq!(
        selected_field_pack_item_id(&mut controller.shell).expect("sole collected native item"),
        ITEM
    );
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("Antidote actions");
    let target = visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &FieldPackPocket::Items,
        false,
    )
    .expect("native item actions")
    .iter()
    .position(|action| *action == FieldPackAction::Use)
    .expect("USE");
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
        Some(FieldPackTargetMode::PartyPokemon)
    );
    assert_eq!(controller.shell.party_cursor, 0);
    press(controller, GameButton::A);
    assert_eq!(
        controller
            .snapshot()
            .expect("actual native status cure")
            .party,
        expected,
        "Antidote changes only poison status, preserving current HP/moves/PP"
    );
    assert_eq!(
        quantity(controller, runtime),
        0,
        "original Antidote consumed once"
    );
    let before = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        tile(controller),
        before,
        "native medicine notice owns movement"
    );
    for _ in 0..64 {
        let snapshot = controller
            .snapshot()
            .expect("close native Antidote surfaces");
        if controller.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            controller.wait_frames(60).expect("medicine text");
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
            settle(controller, "after actual Antidote menus close");
            return;
        }
    }
    panic!("Antidote menus did not release input");
}

#[test]
#[ignore = "requires published R01 guide/Antidote and authoritative native status cure"]
fn r01_hollows_controller_original_guide_antidote_once_cures_poison_and_continues() {
    let (root, runtime, mut controller) = fixture(true);
    let initial_flags = flags(&controller);
    let money = controller.shell.shell.session().state().money;
    enter(&mut controller, &runtime);
    guide(&mut controller, &runtime);
    collect(&mut controller, &runtime);
    let mut expected_flags = initial_flags;
    expected_flags.0.insert(EVENT.to_string());
    assert_eq!(flags(&controller), expected_flags);
    use_antidote(&mut controller, &runtime);
    let cured = controller
        .snapshot()
        .expect("cured party after menus")
        .party;
    leave(&mut controller, &runtime);
    enter(&mut controller, &runtime);
    indoor_walk(&mut controller, &runtime, (10, 5));
    let before_facing = controller
        .snapshot()
        .expect("repeat approach facing")
        .overworld
        .facing;
    press(&mut controller, GameButton::Right);
    assert_eq!(
        controller.snapshot().expect("native turn").overworld.facing,
        Direction::Right
    );
    // The approach ends facing north. The production first directional input
    // turns in place; a later press must genuinely enter the collected tile.
    if before_facing != Direction::Right {
        assert_eq!(
            tile(&mut controller),
            (10, 5),
            "native turn precedes walking"
        );
    }
    let mut observed = vec![tile(&mut controller)];
    for _ in 0..8 {
        if tile(&mut controller) == (11, 5) {
            break;
        }
        press(&mut controller, GameButton::Right);
        observed.push(tile(&mut controller));
    }
    assert_eq!(
        tile(&mut controller),
        (11, 5),
        "collected object no longer blocks original tile; before_facing={before_facing:?}; observed={observed:?}"
    );
    press(&mut controller, GameButton::A);
    settle(&mut controller, "already collected Antidote");
    assert!(collected(&controller));
    assert_eq!(quantity(&controller, &runtime), 0);
    assert_eq!(
        controller
            .snapshot()
            .expect("cure persists through reentry")
            .party,
        cured
    );
    controller = continue_save(root, runtime.clone(), &mut controller);
    assert_eq!(flags(&controller), expected_flags);
    assert!(collected(&controller));
    assert_eq!(quantity(&controller, &runtime), 0);
    assert_eq!(
        controller
            .snapshot()
            .expect("cure persists through Continue")
            .party,
        cured
    );
    assert_eq!(controller.shell.shell.session().state().money, money);
}
