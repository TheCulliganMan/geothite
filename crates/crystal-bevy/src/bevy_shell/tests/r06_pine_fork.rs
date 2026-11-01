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
use super::winona::{load, press, travel};

type Point = (i16, i16);

fn setup(populated: bool) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    setup_at(
        if populated { (34, 19) } else { (43, 25) },
        populated,
        false,
    )
}

fn setup_at(
    at: Point,
    populated: bool,
    qualified: bool,
) -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let (root, r) = load();
    let mut c = fixture(&root, &r, "MnR06C00R00", at);
    let state = c.shell.shell.session_mut().state_mut();
    // Geometry is reviewed with the original trainers already defeated. This
    // grants no earned journey claim and changes no installed trainer parties.
    for index in 1..=5 {
        state
            .flags
            .set_event_flag(&format!("EVENT_MN_R06_TRAINER_{index:02}"), true)
            .expect("declared prior trainer fixture");
    }
    if qualified {
        state.badges.johto = [true; 8];
        for flag in [
            "ENGINE_ZEPHYRBADGE",
            "ENGINE_HIVEBADGE",
            "ENGINE_PLAINBADGE",
            "ENGINE_FOGBADGE",
            "ENGINE_STORMBADGE",
            "ENGINE_MINERALBADGE",
            "ENGINE_GLACIERBADGE",
            "ENGINE_RISINGBADGE",
        ] {
            state
                .flags
                .set_engine_flag(flag, true)
                .expect("consistent initial road qualification");
        }
    }
    if populated {
        let mut partner = r
            .data()
            .create_pokemon("BOREALYNX", 40, Default::default())
            .expect("original Minnesota pine fork companion");
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
    if !(0..64).contains(&x) || !(0..88).contains(&y) {
        return "WALL";
    }
    let map = &r.data().maps[&format!("MnR06C00R{:02}", y / 48)];
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
    let bodies = (0..2)
        .flat_map(|row| {
            r.data().maps[&format!("MnR06C00R{row:02}")]
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
        "ordinary pine fork path {start:?} -> {end:?}"
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
            .expect("actual pine fork movement")
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
        assert_eq!(position(c), old, "unexpected pine fork step");
    }
    panic!("blocked pine fork step {old:?} -> {to:?}");
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
        assert_eq!(position(c), old, "pine fork barrier leaked");
    }
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, label: &str) {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_R06_PINE_FORK_REVIEW_DIR").expect("fresh pine fork saves"),
    );
    std::fs::create_dir_all(&dir).expect("retained pine fork checkpoints");
    let save = dir.join(format!("{label}.crystalsave"));
    assert!(!save.exists(), "retain prior saves");
    let before = c.snapshot().expect("actual reached pine fork state");
    let state = c.shell.shell.session().state().clone();
    c.save(&save).expect("save actual pine fork progress");
    *c = VisibleShellController::load_save(root.clone(), r.clone(), save, None)
        .expect("native pine fork Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("continued pine fork state");
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
#[ignore = "requires pine-fork candidate and fresh CRYSTAL_R06_PINE_FORK_REVIEW_DIR"]
fn pine_fork_controller_two_drops_reverse_sides_grassy_returns_and_continue() {
    let (root, r, mut c) = setup(false);
    for x in [42, 43] {
        walk(&mut c, &r, (x, 25));
        step(&mut c, GameButton::Down, (x, 26));
        step(&mut c, GameButton::Down, (x, 28));
        blocked(&mut c, GameButton::Up);
        continued(&mut c, &root, &r, &format!("new-fork-lane-{x}-landing"));
        let (grass, _) = walk(&mut c, &r, (x, 25));
        assert!(grass >= 6, "dry return bypassed habitat: {grass}");
        step(&mut c, GameButton::Down, (x, 26));
        step(&mut c, GameButton::Down, (x, 28));
        let mut grass = walk(&mut c, &r, (34, 26)).0;
        grass += walk(&mut c, &r, (34, 19)).0;
        grass += walk(&mut c, &r, (x, 25)).0;
        assert!(grass >= 6, "west return bypassed habitat");
    }
    for x in [41, 44] {
        walk(&mut c, &r, (x, 25));
        blocked(&mut c, GameButton::Down);
        walk(&mut c, &r, (x, 28));
        blocked(&mut c, GameButton::Up);
    }
    walk(&mut c, &r, (43, 25));
    continued(&mut c, &root, &r, "new-fork-ordinary-returns");
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
}

#[test]
#[ignore = "requires pine-fork candidate and fresh CRYSTAL_R06_PINE_FORK_REVIEW_DIR"]
fn pine_fork_controller_retained_north_hops_both_bridge_lanes_cabins_and_chunk_seam() {
    let (root, r, mut c) = setup(false);
    for x in [44, 45] {
        walk(&mut c, &r, (x, 9));
        step(&mut c, GameButton::Down, (x, 10));
        step(&mut c, GameButton::Down, (x, 12));
        blocked(&mut c, GameButton::Up);
        let (grass, _) = walk(&mut c, &r, (x, 9));
        assert!(grass >= 8, "original north hop gained a dry bypass");
    }
    for to in [
        (50, 47),
        (50, 48),
        (50, 47),
        (50, 48),
        (9, 62),
        (37, 40),
        (16, 86),
        (43, 25),
    ] {
        walk(&mut c, &r, to);
    }
    for y in [66, 67] {
        walk(&mut c, &r, (25, y));
        for x in 26..=38 {
            step(&mut c, GameButton::Right, (x, y));
        }
        for x in (25..38).rev() {
            step(&mut c, GameButton::Left, (x, y));
        }
    }
    walk(&mut c, &r, (43, 25));
    continued(&mut c, &root, &r, "retained-hops-bridges-cabins");
}

#[test]
#[ignore = "requires pine-fork candidate and fresh CRYSTAL_R06_PINE_FORK_REVIEW_DIR"]
fn pine_fork_controller_new_west_grass_native_wild_run_and_continue() {
    let (root, r, mut c) = setup(true);
    let mut encounters = 0;
    for _ in 0..64 {
        encounters += walk(&mut c, &r, (34, 25)).1;
        encounters += walk(&mut c, &r, (34, 19)).1;
        if encounters > 0 {
            break;
        }
    }
    assert!(
        encounters > 0,
        "new western grass requires a real regional encounter"
    );
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    continued(&mut c, &root, &r, "new-west-habitat-after-native-run");
}

#[test]
#[ignore = "requires pine-fork candidate and fresh CRYSTAL_R06_PINE_FORK_REVIEW_DIR"]
fn pine_fork_controller_two_lanes_both_real_town_seams_and_continue() {
    let (root, r, mut c) = setup_at((43, 25), false, true);
    continued(&mut c, &root, &r, "initial-qualified-road-fixture");
    for (inside, button, neighbor, outside) in [
        ((42, 0), GameButton::Up, "MnBrainerd", (32, 47)),
        ((43, 0), GameButton::Up, "MnBrainerd", (33, 47)),
        ((16, 87), GameButton::Down, "MnStCloud", (24, 0)),
        ((17, 87), GameButton::Down, "MnStCloud", (25, 0)),
    ] {
        walk(&mut c, &r, inside);
        travel(&mut c, button, neighbor, outside);
        let entered = c.snapshot().expect("actual neighboring town arrival");
        assert_eq!(entered.overworld.map_name, neighbor);
        assert_eq!(
            (entered.overworld.tile.x, entered.overworld.tile.y),
            outside
        );
        c.wait_frames(64).expect("stable real town arrival");
        let stable = c.snapshot().expect("settled neighboring town");
        assert_eq!(stable.overworld.map_name, neighbor);
        println!(
            "road arrival {neighbor} {:?}; after64 {:?}",
            outside,
            (stable.overworld.tile.x, stable.overworld.tile.y)
        );
        travel(
            &mut c,
            if button == GameButton::Up {
                GameButton::Down
            } else {
                GameButton::Up
            },
            &format!("MnR06C00R{:02}", inside.1 / 48),
            (inside.0, inside.1 % 48),
        );
        assert_eq!(position(&mut c), inside);
    }
    walk(&mut c, &r, (43, 25));
    continued(&mut c, &root, &r, "both-real-town-seams");
}
