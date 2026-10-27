//! Explicit local fixtures followed exclusively by production controller input.
//!
//! Position, empty geometry party or legal populated party, injury, seven badges,
//! divider are setup. The regional report gate is not implemented by this fixture.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, position,
};

fn fixture(at: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_IRON_RANGE_TEST_PACK")
            .expect("published external Iron Range pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(&path)
            .expect("verified native pack"),
    )
    .expect("route runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR09C{:02}R{:02}", at.0 / 64, at.1 / 48),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit route placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime, start: Point, grass: bool) -> RouteGrid {
    let spec = RouteSpec {
        prefix: "MnR09",
        columns: 2,
        rows: 2,
        tileset: "mn_iron_range",
        start,
        doors: BTreeMap::from([((40, 41), "MnR09Cave1F")]),
    };
    if grass {
        RouteGrid::new_with_grass(runtime, spec)
    } else {
        RouteGrid::new(runtime, spec)
    }
}

fn terrain(runtime: &CrystalRuntime, x: usize, y: usize) -> String {
    let module = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&format!("MnR09C{:02}R{:02}", x / 64, y / 48))
        .expect("native clipped chunk");
    let block = module.blocks[(y % 48) / 2 * usize::from(module.attributes.width) + (x % 64) / 2];
    runtime.data().tilesets["mn_iron_range"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
        .clone()
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = position(controller);
    for _ in 0..3 {
        controller.press(button).expect("native collision attempt");
    }
    assert_eq!(position(controller), before, "native barrier moved player");
}

#[test]
#[ignore = "requires the external Iron Range pack"]
fn iron_range_controller_tours_clipped_seams_rails_drop_rooms_and_open_rewards() {
    let (_, runtime, mut controller) = fixture((2, 50));
    // Empty-party geometry only. Never use it to claim encounter/battle balance.
    let grid = grid(&runtime, (2, 50), true);
    assert!(
        !grid.floor.contains(&(12, 63)),
        "Cut camp must stay excluded until native Cut"
    );
    for y in 46..52 {
        for x in 14..18 {
            assert_eq!(terrain(&runtime, x, y), "TALL_GRASS");
        }
    }
    for (a, b) in [
        ((14, 47), (14, 48)),
        ((63, 24), (64, 24)),
        ((88, 47), (88, 48)),
    ] {
        for at in [a, b, a, b] {
            grid.walk(&mut controller, at);
        }
    }
    // Isolated ore crowns can contain FLOOR across a clipped-map seam. The
    // public component must still have no lower EW walking connection.
    for y in 48..72 {
        assert!(
            !(grid.floor.contains(&(63, y)) && grid.floor.contains(&(64, y))),
            "unexpected public lower EW walking opening"
        );
    }
    for at in [
        (23, 43),
        (24, 43),
        (24, 44),
        (29, 44),
        (29, 43),
        (30, 43),
        (29, 43),
        (29, 44),
        (24, 44),
        (24, 43),
        (23, 43),
    ] {
        grid.walk(&mut controller, at);
    }
    for x in [100, 101] {
        for y in [29, 30, 31, 32, 31, 30, 29] {
            grid.walk(&mut controller, (x, y));
        }
    }
    assert_eq!(terrain(&runtime, 99, 30), "WALL");
    assert_eq!(terrain(&runtime, 101, 30), "FLOOR");
    grid.walk(&mut controller, (99, 29));
    blocked(&mut controller, GameButton::Down);
    grid.walk(&mut controller, (99, 32));
    blocked(&mut controller, GameButton::Up);
    for at in [
        (49, 42),
        (49, 41),
        (49, 40),
        (49, 39),
        (49, 40),
        (49, 41),
        (49, 42),
    ] {
        grid.walk(&mut controller, at);
    }
    grid.walk(&mut controller, (81, 39));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native spoil descent");
        assert_ne!(
            position(&mut controller),
            (81, 41),
            "entered ledge reverse wall"
        );
        if position(&mut controller) == (81, 42) {
            break;
        }
    }
    assert_eq!(position(&mut controller), (81, 42));
    blocked(&mut controller, GameButton::Up);
    for at in [(49, 42), (49, 40), (49, 39), (81, 39)] {
        grid.walk(&mut controller, at);
    }
    for (at, text) in [
        ((4, 46), "IRON RANGE WEST"),
        ((20, 44), "TIMBER CREEK"),
        ((38, 42), "SURVEY MINE"),
        ((60, 44), "RECLAIMED QUARRY"),
        ((96, 40), "RAIL CROSSING"),
        ((120, 22), "GRAND RAPIDS"),
        ((96, 26), "MINE RAIL SIDING"),
        ((55, 19), "These bands"),
        ((95, 27), "Cross only"),
        ((91, 53), "I tend"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "missing interaction {at:?}"
        );
    }
    for at in [(39, 29), (119, 19), (7, 39)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    grid.walk(&mut controller, (40, 41));
    assert_eq!(
        controller
            .snapshot()
            .expect("mine entrance")
            .overworld
            .map_name,
        "MnR09Cave1F"
    );
    indoor_walk(&mut controller, &runtime, (19, 8));
    controller.press(GameButton::Up).expect("face Scout");
    controller.press(GameButton::A).expect("Scout conversation");
    let text = controller.snapshot().expect("Scout text");
    assert!(
        text.ui
            .text
            .as_ref()
            .and_then(|t| t.asm_text.as_ref())
            .is_some_and(|s| s.contains("foreman"))
    );
    let before = text.overworld.tile;
    controller
        .press(GameButton::Down)
        .expect("dialogue ownership");
    assert_eq!(
        controller
            .snapshot()
            .expect("owned movement")
            .overworld
            .tile,
        before
    );
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (13, 18));
    indoor_walk(&mut controller, &runtime, (13, 19));
    assert_eq!(
        controller
            .snapshot()
            .expect("native mine exit")
            .overworld
            .map_name,
        "MnR09C00R00"
    );
    assert_eq!(position(&mut controller), (40, 42));
    for at in [(41, 42), (40, 42)] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("passive return floor")
                .overworld
                .map_name,
            "MnR09C00R00"
        );
    }
    grid.walk(&mut controller, (40, 41));
    // Native warp arrival is on the room exit; step clear to release its
    // arrival suppression before deliberately re-entering the ladder.
    indoor_walk(&mut controller, &runtime, (13, 18));
    indoor_walk(&mut controller, &runtime, (13, 19));
    assert_eq!(
        controller
            .snapshot()
            .expect("native mine exit")
            .overworld
            .map_name,
        "MnR09C00R00"
    );
    assert_eq!(position(&mut controller), (40, 42));
    for n in 1..=3 {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_R09_ITEM_{n:02}"))
                .expect("open pickup flag")
        );
    }
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R09_ITEM_04")
            .expect("Cut reward remains sealed")
    );
    grid.walk(&mut controller, (124, 24));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(
        controller
            .snapshot()
            .expect("native menu")
            .ui
            .menu
            .is_some()
    );
    controller.press(GameButton::B).expect("close Start");
    if let Some(path) = std::env::var_os("CRYSTAL_IRON_RANGE_FIXTURE_SAVE") {
        controller
            .save(PathBuf::from(path))
            .expect("geometry fixture save");
    }
}

