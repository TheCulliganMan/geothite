//! Local party/position/RNG fixtures followed only by production controller input.
//! These cases do not earn River Bluffs access, badges or regional progression.
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
    BevyShellConfig, BevyShellStart, VisibleFieldItemPhase, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};

const ROOM: &str = "MnR05RockCut";
const EVENT: &str = "EVENT_MN_R05_ITEM_03";
const ITEM: &str = "REPEL";

fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_R05_ROCKCUT_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/renders/caves/r05-rockcut-candidate/v02/MnR05RockCut.candidate.json",
            )
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen485652d7 source"))
        .expect("RockCut source")
}

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R05_ROCKCUT_CONTROLLER_PACK")
                    .expect("actual revised RockCut pack"),
            )
            .canonicalize()
            .expect("published RockCut pack");
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict native pack"),
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
    .expect("explicit local RockCut fixture");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if injured_partner {
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 30, Dv::default())
            .expect("legal native learned moves");
        assert!(pokemon.max_hp > 30);
        pokemon.hp = pokemon.max_hp - 30;
        pokemon.status = Some("PARALYSIS".to_string());
        for learned in &mut pokemon.moves {
            learned.current_pp = 1;
        }
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
    // Explicit setup ends here. No later party, flag, inventory or location writes.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(tile(&mut controller), at);
    assert!(!collected(&controller));
    assert_eq!(quantity(&controller, &runtime), 0);
    assert_eq!(repel_steps(&controller), 0);
    let authored = source();
    let expected = authored["rows"]
        .as_array()
        .expect("frozen rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|block| u16::from_str_radix(block, 16).expect("native block"))
        .collect::<Vec<_>>();
    let room = &runtime.data().maps[ROOM];
    assert_eq!([room.attributes.width, room.attributes.height], [8, 10]);
    assert_eq!(room.blocks, expected, "actual frozen geometry");
    assert_eq!(room.attributes.time_of_day.as_deref(), Some("nite"));
    assert!(room.script_runtime_commands.iter().any(|command| {
        command.source_script == "MN_R05_CUT_WALKER"
            && command.command == "special"
            && command.command_index == 4
            && command.args == ["HealParty"]
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
        .expect("original Repel event")
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

fn repel_steps(controller: &VisibleShellController) -> u16 {
    controller
        .shell
        .shell
        .session()
        .state()
        .repel_steps_remaining
}

fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let room = &runtime.data().maps[ROOM];
    let block = room.blocks[at.1 as usize / 2 * 8 + at.0 as usize / 2];
    &runtime.data().tilesets["cave"].collision[&format!("{block:02x}")]
        [at.1 as usize % 2 * 2 + at.0 as usize % 2]
}

fn facing(button: GameButton) -> Direction {
    match button {
        GameButton::Up => Direction::Up,
        GameButton::Down => Direction::Down,
        GameButton::Left => Direction::Left,
        GameButton::Right => Direction::Right,
        _ => panic!("movement direction required"),
    }
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    let before = tile(controller);
    let previous_facing = controller.snapshot().expect("before step").overworld.facing;
    let repel_before = repel_steps(controller);
    press(controller, button);
    if previous_facing != facing(button) {
        assert_eq!(tile(controller), before, "first press turns before walking");
        assert_eq!(
            repel_steps(controller),
            repel_before,
            "turning is not a step"
        );
    }
    for _ in 0..8 {
        if tile(controller) == target {
            if repel_before > 0 {
                assert_eq!(repel_steps(controller), repel_before - 1);
            }
            return;
        }
        assert_eq!(tile(controller), before, "unexpected intermediate movement");
        press(controller, button);
    }
    assert_eq!(tile(controller), target);
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = tile(controller);
    for _ in 0..4 {
        press(controller, button);
        assert_eq!(tile(controller), before, "native sidewall");
    }
}

fn outside(runtime: &CrystalRuntime, north: bool) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR05",
            columns: 1,
            rows: 3,
            tileset: "mn_headwaters",
            start: if north { (30, 28) } else { (16, 78) },
            doors: BTreeMap::from([((16, 77), ROOM), ((30, 27), ROOM)]),
        },
    )
}

