//! Local geometry regression for the bounded R18 shore-trip revision.
//!
//! Local position, an empty party and RNG are
//! explicit initial fixtures; regional access is not earned here. All later
//! movement uses VisibleShellController::press through RouteGrid. Existing
//! populated SQ04/battle/Surf cases remain separate acceptance requirements.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, position};

fn fixture() -> (CrystalRuntime, VisibleShellController, RouteGrid) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_ROUTE_QUALITY_R18_PACK")
            .expect("published external R18 shore-trip revision"),
    );
    let root = AssetRoot::new(path.parent().expect("external pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("strict verified R18 pack"),
    )
    .expect("native R18 runtime");
    let shell = initialize_bevy_runtime_shell(
        root,
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: "MnR18C00R00".into(),
            tile_x: 2,
            tile_y: 10,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit initial western route placement");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture trainer name");
    // Only the initial RNG is assigned. The fresh game's empty party, flags,
    // inventory and badges are retained. No state is assigned after setup.
    controller
        .shell
        .shell
        .session_mut()
        .state_mut()
        .random_state = Default::default();
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert!(
        controller
            .snapshot()
            .expect("initial empty party")
            .party
            .slots
            .is_empty()
    );
    assert_eq!(position(&mut controller), (2, 10));
    let grid = RouteGrid::new_with_grass(
        &runtime,
        RouteSpec {
            prefix: "MnR18",
            columns: 2,
            rows: 1,
            tileset: "mn_headwaters",
            start: (2, 10),
            doors: BTreeMap::new(),
        },
    );
    (runtime, controller, grid)
}

fn collision(runtime: &CrystalRuntime, (x, y): Point) -> &str {
    assert!((0..80).contains(&x) && (0..48).contains(&y));
    let name = format!("MnR18C{:02}R00", x / 64);
    let module = runtime
        .data()
        .assemble_map_module_from_compiled_payloads(&name)
        .expect("installed R18 chunk");
    let local_x = (x % 64) as usize;
    let local_y = y as usize;
    let block = module.blocks[local_y / 2 * usize::from(module.attributes.width) + local_x / 2];
    &runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")]
        [local_y % 2 * 2 + local_x % 2]
}

fn walk_many(controller: &mut VisibleShellController, grid: &RouteGrid, points: &[Point]) {
    for &point in points {
        grid.walk(controller, point);
        let state = controller.snapshot().expect("actual geometry checkpoint");
        assert_eq!(state.phase, RuntimeShellPhase::Overworld);
        assert!(
            state.battle.is_none(),
            "six original trainer sightlines must be avoidable"
        );
        assert_eq!(controller.shell.last_error, None);
    }
}

fn blocked(controller: &mut VisibleShellController, button: GameButton) {
    let before = position(controller);
    for _ in 0..3 {
        controller.press(button).expect("native barrier input");
        assert_eq!(
            position(controller),
            before,
            "native terrain must block movement"
        );
    }
}

