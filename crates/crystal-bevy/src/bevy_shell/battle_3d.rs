// This module is included into the production shell. It extracts only the
// snapshot selected by render_playfield after retained-dialog scene selection.
// It never calls snapshot(), tick(), a command dispatcher, or turn resolution.
use crystal_render_api::{
    VisualBattleBattler, VisualBattleCue, VisualBattleCueKind, VisualBattleEnvironment,
    VisualBattleFrame, VisualBattleSide, VisualBattleSourceFrame, VisualBattleSourceObject,
};

#[derive(Component)]
struct ImmersiveBattleReplaced;

/// Called at the same presentation boundary as battle_canvas_active. Source
/// images remain an explicit honest fallback for species without exact meshes.
fn capture_presented_battle(
    commands: &mut Commands,
    snapshot: &RuntimeShellSnapshot,
    shell: &BevyRuntimeShell,
    canvas_active: bool,
    art: &mut RenderedTilesetArt,
    images: &mut Assets<Image>,
) -> Result<()> {
    #[cfg(feature = "operation-trace")]
    let _span = bevy::log::info_span!("crystal_battle_extract").entered();
    // Clear first so an art error restores the complete classic renderer.
    commands.insert_resource(VisualBattleFrame::default());
    let Some(battle) = snapshot.battle.as_ref().filter(|_| canvas_active) else {
        return Ok(());
    };
    // Trainer portraits and the source sliding introduction remain faithful.
    // Arena handover happens only after their existing presentation completes.
    if shell.visible_battle_sliding_intro.is_some()
        || shell.battle_entry_messages_remaining > 0
        || visible_trainer_result_frame(shell).is_some()
    {
        return Ok(());
    }
    let environment = snapshot
        .maps
        .iter()
        .find(|map| map.map_name == snapshot.overworld.map_name)
        .and_then(|map| {
            map.attributes.environment.as_deref().or_else(|| {
                map.metadata
                    .as_ref()
                    .map(|metadata| metadata.environment.as_str())
            })
        })
        .unwrap_or("");
    let mut frame = VisualBattleFrame {
        active: true,
        map_id: Arc::from(snapshot.overworld.map_name.as_str()),
        environment: immersive_battle_environment(&snapshot.overworld.map_name, environment),
        ..Default::default()
    };
    let animation = shell.visible_move_animations.front();
    let (player_visible, enemy_visible) = visible_move_battler_visibility(animation);
    let (player_art, enemy_art) = visible_move_battler_art_overrides(animation);
    let (player_species, enemy_species) = visible_move_battler_species_overrides(animation);
    let (player_shiny, enemy_shiny) = visible_move_battler_shiny_overrides(animation);
    let pending_faints = visible_pending_faint_sides(shell);
    let player = battle.active_player_party_index.and_then(|index| {
        snapshot
            .party
            .slots
            .iter()
            .find(|slot| slot.index == index)
            .map(|slot| &slot.pokemon)
    });
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        let is_player = side == VisualBattleSide::Player;
        let Some(pokemon) = (if is_player {
            player
        } else {
            Some(&battle.enemy_pokemon)
        }) else {
            continue;
        };
        let art_override = if is_player { player_art } else { enemy_art };
        let species_override = if is_player {
            player_species
        } else {
            enemy_species
        };
        let transformed = if is_player {
            battle.player_transformed_species.as_deref()
        } else {
            battle.enemy_transformed_species.as_deref()
        };
        let transform_pending = animation.is_some_and(|animation| {
            animation.started
                && animation.animation_label == "BattleAnim_Transform"
                && animation.player_move == is_player
                && art_override != VisibleBattlerArtOverride::Transform
        });
        let opposite_species = if is_player {
            Some(
                battle
                    .enemy_transformed_species
                    .as_deref()
                    .unwrap_or(&battle.enemy_pokemon.species.id),
            )
        } else {
            battle
                .player_transformed_species
                .as_deref()
                .or_else(|| player.map(|p| p.species.id.as_str()))
        };
        let species = species_override
            .or_else(|| {
                (art_override == VisibleBattlerArtOverride::Transform)
                    .then_some(opposite_species)
                    .flatten()
            })
            .or_else(|| if transform_pending { None } else { transformed })
            .unwrap_or(&pokemon.species.id);
        let substitute = match art_override {
            VisibleBattlerArtOverride::Substitute => true,
            VisibleBattlerArtOverride::Pokemon => false,
            _ => {
                if is_player {
                    battle.player_substitute_hp > 0
                } else {
                    battle.enemy_substitute_hp > 0
                }
            }
        };
        let minimize = art_override == VisibleBattlerArtOverride::Minimize;
        let dvs = if transform_pending {
            pokemon.dvs
        } else if is_player {
            battle.player_transformed_dvs.unwrap_or(pokemon.dvs)
        } else {
            battle.enemy_transformed_dvs.unwrap_or(pokemon.dvs)
        };
        let shiny = (if is_player { player_shiny } else { enemy_shiny })
            .unwrap_or_else(|| visible_dvs_are_shiny(dvs));
        let frontpic_frame = if is_player {
            0
        } else {
            shell
                .visible_frontpic_animation
                .as_ref()
                .filter(|animation| animation.species_id == battle.enemy_pokemon.species.id)
                .map(|animation| animation.frame)
                .unwrap_or(0)
        };
        let source_frame = if minimize {
            battle_minimize_frame(art, &shell.asset_root, images)?.clone()
        } else if substitute {
            battle_substitute_frames(art, &shell.asset_root, images)?[usize::from(is_player)]
                .clone()
        } else {
            pokemon_animation_frame_for_art(
                art,
                &shell.asset_root,
                &pokemon_asset_id_for_dvs(species, dvs),
                if is_player {
                    PokemonSpriteSide::Back
                } else {
                    PokemonSpriteSide::Front
                },
                shiny,
                frontpic_frame,
                images,
            )
            .context("immersive battle fallback source art unavailable")?
        };
        let unchecked = if is_player {
            battle.player_spikes_zero_hp_unchecked
        } else {
            battle.enemy_spikes_zero_hp_unchecked
        };
        let semi_invulnerable = if is_player {
            battle.player_semi_invulnerable
        } else {
            battle.enemy_semi_invulnerable
        };
        let send_out_pending = if is_player {
            shell.battle_player_send_out_pending
                || (shell.battle_entry_messages_remaining == 0
                    && shell
                        .battle_messages
                        .front()
                        .is_some_and(|message| visible_message_is_player_send_out(message)))
        } else {
            shell.battle_enemy_send_out_pending
        };
        let exiting = shell
            .visible_trainer_exit_animation
            .as_ref()
            .is_some_and(|exit| {
                (exit.side == crate::core::battle::turn::BattleSide::Player) == is_player
            });
        let visible = (pokemon.hp > 0 || unchecked || pending_faints[side.index()])
            && (if is_player {
                player_visible
            } else {
                enemy_visible
            })
            && (!semi_invulnerable
                || visible_move_controls_battler_visibility(animation, is_player))
            && !send_out_pending
            && !exiting
            && (is_player
                || !shell
                    .visible_capture_animation
                    .as_ref()
                    .is_some_and(VisibleCaptureAnimation::enemy_hidden));
        frame.battlers[side.index()] = Some(VisualBattleBattler {
            side,
            species_id: Arc::from(species),
            party_index: if is_player {
                battle.active_player_party_index
            } else {
                battle.active_enemy_party_index
            },
            texture: source_frame.handle,
            texture_size: source_frame.size,
            visible,
            // Authored models currently use the normal palette. Keep actual
            // shiny source art rather than claiming an incorrect shiny mesh.
            allow_species_model: !substitute && !minimize && !shiny,
            shiny,
        });
    }
    if let Some(animation) = animation.filter(|animation| animation.started) {
        frame.source = Some(capture_immersive_source_frame(
            snapshot,
            animation,
            &frame.battlers,
            art,
            &shell.asset_root,
            images,
        )?);
        let side = if animation.player_move {
            VisualBattleSide::Player
        } else {
            VisualBattleSide::Enemy
        };
        let kind = match animation.move_id.as_str() {
            "FAINT_MON" => VisualBattleCueKind::Faint,
            "RETURN_MON" => VisualBattleCueKind::Withdraw,
            _ if animation.animation_label == "BattleAnim_ReturnMon" => {
                VisualBattleCueKind::Withdraw
            }
            _ => VisualBattleCueKind::Move,
        };
        let move_data = snapshot
            .moves
            .iter()
            .find(|entry| entry.move_id == animation.move_id);
        frame.cues.push(VisualBattleCue {
            kind,
            side,
            move_id: Arc::from(animation.move_id.as_str()),
            element: Arc::from(move_data.map_or("NORMAL", |entry| entry.move_type.as_str())),
            progress: immersive_battle_progress(
                u32::from(animation.frame),
                u32::from(animation.total_frames),
            ),
            damaging: move_data.is_some_and(|entry| entry.power > 0),
        });
    }
    if let Some(send_out) = shell.visible_send_out_animation.as_ref() {
        frame.cues.push(VisualBattleCue {
            kind: VisualBattleCueKind::SendOut,
            side: if send_out.side == crate::core::battle::turn::BattleSide::Player {
                VisualBattleSide::Player
            } else {
                VisualBattleSide::Enemy
            },
            move_id: Arc::from("SEND_OUT"),
            element: Arc::from("NORMAL"),
            progress: immersive_battle_progress(
                u32::from(send_out.frame),
                u32::from(send_out.total_frames()),
            ),
            damaging: false,
        });
    }
    if let Some(capture) = shell.visible_capture_animation.as_ref().filter(|capture| {
        capture.retained_objects_visible()
            && (capture.blocked
                || capture.frame < capture.shake_entry_frame() + 8
                || capture.enemy_hidden())
    }) {
        frame.cues.push(VisualBattleCue {
            kind: if capture.blocked {
                VisualBattleCueKind::CaptureDeflect
            } else {
                VisualBattleCueKind::Capture
            },
            side: VisualBattleSide::Enemy,
            move_id: Arc::from(capture.ball_id.as_str()),
            element: Arc::from("NORMAL"),
            progress: immersive_battle_progress(
                u32::from(capture.frame),
                if capture.blocked { 52 } else { 36 },
            ),
            damaging: false,
        });
    }
    // Impact only follows the visible HP tween. A move animation does not
    // imply a hit: misses/protect/status actions must not manufacture damage.
    if let Some(tween) = shell
        .battle_hp_tween
        .as_ref()
        .filter(|_| !visible_battle_animation_owns_frame(shell))
    {
        for (side, current, target) in [
            (
                VisualBattleSide::Player,
                tween.player_pixels,
                tween.player_target_pixels,
            ),
            (
                VisualBattleSide::Enemy,
                tween.enemy_pixels,
                tween.enemy_target_pixels,
            ),
        ] {
            if current > target {
                frame.cues.push(VisualBattleCue {
                    kind: VisualBattleCueKind::Impact,
                    side,
                    move_id: Arc::from("VISIBLE_HP_LOSS"),
                    element: Arc::from("NORMAL"),
                    progress: 1.0 - f32::from(current.saturating_sub(target)) / 48.0,
                    damaging: true,
                });
            }
        }
    }
    debug_assert!(frame.validate().is_ok());
    commands.insert_resource(frame);
    Ok(())
}

