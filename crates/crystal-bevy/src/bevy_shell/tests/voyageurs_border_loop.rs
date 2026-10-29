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
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, opposite, position,
};

fn fixture(start: Point, populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = std::env::var_os("CRYSTAL_VOYAGEURS_BORDER_LOOP_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-voyageurs-border-loop.crystalpack")
        })
        .canonicalize()
        .expect("published Voyageurs Border Loop pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified native pack"),
    )
    .expect("native border runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: format!("MnR20C{:02}R{:02}", start.0 / 64, start.1 / 48),
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
    if populated {
        // Explicit setup only: native-created partners and eight obedience badge
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
        state.badges.johto = [true; 8];
        // Initial native badge projections must agree before Save/Continue.
        for name in [
            "ENGINE_ZEPHYRBADGE",
            "ENGINE_HIVEBADGE",
            "ENGINE_PLAINBADGE",
            "ENGINE_FOGBADGE",
            "ENGINE_STORMBADGE",
            "ENGINE_MINERALBADGE",
            "ENGINE_GLACIERBADGE",
            "ENGINE_RISINGBADGE",
        ] {
            state
                .flags
                .set_engine_flag(name, true)
                .expect("initial badge projection");
        }
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
            prefix: "MnR20",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start,
            // Include the current inland ice-cave mouth so ordinary route planning
            // cannot accidentally enter it while aiming for a different landmark.
            doors: BTreeMap::from([
                ((60, 33), "MnR20Cave1F"),
                ((76, 31), "MnR20Cave1F"),
                ((70, 33), "MnVoyageursIce1F"),
            ]),
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
            "geothite-r20-{label}-{}-{stamp}.crystalsave",
            std::process::id()
        )))
    }
}

impl Drop for OwnedSave {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn cave_position(controller: &mut VisibleShellController) -> Point {
    let snapshot = controller.snapshot().expect("native cave position");
    assert_eq!(snapshot.overworld.map_name, "MnR20Cave1F");
    (snapshot.overworld.tile.x, snapshot.overworld.tile.y)
}

fn cave_step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    let before = cave_position(controller);
    for _ in 0..8 {
        controller.press(button).expect("production cave movement");
        let at = cave_position(controller);
        if at == target {
            return;
        }
        assert_eq!(at, before, "unexpected cave movement toward {target:?}");
    }
    panic!("native cave step {before:?} -> {target:?} stayed blocked");
}

fn cave_exit(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    approach: Point,
    ladder: Point,
    arrival: Point,
) {
    indoor_walk(controller, runtime, approach);
    assert_eq!(cave_position(controller), approach);
    indoor_walk(controller, runtime, ladder);
    assert_eq!(position(controller), arrival, "wrong native mouth return");
    // A passive FLOOR return must permit leaving and revisiting without a
    // self-triggering cave loop. One horizontal step stays on the safe apron.
    step(controller, GameButton::Right, (arrival.0 + 1, arrival.1));
    step(controller, GameButton::Left, arrival);
}

