//! The corrected Minnesota references are printed by the actual room residents.
use std::collections::{BTreeSet, VecDeque};

use crystal_assets::MapModule;
use crystal_runtime::CrystalRuntime;

use super::{Point, collision, fixture, load, talk, walk};

fn approach(
    runtime: &CrystalRuntime,
    map: &MapModule,
    start: Point,
    actor: Point,
) -> (Point, Point) {
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some((x, y)) = todo.pop_front() {
        for q in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if seen.contains(&q)
                || map.objects.iter().any(|o| (o.x as i16, o.y as i16) == q)
                || collision(runtime, map, q) != Some("FLOOR")
            {
                continue;
            }
            seen.insert(q);
            todo.push_back(q);
        }
    }
    for (dx, dy) in [(0, 1), (-1, 0), (1, 0), (0, -1)] {
        let beside = (actor.0 + dx, actor.1 + dy);
        if seen.contains(&beside) {
            return (beside, actor);
        }
        let customer = (actor.0 + dx * 2, actor.1 + dy * 2);
        if collision(runtime, map, beside) == Some("COUNTER") && seen.contains(&customer) {
            return (customer, beside);
        }
    }
    panic!("{}: no native conversation approach to {actor:?}", map.id);
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the active regional flavor overlay"]
fn interior_life_regional_partners_are_read_in_all_eight_room_conversations() {
    let (root, runtime, _) = load();
    for (name, script, native, legacy) in [
        (
            "MnDuluthFieldClinic",
            "MN_DL_INTERIOR_CLINIC_VISITOR",
            "AUGRUB",
            "MACHOP",
        ),
        (
            "MnDuluthHillRanger",
            "MN_DL_INTERIOR_OWL_LISTENER",
            "PIPBOOM",
            "HOOTHOOT",
        ),
        (
            "MnDuluthOreWarehouse1F",
            "MN_DL_INTERIOR_WAREHOUSE_WORKER",
            "AUGRUB",
            "MACHOP",
        ),
        (
            "MnMinneapolisAnglerHome",
            "MN_MP_INTERIOR_ANGLER_CHILD",
            "GILLPUP",
            "WOOPER",
        ),
        (
            "MnMinneapolisFieldClub",
            "MN_MP_FIELDCLUB_HOST",
            "GOOSNIDE",
            "MURKROW",
        ),
        (
            "MnMinneapolisRiverNaturalist",
            "MN_MP_INTERIOR_NATURE_STUDENT",
            "GILLPUP",
            "WOOPER",
        ),
        (
            "MnMinneapolisRiverNaturalist",
            "MN_MP_RIVERNATURALIST_HOST",
            "GILLPUP",
            "WOOPER",
        ),
        (
            "MnStPaulFairCraftHouse",
            "MN_SP_INTERIOR_CRAFT_LEARNER",
            "PINIT",
            "SUNKERN",
        ),
    ] {
        let map = &runtime.data().maps[name];
        let start = map
            .events
            .warps
            .iter()
            .map(|w| (w.x as i16, w.y as i16))
            .find(|p| collision(&runtime, map, *p) == Some("FLOOR"))
            .expect("actual passive room arrival");
        let resident = map
            .objects
            .iter()
            .find(|o| o.script == script)
            .expect("original resident");
        let (stand, target) =
            approach(&runtime, map, start, (resident.x as i16, resident.y as i16));
        let mut c = fixture(&root, &runtime, name, start);
        walk(&mut c, &runtime, stand);
        let printed = talk(&mut c, target);
        assert!(printed.contains(native), "{name}/{script}: {printed}");
        assert!(
            !printed.contains(legacy),
            "old species must not remain in the actual conversation"
        );
        if script == "MN_DL_INTERIOR_OWL_LISTENER" {
            assert!(
                !printed.contains("My owl"),
                "the drawing is now the local grouse, not an owl"
            );
        }
    }
}
