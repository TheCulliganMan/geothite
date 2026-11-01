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
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, position,
};

fn runtime() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_GUNFLINT_TEST_PACK").expect("published external Gunflint pack"),
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
            map_name: format!("MnR12C{:02}R{:02}", at.0 / 64, at.1 / 48),
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
    controller
}

fn grid(runtime: &CrystalRuntime, at: Point, grass: bool) -> RouteGrid {
    let spec = RouteSpec {
        prefix: "MnR12",
        columns: 2,
        rows: 3,
        tileset: if runtime.data().maps["MnR12C00R00"].attributes.tileset_name
            == "mn_winter_gunflint"
        {
            "mn_winter_gunflint"
        } else {
            "mn_gunflint"
        },
        start: at,
        doors: BTreeMap::from([((66, 23), "MnR12Cave1F"), ((85, 83), "MnR12RangerHut")]),
    };
    if grass {
        RouteGrid::new_with_grass(runtime, spec)
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

fn tree(controller: &VisibleShellController) -> u16 {
    controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(27, 11)
        .expect("native herb tree in northwest chunk")
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
    let mut grid = grid(runtime, position(controller), false);
    // The static grid excludes the sealed clearing. Read every FLOOR from the
    // authoritative current chunk only after native Cut opened its entrance.
    // This extends the input planner, never the runtime terrain/player state.
    let snapshot = controller.snapshot().expect("current native chunk");
    assert_eq!(snapshot.overworld.map_name, "MnR12C00R00");
    let map = &controller.shell.shell.session().overworld().map;
    let module = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&snapshot.overworld.map_name)
        .expect("current chunk");
    let collision = &runtime.data().tilesets[&module.attributes.tileset_name].collision;
    for y in 0..i16::try_from(module.attributes.height)
        .expect("native chunk height fits core coordinates")
        * 2
    {
        for x in 0..i16::try_from(module.attributes.width)
            .expect("native chunk width fits core coordinates")
            * 2
        {
            let block = map
                .metatile_at(x / 2, y / 2)
                .expect("authoritative native block");
            if collision[&format!("{block:02x}")][(y % 2 * 2 + x % 2) as usize] == "FLOOR" {
                grid.floor.insert((x, y));
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
#[ignore = "requires the revised external Gunflint pack"]
fn gunflint_controller_walks_six_clipped_maps_portages_and_original_rooms() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, (124, 56));
    // Explicit empty-party geometry. Encounters/combat are separate fixtures.
    let grid = grid(&runtime, (124, 56), true);
    assert!(!grid.floor.contains(&(54, 19)), "herb cache is Cut gated");
    for (a, b) in [
        ((10, 47), (10, 48)),
        ((32, 95), (32, 96)),
        ((63, 58), (64, 58)),
        ((82, 47), (82, 48)),
        ((82, 95), (82, 96)),
    ] {
        for at in [a, b, a, b] {
            grid.walk(&mut controller, at);
        }
    }
    // The cave roof contains source FLOOR quadrants across the upper seam,
    // but the entrance is a terminal warp, not a walking passage through the
    // mountain. Tour the actual exterior component rather than that roof.
    assert!(!grid.floor.contains(&(63, 17)));
    // The lower east/west chunk adjacency is registered but closed by forest.
    for y in 96..112 {
        assert!(!(grid.floor.contains(&(63, y)) && grid.floor.contains(&(64, y))));
    }
    for x in 82..=120 {
        grid.walk(&mut controller, (x, 36));
    }
    for y in 78..=90 {
        grid.walk(&mut controller, (32, y));
    }
    for (at, text) in [
        ((117, 57), "June keeps"),
        ((101, 13), "Loons"),
        ((53, 39), "Windthrow"),
        ((43, 97), "A dry hammock"),
        ((109, 77), "Big tracks"),
        ((11, 39), "The long west bank"),
        ((119, 53), "GUNFLINT"),
        ((81, 25), "LOONS"),
        ((69, 25), "RAVEN"),
        ((81, 87), "RANGER"),
        ((35, 75), "STONE BOG"),
        ((7, 55), "CANOE"),
    ] {
        if at == (35, 75)
            && runtime.data().maps["MnR12C00R01"].attributes.tileset_name == "mn_winter_gunflint"
        {
            // The frozen rime lane makes this optional bay a Surf discovery.
            // Its actual sign and trainer approach are tested from the lake
            // below, rather than claiming a dry mid-slide turning point.
            continue;
        }
        assert!(
            grid.talk(&mut controller, at).contains(text),
            "interaction {at:?}"
        );
    }
    for at in [(113, 13), (45, 97), (45, 17)] {
        assert!(grid.talk(&mut controller, at).contains("found"));
    }
    grid.walk(&mut controller, (66, 23));
    assert_eq!(
        controller
            .snapshot()
            .expect("cave arrival")
            .overworld
            .map_name,
        "MnR12Cave1F"
    );
    indoor_walk(&mut controller, &runtime, (5, 6));
    controller.press(GameButton::Up).expect("face Otta");
    controller
        .press(GameButton::A)
        .expect("shelter conversation");
    assert!(
        controller
            .snapshot()
            .expect("Otta text")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("dry rock roof"))
    );
    finish_text(&mut controller);
    for at in [(12, 8), (19, 8), (27, 6)] {
        indoor_walk(&mut controller, &runtime, at);
    }
    controller
        .press(GameButton::Up)
        .expect("face original cave cache");
    controller
        .press(GameButton::A)
        .expect("collect original Max Ether");
    finish_text(&mut controller);
    for at in [(19, 16), (12, 16), (17, 22), (17, 23)] {
        indoor_walk(&mut controller, &runtime, at);
    }
    assert_eq!(position(&mut controller), (66, 23));
    // The native return arrives at entrance warp1, then presentation settles
    // one tile south on the clear footprint before deliberate re-entry.
    controller
        .wait_frames(60)
        .expect("native exterior return settling");
    assert_eq!(position(&mut controller), (66, 24));
    grid.walk(&mut controller, (66, 24));
    grid.walk(&mut controller, (66, 23));
    indoor_walk(&mut controller, &runtime, (17, 22));
    indoor_walk(&mut controller, &runtime, (17, 23));
    assert_eq!(
        position(&mut controller),
        (66, 23),
        "second cave return arrives at native warp"
    );
    controller
        .wait_frames(60)
        .expect("second cave passive return settles");
    assert_eq!(
        position(&mut controller),
        (66, 24),
        "second cave return clears entrance"
    );
    grid.walk(&mut controller, (66, 24));
    grid.walk(&mut controller, (85, 83));
    assert_eq!(
        controller
            .snapshot()
            .expect("hut arrival")
            .overworld
            .map_name,
        "MnR12RangerHut"
    );
    indoor_walk(&mut controller, &runtime, (7, 4));
    controller.press(GameButton::Up).expect("face June");
    controller
        .press(GameButton::A)
        .expect("native Q07 direction text");
    assert!(
        controller
            .snapshot()
            .expect("June directions")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("LODE went west"))
    );
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (4, 1));
    controller.press(GameButton::Up).expect("face wall map");
    controller.press(GameButton::A).expect("read wall map");
    assert!(
        controller
            .snapshot()
            .expect("wall map")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("GUNFLINT"))
    );
    finish_text(&mut controller);
    indoor_walk(&mut controller, &runtime, (4, 8));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("leave over native carpet");
        if controller.snapshot().expect("hut exit").overworld.map_name != "MnR12RangerHut" {
            break;
        }
    }
    assert_eq!(position(&mut controller), (85, 83));
    controller
        .wait_frames(60)
        .expect("native hut return settling");
    assert_eq!(position(&mut controller), (85, 84));
    grid.walk(&mut controller, (85, 84));
    grid.walk(&mut controller, (85, 83));
    indoor_walk(&mut controller, &runtime, (5, 8));
    for _ in 0..8 {
        controller
            .press(GameButton::Down)
            .expect("leave over second native carpet");
        if controller
            .snapshot()
            .expect("hut second exit")
            .overworld
            .map_name
            != "MnR12RangerHut"
        {
            break;
        }
    }
    assert_eq!(
        position(&mut controller),
        (85, 83),
        "second hut return arrives at native warp"
    );
    controller
        .wait_frames(60)
        .expect("second hut passive return settles");
    assert_eq!(
        position(&mut controller),
        (85, 84),
        "second hut return clears entrance"
    );
    grid.walk(&mut controller, (85, 84));
    for flag in [
        "EVENT_MN_R12_ITEM_02",
        "EVENT_MN_R12_ITEM_03",
        "EVENT_MN_R12_ITEM_04",
        "EVENT_MN_R12_CAVE_ETHER",
    ] {
        assert!(
            controller
                .shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set(flag)
                .expect("native pickup flag")
        );
    }
    assert!(
        !controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R12_ITEM_01")
            .expect("herb cache remains sealed")
    );
    grid.walk(&mut controller, (2, 56));
    controller
        .press(GameButton::Start)
        .expect("production Start");
    assert!(controller.snapshot().expect("Start menu").ui.menu.is_some());
    controller.press(GameButton::B).expect("close Start");
    if let Some(path) = std::env::var_os("CRYSTAL_GUNFLINT_FIXTURE_SAVE") {
        controller
            .save(PathBuf::from(path))
            .expect("geometry fixture export");
    }
}

