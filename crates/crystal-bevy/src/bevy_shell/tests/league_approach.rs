//! Explicit geometry, field-service and late-battle fixtures for the revised route.
//! These do not establish earned badges or the connected Q08 journey.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    RouteGrid, RouteSpec, escape_wild, finish_text, indoor_walk, position,
};

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
            tileset: if runtime.data().maps["MnR13C00R00"].attributes.tileset_name
                == "mn_winter_north_shore"
            {
                "mn_winter_north_shore"
            } else {
                "johto"
            },
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
    super::authored_route_fixture::reached_save_path(&format!(
        "geothite-league-{label}-{stamp}.crystalsave"
    ))
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
        (47, 16),
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
        ((39, 65), "BOOMRUFF"),
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
    super::authored_route_fixture::cleanup_reached_save(&save);
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

// Initial legal local fixture only; every subsequent change uses production input.
fn grass_partner(c: &mut VisibleShellController, runtime: &CrystalRuntime, repel: bool) {
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("same legal level60 local partner");
    partner.moves = ["EARTHQUAKE", "SLUDGE_BOMB", "THUNDERBOLT", "CUT"]
        .into_iter()
        .map(|name| {
            assert!(
                runtime.data().pokemon["MEW"]
                    .tmhm_learnset
                    .iter()
                    .any(|m| m == name)
            );
            LearnedMove {
                name: name.into(),
                current_pp: runtime.data().moves[name].pp,
                pp_ups: 0,
            }
        })
        .collect();
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.badges.johto = [true; 8];
    for name in [
        "ZEPHYR", "HIVE", "PLAIN", "FOG", "STORM", "MINERAL", "GLACIER", "RISING",
    ] {
        state
            .flags
            .set_engine_flag(&format!("ENGINE_{name}BADGE"), true)
            .expect("consistent initial badge fixture");
    }
    if repel {
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["MAX_REPEL"], 2)
                .expect("initial Repel stock")
        );
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
}

fn native_kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let module = &runtime.data().maps[&format!("MnR13C00R{:02}", at.1 / 48)];
    let x = usize::try_from(at.0).expect("positive native x");
    let y = usize::try_from(at.1 % 48).expect("positive native y");
    let block = module.blocks[y / 2 * usize::from(module.attributes.width) + x / 2];
    &runtime.data().tilesets[&module.attributes.tileset_name].collision[&format!("{block:02x}")]
        [y % 2 * 2 + x % 2]
}

#[test]
#[ignore = "requires the current parent League Approach grass pack"]
fn league_approach_controller_native_avrin_battle_spends_effective_pp_wins_and_continues() {
    let (root, runtime, mut c) = fixture("MnR13C00R00", (23, 24));
    grass_partner(&mut c, &runtime, false);
    assert_eq!(
        runtime.data().trainers.trainers["MN_R13_T01"]
            .party
            .iter()
            .map(|p| p.species.as_str())
            .collect::<Vec<_>>(),
        ["BUOYOSTIC", "REEDRAFT"]
    );
    let before = c.snapshot().expect("explicit initial party");
    super::r10_gallery::press(&mut c, GameButton::Down);
    super::r10_gallery::press(&mut c, GameButton::A);
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &runtime,
        "MN_R13_T01",
        "EVENT_MN_R13_TRAINER_01"
    ));
    super::r10_gallery::settle(&mut c, "actual Avrin victory");
    let won = c.snapshot().expect("native battle complete");
    assert!(won.battle.is_none() && won.party.slots[0].pokemon.hp > 0);
    assert!(
        won.party.slots[0]
            .pokemon
            .moves
            .iter()
            .any(|m| before.party.slots[0]
                .pokemon
                .moves
                .iter()
                .any(|old| old.name == m.name && m.current_pp < old.current_pp)),
        "actual effective attack spends PP"
    );
    let save = save_path("current-avrin-win");
    c.save(&save).expect("save actual victory");
    let mut loaded = VisibleShellController::load_save(root, runtime, save, None)
        .expect("Continue actual native victory");
    let after = loaded.snapshot().expect("continued party and route");
    assert_eq!(won.party, after.party);
    assert_eq!(won.bag, after.bag);
    assert_eq!(won.overworld.map_name, after.overworld.map_name);
    assert_eq!(won.overworld.tile, after.overworld.tile);
    assert!(
        loaded
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R13_TRAINER_01")
            .expect("continued victory flag")
    );
    if let Some(path) = std::env::var_os("CRYSTAL_LEAGUE_APPROACH_GRASS_BATTLE_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export current grass battle reached save");
    }
}

#[test]
#[ignore = "requires the current parent League Approach grass pack"]
fn league_approach_controller_grassy_tor_climb_has_dry_hop_return_and_continue() {
    let (root, runtime, mut c) = fixture("MnR13C00R00", (39, 26));
    grass_partner(&mut c, &runtime, true);
    super::minneapolis_falls::grand_marais_harbor::use_max_repel(&mut c, &runtime);
    let route = grid(&runtime);
    let mut grass = 0;
    route.walk_with_actions(
        &mut c,
        (39, 23),
        |c| {
            grass += usize::from(native_kind(&runtime, position(c)) == "TALL_GRASS");
        },
        escape_wild,
    );
    assert!(grass >= 2, "ordinary ramp climb must cross added habitat");
    assert_eq!(native_kind(&runtime, (39, 23)), "FLOOR");
    assert_eq!(native_kind(&runtime, (39, 26)), "FLOOR");
    step(&mut c, GameButton::Down, (39, 26));
    for _ in 0..4 {
        c.press(GameButton::Up).expect("blocked reverse hop");
    }
    assert_eq!(position(&mut c), (39, 26));
    let before = c.snapshot().expect("actual dry drop return");
    let save = save_path("grass-tor-return");
    c.save(&save).expect("save reached tor return");
    let mut loaded =
        VisibleShellController::load_save(root, runtime, save, None).expect("Continue tor return");
    let after = loaded.snapshot().expect("continued tor return");
    assert_eq!(position(&mut loaded), (39, 26));
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
}

#[test]
#[ignore = "requires the current parent League Approach grass pack"]
fn league_approach_controller_new_pine_neck_grass_encounter_native_run_and_continue() {
    let (root, runtime, mut c) = fixture("MnR13C00R02", (35, 9));
    grass_partner(&mut c, &runtime, false);
    assert_eq!(native_kind(&runtime, (35, 106)), "TALL_GRASS");
    for _ in 0..512 {
        if c.snapshot()
            .expect("native habitat encounter")
            .battle
            .is_some()
        {
            break;
        }
        let input = if position(&mut c).1 <= 106 {
            GameButton::Down
        } else {
            GameButton::Up
        };
        super::r10_gallery::press(&mut c, input);
    }
    assert!(matches!(
        c.snapshot()
            .expect("new native grass battle")
            .battle
            .expect("actual habitat must encounter")
            .kind,
        RuntimeBattleKind::Wild { .. }
    ));
    escape_wild(&mut c);
    let escaped = c.snapshot().expect("native RUN complete");
    assert!(escaped.battle.is_none());
    assert_eq!(escaped.overworld.map_name, "MnR13C00R02");
    let save = save_path("pine-neck-wild-run");
    c.save(&save).expect("save actual wild escape");
    let mut loaded = VisibleShellController::load_save(root, runtime, save, None)
        .expect("Continue actual wild escape");
    let after = loaded.snapshot().expect("continued new habitat");
    assert_eq!(escaped.party, after.party);
    assert_eq!(escaped.bag, after.bag);
    assert_eq!(escaped.overworld.tile, after.overworld.tile);
}
