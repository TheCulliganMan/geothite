//! Local SQ01 observations and Headbutt through production controller input.
//! Initial position/party/badge/player-ID/RNG setup is explicit. Detroit Lakes
//! reward delivery and earned regional progression are outside these fixtures.
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, LAKE_OBSERVATION_EVENTS, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::encounters::FieldEncounterKind;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, neighbors, opposite, position,
};

const OBSERVATIONS: [(Point, &str); 3] = [
    ((54, 14), "PINE EDGE"),
    ((40, 34), "SPRING WETLAND"),
    ((22, 44), "FLOWER PRAIRIE"),
];

fn fixture(
    at: Point,
    headbutt: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController, RouteGrid) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_PRAIRIE_FOREST_LINK_TEST_PACK")
            .expect("published external Prairie Forest Link pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified R19 pack"),
    )
    .expect("native R19 runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR19C00R{:02}", at.1 / 48),
            tile_x: at.0,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit route fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    if headbutt {
        // Keep original-only source fixtures compatible, while regional review
        // uses a Minnesota partner that can learn Headbutt.
        let species = if runtime.data().pokemon.contains_key("NOTCHBILL") {
            "NOTCHBILL"
        } else {
            "MEW"
        };
        let mut pokemon = runtime
            .data()
            .create_pokemon(species, 60, Dv::default())
            .expect("source-compatible native fixture partner");
        pokemon.moves = vec![LearnedMove {
            name: "HEADBUTT".into(),
            current_pp: runtime.data().moves["HEADBUTT"].pp,
            pp_ups: 0,
        }];
        let state = controller.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(pokemon));
        state.badges.johto = [true, true, true, true, true, true, false, false];
        state.player_id = 4; // Native tree_score(31,20,4)==0; not an earned ID claim.
        state.random_state = Default::default();
        state.sync_party_from_storage();
        *controller.shell.shell.session_mut().divider_mut_for_tests() =
            crystal_core::random::RuntimeDividerSource::replay(
                [255, 0, 53, 0]
                    .into_iter()
                    .chain((0_u32..262144).map(|n| ((n * 73 + 19) % 256) as u8))
                    .collect::<Vec<u8>>(),
            );
    }
    // No gameplay state, position, divider, flags or inventory is rewritten
    // after this initial setup. Headbutt encounters are real native RNG rolls.
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR19",
            columns: 1,
            rows: 2,
            tileset: "johto",
            start: at,
            doors: BTreeMap::new(),
        },
    );
    assert_eq!(position(&mut controller), at);
    (root, runtime, controller, grid)
}

fn press(controller: &mut VisibleShellController, button: GameButton) {
    controller
        .press(button)
        .expect("production prairie/forest input");
    controller
        .wait_frames(16)
        .expect("native field presentation");
    assert_eq!(controller.shell.last_error, None);
}

fn event(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered route event")
}

fn observations(controller: &VisibleShellController) -> [bool; 3] {
    LAKE_OBSERVATION_EVENTS.map(|name| event(controller, name))
}

fn route_progress(controller: &VisibleShellController) -> BTreeMap<String, bool> {
    // The Detroit reward is intentionally unallocated/unattached. Reading its
    // well-formed event identity must remain false; this creates no saved flag.
    LAKE_OBSERVATION_EVENTS
        .into_iter()
        .chain(["EVENT_MN_LAKE_OBSERVATIONS_REWARDED"])
        .map(str::to_owned)
        .chain((1..=6).map(|index| format!("EVENT_MN_R19_TRAINER_{index:02}")))
        .chain((1..=4).map(|index| format!("EVENT_MN_R19_ITEM_{index:02}")))
        .map(|name| {
            let set = event(controller, &name);
            (name, set)
        })
        .collect()
}

fn collision(runtime: &CrystalRuntime, controller: &VisibleShellController, at: Point) -> String {
    let block = controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(at.0 / 2, (at.1 % 48) / 2)
        .expect("current native block");
    runtime.data().tilesets["johto"].collision[&format!("{block:02x}")]
        [(at.1 % 2 * 2 + at.0 % 2) as usize]
        .clone()
}