#[test]
#[ignore = "requires the revised external Gunflint pack"]
fn gunflint_cut_cache_denies_opens_collects_and_continues_native_tree_reset() {
    let (root, runtime) = runtime();
    for (move_name, hive_badge) in [("SWIFT", true), ("CUT", false)] {
        let mut denied = fixture(&root, &runtime, (54, 24));
        initial_partner(&runtime, &mut denied, move_name, hive_badge);
        assert_eq!(tree(&denied), 0x0f);
        blocked(&mut denied, GameButton::Up);
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
        assert_eq!(position(&mut denied), (54, 24));
        assert!(!grid(&runtime, (54, 24), false).floor.contains(&(54, 19)));
    }
    let mut controller = fixture(&root, &runtime, (54, 24));
    initial_partner(&runtime, &mut controller, "CUT", true);
    assert!(!grid(&runtime, (54, 24), false).floor.contains(&(54, 19)));
    cut(&mut controller, GameButton::Up);
    assert_eq!(tree(&controller), 0x17);
    let opened = opened_grid(&runtime, &mut controller);
    opened.walk(&mut controller, (54, 19));
    let pickup_text = opened.talk(&mut controller, (53, 19));
    assert!(
        pickup_text.contains("found"),
        "actual herb pickup page: {pickup_text}"
    );
    opened.walk(&mut controller, (54, 19));
    let collected = controller.snapshot().expect("actual herb reward");
    assert!(
        collected
            .bag
            .items
            .iter()
            .any(|s| s.item_id == "HYPER_POTION" && s.quantity == 1)
    );
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R12_ITEM_01")
            .expect("native herb pickup flag")
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture save clock")
        .as_nanos();
    let save = super::authored_route_fixture::reached_save_path(&format!(
        "geothite-gunflint-herb-{stamp}.crystalsave"
    ));
    controller.save(&save).expect("save actual reward and Cut");
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), save.clone(), None)
            .expect("Continue sealed-area visit");
    let continued = controller.snapshot().expect("continued herb reward");
    assert_eq!(continued.party, collected.party);
    assert_eq!(continued.bag, collected.bag);
    assert_eq!(position(&mut controller), (54, 19));
    assert_eq!(tree(&controller), 0x0f);
    let inside = grid(&runtime, (54, 19), false);
    inside.walk(&mut controller, (54, 22));
    cut(&mut controller, GameButton::Down);
    opened_grid(&runtime, &mut controller).walk(&mut controller, (54, 24));
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R12_ITEM_01")
            .expect("continued reward flag")
    );
    let escaped_save = super::authored_route_fixture::reached_save_path(&format!(
        "geothite-gunflint-herb-escaped-{stamp}.crystalsave"
    ));
    controller
        .save(&escaped_save)
        .expect("save escaped herb clearing");
    let mut loaded = VisibleShellController::load_save(
        root.clone(),
        runtime.clone(),
        escaped_save.clone(),
        None,
    )
    .expect("Continue escaped clearing");
    assert_eq!(position(&mut loaded), (54, 24));
    assert_eq!(tree(&loaded), 0x0f);
    assert_eq!(
        loaded.snapshot().expect("no duplicate reward").bag,
        collected.bag
    );
    // June is navigation only. Observe the injured/PP-deficient partner through
    // her actual conversation without claiming an unauthored healing service.
    let dry = grid(&runtime, (54, 24), false);
    dry.walk(&mut loaded, (85, 83));
    indoor_walk(&mut loaded, &runtime, (7, 4));
    let before_june = loaded.snapshot().expect("injured partner before June");
    assert!(before_june.party.slots[0].pokemon.hp < before_june.party.slots[0].pokemon.max_hp);
    assert!(
        before_june.party.slots[0].pokemon.moves[0].current_pp < runtime.data().moves["CUT"].pp
    );
    loaded.press(GameButton::Up).expect("face June");
    loaded
        .press(GameButton::A)
        .expect("June navigation dialogue");
    finish_text(&mut loaded);
    assert_eq!(
        loaded.snapshot().expect("navigation does not heal").party,
        before_june.party
    );
    indoor_walk(&mut loaded, &runtime, (4, 8));
    for _ in 0..8 {
        loaded
            .press(GameButton::Down)
            .expect("native hut departure");
        if loaded
            .snapshot()
            .expect("exterior return")
            .overworld
            .map_name
            != "MnR12RangerHut"
        {
            break;
        }
    }
    assert_eq!(position(&mut loaded), (85, 83));
    dry.walk(&mut loaded, (85, 84));
    let after_hut = loaded.snapshot().expect("completed native room return");
    assert_eq!(
        after_hut.party.slots[0].pokemon.hp,
        before_june.party.slots[0].pokemon.hp
    );
    assert_eq!(
        after_hut.party.slots[0].pokemon.moves,
        before_june.party.slots[0].pokemon.moves
    );
    let returned_save = super::authored_route_fixture::reached_save_path(&format!(
        "geothite-gunflint-herb-room-return-{stamp}.crystalsave"
    ));
    loaded
        .save(&returned_save)
        .expect("save native room-return journey");
    let mut continued =
        VisibleShellController::load_save(root, runtime, returned_save.clone(), None)
            .expect("Continue room return");
    assert_eq!(position(&mut continued), (85, 84));
    let final_snapshot = continued.snapshot().expect("continued partner after June");
    assert_eq!(final_snapshot.party, after_hut.party);
    assert_eq!(final_snapshot.bag, collected.bag);
    super::authored_route_fixture::cleanup_reached_save(&save);
    super::authored_route_fixture::cleanup_reached_save(&escaped_save);
    super::authored_route_fixture::cleanup_reached_save(&returned_save);
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
#[ignore = "requires the revised external Gunflint pack"]
fn gunflint_populated_fixture_wins_spends_selected_pp_and_continues() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, (107, 52));
    // Legal local party, badges and deterministic divider are initial setup.
    // The test starts beside Milo; it does not claim earned regional travel.
    for (species, level, move_name) in
        [("FERALIGATR", 52, "SURF"), ("AMPHAROS", 46, "THUNDERPUNCH")]
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
    state.badges.johto = [true; 8];
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
    controller.press(GameButton::Up).expect("face Milo");
    controller
        .press(GameButton::A)
        .expect("native trainer challenge");
    assert_eq!(
        complete_battle(&mut controller),
        BTreeSet::from(["NOCTOWL".into(), "XATU".into()])
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
            .is_event_flag_set("EVENT_MN_R12_TRAINER_02")
            .expect("native victory flag")
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture save clock")
        .as_nanos();
    let save = super::authored_route_fixture::reached_save_path(&format!(
        "geothite-gunflint-battle-{stamp}.crystalsave"
    ));
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
            .is_event_flag_set("EVENT_MN_R12_TRAINER_02")
            .expect("continued trainer victory")
    );
    if let Some(path) = std::env::var_os("CRYSTAL_GUNFLINT_JOURNEY_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("optional controller fixture save");
    }
    super::authored_route_fixture::cleanup_reached_save(&save);
}

