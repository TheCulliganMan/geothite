//! Explicit local setup followed only by production controller inputs.
//!
//! These fixtures do not establish earned badges or the regional SQ03 journey.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
    Point, RouteGrid, RouteSpec, escape_wild, finish_text, indoor_walk, position,
};

fn runtime() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_BORDER_PINES_TEST_PACK")
            .expect("published external Border Pines pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        crystal_assets::read_loaded_verified_compiled_game_pack(path)
            .expect("verified external pack"),
    )
    .expect("native route runtime");
    (root, runtime)
}

fn fixture(root: &AssetRoot, runtime: &CrystalRuntime, at: Point) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("MnR17C00R{:02}", at.1 / 48),
            tile_x: at.0,
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
    controller
}

fn route_tileset(runtime: &CrystalRuntime) -> &'static str {
    match runtime.data().maps["MnR17C00R00"]
        .attributes
        .tileset_name
        .as_str()
    {
        "mn_border_pines" => "mn_border_pines",
        "mn_winter_outdoor" => "mn_winter_outdoor",
        other => panic!("unexpected Border Pines tileset {other}"),
    }
}

fn grid(runtime: &CrystalRuntime, at: Point, grass: bool) -> RouteGrid {
    let spec = RouteSpec {
        prefix: "MnR17",
        columns: 1,
        rows: 3,
        tileset: route_tileset(runtime),
        start: at,
        doors: BTreeMap::from([((10, 43), "MnR17Cave1F"), ((45, 49), "MnR17RangerHut")]),
    };
    if grass {
        RouteGrid::new_with_grass(runtime, spec).with_battle_handler(escape_wild)
    } else {
        RouteGrid::new(runtime, spec)
    }
}

fn blocked(controller: &mut VisibleShellController, direction: GameButton) {
    let before = position(controller);
    for _ in 0..3 {
        controller.press(direction).expect("native barrier input");
    }
    assert_eq!(position(controller), before);
}

fn exit_hut(controller: &mut VisibleShellController, runtime: &CrystalRuntime) {
    assert_eq!(
        controller
            .snapshot()
            .expect("hut exit start")
            .overworld
            .map_name,
        "MnR17RangerHut"
    );
    // Arrival is on the bottom carpet. Step clear to release arrival
    // suppression, then use native Down inputs through and beyond the carpet.
    indoor_walk(controller, runtime, (5, 6));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("native hut carpet departure");
        controller
            .wait_frames(16)
            .expect("native warp presentation");
        let snapshot = controller.snapshot().expect("hut departure state");
        assert!(snapshot.battle.is_none());
        if snapshot.overworld.map_name == "MnR17C00R01" {
            assert_eq!(position(controller), (45, 50));
            return;
        }
        assert_eq!(snapshot.overworld.map_name, "MnR17RangerHut");
    }
    panic!("native hut carpet did not return to its paired exterior");
}

fn tree(controller: &VisibleShellController) -> u16 {
    controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(18, 10)
        .expect("native tree in middle chunk")
}

fn cut(controller: &mut VisibleShellController, direction: GameButton) {
    blocked(controller, direction);
    for _ in 0..64 {
        if tree(controller) == 0x17 {
            finish_text(controller);
            controller.wait_frames(60).expect("native Cut presentation");
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native Cut dialogue");
    }
    panic!("native forest Cut did not replace 0f with 17");
}

fn opened_grid(runtime: &CrystalRuntime, controller: &mut VisibleShellController) -> RouteGrid {
    let mut grid = grid(runtime, position(controller), true);
    // Read the authoritative map replacement; this only plans native inputs.
    // The other three quadrants must retain the source replacement collision.
    let block = tree(controller);
    let collision =
        &runtime.data().tilesets[route_tileset(runtime)].collision[&format!("{block:02x}")];
    for dy in 0..2 {
        for dx in 0..2 {
            if collision[dy * 2 + dx] == "FLOOR" {
                grid.floor.insert((36 + dx as i16, 68 + dy as i16));
            }
        }
    }
    grid
}

fn initial_partner(
    runtime: &CrystalRuntime,
    controller: &mut VisibleShellController,
    move_name: &str,
    hive_badge: bool,
) {
    let mut pokemon = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("source-compatible local field partner");
    assert!(
        runtime.data().pokemon["MEW"]
            .tmhm_learnset
            .iter()
            .any(|m| m == move_name)
    );
    pokemon.moves = vec![LearnedMove {
        name: move_name.into(),
        current_pp: runtime.data().moves[move_name].pp.saturating_sub(1),
        pp_ups: 0,
    }];
    pokemon.hp = pokemon.max_hp.saturating_sub(20);
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.badges.johto = [true; 8];
    state.badges.johto[1] = hive_badge;
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut controller.shell);
}

