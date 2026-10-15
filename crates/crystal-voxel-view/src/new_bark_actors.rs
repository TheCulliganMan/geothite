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
// Presentation scale belongs to the player identity, not its sprite source:
// NPCs using the same trainer art keep their authored stature.
const PLAYER_PRESENTATION_SCALE: f32 = 1.25;

fn character_scale(actor: &VisualActor, tile_height: f32) -> f32 {
    tile_height * 2.0 * if actor.id==VisualActorId::Player {PLAYER_PRESENTATION_SCALE} else {1.0}
}

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
    shadow: Entity,
    idle_offset: f32,
    person: crate::open_people_skin::Person,
}

struct PropInstance {
    entity: Entity,
    pedals: Option<crate::bicycle::Pedals>,
    kind: actor_props::PropKind,
    stone: Option<(CaveStoneFinish, u8)>,
    gold_gym_stone: bool,
    model_palette: Option<crystal_render_api::VisualPokemonModelPalette>,
    skin: Option<WorldPokemonSkin>,
}

/// Render-only playback of the same canonical skeleton used in battle.
struct WorldPokemonSkin {
    rig: &'static crate::species_rig::SpeciesRig,
    joints: Vec<Entity>,
    pose: Vec<Transform>,
    seconds: f32,
    joint_offsets: Vec<(usize,Quat)>,
}

fn world_pokemon_joint_offsets(rig: &crate::species_rig::SpeciesRig, npc: bool) -> Vec<(usize,Quat)> {
    if !npc || rig.species!=crate::species_rig::Species::Butterfree {return Vec::new();}
    [(4,-1.15),(6,1.15),(8,-1.20),(10,1.20)].into_iter()
        .map(|(joint,angle)|(joint,Quat::from_rotation_y(angle))).collect()
}

fn world_pokemon_actor_rig(kind:actor_props::PropKind,id:VisualActorId) -> Option<&'static crate::species_rig::SpeciesRig> {
    let canonical=world_pokemon_rig(kind)?;
    if !matches!(id,VisualActorId::Object(_)) || canonical.species!=crate::species_rig::Species::Butterfree {
        return Some(canonical);
    }
    static PERCHED:std::sync::OnceLock<crate::species_rig::SpeciesRig>=std::sync::OnceLock::new();
    Some(PERCHED.get_or_init(|| {
        let mut rig=canonical.clone();
        for part in &rig.anatomy {
            if !part.name.starts_with("Wing") {continue;}
            let lower=if part.name.starts_with("Wing vein /") {
                part.name.split(" / ").nth(1).unwrap().split_whitespace().nth(1).unwrap().parse::<usize>().unwrap()>=4
            } else {part.name.contains("lower")};
            let left=rig.neutral.positions[part.vertices.start][0]<0.;
            let owner=match (lower,left) {(false,true)=>4,(false,false)=>6,(true,true)=>8,(true,false)=>10};
            // A resting wing is one hinged panel. Its rim, inset membrane and
            // front/back veins must not shear into each other during the fold.
            for v in part.vertices.clone() {
                rig.joint_indices[v]=[owner,0,0,0];rig.joint_weights[v]=[1.,0.,0.,0.];
            }
        }
        rig.animated_bounds=rig.idle_bounds_with_joint_offsets(&world_pokemon_joint_offsets(&rig,true));
        rig
    }))
}

fn world_pokemon_rig(kind: actor_props::PropKind) -> Option<&'static crate::species_rig::SpeciesRig> {
    use actor_props::PropKind as P;
    use crate::species_rig::{self, Species as S};
    let species = match kind {
        P::ExactSpecies(name) => return species_rig::for_species(name),
        P::BattleChikorita => S::Chikorita,
        P::BattleCyndaquil => S::Cyndaquil,
        P::BattleTotodile => S::Totodile,
        P::IconClefairy => S::Clefairy,
        P::IconJigglypuff => S::Jigglypuff,
        P::IconStaryu => S::Staryu,
        P::IconVoltorb => S::Voltorb,
        P::IconDiglett => S::Diglett,
        P::Slowpoke | P::IconSlowpoke => S::Slowpoke,
        P::BigOnix => S::Onix,
        P::Entei => S::Entei,
        P::Raikou => S::Raikou,
        P::Suicune => S::Suicune,
        P::IconUnown => S::Unown(8),
        _ => return None,
    };
    Some(species_rig::rig(species))
}

impl WorldPokemonSkin {
    fn spawn(
        commands: &mut Commands,
        entity: Entity,
        rig: &'static crate::species_rig::SpeciesRig,
        npc: bool,
        binds: &mut Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>,
        cache: &mut HashMap<&'static str, Handle<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
    ) -> Self {
        let inverse_bindposes = cache.entry(rig.species.name()).or_insert_with(|| {
            binds.add(bevy::mesh::skinning::SkinnedMeshInverseBindposes::from(
                rig.joints.iter().map(|joint| joint.inverse_bind).collect::<Vec<_>>()))
        }).clone();
        let joint_offsets=world_pokemon_joint_offsets(rig,npc);
        let mut pose=rig.joints.iter().map(|joint|joint.bind).collect::<Vec<_>>();
        for &(joint,rotation) in &joint_offsets {pose[joint].rotation=(rotation*pose[joint].rotation).normalize();}
        let bounds=rig.animated_bounds;
        let mut joints = Vec::with_capacity(rig.joints.len());
        for (index,joint) in rig.joints.iter().enumerate() {
            let child = commands.spawn((Visibility::default(), pose[index], ModeledActor)).id();
            commands.entity(joint.parent.map_or(entity, |parent| joints[parent])).add_child(child);
            joints.push(child);
        }
        commands.entity(entity).insert((
            bevy::mesh::skinning::SkinnedMesh { inverse_bindposes, joints: joints.clone() },
            bevy::camera::primitives::Aabb::from_min_max(bounds.0,bounds.1),
        ));
        Self { rig, joints, pose, seconds: 0., joint_offsets }
    }