/// Adapt only the interpreter's current presented frame. The source object
/// runtime is shared with the classic draw: advancing twice to the same tick
/// is a no-op, so F3 neither restarts nor doubles the source program.
fn capture_immersive_source_frame(
    snapshot: &RuntimeShellSnapshot,
    animation: &VisibleMoveAnimation,
    battlers: &[Option<VisualBattleBattler>; 2],
    art: &mut RenderedTilesetArt,
    asset_root: &AssetRoot,
    images: &mut Assets<Image>,
) -> Result<VisualBattleSourceFrame> {
    #[cfg(feature = "operation-trace")]
    let _span = bevy::log::info_span!("crystal_battle_source_extract").entered();
    let bundle = battle_anim_render_bundle(art, snapshot)?;
    let mut playback = match art.battle_object_runtime.take() {
        Some(playback)
            if playback.source == animation.object_events
                && playback.player == animation.player_move
                && playback.label == animation.animation_label
                && u32::from(animation.frame) + 1 >= playback.next_tick =>
        {
            playback
        }
        _ => new_visible_battle_objects(&bundle, animation)?,
    };
    advance_visible_battle_objects(&mut playback, &bundle, animation)?;
    let live_slots = playback.slots.clone();
    let object_obp0_write = playback.obp0_write;
    art.battle_object_runtime = Some(playback);
    let mut registers = visible_battle_dmg_palette_registers(Some(animation));
    if let Some((tick, value)) = object_obp0_write {
        if registers
            .obp0_write_frame
            .is_none_or(|frame| tick >= u32::from(frame))
        {
            registers.obp0 = value;
        }
    }
    let (player_bgp, enemy_bgp) = visible_move_battler_bgps(Some(animation));
    let (player_offset, enemy_offset) = visible_move_battler_offsets(Some(animation));
    let source_scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let mut source = VisualBattleSourceFrame {
        frame: animation.frame,
        bgp: registers.bgp,
        battler_bgps: [
            player_bgp.unwrap_or(registers.bgp),
            enemy_bgp.unwrap_or(registers.bgp),
        ],
        battler_palettes: [[[1.0; 4]; 4]; 2],
        battler_textures: [Handle::default(), Handle::default()],
        battler_offsets: [
            player_offset.truncate() / source_scale,
            enemy_offset.truncate() / source_scale,
        ],
        screen_offset: visible_move_screen_offset(Some(animation)).truncate() / source_scale,
        line_x_offsets: visible_battle_line_x_offsets(Some(animation)),
        objects: Vec::with_capacity(10),
    };
    for (index, battler) in battlers.iter().enumerate() {
        let Some(battler) = battler else {
            continue;
        };
        // Source sprite images are immutable cached frames. Palette flashes
        // create separate images, so reading the neutral palette once per
        // source texture preserves all register changes without sorting an
        // entire scaled battler image on each source animation sample.
        let palette = match art.battle_source_palette_cache.entry(battler.texture.id()) {
            std::collections::hash_map::Entry::Occupied(entry) => *entry.get(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let image = images
                    .get(&battler.texture)
                    .context("source battler palette image")?;
                *entry.insert(immersive_source_palette(&image.data))
            }
        };
        source.battler_palettes[index] = palette;
        source.battler_textures[index] = if source.battler_bgps[index] == 0xe4 {
            battler.texture.clone()
        } else {
            battle_battler_bgp_frame(
                art,
                images,
                &SpriteFrame {
                    handle: battler.texture.clone(),
                    size: battler.texture_size,
                },
                source.battler_bgps[index],
            )?
            .handle
        };
    }
    for (slot, live) in live_slots.iter().enumerate() {
        let Some(live) = live.as_ref().filter(|live| !live.oam.entries.is_empty()) else {
            continue;
        };
        let VisibleMoveObjectCommand::Spawn { object_id, .. } =
            &animation.object_events[live.event_index].command
        else {
            continue;
        };
        let object = &bundle["objects"][object_id];
        let frame = &bundle["framesets"][live.frameset][live.frame];
        let palette = match live.bytes[5] & 7 {
            0 => "PAL_BATTLE_OB_GRAY",
            1 => "PAL_BATTLE_OB_YELLOW",
            2 => "PAL_BATTLE_OB_RED",
            3 => "PAL_BATTLE_OB_GREEN",
            4 => "PAL_BATTLE_OB_BLUE",
            5 => "PAL_BATTLE_OB_BROWN",
            other => anyhow::bail!("invalid current source object palette {other}"),
        };
        let mut render = |obp0, obp1| {
            battle_anim_rendered_frame(
                art,
                &bundle,
                asset_root,
                object_id,
                object,
                live.frameset,
                live.frame,
                frame,
                !animation.player_move,
                false,
                false,
                Some(palette),
                obp0,
                obp1,
                Some(&live.oam),
                images,
            )
        };
        let rendered = render(registers.obp0, registers.obp1)?;
        let neutral = if registers.obp0 == 0xe4 && registers.obp1 == 0xe4 {
            rendered.clone()
        } else {
            render(0xe4, 0xe4)?
        };
        let size = rendered.sprite.size / source_scale;
        let center = Vec2::new(
            (live.oam.origin.0 - 8 + i32::from(rendered.offset_x)) as f32,
            (live.oam.origin.1 - 16
                + i32::from(rendered.offset_y)
                + visible_rollout_object_y_offset(animation, slot)) as f32,
        ) + size * 0.5;
        source.objects.push(VisualBattleSourceObject {
            slot,
            object_id: Arc::from(object_id.as_str()),
            texture: rendered.sprite.handle,
            neutral_texture: neutral.sprite.handle,
            center,
            size,
        });
    }
    Ok(source)
}