#[test]
#[ignore = "requires the external published R18 shore-trip revision"]
fn route_quality_r18_controller_whole_shore_trip_and_both_returns() {
    let (runtime, mut controller, grid) = fixture();
    let initial = controller.snapshot().expect("initial resources");
    let initial_flags = controller.shell.shell.session().state().flags.clone();
    // Western headland leads to a visible lake before the mandatory habitat.
    // The new optional shorewalk is far to the east of that habitat; it must
    // not be mistaken for a bypass of the retained six-grass neck.
    walk_many(
        &mut controller,
        &grid,
        &[(11, 11), (19, 14), (27, 14), (31, 10), (35, 13)],
    );
    grid.walk(&mut controller, (42, 13));
    for x in 36..42 {
        assert_eq!(collision(&runtime, (x, 13)), "TALL_GRASS");
    }
    grid.walk(&mut controller, (35, 13));
    let neck = grid.path((35, 13), (42, 13));
    assert_eq!(neck, vec![GameButton::Right; 7]);
    let mut grass_entered = 0;
    for x in 36..=42 {
        grid.walk(&mut controller, (x, 13));
        grass_entered +=
            usize::from(collision(&runtime, position(&mut controller)) == "TALL_GRASS");
    }
    assert_eq!(
        grass_entered, 6,
        "actual forward steps through the original habitat"
    );
    let mut reverse_grass = 0;
    for x in (35..42).rev() {
        grid.walk(&mut controller, (x, 13));
        reverse_grass +=
            usize::from(collision(&runtime, position(&mut controller)) == "TALL_GRASS");
    }
    assert_eq!(reverse_grass, 6, "same habitat cost on the actual return");
    walk_many(&mut controller, &grid, &[(42, 13), (45, 13)]);
    // The optional central two-grass shortcut is a separate local choice.
    assert_eq!(grid.path((45, 13), (48, 13)).len(), 3);
    let dry = RouteGrid::new(
        &runtime,
        RouteSpec {
            prefix: "MnR18",
            columns: 2,
            rows: 1,
            tileset: "mn_headwaters",
            start: (45, 13),
            doors: BTreeMap::new(),
        },
    );
    // RouteGrid already restricts to this start's accessible dry component.
    assert_eq!(dry.path((45, 13), (48, 13)).len(), 9);
    dry.walk(&mut controller, (48, 13));
    grid.walk(&mut controller, (45, 13));
    for x in 46..=48 {
        grid.walk(&mut controller, (x, 13));
        assert_eq!(
            collision(&runtime, (x, 13)),
            if x < 48 { "TALL_GRASS" } else { "FLOOR" }
        );
    }
    walk_many(
        &mut controller,
        &grid,
        &[
            (49, 19),
            (59, 18),
            (63, 18),
            (64, 18),
            (71, 18),
            (75, 18),
            (76, 10),
        ],
    );
    // Full east-west return through the optional lower shorewalk.
    walk_many(
        &mut controller,
        &grid,
        &[
            (75, 18),
            (71, 18),
            (64, 18),
            (63, 18),
            (59, 18),
            (49, 19),
            (48, 13),
        ],
    );
    // A second return uses the inherited upper inhabited road instead.
    walk_many(
        &mut controller,
        &grid,
        &[(75, 18), (73, 11), (65, 11), (57, 11), (49, 11), (48, 13)],
    );
    walk_many(
        &mut controller,
        &grid,
        &[(59, 18), (60, 18), (60, 17), (63, 17), (64, 17), (65, 17)],
    );
    assert_eq!(collision(&runtime, (65, 16)), "WALL");
    assert_eq!(collision(&runtime, (65, 19)), "WALL");
    blocked(&mut controller, GameButton::Up);
    grid.walk(&mut controller, (65, 18));
    blocked(&mut controller, GameButton::Down);
    // Return west around the headland, then actually reach both retained piers,
    // all three walking pickups and the SQ04 volunteers/ranger approaches.
    walk_many(
        &mut controller,
        &grid,
        &[
            (48, 13),
            (48, 16),
            (45, 16),
            (45, 13),
            (42, 13),
            (35, 13),
            (31, 10),
            (27, 12),
            (27, 14),
            (19, 14),
            (11, 11),
            (13, 23),
            (26, 23),
            (29, 23),
            (13, 23),
            (13, 28),
            (14, 33),
            (19, 34),
            (19, 35),
            (13, 35),
            (13, 28),
            (12, 31),
            (26, 7),
            (31, 10),
            (35, 13),
            (42, 13),
            (45, 16),
            (48, 16),
            (48, 13),
            (72, 9),
            (75, 18),
            (49, 19),
        ],
    );
    for actor in [
        (15, 33),
        (31, 9),
        (75, 17),
        (27, 23),
        (11, 31),
        (27, 7),
        (73, 9),
    ] {
        assert!(
            grid.blocked.contains(&actor),
            "native actor body at {actor:?}"
        );
    }
    // No Surf ability is injected into this empty-party geometry case. Prove
    // the original usable launch and closed outer water terrain; the separate
    // populated original controller case earns the actual mount/islet item.
    assert_eq!(collision(&runtime, (49, 19)), "FLOOR");
    assert_eq!(collision(&runtime, (49, 20)), "WATER");
    assert_eq!(collision(&runtime, (58, 32)), "WALL");
    assert_eq!(collision(&runtime, (61, 34)), "FLOOR");
    assert_eq!(collision(&runtime, (79, 25)), "BUOY");
    assert_eq!(collision(&runtime, (65, 47)), "BUOY");
    for y in 0..48 {
        for x in 0..80 {
            if x == 0 || x == 79 || y == 0 || y == 47 {
                assert_ne!(
                    collision(&runtime, (x, y)),
                    "WATER",
                    "no open outer Surf escape"
                );
            }
        }
    }
    let after = controller
        .snapshot()
        .expect("unchanged resources after native walking");
    assert_eq!(after.party, initial.party);
    assert_eq!(after.bag, initial.bag);
    let flags = &controller.shell.shell.session().state().flags;
    // Crossing a native map seam materializes false temporary flags. Compare
    // every event/engine flag's value, including keys absent from either map.
    for flag in initial_flags
        .event_flags
        .keys()
        .chain(flags.event_flags.keys())
    {
        assert_eq!(
            initial_flags
                .is_event_flag_set(flag)
                .expect("known initial event"),
            flags.is_event_flag_set(flag).expect("known current event"),
            "event {flag}"
        );
    }
    for flag in initial_flags
        .engine_flags
        .keys()
        .chain(flags.engine_flags.keys())
    {
        assert_eq!(
            initial_flags
                .is_engine_flag_set(flag)
                .expect("known initial engine flag"),
            flags
                .is_engine_flag_set(flag)
                .expect("known current engine flag"),
            "engine {flag}"
        );
    }
}