fn talk_open(controller: &mut VisibleShellController, grid: &RouteGrid, at: Point) {
    let (approach, facing) = neighbors(at)
        .into_iter()
        .find(|(p, _)| {
            grid.floor.contains(p)
                && !grid.blocked.contains(p)
                && !grid.hazards.contains(p)
                && (p.0 / 64, p.1 / 48) == (at.0 / 64, at.1 / 48)
        })
        .expect("ordinary walking approach to observation sign");
    grid.walk(controller, approach);
    press(controller, opposite(facing));
    press(controller, GameButton::A);
    let snapshot = controller.snapshot().expect("visible authored interaction");
    assert_eq!(snapshot.phase, RuntimeShellPhase::Text);
    assert!(snapshot.ui.text.is_some());
    let at = position(controller);
    press(controller, GameButton::Down);
    assert_eq!(
        position(controller),
        at,
        "authored dialogue must own movement"
    );
}

#[derive(Default)]
struct Dialogue {
    text: Vec<String>,
}

impl Dialogue {
    fn has_text(&self, expected: &str) -> bool {
        self.text.iter().any(|text| text.contains(expected))
    }
}

fn close_dialogue(controller: &mut VisibleShellController) -> Dialogue {
    let mut dialogue = Dialogue::default();
    for _ in 0..128 {
        let snapshot = controller.snapshot().expect("native script continuation");
        assert!(
            snapshot.battle.is_none(),
            "walking observation signs must not become a trainer gauntlet"
        );
        assert!(snapshot.ui.pending_yes_no.is_none());
        assert!(snapshot.pending_shop.is_none());
        if let Some(text) = snapshot.ui.text
            && let Some(asm_text) = text.asm_text
        {
            dialogue.text.push(asm_text);
        }
        if snapshot.phase == RuntimeShellPhase::Overworld
            && !snapshot.ui.text_window_open
            && snapshot.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
            && controller.shell.field_notice.is_none()
        {
            return dialogue;
        }
        press(controller, GameButton::A);
    }
    panic!(
        "SQ01 dialogue failed to return walking: {:?}",
        controller.snapshot().expect("diagnosis").ui
    );
}

fn speak(controller: &mut VisibleShellController, grid: &RouteGrid, at: Point) -> Dialogue {
    talk_open(controller, grid, at);
    close_dialogue(controller)
}

// Fixture-only path preference over RouteGrid's already validated clear cells.
// Cost is (entered grass cells, walking distance), so a short10-grass path must
// not hide the actual8-grass alternative. Every chosen edge still executes
// through RouteGrid/VisibleShellController, with native position verification.
fn least_grass_path(
    runtime: &CrystalRuntime,
    grid: &RouteGrid,
    from: Point,
    to: Point,
) -> Vec<Point> {
    let mut costs = BTreeMap::from([(from, (0_u32, 0_u32))]);
    let mut parents = BTreeMap::new();
    let mut queue = BinaryHeap::from([Reverse((0_u32, 0_u32, from))]);
    while let Some(Reverse((grass, steps, point))) = queue.pop() {
        if costs.get(&point) != Some(&(grass, steps)) {
            continue;
        }
        if point == to {
            break;
        }
        for (next, _) in neighbors(point) {
            if !grid.floor.contains(&next)
                || grid.blocked.contains(&next)
                || grid.hazards.contains(&next)
            {
                continue;
            }
            let map = &runtime.data().maps[&format!("MnR19C00R{:02}", next.1 / 48)];
            let block = map.blocks[(next.1 as usize % 48) / 2 * usize::from(map.attributes.width)
                + next.0 as usize / 2];
            let kind = &runtime.data().tilesets["johto"].collision[&format!("{block:02x}")]
                [(next.1 % 2 * 2 + next.0 % 2) as usize];
            let cost = (grass + u32::from(kind == "TALL_GRASS"), steps + 1);
            if costs.get(&next).is_none_or(|previous| cost < *previous) {
                costs.insert(next, cost);
                parents.insert(next, point);
                queue.push(Reverse((cost.0, cost.1, next)));
            }
        }
    }
    assert!(costs.contains_key(&to), "no nonbattle grass alternative");
    let mut path = Vec::new();
    let mut cursor = to;
    while cursor != from {
        path.push(cursor);
        cursor = parents[&cursor];
    }
    path.reverse();
    path
}