fn battle(controller: &mut VisibleShellController) -> BTreeSet<String> {
    let mut opponents = BTreeSet::new();
    for _ in 0..1024 {
        let snapshot = controller.snapshot().expect("native trainer activity");
        let Some(battle) = &snapshot.battle else {
            if snapshot.phase == RuntimeShellPhase::Overworld
                && !controller.has_pending_script_work()
            {
                return opponents;
            }
            controller
                .press(GameButton::A)
                .expect("native challenge/result text");
            continue;
        };
        assert!(
            !matches!(battle.kind, RuntimeBattleKind::Wild { .. }),
            "unexpected wild encounter on explicit trainer fixture"
        );
        opponents.insert(battle.enemy_pokemon.species.id.clone());
        let options = snapshot
            .ui
            .menu
            .as_ref()
            .map(|m| {
                m.layout
                    .vertical_menus
                    .iter()
                    .flat_map(|v| &v.options)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let button = if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            GameButton::A
        } else if options.iter().any(|s| s.contains("change <PKMN>")) {
            GameButton::B
        } else if options.iter().any(|s| s.contains("FIGHT")) {
            let cursor = controller
                .shell
                .battle_action_cursor
                .as_ref()
                .expect("action cursor")
                .option_index;
            if cursor / 2 > 0 {
                GameButton::Up
            } else if !cursor.is_multiple_of(2) {
                GameButton::Left
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_party_action_cursor {
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_switch_cursor {
            let slot = snapshot
                .party
                .slots
                .iter()
                .position(|p| p.pokemon.hp > 0 && Some(p.index) != battle.active_player_party_index)
                .expect("living replacement");
            if cursor.option_index < slot {
                GameButton::Down
            } else if cursor.option_index > slot {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &controller.shell.battle_move_cursor {
            let target = battle
                .player_moves
                .iter()
                .position(|m| {
                    m.current_pp > 0 && matches!(m.name.as_str(), "SURF" | "THUNDERPUNCH")
                })
                .expect("legal damaging move");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        if snapshot.ui.menu.is_none() && snapshot.ui.text.is_some() {
            controller
                .wait_frames(30)
                .expect("native battle presentation");
        }
        controller.press(button).expect("production battle input");
    }
    panic!("native trainer battle failed to finish");
}

#[test]
#[ignore = "requires the external Iron Range pack"]
fn iron_range_populated_fixture_spends_surf_pp_wins_rests_and_continues() {
    let (root, runtime, mut controller) = fixture((98, 38));
    // Initial setup only. Surf is source-compatible; Thunderpunch is source-
    // compatible. The injury guarantees a real healing effect to observe.
    for (species, level, mov, injury) in [
        ("FERALIGATR", 48, "SURF", 20),
        ("AMPHAROS", 46, "THUNDERPUNCH", 0),
    ] {
        assert!(
            runtime.data().pokemon[species]
                .tmhm_learnset
                .iter()
                .any(|m| m == mov)
        );
        let mut pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("legal fixture partner");
        pokemon.moves = vec![LearnedMove {
            name: mov.into(),
            current_pp: runtime.data().moves[mov].pp,
            pp_ups: 0,
        }];
        pokemon.hp = pokemon.max_hp.saturating_sub(injury);
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(pokemon)
        );
    }
    let state = controller.shell.shell.session_mut().state_mut();
    state.badges.johto = [true, true, true, true, true, true, true, false];
    state.sync_party_from_storage();
    state.random_state = Default::default();
    *controller.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..131072)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<u8>>(),
        );
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // No party, map, flags, divider or clock rewrites occur after this setup.
    let before = controller.snapshot().expect("initial legal party");
    let pp = before.party.slots[0].pokemon.moves[0].current_pp;
    controller.press(GameButton::Up).expect("face Rafe");
    controller
        .press(GameButton::A)
        .expect("native trainer interaction");
    let opponents = battle(&mut controller);
    finish_text(&mut controller);
    assert_eq!(
        opponents,
        BTreeSet::from(["GEODUDE".into(), "SKARMORY".into(), "MACHOP".into()])
    );
    let won = controller.snapshot().expect("trainer victory");
    assert!(won.battle.is_none());
    assert!(won.party.slots.iter().any(|p| p.pokemon.hp > 0));
    assert!(
        won.party.slots[0].pokemon.moves[0].current_pp < pp,
        "selected Surf PP did not decrease"
    );
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R09_TRAINER_06")
            .expect("persistent trainer flag")
    );
    let save = std::env::temp_dir().join(format!(
        "geothite-iron-range-journey-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save real battle result");
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), save.clone(), None)
            .expect("Continue spent PP and victory");
    let continued = controller.snapshot().expect("continued party");
    assert_eq!(won.party, continued.party);
    assert_eq!(won.bag, continued.bag);
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R09_TRAINER_06")
            .expect("continued victory")
    );
    let grid = grid(&runtime, position(&mut controller), false);
    assert!(
        controller
            .snapshot()
            .expect("injured team before rest")
            .party
            .slots
            .iter()
            .any(|p| p.pokemon.hp < p.pokemon.max_hp)
    );
    assert!(grid.talk(&mut controller, (91, 53)).contains("I tend"));
    let rested = controller.snapshot().expect("native field medic result");
    for slot in &rested.party.slots {
        assert_eq!(slot.pokemon.hp, slot.pokemon.max_hp);
        for mov in &slot.pokemon.moves {
            assert_eq!(
                mov.current_pp,
                runtime.data().moves[&mov.name].pp,
                "native rest must restore PP"
            );
        }
    }
    controller.save(&save).expect("save healed party");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue native rest");
    let after = loaded.snapshot().expect("continued healed team");
    assert_eq!(rested.party, after.party);
    assert_eq!(rested.bag, after.bag);
    assert!(
        loaded
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R09_TRAINER_06")
            .expect("victory after rest")
    );
    if let Some(path) = std::env::var_os("CRYSTAL_IRON_RANGE_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export explicit journey fixture");
    }
    std::fs::remove_file(save).expect("remove fixture scratch save");
}
