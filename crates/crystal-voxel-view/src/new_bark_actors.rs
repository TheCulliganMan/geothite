//! Articulated characters consume the authoritative presentation frame only.
//! No input, collision, script, inventory, facing, or warp state is owned here.
#[path = "johto_actor_props.rs"]
pub(crate) mod actor_props;
#[path = "johto_characters.rs"]
mod character_meshes;

use crate::{
    ActorIdCache, TerrainRevisionCache, VOXEL_RENDER_LAYER, VoxelActorCard, VoxelViewStatus,
    actor_foot, resolved_footing_height, visual_point_to_voxel,
};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use character_meshes::*;
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame};
use std::collections::{HashMap, HashSet};
use std::f32::consts::{PI, TAU};

#[derive(Component)]
pub(crate) struct ModeledActor;

/// One ordinary walking cycle travels this many gameplay tiles. The stance foot
/// moves backward at exactly the root's speed; there is no timer-driven skating.
const CYCLE_DISTANCE: f32 = 1.12;
const STRIDE_REACH: f32 = CYCLE_DISTANCE * 0.25;
const RUN_CYCLE_DISTANCE: f32 = 2.80;
const RUN_STRIDE_REACH: f32 = 0.20;

fn cycle_distance(run: f32) -> f32 {
    CYCLE_DISTANCE + (RUN_CYCLE_DISTANCE - CYCLE_DISTANCE) * run
}
fn stance_fraction(run: f32) -> f32 {
    let reach = STRIDE_REACH + (RUN_STRIDE_REACH - STRIDE_REACH) * run;
    2.0 * reach / cycle_distance(run)
}
const LEG_LENGTH: f32 = 0.27;
const ANKLE_HEIGHT: f32 = 0.12;

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
    joints: [Entity; JOINT_COUNT],
    kind: CharacterKind,
    motion: Motion,
    idle_offset: f32,
}

struct PropInstance {
    entity: Entity,
    kind: actor_props::PropKind,
}

