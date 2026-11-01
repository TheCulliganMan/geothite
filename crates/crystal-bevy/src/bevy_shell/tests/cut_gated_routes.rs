//! Explicit position/party/badge fixtures, followed only by production input.
//!
//! These regressions prove local field-move gates and water boundaries. They do
//! not establish an earned Minnesota badge, quest, regional journey or balance.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::world::movement::MovementMode;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellPhase};

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, neighbors, position,
};

struct CutCase {
    prefix: &'static str,
    columns: i16,
    rows: i16,
    tileset: &'static str,
    approach: Point,
    tree: Point,
    inside: Point,
    original: u16,
    replacement: u16,
    reward: Point,
    reward_item: &'static str,
    reward_flag: &'static str,
}

const PINE_COUNTRY: CutCase = CutCase {
    prefix: "MnR06",
    columns: 1,
    rows: 2,
    tileset: "mn_headwaters",
    approach: (29, 33),
    tree: (29, 32),
    inside: (13, 29),
    original: 0x5b,
    replacement: 0x3c,
    reward: (13, 27),
    reward_item: "ETHER",
    reward_flag: "EVENT_MN_R06_ITEM_04",
};
const LEECH_LAKE: CutCase = CutCase {
    prefix: "MnR07",
    columns: 1,
    rows: 2,
    tileset: "mn_headwaters",
    approach: (16, 23),
    tree: (15, 23),
    inside: (14, 23),
    original: 0x5f,
    replacement: 0x3d,
    reward: (13, 23),
    reward_item: "ETHER",
    reward_flag: "EVENT_MN_R07_ITEM_04",
};
const NORTHERN_LAKES: CutCase = CutCase {
    prefix: "MnR08",
    columns: 2,
    rows: 2,
    tileset: "mn_headwaters",
    approach: (6, 22),
    tree: (6, 21),
    inside: (14, 17),
    original: 0x63,
    replacement: 0x3f,
    reward: (15, 17),
    reward_item: "GREAT_BALL",
    reward_flag: "EVENT_MN_R08_ITEM_02",
};

const LEAGUE_APPROACH: CutCase = CutCase {
    prefix: "MnR13",
    columns: 1,
    rows: 3,
    tileset: "johto",
    approach: (18, 102),
    tree: (18, 103),
    inside: (18, 104),
    original: 0x63,
    replacement: 0x3f,
    reward: (21, 109),
    reward_item: "REVIVE",
    reward_flag: "EVENT_MN_R13_ITEM_04",
};

const IRON_RANGE: CutCase = CutCase {
    prefix: "MnR09",
    columns: 2,
    rows: 2,
    tileset: "mn_iron_range",
    approach: (12, 56),
    tree: (12, 57),
    inside: (12, 58),
    original: 0x63,
    replacement: 0x3f,
    reward: (11, 63),
    reward_item: "REVIVE",
    reward_flag: "EVENT_MN_R09_ITEM_04",
};

fn runtime() -> (AssetRoot, CrystalRuntime) {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_CUT_ROUTES_TEST_PACK")
            .expect("published external R05/R06/R07/R08 fixture pack"),
    );
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified external route pack"),
    )
    .expect("native route runtime");
    (root, runtime)
}

fn fixture(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    prefix: &str,
    at: Point,
    moves: &[&str],
    hive_badge: bool,
) -> VisibleShellController {
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: format!("{prefix}C{:02}R{:02}", at.0 / 64, at.1 / 48),
            tile_x: at.0 % 64,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit field-move fixture location");
    let mut controller = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
        .expect("fixture name");
    let mut pokemon = runtime
        .data()
        .create_pokemon("VOYAGEAR", 60, Dv::default())
        .expect("legal HM-compatible fixture partner");
    assert!(
        runtime
            .data()
            .story_event_script_constants
            .global
            .contains_key("MN_DEX_MEMBER_VOYAGEAR"),
        "regional field partner must belong to the active Minnesota roster"
    );
    for name in moves {
        assert!(
            *name == "TACKLE"
                || pokemon
                    .species
                    .tmhm_learnset
                    .iter()
                    .any(|known| known == name),
            "regional field partner must legally learn {name}"
        );
    }
    pokemon.moves = moves
        .iter()
        .map(|&name| LearnedMove {
            name: name.into(),
            current_pp: runtime.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    // This is initial fixture setup, never an earned badge or learned HM claim.
    let state = controller.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(pokemon));
    state.badges.johto = [true; 8];
    state.badges.johto[1] = hive_badge;
    state.sync_party_from_storage();
    controller.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut controller.shell);
    assert_eq!(position(&mut controller), at);
    controller
}

