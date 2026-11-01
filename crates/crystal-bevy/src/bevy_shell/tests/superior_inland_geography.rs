//! Explicit local Floor/party fixtures followed by production controller inputs.
//! These checks establish local traversal and persistence, not earned progression.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, assert_at, at, fixture_with_items_on_runtime,
    mark_runtime_snapshot_dirty, settle, step,
};

const NECK: &str = "MnMaraisInlandNeck";
const CEDAR: &str = "MnMaraisCedarApproach";
const FLAG: &str = "EVENT_MN_GUNFLINT_CAMP_SUPPLY";

fn load() -> (AssetRoot, CrystalRuntime) {
    static LOADED: OnceLock<(AssetRoot, CrystalRuntime)> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let path = PathBuf::from(
                std::env::var_os("CRYSTAL_SUPERIOR_GEOGRAPHY_CONTROLLER_PACK")
                    .expect("explicit completed geography or final winter pack"),
            )
            .canonicalize()
            .expect("completed native pack");
            let root = AssetRoot::new(path.parent().expect("pack directory"));
            let runtime = CrystalRuntime::from_loaded_compiled_pack(
                &root,
                read_loaded_verified_compiled_game_pack(path).expect("verified geography pack"),
            )
            .expect("production native runtime");
            assert_eq!(runtime.data().maps[NECK].attributes.width, 65);
            (root, runtime)
        })
        .clone()
}

fn fixture(
    map: &str,
    p: Point,
    repel: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) =
        fixture_with_items_on_runtime(load(), map, p, false, false, false, &[("MAX_REPEL", 99)]);
    assert_eq!(kind(&r, map, p), "FLOOR", "initial legal Floor fixture");
    if repel {
        use_max_repel(&mut c, &r);
    }
    (root, r, c)
}

fn kind(r: &CrystalRuntime, map: &str, p: Point) -> String {
    let m = &r.data().maps[map];
    let x = usize::try_from(p.0).expect("nonnegative x");
    let y = usize::try_from(p.1).expect("nonnegative y");
    let b = m.blocks[y / 2 * usize::from(m.attributes.width) + x / 2];
    r.data().tilesets[&m.attributes.tileset_name].collision[&format!("{b:02x}")][y % 2 * 2 + x % 2]
        .clone()
}

fn button(a: Point, b: Point) -> GameButton {
    match (b.0 - a.0, b.1 - a.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        d => panic!("nonadjacent walking input {d:?}"),
    }
}

/// Plans ordinary walking only; native jumps are separately exercised below.
fn walk(r: &CrystalRuntime, c: &mut VisibleShellController, target: Point) -> usize {
    let (map, start) = at(c);
    let m = &r.data().maps[&map];
    let occupied = m
        .objects
        .iter()
        .map(|o| {
            (
                i16::try_from(o.x).expect("actor x"),
                i16::try_from(o.y).expect("actor y"),
            )
        })
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
                || occupied.contains(&n)
                || m.events.warps.iter().any(|w| (w.x as i16, w.y as i16) == n)
                || m.events
                    .coord_events
                    .iter()
                    .any(|e| (e.x as i16, e.y as i16) == n && n != target)
                || previous.contains_key(&n)
                || !(matches!(kind(r, &map, n).as_str(), "FLOOR" | "TALL_GRASS")
                    || n == target && kind(r, &map, n) == "HOP_DOWN")
            {
                continue;
            }
            previous.insert(n, Some(p));
            queue.push_back(n);
        }
    }
    assert!(
        previous.contains_key(&target),
        "ordinary return path {map} {start:?}->{target:?}"
    );
    let mut course = vec![];
    let mut end = target;
    while let Some(p) = previous[&end] {
        course.push((p, end));
        end = p;
    }
    let length = course.len();
    for (a, b) in course.into_iter().rev() {
        step(c, button(a, b), &map, b, "ordinary native inland trail");
        assert!(
            c.snapshot().expect("trail snapshot").battle.is_none(),
            "repelled trail must stay clear"
        );
        if c.snapshot()
            .expect("repel duration")
            .progression
            .repel_steps_remaining
            == 0
        {
            use_max_repel(c, r);
        }
    }
    length
}

