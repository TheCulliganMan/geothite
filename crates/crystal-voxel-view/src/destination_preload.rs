//! Sliding outdoor coverage and hidden destinations through the production mesher.
//! The host supplies read-only source art; no game session is advanced here.
use super::*;
use std::sync::Arc;

#[path = "world_stream_retention.rs"]
mod retention;

/// Speculative work must have a real CPU worker, never a frame-thread fallback.
pub fn supported() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    { true }
    #[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
    { terrain_worker::browser::availability() == terrain_worker::browser::Availability::Ready }
    #[cfg(all(target_arch = "wasm32", not(feature = "terrain-worker-browser")))]
    { false }
}

#[derive(Resource, Default)]
pub struct DestinationPreload {
    pub frame: Option<VisualWorldFrame>,
    /// Sliding outdoor coverage prepared independently of doors and map edges.
    pub stream_frame: Option<VisualWorldFrame>,
    /// Resolved presentation identities only; never live game objects.
    pub actor_sources: Vec<std::sync::Arc<str>>,
    /// Completed hidden scenes promoted to the active renderer.
    pub hits: u64,
}

#[derive(Resource, Default)]
pub(super) struct PreparedDestination {
    cache: TerrainRevisionCache,
    builds: TerrainBuildQueue,
    ready_frames: u8,
    source_frame: Option<Arc<VisualWorldFrame>>,
    source_compatibility: Option<(TerrainCacheKey, AssetId<Image>, bool)>,
    live_compatibility: Option<(TerrainCacheKey, AssetId<Image>, bool)>,
    settled_key: Option<(TerrainCacheKey, VoxelCameraControls, Option<Vec3>, u32)>,
}

#[derive(Resource, Default)]
pub(super) struct PreparedWorld {
    next: PreparedDestination,
    retained: retention::RetainedWindows,
}

pub(super) fn cached_compatible(
    source: &VisualWorldFrame,
    live: &VisualWorldFrame,
    receipt: &mut Option<(TerrainCacheKey, AssetId<Image>, bool)>,
) -> bool {
    let key = TerrainCacheKey::from_frame(live);
    let texture = live.map_texture.id();
    if let Some((previous, image, result)) = receipt
        && *previous == key
        && *image == texture
    {
        return *result;
    }
    let result = (source.map_texture == live.map_texture
        && TerrainCacheKey::from_frame(source) == key)
        || compatible(source, live);
    *receipt = Some((key, texture, result));
    result
}

pub(super) fn roots(cache: &TerrainRevisionCache) -> impl Iterator<Item = Entity> {
    [
        cache.instances_root,
        cache.textured_entity,
        cache.solid_entity,
        cache.animated_textured_entity,
        cache.animated_solid_entity,
    ]
    .into_iter()
    .flatten()
}

pub(super) fn hide(cache: &TerrainRevisionCache, commands: &mut Commands) {
    for root in roots(cache) {
        commands.entity(root).insert(Visibility::Hidden);
        #[cfg(not(target_arch = "wasm32"))]
        commands.entity(root).insert(pipeline_preload::HiddenDestination);
    }
}

pub(super) fn compatible(built: &VisualWorldFrame, live: &VisualWorldFrame) -> bool {
    if !terrain_tracking::can_reuse(built, live) { return false; }
    let delta = live.grid_origin - built.grid_origin
        - built.connected_source_shift(live).unwrap_or(IVec2::ZERO);
    let width = built.grid_size.x as i32;
    let height = built.grid_size.y as i32;
    live.tiles.iter().all(|tile| {
        let p = IVec2::new(tile.column as i32, tile.row as i32) + delta;
        if p.x < 0 || p.y < 0 || p.x >= width || p.y >= height { return true; }
        let old = &built.tiles[(p.y * width + p.x) as usize];
        old.texture == tile.texture || (old.animation_frames.is_some()
            && old.animation_frames == tile.animation_frames)
    })
}

