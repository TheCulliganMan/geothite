//! Resolve the actual indexed public doorway pair used by a city fixture.
use crystal_core::map::WarpEvent;
use crystal_runtime::CrystalRuntime;

use super::interior_life::{collision, target_name};

type Point = (i16, i16);

fn point(warp: &WarpEvent) -> Point {
    (
        i16::try_from(warp.x).expect("door x"),
        i16::try_from(warp.y).expect("door y"),
    )
}

pub(super) fn city_port(runtime: &CrystalRuntime, city: &str, room: &str) -> (Point, Point) {
    let city = &runtime.data().maps[city];
    let inside = &runtime.data().maps[room];
    let exit = inside
        .events
        .warps
        .iter()
        .find(|w| target_name(runtime, &w.target_map_constant) == city.id)
        .expect("room's city exit");
    let arrival = city
        .events
        .warps
        .iter()
        .find(|w| w.index == u16::try_from(exit.target_warp_id).expect("city return index"))
        .expect("indexed city return");
    let entrance = city
        .events
        .warps
        .iter()
        .find(|w| target_name(runtime, &w.target_map_constant) == room)
        .expect("city facade");
    let interior_arrival = inside
        .events
        .warps
        .iter()
        .find(|w| w.index == u16::try_from(entrance.target_warp_id).expect("room arrival index"))
        .expect("indexed room arrival");
    let outside = point(arrival);
    let interior = point(interior_arrival);
    assert_eq!(collision(runtime, city, outside), Some("FLOOR"));
    assert_eq!(collision(runtime, inside, interior), Some("FLOOR"));
    (outside, interior)
}
