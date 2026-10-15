/// Publish the already-rendered overworld as a read-only frame for optional
/// presentation plugins.  This is intentionally downstream of
/// `render_playfield`: it observes presentation state and entities only and
/// never reads or mutates authoritative movement, collision, scripts, or game
/// state.
fn publish_visual_world_frame(
    rendered: Res<RenderedViewport>,
    runtime_shell: Res<BevyRuntimeShell>,
    #[cfg(feature = "voxel-view")] settings: Option<Res<crystal_voxel_view::VoxelViewSettings>>,
    #[cfg(feature = "location-tester")] capture_hold: Option<Res<render_capture::PairedCaptureHold>>,
    map_sprites: Query<
        (&Sprite, &Transform),
        (With<PlayfieldTile>, Without<PlayfieldPriorityTile>),
    >,
    players: Query<
        (

            &Sprite,
            &Transform,
            Option<&PlayerSpriteFrames>,
        ),
        With<PlayerMarker>,
    >,
    multiplayer_ghosts: Query<
        (&MultiplayerGhost,  &Sprite, &Transform),
        Without<PlayerMarker>,
    >,
    grass_rustles: Query<
        ( &Sprite, &Transform),
        (With<GrassRustleMarker>, Without<PlayerMarker>),
    >,
    healing: Query<(&HealMachineWorldSprite,&Sprite,&Transform)>,
    objects: Query<(&VisibleObjectSprite,  &Sprite, &Transform), With<ObjectMarker>>,
    battle_entities: Query<
        (),
        Or<(
            With<BattleBattlerMarker>,
            With<BattleHudMarker>,
            With<BattleCommandMarker>,
            With<FixedBattleCanvasMarker>,
            With<BattleWindowFrameMarker>,
        )>,
    >,
    fullscreen_entities: Query<(), Or<(With<TitleScreenMarker>, With<VisibleIntroSurface>)>>,
    mut published: ResMut<crystal_render_api::VisualWorldFrame>,
    presentation: (
        Option<ResMut<crystal_render_api::VisualWorldLighting>>,
        Option<Res<RuntimeTickTimer>>,
        Option<ResMut<crystal_render_api::VisualPlayerTraversal>>,
    ),
    phases: Option<ResMut<crystal_render_api::VisualTileAnimationFrame>>,
    clock: Option<ResMut<crystal_render_api::VisualWorldClock>>,
) {
    let (lighting, tick_timer, mut traversal) = presentation;
    #[cfg(feature = "voxel-view")]
    let _trace = crystal_voxel_view::loading_trace::span("frame.publish_world", || serde_json::json!({"map": rendered.map_name.as_deref()}));
    if let Some(mut lighting) = lighting {
        if *lighting != rendered.visual_lighting {
            *lighting = rendered.visual_lighting;
        }
    }
    if let Some(mut clock) = clock {
        let next_clock = if rendered.visual_clock_enabled {
            runtime_shell.cached_snapshot.as_ref()
                .filter(|(_, snapshot)| {
                    rendered.map_name.as_deref() == Some(snapshot.overworld.map_name.as_str())
                })
                .map(|_| {
                    // RTC ticks can leave the retained presentation snapshot
                    // unchanged. Read the live authoritative clock so idle
                    // lighting still progresses without a terrain rebuild.
                    let registers = &runtime_shell.shell.session().state().time.registers;
                    crystal_render_api::VisualWorldClock {
                        minute_of_day: Some(
                            u16::from(registers.hours) * 60 + u16::from(registers.minutes),
                        ),
                        second_of_minute: registers.seconds,
                    }
                })
                // A warp can commit before its destination is rendered.
                // Keep the retained field's light until that render succeeds.
                .unwrap_or(*clock)
        } else {
            Default::default()
        };
        if *clock != next_clock {
            *clock = next_clock;
        }
    }
    if let Some(traversal) = traversal.as_mut() {
        traversal.step = None;
    }

    // A feature-enabled browser normally starts in classic 2D mode. Do not
    // enumerate, sort, and clone every visible actor for an optional renderer
    // the player has not selected. Tests that exercise extraction in
    // isolation intentionally omit the settings resource.
    #[cfg(feature = "voxel-view")]
    {
        // A frozen passive pair still compares the same source actors while
        // showing 2D. Rendering remains disabled; only this read-only extraction
        // stays live so the normal disabled-frame clear cannot fake state drift.
        #[cfg(feature = "location-tester")]
        let paired_hold = capture_hold.as_ref().is_some_and(|hold| hold.frozen);
        #[cfg(not(feature = "location-tester"))]
        let paired_hold = false;
        if settings.as_ref().is_some_and(|settings| !settings.enabled) && !paired_hold {
            clear_published_visual_world(&mut published);
            return;
        }
    }

    // These short effects are still authored in classic screen coordinates.
    // Keep the last complete optional-renderer frame underneath them instead
    // of clearing it: clearing deactivates the optional world and visibly
    // interrupts a manually selected 2.5D view for the duration of an
    // emote, scripted jump, fishing motion, or dust puff. Ledge traversal
    // publishes its ground endpoints and lift separately. The screen-space effect still
    // composites above the retained world through the normal layer-0 camera.
    // Pack hides the world before USE. If fishing starts from that inactive
    // publication, there is no complete frame to retain yet. Extract the actual
    // restored field once, subject to every ordinary entity/texture/grid check
    // below. This is display recovery only; it never supplies battle anchors.
    let restore_fishing_field = !published.active
        && runtime_shell.visible_fishing_animation.is_some()
        && runtime_shell.field_notice_scene.as_ref().is_some_and(|scene| {
            scene.battle.is_none()
                && rendered.map_name.as_deref() == Some(scene.overworld.map_name.as_str())
                && rendered.tile == Some(scene.overworld.tile)
                && rendered.player_sprite_facing == Some(scene.overworld.facing)
                && rendered.player_sprite_mode == Some(scene.overworld.mode)
        });
    let mounted_ledge_jump = runtime_shell.visible_ledge_jump.is_some()
        && rendered.player_sprite_mode == Some(MovementMode::Bike);
    if (!voxel_spatial_effects_supported(&runtime_shell) || mounted_ledge_jump) && !restore_fishing_field {
        return;
    }

    // A walking or Surf encounter commits core battle state before its original walk
    // lands. Keep extracting that genuinely rendered field so a moving
    // pre-step witness can observe the committed walk on the next update.
    // This exception belongs only to the current bound encounter and ends
    // when the existing transition replaces the map; all scene guards below
    // still apply.
    let step_field_visible = runtime_shell.battle_origin.bound_walking.as_ref()
        .map(|bound| &bound.origin)
        .or_else(|| runtime_shell.battle_origin.bound_surf.as_ref().map(|bound| &bound.origin))
        .is_some_and(|bound_origin| {
            runtime_shell.battle_origin.published()
                .is_some_and(|origin| Arc::ptr_eq(origin, bound_origin))
        })
        && (matches!(runtime_shell.pending_overworld_step_boundary,
            Some(PendingOverworldStepBoundary::WildBattle))
            || runtime_shell.visible_battle_transition
                .is_some_and(|transition| transition.frame < 3));
    if rendered.title_active
        // The naming screen is committed through Commands later in the classic
        // render pass. Check its live state too: waiting for the presenter
        // entity to exist leaves one frame in which an optional world renderer
        // can draw the overworld over the naming LCD.
        || naming_screen_blocks_world_presentation(runtime_shell.pending_name_input.as_ref())
        || runtime_shell.pending_mail_input.is_some()
        || runtime_shell.pending_mail_read.is_some()
        || (runtime_shell.battle_lcd_animation_active && !step_field_visible)
        || battle_entities.iter().next().is_some()
        || fullscreen_entities.iter().next().is_some()
        || rendered.map_name.is_none()
        || rendered.visual_tiles_revision.is_none()
    {
        clear_published_visual_world(&mut published);
        return;
    }

    let (Some(map_id), Some(terrain_revision), Some(map_texture)) = (
        rendered.map_name.as_deref(),
        rendered.visual_tiles_revision,
        rendered.map_texture.as_ref(),
    ) else {
        clear_published_visual_world(&mut published);
        return;
    };
    let Some((_, map_transform)) = map_sprites
        .iter()
        .find(|(sprite, _)| sprite.image.id() == map_texture.id())
    else {
        clear_published_visual_world(&mut published);
        return;
    };
    let center = map_transform.translation.truncate();
    if !center.is_finite() {
        clear_published_visual_world(&mut published);
        return;
    }
    #[cfg(feature = "voxel-view")]
    let (published_map_texture, published_grid_size) = {
        let Some(texture) = rendered.visual_world_texture.as_ref() else {
            clear_published_visual_world(&mut published);
            return;
        };
        (texture.clone(), rendered.visual_world_grid_size)
    };
    #[cfg(not(feature = "voxel-view"))]
    let (published_map_texture, published_grid_size) = (
        map_texture.clone(),
        UVec2::new(VISUAL_WORLD_TILES_X as u32, VISUAL_WORLD_TILES_Y as u32),
    );

    let mut actors = Vec::with_capacity(
        players.iter().count()
            + multiplayer_ghosts.iter().count()
            + objects.iter().count()
            + grass_rustles.iter().count(),
    );
    let mut player_iter = players.iter();
    if let Some((sprite, transform, _frames)) = player_iter.next() {
        let texture = &sprite.image;
        // A second player sprite is an incomplete deferred scene transition,
        // not a valid immutable frame to hand to a renderer mod.
        if player_iter.next().is_some() {
            clear_published_visual_world(&mut published);
            return;
        }
        #[cfg(feature = "voxel-view")]
        let (texture, directional_sprite) = {
            let Some(frames) = _frames else {
                clear_published_visual_world(&mut published);
                return;
            };
            let facing = match rendered.player_sprite_facing.unwrap_or(Direction::Down) {
                Direction::Down => 0,
                Direction::Left => 1,
                Direction::Up => 2,
                Direction::Right => 3,
            };
            let orbit = settings
                .as_ref()
                .map_or(0.0, |settings| settings.camera.rotation_step);
            let side = (facing + (orbit / 2.0).round().rem_euclid(4.0) as usize) % 4;
            let Some((standing, walking)) = frames.directional_frames.get(side) else {
                clear_published_visual_world(&mut published);
                return;
            };
            let action = frames
                .walking
                .as_ref()
                .is_some_and(|walking| walking == texture);
            let mut view_sprite = sprite.clone();
            view_sprite.flip_x = action && side % 2 == 0 && runtime_shell.player_walk_mirror_stride;
            (if action { walking } else { standing }, view_sprite)
        };
        #[cfg(feature = "voxel-view")]
        let sprite = &directional_sprite;
        let Some(mut actor) = visual_actor(
            crystal_render_api::VisualActorId::Player,
            _frames
                .map(|frames| frames.source_id.clone())
                .unwrap_or_else(|| Arc::from("player")),
            texture,
            sprite,
            transform,
            false,
        ) else {
            clear_published_visual_world(&mut published);
            return;
        };
        actor.facing = Some(visual_facing(
            rendered.player_sprite_facing.unwrap_or(Direction::Down),
        ));
        // Strip source LCD lift from the ground coordinate before pitching it
        // into 3D. Height is rendered independently from map displacement.
        actor.center.y -= visible_ledge_jump_y_offset(runtime_shell.visible_ledge_jump);
        if runtime_shell.visible_ledge_jump.is_some() {
            // The classic jump presenter samples integer LCD ticks. Align its
            // ground with the actually rendered terrain center, which may be a
            // retained camera sample, rather than recomputing another subframe.
            actor.center += center
                - visible_overworld_camera_offset(&rendered, &runtime_shell, 0.0);
        }
        if let Some(traversal) = traversal.as_mut() {
            if traversal.map_id.as_ref() != map_id {
                traversal.map_id = Arc::from(map_id);
            }
            let jump = runtime_shell.visible_ledge_jump;
            let from = jump.map(|jump| jump.from).or(runtime_shell.player_walk_from);
            let to = jump.map(|jump| jump.to).or(rendered.tile);
            let (remaining, total) = jump.map_or(
                (runtime_shell.player_walk_frame_ticks, runtime_shell.player_walk_total_ticks),
                |jump| (16_u8.saturating_sub(jump.frame), 16),
            );
            if let (Some(from), Some(to)) = (from, to) {
                if remaining > 0 {
                    let progress = jump.map_or_else(
                        || visible_movement_progress_with_subframe(
                            remaining, total, tick_timer.as_ref().map_or(0.0, |timer| timer.presentation_subframe()),
                        ),
                        |jump| f32::from(jump.frame) / 16.0,
                    );
                    let delta = Vec2::new(f32::from(to.x - from.x), -f32::from(to.y - from.y))
                        * (TILE_SIZE * f32::from(RENDER_TILES_PER_RUNTIME_TILE));
                    let foot = actor.center - Vec2::Y * actor.size.y * 0.5;
                    traversal.step = Some(crystal_render_api::VisualTraversalStep {
                        from: foot - delta * progress,
                        to: foot + delta * (1.0 - progress),
                        progress,
                        ledge_jump: jump.is_some(),
                    });
                }
            }
        }
        actors.push(actor);
    }

    let mut visible_ghosts = multiplayer_ghosts.iter().collect::<Vec<_>>();
    visible_ghosts.sort_by(|left, right| left.0.user_id.cmp(&right.0.user_id));
    for (ghost, sprite, transform) in visible_ghosts {
        let texture = &sprite.image;
        let Some(mut actor) = visual_actor(
            crystal_render_api::VisualActorId::RemotePlayer(remote_player_visual_id(
                &ghost.user_id,
            )),
            ghost.source_id.clone(),
            texture,
            sprite,
            transform,
            false,
        ) else {
            clear_published_visual_world(&mut published);
            return;
        };
        actor.facing = Some(visual_facing(ghost.facing));
        if visual_actor_intersects_grid(&actor, center, published_grid_size) {
            actors.push(actor);
        }
    }

    // ECS query order is not a presentation contract.  Publish objects in
    // stable ASM/object-slot order so renderer caches receive deterministic
    // input even when Bevy archetypes move.
    let mut visible_objects = objects.iter().collect::<Vec<_>>();
    visible_objects.sort_by_key(|(object, _, _)| object.object_index);
    if visible_objects
        .windows(2)
        .any(|pair| pair[0].0.object_index == pair[1].0.object_index)
    {
        clear_published_visual_world(&mut published);
        return;
    }
    for (object, sprite, transform) in visible_objects {
        let texture = &sprite.image;
        let Ok(object_index) = u32::try_from(object.object_index) else {
            clear_published_visual_world(&mut published);
            return;
        };
        #[cfg(feature = "voxel-view")]
        let (texture, directional_sprite) = {
            let facing = match object.world_facing {
                Direction::Down => 0,
                Direction::Left => 1,
                Direction::Up => 2,
                Direction::Right => 3,
            };
            let orbit = settings
                .as_ref()
                .map_or(0.0, |settings| settings.camera.rotation_step);
            let side = (facing + (orbit / 2.0).round().rem_euclid(4.0) as usize) % 4;
            let Some((standing, walking)) = object.directional_frames.get(side) else {
                clear_published_visual_world(&mut published);
                return;
            };
            let action = object
                .walking
                .as_ref()
                .is_some_and(|walking| walking == texture);
            let mut view_sprite = sprite.clone();
            // Read the current stride, not a value retained when this entity
            // was last reconciled: animation can advance without a map redraw.
            let phase = object.object_identifier.as_ref().and_then(|id| {
                (runtime_shell.object_walk_from.contains_key(id)
                    || runtime_shell
                        .trainer_walk_from
                        .as_ref()
                        .is_some_and(|(walking, _)| walking == id))
                .then(|| {
                    runtime_shell
                        .object_walk_phases
                        .get(id)
                        .copied()
                        .unwrap_or(1)
                })
            });
            view_sprite.flip_x = action
                && side % 2 == 0
                && phase.map_or(
                    !runtime_shell.object_walk_stride,
                    object_walk_uses_mirrored_action_frame,
                );
            (
                if action {
                    walking.as_ref().unwrap_or(standing)
                } else {
                    standing
                },
                view_sprite,
            )
        };
        #[cfg(feature = "voxel-view")]
        let sprite = &directional_sprite;
        let Some(mut actor) = visual_actor(
            crystal_render_api::VisualActorId::Object(object_index),
            object.source_id.clone(),
            texture,
            sprite,
            transform,
            object.above_priority,
        ) else {
            clear_published_visual_world(&mut published);
            return;
        };
        actor.facing = Some(visual_facing(object.world_facing));
        actor.model_palette = object.model_palette;
        if visual_actor_intersects_grid(&actor, center, published_grid_size) {
            actors.push(actor);
        }
    }

    let mut rustle_iter = grass_rustles.iter();
    if let Some((sprite, transform)) = rustle_iter.next() {
        let texture = &sprite.image;
        if rustle_iter.next().is_some() {
            clear_published_visual_world(&mut published);
            return;
        }
        let Some(effect) = visual_actor(
            crystal_render_api::VisualActorId::Effect(
                crystal_render_api::VisualEffectId::GrassRustle,
            ),
            Arc::from("effect_grass_rustle"),
            texture,
            sprite,
            transform,
            true,
        ) else {
            clear_published_visual_world(&mut published);
            return;
        };
        if visual_actor_intersects_grid(&effect, center, published_grid_size) {
            actors.push(effect);
        }
    }

    #[cfg(feature="voxel-view")]
    if crystal_voxel_view::native_clinic_healing_map(map_id) {
        for (marker,sprite,transform) in &healing {
            let source=match marker.0 {
                crystal_render_api::VisualEffectId::HealMachineBall(_)=>"effect_heal_ball",
                crystal_render_api::VisualEffectId::HealMachineLamp(_)=>"effect_heal_lamp",
                _=>continue,
            };
            if let Some(effect)=visual_actor(crystal_render_api::VisualActorId::Effect(marker.0),
                Arc::from(source),&sprite.image,sprite,transform,false) {
                actors.push(effect);
            }
        }
    }

    // The downstream 3D follower publisher owns this presentation actor.
    // Retain it during extraction so an idle follower does not invalidate
    // otherwise identical terrain/actor publications every host frame.
    #[cfg(feature = "voxel-view")]
    actors.extend(published.actors.iter().filter(|actor| matches!(actor.id,
        crystal_render_api::VisualActorId::PartyFollower | crystal_render_api::VisualActorId::SurfMount)).cloned());
    let terrain_unchanged = published.active
        && published.map_id.as_ref() == map_id
        && published.source_map_size_core_tiles == rendered.source_map_size_core_tiles
        && published.jump_ledges == rendered.visual_jump_ledges
        && published.terrain_revision == terrain_revision
        && published.map_texture == published_map_texture
        && published.viewport_size == Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT)
        && published.tile_size == Vec2::splat(TILE_SIZE)
        && published.grid_size == published_grid_size;
    if let Some(mut phases) = phases {
        let (x, y) = rendered.viewport_origin.expect("validated viewport origin");
        let grid_origin = IVec2::new(i32::from(x), i32::from(y))
            - (published_grid_size.as_ivec2()
                - IVec2::new(VIEWPORT_TILES_X as i32, VIEWPORT_TILES_Y as i32)) / 2;
        let animated = || rendered.visual_tiles.iter()
            .filter(|tile| tile.animation_frames.is_some());
        // Compare before allocating: an idle world retains both geometry and
        // phase storage. Only live source-art changes publish sparse updates.
        if phases.map_id.as_ref() != map_id || phases.grid_origin != grid_origin
            || !phases.tiles.iter().eq(animated())
        {
            *phases = crystal_render_api::VisualTileAnimationFrame {
                map_id: Arc::from(map_id),
                grid_origin,
                tiles: animated().cloned().collect(),
            };
        }
    }
    if terrain_unchanged && published.center == center && published.actors == actors {
        // The optional renderer consumes change detection. Do not republish an
        // identical 6,888-tile frame on every host update: that cloned the
        // complete grid and forced validation/profile work while standing
        // still, even though neither simulation nor presentation had moved.
        return;
    }
    let tiles = if terrain_unchanged {
        std::mem::take(&mut published.tiles)
    } else {
        rendered.visual_tiles.clone()
    };
    let next = crystal_render_api::VisualWorldFrame {
        active: true,
        map_id: Arc::from(map_id),
        source_map_size_core_tiles: rendered.source_map_size_core_tiles,
        jump_ledges: rendered.visual_jump_ledges.clone(),
        terrain_revision,
        grid_origin: {
            let (x, y) = rendered
                .viewport_origin
                .expect("published terrain has a viewport origin");
            IVec2::new(i32::from(x), i32::from(y))
                - (published_grid_size.as_ivec2()
                    - IVec2::new(VIEWPORT_TILES_X as i32, VIEWPORT_TILES_Y as i32))
                    / 2
        },
        map_texture: published_map_texture,
        center,
        viewport_size: Vec2::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT),
        tile_size: Vec2::splat(TILE_SIZE),
        grid_size: published_grid_size,
        tiles,
        actors,
    };
    if terrain_unchanged || next.validate().is_ok() {
        *published = next;
    } else {
        clear_published_visual_world(&mut published);
    }
}

