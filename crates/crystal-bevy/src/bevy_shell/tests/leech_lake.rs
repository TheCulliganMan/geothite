//! Populated local route journey, not earned badges, regional story or balance.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty, visible_field_dialog_pages,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, neighbors, opposite, position,
};

fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = std::env::var_os("CRYSTAL_LEECH_LAKE_TEST_PACK")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content-packs/minnesota-boundary-waters.crystalpack")
        })
        .canonicalize()
        .expect("published native R07 composition");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified external native pack"),
    )
    .expect("native lake runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("original HOME"),
            map_name: "MnR07C00R01".into(),
            tile_x: 20,
            tile_y: 40,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit south-arrival fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    // Initial setup only: source-created partners, native-legal Mew CUT and
    // eight obedience badges. This does not claim an earned team, HM or badge.
    // After this block, no party/position/flags/inventory are injected.
    let mut partners = ["TYPHLOSION", "FERALIGATR", "MEW"].map(|species| {
        runtime
            .data()
            .create_pokemon(species, 60, Dv::default())
            .expect("legal source-created partner")
    });
    partners[2].moves.truncate(3);
    partners[2].moves.push(LearnedMove {
        name: "CUT".into(),
        current_pp: runtime.data().moves["CUT"].pp,
        pp_ups: 0,
    });
    let state = controller.shell.shell.session_mut().state_mut();
    assert_eq!(state.storage.party.filled_slots(), 0);
    for partner in partners {
        assert!(state.storage.party.add_pokemon(partner));
    }
    state.badges.johto = [true; 8];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<u8>>(),
        );
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.set_runtime_journal_enabled(false);
    assert_eq!(position(&mut controller), (20, 88));
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR07",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start: (20, 88),
            doors: BTreeMap::new(),
        },
    )
}

fn flag(controller: &VisibleShellController, name: &str) -> bool {
    controller
        .shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("registered source event")
}

fn quantity(controller: &mut VisibleShellController, item: &str) -> u16 {
    let bag = controller.snapshot().expect("native bag").bag;
    bag.items
        .iter()
        .chain(&bag.balls)
        .filter(|slot| slot.item_id == item)
        .map(|slot| slot.quantity)
        .sum()
}

fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let name = format!("MnR07C00R{:02}", at.1 / 48);
    let module = &runtime.data().maps[&name];
    let x = at.0 as usize;
    let y = (at.1 % 48) as usize;
    let block = module.blocks[y / 2 * usize::from(module.attributes.width) + x / 2];
    &runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn current_tree(controller: &VisibleShellController) -> u16 {
    assert_eq!(
        controller.shell.shell.session().overworld().map.name,
        "MnR07C00R00"
    );
    controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(7, 11)
        .expect("native bird-hollow tree")
}

#[derive(Default)]
struct Journey {
    wild_battles: usize,
    trainer_flags: BTreeSet<String>,
    selected_pp_spent: usize,
    meadow_steps: usize,
    seam_directions: BTreeSet<(i16, i16)>,
}

struct BattleProof {
    opponents: BTreeSet<(usize, String, u8)>,
    selected_pp_spent: bool,
}

