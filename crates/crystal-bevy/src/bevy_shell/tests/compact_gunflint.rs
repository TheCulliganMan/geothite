//! Compact Gunflint: declared local setup followed only by production inputs.
//! These fixtures do not establish earned badges or regional story progression.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, LearnedMove};
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{
    Point, RouteGrid, RouteSpec, finish_text, indoor_walk, neighbors, position,
};

const SOURCE: &str = include_str!(
    "../../../../../planning/minnesota/revisions/gunflint-quality-v01/source-v08/MN_R12.candidate.json"
);
const UPPER: &str = "MnR12C00R00";
const LOWER: &str = "MnR12C00R01";
const CEDAR: &str = "MnMaraisCedarApproach";
const BW: &str = "MnBoundaryWaters";
const CAMP: &str = "EVENT_MN_GUNFLINT_CAMP_SUPPLY";

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_COMPACT_SUPERIOR_TEST_PACK")
                    .expect("completed actual compact Superior pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack directory"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                crystal_assets::read_loaded_verified_compiled_game_pack(path)
                    .expect("verified compact pack"),
            )
            .expect("native compact runtime");
            assert_eq!(
                [
                    runtime.data().maps[UPPER].attributes.width,
                    runtime.data().maps[UPPER].attributes.height
                ],
                [32, 24]
            );
            assert_eq!(
                [
                    runtime.data().maps[LOWER].attributes.width,
                    runtime.data().maps[LOWER].attributes.height
                ],
                [32, 8]
            );
            assert_eq!(
                runtime
                    .data()
                    .maps
                    .keys()
                    .filter(|n| n.starts_with("MnR12C") && !n.contains("Cave"))
                    .count(),
                2
            );
            (root, runtime)
        })
        .clone()
}

#[derive(Clone, Copy)]
enum Setup {
    Empty,
    Cut,
    NoMove,
    NoBadge,
}

fn fixture(at: Point, setup: Setup) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, runtime) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        runtime.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
            map_name: if at.1 < 48 { UPPER } else { LOWER }.into(),
            tile_x: at.0,
            tile_y: at.1 % 48,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit compact route placement");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("fixture name");
    if !matches!(setup, Setup::Empty) {
        let name = if matches!(setup, Setup::NoMove) {
            "SWIFT"
        } else {
            "CUT"
        };
        let mut pokemon = runtime
            .data()
            .create_pokemon("MEW", 60, Dv::default())
            .expect("legal field partner");
        assert!(
            runtime.data().pokemon["MEW"]
                .tmhm_learnset
                .iter()
                .any(|m| m == name)
                || name == "SWIFT"
        );
        pokemon.moves = vec![LearnedMove {
            name: name.into(),
            current_pp: 1,
            pp_ups: 0,
        }];
        pokemon.hp = pokemon.max_hp.saturating_sub(20);
        assert!(
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .storage
                .party
                .add_pokemon(pokemon)
        );
    }
    let state = c.shell.shell.session_mut().state_mut();
    for (i, name) in [
        "ZEPHYR", "HIVE", "PLAIN", "FOG", "STORM", "MINERAL", "GLACIER", "RISING",
    ]
    .iter()
    .enumerate()
    {
        let enabled = i != 1 || !matches!(setup, Setup::NoBadge);
        state.badges.johto[i] = enabled;
        state
            .flags
            .set_engine_flag(&format!("ENGINE_{name}BADGE"), enabled)
            .expect("consistent initial badge");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", true)
        .expect("explicit initial regional signal");
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["MAX_REPEL"], 5)
            .expect("initial legal Repel stock")
    );
    state.sync_party_from_storage();
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(position(&mut c), at);
    // Setup ends here. No subsequent party, badge, flag, RNG, terrain or position writes.
    (root, runtime, c)
}