fn enter(controller: &mut VisibleShellController, runtime: &CrystalRuntime, north: bool) {
    let (active, inside, clear) = if north {
        ((30, 27), (11, 5), (11, 4))
    } else {
        ((16, 77), (5, 17), (5, 16))
    };
    outside(runtime, north).walk(controller, active);
    assert_eq!(map(controller), ROOM);
    assert_eq!(tile(controller), inside, "original mouth entry identity");
    indoor_walk(controller, runtime, clear);
    assert_eq!(tile(controller), clear);
    if north {
        // The old north ladder is beside its clear tile. Walk around it through
        // Floor instead of letting a generic shortest path use an active warp.
        step(controller, GameButton::Left, (10, 4));
        step(controller, GameButton::Left, (9, 4));
        step(controller, GameButton::Down, (9, 5));
    }
}

fn leave(controller: &mut VisibleShellController, runtime: &CrystalRuntime, north: bool) {
    let (clear, mouth, exterior, local, global) = if north {
        ((11, 4), (11, 5), "MnR05C00R00", (30, 28), (30, 28))
    } else {
        ((5, 16), (5, 17), "MnR05C00R01", (16, 30), (16, 78))
    };
    if north {
        indoor_walk(controller, runtime, (9, 5));
        step(controller, GameButton::Up, (9, 4));
        step(controller, GameButton::Right, (10, 4));
        step(controller, GameButton::Right, clear);
    } else {
        indoor_walk(controller, runtime, clear);
    }
    assert_eq!(tile(controller), clear);
    indoor_walk(controller, runtime, mouth);
    assert_eq!(map(controller), exterior);
    assert_eq!(tile(controller), local, "true passive exterior return");
    controller.wait_frames(60).expect("stable passive Floor");
    assert_eq!(map(controller), exterior);
    assert_eq!(tile(controller), local);
    let grid = outside(runtime, north);
    grid.walk(controller, (global.0 + 1, global.1));
    grid.walk(controller, global);
    assert_eq!(map(controller), exterior, "Floor revisit must not warp");
    assert_eq!(tile(controller), local);
    assert_eq!(
        runtime.data().maps[exterior].events.warps[1].target_warp_id,
        1,
        "retain each existing inactive passive record, including the north one"
    );
}

fn healer(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (6, 13));
    press(controller, GameButton::Left);
    assert_eq!(tile(controller), (6, 13));
    let before = controller.snapshot().expect("injured partner").party;
    assert_eq!(before.slots.len(), 1);
    assert!(before.slots[0].pokemon.hp < before.slots[0].pokemon.max_hp);
    assert_eq!(before.slots[0].pokemon.status.as_deref(), Some("PARALYSIS"));
    assert!(
        before.slots[0]
            .pokemon
            .moves
            .iter()
            .all(|learned| learned.current_pp == 1)
    );
    let mut expected = before.clone();
    for slot in &mut expected.slots {
        slot.pokemon.hp = slot.pokemon.max_hp;
        slot.pokemon.status = None;
        for learned in &mut slot.pokemon.moves {
            assert_eq!(learned.pp_ups, 0);
            learned.current_pp = runtime.data().moves[&learned.name].pp;
        }
    }
    press(controller, GameButton::A);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), (6, 13), "original healer text owns input");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    let mut healed = false;
    for _ in 0..48 {
        controller
            .wait_frames(240)
            .expect("print original healer page");
        remember_dialogue(controller, &mut diagnostics);
        let snapshot = controller.snapshot().expect("actual healing state");
        if snapshot.party == expected && pages.len() == 2 {
            healed = true;
            break;
        }
        assert_eq!(
            snapshot.party, before,
            "HealParty must wait for original pages: {diagnostics:#?}"
        );
        if let Some((index, page)) = completed_visible_page(controller) {
            assert_eq!(
                snapshot.ui.text.as_ref().map(|text| text.label.as_str()),
                Some("MN_R05_CUT_WALKERText")
            );
            pages.insert(index, page);
        }
        press(controller, GameButton::A);
    }
    assert_eq!(
        pages.get(&0).map(String::as_str),
        Some("Rest a while.\nI'll treat them."),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1).map(String::as_str),
        Some("Both mouths join\nthe same old hill."),
        "{diagnostics:#?}"
    );
    assert!(
        healed,
        "native HealParty did not restore HP/status/PP: {diagnostics:#?}"
    );
    settle(controller, "after original RockCut HealParty");
    assert_eq!(
        controller.snapshot().expect("healed partner").party,
        expected
    );
}

