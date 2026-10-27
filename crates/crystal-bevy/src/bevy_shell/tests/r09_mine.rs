//! Empty-party local geometry fixture; no earned Q06 access or battle claim.
//! Original story behavior is covered by the existing iron_range_story cases.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    mark_runtime_snapshot_dirty,
};
use super::authored_route_fixture::{Point, RouteGrid, RouteSpec, indoor_walk, neighbors};
use super::r10_gallery::{
    completed_visible_page, continue_save, flags, map, press, remember_dialogue, settle, tile,
};

const ROOM: &str = "MnR09Cave1F";
const OUTSIDE: &str = "MnR09C00R00";
fn source() -> Value {
    let path = std::env::var_os("CRYSTAL_R09_MINE_SOURCE").map(PathBuf::from).unwrap_or_else(||
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.candidate.json"));
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen accepted R09 candidate"))
        .expect("mine source")
}
fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_R09_MINE_CONTROLLER_PACK")
                    .expect("actual published revised Q06 mine pack"),
            );
            let root = AssetRoot::new(path.parent().expect("pack parent"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("strict native mine pack"),
            )
            .expect("native runtime");
            (root, runtime)
        })
        .clone()
}
fn fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let shell = initialize_bevy_runtime_shell(
        root.clone(),
        r.clone(),
        BevyShellStart::NewGameAtRuntimeTile {
            spawn_identifier: r.title_new_game_spawn_identifier().expect("retained HOME"),
            map_name: OUTSIDE.into(),
            tile_x: 40,
            tile_y: 42,
        },
        BevyShellConfig {
            smoke_player_name: Some("CHRIS".into()),
            ..Default::default()
        },
    )
    .expect("explicit local passive exterior placement");
    let mut c = VisibleShellController { shell };
    complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
        .expect("local trainer name");
    let state = c.shell.shell.session_mut().state_mut();
    state.money = 1000;
    state.random_state = Default::default();
    *c.shell.shell.session_mut().divider_mut_for_tests() =
        crystal_core::random::RuntimeDividerSource::replay(
            (0_u32..262144)
                .map(|n| ((n * 73 + 19) % 256) as u8)
                .collect::<Vec<_>>(),
        );
    // Empty party, local position, money and RNG above are initial geometry setup.
    // No party/badge/report/story/position writes occur after this boundary.
    c.set_runtime_journal_enabled(false);
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert!(
        c.snapshot()
            .expect("honest empty geometry party")
            .party
            .slots
            .is_empty()
    );
    assert_eq!(map(&mut c), OUTSIDE);
    assert_eq!(tile(&mut c), (40, 42));
    let room = &r.data().maps[ROOM];
    let s = source();
    let blocks = s["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .flat_map(|row| row.as_str().expect("row").split_whitespace())
        .map(|v| u16::from_str_radix(v, 16).expect("native block"))
        .collect::<Vec<_>>();
    assert_eq!(
        room.blocks, blocks,
        "native pack must match exact frozen243 source"
    );
    assert_eq!([room.attributes.width, room.attributes.height], [12, 11]);
    assert_eq!(room.attributes.time_of_day.as_deref(), Some("nite"));
    assert_eq!(room.attributes.palette.as_deref(), Some("PALETTE_NITE"));
    (root, r, c)
}
fn kind(r: &CrystalRuntime, p: Point) -> &str {
    let m = &r.data().maps[ROOM];
    let b = m.blocks[p.1 as usize / 2 * 12 + p.0 as usize / 2];
    &r.data().tilesets["cave"].collision[&format!("{b:02x}")]
        [p.1 as usize % 2 * 2 + p.0 as usize % 2]
}
fn direction(a: Point, b: Point) -> GameButton {
    neighbors(a)
        .into_iter()
        .find_map(|(p, input)| (p == b).then_some(input))
        .expect("adjacent authored native step")
}
fn step(c: &mut VisibleShellController, input: GameButton, to: Point) {
    for _ in 0..8 {
        if tile(c) == to {
            return;
        }
        press(c, input);
    }
    assert_eq!(tile(c), to);
}
fn blocked(c: &mut VisibleShellController, input: GameButton) {
    let at = tile(c);
    for _ in 0..4 {
        press(c, input);
        assert_eq!(tile(c), at);
    }
}
fn exterior(r: &CrystalRuntime) -> RouteGrid {
    RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR09",
            columns: 2,
            rows: 2,
            tileset: "mn_iron_range",
            start: (40, 42),
            doors: BTreeMap::from([((40, 41), ROOM)]),
        },
    )
}
fn enter(c: &mut VisibleShellController, r: &CrystalRuntime) {
    exterior(r).walk(c, (40, 41));
    assert_eq!(map(c), ROOM);
    assert_eq!(tile(c), (13, 19));
    indoor_walk(c, r, (13, 18));
    assert_eq!(tile(c), (13, 18));
}
fn leave(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (13, 18));
    indoor_walk(c, r, (13, 19));
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (40, 42));
    c.wait_frames(60)
        .expect("genuine original passive Floor remains stable");
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (40, 42));
    let g = exterior(r);
    g.walk(c, (40, 43));
    g.walk(c, (40, 42));
    assert_eq!(map(c), OUTSIDE);
    assert_eq!(tile(c), (40, 42));
}
fn ordinary_path(r: &CrystalRuntime, from: Point, to: Point) -> Vec<Point> {
    let bodies = r.data().maps[ROOM]
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::from([(from, from)]);
    let mut todo = VecDeque::from([from]);
    while let Some(at) = todo.pop_front() {
        if at == to {
            break;
        }
        for (next, _) in neighbors(at) {
            if next.0 < 0
                || next.1 < 0
                || next.0 >= 24
                || next.1 >= 22
                || kind(r, next) != "FLOOR"
                || bodies.contains(&next)
                || parents.contains_key(&next)
            {
                continue;
            }
            parents.insert(next, at);
            todo.push_back(next);
        }
    }
    assert!(
        parents.contains_key(&to),
        "ordinary mine return{from:?}->{to:?}"
    );
    let mut path = Vec::new();
    let mut at = to;
    while at != from {
        path.push(at);
        at = parents[&at];
    }
    path.reverse();
    path
}
fn scout(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (19, 8));
    press(c, GameButton::Up);
    assert_eq!(tile(c), (19, 8));
    let before = c.snapshot().expect("before original scout");
    let progress = flags(c);
    press(c, GameButton::A);
    let at = tile(c);
    press(c, GameButton::Down);
    assert_eq!(tile(c), at, "fully native scout text owns movement");
    let mut pages = BTreeMap::new();
    let mut diagnostics = VecDeque::new();
    for _ in 0..32 {
        c.wait_frames(240).expect("fully print original scout page");
        remember_dialogue(c, &mut diagnostics);
        if let Some((index, page)) = completed_visible_page(c) {
            assert_eq!(
                c.snapshot()
                    .expect("original text owner")
                    .ui
                    .text
                    .as_ref()
                    .map(|t| t.label.as_str()),
                Some("MN_R09_MINE_SCOUTText"),
                "{diagnostics:#?}"
            );
            pages.insert(index, page);
        }
        if pages.len() == 2 {
            break;
        }
        press(c, GameButton::A);
    }
    assert_eq!(
        pages.get(&0).map(String::as_str),
        Some("The foreman took\nthe instrument."),
        "{diagnostics:#?}"
    );
    assert_eq!(
        pages.get(&1).map(String::as_str),
        Some("The upper passage\nreaches his cache."),
        "{diagnostics:#?}"
    );
    settle(c, "after both actually printed original scout pages");
    let after = c.snapshot().expect("scout complete");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    assert_eq!(flags(c), progress);
    assert_eq!(c.shell.shell.session().state().money, 1000);
}