fn grid(runtime: &CrystalRuntime, at: Point) -> RouteGrid {
    RouteGrid::new_with_grass(
        runtime,
        RouteSpec {
            prefix: "MnR12",
            columns: 1,
            rows: 2,
            tileset: "mn_winter_gunflint",
            start: at,
            doors: BTreeMap::from([((40, 15), "MnR12Cave1F"), ((49, 47), "MnR12RangerHut")]),
        },
    )
}
fn source() -> Value {
    serde_json::from_str(SOURCE).expect("frozen compact dossier")
}
fn point(v: &Value) -> Point {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn local(c: &mut VisibleShellController) -> (String, Point) {
    let s = c.snapshot().expect("native local location");
    (
        s.overworld.map_name,
        (s.overworld.tile.x, s.overworld.tile.y),
    )
}
fn step(c: &mut VisibleShellController, input: GameButton, map: &str, target: Point) {
    for _ in 0..8 {
        if local(c) == (map.into(), target) {
            return;
        }
        c.press(input).expect("native compact seam input");
        c.wait_frames(1).expect("native seam frame");
    }
    assert_eq!(local(c), (map.into(), target));
}
fn retain(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save clock")
        .as_nanos();
    let path = super::authored_route_fixture::reached_save_path(&format!(
        "compact-gunflint-{label}-{}-{stamp}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("actual reached native save");
    path
}
fn quantity(c: &VisibleShellController, runtime: &CrystalRuntime, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&runtime.data().items[item])
}
fn tree(c: &VisibleShellController) -> u16 {
    c.shell
        .shell
        .session()
        .overworld()
        .map
        .metatile_at(7, 8)
        .expect("current native Cut tree")
}
fn cut(c: &mut VisibleShellController, direction: GameButton) {
    c.press(direction).expect("face actual Cut gate");
    for _ in 0..64 {
        if tree(c) == 0x17 {
            finish_text(c);
            c.wait_frames(60).expect("native Cut presentation");
            return;
        }
        c.press(GameButton::A)
            .expect("native Cut prompt/confirmation");
        c.wait_frames(16).expect("native Cut frame");
    }
    panic!("native compact Cut did not apply0f->17");
}
fn opened_grid(runtime: &CrystalRuntime, c: &mut VisibleShellController) -> RouteGrid {
    let mut result = grid(runtime, position(c));
    let (map, _) = local(c);
    assert_eq!(map, UPPER);
    let collision = &runtime.data().tilesets["mn_winter_gunflint"].collision;
    for y in 0..48 {
        for x in 0..64 {
            let block = c
                .shell
                .shell
                .session()
                .overworld()
                .map
                .metatile_at(x / 2, y / 2)
                .expect("authoritative current block");
            if matches!(
                collision[&format!("{block:02x}")][(y % 2 * 2 + x % 2) as usize].as_str(),
                "FLOOR" | "TALL_GRASS"
            ) {
                result.floor.insert((x, y));
            }
        }
    }
    result
}

#[test]
#[ignore = "requires published compact pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_gunflint_controller_both_real_ports_all_ordinary_actor_approaches_and_dry_bog_discovery()
{
    let (_, runtime, mut c) = fixture((62, 54), Setup::Empty);
    for y in [6, 7] {
        step(&mut c, GameButton::Right, LOWER, (63, y));
        step(&mut c, GameButton::Right, CEDAR, (0, y + 8));
        step(&mut c, GameButton::Left, LOWER, (63, y));
        if y == 6 {
            step(&mut c, GameButton::Down, LOWER, (63, 7));
        }
    }
    let route = grid(&runtime, position(&mut c));
    for y in [28, 29] {
        route.walk(&mut c, (0, y));
        step(&mut c, GameButton::Left, BW, (63, y - 4));
        step(&mut c, GameButton::Right, UPPER, (0, y));
    }
    let d = source();
    for field in ["residents", "trainers", "items", "signs"] {
        for actor in d[field].as_array().expect("exact source actors") {
            if actor["id"] == "MN_R12_ITEM_01" {
                continue;
            }
            let at = point(&actor["tile"]);
            let approach = neighbors(at)
                .into_iter()
                .map(|(p, _)| p)
                .find(|p| {
                    route.floor.contains(p)
                        && !route.blocked.contains(p)
                        && !route.hazards.contains(p)
                        && (p.1 / 48 == at.1 / 48)
                })
                .unwrap_or_else(|| panic!("ordinary actor approach {} {at:?}", actor["id"]));
            route.walk(&mut c, approach);
            if field == "signs" {
                assert!(!route.talk(&mut c, at).is_empty());
            }
        }
    }
    assert!(
        route.talk(&mut c, (23, 45)).contains("STONE BOG"),
        "previously Surf-only sign now reached by ordinary Foot input"
    );
    let lower = &runtime.data().maps[LOWER];
    let b = lower.blocks[7 * 32 + 27];
    assert!(
        runtime.data().tilesets["mn_winter_gunflint"].collision[&format!("{b:02x}")]
            .iter()
            .all(|k| k == "WALL"),
        "superseded south exit visibly closed by native pines"
    );
    retain(&mut c, "all-foot-approaches");
}

#[test]
#[ignore = "requires published compact pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_gunflint_controller_original_rooms_pp_resource_and_hut_passive_return_crosses_chunk() {
    let (_, runtime, mut c) = fixture((62, 54), Setup::Cut);
    super::minneapolis_falls::grand_marais_harbor::use_max_repel(&mut c, &runtime);
    let route = grid(&runtime, position(&mut c));
    route.walk(&mut c, (40, 15));
    assert_eq!(local(&mut c).0, "MnR12Cave1F");
    indoor_walk(&mut c, &runtime, (6, 5));
    c.press(GameButton::Left).expect("face original Otta");
    c.press(GameButton::A).expect("original shelter dialogue");
    c.wait_frames(240).expect("print original Otta page");
    assert!(
        c.snapshot()
            .expect("Otta page")
            .ui
            .text
            .and_then(|t| t.asm_text)
            .is_some_and(|t| t.contains("dry rock roof"))
    );
    let held = local(&mut c);
    c.press(GameButton::Down).expect("dialogue owns movement");
    assert_eq!(local(&mut c), held);
    finish_text(&mut c);
    indoor_walk(&mut c, &runtime, (26, 5));
    c.press(GameButton::Right).expect("face original Max Ether");
    c.press(GameButton::A)
        .expect("collect original cave resource");
    super::r10_gallery::settle(&mut c, "native compact cave pickup");
    assert_eq!(quantity(&c, &runtime, "MAX_ETHER"), 1);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R12_CAVE_ETHER")
            .expect("original cave pickup")
    );
    super::r12_shelter::use_pp_item(&mut c, &runtime, "MAX_ETHER", Some(0));
    assert_eq!(quantity(&c, &runtime, "MAX_ETHER"), 0);
    assert_eq!(
        c.shell.shell.session().state().storage.party.pokemon[0]
            .as_ref()
            .expect("partner")
            .moves[0]
            .current_pp,
        runtime.data().moves["CUT"].pp
    );
    indoor_walk(&mut c, &runtime, (17, 22));
    indoor_walk(&mut c, &runtime, (17, 23));
    assert_eq!(local(&mut c), (UPPER.into(), (40, 15)));
    c.wait_frames(60).expect("native cave clear step");
    assert_eq!(local(&mut c), (UPPER.into(), (40, 16)));
    for carpet_x in [4, 5] {
        route.walk(&mut c, (49, 47));
        assert_eq!(local(&mut c).0, "MnR12RangerHut");
        indoor_walk(&mut c, &runtime, (7, 4));
        let before = c.snapshot().expect("before June").party;
        c.press(GameButton::Up).expect("face original June");
        c.press(GameButton::A)
            .expect("original Q07 direction dialogue");
        finish_text(&mut c);
        assert_eq!(
            c.snapshot().expect("June navigation preserves party").party,
            before
        );
        indoor_walk(&mut c, &runtime, (carpet_x, 8));
        for _ in 0..8 {
            c.press(GameButton::Down).expect("native hut carpet");
            if local(&mut c).0 != "MnR12RangerHut" {
                break;
            }
        }
        assert_eq!(
            local(&mut c),
            (UPPER.into(), (49, 47)),
            "both carpets return through upper exterior warp2"
        );
        c.wait_frames(60)
            .expect("native passive hut return across actual chunk seam");
        assert_eq!(
            local(&mut c),
            (LOWER.into(), (49, 0)),
            "clear return is lower chunk, not invalid upper y48"
        );
    }
    retain(&mut c, "rooms-pp-and-cross-chunk-return");
}