fn direction(from: Point, to: Point) -> GameButton {
    neighbors(from)
        .into_iter()
        .find_map(|(point, button)| (point == to).then_some(button))
        .expect("adjacent native interaction target")
}

fn current_block(controller: &VisibleShellController, tree: Point) -> u16 {
    controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at((tree.0 % 64) / 2, (tree.1 % 48) / 2)
        .expect("current gate metatile")
}

fn quantity(controller: &mut VisibleShellController, item: &str) -> u16 {
    let snapshot = controller.snapshot().expect("native bag");
    snapshot
        .bag
        .items
        .iter()
        .chain(&snapshot.bag.balls)
        .filter(|slot| slot.item_id == item)
        .map(|slot| slot.quantity)
        .sum()
}

fn grid(
    runtime: &CrystalRuntime,
    controller: &mut VisibleShellController,
    case: &CutCase,
    include_current_map: bool,
) -> RouteGrid {
    let mut grid = RouteGrid::new(
        runtime,
        RouteSpec {
            prefix: case.prefix,
            columns: case.columns,
            rows: case.rows,
            tileset: if case.prefix == "MnR13"
                && runtime.data().maps["MnR13C00R00"].attributes.tileset_name
                    == "mn_winter_north_shore"
            {
                "mn_winter_north_shore"
            } else {
                case.tileset
            },
            start: position(controller),
            doors: BTreeMap::new(),
        },
    );
    if include_current_map {
        // Read the authoritative post-input map, including its native Cut
        // override. This only plans button presses; it never rewrites tiles.
        let snapshot = controller.snapshot().expect("current native map");
        let origin = (
            position(controller).0 - snapshot.overworld.tile.x,
            position(controller).1 - snapshot.overworld.tile.y,
        );
        let map = &controller.shell.shell.session().overworld().map;
        let module = runtime
            .data()
            .assemble_map_module_from_compiled_payloads(&snapshot.overworld.map_name)
            .expect("native current chunk");
        let collision = &runtime.data().tilesets[&module.attributes.tileset_name].collision;
        for y in 0..module.attributes.height as i16 * 2 {
            for x in 0..module.attributes.width as i16 * 2 {
                let block = map.metatile_at(x / 2, y / 2).expect("native metatile");
                if collision[&format!("{block:02x}")][(y % 2 * 2 + x % 2) as usize] == "FLOOR" {
                    grid.floor.insert((origin.0 + x, origin.1 + y));
                }
            }
        }
    }
    grid
}