#[test]
#[ignore = "requires actual revised R09 native pack and frozen243 source; local empty-party geometry only"]
fn r09_mine_controller_original_mouth_three_stairs_two_short_hops_scout_and_continue() {
    let (root, r, mut c) = fixture();
    let initial_flags = flags(&c);
    enter(&mut c, &r);
    // Cave30's left lane is ordinary Floor; the scout stair enters at a corner,
    // because16,9 is an authentic Wall rather than a fictitious north landing.
    for (path, wall_from, wall_target) in [
        ([(8, 5), (8, 6), (8, 7), (8, 8)], (8, 7), (9, 7)),
        ([(17, 10), (16, 10), (16, 11), (16, 12)], (16, 11), (17, 11)),
        ([(10, 13), (10, 14), (10, 15), (10, 16)], (10, 15), (11, 15)),
    ] {
        for p in path {
            assert_eq!(kind(&r, p), "FLOOR");
        }
        indoor_walk(&mut c, &r, path[0]);
        for p in path.into_iter().skip(1) {
            let at = tile(&mut c);
            step(&mut c, direction(at, p), p);
        }
        assert_eq!(kind(&r, wall_target), "WALL");
        indoor_walk(&mut c, &r, wall_from);
        blocked(&mut c, direction(wall_from, wall_target));
        indoor_walk(&mut c, &r, path[3]);
        for p in path.into_iter().rev().skip(1) {
            let at = tile(&mut c);
            step(&mut c, direction(at, p), p);
        }
    }
    for x in [10, 11] {
        let upper = (x, 7);
        let hop = (x, 8);
        let reverse = (x, 9);
        let landing = (x, 10);
        assert_eq!(kind(&r, hop), "HOP_DOWN");
        assert_eq!(kind(&r, reverse), "WALL");
        indoor_walk(&mut c, &r, upper);
        for _ in 0..12 {
            if tile(&mut c) == landing {
                break;
            }
            press(&mut c, GameButton::Down);
            let at = tile(&mut c);
            assert!(
                at == upper || at == hop || at == landing,
                "native hop crossed an unexpected tile{at:?}"
            );
            assert_ne!(at, reverse, "jump must cross the native blocking quadrant");
        }
        assert_eq!(tile(&mut c), landing);
        blocked(&mut c, GameButton::Up);
        let path = ordinary_path(&r, landing, upper);
        assert_eq!(path.len(), if x == 10 { 9 } else { 11 });
        for p in path {
            let at = tile(&mut c);
            step(&mut c, direction(at, p), p);
            assert_eq!(map(&mut c), ROOM);
        }
        assert_eq!(tile(&mut c), upper);
    }
    for p in [(9, 8), (12, 8), (9, 9), (12, 9)] {
        assert_eq!(kind(&r, p), "WALL");
    }
    // Exercise real adjacent approaches while keeping Q06 flags untouched.
    for p in [(7, 4), (9, 6), (19, 8)] {
        indoor_walk(&mut c, &r, p);
        assert_eq!(tile(&mut c), p);
    }
    scout(&mut c, &r);
    assert_eq!(flags(&c), initial_flags);
    c = continue_save(root, r.clone(), &mut c);
    assert_eq!(flags(&c), initial_flags);
    leave(&mut c, &r);
    enter(&mut c, &r);
    assert_eq!(flags(&c), initial_flags);
}