pub(super) fn activate(
    frame: &VisualWorldFrame,
    profiles: &live_profiles::LiveProfiles,
    destination: &mut PreparedDestination,
    request: &mut DestinationPreload,
    live: &mut TerrainRevisionCache,
    builds: &mut TerrainBuildQueue,
    commands: &mut Commands,
) {
    if live.built_frame.as_ref().is_some_and(|old| old.map_id == frame.map_id) { return; }
    if live.key.as_ref().is_some_and(|key| key.profiles_revision == profiles.revision)
        && retain_connected_terrain(frame, live)
    {
        // The camera is still inside resident world coverage. Crossing a
        // logical map edge must not exchange meshes, atlases or render roots.
        return;
    }
    // A resident base scene is reusable even while its grove pages settle.
    // The active streamer can finish those pages without rebuilding the town.
    let ready = destination.cache.uploads.ready()
        && destination.cache.key.as_ref().is_some_and(|key| key.profiles_revision == profiles.revision)
        && destination.cache.built_frame.as_ref().is_some_and(|old| compatible(old, frame));
    let in_flight = destination.builds.key.as_ref().is_some_and(|key|
        key.profiles_revision == profiles.revision && key.map_id == frame.map_id)
        && destination.source_frame.as_ref().is_some_and(|old| compatible(old, frame));
    if !ready && !in_flight { return; }
    crate::terrain_publication::discard(&mut builds.publication, commands);
    crate::terrain_publication::discard(&mut destination.builds.publication, commands);
    if in_flight && !ready && retain_connected_terrain(frame, live) {
        // Adopt only the work queue. The completed old scene covers the shared
        // authored acreage and keeps its immutable atlas until publication.
        // Swapping an empty pending cache here used to expose the classic view.
        let started = builds.started;
        std::mem::swap(builds, &mut destination.builds);
        builds.started = started;
        destination.builds = TerrainBuildQueue::default();
        destination.source_frame = None;
        destination.settled_key = None;
        request.hits += 1;
        return;
    }
    // Keep the old room resident for the return journey. Both scenes retain
    // their own immutable atlas; the host's live compositor never aliases it.
    hide(live, commands);
    std::mem::swap(live, &mut destination.cache);
    if ready { bind_preloaded_compositor(frame, live); }
    let started = builds.started;
    if in_flight && !ready {
        // A fast exit adopts the already-running task instead of throwing it
        // away and starting the same town from scratch at the doorway.
        std::mem::swap(builds, &mut destination.builds);
        builds.started = started;
    } else {
        *builds = TerrainBuildQueue { started, ..default() };
    }
    destination.builds = TerrainBuildQueue::default();
    destination.source_frame = None;
    destination.ready_frames = 3;
    destination.settled_key = None;
    for root in roots(live) {
        commands.entity(root).insert(Visibility::Visible);
        #[cfg(not(target_arch = "wasm32"))]
        commands.entity(root).remove::<pipeline_preload::HiddenDestination>();
    }
    request.hits += 1;
}