fn cut(controller: &mut VisibleShellController, case: &CutCase) {
    let at = position(controller);
    controller
        .press(direction(at, case.tree))
        .expect("face blocked Cut entrance");
    assert_eq!(position(controller), at, "uncut tree must block movement");
    for _ in 0..64 {
        if current_block(controller, case.tree) == case.replacement {
            finish_text(controller);
            controller.wait_frames(60).expect("native Cut presentation");
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native Cut prompt, confirmation or use text");
    }
    panic!("{} did not apply its native Cut replacement", case.prefix);
}

fn verify_cut_gate(case: &CutCase) {
    let (root, runtime) = runtime();
    for (moves, hive_badge) in [(&["TACKLE"][..], true), (&["CUT"][..], false)] {
        let mut denied = fixture(
            &root,
            &runtime,
            case.prefix,
            case.approach,
            moves,
            hive_badge,
        );
        assert_eq!(current_block(&denied, case.tree), case.original);
        denied
            .press(direction(case.approach, case.tree))
            .expect("face tree without permission");
        denied.press(GameButton::A).expect("native Cut denial");
        let snapshot = denied.snapshot().expect("denied field move");
        assert!(
            snapshot
                .ui
                .text
                .as_ref()
                .and_then(|text| text.asm_text.as_ref())
                .is_some_and(|text| text.contains("CUT")),
            "{} must show native denial text",
            case.prefix
        );
        finish_text(&mut denied);
        assert_eq!(current_block(&denied, case.tree), case.original);
        assert_eq!(position(&mut denied), case.approach);
        assert!(
            !grid(&runtime, &mut denied, case, false)
                .floor
                .contains(&case.inside)
        );
    }
    let mut controller = fixture(&root, &runtime, case.prefix, case.approach, &["CUT"], true);
    assert!(
        !grid(&runtime, &mut controller, case, false)
            .floor
            .contains(&case.inside)
    );
    cut(&mut controller, case);
    assert_eq!(current_block(&controller, case.tree), case.replacement);
    let opened = grid(&runtime, &mut controller, case, true);
    opened.walk(&mut controller, case.inside);
    let before_quantity = quantity(&mut controller, case.reward_item);
    opened.talk(&mut controller, case.reward);
    assert_eq!(
        quantity(&mut controller, case.reward_item),
        before_quantity + 1
    );
    assert!(
        controller
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(case.reward_flag)
            .expect("native gated-pickup flag")
    );
    assert_eq!(current_block(&controller, case.tree), case.replacement);
    let saved_position = position(&mut controller);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("fixture save clock")
        .as_nanos();
    let save = super::authored_route_fixture::reached_save_path(&format!(
        "geothite-{}-cut-{stamp}.crystalsave",
        case.prefix
    ));
    controller
        .save(&save)
        .expect("save actual opened gate and pickup");
    let mut loaded = VisibleShellController::load_save(root, runtime.clone(), save.clone(), None)
        .expect("native Continue of gated resource");
    assert_eq!(position(&mut loaded), saved_position);
    assert_eq!(quantity(&mut loaded, case.reward_item), before_quantity + 1);
    assert!(
        loaded
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(case.reward_flag)
            .expect("persisted gated-pickup flag")
    );
    // Crystal's native TILES setup rebuilds temporary vegetation on Continue.
    // The resource event persists independently; no permanent gate flag exists.
    assert_eq!(current_block(&loaded, case.tree), case.original);
    let inside_approach = (
        case.tree.0 * 2 - case.approach.0,
        case.tree.1 * 2 - case.approach.1,
    );
    grid(&runtime, &mut loaded, case, true).walk(&mut loaded, inside_approach);
    cut(&mut loaded, case);
    grid(&runtime, &mut loaded, case, true).walk(&mut loaded, case.approach);
    assert_eq!(quantity(&mut loaded, case.reward_item), before_quantity + 1);
    super::authored_route_fixture::cleanup_reached_save(&save);
}

#[test]
#[ignore = "requires the published external R05/R06/R07/R08 pack"]
fn pine_country_cut_gate_denies_opens_collects_and_restores_native_reload_state() {
    verify_cut_gate(&PINE_COUNTRY);
}

#[test]
#[ignore = "requires the published external R05/R06/R07/R08 pack"]
fn leech_lake_cut_gate_denies_opens_collects_and_restores_native_reload_state() {
    verify_cut_gate(&LEECH_LAKE);
}

#[test]
#[ignore = "requires the published external R05/R06/R07/R08 pack"]
fn northern_lakes_cut_gate_denies_opens_collects_and_restores_native_reload_state() {
    verify_cut_gate(&NORTHERN_LAKES);
}

#[test]
#[ignore = "requires the revised external League Approach pack"]
fn league_approach_cut_gate_denies_opens_collects_and_restores_native_reload_state() {
    verify_cut_gate(&LEAGUE_APPROACH);
}

#[test]
#[ignore = "requires the published external Iron Range pack"]
fn iron_range_cut_gate_denies_opens_collects_and_restores_native_reload_state() {
    verify_cut_gate(&IRON_RANGE);
}

fn settle_water_activity(controller: &mut VisibleShellController) {
    for _ in 0..256 {
        let snapshot = controller.snapshot().expect("water input owner");
        if let Some(battle) = snapshot.battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            let button = if !controller.shell.battle_messages.is_empty() {
                GameButton::A
            } else if let Some(cursor) = &controller.shell.battle_action_cursor {
                match cursor.option_index {
                    0 | 1 => GameButton::Down,
                    2 => GameButton::Right,
                    3 => GameButton::A,
                    _ => panic!("invalid native battle action cursor"),
                }
            } else {
                GameButton::A
            };
            controller
                .press(button)
                .expect("resolve incidental water encounter through RUN");
        } else if snapshot.phase == RuntimeShellPhase::Text
            || controller.has_pending_script_work()
            || controller.shell.field_notice.is_some()
            || !controller.shell.battle_messages.is_empty()
            || snapshot
                .ui
                .text
                .as_ref()
                .is_some_and(|text| text.label == "visible-shell:battle-message")
        {
            // RUN can end the authoritative battle before its final visible
            // message releases input. Direction presses cannot dismiss that
            // owner, even though the snapshot phase is already Overworld.
            controller
                .press(GameButton::A)
                .expect("native Surf or encounter text");
            controller
                .wait_frames(16)
                .expect("source field presentation");
        } else {
            return;
        }
    }
    panic!("native water activity failed to release input");
}