struct OwnedSave(PathBuf);
impl OwnedSave {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "geothite-r07-journey-{}-{stamp}.crystalsave",
            std::process::id()
        )))
    }
}
impl Drop for OwnedSave {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
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

fn resolve_battle(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    expected_flag: Option<&str>,
) -> BattleProof {
    let mut entered = false;
    let mut opponents = BTreeSet::new();
    let mut selected = None;
    let mut pp_spent = false;
    for _ in 0..1024 {
        let snapshot = controller.snapshot().expect("native battle owner");
        let Some(battle) = &snapshot.battle else {
            if entered {
                break;
            }
            controller
                .press(GameButton::A)
                .expect("trainer introduction");
            continue;
        };
        entered = true;
        match (&battle.kind, expected_flag) {
            (RuntimeBattleKind::Trainer { event_flag, .. }, Some(expected)) => {
                assert_eq!(event_flag, expected)
            }
            (RuntimeBattleKind::Wild { .. }, None) => {}
            _ => panic!("unexpected native battle kind for {expected_flag:?}"),
        }
        if let Some(index) = battle.active_enemy_party_index {
            opponents.insert((
                index,
                battle.enemy_pokemon.species.id.clone(),
                battle.enemy_pokemon.level,
            ));
        }
        let button = battle_button(controller, runtime, &snapshot);
        if selected.is_none()
            && button == GameButton::A
            && controller.shell.battle_messages.is_empty()
            && snapshot.ui.menu.is_some()
            && let Some(cursor) = &controller.shell.battle_move_cursor
        {
            let movement = &battle.player_moves[cursor.option_index];
            selected = Some((
                battle.active_player_party_index.expect("active partner"),
                movement.name.clone(),
                movement.current_pp,
            ));
        }
        controller.press(button).expect("production battle input");
        if let Some((index, name, before)) = &selected {
            let now = controller.snapshot().expect("actual PP mutation");
            pp_spent |= now.party.slots[*index]
                .pokemon
                .moves
                .iter()
                .any(|movement| movement.name == *name && movement.current_pp < *before);
        }
    }
    let ended = controller.snapshot().expect("battle resolution");
    assert!(
        entered && ended.battle.is_none(),
        "battle did not finish: {}",
        battle_note(controller, &ended)
    );
    finish_text(controller);
    if let Some(event) = expected_flag {
        assert!(
            flag(controller, event),
            "actual trainer victory flag missing: {event}"
        );
        assert!(
            selected.is_some() && pp_spent,
            "selected native move did not spend PP against {event}"
        );
    }
    BattleProof {
        opponents,
        selected_pp_spent: pp_spent,
    }
}

fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .into_iter()
        .find_map(|(point, button)| (point == to).then_some(button))
        .expect("adjacent native tile")
}

fn moved(before: Point, button: GameButton) -> Point {
    neighbors(before)
        .into_iter()
        .find_map(|(point, value)| (button == value).then_some(point))
        .expect("directional movement")
}

fn step(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    journey: &mut Journey,
    button: GameButton,
    target: Point,
) {
    let before = position(controller);
    for _ in 0..12 {
        let owner = controller.snapshot().expect("walking input owner");
        if owner.phase == RuntimeShellPhase::Text && owner.battle.is_none() {
            // Ordinary walking can produce native poison/faint notices. Their
            // visible pages own input just as authored dialogue does; another
            // direction cannot dismiss them. Acknowledge them through A.
            eprintln!("R07 walking text at {before:?}: {:?}", owner.ui.text);
            finish_text(controller);
            assert_eq!(
                position(controller),
                before,
                "walking notice changed the route position"
            );
        }
        controller
            .press(button)
            .expect("native populated-route movement");
        if controller
            .snapshot()
            .expect("walking battle check")
            .battle
            .is_some()
        {
            resolve_battle(controller, runtime, None);
            journey.wild_battles += 1;
        }
        let at = position(controller);
        if at == target {
            if before.1 / 48 != at.1 / 48 {
                journey.seam_directions.insert((before.1 / 48, at.1 / 48));
            }
            if ((44..56).contains(&at.0) && (42..50).contains(&at.1))
                || ((42..44).contains(&at.0) && (48..50).contains(&at.1))
            {
                assert_eq!(kind(runtime, at), "TALL_GRASS");
                journey.meadow_steps += 1;
            }
            return;
        }
        assert_eq!(
            at, before,
            "unexpected move {before:?} -> {at:?}, wanted {target:?}"
        );
    }
    let owner = controller.snapshot().expect("blocked walking input owner");
    panic!(
        "native route step {before:?} -> {target:?} blocked: phase={:?}; ui={:?}; objects={:?}",
        owner.phase, owner.ui, owner.visible_object_runtime_tiles
    );
}

fn walk(
    grid: &RouteGrid,
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    journey: &mut Journey,
    target: Point,
) {
    for button in grid.path(position(controller), target) {
        let at = position(controller);
        step(controller, runtime, journey, button, moved(at, button));
    }
    assert_eq!(position(controller), target);
}

fn approach(grid: &RouteGrid, at: Point) -> (Point, GameButton) {
    let (point, away) = neighbors(at)
        .into_iter()
        .find(|(point, _)| {
            grid.floor.contains(point)
                && !grid.blocked.contains(point)
                && !grid.hazards.contains(point)
        })
        .expect("same-map source interaction approach");
    assert_eq!(point.1 / 48, at.1 / 48);
    (point, opposite(away))
}