    fn advance(&mut self, delta: f32, transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        use crate::species_rig::{Clip, Playback};
        if !delta.is_finite() || delta < 0. { return; }
        self.seconds = (self.seconds + delta).rem_euclid(self.rig.clip(Clip::Idle).duration);
        self.rig.sample_into(Clip::Idle, self.seconds, Playback::Loop, &mut self.pose)
            .expect("validated world Pokemon idle sample");
        for &(joint,rotation) in &self.joint_offsets {
            self.pose[joint].rotation=(rotation*self.pose[joint].rotation).normalize();
        }
        for (&entity, pose) in self.joints.iter().zip(&self.pose) {
            if let Ok(mut transform) = transforms.get_mut(entity) { *transform = *pose; }
        }
    }
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
    #[cfg(not(target_arch = "wasm32"))]
    asset_warmup: NativeAssetWarmup,
    people_gpu: crate::open_people_skin::PeopleGpu,
    material: Option<Handle<StandardMaterial>>,
    remote_material: Option<Handle<StandardMaterial>>,
    perched_wing_material: Option<Handle<StandardMaterial>>,
    shadow_mesh: Option<Handle<Mesh>>,
    shadow_material: Option<Handle<StandardMaterial>>,
    instances: HashMap<VisualActorId, Instance>,
    map_id: String,
    gpu_map_id: String,
    seating_key: Option<crate::TerrainCacheKey>,
    seats: Vec<crate::seating::Seat>,
    prop_meshes: HashMap<(actor_props::PropKind, Option<(CaveStoneFinish, u8)>, bool, Option<crystal_render_api::VisualPokemonModelPalette>,bool), Handle<Mesh>>,
    perched_wing_meshes: HashMap<(actor_props::PropKind,Option<crystal_render_api::VisualPokemonModelPalette>),Handle<Mesh>>,
    stone_materials: HashMap<CaveStoneFinish, Handle<VoxelMaterial>>,
    props: HashMap<VisualActorId, PropInstance>,
    pokemon_binds: HashMap<&'static str, Handle<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct NativeAssetWarmup {
    interest: Vec<(&'static str, &'static str)>,
    task: Option<bevy::tasks::Task<()>>,
    ready: bool,
}
#[cfg(not(target_arch = "wasm32"))]
impl NativeAssetWarmup {
    fn pending(&mut self, frame: &VisualWorldFrame) -> bool {
        use bevy::tasks::{AsyncComputeTaskPool, futures_lite::future};
        let Some(pool) = AsyncComputeTaskPool::try_get() else { return false; };
        let mut interest = frame.actors.iter().filter_map(character_kind)
            .map(crate::npc_appearance::look).map(|look| (look.base,look.texture_key)).collect::<Vec<_>>();
        interest.sort_unstable(); interest.dedup();
        if self.interest != interest || (!self.ready && self.task.is_none()) {
            self.interest = interest.clone();
            self.ready = false;
            self.task = Some(pool.spawn(async move {
                let _trace = crate::loading_trace::span("region.prepare_actor_assets", || serde_json::json!({"requested":interest.len()}));
                for (base,painting) in interest {
                    let rig = crate::open_people::get(base).expect("required production character rig");
                    let _ = rig.refined_body();
                    crate::npc_textures::get(painting).expect("required production character painting");
                }
            }));
        }
        if let Some(task) = &mut self.task {
            if future::block_on(future::poll_once(task)).is_some() {
                self.task = None;
                self.ready = true;
            }
        }
        !self.ready
    }
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
                && (character_kind(actor).is_some() || prop_kind(actor).is_some()
                    || frame.actors.iter().any(|a| a.id == VisualActorId::SurfMount))
        })
}

// Every human source receives an explicit adaptation of the external rig.
// Character assets are required; there is no legacy model renderer.
fn person_name(kind: CharacterKind) -> &'static str {
    crate::npc_appearance::look(kind).base
}
pub(crate) fn player_eye_height(actor: &VisualActor) -> f32 {
    character_kind(actor).map_or(1.2, person_height) * 0.85
}

fn person_height(kind: CharacterKind) -> f32 {
    crate::npc_appearance::look(kind).height
}