/// Party, initial placement and RNG are local fixtures; encounters and the
/// journey thereafter use native inputs without rewriting game state.
#[test]
#[ignore = "requires the current R18 grass candidate"]
fn route_quality_r18_controller_western_grass_wild_run_return_and_continue() {
    use super::authored_route_fixture::{cleanup_reached_save, escape_wild, reached_save_path};

    let (runtime, mut controller, grid) = fixture();
    let partner = runtime
        .data()
        .create_pokemon("MEW", 60, crystal_core::models::Dv::default())
        .expect("explicit native fixture partner; not authored Minnesota content");
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut controller.shell);

    let mut grass = 0;
    grid.walk_with_actions(
        &mut controller,
        (26, 12),
        |c| grass += usize::from(collision(&runtime, position(c)) == "TALL_GRASS"),
        escape_wild,
    );
    assert!(grass >= 4, "western approach must cross the new meadow");
    grid.walk_with_actions(&mut controller, (15, 11), |_| {}, escape_wild);
    assert_eq!(collision(&runtime, (15, 11)), "TALL_GRASS");
    assert_eq!(collision(&runtime, (16, 11)), "TALL_GRASS");
    let mut encounters = 0;
    for index in 0..256 {
        let target = if index % 2 == 0 { (16, 11) } else { (15, 11) };
        grid.walk_with_actions(
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
        "new grass must start a native wild encounter"
    );
    grid.walk_with_actions(&mut controller, (6, 10), |_| {}, escape_wild);
    let before = controller
        .snapshot()
        .expect("actual reached western return");
    let flags = controller.shell.shell.session().state().flags.clone();
    let save = reached_save_path(&format!(
        "geothite-r18-grass-{}.crystalsave",
        std::process::id()
    ));
    controller.save(&save).expect("save actual grass return");
    let pack = PathBuf::from(std::env::var_os("CRYSTAL_ROUTE_QUALITY_R18_PACK").expect("R18 pack"));
    let root = AssetRoot::new(pack.parent().expect("external pack parent"));
    let mut loaded = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("Continue actual grass return");
    let after = loaded.snapshot().expect("continued resources");
    assert_eq!(position(&mut loaded), (6, 10));
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(loaded.shell.shell.session().state().flags, flags);
    if let Some(path) = std::env::var_os("CRYSTAL_R18_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached save");
    }
    cleanup_reached_save(&save);
}
