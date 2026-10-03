// Presentation-only evidence for checked SquirtBottle entries. The controller
// observes existing results; no added runtime command, RNG read, or save field.
#[derive(Debug, Clone)]
struct VisibleStaticEncounterCandidate {
    source: crate::core::world::session::OverworldSnapshot,
    target_tile: TilePosition,
    target_identifier: String,
    target_movement: String,
    object_script: String,
    trigger_script: String,
    expected_step: Option<RuntimeCompiledScriptCursor>,
    witnessed_start: Option<RuntimeCompiledScriptCursor>,
    map_size: Option<UVec2>,
    capture_attempted: bool,
    anchors: Option<Arc<crystal_render_api::VisualBattleAnchorFrame>>,
}

#[derive(Debug)]
struct VisibleBoundStaticEncounter {
    generation: u64,
    candidate: VisibleStaticEncounterCandidate,
    publication: Option<Arc<crystal_render_api::VisualBattleLocation>>,
}

fn same_static_encounter_pose(
    source: &crate::core::world::session::OverworldSnapshot,
    current: &crate::core::world::session::OverworldSnapshot,
) -> bool {
    source.map_name == current.map_name
        && source.tile == current.tile
        && source.facing == current.facing
        && source.mode == current.mode
        && current.frame >= source.frame
}

fn checked_static_target_present(
    snapshot: &RuntimeShellSnapshot,
    identifier: &str,
    tile: TilePosition,
    movement: &str,
    script: &str,
) -> bool {
    let mut matches = snapshot
        .visible_objects
        .iter()
        .filter(|object| object.object_identifier.as_deref() == Some(identifier));
    let Some(object) = matches.next() else {
        return false;
    };
    matches.next().is_none()
        && object.spritemovedata == movement
        && object.script == script
        && snapshot
            .visible_object_runtime_tiles
            .get(identifier)
            .copied()
            .or_else(|| object_tile_position_checked(object))
            == Some(tile)
}

fn stage_visible_squirtbottle_candidate(
    shell: &mut BevyRuntimeShell,
    before: &RuntimeShellSnapshot,
    checked: &crate::RuntimeSquirtBottleUse,
) {
    shell.battle_origin.static_candidate = None;
    let (Some(identifier), Some(script)) = (
        checked.target_object_identifier.as_deref(),
        checked.target_script.as_deref(),
    ) else {
        return;
    };
    if identifier.is_empty() || script.is_empty() {
        return;
    }
    let source = &before.overworld;
    let Some(object_script) = before
        .visible_objects
        .iter()
        .find(|object| object.object_identifier.as_deref() == Some(identifier))
        .map(|object| object.script.clone())
    else {
        return;
    };
    let dispatched = shell
        .shell
        .session()
        .state()
        .script_runtime
        .next_script
        .as_ref();
    if !dispatched
        .is_some_and(|next| next.origin_map_name == source.map_name && next.script == script)
        || shell
            .shell
            .session()
            .state()
            .script_runtime
            .last_talked_object
            .as_deref()
            != Some(identifier)
    {
        return;
    }
    let current = shell.shell.session().snapshot();
    if source.tile != checked.player_tile
        || !same_static_encounter_pose(source, &current)
        || !checked_static_target_present(
            before,
            identifier,
            checked.target_tile,
            &checked.target_movement,
            &object_script,
        )
    {
        return;
    }
    let map = &shell.shell.session().overworld().map;
    let map_size = (map.name == source.map_name && map.width > 0 && map.height > 0)
        .then(|| map.checked_tile_bounds())
        .flatten()
        .map(|(width, height)| UVec2::new(u32::from(width), u32::from(height)));
    shell.battle_origin.static_candidate = Some(VisibleStaticEncounterCandidate {
        source: source.clone(),
        target_tile: checked.target_tile,
        target_identifier: identifier.to_string(),
        target_movement: checked.target_movement.clone(),
        object_script,
        trigger_script: script.to_string(),
        expected_step: Some(RuntimeCompiledScriptCursor {
            origin_map_name: source.map_name.clone(),
            source_script: script.to_string(),
            command_index: 0,
        }),
        witnessed_start: None,
        map_size,
        capture_attempted: false,
        anchors: None,
    });
}

/// Observe only the interpreter's actual sequential continuation. Direct
/// starts, guessed reachable labels, and an unrelated same-map battle cannot
/// claim this candidate. One cursor bounds memory regardless of script length.
/// A callee end without an explicit next cursor is unavailable in this bounded
/// bridge; do not infer a caller or resolve a script graph to extend lineage.
fn observe_visible_static_encounter_step(
    shell: &mut BevyRuntimeShell,
    step: &crate::RuntimeCompiledScriptStep,
) {
    let Some(candidate) = shell.battle_origin.static_candidate.as_mut() else {
        return;
    };
    let observed = RuntimeCompiledScriptCursor {
        origin_map_name: step.origin_map_name.clone(),
        source_script: step.source_script.clone(),
        command_index: step.command_index,
    };
    if candidate.expected_step.as_ref() != Some(&observed)
        || !same_static_encounter_pose(&candidate.source, &shell.shell.session().snapshot())
    {
        shell.battle_origin.static_candidate = None;
        return;
    }
    if matches!(
        &step.mutation.result,
        RuntimeMutationResult::ScriptedWildBattleStarted(_)
    ) {
        candidate.witnessed_start = Some(observed);
        candidate.expected_step = None;
    } else if let Some(next) = &step.next_cursor {
        candidate.expected_step = Some(next.clone());
    } else {
        shell.battle_origin.static_candidate = None;
    }
}

