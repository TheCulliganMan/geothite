//! Source-approved ground cells combined with the exact built elevation grid.
//! A complete animated footprint must fit; a union AABB cannot fill holes.
//! Walking steps and settled trainer contacts keep separate witness contracts.
use super::*;
use crystal_render_api::{VisualActorId, VisualBattleTarget};

pub(super) struct WalkingGround {
    origin: Vec2,
    cells: HashMap<IVec2, f32>,
    grass_cells: std::collections::HashSet<IVec2>,
}
impl WalkingGround {
    pub(super) fn grass_contacts(&self, anchors: [Vec3; 2]) -> usize {
        anchors.into_iter().filter(|point| {
            let cell = ((Vec2::new(point.x, point.z) - self.origin)
                / (SOURCE_CELL_SIZE * RETAINED_SCALE) - Vec2::splat(0.00001)).floor().as_ivec2();
            self.grass_cells.contains(&cell)
        }).count()
    }
    pub(super) fn nearby_stage_pairs(&self, original: [Vec3; 2], bodies: [Option<BattleBody>; 2]) -> Vec<[Vec3; 2]> {
        let tile_size = SOURCE_CELL_SIZE * RETAINED_SCALE * 2.0;
        // A large body can fit a two-row floor band only between the rows'
        // tile centers. Sample the already witnessed half-tile grid for those
        // bodies, keeping the same world radius and separation candidates.
        let fine = bodies.into_iter().flatten().any(|body| {
            let (min, max) = body.visual_bounds.unwrap_or((body.min, body.max));
            let extent = (max - min) * body.scale;
            extent.x.max(extent.z) > tile_size
        });
        let (radius, step) = if fine { (10, tile_size * 0.5) } else { (5, tile_size) };
        let delta = original[1] - original[0];
        let direction = Vec3::new(delta.x, 0.0, delta.z).normalize_or_zero();
        if direction == Vec3::ZERO { return Vec::new(); }
        let right = Vec3::new(-direction.z, 0.0, direction.x);
        let height = |point: Vec3| {
            let cell = ((Vec2::new(point.x, point.z) - self.origin)
                / (tile_size * 0.5) - Vec2::splat(0.00001)).floor().as_ivec2();
            self.cells.get(&cell).copied()
        };
        let mut offsets: Vec<_> = (-radius..=radius).flat_map(|z|
            (-radius..=radius).map(move |x| (x, z))).collect();
        offsets.sort_by_key(|&(x, z)| (x * x + z * z, z, x));
        let mut pairs = Vec::new();
        for (x, z) in offsets {
            let source = original[0] + Vec3::new(x as f32, 0.0, z as f32) * step;
            for facing in [direction, right, -right, -direction] {
                for separation in [2.0, 1.0] {
                    let target = source + facing * tile_size * separation;
                    if let (Some(a), Some(b)) = (height(source), height(target)) {
                        pairs.push([Vec3::new(source.x, a, source.z), Vec3::new(target.x, b, target.z)]);
                    }
                }
            }
        }
        pairs
    }

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
            VisualBattleTarget::Trainer { presentation, .. }
            | VisualBattleTarget::ScriptedGround { presentation } => (
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
        let radius = match location.target {
            VisualBattleTarget::WalkingGrass { .. } => crystal_render_api::WALKING_BATTLE_GROUND_RADIUS,
            VisualBattleTarget::ScriptedGround { .. } => 5,
            _ => 3,
        };
        if delta.abs().element_sum() != 2
            || (delta.x != 0 && delta.y != 0)
            || tiles.is_empty()
            || tiles.len() > ((radius * 2 + 1) * (radius * 2 + 1)) as usize
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
            VisualBattleTarget::ScriptedGround { .. } => {
                if location.source.movement != crystal_render_api::VisualBattleSourceMovement::Normal {
                    return Err(invalid);
                }
            }
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
        let mut grass_cells = std::collections::HashSet::new();
        for &tile in tiles.iter() {
            if tile.cmplt(IVec2::ZERO).any()
                || tile.cmpge(map).any()
                || (tile - source).abs().max_element() > radius
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
                if matches!(location.target, VisualBattleTarget::WalkingGrass { .. }) {
                    if frame.tiles.iter().find(|tile| tile.column == grid.x as u32 && tile.row == grid.y as u32)
                        .is_some_and(|tile| crate::grass::grass_shape(&tile.source).is_some()) {
                        grass_cells.insert(cell);
                    }
                }
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
            grass_cells,
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

#[cfg(test)]
mod staging_tests {
    use super::*;

    #[test]
    fn onix_animation_requires_real_room_beyond_the_narrow_corridor() {
        let make_body = |species, meters| {
            let rig = crate::species_rig::for_species(species).unwrap();
            let (min, max) = rig.neutral_bounds;
            let scale = crate::battle_layout::model_scale(species, Some(meters), min, max).unwrap();
            let mut body = BattleBody::modeled(species, min, max, scale);
            body.visual_bounds = Some(rig.animated_bounds);
            body
        };
        let bodies = [Some(make_body("CYNDAQUIL", 0.508)), Some(make_body("ONIX", 8.7884))];
        let original = [Vec3::new(0.0, 0.0, 2.0), Vec3::new(8.0, 0.0, 2.0)];
        let floor = |rows: std::ops::Range<i32>| WalkingGround {
            origin: Vec2::ZERO,
            cells: (-20..=20).flat_map(|x| rows.clone().map(move |z|
                (IVec2::new(x, z), 0.0))).collect(),
            grass_cells: Default::default(),
        };
        let fits = |ground: &WalkingGround, anchors| {
            BattleSceneLayout::for_map_bodies(bodies, anchors, Vec2::new(1280.0, 720.0))
                .is_some_and(|layout| (0..2).all(|index|
                    ground.contains(bodies[index].unwrap(), layout.body_poses[index].unwrap(), anchors[index].y)))
        };
        let room = floor(-20..21);
        assert!(fits(&room, original), "the complete authored animation fits real room-sized support");
        let corridor = floor(0..4);
        assert!(!corridor.nearby_stage_pairs(original, bodies).into_iter().any(|anchors| fits(&corridor, anchors)),
            "finer staging must not pretend the real Onix animation fits an eight-unit corridor");
        let onix = bodies[1].unwrap();
        let (min, max) = onix.visual_bounds.unwrap();
        println!("Onix world animation extent: {:?}; neutral extent: {:?}",
            (max - min) * onix.scale, (onix.max - onix.min) * onix.scale);
    }

    #[test]
    fn large_bodies_can_center_on_a_two_row_floor_without_extending_it() {
        let ground = WalkingGround {
            origin: Vec2::ZERO,
            cells: (-12..=12).flat_map(|x| (0..4).map(move |z|
                (IVec2::new(x, z), 0.0))).collect(),
            grass_cells: Default::default(),
        };
        let original = [Vec3::new(0.0, 0.0, 2.0), Vec3::new(8.0, 0.0, 2.0)];
        let body = BattleBody::modeled("TEST", Vec3::new(-3.75, 0.0, -3.75),
            Vec3::new(3.75, 1.0, 3.75), 1.0);
        let bodies = [Some(body); 2];
        let fits = |anchors| {
            BattleSceneLayout::for_map_bodies(bodies, anchors, Vec2::new(1280.0, 720.0))
                .is_some_and(|layout| (0..2).all(|index|
                    ground.contains(body, layout.body_poses[index].unwrap(), anchors[index].y)))
        };
        assert!(!ground.nearby_stage_pairs(original, [None; 2]).into_iter().any(fits),
            "tile centers miss the actual center of the eight-unit-wide band");
        let pairs = ground.nearby_stage_pairs(original, bodies);
        assert!(pairs.len() <= 3528);
        assert!(pairs.into_iter().any(fits),
            "half-tile placement must fit the same complete body on the existing floor");
        let outside = Transform::from_xyz(0.0, 0.0, 2.0);
        assert!(!ground.contains(body, outside, 0.0),
            "finer staging must not approve the absent floor beyond the band");
        let small = [Some(BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5),
            Vec3::new(0.5, 1.0, 0.5), 1.0)); 2];
        assert!(ground.nearby_stage_pairs(original, small).len() <= 968,
            "small bodies retain the original search budget");
    }
}
