//! Explicit native fixtures; neither earned regional progress nor battle balance.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, escape_wild, finish_text, neighbors, opposite, position,
};

#[derive(Clone, Copy)]
enum Setup {
    Empty,
    Battle,
    Cut { learns_cut: bool, hive_badge: bool },
}

fn fixture(start: Point, setup: Setup) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = std::env::var_os("CRYSTAL_HEADWATERS_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-headwaters.crystalpack")
        })
        .canonicalize()
        .expect("published Headwaters pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified native pack"),
    )
    .expect("native Headwaters runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: format!("MnR14C{:02}R{:02}", start.0 / 64, start.1 / 48),
            tile_x: start.0 % 64,
            tile_y: start.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit native review placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    assert!(
        controller
            .snapshot()
            .expect("fresh party")
            .party
            .slots
            .is_empty()
    );
    if matches!(setup, Setup::Battle) {
        // Explicit setup only: native-created partners and six obedience badge
        // bits, not an earned badge/team claim. No state is rewritten afterward.
        let partners = ["TYPHLOSION", "FERALIGATR"].map(|species| {
            runtime
                .data()
                .create_pokemon(species, 45, Dv::default())
                .expect("legal source partner")
        });
        let state = controller.shell.shell.session_mut().state_mut();
        for partner in partners {
            assert!(state.storage.party.add_pokemon(partner));
        }
        state.badges.johto = [true, true, true, true, true, true, false, false];
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut controller.shell);
    }
    if let Setup::Cut {
        learns_cut,
        hive_badge,
    } = setup
    {
        // Initial field-move fixture setup only: native HM-compatible partner,
        // legal moves and permission bits, not an earned team/HM/badge claim.
        let mut partner = runtime
            .data()
            .create_pokemon("MEW", 45, Dv::default())
            .expect("native HM-compatible source partner");
        let moves = if learns_cut {
            vec!["CUT", "PSYCHIC_M"]
        } else {
            vec!["PSYCHIC_M"]
        };
        partner.moves = moves
            .into_iter()
            .map(|name| LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            })
            .collect();
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.badges.johto = [true, true, true, true, true, true, false, false];
        state.badges.johto[1] = hive_badge;
        state.sync_party_from_storage();
        mark_runtime_snapshot_dirty(&mut controller.shell);
    }
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR14",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start,
            doors: BTreeMap::new(),
        },
    )
}

fn quantity(controller: &mut VisibleShellController, item: &str) -> u16 {
    let bag = controller.snapshot().expect("bag").bag;
    bag.items
        .iter()
        .chain(&bag.balls)
        .filter(|slot| slot.item_id == item)
        .map(|slot| slot.quantity)
        .sum()
}

fn flag(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("native flag")
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    let before = position(controller);
    for _ in 0..8 {
        controller.press(button).expect("production movement");
        let snapshot = controller.snapshot().expect("native step");
        assert!(
            snapshot.battle.is_none(),
            "unexpected battle while walking {before:?}"
        );
        let at = position(controller);
        if at == target {
            return;
        }
        assert_eq!(at, before, "unexpected movement toward {target:?}");
    }
    panic!("native step {before:?} -> {target:?} stayed blocked");
}

fn barrier(controller: &mut VisibleShellController, at: Point, button: GameButton) {
    assert_eq!(position(controller), at);
    for _ in 0..4 {
        controller.press(button).expect("try actual native barrier");
        assert_eq!(position(controller), at, "native barrier leaked");
    }
}

struct OwnedSave(PathBuf);

impl OwnedSave {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "geothite-r14-{label}-{}-{stamp}.crystalsave",
            std::process::id()
        )))
    }
}

