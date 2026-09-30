//! A render-only arena. Presentation cues come from the production shell's
//! retained visible scene; this module has no runtime or battle-engine imports.
use crate::{VoxelViewSettings, mesh::SurfaceMeshData};
use bevy::{
    core_pipeline::{fxaa::Fxaa, tonemapping::Tonemapping},
    pbr::{CascadeShadowConfigBuilder, FogFalloff, FogSettings},
    prelude::*,
    render::{camera::ClearColorConfig, view::RenderLayers},
};
use crystal_render_api::{
    BattleFlashMode, VisualBattleBattler, VisualBattleCue, VisualBattleCueKind,
    VisualBattleEnvironment, VisualBattleFrame, VisualBattleSide, VisualBattleSourceFrame,
    WorldRenderSet,
};
use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    sync::{Arc, OnceLock},
};

const BATTLE_LAYER: usize = 29;
const MODEL_SCALE: f32 = 2.65;
const PARTICLES: usize = 32;

#[derive(Default)]
pub struct BattleViewPlugin;
impl Plugin for BattleViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualBattleFrame>()
            .init_resource::<BattleFlashMode>()
            .init_resource::<BattleViewStatus>()
            .init_resource::<BattleScene>()
            .add_systems(Startup, setup_battle_scene)
            .add_systems(
                Update,
                (
                    toggle_battle_flash_mode,
                    sync_battle_scene,
                    sync_source_objects,
                    sync_modeled_source_effects,
                )
                    .chain()
                    .in_set(WorldRenderSet::RenderSync),
            );
    }
}