/// Swimmer NPCs retain their authored map foot, but most of the rig lies
/// beneath the live water plane. Shore/land poses keep ordinary support.
fn swimmer_depth(frame: &VisualWorldFrame, actor: &VisualActor, kind: CharacterKind) -> f32 {
    if !matches!(actor.id, VisualActorId::Object(_))
        || !matches!(kind, CharacterKind::SwimmerGuy | CharacterKind::SwimmerGirl)
    { return 0.; }
    let Some(cell) = crate::tile_at_visual_point(frame, actor_foot(actor) + Vec2::Y * 0.01)
    else { return 0.; };
    if !frame.tiles.iter().any(|tile| tile.column == cell.x && tile.row == cell.y
        && crate::shape_for_source(&tile.source) == crate::CellShape::Water)
    { return 0.; }
    character_scale(actor, frame.tile_size.y) * person_height(kind) * 0.68
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
    // Ground displacement stays on the production foot anchor. Height includes
    // only resolved terrain support and the current presentation hop.
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

fn furniture_seat_target(frame: &VisualWorldFrame, seat: crate::seating::Seat,
    actor: &VisualActor, ground: f32) -> Option<Vec3> {
    let foot = actor_foot(actor);
    if let Some(target) = seat.target(frame, foot, ground) { return Some(target); }
    // Authored NPCs sit on their actual tile; front-approach presentation is
    // reserved for players, so dialogue scripts cannot visually relocate Mom.
    if !matches!(actor.id, VisualActorId::Player | VisualActorId::RemotePlayer(_)) { return None; }
    let facing = actor.facing.filter(|f| f.is_finite() && f.length_squared() > 0.5)?;
    let ahead = foot + facing.normalize() * frame.tile_size * 2.;
    let radius = frame.tile_size.min_element() * 0.5;
    if frame.actors.iter().any(|other| other.id != actor.id
        && matches!(other.id, VisualActorId::Object(_) | VisualActorId::Player | VisualActorId::RemotePlayer(_))
        && actor_foot(other).distance_squared(ahead) < radius * radius) { return None; }
    seat.actor_target(frame, foot, actor.facing, ground)
}

fn surf_seat_height(frame: &VisualWorldFrame) -> Option<f32> {
    let mount = frame.actors.iter().find(|a| a.id == VisualActorId::SurfMount)?;
    let kind = prop_kind(mount)?;
    static SEATS: std::sync::OnceLock<std::sync::Mutex<HashMap<actor_props::PropKind, f32>>> = std::sync::OnceLock::new();
    let mut seats = SEATS.get_or_init(Default::default).lock().unwrap();
    let seat = *seats.entry(kind).or_insert_with(|| {
        let mesh = actor_props::mesh(kind);
        let low = mesh.positions.iter().map(|p| Vec3::from_array(*p))
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let high = mesh.positions.iter().map(|p| Vec3::from_array(*p))
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        let center = (low + high) * 0.5;
        // Sample the torso's central support, excluding tall forward heads,
        // horns and tails that do not provide a seat for the rider.
        mesh.positions.iter().filter(|p|
            (p[0] - center.x).abs() <= (high.x-low.x) * 0.25
                && (p[2] - center.z).abs() <= (high.z-low.z) * 0.15)
            .map(|p| p[1]).fold(0.25_f32, f32::max)
    });
    Some(seat * frame.tile_size.y * 2.0)
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
    #[cfg(not(target_arch = "wasm32"))]
    mut status: ResMut<VoxelViewStatus>,
    #[cfg(target_arch = "wasm32")]
    status: Res<VoxelViewStatus>,
    settings: Option<Res<crate::VoxelViewSettings>>,
    presentation: (
        Option<Res<crystal_render_api::VisualSurfRider>>,
        Option<Res<crystal_render_api::VisualPlayerTraversal>>,
    ),
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
    let _trace = crate::loading_trace::span("frame.modeled_actors", || serde_json::json!({"map":frame.map_id.as_ref()}));
    let (surf_rider, traversal) = presentation;
    let player_visibility = if settings.as_ref().is_some_and(|s| s.camera.first_person) {
        Visibility::Hidden
    } else { Visibility::Inherited };
    #[cfg(not(target_arch = "wasm32"))]
    if frame.active && enabled_map(&frame.map_id)
        && settings.as_ref().is_none_or(|settings| settings.enabled)
        && state.asset_warmup.pending(&frame) && status.active {
        status.terrain_pending = true;
        status.active_frames = 0;
        return;
    }
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
    if state.gpu_map_id != frame.map_id.as_ref() {
        // Retain common roles (especially the player) across the transition,
        // but release uploaded bodies and paintings belonging only to old maps.
        // Entity handles keep the retiring frame alive until deferred despawns.
        let needed = frame.actors.iter().filter_map(character_kind).collect();
        state.people_gpu.retain_kinds(&needed);
        state.gpu_map_id = frame.map_id.to_string();
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
    let surf_mounted = surf_seat_height(&frame).is_some();
    let visible: HashSet<_> = frame
        .actors
        .iter()
        .filter(|actor| character_kind(actor).is_some()
            || (surf_mounted && actor.id == VisualActorId::Player))
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
        .filter(|actor| prop_kind(actor).is_some()
            && !(surf_mounted && actor.id == VisualActorId::Player))
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
        .filter(|actor| prop_kind(actor).is_some()
            && !(surf_mounted && actor.id == VisualActorId::Player))
    {
        let material = if matches!(actor.id, VisualActorId::RemotePlayer(_)) {
            remote_material.clone()
        } else {
            material.clone()
        };
        let kind = prop_kind(actor).expect("filtered resolved prop");
        let rig=world_pokemon_actor_rig(kind,actor.id);
        let perched=matches!(actor.id,VisualActorId::Object(_)) && rig.is_some_and(|r|r.species==crate::species_rig::Species::Butterfree);
        let gold_gym_stone = matches!(frame.map_id.as_ref(), "CianwoodGym" | "BlackthornGym2F")
            && matches!(actor.id, VisualActorId::Object(_))
            && kind == actor_props::PropKind::Boulder;
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
        // Flat stationary decorations use the center of their source drawing
        // only when a complete, actually modeled table supplies their support.
        let tabletop=if kind==actor_props::PropKind::Pokedex
            && matches!(actor.id,VisualActorId::Object(_)) {
            footing.built_frame.as_ref().filter(|built|
                built.map_id==frame.map_id&&built.tile_size==frame.tile_size)
                .and_then(|built|crate::mesh::traditional_tabletop_height(built,
                    footing.built_profiles.as_deref(),actor.center))
        } else {None};
        let render_foot=if tabletop.is_some() {actor.center}else{prop_foot(actor,kind)};
        let Some(height) = tabletop.or_else(||
            resolved_footing_height(&frame, render_foot, &footing.footing_heights))
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
        let prop_scale = match (actor.id, kind) {
            (_, actor_props::PropKind::PokeBall) => scale * 0.5,
            // The supplied moth icon has two 16px frames with visible heights
            // of 12px and 14px. Use the maximum so its resting NPC keeps a
            // stable source-sized silhouette throughout the icon animation.
            (VisualActorId::Object(_), actor_props::PropKind::ExactSpecies("BUTTERFREE"))
                if actor.source_id.ends_with(":icon_moth") => {
                    let rig = rig.expect("Butterfree skin");
                    actor.size.y * (14.0 / 16.0)
                        / (rig.neutral_bounds.1.y - rig.neutral_bounds.0.y)
                }
            // The battle sculpture's one-unit height is a model datum. A
            // walking Totodile is a small starter, below the trainer's waist.
            (VisualActorId::PartyFollower, actor_props::PropKind::ExactSpecies("TOTODILE")) => scale * 0.45,
            _ => scale,
        };
        let transform = Transform::from_translation(visual_point_to_voxel(render_foot, height + 0.04))
            .with_rotation(Quat::from_rotation_y(facing_yaw(facing)))
            .with_scale(Vec3::splat(prop_scale));
        // A variable sprite/decor/species can change without a new object ID.
        // Replace its cached instance rather than leaving yesterday's mesh.
        if state
            .props
            .get(&actor.id)
            .is_some_and(|instance| instance.kind != kind || instance.stone != stone
                || instance.gold_gym_stone != gold_gym_stone || instance.model_palette != actor.model_palette)
        {
            let old = state.props.remove(&actor.id).unwrap();
            commands.entity(old.entity).despawn();
        }
        if let Some(instance) = state.props.get_mut(&actor.id) {
            if let Ok(mut current) = transforms.get_mut(instance.entity) {
                *current = transform;
            }
            if let Some(skin) = &mut instance.skin {
                skin.advance(time.delta_secs(), &mut transforms);
            }
        } else {
            let key=(kind,stone,gold_gym_stone,actor.model_palette,perched);
            if perched && !state.prop_meshes.contains_key(&key) {
                let rig=rig.expect("perched skin");
                let data=actor.model_palette.map_or(rig.neutral.clone(),|palette|
                    crate::pokemon_appearance::mesh(&rig.neutral,palette));
                let mut body=data.clone();let mut wings=data;
                body.indices.clear();wings.indices.clear();
                for part in &rig.anatomy {
                    let indices=&rig.neutral.indices[part.indices.clone()];
                    if part.name.starts_with("Wing") {wings.indices.extend_from_slice(indices);}
                    else {body.indices.extend_from_slice(indices);}
                }
                let make_mesh=|data:crate::mesh::SurfaceMeshData| {
                    let mut mesh=data.into_mesh();
                    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX,
                        bevy::mesh::VertexAttributeValues::Uint16x4(rig.joint_indices.clone()));
                    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT,rig.joint_weights.clone());
                    mesh
                };
                let body=meshes.add(make_mesh(body));let wings=meshes.add(make_mesh(wings));
                state.prop_meshes.insert(key,body);
                state.perched_wing_meshes.insert((kind,actor.model_palette),wings);
                if state.perched_wing_material.is_none() {
                    state.perched_wing_material=Some(materials.add(StandardMaterial {
                        base_color:Color::WHITE,perceptual_roughness:0.95,reflectance:0.08,
                        diffuse_transmission:0.5,..default()
                    }));
                }
            }
            let mesh = state
                .prop_meshes
                .entry(key)
                .or_insert_with(|| meshes.add(if gold_gym_stone {
                    actor_props::gold_gym_boulder_mesh().into_mesh()
                } else if let Some((finish,variant))=stone {
                    let mut data=crate::mesh::movable_cave_stone(finish==CaveStoneFinish::Frozen,
                        finish==CaveStoneFinish::Dark, variant);
                    // Smashable rocks retain their smaller authored extent.
                    if kind==actor_props::PropKind::Rock {
                        for p in &mut data.positions {
                            *p=(Vec3::from_array(*p)*(0.43/0.64)).to_array();
                        }
                    }
                    data.into_mesh()
                } else {
                    let data = actor_props::mesh(kind);
                    let mut mesh = actor.model_palette.map_or(data.clone(), |palette|
                        crate::pokemon_appearance::mesh(&data, palette)).into_mesh();
                    if let Some(rig) = rig {
                        assert_eq!(mesh.count_vertices(), rig.neutral.positions.len(), "canonical world skin vertex order");
                        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX,
                            bevy::mesh::VertexAttributeValues::Uint16x4(rig.joint_indices.clone()));
                        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, rig.joint_weights.clone());
                    }
                    mesh
                }))
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
            let pedals = matches!(kind, actor_props::PropKind::ChrisBike | actor_props::PropKind::KrisBike)
                .then(|| crate::bicycle::spawn_pedals(&mut commands, &mut meshes,
                    material.clone(), entity, 0.));
            if actor.id == VisualActorId::Player {
                commands.entity(entity).insert(player_visibility);
            }
            let skin = rig.map(|rig| WorldPokemonSkin::spawn(&mut commands, entity, rig,
                matches!(actor.id,VisualActorId::Object(_)),
                &mut binds, &mut state.pokemon_binds));
            if perched {
                let skin=skin.as_ref().expect("perched body skin");
                let wing=commands.spawn((Mesh3d(state.perched_wing_meshes[&(kind,actor.model_palette)].clone()),
                    MeshMaterial3d(state.perched_wing_material.as_ref().unwrap().clone()),
                    bevy::mesh::skinning::SkinnedMesh {
                        inverse_bindposes:state.pokemon_binds[skin.rig.species.name()].clone(),joints:skin.joints.clone()
                    },bevy::camera::primitives::Aabb::from_min_max(skin.rig.animated_bounds.0,skin.rig.animated_bounds.1),
                    Transform::IDENTITY,Visibility::default(),ModeledActor,RenderLayers::layer(VOXEL_RENDER_LAYER))).id();
                commands.entity(entity).add_child(wing);
            }
            state.props.insert(actor.id, PropInstance { entity, pedals, kind, stone, gold_gym_stone, model_palette: actor.model_palette, skin });
        }
        if let Some(entity) = cards.entities.get(&actor.id)
            && let Ok(mut visibility) = card_visibility.get_mut(*entity)
        {
            *visibility = Visibility::Hidden;
        }
    }
    if let Some(instance) = state.instances.get(&VisualActorId::Player) {
        commands.entity(instance.entity).insert(player_visibility);
    }
    if let Some(instance) = state.props.get(&VisualActorId::Player) {
        commands.entity(instance.entity).insert(player_visibility);
    }
    let mut seating_key = crate::TerrainCacheKey::from_frame(&frame);
    seating_key.profiles_revision = footing.key.as_ref().map_or(0, |key| key.profiles_revision);
    let seating_key = Some(seating_key);
    if state.seating_key != seating_key {
        state.seats = crate::mesh::seating_surfaces(&frame, footing.built_profiles.as_deref());
        state.seating_key = seating_key;
    }
    for actor in &frame.actors {
        let mounted = actor.id == VisualActorId::Player && surf_seat_height(&frame).is_some();
        let kind = if mounted {
            Some(if surf_rider.as_ref().is_some_and(|rider| rider.female) {
                CharacterKind::TrainerFemale
            } else { CharacterKind::Trainer })
        } else { character_kind(actor) };
        let Some(kind) = kind else {
            continue;
        };
        // Human sprites never substitute for the required 3D character,
        // including a frame waiting for valid ground support.
        if let Some(entity) = cards.entities.get(&actor.id)
            && let Ok(mut visibility) = card_visibility.get_mut(*entity)
        {
            *visibility = Visibility::Hidden;
        }
        let Some((height, hop_weight)) =
            crate::footing::traversal_height(&frame, actor, &footing.footing_heights, traversal.as_deref())
        else {
            // Wait for valid support; neither a stale rig nor an old sprite substitutes.
            if let Some(instance) = state.instances.remove(&actor.id) {
                commands.entity(instance.entity).despawn();
            }
            continue;
        };
        let surf_seat = (actor.id == VisualActorId::Player).then(|| surf_seat_height(&frame)).flatten();
        let height = height + surf_seat.unwrap_or(0.0) - swimmer_depth(&frame, actor, kind);
        let world_foot = world_foot(&frame, actor);
        let scale = character_scale(actor,frame.tile_size.y);
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
        let seat = if surf_seat.is_some() || actor.source_id.ends_with("_bike") { None }
            else { state.seats.iter().find_map(|seat| furniture_seat_target(&frame, *seat, actor, height)
                .map(|offset| (offset, seat.height, seat.yaw))) };
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
                person.surfing = surf_seat.is_some();
                person.hop_step = (hop_weight > 0.0).then(|| (
                    traversal.as_ref().and_then(|t| t.step).unwrap().progress,
                    hop_weight,
                ));
                if person.bicycle || person.surfing {
                    person.sitting = Default::default();
                } else {
                    person.sitting.advance(seat, instance.motion.still_for > 0.30 && hop_weight == 0., time.delta_secs(), scale);
                }
                if let Ok(mut transform) = transforms.get_mut(instance.entity) {
                    *transform = model_transform(
                        actor,
                        height,
                        instance.motion.yaw,
                        scale * person_height(kind) / person.rig.height,
                    );
                    transform.scale.x *= crate::npc_appearance::look(kind).width;
                    transform.translation += person.sitting.offset * person.sitting.eased_weight();
                    if let Some(yaw) = person.sitting.yaw {
                        transform.rotation = transform.rotation.slerp(Quat::from_rotation_y(yaw), person.sitting.eased_weight());
                    }
                }
                person.update(
                    instance.motion.phase,
                    instance.motion.walk_weight * (1.0 - hop_weight),
                    time.elapsed_secs() + instance.idle_offset,
                    &mut transforms,
                );
                // The soft contact shadow stays on the terrain while the root
                // rises; its local offset must account for the imported scale.
                let root_scale = scale * person_height(kind) / person.rig.height;
                let ground = resolved_footing_height(&frame, actor_foot(actor), &footing.footing_heights)
                    .unwrap_or(height);
                if let Ok(mut shadow) = transforms.get_mut(instance.shadow) {
                    shadow.translation.y = (ground - height - person.sitting.offset.y * person.sitting.eased_weight()) / root_scale - 0.001;
                }
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
                        (if actor.id == VisualActorId::Player { player_visibility } else { Visibility::Inherited },
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
                    actor.source_id.ends_with("_bike"),
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut images,
                    &mut binds,
                );
                person.surfing = surf_seat.is_some();
                let shadow = commands
                    .spawn((
                        (Mesh3d(state.shadow_mesh.as_ref().unwrap().clone()),
MeshMaterial3d(state.shadow_material.as_ref().unwrap().clone()),
Transform::from_xyz(0.0,
                                if hop_weight > 0.0 {
                                    (resolved_footing_height(&frame, actor_foot(actor), &footing.footing_heights)
                                        .unwrap_or(height) - height)
                                        / (scale * person_height(kind) / person.rig.height) - 0.001
                                } else { -0.001 },
                                0.0)
                                .with_scale(Vec3::splat(person.rig.height / person_height(kind))),),
                        NotShadowCaster,
                        NotShadowReceiver,
                        ModeledActor,
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
                        shadow,
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
    // Update after the rider has advanced, including its initial zero phase.
    if let (Some(mount), Some(rider)) = (state.props.get(&VisualActorId::SurfMount),
        state.instances.get(&VisualActorId::Player)) {
        if let Ok(rider_root) = transforms.get(rider.entity) {
            let rotation = rider_root.rotation;
            if let Ok(mut mount_root) = transforms.get_mut(mount.entity) {
                mount_root.rotation = rotation;
            }
        }
    }
    // Each bike owns its articulated entities; cached frame meshes stay shared.
    for (id, prop) in &state.props {
        if let Some(pedals) = &prop.pedals {
            let phase = state.instances.get(id).map(|rider| rider.motion.phase).unwrap_or(0.);
            if let Some(rider) = state.instances.get(id)
                && let Ok(rider_root) = transforms.get(rider.entity)
            {
                let position = rider_root.translation;
                let rotation = rider_root.rotation;
                if let Ok(mut bike_root) = transforms.get_mut(prop.entity) {
                    // Preserve the separately authored bike scale, but use
                    // the rider's presented turn and exact ground anchor.
                    bike_root.translation = position;
                    bike_root.rotation = rotation;
                }
            }
            pedals.set_phase(&mut commands, phase);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn actor() -> VisualActor {
        VisualActor { model_palette: None,
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
    #[test]
    fn automatic_seating_respects_npc_placement_and_occupied_seats() {
        let mut frame = VisualWorldFrame::default();
        frame.grid_size = UVec2::new(4, 4);
        frame.tile_size = Vec2::splat(8.);
        frame.center = Vec2::new(16., -16.);
        let seat = crate::seating::Seat { rect: [0., 0., 2., 2.], height: 5., yaw: Some(0.) };
        let mut player = actor();
        player.center = Vec2::new(8., -24.);
        player.facing = Some(Vec2::Y);
        assert_eq!(furniture_seat_target(&frame, seat, &player, 0.), Some(Vec3::new(0., 5., -24.)));
        let mut npc = player.clone();
        npc.id = VisualActorId::Object(7);
        assert!(furniture_seat_target(&frame, seat, &npc, 0.).is_none(), "NPC must retain its authored tile");
        npc.center.y += 16.;
        assert_eq!(furniture_seat_target(&frame, seat, &npc, 0.), Some(Vec3::new(0., 5., -8.)));
        frame.actors = vec![player.clone(), npc];
        assert!(furniture_seat_target(&frame, seat, &player, 0.).is_none(), "occupied seat must not be claimed");
        frame.actors[1].source_id = "growlithe".into();
        assert!(furniture_seat_target(&frame, seat, &player, 0.).is_none(), "animal occupants also keep their seat");
    }

    #[test]
    fn swimmers_show_head_and_shoulders_above_authenticated_water_only() {
        let mut a = actor();
        a.id = VisualActorId::Object(7);
        a.center = Vec2::new(0., 8.);
        let mut f = VisualWorldFrame::default();
        f.grid_size = UVec2::ONE;
        f.viewport_size = Vec2::splat(8.);
        f.tile_size = Vec2::splat(8.);
        f.tiles.push(crystal_render_api::VisualTile {
            column: 0, row: 0, texture: Handle::default(), animation_frames: None, priority: false,
            source: crystal_render_api::VisualTileSource {
                tileset_id: "kanto".into(), metatile_id: 0x43,
                subtile_column: 0, subtile_row: 0, tile_index: 0x14,
            },
        });
        for kind in [CharacterKind::SwimmerGuy, CharacterKind::SwimmerGirl] {
            let total = character_scale(&a, f.tile_size.y) * person_height(kind);
            let above = total - swimmer_depth(&f, &a, kind);
            assert!(above > total * 0.25 && above < total * 0.40,
                "only the upper head/shoulder portion should clear water");
            let mut player = a.clone(); player.id = VisualActorId::Player;
            assert_eq!(swimmer_depth(&f, &player, kind), 0.);
            f.tiles[0].source.tile_index = 0x2c;
            assert_eq!(swimmer_depth(&f, &a, kind), 0., "land swimmer keeps full stature");
            f.tiles[0].source.tile_index = 0x14;
        }
        assert_eq!(swimmer_depth(&f, &a, CharacterKind::Trainer), 0.);
        assert_eq!(a.center, Vec2::new(0., 8.), "presentation does not move map position");
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
    fn world_pokemon_skins_share_assets_but_advance_independent_joints() {
        use bevy::mesh::skinning::SkinnedMesh;
        let mut app=actor_test_app();
        let mut first=actor();first.id=VisualActorId::Object(10);
        first.source_id="species:FURRET:icon_monster".into();
        let mut second=first.clone();second.id=VisualActorId::Object(11);second.center.x+=16.;
        app.world_mut().resource_mut::<VisualWorldFrame>().actors=vec![first.clone(),second.clone()];
        app.update();
        let state=app.world().resource::<ModeledActors>();
        let a=&state.props[&first.id];let b=&state.props[&second.id];
        let roots=[a.entity,b.entity];
        let a_skin=app.world().get::<SkinnedMesh>(a.entity).unwrap();
        let b_skin=app.world().get::<SkinnedMesh>(b.entity).unwrap();
        assert_eq!(a_skin.inverse_bindposes,b_skin.inverse_bindposes);
        assert_eq!(app.world().get::<Mesh3d>(a.entity).unwrap().0,app.world().get::<Mesh3d>(b.entity).unwrap().0);
        assert_eq!(a_skin.joints.len(),14);
        assert!(a_skin.joints.iter().all(|joint|!b_skin.joints.contains(joint)));
        let root_transform=*app.world().get::<Transform>(roots[0]).unwrap();
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(250));
        app.update();
        let state=app.world().resource::<ModeledActors>();
        let skin=state.props[&first.id].skin.as_ref().unwrap();
        assert!(skin.pose[4].rotation.angle_between(skin.rig.joints[4].bind.rotation)>0.001);
        let mut matrices=vec![Mat4::IDENTITY;skin.rig.joints.len()];
        skin.rig.skin_matrices_into(&skin.pose,&mut matrices).unwrap();
        for part in skin.rig.anatomy.iter().filter(|part|part.name.starts_with("Foot /")) {
            for vertex in part.vertices.clone() {
                assert!(skin.rig.skin_point(vertex,&matrices).distance(Vec3::from_array(skin.rig.neutral.positions[vertex]))<1e-6);
            }
        }
        assert_eq!(*app.world().get::<Transform>(roots[0]).unwrap(),root_transform);
        assert_eq!(app.world().resource::<VisualWorldFrame>().actors,vec![first,second]);
        assert_eq!(state.pokemon_binds.len(),1);
    }

    #[test]
    fn butterfly_npc_folds_wings_with_fixed_feet_and_complete_idle_bounds() {
        use crate::species_rig::{Clip,Playback,Species};
        let mut app=actor_test_app();
        let mut npc=actor();npc.id=VisualActorId::Object(24);
        npc.source_id="species:BUTTERFREE:icon_moth".into();
        let mut follower=npc.clone();follower.id=VisualActorId::PartyFollower;follower.center.x+=16.;
        app.world_mut().resource_mut::<VisualWorldFrame>().actors=vec![npc.clone(),follower.clone()];
        app.update();
        let state=app.world().resource::<ModeledActors>();
        let perched=state.props[&npc.id].skin.as_ref().unwrap();
        let flying=state.props[&follower.id].skin.as_ref().unwrap();
        assert_eq!(perched.joint_offsets.len(),4);assert!(flying.joint_offsets.is_empty());
        let rig=perched.rig;
        assert_eq!(rig.species,Species::Butterfree);
        assert_eq!(rig.neutral.positions,flying.rig.neutral.positions);
        assert_eq!(rig.neutral.normals,flying.rig.neutral.normals);
        assert_eq!(rig.neutral.indices,flying.rig.neutral.indices);
        assert_ne!(app.world().get::<Mesh3d>(state.props[&npc.id].entity).unwrap().0,
            app.world().get::<Mesh3d>(state.props[&follower.id].entity).unwrap().0);
        for part in rig.anatomy.iter().filter(|p|p.name.starts_with("Wing")) {
            let owner=rig.joint_indices[part.vertices.start][0];
            assert!([4,6,8,10].contains(&owner));
            for v in part.vertices.clone() {
                assert_eq!(rig.joint_indices[v],[owner,0,0,0]);
                assert_eq!(rig.joint_weights[v],[1.,0.,0.,0.]);
            }
        }
        let mut matrices=vec![Mat4::IDENTITY;rig.joints.len()];
        rig.skin_matrices_into(&perched.pose,&mut matrices).unwrap();
        let width=|matrices:&[Mat4]| {
            let xs=(0..rig.neutral.positions.len()).map(|v|rig.skin_point(v,matrices).x).collect::<Vec<_>>();
            xs.iter().copied().fold(f32::NEG_INFINITY,f32::max)-xs.iter().copied().fold(f32::INFINITY,f32::min)
        };
        let folded_width=width(&matrices);
        rig.skin_matrices_into(&flying.pose,&mut matrices).unwrap();
        assert!(folded_width<width(&matrices)*0.65);
        let entity=state.props[&npc.id].entity;
        let body_mesh=app.world().get::<Mesh3d>(entity).unwrap().0.clone();
        let wing_mesh=state.perched_wing_meshes[&(actor_props::PropKind::ExactSpecies("BUTTERFREE"),npc.model_palette)].clone();
        let assets=app.world().resource::<Assets<Mesh>>();
        let count=|handle:&Handle<Mesh>|assets.get(handle).unwrap().indices().unwrap().len();
        assert_eq!(count(&body_mesh)+count(&wing_mesh),rig.neutral.indices.len());
        let body_material=&app.world().get::<MeshMaterial3d<StandardMaterial>>(entity).unwrap().0;
        let materials=app.world().resource::<Assets<StandardMaterial>>();
        assert_eq!(materials.get(body_material).unwrap().diffuse_transmission,0.);
        assert_eq!(materials.get(state.perched_wing_material.as_ref().unwrap()).unwrap().diffuse_transmission,0.5);
        let wing_entity=app.world().get::<Children>(entity).unwrap().iter()
            .find(|&e|app.world().get::<Mesh3d>(e).is_some()).unwrap();
        let wing_skin=app.world().get::<bevy::mesh::skinning::SkinnedMesh>(wing_entity).unwrap();
        assert_eq!(wing_skin.joints,perched.joints);
        let root=*app.world().get::<Transform>(entity).unwrap();
        let height=rig.neutral_bounds.1.y-rig.neutral_bounds.0.y;
        assert!((root.scale.y*height-npc.size.y*14.0/16.0).abs()<0.00001);
        let follower_root=app.world().get::<Transform>(state.props[&follower.id].entity).unwrap();
        assert_eq!(follower_root.scale,Vec3::splat(app.world().resource::<VisualWorldFrame>().tile_size.y*2.0));
        let aabb=app.world().get::<bevy::camera::primitives::Aabb>(entity).unwrap();
        let lo=Vec3::from(aabb.center-aabb.half_extents);let hi=Vec3::from(aabb.center+aabb.half_extents);
        let offsets=perched.joint_offsets.clone();
        let supports=rig.anatomy.iter().filter(|p|p.name.starts_with("Foot /") || p.name.starts_with("Leg /"))
            .flat_map(|p|p.vertices.clone()).collect::<Vec<_>>();
        assert!(!supports.is_empty());
        let mut pose=rig.joints.iter().map(|j|j.bind).collect::<Vec<_>>();
        for frame in 0..=120 {
            let time=rig.clip(Clip::Idle).duration*(frame as f32+0.37)/121.;
            rig.sample_into(Clip::Idle,time,Playback::Clamp,&mut pose).unwrap();
            for &(joint,rotation) in &offsets {pose[joint].rotation=(rotation*pose[joint].rotation).normalize();}
            rig.skin_matrices_into(&pose,&mut matrices).unwrap();
            for v in 0..rig.neutral.positions.len() {
                let point=rig.skin_point(v,&matrices);
                assert!(point.cmpge(lo-Vec3::splat(0.00001)).all() && point.cmple(hi+Vec3::splat(0.00001)).all(),"folded idle outside culling bounds");
                assert!(point.y>=-0.000001);
            }
            for &v in &supports {assert!(rig.skin_point(v,&matrices).distance(Vec3::from_array(rig.neutral.positions[v]))<0.000001);}
        }
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(200));
        app.update();
        assert_eq!(*app.world().get::<Transform>(entity).unwrap(),root);
        assert_eq!(app.world().resource::<VisualWorldFrame>().actors,vec![npc,follower]);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.remove(0);
        app.update();
        assert!(app.world().get_entity(entity).is_err());
        assert!(app.world().get_entity(wing_entity).is_err());
    }

    #[test]
    fn world_pokemon_skin_replacement_and_retirement_remove_owned_joints() {
        let mut app=actor_test_app();let mut pokemon=actor();
        pokemon.id=VisualActorId::Object(10);pokemon.source_id="species:FURRET:icon_monster".into();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors=vec![pokemon];
        app.update();
        let state=app.world().resource::<ModeledActors>();
        let old_root=state.props[&VisualActorId::Object(10)].entity;
        let old_joints=state.props[&VisualActorId::Object(10)].skin.as_ref().unwrap().joints.clone();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id="species:NOCTOWL:icon_bird".into();
        app.update();
        assert!(app.world().get_entity(old_root).is_err());
        assert!(old_joints.iter().all(|&joint|app.world().get_entity(joint).is_err()));
        assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(10)].skin.as_ref().unwrap().rig.species,crate::species_rig::Species::Noctowl);
        app.world_mut().resource_mut::<VoxelViewStatus>().active=false;
        app.update();
        assert!(app.world().resource::<ModeledActors>().props.is_empty());
        assert_eq!(app.world_mut().query_filtered::<Entity,With<ModeledActor>>().iter(app.world()).count(),0);
    }

    #[test]
    fn world_pokemon_rig_aliases_preserve_exact_canonical_vertex_order() {
        for &kind in actor_props::ALL_KINDS {
            if let Some(rig)=world_pokemon_rig(kind) {
                assert_eq!(actor_props::mesh(kind).positions,rig.neutral.positions,"{}",kind.label());
            }
        }
        assert!(world_pokemon_rig(actor_props::PropKind::IconBird).is_none());
        assert!(world_pokemon_rig(actor_props::PropKind::IconFish).is_none());
    }
    #[test]
    fn uphill_hop_moves_the_character_root_and_keeps_its_shadow_grounded() {
        use crystal_render_api::{VisualPlayerTraversal, VisualTraversalStep};
        let mut app = actor_test_app();
        let from = Vec2::new(0.0, -4.0);
        let to = Vec2::new(0.0, 4.0);
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.grid_size = UVec2::new(1, 2);
            frame.viewport_size = Vec2::new(8.0, 16.0);
            frame.actors[0].center = from + Vec2::Y * 8.0;
        }
        app.world_mut().resource_mut::<TerrainRevisionCache>().footing_heights = vec![6.0, 0.0];
        app.insert_resource(VisualPlayerTraversal {
            map_id: "NewBarkTown".into(),
            step: Some(VisualTraversalStep { from, to, progress: 0.0, ledge_jump: false }),
        });
        for (progress, expected, ground) in [(0.0, 0.0, 0.0), (0.5, 9.0, 6.0), (1.0, 6.0, 6.0)] {
            app.world_mut().resource_mut::<VisualPlayerTraversal>().step.as_mut().unwrap().progress = progress;
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].center = from.lerp(to, progress) + Vec2::Y * 8.0;
            app.update();
            let world = app.world();
            let instance = &world.resource::<ModeledActors>().instances[&VisualActorId::Player];
            let root = world.get::<Transform>(instance.entity).unwrap();
            let shadow = world.get::<Transform>(instance.shadow).unwrap();
            assert!((root.translation.y - expected - 0.04).abs() < 0.001);
            let shadow_y = root.translation.y + shadow.translation.y * root.scale.y;
            assert!((shadow_y - ground - 0.04 + 0.001 * root.scale.y).abs() < 0.001);
            assert_eq!(root.translation.x, 0.0);
            assert_eq!(root.translation.z, -from.lerp(to, progress).y);
            if progress == 0.5 {
                let mut expected_pose = instance.person.rig.nodes.clone();
                instance.person.rig.sample("Jump", instance.person.rig.duration("Jump") * progress, &mut expected_pose);
                let mut descendants = vec![instance.entity];
                let mut checked_knee = false;
                while let Some(entity) = descendants.pop() {
                    if let Some(skin) = world.get::<bevy::mesh::skinning::SkinnedMesh>(entity) {
                        let knee = instance.person.rig.leg_joints[0][1];
                        let actual = world.get::<Transform>(skin.joints[knee]).unwrap().rotation;
                        let expected = expected_pose[instance.person.rig.skin_nodes[knee]].rotation;
                        assert!(actual.dot(expected).abs() > 0.999, "airborne knees use the authored Jump clip");
                        checked_knee = true;
                        break;
                    }
                    if let Some(children) = world.get::<Children>(entity) {
                        descendants.extend(children.iter());
                    }
                }
                assert!(checked_knee);
            }
            let frame = world.resource::<VisualWorldFrame>();
            let traversal = world.resource::<VisualPlayerTraversal>();
            for first_person in [false, true] {
                let mut camera = crate::camera::VoxelCameraControls::default();
                camera.first_person = first_person;
                let plain = camera.world_pose(frame, &[6.0, 0.0]);
                let animated = camera.world_pose_with_traversal(frame, &[6.0, 0.0], Some(traversal));
                let expected_delta = if first_person { expected - ground } else { 6.0 * progress - ground };
                assert!((animated.eye.y - plain.eye.y - expected_delta).abs() < 0.001);
            }
        }
    }

    #[test]
    #[ignore = "requires supplied mounted player rigs and paintings"]
    fn mounted_runtime_riders_and_bikes_share_turns_anchors_and_independent_phases() {
        let mut app = actor_test_app();
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.actors[0].source_id = "chris_bike".into();
            let mut other = frame.actors[0].clone();
            other.id = VisualActorId::Object(19);
            other.source_id = "kris_bike".into();
            other.center.x += 40.;
            other.facing = Some(Vec2::NEG_Y);
            frame.actors.push(other);
        }
        app.update();
        for step in 0..24 {
            app.world_mut().resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(1./60.));
            {
                let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
                frame.actors[0].center.x += 0.3;
                frame.actors[0].facing = Some(Vec2::NEG_Y);
            }
            app.update();
            let state = app.world().resource::<ModeledActors>();
            for id in [VisualActorId::Player,VisualActorId::Object(19)] {
                let rider = &state.instances[&id];
                let prop = &state.props[&id];
                let person = app.world().get::<Transform>(rider.entity).unwrap();
                let bike = app.world().get::<Transform>(prop.entity).unwrap();
                assert_eq!(person.translation,bike.translation,"mounted anchors diverged");
                assert!(person.rotation.abs_diff_eq(bike.rotation,1e-6),"bike snapped ahead of rider turn");
                assert_eq!(bike.scale,Vec3::splat(16.),"bike must retain independent authored scale");
                let children = app.world().get::<Children>(prop.entity).unwrap();
                assert_eq!(children.len(),3);
                let crank = app.world().get::<Transform>(children[0]).unwrap();
                assert!(crank.rotation.abs_diff_eq(Quat::from_rotation_x(-rider.motion.phase),1e-6),
                    "crank must use the same frame's rider phase");
            }
            assert_eq!(state.instances[&VisualActorId::Object(19)].motion.phase,0.);
            assert!(state.instances[&VisualActorId::Player].motion.phase>0.);
            if step==0 {
                let rider = &state.instances[&VisualActorId::Player];
                assert!((rider.motion.yaw-facing_yaw(Vec2::NEG_Y)).abs()>0.1,
                    "test must exercise an interpolated turn rather than its settled endpoint");
            }
        }
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
    #[ignore = "requires supplied character rigs and paintings"]
    fn map_replacement_retires_unused_character_uploads_and_keeps_the_player() {
        let mut app = actor_test_app();
        // Exercise real handle-drop processing rather than only the cache map.
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<Image>()
            .init_asset::<StandardMaterial>()
            .init_asset::<VoxelMaterial>()
            .init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>();
        let mut other = actor();
        other.id = VisualActorId::Object(19);
        other.source_id = "teacher".into();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.push(other);
        app.update();
        let state = app.world().resource::<ModeledActors>();
        assert_eq!(state.people_gpu.0.len(), 2);
        assert!(state.people_gpu.0.keys().any(|(_, kind)| *kind == CharacterKind::Teacher));
        let old_player = state.instances[&VisualActorId::Player].entity;
        let before_meshes = app.world().resource::<Assets<Mesh>>().len();
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 2);
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.map_id = "VioletCity".into();
            frame.actors.retain(|actor| actor.id == VisualActorId::Player);
        }
        let before = app.world().resource::<VisualWorldFrame>().clone();
        app.update();
        assert!(app.world().get_entity(old_player).is_err());
        let state = app.world().resource::<ModeledActors>();
        assert_eq!(state.people_gpu.0.len(), 1);
        assert!(state.people_gpu.0.keys().all(|(_, kind)| *kind == CharacterKind::Trainer));
        assert_eq!(state.instances.len(), 1);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(), before,
            "cache retirement cannot alter the authoritative presentation frame");
        for _ in 0..4 { app.update(); }
        assert_eq!(app.world().resource::<Assets<Image>>().len(), 1,
            "the old painting asset must be removed after its material handles drop");
        assert!(app.world().resource::<Assets<Mesh>>().len() < before_meshes,
            "retired character meshes must leave the asset collection too");
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
    fn larger_player_keeps_authoritative_foot_and_does_not_resize_matching_npcs() {
        let mut app=actor_test_app();
        let player=app.world().resource::<VisualWorldFrame>().actors[0].clone();
        let mut npc=player.clone();npc.id=VisualActorId::Object(99);npc.center.x+=16.;
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.push(npc.clone());
        app.update();
        let state=app.world().resource::<ModeledActors>();
        let p=app.world().get::<Transform>(state.instances[&player.id].entity).unwrap();
        let n=app.world().get::<Transform>(state.instances[&npc.id].entity).unwrap();
        assert!((p.scale.y/n.scale.y-PLAYER_PRESENTATION_SCALE).abs()<0.001);
        assert_eq!(p.translation,visual_point_to_voxel(actor_foot(&player),0.04));
        assert_eq!(n.translation,visual_point_to_voxel(actor_foot(&npc),0.04));
        assert_eq!(app.world().resource::<VisualWorldFrame>().actors[0],player);
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
    fn gold_gym_boulder_palette_moves_with_same_object_and_does_not_leak_maps() {
        let mut app = actor_test_app();
        let mut prop = actor();
        prop.id = VisualActorId::Object(19);
        prop.source_id = "boulder".into();
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.map_id = "CianwoodGym".into();
            frame.actors.push(prop);
        }
        app.update();
        let first = app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity;
        assert!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].gold_gym_stone);
        let count = app.world().resource::<Assets<Mesh>>().len();
        let position = app.world().get::<Transform>(first).unwrap().translation;
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].center.x += 8.;
        app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity, first);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), count);
        let moved = app.world().get::<Transform>(first).unwrap().translation;
        assert_eq!(moved.x - position.x, 8.);
        assert_eq!(moved.y, position.y);
        let gold_mesh = app.world().get::<Mesh3d>(first).unwrap().0.clone();
        let prop_mesh_count = app.world().resource::<ModeledActors>().prop_meshes.len();
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "BlackthornGym2F".into();
        app.update();
        let other_gold = &app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)];
        assert!(other_gold.gold_gym_stone);
        let blackthorn_entity = other_gold.entity;
        // A real map transition rebuilds entities, while preserving the
        // dedicated palette mesh cache shared by these two explicit maps.
        assert_ne!(blackthorn_entity, first);
        assert!(app.world().get_entity(first).is_err());
        assert_eq!(app.world().get::<Mesh3d>(blackthorn_entity).unwrap().0, gold_mesh);
        assert_eq!(app.world().resource::<ModeledActors>().prop_meshes.len(), prop_mesh_count);
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].center.x += 8.;
        app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)].entity, blackthorn_entity);
        let blackthorn_moved = app.world().get::<Transform>(blackthorn_entity).unwrap().translation;
        assert_eq!(blackthorn_moved.x - moved.x, 8.);
        assert_eq!(blackthorn_moved.y, moved.y);
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id = "PewterGym".into();
        app.update();
        let next = &app.world().resource::<ModeledActors>().props[&VisualActorId::Object(19)];
        assert!(!next.gold_gym_stone);
        assert_ne!(next.entity, blackthorn_entity);
        assert_eq!(app.world().get::<Transform>(next.entity).unwrap().translation, blackthorn_moved);
        assert!(app.world().get_entity(blackthorn_entity).is_err());
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
