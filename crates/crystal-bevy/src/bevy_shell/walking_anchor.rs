// Observe one already committed ordinary grass step. No runtime command,
// movement mutation, RNG/divider read, or source animation-clock change.
#[derive(Debug, Clone)]
struct VisibleBoundWalkingEncounter {
    origin: Arc<BattlePresentationOrigin>,
    from: TilePosition,
    map_size: UVec2,
    placement: crystal_render_api::VisualBattleDerivedGrassPlacement,
    minimum_snapshot_revision: u64,
    warm_checked: bool,
    capture_attempted: bool,
    anchors: Option<Arc<crystal_render_api::VisualBattleAnchorFrame>>,
    publication: Option<Arc<crystal_render_api::VisualBattleLocation>>,
}

fn walking_plain_support(permission: u8) -> bool {
    use crate::core::world::collision::{
        Terrain, describe_collision, is_grass_encounter_permission, permissions,
    };
    describe_collision(permission).terrain == Terrain::Land
        && (permission == permissions::FLOOR || is_grass_encounter_permission(permission))
}

fn walking_presentation_tile(source: IVec2, facing: IVec2, walkable: &[IVec2]) -> Option<IVec2> {
    // Source coordinates have +Y south. Fixed facing/right/left/back order;
    // no RNG and no camera-dependent choice. Both corridor cells must be real
    // source-map, unoccupied plain land. Body-sized clearance is a later gate.
    let right = IVec2::new(-facing.y, facing.x);
    [facing, right, -right, -facing]
        .into_iter()
        .find_map(|direction| {
            let middle = source + direction;
            let target = middle + direction;
            (walkable.contains(&middle) && walkable.contains(&target)).then_some(target)
        })
}

fn walking_origin_step(origin: &BattlePresentationOrigin) -> Option<(TilePosition, TilePosition)> {
    if origin.kind != BattleOriginKind::Wild
        || origin.source.mode != MovementMode::Normal
        || origin.battle_type != "BATTLETYPE_NORMAL"
        || origin.visual_step.ledge_jump.is_some()
    {
        return None;
    }
    let Some(BattleOriginContact::OverworldEncounter {
        map_id,
        tile,
        surface,
    }) = &origin.contact
    else {
        return None;
    };
    if map_id != &origin.source.map_name
        || *tile != origin.source.tile
        || *surface != crate::core::world::encounters::EncounterSurface::Grass
    {
        return None;
    }
    let Some(StepOutcome::Moved {
        from,
        to,
        speed_multiplier: 1,
    }) = &origin.authoritative_step
    else {
        return None;
    };
    if *to != origin.source.tile
        || crate::core::world::movement::checked_move_by_stride(*from, origin.source.facing, 1)
            != Some(*to)
    {
        return None;
    }
    Some((*from, *to))
}

fn bind_visible_walking_encounter(
    shell: &mut BevyRuntimeShell,
    frame: &crate::RuntimeOverworldFrame,
) {
    let Some(origin) = shell.battle_origin.pending.as_ref() else {
        return;
    };
    if shell.battle_origin.bound_walking.is_some()
        || origin.source != frame.snapshot
        || origin.kind != BattleOriginKind::Wild
        || origin.source.mode != MovementMode::Normal
        || origin.battle_type != "BATTLETYPE_NORMAL"
        || frame.warp.is_some()
        || frame.connection.is_some()
        || frame.coord_event.is_some()
        || frame.trainer_sight.is_some()
        || frame.phone_call.is_some()
        || frame.ledge_jump.is_some()
        || origin.visual_step.ledge_jump.is_some()
    {
        return;
    }
    let Some((from, to)) = walking_origin_step(origin) else {
        return;
    };
    let overworld = shell.shell.session().overworld();
    if overworld.snapshot() != origin.source {
        return;
    }
    let (width, height) = match overworld.map.checked_tile_bounds() {
        Some((width, height)) if width > 0 && height > 0 => (width, height),
        _ => return,
    };
    use crate::core::world::collision::{is_grass_encounter_permission, sample_collision};
    if !sample_collision(&overworld.map, &overworld.tileset, to).is_some_and(|sample| {
        walking_plain_support(sample.permission) && is_grass_encounter_permission(sample.permission)
    }) {
        return;
    }
    let Ok(occupied) = overworld.occupied_tiles_checked() else {
        return;
    };
    let core = IVec2::new(i32::from(to.x), i32::from(to.y));
    let map_size = UVec2::new(u32::from(width), u32::from(height));
    let mut walkable = Vec::new();
    // Fixed 7x7 budget; never scan or copy a whole map on a render tick.
    for y in -3..=3 {
        for x in -3..=3 {
            let p = core + IVec2::new(x, y);
            if p.cmplt(IVec2::ZERO).any() || p.cmpge(map_size.as_ivec2()).any() {
                continue;
            }
            let tile = TilePosition::new(p.x as i16, p.y as i16);
            if !occupied.iter().any(|entry| entry.tile == tile)
                && sample_collision(&overworld.map, &overworld.tileset, tile)
                    .is_some_and(|sample| walking_plain_support(sample.permission))
            {
                walkable.push(p);
            }
        }
    }
    let (dx, dy) = origin.source.facing.delta();
    let Some(target) =
        walking_presentation_tile(core, IVec2::new(i32::from(dx), i32::from(dy)), &walkable)
    else {
        return;
    };
    shell.battle_origin.bound_walking = Some(VisibleBoundWalkingEncounter {
        origin: origin.clone(),
        from,
        map_size,
        placement: crystal_render_api::VisualBattleDerivedGrassPlacement {
            step_from_core_tile: IVec2::new(i32::from(from.x), i32::from(from.y)),
            witnessed_player_foot: None,
            presentation_core_tile: target,
            walkable_core_tiles: walkable.into(),
        },
        minimum_snapshot_revision: shell.snapshot_revision,
        warm_checked: false,
        capture_attempted: false,
        anchors: None,
        publication: None,
    });
}

