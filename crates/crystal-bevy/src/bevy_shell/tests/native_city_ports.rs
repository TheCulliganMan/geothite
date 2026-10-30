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

/// Finds a reachable dry approach to an authored eastern road connection.
/// Planning reads live collision and actors; crossing still uses controller input.
pub(super) fn east_road_port(
    controller: &mut super::super::VisibleShellController,
    runtime: &CrystalRuntime,
    road: &str,
) -> (Point, Point) {
    let name = controller
        .snapshot()
        .expect("current city departure")
        .overworld
        .map_name;
    let city = &runtime.data().maps[&name];
    let target = &runtime.data().maps[road];
    let connection = city
        .attributes
        .connections
        .iter()
        .find(|link| link.direction == "east" && link.target_map == road)
        .expect("authored eastern road connection");
    let edge_x =
        i16::try_from(i32::from(city.attributes.width) * 2 - 1).expect("native city east edge");
    let mut best: Option<(usize, Point, Point)> = None;
    for y in 1..i32::from(city.attributes.height) * 2 - 1 {
        let target_y = y - connection.offset * 2;
        if target_y < 0 || target_y >= i32::from(target.attributes.height) * 2 {
            continue;
        }
        let y = i16::try_from(y).expect("native city edge row");
        let target_y = i16::try_from(target_y).expect("native road arrival row");
        let approach = (edge_x - 1, y);
        let arrival = (0, target_y);
        if collision(runtime, city, (edge_x, y)) != Some("FLOOR")
            || collision(runtime, city, approach) != Some("FLOOR")
            || !matches!(
                collision(runtime, target, arrival),
                Some("FLOOR" | "TALL_GRASS")
            )
        {
            continue;
        }
        if let Some(path) = super::northstar_ice_lair::path(controller, runtime, approach)
            && super::northstar_ice_lair::path(controller, runtime, (edge_x, y)).is_some()
            && best
                .as_ref()
                .is_none_or(|(length, _, _)| path.len() < *length)
        {
            best = Some((path.len(), approach, arrival));
        }
    }
    let (_, approach, arrival) = best.expect("reachable native city-to-road departure");
    (approach, arrival)
}