#[test]
#[ignore = "requires published compact pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_gunflint_controller_cut_denial_native_open_reward_continue_and_map_reentry_reset() {
    for setup in [Setup::NoMove, Setup::NoBadge] {
        let (_, runtime, mut c) = fixture((14, 18), setup);
        c.press(GameButton::Up).expect("face sealed tree");
        c.press(GameButton::A).expect("native denial");
        finish_text(&mut c);
        assert_eq!(tree(&c), 0x0f);
        assert_eq!(position(&mut c), (14, 18));
        assert!(
            !grid(&runtime, (14, 18)).floor.contains(&(14, 11)),
            "reward clearing truly sealed"
        );
    }
    let (root, runtime, mut c) = fixture((14, 18), Setup::Cut);
    super::minneapolis_falls::grand_marais_harbor::use_max_repel(&mut c, &runtime);
    cut(&mut c, GameButton::Up);
    let opened = opened_grid(&runtime, &mut c);
    opened.walk(&mut c, (14, 11));
    let before = quantity(&c, &runtime, "HYPER_POTION");
    opened.talk(&mut c, (13, 11));
    assert_eq!(quantity(&c, &runtime, "HYPER_POTION"), before + 1);
    let saved = retain(&mut c, "cut-cache-collected");
    c = VisibleShellController::load_save(root, runtime.clone(), saved, None)
        .expect("actual Continue");
    assert_eq!(tree(&c), 0x0f, "native temporary tree resets on Continue");
    assert_eq!(quantity(&c, &runtime, "HYPER_POTION"), before + 1);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R12_ITEM_01")
            .expect("reward persists")
    );
    grid(&runtime, position(&mut c)).walk(&mut c, (14, 16));
    cut(&mut c, GameButton::Down);
    opened_grid(&runtime, &mut c).walk(&mut c, (14, 18));
    let route = grid(&runtime, position(&mut c));
    route.walk(&mut c, (63, 54));
    step(&mut c, GameButton::Right, CEDAR, (0, 14));
    step(&mut c, GameButton::Left, LOWER, (63, 6));
    grid(&runtime, position(&mut c)).walk(&mut c, (14, 18));
    assert_eq!(
        tree(&c),
        0x0f,
        "actual map departure/reentry restores native vegetation"
    );
    assert_eq!(
        quantity(&c, &runtime, "HYPER_POTION"),
        before + 1,
        "no duplicate reward across Continue and map return"
    );
    retain(&mut c, "cut-return-preserved-reward");
}