#[test]
#[ignore = "requires actual revised RockCut pack and frozen485652d7 source"]
fn r05_rockcut_controller_both_mouths_stair_healer_two_pages_and_continue() {
    let (root, runtime, mut controller) = fixture("MnR05C00R01", (16, 30), true);
    let initial_flags = flags(&controller);
    let bag = controller.snapshot().expect("original inventory").bag;
    enter(&mut controller, &runtime, false);
    healer(&mut controller, &runtime);
    indoor_walk(&mut controller, &runtime, (9, 5));
    for (button, target) in [
        (GameButton::Down, (9, 6)),
        (GameButton::Down, (9, 7)),
        (GameButton::Down, (9, 8)),
        (GameButton::Up, (9, 7)),
    ] {
        assert_eq!(kind(&runtime, target), "FLOOR");
        step(&mut controller, button, target);
    }
    assert_eq!(kind(&runtime, (8, 7)), "WALL");
    blocked(&mut controller, GameButton::Left);
    step(&mut controller, GameButton::Up, (9, 6));
    step(&mut controller, GameButton::Up, (9, 5));
    leave(&mut controller, &runtime, true);
    enter(&mut controller, &runtime, true);
    leave(&mut controller, &runtime, false);
    enter(&mut controller, &runtime, false);
    assert_eq!(flags(&controller), initial_flags);
    assert_eq!(
        controller.snapshot().expect("healer grants no items").bag,
        bag
    );
    assert_eq!(controller.shell.shell.session().state().money, 1000);
    controller = continue_save(root, runtime.clone(), &mut controller);
    let party = controller
        .snapshot()
        .expect("continued healed partner")
        .party;
    assert_eq!(party.slots.len(), 1);
    let pokemon = &party.slots[0].pokemon;
    assert_eq!(pokemon.hp, pokemon.max_hp);
    assert_eq!(pokemon.status, None);
    for learned in &pokemon.moves {
        assert_eq!(learned.current_pp, runtime.data().moves[&learned.name].pp);
    }
    assert_eq!(quantity(&controller, &runtime), 0);
    assert!(!collected(&controller));
}

fn collect(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    indoor_walk(controller, runtime, (6, 5));
    press(controller, GameButton::Left);
    assert_eq!(tile(controller), (6, 5));
    press(controller, GameButton::A);
    assert!(collected(controller));
    assert_eq!(quantity(controller, runtime), 1);
    let before = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        tile(controller),
        before,
        "native pickup notice owns movement"
    );
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        controller
            .wait_frames(240)
            .expect("native Found/fanfare then pocket printing");
        remember_dialogue(controller, &mut diagnostics);
        if controller
            .shell
            .visible_field_item_notice
            .as_ref()
            .is_some_and(|notice| notice.phase == VisibleFieldItemPhase::PocketText)
            && let Some((index, page)) = completed_visible_page(controller)
        {
            pages.insert(index, page);
        }
        if controller.shell.field_notice.is_none()
            && controller.shell.visible_field_item_notice.is_none()
            && !controller.has_pending_script_work()
        {
            break;
        }
        // Successful Found/fanfare is autonomous and need not remain visible
        // after an input settles. A only acknowledges fully printed pocket pages.
        if completed_visible_page(controller).is_some() {
            press(controller, GameButton::A);
        }
    }
    assert_eq!(
        pages.get(&0).map(String::as_str),
        Some("CHRIS put the\nREPEL in"),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1).map(String::as_str),
        Some("REPEL in\nthe ITEM POCKET."),
        "{diagnostics:#?}"
    );
    settle(controller, "after original Repel pickup notice");
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