fn immersive_source_palette(pixels: &[u8]) -> [[f32; 4]; 4] {
    let mut colors: Vec<_> = pixels
        .chunks_exact(4)
        .filter(|pixel| pixel[3] != 0)
        .map(|pixel| [pixel[0], pixel[1], pixel[2]])
        .collect();
    colors.sort_unstable();
    colors.dedup();
    colors.sort_by_key(|c| {
        std::cmp::Reverse(u32::from(c[0]) * 299 + u32::from(c[1]) * 587 + u32::from(c[2]) * 114)
    });
    if colors.len() < 4 {
        colors.insert(0, [255; 3]);
    }
    std::array::from_fn(|shade| {
        let color = colors.get(shade).unwrap_or_else(|| colors.last().unwrap());
        [
            f32::from(color[0]) / 255.0,
            f32::from(color[1]) / 255.0,
            f32::from(color[2]) / 255.0,
            1.0,
        ]
    })
}

fn immersive_battle_progress(frame: u32, total: u32) -> f32 {
    (frame as f32 / total.max(1) as f32).clamp(0.0, 1.0)
}

fn immersive_battle_environment(map: &str, environment: &str) -> VisualBattleEnvironment {
    let map = map.to_ascii_lowercase();
    let environment = environment.to_ascii_lowercase();
    if map.contains("icepath") {
        VisualBattleEnvironment::Ice
    } else if environment.contains("cave") || map.contains("cave") || map.contains("tunnel") {
        VisualBattleEnvironment::Cave
    } else if environment.contains("water") || map.contains("whirl") {
        VisualBattleEnvironment::Water
    } else if map.contains("forest") {
        VisualBattleEnvironment::Forest
    } else if environment.contains("indoor")
        || environment.contains("gate")
        || environment.contains("dungeon")
    {
        VisualBattleEnvironment::Interior
    } else {
        VisualBattleEnvironment::Meadow
    }
}

