//! Visible terrain cues use the active map's blocks and static walking permissions.
use std::collections::BTreeMap;

use crystal_core::map::MapConnection;
use crystal_core::world::collision::{
    PlayerTraversalState, Terrain, allows_ledge_direction, describe_collision,
    is_grass_encounter_permission, is_permission_passable, is_warp_permission, permissions,
};
use crystal_core::world::map::{Direction, TilePosition, determine_quadrant_index};
use crystal_core::world::movement::{
    DEFAULT_RUNTIME_TILE_STRIDE, checked_move_by_stride, crosses_declared_map_edge,
};
use crystal_runtime::{RuntimeMapCatalogSnapshot, RuntimeShellSnapshot};

use super::LineKind;

const UNKNOWN: (char, LineKind) = ('?', LineKind::Danger);

pub(crate) fn terrain_glyph(
    source: &RuntimeShellSnapshot,
    map: &RuntimeMapCatalogSnapshot,
    tile: TilePosition,
) -> (char, LineKind) {
    let Some(tileset) = source
        .tilesets
        .iter()
        .find(|t| t.tileset_id == map.attributes.tileset_name)
    else {
        return UNKNOWN;
    };
    TerrainMap {
        width: map.attributes.width,
        height: map.attributes.height,
        connections: &map.attributes.connections,
        blocks: &map.blocks,
        collision: &tileset.collision,
    }
    .glyph(tile)
}

struct TerrainMap<'a> {
    width: u16,
    height: u16,
    connections: &'a [MapConnection],
    blocks: &'a [u16],
    collision: &'a BTreeMap<String, Vec<String>>,
}

impl TerrainMap<'_> {
    fn glyph(&self, tile: TilePosition) -> (char, LineKind) {
        let bounds = (self.width.saturating_mul(2), self.height.saturating_mul(2));
        if !within(tile, bounds) {
            return (' ', LineKind::Normal);
        }
        let Some(permission) = self.permission(tile) else {
            return UNKNOWN;
        };
        glyph_for_permission(permission, |direction| {
            let landing = checked_move_by_stride(tile, direction, 2 * DEFAULT_RUNTIME_TILE_STRIDE)?;
            if !within(landing, bounds) {
                return Some(crosses_declared_map_edge(
                    bounds,
                    self.connections,
                    landing,
                    direction,
                ));
            }
            // A trainer crossing the landing can delay a jump, but does not change the terrain.
            let permission = self.permission(landing)?;
            Some(is_permission_passable(
                permission,
                direction,
                PlayerTraversalState::Walk,
            ))
        })
    }

    fn permission(&self, tile: TilePosition) -> Option<u8> {
        let x = usize::try_from(tile.x).ok()? / 2;
        let y = usize::try_from(tile.y).ok()? / 2;
        let block = self.blocks.get(y * usize::from(self.width) + x)?;
        let token = self
            .collision
            .get(&format!("{block:02x}"))?
            .get(determine_quadrant_index(tile.x, tile.y)?)?;
        resolve_permission(token)
    }
}

fn within(tile: TilePosition, (width, height): (u16, u16)) -> bool {
    (0..i32::from(width)).contains(&i32::from(tile.x))
        && (0..i32::from(height)).contains(&i32::from(tile.y))
}

fn resolve_permission(token: &str) -> Option<u8> {
    crystal_assets::resolve_collision_token(token.strip_prefix("COLL_").unwrap_or(token)).ok()
}

fn glyph_for_permission(
    permission: u8,
    can_land: impl Fn(Direction) -> Option<bool>,
) -> (char, LineKind) {
    if (permissions::HOP_RIGHT..=permissions::HOP_UP_LEFT).contains(&permission) {
        let mut dx = 0;
        let mut dy = 0;
        let mut unknown = false;
        for direction in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            if allows_ledge_direction(permission, direction) {
                match can_land(direction) {
                    Some(true) => {
                        let (x, y) = direction.delta();
                        dx += x;
                        dy += y;
                    }
                    None => unknown = true,
                    Some(false) => {}
                }
            }
        }
        return match (dx, dy) {
            (1, 0) => ('→', LineKind::Hint),
            (-1, 0) => ('←', LineKind::Hint),
            (0, -1) => ('↑', LineKind::Hint),
            (0, 1) => ('↓', LineKind::Hint),
            (1, 1) => ('↘', LineKind::Hint),
            (-1, 1) => ('↙', LineKind::Hint),
            (1, -1) => ('↗', LineKind::Hint),
            (-1, -1) => ('↖', LineKind::Hint),
            _ if unknown => UNKNOWN,
            _ => ('.', LineKind::Normal),
        };
    }
    match permission {
        0xb0..=0xbf | 0xc0..=0xcf => return ('║', LineKind::Hint),
        0x12 | 0x15 | 0x1a | 0x1d => return ('T', LineKind::Hint),
        _ => {}
    }
    if describe_collision(permission).terrain == Terrain::Wall {
        ('#', LineKind::Normal)
    } else if is_warp_permission(permission) {
        ('D', LineKind::Hint)
    } else if describe_collision(permission).terrain == Terrain::Water {
        ('≈', LineKind::Water)
    } else if is_grass_encounter_permission(permission) {
        ('"', LineKind::Grass)
    } else {
        ('.', LineKind::Normal)
    }
}

#[cfg(test)]
#[path = "terrain_glyph_tests.rs"]
mod tests;