fn remote_player_visual_id(user_id: &str) -> u64 {
    if let Some(id) = user_id
        .strip_prefix("player-")
        .and_then(|value| value.parse::<u64>().ok())
    {
        return id;
    }
    user_id.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn clear_published_visual_world(published: &mut crystal_render_api::VisualWorldFrame) {
    // Avoid marking an already-inactive resource as changed. Optional
    // renderers can then skip their expensive validation and profile passes.
    if published.active {
        *published = crystal_render_api::VisualWorldFrame::default();
    }
}

fn naming_screen_blocks_world_presentation(input: Option<&PendingNameInput>) -> bool {
    input.is_some()
}

/// Scripted effects in this list are still authored in classic screen coordinates.
/// Publishing a displaced actor against an undisplaced ground point would
/// turn a jump into motion across the pitched floor. Callers retain the last
/// complete world frame while these screen-space overlays run; they must not
/// change the user's manually selected presentation mode.
fn voxel_spatial_effects_supported(runtime_shell: &BevyRuntimeShell) -> bool {
    let scripted_actor_displacement = runtime_shell.visible_player_sprite_y_offset != 0
        || runtime_shell
            .visible_script_movement
            .as_ref()
            .is_some_and(|movement| {
                movement.active_jump_duration.is_some()
                    || movement.follower_active_jump_duration.is_some()
                    || movement.stationary_y_offset != 0
            });

    !scripted_actor_displacement
        && runtime_shell.visible_fishing_animation.is_none()
        && runtime_shell.visible_strength_boulder_dust.is_none()
        && runtime_shell.visible_overworld_emote.is_none()
}

fn visual_facing(direction: Direction) -> Vec2 {
    match direction {
        Direction::Down => Vec2::NEG_Y,
        Direction::Up => Vec2::Y,
        Direction::Left => Vec2::NEG_X,
        Direction::Right => Vec2::X,
    }
}

fn visual_actor(
    id: crystal_render_api::VisualActorId,
    source_id: Arc<str>,
    texture: &Handle<Image>,
    sprite: &Sprite,
    transform: &Transform,
    above_priority: bool,
) -> Option<crystal_render_api::VisualActor> {
    let size = sprite.custom_size?;
    let center = transform.translation.truncate();
    if !size.is_finite() || size.x <= 0.0 || size.y <= 0.0 || !center.is_finite() {
        return None;
    }
    Some(crystal_render_api::VisualActor {
        id,
        source_id,
        texture: texture.clone(),
        center,
        size,
        flip_x: sprite.flip_x,
        above_priority,
        facing: None,
        model_palette: None,
    })
}

fn visual_actor_intersects_grid(
    actor: &crystal_render_api::VisualActor,
    center: Vec2,
    grid_size: UVec2,
) -> bool {
    let half_grid = grid_size.as_vec2() * TILE_SIZE * 0.5;
    let half_actor = actor.size * 0.5;
    let delta = actor.center - center;
    delta.x.abs() <= half_grid.x + half_actor.x && delta.y.abs() <= half_grid.y + half_actor.y
}

fn visual_tile_grid_is_complete(tiles: &[crystal_render_api::VisualTile]) -> bool {
    let Ok(width) = usize::try_from(VISUAL_WORLD_TILES_X) else {
        return false;
    };
    let Ok(height) = usize::try_from(VISUAL_WORLD_TILES_Y) else {
        return false;
    };
    tiles.len() == width * height
        && tiles.iter().enumerate().all(|(index, tile)| {
            usize::try_from(tile.column).ok() == Some(index % width)
                && usize::try_from(tile.row).ok() == Some(index / width)
        })
}

fn visual_terrain_revision(
    map_visual_key: u64,
    viewport_origin: (i16, i16),
    tiles: &[crystal_render_api::VisualTile],
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    map_visual_key.hash(&mut hasher);
    viewport_origin.hash(&mut hasher);
    // Geometry is selected by stable source identity, not by the current
    // animation-frame image handle. Hashing live handles here caused water
    // animation to replace the async mesh request every frame, so a cave with
    // animated water could keep the selected 2.5D renderer inactive forever.
    for tile in tiles {
        tile.column.hash(&mut hasher);
        tile.row.hash(&mut hasher);
        tile.source.hash(&mut hasher);
        tile.priority.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod render_mod_tests {
    use super::*;

    #[test]
    fn remote_player_visual_ids_are_stable() {
        assert_eq!(remote_player_visual_id("player-42"), 42);
        assert_eq!(
            remote_player_visual_id("custom-user"),
            remote_player_visual_id("custom-user")
        );
        assert_ne!(
            remote_player_visual_id("custom-user"),
            remote_player_visual_id("other-user")
        );
    }

    fn complete_visual_grid() -> Vec<crystal_render_api::VisualTile> {
        (0..VISUAL_WORLD_TILES_Y as u32)
            .flat_map(|row| {
                (0..VISUAL_WORLD_TILES_X as u32).map(move |column| crystal_render_api::VisualTile {
                    column,
                    row,
                    source: crystal_render_api::VisualTileSource {
                        tileset_id: Arc::from("johto"),
                        metatile_id: 3,
                        subtile_column: u8::try_from(column % 4)
                            .expect("test subtile column fits u8"),
                        subtile_row: u8::try_from(row % 4).expect("test subtile row fits u8"),
                        tile_index: u16::try_from(row * VISUAL_WORLD_TILES_X as u32 + column)
                            .expect("test tile index fits u16"),
                    },
                    texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1 + u128::from(row * VISUAL_WORLD_TILES_X as u32 + column)), std::marker::PhantomData),
                    animation_frames: None,
                    priority: false,
                })
            })
            .collect()
    }

    #[test]
    fn visual_tile_grid_requires_one_row_major_cell_per_terrain_position() {
        let mut tiles = complete_visual_grid();
        assert!(visual_tile_grid_is_complete(&tiles));

        tiles.pop();
        assert!(!visual_tile_grid_is_complete(&tiles));

        let mut tiles = complete_visual_grid();
        tiles.swap(0, 1);
        assert!(!visual_tile_grid_is_complete(&tiles));
    }

    #[test]
    fn fully_clipped_actor_is_not_published_to_renderer_mod() {
        let grid_size = UVec2::new(40, 36);
        let half_grid_width = grid_size.x as f32 * TILE_SIZE * 0.5;
        let actor = crystal_render_api::VisualActor { model_palette: None,
            id: crystal_render_api::VisualActorId::Object(1),
            source_id: Arc::from("edge_npc"),
            texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(7), std::marker::PhantomData),
            center: Vec2::new(half_grid_width + 17.0, 0.0),
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: None,
        };
        assert!(!visual_actor_intersects_grid(&actor, Vec2::ZERO, grid_size));

        let touching = crystal_render_api::VisualActor {
            center: Vec2::new(half_grid_width + 8.0, 0.0),
            ..actor
        };
        assert!(visual_actor_intersects_grid(
            &touching,
            Vec2::ZERO,
            grid_size
        ));
    }

    #[test]
    fn naming_screen_blocks_world_presentation_before_its_lcd_entity_is_spawned() {
        let input = PendingNameInput {
            label: "YOUR NAME?".to_string(),
            value: String::new(),
            max_length: 7,
            cursor_column: 0,
            cursor_row: 0,
            case: NameInputCase::Upper,
        };

        assert!(!naming_screen_blocks_world_presentation(None));
        assert!(naming_screen_blocks_world_presentation(Some(&input)));
    }

    #[test]
    #[ignore = "manual retained-frame CPU benchmark; run with --ignored --nocapture"]
    fn benchmark_retained_terrain_revision() {
        use std::{hint::black_box, time::Instant};
        let tiles = complete_visual_grid();
        let iterations = 10_000;
        let start = Instant::now();
        for _ in 0..iterations {
            assert!(visual_tile_grid_is_complete(black_box(&tiles)));
            black_box(visual_terrain_revision(7, (-4, 12), black_box(&tiles)));
        }
        let uncached = start.elapsed();
        let revision = Some(visual_terrain_revision(7, (-4, 12), &tiles));
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(black_box(revision).expect("validated retained terrain"));
        }
        eprintln!(
            "{iterations} retained frames / {} tiles: rescan {uncached:?}, cached {:?}",
            tiles.len(),
            start.elapsed()
        );
    }

    #[test]
    fn visual_terrain_revision_tracks_viewport_and_source_art() {
        let tiles = complete_visual_grid();
        let baseline = visual_terrain_revision(7, (-4, 12), &tiles);

        assert_eq!(baseline, visual_terrain_revision(7, (-4, 12), &tiles));
        assert_ne!(baseline, visual_terrain_revision(8, (-4, 12), &tiles));
        assert_ne!(baseline, visual_terrain_revision(7, (-3, 12), &tiles));

        let mut changed_tiles = tiles;
        changed_tiles[0].source.tile_index += 1;
        assert_ne!(
            baseline,
            visual_terrain_revision(7, (-4, 12), &changed_tiles)
        );

        let mut changed_texture = complete_visual_grid();
        changed_texture[0].texture = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(10_000), std::marker::PhantomData);
        assert_eq!(
            baseline,
            visual_terrain_revision(7, (-4, 12), &changed_texture)
        );
    }
}

#[cfg(feature="voxel-view")]
fn sync_native_healing_sprite_visibility(
    frame:Res<crystal_render_api::VisualWorldFrame>,
    status:Res<crystal_voxel_view::VoxelViewStatus>,
    healing:Option<Res<crystal_voxel_view::NativeHealingEffectsStatus>>,
    mut sprites:Query<&mut Visibility,With<HealMachineWorldSprite>>,
) {
    let consumed=frame.active&&status.active&&healing.is_some_and(|h|h.consumed);
    for mut visibility in &mut sprites {
        *visibility=if consumed {Visibility::Hidden}else{Visibility::Inherited};
    }
}
