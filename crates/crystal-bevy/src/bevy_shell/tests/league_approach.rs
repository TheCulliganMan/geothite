//! Explicit geometry, field-service and late-battle fixtures for the revised route.
//! These do not establish earned badges or the connected Q08 journey.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{RouteGrid, RouteSpec, finish_text, indoor_walk, position};

type Point = (i16, i16);

fn fixture(map: &str, at: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_LEAGUE_APPROACH_TEST_PACK").expect("external revised pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified native pack"),
    )
    .expect("runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: map.into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit fixture placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

fn grid(runtime: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR13",
            columns: 1,
            rows: 3,
            tileset: "johto",
            start: (30, 2),
            doors: BTreeMap::from([((14, 53), "MnR13Cave1F")]),
        },
    )
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    for _ in 0..8 {
        if position(controller) == target {
            return;
        }
        controller.press(button).expect("native geometry input");
        assert_ne!(
            position(controller),
            (39, 25),
            "walked into the ledge reverse wall"
        );
    }
    assert_eq!(position(controller), target);
}

fn save_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("geothite-league-{label}-{stamp}.crystalsave"))
}

#[test]
#[ignore = "requires the revised external League Approach pack"]
fn league_approach_controller_walks_clipped_maps_hop_ramp_shelter_and_resources() {
    let (root, runtime, mut controller) = fixture("MnR13C00R00", (30, 2));
    // Empty-party geometry: encounters and battle balance are separate fixtures.
    let grid = grid(&runtime);
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
    for at in [
        (30, 2),
        (34, 124),
        (18, 25),
        (47, 15),
        (47, 26),
        (20, 41),
        (15, 55),
        (40, 60),
        (47, 83),
        (18, 102),
        (35, 119),
    ] {
        grid.walk(&mut controller, at);
    }
    for at in [(20, 47), (20, 48), (20, 47), (34, 95), (34, 96), (34, 95)] {
        grid.walk(&mut controller, at);
    }
    grid.walk(&mut controller, (39, 23));
    step(&mut controller, GameButton::Down, (39, 26));
    for _ in 0..4 {
        controller
            .press(GameButton::Up)
            .expect("reject reverse ledge");
    }
    assert_eq!(position(&mut controller), (39, 26));
    for at in [(47, 26), (47, 25), (47, 24), (47, 23), (39, 23)] {
        grid.walk(&mut controller, at);
    }
    // The viewing rail is genuinely blocking, next to actual lake water.
    grid.walk(&mut controller, (40, 60));
    for _ in 0..4 {
        controller
            .press(GameButton::Right)
            .expect("lake viewing rail");
    }
    assert_eq!(position(&mut controller), (40, 60));
    for (tile, text) in [
        ((31, 9), "Eight badges"),
        ((19, 27), "sheltered bay"),
        ((17, 57), "Ice shaped"),
        ((27, 71), "cave keeper"),
        ((39, 65), "Noctowl"),
        ((33, 121), "Ely is just south"),
        ((28, 12), "LEAGUE APPROACH"),
        ((32, 24), "BORDER LAKE"),
        ((18, 40), "GRASS SADDLE"),
        ((16, 56), "CEDAR SHELTER"),
        ((38, 62), "LONG LAKE"),
        ((36, 120), "ELY"),
    ] {
        assert!(
            grid.talk(&mut controller, tile).contains(text),
            "interaction {tile:?}"
        );
    }
    for (tile, item) in [
        ((13, 27), "HYPER_POTION"),
        ((53, 19), "ULTRA_BALL"),
        ((37, 61), "MAX_REPEL"),
    ] {
        assert!(grid.talk(&mut controller, tile).contains("found"));
        let bag = controller.snapshot().expect("actual pickup").bag;
        assert!(
            bag.items
                .iter()
                .chain(&bag.balls)
                .any(|slot| slot.item_id == item && slot.quantity == 1)
        );
    }
    grid.walk(&mut controller, (14, 53));
    assert_eq!(
        controller
            .snapshot()
            .expect("shelter arrival")
            .overworld
            .map_name,
        "MnR13Cave1F"
    );
    indoor_walk(&mut controller, &runtime, (13, 6));
    controller.press(GameButton::Up).expect("face keeper");
    controller.press(GameButton::A).expect("shelter keeper");
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (15, 10));
    controller.press(GameButton::Down).expect("face cache");
    controller.press(GameButton::A).expect("native cave pickup");
    finish_text(&mut controller);
    assert!(
        controller
            .snapshot()
            .expect("cave cache")
            .bag
            .items
            .iter()
            .any(|slot| slot.item_id == "MAX_ETHER" && slot.quantity == 1)
    );
    indoor_walk(&mut controller, &runtime, (7, 14));
    indoor_walk(&mut controller, &runtime, (7, 15));
    assert_eq!(
        controller
            .snapshot()
            .expect("safe cave return")
            .overworld
            .map_name,
        "MnR13C00R01"
    );
    assert_eq!(position(&mut controller), (14, 54));
    grid.walk(&mut controller, (30, 8));
    let save = save_path("geometry");
    controller
        .save(&save)
        .expect("save collected native resources");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue geometry");
    assert_eq!(position(&mut loaded), (30, 8));
    for flag in [
        "EVENT_MN_R13_ITEM_01",
        "EVENT_MN_R13_ITEM_02",
        "EVENT_MN_R13_ITEM_03",
        "EVENT_MN_R13_SHELTER_ETHER",
    ] {
        assert!(
            loaded
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("saved resource flag")
        );
    }
    if let Some(path) = std::env::var_os("CRYSTAL_LEAGUE_APPROACH_FIXTURE_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export explicit geometry review save");
    }
    std::fs::remove_file(save).expect("remove disposable save");
}