fn use_repel(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    settle(controller, "before actual Repel Pack USE");
    assert_eq!(
        runtime.data().items[ITEM].repel_steps,
        Some(100),
        "original Repel must last100 steps, not an unrelated Repel item's duration"
    );
    assert_eq!(repel_steps(controller), 0);
    let party = controller.snapshot().expect("party before Repel").party;
    press(controller, GameButton::Start);
    let snapshot = controller.snapshot().expect("production Start");
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
        .expect("carried Repel")
        .bag
        .items
        .iter()
        .filter(|item| item.quantity > 0)
        .position(|item| item.item_id == ITEM)
        .expect("original collected Repel");
    cursor(controller, target, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Pack cursor")
            .option_index
    });
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("Repel actions");
    let actions = super::super::visible_selected_pack_item_actions(
        &snapshot,
        &controller.shell,
        &super::super::FieldPackPocket::Items,
        false,
    )
    .expect("native Repel actions");
    let target = actions
        .iter()
        .position(|action| *action == super::super::FieldPackAction::Use)
        .expect("USE action");
    cursor(controller, target, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("native item use notice");
    let expected_notice = snapshot
        .presentation
        .asm_text_for_label("_ItemUsedText")
        .expect("original source item-use text")
        .replace("<PLAYER>", &snapshot.trainer.player_name)
        .replace(
            "<STRING_BUFFER_2>",
            &format!("\n{}", runtime.data().items[ITEM].name.replace('_', " ")),
        );
    assert_eq!(
        controller.shell.field_notice.as_deref(),
        Some(expected_notice.as_str())
    );
    assert!(
        !expected_notice.contains('<'),
        "item-use buffers must be resolved"
    );
    assert_eq!(
        repel_steps(controller),
        100,
        "actual authoritative Repel effect"
    );
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .active_repel_item
            .as_deref(),
        Some(ITEM)
    );
    assert_eq!(
        quantity(controller, runtime),
        0,
        "collected Repel consumed once"
    );
    assert_eq!(
        controller.snapshot().expect("Repel preserves party").party,
        party
    );
    let at = tile(controller);
    press(controller, GameButton::Down);
    assert_eq!(tile(controller), at, "native use notice owns movement");
    for _ in 0..64 {
        let snapshot = controller.snapshot().expect("close native Repel notice");
        if controller.shell.field_notice.is_some() || snapshot.ui.pending_text_wait.is_some() {
            controller.wait_frames(60).expect("Repel notice printing");
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
            settle(controller, "after native Repel menu closes");
            assert_eq!(repel_steps(controller), 100, "menu input is not walking");
            return;
        }
    }
    panic!("native Repel menu did not release input");
}

#[test]
#[ignore = "requires revised RockCut and genuine native100-step Repel catalog"]
fn r05_rockcut_controller_original_repel_once_pack_use_movement_and_continue() {
    let (root, runtime, mut controller) = fixture(ROOM, (5, 16), false);
    let mut expected_flags = flags(&controller);
    collect(&mut controller, &runtime);
    expected_flags.0.insert(EVENT.to_string());
    assert_eq!(flags(&controller), expected_flags);
    leave(&mut controller, &runtime, false);
    enter(&mut controller, &runtime, false);
    indoor_walk(&mut controller, &runtime, (6, 5));
    assert_eq!(kind(&runtime, (5, 5)), "FLOOR");
    step(&mut controller, GameButton::Left, (5, 5));
    press(&mut controller, GameButton::A);
    settle(&mut controller, "already collected original Repel tile");
    assert_eq!(
        quantity(&controller, &runtime),
        1,
        "no second grant on revisit"
    );
    step(&mut controller, GameButton::Right, (6, 5));
    use_repel(&mut controller, &runtime);
    step(&mut controller, GameButton::Right, (7, 5));
    assert_eq!(repel_steps(&controller), 99);
    step(&mut controller, GameButton::Left, (6, 5));
    assert_eq!(repel_steps(&controller), 98);
    controller = continue_save(root, runtime.clone(), &mut controller);
    assert_eq!(
        repel_steps(&controller),
        98,
        "Continue retains actual remaining steps"
    );
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .state()
            .active_repel_item
            .as_deref(),
        Some(ITEM)
    );
    assert_eq!(quantity(&controller, &runtime), 0);
    assert!(collected(&controller));
    assert_eq!(flags(&controller), expected_flags);
    assert_eq!(controller.shell.shell.session().state().money, 1000);
    step(&mut controller, GameButton::Right, (7, 5));
    assert_eq!(
        repel_steps(&controller),
        97,
        "continued effect still counts real steps"
    );
}
