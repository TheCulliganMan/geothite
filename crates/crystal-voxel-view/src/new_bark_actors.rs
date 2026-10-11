//! Articulated characters consume the authoritative presentation frame only.
//! No input, collision, script, inventory, facing, or warp state is owned here.
#[path = "johto_actor_props.rs"]
pub(crate) mod actor_props;
#[path = "johto_characters.rs"]
pub(crate) mod character_meshes;

use crate::{
    ActorIdCache, TerrainRevisionCache, VOXEL_RENDER_LAYER, VoxelActorCard, VoxelViewStatus,
    actor_foot, resolved_footing_height, visual_point_to_voxel, VoxelMaterial,
};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use character_meshes::*;
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame};
use std::collections::{HashMap, HashSet};
use std::f32::consts::{PI, TAU};

#[derive(Component)]
pub(crate) struct ModeledActor;

/// One ordinary walking cycle travels this many gameplay tiles. The stance foot
/// moves backward at exactly the root's speed; there is no timer-driven skating.
const CYCLE_DISTANCE: f32 = 1.12;
const RUN_CYCLE_DISTANCE: f32 = 2.80;

fn cycle_distance(run: f32) -> f32 {
    CYCLE_DISTANCE + (RUN_CYCLE_DISTANCE - CYCLE_DISTANCE) * run
}
#[derive(Clone, Copy, Debug)]
struct Motion {
    last_world_foot: Vec2,
    yaw: f32,
    phase: f32,
    walk_weight: f32,
    still_for: f32,
    sample_interval: f32,
    speed: f32,
    run_weight: f32,
}
impl Motion {
    fn new(foot: Vec2, facing: Vec2) -> Self {
        Self {
            last_world_foot: foot,
            yaw: facing_yaw(facing),
            phase: 0.0,
            walk_weight: 0.0,
            still_for: 1.0,
            sample_interval: 1.0 / 60.0,
            speed: 0.0,
            run_weight: 0.0,
        }
    }
    fn advance(&mut self, foot: Vec2, facing: Option<Vec2>, scale: f32, dt: f32) {
        let delta = foot - self.last_world_foot;
        let distance = delta.length();
        let dt = dt.clamp(0.0, 0.25);
        let discontinuity = !distance.is_finite() || distance > scale * 1.5;
        let moving = distance > 0.001 && !discontinuity;
        let target_facing = facing
            .filter(|f| f.is_finite() && f.length_squared() > 0.5)
            .or_else(|| moving.then(|| delta.normalize()));
        if discontinuity {
            self.phase = 0.0;
            self.walk_weight = 0.0;
            self.still_for = 1.0;
            self.speed = 0.0;
            self.run_weight = 0.0;
            if let Some(facing) = target_facing {
                self.yaw = facing_yaw(facing);
            }
        } else {
            let previous_run = self.run_weight;
            if moving {
                let interval = if self.walk_weight > 0.05 {
                    self.still_for + dt
                } else {
                    dt
                };
                let interval = interval.max(1.0 / 240.0);
                if self.walk_weight > 0.05 {
                    let observed = interval.clamp(1.0 / 240.0, 0.2);
                    self.sample_interval += (observed - self.sample_interval) * 0.35;
                }
                // Measure only distance the authoritative frame actually
                // published. This never predicts or advances the root.
                let observed_speed = distance / scale / interval;
                self.speed += (observed_speed - self.speed) * (1.0 - (-10.0 * interval).exp());
                self.still_for = 0.0;
            } else {
                self.still_for += dt;
            }
            // A publication gap is not a stop. At 9Hz the former fixed 65ms
            // timeout dropped to idle on every held sample. Learn the actual
            // sample cadence, with a bounded grace period so an explicit stop
            // still settles promptly. Root placement never extrapolates.
            let stop_grace = (self.sample_interval * 1.5)
                .max(dt * 2.25)
                .clamp(0.08, 0.30);
            let target_weight = if self.still_for < stop_grace {
                1.0
            } else {
                0.0
            };
            let response = if target_weight > self.walk_weight {
                14.0
            } else {
                10.0
            };
            self.walk_weight += (target_weight - self.walk_weight) * (1.0 - (-response * dt).exp());
            // Walking at 60Hz is ~7.466 tiles/s in the production host.
            // A walking-only 1.12-tile cycle produced 13.3 footfalls/s. Blend
            // into a genuinely different running gait: a longer distance per
            // cycle, short planted stance, and an aerial return on both legs.
            let target_run = if target_weight > 0.0 {
                ((self.speed - 1.6) / 2.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            self.run_weight += (target_run - self.run_weight) * (1.0 - (-8.0 * dt).exp());
            if moving {
                let inverse_cycle = (1.0 / cycle_distance(previous_run)
                    + 1.0 / cycle_distance(self.run_weight))
                    * 0.5;
                self.phase = (self.phase + distance / scale * TAU * inverse_cycle).rem_euclid(TAU);
            }
            if self.walk_weight < 0.001 {
                self.walk_weight = 0.0;
                self.speed = 0.0;
            }
            if let Some(facing) = target_facing {
                let difference = (facing_yaw(facing) - self.yaw + PI).rem_euclid(TAU) - PI;
                // Bound the angular step instead of nearly snapping a 90-degree
                // turn in one slow host frame. World-facing remains the target.
                self.yaw += (difference * (1.0 - (-12.0 * dt).exp())).clamp(-6.0 * dt, 6.0 * dt);
                self.yaw = (self.yaw + PI).rem_euclid(TAU) - PI;
            }
        }
        self.last_world_foot = foot;
    }
}

struct Instance {
    entity: Entity,
    kind: CharacterKind,
    motion: Motion,
    idle_offset: f32,
    person: crate::open_people_skin::Person,
}

struct PropInstance {
    entity: Entity,
    kind: actor_props::PropKind,
    stone: Option<(CaveStoneFinish, u8)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CaveStoneFinish { Rock, Dark, Frozen }

fn cave_stone_finish(frame: &VisualWorldFrame) -> Option<CaveStoneFinish> {
    frame.tiles.iter().find_map(|tile| match tile.source.tileset_id.as_ref() {
        "ice_path" => Some(CaveStoneFinish::Frozen),
        "dark_cave" => Some(CaveStoneFinish::Dark),
        "cave" => Some(CaveStoneFinish::Rock),
        _ => None,
    })
}

#[derive(Resource, Default)]
pub(crate) struct ModeledActors {
    people_gpu: crate::open_people_skin::PeopleGpu,
    material: Option<Handle<StandardMaterial>>,
    remote_material: Option<Handle<StandardMaterial>>,
    shadow_mesh: Option<Handle<Mesh>>,
    shadow_material: Option<Handle<StandardMaterial>>,
    instances: HashMap<VisualActorId, Instance>,
    map_id: String,
    prop_meshes: HashMap<(actor_props::PropKind, Option<(CaveStoneFinish, u8)>), Handle<Mesh>>,
    stone_materials: HashMap<CaveStoneFinish, Handle<VoxelMaterial>>,
    props: HashMap<VisualActorId, PropInstance>,
}

fn enabled_map(map: &str) -> bool {
    // Actor assets are selected by the resolved presentation source. The map
    // name cannot change a nurse into a scientist or suppress an indoor rig.
    !map.is_empty()
}

/// The silhouette pass must use the same appearance test as the actual rig.
/// Unsupported mounts keep their original card and its matching silhouette.
pub(crate) fn has_modeled_player(frame: &VisualWorldFrame) -> bool {
    enabled_map(&frame.map_id)
        && frame.actors.iter().any(|actor| {
            actor.id == VisualActorId::Player
                && (character_kind(actor).is_some() || prop_kind(actor).is_some())
        })
}

// Every human source receives an explicit adaptation of the external rig.
// Character assets are required; there is no legacy model renderer.
fn person_name(kind: CharacterKind) -> &'static str {
    crate::npc_appearance::look(kind).base
}
fn person_height(kind: CharacterKind) -> f32 {
    crate::npc_appearance::look(kind).height
}

pub(crate) fn npc_texture_names() -> Vec<&'static str> {
    SOURCE_KINDS.iter().map(|&(name, _)| name).chain(std::iter::once("outdoorsman")).collect()
}

fn character_kind(actor: &VisualActor) -> Option<CharacterKind> {
    if matches!(actor.id, VisualActorId::Effect(_)) {
        return None;
    }
    character_meshes::kind_for_source(&actor.source_id)
}

pub(crate) fn authored_source(source: &str) -> Option<&'static str> {
    character_meshes::SOURCE_KINDS
        .iter()
        .find_map(|&(name, _)| (name == source).then_some(name))
        .or_else(|| actor_props::prop_kind_for_source(source).map(actor_props::PropKind::label))
}

