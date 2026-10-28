//! Explicit bridge decks above navigable water in authored content packs.
//!
//! These four reserved permissions are opt-in derived tiles. Original Crystal
//! collision bytes and `describe_collision` retain their existing semantics.
//! Walkers use the deck/rail layer; an already Surfing player uses the water
//! layer along its declared axis, without climbing sideways onto an abutment.

use super::collision::{PlayerTraversalState, permissions};
use super::map::Direction;

/// Walkable deck above a north/south water passage.
pub const NORTH_SOUTH_DECK: u8 = 0xe0;
/// Blocking road rail above a north/south water passage.
pub const NORTH_SOUTH_RAIL: u8 = 0xe1;
/// Walkable deck above an east/west water passage.
pub const EAST_WEST_DECK: u8 = 0xe2;
/// Blocking road rail above an east/west water passage.
pub const EAST_WEST_RAIL: u8 = 0xe3;

/// Whether this is an explicit authored bridge, rather than an ordinary road.
pub const fn is_bridge(permission: u8) -> bool {
    matches!(permission, NORTH_SOUTH_DECK..=EAST_WEST_RAIL)
}

/// Collision seen by the current traversal layer. Unrelated bytes are exact.
pub const fn layer_permission(permission: u8, traversal: PlayerTraversalState) -> u8 {
    if !is_bridge(permission) {
        return permission;
    }
    match traversal {
        PlayerTraversalState::Surf => permissions::WATER,
        PlayerTraversalState::Walk => match permission {
            NORTH_SOUTH_DECK | EAST_WEST_DECK => permissions::FLOOR,
            _ => permissions::WALL,
        },
    }
}

/// Prevents Surf from exiting sideways into the elevated road or bank.
pub const fn blocks_exit(
    permission: u8,
    facing: Direction,
    traversal: PlayerTraversalState,
) -> bool {
    if !matches!(traversal, PlayerTraversalState::Surf) {
        return false;
    }
    match permission {
        NORTH_SOUTH_DECK | NORTH_SOUTH_RAIL => matches!(facing, Direction::Left | Direction::Right),
        EAST_WEST_DECK | EAST_WEST_RAIL => matches!(facing, Direction::Up | Direction::Down),
        _ => false,
    }
}

/// Returns the two-layer entry rule, or `None` for original collision bytes.
pub const fn can_enter(
    permission: u8,
    facing: Direction,
    traversal: PlayerTraversalState,
) -> Option<bool> {
    if !is_bridge(permission) {
        return None;
    }
    Some(match traversal {
        PlayerTraversalState::Surf => !blocks_exit(permission, facing, traversal),
        PlayerTraversalState::Walk => matches!(permission, NORTH_SOUTH_DECK | EAST_WEST_DECK),
    })
}

#[cfg(test)]
#[path = "bridge_tests.rs"]
mod tests;
