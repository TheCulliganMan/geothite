// Presentation-only binding of an already committed, checked fishing cast.
// This never issues a runtime command or changes source animation clocks.
#[derive(Debug)]
struct VisibleFishingSourceFrame {
    terrain: crystal_render_api::VisualBattleTerrainEvidence,
    source_foot: Vec2,
}

#[derive(Debug)]
struct VisibleFishingPackScene {
    source: crate::core::world::session::OverworldSnapshot,
    map_size: Option<UVec2>,
    capture_attempted: bool,
    scene: Option<Arc<VisibleFishingSourceFrame>>,
}

fn stage_visible_fishing_pack_scene(shell: &mut BevyRuntimeShell, snapshot: &RuntimeShellSnapshot) {
    shell.battle_origin.fishing_pack_scene = None;
    let rods = shell.shell.fishing_rod_ids();
    if snapshot.battle.is_some()
        || !snapshot
            .bag
            .key_items
            .iter()
            .any(|item| item.quantity > 0 && rods.contains(&item.item_id))
    {
        return;
    }
    let map = &shell.shell.session().overworld().map;
    let map_size = (map.name == snapshot.overworld.map_name && map.width > 0 && map.height > 0)
        .then(|| map.checked_tile_bounds())
        .flatten()
        .map(|(width, height)| UVec2::new(u32::from(width), u32::from(height)));
    shell.battle_origin.fishing_pack_scene = Some(VisibleFishingPackScene {
        source: snapshot.overworld.clone(),
        map_size,
        capture_attempted: false,
        scene: None,
    });
}

/// Pack owns a fullscreen LCD. Capture only its opening boundary, before that
/// presenter hides the world. This private source evidence is not an encounter.
fn capture_visible_fishing_pack_scene(
    shell: &mut BevyRuntimeShell,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) {
    if !visible_field_pack_is_open(shell) {
        shell.battle_origin.fishing_pack_scene = None;
        return;
    }
    let current = shell.shell.session().snapshot();
    let Some(pack) = shell.battle_origin.fishing_pack_scene.as_mut() else {
        return;
    };
    if !same_static_encounter_pose(&pack.source, &current) {
        pack.capture_attempted = true;
        pack.scene = None;
        return;
    }
    if pack.scene.as_ref().is_some_and(|scene| {
        rendered.map_name.as_deref() != Some(pack.source.map_name.as_str())
            || rendered.source_map_size_core_tiles != pack.map_size
            || rendered.visual_tiles_revision != Some(scene.terrain.terrain_revision)
    }) {
        pack.scene = None;
    }
    if pack.capture_attempted {
        return;
    }
    pack.capture_attempted = true;
    pack.scene = capture_visible_fishing_source_frame(&pack.source, pack.map_size, rendered, frame)
        .map(Arc::new);
}
#[derive(Debug)]
struct VisibleBoundFishingEncounter {
    origin: Arc<BattlePresentationOrigin>,
    water_tile: TilePosition,
    map_size: Option<UVec2>,
    capture_attempted: bool,
    anchors: Option<Arc<crystal_render_api::VisualBattleAnchorFrame>>,
    publication: Option<Arc<crystal_render_api::VisualBattleLocation>>,
}

fn bind_visible_fishing_encounter(
    shell: &mut BevyRuntimeShell,
    source: &crate::core::world::session::OverworldSnapshot,
    water_tile: TilePosition,
) {
    let Some(origin) = shell.battle_origin.pending.as_ref() else {
        return;
    };
    if origin.source != *source
        || origin.kind != BattleOriginKind::Wild
        || origin.contact.as_ref()
            != Some(&BattleOriginContact::FishingWaterTarget {
                map_id: source.map_name.clone(),
                tile: water_tile,
            })
        || crate::core::world::movement::checked_move_by_stride(
            source.tile,
            source.facing,
            crate::core::world::movement::StepOptions::default().stride_tiles,
        ) != Some(water_tile)
    {
        return;
    }
    let map = &shell.shell.session().overworld().map;
    let map_size = (map.name == source.map_name && map.width > 0 && map.height > 0)
        .then(|| map.checked_tile_bounds())
        .flatten()
        .map(|(width, height)| UVec2::new(u32::from(width), u32::from(height)));
    let origin = origin.clone();
    let pack = shell.battle_origin.fishing_pack_scene.take();
    let capture_attempted = pack.is_some();
    let anchors = pack
        .as_ref()
        .filter(|pack| {
            pack.capture_attempted
                && pack.map_size == map_size
                && same_static_encounter_pose(&pack.source, source)
        })
        .and_then(|pack| pack.scene.as_deref())
        .and_then(|scene| fishing_anchors_from_source(scene, water_tile));
    shell.battle_origin.bound_fishing = Some(VisibleBoundFishingEncounter {
        origin,
        water_tile,
        map_size,
        capture_attempted,
        anchors,
        publication: None,
    });
}