#[test]
#[ignore = "requires the published external Border Pines pack"]
fn border_pines_controller_tours_three_chunks_rails_rooms_and_open_resources() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, (32, 108));
    // Empty party is deliberate geometry setup, not a battle/encounter claim.
    let grid = grid(&runtime, (32, 108), true);
    for (a, b) in [
        ((24, 95), (24, 96)),
        ((10, 47), (10, 48)),
        ((40, 47), (40, 48)),
    ] {
        for at in [a, b, a, b] {
            grid.walk(&mut controller, at);
        }
    }
    for y in 88..92 {
        for x in 22..28 {
            assert!(grid.floor.contains(&(x, y)), "required sedge habitat");
        }
    }
    for x in 22..30 {
        for y in [65, 66, 65] {
            grid.walk(&mut controller, (x, y));
        }
    }
    grid.walk(&mut controller, (26, 65));
    blocked(&mut controller, GameButton::Up);
    grid.walk(&mut controller, (26, 66));
    blocked(&mut controller, GameButton::Down);
    grid.walk(&mut controller, (28, 59));
    blocked(&mut controller, GameButton::Left);
    for (at, text) in [
        ((35, 107), "BORDER PINES"),
        ((27, 85), "SEDGE TRAIL"),
        ((19, 65), "MARSH CROSSING"),
        ((47, 51), "RANGER REST HUT"),
        ((13, 43), "GRANITE DEN"),
        ((25, 25), "OLD PINES ROOST"),
        ((37, 7), "INTERNATIONAL"),
        ((53, 29), "FERN ALCOVE"),
        ((27, 45), "GRANITE DEN"),
        ((47, 53), "The ranger desk"),
        ((37, 77), "That cache"),
        ((15, 27), "These old crowns"),
    ] {
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "interaction {at:?}"
        );
    }
    for at in [(55, 27), (11, 59), (37, 73)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    grid.walk(&mut controller, (10, 43));
    assert_eq!(
        controller.snapshot().expect("den entry").overworld.map_name,
        "MnR17Cave1F"
    );
    indoor_walk(&mut controller, &runtime, (5, 6));
    controller.press(GameButton::Up).expect("face den reward");
    controller.press(GameButton::A).expect("collect Revive");
    assert_eq!(
        controller.snapshot().expect("room reward text").phase,
        RuntimeShellPhase::Text
    );
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (9, 13));
    assert_eq!(position(&mut controller), (10, 44));
    for at in [(10, 45), (10, 44)] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("passive den return")
                .overworld
                .map_name,
            "MnR17C00R00"
        );
    }
    grid.walk(&mut controller, (10, 43));
    indoor_walk(&mut controller, &runtime, (9, 12));
    indoor_walk(&mut controller, &runtime, (9, 13));
    assert_eq!(position(&mut controller), (10, 44));
    grid.walk(&mut controller, (45, 49));
    assert_eq!(
        controller.snapshot().expect("hut entry").overworld.map_name,
        "MnR17RangerHut"
    );
    indoor_walk(&mut controller, &runtime, (5, 6));
    controller.press(GameButton::Up).expect("face keeper");
    controller.press(GameButton::A).expect("rest conversation");
    assert_eq!(
        controller.snapshot().expect("keeper text").phase,
        RuntimeShellPhase::Text
    );
    finish_text(&mut controller);
    exit_hut(&mut controller, &runtime);
    assert_eq!(position(&mut controller), (45, 50));
    for at in [(46, 50), (45, 50)] {
        grid.walk(&mut controller, at);
        assert_eq!(
            controller
                .snapshot()
                .expect("passive hut return")
                .overworld
                .map_name,
            "MnR17C00R01"
        );
    }
    grid.walk(&mut controller, (45, 49));
    indoor_walk(&mut controller, &runtime, (5, 6));
    exit_hut(&mut controller, &runtime);
    assert_eq!(position(&mut controller), (45, 50));
    for n in 1..=4 {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(&format!("EVENT_MN_R17_ITEM_{n:02}"))
                .expect("native resource flag")
        );
    }
    grid.walk(&mut controller, (36, 2));
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
    if let Some(path) = std::env::var_os("CRYSTAL_BORDER_PINES_FIXTURE_SAVE") {
        controller
            .save(PathBuf::from(path))
            .expect("save geometry fixture");
    }
}