fn step(controller: &mut VisibleShellController, button: GameButton, target: Point) {
    for _ in 0..8 {
        if position(controller) == target {
            return;
        }
        controller.press(button).expect("native Surf movement");
        settle_water_activity(controller);
    }
    let snapshot = controller.snapshot().expect("blocked native water step");
    assert_eq!(
        position(controller),
        target,
        "native water step {button:?}: mode={:?}, phase={:?}, text={:?}, remaining_battle_messages={}",
        snapshot.overworld.mode,
        snapshot.phase,
        snapshot.ui.text,
        controller.shell.battle_messages.len()
    );
}

fn mount_surf(controller: &mut VisibleShellController, facing: GameButton, target: Point) {
    assert_eq!(
        controller.snapshot().expect("shore state").overworld.mode,
        MovementMode::Normal
    );
    controller.press(facing).expect("face water from land");
    controller
        .press(GameButton::A)
        .expect("open native Surf prompt");
    for _ in 0..128 {
        if controller.snapshot().expect("mount state").overworld.mode == MovementMode::Surf {
            controller
                .wait_frames(60)
                .expect("native Surf mount presentation");
            settle_water_activity(controller);
            assert_eq!(position(controller), target);
            return;
        }
        controller
            .press(GameButton::A)
            .expect("native Surf confirmation and use text");
        controller
            .wait_frames(16)
            .expect("native source mount frames");
    }
    panic!("native Surf prompt failed to mount");
}

fn blocked_buoy(
    runtime: &CrystalRuntime,
    controller: &mut VisibleShellController,
    facing: GameButton,
) {
    let before = position(controller);
    let target = neighbors(before)
        .into_iter()
        .find_map(|(point, button)| (button == facing).then_some(point))
        .expect("outward water-rock direction");
    let block = controller
        .shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at((target.0 % 64) / 2, (target.1 % 48) / 2)
        .expect("native boundary block");
    assert_eq!(
        runtime.data().tilesets["mn_headwaters"].collision[&format!("{block:02x}")]
            [(target.1 % 2 * 2 + target.0 % 2) as usize],
        "BUOY",
        "the visible water-rock boundary must have native BUOY collision"
    );
    for _ in 0..4 {
        controller
            .press(facing)
            .expect("Surf against native water-rock BUOY");
        settle_water_activity(controller);
        assert_eq!(
            position(controller),
            before,
            "water-rock boundary must reject Surf"
        );
        assert_eq!(
            controller
                .snapshot()
                .expect("blocked Surf mode")
                .overworld
                .mode,
            MovementMode::Surf
        );
    }
}