/// Park only replaced visual layers. Controls, text, HP, PP, choices and their
/// input ownership remain exclusively in the production classic shell.
fn sync_immersive_battle_layers(
    mut commands: Commands,
    status: Res<crystal_voxel_view::BattleViewStatus>,
    world_status: Res<crystal_voxel_view::VoxelViewStatus>,
    mut cameras: Query<&mut Camera, With<MainCameraMarker>>,
    settings: Res<crystal_voxel_view::VoxelViewSettings>,
    rendered: Res<RenderedViewport>,
    shell: Res<BevyRuntimeShell>,
    hud: Query<
        (
            Entity,
            &Transform,
            Option<&ImmersiveBattleUiSource>,
            Option<&bevy::render::view::RenderLayers>,
        ),
        With<BattleHudMarker>,
    >,
    replaced: Query<
        (Entity, Option<&bevy::render::view::RenderLayers>),
        Or<(
            With<FixedBattleCanvasMarker>,
            With<BattleBattlerMarker>,
            With<ImmersiveBattleReplaced>,
        )>,
    >,
) {
    // With no 3D underlay, load-only compositing would retain old arena
    // pixels in uncovered margins. Preserve the underlay only while an
    // actual modeled world or battle camera is rendering this frame.
    let clear = immersive_composite_clear_color(world_status.active, status.active);
    for mut camera in &mut cameras {
        use bevy::render::camera::ClearColorConfig::{Default, None};
        if !matches!(
            (clear, camera.clear_color),
            (Default, Default) | (None, None)
        ) {
            camera.clear_color = clear;
        }
    }
    let active = settings.enabled
        && status.active
        && !rendered.title_active
        && shell.visible_battle_transition.is_none()
        && !(shell.pokedex_menu_open
            && shell.pokedex_scripted_entry
            && shell.pending_standard_capture.is_some());
    for (entity, layers) in &replaced {
        set_immersive_hidden_layer(&mut commands, entity, layers, active);
    }
    // The classic clear sprites are opaque erasers. Hiding only the white
    // eraser would resurrect the erased HUD. Apply the same source-space
    // mask to the underlying HUD entities before parking the eraser itself.
    let mut cleared = shell.battle_fainted_hud;
    if let Some(animation) = shell.visible_move_animations.front().filter(|animation| {
        animation.started && shell.runtime.data().moves.contains_key(&animation.move_id)
    }) {
        cleared[usize::from(!animation.player_move)] = true;
    }
    for (entity, transform, source, layers) in &hud {
        let source_transform = source.map_or(*transform, |source| source.0);
        if active
            && immersive_battle_hud_is_erased(source_transform.translation.truncate(), cleared)
        {
            set_immersive_hidden_layer(&mut commands, entity, layers, true);
        } else if replaced.get(entity).is_err() {
            set_immersive_hidden_layer(&mut commands, entity, layers, false);
        }
    }
}