pub(super) fn activate_world(
    frame: &VisualWorldFrame,
    profiles: &live_profiles::LiveProfiles,
    destination: &mut PreparedWorld,
    status: &mut crate::WorldStreamStatus,
    live: &mut TerrainRevisionCache,
    builds: &mut TerrainBuildQueue,
    commands: &mut Commands,
) {
    let Some(resident) = live.built_frame.as_ref() else {
        return;
    };
    if !crate::sky::outdoors(frame) || !world_stream::needs_refresh(resident, frame) {
        return;
    }
    let retained = destination.retained.take_ready(resident, frame, profiles);
    let from_retained = retained.is_some();
    let mut next = if let Some(cache) = retained {
        cache
    } else {
        let prepared = &mut destination.next;
        let ready = prepared.ready_frames >= 3
            && prepared.cache.uploads.ready()
            && prepared
                .cache
                .key
                .as_ref()
                .is_some_and(|key| key.profiles_revision == profiles.revision)
            && prepared.cache.built_frame.as_ref().is_some_and(|built| {
                world_stream::advances_coverage(resident, built, frame) && compatible(built, frame)
            });
        if !ready {
            return;
        }
        crate::terrain_publication::discard(&mut prepared.builds.publication, commands);
        prepared.builds = TerrainBuildQueue::default();
        prepared.source_frame = None;
        prepared.source_compatibility = None;
        prepared.live_compatibility = None;
        prepared.ready_frames = 0;
        prepared.settled_key = None;
        std::mem::take(&mut prepared.cache)
    };
    let _trace = loading_trace::span("world.window_promote", || {
        serde_json::json!({
            "map": frame.map_id.as_ref(),
            "resident_origin": live.built_frame.as_ref().map(|built| built.grid_origin.to_array()),
            "next_origin": next.built_frame.as_ref().map(|built| built.grid_origin.to_array()),
            "old_roots": roots(live).map(|entity| entity.to_bits()).collect::<Vec<_>>(),
            "next_roots": roots(&next).map(|entity| entity.to_bits()).collect::<Vec<_>>(),
            "retained": from_retained,
        })
    });
    hide(live, commands);
    std::mem::swap(live, &mut next);
    bind_preloaded_compositor(frame, live);
    destination
        .retained
        .remember(next, "previously_visible", commands);
    let started = builds.started;
    crate::terrain_publication::discard(&mut builds.publication, commands);
    *builds = TerrainBuildQueue {
        started,
        ..default()
    };
    for root in roots(live) {
        commands.entity(root).insert(Visibility::Visible);
        #[cfg(not(target_arch = "wasm32"))]
        commands
            .entity(root)
            .remove::<pipeline_preload::HiddenDestination>();
    }
    status.promotions += 1;
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    request: Res<DestinationPreload>,
    current: Res<VisualWorldFrame>,
    battle: Option<Res<crystal_render_api::VisualBattleFrame>>,
    settings: Res<VoxelViewSettings>,
    cameras: Query<&Projection, With<VoxelWorldCamera>>,
    profiles: Res<live_profiles::LiveProfiles>,
    sun: Query<
        (&DirectionalLight, &Transform),
        (
            With<VoxelSun>,
            Without<VoxelTerrain>,
            Without<VoxelActorCard>,
        ),
    >,
    active: Res<TerrainRevisionCache>,
    #[cfg(not(target_arch = "wasm32"))] active_builds: Res<TerrainBuildQueue>,
    prepared: (ResMut<PreparedDestination>, Option<ResMut<PreparedWorld>>),
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
) {
    if !settings.enabled || !current.active {
        return;
    }
    // Finished speculative work can wait; GPU publication must not interrupt
    // the authored field wipe or the battle's own preparation/handoff.
    if battle
        .as_ref()
        .is_some_and(|frame| frame.active || frame.preparing)
    {
        return;
    }
    // A second browser worker can prepare coverage while the first scene
    // initializes, as native workers do. Waiting for visible 3D here loses
    // the entire initial lookahead and can deadlock recovery of a late window.
    // Never run a blocking fallback mesher as speculative work on the browser
    // frame thread. A worker-capable browser uses the same paged path as play.
    #[cfg(target_arch = "wasm32")]
    {
        #[cfg(feature = "terrain-worker-browser")]
        if terrain_worker::browser::availability() != terrain_worker::browser::Availability::Ready {
            return;
        }
        #[cfg(not(feature = "terrain-worker-browser"))]
        return;
    }
    let (mut destination, mut world) = prepared;
    let lanes = [
        world.as_mut().map(|world| {
            let world = &mut **world;
            (
                request.stream_frame.as_ref(),
                &mut world.next,
                Some(&mut world.retained),
                true,
            )
        }),
        Some((request.frame.as_ref(), &mut *destination, None, false)),
    ];
    for (requested, destination, mut retained_windows, streaming) in lanes.into_iter().flatten() {
        let Some(requested) = requested else {
            continue;
        };
        #[cfg(all(target_arch = "wasm32", feature = "terrain-worker-browser"))]
        if destination.cache.built_frame.is_none()
            && destination.builds.pending.is_none()
            && (terrain_worker::browser::busy()
                || terrain_worker::browser::availability()
                    != terrain_worker::browser::Availability::Ready)
        {
            // Do not repeatedly clone a full source window while its worker
            // is unavailable. Existing jobs still poll and ready scenes
            // can still finish their GPU/grove publication.
            continue;
        }
        if !streaming && requested.map_id == current.map_id {
            continue;
        }
        #[cfg(target_arch = "wasm32")]
        if !streaming && crate::sky::outdoors(&current) && crate::sky::outdoors(requested)
        {
            // Connected outdoor acreage belongs to the progressive lane,
            // including the frame before its first request is published.
            // Starting a second border job here occupies the worker needed
            // for continuous coverage throughout the initial build.
            continue;
        }
        // A moving camera must not cancel compatible speculative work every
        // time its lookahead advances or the source coordinate basis changes.
        let retained = if streaming {
            destination.source_frame.as_ref().and_then(|old| {
                let request_compatible =
                    cached_compatible(old, requested, &mut destination.source_compatibility);
                // Finish useful late work even after the lookahead has moved
                // farther ahead. Requiring it to cover the newest future
                // viewport cancels a moving player's work before publication.
                let useful_now = active.built_frame.as_ref().is_some_and(|resident| {
                    world_stream::advances_coverage(resident, old, &current)
                        && cached_compatible(old, &current, &mut destination.live_compatibility)
                });
                let publishing = destination.cache.built_frame.is_some()
                    && destination.ready_frames < 3;
                let publication_covers_live = publishing
                    && cached_compatible(old, &current, &mut destination.live_compatibility);
                // Finish CPU/GPU publication before retargeting. Once ready,
                // keep this bridge in retained coverage and start the farther
                // request immediately, rather than waiting for promotion.
                (((destination.builds.key.is_some() || publishing)
                    && (request_compatible || useful_now))
                    || publication_covers_live
                    || (world_stream::same_window(old, requested) && request_compatible))
                    .then(|| Arc::clone(old))
            })
        } else {
            None
        };
        if streaming && retained.is_none()
            && let Some(old) = destination.source_frame.as_ref()
            && !world_stream::same_window(old, requested)
        {
            let _retarget = loading_trace::span("world.preload_retarget", || {
                serde_json::json!({"map": current.map_id.as_ref(),
                    "source_map": old.map_id.as_ref(), "source_origin": old.grid_origin.to_array(),
                    "requested_map": requested.map_id.as_ref(), "requested_origin": requested.grid_origin.to_array(),
                    "publishing": destination.cache.built_frame.is_some(),
                    "live_compatible": compatible(old, &current),
                    "connected_rejection": (old.map_id != current.map_id).then(||
                        terrain_tracking::connected_frame_shift(old, &current).err()).flatten()})
            });
        }
        let frame = retained.as_deref().unwrap_or(requested);
        let mut key = TerrainCacheKey::from_frame(frame);
        key.profiles_revision = profiles.revision;
        let sunlight = sun.iter().find_map(|(light, transform)| {
            (light.shadow_maps_enabled && light.illuminance > 0.)
                .then(|| -transform.forward().as_vec3())
        });
        let PreparedDestination {
            cache,
            builds,
            ready_frames,
            settled_key,
            source_frame,
            source_compatibility,
            live_compatibility,
        } = &mut *destination;
        // Only streamed groves depend on the sun's caster footprint. An indoor
        // destination must not wake up whenever the outdoor daylight moves.
        // The classic playfield's aspect differs from a native window or
        // capture target. Use the same actual camera projection as live streaming.
        let aspect = cameras
            .iter()
            .find_map(|projection| match projection {
                Projection::Perspective(p) if p.aspect_ratio.is_finite() && p.aspect_ratio > 0. => {
                    Some(p.aspect_ratio)
                }
                _ => None,
            })
            .unwrap_or(frame.viewport_size.x / frame.viewport_size.y);
        let next = (
            key,
            settings.camera,
            if cache.groves.is_some() {
                sunlight
            } else {
                None
            },
            aspect.to_bits(),
        );
        // A fully prepared hidden scene must stay untouched. Re-entering terrain
        // sync made its roots visible then hidden again on every indoor frame,
        // needlessly invalidating Bevy visibility and rescanning grove interest.
        if *ready_frames >= 3 && settled_key.as_ref() == Some(&next) {
            continue;
        }

        if cache.built_frame.as_ref().is_some_and(|old| {
            if streaming {
                !world_stream::same_window(old, frame) || !compatible(old, frame)
            } else {
                old.map_id == frame.map_id && !compatible(old, frame)
            }
        }) {
            let stone = cache.painted_stone.clone();
            let bedrock = cache.painted_bedrock.clone();
            let previous = std::mem::replace(
                cache,
                TerrainRevisionCache {
                    painted_stone: stone,
                    painted_bedrock: bedrock,
                    ..default()
                },
            );
            if *ready_frames >= 3
                && let Some(windows) = retained_windows.as_mut()
            {
                // A ready nearby window can bridge the camera to the farther
                // lookahead. Do not throw it away before it becomes useful.
                windows.remember(previous, "lookahead_advanced", &mut commands);
            } else {
                for root in roots(&previous) {
                    commands.entity(root).despawn();
                }
                crate::terrain_disposal::retire(previous);
            };
            crate::terrain_publication::discard(&mut builds.publication, &mut commands);
            crate::terrain_disposal::retire(std::mem::take(builds));
            *ready_frames = 0;
            *source_compatibility = None;
            *live_compatibility = None;
        }

        if cache.built_frame.is_none() && builds.key.is_none() {
            cache.forest_library = active.forest_library.clone();
            // Join the foreground's immutable image work before it has handles.
            // Each scene still owns its independent geometry task and atlas.
            #[cfg(not(target_arch = "wasm32"))]
            {
                builds.painted = active_builds.painted.clone();
            }
        }
        if cache.painted_stone.is_none() {
            cache.painted_stone = active.painted_stone.clone();
        }
        if cache.painted_bedrock.is_none() {
            cache.painted_bedrock = active.painted_bedrock.clone();
        }
        let previous_started = builds.started;
        cache.uploads.require_gpu(active.uploads.gpu_required());
        builds.speculative = true;
        builds.defer_to_world_preload = false;
        let terrain_state = sync_terrain(
            frame,
            &profiles,
            &mut commands,
            cache,
            builds,
            &mut images,
            &mut meshes,
            &mut materials,
            &mut entities,
        );
        if builds.started != previous_started {
            *source_frame = Some(Arc::new(frame.clone()));
            *source_compatibility = None;
            *live_compatibility = None;
            let _submit = loading_trace::span("world.preload_submit", || {
                serde_json::json!({"streaming": streaming,
                    "current_map": current.map_id.as_ref(), "map": frame.map_id.as_ref(),
                    "origin": frame.grid_origin.to_array()})
            });
        }
        match terrain_state {
            Ok(TerrainSyncState::Ready) => {
                let pose = settings.camera.world_pose(frame, &cache.footing_heights);
                let projection =
                    Mat4::perspective_rh(pose.vertical_fov_radians, aspect, pose.near, pose.far);
                let matrix = projection * Mat4::look_at_rh(pose.eye, pose.target, pose.up);
                if grove_stream::sync(
                    frame,
                    matrix,
                    sunlight,
                    cache,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                ) {
                    *ready_frames = 0;
                } else {
                    *ready_frames = ready_frames.saturating_add(1);
                    if *ready_frames == 3 {
                        let _ready = loading_trace::span("world.preload_ready", || {
                            serde_json::json!({"streaming": streaming,
                                "current_map": current.map_id.as_ref(), "map": frame.map_id.as_ref(),
                                "origin": frame.grid_origin.to_array()})
                        });
                    }
                }
            }
            Ok(TerrainSyncState::Pending) => *ready_frames = 0,
            Err(error) => {
                *ready_frames = 0;
                bevy::log::warn!("destination terrain preload failed: {error:?}");
            }
        }
        *settled_key = (*ready_frames >= 3).then_some(next);
        // This is queued after sync, including for roots spawned in that call.
        // Mesh/material asset preparation still uploads the hidden scene to GPU.
        hide(cache, &mut commands);
    }
}