#[test]
#[ignore = "requires the published external Border Pines pack"]
fn border_pines_cut_shortcut_denies_opens_walks_rests_and_resets_on_continue() {
    let (root, runtime) = runtime();
    for (move_name, hive_badge) in [("SWIFT", true), ("CUT", false)] {
        let mut denied = fixture(&root, &runtime, (36, 68));
        initial_partner(&runtime, &mut denied, move_name, hive_badge);
        assert_eq!(tree(&denied), 0x0f);
        blocked(&mut denied, GameButton::Down);
        denied.press(GameButton::A).expect("native Cut denial");
        assert!(
            denied
                .snapshot()
                .expect("denial text")
                .ui
                .text
                .as_ref()
                .and_then(|t| t.asm_text.as_ref())
                .is_some_and(|t| t.contains("CUT"))
        );
        finish_text(&mut denied);
        assert_eq!(tree(&denied), 0x0f);
        assert_eq!(position(&mut denied), (36, 68));
        assert_eq!(
            grid(&runtime, (36, 67), true)
                .path((36, 67), (36, 73))
                .len(),
            66
        );
    }
    let mut controller = fixture(&root, &runtime, (36, 67));
    initial_partner(&runtime, &mut controller, "CUT", true);
    let ordinary = grid(&runtime, (36, 67), true);
    assert_eq!(ordinary.path((36, 67), (36, 73)).len(), 66);
    // Both arms already accessible. Walk the actual longer dry route first.
    ordinary.walk(&mut controller, (36, 73));
    ordinary.walk(&mut controller, (36, 67));
    ordinary.walk(&mut controller, (36, 68));
    cut(&mut controller, GameButton::Down);
    assert_eq!(tree(&controller), 0x17);
    let opened = opened_grid(&runtime, &mut controller);
    assert_eq!(opened.path((36, 67), (36, 73)).len(), 6);
    opened.walk(&mut controller, (36, 67));
    opened.walk(&mut controller, (36, 73));
    assert!(opened.talk(&mut controller, (37, 73)).contains("found"));
    assert!(
        opened
            .talk(&mut controller, (37, 77))
            .contains("That cache")
    );
    opened.walk(&mut controller, (36, 73));
    let collected = controller.snapshot().expect("actual cache collection");
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R17_ITEM_03")
            .expect("cache flag")
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture save clock")
        .as_nanos();
    let save = std::env::temp_dir().join(format!("geothite-border-pines-cut-{stamp}.crystalsave"));
    controller.save(&save).expect("save actual Cut and cache");
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), save.clone(), None)
            .expect("Continue local shortcut journey");
    let continued = controller.snapshot().expect("continued cache and partner");
    assert_eq!(continued.party, collected.party);
    assert_eq!(continued.bag, collected.bag);
    assert_eq!(position(&mut controller), (36, 73));
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R17_ITEM_03")
            .expect("persisted cache flag")
    );
    // Native TILES reload restores vegetation independently of pickup flags.
    assert_eq!(tree(&controller), 0x0f);
    let ordinary = grid(&runtime, (36, 73), true);
    ordinary.walk(&mut controller, (36, 70));
    cut(&mut controller, GameButton::Up);
    opened_grid(&runtime, &mut controller).walk(&mut controller, (36, 67));
    let grid = grid(&runtime, (36, 67), true);
    grid.walk(&mut controller, (45, 49));
    indoor_walk(&mut controller, &runtime, (5, 6));
    let before = controller
        .snapshot()
        .expect("injured partner before actual rest");
    assert!(before.party.slots[0].pokemon.hp < before.party.slots[0].pokemon.max_hp);
    assert!(before.party.slots[0].pokemon.moves[0].current_pp < runtime.data().moves["CUT"].pp);
    controller
        .press(GameButton::Up)
        .expect("face native keeper");
    controller
        .press(GameButton::A)
        .expect("invoke authored HealParty");
    finish_text(&mut controller);
    let rested = controller.snapshot().expect("native rest result");
    assert_eq!(
        rested.party.slots[0].pokemon.hp,
        rested.party.slots[0].pokemon.max_hp
    );
    assert_eq!(
        rested.party.slots[0].pokemon.moves[0].current_pp,
        runtime.data().moves["CUT"].pp
    );
    exit_hut(&mut controller, &runtime);
    assert_eq!(position(&mut controller), (45, 50));
    let before_continue = controller
        .snapshot()
        .expect("rested team after actual hut return");
    controller.save(&save).expect("save actual rest result");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue healed partner");
    let loaded_snapshot = loaded.snapshot().expect("persisted rest");
    assert_eq!(loaded_snapshot.party, before_continue.party);
    assert_eq!(loaded_snapshot.bag, before_continue.bag);
    assert_eq!(tree(&loaded), 0x0f);
    std::fs::remove_file(save).expect("remove disposable fixture save");
}

