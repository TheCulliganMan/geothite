//! Use actual indexed city doors after native facade fitting.
use crystal_core::map::WarpEvent;
use crystal_runtime::CrystalRuntime;

use super::super::interior_life::{collision, target_name};

type Point = (i16, i16);

fn point(warp: &WarpEvent) -> Point {
    (
        i16::try_from(warp.x).expect("native warp x"),
        i16::try_from(warp.y).expect("native warp y"),
    )
}

pub(super) fn city_port(runtime: &CrystalRuntime, room: &str) -> (Point, Point) {
    let city = &runtime.data().maps["MnMinneapolis"];
    let inside = &runtime.data().maps[room];
    let exit = inside
        .events
        .warps
        .iter()
        .find(|w| target_name(runtime, &w.target_map_constant) == city.id)
        .expect("room's actual city exit");
    let arrival = city
        .events
        .warps
        .iter()
        .find(|w| w.index == u16::try_from(exit.target_warp_id).expect("city return index"))
        .expect("indexed passive city return");
    let entrance = city
        .events
        .warps
        .iter()
        .find(|w| target_name(runtime, &w.target_map_constant) == room)
        .expect("room's actual city facade");
    let interior_arrival = inside
        .events
        .warps
        .iter()
        .find(|w| w.index == u16::try_from(entrance.target_warp_id).expect("room arrival index"))
        .expect("indexed passive room arrival");
    let outside = point(arrival);
    let interior = point(interior_arrival);
    assert_eq!(collision(runtime, city, outside), Some("FLOOR"));
    assert_eq!(collision(runtime, inside, interior), Some("FLOOR"));
    (outside, interior)
}

#[test]
#[ignore = "requires the actual same-pack CRYSTAL_ICE_RELAY_CHECKPOINT and isolated saves"]
fn ice_headquarters_reached_relay_checkpoint_walks_city_bridges_and_finishes_rescue() {
    let (root, runtime) = super::load();
    let checkpoint = std::env::var_os("CRYSTAL_ICE_RELAY_CHECKPOINT")
        .expect("relay checkpoint written by the declared-fixture controller course");
    let controller = super::VisibleShellController::load_save(
        root.clone(),
        runtime.clone(),
        std::path::PathBuf::from(checkpoint),
        None,
    )
    .expect("same-pack Continue at the reached relay stage");
    let mut controller =
        super::finish_from_relay(controller, &root, &runtime, &super::source(), true);
    assert!(super::event(&controller, super::RESCUED));
    assert!(!super::event(&controller, "EVENT_MN_DISPATCH_NOTE"));
    assert!(!controller.has_pending_script_work());
    assert_eq!(
        controller
            .snapshot()
            .expect("rescue Continue")
            .overworld
            .map_name,
        "MnMinneapolisDispatchOffice"
    );
}