impl Drop for OwnedSave {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
#[ignore = "requires the published external Headwaters pack"]
fn headwaters_controller_walks_four_clipped_maps_rails_habitat_bank_and_continue() {
    let (root, runtime, mut controller) = fixture((88, 26), Setup::Empty);
    // Empty-party geometry fixture: grass movement is real, encounter/balance
    // coverage belongs to the populated test and regional journey separately.
    let grid = grid(&runtime, (88, 26));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller
        .press(GameButton::B)
        .expect("close production menu");
    let mut seams = BTreeMap::new();
    for &a in &grid.floor {
        for (b, _) in neighbors(a) {
            if a >= b
                || !grid.floor.contains(&b)
                || grid.blocked.contains(&a)
                || grid.blocked.contains(&b)
                || grid.hazards.contains(&a)
                || grid.hazards.contains(&b)
            {
                continue;
            }
            let pair = ((a.0 / 64, a.1 / 48), (b.0 / 64, b.1 / 48));
            if pair.0 != pair.1 {
                seams.entry(pair).or_insert((a, b));
            }
        }
    }
    assert_eq!(
        seams.len(),
        3,
        "reachable clipped-map walking pairs changed"
    );
    for &(a, b) in seams.values() {
        grid.walk(&mut controller, a);
        grid.walk(&mut controller, b);
        grid.walk(&mut controller, a);
    }
    // The installed Itasca gatehouse facade occupies the old (2,42) endpoint.
    // Reach its actual eastern approach without replacing the native gate.
    for at in [(92, 26), (6, 42)] {
        grid.walk(&mut controller, at);
    }
    // Cross both physical boardwalk rows in both directions.
    for y in [37, 38] {
        for x in 42..=53 {
            grid.walk(&mut controller, (x, y));
        }
        for x in (42..=53).rev() {
            grid.walk(&mut controller, (x, y));
        }
    }
    for (at, button) in [
        ((47, 37), GameButton::Up),
        ((47, 38), GameButton::Down),
        ((11, 17), GameButton::Left),
        ((11, 18), GameButton::Left),
        ((15, 17), GameButton::Up),
        ((15, 18), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        barrier(&mut controller, at, button);
    }
    grid.walk(&mut controller, (55, 38));
    for x in 56..=60 {
        step(&mut controller, GameButton::Right, (x, 38));
    }
    for x in (55..=59).rev() {
        step(&mut controller, GameButton::Left, (x, 38));
    }
    grid.walk(&mut controller, (29, 23));
    let mut saw_face = false;
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native survey hop");
        let at = position(&mut controller);
        assert_ne!(at, (29, 25), "ordinary movement entered the reverse wall");
        assert!(
            matches!(at, (29, 23) | (29, 24) | (29, 26)),
            "unexpected hop position {at:?}"
        );
        saw_face |= at == (29, 24);
        if at == (29, 26) {
            break;
        }
    }
    assert!(saw_face, "native HOP_DOWN face was not traversed");
    assert_eq!(position(&mut controller), (29, 26));
    barrier(&mut controller, (29, 26), GameButton::Up);
    let uphill = grid.path((29, 26), (29, 23));
    assert_eq!(uphill.len(), 53, "ordinary survey return changed");
    for button in uphill {
        let at = position(&mut controller);
        let target = match button {
            GameButton::Left => (at.0 - 1, at.1),
            GameButton::Right => (at.0 + 1, at.1),
            GameButton::Up => (at.0, at.1 - 1),
            GameButton::Down => (at.0, at.1 + 1),
            _ => panic!("path planner produced a non-direction"),
        };
        step(&mut controller, button, target);
    }
    assert_eq!(position(&mut controller), (29, 23));
    for (at, text) in [
        ((89, 27), "Lake Itasca"),
        ((65, 15), "pines shade"),
        ((19, 11), "count the birds"),
        ((55, 37), "small stream"),
        ((17, 55), "We pause"),
        ((79, 45), "WINDHOPPER"),
        ((90, 24), "HEADWATERS"),
        ((68, 10), "OLD PINE"),
        ((54, 34), "MISSISSIPPI"),
        ((20, 12), "SURVEY POND"),
        ((26, 50), "FERN HOLLOW"),
        ((76, 44), "MARSH"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "wrong interaction {at:?}"
        );
    }
    let pickups = [
        ((31, 15), "HYPER_POTION", "EVENT_MN_R14_ITEM_01"),
        ((85, 13), "ULTRA_BALL", "EVENT_MN_R14_ITEM_02"),
        ((11, 57), "MAX_REPEL", "EVENT_MN_R14_ITEM_03"),
        ((91, 55), "REVIVE", "EVENT_MN_R14_ITEM_04"),
    ];
    for (at, item, event) in pickups {
        let before = quantity(&mut controller, item);
        assert!(grid.talk(&mut controller, at).contains("found"));
        assert_eq!(quantity(&mut controller, item), before + 1);
        assert!(flag(&controller, event));
    }
    for index in 1..=6 {
        assert!(
            !flag(&controller, &format!("EVENT_MN_R14_TRAINER_{index:02}")),
            "geometry fixture unexpectedly defeated a trainer"
        );
    }
    grid.walk(&mut controller, (88, 26));
    let save = OwnedSave::new("geometry");
    let bag = controller.snapshot().expect("collected bag").bag;
    controller.save(&save.0).expect("production save");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("production Continue");
    assert_eq!(position(&mut loaded), (88, 26));
    assert_eq!(loaded.snapshot().expect("continued bag").bag, bag);
    for (at, _, event) in pickups {
        assert!(flag(&loaded, event));
        let (approach, away) = neighbors(at)
            .into_iter()
            .find(|(point, _)| {
                grid.floor.contains(point)
                    && !grid.blocked.contains(point)
                    && !grid.hazards.contains(point)
            })
            .expect("collected pickup approach");
        grid.walk(&mut loaded, approach);
        assert!(
            !loaded
                .snapshot()
                .expect("hidden pickup")
                .visible_objects
                .iter()
                .any(|object| object.script == event.trim_start_matches("EVENT_")),
            "saved pickup respawned"
        );
        step(&mut loaded, opposite(away), at);
        step(&mut loaded, away, approach);
    }
}

fn tree_block(controller: &VisibleShellController) -> u16 {
    controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(3, 10)
        .expect("current pine shortcut source block")
}

fn open_cut(controller: &mut VisibleShellController, direction: GameButton) {
    controller
        .press(direction)
        .expect("face actual native Cut tree");
    for _ in 0..64 {
        if tree_block(controller) == 0x3c {
            finish_text(controller);
            controller.wait_frames(60).expect("native Cut presentation");
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native Cut prompt or confirmation");
    }
    panic!("Headwaters Cut did not apply actual5b-to3c source replacement");
}

#[test]
#[ignore = "requires the published external Headwaters pack"]
fn headwaters_cut_shortcut_denies_permissions_opens_real_link_and_resets_natively() {
    for (learns_cut, hive_badge) in [(false, true), (true, false)] {
        let (_, _, mut denied) = fixture(
            (71, 21),
            Setup::Cut {
                learns_cut,
                hive_badge,
            },
        );
        assert_eq!(tree_block(&denied), 0x5b);
        barrier(&mut denied, (71, 21), GameButton::Up);
        denied.press(GameButton::A).expect("native Cut denial");
        assert!(
            denied
                .snapshot()
                .expect("denied Cut text")
                .ui
                .text
                .and_then(|text| text.asm_text)
                .is_some_and(|text| text.contains("CUT"))
        );
        finish_text(&mut denied);
        assert_eq!(tree_block(&denied), 0x5b);
        assert_eq!(position(&mut denied), (71, 21));
    }
    let (root, runtime, mut controller) = fixture(
        (71, 21),
        Setup::Cut {
            learns_cut: true,
            hive_badge: true,
        },
    );
    // A dry-floor planner avoids incidental wild battles; the native source
    // comparator independently proves the same34-to2 saving with Surf too.
    let dry = RouteGrid::new(
        &runtime,
        RouteSpec {
            prefix: "MnR14",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start: (71, 21),
            doors: BTreeMap::new(),
        },
    );
    assert_eq!(dry.path((71, 21), (71, 19)).len(), 34);
    assert!(!dry.floor.contains(&(71, 20)));
    dry.walk(&mut controller, (71, 19));
    dry.walk(&mut controller, (71, 21));
    assert_eq!(tree_block(&controller), 0x5b);
    open_cut(&mut controller, GameButton::Up);
    assert_eq!(tree_block(&controller), 0x3c);
    step(&mut controller, GameButton::Up, (71, 20));
    step(&mut controller, GameButton::Up, (71, 19));
    step(&mut controller, GameButton::Down, (71, 20));
    step(&mut controller, GameButton::Down, (71, 21));
    // Leave the actual native map over its reciprocal seam, then return.
    dry.walk(&mut controller, (63, 10));
    dry.walk(&mut controller, (71, 21));
    assert_eq!(
        tree_block(&controller),
        0x5b,
        "native map load must regrow temporary vegetation"
    );
    open_cut(&mut controller, GameButton::Up);
    step(&mut controller, GameButton::Up, (71, 20));
    step(&mut controller, GameButton::Up, (71, 19));
    let party = controller
        .snapshot()
        .expect("actual Cut partner state")
        .party;
    let save = OwnedSave::new("cut");
    controller
        .save(&save.0)
        .expect("save actual opened shortcut");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("Continue actual shortcut fixture");
    assert_eq!(position(&mut loaded), (71, 19));
    assert_eq!(
        loaded.snapshot().expect("continued Cut partner").party,
        party
    );
    assert_eq!(
        tree_block(&loaded),
        0x5b,
        "Continue rebuilds native temporary Cut vegetation"
    );
    open_cut(&mut loaded, GameButton::Down);
    step(&mut loaded, GameButton::Down, (71, 20));
    step(&mut loaded, GameButton::Down, (71, 21));
    for index in 1..=6 {
        assert!(!flag(&loaded, &format!("EVENT_MN_R14_TRAINER_{index:02}")));
    }
}

fn battle_button(
    controller: &VisibleShellController,
    runtime: &CrystalRuntime,
    snapshot: &crystal_runtime::RuntimeShellSnapshot,
) -> GameButton {
    let battle = snapshot.battle.as_ref().expect("battle owner");
    if !controller.shell.battle_messages.is_empty() || snapshot.ui.menu.is_none() {
        return GameButton::A;
    }
    let menu = snapshot.ui.menu.as_ref().expect("visible battle menu");
    let options = menu
        .layout
        .vertical_menus
        .iter()
        .flat_map(|menu| &menu.options)
        .collect::<Vec<_>>();
    if options
        .iter()
        .any(|option| option.contains("change <PKMN>"))
    {
        return GameButton::B;
    }
    if options.iter().any(|option| option.contains("FIGHT")) {
        let cursor = controller
            .shell
            .battle_action_cursor
            .as_ref()
            .expect("native action cursor")
            .option_index;
        return if cursor / 2 > 0 {
            GameButton::Up
        } else if !cursor.is_multiple_of(2) {
            GameButton::Left
        } else {
            GameButton::A
        };
    }
    if let Some(cursor) = &controller.shell.battle_party_action_cursor {
        return if cursor.option_index > 0 {
            GameButton::Up
        } else {
            GameButton::A
        };
    }
    if let Some(cursor) = &controller.shell.battle_switch_cursor {
        let target = snapshot
            .party
            .slots
            .iter()
            .position(|slot| {
                slot.pokemon.hp > 0 && Some(slot.index) != battle.active_player_party_index
            })
            .expect("living source-created reserve");
        return if cursor.option_index < target {
            GameButton::Down
        } else if cursor.option_index > target {
            GameButton::Up
        } else {
            GameButton::A
        };
    }
    if let Some(cursor) = &controller.shell.battle_move_cursor {
        let target = battle
            .player_moves
            .iter()
            .enumerate()
            .filter(|(_, learned)| {
                learned.current_pp > 0 && runtime.data().moves[&learned.name].power > 0
            })
            .max_by_key(|(_, learned)| {
                let movement = &runtime.data().moves[&learned.name];
                u32::from(movement.power) * u32::from(movement.accuracy)
            })
            .map(|(index, _)| index)
            .expect("legal damaging native move with PP");
        return if cursor.option_index < target {
            GameButton::Down
        } else if cursor.option_index > target {
            GameButton::Up
        } else {
            GameButton::A
        };
    }
    GameButton::A
}

fn battle_note(
    controller: &VisibleShellController,
    snapshot: &crystal_runtime::RuntimeShellSnapshot,
) -> String {
    let options = snapshot.ui.menu.as_ref().map(|menu| {
        menu.layout
            .vertical_menus
            .iter()
            .flat_map(|menu| &menu.options)
            .take(8)
            .collect::<Vec<_>>()
    });
    format!(
        "phase={:?}; text={:?}; enemy={:?}; options={options:?}; action/move/switch={:?}; messages={}",
        snapshot.phase,
        snapshot.ui.text.as_ref().map(|text| text.label.as_str()),
        snapshot.battle.as_ref().map(|battle| (
            battle.enemy_pokemon.species.id.as_str(),
            battle.enemy_pokemon.hp
        )),
        (
            controller
                .shell
                .battle_action_cursor
                .as_ref()
                .map(|cursor| cursor.option_index),
            controller
                .shell
                .battle_move_cursor
                .as_ref()
                .map(|cursor| cursor.option_index),
            controller
                .shell
                .battle_switch_cursor
                .as_ref()
                .map(|cursor| cursor.option_index),
        ),
        controller.shell.battle_messages.len()
    )
}

#[test]
#[ignore = "requires the published external Headwaters pack"]
fn headwaters_controller_wins_native_mara_battle_spends_pp_and_continues_party() {
    let (root, runtime, mut controller) = fixture((80, 31), Setup::Battle);
    let money_before = controller.shell.shell.session().state().money;
    controller
        .press(GameButton::Right)
        .expect("face actual Camper");
    controller
        .press(GameButton::A)
        .expect("native trainer interaction");
    let mut entered = false;
    let mut opponents_seen = BTreeSet::new();
    let mut selected = None;
    let mut pp_spent = false;
    for _ in 0..768 {
        let snapshot = controller.snapshot().expect("production battle state");
        if snapshot.battle.is_none() {
            if entered {
                break;
            }
            controller
                .press(GameButton::A)
                .expect("trainer introduction");
            continue;
        }
        entered = true;
        if let Some(index) = snapshot
            .battle
            .as_ref()
            .and_then(|battle| battle.active_enemy_party_index)
        {
            opponents_seen.insert(index);
        }
        let button = battle_button(&controller, &runtime, &snapshot);
        if selected.is_none()
            && button == GameButton::A
            && controller.shell.battle_messages.is_empty()
            && snapshot.ui.menu.is_some()
            && let Some(cursor) = &controller.shell.battle_move_cursor
        {
            let battle = snapshot.battle.as_ref().expect("move owner");
            let movement = &battle.player_moves[cursor.option_index];
            selected = Some((
                battle.active_player_party_index.expect("active partner"),
                movement.name.clone(),
                movement.current_pp,
            ));
        }
        controller
            .press(button)
            .expect("authoritative battle input");
        if let Some((active, name, before)) = &selected {
            let now = controller.snapshot().expect("authoritative PP");
            pp_spent |= now.party.slots[*active]
                .pokemon
                .moves
                .iter()
                .any(|movement| movement.name == *name && movement.current_pp < *before);
        }
    }
    assert!(entered, "actual trainer battle never started");
    assert_eq!(
        opponents_seen,
        BTreeSet::from([0, 1]),
        "native two-partner Camper party was not faced"
    );
    assert!(
        selected.is_some() && pp_spent,
        "selected native move did not spend PP"
    );
    let ended = controller.snapshot().expect("battle ended");
    assert!(
        ended.battle.is_none(),
        "battle did not finish: {}",
        battle_note(&controller, &ended)
    );
    finish_text(&mut controller);
    assert!(flag(&controller, "EVENT_MN_R14_TRAINER_01"));
    assert!(
        controller.shell.shell.session().state().money > money_before,
        "native victory reward missing"
    );
    // Actual clear-floor movement after victory, with no state rewriting.
    step(&mut controller, GameButton::Up, (80, 30));
    let party = controller.snapshot().expect("post-victory party").party;
    let save = OwnedSave::new("victory");
    controller.save(&save.0).expect("save actual victory");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("Continue actual victory");
    assert_eq!(position(&mut loaded), (80, 30));
    assert_eq!(loaded.snapshot().expect("continued party").party, party);
    assert!(flag(&loaded, "EVENT_MN_R14_TRAINER_01"));
    for index in (1..=6).filter(|index| *index != 1) {
        assert!(
            !flag(&loaded, &format!("EVENT_MN_R14_TRAINER_{index:02}")),
            "one win must not claim a full-route trainer journey"
        );
    }
    step(&mut loaded, GameButton::Down, (80, 31));
    loaded
        .press(GameButton::Right)
        .expect("face defeated Camper");
    loaded
        .press(GameButton::A)
        .expect("defeated native trainer dialogue");
    assert!(
        loaded
            .snapshot()
            .expect("repeat interaction")
            .battle
            .is_none()
    );
    finish_text(&mut loaded);
    assert_eq!(
        loaded.snapshot().expect("repeat dialogue settled").phase,
        RuntimeShellPhase::Overworld
    );
    assert!(
        loaded
            .snapshot()
            .expect("no repeat battle")
            .battle
            .is_none()
    );
}

#[test]
#[ignore = "requires the current Headwaters grass revision pack"]
fn headwaters_controller_grassy_survey_climb_dry_drop_and_continue() {
    let (root, runtime, mut controller) = fixture(
        (29, 26),
        Setup::Cut {
            learns_cut: false,
            hive_badge: false,
        },
    );
    let route = grid(&runtime, (29, 26));
    let mut grass_steps = 0;
    route.walk_with_actions(
        &mut controller,
        (29, 23),
        |c| {
            let (x, y) = position(c);
            let x = usize::try_from(x).expect("nonnegative route x");
            let y = usize::try_from(y).expect("nonnegative route y");
            let name = format!("MnR14C{:02}R{:02}", x / 64, y / 48);
            let map = &runtime.data().maps[&name];
            let block = map.blocks[(y % 48 / 2) * usize::from(map.attributes.width) + x % 64 / 2];
            let collision =
                &runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")];
            grass_steps += usize::from(collision[y % 2 * 2 + x % 2] == "TALL_GRASS");
        },
        escape_wild,
    );
    assert!(
        grass_steps >= 4,
        "ordinary survey climb must cross woodland grass"
    );
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native dry survey drop");
        let at = position(&mut controller);
        assert!(matches!(at, (29, 23) | (29, 24) | (29, 26)));
        assert!(
            controller
                .snapshot()
                .expect("dry hop state")
                .battle
                .is_none()
        );
        if at == (29, 26) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (29, 26));
    barrier(&mut controller, (29, 26), GameButton::Up);
    let before = controller.snapshot().expect("actual dry ledge return");
    let save = OwnedSave::new("grassy-survey-return");
    controller.save(&save.0).expect("save reached dry return");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("Continue reached survey return");
    let after = loaded.snapshot().expect("continued survey return");
    assert_eq!(position(&mut loaded), (29, 26));
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
}

#[test]
#[ignore = "requires the current Headwaters grass revision pack"]
fn headwaters_controller_new_woodland_grass_encounter_run_and_continue() {
    let (root, runtime, mut controller) = fixture(
        (15, 24),
        Setup::Cut {
            learns_cut: false,
            hive_badge: false,
        },
    );
    let route = grid(&runtime, (15, 24));
    let mut encounters = 0;
    for index in 0..256 {
        let target = if index % 2 == 0 { (15, 25) } else { (15, 24) };
        route.walk_with_actions(
            &mut controller,
            target,
            |_| {},
            |c| {
                encounters += 1;
                escape_wild(c);
            },
        );
        if encounters > 0 {
            break;
        }
    }
    assert!(
        encounters > 0,
        "new woodland grass never started an authoritative wild battle"
    );
    let reached = position(&mut controller);
    let before = controller.snapshot().expect("actual native wild escape");
    let save = OwnedSave::new("woodland-wild-run");
    controller
        .save(&save.0)
        .expect("save actual woodland escape");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("Continue actual woodland escape");
    let after = loaded.snapshot().expect("continued woodland escape");
    assert_eq!(position(&mut loaded), reached);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    if let Some(path) = std::env::var_os("CRYSTAL_HEADWATERS_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export reached woodland escape");
    }
}