fn complete_battle(controller: &mut VisibleShellController) -> BTreeSet<String> {
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
#[ignore = "requires the published external Border Pines pack"]
fn border_pines_populated_fixture_wins_spends_selected_pp_and_continues() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, (11, 46));
    // Legal local party, badges and deterministic divider are initial setup.
    // The test starts beside the actual T04; it does not claim earned travel.
    for (species, level, move_name) in
        [("FERALIGATR", 48, "SURF"), ("AMPHAROS", 46, "THUNDERPUNCH")]
    {
        assert!(
            runtime.data().pokemon[species]
                .tmhm_learnset
                .iter()
                .any(|m| m == move_name)
        );
        let mut pokemon = runtime
            .data()
            .create_pokemon(species, level, Dv::default())
            .expect("legal local partner");
        pokemon.moves = vec![LearnedMove {
            name: move_name.into(),
            current_pp: runtime.data().moves[move_name].pp,
            pp_ups: 0,
        }];
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
    // No state, party, map, position, flags or divider rewrites after setup.
    let pp = controller.snapshot().expect("initial PP").party.slots[0]
        .pokemon
        .moves[0]
        .current_pp;
    controller
        .press(GameButton::Up)
        .expect("face actual trainer");
    controller
        .press(GameButton::A)
        .expect("native trainer challenge");
    assert_eq!(
        complete_battle(&mut controller),
        runtime.data().trainers.trainers["MN_R17_T04"]
            .party
            .iter()
            .map(|pokemon| pokemon.species.clone())
            .collect::<BTreeSet<_>>()
    );
    finish_text(&mut controller);
    let won = controller.snapshot().expect("native trainer result");
    assert!(won.battle.is_none());
    assert!(won.party.slots.iter().any(|p| p.pokemon.hp > 0));
    assert!(
        won.party.slots[0].pokemon.moves[0].current_pp < pp,
        "selected Surf PP must decrease"
    );
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R17_TRAINER_04")
            .expect("native victory flag")
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture save clock")
        .as_nanos();
    let save =
        std::env::temp_dir().join(format!("geothite-border-pines-battle-{stamp}.crystalsave"));
    controller.save(&save).expect("save real trainer victory");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue battle result");
    let continued = loaded.snapshot().expect("continued party/PP");
    assert_eq!(continued.party, won.party);
    assert_eq!(continued.bag, won.bag);
    assert!(
        loaded
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R17_TRAINER_04")
            .expect("continued trainer victory")
    );
    if let Some(path) = std::env::var_os("CRYSTAL_BORDER_PINES_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("optional controller fixture save");
    }
    std::fs::remove_file(save).expect("remove disposable battle save");
}

