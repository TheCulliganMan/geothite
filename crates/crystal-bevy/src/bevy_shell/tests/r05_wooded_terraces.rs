//! Local fixtures followed only by production movement, wild battles and item input.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::authored_route_fixture::{escape_wild, neighbors, position};
use super::interior_life::fixture;
use super::winona::{finish, load, press};

type Point = (i16, i16);

fn setup(populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnR05C00R00", (15, 41));
    let state = c.shell.shell.session_mut().state_mut();
    // Geometry is reviewed with the original trainers already defeated. This
    // grants no earned journey claim and changes no installed trainer parties.
    for index in 1..=5 {
        state
            .flags
            .set_event_flag(&format!("EVENT_MN_R05_TRAINER_{index:02}"), true)
            .expect("declared prior trainer fixture");
    }
    if populated {
        let mut partner = r
            .data()
            .create_pokemon("BOREALYNX", 40, Default::default())
            .expect("original Minnesota terrace companion");
        partner.item = Some("SMOKE_BALL".into());
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    c.set_runtime_journal_enabled(false);
    // No subsequent position, progress, party, bag, clock, HP/PP or RNG writes.
    (root, r, c)
}

fn kind(r: &CrystalRuntime, (x, y): Point) -> &str {
    if !(0..64).contains(&x) || !(0..112).contains(&y) {
        return "WALL";
    }
    let map = &r.data().maps[&format!("MnR05C00R{:02}", y / 48)];
    let (x, y) = (
        usize::try_from(x).expect("x"),
        usize::try_from(y % 48).expect("y"),
    );
    let block = map.blocks[y / 2 * usize::from(map.attributes.width) + x / 2];
    &r.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
        [y % 2 * 2 + x % 2]
}

fn mask(kind: &str, (dx, dy): Point) -> bool {
    (dy == 1 && matches!(kind, "UP_WALL" | "UP_LEFT_WALL" | "UP_RIGHT_WALL"))
        || (dy == -1 && matches!(kind, "DOWN_WALL" | "DOWN_LEFT_WALL" | "DOWN_RIGHT_WALL"))
        || (dx == 1 && matches!(kind, "LEFT_WALL" | "UP_LEFT_WALL" | "DOWN_LEFT_WALL"))
        || (dx == -1 && matches!(kind, "RIGHT_WALL" | "UP_RIGHT_WALL" | "DOWN_RIGHT_WALL"))
}

fn ordinary(kind: &str) -> bool {
    matches!(
        kind,
        "FLOOR"
            | "BRIDGE_EW_DECK"
            | "TALL_GRASS"
            | "UP_WALL"
            | "DOWN_WALL"
            | "LEFT_WALL"
            | "RIGHT_WALL"
            | "UP_LEFT_WALL"
            | "UP_RIGHT_WALL"
            | "DOWN_LEFT_WALL"
            | "DOWN_RIGHT_WALL"
    )
}

fn path(r: &CrystalRuntime, start: Point, end: Point) -> Vec<(Point, GameButton)> {
    let bodies = (0..3)
        .flat_map(|row| {
            r.data().maps[&format!("MnR05C00R{row:02}")]
                .objects
                .iter()
                .map(move |o| {
                    (
                        i16::try_from(o.x).expect("x"),
                        i16::try_from(o.y).expect("y") + row * 48,
                    )
                })
        })
        .collect::<BTreeSet<_>>();
    let mut previous = BTreeMap::from([(start, (start, GameButton::A))]);
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        if p == end {
            break;
        }
        for (q, button) in neighbors(p) {
            let d = (q.0 - p.0, q.1 - p.1);
            if bodies.contains(&q)
                || previous.contains_key(&q)
                || !ordinary(kind(r, q))
                || mask(kind(r, q), d)
                || mask(kind(r, p), (-d.0, -d.1))
            {
                continue;
            }
            previous.insert(q, (p, button));
            queue.push_back(q);
        }
    }
    assert!(
        previous.contains_key(&end),
        "ordinary terrace path {start:?} -> {end:?}"
    );
    let mut route = Vec::new();
    let mut at = end;
    while at != start {
        let (before, button) = previous[&at];
        route.push((at, button));
        at = before;
    }
    route.reverse();
    route
}

fn step(c: &mut VisibleShellController, button: GameButton, to: Point) -> bool {
    let old = position(c);
    let mut wild = false;
    for _ in 0..8 {
        press(c, button);
        if c.snapshot()
            .expect("actual terrace movement")
            .battle
            .is_some()
        {
            // This helper requires a wild battle; it cannot flee a trainer.
            escape_wild(c);
            wild = true;
        }
        if position(c) == to {
            return wild;
        }
        assert_eq!(position(c), old, "unexpected terrace step");
    }
    panic!("blocked terrace step {old:?} -> {to:?}");
}

