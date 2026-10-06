//! Source-approved ground cells combined with the exact built elevation grid.
//! A complete animated footprint must fit; a union AABB cannot fill holes.
//! Walking steps and settled trainer contacts keep separate witness contracts.
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
        let (target, tiles) = match &location.target {
            VisualBattleTarget::WalkingGrass { presentation, .. } => (
                presentation.presentation_core_tile,
                &presentation.walkable_core_tiles,
            ),
            VisualBattleTarget::SurfWater { presentation, .. } => (
                presentation.presentation_core_tile,
                &presentation.water_core_tiles,
            ),
            VisualBattleTarget::Trainer { presentation, .. } => (
                presentation.presentation_core_tile,
                &presentation.walkable_core_tiles,
            ),
            _ => return Ok(None),
        };
        let invalid = "derived battle presentation lacks checked source ground";
        let water = matches!(location.target, VisualBattleTarget::SurfWater { .. });
        if water && cache.built_profiles.is_none() {
            return Err("water support lacks actual build profile provenance");
        }
        let frame = cache.built_frame.as_ref().ok_or(invalid)?;
        if water && frame.validate().is_err() {
            return Err(invalid);
        }
        let map_size = location.source_map_size_core_tiles.ok_or(invalid)?;
        if map_size.min_element() == 0 || map_size.max_element() > i32::MAX as u32 / 2 {
            return Err(invalid);
        }
        let map = map_size.as_ivec2();
        let anchors = location.anchors.as_ref().ok_or(invalid)?;
        let source = location.source.core_tile;
        let facing = location.source.facing;
        if [source, target]
            .into_iter()
            .any(|tile| tile.cmplt(IVec2::ZERO).any() || tile.cmpge(map).any())
            || ![IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y].contains(&facing)
        {
            return Err(invalid);
        }
        let delta = target - source;
        if delta.abs().element_sum() != 2
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
        let landed = source_foot(source);
        if !anchors.source_foot.is_finite()
            || !anchors.target_foot.is_finite()
            || anchors.source_foot.distance_squared(landed) > 0.0001
            || anchors.target_foot.distance_squared(source_foot(target)) > 0.0001
        {
            return Err(invalid);
        }
        match &location.target {
            VisualBattleTarget::WalkingGrass {
                core_tile,
                presentation,
            } => {
                if location.source.movement
                    != crystal_render_api::VisualBattleSourceMovement::Normal
                    || *core_tile != source
                    || presentation.step_from_core_tile + facing != source
                {
                    return Err(invalid);
                }
                let start = source_foot(presentation.step_from_core_tile);
                let witness = presentation.witnessed_player_foot.ok_or(invalid)?;
                let step = landed - start;
                let progress = (witness - start).dot(step) / step.length_squared();
                if !witness.is_finite()
                    || !(-0.0001..=1.0001).contains(&progress)
                    || witness.distance_squared(start + step * progress) > 0.0001
                {
                    return Err(invalid);
                }
            }
            VisualBattleTarget::SurfWater {
                core_tile,
                presentation,
            } => {
                if location.source.movement != crystal_render_api::VisualBattleSourceMovement::Surf
                    || *core_tile != source
                    || presentation.step_from_core_tile + facing != source
                {
                    return Err(invalid);
                }
                let start = source_foot(presentation.step_from_core_tile);
                let witness = presentation.witnessed_player_foot.ok_or(invalid)?;
                let step = landed - start;
                let progress = (witness - start).dot(step) / step.length_squared();
                if !witness.is_finite()
                    || !(-0.0001..=1.0001).contains(&progress)
                    || witness.distance_squared(start + step * progress) > 0.0001
                {
                    return Err(invalid);
                }
            }
            VisualBattleTarget::Trainer { contact, .. } => {
                // The trainer remains a separately witnessed field actor. Its
                // occupied tile can never authorize the Pokémon's floor area.
                // Both checked entry paths have settled adjacent, facing actors.
                // A scripted start has no invented trainer-table defeat flag.
                let provenance_valid = match contact.provenance {
                    crystal_render_api::VisualBattleTrainerProvenance::TrainerTable { .. } => {
                        !contact.event_flag.is_empty()
                    }
                    crystal_render_api::VisualBattleTrainerProvenance::Scripted {
                        loadtrainer_command_index,
                        startbattle_command_index,
                    } => loadtrainer_command_index < startbattle_command_index,
                };
                let witness = contact.witnessed_foot.ok_or(invalid)?;
                if location.source.movement
                    != crystal_render_api::VisualBattleSourceMovement::Normal
                    || contact.core_tile != source + facing
                    || contact.facing != -facing
                    || !matches!(contact.witnessed_actor, Some(VisualActorId::Object(_)))
                    || !witness.is_finite()
                    || witness.distance_squared(source_foot(contact.core_tile)) > 0.0001
                    || tiles.contains(&contact.core_tile)
                    || contact.object_identifier.is_empty()
                    || contact.object_script.is_empty()
                    || contact.trainer_class.is_empty()
                    || contact.trainer_id.is_empty()
                    || !provenance_valid
                    || contact.battle_source_script.is_empty()
                {
                    return Err(invalid);
                }
            }
            _ => unreachable!("derived ground variants selected above"),
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
                if water {
                    let at_cell = |tile: &&crystal_render_api::VisualTile| {
                        tile.column == grid.x as u32 && tile.row == grid.y as u32
                    };
                    let tile = frame
                        .tiles
                        .get(index)
                        .filter(at_cell)
                        .or_else(|| frame.tiles.iter().find(at_cell))
                        .ok_or(invalid)?;
                    let profiles = cache.built_profiles.as_deref().ok_or(invalid)?;
                    if !proven_water_cell(location.source.map_id.as_ref(), &tile.source, profiles) {
                        // An approved collision tile may contain a drawn bank.
                        // Keep a hole here; only a body's actually covered cells
                        // must prove water. Never fill an outer ring by its AABB.
                        continue;
                    }
                }
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

/// Collision permission alone cannot prove the shape actually built beneath
/// the waterline. A live profile can override either the drawing or its ground.
fn proven_water_cell(
    map: &str,
    source: &crystal_render_api::VisualTileSource,
    profiles: &crate::live_profiles::Document,
) -> bool {
    crate::profile::shape_for_source_on_map(map, source) == crate::profile::CellShape::Water
        && !profiles.objects.iter().any(|object| {
            object.tileset == source.tileset_id.as_ref()
                && object.map.as_deref().is_none_or(|id| id == map)
                && object
                    .maps
                    .as_ref()
                    .is_none_or(|ids| ids.iter().any(|id| id == map))
                && (object.ground == source.tile_index
                    || object
                        .tiles
                        .iter()
                        .flatten()
                        .any(|&id| id == source.tile_index))
        })
}