fn take_matching_static_encounter_candidate(
    shell: &mut BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    battle: &crate::RuntimeBattleSnapshot,
) -> Option<VisibleStaticEncounterCandidate> {
    let candidate = shell.battle_origin.static_candidate.take()?;
    let crate::RuntimeBattleKind::StaticWild {
        origin_map_name,
        source_script,
        startbattle_command_index,
        ..
    } = &battle.kind
    else {
        return None;
    };
    let witness = candidate.witnessed_start.as_ref()?;
    (same_static_encounter_pose(&candidate.source, &snapshot.overworld)
        && &candidate.source.map_name == origin_map_name
        && &witness.origin_map_name == origin_map_name
        && &witness.source_script == source_script
        && witness.command_index == *startbattle_command_index
        && checked_static_target_present(
            snapshot,
            &candidate.target_identifier,
            candidate.target_tile,
            &candidate.target_movement,
            &candidate.object_script,
        ))
    .then_some(candidate)
}

/// Read the previous completed world publication before render_playfield can
/// clear it for battle/LCD entities. Try exactly once, including cold frames.
#[cfg(any(test, feature = "voxel-view"))]
fn freeze_visible_static_encounter_anchors(
    candidate: &mut VisibleStaticEncounterCandidate,
    rendered: &RenderedViewport,
    frame: Option<&crystal_render_api::VisualWorldFrame>,
    objects: &[(usize, Option<&str>, &str)],
) {
    use crystal_render_api::{VisualActorId, VisualBattleAnchorFrame, VisualBattleTerrainEvidence};
    if candidate.capture_attempted {
        return;
    }
    candidate.capture_attempted = true;
    let Some(frame) = frame.filter(|frame| frame.active && frame.validate().is_ok()) else {
        return;
    };
    if frame.map_id.as_ref() != candidate.source.map_name
        || frame.source_map_size_core_tiles != candidate.map_size
        || frame.source_map_size_core_tiles != rendered.source_map_size_core_tiles
        || rendered.map_name.as_deref() != Some(candidate.source.map_name.as_str())
        || rendered.tile != Some(candidate.source.tile)
        || rendered.player_sprite_facing != Some(candidate.source.facing)
        || rendered.player_sprite_mode != Some(candidate.source.mode)
        || rendered.visual_tiles_revision != Some(frame.terrain_revision)
    {
        return;
    }
    let Some((start_x, start_y)) = rendered.viewport_origin else {
        return;
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
        return;
    }
    let mut targets = objects
        .iter()
        .filter(|(_, identifier, _)| *identifier == Some(candidate.target_identifier.as_str()));
    let Some((index, _, source_id)) = targets.next() else {
        return;
    };
    if targets.next().is_some() || objects.iter().filter(|(slot, _, _)| slot == index).count() != 1
    {
        return;
    }
    let Ok(index) = u32::try_from(*index) else {
        return;
    };
    let target_id = VisualActorId::Object(index);
    let Some(source_actor) = frame
        .actors
        .iter()
        .find(|actor| actor.id == VisualActorId::Player)
    else {
        return;
    };
    let Some(target_actor) = frame.actors.iter().find(|actor| actor.id == target_id) else {
        return;
    };
    if target_actor.source_id.as_ref() != *source_id {
        return;
    }
    // Grid NW is in 8x8 source cells; core tiles are 16x16. Verify complete
    // sprite top-left correspondence rather than guessing actor feet from a
    // runtime tile, or confusing a visible roster index with a pack index.
    let northwest = frame.center
        + Vec2::new(
            -(frame.grid_size.x as f32) * frame.tile_size.x * 0.5,
            frame.grid_size.y as f32 * frame.tile_size.y * 0.5,
        );
    let expected_center = |tile: TilePosition, size: Vec2| {
        let cell = IVec2::new(i32::from(tile.x) * 2, i32::from(tile.y) * 2) - frame.grid_origin;
        northwest
            + Vec2::new(
                cell.x as f32 * frame.tile_size.x + size.x * 0.5,
                -cell.y as f32 * frame.tile_size.y - size.y * 0.5,
            )
    };
    if (source_actor.center - expected_center(candidate.source.tile, source_actor.size))
        .length_squared()
        > 0.0001
        || (target_actor.center - expected_center(candidate.target_tile, target_actor.size))
            .length_squared()
            > 0.0001
        || source_actor.facing != Some(visual_facing(candidate.source.facing))
    {
        return;
    }
    let foot =
        |actor: &crystal_render_api::VisualActor| actor.center - Vec2::Y * actor.size.y * 0.5;
    candidate.anchors = Some(Arc::new(VisualBattleAnchorFrame {
        terrain: VisualBattleTerrainEvidence {
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
        },
        source_actor: VisualActorId::Player,
        target_actor: target_id,
        source_foot: foot(source_actor),
        target_foot: foot(target_actor),
    }));
}

