//! Clockwise rectangular walks for authored map objects.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::map::{Direction, TilePosition};
use super::movement::checked_move_by_stride;
use super::session::object_tile_position_checked;
use crate::map::ObjectEvent;

/// The sight cutscene writes a trainer's approached coordinates into the
/// mutable map event. Keep the original walking circuit across that write.
pub(super) fn trainer_origins(
    map: &str,
    objects: &[ObjectEvent],
) -> BTreeMap<String, TilePosition> {
    if !map.starts_with("Mn") {
        return BTreeMap::new();
    }
    objects
        .iter()
        .filter(|object| {
            object.object_type == "OBJECTTYPE_TRAINER"
                && matches!(
                    object.spritemovedata.as_str(),
                    "SPRITEMOVEDATA_PATROL_CLOCKWISE" | "SPRITEMOVEDATA_PATROL_CLOCKWISE_SLOW"
                )
        })
        .filter_map(|object| {
            Some((
                object.object_identifier.clone()?,
                object_tile_position_checked(object)?,
            ))
        })
        .collect()
}

/// Select the next perimeter step without keeping a separate route cursor.
/// Width and height count walking tiles from the original top-left event tile.
/// Invalid rectangles and positions off the perimeter have no next step.
pub(crate) fn clockwise_step(
    anchor: TilePosition,
    current: TilePosition,
    width: u16,
    height: u16,
) -> Option<Direction> {
    if width == 0 || height == 0 {
        return None;
    }
    let right = anchor.x.checked_add(i16::try_from(width).ok()?)?;
    let bottom = anchor.y.checked_add(i16::try_from(height).ok()?)?;
    if !(anchor.x..=right).contains(&current.x) || !(anchor.y..=bottom).contains(&current.y) {
        return None;
    }
    if current.y == anchor.y && current.x < right {
        Some(Direction::Right)
    } else if current.x == right && current.y < bottom {
        Some(Direction::Down)
    } else if current.y == bottom && current.x > anchor.x {
        Some(Direction::Left)
    } else if current.x == anchor.x && current.y > anchor.y {
        Some(Direction::Up)
    } else {
        None
    }
}

/// Find the nearest accessible point on the circuit after a trainer steps
/// out to challenge the player. Leave room for a detour around a guest.
pub(super) fn return_step(
    anchor: TilePosition,
    current: TilePosition,
    width: u16,
    height: u16,
    mut can_step: impl FnMut(TilePosition, Direction, TilePosition) -> bool,
) -> Option<Direction> {
    let right = anchor.x.checked_add(i16::try_from(width).ok()?)?;
    let bottom = anchor.y.checked_add(i16::try_from(height).ok()?)?;
    if width == 0 || height == 0 {
        return None;
    }
    let min_x = current.x.min(anchor.x).saturating_sub(2);
    let max_x = current.x.max(right).saturating_add(2);
    let min_y = current.y.min(anchor.y).saturating_sub(2);
    let max_y = current.y.max(bottom).saturating_add(2);
    let mut visited = BTreeSet::from([(current.x, current.y)]);
    let mut queue = VecDeque::from([(current, None)]);
    while let Some((from, first)) = queue.pop_front() {
        for direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let Some(to) = checked_move_by_stride(from, direction, 1) else {
                continue;
            };
            if !(min_x..=max_x).contains(&to.x)
                || !(min_y..=max_y).contains(&to.y)
                || visited.contains(&(to.x, to.y))
                || !can_step(from, direction, to)
            {
                continue;
            }
            let heading = first.unwrap_or(direction);
            if clockwise_step(anchor, to, width, height).is_some() {
                return Some(heading);
            }
            visited.insert((to.x, to.y));
            queue.push_back((to, Some(heading)));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::clockwise_step;
    use crate::world::map::{Direction, TilePosition};

    #[test]
    fn rectangular_patrol_closes_each_lap_with_the_same_direction_order() {
        let anchor = TilePosition::new(3, 5);
        let mut at = anchor;
        for _ in 0..3 {
            for direction in [
                Direction::Right,
                Direction::Right,
                Direction::Right,
                Direction::Down,
                Direction::Down,
                Direction::Left,
                Direction::Left,
                Direction::Left,
                Direction::Up,
                Direction::Up,
            ] {
                assert_eq!(clockwise_step(anchor, at, 3, 2), Some(direction));
                let (dx, dy) = direction.delta();
                at.x += dx;
                at.y += dy;
            }
            assert_eq!(at, anchor);
        }
    }

    #[test]
    fn malformed_or_displaced_patrol_does_not_walk_outside_its_route() {
        let anchor = TilePosition::new(3, 5);
        for at in [
            TilePosition::new(2, 5),
            TilePosition::new(7, 5),
            TilePosition::new(4, 6),
            TilePosition::new(3, 8),
        ] {
            assert_eq!(clockwise_step(anchor, at, 3, 2), None);
        }
        assert_eq!(clockwise_step(anchor, anchor, 0, 2), None);
        assert_eq!(clockwise_step(anchor, anchor, 3, 0), None);
        assert_eq!(
            clockwise_step(TilePosition::new(i16::MAX, 0), anchor, 1, 1),
            None
        );
    }
}