fn retain(c: &mut VisibleShellController, label: &str) -> PathBuf {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_SUPERIOR_GEOGRAPHY_SAVE_DIR")
            .expect("caller-named retained save directory"),
    );
    std::fs::create_dir_all(&dir).expect("save directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = dir.join(format!(
        "superior-{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists());
    c.save(&path).expect("production reached Save");
    path
}

#[test]
#[ignore = "requires completed external geography pack and retained save directory"]
fn superior_geography_controller_inland_seams_and_full_connector_return() {
    let (root, r, mut c) = fixture("MnR12C01R01", (63, 8), true);
    step(
        &mut c,
        GameButton::Right,
        NECK,
        (0, 8),
        "R12 actual east seam",
    );
    walk(&r, &mut c, (129, 12));
    step(
        &mut c,
        GameButton::Right,
        CEDAR,
        (0, 12),
        "Cedar actual west seam",
    );
    step(
        &mut c,
        GameButton::Left,
        NECK,
        (129, 12),
        "Cedar ordinary return seam",
    );
    walk(&r, &mut c, (0, 8));
    let before = c.snapshot().expect("reached connector return");
    let saved = retain(&mut c, "inland-crossing");
    c = VisibleShellController::load_save(root, r.clone(), saved, None).expect("real Continue");
    let after = c.snapshot().expect("continued connector");
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.party, after.party);
    step(
        &mut c,
        GameButton::Left,
        "MnR12C01R01",
        (63, 8),
        "R12 ordinary return seam",
    );
    retain(&mut c, "r12-return");
}

#[test]
#[ignore = "requires completed external geography pack and retained save directory"]
fn superior_geography_controller_two_one_way_ledges_and_useful_walking_returns() {
    for (takeoff, landing, return_length) in [((24, 10), (24, 12), 26), ((82, 12), (82, 14), 24)] {
        let (_, r, mut c) = fixture(NECK, (takeoff.0, takeoff.1 - 1), true);
        assert_eq!(kind(&r, NECK, takeoff), "HOP_DOWN");
        step(
            &mut c,
            GameButton::Down,
            NECK,
            takeoff,
            "enter ledge upper lip",
        );
        c.press(GameButton::Down)
            .expect("native downward ledge input");
        c.wait_frames(64).expect("native hop completion");
        settle(&mut c);
        assert_at(&mut c, NECK, landing, "native jump lands two tiles below");
        for _ in 0..2 {
            c.press(GameButton::Up).expect("reverse ledge attempt");
            c.wait_frames(16).expect("blocked reverse");
            settle(&mut c);
            assert_at(&mut c, NECK, landing, "native reverse blocked");
        }
        retain(&mut c, "ledge-landing");
        assert_eq!(
            walk(&r, &mut c, takeoff),
            return_length,
            "ordinary useful return around cliff"
        );
        retain(&mut c, "ledge-return");
    }
}