#[test]
#[ignore = "requires the published external Prairie Forest Link pack"]
fn prairie_forest_link_controller_walks_two_ramp_loop_reads_observations_and_continues() {
    let (root, runtime, mut controller, grid) = fixture((56, 12), false);
    assert!(
        controller
            .snapshot()
            .expect("empty walking party")
            .party
            .slots
            .is_empty()
    );
    for at in [
        (60, 12),
        (24, 56),
        (24, 47),
        (24, 48),
        (24, 47),
        (24, 48),
        (24, 52),
    ] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("clipped walking seam")
                .overworld
                .map_name,
            format!("MnR19C00R{:02}", at.1 / 48)
        );
    }
    for (from, to) in [((49, 23), (39, 23)), ((39, 23), (49, 23))] {
        grid.walk(&mut controller, from);
        let mut grass_steps = 0;
        for target in least_grass_path(&runtime, &grid, from, to) {
            grid.walk(&mut controller, target);
            grass_steps += usize::from(collision(&runtime, &controller, target) == "TALL_GRASS");
        }
        assert_eq!(position(&mut controller), to);
        assert_eq!(grass_steps, 8, "actual two-way nonbattle grass alternative");
        assert!(!event(&controller, "EVENT_MN_R19_TRAINER_01"));
    }
    for y in 47..=52 {
        grid.walk(&mut controller, (24, y));
        if (48..52).contains(&y) {
            assert_eq!(collision(&runtime, &controller, (24, y)), "TALL_GRASS");
        }
    }
    for y in (47..52).rev() {
        grid.walk(&mut controller, (24, y));
    }
    for x in [11, 19] {
        for y in (35..=38).rev() {
            grid.walk(&mut controller, (x, y));
            assert!(!collision(&runtime, &controller, (x, y)).starts_with("HOP"));
            if y < 38 {
                assert_eq!(collision(&runtime, &controller, (x, y)), "FLOOR");
            }
        }
        for y in 36..=38 {
            grid.walk(&mut controller, (x, y));
        }
    }
    let upper = [(11, 35), (12, 35), (12, 32), (19, 32), (19, 35)];
    let mut steps = 0;
    grid.walk(&mut controller, upper[0]);
    for pair in upper.windows(2) {
        steps += grid.path(pair[0], pair[1]).len();
        grid.walk(&mut controller, pair[1]);
    }
    assert_eq!(steps, 14, "actual upper inspection connection");
    for at in upper.into_iter().rev() {
        grid.walk(&mut controller, at);
    }
    grid.walk(&mut controller, (11, 38));
    grid.walk(&mut controller, (19, 38));
    for (at, text) in [
        ((36, 20), "LAKE ITASCA"),
        ((14, 36), "WESTERN SPUR"),
        ((26, 52), "DETROIT LAKES"),
        ((27, 29), "Pines give way"),
        ((27, 39), "prairie meets"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "ordinary interaction {at:?}"
        );
    }
    for at in [(11, 29), (53, 13), (29, 29), (33, 41)] {
        assert!(grid.talk(&mut controller, at).contains("put the"));
    }
    for n in 1..=4 {
        assert!(event(&controller, &format!("EVENT_MN_R19_ITEM_{n:02}")));
    }
    let bag = controller.snapshot().expect("actual route pickups").bag;
    for id in ["HYPER_POTION", "ULTRA_BALL", "MAX_REPEL", "REVIVE"] {
        assert!(
            bag.items
                .iter()
                .chain(&bag.balls)
                .any(|s| s.item_id == id && s.quantity == 1),
            "{id}"
        );
    }
    assert_eq!(observations(&controller), [false; 3]);
    for (index, (at, text)) in OBSERVATIONS.into_iter().enumerate() {
        talk_open(&mut controller, &grid, at);
        assert!(
            !event(&controller, LAKE_OBSERVATION_EVENTS[index]),
            "observation must wait for acknowledgment"
        );
        assert!(close_dialogue(&mut controller).has_text(text));
        let expected = [true, index >= 1, index >= 2];
        assert_eq!(observations(&controller), expected);
        assert_eq!(
            controller
                .snapshot()
                .expect("observations grant no item")
                .bag,
            bag
        );
        assert!(speak(&mut controller, &grid, at).has_text(text));
        assert_eq!(observations(&controller), expected);
        assert_eq!(
            controller
                .snapshot()
                .expect("repeat observation is idempotent")
                .bag,
            bag
        );
    }
    // Detroit's reward is not attached to these native maps. Reading signs
    // must not invent its hand-in, item grant or completion script.
    for map in runtime
        .data()
        .maps
        .values()
        .filter(|map| map.id.starts_with("MnR19"))
    {
        assert!(map.script_item_grants.is_empty());
        assert!(
            !map.scripts
                .contains_key("MN_DETROIT_LAKE_OBSERVATION_RANGER")
        );
        assert!(
            !map.script_flag_commands
                .iter()
                .any(|c| c.flag_id == "EVENT_MN_LAKE_OBSERVATIONS_REWARDED")
        );
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("ordinary observation mode")
            .overworld
            .mode,
        MovementMode::Normal
    );
    grid.walk(&mut controller, (24, 54));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
    let before = controller
        .snapshot()
        .expect("actual walking observation state");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r19-observations-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller.save(&path).expect("save actual observations");
    let mut loaded = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("Continue observations");
    let after = loaded.snapshot().expect("continued notes/resources");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(observations(&loaded), [true; 3]);
    for n in 1..=4 {
        assert!(event(&loaded, &format!("EVENT_MN_R19_ITEM_{n:02}")));
    }
    assert!(speak(&mut loaded, &grid, OBSERVATIONS[0].0).has_text(OBSERVATIONS[0].1));
    assert_eq!(
        loaded
            .snapshot()
            .expect("continued observation cannot reward")
            .bag,
        bag
    );
    if let Some(save) = std::env::var_os("CRYSTAL_PRAIRIE_FOREST_LINK_GEOMETRY_SAVE") {
        loaded
            .save(PathBuf::from(save))
            .expect("export walked observation save");
    }
    std::fs::remove_file(path).expect("remove unique temporary observation save");
}

