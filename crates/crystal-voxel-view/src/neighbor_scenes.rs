//! Retain each visible outdoor map at the ordinary terrain-controller boundary.
//! These are scenery-only scenes: no simulation, actors or second dispatcher.
use super::*;
use crystal_render_api::{VisualNeighborMap, VisualWorldNeighborhood};
use std::{collections::HashMap, sync::Arc};

#[derive(Resource, Default)]
pub(super) struct Scenes {
    pub entries: HashMap<Arc<str>, Scene>,
    serial: u64,
}

pub(super) struct Scene {
    source: Arc<VisualWorldFrame>,
    pub live: VisualWorldFrame,
    pub cache: TerrainRevisionCache,
    builds: TerrainBuildQueue,
    seen: u64,
    phases: Option<Arc<crystal_render_api::VisualTileAnimationFrame>>,
    phase_revision: Option<u64>,
}

impl Scenes {
    pub(super) fn world_emitters(&self) -> impl Iterator<Item = city_lights::Emitter> + '_ {
        self.entries
            .values()
            .filter(|scene| scene.seen == self.serial)
            .flat_map(|scene| {
                let offset = retained_terrain_transform(&scene.live, &scene.cache)
                    .map_or(Vec3::ZERO, |transform| transform.translation);
                scene
                    .cache
                    .city_emitters
                    .iter()
                    .map(move |emitter| city_lights::Emitter {
                        position: (emitter.position.truncate() + offset).extend(emitter.position.w),
                        direction: emitter.direction,
                    })
            })
    }
}

/// A connection can enter a map that is already visible and fully meshed.
/// Transfer that scene to the active controller rather than discarding it and
/// presenting an empty destination while its camera grid is prepared.
pub(super) fn promote(
    frame: &VisualWorldFrame,
    profiles_revision: u64,
    scenes: &mut Scenes,
    cache: &mut TerrainRevisionCache,
    builds: &mut TerrainBuildQueue,
    commands: &mut Commands,
) {
    if cache
        .built_frame
        .as_ref()
        .is_some_and(|built| built.map_id == frame.map_id)
    {
        return;
    }
    if !scenes.entries.get(&frame.map_id).is_some_and(|scene| {
        scene.cache.built_frame.is_some()
            && scene
                .cache
                .key
                .as_ref()
                .is_some_and(|key| key.profiles_revision == profiles_revision)
    }) {
        return;
    }
    let scene = scenes
        .entries
        .remove(&frame.map_id)
        .expect("checked ready neighbor");
    let mut old_cache = std::mem::replace(cache, scene.cache);
    let old_builds = std::mem::replace(builds, scene.builds);
    cache.neighbor = false;
    cache.footing_origin = None;
    if let Some(backdrop) = old_cache.backdrop.take() {
        commands.entity(backdrop.entity).despawn();
    }
    if let Some(old_frame) = old_cache.built_frame.clone() {
        let id = old_frame.map_id.clone();
        if let Some(retired) = scenes.entries.remove(&id) {
            for entity in roots(&retired.cache) {
                commands.entity(entity).despawn();
            }
        }
        old_cache.neighbor = true;
        scenes.entries.insert(
            id,
            Scene {
                source: Arc::new(old_frame.clone()),
                live: old_frame,
                cache: old_cache,
                builds: old_builds,
                seen: 0,
                phases: None,
                phase_revision: None,
            },
        );
    } else {
        for entity in roots(&old_cache) {
            commands.entity(entity).despawn();
        }
    }
}

