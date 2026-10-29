//! Explicit local party/position fixtures followed only by production inputs.
//! These checks do not establish earned regional access or encounter balance.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::CrystalRuntime;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, cleanup_reached_save, escape_wild, position, reached_save_path,
};

fn fixture(at: Point) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let path =
        PathBuf::from(std::env::var_os("CRYSTAL_CENTRAL_LAKES_TEST_PACK").expect("current pack"));
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(path).expect("verified current pack"),
    )
    .expect("native runtime");
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime
                .title_new_game_spawn_identifier()
                .expect("native HOME"),
            map_name: "MnR15C00R00".into(),
            tile_x: at.0,
            tile_y: at.1,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    // Initial native-created partner, legal move PP and consistent obedience bits.
    let mut partner = runtime
        .data()
        .create_pokemon("MEW", 60, Dv::default())
        .expect("native fixture partner");
    partner.moves = ["EARTHQUAKE", "SLUDGE_BOMB", "THUNDERBOLT", "PSYCHIC_M"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: runtime.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.pokemon.iter().all(Option::is_none));
    assert!(state.storage.party.add_pokemon(partner));
    state.badges.johto = [true; 8];
    for name in [
        "ZEPHYR", "HIVE", "PLAIN", "FOG", "STORM", "MINERAL", "GLACIER", "RISING",
    ] {
        state
            .flags
            .set_engine_flag(&format!("ENGINE_{name}BADGE"), true)
            .expect("initial permission bits");
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut controller.shell);
    controller.set_runtime_journal_enabled(false);
    (root, runtime, controller)
}

fn route(runtime: &CrystalRuntime, start: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR15",
            columns: 1,
            rows: 1,
            tileset: "kanto",
            start,
            doors: BTreeMap::from([((53, 11), "MnR15LakeCabin")]),
        },
    )
}

fn kind(runtime: &CrystalRuntime, at: Point) -> &str {
    let m = &runtime.data().maps["MnR15C00R00"];
    let x = usize::try_from(at.0).expect("nonnegative x");
    let y = usize::try_from(at.1).expect("nonnegative y");
    let block = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    &runtime.data().tilesets["kanto"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn continue_reached(
    root: AssetRoot,
    runtime: CrystalRuntime,
    c: &mut VisibleShellController,
    label: &str,
) {
    let before = c.snapshot().expect("reached native state");
    let at = position(c);
    let save = reached_save_path(&format!(
        "geothite-r15-{label}-{}.crystalsave",
        std::process::id()
    ));
    c.save(&save).expect("save actual reached state");
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("native Continue");
    let after = loaded.snapshot().expect("continued native state");
    assert_eq!(position(&mut loaded), at);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    if let Some(path) = std::env::var_os("CRYSTAL_CENTRAL_LAKES_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("export reached native state");
    }
    cleanup_reached_save(&save);
}

#[test]
#[ignore = "requires current Central Lakes grass pack"]
fn central_lakes_grass_controller_climb_dry_drop_reverse_denial_and_continue() {
    let (root, runtime, mut c) = fixture((22, 20));
    // The installed trainer and its sight tile seal the lower woodland join.
    // Pay that existing battle through native input before testing the climb.
    route(&runtime, (22, 20)).walk_with_actions(&mut c, (18, 24), |_| {}, escape_wild);
    c.press(GameButton::Left).expect("face woodland trainer");
    c.press(GameButton::A)
        .expect("native woodland introduction");
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &runtime,
        "MN_R15_T02",
        "EVENT_MN_R15_TRAINER_02",
    ));
    super::authored_route_fixture::finish_text(&mut c);
    for (button, target) in [
        (GameButton::Down, (18, 25)),
        (GameButton::Left, (17, 25)),
        (GameButton::Left, (16, 25)),
    ] {
        let before = position(&mut c);
        for _ in 0..8 {
            c.press(button)
                .expect("walk past actually defeated trainer");
            let at = position(&mut c);
            assert!(at == before || at == target);
            assert!(c.snapshot().expect("won trainer passage").battle.is_none());
            if at == target {
                break;
            }
        }
        assert_eq!(position(&mut c), target);
    }
    let mut grass = 0;
    route(&runtime, (16, 25)).walk_with_actions(
        &mut c,
        (22, 17),
        |c| {
            grass += usize::from(kind(&runtime, position(c)) == "TALL_GRASS");
        },
        escape_wild,
    );
    assert!(
        grass >= 8,
        "ordinary meadow climb must cross its existing and added habitat"
    );
    for _ in 0..8 {
        c.press(GameButton::Down).expect("native meadow drop");
        let at = position(&mut c);
        assert!(matches!(at, (22, 17) | (22, 18) | (22, 20)));
        assert!(c.snapshot().expect("dry return").battle.is_none());
        if at == (22, 20) {
            break;
        }
    }
    assert_eq!(position(&mut c), (22, 20));
    assert_eq!(kind(&runtime, (22, 20)), "FLOOR");
    for _ in 0..4 {
        c.press(GameButton::Up).expect("reject reverse ascent");
    }
    assert_eq!(position(&mut c), (22, 20));
    continue_reached(root, runtime, &mut c, "grass-return");
}

#[test]
#[ignore = "requires current Central Lakes grass pack"]
fn central_lakes_grass_controller_new_shore_encounter_native_run_and_continue() {
    let (root, runtime, mut c) = fixture((28, 28));
    assert_eq!(kind(&runtime, (28, 28)), "TALL_GRASS");
    assert_eq!(kind(&runtime, (29, 28)), "TALL_GRASS");
    let grid = route(&runtime, (28, 28));
    let mut encounters = 0;
    for index in 0..256 {
        let target = if index % 2 == 0 { (29, 28) } else { (28, 28) };
        grid.walk_with_actions(
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
        "new shore grass never triggered a native wild battle"
    );
    grid.walk_with_actions(&mut c, (30, 28), |_| {}, escape_wild);
    continue_reached(root, runtime, &mut c, "shore-wild");
}

#[test]
#[ignore = "requires current Central Lakes grass pack"]
fn central_lakes_grass_controller_native_trainer_pp_victory_and_continue() {
    let (root, runtime, mut c) = fixture((16, 24));
    let before = c.snapshot().expect("initial native party");
    c.press(GameButton::Right).expect("face actual trainer");
    c.press(GameButton::A).expect("native trainer introduction");
    assert!(super::rochester_earned::try_battle(
        &mut c,
        &runtime,
        "MN_R15_T02",
        "EVENT_MN_R15_TRAINER_02"
    ));
    let won = c.snapshot().expect("actual trainer victory");
    assert!(
        before.party.slots[0]
            .pokemon
            .moves
            .iter()
            .zip(&won.party.slots[0].pokemon.moves)
            .any(|(a, b)| b.current_pp < a.current_pp)
    );
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R15_TRAINER_02")
            .expect("victory flag")
    );
    continue_reached(root, runtime, &mut c, "trainer-victory");
}
