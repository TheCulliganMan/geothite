//! Empty-party/local-position/RNG fixture, then production input only.
//! Child of r01_hollows; original guide, medicine and warp cases stay intact.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::{
    Point, ROOM, VisibleShellController, collected, continue_save, enter, fixture, flags,
    indoor_walk, kind, leave, map, neighbors, point, press, quantity, source, tile,
};

fn input_plan() -> Value {
    let path = std::env::var_os("CRYSTAL_R01_UPPER_LOOP_PLAN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../planning/minnesota/renders/route-quality-next/r01/\
                 cave-v02-upper-loop-c4816242/controller-input-plan.json",
            )
        });
    serde_json::from_str(&std::fs::read_to_string(path).expect("frozen R01 upper-loop input plan"))
        .expect("explicit R01 upper-loop course JSON")
}

fn course(plan: &Value, key: &str, steps: usize) -> Vec<Point> {
    let points = plan[key]
        .as_array()
        .expect("declared adjacent-step course")
        .iter()
        .map(point)
        .collect::<Vec<_>>();
    assert_eq!(points.len(), steps + 1, "exact source course length: {key}");
    for pair in points.windows(2) {
        assert_eq!(
            (pair[1].0 - pair[0].0).abs() + (pair[1].1 - pair[0].1).abs(),
            1,
            "source course must contain every cardinal step: {key}, {pair:?}"
        );
    }
    points
}

fn walk_course(
    controller: &mut VisibleShellController,
    runtime: &CrystalRuntime,
    points: &[Point],
    label: &str,
    initial_flags: &(BTreeSet<String>, BTreeSet<String>),
) {
    assert_eq!(map(controller), ROOM, "{label} starts in the native cave");
    assert_eq!(
        tile(controller),
        *points.first().expect("nonempty explicit course"),
        "{label} exact starting tile"
    );
    for (index, pair) in points.windows(2).enumerate() {
        let [from, target] = [pair[0], pair[1]];
        assert_eq!(kind(runtime, from), "FLOOR", "{label}, {from:?}");
        assert_eq!(kind(runtime, target), "FLOOR", "{label}, {target:?}");
        let button = neighbors(from)
            .into_iter()
            .find_map(|(at, button)| (at == target).then_some(button))
            .expect("explicit adjacent native input");
        let mut observations = Vec::new();
        for _ in 0..4 {
            if tile(controller) == target {
                break;
            }
            assert_eq!(
                tile(controller),
                from,
                "{label} cannot leave its named edge"
            );
            // A native turn may precede movement. Every later input still uses
            // the same production boundary, never an indoor pathfinder detour.
            press(controller, button);
            let observed = controller.snapshot().expect("observed native edge input");
            observations.push((observed.overworld.tile, observed.overworld.facing));
            assert_eq!(observed.overworld.map_name, ROOM, "{label} cannot warp");
            assert_eq!(
                &flags(controller),
                initial_flags,
                "{label} step{index} cannot change event/engine flags"
            );
        }
        assert_eq!(
            tile(controller),
            target,
            "{label} step{index}: {from:?} -> {target:?}, observations={observations:?}"
        );
    }
}

#[test]
#[ignore = "requires published R01 upper-loop pack and matching frozen cave source"]
fn r01_upper_loop_controller_complete_cycle_short_long_returns_and_upper_seam_continue() {
    let (root, runtime, mut controller) = fixture(false);
    let initial_flags = flags(&controller);
    let before = controller
        .snapshot()
        .expect("explicit empty geometry fixture");
    assert!(
        before.party.slots.is_empty(),
        "no party or badges are earned here"
    );
    assert_eq!(quantity(&controller, &runtime), 0);
    let plan = input_plan();
    let loop_path = course(&plan, "complete_loop_core", 26);
    let short_return = course(&plan, "short_east_return", 6);
    let guided_return = course(&plan, "long_guided_return", 18);
    assert_eq!(loop_path.first(), Some(&(9, 10)));
    assert_eq!(loop_path.last(), Some(&(9, 10)));
    assert!(
        loop_path.contains(&(4, 5)),
        "original guide approach is visited"
    );
    assert!(
        loop_path.contains(&(8, 3)),
        "new upper seam is genuinely traversed"
    );
    assert!(
        loop_path.contains(&(10, 5)),
        "original Antidote approach is visited"
    );
    assert!(
        loop_path.contains(&(10, 7)),
        "short east stair closes the cycle"
    );
    assert_eq!(source()["dimensions_metatiles"], serde_json::json!([8, 7]));
    enter(&mut controller, &runtime);
    assert_eq!(flags(&controller), initial_flags);

    let save_at = (7, 3);
    assert_eq!(
        kind(&runtime, save_at),
        "FLOOR",
        "real new unoccupied save tile"
    );
    assert!(
        runtime.data().maps[ROOM]
            .objects
            .iter()
            .all(|object| (object.x as i16, object.y as i16) != save_at)
    );
    let save_index = loop_path
        .iter()
        .position(|&at| at == save_at)
        .expect("actual source cycle passes the upper-seam save tile");
    walk_course(
        &mut controller,
        &runtime,
        &loop_path[..=save_index],
        "guide hollow to upper seam",
        &initial_flags,
    );
    controller = continue_save(root, runtime.clone(), &mut controller);
    assert_eq!(map(&mut controller), ROOM);
    assert_eq!(
        tile(&mut controller),
        save_at,
        "Continue retains the new Floor position"
    );
    assert_eq!(flags(&controller), initial_flags);
    walk_course(
        &mut controller,
        &runtime,
        &loop_path[save_index..],
        "continued seam to Antidote alcove and short return",
        &initial_flags,
    );

    indoor_walk(&mut controller, &runtime, (10, 5));
    assert_eq!(flags(&controller), initial_flags);
    walk_course(
        &mut controller,
        &runtime,
        &short_return,
        "six-step east-stair return",
        &initial_flags,
    );
    indoor_walk(&mut controller, &runtime, (10, 5));
    assert_eq!(flags(&controller), initial_flags);
    walk_course(
        &mut controller,
        &runtime,
        &guided_return,
        "eighteen-step guide-side return",
        &initial_flags,
    );
    assert_eq!(tile(&mut controller), (9, 10));
    assert!(
        !collected(&controller),
        "walking alone cannot grant the Antidote"
    );
    assert_eq!(quantity(&controller, &runtime), 0);
    leave(&mut controller, &runtime);
    assert_eq!(flags(&controller), initial_flags);
    let after = controller
        .snapshot()
        .expect("complete native geometry journey");
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
}