/// Capture the preceding completed scene once, before fishing changes sprites.
fn freeze_visible_fishing_encounter_anchors(
    bound: &mut VisibleBoundFishingEncounter,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) {
    if bound.capture_attempted {
        return;
    }
    bound.capture_attempted = true;
    bound.anchors =
        capture_visible_fishing_source_frame(&bound.origin.source, bound.map_size, rendered, frame)
            .and_then(|scene| fishing_anchors_from_source(&scene, bound.water_tile));
}

#[cfg(any(test, feature = "voxel-view"))]
fn capture_visible_fishing_source_frame(
    source: &crate::core::world::session::OverworldSnapshot,
    map_size: Option<UVec2>,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> Option<VisibleFishingSourceFrame> {
    use crystal_render_api::{VisualActorId, VisualBattleTerrainEvidence};
    let Some(frame) = frame.filter(|frame| frame.active && frame.validate().is_ok()) else {
        return None;
    };
    let Some(map_size) = map_size else {
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
    let source_foot = fishing_source_support(&terrain, source.tile)?;
    if (player.center - Vec2::Y * player.size.y * 0.5 - source_foot).length_squared() > 0.0001
        || player.facing != Some(visual_facing(source.facing))
    {
        return None;
    }
    Some(VisibleFishingSourceFrame {
        terrain,
        source_foot,
    })
}

#[cfg(not(any(test, feature = "voxel-view")))]
fn capture_visible_fishing_source_frame(
    _source: &crate::core::world::session::OverworldSnapshot,
    _map_size: Option<UVec2>,
    _rendered: &RenderedViewport,
    _frame: Option<&crystal_render_api::VisualWorldFrame>,
) -> Option<VisibleFishingSourceFrame> {
    None
}

fn fishing_source_support(
    terrain: &crystal_render_api::VisualBattleTerrainEvidence,
    tile: TilePosition,
) -> Option<Vec2> {
    let map_size = terrain.source_map_size_core_tiles?;
    let core = IVec2::new(i32::from(tile.x), i32::from(tile.y));
    let cell = core * 2 - terrain.grid_origin;
    // Original map acreage and complete published cells are both mandatory.
    if core.cmplt(IVec2::ZERO).any()
        || core.as_uvec2().cmpge(map_size).any()
        || cell.cmplt(IVec2::ZERO).any()
        || (cell + IVec2::ONE)
            .cmpge(terrain.grid_size.as_ivec2())
            .any()
    {
        return None;
    }
    let northwest = terrain.center
        + Vec2::new(
            -(terrain.grid_size.x as f32) * terrain.tile_size.x * 0.5,
            terrain.grid_size.y as f32 * terrain.tile_size.y * 0.5,
        );
    Some(
        northwest
            + Vec2::new(
                (cell.x + 1) as f32 * terrain.tile_size.x,
                -(cell.y + 2) as f32 * terrain.tile_size.y,
            ),
    )
}

fn fishing_anchors_from_source(
    scene: &VisibleFishingSourceFrame,
    checked_water: TilePosition,
) -> Option<Arc<crystal_render_api::VisualBattleAnchorFrame>> {
    Some(Arc::new(crystal_render_api::VisualBattleAnchorFrame {
        terrain: scene.terrain.clone(),
        source_actor: crystal_render_api::VisualActorId::Player,
        target_actor: None,
        source_foot: scene.source_foot,
        target_foot: fishing_source_support(&scene.terrain, checked_water)?,
    }))
}

fn fishing_encounter_location(
    bound: &mut VisibleBoundFishingEncounter,
) -> Arc<crystal_render_api::VisualBattleLocation> {
    use crystal_render_api::{
        VisualBattleLocation, VisualBattleSourceMovement as Mode, VisualBattleSourcePose,
        VisualBattleTarget,
    };
    bound
        .publication
        .get_or_insert_with(|| {
            let source = &bound.origin.source;
            Arc::new(VisualBattleLocation {
                generation: bound.origin.generation,
                source: VisualBattleSourcePose {
                    map_id: Arc::from(source.map_name.as_str()),
                    source_frame: source.frame,
                    core_tile: IVec2::new(i32::from(source.tile.x), i32::from(source.tile.y)),
                    facing: match source.facing {
                        Direction::Up => IVec2::NEG_Y,
                        Direction::Right => IVec2::X,
                        Direction::Down => IVec2::Y,
                        Direction::Left => IVec2::NEG_X,
                    },
                    movement: match source.mode {
                        MovementMode::Normal => Mode::Normal,
                        MovementMode::Bike => Mode::Bike,
                        MovementMode::Skate => Mode::Skate,
                        MovementMode::Surf => Mode::Surf,
                        MovementMode::SurfPika => Mode::SurfPika,
                    },
                },
                target: VisualBattleTarget::FishingWater {
                    core_tile: IVec2::new(
                        i32::from(bound.water_tile.x),
                        i32::from(bound.water_tile.y),
                    ),
                },
                source_map_size_core_tiles: bound.map_size,
                anchors: bound.anchors.clone(),
            })
        })
        .clone()
}
