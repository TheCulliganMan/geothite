//! Explicit local Floor/partner/badge fixtures, then production input only.
//! These local seam checks do not establish earned campaign progression.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, assert_at, at, fixture_with_items_on_runtime,
    mark_runtime_snapshot_dirty, mount, settle, step,
};

const COAST: &str = "MnR11C01R00";
const TOWN: &str = "MnGrandMarais";
const BADGES: [&str; 8] = [
    "ENGINE_ZEPHYRBADGE",
    "ENGINE_HIVEBADGE",
    "ENGINE_PLAINBADGE",
    "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE",
    "ENGINE_MINERALBADGE",
    "ENGINE_GLACIERBADGE",
    "ENGINE_RISINGBADGE",
];

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_COMPACT_SUPERIOR_TEST_PACK")
                    .expect("explicit completed compact coast pack"),
            )
            .canonicalize()
            .expect("native pack");
            let root = AssetRoot::new(path.parent().expect("pack directory"));
            let r = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified compact pack"),
            )
            .expect("native compact runtime");
            assert_eq!(
                (
                    r.data().maps[COAST].attributes.width,
                    r.data().maps[COAST].attributes.height
                ),
                (4, 24)
            );
            assert!(
                r.data().maps[TOWN]
                    .attributes
                    .connections
                    .iter()
                    .any(|e| e.direction == "west" && e.target_map == COAST && e.offset == 0)
            );
            (root, r)
        })
        .clone()
}

fn fixture(
    map: &str,
    p: Point,
    badges: bool,
    signal: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) =
        fixture_with_items_on_runtime(load(), map, p, false, false, false, &[("MAX_REPEL", 5)]);
    let state = c.shell.shell.session_mut().state_mut();
    for (i, flag) in BADGES.iter().enumerate() {
        let enabled = badges || i == 3;
        state.badges.johto[i] = enabled;
        state
            .flags
            .set_engine_flag(flag, enabled)
            .expect("initial badge fixture");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", signal)
        .expect("initial Duluth signal");
    state
        .flags
        .set_event_flag("EVENT_MN_GRAND_MARAIS_SIGNAL", true)
        .expect("local lamp cannot replace Duluth signal");
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(kind(&r, map, p), "FLOOR", "legal initial Floor fixture");
    // No later position, mode, party, badge, flag or RNG writes.
    (root, r, c)
}

fn kind(r: &CrystalRuntime, map: &str, p: Point) -> String {
    let m = &r.data().maps[map];
    let x = usize::try_from(p.0).expect("x");
    let y = usize::try_from(p.1).expect("y");
    let b = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")][y % 2 * 2 + x % 2]
        .clone()
}

fn path(r: &CrystalRuntime, c: &mut VisibleShellController, target: Point, water: bool) -> usize {
    let (map, start) = at(c);
    let m = &r.data().maps[&map];
    let occupied = m
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut queue = VecDeque::from([start]);
    let mut previous = BTreeMap::from([(start, None)]);
    while let Some(p) = queue.pop_front() {
        if p == target {
            break;
        }
        for n in [
            (p.0 - 1, p.1),
            (p.0 + 1, p.1),
            (p.0, p.1 - 1),
            (p.0, p.1 + 1),
        ] {
            if n.0 < 0
                || n.1 < 0
                || n.0 >= m.attributes.width as i16 * 2
                || n.1 >= m.attributes.height as i16 * 2
                || previous.contains_key(&n)
                || occupied.contains(&n)
                || !(if water {
                    kind(r, &map, n) == "WATER"
                } else {
                    matches!(kind(r, &map, n).as_str(), "FLOOR" | "TALL_GRASS")
                })
                || m.events.warps.iter().any(|w| (w.x as i16, w.y as i16) == n)
                || m.events
                    .coord_events
                    .iter()
                    .any(|e| (e.x as i16, e.y as i16) == n)
            {
                continue;
            }
            previous.insert(n, Some(p));
            queue.push_back(n);
        }
    }
    assert!(
        previous.contains_key(&target),
        "ordinary native {map} path {start:?}->{target:?}"
    );
    let mut course = vec![];
    let mut end = target;
    while let Some(p) = previous[&end] {
        course.push((p, end));
        end = p;
    }
    let length = course.len();
    for (a, b) in course.into_iter().rev() {
        let key = match (b.0 - a.0, b.1 - a.1) {
            (1, 0) => GameButton::Right,
            (-1, 0) => GameButton::Left,
            (0, 1) => GameButton::Down,
            (0, -1) => GameButton::Up,
            _ => unreachable!("adjacent path"),
        };
        step(c, key, &map, b, "actual compact native walking/Surf path");
    }
    length
}

