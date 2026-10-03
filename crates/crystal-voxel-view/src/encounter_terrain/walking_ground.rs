//! Source-approved ground cells combined with the exact built elevation grid.
//! A complete animated footprint must fit; a union AABB cannot fill holes.
use super::*;
use crystal_render_api::{VisualActorId, VisualBattleTarget};

pub(super) struct WalkingGround {
    origin: Vec2,
    cells: HashMap<IVec2, f32>,
}
impl WalkingGround {
    pub(super) fn from_context(
        location: &VisualBattleLocation,
        cache: &TerrainRevisionCache,
        pose: Transform,
    ) -> Result<Option<Self>, &'static str> {
        let VisualBattleTarget::WalkingGrass {
            core_tile,
            presentation,
        } = &location.target
        else {
            return Ok(None);
        };
        let invalid = "walking presentation lacks checked source ground";
        let frame = cache.built_frame.as_ref().ok_or(invalid)?;
        let map = location
            .source_map_size_core_tiles
            .ok_or(invalid)?
            .as_ivec2();
        let anchors = location.anchors.as_ref().ok_or(invalid)?;
        let source = location.source.core_tile;
        let facing = location.source.facing;
        let target = presentation.presentation_core_tile;
        let delta = target - source;
        let tiles = &presentation.walkable_core_tiles;
        if *core_tile != source
            || facing.abs().element_sum() != 1
            || presentation.step_from_core_tile + facing != source
            || delta.abs().element_sum() != 2
            || (delta.x != 0 && delta.y != 0)
            || tiles.is_empty()
            || tiles.len() > 49
            || anchors.source_actor != VisualActorId::Player
            || anchors.target_actor.is_some()
            || !tiles.contains(&source)
            || !tiles.contains(&target)
            || !tiles.contains(&(source + delta / 2))
        {
            return Err(invalid);
        }
        let grid_delta =
            (frame.grid_origin - anchors.terrain.grid_origin).as_vec2() * frame.tile_size;
        let center = anchors.terrain.center + Vec2::new(grid_delta.x, -grid_delta.y);
        let local_zero = -frame.grid_size.as_vec2() * frame.tile_size * 0.5
            - frame.grid_origin.as_vec2() * frame.tile_size;
        let source_foot = |tile: IVec2| {
            let local = local_zero + (tile.as_vec2() * 2.0 + Vec2::new(1.0, 2.0)) * frame.tile_size;
            Vec2::new(local.x + center.x, -local.y + center.y)
        };
        let start = source_foot(presentation.step_from_core_tile);
        let landed = source_foot(source);
        let witness = presentation.witnessed_player_foot.ok_or(invalid)?;
        let step = landed - start;
        let progress = (witness - start).dot(step) / step.length_squared();
        if !witness.is_finite()
            || !(-0.0001..=1.0001).contains(&progress)
            || witness.distance_squared(start + step * progress) > 0.0001
            || anchors.source_foot.distance_squared(landed) > 0.0001
            || anchors.target_foot.distance_squared(source_foot(target)) > 0.0001
        {
            return Err(invalid);
        }
        let origin = pose.transform_point(Vec3::new(local_zero.x, 0.0, local_zero.y));
        let mut cells = HashMap::new();
        for &tile in tiles.iter() {
            if tile.cmplt(IVec2::ZERO).any()
                || tile.cmpge(map).any()
                || (tile - source).abs().max_element() > 3
            {
                return Err(invalid);
            }
            for offset in [IVec2::ZERO, IVec2::X, IVec2::Y, IVec2::ONE] {
                let cell = tile * 2 + offset;
                let grid = cell - frame.grid_origin;
                if grid.cmplt(IVec2::ZERO).any() || grid.cmpge(frame.grid_size.as_ivec2()).any() {
                    // Collision support without corresponding built ground is unavailable.
                    continue;
                }
                let index = grid.y as usize * frame.grid_size.x as usize + grid.x as usize;
                let height = *cache.built_footing_heights.get(index).ok_or(invalid)?;
                if !height.is_finite() {
                    return Err(invalid);
                }
                let height = pose.transform_point(Vec3::Y * height).y;
                if cells.insert(cell, height).is_some() {
                    return Err(invalid);
                }
            }
        }
        Ok(Some(Self {
            origin: Vec2::new(origin.x, origin.z),
            cells,
        }))
    }

    pub(super) fn contains(&self, body: BattleBody, pose: Transform, ground_y: f32) -> bool {
        let corners = body.corners(pose);
        if !ground_y.is_finite() || corners.iter().any(|p| !p.is_finite()) {
            return false;
        }
        let min = corners
            .iter()
            .fold(Vec3::splat(f32::INFINITY), |a, &b| a.min(b));
        let max = corners
            .iter()
            .fold(Vec3::splat(f32::NEG_INFINITY), |a, &b| a.max(b));
        let cell_size = SOURCE_CELL_SIZE * RETAINED_SCALE;
        let lo = ((Vec2::new(min.x, min.z) - self.origin) / cell_size + Vec2::splat(0.00001))
            .floor()
            .as_ivec2();
        let hi = ((Vec2::new(max.x, max.z) - self.origin) / cell_size - Vec2::splat(0.00001))
            .floor()
            .as_ivec2()
            .max(lo);
        if (hi - lo).max_element() > 13 {
            return false;
        }
        for y in lo.y..=hi.y {
            for x in lo.x..=hi.x {
                if !self
                    .cells
                    .get(&IVec2::new(x, y))
                    .is_some_and(|&height| (height - ground_y).abs() <= 0.02)
                {
                    return false;
                }
            }
        }
        true
    }
}