#[test]
#[ignore = "requires final winter pack; explicit legal Surf/party/badge setup"]
fn gunflint_winter_bog_side_bay_surfs_from_main_bank_reads_sign_and_approaches_trainer() {
    use crystal_core::world::movement::MovementMode;
    use std::collections::VecDeque;

    let (root, runtime) = runtime();
    assert_eq!(
        runtime.data().maps["MnR12C00R01"].attributes.tileset_name,
        "mn_winter_gunflint"
    );
    assert!(
        grid(&runtime, (124, 56), true).floor.contains(&(50, 84)),
        "departure belongs to main route Foot component"
    );
    let mut controller = fixture(&root, &runtime, (50, 84));
    initial_partner(&runtime, &mut controller, "SURF", true);
    assert!(
        controller
            .shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&runtime.data().items["MAX_REPEL"], 5)
            .expect("initial legal Repel fixture")
    );
    mark_runtime_snapshot_dirty(&mut controller.shell);
    // Initial setup ends here. The cold side pocket is reached only by native
    // mount, paddling and automatic dismount; no terrain or position writes.
    super::minneapolis_falls::grand_marais_harbor::use_max_repel(&mut controller, &runtime);
    let map = "MnR12C00R01";
    super::minneapolis_falls::mount(&mut controller, GameButton::Left, map, (49, 36));
    let module = &runtime.data().maps[map];
    let collision = &runtime.data().tilesets[&module.attributes.tileset_name].collision;
    let start = (49_i16, 36_i16);
    let target = (38_i16, 30_i16);
    let mut previous = BTreeMap::from([(start, None)]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for (next, button) in super::authored_route_fixture::neighbors(at) {
            if next.0 < 0
                || next.1 < 0
                || next.0 >= 64
                || next.1 >= 48
                || previous.contains_key(&next)
            {
                continue;
            }
            let block = module.blocks[(next.1 as usize / 2) * usize::from(module.attributes.width)
                + next.0 as usize / 2];
            if collision[&format!("{block:02x}")][(next.1 as usize % 2) * 2 + next.0 as usize % 2]
                != "WATER"
            {
                continue;
            }
            previous.insert(next, Some((at, button)));
            queue.push_back(next);
        }
    }
    assert!(
        previous.contains_key(&target),
        "real connected lake reaches optional side bay"
    );
    let mut cursor = target;
    let mut path = Vec::new();
    while let Some((prior, button)) = previous[&cursor] {
        path.push((cursor, button));
        cursor = prior;
    }
    for (at, button) in path.into_iter().rev() {
        super::minneapolis_falls::step(
            &mut controller,
            button,
            map,
            at,
            "actual bog lake paddling",
        );
    }
    super::minneapolis_falls::step(
        &mut controller,
        GameButton::Left,
        map,
        (37, 30),
        "native bay bank dismount",
    );
    assert_eq!(
        controller
            .snapshot()
            .expect("dismounted bog bay")
            .overworld
            .mode,
        MovementMode::Normal
    );
    let pocket = grid(&runtime, (37, 78), false);
    assert!(
        pocket.talk(&mut controller, (35, 75)).contains("STONE BOG"),
        "native optional bay sign remains useful"
    );
    pocket.walk(&mut controller, (36, 79));
    assert_eq!(
        position(&mut controller),
        (36, 79),
        "actual safe trainer approach in side bay"
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("reached bay clock")
        .as_nanos();
    let save = super::authored_route_fixture::reached_save_path(&format!(
        "gunflint-winter-bog-side-bay-{stamp}.crystalsave"
    ));
    controller
        .save(&save)
        .expect("preserve reached sign and native trainer approach");
    controller
        .press(GameButton::Right)
        .expect("face retained bay trainer");
    controller
        .press(GameButton::A)
        .expect("interact with original bay trainer");
    for _ in 0..64 {
        if controller
            .snapshot()
            .expect("native trainer transition")
            .battle
            .is_some()
        {
            break;
        }
        controller
            .press(GameButton::A)
            .expect("original trainer challenge page");
        controller
            .wait_frames(16)
            .expect("native trainer transition frames");
    }
    assert!(
        controller
            .snapshot()
            .expect("real bay trainer challenge")
            .battle
            .is_some()
    );
    super::authored_route_fixture::cleanup_reached_save(&save);
}