fn save(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let dir = std::env::var_os("CRYSTAL_COMPACT_SUPERIOR_SAVE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("geothite-compact-superior-saves"));
    std::fs::create_dir_all(&dir).expect("retained saves");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = dir.join(format!(
        "compact-{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists());
    c.save(&path).expect("reached production Save");
    path
}

#[test]
#[ignore = "requires completed compact native pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_superior_coast_controller_both_foot_lanes_cross_both_seams_and_return() {
    for lane in 0..2 {
        let (_, r, mut c) = fixture("MnR11C00R00", (63, 12 + lane), true, true);
        step(
            &mut c,
            GameButton::Right,
            COAST,
            (0, 12 + lane),
            "preserved original coast to cropped bend",
        );
        path(&r, &mut c, (7, 8 + lane), false);
        step(
            &mut c,
            GameButton::Right,
            TOWN,
            (0, 8 + lane),
            "compact coast directly reaches town west",
        );
        step(
            &mut c,
            GameButton::Right,
            TOWN,
            (3, 8 + lane),
            "free inward native officer wrapper",
        );
        path(&r, &mut c, (4, 8 + lane), false);
        for x in (0..4).rev() {
            step(
                &mut c,
                GameButton::Left,
                TOWN,
                (x, 8 + lane),
                "qualified town west departure",
            );
        }
        step(
            &mut c,
            GameButton::Left,
            COAST,
            (7, 8 + lane),
            "town west returns to compact coast",
        );
        path(&r, &mut c, (0, 12 + lane), false);
        step(
            &mut c,
            GameButton::Left,
            "MnR11C00R00",
            (63, 12 + lane),
            "compact coast returns to original R11",
        );
        save(&mut c, "both-foot-seams-return");
    }
}

#[test]
#[ignore = "requires completed compact native pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_superior_coast_controller_shared_superior_surfs_both_directions_and_continue() {
    let (root, r, mut c) = fixture(COAST, (0, 13), true, true);
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Down, COAST, (0, 14));
    for y in [16, 20, 35] {
        path(&r, &mut c, (7, y), true);
        step(
            &mut c,
            GameButton::Right,
            TOWN,
            (0, y),
            "same Superior incoming water seam",
        );
        step(
            &mut c,
            GameButton::Right,
            TOWN,
            (3, y),
            "native inward Surf officer wrapper",
        );
        for x in (0..3).rev() {
            step(
                &mut c,
                GameButton::Left,
                TOWN,
                (x, y),
                "qualified outward water lane",
            );
        }
        step(
            &mut c,
            GameButton::Left,
            COAST,
            (7, y),
            "same Superior outgoing water seam",
        );
    }
    assert_eq!(
        c.snapshot().expect("native Surf mode").overworld.mode,
        MovementMode::Surf
    );
    let before = c.snapshot().expect("reached compact Surf");
    let reached = save(&mut c, "superior-surf-seam");
    c = VisibleShellController::load_save(root, r.clone(), reached, None)
        .expect("actual compact Surf Continue");
    let after = c.snapshot().expect("continued Surf");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.overworld.mode, after.overworld.mode);
    assert_eq!(before.party, after.party);
    path(&r, &mut c, (5, 36), true);
    for _ in 0..2 {
        c.press(GameButton::Right)
            .expect("unmatched compact east water edge attempt");
        c.wait_frames(16).expect("native rock collision");
        settle(&mut c);
        assert_at(
            &mut c,
            COAST,
            (5, 36),
            "visible native reef closes unmatched east rows",
        );
    }
    path(&r, &mut c, (0, 14), true);
    step(
        &mut c,
        GameButton::Up,
        COAST,
        (0, 13),
        "actual dry bank dismount",
    );
    assert_eq!(
        c.snapshot().expect("dry return").overworld.mode,
        MovementMode::Normal
    );
    save(&mut c, "superior-dry-bank-return");
}

