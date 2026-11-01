// Observe one already committed ordinary water encounter step. No runtime command,
// movement mutation, RNG/divider read, or source animation-clock change.
#[derive(Debug, Clone)]
struct VisibleBoundSurfEncounter {
    origin: Arc<BattlePresentationOrigin>,
    from: TilePosition,
    map_size: UVec2,
    placement: crystal_render_api::VisualBattleDerivedSurfPlacement,
    minimum_snapshot_revision: u64,
    warm_checked: bool,
    capture_attempted: bool,
    anchors: Option<Arc<crystal_render_api::VisualBattleAnchorFrame>>,
    publication: Option<Arc<crystal_render_api::VisualBattleLocation>>,
}

fn surf_plain_support(permission: u8) -> bool {
    // Ordinary original water only. Currents, waterfalls, whirlpools and every
    // other special collision remain unsupported, even if traversable by Surf.
    permission == crate::core::world::collision::permissions::WATER
}

fn surf_origin_step(origin: &BattlePresentationOrigin) -> Option<(TilePosition, TilePosition)> {
    if origin.kind != BattleOriginKind::Wild
        || origin.source.mode != MovementMode::Surf
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
        || *surface != crate::core::world::encounters::EncounterSurface::Water
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

fn bind_visible_surf_encounter(shell: &mut BevyRuntimeShell, frame: &crate::RuntimeOverworldFrame) {
    let Some(origin) = shell.battle_origin.pending.as_ref() else {
        return;
    };
    if shell.battle_origin.bound_surf.is_some()
        || origin.source != frame.snapshot
        || origin.authoritative_step != frame.movement
        || shell.pending_surf_start_from.is_some()
        || origin.kind != BattleOriginKind::Wild
        || origin.source.mode != MovementMode::Surf
        || origin.battle_type != "BATTLETYPE_NORMAL"
        || frame.warp.is_some()
        || frame.connection.is_some()
        || frame.coord_event.is_some()
        || frame.trainer_sight.is_some()
        || frame.phone_call.is_some()
        || frame.interaction.is_some()
        || frame.ledge_jump.is_some()
        || origin.visual_step.ledge_jump.is_some()
    {
        return;
    }
    let Some((from, to)) = surf_origin_step(origin) else {
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
    use crate::core::world::collision::sample_collision;
    // Require gameplay's checked Water surface and plain water on BOTH ends.
    // In particular, the scripted slow_step that mounts from land cannot bind.
    if !matches!(
        shell.shell.current_encounter_surface_checked(),
        Ok(Some(
            crate::core::world::encounters::EncounterSurface::Water
        ))
    ) || ![from, to].into_iter().all(|tile| {
        sample_collision(&overworld.map, &overworld.tileset, tile)
            .is_some_and(|sample| surf_plain_support(sample.permission))
    }) {
        return;
    }
    let Ok(occupied) = overworld.occupied_tiles_checked() else {
        return;
    };
    let core = IVec2::new(i32::from(to.x), i32::from(to.y));
    let map_size = UVec2::new(u32::from(width), u32::from(height));
    let mut water = Vec::new();
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
                    .is_some_and(|sample| surf_plain_support(sample.permission))
            {
                water.push(p);
            }
        }
    }
    // The player's complete body needs source water approval too; a runtime
    // actor occupying the landed cell cannot be ignored by choosing a corridor.
    if !water.contains(&core) {
        return;
    }
    let (dx, dy) = origin.source.facing.delta();
    let Some(target) =
        walking_presentation_tile(core, IVec2::new(i32::from(dx), i32::from(dy)), &water)
    else {
        return;
    };
    shell.battle_origin.bound_surf = Some(VisibleBoundSurfEncounter {
        origin: origin.clone(),
        from,
        map_size,
        placement: crystal_render_api::VisualBattleDerivedSurfPlacement {
            step_from_core_tile: IVec2::new(i32::from(from.x), i32::from(from.y)),
            witnessed_player_foot: None,
            presentation_core_tile: target,
            water_core_tiles: water.into(),
        },
        minimum_snapshot_revision: shell.snapshot_revision,
        warm_checked: false,
        capture_attempted: false,
        anchors: None,
        publication: None,
    });
}

fn publish_visible_surf_location(
    shell: &mut BevyRuntimeShell,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> Option<Arc<crystal_render_api::VisualBattleLocation>> {
    let valid = shell
        .battle_origin
        .bound_surf
        .as_ref()
        .is_some_and(|bound| {
            shell
                .battle_origin
                .published()
                .is_some_and(|origin| Arc::ptr_eq(origin, &bound.origin))
        });
    if !valid {
        shell.battle_origin.bound_surf = None;
        return None;
    }
    let current = shell.shell.session().snapshot();
    let entry_window = matches!(
        shell.pending_overworld_step_boundary,
        Some(PendingOverworldStepBoundary::WildBattle)
    ) || shell
        .visible_battle_transition
        .is_some_and(|transition| transition.frame < 3);
    let bound = shell.battle_origin.bound_surf.as_mut()?;
    if let Some(location) = &bound.publication {
        return Some(location.clone());
    }
    if !same_static_encounter_pose(&bound.origin.source, &current) {
        shell.battle_origin.bound_surf = None;
        return None;
    }
    freeze_visible_surf_anchors(bound, rendered, frame, entry_window);
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
            movement: crystal_render_api::VisualBattleSourceMovement::Surf,
        },
        target: crystal_render_api::VisualBattleTarget::SurfWater {
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
fn freeze_visible_surf_anchors(
    bound: &mut VisibleBoundSurfEncounter,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
    entry_window: bool,
) {
    let Some(scene) = freeze_visible_step_source_frame(
        &bound.origin.source,
        bound.from,
        bound.map_size,
        bound.minimum_snapshot_revision,
        &mut bound.warm_checked,
        &mut bound.capture_attempted,
        Some("surf"),
        rendered,
        frame,
        entry_window,
    ) else {
        return;
    };
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
fn freeze_visible_surf_anchors(
    bound: &mut VisibleBoundSurfEncounter,
    _rendered: &RenderedViewport,
    _frame: Option<&crystal_render_api::VisualWorldFrame>,
    _entry_window: bool,
) {
    bound.capture_attempted = true;
}