fn roots(cache: &TerrainRevisionCache) -> impl Iterator<Item = Entity> {
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

fn set_visible(scene: &Scene, visible: bool, commands: &mut Commands) {
    for entity in roots(&scene.cache) {
        commands.entity(entity).insert(if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}

fn update_source(scene: &mut Scene, map: &VisualNeighborMap) {
    if !Arc::ptr_eq(&scene.source, &map.frame) {
        scene.live = (*map.frame).clone();
        scene.source = map.frame.clone();
    }
    scene.live.center = map.center;
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync(
    frame: Res<VisualWorldFrame>,
    neighborhood: Res<VisualWorldNeighborhood>,
    settings: Res<VoxelViewSettings>,
    profiles: Res<live_profiles::LiveProfiles>,
    mut primary: ResMut<TerrainRevisionCache>,
    mut scenes: ResMut<Scenes>,
    mut status: ResMut<VoxelViewStatus>,
    time: Res<Time>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut entities: Query<(&mut Visibility, &mut Transform), VoxelTerrainFilter>,
    cameras: Query<(&Projection, &Transform), (With<VoxelWorldCamera>, Without<VoxelTerrain>)>,
    sun: Query<(&DirectionalLight, &Transform), (With<VoxelSun>, Without<VoxelTerrain>)>,
) {
    scenes.serial = scenes.serial.wrapping_add(1);
    if !settings.enabled
        || !frame.active
        || !status.active
        || neighborhood.active_map_id != frame.map_id
    {
        for scene in scenes.entries.values() {
            set_visible(scene, false, &mut commands);
        }
        return;
    }
    let matrix = cameras
        .iter()
        .find_map(|(projection, transform)| match projection {
            Projection::Perspective(p) => Some(
                bevy::math::proj::perspective(p.fov, p.aspect_ratio, p.near, p.far)
                    * transform.to_matrix().inverse(),
            ),
            _ => None,
        });
    let sun = sun.iter().find_map(|(light, transform)| {
        (light.shadow_maps_enabled && light.illuminance > 0.)
            .then(|| -transform.forward().as_vec3())
    });
    let serial = scenes.serial;
    backdrop_clip::sync(&frame, &neighborhood, &mut primary, &mut meshes);
    let wanted = |id: &str| {
        neighborhood
            .maps
            .iter()
            .any(|map| map.frame.map_id.as_ref() == id)
    };
    // Keep at most two recently hidden maps for camera reversals. Cancel
    // offscreen work and retire older roots/assets through normal Bevy drops.
    let mut hidden: Vec<_> = scenes
        .entries
        .iter()
        .filter(|(id, _)| !wanted(id))
        .map(|(id, scene)| (id.clone(), scene.seen))
        .collect();
    hidden.sort_unstable_by_key(|(_, seen)| std::cmp::Reverse(*seen));
    for (id, _) in hidden.into_iter().skip(2) {
        if let Some(scene) = scenes.entries.remove(&id) {
            for entity in roots(&scene.cache) {
                commands.entity(entity).despawn();
            }
        }
    }
    for (id, scene) in &mut scenes.entries {
        if !wanted(id) {
            set_visible(scene, false, &mut commands);
            // Dropping a Task cancels its map-specific build.
            scene.builds = TerrainBuildQueue::default();
        }
    }
    // Bound overlapping source/mesh preparation. Visible maps are independent
    // retained scenes, but only one neighbor starts heavy work at a time.
    let mut running = scenes
        .entries
        .values()
        .filter(|scene| scene.builds.key.is_some())
        .count();
    let mut pending = false;
    for map in &neighborhood.maps {
        let scene = scenes
            .entries
            .entry(map.frame.map_id.clone())
            .or_insert_with(|| Scene {
                source: map.frame.clone(),
                live: (*map.frame).clone(),
                cache: TerrainRevisionCache {
                    neighbor: true,
                    ..default()
                },
                builds: TerrainBuildQueue::default(),
                seen: serial,
                phases: None,
                phase_revision: None,
            });
        update_source(scene, map);
        scene.seen = serial;
        if scene.cache.painted_stone.is_none() {
            scene.cache.painted_stone = primary.painted_stone.clone();
        }
        if scene.cache.painted_bedrock.is_none() {
            scene.cache.painted_bedrock = primary.painted_bedrock.clone();
        }
        scene.cache.forest_library = primary.forest_library.clone();
        let mut key = TerrainCacheKey::from_frame(&scene.live);
        key.profiles_revision = profiles.revision;
        let was_running = scene.builds.key.is_some();
        if !was_running && scene.cache.key.as_ref() != Some(&key) && running > 0 {
            pending = true;
            // A previous complete crop can still follow the current camera.
            if let Some(transform) = retained_terrain_transform(&scene.live, &scene.cache) {
                for entity in roots(&scene.cache) {
                    commands
                        .entity(entity)
                        .insert((transform, Visibility::Visible));
                }
            }
            continue;
        }
        match sync_terrain(
            &scene.live,
            &profiles,
            &mut commands,
            &mut scene.cache,
            &mut scene.builds,
            &mut images,
            &mut meshes,
            &mut materials,
            &mut entities,
        ) {
            Ok(TerrainSyncState::Ready) => {}
            Ok(TerrainSyncState::Pending) => pending = true,
            Err(error) => {
                status.active = false;
                status.inactive_reason = Some(format!("neighbor {}: {error:?}", map.frame.map_id));
                status.active_frames = 0;
                return;
            }
        }
        if was_running && scene.builds.key.is_none() {
            running = running.saturating_sub(1);
        }
        if !was_running && scene.builds.key.is_some() {
            running += 1;
        }
        if let Some(matrix) = matrix {
            pending |= grove_stream::sync(
                &scene.live,
                matrix,
                sun,
                &mut scene.cache,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        }
        let revision = scene.cache.key.as_ref().map(|key| key.revision);
        let phase_changed = scene.phase_revision != revision
            || scene
                .phases
                .as_ref()
                .is_none_or(|old| !Arc::ptr_eq(old, &map.phases));
        if phase_changed {
            if let Err(error) = terrain_animation::sync_cache(
                &map.phases,
                &mut scene.cache,
                &mut images,
                &mut meshes,
                &mut materials,
            ) {
                status.active = false;
                status.inactive_reason = Some(format!("neighbor {}: {error}", map.frame.map_id));
                status.active_frames = 0;
                return;
            }
            if scene.cache.built_frame.is_some() {
                scene.phase_revision = revision;
                scene.phases = Some(map.phases.clone());
            }
        }
        sync_water(
            &scene.live,
            &map.phases,
            &mut scene.cache,
            phase_changed,
            time.elapsed_secs_wrapped(),
            &mut images,
            &mut materials,
        );
        if let Some(handle) = scene.cache.solid_material.as_ref() {
            if scene.cache.built_animated_bounds[1].is_some() {
                if let Some(mut material) = materials.get_mut(handle) {
                    material.extension.grass_rustle.height_scale.y = time.elapsed_secs();
                }
            }
        }
    }
    status.terrain_pending |= pending;
    if pending {
        status.active_frames = 0;
    }
}

fn sync_water(
    live: &VisualWorldFrame,
    phases: &crystal_render_api::VisualTileAnimationFrame,
    cache: &mut TerrainRevisionCache,
    phase_changed: bool,
    time: f32,
    images: &mut Assets<Image>,
    materials: &mut Assets<VoxelMaterial>,
) {
    let Some(handle) = cache.textured_material.clone() else {
        return;
    };
    let mask = materials
        .get(&handle)
        .and_then(|m| m.extension.water_mask.clone());
    let Some(built) = cache.built_frame.as_mut() else {
        return;
    };
    if phase_changed {
        cave_water::refresh_phases(built, phases, images);
    }
    let (changed, steps) = mask
        .as_ref()
        .filter(|_| phase_changed)
        .map_or((false, Vec2::ZERO), |mask| {
            cave_water::refresh_mask_with_steps(built, mask, images)
        });
    let palette = changed
        .then(|| cave_water::native_palette(built, images))
        .flatten();
    let registration = cave_water::retained_registration(built, live);
    if !changed
        && palette.is_none()
        && materials.get(&handle).is_none_or(|material| {
            material.extension.water_enabled.x <= 0.5
                || material.extension.water_enabled == registration
        })
    {
        return;
    }
    if let Some(mut material) = materials.get_mut(&handle) {
        if changed {
            material.extension.water_motion =
                water_surface::advance_motion(material.extension.water_motion, steps);
            material.extension.water_phase =
                water_surface::blend_clock(material.extension.water_phase, time, steps);
        }
        if let Some(palette) = palette {
            material.extension.water_deep = palette[0];
            material.extension.water_foam = palette[1];
        }
        if material.extension.water_enabled.x > 0.5 {
            material.extension.water_enabled = registration;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_render_api::{VisualTile, VisualTileAnimationFrame, VisualTileSource};

    #[test]
    fn a_retained_neighbor_moves_as_one_scene_keeps_its_own_identity_and_hides_outside_the_view() {
        let image = Handle::Uuid(
            bevy::asset::uuid::Uuid::from_u128(9181),
            std::marker::PhantomData,
        );
        let source = Arc::new(VisualWorldFrame {
            active: true,
            map_id: "CherrygroveCity".into(),
            map_texture: image.clone(),
            viewport_size: Vec2::splat(64.),
            tile_size: Vec2::splat(32.),
            grid_size: UVec2::splat(2),
            terrain_revision: 7,
            tiles: (0..4)
                .map(|i| VisualTile {
                    column: i % 2,
                    row: i / 2,
                    source: VisualTileSource {
                        tileset_id: "johto".into(),
                        metatile_id: 0,
                        subtile_column: (i % 2) as u8,
                        subtile_row: (i / 2) as u8,
                        tile_index: 5,
                    },
                    texture: image.clone(),
                    animation_frames: None,
                    priority: false,
                })
                .collect(),
            ..default()
        });
        assert!(source.validate().is_ok());
        let phases = Arc::new(VisualTileAnimationFrame {
            map_id: source.map_id.clone(),
            ..default()
        });
        let mut app = App::new();
        let root = app
            .world_mut()
            .spawn((VoxelTerrain, Transform::default(), Visibility::Visible))
            .id();
        let mut scenes = Scenes::default();
        scenes.entries.insert(
            source.map_id.clone(),
            Scene {
                source: source.clone(),
                live: (*source).clone(),
                cache: TerrainRevisionCache {
                    neighbor: true,
                    built_frame: Some((*source).clone()),
                    key: Some(TerrainCacheKey::from_frame(&source)),
                    instances_root: Some(root),
                    built_footing_heights: vec![0.; 4],
                    city_emitters: vec![city_lights::Emitter {
                        position: Vec4::new(2., 4., 6., 3.),
                        direction: Vec4::X,
                    }],
                    ..default()
                },
                builds: TerrainBuildQueue::default(),
                seen: 0,
                phases: Some(phases.clone()),
                phase_revision: Some(7),
            },
        );
        app.insert_resource(VisualWorldFrame {
            active: true,
            map_id: "Route29".into(),
            ..default()
        })
        .insert_resource(VisualWorldNeighborhood {
            active_map_id: "Route29".into(),
            maps: vec![VisualNeighborMap {
                frame: source.clone(),
                center: Vec2::new(128., -64.),
                phases,
            }],
        })
        .insert_resource(scenes)
        .insert_resource(VoxelViewSettings {
            enabled: true,
            ..default()
        })
        .insert_resource(VoxelViewStatus {
            active: true,
            ..default()
        })
        .init_resource::<live_profiles::LiveProfiles>()
        .init_resource::<TerrainRevisionCache>()
        .init_resource::<TerrainBuildQueue>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<VoxelMaterial>>()
        .insert_resource(Time::<()>::default())
        .add_systems(Update, sync);
        let previous_root = app
            .world_mut()
            .spawn((VoxelTerrain, Transform::default(), Visibility::Visible))
            .id();
        let previous_frame = VisualWorldFrame {
            map_id: "Route29".into(),
            ..(*source).clone()
        };
        *app.world_mut().resource_mut::<TerrainRevisionCache>() = TerrainRevisionCache {
            built_frame: Some(previous_frame.clone()),
            key: Some(TerrainCacheKey::from_frame(&previous_frame)),
            instances_root: Some(previous_root),
            ..default()
        };
        app.add_systems(
            Update,
            (|frame: Res<VisualWorldFrame>,
              mut scenes: ResMut<Scenes>,
              mut cache: ResMut<TerrainRevisionCache>,
              mut builds: ResMut<TerrainBuildQueue>,
              mut commands: Commands| {
                promote(
                    &frame,
                    0,
                    &mut scenes,
                    &mut cache,
                    &mut builds,
                    &mut commands,
                );
            })
            .before(sync),
        );
        app.update();
        assert_eq!(
            app.world().get::<Transform>(root).unwrap().translation,
            Vec3::new(128., 0., 64.)
        );
        {
            let scenes = app.world().resource::<Scenes>();
            let scene = &scenes.entries["CherrygroveCity"];
            assert_eq!(
                scene.cache.key.as_ref().unwrap().map_id.as_ref(),
                "CherrygroveCity"
            );
            assert_eq!(
                scene.builds.started, 0,
                "camera placement must not rebuild retained geometry"
            );
            assert!(Arc::ptr_eq(&scene.source, &source));
            let emitter = scenes.world_emitters().next().unwrap();
            assert_eq!(emitter.position, Vec4::new(130., 4., 70., 3.));
        }
        app.world_mut()
            .resource_mut::<VisualWorldNeighborhood>()
            .maps[0]
            .center = Vec2::new(96., -32.);
        app.update();
        assert_eq!(
            app.world().get::<Transform>(root).unwrap().translation,
            Vec3::new(96., 0., 32.)
        );
        app.world_mut()
            .resource_mut::<VisualWorldNeighborhood>()
            .maps
            .clear();
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            app.world().resource::<Scenes>().world_emitters().count(),
            0,
            "hidden neighbors cannot leave lights in the active world"
        );
        *app.world_mut().resource_mut::<VisualWorldFrame>() = (*source).clone();
        app.update();
        let primary = app.world().resource::<TerrainRevisionCache>();
        assert_eq!(
            primary.instances_root,
            Some(root),
            "entering a prepared neighbor must keep its actual scene"
        );
        assert!(!primary.neighbor);
        assert_eq!(
            primary.key.as_ref().unwrap().map_id.as_ref(),
            "CherrygroveCity"
        );
        assert_eq!(
            primary.footing_origin, None,
            "the active grid realigns its support after promotion"
        );
        let scenes = app.world().resource::<Scenes>();
        assert!(!scenes.entries.contains_key("CherrygroveCity"));
        assert_eq!(
            scenes.entries["Route29"].cache.instances_root,
            Some(previous_root)
        );
        assert!(scenes.entries["Route29"].cache.neighbor);
    }
}
