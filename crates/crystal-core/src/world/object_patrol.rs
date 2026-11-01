//! Clockwise rectangular walks for authored map objects.
use super::map::{Direction, TilePosition};

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