#[test]
#[ignore = "requires completed external geography pack and retained save directory"]
fn superior_geography_controller_camp_three_berries_once_only_after_continue_and_reentry() {
    let (root, r, mut c) = fixture(NECK, (62, 27), true);
    let mut expected = c.shell.shell.session().state().bag.clone();
    assert!(
        expected
            .add_item(&r.data().items["BERRY"], 3)
            .expect("three Berry capacity")
    );
    c.press(GameButton::Right).expect("face camp supplies");
    c.press(GameButton::A).expect("native itemball interaction");
    settle(&mut c);
    assert_eq!(c.shell.shell.session().state().bag, expected);
    assert!(
        c.snapshot()
            .expect("pickup flag")
            .progression
            .active_event_flags
            .contains(FLAG)
    );
    let path = retain(&mut c, "camp-collected");
    c = VisibleShellController::load_save(root, r.clone(), path, None)
        .expect("actual camp Continue");
    c.press(GameButton::A).expect("repeat collected pickup");
    settle(&mut c);
    assert_eq!(c.shell.shell.session().state().bag, expected);
    walk(&r, &mut c, (129, 12));
    step(
        &mut c,
        GameButton::Right,
        CEDAR,
        (0, 12),
        "leave collected camp map",
    );
    step(
        &mut c,
        GameButton::Left,
        NECK,
        (129, 12),
        "reenter collected camp map",
    );
    walk(&r, &mut c, (62, 27));
    c.press(GameButton::Right).expect("face former pickup");
    c.press(GameButton::A).expect("repeat after map return");
    settle(&mut c);
    assert_eq!(c.shell.shell.session().state().bag, expected);
    assert!(
        c.snapshot()
            .expect("persistent camp flag")
            .progression
            .active_event_flags
            .contains(FLAG)
    );
    retain(&mut c, "camp-reentered");
}

#[test]
#[ignore = "requires completed external geography pack and retained save directory"]
fn superior_geography_controller_native_cold_grass_starts_real_encounter() {
    let (_, r, mut c) = fixture(NECK, (29, 14), false);
    assert_eq!(kind(&r, NECK, (30, 14)), "TALL_GRASS");
    assert_eq!(kind(&r, NECK, (31, 14)), "TALL_GRASS");
    let mut grass_steps = 0;
    for _ in 0..2048 {
        if c.snapshot()
            .expect("native encounter check")
            .battle
            .is_some()
        {
            break;
        }
        let before = at(&mut c);
        assert_eq!(before.0, NECK);
        assert_eq!(before.1.1, 14);
        assert!((29..=31).contains(&before.1.0));
        // Native movement first turns when facing changes. Choose direction
        // from the reached tile, keeping it held across that turning input,
        // rather than alternating Left/Right and turning forever in place.
        c.press(if before.1.0 < 31 {
            GameButton::Right
        } else {
            GameButton::Left
        })
        .expect("ordinary encounter habitat input");
        c.wait_frames(16).expect("native encounter engine clock");
        let after = at(&mut c);
        if before != after && kind(&r, NECK, after.1) == "TALL_GRASS" {
            grass_steps += 1;
        }
        // Do not call the shared water-travel settle helper here: its explicit
        // battle branch escapes encounters and would erase the evidence.
    }
    assert!(
        c.snapshot()
            .expect("actual wild encounter")
            .battle
            .is_some(),
        "copied native table must produce a battle after {grass_steps} actual grass steps"
    );
    retain(&mut c, "cold-grass-battle");
}

fn qualified_coastal_fixture() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r, mut c) = fixture("MnR11C00R00", (40, 6), false);
    let state = c.shell.shell.session_mut().state_mut();
    for (index, flag) in [
        "ENGINE_ZEPHYRBADGE",
        "ENGINE_HIVEBADGE",
        "ENGINE_PLAINBADGE",
        "ENGINE_FOGBADGE",
        "ENGINE_STORMBADGE",
        "ENGINE_MINERALBADGE",
        "ENGINE_GLACIERBADGE",
        "ENGINE_RISINGBADGE",
    ]
    .iter()
    .enumerate()
    {
        state.badges.johto[index] = true;
        state
            .flags
            .set_engine_flag(flag, true)
            .expect("explicit qualified badge fixture");
    }
    state
        .flags
        .set_event_flag("EVENT_MN_LIGHTHOUSE_SIGNAL", true)
        .expect("explicit initial Duluth signal fixture");
    mark_runtime_snapshot_dirty(&mut c.shell);
    // Fixture boundary: all later movement, gates, map changes and item use are
    // production controller inputs, with no state, flag, party or RNG writes.
    use_max_repel(&mut c, &r);
    (root, r, c)
}