#[test]
#[ignore = "requires the published external R05/R06/R07/R08 pack"]
fn pine_country_surf_blocks_at_water_rocks_and_crosses_reciprocal_water_seam() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, "MnR06", (18, 33), &["SURF"], true);
    mount_surf(&mut controller, GameButton::Down, (18, 34));
    for x in (1..18).rev() {
        step(&mut controller, GameButton::Left, (x, 34));
    }
    blocked_buoy(&runtime, &mut controller, GameButton::Left);
    for x in 2..=18 {
        step(&mut controller, GameButton::Right, (x, 34));
    }
    for y in 35..=48 {
        step(&mut controller, GameButton::Down, (18, y));
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("south water neighbor")
            .overworld
            .map_name,
        "MnR06C00R01"
    );
    step(&mut controller, GameButton::Up, (18, 47));
    assert_eq!(
        controller
            .snapshot()
            .expect("north water return")
            .overworld
            .map_name,
        "MnR06C00R00"
    );
    assert_eq!(
        controller
            .snapshot()
            .expect("seam Surf mode")
            .overworld
            .mode,
        MovementMode::Surf
    );
}

#[test]
#[ignore = "requires the published external R05/R06/R07/R08 pack"]
fn leech_lake_surf_blocks_at_water_rocks_and_crosses_reciprocal_water_seam() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, "MnR07", (59, 65), &["SURF"], true);
    mount_surf(&mut controller, GameButton::Right, (60, 65));
    step(&mut controller, GameButton::Right, (61, 65));
    step(&mut controller, GameButton::Right, (62, 65));
    blocked_buoy(&runtime, &mut controller, GameButton::Right);
    step(&mut controller, GameButton::Left, (61, 65));
    for y in (47..65).rev() {
        step(&mut controller, GameButton::Up, (61, y));
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("north water neighbor")
            .overworld
            .map_name,
        "MnR07C00R00"
    );
    step(&mut controller, GameButton::Down, (61, 48));
    assert_eq!(
        controller
            .snapshot()
            .expect("south water return")
            .overworld
            .map_name,
        "MnR07C00R01"
    );
    assert_eq!(
        controller
            .snapshot()
            .expect("seam Surf mode")
            .overworld
            .mode,
        MovementMode::Surf
    );
}