fn talk(
    grid: &RouteGrid,
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    journey: &mut Journey,
    at: Point,
) -> String {
    let (from, button) = approach(grid, at);
    walk(grid, controller, runtime, journey, from);
    controller.press(button).expect("face source interaction");
    controller
        .press(GameButton::A)
        .expect("source speech or resource");
    let snapshot = controller.snapshot().expect("native interaction text");
    assert!(
        snapshot.battle.is_none(),
        "unexpected trainer instead of interaction at {at:?}"
    );
    assert_eq!(snapshot.phase, RuntimeShellPhase::Text);
    // Native item presentation retains its actual first page while the LCD
    // can already show the following "put ... in PACK" page.
    let first_item_page = controller
        .shell
        .visible_field_item_notice
        .as_ref()
        .map(|notice| notice.sound_trigger_text.clone());
    controller
        .press(GameButton::Down)
        .expect("try movement while dialogue owns input");
    assert_eq!(position(controller), from);
    // Controller snapshots intentionally expose only the current LCD page.
    // Observe each page after real A presses rather than treating that first
    // page as the complete authored dialogue or reading ahead from the catalog.
    let mut streams = BTreeMap::<String, (usize, BTreeSet<usize>)>::new();
    let mut observed = BTreeSet::new();
    let mut printed = Vec::new();
    for _ in 0..128 {
        let visible = controller.snapshot().expect("actual visible dialogue page");
        if visible.phase == RuntimeShellPhase::Overworld
            && visible.ui.pending_text_wait.is_none()
            && !controller.has_pending_script_work()
        {
            assert!(!printed.is_empty(), "no visible text was read at {at:?}");
            for (identity, (count, seen)) in &streams {
                assert_eq!(
                    *seen,
                    (0..*count).collect::<BTreeSet<_>>(),
                    "dialogue at {at:?} skipped an input-gated page in {identity:?}"
                );
            }
            let text = first_item_page
                .iter()
                .chain(printed.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!(
                "R07 text at {at:?}: {text:?} ({} observed pages)",
                printed.len()
            );
            return text;
        }
        assert_eq!(position(controller), from, "text moved its reader");
        let source = controller
            .shell
            .shell
            .presentation_snapshot()
            .expect("live text owner");
        if let Some(pages) = visible_field_dialog_pages(&source, &controller.shell)
            && let Some(reveal) = &controller.shell.field_text_reveal
            && reveal.text == pages.join("\u{1e}")
        {
            let index = reveal.page_index;
            let identity = reveal.text.clone();
            let actual = visible
                .ui
                .text
                .as_ref()
                .and_then(|text| text.asm_text.as_ref())
                .expect("controller must project the actual current LCD page");
            assert_eq!(actual, &pages[index], "wrong visible page at {at:?}");
            streams
                .entry(identity.clone())
                .or_insert_with(|| (pages.len(), BTreeSet::new()))
                .1
                .insert(index);
            if observed.insert((identity, index)) {
                printed.push(actual.clone());
            }
        }
        controller
            .press(GameButton::A)
            .expect("reveal or acknowledge actual dialogue page");
    }
    panic!("dialogue at {at:?} did not finish; observed={printed:?}");
}

fn trainer(
    grid: &RouteGrid,
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    journey: &mut Journey,
    index: u8,
    at: Point,
    expected: [(&str, u8); 2],
) {
    let event = format!("EVENT_MN_R07_TRAINER_{index:02}");
    assert!(
        !flag(controller, &event),
        "trainer was defeated outside this journey"
    );
    let (from, button) = approach(grid, at);
    walk(grid, controller, runtime, journey, from);
    let money = controller.shell.shell.session().state().money;
    controller
        .press(button)
        .expect("face actual source trainer");
    controller
        .press(GameButton::A)
        .expect("start actual native trainer");
    let proof = resolve_battle(controller, runtime, Some(&event));
    assert_eq!(
        proof.opponents,
        expected
            .into_iter()
            .enumerate()
            .map(|(i, (species, level))| (i, species.into(), level))
            .collect::<BTreeSet<_>>()
    );
    assert!(
        controller.shell.shell.session().state().money > money,
        "actual trainer victory reward missing"
    );
    assert!(journey.trainer_flags.insert(event));
    journey.selected_pp_spent += usize::from(proof.selected_pp_spent);
}

fn pickup(
    grid: &mut RouteGrid,
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    journey: &mut Journey,
    at: Point,
    item: &str,
    event: &str,
) {
    let before = quantity(controller, item);
    assert!(!flag(controller, event));
    assert!(talk(grid, controller, runtime, journey, at).contains("found"));
    assert_eq!(quantity(controller, item), before + 1);
    assert!(flag(controller, event));
    // Only the fixture planner changes: the native pickup flag has already
    // hidden its body through production input. Walk the vacated tile itself.
    grid.blocked.remove(&at);
    grid.floor.insert(at);
    walk(grid, controller, runtime, journey, at);
}

fn cut_and_open_cache(
    grid: &mut RouteGrid,
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    journey: &mut Journey,
) {
    walk(grid, controller, runtime, journey, (16, 23));
    assert_eq!(current_tree(controller), 0x5f);
    assert!(
        !grid.floor.contains(&(14, 23)),
        "uncut cache should be sealed"
    );
    controller
        .press(GameButton::Left)
        .expect("face actual Cut gate");
    assert_eq!(position(controller), (16, 23));
    for _ in 0..64 {
        if current_tree(controller) == 0x3d {
            break;
        }
        controller
            .press(GameButton::A)
            .expect("actual Cut confirmation and use");
    }
    assert_eq!(current_tree(controller), 0x3d);
    finish_text(controller);
    controller.wait_frames(60).expect("native Cut presentation");
    // Read the actually changed active map into the planner. Never replace
    // gameplay blocks or relocate the player through the fixture.
    let map = &controller.shell.shell.session().overworld().map;
    let module = &runtime.data().maps["MnR07C00R00"];
    for y in 0..module.attributes.height as i16 * 2 {
        for x in 0..module.attributes.width as i16 * 2 {
            let block = map.metatile_at(x / 2, y / 2).expect("active native block");
            let kind = &runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")]
                [(y % 2 * 2 + x % 2) as usize];
            if matches!(kind.as_str(), "FLOOR" | "TALL_GRASS") {
                grid.floor.insert((x, y));
            }
        }
    }
    walk(grid, controller, runtime, journey, (14, 23));
    pickup(
        grid,
        controller,
        runtime,
        journey,
        (13, 23),
        "ETHER",
        "EVENT_MN_R07_ITEM_04",
    );
    walk(grid, controller, runtime, journey, (16, 23));
}

#[test]
#[ignore = "requires the published external Minnesota pack containing current R07"]
fn leech_lake_controller_populated_round_trip_battles_resources_cut_and_continue() {
    let (root, runtime, mut controller) = fixture();
    let native = runtime
        .data()
        .maps
        .iter()
        .filter(|(name, _)| name.starts_with("MnR07"))
        .collect::<Vec<_>>();
    assert_eq!(
        native.len(),
        2,
        "R07 has two exteriors and no invented room"
    );
    assert!(
        native
            .iter()
            .all(|(_, module)| module.events.warps.is_empty()),
        "R07 source has no enterable facade/interior"
    );
    let mut grid = grid(&runtime);
    let mut journey = Journey::default();
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(
        controller
            .snapshot()
            .expect("Start owner")
            .ui
            .menu
            .is_some()
    );
    controller.press(GameButton::B).expect("close Start");
    assert!(
        talk(&grid, &mut controller, &runtime, &mut journey, (23, 89)).contains("railed crossing")
    );
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        1,
        (13, 83),
        [("GILLPUP", 27), ("CHORIP", 27)],
    );
    pickup(
        &mut grid,
        &mut controller,
        &runtime,
        &mut journey,
        (11, 79),
        "SUPER_POTION",
        "EVENT_MN_R07_ITEM_01",
    );
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        2,
        (19, 63),
        [("CHORIP", 27), ("FLOSSAIL", 27)],
    );
    // Required journey geometry: actual long southern causeway, native seam
    // through the paid grass neck, and the separate narrow northern crossing.
    for at in [(24, 69), (45, 69)] {
        walk(&grid, &mut controller, &runtime, &mut journey, at);
    }
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        3,
        (55, 65),
        [("FLOSSAIL", 28), ("CLINKLAW", 28)],
    );
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        4,
        (45, 51),
        [("CLINKLAW", 28), ("RICELING", 28)],
    );
    for at in [(49, 48), (49, 47)] {
        walk(&grid, &mut controller, &runtime, &mut journey, at);
    }
    assert!(
        talk(&grid, &mut controller, &runtime, &mut journey, (51, 39))
            .contains("No SURF is needed")
    );
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        5,
        (51, 31),
        [("RICELING", 29), ("RIVVIT", 29)],
    );
    pickup(
        &mut grid,
        &mut controller,
        &runtime,
        &mut journey,
        (49, 27),
        "GREAT_BALL",
        "EVENT_MN_R07_ITEM_02",
    );
    for at in [(45, 37), (22, 37)] {
        walk(&grid, &mut controller, &runtime, &mut journey, at);
    }
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        6,
        (13, 31),
        [("GILLPUP", 29), ("WINDHOPPER", 29)],
    );
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        7,
        (41, 15),
        [("CHORIP", 30), ("GILLPUP", 30)],
    );
    trainer(
        &grid,
        &mut controller,
        &runtime,
        &mut journey,
        8,
        (13, 13),
        [("FLOSSAIL", 30), ("CHORIP", 30)],
    );
    pickup(
        &mut grid,
        &mut controller,
        &runtime,
        &mut journey,
        (25, 13),
        "REPEL",
        "EVENT_MN_R07_ITEM_03",
    );
    cut_and_open_cache(&mut grid, &mut controller, &runtime, &mut journey);
    walk(&grid, &mut controller, &runtime, &mut journey, (20, 6));
    for (at, text) in [
        ((22, 90), "LEECH LAKE TRAIL"),
        ((12, 78), "OUTLET REST"),
        ((20, 40), "LOW SANDBANK"),
        ((44, 64), "PINE PENINSULA"),
        ((54, 50), "REED MEADOW"),
        ((22, 4), "LEECH LAKE NORTH"),
        ((28, 12), "FISHING LANDING"),
        ((13, 39), "Two bridges"),
    ] {
        assert!(talk(&grid, &mut controller, &runtime, &mut journey, at).contains(text));
    }
    // Actual native bank hop and reverse rejection on the southbound return.
    walk(&grid, &mut controller, &runtime, &mut journey, (17, 53));
    let mut saw_face = false;
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native one-way sandbank");
        let at = position(&mut controller);
        saw_face |= at == (17, 54);
        assert!(
            matches!(at, (17, 53) | (17, 54) | (17, 56)),
            "wrong ledge landing {at:?}"
        );
        if at == (17, 56) {
            break;
        }
    }
    assert!(saw_face);
    assert_eq!(position(&mut controller), (17, 56));
    for _ in 0..3 {
        controller
            .press(GameButton::Up)
            .expect("reverse bank rejection");
        assert_eq!(position(&mut controller), (17, 56));
    }
    walk(&grid, &mut controller, &runtime, &mut journey, (20, 88));
    assert_eq!(journey.trainer_flags.len(), 8);
    assert_eq!(
        journey.selected_pp_spent, 8,
        "all 8 actual trainer battles must spend selected PP"
    );
    assert!(
        journey.meadow_steps >= 8,
        "required route meadow was skipped"
    );
    assert_eq!(journey.seam_directions, BTreeSet::from([(0, 1), (1, 0)]));
    eprintln!(
        "R07 populated journey: {} trainer victories with selected PP, {} ordinary wild battles, {} required-meadow steps",
        journey.selected_pp_spent, journey.wild_battles, journey.meadow_steps
    );
    let saved = controller
        .snapshot()
        .expect("completed populated round trip");
    assert!(saved.battle.is_none());
    let money = controller.shell.shell.session().state().money;
    let save = OwnedSave::new();
    controller.save(&save.0).expect("save real journey");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), save.0.clone(), None)
        .expect("Continue actual populated journey");
    assert_eq!(position(&mut loaded), (20, 88));
    let continued = loaded
        .snapshot()
        .expect("continued actual team and resources");
    assert_eq!(continued.party, saved.party);
    assert_eq!(continued.bag, saved.bag);
    assert_eq!(loaded.shell.shell.session().state().money, money);
    for i in 1..=8 {
        assert!(flag(&loaded, &format!("EVENT_MN_R07_TRAINER_{i:02}")));
    }
    for i in 1..=4 {
        assert!(flag(&loaded, &format!("EVENT_MN_R07_ITEM_{i:02}")));
    }
    if let Some(path) = std::env::var_os("CRYSTAL_LEECH_LAKE_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export reached populated R07 save");
    }
    assert!(
        talk(&grid, &mut loaded, &runtime, &mut journey, (13, 83)).contains("Listen before you")
    );
    assert!(
        loaded
            .snapshot()
            .expect("defeated trainer repeat")
            .battle
            .is_none()
    );
}
