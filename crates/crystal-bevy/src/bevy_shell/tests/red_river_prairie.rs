//! Explicit native fixtures; neither earned regional progress nor battle balance.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, neighbors, opposite, position,
};

fn fixture(start: Point, populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = std::env::var_os("CRYSTAL_RED_RIVER_PRAIRIE_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-red-river-prairie.crystalpack")
        })
        .canonicalize()
        .expect("published Red River Prairie pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified native pack"),
    )
    .expect("native prairie runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: format!("MnR16C{:02}R00", start.0 / 64),
            tile_x: start.0 % 64,
            tile_y: start.1,
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
    if populated {
        // Explicit setup only: native-created partners and six obedience badge
        // bits, not an earned badge/team claim. No state is rewritten afterward.
        let partners = ["TYPHLOSION", "FERALIGATR"].map(|species| {
            runtime
                .data()
                .create_pokemon(species, 60, Dv::default())
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
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR16",
            columns: 2,
            rows: 1,
            tileset: if runtime
                .data()
                .tilesets
                .contains_key("mn_prairie_headwaters")
            {
                "mn_prairie_headwaters"
            } else {
                "mn_headwaters"
            },
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
            "geothite-r16-{label}-{}-{stamp}.crystalsave",
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
#[ignore = "requires the published external Red River Prairie pack"]
fn red_river_controller_walks_both_maps_bridges_bank_interactions_and_continue() {
    let (root, runtime, mut controller) = fixture((88, 24), false);
    // Empty-party geometry fixture: grass movement is real, encounter/balance
    // coverage belongs to the populated test and regional journey separately.
    let grid = grid(&runtime, (88, 24));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller
        .press(GameButton::B)
        .expect("close production menu");
    for at in [(92, 24), (2, 24), (63, 30), (64, 30), (63, 30), (64, 30)] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("native chunk")
                .overworld
                .map_name,
            format!("MnR16C{:02}R00", at.0 / 64)
        );
    }
    // Cross both physical deck rows in both directions, including the clipped
    // native-map seam in the drainage bridge.
    for (x0, x1, rows) in [(10, 27, [23, 24]), (60, 67, [29, 30])] {
        for y in rows {
            for x in x0..=x1 {
                grid.walk(&mut controller, (x, y));
            }
            for x in (x0..=x1).rev() {
                grid.walk(&mut controller, (x, y));
            }
        }
    }
    for (at, button) in [
        ((19, 23), GameButton::Up),
        ((19, 24), GameButton::Down),
        ((63, 29), GameButton::Up),
        ((63, 30), GameButton::Down),
        ((18, 15), GameButton::Right),
        ((18, 16), GameButton::Right),
        ((16, 15), GameButton::Up),
        ((16, 16), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        barrier(&mut controller, at, button);
    }
    // Both lanes of the short hayfield lip must perform an actual native hop.
    grid.walk(&mut controller, (52, 15));
    let mut saw_left_face = false;
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("left hayfield hop lane");
        let at = position(&mut controller);
        assert!(
            matches!(at, (52, 15) | (52, 16) | (52, 18)),
            "left hop leaked: {at:?}"
        );
        saw_left_face |= at == (52, 16);
        if at == (52, 18) {
            break;
        }
    }
    assert!(saw_left_face);
    assert_eq!(position(&mut controller), (52, 18));
    barrier(&mut controller, (52, 18), GameButton::Up);
    grid.walk(&mut controller, (53, 15));
    let mut saw_face = false;
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native hayfield hop");
        let at = position(&mut controller);
        assert_ne!(at, (53, 17), "ordinary movement entered the reverse wall");
        assert!(
            matches!(at, (53, 15) | (53, 16) | (53, 18)),
            "unexpected hop position {at:?}"
        );
        saw_face |= at == (53, 16);
        if at == (53, 18) {
            break;
        }
    }
    assert!(saw_face, "native HOP_DOWN face was not traversed");
    assert_eq!(position(&mut controller), (53, 18));
    barrier(&mut controller, (53, 18), GameButton::Up);
    let uphill = grid.path((53, 18), (53, 15));
    assert_eq!(uphill.len(), 59, "ordinary hayfield return changed");
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
    assert_eq!(position(&mut controller), (53, 15));
    for (at, text) in [
        ((3, 25), "MOORHEAD"),
        ((47, 29), "west ditch"),
        ((87, 23), "cottonwoods"),
        ((4, 22), "MOORHEAD"),
        ((30, 20), "hayfield"),
        ((56, 28), "DRAINAGE"),
        ((78, 22), "PRAIRIE SEED"),
        ((86, 22), "COTTONWOOD"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "wrong interaction at {at:?}"
        );
    }
    let pickups = [
        ((49, 7), "HYPER_POTION", "EVENT_MN_R16_ITEM_01"),
        ((91, 7), "ULTRA_BALL", "EVENT_MN_R16_ITEM_02"),
        ((5, 37), "MAX_REPEL", "EVENT_MN_R16_ITEM_03"),
        ((79, 39), "REVIVE", "EVENT_MN_R16_ITEM_04"),
    ];
    for (at, item, event) in pickups {
        let before = quantity(&mut controller, item);
        assert!(grid.talk(&mut controller, at).contains("put the"));
        assert_eq!(quantity(&mut controller, item), before + 1);
        assert!(flag(&controller, event));
    }
    for index in 1..=10 {
        assert!(
            !flag(&controller, &format!("EVENT_MN_R16_TRAINER_{index:02}")),
            "geometry fixture unexpectedly defeated a trainer"
        );
    }
    grid.walk(&mut controller, (88, 24));
    let save = OwnedSave::new("geometry");
    let bag = controller.snapshot().expect("collected bag").bag;
    controller.save(&save.0).expect("production save");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("production Continue");
    assert_eq!(position(&mut loaded), (88, 24));
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
#[ignore = "requires the published external Red River Prairie pack"]
fn red_river_controller_wins_real_camper_battle_mutates_pp_and_continues_party_flags() {
    let (root, runtime, mut controller) = fixture((28, 25), true);
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
        {
            if let Some(cursor) = &controller.shell.battle_move_cursor {
                let battle = snapshot.battle.as_ref().expect("move owner");
                let movement = &battle.player_moves[cursor.option_index];
                selected = Some((
                    battle.active_player_party_index.expect("active partner"),
                    movement.name.clone(),
                    movement.current_pp,
                ));
            }
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
        BTreeSet::from([0, 1, 2]),
        "native three-partner Camper party was not faced"
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
    assert!(flag(&controller, "EVENT_MN_R16_TRAINER_07"));
    assert!(
        controller.shell.shell.session().state().money > money_before,
        "native victory reward missing"
    );
    // Victory makes the previously guarded dry apron usable. All movement is
    // real input; only the fixture path planner retains static trainer hazards.
    step(&mut controller, GameButton::Up, (28, 24));
    step(&mut controller, GameButton::Right, (29, 24));
    step(&mut controller, GameButton::Right, (30, 24));
    let party = controller.snapshot().expect("post-victory party").party;
    let save = OwnedSave::new("victory");
    controller.save(&save.0).expect("save actual victory");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("Continue actual victory");
    assert_eq!(position(&mut loaded), (30, 24));
    assert_eq!(loaded.snapshot().expect("continued party").party, party);
    assert!(flag(&loaded, "EVENT_MN_R16_TRAINER_07"));
    for index in (1..=10).filter(|index| *index != 7) {
        assert!(
            !flag(&loaded, &format!("EVENT_MN_R16_TRAINER_{index:02}")),
            "one win must not claim a full-route trainer journey"
        );
    }
    step(&mut loaded, GameButton::Down, (30, 25));
    loaded
        .press(GameButton::Left)
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