fn town_gate(c: &mut VisibleShellController, key: GameButton, target: Point) {
    for _ in 0..12 {
        if at(c) == ("MnGrandMarais".into(), target) {
            return;
        }
        c.press(key).expect("qualified native officer lane input");
        c.wait_frames(16)
            .expect("native officer movement presentation");
        settle(c);
        assert_eq!(
            at(c).0,
            "MnGrandMarais",
            "gate movement stays inside until physical seam"
        );
    }
    assert_at(
        c,
        "MnGrandMarais",
        target,
        "qualified native gate reaches physical edge",
    );
}

#[test]
#[ignore = "requires completed external geography pack and retained save directory"]
fn superior_geography_controller_continuous_warm_coast_town_cedar_gunflint_and_return_both_lanes() {
    // Both walking quadrants at every new seam are exercised in a continuous
    // journey, each beginning on the preserved original North Shore trail.
    for lane in 0..2 {
        let (root, r, mut c) = qualified_coastal_fixture();
        walk(&r, &mut c, (63, 12 + lane));
        step(
            &mut c,
            GameButton::Right,
            "MnR11C01R00",
            (0, 12 + lane),
            "actual R11 chunk seam",
        );
        walk(&r, &mut c, (47, 8 + lane));
        step(
            &mut c,
            GameButton::Right,
            "MnGrandMarais",
            (0, 8 + lane),
            "warm bend enters town west",
        );
        town_gate(&mut c, GameButton::Right, (3, 8 + lane));
        walk(&r, &mut c, (18 + lane, 4));
        town_gate(&mut c, GameButton::Up, (18 + lane, 0));
        step(
            &mut c,
            GameButton::Up,
            CEDAR,
            (56 + lane, 37),
            "separate town north exit reaches Cedar south",
        );
        walk(&r, &mut c, (0, 12 + lane));
        step(
            &mut c,
            GameButton::Left,
            NECK,
            (129, 12 + lane),
            "full Cedar walk reaches inland connector",
        );
        walk(&r, &mut c, (0, 8 + lane));
        step(
            &mut c,
            GameButton::Left,
            "MnR12C01R01",
            (63, 8 + lane),
            "continuous coast to fixed Gunflint",
        );
        let saved = retain(&mut c, "continuous-coast-to-gunflint");
        let before = c.snapshot().expect("reached Gunflint");
        c = VisibleShellController::load_save(root, r.clone(), saved, None)
            .expect("reached Gunflint Continue");
        let after = c.snapshot().expect("continued Gunflint");
        assert_eq!(before.overworld.map_name, after.overworld.map_name);
        assert_eq!(before.overworld.tile, after.overworld.tile);
        assert_eq!(before.party, after.party);
        assert_eq!(
            before.progression.active_event_flags,
            after.progression.active_event_flags
        );
        step(
            &mut c,
            GameButton::Right,
            NECK,
            (0, 8 + lane),
            "Gunflint return to actual connector",
        );
        walk(&r, &mut c, (129, 12 + lane));
        step(
            &mut c,
            GameButton::Right,
            CEDAR,
            (0, 12 + lane),
            "connector return to Cedar west",
        );
        walk(&r, &mut c, (56 + lane, 37));
        step(
            &mut c,
            GameButton::Down,
            "MnGrandMarais",
            (18 + lane, 0),
            "full Cedar return to town north",
        );
        town_gate(&mut c, GameButton::Down, (18 + lane, 3));
        walk(&r, &mut c, (4, 8 + lane));
        town_gate(&mut c, GameButton::Left, (0, 8 + lane));
        step(
            &mut c,
            GameButton::Left,
            "MnR11C01R00",
            (47, 8 + lane),
            "town west returns to same warm coast",
        );
        walk(&r, &mut c, (0, 12 + lane));
        step(
            &mut c,
            GameButton::Left,
            "MnR11C00R00",
            (63, 12 + lane),
            "headland returns through real chunk seam",
        );
        walk(&r, &mut c, (40, 6));
        retain(&mut c, "continuous-gunflint-to-original-coast");
    }
}