fn set_immersive_hidden_layer(
    commands: &mut Commands,
    entity: Entity,
    current: Option<&bevy::render::view::RenderLayers>,
    hidden: bool,
) {
    let hidden_layer = bevy::render::view::RenderLayers::layer(
        crystal_voxel_view::HIDDEN_CLASSIC_WORLD_RENDER_LAYER,
    );
    if hidden && current != Some(&hidden_layer) {
        commands.entity(entity).insert(hidden_layer);
    } else if !hidden && current.is_some() {
        commands
            .entity(entity)
            .remove::<bevy::render::view::RenderLayers>();
    }
}

fn immersive_composite_clear_color(
    world_active: bool,
    battle_active: bool,
) -> bevy::render::camera::ClearColorConfig {
    if world_active || battle_active {
        bevy::render::camera::ClearColorConfig::None
    } else {
        bevy::render::camera::ClearColorConfig::Default
    }
}

fn immersive_battle_hud_is_erased(point: Vec2, cleared: [bool; 2]) -> bool {
    [
        (cleared[0], 9.0, 7.0, 11.0, 5.0),
        (cleared[1], 1.0, 0.0, 10.0, 4.0),
    ]
    .into_iter()
    .any(|(cleared, left, top, width, height)| {
        let (x, y) = field_window_center(left, top, width, height);
        let extent = Vec2::new(width, height) * TILE_SIZE * 0.5;
        cleared && (point - Vec2::new(x, y)).abs().cmple(extent).all()
    })
}

/// Early title, transition and post-capture Dex branches can return before
/// the ordinary scene extractor. Explicitly retire the arena in those states.
fn clear_inactive_visual_battle(
    mut frame: ResMut<VisualBattleFrame>,
    rendered: Res<RenderedViewport>,
    shell: Res<BevyRuntimeShell>,
) {
    if rendered.title_active
        || shell.visible_battle_transition.is_some()
        || (shell.pokedex_menu_open
            && shell.pokedex_scripted_entry
            && shell.pending_standard_capture.is_some())
    {
        if frame.active {
            *frame = VisualBattleFrame::default();
        }
    }
}