#[test]
#[ignore = "requires completed compact native pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_superior_coast_controller_north_gate_requires_badges_and_duluth_signal_both_lanes() {
    for x in [18, 19] {
        for (badges, signal) in [(false, true), (true, false)] {
            let (_, _, mut c) = fixture(TOWN, (x, 4), badges, signal);
            let before = c.snapshot().expect("closed north gate");
            let mut refused = false;
            for _ in 0..4 {
                c.press(GameButton::Up)
                    .expect("unqualified north gate input");
                c.wait_frames(128).expect("print gate refusal");
                let s = c.snapshot().expect("gate page");
                if s.ui.text_window_open {
                    let text =
                        s.ui.text
                            .as_ref()
                            .and_then(|p| p.asm_text.as_ref())
                            .expect("native refusal text");
                    assert!(
                        text.contains("BADGES") || text.contains("beacon"),
                        "actual prerequisite explanation: {text}"
                    );
                    let held = at(&mut c);
                    c.press(GameButton::Up).expect("dialogue input ownership");
                    assert_eq!(at(&mut c), held);
                    refused = true;
                    break;
                }
            }
            assert!(refused);
            settle(&mut c);
            assert_at(&mut c, TOWN, (x, 4), "refusal returns inside town");
            let after = c.snapshot().expect("denial preserved state");
            assert_eq!(before.party, after.party);
            assert_eq!(before.bag, after.bag);
            assert_eq!(
                before.progression.active_event_flags,
                after.progression.active_event_flags
            );
            save(&mut c, "north-denied");
        }
        let (_, r, mut c) = fixture(TOWN, (x, 4), true, true);
        let north = r.data().maps[TOWN]
            .attributes
            .connections
            .iter()
            .find(|e| e.direction == "north")
            .expect("actual new north connection");
        let neighbor = north.target_map.clone();
        let target = (
            x - north.offset as i16 * 2,
            r.data().maps[&neighbor].attributes.height as i16 * 2 - 1,
        );
        for y in (0..4).rev() {
            step(
                &mut c,
                GameButton::Up,
                TOWN,
                (x, y),
                "qualified north gate lane",
            );
        }
        step(
            &mut c,
            GameButton::Up,
            &neighbor,
            target,
            "native compact inland north seam",
        );
        step(
            &mut c,
            GameButton::Down,
            TOWN,
            (x, 0),
            "free return to town north",
        );
        step(
            &mut c,
            GameButton::Down,
            TOWN,
            (x, 3),
            "native inward north officer wrapper",
        );
        save(&mut c, "north-qualified-return");
    }
}

#[test]
#[ignore = "requires completed compact native pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_superior_cedar_controller_ledge_is_one_way_with_short_walking_return() {
    const CEDAR: &str = "MnMaraisCedarApproach";
    for x in [26, 27] {
        let (_, r, mut c) = fixture(CEDAR, (x, 17), true, true);
        use_max_repel(&mut c, &r);
        assert_eq!(kind(&r, CEDAR, (x, 18)), "HOP_DOWN");
        step(
            &mut c,
            GameButton::Down,
            CEDAR,
            (x, 18),
            "step onto real native ledge lip",
        );
        c.press(GameButton::Down).expect("native downward hop");
        c.wait_frames(64).expect("complete native hop");
        settle(&mut c);
        assert_at(&mut c, CEDAR, (x, 20), "safe native ledge landing");
        for _ in 0..2 {
            c.press(GameButton::Up).expect("attempt prohibited reverse");
            c.wait_frames(16).expect("native blocked reverse");
            settle(&mut c);
            assert_at(&mut c, CEDAR, (x, 20), "ledge cannot be climbed backward");
        }
        save(&mut c, "cedar-ledge-landing");
        let length = path(&r, &mut c, (x, 17), false);
        assert!(
            length > 10 && length < 30,
            "short ordinary return should cost more than drop: {length}"
        );
        assert_at(
            &mut c,
            CEDAR,
            (x, 17),
            "ordinary grassy uphill route returns to lip",
        );
        save(&mut c, "cedar-ordinary-return");
    }
}

#[test]
#[ignore = "requires completed compact native pack in CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
fn compact_superior_coast_controller_lower_neighbor_overhang_reef_blocks_all_north_water_approaches() {
    let (_, r, mut c) = fixture(COAST, (0, 13), true, true);
    use_max_repel(&mut c, &r);
    mount(&mut c, GameButton::Down, COAST, (0, 14));
    path(&r, &mut c, (0, 47), true);
    step(&mut c, GameButton::Down, "MnR11C01R01", (0, 0), "actual narrow upper-to-lower Superior aperture");
    path(&r, &mut c, (8, 2), true);
    for x in 8..47 {
        path(&r, &mut c, (x, 2), true);
        assert_eq!(kind(&r, "MnR11C01R01", (x, 1)), "WALL", "visible native rock closes the clipped north neighbor span");
        for _ in 0..2 {
            c.press(GameButton::Up).expect("real Surf attempt toward the retired northern extent");
            c.wait_frames(16).expect("native reef collision presentation");
            settle(&mut c);
            assert_at(&mut c, "MnR11C01R01", (x, 2), "lower neighbor cannot Surf into missing upper-map terrain");
            assert_eq!(c.snapshot().expect("reef approach mode").overworld.mode, MovementMode::Surf);
        }
    }
    save(&mut c, "lower-north-reef-all-approaches");
    path(&r, &mut c, (0, 0), true);
    step(&mut c, GameButton::Up, COAST, (0, 47), "actual lower-to-upper Superior aperture remains open");
    path(&r, &mut c, (0, 14), true);
    step(&mut c, GameButton::Up, COAST, (0, 13), "ordinary bank return after reciprocal Surf boundary audit");
    save(&mut c, "lower-north-reef-return");
}