#[test]
#[ignore = "requires the published external R05/R06/R07/R08 pack"]
fn river_bluffs_surf_blocks_at_water_rocks_and_crosses_both_reciprocal_water_seams() {
    let (root, runtime) = runtime();
    let mut controller = fixture(&root, &runtime, "MnR05", (60, 88), &["SURF"], true);
    mount_surf(&mut controller, GameButton::Up, (60, 87));
    step(&mut controller, GameButton::Right, (61, 87));
    step(&mut controller, GameButton::Right, (62, 87));
    blocked_buoy(&runtime, &mut controller, GameButton::Right);
    for y in (82..87).rev() {
        step(&mut controller, GameButton::Up, (62, y));
    }
    // The current regional river has a native forest bank through row 81.
    // This terrain predates the grass pass and is preserved exactly.
    controller
        .press(GameButton::Up)
        .expect("existing cross-river bank");
    assert_eq!(position(&mut controller), (62, 82));
    for y in 83..=96 {
        step(&mut controller, GameButton::Down, (62, y));
    }
    assert_eq!(
        controller
            .snapshot()
            .expect("lower river seam")
            .overworld
            .map_name,
        "MnR05C00R02"
    );
    step(&mut controller, GameButton::Up, (62, 95));
    assert_eq!(
        controller
            .snapshot()
            .expect("lower seam reverse")
            .overworld
            .map_name,
        "MnR05C00R01"
    );
    // The lower external seam has both a working water lane and a real solid
    // bank. Both use the actual offset -18 / +18 reciprocal map projection.
    step(&mut controller, GameButton::Down, (62, 96));
    step(&mut controller, GameButton::Right, (63, 96));
    controller
        .press(GameButton::Right)
        .expect("existing external river lane");
    let across = controller.snapshot().expect("actual neighboring river");
    assert_eq!(
        across.overworld.map_name,
        "MnMississippiUpstreamWoodsC00R02"
    );
    assert_eq!((across.overworld.tile.x, across.overworld.tile.y), (0, 36));
    assert_eq!(across.overworld.mode, MovementMode::Surf);
    for _ in 0..8 {
        if controller
            .snapshot()
            .expect("river return map")
            .overworld
            .map_name
            == "MnR05C00R02"
        {
            break;
        }
        controller
            .wait_frames(16)
            .expect("native seam presentation");
        controller
            .press(GameButton::Left)
            .expect("reciprocal river return");
        settle_water_activity(&mut controller);
    }
    assert_eq!(position(&mut controller), (63, 96));
    step(&mut controller, GameButton::Down, (63, 97));
    step(&mut controller, GameButton::Down, (63, 98));
    for _ in 0..3 {
        controller
            .wait_frames(16)
            .expect("bank presentation frames");
        controller
            .press(GameButton::Right)
            .expect("neighboring tree bank blocks Surf");
        settle_water_activity(&mut controller);
    }
    assert_eq!(position(&mut controller), (63, 98));
    // Independent initial Surf placement in the actual northern water pocket.
    // The former upper chunk seam is now forest; never place Surf on that wall
    // or claim a continuous river through it. Exercise its real east port.
    let mut upper = fixture(&root, &runtime, "MnR05", (62, 1), &["SURF"], true);
    upper.shell.shell.session_mut().overworld_mut().player.mode = MovementMode::Surf;
    mark_runtime_snapshot_dirty(&mut upper.shell);
    blocked_buoy(&runtime, &mut upper, GameButton::Up);
    for y in 2..=27 {
        step(&mut upper, GameButton::Down, (62, y));
    }
    upper
        .press(GameButton::Down)
        .expect("northern pocket forest bank");
    assert_eq!(position(&mut upper), (62, 27));
    for y in (4..27).rev() {
        step(&mut upper, GameButton::Up, (62, y));
    }
    step(&mut upper, GameButton::Right, (63, 4));
    upper
        .press(GameButton::Right)
        .expect("northern live river port");
    let across = upper.snapshot().expect("northern connected river");
    assert_eq!(
        across.overworld.map_name,
        "MnMississippiUpstreamWoodsC00R00"
    );
    assert_eq!((across.overworld.tile.x, across.overworld.tile.y), (0, 40));
    assert_eq!(across.overworld.mode, MovementMode::Surf);
    for _ in 0..8 {
        if upper
            .snapshot()
            .expect("north return map")
            .overworld
            .map_name
            == "MnR05C00R00"
        {
            break;
        }
        upper
            .wait_frames(16)
            .expect("native north seam presentation");
        upper
            .press(GameButton::Left)
            .expect("reciprocal northern river return");
        settle_water_activity(&mut upper);
    }
    assert_eq!(position(&mut upper), (63, 4));
    assert_eq!(
        upper
            .snapshot()
            .expect("north Surf retained")
            .overworld
            .mode,
        MovementMode::Surf
    );

    assert_eq!(
        controller
            .snapshot()
            .expect("lower connected bank remains in the source map")
            .overworld
            .map_name,
        "MnR05C00R02"
    );
    assert_eq!(
        controller
            .snapshot()
            .expect("both river seams retain Surf mode")
            .overworld
            .mode,
        MovementMode::Surf
    );
}
