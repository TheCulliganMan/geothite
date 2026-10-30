//! One unpaid basecamp/trail/court circuit, then an actual paid admission.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::world::map::TilePosition;
use crystal_runtime::CrystalRuntime;

use super::super::minneapolis_falls::grand_marais_harbor::use_max_repel;
use super::{
    Point, VisibleShellController, button, collision, continue_save, controller_fixture,
    mark_runtime_snapshot_dirty, press, save,
};

fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, target: Point) {
    let snapshot = c.snapshot().expect("public walking origin");
    let name = snapshot.overworld.map_name;
    let map = &runtime.data().maps[&name];
    let start = (snapshot.overworld.tile.x, snapshot.overworld.tile.y);
    let blocked = map
        .objects
        .iter()
        .map(|o| (o.x as i16, o.y as i16))
        .collect::<BTreeSet<_>>();
    let mut previous = BTreeMap::from([(start, start)]);
    let mut pending = VecDeque::from([start]);
    while let Some(at) = pending.pop_front() {
        if at == target {
            break;
        }
        for next in [
            (at.0 - 1, at.1),
            (at.0 + 1, at.1),
            (at.0, at.1 - 1),
            (at.0, at.1 + 1),
        ] {
            if !previous.contains_key(&next)
                && !blocked.contains(&next)
                && matches!(collision(runtime, map, next), Some("FLOOR" | "TALL_GRASS"))
            {
                previous.insert(next, at);
                pending.push_back(next);
            }
        }
    }
    assert!(
        previous.contains_key(&target),
        "unpaid public path in {name}: {start:?}->{target:?}"
    );
    let mut route = vec![target];
    while route.last().copied() != Some(start) {
        route.push(previous[route.last().expect("public route tail")]);
    }
    route.reverse();
    for pair in route.windows(2) {
        for _ in 0..8 {
            let s = c.snapshot().expect("native public walking");
            if s.overworld.tile == TilePosition::new(pair[1].0, pair[1].1) {
                break;
            }
            press(c, button(pair[0], pair[1]));
        }
        let s = c.snapshot().expect("reached public tile");
        assert_eq!(s.overworld.map_name, name);
        assert_eq!(s.overworld.tile, TilePosition::new(pair[1].0, pair[1].1));
        assert!(s.safari.is_none(), "public walking cannot issue a permit");
        assert!(
            s.battle.is_none(),
            "genuinely used Repel protects this traversal fixture"
        );
        assert_eq!(s.trainer.money, 1500);
    }
}

fn seam(c: &mut VisibleShellController, input: GameButton, map: &str, tile: Point) {
    for _ in 0..8 {
        press(c, input);
        if c.snapshot()
            .expect("actual public connection")
            .overworld
            .map_name
            == map
        {
            break;
        }
    }
    let s = c.snapshot().expect("public connection arrival");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!(s.overworld.tile, TilePosition::new(tile.0, tile.1));
    assert!(s.safari.is_none());
    assert_eq!(s.trainer.money, 1500);
}

#[test]
#[ignore = "requires enabled Safari review with obsolete basecamp blockers retired"]
fn safari_unpaid_hub_pine_court_circuit_then_real_paid_entry() {
    let (root, runtime, mut c) = controller_fixture();
    // Explicit previously earned regional team/item fixture, before any input.
    // A real Max Repel use protects the free trail, not a post-input RNG edit.
    let mut lead = runtime
        .data()
        .create_pokemon("GILLPUP", 75, Dv::default())
        .expect("regional strong lead");
    lead.moves = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("Surf partner")
        .moves
        .clone();
    let state = c.shell.shell.session_mut().state_mut();
    state.storage.party.pokemon[0] = Some(lead);
    for species in ["VOYAGEAR", "DOZEBEAR"] {
        assert!(
            state.storage.party.add_pokemon(
                runtime
                    .data()
                    .create_pokemon(species, 40, Dv::default())
                    .expect("legal regional companion")
            )
        );
    }
    assert!(
        state
            .bag
            .add_item(&runtime.data().items["MAX_REPEL"], 1)
            .expect("initial earned Max Repel")
    );
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    let original_party = c.snapshot().expect("initial regional team").party;
    use_max_repel(&mut c, &runtime);
    walk(&mut c, &runtime, (32, 5));
    for x in [32, 33] {
        assert!(
            c.shell
                .shell
                .session()
                .overworld()
                .visible_object_at_checked(TilePosition::new(x, 6))
                .expect("retired basecamp slot")
                .is_none()
        );
    }
    save(&mut c, "public-hub-throat-open");
    walk(&mut c, &runtime, (32, 0));
    seam(&mut c, GameButton::Up, "MnPreservePineEntry", (16, 31));
    walk(&mut c, &runtime, (16, 0));
    seam(&mut c, GameButton::Up, "MnBoundarySafariC00R02", (14, 15));
    for x in [11, 12] {
        walk(&mut c, &runtime, (x, 6));
        assert!(
            c.shell
                .shell
                .session()
                .overworld()
                .visible_object_at_checked(TilePosition::new(x, 5))
                .expect("actual unpaid permit guard")
                .is_some()
        );
        for _ in 0..4 {
            press(&mut c, GameButton::Up);
            let s = c.snapshot().expect("unpaid preserve gate stays closed");
            assert_eq!(s.overworld.tile, TilePosition::new(x, 6));
            assert!(s.safari.is_none());
        }
    }
    save(&mut c, "public-court-without-permit");
    c = continue_save(root.clone(), runtime.clone(), &mut c);
    walk(&mut c, &runtime, (15, 15));
    seam(&mut c, GameButton::Down, "MnPreservePineEntry", (17, 0));
    walk(&mut c, &runtime, (17, 31));
    seam(&mut c, GameButton::Down, "MnBoundaryWaters", (33, 0));
    walk(&mut c, &runtime, (33, 7));
    walk(&mut c, &runtime, (30, 10));
    let before = c.snapshot().expect("complete free circuit");
    assert!(before.safari.is_none());
    assert_eq!(before.trainer.money, 1500);
    let mut expected_party = original_party;
    for (expected, actual) in expected_party.slots.iter_mut().zip(&before.party.slots) {
        expected.pokemon.happiness = actual.pokemon.happiness;
    }
    assert_eq!(expected_party, before.party);
    save(&mut c, "public-circuit-returned-basecamp");
    c = continue_save(root, runtime, &mut c);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    let paid = c
        .snapshot()
        .expect("actual admission after the public loop");
    let session = paid.safari.expect("fresh paid permit");
    assert_eq!(session.steps_remaining, 500);
    assert_eq!(session.balls_remaining, 30);
    assert_eq!(paid.trainer.money, 1000);
    assert_eq!(paid.overworld.map_name, "MnBoundarySafariC00R02");
    assert_eq!(paid.overworld.tile, TilePosition::new(15, 9));
    save(&mut c, "public-circuit-paid-entry");
}