// Player support can temporarily leave a late resident window. That must not
// stop the very worker/upload publication needed to recover its coverage.
pub(super) fn pending_world_covers(
    frame: &VisualWorldFrame,
    profiles: &live_profiles::LiveProfiles,
    world: &mut PreparedWorld,
) -> bool {
    let next = &mut world.next;
    let key = next.builds.key.as_ref().or(next.cache.key.as_ref());
    let pending = next.builds.key.is_some()
        || (next.cache.built_frame.is_some() && next.ready_frames < 3);
    crate::sky::outdoors(frame) && pending
        && key.is_some_and(|key| key.profiles_revision == profiles.revision)
        && next.source_frame.as_ref().is_some_and(|source| {
            cached_compatible(source, frame, &mut next.live_compatibility)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    include!("world_stream_residency_tests.rs");
    #[test]
    fn destination_rejects_recolored_source_even_when_geometry_is_reusable() {
        use crystal_render_api::{VisualTile, VisualTileSource};
        let mut images = Assets::<Image>::default();
        let texture = images.add(Image::default());
        let frame = VisualWorldFrame { active: true, map_id: "NewBarkTown".into(),
            grid_size: UVec2::ONE, tile_size: Vec2::splat(8.), viewport_size: Vec2::splat(8.),
            tiles: vec![VisualTile { column: 0, row: 0, texture, priority: false,
                animation_frames: None, source: VisualTileSource { tileset_id: "johto".into(),
                    metatile_id: 1, tile_index: 1, subtile_column: 0, subtile_row: 0 } }],
            ..default() };
        assert!(compatible(&frame, &frame));
        let mut live = frame.clone();
        live.tiles[0].texture = images.add(Image::default());
        assert!(!compatible(&frame, &live));
        live.tiles[0].source.metatile_id = 2;
        assert!(!compatible(&frame, &live));
    }
    #[test]
    fn resident_destination_promotes_before_groves_settle_and_keeps_old_room_hidden() {
        let mut app = App::new();
        let old_root = app.world_mut().spawn(Visibility::Visible).id();
        let town_root = app.world_mut().spawn(Visibility::Hidden).id();
        #[cfg(not(target_arch = "wasm32"))]
        app.world_mut().entity_mut(town_root).insert(pipeline_preload::HiddenDestination);
        let mut frame = VisualWorldFrame { active: true, map_id: "NewBarkTown".into(),
            grid_size: UVec2::ONE, tile_size: Vec2::splat(8.), viewport_size: Vec2::splat(8.),
            ..default() };
        // Use the complete real tile fixture from the compatibility test.
        frame.tiles.push(crystal_render_api::VisualTile {
            column: 0, row: 0, texture: Handle::default(), priority: false, animation_frames: None,
            source: crystal_render_api::VisualTileSource { tileset_id: "johto".into(),
                metatile_id: 1, tile_index: 1, subtile_column: 0, subtile_row: 0 } });
        let mut interior = frame.clone(); interior.map_id = "PlayersHouse1F".into();
        app.insert_resource(frame.clone())
            .insert_resource(live_profiles::LiveProfiles::default())
            .insert_resource(DestinationPreload { frame: Some(frame.clone()), ..default() })
            .insert_resource(TerrainRevisionCache { solid_entity: Some(old_root),
                built_frame: Some(interior), ..default() })
            .insert_resource(TerrainBuildQueue::default())
            .insert_resource(PreparedDestination { ready_frames: 0,
                cache: TerrainRevisionCache { key: Some(TerrainCacheKey::from_frame(&frame)),
                    built_frame: Some(frame), solid_entity: Some(town_root),
                    built_footing_heights: vec![7.], ..default() }, ..default() })
            .add_systems(Update, |frame: Res<VisualWorldFrame>, profiles: Res<live_profiles::LiveProfiles>,
                mut destination: ResMut<PreparedDestination>, mut request: ResMut<DestinationPreload>,
                mut cache: ResMut<TerrainRevisionCache>, mut builds: ResMut<TerrainBuildQueue>, mut commands: Commands| {
                activate(&frame, &profiles, &mut destination, &mut request, &mut cache, &mut builds, &mut commands);
            });
        app.update();
        assert_eq!(app.world().resource::<DestinationPreload>().hits, 1);
        let cache = app.world().resource::<TerrainRevisionCache>();
        assert_eq!(cache.solid_entity, Some(town_root));
        assert_eq!(cache.built_footing_heights, [7.]);
        assert_eq!(*app.world().get::<Visibility>(old_root).unwrap(), Visibility::Hidden);
        assert_eq!(*app.world().get::<Visibility>(town_root).unwrap(), Visibility::Visible);
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert!(app.world().get::<pipeline_preload::HiddenDestination>(old_root).is_some());
            assert!(app.world().get::<pipeline_preload::HiddenDestination>(town_root).is_none());
        }
        assert_eq!(app.world().resource::<TerrainBuildQueue>().started, 0,
            "promoting a destination must not submit another mesh build");
        assert_eq!(app.world().resource::<PreparedDestination>().cache.solid_entity, Some(old_root));
        app.update();
        assert_eq!(app.world().resource::<DestinationPreload>().hits, 1);
    }

    #[test]
    fn in_flight_destination_survives_the_hosts_next_doorway_request() {
        let mut app = App::new();
        let mut frame = VisualWorldFrame { active: true, map_id: "NewBarkTown".into(),
            grid_size: UVec2::ONE, tile_size: Vec2::splat(8.), viewport_size: Vec2::splat(8.),
            ..default() };
        frame.tiles.push(crystal_render_api::VisualTile {
            column: 0, row: 0, texture: Handle::default(), priority: false, animation_frames: None,
            source: crystal_render_api::VisualTileSource { tileset_id: "johto".into(),
                metatile_id: 1, tile_index: 1, subtile_column: 0, subtile_row: 0 } });
        let mut next = frame.clone(); next.map_id = "PlayersHouse1F".into();
        let old_root = app.world_mut().spawn(Visibility::Visible).id();
        let mut live = TerrainRevisionCache { built_frame: Some(next.clone()),
            solid_entity: Some(old_root), ..default() };
        let mut builds = TerrainBuildQueue { started: 7, ..default() };
        let mut destination = PreparedDestination {
            source_frame: Some(Arc::new(frame.clone())),
            builds: TerrainBuildQueue { key: Some(TerrainCacheKey::from_frame(&frame)),
                started: 1, ..default() }, ..default() };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let pool = bevy::tasks::AsyncComputeTaskPool::get_or_init(||
                bevy::tasks::TaskPoolBuilder::new().num_threads(2).build());
            destination.builds.task = Some(pool.spawn(std::future::pending::<TerrainBuildResult>()));
        }
        // The compositor already points speculation at the return doorway.
        let mut request = DestinationPreload { frame: Some(next), ..default() };
        let profiles = live_profiles::LiveProfiles::default();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, app.world());
            activate(&frame, &profiles, &mut destination, &mut request, &mut live, &mut builds, &mut commands);
        }
        queue.apply(app.world_mut());
        assert_eq!(request.hits, 1);
        assert_eq!(builds.key.as_ref().map(|key| key.map_id.as_ref()), Some("NewBarkTown"));
        assert_eq!(builds.started, 7, "adoption must not count as a new foreground build");
        #[cfg(not(target_arch = "wasm32"))]
        assert!(builds.task.is_some(), "the existing asynchronous task must transfer with its key");
        assert!(live.built_frame.is_none(), "continue the original unfinished job");
        assert_eq!(destination.cache.solid_entity, Some(old_root));
        assert_eq!(*app.world().get::<Visibility>(old_root).unwrap(), Visibility::Hidden);
    }

    #[test]
    fn pending_connected_destination_keeps_the_completed_scene_visible() {
        use crystal_render_api::{VisualSourceRegion, VisualTile, VisualTileSource};
        let texture=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(1),std::marker::PhantomData);
        let mut built = VisualWorldFrame { map_texture:texture.clone(), active: true, map_id: "RouteA".into(),
            grid_size: UVec2::splat(28), tile_size: Vec2::splat(8.),
            viewport_size: Vec2::splat(8.), source_regions: vec![
                VisualSourceRegion {map_id:"RouteA".into(),origin:IVec2::ZERO,size:UVec2::new(20,28)},
                VisualSourceRegion {map_id:"RouteB".into(),origin:IVec2::new(20,0),size:UVec2::new(20,28)},
            ].into(), ..default() };
        for row in 0..28 { for column in 0..28 {
            built.tiles.push(VisualTile {row,column,texture:texture.clone(),priority:false,
                animation_frames:None, source:VisualTileSource {tileset_id:"johto".into(),
                    metatile_id:1,tile_index:1,subtile_column:0,subtile_row:0}});
        }}
        let mut frame=built.clone();frame.map_id="RouteB".into();frame.grid_origin=IVec2::new(-20,0);
        frame.source_regions=vec![
            VisualSourceRegion {map_id:"RouteB".into(),origin:IVec2::ZERO,size:UVec2::new(20,28)},
            VisualSourceRegion {map_id:"RouteA".into(),origin:IVec2::new(-20,0),size:UVec2::new(20,28)},
        ].into();
        assert!(terrain_tracking::connected_frame_shift(&built,&frame).is_ok());
        let mut app=App::new();let root=app.world_mut().spawn(Visibility::Visible).id();
        let mut live=TerrainRevisionCache {key:Some(TerrainCacheKey::from_frame(&built)),
            built_frame:Some(built),solid_entity:Some(root),..default()};
        let mut destination=PreparedDestination {source_frame:Some(Arc::new(frame.clone())),
            builds:TerrainBuildQueue {key:Some(TerrainCacheKey::from_frame(&frame)),..default()},..default()};
        let mut request=DestinationPreload::default();let mut builds=TerrainBuildQueue {started:7,..default()};
        let mut queue=bevy::ecs::world::CommandQueue::default();
        {
            let mut commands=Commands::new(&mut queue,app.world());
            activate(&frame,&live_profiles::LiveProfiles::default(),&mut destination,
                &mut request,&mut live,&mut builds,&mut commands);
        }
        queue.apply(app.world_mut());
        assert_eq!(request.hits,0);
        assert_eq!(live.solid_entity,Some(root));
        assert_eq!(live.built_frame.as_ref().unwrap().map_id.as_ref(),"RouteA");
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(),Visibility::Visible);
        assert!(builds.key.is_none());
        assert_eq!(builds.started,7);
        assert_eq!(destination.builds.key.as_ref().unwrap().map_id.as_ref(),"RouteB");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn destination_builds_and_stays_hidden_while_initial_room_is_pending() {
        use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
        let pool=bevy::tasks::TaskPoolBuilder::new().num_threads(1).build();
        let gate=Arc::new(AtomicBool::new(false));
        let worker_gate=gate.clone();
        let painted=painted_stone::NativePreparation::from_task(pool.spawn(async move {
            while !worker_gate.load(Ordering::SeqCst) {
                bevy::tasks::futures_lite::future::yield_now().await;
            }
            painted_stone::PreparedImages {stone:Image::default(),bedrock:Image::default()}
        }));
        let mut app=App::new();
        app.add_plugins((MinimalPlugins,bevy::transform::TransformPlugin));
        let mut images=Assets::<Image>::default();
        let texture=images.add(Image::default());
        let frame=VisualWorldFrame {active:true,map_id:"NewBarkTown".into(),
            map_texture:texture.clone(),grid_size:UVec2::ONE,tile_size:Vec2::splat(8.),
            viewport_size:Vec2::splat(8.),tiles:vec![crystal_render_api::VisualTile {
                column:0,row:0,texture:texture.clone(),priority:false,animation_frames:None,
                source:crystal_render_api::VisualTileSource {tileset_id:"johto".into(),
                    metatile_id:1,tile_index:1,subtile_column:0,subtile_row:0}}],..default()};
        frame.validate().unwrap();
        app.insert_resource(images)
            .insert_resource(VisualWorldFrame {active:true,map_id:"PlayersHouse2F".into(),..default()})
            .insert_resource(VoxelViewStatus {active:false,terrain_pending:true,..default()})
            .insert_resource(VoxelViewSettings {enabled:true,..default()})
            .insert_resource(DestinationPreload {frame:Some(frame),..default()})
            .init_resource::<TerrainRevisionCache>()
            .insert_resource(TerrainBuildQueue {painted,..default()})
            .init_resource::<PreparedDestination>().init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<Assets<Mesh>>().init_resource::<Assets<VoxelMaterial>>()
            .add_systems(Update,prepare);
        app.update();
        assert_eq!(app.world().resource::<PreparedDestination>().builds.started,1);
        assert!(app.world().resource::<PreparedDestination>().builds.painted.pending());
        assert!(app.world().resource::<PreparedDestination>().cache.key.is_none());
        assert_eq!(app.world().resource::<Assets<Image>>().len(),2,
            "the destination retains its independent source-atlas snapshot");
        gate.store(true,Ordering::SeqCst);
        let started=std::time::Instant::now();
        while app.world().resource::<PreparedDestination>().ready_frames < 3
            && started.elapsed()<std::time::Duration::from_secs(5) {
            app.update();std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let cache=&app.world().resource::<PreparedDestination>().cache;
        let expected=(cache.painted_stone.clone().unwrap(),cache.painted_bedrock.clone().unwrap());
        let mut foreground=app.world().resource::<TerrainBuildQueue>().painted.clone();
        assert_eq!(foreground.poll(&mut app.world_mut().resource_mut::<Assets<Image>>()),Some(expected));
        assert_eq!(app.world().resource::<Assets<Image>>().len(),4,
            "foreground and destination must publish each immutable image only once");
        let destination=app.world().resource::<PreparedDestination>();
        assert_eq!(destination.ready_frames,3,"the hidden destination must finish before the room is ready");
        assert_eq!(destination.builds.started,1,"pending room must not restart destination work");
        let roots=roots(&destination.cache).collect::<Vec<_>>();
        assert!(!roots.is_empty());
        for root in roots {assert_eq!(app.world().get::<Visibility>(root),Some(&Visibility::Hidden));}
        let status=app.world().resource::<VoxelViewStatus>();
        assert!(!status.active && status.terrain_pending);
        assert_eq!(app.world().resource::<VisualWorldFrame>().map_id.as_ref(),"PlayersHouse2F");
    }

    #[test]
    fn settled_destination_does_not_mutate_hidden_roots_or_restart_builds() {
        let mut app = App::new();
        let camera = app.world_mut().spawn((VoxelWorldCamera,
            Projection::Perspective(PerspectiveProjection { aspect_ratio: 1., ..default() }))).id();
        let root = app.world_mut().spawn((VoxelTerrain, Visibility::Hidden, Transform::default())).id();
        let mut images = Assets::<Image>::default();
        let texture = images.add(Image::default());
        let frame = VisualWorldFrame { active: true, map_id: "ElmsLab".into(),
            map_texture: texture.clone(),
            grid_size: UVec2::ONE, tile_size: Vec2::splat(8.), viewport_size: Vec2::splat(8.),
            tiles: vec![crystal_render_api::VisualTile { column: 0, row: 0,
                texture, priority: false, animation_frames: None,
                source: crystal_render_api::VisualTileSource { tileset_id: "johto".into(),
                    metatile_id: 1, tile_index: 1, subtile_column: 0, subtile_row: 0 } }], ..default() };
        frame.validate().unwrap();
        app.insert_resource(images);
        let sunlight = app.world_mut().spawn((VoxelSun,
            DirectionalLight { illuminance: 1000., shadow_maps_enabled: true, ..default() },
            Transform::default())).id();
        let settings = VoxelViewSettings { enabled: true, ..default() };
        let key = TerrainCacheKey::from_frame(&frame);
        app.insert_resource(DestinationPreload { frame: Some(frame.clone()), ..default() })
            .insert_resource(VisualWorldFrame { active: true, map_id: "NewBarkTown".into(), ..default() })
            .insert_resource(VoxelViewStatus { active: true, ..default() })
            .insert_resource(settings.clone())
            .init_resource::<live_profiles::LiveProfiles>()
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<TerrainBuildQueue>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .insert_resource(PreparedDestination { ready_frames: 3,
                settled_key: Some((key.clone(), settings.camera, None, 1_f32.to_bits())),
                cache: TerrainRevisionCache { key: Some(key), built_frame: Some(frame),
                    built_footing_heights: vec![0.],
                    solid_entity: Some(root), ..default() }, ..default() })
            .add_systems(Update, prepare);
        app.update();
        let before = app.world().entity(root).get_ref::<Visibility>().unwrap().last_changed();
        for step in 0..10 {
            app.world_mut().get_mut::<Transform>(sunlight).unwrap().rotation =
                Quat::from_rotation_y(step as f32 * 0.1);
            app.update();
        }
        assert_eq!(app.world().entity(root).get_ref::<Visibility>().unwrap().last_changed(), before,
            "a hidden preload must not toggle visibility on idle frames");
        let destination = app.world().resource::<PreparedDestination>();
        assert_eq!(destination.ready_frames, 3);
        assert_eq!(destination.builds.started, 0);
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(), Visibility::Hidden);
        // The source/settings remain identical, but a wider output changes
        // the destination's visible grove footprint. Refresh it while hidden.
        let Projection::Perspective(mut projection) = app.world().get::<Projection>(camera).unwrap().clone()
            else { unreachable!() };
        projection.aspect_ratio = 2.;
        app.world_mut().entity_mut(camera).insert(Projection::Perspective(projection));
        app.update();
        assert_ne!(app.world().entity(root).get_ref::<Visibility>().unwrap().last_changed(), before,
            "resizing the actual camera must refresh the settled hidden destination");
        assert_eq!(*app.world().get::<Visibility>(root).unwrap(), Visibility::Hidden);
        assert_eq!(app.world().resource::<PreparedDestination>().builds.started, 0);
    }

}