fn walking_foot_is_on_original_step(foot: Vec2, from: Vec2, to: Vec2) -> bool {
    if !foot.is_finite() || !from.is_finite() || !to.is_finite() {
        return false;
    }
    let delta = to - from;
    let length = delta.length_squared();
    if length <= 0.0 {
        return false;
    }
    let progress = (foot - from).dot(delta) / length;
    (-0.0001..=1.0001).contains(&progress)
        && (foot - (from + delta * progress)).length_squared() <= 0.0001
}

fn walking_snapshot_is_current(rendered: Option<u64>, minimum: u64) -> bool {
    rendered.is_some_and(|revision| revision.wrapping_sub(minimum) < (1_u64 << 63))
}

fn publish_visible_walking_location(
    shell: &mut BevyRuntimeShell,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> Option<Arc<crystal_render_api::VisualBattleLocation>> {
    let valid = shell
        .battle_origin
        .bound_walking
        .as_ref()
        .is_some_and(|bound| {
            shell
                .battle_origin
                .published()
                .is_some_and(|origin| Arc::ptr_eq(origin, &bound.origin))
        });
    if !valid {
        shell.battle_origin.bound_walking = None;
        return None;
    }
    let current = shell.shell.session().snapshot();
    let entry_window = matches!(
        shell.pending_overworld_step_boundary,
        Some(PendingOverworldStepBoundary::WildBattle)
    ) || shell
        .visible_battle_transition
        .is_some_and(|transition| transition.frame < 3);
    let bound = shell.battle_origin.bound_walking.as_mut()?;
    if let Some(location) = &bound.publication {
        return Some(location.clone());
    }
    if !same_static_encounter_pose(&bound.origin.source, &current) {
        shell.battle_origin.bound_walking = None;
        return None;
    }
    let attempted_before = bound.capture_attempted;
    freeze_visible_walking_anchors(bound, rendered, frame, entry_window);
    #[cfg(not(target_arch = "wasm32"))]
    if !attempted_before
        && bound.capture_attempted
        && std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some()
    {
        eprintln!(
            "walking witness: generation={} anchored={} entry_window={} from={:?} landed={:?} facing={:?} min_revision={} rendered_tile={:?} rendered_revision={:?} rendered_facing={:?} rendered_mode={:?} rendered_origin={:?} rendered_visual_revision={:?}",
            bound.origin.generation,
            bound.anchors.is_some(),
            entry_window,
            bound.from,
            bound.origin.source.tile,
            bound.origin.source.facing,
            bound.minimum_snapshot_revision,
            rendered.tile,
            rendered.snapshot_revision,
            rendered.player_sprite_facing,
            rendered.player_sprite_mode,
            rendered.viewport_origin,
            rendered.visual_tiles_revision
        );
        if let Some(frame) = frame {
            eprintln!(
                "walking witness frame: active={} valid={:?} map={} revision={} origin={:?} grid={:?} center={:?} tile_size={:?} viewport={:?} texture={:?} player={:?}",
                frame.active,
                frame.validate(),
                frame.map_id,
                frame.terrain_revision,
                frame.grid_origin,
                frame.grid_size,
                frame.center,
                frame.tile_size,
                frame.viewport_size,
                frame.map_texture,
                frame
                    .actors
                    .iter()
                    .find(|actor| actor.id == crystal_render_api::VisualActorId::Player)
                    .map(|a| (a.center, a.size, a.facing))
            );
        }
    }
    #[cfg(target_arch = "wasm32")]
    let _ = attempted_before;
    if !bound.capture_attempted {
        return None;
    }
    let source = &bound.origin.source;
    let (dx, dy) = source.facing.delta();
    let location = Arc::new(crystal_render_api::VisualBattleLocation {
        generation: bound.origin.generation,
        source: crystal_render_api::VisualBattleSourcePose {
            map_id: Arc::from(source.map_name.as_str()),
            source_frame: source.frame,
            core_tile: IVec2::new(i32::from(source.tile.x), i32::from(source.tile.y)),
            facing: IVec2::new(i32::from(dx), i32::from(dy)),
            movement: crystal_render_api::VisualBattleSourceMovement::Normal,
        },
        target: crystal_render_api::VisualBattleTarget::WalkingGrass {
            core_tile: IVec2::new(i32::from(source.tile.x), i32::from(source.tile.y)),
            presentation: bound.placement.clone(),
        },
        source_map_size_core_tiles: Some(bound.map_size),
        anchors: bound.anchors.clone(),
    });
    bound.publication = Some(location.clone());
    Some(location)
}

#[cfg(any(test, feature = "voxel-view"))]
fn freeze_visible_walking_anchors(
    bound: &mut VisibleBoundWalkingEncounter,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
    entry_window: bool,
) {
    if bound.capture_attempted {
        return;
    }
    let source = &bound.origin.source;
    if !bound.warm_checked {
        bound.warm_checked = true;
        // An initially cold or unrelated renderer cannot be rehabilitated by
        // an arbitrary later same-map frame after encounter commitment.
        if !frame.is_some_and(|frame| {
            frame.active
                && frame.validate().is_ok()
                && frame.map_id.as_ref() == source.map_name
                && frame.source_map_size_core_tiles == Some(bound.map_size)
        }) || rendered.map_name.as_deref() != Some(source.map_name.as_str())
            || ![Some(bound.from), Some(source.tile)].contains(&rendered.tile)
        {
            bound.capture_attempted = true;
            return;
        }
    }
    if !entry_window {
        bound.capture_attempted = true;
        return;
    }
    if rendered.tile == Some(bound.from) {
        // An exact settled pre-step sprite is already a sufficient witness.
        // Its expected tile comes from this committed StepOutcome.from. The
        // authoritative origin is unchanged, and both eventual support points
        // are resolved inside this genuinely rendered grid. A prior step still
        // interpolating toward `from` cannot claim this branch.
        let mut previous = source.clone();
        previous.tile = bound.from;
        if let Some(scene) =
            capture_visible_fishing_source_frame(&previous, Some(bound.map_size), rendered, frame)
        {
            finish_visible_walking_anchors(bound, scene);
            return;
        }
        // Only a coherent preceding walk may wait for this step's frame.
        // A missing player or stale terrain/texture/grid is a failed witness,
        // not evidence that the sprite is still approaching `from`.
        let approaching_from = rendered.player_sprite_facing.and_then(|facing| {
            previous.facing = facing;
            let previous_from = crate::core::world::movement::checked_move_by_stride(
                bound.from,
                visible_opposite_direction(facing),
                1,
            )?;
            capture_visible_walking_source_frame(
                &previous,
                previous_from,
                bound.map_size,
                rendered,
                frame,
            )
        });
        if approaching_from.is_none() {
            bound.capture_attempted = true;
        }
        return;
    }
    // The destination must belong to this committed snapshot. Wait for its
    // existing classic-world publication, never a desired cache key.
    if rendered.tile != Some(source.tile)
        || !walking_snapshot_is_current(rendered.snapshot_revision, bound.minimum_snapshot_revision)
    {
        return;
    }
    bound.capture_attempted = true;
    let Some(scene) =
        capture_visible_walking_source_frame(source, bound.from, bound.map_size, rendered, frame)
    else {
        return;
    };
    finish_visible_walking_anchors(bound, scene);
}

#[cfg(any(test, feature = "voxel-view"))]
fn finish_visible_walking_anchors(
    bound: &mut VisibleBoundWalkingEncounter,
    scene: VisibleFishingSourceFrame,
) {
    bound.capture_attempted = true;
    let target = bound.placement.presentation_core_tile;
    let target = TilePosition::new(target.x as i16, target.y as i16);
    let (Some(landed), Some(enemy)) = (
        fishing_source_support(&scene.terrain, bound.origin.source.tile),
        fishing_source_support(&scene.terrain, target),
    ) else {
        return;
    };
    bound.placement.witnessed_player_foot = Some(scene.source_foot);
    bound.anchors = Some(Arc::new(crystal_render_api::VisualBattleAnchorFrame {
        terrain: scene.terrain,
        source_actor: crystal_render_api::VisualActorId::Player,
        target_actor: None,
        source_foot: landed,
        target_foot: enemy,
    }));
}

#[cfg(not(any(test, feature = "voxel-view")))]
fn freeze_visible_walking_anchors(
    bound: &mut VisibleBoundWalkingEncounter,
    _rendered: &RenderedViewport,
    _frame: Option<&crystal_render_api::VisualWorldFrame>,
    _entry_window: bool,
) {
    bound.capture_attempted = true;
}

#[cfg(any(test, feature = "voxel-view"))]
fn capture_visible_walking_source_frame(
    source: &crate::core::world::session::OverworldSnapshot,
    from: TilePosition,
    map_size: UVec2,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> Option<VisibleFishingSourceFrame> {
    use crystal_render_api::{VisualActorId, VisualBattleTerrainEvidence};
    let Some(frame) = frame.filter(|frame| frame.active && frame.validate().is_ok()) else {
        return None;
    };
    if frame.map_id.as_ref() != source.map_name
        || frame.source_map_size_core_tiles != Some(map_size)
        || rendered.source_map_size_core_tiles != Some(map_size)
        || rendered.map_name.as_deref() != Some(source.map_name.as_str())
        || rendered.tile != Some(source.tile)
        || rendered.player_sprite_facing != Some(source.facing)
        || rendered.player_sprite_mode != Some(source.mode)
        || rendered.visual_tiles_revision != Some(frame.terrain_revision)
    {
        return None;
    }
    let Some((start_x, start_y)) = rendered.viewport_origin else {
        return None;
    };
    #[cfg(feature = "voxel-view")]
    let (expected_texture, expected_grid) = (
        rendered.visual_world_texture.as_ref(),
        rendered.visual_world_grid_size,
    );
    #[cfg(not(feature = "voxel-view"))]
    let (expected_texture, expected_grid) = (
        rendered.map_texture.as_ref(),
        UVec2::new(VISUAL_WORLD_TILES_X as u32, VISUAL_WORLD_TILES_Y as u32),
    );
    let expected_origin = IVec2::new(i32::from(start_x), i32::from(start_y))
        - (expected_grid.as_ivec2() - IVec2::new(VIEWPORT_TILES_X as i32, VIEWPORT_TILES_Y as i32))
            / 2;
    if expected_texture != Some(&frame.map_texture)
        || frame.grid_size != expected_grid
        || frame.grid_origin != expected_origin
        || frame.tile_size != Vec2::splat(TILE_SIZE)
        || frame.viewport_size != Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)
    {
        return None;
    }
    let Some(player) = frame
        .actors
        .iter()
        .find(|actor| actor.id == VisualActorId::Player)
    else {
        return None;
    };
    let terrain = VisualBattleTerrainEvidence {
        map_id: frame.map_id.clone(),
        source_map_size_core_tiles: frame.source_map_size_core_tiles,
        terrain_revision: frame.terrain_revision,
        grid_origin: frame.grid_origin,
        grid_size: frame.grid_size,
        center: frame.center,
        viewport_size: frame.viewport_size,
        tile_size: frame.tile_size,
        map_texture: frame.map_texture.clone(),
        tiles: frame.tiles.clone().into(),
    };
    let from_foot = fishing_source_support(&terrain, from)?;
    let to_foot = fishing_source_support(&terrain, source.tile)?;
    let source_foot = player.center - Vec2::Y * player.size.y * 0.5;
    if !walking_foot_is_on_original_step(source_foot, from_foot, to_foot)
        || player.facing != Some(visual_facing(source.facing))
    {
        return None;
    }
    Some(VisibleFishingSourceFrame {
        terrain,
        source_foot,
    })
}