fn walk(c: &mut VisibleShellController, r: &CrystalRuntime, to: Point) -> (usize, usize) {
    let route = path(r, position(c), to);
    let mut grass = 0;
    let mut encounters = 0;
    for (at, button) in route {
        encounters += usize::from(step(c, button, at));
        grass += usize::from(kind(r, at) == "TALL_GRASS");
    }
    (grass, encounters)
}

fn blocked(c: &mut VisibleShellController, button: GameButton) {
    let old = position(c);
    for _ in 0..4 {
        press(c, button);
        assert_eq!(position(c), old, "terrace barrier leaked");
    }
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, label: &str) {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_R05_TERRACE_REVIEW_DIR").expect("fresh terrace saves"),
    );
    std::fs::create_dir_all(&dir).expect("retained terrace checkpoints");
    let save = dir.join(format!("{label}.crystalsave"));
    assert!(!save.exists(), "retain prior saves");
    let before = c.snapshot().expect("actual reached terrace state");
    let state = c.shell.shell.session().state().clone();
    c.save(&save).expect("save actual terrace progress");
    *c = VisibleShellController::load_save(root.clone(), r.clone(), save, None)
        .expect("native terrace Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("continued terrace state");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(after.overworld.mode, MovementMode::Normal);
    assert_eq!(state.flags, c.shell.shell.session().state().flags);
    assert_eq!(state.money, c.shell.shell.session().state().money);
    assert_eq!(state.badges, c.shell.shell.session().state().badges);
}

#[test]
#[ignore = "requires current terrace candidate and fresh CRYSTAL_R05_TERRACE_REVIEW_DIR"]
fn wooded_terraces_controller_winding_paths_five_stairs_both_directions_and_continue() {
    let (root, r, mut c) = setup(false);
    for to in [
        (16, 34),
        (20, 30),
        (20, 26),
        (25, 16),
        (25, 13),
        (25, 5),
        (30, 28),
        (33, 28),
        (37, 39),
        (37, 42),
        (29, 47),
        (29, 48),
        (29, 54),
        (25, 64),
        (31, 64),
        (37, 73),
        (37, 75),
        (37, 78),
        (13, 78),
        (16, 78),
        (22, 95),
        (22, 96),
        (22, 106),
    ] {
        walk(&mut c, &r, to);
    }
    for (x, top, bottom) in [
        (13, 75, 78),
        (29, 51, 54),
        (15, 39, 42),
        (25, 13, 16),
        (37, 25, 28),
    ] {
        walk(&mut c, &r, (x, top));
        for y in top + 1..=bottom {
            step(&mut c, GameButton::Down, (x, y));
        }
        for y in (top..bottom).rev() {
            step(&mut c, GameButton::Up, (x, y));
        }
    }
    for (at, button) in [
        ((26, 31), GameButton::Down),
        ((17, 22), GameButton::Right),
        ((20, 65), GameButton::Down),
        ((25, 34), GameButton::Down),
        ((23, 68), GameButton::Down),
        ((18, 71), GameButton::Right),
        ((33, 28), GameButton::Up),
        ((40, 32), GameButton::Right),
    ] {
        walk(&mut c, &r, at);
        blocked(&mut c, button);
    }
    walk(&mut c, &r, (31, 28));
    continued(&mut c, &root, &r, "wooded-upper-cave-approach");
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
}

#[test]
#[ignore = "requires current terrace candidate and fresh CRYSTAL_R05_TERRACE_REVIEW_DIR"]
fn wooded_terraces_controller_grass_reward_once_and_continue() {
    let (root, r, mut c) = setup(false);
    let item = &r.data().items["GREAT_BALL"];
    let before = c.shell.shell.session().state().bag.quantity(item);
    walk(&mut c, &r, (37, 24));
    let (grass, _) = walk(&mut c, &r, (37, 20));
    assert!(grass >= 4, "reward approach bypassed grass");
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R05_ITEM_02")
            .expect("existing reward flag")
    );
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
    continued(&mut c, &root, &r, "great-ball-earned-through-grass");
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert_eq!(
        c.shell.shell.session().state().bag.quantity(item),
        before + 1
    );
    walk(&mut c, &r, (37, 24));
}

#[test]
#[ignore = "requires current terrace candidate and fresh CRYSTAL_R05_TERRACE_REVIEW_DIR"]
fn wooded_terraces_controller_new_grass_native_wild_run_and_continue() {
    let (root, r, mut c) = setup(true);
    let mut encounters = 0;
    walk(&mut c, &r, (37, 24));
    for _ in 0..64 {
        encounters += walk(&mut c, &r, (37, 20)).1;
        encounters += walk(&mut c, &r, (37, 24)).1;
        if encounters > 0 {
            break;
        }
    }
    assert!(
        encounters > 0,
        "new grass must support real regional wild encounters"
    );
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    continued(&mut c, &root, &r, "new-terrace-grass-after-native-run");
}