#[cfg(not(any(test, feature = "voxel-view")))]
fn freeze_visible_static_encounter_anchors(
    candidate: &mut VisibleStaticEncounterCandidate,
    _rendered: &RenderedViewport,
    _frame: Option<&crystal_render_api::VisualWorldFrame>,
    _objects: &[(usize, Option<&str>, &str)],
) {
    candidate.capture_attempted = true;
}

fn publish_visible_battle_location(
    mut shell: ResMut<BevyRuntimeShell>,
    rendered: Res<RenderedViewport>,
    world_frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    objects: Query<&VisibleObjectSprite>,
    mut published: ResMut<crystal_render_api::BattleLocationFrame>,
) {
    if shell.battle_origin.static_candidate.is_none() && shell.battle_origin.bound_static.is_none()
    {
        if published.location.is_some() {
            published.location = None;
        }
        return;
    }
    let current = shell.shell.session().snapshot();
    if shell
        .battle_origin
        .static_candidate
        .as_ref()
        .is_some_and(|candidate| !same_static_encounter_pose(&candidate.source, &current))
    {
        shell.battle_origin.static_candidate = None;
    }
    let needs_capture = shell
        .battle_origin
        .static_candidate
        .as_ref()
        .is_some_and(|candidate| !candidate.capture_attempted)
        || shell
            .battle_origin
            .bound_static
            .as_ref()
            .is_some_and(|bound| !bound.candidate.capture_attempted);
    if needs_capture {
        let objects: Vec<_> = objects
            .iter()
            .map(|object| {
                (
                    object.object_index,
                    object.object_identifier.as_deref(),
                    object.source_id.as_ref(),
                )
            })
            .collect();
        if let Some(candidate) = shell.battle_origin.static_candidate.as_mut() {
            freeze_visible_static_encounter_anchors(
                candidate,
                &rendered,
                world_frame.as_deref(),
                &objects,
            );
        }
        if let Some(bound) = shell.battle_origin.bound_static.as_mut() {
            freeze_visible_static_encounter_anchors(
                &mut bound.candidate,
                &rendered,
                world_frame.as_deref(),
                &objects,
            );
        }
    }
    let next = shell.battle_origin.bound_static.as_mut().map(|bound| {
        bound
            .publication
            .get_or_insert_with(|| {
                use crystal_render_api::{
                    VisualBattleLocation, VisualBattleObjectTarget,
                    VisualBattleSourceMovement as Mode, VisualBattleSourcePose,
                };
                let candidate = &bound.candidate;
                let witness = candidate
                    .witnessed_start
                    .as_ref()
                    .expect("bound only after witnessed start");
                Arc::new(VisualBattleLocation {
                    generation: bound.generation,
                    source: VisualBattleSourcePose {
                        map_id: Arc::from(candidate.source.map_name.as_str()),
                        source_frame: candidate.source.frame,
                        core_tile: IVec2::new(
                            i32::from(candidate.source.tile.x),
                            i32::from(candidate.source.tile.y),
                        ),
                        facing: match candidate.source.facing {
                            Direction::Up => IVec2::NEG_Y,
                            Direction::Right => IVec2::X,
                            Direction::Down => IVec2::Y,
                            Direction::Left => IVec2::NEG_X,
                        },
                        movement: match candidate.source.mode {
                            MovementMode::Normal => Mode::Normal,
                            MovementMode::Bike => Mode::Bike,
                            MovementMode::Skate => Mode::Skate,
                            MovementMode::Surf => Mode::Surf,
                            MovementMode::SurfPika => Mode::SurfPika,
                        },
                    },
                    target: VisualBattleObjectTarget {
                        object_identifier: Arc::from(candidate.target_identifier.as_str()),
                        object_script: Arc::from(candidate.object_script.as_str()),
                        core_tile: IVec2::new(
                            i32::from(candidate.target_tile.x),
                            i32::from(candidate.target_tile.y),
                        ),
                        trigger_script: Arc::from(candidate.trigger_script.as_str()),
                        battle_source_script: Arc::from(witness.source_script.as_str()),
                        startbattle_command_index: witness.command_index,
                    },
                    source_map_size_core_tiles: candidate.map_size,
                    anchors: candidate.anchors.clone(),
                })
            })
            .clone()
    });
    let unchanged = match (&published.location, &next) {
        (Some(old), Some(new)) => Arc::ptr_eq(old, new),
        (None, None) => true,
        _ => false,
    };
    if !unchanged {
        published.location = next;
    }
}