#[test]
#[ignore = "requires the published external Prairie Forest Link pack"]
fn prairie_forest_link_controller_oak_without_headbutt_does_not_dispatch_or_mutate_tree() {
    let (_, runtime, mut controller, _) = fixture((32, 20), false);
    assert_eq!(collision(&runtime, &controller, (31, 20)), "HEADBUTT_TREE");
    let blocks = controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_ids
        .clone();
    let flags = controller.shell.shell.session().state().flags.clone();
    let party = controller.snapshot().expect("no move fixture").party;
    press(&mut controller, GameButton::Left);
    assert_eq!(position(&mut controller), (32, 20));
    press(&mut controller, GameButton::A);
    for _ in 0..12 {
        let s = controller.snapshot().expect("native no-Headbutt result");
        assert!(s.battle.is_none());
        assert!(s.ui.pending_yes_no.is_none());
        if controller.shell.field_notice.is_none() && !controller.has_pending_script_work() {
            break;
        }
        press(&mut controller, GameButton::B);
    }
    assert_eq!(position(&mut controller), (32, 20));
    assert_eq!(
        controller
            .snapshot()
            .expect("unchanged no-move party")
            .party,
        party
    );
    assert_eq!(controller.shell.shell.session().state().flags, flags);
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .overworld()
            .map
            .metatile_ids,
        blocks
    );
}