#[test]
#[ignore = "requires the revised external League Approach pack"]
fn league_approach_controller_native_keeper_restores_fainted_hp_status_and_pp() {
    let (_, runtime, mut controller) = fixture("MnR13Cave1F", (7, 14));
    for (species, hp) in [("MEW", 1), ("LAPRAS", 0)] {
        let mut partner = runtime
            .data()
            .create_pokemon(species, 60, Dv::default())
            .expect("legal explicit injured fixture");
        partner.hp = hp;
        partner.status = Some("POISON".into());
        for learned in &mut partner.moves {
            learned.current_pp = 0;
        }
        assert!(
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(partner)
        );
    }
    controller
        .shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut controller.shell);
    indoor_walk(&mut controller, &runtime, (13, 6));
    controller.press(GameButton::Up).expect("face keeper");
    controller.press(GameButton::A).expect("ask native keeper");
    finish_text(&mut controller);
    for slot in controller
        .snapshot()
        .expect("actual restored party")
        .party
        .slots
    {
        assert_eq!(slot.pokemon.hp, slot.pokemon.max_hp);
        assert!(slot.pokemon.status.is_none());
        for learned in slot.pokemon.moves {
            assert_eq!(learned.current_pp, runtime.data().moves[&learned.name].pp);
        }
    }
}

#[test]
#[ignore = "requires the revised external League Approach pack"]
fn league_approach_controller_native_ada_battle_spends_pp_wins_and_persists() {
    let (root, runtime, mut controller) = fixture("MnR13C00R00", (23, 24));
    // Explicit level60 partner/badges; this proves battle execution, not balance.
    let mut mew = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("legal late fixture partner");
    mew.moves = ["PSYCHIC_M", "CUT"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: runtime.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(mew));
    state.badges.johto = [true; 8];
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.press(GameButton::Down).expect("face Ada");
    controller.press(GameButton::A).expect("challenge Ada");
    for _ in 0..64 {
        let s = controller.snapshot().expect("trainer intro");
        if s.ui.menu.as_ref().is_some_and(|menu| {
            menu.layout
                .vertical_menus
                .iter()
                .flat_map(|m| &m.options)
                .any(|option| option.contains("FIGHT"))
        }) {
            break;
        }
        controller.press(GameButton::A).expect("intro page");
    }
    assert!(
        controller
            .snapshot()
            .expect("native trainer battle")
            .battle
            .is_some()
    );
    controller.press(GameButton::A).expect("FIGHT");
    let battle = controller
        .snapshot()
        .expect("move menu")
        .battle
        .expect("battle");
    let active = battle.active_player_party_index.expect("active");
    let selected = battle.player_moves[0].clone();
    controller
        .press(GameButton::A)
        .expect("actual selected move");
    assert!(
        controller.snapshot().expect("PP mutation").party.slots[active]
            .pokemon
            .moves
            .iter()
            .find(|m| m.name == selected.name)
            .expect("selected move")
            .current_pp
            < selected.current_pp
    );
    for _ in 0..512 {
        let snapshot = controller.snapshot().expect("battle state");
        let Some(battle) = &snapshot.battle else {
            break;
        };
        let button = if let Some(menu) = &snapshot.ui.menu {
            let options = menu
                .layout
                .vertical_menus
                .iter()
                .flat_map(|m| &m.options)
                .collect::<Vec<_>>();
            if options.iter().any(|entry| entry.contains("change <PKMN>")) {
                GameButton::B
            } else if options.iter().any(|entry| entry.contains("FIGHT")) {
                let cursor = controller
                    .shell
                    .battle_action_cursor
                    .as_ref()
                    .expect("production action cursor")
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
                let active = battle.active_player_party_index;
                let target = snapshot
                    .party
                    .slots
                    .iter()
                    .position(|slot| slot.pokemon.hp > 0 && Some(slot.index) != active)
                    .expect("living replacement");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else if let Some(cursor) = &controller.shell.battle_move_cursor {
                let active = battle.active_player_party_index.expect("active partner");
                let name = if snapshot.party.slots[active].pokemon.species.id == "MEW" {
                    "PSYCHIC_M"
                } else {
                    "ICE_BEAM"
                };
                let target = battle
                    .player_moves
                    .iter()
                    .position(|m| m.name == name && m.current_pp > 0)
                    .expect("chosen legal attack");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        controller
            .press(button)
            .expect("native battle choice or text page");
    }
    let ended = controller.snapshot().expect("battle complete");
    assert!(ended.battle.is_none(), "Ada battle failed to end");
    assert!(ended.party.slots.iter().any(|slot| slot.pokemon.hp > 0));
    finish_text(&mut controller);
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R13_TRAINER_01")
            .expect("native defeat flag")
    );
    let save = save_path("battle");
    controller.save(&save).expect("save native battle outcome");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue battle");
    assert!(
        loaded
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R13_TRAINER_01")
            .expect("saved defeat flag")
    );
    assert_eq!(
        loaded.snapshot().expect("saved party").party,
        controller.snapshot().expect("party").party
    );
    std::fs::remove_file(save).expect("remove disposable battle save");
}