fn prop_kind(actor: &VisualActor) -> Option<actor_props::PropKind> {
    if matches!(actor.id, VisualActorId::Effect(_)) {
        return None;
    }
    actor_props::prop_kind_for_source(&actor.source_id)
}

#[cfg(test)]
fn is_fruit_tree(actor: &VisualActor) -> bool {
    prop_kind(actor) == Some(actor_props::PropKind::FruitTree)
}

fn facing_yaw(facing: Vec2) -> f32 {
    facing.x.atan2(-facing.y)
}

fn model_transform(actor: &VisualActor, height: f32, yaw: f32, scale: f32) -> Transform {
    // The floor-centered root never bobs or slides: every shoe is solved relative
    // to the exact production foot anchor, independently of the camera.
    Transform::from_translation(visual_point_to_voxel(actor_foot(actor), height + 0.04))
        .with_rotation(Quat::from_rotation_y(yaw))
        .with_scale(Vec3::splat(scale))
}

fn prop_foot(actor: &VisualActor, kind: actor_props::PropKind) -> Vec2 {
    // The ball occupies the middle eight pixels of its sixteen-pixel sprite.
    // Its lower edge sits inside the gameplay tile, unlike a person's feet.
    if kind == actor_props::PropKind::PokeBall {
        actor.center - Vec2::new(0.0, actor.size.y * 0.25)
    } else {
        actor_foot(actor)
    }
}

fn world_foot(frame: &VisualWorldFrame, actor: &VisualActor) -> Vec2 {
    actor_foot(actor) - frame.center
        + Vec2::new(
            frame.grid_origin.x as f32 * frame.tile_size.x,
            -frame.grid_origin.y as f32 * frame.tile_size.y,
        )
}