#[test]
#[ignore = "requires the published external Prairie Forest Link pack"]
fn prairie_forest_link_controller_native_headbutt_starts_tree_battle_spends_pp_and_continues() {
    let (root, runtime, mut controller, _) = fixture((32, 20), true);
    assert_eq!(collision(&runtime, &controller, (31, 20)), "HEADBUTT_TREE");
    let blocks = controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_ids
        .clone();
    let progress = route_progress(&controller);
    assert!(progress.values().all(|set| !set));
    let badges = controller.shell.shell.session().state().badges.clone();
    let money = controller.shell.shell.session().state().money;
    press(&mut controller, GameButton::Left);
    assert_eq!(position(&mut controller), (32, 20));
    // Repeated native A/YES actions allow genuine source tree misses. The DIV
    // stream is installed once during initial setup and never reset on a miss.
    for _ in 0..1024 {
        if controller
            .snapshot()
            .expect("Headbutt source activity")
            .battle
            .is_some()
        {
            break;
        }
        let button = if controller
            .shell
            .yes_no_cursor
            .as_ref()
            .is_some_and(|cursor| cursor.option_index != 0)
        {
            GameButton::Up
        } else {
            GameButton::A
        };
        press(&mut controller, button);
    }
    let initial = controller.snapshot().expect("actual tree encounter");
    let battle = initial
        .battle
        .as_ref()
        .expect("native Headbutt must eventually produce an encounter");
    assert_eq!(battle.battle_type, "BATTLETYPE_TREE");
    assert!(
        matches!(&battle.kind,RuntimeBattleKind::StaticWild{origin_map_name,source_script,startbattle_command_index,resume_command_index,..}
        if origin_map_name=="MnR19C00R00"&&source_script=="HeadbuttScript"&&*startbattle_command_index==8&&*resume_command_index==9)
    );
    // The source encounter rolls level28 before the inherited default-battle
    // policy materializes it. Preserve and test both boundaries: a level60
    // fixture partner scales this battle to59 under the current native rules.
    let memory = &controller
        .shell
        .shell
        .session()
        .state()
        .script_runtime
        .memory;
    assert_eq!(memory.get("wCurPartyLevel").map(String::as_str), Some("28"));
    assert_eq!(
        memory.get("wTempWildMonSpecies").map(String::as_str),
        Some(battle.enemy_pokemon.species.id.as_str())
    );
    let party: Vec<_> = initial
        .party
        .slots
        .iter()
        .map(|slot| slot.pokemon.clone())
        .collect();
    // start_scripted_wild_battle uses the native scripted variation2, rather
    // than the field roll's entry selector, when applying this existing policy.
    let expected_level = runtime
        .data()
        .wild_challenge
        .map_or(28, |rules| rules.encounter_level(&party, 28, 2));
    assert_eq!(battle.enemy_pokemon.level, expected_level);
    let tree_table = runtime.data().field_encounters["MnR19C00R00"]
        .table(FieldEncounterKind::Headbutt)
        .expect("native per-map Headbutt table");
    assert!(
        tree_table
            .common
            .iter()
            .chain(&tree_table.rare)
            .all(|entry| entry.level == 28)
    );
    assert!(
        tree_table
            .rare
            .iter()
            .any(|entry| entry.species == battle.enemy_pokemon.species.id && entry.level == 28),
        "initial playerID4 makes this a real rare-score tree"
    );
    let origin = controller
        .battle_presentation_origin()
        .expect("native field battle provenance");
    assert_eq!(origin.source.map_name, "MnR19C00R00");
    assert_eq!((origin.source.tile.x, origin.source.tile.y), (32, 20));
    let initial_pp = initial.party.slots[0].pokemon.moves[0].current_pp;
    let initial_exp = initial.party.slots[0].pokemon.experience;
    let mut selected = 0;
    for _ in 0..1024 {
        let snapshot = controller.snapshot().expect("production tree battle");
        let Some(battle) = &snapshot.battle else {
            break;
        };
        assert_eq!(battle.battle_type, "BATTLETYPE_TREE");
        let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            controller.wait_frames(30).expect("battle narration");
            GameButton::A
        } else if let Some(cursor) = &controller.shell.battle_move_cursor {
            assert_eq!(battle.player_moves[0].name, "HEADBUTT");
            assert!(battle.player_moves[0].current_pp > 0);
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                selected += 1;
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_action_cursor {
            match cursor.option_index {
                0 => GameButton::A,
                1 => GameButton::Left,
                2 | 3 => GameButton::Up,
                _ => panic!("invalid battle action cursor"),
            }
        } else {
            GameButton::A
        };
        press(&mut controller, button);
    }
    finish_text(&mut controller);
    let before = controller.snapshot().expect("won actual tree battle");
    assert!(before.battle.is_none());
    assert_eq!(position(&mut controller), (32, 20));
    assert!(selected > 0);
    assert!(before.party.slots[0].pokemon.hp > 0);
    assert!(
        before.party.slots[0].pokemon.moves[0].current_pp < initial_pp,
        "battle Headbutt selection must spend real PP"
    );
    assert!(
        before.party.slots[0].pokemon.experience > initial_exp,
        "wild battle must award actual victory experience"
    );
    assert_eq!(
        controller
            .shell
            .shell
            .session()
            .overworld()
            .map
            .metatile_ids,
        blocks,
        "Headbutt cannot Cut/replace the oak"
    );
    assert_eq!(
        route_progress(&controller),
        progress,
        "tree battle cannot complete observations/reward, trainers or pickups"
    );
    assert_eq!(controller.shell.shell.session().state().badges, badges);
    assert_eq!(controller.shell.shell.session().state().money, money);
    assert_eq!(before.bag, initial.bag);
    assert_eq!(before.overworld.mode, MovementMode::Normal);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r19-headbutt-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    controller.save(&path).expect("production tree battle save");
    let mut loaded = VisibleShellController::load_save(root, runtime, path.clone(), None)
        .expect("Continue Headbutt victory");
    let after = loaded.snapshot().expect("continued field victory");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(route_progress(&loaded), progress);
    assert_eq!(loaded.shell.shell.session().state().badges, badges);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    if let Some(save) = std::env::var_os("CRYSTAL_PRAIRIE_FOREST_LINK_HEADBUTT_SAVE") {
        loaded
            .save(PathBuf::from(save))
            .expect("export actual tree victory save");
    }
    std::fs::remove_file(path).expect("remove unique temporary Headbutt save");
}
