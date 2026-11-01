//! Declared local fixtures; subsequent hops, grass, dialogue and saves use input.
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
    let mut c = fixture(&root, &r, "MnR16C00R00", (43, 25));
    let state = c.shell.shell.session_mut().state_mut();
    // Geometry is reviewed with the original trainers already defeated. This
    // grants no earned journey claim and changes no installed trainer parties.
    for index in 1..=9 {
        state
            .flags
            .set_event_flag(&format!("EVENT_MN_R16_TRAINER_{index:02}"), true)
            .expect("declared prior trainer fixture");
    }
    if populated {
        let mut partner = r
            .data()
            .create_pokemon("BOREALYNX", 40, Default::default())
            .expect("original Minnesota prairie companion");
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
    if !(0..96).contains(&x) || !(0..48).contains(&y) {
        return "WALL";
    }
    let map = &r.data().maps[&format!("MnR16C{:02}R00", x / 64)];
    let (x, y) = (
        usize::try_from(x % 64).expect("x"),
        usize::try_from(y).expect("y"),
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
        .flat_map(|col| {
            r.data().maps[&format!("MnR16C{col:02}R00")]
                .objects
                .iter()
                .map(move |o| {
                    (
                        i16::try_from(o.x).expect("x") + col * 64,
                        i16::try_from(o.y).expect("y"),
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
        "ordinary prairie path {start:?} -> {end:?}"
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
            .expect("actual prairie movement")
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
        assert_eq!(position(c), old, "unexpected prairie step");
    }
    panic!("blocked prairie step {old:?} -> {to:?}");
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
        assert_eq!(position(c), old, "prairie barrier leaked");
    }
}

fn continued(c: &mut VisibleShellController, root: &AssetRoot, r: &CrystalRuntime, label: &str) {
    let dir = PathBuf::from(
        std::env::var_os("CRYSTAL_SEED_BANK_REVIEW_DIR").expect("fresh prairie saves"),
    );
    std::fs::create_dir_all(&dir).expect("retained prairie checkpoints");
    let save = dir.join(format!("{label}.crystalsave"));
    assert!(!save.exists(), "retain prior saves");
    let before = c.snapshot().expect("actual reached prairie state");
    let state = c.shell.shell.session().state().clone();
    c.save(&save).expect("save actual prairie progress");
    *c = VisibleShellController::load_save(root.clone(), r.clone(), save, None)
        .expect("native prairie Continue");
    c.set_runtime_journal_enabled(false);
    let after = c.snapshot().expect("continued prairie state");
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
#[ignore = "requires seed-bank candidate and fresh CRYSTAL_SEED_BANK_REVIEW_DIR"]
fn seed_bank_controller_two_hops_closed_sides_grassy_returns_grower_and_continue() {
    let (root, r, mut c) = setup(false);
    for x in [42, 43] {
        walk(&mut c, &r, (x, 25));
        step(&mut c, GameButton::Down, (x, 26));
        step(&mut c, GameButton::Down, (x, 28));
        blocked(&mut c, GameButton::Up);
        continued(&mut c, &root, &r, &format!("lane-{x}-landing"));
        let (grass, _) = walk(&mut c, &r, (x, 25));
        assert!(grass >= 4, "ordinary uphill return bypassed grass: {grass}");
        for via in [(37, 28), (51, 28)] {
            walk(&mut c, &r, via);
            walk(&mut c, &r, (x, 25));
        }
    }
    for x in [41, 44] {
        walk(&mut c, &r, (x, 28));
        blocked(&mut c, GameButton::Up);
    }
    walk(&mut c, &r, (47, 30));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    let text = c
        .snapshot()
        .expect("existing seed grower")
        .ui
        .text
        .expect("grower dialogue");
    println!("seed grower: {}", text.label);
    finish(&mut c);
    continued(&mut c, &root, &r, "seed-grower-return");
}

#[test]
#[ignore = "requires seed-bank candidate and fresh CRYSTAL_SEED_BANK_REVIEW_DIR"]
fn seed_bank_controller_retains_two_bridges_both_hayfield_lanes_and_chunk_seam() {
    let (root, r, mut c) = setup(false);
    for (x0, x1, ys) in [(10, 27, [23, 24]), (60, 67, [29, 30])] {
        for y in ys {
            walk(&mut c, &r, (x0, y));
            for x in x0 + 1..=x1 {
                step(&mut c, GameButton::Right, (x, y));
            }
            for x in (x0..x1).rev() {
                step(&mut c, GameButton::Left, (x, y));
            }
        }
    }
    for x in [52, 53] {
        walk(&mut c, &r, (x, 15));
        step(&mut c, GameButton::Down, (x, 16));
        step(&mut c, GameButton::Down, (x, 18));
        blocked(&mut c, GameButton::Up);
    }
    for p in [(0, 24), (95, 24), (0, 24), (43, 25)] {
        walk(&mut c, &r, p);
    }
    continued(&mut c, &root, &r, "bridges-hayfield-seam");
}

#[test]
#[ignore = "requires seed-bank candidate and fresh CRYSTAL_SEED_BANK_REVIEW_DIR"]
fn seed_bank_controller_yellow_grass_native_wild_run_zero_badges_and_continue() {
    let (root, r, mut c) = setup(true);
    let mut encounters = 0;
    for _ in 0..32 {
        encounters += walk(&mut c, &r, (36, 24)).1;
        encounters += walk(&mut c, &r, (43, 25)).1;
        if encounters > 0 {
            break;
        }
    }
    assert!(
        encounters > 0,
        "new yellow grass never started a native battle"
    );
    assert_eq!(c.shell.shell.session().state().badges.johto, [false; 8]);
    assert_eq!(
        c.snapshot().expect("normal prairie walking").overworld.mode,
        MovementMode::Normal
    );
    continued(&mut c, &root, &r, "new-grass-wild-run");
}