#[test]
#[ignore = "requires the published external Voyageurs Border Loop pack"]
fn voyageurs_controller_walks_seams_bridges_habitat_cave_returns_and_continue() {
    let (root, runtime, mut controller) = fixture((122, 36), false);
    // Explicit empty-party geometry fixture; this does not earn eight badges,
    // lighthouse permission, the SQ03 circuit or League progression.
    let grid = grid(&runtime, (122, 36));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
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
    assert_eq!(seams.len(), 3, "reachable walking neighbor pairs changed");
    for &(a, b) in seams.values() {
        grid.walk(&mut controller, a);
        grid.walk(&mut controller, b);
        grid.walk(&mut controller, a);
    }
    for at in [(2, 44), (125, 36)] {
        grid.walk(&mut controller, at);
    }
    // Both real deck rows, both directions, at both independently necessary
    // outlets. Native compilation separately proves removal disconnects them.
    for (x0, x1, rows) in [(38, 49, [41, 42]), (90, 99, [35, 36])] {
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
        ((43, 41), GameButton::Up),
        ((43, 42), GameButton::Down),
        ((95, 35), GameButton::Up),
        ((95, 36), GameButton::Down),
        ((18, 25), GameButton::Right),
        ((18, 26), GameButton::Right),
        ((16, 25), GameButton::Up),
        ((16, 26), GameButton::Down),
    ] {
        grid.walk(&mut controller, at);
        barrier(&mut controller, at, button);
    }
    // Walk all four actual TALL_GRASS cells across the required neck, in both
    // directions. Empty-party setup intentionally suppresses battle coverage.
    grid.walk(&mut controller, (111, 36));
    for x in 112..=116 {
        step(&mut controller, GameButton::Right, (x, 36));
    }
    for x in (111..=115).rev() {
        step(&mut controller, GameButton::Left, (x, 36));
    }
    grid.walk(&mut controller, (19, 49));
    let mut saw_face = false;
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native wind bank hop");
        let at = position(&mut controller);
        assert_ne!(at, (19, 51), "ordinary step entered reverse wall");
        assert!(
            matches!(at, (19, 49) | (19, 50) | (19, 52)),
            "unexpected hop {at:?}"
        );
        saw_face |= at == (19, 50);
        if at == (19, 52) {
            break;
        }
    }
    assert!(saw_face, "actual native HOP_DOWN was not traversed");
    assert_eq!(position(&mut controller), (19, 52));
    barrier(&mut controller, (19, 52), GameButton::Up);
    let return_path = grid.path((19, 52), (19, 49));
    assert_eq!(return_path.len(), 15, "ordinary wind bank return changed");
    for button in return_path {
        let at = position(&mut controller);
        let target = match button {
            GameButton::Left => (at.0 - 1, at.1),
            GameButton::Right => (at.0 + 1, at.1),
            GameButton::Up => (at.0, at.1 - 1),
            GameButton::Down => (at.0, at.1 + 1),
            _ => panic!("path planner produced non-direction"),
        };
        step(&mut controller, button, target);
    }
    for (at, text) in [
        ((5, 45), "border trail"),
        ((55, 43), "mainland path"),
        ((119, 37), "BOUNDARY WATERS"),
        ((4, 42), "INTERNATIONAL"),
        ((8, 28), "BORDER LAKE"),
        ((58, 34), "GRANITE NARROWS"),
        ((84, 40), "SHELTERED MEADOW"),
        ((118, 34), "BOUNDARY WATERS"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "wrong dialogue at {at:?}"
        );
    }
    let pickups = [
        ((13, 26), "ULTRA_BALL", "EVENT_MN_R20_ITEM_02"),
        ((83, 55), "MAX_REPEL", "EVENT_MN_R20_ITEM_03"),
        ((119, 25), "REVIVE", "EVENT_MN_R20_ITEM_04"),
    ];
    for (at, item, event) in pickups {
        let before = quantity(&mut controller, item);
        assert!(grid.talk(&mut controller, at).contains("found"));
        assert_eq!(quantity(&mut controller, item), before + 1);
        assert!(flag(&controller, event));
    }
    // West entry owns room warp1. Cross the two chambers to east exit2.
    grid.walk(&mut controller, (60, 33));
    assert_eq!(cave_position(&mut controller), (5, 13));
    indoor_walk(&mut controller, &runtime, (5, 6));
    assert_eq!(cave_position(&mut controller), (5, 6));
    let before = quantity(&mut controller, "HYPER_POTION");
    let mut pickup_pages = Vec::new();
    for button in [GameButton::Up, GameButton::A] {
        controller.press(button).expect("native cave pickup input");
        let snapshot = controller.snapshot().expect("actual cave pickup page");
        if let Some(page) = snapshot.ui.text.and_then(|text| text.asm_text) {
            pickup_pages.push(page);
        }
        if let Some(notice) = &controller.shell.visible_field_item_notice {
            pickup_pages.push(notice.sound_trigger_text.clone());
        }
    }
    let text = controller.snapshot().expect("cave pickup text ownership");
    assert_eq!(text.phase, RuntimeShellPhase::Text);
    assert!(
        pickup_pages.iter().any(|page| page.contains("found")),
        "missing native found page: {pickup_pages:?}"
    );
    assert!(
        pickup_pages
            .iter()
            .any(|page| page.contains("HYPER POTION"))
    );
    finish_text(&mut controller);
    assert_eq!(quantity(&mut controller, "HYPER_POTION"), before + 1);
    assert!(flag(&controller, "EVENT_MN_R20_ITEM_01"));
    cave_exit(&mut controller, &runtime, (13, 12), (13, 13), (76, 32));
    // East entry owns room warp2; return through west exit1.
    grid.walk(&mut controller, (76, 31));
    assert_eq!(cave_position(&mut controller), (13, 13));
    cave_exit(&mut controller, &runtime, (5, 12), (5, 13), (60, 34));
    // Each mouth must also permit its own honest same-mouth exit.
    for (mouth, inside, approach, ladder, arrival) in [
        ((60, 33), (5, 13), (5, 12), (5, 13), (60, 34)),
        ((76, 31), (13, 13), (13, 12), (13, 13), (76, 32)),
    ] {
        grid.walk(&mut controller, mouth);
        assert_eq!(cave_position(&mut controller), inside);
        cave_exit(&mut controller, &runtime, approach, ladder, arrival);
    }
    for index in 1..=10 {
        assert!(!flag(
            &controller,
            &format!("EVENT_MN_R20_TRAINER_{index:02}")
        ));
    }
    grid.walk(&mut controller, (122, 36));
    let save = OwnedSave::new("geometry");
    let bag = controller.snapshot().expect("all collected items").bag;
    controller.save(&save.0).expect("native geometry save");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), save.0.clone(), None)
        .expect("production Continue");
    assert_eq!(position(&mut loaded), (122, 36));
    assert_eq!(loaded.snapshot().expect("continued bag").bag, bag);
    for (at, _, event) in pickups {
        assert!(flag(&loaded, event));
        let (approach, away) = neighbors(at)
            .into_iter()
            .find(|(p, _)| {
                grid.floor.contains(p) && !grid.blocked.contains(p) && !grid.hazards.contains(p)
            })
            .expect("saved pickup approach");
        grid.walk(&mut loaded, approach);
        assert!(
            !loaded
                .snapshot()
                .expect("saved pickup visibility")
                .visible_objects
                .iter()
                .any(|object| object.script == event.trim_start_matches("EVENT_"))
        );
        step(&mut loaded, opposite(away), at);
        step(&mut loaded, away, approach);
    }
    assert!(flag(&loaded, "EVENT_MN_R20_ITEM_01"));
    grid.walk(&mut loaded, (60, 33));
    indoor_walk(&mut loaded, &runtime, (5, 6));
    assert!(
        !loaded
            .snapshot()
            .expect("saved cave pickup visibility")
            .visible_objects
            .iter()
            .any(|object| object.script == "MN_R20_ITEM_01")
    );
    // The shared room planner deliberately retains source actor bodies. Use
    // actual input to prove the saved item no longer occupies its former tile.
    cave_step(&mut loaded, GameButton::Up, (5, 5));
    assert_eq!(
        cave_position(&mut loaded),
        (5, 5),
        "saved item body remained blocked"
    );
    cave_step(&mut loaded, GameButton::Down, (5, 6));
    cave_exit(&mut loaded, &runtime, (5, 12), (5, 13), (60, 34));
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
#[ignore = "requires the published external Voyageurs Border Loop pack"]
fn voyageurs_controller_wins_real_officer_battle_mutates_pp_and_continues_party_flags() {
    let (root, runtime, mut controller) = fixture((10, 43), true);
    let money_before = controller.shell.shell.session().state().money;
    controller
        .press(GameButton::Right)
        .expect("face actual Officer");
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
        BTreeSet::from([0, 1, 2]),
        "native three-partner Officer party was not faced"
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
    assert!(flag(&controller, "EVENT_MN_R20_TRAINER_01"));
    assert!(
        controller.shell.shell.session().state().money > money_before,
        "native victory reward missing"
    );
    // Actual movement after victory and Continue; no state rewrites.
    step(&mut controller, GameButton::Down, (10, 44));
    let party = controller.snapshot().expect("post-victory party").party;
    let save = OwnedSave::new("victory");
    controller.save(&save.0).expect("save actual victory");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.0.clone(), None)
        .expect("Continue actual victory");
    assert_eq!(position(&mut loaded), (10, 44));
    assert_eq!(loaded.snapshot().expect("continued party").party, party);
    assert!(flag(&loaded, "EVENT_MN_R20_TRAINER_01"));
    for index in (1..=10).filter(|index| *index != 1) {
        assert!(
            !flag(&loaded, &format!("EVENT_MN_R20_TRAINER_{index:02}")),
            "one win must not claim a full-route trainer journey"
        );
    }
    step(&mut loaded, GameButton::Up, (10, 43));
    loaded
        .press(GameButton::Right)
        .expect("face defeated Officer");
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
#[ignore = "requires the current R20 grass candidate"]
fn voyageurs_controller_new_grass_camp_climb_dry_drop_wild_run_and_continue() {
    use super::authored_route_fixture::{cleanup_reached_save, escape_wild, reached_save_path};

    // Existing native-created party/badge/position fixtures are setup only.
    // No state is rewritten after setup; every subsequent action uses input.
    let (root, runtime, mut controller) = fixture((19, 52), true);
    let route = grid(&runtime, (19, 52));
    let mut grass = 0;
    route.walk_with_actions(
        &mut controller,
        (19, 49),
        |c| grass += usize::from(route_grass_kind(&runtime, position(c)) == "TALL_GRASS"),
        escape_wild,
    );
    assert!(grass >= 6, "ordinary camp climb must cross its new grass");
    step(&mut controller, GameButton::Down, (19, 50));
    step(&mut controller, GameButton::Down, (19, 52));
    assert_eq!(route_grass_kind(&runtime, (19, 49)), "FLOOR");
    assert_eq!(route_grass_kind(&runtime, (19, 52)), "FLOOR");
    barrier(&mut controller, (19, 52), GameButton::Up);

    route.walk_with_actions(&mut controller, (13, 50), |_| {}, escape_wild);
    assert_eq!(route_grass_kind(&runtime, (13, 50)), "TALL_GRASS");
    assert_eq!(route_grass_kind(&runtime, (13, 49)), "TALL_GRASS");
    let mut encounters = 0;
    for index in 0..256 {
        let target = if index % 2 == 0 { (13, 49) } else { (13, 50) };
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
        "new camp grass must start a native encounter"
    );
    route.walk_with_actions(&mut controller, (19, 36), |_| {}, escape_wild);
    let mut meadow_grass = 0;
    route.walk_with_actions(
        &mut controller,
        (27, 42),
        |c| meadow_grass += usize::from(route_grass_kind(&runtime, position(c)) == "TALL_GRASS"),
        escape_wild,
    );
    assert!(meadow_grass >= 3, "western meadow must enter its habitat");
    let before = controller.snapshot().expect("reached meadow resources");
    let flags = controller.shell.shell.session().state().flags.clone();
    let save = reached_save_path(&format!(
        "geothite-r20-grass-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save actual grass journey");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue actual grass journey");
    let after = loaded.snapshot().expect("continued meadow resources");
    assert_eq!(position(&mut loaded), (27, 42));
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(loaded.shell.shell.session().state().flags, flags);
    if let Some(path) = std::env::var_os("CRYSTAL_R20_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached save");
    }
    cleanup_reached_save(&save);
}

fn route_grass_kind(runtime: &CrystalRuntime, (x, y): Point) -> &str {
    let module = &runtime.data().maps[&format!("MnR20C{:02}R{:02}", x / 64, y / 48)];
    let local_x = usize::try_from(x % 64).expect("nonnegative x");
    let local_y = usize::try_from(y % 48).expect("nonnegative y");
    let block = module.blocks[local_y / 2 * usize::from(module.attributes.width) + local_x / 2];
    &runtime.data().tilesets[&module.attributes.tileset_name].collision[&format!("{block:02x}")]
        [local_y % 2 * 2 + local_x % 2]
}