fn contact_shadow() -> Mesh {
    let mut data = crate::mesh::SurfaceMeshData::default();
    // Soft opacity falloff is vertex-color geometry, independent of unsupported
    // driver shadow maps. No opaque black disk and no sprite artwork is involved.
    data.positions.push([0.0, 0.0, 0.0]);
    data.normals.push([0.0, 1.0, 0.0]);
    data.uvs.push([0.0, 0.0]);
    data.colors.push([0.05, 0.08, 0.09, 0.28]);
    for i in 0..32 {
        let a = i as f32 / 32.0 * TAU;
        data.positions.push([a.cos() * 0.32, 0.0, a.sin() * 0.26]);
        data.normals.push([0.0, 1.0, 0.0]);
        data.uvs.push([0.0, 0.0]);
        data.colors.push([0.05, 0.08, 0.09, 0.0]);
        data.indices.extend([0, (i + 1) % 32 + 1, i + 1]);
    }
    data.into_mesh()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync(
    mut commands: Commands,
    frame: Res<VisualWorldFrame>,
    status: Res<VoxelViewStatus>,
    footing: Res<TerrainRevisionCache>,
    cards: Res<ActorIdCache>,
    time: Res<Time>,
    mut state: ResMut<ModeledActors>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stone_materials: ResMut<Assets<VoxelMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
    mut card_visibility: Query<&mut Visibility, (With<VoxelActorCard>, Without<ModeledActor>)>,
    mut transforms: Query<&mut Transform, With<ModeledActor>>,
) {
    let enabled = frame.active && status.active && enabled_map(&frame.map_id);
    if !enabled || state.map_id != frame.map_id.as_ref() {
        for (_, instance) in state.instances.drain() {
            commands.entity(instance.entity).despawn();
        }
        for (_, instance) in state.props.drain() {
            commands.entity(instance.entity).despawn();
        }
        state.map_id = frame.map_id.to_string();
        if !enabled {
            return;
        }
    }
    if state.material.is_none() {
        state.material = Some(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.88,
            reflectance: 0.14,
            ..default()
        }));
        state.remote_material = Some(materials.add(StandardMaterial {
            base_color: Color::srgba(0.48, 0.88, 1.0, 0.62),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.88,
            reflectance: 0.14,
            ..default()
        }));
        state.shadow_mesh = Some(meshes.add(contact_shadow()));
        state.shadow_material = Some(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }));
    }
    let material = state.material.as_ref().unwrap().clone();
    let remote_material = state.remote_material.as_ref().unwrap().clone();
    let visible: HashSet<_> = frame
        .actors
        .iter()
        .filter(|actor| character_kind(actor).is_some())
        .map(|a| a.id)
        .collect();
    let stale: Vec<_> = state
        .instances
        .keys()
        .copied()
        .filter(|id| !visible.contains(id))
        .collect();
    for id in stale {
        if let Some(instance) = state.instances.remove(&id) {
            commands.entity(instance.entity).despawn();
        }
    }
    let visible_props: HashSet<_> = frame
        .actors
        .iter()
        .filter(|actor| prop_kind(actor).is_some())
        .map(|a| a.id)
        .collect();
    let stale_props: Vec<_> = state
        .props
        .keys()
        .copied()
        .filter(|id| !visible_props.contains(id))
        .collect();
    for id in stale_props {
        if let Some(instance) = state.props.remove(&id) {
            commands.entity(instance.entity).despawn();
        }
    }
    for actor in frame
        .actors
        .iter()
        .filter(|actor| prop_kind(actor).is_some())
    {
        let material = if matches!(actor.id, VisualActorId::RemotePlayer(_)) {
            remote_material.clone()
        } else {
            material.clone()
        };
        let kind = prop_kind(actor).expect("filtered resolved prop");
        let stone = if matches!(actor.id, VisualActorId::Object(_))
            && matches!(kind, actor_props::PropKind::Boulder | actor_props::PropKind::Rock) {
            cave_stone_finish(&frame).map(|finish| {
                let variant = match actor.id { VisualActorId::Object(id) => (id%4) as u8, _=>0 };
                (finish, variant)
            })
        } else { None };
        let cave_material = stone.map(|(finish,_)| {
            state.stone_materials.entry(finish).or_insert_with(|| {
                let texture = images.add(crate::painted_stone::movable_bedrock_image(finish==CaveStoneFinish::Frozen));
                // The rock mesh already contains authored face shading. Use
                // the same real-light energy response as the cave terrain,
                // rather than adding a second PBR shade that blacks out faces.
                stone_materials.add(crate::voxel_material(StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(texture),
                    perceptual_roughness: 0.94, reflectance: 0.10, ..default() }))
            }).clone()
        });
        let Some(height) =
            resolved_footing_height(&frame, prop_foot(actor, kind), &footing.footing_heights)
        else {
            if let Some(instance) = state.props.remove(&actor.id) {
                commands.entity(instance.entity).despawn();
            }
            continue;
        };
        let scale = frame.tile_size.y * 2.0;
        if !scale.is_finite() || scale <= 0.0 {
            continue;
        }
        let facing = actor
            .facing
            .filter(|f| f.is_finite() && f.length_squared() > 0.5)
            .unwrap_or(Vec2::NEG_Y);
        let prop_scale = if kind == actor_props::PropKind::PokeBall { scale * 0.5 } else { scale };
        let transform = Transform::from_translation(visual_point_to_voxel(prop_foot(actor, kind), height + 0.04))
            .with_rotation(Quat::from_rotation_y(facing_yaw(facing)))
            .with_scale(Vec3::splat(prop_scale));
        // A variable sprite/decor/species can change without a new object ID.
        // Replace its cached instance rather than leaving yesterday's mesh.
        if state
            .props
            .get(&actor.id)
            .is_some_and(|instance| instance.kind != kind || instance.stone != stone)
        {
            let old = state.props.remove(&actor.id).unwrap();
            commands.entity(old.entity).despawn();
        }
        if let Some(instance) = state.props.get(&actor.id) {
            if let Ok(mut current) = transforms.get_mut(instance.entity) {
                *current = transform;
            }
        } else {
            let mesh = state
                .prop_meshes
                .entry((kind,stone))
                .or_insert_with(|| meshes.add(if let Some((finish,variant))=stone {
                    let mut data=crate::mesh::movable_cave_stone(finish==CaveStoneFinish::Frozen,
                        finish==CaveStoneFinish::Dark, variant);
                    // Smashable rocks retain their smaller authored extent.
                    if kind==actor_props::PropKind::Rock {
                        for p in &mut data.positions {
                            *p=(Vec3::from_array(*p)*(0.43/0.64)).to_array();
                        }
                    }
                    data.into_mesh()
                } else { actor_props::mesh(kind).into_mesh() }))
                .clone();
            let entity = if let Some(material) = cave_material {
                commands.spawn(((Mesh3d(mesh),
MeshMaterial3d(material),
transform,), ModeledActor, RenderLayers::layer(VOXEL_RENDER_LAYER))).id()
            } else { commands
                .spawn((
                    (Mesh3d(mesh),
MeshMaterial3d(material.clone()),
transform,),
                    ModeledActor,
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ))
                .id() };
            state.props.insert(actor.id, PropInstance { entity, kind, stone });
        }
        if let Some(entity) = cards.entities.get(&actor.id)
            && let Ok(mut visibility) = card_visibility.get_mut(*entity)
        {
            *visibility = Visibility::Hidden;
        }
    }
    for actor in &frame.actors {
        let Some(kind) = character_kind(actor) else {
            continue;
        };
        // Human sprites never substitute for the required 3D character,
        // including a frame waiting for valid ground support.
        if let Some(entity) = cards.entities.get(&actor.id)
            && let Ok(mut visibility) = card_visibility.get_mut(*entity)
        {
            *visibility = Visibility::Hidden;
        }
        let Some(height) =
            resolved_footing_height(&frame, actor_foot(actor), &footing.footing_heights)
        else {
            // Wait for valid support; neither a stale rig nor an old sprite substitutes.
            if let Some(instance) = state.instances.remove(&actor.id) {
                commands.entity(instance.entity).despawn();
            }
            continue;
        };
        let world_foot = world_foot(&frame, actor);
        let scale = frame.tile_size.y * 2.0;
        if scale <= 0.0 || !scale.is_finite() || !world_foot.is_finite() {
            continue;
        }
        if state
            .instances
            .get(&actor.id)
            .is_some_and(|instance| instance.kind != kind)
        {
            let old = state.instances.remove(&actor.id).unwrap();
            commands.entity(old.entity).despawn();
        }
        if let Some(instance) = state.instances.get_mut(&actor.id) {
            instance
                .motion
                .advance(world_foot, actor.facing, scale, time.delta_secs());
            if let Ok(mut transform) = transforms.get_mut(instance.entity) {
                *transform = model_transform(actor, height, instance.motion.yaw, scale);
            }
            {
                let person = &mut instance.person;
                person.bicycle = actor.source_id.ends_with("_bike");
                if let Ok(mut transform) = transforms.get_mut(instance.entity) {
                    *transform = model_transform(
                        actor,
                        height,
                        instance.motion.yaw,
                        scale * person_height(kind) / person.rig.height,
                    );
                    transform.scale.x *= crate::npc_appearance::look(kind).width;
                }
                person.update(
                    instance.motion.phase,
                    instance.motion.walk_weight,
                    time.elapsed_secs() + instance.idle_offset,
                    &mut transforms,
                );
                if let Some(entity) = cards.entities.get(&actor.id)
                    && let Ok(mut visibility) = card_visibility.get_mut(*entity)
                {
                    *visibility = Visibility::Hidden;
                }
                continue;
            }
        } else {
            let name = person_name(kind);
            let imported = crate::open_people::get(name)
                .unwrap_or_else(|| panic!("required character rig {name} is unavailable; supply assets/open-models or CRYSTAL_OPEN_MODEL_ROOT"))
                .clone();
            {
                let facing = actor
                    .facing
                    .filter(|f| f.is_finite() && f.length_squared() > 0.5)
                    .unwrap_or(Vec2::NEG_Y);
                let motion = Motion::new(world_foot, facing);
                let entity = commands
                    .spawn((
                        (Visibility::default(),
model_transform(
                                actor,
                                height,
                                motion.yaw,
                                scale * person_height(kind) / imported.height,
                            ),),
                        ModeledActor,
                    ))
                    .id();
                commands.entity(entity).insert(model_transform(actor, height, motion.yaw,
                    scale * person_height(kind) / imported.height).with_scale(Vec3::new(
                        scale * person_height(kind) / imported.height * crate::npc_appearance::look(kind).width,
                        scale * person_height(kind) / imported.height,
                        scale * person_height(kind) / imported.height)));
                let mut person = state.people_gpu.spawn(
                    name,
                    kind,
                    imported,
                    entity,
                    matches!(actor.id, VisualActorId::RemotePlayer(_)),
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut images,
                    &mut binds,
                );
                person.bicycle = actor.source_id.ends_with("_bike");
                let shadow = commands
                    .spawn((
                        (Mesh3d(state.shadow_mesh.as_ref().unwrap().clone()),
MeshMaterial3d(state.shadow_material.as_ref().unwrap().clone()),
Transform::from_xyz(0.0, -0.001, 0.0)
                                .with_scale(Vec3::splat(person.rig.height / person_height(kind))),),
                        NotShadowCaster,
                        NotShadowReceiver,
                        RenderLayers::layer(VOXEL_RENDER_LAYER),
                    ))
                    .id();
                commands.entity(entity).add_child(shadow);
                state.instances.insert(
                    actor.id,
                    Instance {
                        entity,
                        kind,
                        motion,
                        idle_offset: (world_foot.x * 0.137 + world_foot.y * 0.241).rem_euclid(4.8),
                        person,
                    },
                );
                if let Some(entity) = cards.entities.get(&actor.id)
                    && let Ok(mut visibility) = card_visibility.get_mut(*entity)
                {
                    *visibility = Visibility::Hidden;
                }
                continue;
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn actor() -> VisualActor {
        VisualActor {
            id: VisualActorId::Player,
            source_id: "chris".into(),
            texture: Handle::default(),
            center: Vec2::new(20.0, 30.0),
            size: Vec2::splat(16.0),
            flip_x: false,
            above_priority: false,
            facing: Some(Vec2::X),
        }
    }
    fn actor_test_app() -> App {
        let mut app = App::new();
        let mut frame = VisualWorldFrame::default();
        frame.active = true;
        frame.map_id = "NewBarkTown".into();
        frame.tile_size = Vec2::splat(8.0);
        // A zero-size halo deliberately exercises the supported outside-grid
        // ordinary-ground footing fallback without needing gameplay content.
        frame.actors.push(actor());
        app.insert_resource(frame)
            .insert_resource(VoxelViewStatus {
                active: true,
                ..default()
            })
            .insert_resource(Time::<()>::default())
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<ActorIdCache>()
            .init_resource::<ModeledActors>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>()
            .add_systems(Update, sync);
        app
    }
    #[test]
    fn every_human_kind_has_an_explicit_external_appearance() {
        assert_eq!(KINDS.len(), 75);
        for kind in KINDS {
            assert!(matches!(person_name(kind), "human-male" | "human-female"));
            assert!(person_height(kind) > 0.8 && person_height(kind) < 1.6);
        }
    }
    #[test]
    #[ignore = "run separately with the locally supplied converted Kenney rigs"]
    fn external_people_cover_every_resolved_human_source_without_old_rigs() {
        use bevy::mesh::skinning::SkinnedMesh;
        let mut app = actor_test_app();
        for (index, &(source, expected_kind)) in SOURCE_KINDS.iter().chain([
            ("outdoorsman", CharacterKind::Outdoorsman),
            ("chris_bike", CharacterKind::Trainer),
            ("kris_bike", CharacterKind::TrainerFemale),
        ].iter()).enumerate() {
            app.world_mut().resource_mut::<VisualWorldFrame>().map_id =
                ["NewBarkTown", "TinTower5F", "PlayersHouse1F"][index % 3].into();
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = source.into();
            let frame = app.world().resource::<VisualWorldFrame>().clone();
            app.update();
            app.update(); // Exercise the live weighted pose, including mounted riders.
            let instance = &app.world().resource::<ModeledActors>().instances[&VisualActorId::Player];
            assert_eq!(instance.kind, expected_kind);
            assert!(!instance.person.rig.nodes.is_empty(), "missing weighted rig for {source}");
            assert_eq!(app.world().resource::<VisualWorldFrame>().actors[0].center, frame.actors[0].center);
            let joint_count = instance.person.rig.skin_nodes.len();
            let skins: Vec<_> = app.world_mut().query::<&SkinnedMesh>().iter(app.world())
                .map(|skin| skin.joints.clone()).collect();
            assert!(!skins.is_empty(), "no weighted body for {source}");
            // Costume overlays are also weighted; they must use exactly the
            // current actor skeleton rather than a second/stale human rig.
            for joints in &skins {
                assert_eq!(joints.len(), joint_count, "wrong skin for {source}");
                assert_eq!(joints, &skins[0], "stale costume skeleton for {source}");
                assert!(joints.iter().all(|&entity| app.world().get::<Transform>(entity).is_some()));
            }
        }
        assert_eq!(app.world().resource::<ModeledActors>().people_gpu.0.len(), SOURCE_KINDS.len()+1);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.clear();
        app.update();
        assert_eq!(app.world_mut().query_filtered::<Entity, With<ModeledActor>>().iter(app.world()).count(), 0);
    }
    #[test]
    #[ignore = "run separately with the locally supplied converted Kenney rigs"]
    fn external_people_reuse_gpu_skins_and_follow_only_the_production_actor() {
        use bevy::mesh::skinning::SkinnedMesh;
        assert!(
            crate::open_people::get("skater-male").is_some(),
            "supply CRYSTAL_OPEN_MODEL_ROOT"
        );
        let mut app = actor_test_app();
        app.update();
        let instance = &app.world().resource::<ModeledActors>().instances[&VisualActorId::Player];
        assert!(!instance.person.rig.nodes.is_empty());
        let root = instance.entity;
        let meshes = app.world().resource::<Assets<Mesh>>().len();
        let images = app.world().resource::<Assets<Image>>().len();
        let entities = app
            .world_mut()
            .query_filtered::<Entity, With<ModeledActor>>()
            .iter(app.world())
            .count();
        assert!(entities > 58);
        assert_eq!(
            app.world_mut()
                .query::<&SkinnedMesh>()
                .iter(app.world())
                .count(),
            1
        );
        for i in 0..30 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[0]
                .center
                .x += 0.2;
            let frame = app.world().resource::<VisualWorldFrame>().clone();
            app.update();
            assert_eq!(
                app.world().resource::<ModeledActors>().instances[&VisualActorId::Player].entity,
                root
            );
            let actor = &frame.actors[0];
            let expected = model_transform(
                actor,
                0.0,
                app.world().resource::<ModeledActors>().instances[&VisualActorId::Player]
                    .motion
                    .yaw,
                16.0 * 1.20 / crate::open_people::get("skater-male").unwrap().height,
            );
            assert!(
                app.world()
                    .get::<Transform>(root)
                    .unwrap()
                    .translation
                    .distance(expected.translation)
                    < 0.001
            );
            assert_eq!(
                app.world().resource::<VisualWorldFrame>().actors[0].center,
                actor.center
            );
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
            assert_eq!(app.world().resource::<Assets<Image>>().len(), images);
        }
        let mut remote = actor();
        remote.id = VisualActorId::RemotePlayer(23);
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .push(remote);
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
        assert_eq!(app.world().resource::<Assets<Image>>().len(), images);
        let handles: Vec<_> = app
            .world_mut()
            .query_filtered::<&MeshMaterial3d<StandardMaterial>, With<SkinnedMesh>>()
            .iter(app.world())
            .map(|material| material.0.clone())
            .collect();
        assert_eq!(handles.len(), 2);
        assert_ne!(handles[0], handles[1]);
        let root = app.world().resource::<ModeledActors>().instances[&VisualActorId::RemotePlayer(23)].entity;
        let mut descendants = vec![root];
        let mut remote_materials = Vec::new();
        while let Some(entity) = descendants.pop() {
            if let Some(children) = app.world().get::<Children>(entity) {
                descendants.extend(children.to_vec());
            }
            if let Some(handle) = app
                .world()
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .map(|material| &material.0)
            {
                let material = app.world().resource::<Assets<StandardMaterial>>().get(handle).unwrap();
                if !material.unlit {
                    assert!(app.world().get::<NotShadowReceiver>(entity).is_some(),
                        "painted bodies and attached costumes retain readable shadow contrast");
                    assert!(app.world().get::<NotShadowCaster>(entity).is_none(),
                        "people still cast grounding shadows on the world");
                }
                remote_materials.push(handle.clone());
            }
        }
        // The player's cap/hair must share the ghost treatment with the body.
        assert!(remote_materials.len() > 1);
        for handle in remote_materials {
            let material = app.world().resource::<Assets<StandardMaterial>>().get(&handle).unwrap();
            // The unlit contact shadow carries its opacity in vertex colors.
            if material.unlit { continue; }
            assert_eq!(material.alpha_mode, AlphaMode::Blend);
            assert!((material.base_color.alpha() - 0.62).abs() < 0.001);
        }
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .clear();
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&SkinnedMesh>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<ModeledActor>>()
                .iter(app.world())
                .count(),
            0
        );
    }
    #[test]
    fn actor_lifecycle_replaces_rigs_across_maps_appearances_and_toggles() {
        let mut app = actor_test_app();
        app.update();
        let root = app.world().resource::<ModeledActors>().instances[&VisualActorId::Player].entity;
        assert_eq!(app.world().resource::<ModeledActors>().instances.len(), 1);
        let mut descendants = vec![root];
        let mut owned = std::collections::HashSet::new();
        while let Some(entity) = descendants.pop() {
            if let Some(children) = app.world().get::<Children>(entity) {
                descendants.extend(children.to_vec());
            }
            if app.world().get::<ModeledActor>(entity).is_some() { owned.insert(entity); }
        }
        let modeled: std::collections::HashSet<_> = app.world_mut()
            .query_filtered::<Entity, With<ModeledActor>>().iter(app.world()).collect();
        assert_eq!(modeled, owned, "every modeled node belongs to the current actor");
        let instance = &app.world().resource::<ModeledActors>().instances[&VisualActorId::Player];
        assert!(modeled.len() >= instance.person.rig.nodes.len() + 2);
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "VioletCity".into();
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert_eq!(app.world().resource::<ModeledActors>().instances.len(), 1);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = "kris".into();
        app.update();
        assert_eq!(
            app.world().resource::<ModeledActors>().instances[&VisualActorId::Player].kind,
            CharacterKind::TrainerFemale
        );
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = "kris_bike".into();
        app.update();
        assert_eq!(app.world().resource::<ModeledActors>().instances[&VisualActorId::Player].kind, CharacterKind::TrainerFemale);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = "chris".into();
        app.update();
        app.world_mut().resource_mut::<VoxelViewStatus>().active = false;
        app.update();
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<ModeledActor>>()
                .iter(app.world())
                .count(),
            0
        );
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
        app.update();
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "PlayersHouse1F".into();
        app.update();
        assert_eq!(
            app.world().resource::<ModeledActors>().instances.len(),
            1,
            "resolved actors remain modeled indoors"
        );
    }
    #[test]
    fn duplicate_characters_share_meshes_and_leave_production_frame_untouched() {
        let mut app = actor_test_app();
        app.update();
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        let mut other = actor();
        other.id = VisualActorId::RemotePlayer(23);
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .push(other);
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(app.world().resource::<ModeledActors>().instances.len(), 2);
        let state = app.world().resource::<ModeledActors>();
        let materials = |root: Entity| {
            let mut pending = vec![root];
            let mut handles = std::collections::HashSet::new();
            while let Some(entity) = pending.pop() {
                if let Some(children) = app.world().get::<Children>(entity) {
                    pending.extend(children.to_vec());
                }
            if let Some(handle) = app
                .world()
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .map(|material| &material.0)
            {
                    if !app.world().resource::<Assets<StandardMaterial>>().get(handle).unwrap().unlit {
                        handles.insert(handle.clone());
                    }
                }
            }
            handles
        };
        let local = materials(state.instances[&VisualActorId::Player].entity);
        let remote = materials(state.instances[&VisualActorId::RemotePlayer(23)].entity);
        assert!(!local.is_empty() && !remote.is_empty());
        assert!(local.is_disjoint(&remote), "remote bodies and accessories use distinct ghost materials");
        for handle in remote {
            let material = app.world().resource::<Assets<StandardMaterial>>().get(&handle).unwrap();
            assert_eq!(material.alpha_mode, AlphaMode::Blend);
            assert!((material.base_color.alpha() - 0.62).abs() < 0.001);
        }
        let frame = app.world().resource::<VisualWorldFrame>();
        assert_eq!(frame.actors[0].center, actor().center);
        assert_eq!(frame.actors[0].facing, actor().facing);
        assert!(has_modeled_player(frame));
    }
    #[test]
    fn fruit_tree_prop_tracks_only_verified_current_source_and_visibility() {
        let mut app = actor_test_app();
        let mut tree = actor();
        tree.id = VisualActorId::Object(7);
        tree.source_id = "fruit_tree".into();
        assert!(is_fruit_tree(&tree));
        assert_eq!(character_kind(&tree), None);
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .push(tree);
        app.update();
        let root = app.world().resource::<ModeledActors>().props[&VisualActorId::Object(7)].entity;
        let meshes = app.world().resource::<Assets<Mesh>>().len();
        let mut other = app.world().resource::<VisualWorldFrame>().actors[1].clone();
        other.id = VisualActorId::Object(8);
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .push(other);
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id =
            "fruit_tree_picked".into();
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert!(
            !app.world()
                .resource::<ModeledActors>()
                .props
                .contains_key(&VisualActorId::Object(7))
        );
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .truncate(1);
        app.update();
        assert!(app.world().resource::<ModeledActors>().props.is_empty());
    }
    #[test]
    fn modeled_facing_and_foot_remain_world_anchored() {
        let actor = actor();
        let transform = model_transform(&actor, 2.0, facing_yaw(Vec2::X), 16.0);
        assert_eq!(transform.translation, Vec3::new(20.0, 2.04, -22.0));
        assert!((transform.rotation * Vec3::Z - Vec3::X).length() < 0.0001);
    }
    #[test]
    fn item_ball_uses_its_art_bottom_and_smaller_grounded_scale() {
        let mut app = actor_test_app();
        let mut ball = actor();
        ball.id = VisualActorId::Object(19);
        ball.source_id = "poke_ball".into();
        let expected_foot = ball.center - Vec2::Y * ball.size.y * 0.25;
        let frame = app.world().resource::<VisualWorldFrame>().clone();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.push(ball.clone());
        app.update();
        let entity = app.world().resource::<ModeledActors>().props[&ball.id].entity;
        let transform = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(transform.translation.x, expected_foot.x);
        assert_eq!(transform.translation.z, -expected_foot.y);
        assert_eq!(transform.scale, Vec3::splat(frame.tile_size.y));
        assert_eq!(app.world().resource::<VisualWorldFrame>().actors[1].center, ball.center);
    }
    #[test]
    fn variable_prop_source_replaces_same_id_and_reuses_per_kind_mesh() {
        let mut app = actor_test_app();
        let mut prop = actor();
        prop.id = VisualActorId::Object(19);
        prop.source_id = "poke_ball".into();
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .push(prop);
        app.update();
        let old = app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id = "boulder".into();
        app.update();
        let state = app.world().resource::<ModeledActors>();
        assert_ne!(state.props[&VisualActorId::Object(19)].entity, old);
        assert_eq!(
            state.props[&VisualActorId::Object(19)].kind,
            actor_props::PropKind::Boulder
        );
        assert!(app.world().get_entity(old).is_err());
        let count = app.world().resource::<Assets<Mesh>>().len();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id = "poke_ball".into();
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), count);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id =
            "unknown_decoration".into();
        app.update();
        assert!(app.world().resource::<ModeledActors>().props.is_empty());
    }
    #[test]
    fn cave_boulder_material_and_mesh_follow_environment_while_movement_reuses_the_object() {
        use crystal_render_api::{VisualTile, VisualTileSource};
        let mut app=actor_test_app();
        let mut boulder=actor();
        boulder.id=VisualActorId::Object(19);
        boulder.source_id="boulder".into();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.push(boulder);
        app.world_mut().resource_mut::<VisualWorldFrame>().tiles.push(VisualTile {
            column:0,row:0,source:VisualTileSource {tileset_id:"ice_path".into(),
                metatile_id:0,subtile_column:0,subtile_row:0,tile_index:0x19},
            texture:Handle::default(),animation_frames:None,priority:false,
        });
        app.update();
        let first=app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].kind,actor_props::PropKind::Boulder);
        assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].stone,Some((CaveStoneFinish::Frozen,3)));
        let first_position=app.world().get::<Transform>(first).unwrap().translation;
        let count=app.world().resource::<Assets<Mesh>>().len();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].center.x+=8.;
        app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity,first);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(),count);
        let second_position=app.world().get::<Transform>(first).unwrap().translation;
        assert_eq!(second_position.x-first_position.x,8.);
        assert_eq!(second_position.y,first_position.y);
        let handle=&app.world().get::<MeshMaterial3d<VoxelMaterial>>(first).unwrap().0;
        assert!(app.world().resource::<Assets<VoxelMaterial>>().get(handle).unwrap().base.base_color_texture.is_some());
        app.world_mut().resource_mut::<VisualWorldFrame>().tiles[0].source.tileset_id="cave".into();
        app.update();
        let state=app.world().resource::<ModeledActors>();
        let next=&state.props[&VisualActorId::Object(19)];
        assert_eq!(next.stone,Some((CaveStoneFinish::Rock,3)));
        assert_ne!(next.entity,first);
        assert_eq!(app.world().get::<Transform>(next.entity).unwrap().translation,second_position);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id="rock".into();
        app.update();
        let rock=app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        let handle=&app.world().get::<Mesh3d>(rock).unwrap().0;
        let mesh=app.world().resource::<Assets<Mesh>>().get(handle).unwrap();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions))=mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else { panic!("rock positions"); };
        assert!((positions.iter().map(|p|p[1]).fold(0.,f32::max)-0.43).abs()<0.00001);
        // Leaving the cave restores the ordinary prop material and retains
        // its authoritative position, rather than leaking cave paint outdoors.
        app.world_mut().resource_mut::<VisualWorldFrame>().tiles[0].source.tileset_id="johto".into();
        app.update();
        let outside=app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        assert!(app.world().get::<MeshMaterial3d<StandardMaterial>>(outside).is_some());
        assert!(app.world().get::<MeshMaterial3d<VoxelMaterial>>(outside).is_none());
        assert_eq!(app.world().get::<Transform>(outside).unwrap().translation,second_position);
    }
    #[test]
    fn dynamic_species_decorations_replace_reuse_and_disappear_without_stale_entities() {
        let mut app = actor_test_app();
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "PlayersHouse2F".into();
        let mut doll = actor();
        doll.id = VisualActorId::Object(19);
        doll.source_id = "species:MAGIKARP:icon_fish".into();
        let foot = actor_foot(&doll);
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .push(doll);
        app.update();
        let old = app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        let transform = *app.world().get::<Transform>(old).unwrap();
        for source in [
            "species:GENGAR:icon_ghost",
            "species:GRIMER:icon_blob",
            "species:MAGIKARP:icon_fish",
        ] {
            let previous =
                app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id = source.into();
            app.update();
            let root =
                app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
            assert_ne!(root, previous);
            assert!(app.world().get_entity(previous).is_err());
            assert_eq!(*app.world().get::<Transform>(root).unwrap(), transform);
            assert_eq!(
                actor_foot(&app.world().resource::<VisualWorldFrame>().actors[1]),
                foot
            );
        }
        let count = app.world().resource::<Assets<Mesh>>().len();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id =
            "species:GENGAR:icon_ghost".into();
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), count);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id =
            "species:UNKNOWN:icon_fish".into();
        app.update();
        assert!(app.world().resource::<ModeledActors>().props.is_empty());
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].source_id =
            "species:MAGIKARP:icon_fish".into();
        app.update();
        let root = app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        app.world_mut()
            .resource_mut::<VisualWorldFrame>()
            .actors
            .truncate(1);
        app.update();
        assert!(app.world().resource::<ModeledActors>().props.is_empty());
        assert!(app.world().get_entity(root).is_err());
    }
    #[test]
    fn character_appearance_is_source_exact_across_indoor_and_outdoor_maps() {
        for map in [
            "NewBarkTown",
            "Route29",
            "CherrygroveCity",
            "Route30",
            "Route31",
            "VioletCity",
        ] {
            assert!(enabled_map(map));
        }
        assert!(enabled_map("PlayersHouse1F"));
        assert!(!enabled_map(""));
        let mut actor = actor();
        actor.source_id = "kris".into();
        assert_eq!(character_kind(&actor), Some(CharacterKind::TrainerFemale));
        for source in ["surf", "surf_pikachu"] {
            actor.source_id = source.into();
            assert_eq!(character_kind(&actor), None);
        }
    }
    #[test]
    fn distance_drives_gait_independent_of_frame_rate_and_pause() {
        let mut a = Motion::new(Vec2::ZERO, Vec2::X);
        let mut b = a;
        for i in 1..=16 {
            a.advance(Vec2::new(i as f32, 0.0), Some(Vec2::X), 16.0, 1.0 / 24.0);
        }
        for i in 1..=8 {
            b.advance(
                Vec2::new(i as f32 * 2.0, 0.0),
                Some(Vec2::X),
                16.0,
                1.0 / 12.0,
            );
        }
        assert!((a.phase - b.phase).abs() < 0.00001);
        let phase = a.phase;
        a.advance(a.last_world_foot, Some(Vec2::X), 16.0, 1.0 / 60.0);
        assert_eq!(a.phase, phase);
        assert!(
            a.walk_weight > 0.9,
            "a missing host update must not reset the walk"
        );
        for _ in 0..60 {
            a.advance(a.last_world_foot, Some(Vec2::X), 16.0, 1.0 / 60.0);
        }
        assert_eq!(a.phase, phase);
        assert_eq!(a.walk_weight, 0.0);
    }
    #[test]
    fn map_camera_changes_do_not_make_characters_walk() {
        let mut frame = VisualWorldFrame::default();
        frame.tile_size = Vec2::splat(8.0);
        frame.center = Vec2::new(50.0, 40.0);
        frame.grid_origin = IVec2::new(3, 7);
        let mut actor = actor();
        let old = world_foot(&frame, &actor);
        // Grid origin advances one source tile; camera-relative actor scrolls
        // by the matching amount. Their map-relative foot is unchanged.
        frame.grid_origin += IVec2::new(1, 2);
        actor.center += Vec2::new(-8.0, 16.0);
        assert_eq!(world_foot(&frame, &actor), old);
        frame.center += Vec2::new(11.0, -9.0);
        actor.center += Vec2::new(11.0, -9.0);
        assert_eq!(world_foot(&frame, &actor), old);
    }
    #[test]
    fn gait_does_not_pump_to_idle_between_sparse_published_positions() {
        for (render_hz, publish_every) in [(60, 6), (30, 3), (9, 2)] {
            let dt = 1.0 / render_hz as f32;
            let mut motion = Motion::new(Vec2::ZERO, Vec2::X);
            let mut published = Vec2::ZERO;
            for frame in 1..=render_hz * 4 {
                if frame % publish_every == 0 {
                    published.x = frame as f32 * dt * 1.5 * 16.0;
                }
                motion.advance(published, Some(Vec2::X), 16.0, dt);
                if frame > render_hz {
                    assert!(
                        motion.walk_weight > 0.98,
                        "{render_hz}Hz frame {frame}: publication gaps must not imitate stops: {}",
                        motion.walk_weight
                    );
                }
            }
            let phase = motion.phase;
            for _ in 0..render_hz * 2 {
                motion.advance(published, Some(Vec2::X), 16.0, dt);
            }
            assert_eq!(motion.walk_weight, 0.0);
            assert_eq!(motion.phase, phase, "stopping must not invent a step");
        }
    }
    #[test]
    fn turning_is_rate_bounded_at_normal_and_slow_host_cadences() {
        for hz in [9, 30, 60] {
            let dt = 1.0 / hz as f32;
            let mut motion = Motion::new(Vec2::ZERO, Vec2::NEG_Y);
            for _ in 0..hz {
                let old = motion.yaw;
                motion.advance(Vec2::ZERO, Some(Vec2::X), 16.0, dt);
                assert!((motion.yaw - old).abs() <= 6.0 * dt + 0.00001);
            }
            assert!((motion.yaw - PI / 2.0).abs() < 0.001);
        }
    }
    #[test]
    fn teleports_reset_gait_without_inventing_motion() {
        let mut motion = Motion::new(Vec2::ZERO, Vec2::NEG_Y);
        motion.advance(Vec2::X, Some(Vec2::X), 16.0, 0.02);
        motion.advance(Vec2::new(500.0, 600.0), Some(Vec2::Y), 16.0, 0.02);
        assert_eq!(motion.phase, 0.0);
        assert_eq!(motion.walk_weight, 0.0);
        assert!((Quat::from_rotation_y(motion.yaw) * Vec3::Z - Vec3::NEG_Z).length() < 0.0001);
    }
    #[test]
    fn speed_changes_blend_gait_without_resetting_phase_or_root_anchor() {
        let dt = 1.0 / 60.0;
        let mut state = Motion::new(Vec2::ZERO, Vec2::X);
        let mut position = Vec2::ZERO;
        for frame in 0..360 {
            let speed = if frame < 60 {
                1.5
            } else if frame < 240 {
                7.466
            } else {
                1.5
            };
            position.x += speed * dt * 16.0;
            let old_phase = state.phase;
            let old_run = state.run_weight;
            state.advance(position, Some(Vec2::X), 16.0, dt);
            let step = (state.phase - old_phase).rem_euclid(TAU);
            assert!(step > 0.0 && step < 0.71, "phase jumped by {step}");
            assert!((state.run_weight - old_run).abs() < 0.13);
            assert_eq!(state.last_world_foot, position);
            if frame == 230 {
                assert!(state.run_weight > 0.999);
            }
        }
        assert!(state.run_weight < 0.01);
    }

}