#[derive(Resource, Default)]
pub(crate) struct ModeledActors {
    meshes: HashMap<CharacterKind, [Handle<Mesh>; JOINT_COUNT]>,
    material: Option<Handle<StandardMaterial>>,
    remote_material: Option<Handle<StandardMaterial>>,
    shadow_mesh: Option<Handle<Mesh>>,
    shadow_material: Option<Handle<StandardMaterial>>,
    instances: HashMap<VisualActorId, Instance>,
    map_id: String,
    prop_meshes: HashMap<actor_props::PropKind, Handle<Mesh>>,
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

fn world_foot(frame: &VisualWorldFrame, actor: &VisualActor) -> Vec2 {
    actor_foot(actor) - frame.center
        + Vec2::new(
            frame.grid_origin.x as f32 * frame.tile_size.x,
            -frame.grid_origin.y as f32 * frame.tile_size.y,
        )
}

/// Sagittal ankle target: a linear planted half-cycle, then a smooth lifted
/// return. Endpoints are continuous, the grounded half has zero vertical motion.
fn foot_target(phase: f32, weight: f32, run: f32) -> (f32, f32) {
    let t = phase.rem_euclid(TAU) / TAU;
    let stance = stance_fraction(run);
    let reach = STRIDE_REACH + (RUN_STRIDE_REACH - STRIDE_REACH) * run;
    if t < stance {
        // 2*reach == cycle_distance*stance: planted foot speed exactly
        // cancels root translation for walking and the shorter running stance.
        (reach * (1.0 - 2.0 * t / stance) * weight, ANKLE_HEIGHT)
    } else {
        let swing = (t - stance) / (1.0 - stance);
        let tangent = -2.0 * (1.0 - stance) / stance;
        let return_z = -1.0
            + tangent * swing
            + (6.0 - 3.0 * tangent) * swing * swing
            + (2.0 * tangent - 4.0) * swing.powi(3);
        let lift = (PI * swing).sin().powi(2);
        // Fast heel recovery is still C1 continuous at contact. It keeps the
        // shorter sprint stance within anatomical reach without scaling legs.
        let walk_lift = 0.060 * lift;
        let run_lift = 0.170 * 1.04 * lift / (0.04 + lift);
        (
            reach * return_z * weight,
            ANKLE_HEIGHT + (walk_lift + (run_lift - walk_lift) * run) * weight,
        )
    }
}

/// Two real rotational joints solve each ankle. The shoe counter-rotates so
/// soles stay level during stance; knees bend forward rather than backwards.
fn leg_angles(hip_height: f32, ankle_z: f32, ankle_y: f32) -> (f32, f32) {
    let down = hip_height - ankle_y;
    let distance = down.hypot(ankle_z).clamp(0.001, LEG_LENGTH * 2.0 - 0.0001);
    let knee = PI
        - ((2.0 * LEG_LENGTH * LEG_LENGTH - distance * distance) / (2.0 * LEG_LENGTH * LEG_LENGTH))
            .clamp(-1.0, 1.0)
            .acos();
    let hip = (-ankle_z).atan2(down) - (distance / (2.0 * LEG_LENGTH)).clamp(-1.0, 1.0).acos();
    (hip, knee)
}

fn pose(kind: CharacterKind, motion: &Motion, elapsed: f32) -> [Transform; JOINT_COUNT] {
    let mut result = rig(kind).joints.each_ref().map(|joint| joint.bind);
    let w = motion.walk_weight;
    let phase = motion.phase;
    let run = motion.run_weight;
    // Lowering by only a few centimeters gives the planted legs adequate reach.
    // This moves the hips and upper body, never the floor anchor or planted shoe.
    // Squared cosine removes the acceleration cusp from abs(cos),
    // while the extra reach allows a less frantic 1.12-tile cycle.
    let walk_drop = 0.065 + phase.cos().powi(2) * 0.030;
    let run_drop = 0.068 + phase.cos().powi(2) * 0.025;
    let walking_drop = walk_drop + (run_drop - walk_drop) * run;
    // Ease the crouch into motion quadratically; a linear weight made
    // the whole body drop abruptly on the first walking sample.
    let hip_offset = -0.012 - (walking_drop - 0.012) * w * w;
    result[PELVIS].translation.y += hip_offset;
    let breath = (elapsed * 2.1).sin() * 0.0025 * (1.0 - w * 0.65);
    result[TORSO].translation.y += breath;
    result[TORSO].rotation = Quat::from_euler(
        EulerRot::XYZ,
        (0.065 + 0.12 * run) * w,
        phase.cos() * 0.045 * w,
        phase.sin() * 0.025 * w,
    );
    result[HEAD].rotation = Quat::from_euler(
        EulerRot::XYZ,
        -(0.038 + 0.07 * run) * w + (elapsed * 1.4).sin() * 0.008 * (1.0 - w),
        -phase.cos() * 0.030 * w,
        -phase.sin() * 0.018 * w,
    );
    let blink_t = elapsed.rem_euclid(4.8);
    let blink = if blink_t < 0.145 {
        1.0 - (blink_t / 0.145 * PI).sin().powi(2) * 0.94
    } else {
        1.0
    };
    result[EYES].scale.y = blink;
    for (side, upper, forearm, hand, thigh, shin, shoe) in [
        (
            -1.0,
            UPPER_ARM_L,
            FOREARM_L,
            HAND_L,
            THIGH_L,
            SHIN_L,
            SHOE_L,
        ),
        (1.0, UPPER_ARM_R, FOREARM_R, HAND_R, THIGH_R, SHIN_R, SHOE_R),
    ] {
        let leg_phase = phase + if side < 0.0 { 0.0 } else { PI };
        let (z, y) = foot_target(leg_phase, w, run);
        let (hip, knee) = leg_angles(0.66 + hip_offset, z, y);
        result[thigh].rotation = Quat::from_rotation_x(hip);
        result[shin].rotation = Quat::from_rotation_x(knee);
        result[shoe].rotation = Quat::from_rotation_x(-hip - knee);
        // Opposing shoulders and elbows give a readable natural walk, including
        // independent mitten hands. All pivots are real hierarchy transforms.
        let swing = leg_phase.cos() * (0.48 + 0.12 * run) * w;
        result[upper].rotation = Quat::from_euler(
            EulerRot::XYZ,
            swing - 0.055 + breath * 2.0,
            0.0,
            side * 0.035,
        );
        result[forearm].rotation =
            Quat::from_rotation_x(-0.13 - 0.65 * run * w - swing.min(0.0).abs() * 0.28);
        result[hand].rotation = Quat::from_rotation_x(0.045 + swing * 0.08);
    }
    if matches!(kind, CharacterKind::SwimmerGirl | CharacterKind::SwimmerGuy) {
        // The source is a swimmer in the water, not a walking person balanced
        // on its surface. Lower only the visual body; the production foot/root
        // remains fixed and no physics or movement mode is inferred here.
        result[PELVIS].translation.y -= 0.48;
        result[TORSO].rotation = Quat::from_rotation_x(0.24);
        for (side, upper, forearm, thigh, shin) in [
            (-1.0, UPPER_ARM_L, FOREARM_L, THIGH_L, SHIN_L),
            (1.0, UPPER_ARM_R, FOREARM_R, THIGH_R, SHIN_R),
        ] {
            let stroke = elapsed * 1.7 + side * PI * 0.5;
            result[upper].rotation = Quat::from_euler(
                EulerRot::XYZ,
                -0.64 + stroke.sin() * (0.12 + w * 0.16),
                0.0,
                side * 0.48,
            );
            result[forearm].rotation = Quat::from_rotation_x(-0.42);
            result[thigh].rotation = Quat::from_rotation_x(0.24 + stroke.sin() * 0.10);
            result[shin].rotation = Quat::from_rotation_x(0.25);
        }
    }
    result
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
    mut card_visibility: Query<&mut Visibility, (With<VoxelActorCard>, Without<ModeledActor>)>,
    mut transforms: Query<&mut Transform, With<ModeledActor>>,
) {
    let enabled = frame.active && status.active && enabled_map(&frame.map_id);
    if !enabled || state.map_id != frame.map_id.as_ref() {
        for (_, instance) in state.instances.drain() {
            commands.entity(instance.entity).despawn_recursive();
        }
        for (_, instance) in state.props.drain() {
            commands.entity(instance.entity).despawn_recursive();
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
            commands.entity(instance.entity).despawn_recursive();
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
            commands.entity(instance.entity).despawn_recursive();
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
        let Some(height) =
            resolved_footing_height(&frame, actor_foot(actor), &footing.footing_heights)
        else {
            if let Some(instance) = state.props.remove(&actor.id) {
                commands.entity(instance.entity).despawn_recursive();
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
        let transform = model_transform(actor, height, facing_yaw(facing), scale);
        // A variable sprite/decor/species can change without a new object ID.
        // Replace its cached instance rather than leaving yesterday's mesh.
        if state
            .props
            .get(&actor.id)
            .is_some_and(|instance| instance.kind != kind)
        {
            let old = state.props.remove(&actor.id).unwrap();
            commands.entity(old.entity).despawn_recursive();
        }
        if let Some(instance) = state.props.get(&actor.id) {
            if let Ok(mut current) = transforms.get_mut(instance.entity) {
                *current = transform;
            }
        } else {
            let mesh = state
                .prop_meshes
                .entry(kind)
                .or_insert_with(|| meshes.add(actor_props::mesh(kind).into_mesh()))
                .clone();
            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh,
                        material: material.clone(),
                        transform,
                        ..default()
                    },
                    ModeledActor,
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ))
                .id();
            state.props.insert(actor.id, PropInstance { entity, kind });
        }
        if let Some(entity) = cards.entities.get(&actor.id)
            && let Ok(mut visibility) = card_visibility.get_mut(*entity)
        {
            *visibility = Visibility::Hidden;
        }
    }
    for actor in &frame.actors {
        let material = if matches!(actor.id, VisualActorId::RemotePlayer(_)) {
            remote_material.clone()
        } else {
            material.clone()
        };
        let Some(kind) = character_kind(actor) else {
            continue;
        };
        let Some(height) =
            resolved_footing_height(&frame, actor_foot(actor), &footing.footing_heights)
        else {
            // No sampled support: keep the normal card rather than a stale rig.
            if let Some(instance) = state.instances.remove(&actor.id) {
                commands.entity(instance.entity).despawn_recursive();
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
            commands.entity(old.entity).despawn_recursive();
        }
        if let Some(instance) = state.instances.get_mut(&actor.id) {
            instance
                .motion
                .advance(world_foot, actor.facing, scale, time.delta_seconds());
            if let Ok(mut transform) = transforms.get_mut(instance.entity) {
                *transform = model_transform(actor, height, instance.motion.yaw, scale);
            }
            let joints = pose(
                kind,
                &instance.motion,
                time.elapsed_seconds() + instance.idle_offset,
            );
            for (&entity, value) in instance.joints.iter().zip(joints) {
                if let Ok(mut transform) = transforms.get_mut(entity) {
                    *transform = value;
                }
            }
        } else {
            let model = rig(kind);
            let mesh_handles = state
                .meshes
                .entry(kind)
                .or_insert_with(|| {
                    model
                        .joints
                        .each_ref()
                        .map(|joint| meshes.add(joint.mesh.clone().into_mesh()))
                })
                .clone();
            let facing = actor
                .facing
                .filter(|f| f.is_finite() && f.length_squared() > 0.5)
                .unwrap_or(Vec2::NEG_Y);
            let motion = Motion::new(world_foot, facing);
            let idle_offset = (world_foot.x * 0.137 + world_foot.y * 0.241).rem_euclid(4.8);
            let entity = commands
                .spawn((
                    SpatialBundle {
                        transform: model_transform(actor, height, motion.yaw, scale),
                        ..default()
                    },
                    ModeledActor,
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ))
                .id();
            let joint_transforms = pose(kind, &motion, time.elapsed_seconds() + idle_offset);
            let mut joints = [Entity::PLACEHOLDER; JOINT_COUNT];
            for index in 0..JOINT_COUNT {
                let joint = commands
                    .spawn((
                        PbrBundle {
                            mesh: mesh_handles[index].clone(),
                            material: material.clone(),
                            transform: joint_transforms[index],
                            ..default()
                        },
                        ModeledActor,
                        RenderLayers::layer(VOXEL_RENDER_LAYER),
                    ))
                    .id();
                commands
                    .entity(
                        model.joints[index]
                            .parent
                            .map(|i| joints[i])
                            .unwrap_or(entity),
                    )
                    .add_child(joint);
                joints[index] = joint;
            }
            let shadow = commands
                .spawn((
                    PbrBundle {
                        mesh: state.shadow_mesh.as_ref().unwrap().clone(),
                        material: state.shadow_material.as_ref().unwrap().clone(),
                        transform: Transform::from_xyz(0.0, -0.001, 0.0),
                        ..default()
                    },
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
                    joints,
                    kind,
                    motion,
                    idle_offset,
                },
            );
        }
        if let Some(entity) = cards.entities.get(&actor.id)
            && let Ok(mut visibility) = card_visibility.get_mut(*entity)
        {
            *visibility = Visibility::Hidden;
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
            .add_systems(Update, sync);
        app
    }
    #[test]
    fn actor_lifecycle_replaces_rigs_across_maps_appearances_and_toggles() {
        let mut app = actor_test_app();
        app.update();
        let root = app.world().resource::<ModeledActors>().instances[&VisualActorId::Player].entity;
        assert_eq!(app.world().resource::<ModeledActors>().instances.len(), 1);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<ModeledActor>>()
                .iter(app.world())
                .count(),
            JOINT_COUNT + 1
        );
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "VioletCity".into();
        app.update();
        assert!(app.world().get_entity(root).is_none());
        assert_eq!(app.world().resource::<ModeledActors>().instances.len(), 1);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = "kris".into();
        app.update();
        assert_eq!(
            app.world().resource::<ModeledActors>().instances[&VisualActorId::Player].kind,
            CharacterKind::TrainerFemale
        );
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = "kris_bike".into();
        app.update();
        assert!(app.world().resource::<ModeledActors>().instances.is_empty());
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
        let local_head = state.instances[&VisualActorId::Player].joints[HEAD];
        let remote_head = state.instances[&VisualActorId::RemotePlayer(23)].joints[HEAD];
        assert_ne!(
            app.world().get::<Handle<StandardMaterial>>(local_head),
            app.world().get::<Handle<StandardMaterial>>(remote_head),
            "modeled remote players retain the existing distinct ghost material"
        );
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
        assert!(app.world().get_entity(root).is_none());
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
        assert!(app.world().get_entity(old).is_none());
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
    fn swimming_look_has_a_waterline_pose_without_moving_the_world_root() {
        let motion = Motion::new(Vec2::ZERO, Vec2::NEG_Y);
        let standing = pose(CharacterKind::Trainer, &motion, 1.0);
        let swimming = pose(CharacterKind::SwimmerGuy, &motion, 1.0);
        assert!(
            (standing[PELVIS].translation.y - swimming[PELVIS].translation.y - 0.48).abs() < 0.0001
        );
        assert!(
            swimming
                .iter()
                .all(|joint| joint.translation.is_finite() && joint.rotation.is_finite())
        );
        assert_eq!(motion.last_world_foot, Vec2::ZERO);
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
        for source in ["chris_bike", "kris_bike", "surf", "surf_pikachu"] {
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
    fn foot_return_matches_stance_velocity_at_both_contact_boundaries() {
        let epsilon = 0.0001;
        for run in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for phase in [0.0, TAU * stance_fraction(run)] {
                let center = foot_target(phase, 1.0, run).0;
                let before = foot_target(phase - epsilon, 1.0, run).0;
                let after = foot_target(phase + epsilon, 1.0, run).0;
                let left = (center - before) / epsilon;
                let right = (after - center) / epsilon;
                assert!(
                    (left - right).abs() < 0.03,
                    "contact velocity discontinuity run={run}: {left} -> {right}"
                );
            }
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
    fn sampled_walk_trajectories_remain_continuous_at_30_and_60_hz() {
        let mut finals = Vec::new();
        for hz in [9, 30, 60] {
            let dt = 1.0 / hz as f32;
            let mut motion = Motion::new(Vec2::ZERO, Vec2::X);
            let mut previous = pose(CharacterKind::Trainer, &motion, 1.0);
            let mut maximum_limb_step = 0.0_f32;
            let mut maximum_hip_step = 0.0_f32;
            for frame in 1..=hz * 4 {
                motion.advance(
                    Vec2::new(frame as f32 * dt * 24.0, 0.0),
                    Some(Vec2::X),
                    16.0,
                    dt,
                );
                let current = pose(CharacterKind::Trainer, &motion, 1.0 + frame as f32 * dt);
                for joint in [THIGH_L, THIGH_R, SHIN_L, SHIN_R, UPPER_ARM_L, UPPER_ARM_R] {
                    maximum_limb_step = maximum_limb_step.max(
                        previous[joint]
                            .rotation
                            .angle_between(current[joint].rotation)
                            .abs(),
                    );
                }
                maximum_hip_step = maximum_hip_step
                    .max((previous[PELVIS].translation.y - current[PELVIS].translation.y).abs());
                previous = current;
            }
            println!(
                "gait {hz}Hz at 1.5tiles/s: maximum limb step={:.2}deg hip step={:.4}tiles",
                maximum_limb_step.to_degrees(),
                maximum_hip_step
            );
            if hz >= 30 {
                assert!(maximum_limb_step < 0.60);
                assert!(maximum_hip_step < 0.018);
            }
            finals.push(motion.phase);
        }
        for phase in finals {
            assert!((phase - (6.0 / CYCLE_DISTANCE * TAU).rem_euclid(TAU)).abs() < 0.0001);
        }
    }
    #[test]
    fn production_speed_pose_sampling_audit() {
        // Authoritative Crystal step: eight 70224-cycle Game Boy VBlanks.
        for (hz, speed) in [(30, 3.716_f32), (60, 4_194_304.0_f32 / (8.0 * 70_224.0))] {
            let dt = 1.0 / hz as f32;
            let mut motion = Motion::new(Vec2::ZERO, Vec2::X);
            let mut previous = pose(CharacterKind::Trainer, &motion, 0.0);
            let mut max_angle = 0.0_f32;
            let mut max_foot_step = 0.0_f32;
            let mut previous_foot = Vec2::ZERO;
            for frame in 1..=hz * 4 {
                let root_x = frame as f32 * dt * speed;
                motion.advance(Vec2::new(root_x * 16.0, 0.0), Some(Vec2::X), 16.0, dt);
                let current = pose(CharacterKind::Trainer, &motion, frame as f32 * dt);
                let target = foot_target(motion.phase, motion.walk_weight, motion.run_weight);
                let foot = Vec2::new(root_x + target.0, target.1);
                if frame > hz {
                    for joint in [THIGH_L, THIGH_R, SHIN_L, SHIN_R, UPPER_ARM_L, UPPER_ARM_R] {
                        max_angle = max_angle.max(
                            previous[joint]
                                .rotation
                                .angle_between(current[joint].rotation)
                                .abs(),
                        );
                    }
                    max_foot_step = max_foot_step.max((foot - previous_foot).length());
                }
                previous = current;
                previous_foot = foot;
            }
            println!(
                "production gait {hz}Hz: speed={speed:.4}tiles/s cycles={:.3}/s contacts={:.3}/s max_joint_step={:.2}deg max_foot_step={max_foot_step:.4}tiles",
                speed / cycle_distance(motion.run_weight),
                speed / cycle_distance(motion.run_weight) * 2.0,
                max_angle.to_degrees()
            );
            assert!(motion.run_weight > 0.999);
            assert!(
                max_angle < 0.49,
                "production running limb delta: {} degrees",
                max_angle.to_degrees()
            );
            assert!(max_foot_step < 0.24);
            assert!(2.0 * speed / cycle_distance(motion.run_weight) < 5.4);
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
    fn articulated_gait_keeps_stance_shoes_planted_through_entire_cycle() {
        for run in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for i in 0..256 {
                let phase = i as f32 / 256.0 * TAU;
                for weight in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    let motion = Motion {
                        phase,
                        walk_weight: weight,
                        run_weight: run,
                        ..Motion::new(Vec2::ZERO, Vec2::NEG_Y)
                    };
                    let transforms = pose(CharacterKind::Trainer, &motion, 1.0);
                    let rig = rig(CharacterKind::Trainer);
                    let mut world = [Mat4::IDENTITY; JOINT_COUNT];
                    for j in 0..JOINT_COUNT {
                        world[j] = rig.joints[j]
                            .parent
                            .map(|p| world[p])
                            .unwrap_or(Mat4::IDENTITY)
                            * transforms[j].compute_matrix();
                    }
                    for (shoe, offset) in [(SHOE_L, 0.0), (SHOE_R, PI)] {
                        let (z, y) = foot_target(phase + offset, weight, run);
                        let ankle = world[shoe].transform_point3(Vec3::ZERO);
                        assert!(
                            (ankle.y - y).abs() < 0.0001,
                            "foot y phase={phase} weight={weight} run={run}: {ankle:?}, target={y}"
                        );
                        assert!(
                            (ankle.z - z).abs() < 0.0001,
                            "foot z phase={phase} weight={weight} run={run}: {ankle:?}, target={z}"
                        );
                        assert!(
                            (world[shoe].transform_vector3(Vec3::Y) - Vec3::Y).length() < 0.0001
                        );
                        assert!(ankle.y >= ANKLE_HEIGHT - 0.0001);
                    }
                }
            }
        }
    }
    #[test]
    fn running_has_an_aerial_phase_without_stance_sliding_or_leg_scaling() {
        for run in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let stance = stance_fraction(run);
            let a = foot_target(stance * 0.25 * TAU, 1.0, run);
            let b = foot_target(stance * 0.75 * TAU, 1.0, run);
            let root_travel = cycle_distance(run) * stance * 0.5;
            assert!((root_travel + b.0 - a.0).abs() < 0.00001);
            assert_eq!(a.1, ANKLE_HEIGHT);
            assert_eq!(b.1, ANKLE_HEIGHT);
            let air_samples = (0..128)
                .filter(|&i| {
                    let p = i as f32 / 128.0 * TAU;
                    foot_target(p, 1.0, run).1 > ANKLE_HEIGHT + 0.001
                        && foot_target(p + PI, 1.0, run).1 > ANKLE_HEIGHT + 0.001
                })
                .count();
            if run == 0.0 {
                assert_eq!(air_samples, 0);
            } else {
                assert!(air_samples > 0);
            }
            let state = Motion {
                run_weight: run,
                walk_weight: 1.0,
                ..Motion::new(Vec2::ZERO, Vec2::NEG_Y)
            };
            for transform in pose(CharacterKind::Trainer, &state, 1.0) {
                assert_eq!(transform.scale, Vec3::ONE);
            }
        }
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
    #[test]
    fn opposite_limbs_really_articulate_and_blinks_preserve_the_head() {
        let motion = Motion {
            phase: 0.3,
            walk_weight: 1.0,
            ..Motion::new(Vec2::ZERO, Vec2::NEG_Y)
        };
        let a = pose(CharacterKind::Trainer, &motion, 1.0);
        assert_ne!(a[THIGH_L].rotation, a[THIGH_R].rotation);
        assert_ne!(a[UPPER_ARM_L].rotation, a[UPPER_ARM_R].rotation);
        assert_ne!(a[SHIN_L].rotation, Quat::IDENTITY);
        let blink = pose(CharacterKind::Trainer, &motion, 0.0725);
        assert!(blink[EYES].scale.y < 0.1);
        assert_eq!(blink[HEAD].scale, Vec3::ONE);
    }
}