#[test]
#[ignore = "requires published compact pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_gunflint_controller_camp_three_berries_once_continue_and_real_reentry() {
    let (root, runtime, mut c) = fixture((50, 53), Setup::Cut);
    super::minneapolis_falls::grand_marais_harbor::use_max_repel(&mut c, &runtime);
    let before = quantity(&c, &runtime, "BERRY");
    let route = grid(&runtime, position(&mut c));
    route.talk(&mut c, (51, 53));
    assert_eq!(quantity(&c, &runtime, "BERRY"), before + 3);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(CAMP)
            .expect("same native2387 pickup flag")
    );
    let saved = retain(&mut c, "camp-collected");
    c = VisibleShellController::load_save(root, runtime.clone(), saved, None)
        .expect("actual camp Continue");
    route.walk(&mut c, (63, 54));
    step(&mut c, GameButton::Right, CEDAR, (0, 14));
    step(&mut c, GameButton::Left, LOWER, (63, 6));
    route.walk(&mut c, (50, 53));
    c.press(GameButton::Right).expect("face collected stock");
    c.press(GameButton::A).expect("repeat pickup attempt");
    finish_text(&mut c);
    assert_eq!(quantity(&c, &runtime, "BERRY"), before + 3);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set(CAMP)
            .expect("persistent camp flag")
    );
    retain(&mut c, "camp-return-no-duplicate");
}

#[test]
#[ignore = "requires published compact pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_gunflint_controller_meaningful_frost_courses_force_dry_and_wall_stops_with_returns() {
    for (from, to, button, back) in [
        ((43, 16), (58, 16), GameButton::Right, GameButton::Left),
        ((53, 20), (53, 11), GameButton::Up, GameButton::Down),
        ((58, 13), (48, 13), GameButton::Left, GameButton::Right),
    ] {
        let (_, _, mut c) = fixture(from, Setup::Empty);
        for (start, end, input) in [(from, to, button), (to, from, back)] {
            assert_eq!(position(&mut c), start);
            for _ in 0..4 {
                c.press(input).expect("native frost entry");
                c.wait_frames(1).expect("native slide frame");
                if position(&mut c) != start {
                    break;
                }
            }
            assert_ne!(position(&mut c), start);
            c.wait_frames(400).expect("native forced movement settles");
            assert_eq!(position(&mut c), end, "exact authored native frost stop");
            c.wait_frames(80).expect("stable resting frost stop");
            assert_eq!(position(&mut c), end);
        }
        retain(&mut c, "frost-return");
    }
}

#[test]
#[ignore = "requires published compact pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_gunflint_controller_required_west_grass_starts_authoritative_wild_battle() {
    let (_, _, mut c) = fixture((6, 28), Setup::Cut);
    // Repel stock remains unused: the actual native habitat must encounter.
    for _ in 0..512 {
        if c.snapshot()
            .expect("actual encounter state")
            .battle
            .is_some()
        {
            break;
        }
        // Choose from authoritative position, so a facing-only input repeats
        // instead of alternating forever without taking an encounter step.
        let input = if position(&mut c).0 <= 4 {
            GameButton::Right
        } else {
            GameButton::Left
        };
        c.press(input).expect("walk native required grass");
        c.wait_frames(16).expect("native encounter engine");
    }
    assert!(matches!(
        c.snapshot()
            .expect("real native grass battle")
            .battle
            .expect("wild battle from actual walking")
            .kind,
        RuntimeBattleKind::Wild { .. }
    ));
    retain(&mut c, "west-grass-battle");
}