/// Explicit diagnostics distinguish exact modeled species from source-art
/// fallback; support for an icon family never claims a modeled Pokédex entry.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct BattleViewStatus {
    pub active: bool,
    pub active_frames: u32,
    pub modeled_species: Vec<String>,
    pub source_art_species: Vec<String>,
    pub last_error: Option<String>,
}
#[derive(Component)]
struct BattleCamera;
#[derive(Component)]
struct BattleLight;
#[derive(Component)]
struct BattleArena;
#[derive(Component)]
struct BattleActor;
#[derive(Component)]
struct BattleParticle;
#[derive(Component)]
struct BattleBall;
#[derive(Component)]
struct BattleContactShadow(VisualBattleSide);
#[derive(Component)]
struct BattleSourceObject(usize);
#[derive(Component)]
struct BattleModeledSourceEffect(usize);
#[derive(Clone, PartialEq)]
struct ActorKey {
    species: Arc<str>,
    modeled: bool,
    texture: Handle<Image>,
}
struct ActorInstance {
    entity: Entity,
    key: ActorKey,
    material: Handle<StandardMaterial>,
    mesh: Option<Handle<Mesh>>,
    palette_key: (u8, BattleFlashMode),
    deformation_key: Option<[i8; 0x5f]>,
}
#[derive(Resource, Default)]
struct BattleScene {
    arena: Option<Entity>,
    arena_key: Option<VisualBattleEnvironment>,
    arena_mesh: Option<Handle<Mesh>>,
    actors: [Option<ActorInstance>; 2],
    species_meshes: HashMap<Arc<str>, Option<Arc<SurfaceMeshData>>>,
    surface_material: Handle<StandardMaterial>,
    fallback_mesh: Handle<Mesh>,
    particle_mesh: Handle<Mesh>,
    particle_materials: Vec<Handle<StandardMaterial>>,
    particles: Vec<Entity>,
    ball: Option<Entity>,
    elapsed: f32,
    was_active: bool,
    source_effect_meshes: Vec<Handle<Mesh>>,
}
#[derive(Clone, serde::Deserialize)]
struct ArenaPalette {
    ground: [f32; 4],
    stage: [f32; 4],
    edge: [f32; 4],
    accent: [f32; 4],
    sky: [f32; 4],
}
fn palette(environment: VisualBattleEnvironment) -> &'static ArenaPalette {
    static PALETTES: OnceLock<HashMap<String, ArenaPalette>> = OnceLock::new();
    let all = PALETTES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../art/battles/arena-palettes.json"))
            .expect("verified authored arena palette")
    });
    &all[match environment {
        VisualBattleEnvironment::Meadow => "meadow",
        VisualBattleEnvironment::Forest => "forest",
        VisualBattleEnvironment::Cave => "cave",
        VisualBattleEnvironment::Water => "water",
        VisualBattleEnvironment::Interior => "interior",
        VisualBattleEnvironment::Ice => "ice",
    }]
}
fn rgb(c: [f32; 4]) -> Color {
    Color::srgba(c[0], c[1], c[2], c[3])
}
fn side_position(side: VisualBattleSide) -> Vec3 {
    match side {
        VisualBattleSide::Player => Vec3::new(-2.1, 0.12, 1.5),
        VisualBattleSide::Enemy => Vec3::new(2.1, 0.12, -1.5),
    }
}
fn camera_pose(elapsed: f32) -> Transform {
    // Subtle breathing camera rather than a free orbit that can hide commands.
    Transform::from_xyz(7.8 + (elapsed * 0.13).sin() * 0.14, 6.3, 11.6)
        .looking_at(Vec3::new(0.0, 0.80, 0.0), Vec3::Y)
}
fn setup_battle_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene: ResMut<BattleScene>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 0,
                is_active: false,
                clear_color: ClearColorConfig::Custom(rgb(palette(
                    VisualBattleEnvironment::Meadow,
                )
                .sky)),
                ..default()
            },
            exposure: bevy::render::camera::Exposure { ev100: 12.0 },
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 0.58,
                near: 0.1,
                far: 100.0,
                ..default()
            }),
            transform: camera_pose(0.0),
            tonemapping: Tonemapping::AcesFitted,
            ..default()
        },
        RenderLayers::layer(BATTLE_LAYER),
        BattleCamera,
        FogSettings {
            color: rgb(palette(VisualBattleEnvironment::Meadow).sky),
            falloff: FogFalloff::Linear {
                start: 19.0,
                end: 39.0,
            },
            ..default()
        },
        Fxaa {
            enabled: true,
            ..default()
        },
    ));
    #[cfg(not(target_arch = "wasm32"))]
    let hardware_shadows = std::env::var("CRYSTAL_BATTLE_SHADOWS").is_ok_and(|value| value == "on");
    #[cfg(target_arch = "wasm32")]
    let hardware_shadows = false;
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: hardware_shadows,
                color: Color::srgb(1.0, 0.94, 0.82),
                shadow_depth_bias: 0.015,
                shadow_normal_bias: 0.6,
                ..default()
            },
            transform: Transform::from_xyz(6.0, 12.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            cascade_shadow_config: CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 40.0,
                first_cascade_far_bound: 40.0,
                ..default()
            }
            .build(),
            visibility: Visibility::Hidden,
            ..default()
        },
        RenderLayers::layer(BATTLE_LAYER),
        BattleLight,
    ));
    scene.surface_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.96,
        reflectance: 0.1,
        ..default()
    });
    scene.fallback_mesh = meshes.add(Rectangle::new(1.9, 1.9));
    scene.particle_mesh = meshes.add(
        Sphere::new(1.0)
            .mesh()
            .ico(1)
            .expect("valid low-poly particle"),
    );
    for element in [
        "NORMAL", "FIRE", "WATER", "GRASS", "ELECTRIC", "ICE", "PSYCHIC", "POISON",
    ] {
        let color = element_color(element);
        let material = materials.add(StandardMaterial {
            base_color: rgb(color),
            unlit: true,
            ..default()
        });
        scene.particle_materials.push(material);
    }
    for _ in 0..PARTICLES {
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: scene.particle_mesh.clone(),
                    material: scene.particle_materials[0].clone(),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                RenderLayers::layer(BATTLE_LAYER),
                BattleParticle,
            ))
            .id();
        scene.particles.push(entity);
    }
    let ball = commands
        .spawn((
            PbrBundle {
                mesh: meshes.add(capture_ball_mesh().into_mesh()),
                material: scene.surface_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleBall,
        ))
        .id();
    scene.ball = Some(ball);
    let object_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    for slot in 0..10 {
        commands.spawn((
            PbrBundle {
                mesh: object_mesh.clone(),
                material: materials.add(StandardMaterial {
                    alpha_mode: AlphaMode::Mask(0.1),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleSourceObject(slot),
            bevy::pbr::NotShadowCaster,
        ));
    }
    // A bounded companion pool adds depth to the currently visible source
    // objects. Its clock, lifetimes and positions remain the interpreter's.
    let mut beam = SurfaceMeshData::default();
    append_cylinder(&mut beam, Vec3::Y * -0.5, 1.0, 1.0, 16, [1.0; 4]);
    scene.source_effect_meshes = vec![
        meshes.add(source_wave_ring_mesh().into_mesh()),
        meshes.add(beam.into_mesh()),
        scene.particle_mesh.clone(),
    ];
    for slot in 0..13 {
        commands.spawn((
            PbrBundle {
                mesh: scene.source_effect_meshes[0].clone(),
                material: materials.add(StandardMaterial {
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                }),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleModeledSourceEffect(slot),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
    let contact_mesh = meshes.add(contact_shadow_mesh().into_mesh());
    let contact_material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        commands.spawn((
            PbrBundle {
                mesh: contact_mesh.clone(),
                material: contact_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
            RenderLayers::layer(BATTLE_LAYER),
            BattleContactShadow(side),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_battle_scene(
    mut commands: Commands,
    frame: Res<VisualBattleFrame>,
    settings: Res<VoxelViewSettings>,
    flash_mode: Res<BattleFlashMode>,
    time: Res<Time>,
    mut status: ResMut<BattleViewStatus>,
    mut scene: ResMut<BattleScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<
        (&mut Camera, &mut Transform, &mut FogSettings),
        (
            With<BattleCamera>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut lights: Query<
        &mut Visibility,
        (
            With<BattleLight>,
            Without<BattleArena>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut arenas: Query<
        &mut Visibility,
        (
            With<BattleArena>,
            Without<BattleLight>,
            Without<BattleActor>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut actors: Query<
        (&mut Transform, &mut Visibility),
        (
            With<BattleActor>,
            Without<BattleCamera>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
    mut effects: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Handle<StandardMaterial>,
        ),
        (
            With<BattleParticle>,
            Without<BattleCamera>,
            Without<BattleActor>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleBall>,
        ),
    >,
    mut balls: Query<
        (&mut Transform, &mut Visibility),
        (
            With<BattleBall>,
            Without<BattleCamera>,
            Without<BattleActor>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleParticle>,
        ),
    >,
    mut contact_shadows: Query<
        (&BattleContactShadow, &mut Transform, &mut Visibility),
        (
            Without<BattleCamera>,
            Without<BattleActor>,
            Without<BattleLight>,
            Without<BattleArena>,
            Without<BattleParticle>,
            Without<BattleBall>,
        ),
    >,
) {
    let valid = frame.validate();
    let active = settings.enabled && frame.active && valid.is_ok();
    status.active = active;
    status.active_frames = if active {
        status.active_frames.saturating_add(1)
    } else {
        0
    };
    status.last_error = valid.err().map(str::to_owned);
    for (mut camera, mut transform, mut fog) in &mut cameras {
        camera.is_active = active;
        if active {
            *transform = camera_pose(scene.elapsed);
            let (dark, light) = source_environment_palette(frame.source.as_ref(), *flash_mode);
            let sky = rgb(palette(frame.environment).sky)
                .mix(&Color::BLACK, dark)
                .mix(&Color::WHITE, light);
            camera.clear_color = ClearColorConfig::Custom(sky);
            fog.color = sky;
            if let Some(source) = &frame.source {
                // Screen shake stays source-clocked. It affects the arena,
                // leaving the readable production HUD in its fixed panels.
                let offset = source.screen_offset * 0.012;
                let displacement = transform.rotation * Vec3::new(-offset.x, -offset.y, 0.0);
                transform.translation += displacement;
            }
        }
    }
    for mut visibility in &mut lights {
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for mut visibility in &mut arenas {
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (_, mut visibility) in &mut actors {
        *visibility = Visibility::Hidden;
    }
    for (_, mut visibility, _) in &mut effects {
        *visibility = Visibility::Hidden;
    }
    for (_, mut visibility) in &mut balls {
        *visibility = Visibility::Hidden;
    }
    for (_, _, mut visibility) in &mut contact_shadows {
        *visibility = Visibility::Hidden;
    }
    if !active {
        scene.was_active = false;
        status.modeled_species.clear();
        status.source_art_species.clear();
        return;
    }
    if !scene.was_active {
        scene.elapsed = 0.0;
    }
    scene.was_active = true;
    scene.elapsed += time.delta_seconds().clamp(0.0, 0.1);
    if let Some(material) = materials.get_mut(&scene.surface_material) {
        let (dark, light) = source_environment_palette(frame.source.as_ref(), *flash_mode);
        material.base_color = Color::srgb(1.0 - dark, 1.0 - dark, 1.0 - dark);
        material.emissive = LinearRgba::new(light, light, light, 1.0);
    }
    if scene.arena_key != Some(frame.environment) {
        if let Some(entity) = scene.arena.take() {
            commands.entity(entity).despawn_recursive();
        }
        if let Some(mesh) = scene.arena_mesh.take() {
            meshes.remove(mesh.id());
        }
        let mesh = meshes.add(arena_mesh(frame.environment).into_mesh());
        scene.arena = Some(
            commands
                .spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: scene.surface_material.clone(),
                        ..default()
                    },
                    RenderLayers::layer(BATTLE_LAYER),
                    BattleArena,
                ))
                .id(),
        );
        scene.arena_mesh = Some(mesh);
        scene.arena_key = Some(frame.environment);
    }
    status.modeled_species.clear();
    status.source_art_species.clear();
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        let index = side.index();
        let Some(battler) = frame.battlers[index].as_ref() else {
            if let Some(instance) = scene.actors[index].take() {
                commands.entity(instance.entity).despawn_recursive();
                materials.remove(instance.material.id());
                if let Some(mesh) = instance.mesh {
                    meshes.remove(mesh.id());
                }
            }
            continue;
        };
        if battler.allow_species_model && !scene.species_meshes.contains_key(&battler.species_id) {
            let mesh = crate::battle_species_models::mesh(&battler.species_id)
                .or_else(|| {
                    crate::new_bark_actors::actor_props::battle_species_mesh(&battler.species_id)
                })
                .map(Arc::new);
            scene
                .species_meshes
                .insert(battler.species_id.clone(), mesh);
        }
        let modeled_data = if battler.allow_species_model {
            scene
                .species_meshes
                .get(&battler.species_id)
                .cloned()
                .flatten()
        } else {
            None
        };
        let modeled = modeled_data.is_some();
        let source_texture = frame
            .source
            .as_ref()
            .filter(|_| *flash_mode == BattleFlashMode::Full)
            .map(|source| &source.battler_textures[index])
            .filter(|texture| **texture != Handle::default())
            .unwrap_or(&battler.texture);
        let key = ActorKey {
            species: battler.species_id.clone(),
            modeled,
            texture: if modeled {
                Handle::default()
            } else {
                source_texture.clone()
            },
        };
        if modeled {
            status.modeled_species.push(battler.species_id.to_string());
        } else {
            status
                .source_art_species
                .push(battler.species_id.to_string());
        }
        if scene.actors[index]
            .as_ref()
            .is_none_or(|instance| instance.key != key)
        {
            if let Some(instance) = scene.actors[index].take() {
                commands.entity(instance.entity).despawn_recursive();
                materials.remove(instance.material.id());
                if let Some(mesh) = instance.mesh {
                    meshes.remove(mesh.id());
                }
            }
            let material = materials.add(if modeled {
                StandardMaterial {
                    perceptual_roughness: 0.92,
                    reflectance: 0.15,
                    ..default()
                }
            } else {
                StandardMaterial {
                    base_color_texture: Some(source_texture.clone()),
                    alpha_mode: AlphaMode::Mask(0.1),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                }
            });
            let transform = actor_pose(
                battler,
                &frame.cues,
                scene.elapsed,
                modeled,
                frame.source.as_ref(),
            );
            let mesh = modeled_data.as_ref().map(|data| {
                let mut mesh = data.as_ref().clone().into_mesh();
                mesh.asset_usage = bevy::render::render_asset::RenderAssetUsages::MAIN_WORLD
                    | bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD;
                meshes.add(mesh)
            });
            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: mesh.clone().unwrap_or_else(|| scene.fallback_mesh.clone()),
                        material: material.clone(),
                        transform,
                        visibility: if battler.visible {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        },
                        ..default()
                    },
                    RenderLayers::layer(BATTLE_LAYER),
                    BattleActor,
                ))
                .id();
            scene.actors[index] = Some(ActorInstance {
                entity,
                key,
                material,
                mesh,
                palette_key: (0xe4, *flash_mode),
                deformation_key: None,
            });
        }
        if let Some(instance) = &mut scene.actors[index] {
            let deformation = frame
                .source
                .as_ref()
                .and_then(|source| source.line_x_offsets);
            if instance.deformation_key != deformation {
                if let (Some(data), Some(mesh)) = (modeled_data.as_ref(), instance.mesh.as_ref()) {
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        mesh.insert_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            source_deformed_positions(data, battler.side, deformation.as_ref()),
                        );
                    }
                }
                instance.deformation_key = deformation;
            }
            let bgp = frame
                .source
                .as_ref()
                .map_or(0xe4, |source| source.battler_bgps[index]);
            if instance.palette_key != (bgp, *flash_mode) {
                if let (Some(data), Some(mesh)) = (modeled_data.as_ref(), instance.mesh.as_ref()) {
                    if let Some(mesh) = meshes.get_mut(mesh) {
                        let colors = data
                            .colors
                            .iter()
                            .map(|color| {
                                frame.source.as_ref().map_or(*color, |source| {
                                    source_model_color(
                                        *color,
                                        bgp,
                                        &source.battler_palettes[index],
                                        *flash_mode,
                                    )
                                })
                            })
                            .collect::<Vec<_>>();
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    }
                }
                instance.palette_key = (bgp, *flash_mode);
            }
            if let Some(material) = materials.get_mut(&instance.material) {
                material.unlit = !modeled || (bgp != 0xe4 && *flash_mode == BattleFlashMode::Full);
            }
            if let Ok((mut transform, mut visibility)) = actors.get_mut(instance.entity) {
                *transform = actor_pose(
                    battler,
                    &frame.cues,
                    scene.elapsed,
                    modeled,
                    frame.source.as_ref(),
                );
                *visibility = if battler.visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
    for (shadow, mut transform, mut visibility) in &mut contact_shadows {
        let Some(battler) = frame.battlers[shadow.0.index()]
            .as_ref()
            .filter(|battler| battler.visible)
        else {
            continue;
        };
        let pose = actor_pose(
            battler,
            &frame.cues,
            scene.elapsed,
            true,
            frame.source.as_ref(),
        );
        transform.translation = Vec3::new(pose.translation.x, 0.086, pose.translation.z);
        transform.scale = Vec3::new(0.38, 1.0, 0.23) * pose.scale;
        *visibility = Visibility::Visible;
    }
    for (i, entity) in scene.particles.iter().enumerate() {
        if frame.source.is_some() {
            continue;
        }
        let Some((position, scale, element)) = particle_pose(&frame.cues, i, scene.elapsed) else {
            continue;
        };
        if let Ok((mut transform, mut visibility, mut material)) = effects.get_mut(*entity) {
            *transform = Transform::from_translation(position).with_scale(Vec3::splat(scale));
            *visibility = Visibility::Visible;
            *material = scene.particle_materials[element].clone();
        }
    }
    if let Some(cue) = frame.cues.iter().find(|cue| {
        matches!(
            cue.kind,
            VisualBattleCueKind::Capture | VisualBattleCueKind::CaptureDeflect
        )
    }) {
        if let Some(entity) = scene.ball {
            if let Ok((mut transform, mut visibility)) = balls.get_mut(entity) {
                let deflected = cue.kind == VisualBattleCueKind::CaptureDeflect;
                let p = if deflected {
                    (cue.progress / 0.4).clamp(0.0, 1.0)
                } else {
                    cue.progress
                };
                let from = side_position(VisualBattleSide::Player) + Vec3::Y * 1.2;
                let to = side_position(VisualBattleSide::Enemy) + Vec3::Y * 0.18;
                transform.translation = from.lerp(to, p) + Vec3::Y * (PI * p).sin() * 2.2;
                if deflected && cue.progress > 0.4 {
                    let recoil = (cue.progress - 0.4) / 0.6;
                    transform.translation +=
                        Vec3::new(-4.0 * recoil, 1.8 * (PI * recoil).sin(), 2.0 * recoil);
                }
                transform.rotation = Quat::from_rotation_z(
                    (scene.elapsed * 13.0).sin() * 0.22 * (if p < 1.0 { 0.0 } else { 1.0 }),
                );
                *visibility = Visibility::Visible;
            }
        }
    }
}

fn toggle_battle_flash_mode(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    settings: Res<VoxelViewSettings>,
    mut mode: ResMut<BattleFlashMode>,
) {
    if settings.allow_f3_toggle && keys.is_some_and(|keys| keys.just_pressed(KeyCode::F4)) {
        *mode = if *mode == BattleFlashMode::Full {
            BattleFlashMode::Reduced
        } else {
            BattleFlashMode::Full
        };
    }
}

fn source_environment_palette(
    source: Option<&VisualBattleSourceFrame>,
    mode: BattleFlashMode,
) -> (f32, f32) {
    let Some(source) = source else {
        return (0.0, 0.0);
    };
    let strength = mode.palette_strength();
    let dark = f32::from(source.bgp & 3) / 3.0 * strength;
    let light = if source.bgp == 0 { strength } else { 0.0 };
    (dark, light)
}

/// Map authored model colors to the source palette's closest shade only while
/// its register is changed. Neutral frames retain every authored mesh color.
fn source_model_color(
    color: [f32; 4],
    bgp: u8,
    palette: &[[f32; 4]; 4],
    mode: BattleFlashMode,
) -> [f32; 4] {
    if bgp == 0xe4 {
        return color;
    }
    let shade = (0..4)
        .min_by(|a, b| {
            let distance = |index: usize| {
                (0..3)
                    .map(|channel| (palette[index][channel] - color[channel]).powi(2))
                    .sum::<f32>()
            };
            distance(*a).total_cmp(&distance(*b))
        })
        .unwrap_or(0);
    let mapped = palette[usize::from((bgp >> (shade * 2)) & 3)];
    let strength = mode.palette_strength();
    [
        color[0] + (mapped[0] - color[0]) * strength,
        color[1] + (mapped[1] - color[1]) * strength,
        color[2] + (mapped[2] - color[2]) * strength,
        color[3],
    ]
}

fn source_object_pose(center: Vec2, size: Vec2, elapsed: f32) -> Transform {
    let across = (center.x - 48.0) / 84.0;
    let baseline_y = 88.0 - across * 40.0;
    let position = side_position(VisualBattleSide::Player)
        .lerp(side_position(VisualBattleSide::Enemy), across)
        + Vec3::Y * (1.0 + (baseline_y - center.y) * 0.045);
    Transform::from_translation(position)
        .with_rotation(camera_pose(elapsed).rotation)
        .with_scale(Vec3::new(size.x * 0.045, size.y * 0.045, 1.0))
}

fn sync_source_objects(
    frame: Res<VisualBattleFrame>,
    status: Res<BattleViewStatus>,
    mode: Res<BattleFlashMode>,
    scene: Res<BattleScene>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut objects: Query<(
        &BattleSourceObject,
        &mut Transform,
        &mut Visibility,
        &Handle<StandardMaterial>,
    )>,
) {
    for (slot, mut transform, mut visibility, material) in &mut objects {
        *visibility = Visibility::Hidden;
        let Some(source) = frame.source.as_ref().filter(|_| status.active) else {
            continue;
        };
        let Some(object) = source.objects.iter().find(|object| object.slot == slot.0) else {
            continue;
        };
        *transform = source_object_pose(object.center, object.size, scene.elapsed);
        *visibility = Visibility::Visible;
        if let Some(material) = materials.get_mut(material) {
            material.base_color_texture = Some(if *mode == BattleFlashMode::Reduced {
                object.neutral_texture.clone()
            } else {
                object.texture.clone()
            });
        }
    }
}

/// Map the current LCD wave onto the height of the original model. This is a
/// geometry adaptation of the source scanlines, not a new attack or hit pose.
fn source_deformed_positions(
    data: &SurfaceMeshData,
    side: VisualBattleSide,
    offsets: Option<&[i8; 0x5f]>,
) -> Vec<[f32; 3]> {
    let Some(offsets) = offsets else {
        return data.positions.clone();
    };
    let height = data.positions.iter().map(|p| p[1]).fold(0.01_f32, f32::max);
    let (top, bottom) = if side == VisualBattleSide::Player {
        (48.0, 94.0)
    } else {
        (0.0, 54.0)
    };
    data.positions
        .iter()
        .map(|position| {
            let row =
                (bottom - (position[1] / height).clamp(0.0, 1.0) * (bottom - top)).round() as usize;
            let mut position = *position;
            position[0] += f32::from(offsets[row]) * 0.045 / MODEL_SCALE;
            position
        })
        .collect()
}

/// All added volumes are keyed by the actual presented move ID and existing
/// live source objects. No dialogue parsing, predicted target state or timer.
fn modeled_source_effect_pose(
    frame: &VisualBattleFrame,
    slot: usize,
) -> Option<(usize, Transform, [f32; 4])> {
    let source = frame.source.as_ref()?;
    let cue = frame
        .cues
        .iter()
        .find(|cue| cue.kind == VisualBattleCueKind::Move)?;
    let direction = (side_position(cue.side.opposite()) - side_position(cue.side)).normalize();
    let ring_rotation = Quat::from_rotation_arc(Vec3::Y, direction);
    if slot >= 10 {
        if cue.move_id.as_ref() != "PSYCHIC_M" || source.line_x_offsets.is_none() {
            return None;
        }
        // A violet aura makes the source's sustained wavy-screen phase legible
        // in depth. Its style is original; phase and lifetime are source-bound.
        let band = (slot - 10) as f32;
        let phase = f32::from(source.frame) * 0.095 + band * 2.1;
        let radius = 0.78 + phase.sin() * 0.06;
        let pose = Transform::from_translation(
            side_position(cue.side.opposite()) + Vec3::Y * (0.35 + band * 0.55),
        )
        .with_rotation(Quat::from_rotation_x(phase.sin() * 0.12))
        .with_scale(Vec3::splat(radius));
        return Some((0, pose, [0.65, 0.20, 1.0, 0.38]));
    }
    let object = source.objects.iter().find(|object| object.slot == slot)?;
    let center = source_object_pose(object.center, object.size, 0.0).translation;
    match (cue.move_id.as_ref(), object.object_id.as_ref()) {
        ("PSYCHIC_M", "BATTLE_ANIM_OBJ_WAVE") => {
            let radius = (object.size.max_element() * 0.025).max(0.20);
            Some((
                0,
                Transform::from_translation(center)
                    .with_rotation(ring_rotation)
                    .with_scale(Vec3::splat(radius)),
                [0.73, 0.31, 1.0, 0.82],
            ))
        }
        ("HYPER_BEAM", "BATTLE_ANIM_OBJ_BEAM") => {
            let length = object.size.x
                * side_position(VisualBattleSide::Player)
                    .distance(side_position(VisualBattleSide::Enemy))
                / 84.0;
            let radius = (object.size.y * 0.013).clamp(0.08, 0.24);
            Some((
                1,
                Transform::from_translation(center)
                    .with_rotation(ring_rotation)
                    .with_scale(Vec3::new(radius, length, radius)),
                [1.0, 0.73, 0.12, 0.72],
            ))
        }
        ("HYPER_BEAM", "BATTLE_ANIM_OBJ_BEAM_TIP") => Some((
            2,
            Transform::from_translation(center)
                .with_scale(Vec3::splat(object.size.max_element() * 0.020)),
            [1.0, 0.92, 0.46, 0.78],
        )),
        _ => None,
    }
}

fn sync_modeled_source_effects(
    frame: Res<VisualBattleFrame>,
    status: Res<BattleViewStatus>,
    mode: Res<BattleFlashMode>,
    scene: Res<BattleScene>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: Query<(
        &BattleModeledSourceEffect,
        &mut Transform,
        &mut Visibility,
        &mut Handle<Mesh>,
        &Handle<StandardMaterial>,
    )>,
) {
    for (slot, mut transform, mut visibility, mut mesh, material) in &mut effects {
        *visibility = Visibility::Hidden;
        if !status.active {
            continue;
        }
        let Some((shape, pose, mut color)) = modeled_source_effect_pose(&frame, slot.0) else {
            continue;
        };
        *transform = pose;
        *mesh = scene.source_effect_meshes[shape].clone();
        *visibility = Visibility::Visible;
        if *mode == BattleFlashMode::Reduced {
            color[3] *= 0.65;
        }
        if let Some(material) = materials.get_mut(material) {
            material.base_color = rgb(color);
        }
    }
}

fn source_wave_ring_mesh() -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    let point = |i: usize, j: usize| {
        let angle = i as f32 / 40.0 * TAU;
        let tube = j as f32 / 6.0 * TAU;
        Vec3::new(
            angle.cos() * (1.0 + 0.055 * tube.cos()),
            tube.sin() * 0.055,
            angle.sin() * (1.0 + 0.055 * tube.cos()),
        )
    };
    for i in 0..40 {
        for j in 0..6 {
            let [a, b, c, d] = [
                point(i, j),
                point(i + 1, j),
                point(i + 1, j + 1),
                point(i, j + 1),
            ];
            triangle(&mut mesh, a, b, c, [1.0; 4]);
            triangle(&mut mesh, a, c, d, [1.0; 4]);
        }
    }
    mesh
}

fn actor_pose(
    battler: &VisualBattleBattler,
    cues: &[VisualBattleCue],
    elapsed: f32,
    modeled: bool,
    source: Option<&VisualBattleSourceFrame>,
) -> Transform {
    let origin = side_position(battler.side);
    let direction = (side_position(battler.side.opposite()) - origin).normalize();
    let mut transform = Transform::from_translation(origin);
    let mut scale = 1.0 + 0.012 * (elapsed * 2.2 + battler.side.index() as f32).sin();
    let mut roll = 0.0;
    for cue in cues.iter().filter(|cue| cue.side == battler.side) {
        match cue.kind {
            VisualBattleCueKind::Move => {
                if source.is_some() {
                    continue;
                }
                let envelope = (PI * cue.progress).sin().max(0.0);
                if cue.damaging {
                    transform.translation += direction * envelope * 0.62;
                    transform.translation.y += envelope * 0.14;
                    roll = -0.07 * envelope;
                } else {
                    transform.translation.y += envelope * 0.10;
                }
            }
            VisualBattleCueKind::Impact => {
                transform.translation -= direction * 0.10 * (elapsed * 35.0).sin();
                roll = (elapsed * 32.0).sin() * 0.08;
            }
            VisualBattleCueKind::Faint => {
                scale *= (1.0 - cue.progress).max(0.001);
                roll = cue.progress * 0.9;
            }
            VisualBattleCueKind::Withdraw => scale *= (1.0 - cue.progress * 1.8).clamp(0.001, 1.0),
            VisualBattleCueKind::SendOut => scale *= (cue.progress * 5.0).clamp(0.001, 1.0),
            VisualBattleCueKind::Capture | VisualBattleCueKind::CaptureDeflect => {}
        }
    }
    if let Some(source) = source {
        // Translate source image-space motion onto the actor-to-actor axis.
        let offset = source.battler_offsets[battler.side.index()];
        let forward = (side_position(VisualBattleSide::Enemy)
            - side_position(VisualBattleSide::Player))
        .normalize();
        transform.translation += forward * offset.x * 0.045 + Vec3::Y * offset.y * 0.045;
    }
    if modeled {
        // Open the stage toward the viewer while retaining the opponent-facing
        // direction. An exact side-on lineup hides faces and thin silhouettes.
        let camera_direction =
            (camera_pose(elapsed).translation - origin) * Vec3::new(1.0, 0.0, 1.0);
        let facing = direction
            .lerp(camera_direction.normalize(), 0.40)
            .normalize();
        transform.rotation =
            Quat::from_rotation_y(facing.x.atan2(facing.z)) * Quat::from_rotation_z(roll);
        transform.scale = Vec3::splat(MODEL_SCALE * scale);
    } else {
        let camera = camera_pose(elapsed);
        transform.translation.y += 0.94 * scale;
        transform.rotation = camera.rotation * Quat::from_rotation_z(roll);
        transform.scale =
            Vec3::new(battler.texture_size.x / battler.texture_size.y, 1.0, 1.0) * scale;
    }
    transform
}
fn element_index(element: &str) -> usize {
    match element {
        "FIRE" => 1,
        "WATER" => 2,
        "GRASS" | "BUG" => 3,
        "ELECTRIC" => 4,
        "ICE" => 5,
        "PSYCHIC" | "GHOST" | "DRAGON" => 6,
        "POISON" => 7,
        _ => 0,
    }
}
fn element_color(element: &str) -> [f32; 4] {
    match element_index(element) {
        1 => [1.0, 0.32, 0.055, 1.0],
        2 => [0.18, 0.67, 1.0, 1.0],
        3 => [0.47, 0.92, 0.22, 1.0],
        4 => [1.0, 0.88, 0.12, 1.0],
        5 => [0.59, 0.96, 1.0, 1.0],
        6 => [0.95, 0.43, 1.0, 1.0],
        7 => [0.62, 0.24, 0.8, 1.0],
        _ => [1.0, 0.85, 0.59, 1.0],
    }
}
fn particle_pose(cues: &[VisualBattleCue], i: usize, elapsed: f32) -> Option<(Vec3, f32, usize)> {
    let cue = cues
        .iter()
        .find(|cue| cue.kind == VisualBattleCueKind::Impact)
        .or_else(|| {
            cues.iter().find(|cue| {
                matches!(
                    cue.kind,
                    VisualBattleCueKind::Move | VisualBattleCueKind::SendOut
                )
            })
        })?;
    let angle = i as f32 * 2.399_963;
    let origin = side_position(cue.side) + Vec3::Y;
    match cue.kind {
        VisualBattleCueKind::Impact => {
            let phase = (elapsed * 3.0 + i as f32 / PARTICLES as f32).fract();
            let vector = Vec3::new(angle.cos(), 0.4 + (i % 4) as f32 * 0.25, angle.sin());
            Some((origin + vector * phase * 0.9, 0.065 * (1.0 - phase), 0))
        }
        VisualBattleCueKind::SendOut => {
            if cue.progress > 0.65 {
                return None;
            }
            let radius = cue.progress * 2.4;
            Some((
                origin
                    + Vec3::new(
                        angle.cos() * radius,
                        angle.sin() * radius,
                        (angle * 0.7).sin() * radius,
                    ),
                0.045,
                4,
            ))
        }
        VisualBattleCueKind::Move => {
            if cue.progress < 0.12 || cue.progress > 0.88 {
                return None;
            }
            let phase = ((cue.progress - 0.12) / 0.76 - i as f32 * 0.018).clamp(0.0, 1.0);
            if phase == 0.0 {
                return None;
            }
            if cue.damaging && element_index(&cue.element) != 0 {
                let target = side_position(cue.side.opposite()) + Vec3::Y;
                let drift = Vec3::new(angle.cos(), angle.sin(), (angle * 0.8).sin()) * 0.13;
                Some((
                    origin.lerp(target, phase) + drift,
                    0.07 + (i % 3) as f32 * 0.023,
                    element_index(&cue.element),
                ))
            } else if !cue.damaging {
                let a = angle + elapsed * 2.3;
                Some((
                    origin + Vec3::new(a.cos() * 0.55, phase * 1.4 - 0.5, a.sin() * 0.55),
                    0.055,
                    element_index(&cue.element),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

// Arena source generator. All geometry is original authored procedural art;
// palette sources live in art/battles/arena-palettes.json, not an external game pack.
fn arena_mesh(environment: VisualBattleEnvironment) -> SurfaceMeshData {
    let p = palette(environment);
    let mut mesh = SurfaceMeshData::default();
    append_cylinder(
        &mut mesh,
        Vec3::new(0.0, -0.18, 0.0),
        24.0,
        0.16,
        96,
        p.ground,
    );
    append_cylinder(&mut mesh, Vec3::new(0.0, -0.02, 0.0), 5.4, 0.10, 64, p.edge);
    append_cylinder(
        &mut mesh,
        Vec3::new(0.0, 0.035, 0.0),
        5.2,
        0.035,
        64,
        p.stage,
    );
    for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
        let center = side_position(side) - Vec3::Y * 0.036;
        append_ring(&mut mesh, center, 1.48, 1.53, p.accent);
        append_ring(&mut mesh, center, 1.56, 1.59, p.edge);
    }
    let interior = environment == VisualBattleEnvironment::Interior;
    for i in 0..28 {
        let angle = i as f32 / 28.0 * TAU;
        let radius = 9.5 + (i % 3) as f32 * 1.9;
        let center = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        if matches!(
            environment,
            VisualBattleEnvironment::Meadow | VisualBattleEnvironment::Forest
        ) {
            // Preserve the foreground sightline. Distant tree silhouettes give
            // the arena geographic depth without obscuring either battler.
            if center.z < 2.0 || center.x.abs() > 11.0 {
                let tree =
                    crate::new_bark_models::model(crate::new_bark_models::ModelKind::TreeLod);
                let scale = (3.5 + (i % 4) as f32 * 0.38) / (tree.max[1] - tree.min[1]);
                append_transformed(
                    &mut mesh,
                    &tree.surface_mesh(),
                    center,
                    Vec3::splat(scale),
                    Quat::from_rotation_y(angle),
                );
            }
            for j in 0..3 {
                let c = center + Vec3::new(j as f32 * 0.27, 0.0, 0.3);
                append_cone(
                    &mut mesh,
                    c,
                    0.12,
                    0.43 + (i % 3) as f32 * 0.08,
                    4,
                    p.ground,
                );
            }
        } else if interior {
            if i % 2 == 0 {
                append_cylinder(&mut mesh, center, 0.40, 3.5, 8, p.edge);
                append_cylinder(&mut mesh, center + Vec3::Y * 3.5, 0.58, 0.20, 8, p.accent);
            }
        } else if matches!(
            environment,
            VisualBattleEnvironment::Cave | VisualBattleEnvironment::Ice
        ) {
            if center.z < 4.0 || center.x.abs() > 8.0 {
                append_cone(
                    &mut mesh,
                    center,
                    1.1 + (i % 4) as f32 * 0.16,
                    2.0 + (i % 5) as f32 * 0.65,
                    5,
                    if i % 3 == 0 { p.edge } else { p.ground },
                );
                if environment == VisualBattleEnvironment::Ice {
                    append_cone(
                        &mut mesh,
                        center + Vec3::new(0.6, 0.0, 0.2),
                        0.25,
                        2.2,
                        5,
                        p.accent,
                    );
                }
            }
        } else if i % 2 == 0 {
            append_ring(&mut mesh, center + Vec3::Y * 0.01, 1.1, 1.12, p.accent);
        }
    }
    if interior {
        for i in -9..10 {
            for j in -9..10 {
                let center = Vec3::new(i as f32 * 1.4, 0.001, j as f32 * 1.4);
                if center.length() > 6.0 {
                    append_cylinder(
                        &mut mesh,
                        center,
                        0.52,
                        0.014,
                        4,
                        if (i + j) % 2 == 0 { p.stage } else { p.ground },
                    );
                }
            }
        }
    }
    mesh
}
fn append_transformed(
    out: &mut SurfaceMeshData,
    source: &SurfaceMeshData,
    translation: Vec3,
    scale: Vec3,
    rotation: Quat,
) {
    let base = out.positions.len() as u32;
    out.positions.extend(
        source
            .positions
            .iter()
            .map(|p| (translation + rotation * (Vec3::from_array(*p) * scale)).to_array()),
    );
    out.normals.extend(source.normals.iter().map(|n| {
        (rotation * (Vec3::from_array(*n) / scale))
            .normalize()
            .to_array()
    }));
    out.uvs.extend_from_slice(&source.uvs);
    out.colors.extend_from_slice(&source.colors);
    out.indices
        .extend(source.indices.iter().map(|index| base + index));
}
fn triangle(mesh: &mut SurfaceMeshData, a: Vec3, b: Vec3, c: Vec3, color: [f32; 4]) {
    let normal = (b - a).cross(c - a).normalize_or_zero();
    if normal == Vec3::ZERO {
        return;
    }
    let base = mesh.positions.len() as u32;
    for p in [a, b, c] {
        mesh.positions.push(p.to_array());
        mesh.normals.push(normal.to_array());
        mesh.uvs.push([0.0, 0.0]);
        mesh.colors.push(color);
    }
    mesh.indices.extend([base, base + 1, base + 2]);
}
fn append_cylinder(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    radius: f32,
    height: f32,
    segments: usize,
    color: [f32; 4],
) {
    for i in 0..segments {
        let a = i as f32 / segments as f32 * TAU;
        let b = (i + 1) as f32 / segments as f32 * TAU;
        let p = center + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius);
        let q = center + Vec3::new(b.cos() * radius, 0.0, b.sin() * radius);
        let top = Vec3::Y * height;
        triangle(mesh, center + top, q + top, p + top, color);
        let side = [color[0] * 0.82, color[1] * 0.82, color[2] * 0.82, color[3]];
        triangle(mesh, p, q + top, q, side);
        triangle(mesh, p, p + top, q + top, side);
    }
}
fn append_cone(
    mesh: &mut SurfaceMeshData,
    center: Vec3,
    radius: f32,
    height: f32,
    segments: usize,
    color: [f32; 4],
) {
    for i in 0..segments {
        let a = i as f32 / segments as f32 * TAU;
        let b = (i + 1) as f32 / segments as f32 * TAU;
        triangle(
            mesh,
            center + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius),
            center + Vec3::Y * height,
            center + Vec3::new(b.cos() * radius, 0.0, b.sin() * radius),
            color,
        );
    }
}
fn append_ring(mesh: &mut SurfaceMeshData, center: Vec3, inner: f32, outer: f32, color: [f32; 4]) {
    for i in 0..64 {
        let a = i as f32 / 64.0 * TAU;
        let b = (i + 1) as f32 / 64.0 * TAU;
        let p = Vec3::new(a.cos(), 0.0, a.sin());
        let q = Vec3::new(b.cos(), 0.0, b.sin());
        triangle(
            mesh,
            center + p * inner,
            center + q * outer,
            center + p * outer,
            color,
        );
        triangle(
            mesh,
            center + p * inner,
            center + q * inner,
            center + q * outer,
            color,
        );
    }
}
/// One cheap translucent fan with a soft radial falloff, shared by both sides.
/// Grounding follows only presented actor visibility/pose, including fainting.
fn contact_shadow_mesh() -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    for i in 0..48 {
        let a = i as f32 / 48.0 * TAU;
        let b = (i + 1) as f32 / 48.0 * TAU;
        let base = mesh.positions.len() as u32;
        for (point, alpha) in [
            (Vec3::ZERO, 0.40),
            (Vec3::new(b.cos(), 0.0, b.sin()), 0.0),
            (Vec3::new(a.cos(), 0.0, a.sin()), 0.0),
        ] {
            mesh.positions.push(point.to_array());
            mesh.normals.push(Vec3::Y.to_array());
            mesh.uvs.push([0.0, 0.0]);
            mesh.colors.push([0.015, 0.022, 0.012, alpha]);
        }
        mesh.indices.extend([base, base + 1, base + 2]);
    }
    mesh
}

fn capture_ball_mesh() -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    for row in 0..12 {
        for col in 0..24 {
            let vertex = |r: usize, c: usize| {
                let lat = r as f32 / 12.0 * PI;
                let lon = c as f32 / 24.0 * TAU;
                Vec3::new(lat.sin() * lon.cos(), lat.cos(), lat.sin() * lon.sin()) * 0.19
            };
            let color = if row < 5 {
                [0.82, 0.08, 0.12, 1.0]
            } else if row < 7 {
                [0.10, 0.12, 0.15, 1.0]
            } else {
                [0.94, 0.94, 0.88, 1.0]
            };
            let a = vertex(row, col);
            let b = vertex(row + 1, col);
            let c = vertex(row, col + 1);
            let d = vertex(row + 1, col + 1);
            if row > 0 {
                triangle(&mut mesh, a, c, b, color);
            }
            if row < 11 {
                triangle(&mut mesh, c, d, b, color);
            }
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_authored_arena_has_finite_grounded_geometry() {
        for environment in [
            VisualBattleEnvironment::Meadow,
            VisualBattleEnvironment::Forest,
            VisualBattleEnvironment::Cave,
            VisualBattleEnvironment::Water,
            VisualBattleEnvironment::Interior,
            VisualBattleEnvironment::Ice,
        ] {
            let mesh = arena_mesh(environment);
            assert!(mesh.positions.len() > 1000);
            assert_eq!(mesh.positions.len(), mesh.normals.len());
            assert_eq!(mesh.positions.len(), mesh.colors.len());
            assert!(mesh.positions.iter().flatten().all(|v| v.is_finite()));
            assert!(
                mesh.indices
                    .iter()
                    .all(|i| (*i as usize) < mesh.positions.len())
            );
        }
    }
    #[test]
    fn only_explicit_impact_cues_recoil_a_battler() {
        let battler = VisualBattleBattler {
            side: VisualBattleSide::Enemy,
            species_id: Arc::from("RATTATA"),
            party_index: Some(0),
            texture: Handle::weak_from_u128(1),
            texture_size: Vec2::splat(56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        };
        let cue = VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: Arc::from("TACKLE"),
            element: Arc::from("NORMAL"),
            progress: 0.5,
            damaging: true,
        };
        assert_eq!(
            actor_pose(&battler, &[cue], 0.0, true, None).translation,
            side_position(VisualBattleSide::Enemy)
        );
    }
    #[test]
    fn move_particles_do_not_invent_a_hit() {
        let cue = VisualBattleCue {
            kind: VisualBattleCueKind::Move,
            side: VisualBattleSide::Player,
            move_id: Arc::from("TACKLE"),
            element: Arc::from("NORMAL"),
            progress: 0.5,
            damaging: true,
        };
        assert!(particle_pose(&[cue], 0, 0.5).is_none());
    }
    #[test]
    fn faint_and_switch_poses_never_produce_invalid_transforms() {
        let battler = VisualBattleBattler {
            side: VisualBattleSide::Player,
            species_id: Arc::from("CHIKORITA"),
            party_index: Some(3),
            texture: Handle::weak_from_u128(1),
            texture_size: Vec2::splat(56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        };
        for kind in [
            VisualBattleCueKind::Faint,
            VisualBattleCueKind::SendOut,
            VisualBattleCueKind::Withdraw,
        ] {
            for progress in [0.0, 0.5, 1.0] {
                let cue = VisualBattleCue {
                    kind,
                    side: battler.side,
                    move_id: Arc::from(""),
                    element: Arc::from(""),
                    progress,
                    damaging: false,
                };
                let pose = actor_pose(&battler, &[cue], 0.2, true, None);
                assert!(pose.translation.is_finite());
                assert!(pose.scale.min_element() > 0.0);
            }
        }
    }
    #[test]
    fn capture_ball_is_original_volume_with_outward_normals() {
        let mesh = capture_ball_mesh();
        assert!(mesh.indices.len() > 500);
        for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
            assert!(Vec3::from_array(*p).dot(Vec3::from_array(*n)) > 0.0);
        }
    }

    fn headless_battle_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .insert_resource(VoxelViewSettings {
                enabled: true,
                ..Default::default()
            })
            .add_plugins(BattleViewPlugin);
        let battler = |side, species: &str| VisualBattleBattler {
            side,
            species_id: Arc::from(species),
            party_index: Some(side.index()),
            texture: Handle::weak_from_u128(20 + side.index() as u128),
            texture_size: Vec2::splat(56.0),
            visible: true,
            allow_species_model: true,
            shiny: false,
        };
        app.insert_resource(VisualBattleFrame {
            active: true,
            map_id: Arc::from("Route29"),
            battlers: [
                Some(battler(VisualBattleSide::Player, "CYNDAQUIL")),
                Some(battler(VisualBattleSide::Enemy, "UNAUTHORED_SPECIES")),
            ],
            ..Default::default()
        });
        app
    }

    #[test]
    fn battle_scene_switches_modes_restores_world_and_bounds_entity_counts() {
        let mut app = headless_battle_app();
        app.update();
        assert!(app.world().resource::<BattleViewStatus>().active);
        assert_eq!(
            app.world().resource::<BattleViewStatus>().modeled_species,
            ["CYNDAQUIL"]
        );
        assert_eq!(
            app.world()
                .resource::<BattleViewStatus>()
                .source_art_species,
            ["UNAUTHORED_SPECIES"]
        );
        let count = app.world().entities().len();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(
            app.world().entities().len(),
            count,
            "idle must reuse the scene and bounded particle pool"
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        assert!(!app.world().resource::<BattleViewStatus>().active);
        {
            let world = app.world_mut();
            assert!(
                !world
                    .query_filtered::<&Camera, With<BattleCamera>>()
                    .single(world)
                    .is_active
            );
            assert!(
                world
                    .query_filtered::<&Visibility, With<BattleActor>>()
                    .iter(world)
                    .all(|v| *v == Visibility::Hidden)
            );
        }
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        assert!(app.world().resource::<BattleViewStatus>().active);
        *app.world_mut().resource_mut::<VisualBattleFrame>() = VisualBattleFrame::default();
        app.update();
        assert!(!app.world().resource::<BattleViewStatus>().active);
        let world = app.world_mut();
        assert!(
            !world
                .query_filtered::<&Camera, With<BattleCamera>>()
                .single(world)
                .is_active
        );
    }

    fn source_test_frame(bgp: u8) -> VisualBattleSourceFrame {
        VisualBattleSourceFrame {
            frame: 17,
            bgp,
            battler_bgps: [bgp; 2],
            battler_palettes: [[
                [1.0, 1.0, 1.0, 1.0],
                [0.7, 0.7, 0.7, 1.0],
                [0.3, 0.3, 0.3, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ]; 2],
            battler_textures: [Handle::weak_from_u128(50), Handle::weak_from_u128(51)],
            battler_offsets: [Vec2::new(4.0, 0.0), Vec2::ZERO],
            screen_offset: Vec2::ZERO,
            line_x_offsets: None,
            objects: vec![crystal_render_api::VisualBattleSourceObject {
                slot: 0,
                object_id: Arc::from("PRESENTED_SOURCE_OBJECT"),
                texture: Handle::weak_from_u128(60),
                neutral_texture: Handle::weak_from_u128(61),
                center: Vec2::new(92.0, 68.0),
                size: Vec2::splat(16.0),
            }],
        }
    }

    #[test]
    fn source_palette_full_and_reduced_preserve_timing_and_limit_contrast() {
        let source = source_test_frame(0x1b);
        let dark = [0.0, 0.0, 0.0, 1.0];
        let palette = &source.battler_palettes[0];
        assert_eq!(
            source_model_color(dark, 0x1b, palette, BattleFlashMode::Full),
            [1.0; 4]
        );
        let reduced = source_model_color(dark, 0x1b, palette, BattleFlashMode::Reduced);
        assert_eq!(reduced, [0.12, 0.12, 0.12, 1.0]);
        assert_eq!(
            source_model_color(dark, 0xe4, palette, BattleFlashMode::Full),
            dark
        );
        assert_eq!(
            source_environment_palette(Some(&source), BattleFlashMode::Full),
            (1.0, 0.0)
        );
        assert_eq!(
            source_environment_palette(Some(&source), BattleFlashMode::Reduced),
            (0.12, 0.0)
        );
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source);
        let before = app.world().resource::<VisualBattleFrame>().clone();
        app.update();
        let source_pose = |app: &mut App| {
            let world = app.world_mut();
            world
                .query::<(&BattleSourceObject, &Transform)>()
                .iter(world)
                .find(|(slot, _)| slot.0 == 0)
                .unwrap()
                .1
                .to_owned()
        };
        let full = source_pose(&mut app);
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        let reduced = source_pose(&mut app);
        assert_eq!(full.translation, reduced.translation);
        assert_eq!(full.scale, reduced.scale);
        assert_eq!(
            *app.world().resource::<VisualBattleFrame>(),
            before,
            "flash intensity cannot rewrite the source frame, cue phase, HP or command data"
        );
        let world = app.world_mut();
        let material = world
            .query::<(&BattleSourceObject, &Handle<StandardMaterial>)>()
            .iter(world)
            .find(|(slot, _)| slot.0 == 0)
            .unwrap()
            .1
            .clone();
        assert_eq!(
            world
                .resource::<Assets<StandardMaterial>>()
                .get(&material)
                .unwrap()
                .base_color_texture,
            Some(Handle::weak_from_u128(61)),
            "reduced mode retains OAM/frame but suppresses object palette cycling"
        );
    }

    #[test]
    fn psychic_and_beam_volumes_require_current_source_objects() {
        let mut frame = VisualBattleFrame {
            source: Some(source_test_frame(0xe4)),
            cues: vec![VisualBattleCue {
                kind: VisualBattleCueKind::Move,
                side: VisualBattleSide::Player,
                move_id: Arc::from("PSYCHIC_M"),
                element: Arc::from("PSYCHIC"),
                progress: 0.3,
                damaging: true,
            }],
            ..Default::default()
        };
        assert!(modeled_source_effect_pose(&frame, 0).is_none());
        frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_WAVE");
        let ring = modeled_source_effect_pose(&frame, 0).unwrap();
        assert_eq!(ring.0, 0);
        assert!(ring.1.translation.is_finite());
        assert!(modeled_source_effect_pose(&frame, 1).is_none());
        assert!(modeled_source_effect_pose(&frame, 10).is_none());
        frame.source.as_mut().unwrap().line_x_offsets = Some([5; 0x5f]);
        assert!(modeled_source_effect_pose(&frame, 10).is_some());
        frame.cues[0].move_id = Arc::from("HYPER_BEAM");
        assert!(modeled_source_effect_pose(&frame, 0).is_none());
        assert!(modeled_source_effect_pose(&frame, 10).is_none());
        frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_BEAM");
        let beam = modeled_source_effect_pose(&frame, 0).unwrap();
        assert_eq!(beam.0, 1);
        assert_eq!(beam.1.translation, ring.1.translation);
        frame.source.as_mut().unwrap().objects.clear();
        assert!(modeled_source_effect_pose(&frame, 0).is_none());
        frame.source = None;
        assert!((0..13).all(|slot| modeled_source_effect_pose(&frame, slot).is_none()));
    }

    #[test]
    fn source_scanline_deformation_restores_original_geometry() {
        let data = SurfaceMeshData {
            positions: vec![[0.0, 0.0, 0.0], [0.2, 0.5, 0.1], [0.0, 1.0, 0.0]],
            ..Default::default()
        };
        let source = [5_i8; 0x5f];
        for side in [VisualBattleSide::Player, VisualBattleSide::Enemy] {
            let deformed = source_deformed_positions(&data, side, Some(&source));
            assert_ne!(deformed, data.positions);
            assert!(deformed.iter().all(|p| p.iter().all(|v| v.is_finite())));
            assert_eq!(source_deformed_positions(&data, side, None), data.positions);
        }
        let ring = source_wave_ring_mesh();
        assert!(!ring.indices.is_empty());
        assert!(
            ring.positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite()))
        );
    }

    #[test]
    fn modeled_source_effects_restore_on_interruption_and_reduce_only_intensity() {
        let mut app = headless_battle_app();
        {
            let mut frame = app.world_mut().resource_mut::<VisualBattleFrame>();
            frame.source = Some(source_test_frame(0xe4));
            frame.source.as_mut().unwrap().objects[0].object_id = Arc::from("BATTLE_ANIM_OBJ_WAVE");
            frame.source.as_mut().unwrap().line_x_offsets = Some([5; 0x5f]);
            frame.cues.push(VisualBattleCue {
                kind: VisualBattleCueKind::Move,
                side: VisualBattleSide::Player,
                move_id: Arc::from("PSYCHIC_M"),
                element: Arc::from("PSYCHIC"),
                progress: 0.3,
                damaging: true,
            });
        }
        app.update();
        let poses = |app: &mut App| {
            let mut query = app
                .world_mut()
                .query::<(&BattleModeledSourceEffect, &Transform, &Visibility)>();
            query
                .iter(app.world())
                .map(|(slot, pose, visibility)| (slot.0, *pose, *visibility))
                .collect::<Vec<_>>()
        };
        let full = poses(&mut app);
        assert_eq!(
            full.iter()
                .filter(|(_, _, visibility)| *visibility == Visibility::Visible)
                .count(),
            4
        );
        app.world_mut().insert_resource(BattleFlashMode::Reduced);
        app.update();
        assert_eq!(poses(&mut app), full);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        assert!(
            poses(&mut app)
                .iter()
                .all(|(_, _, visibility)| *visibility == Visibility::Hidden)
        );
    }

    #[test]
    fn source_effect_pool_restores_after_toggle_and_interruption() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().source = Some(source_test_frame(0x1b));
        app.update();
        let counts = (
            app.world().entities().len(),
            app.world().resource::<Assets<Mesh>>().len(),
            app.world().resource::<Assets<StandardMaterial>>().len(),
        );
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(
            (
                app.world().entities().len(),
                app.world().resource::<Assets<Mesh>>().len(),
                app.world().resource::<Assets<StandardMaterial>>().len()
            ),
            counts
        );
        let visible_count = |app: &mut App| {
            let world = app.world_mut();
            world
                .query_filtered::<&Visibility, With<BattleSourceObject>>()
                .iter(world)
                .filter(|visibility| **visibility == Visibility::Visible)
                .count()
        };
        assert_eq!(visible_count(&mut app), 1);
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = false;
        app.update();
        assert_eq!(
            visible_count(&mut app),
            0,
            "classic must not double-render source objects"
        );
        app.world_mut().resource_mut::<VoxelViewSettings>().enabled = true;
        app.update();
        assert_eq!(visible_count(&mut app), 1);
        app.world_mut().resource_mut::<VisualBattleFrame>().source = None;
        app.update();
        assert_eq!(visible_count(&mut app), 0);
        assert_eq!(
            app.world().resource::<BattleScene>().actors[0]
                .as_ref()
                .unwrap()
                .palette_key
                .0,
            0xe4
        );
        *app.world_mut().resource_mut::<VisualBattleFrame>() = VisualBattleFrame::default();
        app.update();
        assert_eq!(
            visible_count(&mut app),
            0,
            "capture/Dex/world interruptions retire all source effects"
        );
    }

    #[test]
    fn rejected_battle_frame_cannot_hide_the_classic_scene() {
        let mut app = headless_battle_app();
        app.world_mut().resource_mut::<VisualBattleFrame>().battlers[0]
            .as_mut()
            .unwrap()
            .texture = Handle::default();
        app.update();
        let status = app.world().resource::<BattleViewStatus>();
        assert!(!status.active);
        assert!(status.last_error.is_some());
    }
}