#[cfg(test)]
mod immersive_battle_bridge_tests {
    use super::*;
    #[test]
    fn source_palette_cache_keeps_luminance_order_and_transparency() {
        let pixels = [
            0, 0, 0, 255, 180, 160, 140, 255, 80, 70, 60, 255, 200, 200, 200, 0, 0, 0, 0, 255,
        ];
        let colors = immersive_source_palette(&pixels);
        assert_eq!(colors[0], [1.0; 4]);
        assert_eq!(
            colors[1],
            [180.0 / 255.0, 160.0 / 255.0, 140.0 / 255.0, 1.0]
        );
        assert_eq!(colors[2], [80.0 / 255.0, 70.0 / 255.0, 60.0 / 255.0, 1.0]);
        assert_eq!(colors[3], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(immersive_source_palette(&[]), [[1.0; 4]; 4]);
    }
    #[test]
    fn immersive_battle_compositor_preserves_active_modeled_overworld() {
        assert!(matches!(
            immersive_composite_clear_color(true, false),
            bevy::render::camera::ClearColorConfig::None
        ));
    }
    #[test]
    fn immersive_battle_compositor_preserves_active_modeled_battle() {
        assert!(matches!(
            immersive_composite_clear_color(false, true),
            bevy::render::camera::ClearColorConfig::None
        ));
    }
    #[test]
    fn immersive_battle_compositor_clears_stale_pixels_when_both_underlays_stop() {
        for (world, battle) in [
            (true, false),
            (false, true),
            (false, false),
            (false, true),
            (false, false),
        ] {
            let mut camera = Camera::default();
            camera.clear_color = immersive_composite_clear_color(world, battle);
            assert_eq!(
                matches!(
                    camera.clear_color,
                    bevy::render::camera::ClearColorConfig::Default
                ),
                !world && !battle
            );
        }
    }
    #[test]
    fn arena_selection_is_presentation_only() {
        assert_eq!(
            immersive_battle_environment("IcePathB1F", "CAVE"),
            VisualBattleEnvironment::Ice
        );
        assert_eq!(
            immersive_battle_environment("IlexForest", "ROUTE"),
            VisualBattleEnvironment::Forest
        );
        assert_eq!(
            immersive_battle_environment("Route29", "ROUTE"),
            VisualBattleEnvironment::Meadow
        );
        assert_eq!(
            immersive_battle_environment("WhirlIslandB2F", "CAVE"),
            VisualBattleEnvironment::Cave
        );
    }
    #[test]
    fn cue_progress_is_bounded_even_for_empty_or_finished_animation() {
        assert_eq!(immersive_battle_progress(30, 60), 0.5);
        assert_eq!(immersive_battle_progress(0, 0), 0.0);
        assert_eq!(immersive_battle_progress(90, 60), 1.0);
    }
}

/// Opt-in fresh-session native preview fixture. It seeds only this disposable
/// location-test session, then reaches battle commands through the production
/// controller. Normal play, loaded saves and the battle renderer never call it.
#[cfg(feature = "location-tester")]
fn prepare_immersive_battle_preview(
    mut shell: BevyRuntimeShell,
    shadow_ball: bool,
    psychic: bool,
    hyper_beam: bool,
) -> Result<BevyRuntimeShell> {
    anyhow::ensure!(
        [shadow_ball, psychic, hyper_beam]
            .into_iter()
            .filter(|active| *active)
            .count()
            <= 1,
        "battle preview move fixtures are mutually exclusive"
    );
    complete_visible_smoke_player_name_if_needed(&mut shell, Some("CHRIS"))?;
    let initial = shell.shell.snapshot()?;
    anyhow::ensure!(
        initial.overworld.map_name == "Route36" && initial.party.slots.is_empty(),
        "immersive battle preview requires a fresh empty-party Route36 session"
    );
    let trainer = initial.trainer;
    let move_fixture = if shadow_ball {
        Some(("GENGAR", 40, "TM_SHADOW_BALL", "SHADOW_BALL"))
    } else if psychic {
        Some(("KADABRA", 20, "TM_PSYCHIC_M", "PSYCHIC_M"))
    } else if hyper_beam {
        // Sudowoodo resists this level-20 Normal actor's Hyper Beam, keeping
        // the opponent alive so the next real turn can exercise recharge.
        Some(("RATICATE", 20, "TM_HYPER_BEAM", "HYPER_BEAM"))
    } else {
        None
    };
    if let Some((species, level, tm, move_name)) = move_fixture {
        shell.shell.add_party_pokemon(
            species,
            level,
            None,
            None,
            &trainer.player_name,
            trainer.player_id,
            Dv::from_non_hp(9, 9, 9, 9),
        )?;
        // Let the real item mutation validate compatibility and set the move's
        // legal PP. Never inject a fabricated move or mutate a user save.
        shell.shell.add_bag_item(tm, 1)?;
        shell.shell.use_bag_tmhm_on_party_pokemon(tm, 0, Some(0))?;
        let snapshot = shell.shell.snapshot()?;
        anyhow::ensure!(
            snapshot.party.slots[0].pokemon.moves[0].name == move_name,
            "pack did not teach {move_name} in the preview's first move slot"
        );
    } else {
        shell.shell.add_party_pokemon(
            "CYNDAQUIL",
            10,
            None,
            None,
            &trainer.player_name,
            trainer.player_id,
            Dv::from_non_hp(9, 9, 9, 9),
        )?;
        // Keep the legal level-10 move list (Tackle first) while giving this
        // disposable QA actor enough health to exercise menus and several turns.
        let low_level_moves = {
            let state = shell.shell.session_mut().state_mut();
            let moves = state.storage.party.pokemon[0]
                .take()
                .context("preview lead")?
                .moves;
            state.sync_party_from_storage();
            moves
        };
        shell.shell.add_party_pokemon(
            "CYNDAQUIL",
            40,
            None,
            None,
            &trainer.player_name,
            trainer.player_id,
            Dv::from_non_hp(9, 9, 9, 9),
        )?;
        {
            let state = shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0]
                .as_mut()
                .context("preview replacement lead")?
                .moves = low_level_moves;
            state.sync_party_from_storage();
        }
    }
    shell.shell.add_party_pokemon(
        "TOTODILE",
        35,
        None,
        None,
        &trainer.player_name,
        trainer.player_id,
        Dv::from_non_hp(9, 9, 9, 9),
    )?;
    for (item, quantity) in [("POKE_BALL", 5), ("POTION", 5), ("MASTER_BALL", 1)] {
        shell.shell.add_bag_item(item, quantity)?;
    }
    // This disposable developer fixture also exercises the real new-entry Dex
    // transition after capture. It never loads or changes a user save.
    shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_engine_flag("ENGINE_POKEDEX", true)?;
    mark_runtime_snapshot_dirty(&mut shell);
    settle_visible_shell_smoke_until_idle(&mut shell)?;
    shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)?;
    prepare_visible_battle_entry(&mut shell)?;
    let mut controller = VisibleShellController { shell };
    for _ in 0..64 {
        controller.wait_frames(1)?;
        let ready = controller.shell.battle_messages.is_empty()
            && controller.shell.visible_battle_transition.is_none()
            && controller.shell.visible_battle_sliding_intro.is_none()
            && controller.shell.visible_send_out_animation.is_none()
            && controller.shell.battle_action_cursor.is_some();
        if ready {
            if hyper_beam {
                // Reproducible developer DIV stimuli, using the same LFSR as
                // controller regressions. This is not captured cartridge
                // timing: normal play retains its live divider and the core
                // still performs its real accuracy, damage and recharge logic.
                let mut divider_state = 0xc5afu16;
                let samples = (0..16_384).map(|_| {
                    let feedback = divider_state & 1;
                    divider_state >>= 1;
                    if feedback != 0 {
                        divider_state ^= 0xb400;
                    }
                    divider_state as u8
                });
                let session = controller.shell.shell.session_mut();
                session.state_mut().random_state = Default::default();
                *session.divider_mut_for_tests() =
                    crystal_core::random::RuntimeDividerSource::replay(samples);
            }
            mark_runtime_snapshot_dirty(&mut controller.shell);
            return Ok(controller.shell);
        }
        controller.press(GameButton::A)?;
    }
    anyhow::bail!("production battle introduction did not reach a command menu in preview fixture")
}