fn native_route_kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let m = &runtime.data().maps[&format!("MnR17C00R{:02}", at.1 / 48)];
    let x = usize::try_from(at.0).expect("nonnegative route x");
    let y = usize::try_from(at.1 % 48).expect("nonnegative local y");
    let block = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    &runtime.data().tilesets[&m.attributes.tileset_name].collision[&format!("{block:02x}")]
        [y % 2 * 2 + x % 2]
}

fn continue_grass_state(
    root: AssetRoot,
    runtime: CrystalRuntime,
    controller: &mut VisibleShellController,
    label: &str,
) {
    let at = position(controller);
    let before = controller.snapshot().expect("actual reached grass state");
    let save = super::authored_route_fixture::reached_save_path(&format!(
        "geothite-r17-{label}-{}.crystalsave",
        std::process::id(),
    ));
    controller.save(&save).expect("save reached grass state");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue actual grass state");
    let after = loaded.snapshot().expect("continued grass state");
    assert_eq!(position(&mut loaded), at);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    if let Some(path) = std::env::var_os("CRYSTAL_BORDER_PINES_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached grass state");
    }
    super::authored_route_fixture::cleanup_reached_save(&save);
}

#[test]
#[ignore = "requires the current selected-parent Border Pines grass candidate"]
fn border_pines_grass_controller_northern_arrival_crossing_and_continue() {
    let (root, runtime) = runtime();
    let mut c = fixture(&root, &runtime, (36, 6));
    initial_partner(&runtime, &mut c, "SWIFT", true);
    let route = grid(&runtime, (36, 6), true);
    let mut grass = 0;
    route.walk_with_actions(
        &mut c,
        (36, 12),
        |c| {
            grass += usize::from(native_route_kind(&runtime, position(c)) == "TALL_GRASS");
        },
        escape_wild,
    );
    assert!(grass >= 2, "northern arrival must cross its new pine grass");
    continue_grass_state(root, runtime, &mut c, "north-arrival");
}

#[test]
#[ignore = "requires the current selected-parent Border Pines grass candidate"]
fn border_pines_grass_controller_marsh_new_habitat_wild_run_and_continue() {
    let (root, runtime) = runtime();
    let mut c = fixture(&root, &runtime, (19, 78));
    initial_partner(&runtime, &mut c, "SWIFT", true);
    assert_eq!(native_route_kind(&runtime, (19, 78)), "TALL_GRASS");
    assert_eq!(native_route_kind(&runtime, (19, 79)), "TALL_GRASS");
    let route = grid(&runtime, (19, 78), true);
    let mut encounters = 0;
    for index in 0..256 {
        let target = if index % 2 == 0 { (19, 79) } else { (19, 78) };
        route.walk_with_actions(
            &mut c,
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
        "new middle-chunk habitat never started a native encounter"
    );
    route.walk_with_actions(&mut c, (19, 82), |_| {}, escape_wild);
    continue_grass_state(root, runtime, &mut c, "marsh-wild");
}