#[derive(Component, Clone, Copy)]
struct ImmersiveBattleUiSource(Transform);
#[derive(Component)]
struct ImmersiveBattleHudBacking {
    side: usize,
    border: bool,
}

/// Re-layout the exact production sprites, never their values or input state.
/// Run after fullscreen parenting so compact battle panels do not inherit the
/// fullscreen LCD zoom. Stored source transforms also make F3 reversible.
fn sync_immersive_battle_ui_layout(
    mut commands: Commands,
    status: Res<crystal_voxel_view::BattleViewStatus>,
    shell: Res<BevyRuntimeShell>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut ui: Query<
        (
            Entity,
            &Sprite,
            &mut Transform,
            Option<&ImmersiveBattleUiSource>,
            Option<&BattleHudMarker>,
        ),
        (
            Or<(
                With<BattleCommandMarker>,
                With<BattleHudMarker>,
                With<BattleWindowFrameMarker>,
            )>,
            Without<FixedBattleCanvasMarker>,
            Without<BattleBattlerMarker>,
            Without<ImmersiveBattleReplaced>,
            Without<ImmersiveBattleHudBacking>,
        ),
    >,
    mut panels: Query<
        (
            &ImmersiveBattleHudBacking,
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
        ),
        With<ImmersiveBattleHudBacking>,
    >,
    mut initialized: Local<bool>,
) {
    if !*initialized {
        for side in 0..2 {
            for border in [true, false] {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: if border {
                                Color::srgb(0.11, 0.17, 0.18)
                            } else {
                                Color::srgb(0.93, 0.94, 0.83)
                            },
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    ImmersiveBattleHudBacking { side, border },
                ));
            }
        }
        *initialized = true;
    }
    let full_modal = shell.battle_switch_cursor.is_some()
        || shell.battle_party_summary_open
        || shell.battle_pack_target_mode.is_some()
        || shell.bag_cursor.is_some()
        || shell.ball_cursor.is_some()
        || shell.key_item_cursor.is_some()
        || shell.tmhm_cursor.is_some()
        || shell
            .battle_level_stats
            .front()
            .is_some_and(|stats| stats.active);
    let active = status.active && !full_modal;
    if !active {
        for (entity, _, mut transform, source, _) in &mut ui {
            if let Some(source) = source {
                *transform = source.0;
                commands
                    .entity(entity)
                    .remove::<ImmersiveBattleUiSource>()
                    .remove::<bevy::render::view::RenderLayers>();
            }
        }
        for (_, _, _, mut visibility) in &mut panels {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    let Ok(window) = windows.get_single() else {
        return;
    };
    let physical = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    if physical.min_element() <= 0.0 {
        return;
    }
    #[cfg(feature = "fullscreen-scaling")]
    let pixels_per_unit =
        fullscreen_pixels_per_world_unit(physical, window.scale_factor()) * window.scale_factor();
    #[cfg(not(feature = "fullscreen-scaling"))]
    let pixels_per_unit = (physical / Vec2::new(640.0, 576.0)).min_element();
    let view = physical / pixels_per_unit;
    let preferred = ((view.x - 48.0) / 720.0).min(1.0);
    // Keep source glyphs at about 24 CSS pixels even when the host was
    // compiled without fullscreen-scaling and its LCD camera magnifies units.
    let source_pixels = (4.0 * pixels_per_unit * preferred)
        .min(3.0 * window.scale_factor())
        .floor()
        .max(1.0);
    let scale = (source_pixels / (4.0 * pixels_per_unit)).min(preferred);
    let mut bounds = [None::<Rect>; 2];
    for (_, sprite, transform, source, hud) in &ui {
        if hud.is_none() {
            continue;
        }
        let base = source.map_or(*transform, |source| source.0);
        let side = usize::from(base.translation.y > PLAYFIELD_TOP - TILE_SIZE * 4.5);
        let center = base.translation.truncate();
        let half =
            sprite.custom_size.unwrap_or(Vec2::splat(TILE_SIZE)) * base.scale.truncate() * 0.5;
        let rect = Rect::from_corners(center - half, center + half);
        bounds[side] = Some(bounds[side].map_or(rect, |previous| {
            Rect::from_corners(previous.min.min(rect.min), previous.max.max(rect.max))
        }));
    }
    let panel_center = |side: usize, rect: Rect| {
        let size = rect.size() * scale;
        Vec2::new(
            if side == 0 {
                -view.x * 0.5 + 32.0 + size.x * 0.5
            } else {
                view.x * 0.5 - 32.0 - size.x * 0.5
            },
            view.y * 0.5 - 32.0 - size.y * 0.5,
        )
    };
    let main_menu = shell.battle_action_cursor.is_some()
        && shell.battle_move_cursor.is_none()
        && shell.battle_messages.is_empty()
        && !visible_battle_command_animation_active(&shell)
        && shell.battle_faint_prompt_cursor.is_none()
        && shell.battle_shift_prompt_cursor.is_none()
        && ui.iter().any(|(_, sprite, transform, source, hud)| {
            let base = source.map_or(*transform, |source| source.0);
            hud.is_none()
                && sprite.custom_size == Some(Vec2::new(384.0, 192.0))
                && (base.translation.x - 128.0).abs() < 0.01
        });
    let command_anchor = if main_menu {
        Vec2::new(128.0, -192.0)
    } else {
        Vec2::new(0.0, -192.0)
    };
    let command_center = Vec2::new(0.0, -view.y * 0.5 + 24.0 + 96.0 * scale);
    for (entity, sprite, mut transform, source, hud) in &mut ui {
        let base = source.map_or(*transform, |source| source.0);
        if source.is_none() {
            commands
                .entity(entity)
                .insert(ImmersiveBattleUiSource(base));
        }
        commands.entity(entity).remove_parent();
        *transform = base;
        if hud.is_some() {
            let side = usize::from(base.translation.y > PLAYFIELD_TOP - TILE_SIZE * 4.5);
            if let Some(rect) = bounds[side] {
                let point = panel_center(side, rect)
                    + (base.translation.truncate() - rect.center()) * scale;
                transform.translation.x = point.x;
                transform.translation.y = point.y;
                transform.scale = base.scale * Vec3::new(scale, scale, 1.0);
            }
        } else {
            // The current actor is already named in its HUD. The cartridge's
            // seven-column duplicate prompt splits long names mid-word; omit
            // that redundant panel while retaining every actual menu choice.
            let omit = main_menu
                && (base.translation.x < -64.0
                    || sprite.custom_size.is_some_and(|size| size.x > 384.1));
            if omit {
                commands
                    .entity(entity)
                    .insert(bevy::render::view::RenderLayers::layer(
                        crystal_voxel_view::HIDDEN_CLASSIC_WORLD_RENDER_LAYER,
                    ));
            } else {
                commands
                    .entity(entity)
                    .remove::<bevy::render::view::RenderLayers>();
            }
            let point = command_center + (base.translation.truncate() - command_anchor) * scale;
            transform.translation.x = point.x;
            transform.translation.y = point.y;
            transform.scale = base.scale * Vec3::new(scale, scale, 1.0);
        }
    }
    let mut cleared = shell.battle_fainted_hud;
    if let Some(animation) = shell.visible_move_animations.front().filter(|animation| {
        animation.started && shell.runtime.data().moves.contains_key(&animation.move_id)
    }) {
        cleared[usize::from(!animation.player_move)] = true;
    }
    for (panel, mut sprite, mut transform, mut visibility) in &mut panels {
        let Some(rect) = bounds[panel.side] else {
            *visibility = Visibility::Hidden;
            continue;
        };
        *visibility = if cleared[panel.side] {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        let center = panel_center(panel.side, rect);
        transform.translation =
            Vec3::new(center.x, center.y, if panel.border { 3.20 } else { 3.21 });
        sprite.custom_size =
            Some(rect.size() * scale + Vec2::splat(if panel.border { 28.0 } else { 20.0 }));
    }
}
