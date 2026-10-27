//! Articulated characters consume the authoritative presentation frame only.
//! No input, collision, script, inventory, facing, or warp state is owned here.
#[path = "johto_actor_props.rs"]
pub(crate) mod actor_props;
#[path = "johto_characters.rs"]
pub(crate) mod character_meshes;
#[path = "surf_riding.rs"]
mod surf_riding;
#[path = "connected_actor_motion.rs"]
mod connected_motion;

use crate::{
    ActorIdCache, TerrainRevisionCache, VOXEL_RENDER_LAYER, VoxelActorCard, VoxelViewStatus,
    actor_foot, resolved_footing_height, visual_point_to_voxel, VoxelMaterial,
};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use character_meshes::*;
use crystal_render_api::{VisualActor, VisualActorId, VisualWorldFrame};
use crate::footing::actor_ground_point;
use std::collections::{HashMap, HashSet};
use std::f32::consts::{PI, TAU};

#[derive(Component)]
pub(crate) struct ModeledActor;

// Presentation scale belongs to the player identity, not its sprite source:
// NPCs using the same trainer art keep their authored stature.
const PLAYER_PRESENTATION_SCALE: f32 = 1.25;

pub(crate) fn character_scale(actor: &VisualActor, tile_height: f32) -> f32 {
    tile_height * 2.0 * if actor.id==VisualActorId::Player {PLAYER_PRESENTATION_SCALE} else {1.0}
}

fn cycle_distance(run: f32) -> f32 {
    crate::human_locomotion::cycle_distance(run)
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
        self.advance_with_stride(foot, facing, scale, dt, 1.0);
    }
    fn advance_with_stride(&mut self, foot: Vec2, facing: Option<Vec2>, scale: f32,
        dt: f32, stride: f32) {
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
            // Blend into a longer running cycle with a short planted stance
            // and aerial return. The same blend drives the visible skin.
            let target_run = if target_weight > 0.0 {
                ((self.speed - 1.6) / 2.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            self.run_weight += (target_run - self.run_weight) * (1.0 - (-8.0 * dt).exp());
            if moving {
                let inverse_cycle = (1.0 / cycle_distance(previous_run)
                    + 1.0 / cycle_distance(self.run_weight))
                    * 0.5 / stride;
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

/// Exercise the real distance publisher and posed sole contacts together. A smaller
/// stride without the matching cadence passes pose tests but slides on ground.
#[cfg(any(test, feature = "people-review"))]
pub(crate) fn validate_human_cadence() {
    use crate::human_locomotion::{HumanLocomotion, REFERENCE_HEIGHT, stride_scale};
    use character_meshes::SOURCE_KINDS;
    for kind in SOURCE_KINDS.iter().map(|&(_, kind)| kind).chain([CharacterKind::Outdoorsman]) {
        let look = crate::npc_appearance::look(kind);
        let rig = crate::open_people::get(look.base).expect("cadence check needs the supplied rig");
        let gait = HumanLocomotion::for_character(rig, kind);
        let mut pose = rig.nodes.clone();
        let scale = 16.0 * look.height / REFERENCE_HEIGHT;
        for speed in [1.5, 4.0] {
            let mut motion = Motion::new(Vec2::ZERO, Vec2::X);
            let mut baseline = motion;
            let mut position = Vec2::ZERO;
            let mut planted = [0; 2];
            for frame in 0..420 {
                let old_phase = motion.phase;
                let old_run = motion.run_weight;
                gait.sample(rig, old_phase, old_run, &mut pose);
                let before = std::array::from_fn::<_, 2, _>(|side|
                    gait.sole_contacts(rig, &pose, side));
                let distance = speed / 60.0 * scale;
                position.x += distance;
                motion.advance_with_stride(position, Some(Vec2::X), scale, 1.0 / 60.0, stride_scale(kind));
                baseline.advance(position, Some(Vec2::X), scale, 1.0 / 60.0);
                assert_eq!(motion.last_world_foot, position, "{kind:?} cadence moved the actor");
                assert_eq!(motion.run_weight, baseline.run_weight);
                assert_eq!(motion.walk_weight, baseline.walk_weight);
                assert_eq!(motion.yaw, baseline.yaw);
                if frame < 240 { continue; }
                gait.sample(rig, motion.phase, motion.run_weight, &mut pose);
                let stance = 0.54 - 0.32 * motion.run_weight;
                for side in 0..2 {
                    let a = (old_phase / TAU + side as f32 * 0.5).rem_euclid(1.0);
                    let b = (motion.phase / TAU + side as f32 * 0.5).rem_euclid(1.0);
                    if a < 0.02 || b < a || b > stance - 0.02 { continue; }
                    let after = gait.sole_contacts(rig, &pose, side);
                    // A rolling ankle travels around a stationary contact.
                    // Use the heel while either sample is heel-supported;
                    // otherwise use the toe (also planted during flat stance).
                    let heel = before[side][0].y < before[side][1].y - 1e-5
                        || after[0].y < after[1].y - 1e-5;
                    let contact = if heel { 0 } else { 1 };
                    let drift = (after[contact].z - before[side][contact].z) * REFERENCE_HEIGHT / rig.height
                        + distance / scale;
                    assert!(drift.abs() < 0.0002,
                        "{kind:?} speed={speed} planted sole {side} slides {drift} at frame {frame}");
                    assert!((after[contact].y - before[side][contact].y).abs() < 0.0002,
                        "{kind:?} planted sole lifts off the floor");
                    planted[side] += 1;
                }
            }
            assert!(planted.iter().all(|&count| count > 15), "{kind:?} missing stance samples {planted:?}");
        }
    }
}

struct Instance {
    entity: Entity,
    kind: CharacterKind,
    uniform: Option<[u8; 3]>,
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
    matrices: Vec<Mat4>,
    seconds: f32,
    motion: Motion,
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
        return Some(crate::species_rig::motion::world_rig(canonical));
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
    kind.pokemon_rig()
}

impl WorldPokemonSkin {
    fn spawn(
        commands: &mut Commands,
        entity: Entity,
        rig: &'static crate::species_rig::SpeciesRig,
        npc: bool,
        foot: Vec2,
        facing: Vec2,
        binds: &mut Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>,
        cache: &mut HashMap<(&'static str, usize), Handle<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
    ) -> Self {
        let inverse_bindposes = cache.entry((rig.species.name(),rig.joints.len())).or_insert_with(|| {
            binds.add(bevy::mesh::skinning::SkinnedMeshInverseBindposes::from(
                rig.joints.iter().map(|joint| joint.inverse_bind).collect::<Vec<_>>()))
        }).clone();
        let joint_offsets=world_pokemon_joint_offsets(rig,npc);
        let mut pose=rig.joints.iter().map(|joint|joint.bind).collect::<Vec<_>>();
        for &(joint,rotation) in &joint_offsets {pose[joint].rotation=(rotation*pose[joint].rotation).normalize();}
        let bounds=if npc && rig.species==crate::species_rig::Species::Butterfree {rig.animated_bounds}else{rig.locomotion_bounds()};
        let mut joints = Vec::with_capacity(rig.joints.len());
        for (index,joint) in rig.joints.iter().enumerate() {
            let child = commands.spawn((Visibility::default(), pose[index], ModeledActor)).id();
            commands.entity(joint.parent.map_or(entity, |parent| joints[parent])).add_child(child);
            joints.push(child);
        }
        commands.entity(entity).insert((
            bevy::mesh::skinning::SkinnedMesh { inverse_bindposes, joints: joints.clone() },
            bevy::camera::primitives::Aabb::from_min_max(bounds.0,bounds.1),
            bevy::camera::visibility::NoAutoAabb,
        ));
        Self { rig, joints, pose, matrices: vec![Mat4::IDENTITY; rig.joints.len()], seconds: 0., motion: Motion::new(foot,facing), joint_offsets }
    }

    fn advance(&mut self, delta: f32, foot: Vec2, facing: Option<Vec2>, scale: f32, swimming: bool, transforms: &mut Query<&mut Transform, With<ModeledActor>>) {
        use crate::species_rig::{Clip, Playback};
        if !delta.is_finite() || delta < 0. { return; }
        self.seconds = (self.seconds + delta).rem_euclid(120.);
        let distance=foot.distance(self.motion.last_world_foot);
        let phase=self.motion.phase;
        self.motion.advance(foot, facing, scale, delta);
        if distance.is_finite() && distance<=scale*1.5 {
            self.motion.phase=(phase+self.rig.locomotion_phase_delta(distance/scale,delta)).rem_euclid(TAU);
        }
        if swimming {
            self.rig.sample_swimming(self.seconds, self.motion.phase,
                self.motion.walk_weight, &mut self.pose);
        } else if self.joint_offsets.is_empty() {
            self.rig.sample_locomotion(self.seconds, self.motion.phase, self.motion.walk_weight, &mut self.pose);
        } else {
            self.rig.sample_into(Clip::Idle, self.seconds, Playback::Loop, &mut self.pose)
                .expect("validated world Pokemon idle sample");
        }
        for &(joint,rotation) in &self.joint_offsets {
            self.pose[joint].rotation=(rotation*self.pose[joint].rotation).normalize();
        }
        for (&entity, pose) in self.joints.iter().zip(&self.pose) {
            if let Ok(mut transform) = transforms.get_mut(entity) { transform.set_if_neq(*pose); }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CaveStoneFinish { Rock, Dark, Frozen, Native([u8;3]) }

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
    #[cfg(not(target_arch = "wasm32"))]
    destination_people: DestinationPeople,
    #[cfg(all(target_arch = "wasm32", feature = "actor-worker-browser"))]
    actor_warmup: crate::actor_worker::browser::Warmup,
    pub(crate) people_gpu: crate::open_people_skin::PeopleGpu,
    material: Option<Handle<StandardMaterial>>,
    remote_material: Option<Handle<StandardMaterial>>,
    perched_wing_material: Option<Handle<StandardMaterial>>,
    shadow_mesh: Option<Handle<Mesh>>,
    shadow_material: Option<Handle<StandardMaterial>>,
    instances: HashMap<VisualActorId, Instance>,
    map_id: String,
    motion_basis: Option<connected_motion::Basis>,
    gpu_map_id: String,
    seating_key: Option<crate::TerrainCacheKey>,
    seats: Vec<crate::seating::Seat>,
    seating_tiles: Vec<crystal_render_api::VisualTile>,
    prop_meshes: HashMap<(actor_props::PropKind, Option<(CaveStoneFinish, u8)>, bool, Option<crystal_render_api::VisualPokemonModelPalette>,bool), Handle<Mesh>>,
    perched_wing_meshes: HashMap<(actor_props::PropKind,Option<crystal_render_api::VisualPokemonModelPalette>),Handle<Mesh>>,
    stone_materials: HashMap<CaveStoneFinish, Handle<VoxelMaterial>>,
    foliage_material: Option<Handle<VoxelMaterial>>,
    props: HashMap<VisualActorId, PropInstance>,
    pokemon_binds: HashMap<(&'static str, usize), Handle<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
}

#[cfg(not(target_arch = "wasm32"))]
type ActorAssetInterest = (&'static str, Vec<&'static str>);
#[cfg(target_arch = "wasm32")]
impl ModeledActors {
    pub(crate) fn synchronous_battle_people(&self) -> bool {
        #[cfg(feature = "actor-worker-browser")]
        {
            self.actor_warmup.synchronous_fallback()
        }
        #[cfg(not(feature = "actor-worker-browser"))]
        {
            true
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct NativeAssetWarmup {
    interest: Vec<(&'static str, &'static str)>,
    appearance_interest: Vec<CharacterKind>,
    bootstrap: Option<bevy::tasks::Task<()>>,
    queued: std::collections::VecDeque<ActorAssetInterest>,
    tasks: Vec<bevy::tasks::Task<()>>,
    ready: bool,
}
#[cfg(not(target_arch = "wasm32"))]
impl NativeAssetWarmup {
    fn replace_interest(&mut self, interest: Vec<(&'static str, &'static str)>) {
        self.bootstrap = None;
        self.tasks.clear();
        self.queued.clear();
        self.ready = interest.is_empty();
        let mut groups = std::collections::BTreeMap::<_,Vec<_>>::new();
        for &(base, painting) in &interest {
            let paintings = groups.entry(base).or_default();
            if !paintings.contains(&painting) { paintings.push(painting); }
        }
        self.queued.extend(groups);
        self.interest = interest;
    }
    fn poll_and_dispatch(&mut self, pool: &bevy::tasks::TaskPool,
        mut spawn: impl FnMut(ActorAssetInterest) -> bevy::tasks::Task<()>) {
        use bevy::tasks::futures_lite::future;
        self.tasks.retain_mut(|task| future::block_on(future::poll_once(task)).is_none());
        let limit = pool.thread_num().max(1);
        while self.tasks.len() < limit {
            let Some(interest) = self.queued.pop_front() else { break; };
            self.tasks.push(spawn(interest));
        }
        self.ready = self.bootstrap.is_none() && self.queued.is_empty() && self.tasks.is_empty();
    }
    fn pending(&mut self, frame: &VisualWorldFrame, gpu: &mut crate::open_people_skin::PeopleGpu) -> bool {
        self.pending_with_prepared(frame, gpu, None)
    }
    fn pending_with_prepared(&mut self, frame: &VisualWorldFrame,
        gpu: &mut crate::open_people_skin::PeopleGpu,
        staged: Option<crate::open_people_skin::PreparedAppearances>) -> bool {
        self.pending_kinds(frame.actors.iter().filter_map(character_kind).collect(), gpu, staged)
    }
    fn pending_kinds(&mut self, mut kinds: Vec<CharacterKind>, gpu: &mut crate::open_people_skin::PeopleGpu,
        staged: Option<crate::open_people_skin::PreparedAppearances>) -> bool {
        use bevy::tasks::{AsyncComputeTaskPool, futures_lite::future};
        let Some(pool) = AsyncComputeTaskPool::try_get() else { return false; };
        kinds.sort_unstable_by_key(|kind| *kind as usize); kinds.dedup();
        let mut interest = kinds.iter().copied().map(crate::npc_appearance::look)
            .map(|look| (look.base,look.texture_key)).collect::<Vec<_>>();
        interest.sort_unstable(); interest.dedup();
        if self.interest != interest || self.appearance_interest != kinds {
            self.replace_interest(interest);
            self.appearance_interest = kinds;
            // Retiring the app-owned staging map also isolates results from a
            // canceled synchronous job that finishes after interest changes.
            gpu.2 = staged.unwrap_or_default();
            // Complete staged appearances already own the refined body and
            // validated painting. Do not restart preparation at the doorway.
            if self.appearance_interest.iter().all(|&kind| {
                let key = (crate::npc_appearance::look(kind).base, kind);
                gpu.0.contains_key(&key) || gpu.2.lock().expect("prepared actor lock").contains_key(&key)
            }) {
                self.queued.clear();
                self.ready = true;
                return false;
            }
            if let Some(&(base,_)) = self.interest.first() {
                // get() initializes the shared complete rig catalog once. Finish
                // that single cold parse before other workers refine different
                // bases, rather than occupying every worker on its OnceLock.
                self.bootstrap = Some(pool.spawn(async move {
                    let _trace = crate::loading_trace::span("region.prepare_actor_catalog", ||
                        serde_json::json!({"base":base}));
                    crate::open_people::get(base).expect("required production character rig");
                }));
            }
        }
        if let Some(task) = &mut self.bootstrap {
            if future::block_on(future::poll_once(task)).is_none() { return true; }
            self.bootstrap = None;
        }
        if !self.ready {
            let prepared = gpu.2.clone();
            let missing = self.appearance_interest.iter().copied().filter(|kind| {
                let look = crate::npc_appearance::look(*kind);
                !gpu.0.contains_key(&(look.base,*kind))
                    && !gpu.2.lock().expect("prepared actor lock").contains_key(&(look.base,*kind))
            }).collect::<Vec<_>>();
            self.poll_and_dispatch(pool, |(base,paintings)| {
                let prepared = prepared.clone();
                let kinds = missing.iter().copied().filter(|kind|
                    crate::npc_appearance::look(*kind).base == base).collect::<Vec<_>>();
                pool.spawn(async move {
                let _trace = crate::loading_trace::span("region.prepare_actor_assets", ||
                    serde_json::json!({"base":base,"paintings":paintings.len()}));
                let rig = crate::open_people::get(base).expect("required production character rig");
                let _ = rig.refined_body();
                for painting in paintings {
                    crate::npc_textures::get(painting).expect("required production character painting");
                }
                // Same-binary diagnostic control; production prepares every
                // missing appearance before the main-thread publication step.
                let prepare_looks = !std::env::var("CRYSTAL_ACTOR_APPEARANCE_WARMUP")
                    .is_ok_and(|value| matches!(value.as_str(), "0" | "off" | "false"));
                for kind in kinds.into_iter().filter(|_| prepare_looks) {
                    let appearance = crate::open_people_skin::prepare_appearance(&rig, kind);
                    prepared.lock().expect("prepared actor lock").insert((base,kind),appearance);
                }
            })});
        }
        !self.ready
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct DestinationPeople {
    map_id: String,
    sources: Vec<std::sync::Arc<str>>,
    warmup: NativeAssetWarmup,
    gpu: crate::open_people_skin::PeopleGpu,
    uploaded: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl DestinationPeople {
    fn adopt_lookahead(&self, kinds:&[CharacterKind], live:&mut crate::open_people_skin::PeopleGpu)
        -> Option<crate::open_people_skin::PreparedAppearances> {
        if self.map_id.is_empty() {return None;}
        // Appearance keys include the exact base and costume; their immutable
        // assets are independent of map placement. Borrow only needed roles,
        // including completed parts of an in-flight exterior preparation.
        live.adopt_uploaded_kinds(&self.gpu,kinds);
        Some(self.gpu.2.clone())
    }
    fn promote_uploaded(&mut self, previous: &str, next: &str,
        live: &mut crate::open_people_skin::PeopleGpu) {
        if previous.is_empty() || previous == next || self.map_id != next || !self.uploaded {
            return;
        }
        // Exchange the two bounded asset sets just like terrain rooms. The
        // retiring room keeps its original GPU handles for the return door.
        std::mem::swap(live, &mut self.gpu);
        self.map_id = previous.to_owned();
        self.sources.clear();
        self.warmup = NativeAssetWarmup::default();
        self.uploaded = false;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prepare_destination_people(
    request: Res<crate::DestinationPreload>, frame: Res<VisualWorldFrame>,
    status: Res<VoxelViewStatus>, settings: Res<crate::VoxelViewSettings>,
    mut state: ResMut<ModeledActors>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
) {
    let Some(target) = request.frame.as_ref() else { return; };
    if !settings.enabled || !frame.active || !status.active || status.terrain_pending
        || target.map_id == frame.map_id { return; }
    let ModeledActors { people_gpu, destination_people, .. } = &mut *state;
    let changed_map = destination_people.map_id != target.map_id.as_ref();
    if changed_map {
        *destination_people = DestinationPeople { map_id: target.map_id.to_string(), ..default() };
    }
    if destination_people.uploaded && destination_people.sources == request.actor_sources { return; }
    if destination_people.sources != request.actor_sources {
        destination_people.uploaded = false;
    }
    destination_people.sources.clone_from(&request.actor_sources);
    let kinds = request.actor_sources.iter().filter_map(|source|
        character_meshes::kind_for_source(source)).collect::<Vec<_>>();
    if changed_map {
        // Borrow only common roles needed by the destination. Other actors
        // from the current map must still retire when that map is left.
        destination_people.gpu.0 = people_gpu.0.iter()
            .filter(|((_, kind), _)| kinds.contains(kind))
            .map(|(key, gpu)| (*key, gpu.clone())).collect();
    }
    destination_people.gpu.retain_kinds(&kinds.iter().copied().collect());
    if !destination_people.warmup.pending_kinds(kinds.clone(), &mut destination_people.gpu, None) {
        destination_people.uploaded = destination_people.gpu.upload_next_prepared(&kinds,
            &mut meshes, &mut materials, &mut images, &mut binds);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod asset_warmup_tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicBool,AtomicUsize,Ordering}};
    #[test]
    fn prepared_destination_appearance_is_adopted_without_foreground_jobs() {
        bevy::tasks::AsyncComputeTaskPool::get_or_init(||
            bevy::tasks::TaskPoolBuilder::new().num_threads(2).build());
        let mut background = NativeAssetWarmup::default();
        let mut destination = crate::open_people_skin::PeopleGpu::default();
        let kinds = vec![CharacterKind::Elm, CharacterKind::Scientist];
        wait_until(|| !background.pending_kinds(kinds.clone(), &mut destination, None));
        assert!(destination.0.is_empty(), "speculation prepares CPU appearances without spawning people");
        let prepared = destination.2.clone();
        for &kind in &kinds {
            let key = (crate::npc_appearance::look(kind).base, kind);
            assert!(prepared.lock().unwrap().contains_key(&key));
        }
        let mut foreground = NativeAssetWarmup::default();
        let mut live = crate::open_people_skin::PeopleGpu::default();
        assert!(!foreground.pending_kinds(kinds, &mut live, Some(prepared.clone())),
            "a complete destination must not re-enter the loading hold");
        assert!(foreground.bootstrap.is_none() && foreground.queued.is_empty() && foreground.tasks.is_empty());
        assert!(Arc::ptr_eq(&prepared, &live.2), "consume the original prepared meshes");
    }

    #[test]
    fn intermediate_room_adopts_only_needed_uploaded_appearances_without_jobs() {
        bevy::tasks::AsyncComputeTaskPool::get_or_init(||
            bevy::tasks::TaskPoolBuilder::new().num_threads(2).build());
        let mut meshes=Assets::<Mesh>::default();
        let mut materials=Assets::<StandardMaterial>::default();
        let mut images=Assets::<Image>::default();
        let mut binds=Assets::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>::default();
        let kinds=vec![CharacterKind::Mom,CharacterKind::Fisher];
        let mut destination=DestinationPeople {map_id:"NewBarkTown".into(),..default()};
        wait_until(||!destination.warmup.pending_kinds(kinds.clone(),&mut destination.gpu,None));
        while !destination.gpu.upload_next_prepared(&kinds,&mut meshes,&mut materials,&mut images,&mut binds) {}
        let counts=(meshes.len(),materials.len(),images.len(),binds.len());
        let mut live=crate::open_people_skin::PeopleGpu::default();
        destination.promote_uploaded("PlayersHouse2F","PlayersHouse1F",&mut live);
        assert!(live.0.is_empty(),"an unrelated destination cannot replace the active asset set");
        let staged=destination.adopt_lookahead(&[CharacterKind::Mom],&mut live);
        assert_eq!(destination.map_id,"NewBarkTown");
        assert_eq!(destination.gpu.0.len(),2,"keep the exterior preparation resident");
        assert_eq!(live.0.len(),1,"do not import the unrelated outdoor NPC");
        assert!(live.0.keys().all(|(_,kind)|*kind==CharacterKind::Mom));
        let mut foreground=NativeAssetWarmup::default();
        assert!(!foreground.pending_kinds(vec![CharacterKind::Mom],&mut live,staged));
        assert!(foreground.bootstrap.is_none()&&foreground.tasks.is_empty()&&foreground.queued.is_empty());
        assert_eq!(counts,(meshes.len(),materials.len(),images.len(),binds.len()));
    }

    #[test]
    fn uploaded_room_characters_survive_a_round_trip_without_jobs_or_asset_growth() {
        bevy::tasks::AsyncComputeTaskPool::get_or_init(||
            bevy::tasks::TaskPoolBuilder::new().num_threads(2).build());
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let mut images = Assets::<Image>::default();
        let mut binds = Assets::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>::default();
        let house = vec![CharacterKind::Trainer, CharacterKind::Mom];
        let town = vec![CharacterKind::Trainer, CharacterKind::Fisher];
        let mut live = crate::open_people_skin::PeopleGpu::default();
        let mut initial = NativeAssetWarmup::default();
        wait_until(|| !initial.pending_kinds(house.clone(), &mut live, None));
        while !live.upload_next_prepared(&house, &mut meshes, &mut materials, &mut images, &mut binds) {}
        let mut destination = DestinationPeople { map_id: "NewBarkTown".into(), ..default() };
        destination.gpu.0 = live.0.iter().filter(|((_, kind), _)| town.contains(kind))
            .map(|(key, value)| (*key, value.clone())).collect();
        wait_until(|| !destination.warmup.pending_kinds(town.clone(), &mut destination.gpu, None));
        // An unfinished preload must not displace the room's live people.
        destination.promote_uploaded("PlayersHouse1F", "NewBarkTown", &mut live);
        assert!(live.0.keys().any(|(_, kind)| *kind == CharacterKind::Mom));
        while !destination.gpu.upload_next_prepared(&town,
            &mut meshes, &mut materials, &mut images, &mut binds) {}
        destination.uploaded = true;
        let counts = (meshes.len(), materials.len(), images.len(), binds.len());
        destination.promote_uploaded("PlayersHouse1F", "NewBarkTown", &mut live);
        assert_eq!(destination.map_id, "PlayersHouse1F");
        assert!(live.0.keys().any(|(_, kind)| *kind == CharacterKind::Fisher));
        assert!(destination.gpu.0.keys().any(|(_, kind)| *kind == CharacterKind::Mom));
        assert_eq!((live.0.len(), destination.gpu.0.len()), (2, 2));
        assert!(!destination.warmup.pending_kinds(house.clone(), &mut destination.gpu, None));
        assert!(destination.warmup.tasks.is_empty() && destination.warmup.bootstrap.is_none()
            && destination.warmup.queued.is_empty(), "the return doorway must reuse uploaded people");
        destination.uploaded = destination.gpu.upload_next_prepared(&house,
            &mut meshes, &mut materials, &mut images, &mut binds);
        assert!(destination.uploaded);
        destination.promote_uploaded("NewBarkTown", "PlayersHouse1F", &mut live);
        assert_eq!(destination.map_id, "NewBarkTown");
        assert!(live.0.keys().any(|(_, kind)| *kind == CharacterKind::Mom));
        assert_eq!((live.0.len(), destination.gpu.0.len()), (2, 2));
        assert_eq!(counts, (meshes.len(), materials.len(), images.len(), binds.len()));
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(5);
        while !condition() {
            assert!(std::time::Instant::now()<deadline,"asset worker did not progress");
            std::thread::yield_now();
        }
    }
    #[test]
    fn native_actor_jobs_group_bases_bound_workers_and_finish_complete_interest() {
        let pool=bevy::tasks::TaskPoolBuilder::new().num_threads(2).build();
        let mut warmup=NativeAssetWarmup::default();
        warmup.replace_interest(vec![("a","red"),("a","blue"),("a","red"),
            ("b","green"),("c","gold"),("d","silver")]);
        assert_eq!(warmup.queued.len(),4);
        assert_eq!(warmup.queued[0],("a",vec!["red","blue"]));
        let gate=Arc::new(AtomicBool::new(false));
        let started=Arc::new(AtomicUsize::new(0));
        let finished=Arc::new(std::sync::Mutex::new(Vec::new()));
        let spawn=|work:ActorAssetInterest| {
            let gate=gate.clone();let started=started.clone();let finished=finished.clone();
            pool.spawn(async move {
                started.fetch_add(1,Ordering::SeqCst);
                while !gate.load(Ordering::SeqCst) {bevy::tasks::futures_lite::future::yield_now().await;}
                finished.lock().unwrap().push(work);
            })
        };
        warmup.poll_and_dispatch(&pool,spawn);
        wait_until(||started.load(Ordering::SeqCst)==2);
        assert_eq!(warmup.tasks.len(),pool.thread_num());
        assert_eq!(warmup.queued.len(),2);
        assert!(!warmup.ready);
        gate.store(true,Ordering::SeqCst);
        wait_until(|| {warmup.poll_and_dispatch(&pool,spawn);warmup.ready});
        let mut actual=finished.lock().unwrap().clone();actual.sort_unstable();
        assert_eq!(actual,vec![("a",vec!["red","blue"]),("b",vec!["green"]),
            ("c",vec!["gold"]),("d",vec!["silver"])]);
        assert!(warmup.tasks.is_empty() && warmup.queued.is_empty());
    }
    #[test]
    fn changing_actor_interest_cancels_old_jobs_and_empty_interest_is_ready() {
        let pool=bevy::tasks::TaskPoolBuilder::new().num_threads(1).build();
        let mut warmup=NativeAssetWarmup::default();
        warmup.replace_interest(vec![("old","old-paint"),("queued","unused")]);
        struct Retired(Arc<AtomicBool>);
        impl Drop for Retired {fn drop(&mut self) {self.0.store(true,Ordering::SeqCst);}}
        let started=Arc::new(AtomicBool::new(false));
        let retired=Arc::new(AtomicBool::new(false));
        warmup.poll_and_dispatch(&pool, |_| {
            let started=started.clone();let retired=retired.clone();
            pool.spawn(async move {
                let _retired=Retired(retired);
                started.store(true,Ordering::SeqCst);
                bevy::tasks::futures_lite::future::pending::<()>().await;
            })
        });
        wait_until(||started.load(Ordering::SeqCst));
        warmup.replace_interest(vec![("new","new-paint")]);
        assert!(warmup.tasks.is_empty());
        assert_eq!(warmup.queued[0],("new",vec!["new-paint"]));
        let completed=Arc::new(AtomicBool::new(false));
        let spawn=|work:ActorAssetInterest| {let completed=completed.clone();pool.spawn(async move {
            assert_eq!(work,("new",vec!["new-paint"]));completed.store(true,Ordering::SeqCst);
        })};
        wait_until(|| {warmup.poll_and_dispatch(&pool,spawn);warmup.ready});
        assert!(completed.load(Ordering::SeqCst));
        wait_until(||retired.load(Ordering::SeqCst));
        warmup.replace_interest(Vec::new());
        assert!(warmup.ready && warmup.tasks.is_empty() && warmup.queued.is_empty());
    }
}

fn enabled_map(map: &str) -> bool {
    // Actor assets are selected by the resolved presentation source. The map
    // name cannot change a nurse into a scientist or suppress an indoor rig.
    !map.is_empty()
}

/// Source Surf sprites remain 2D witnesses, never alternative 3D mounts.
pub(crate) fn is_surf_avatar(actor: &VisualActor) -> bool {
    actor.id == VisualActorId::Player && matches!(actor.source_id.as_ref(),
        "surf" | "surfing_pikachu" | "surf_pikachu"
        | "gfx/sprites/surf.png" | "gfx/sprites/surfing_pikachu.png")
}

/// The silhouette pass must use the same appearance test as the actual rig.
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
        * if actor.id == VisualActorId::Player { PLAYER_PRESENTATION_SCALE } else { 1.0 }
}

/// Visible stature above the source foot, including the rig's swimmer pose.
pub(crate) fn modeled_character_height(frame: &VisualWorldFrame, actor: &VisualActor) -> Option<f32> {
    let kind=character_kind(actor)?;
    let height=character_scale(actor,frame.tile_size.y)*person_height(kind)
        -swimmer_depth(frame,actor,kind);
    (height.is_finite()&&height>0.).then_some(height)
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
    let Some(cell) = crate::tile_at_visual_point(frame, actor_ground_point(frame, actor) + Vec2::Y * 0.01)
    else { return 0.; };
    if !frame.tiles.iter().any(|tile| tile.column == cell.x && tile.row == cell.y
        && crate::shape_for_source(&tile.source) == crate::CellShape::Water)
    { return 0.; }
    character_scale(actor, frame.tile_size.y) * person_height(kind) * 0.68
}

pub(crate) fn npc_texture_names() -> Vec<&'static str> {
    SOURCE_KINDS.iter().map(|&(name, _)| name).chain(std::iter::once("outdoorsman")).collect()
}

#[cfg(feature = "actor-worker")]
pub(crate) fn worker_kind_for_source(source: &str) -> Option<CharacterKind> {
    character_meshes::kind_for_source(source)
}
#[cfg(feature = "actor-worker")]
pub(crate) fn worker_interests(frame: &VisualWorldFrame) -> Vec<crate::actor_worker::Key> {
    let mut keys = frame.actors.iter().filter(|actor| character_kind(actor).is_some())
        .filter_map(|actor| crate::actor_worker::key_for_source(&actor.source_id).ok()).collect::<Vec<_>>();
    keys.sort_unstable_by_key(|key| key.source);
    keys.dedup();
    keys
}

pub(crate) fn is_human(actor: &VisualActor) -> bool {
    character_kind(actor).is_some()
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

fn model_transform(frame: &VisualWorldFrame, actor: &VisualActor, height: f32, yaw: f32, scale: f32) -> Transform {
    // Source OAM bottoms are tile edges, not the ground root of a 3D body.
    Transform::from_translation(visual_point_to_voxel(actor_ground_point(frame, actor), height + 0.04))
        .with_rotation(Quat::from_rotation_y(yaw))
        .with_scale(Vec3::splat(scale))
}

fn prop_foot(frame: &VisualWorldFrame, actor: &VisualActor, kind: actor_props::PropKind) -> Vec2 {
    // The ball occupies the middle eight pixels of its sixteen-pixel sprite.
    // Its lower edge sits inside the gameplay tile, unlike a person's feet.
    if kind == actor_props::PropKind::PokeBall {
        actor.center - Vec2::new(0.0, actor.size.y * 0.25)
    } else if kind.pokemon_model().is_some() || kind.is_bicycle() {
        actor_ground_point(frame, actor)
    } else {
        actor_foot(actor)
    }
}

fn world_foot(frame: &VisualWorldFrame, actor: &VisualActor) -> Vec2 {
    actor_ground_point(frame, actor) - frame.center
        + Vec2::new(
            frame.grid_origin.x as f32 * frame.tile_size.x,
            -frame.grid_origin.y as f32 * frame.tile_size.y,
        )
}

fn furniture_seat_target(frame: &VisualWorldFrame, seat: crate::seating::Seat,
    actor: &VisualActor, ground: f32) -> Option<Vec3> {
    let foot = actor_ground_point(frame, actor);
    if let Some(target) = seat.target(frame, foot, ground) { return Some(target); }
    // Authored NPCs sit on their actual tile; front-approach presentation is
    // reserved for players, so dialogue scripts cannot visually relocate Mom.
    if !matches!(actor.id, VisualActorId::Player | VisualActorId::RemotePlayer(_)) { return None; }
    let facing = actor.facing.filter(|f| f.is_finite() && f.length_squared() > 0.5)?;
    let ahead = foot + facing.normalize() * frame.tile_size * 2.;
    let radius = frame.tile_size.min_element() * 0.5;
    if frame.actors.iter().any(|other| other.id != actor.id
        && matches!(other.id, VisualActorId::Object(_) | VisualActorId::Player | VisualActorId::RemotePlayer(_))
        && actor_ground_point(frame, other).distance_squared(ahead) < radius * radius) { return None; }
    seat.actor_target(frame, foot, actor.facing, ground)
}

fn pokemon_world_scale(frame: &VisualWorldFrame, rig: &crate::species_rig::SpeciesRig,
    dimensions: Option<&crystal_render_api::VisualPokemonDimensions>) -> Option<f32> {
    let species = if rig.species.name().starts_with("unown_") { "UNOWN".into() }
        else { rig.species.name().to_ascii_uppercase() };
    let meters = *dimensions?.meters.get(&species)?;
    // The travelling Furret's long body is measured nose to tail. Reposing
    // onto four paws must not turn its listed length into shoulder height.
    let reference = crate::battle_layout::world_reference_span(&species,rig);
    (meters.is_finite() && meters > 0. && reference > 0.).then_some(
        frame.tile_size.y * 2. * meters / reference)
}

/// Use actual published water, never map names or a sprite's shared icon.
/// Raised support (a deck/bridge/table) keeps its ordinary land presentation.
fn pokemon_swims_here(frame: &VisualWorldFrame, actor: &VisualActor,
    rig: &crate::species_rig::SpeciesRig, support: f32) -> bool {
    if !support.is_finite() || actor.source_id.starts_with("display:")
        || !crate::species_rig::motion::swims_in_water(rig.species.name()) { return false; }
    let Some(cell) = crate::tile_at_visual_point(frame,
        actor_ground_point(frame, actor) + Vec2::Y * 0.01) else { return false; };
    frame.tiles.iter().any(|tile| tile.column == cell.x && tile.row == cell.y
        && crate::shape_for_source(&tile.source) == crate::CellShape::Water)
        && (support <= 0.01
            || support <= crate::mesh::native_water_support_offset(frame, cell) + 0.01)
}

fn surf_mount_seat(frame: &VisualWorldFrame,
    dimensions: Option<&crystal_render_api::VisualPokemonDimensions>) -> Option<(surf_riding::SurfSeat, f32)> {
    let mount = frame.actors.iter().find(|a| a.id == VisualActorId::SurfMount)?;
    let rig = world_pokemon_actor_rig(prop_kind(mount)?, mount.id)?;
    Some((surf_riding::SurfSeat::for_rig(rig), pokemon_world_scale(frame, rig, dimensions)?))
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

impl ModeledActors {
    fn refresh_seats(&mut self, frame: &VisualWorldFrame, profiles_revision: u64,
        profiles: Option<&crate::live_profiles::Document>) -> bool {
        let _trace = crate::loading_trace::span("actor.seating", || serde_json::json!({
            "map": frame.map_id.as_ref(), "tiles": frame.tiles.len(),
        }));
        let mut key = crate::TerrainCacheKey::from_frame(frame);
        key.profiles_revision = profiles_revision;
        if self.seating_key.as_ref() == Some(&key) { return false; }
        // The broad terrain key is a cheap invalidation hint. Seat resolution
        // reads map, grid, scale, profile document and the complete source tiles;
        // crop origin/viewport/ledges alone cannot alter local seat rectangles.
        // Retain every tile field, including texture-family and priority evidence.
        let same_inputs = self.seating_key.as_ref().is_some_and(|previous|
            previous.map_id == key.map_id && previous.grid_size == key.grid_size
                && previous.tile_bits == key.tile_bits
                && previous.profiles_revision == key.profiles_revision)
            && self.seating_tiles == frame.tiles;
        self.seating_key = Some(key);
        if same_inputs { return false; }
        let _resolve = crate::loading_trace::span("actor.seating.resolve", || serde_json::json!({
            "map": frame.map_id.as_ref(), "tiles": frame.tiles.len(),
        }));
        self.seats = crate::mesh::seating_surfaces(frame, profiles);
        self.seating_tiles.clone_from(&frame.tiles);
        true
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sync(
    mut commands: Commands,
    frame: Res<VisualWorldFrame>,
    #[cfg(any(not(target_arch = "wasm32"), feature = "actor-worker-browser"))]
    mut status: ResMut<VoxelViewStatus>,
    #[cfg(all(target_arch = "wasm32", not(feature = "actor-worker-browser")))]
    status: Res<VoxelViewStatus>,
    settings: Option<Res<crate::VoxelViewSettings>>,
    presentation: (
        Option<Res<crystal_render_api::VisualSurfRider>>,
        Option<Res<crystal_render_api::VisualPlayerTraversal>>,
        Option<Res<crystal_render_api::VisualPokemonDimensions>>,
        Option<Res<crate::DestinationPreload>>,
        Option<Res<crystal_render_api::VisualBattleFrame>>,
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
    let (surf_rider, traversal, dimensions, _destination, battle) = presentation;
    let battle = battle.as_deref().filter(|battle| battle.active);
    let battle_kinds = battle
        .into_iter()
        .flat_map(|battle| battle.trainers.iter().flatten())
        .filter_map(|trainer| crate::battle_view::trainers::kind(&trainer.source_id))
        .collect::<Vec<_>>();
    let needed_kinds = || {
        frame
            .actors
            .iter()
            .filter_map(character_kind)
            .chain(battle_kinds.iter().copied())
            .collect::<Vec<_>>()
    };
    let player_visibility = if settings.as_ref().is_some_and(|s| s.camera.first_person) {
        Visibility::Hidden
    } else { Visibility::Inherited };
    #[cfg(all(target_arch = "wasm32", feature = "actor-worker-browser"))]
    {
        let requested = ((frame.active && enabled_map(&frame.map_id)) || battle.is_some())
            && settings.as_ref().is_none_or(|settings| settings.enabled);
        let mut current = worker_interests(&frame);
        for &kind in &battle_kinds {
            let source = crate::npc_appearance::look(kind).texture_key;
            if let Ok(key) = crate::actor_worker::key_for_source(source)
                && !current.contains(&key) { current.push(key); }
        }
        let mut interests = current.clone();
        // Share the single worker queue with the approaching destination.
        // Current roles stay first; preparing future roles must never freeze
        // already-uploaded actors on the live map.
        if requested && status.active && !status.profiles_pending {
            if let Some(destination) = _destination.as_deref().filter(|request|
                request.frame.as_ref().is_some_and(|target| target.map_id != frame.map_id)) {
                for source in &destination.actor_sources {
                    if let Ok(key) = crate::actor_worker::key_for_source(source)
                        && !interests.contains(&key) { interests.push(key); }
                }
            }
        }
        let ModeledActors { actor_warmup, people_gpu, .. } = &mut *state;
        let kinds = interests.iter().map(|key| key.kind).collect::<Vec<_>>();
        if requested {
            people_gpu.retain_kinds(&kinds.iter().copied().collect());
        }
        let waiting = actor_warmup.pending(interests, requested, people_gpu);
        // Match native publication: a completed worker batch must not upload
        // every new NPC body and painting on the same interactive frame.
        let uploaded_before = people_gpu.0.len();
        let upload_pending = requested && actor_warmup.prepared_uploads_enabled()
            && !people_gpu.upload_next_prepared(
                &needed_kinds(),
                &mut meshes, &mut materials, &mut images, &mut binds);
        let foreground_ready = current.iter().all(|key| people_gpu.0.contains_key(&(key.base, key.kind)));
        if requested && foreground_ready && people_gpu.0.len() == uploaded_before
            && actor_warmup.prepared_uploads_enabled() {
            // At most one new appearance is uploaded per frame, including
            // lookahead. Common roles retain exactly their existing handles.
            people_gpu.upload_next_prepared(&kinds,
                &mut meshes, &mut materials, &mut images, &mut binds);
        }
        if ((waiting && !foreground_ready) || upload_pending) && status.active {
            status.terrain_pending = true;
            status.active_frames = 0;
            return;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if ((frame.active && enabled_map(&frame.map_id)) || battle.is_some())
        && settings.as_ref().is_none_or(|settings| settings.enabled)
        && {
            let ModeledActors { asset_warmup, people_gpu, destination_people, gpu_map_id, .. } = &mut *state;
            destination_people.promote_uploaded(gpu_map_id, &frame.map_id, people_gpu);
            let kinds=needed_kinds();
            let staged=destination_people.adopt_lookahead(&kinds,people_gpu);
            asset_warmup.pending_kinds(kinds, people_gpu, staged)
        } && status.active {
        status.terrain_pending = true;
        status.active_frames = 0;
        return;
    }
    // Background body preparation may advance for the destination, but its
    // instances must stay with the retained room until terrain publication.
    if status.scene_held || (status.active && crate::retain_lighthouse_room(&frame, &footing)) {
        return;
    }
    let enabled = frame.active && status.active && enabled_map(&frame.map_id);
    connected_motion::handoff(&mut state, &frame, enabled, &mut commands);
    if !enabled {
        return;
    }
    if state.gpu_map_id != frame.map_id.as_ref() {
        // Retain common roles (especially the player) across the transition,
        // but release uploaded bodies and paintings belonging only to old maps.
        // Entity handles keep the retiring frame alive until deferred despawns.
        #[cfg(not(all(target_arch = "wasm32", feature = "actor-worker-browser")))]
        {
            let needed = needed_kinds().into_iter().collect();
            state.people_gpu.retain_kinds(&needed);
        }
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
    let mount_seat = surf_mount_seat(&frame, dimensions.as_deref());
    let surf_mounted = mount_seat.is_some();
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
            cave_stone_finish(&frame).or_else(|| {
                let exterior=crate::cave_water::outdoor(&frame.map_id)
                    && frame.tiles.first().is_some_and(|tile|matches!(tile.source.tileset_id.as_ref(),
                        "johto"|"johto_modern"|"kanto"));
                exterior.then(||images.get(&actor.texture).and_then(actor_props::native_stone_color))
                    .flatten().map(CaveStoneFinish::Native)
            }).map(|finish| {
                let variant = if matches!(finish,CaveStoneFinish::Native(_)) {0}
                    else {match actor.id { VisualActorId::Object(id) => (id%4) as u8, _=>0 }};
                (finish, variant)
            })
        } else { None };
        // Fruit trees belong to the palette-preserving terrain lighting path,
        // just like the surrounding groves. Character PBR double-shades leaves.
        let terrain_material = if kind == actor_props::PropKind::FruitTree {
            Some(footing.solid_material.clone().unwrap_or_else(||
                state.foliage_material.get_or_insert_with(|| {
                    let mut material = crate::solid_terrain_material();
                    material.extension.cave_reveal.z = 1.;
                    stone_materials.add(material)
                }).clone()))
        } else { stone.map(|(finish,_)| {
            // Native pigment lives on the shared mesh; all native palette
            // variants reuse one neutral grain texture and material.
            let material_finish=if matches!(finish,CaveStoneFinish::Native(_)) {
                CaveStoneFinish::Native([0;3])
            } else {finish};
            state.stone_materials.entry(material_finish).or_insert_with(|| {
                let texture = images.add(crate::painted_stone::movable_bedrock_image(
                    matches!(finish,CaveStoneFinish::Frozen|CaveStoneFinish::Native(_))));
                // The rock mesh already contains authored face shading. Use
                // the same real-light energy response as the cave terrain,
                // rather than adding a second PBR shade that blacks out faces.
                stone_materials.add(crate::voxel_material(StandardMaterial { base_color: Color::WHITE, base_color_texture: Some(texture),
                    perceptual_roughness: 0.94, reflectance: 0.10, ..default() }))
            }).clone()
        }) };
        // Flat stationary decorations use the center of their source drawing
        // only when a complete, actually modeled table supplies their support.
        let tabletop=if (kind==actor_props::PropKind::Pokedex
                ||kind==actor_props::PropKind::PokeBall&&frame.map_id.as_ref()=="ElmsLab"
                ||kind.display_height().is_some()&&frame.map_id.as_ref()=="PlayersHouse2F")
            && matches!(actor.id,VisualActorId::Object(_)) {
            footing.built_frame.as_ref().filter(|built|
                built.map_id==frame.map_id&&built.tile_size==frame.tile_size)
                .and_then(|built|crate::mesh::traditional_tabletop_height(built,
                    footing.built_profiles.as_deref(),actor.center))
        } else {None};
        let render_foot=if tabletop.is_some() {actor.center}else{prop_foot(&frame,actor,kind)};
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
        let prop_scale = if let Some(rig) = rig {
            let Some(physical) = pokemon_world_scale(&frame, rig, dimensions.as_deref()) else {
                if let Some(instance) = state.props.remove(&actor.id) {
                    commands.entity(instance.entity).despawn();
                }
                continue;
            };
            physical
        } else if kind == actor_props::PropKind::PokeBall { scale * 0.5 }
        else { scale * kind.display_scale() };

        let water_actor = rig.is_some_and(|rig| pokemon_swims_here(&frame, actor, rig, height));
        let swimming = actor.id == VisualActorId::SurfMount || water_actor;
        let immersion = if actor.id == VisualActorId::SurfMount {
            mount_seat.map_or(0., |(seat, scale)| seat.waterline * scale)
        } else if water_actor {
            surf_riding::SurfSeat::for_rig(rig.expect("living aquatic rig")).waterline * prop_scale
        } else { 0. };
        let transform = Transform::from_translation(visual_point_to_voxel(render_foot, height + 0.04 - immersion))
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
                // Remove camera scrolling before measuring actual travelled distance.
                let scroll=frame.center-Vec2::new(frame.grid_origin.x as f32*frame.tile_size.x,
                    -frame.grid_origin.y as f32*frame.tile_size.y);
                skin.advance(time.delta_secs(), render_foot-scroll, actor.facing, prop_scale,
                    swimming, &mut transforms);
            }
        } else {
            let key=(kind,stone,gold_gym_stone,actor.model_palette,perched);
            if let Some(rig) = rig {
                let _trace = crate::loading_trace::span("pokemon.world_placement", || {
                    let cell = crate::tile_at_visual_point(&frame,
                        actor_ground_point(&frame, actor) + Vec2::Y * 0.01);
                    let source = cell.and_then(|cell| frame.tiles.iter()
                        .find(|tile| tile.column == cell.x && tile.row == cell.y));
                    serde_json::json!({"species":rig.species.name(),
                        "source":actor.source_id.as_ref(),"support":height,
                        "swimming":swimming,"immersion":immersion,"scale":prop_scale,
                        "native_water_support":cell.map(|cell|crate::mesh::native_water_support_offset(&frame,cell)),
                        "waterline":surf_riding::SurfSeat::for_rig(rig).waterline,
                        "tile":source.map(|tile|format!("{:?}",tile.source)),
                        "shape":source.map(|tile|format!("{:?}",crate::shape_for_source(&tile.source)))})
                });
            }
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
                } else if let Some((CaveStoneFinish::Native(rgb),_))=stone {
                    actor_props::native_stone_mesh(kind,rgb).into_mesh()
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
                    // World rigs may add flight membranes while preserving
                    // the canonical body. Mesh and skin must share that rig.
                    let data = rig.map_or_else(|| actor_props::mesh(kind), |rig| rig.neutral.clone());
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
            let entity = if let Some(material) = terrain_material {
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
            // Match painted people: keep directional contour lighting and
            // cast grounding shadows, but do not turn a small Pokemon into a
            // silhouette inside building shadows or its own shadow texels.
            if kind.pokemon_model().is_some() {
                commands.entity(entity).insert(NotShadowReceiver);
            }
            let pedals = kind.is_bicycle()
                .then(|| crate::bicycle::spawn_pedals(&mut commands, &mut meshes,
                    material.clone(), entity, 0.));
            if actor.id == VisualActorId::Player {
                commands.entity(entity).insert(player_visibility);
            }
            let skin = rig.map(|rig| WorldPokemonSkin::spawn(&mut commands, entity, rig,
                matches!(actor.id,VisualActorId::Object(_)),
                render_foot-(frame.center-Vec2::new(frame.grid_origin.x as f32*frame.tile_size.x,
                    -frame.grid_origin.y as f32*frame.tile_size.y)), facing,
                &mut binds, &mut state.pokemon_binds));
            if perched {
                let skin=skin.as_ref().expect("perched body skin");
                let wing=commands.spawn((Mesh3d(state.perched_wing_meshes[&(kind,actor.model_palette)].clone()),
                    MeshMaterial3d(state.perched_wing_material.as_ref().unwrap().clone()),
                    bevy::mesh::skinning::SkinnedMesh {
                        inverse_bindposes:state.pokemon_binds[&(skin.rig.species.name(),skin.rig.joints.len())].clone(),joints:skin.joints.clone()
                    },bevy::camera::primitives::Aabb::from_min_max(skin.rig.animated_bounds.0,skin.rig.animated_bounds.1),
                    bevy::camera::visibility::NoAutoAabb,
                    Transform::IDENTITY,Visibility::default(),ModeledActor,NotShadowReceiver,RenderLayers::layer(VOXEL_RENDER_LAYER))).id();
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
    state.refresh_seats(&frame, footing.key.as_ref().map_or(0, |key| key.profiles_revision),
        footing.built_profiles.as_deref());
    for actor in &frame.actors {
        let mounted = actor.id == VisualActorId::Player && surf_mounted;
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
        let surf_seat = (actor.id == VisualActorId::Player).then_some(mount_seat).flatten();
        let height = height + surf_seat.map_or(0., |(seat, scale)| seat.point.y * scale) - swimmer_depth(&frame, actor, kind);
        let world_foot = world_foot(&frame, actor);
        let scale = character_scale(actor,frame.tile_size.y);
        if scale <= 0.0 || !scale.is_finite() || !world_foot.is_finite() {
            continue;
        }
        let uniform = crate::npc_uniform::source_color(kind, images.get(&actor.texture));
        if state
            .instances
            .get(&actor.id)
            .is_some_and(|instance| instance.kind != kind || instance.uniform != uniform)
        {
            let old = state.instances.remove(&actor.id).unwrap();
            commands.entity(old.entity).despawn();
        }
        let bicycle = prop_kind(actor).is_some_and(actor_props::PropKind::is_bicycle);
        if !state.instances.contains_key(&actor.id) {
            let _trace = crate::loading_trace::span("human.world_placement", || {
                let cell = crate::tile_at_visual_point(&frame,
                    actor_ground_point(&frame, actor) + Vec2::Y * 0.01);
                let tile = cell.and_then(|cell| frame.tiles.iter()
                    .find(|t| t.column == cell.x && t.row == cell.y));
                serde_json::json!({"map":frame.map_id.as_ref(),"id":format!("{:?}",actor.id),
                    "source":actor.source_id.as_ref(),"point":actor_ground_point(&frame,actor).to_array(),
                    "support":height + swimmer_depth(&frame,actor,kind),
                    "root":height,"immersion":swimmer_depth(&frame,actor,kind),"scale":scale,
                    "cell":cell.map(|c|c.to_array()),"tile":tile.map(|t|format!("{:?}",t.source)),
                    "shape":tile.map(|t|format!("{:?}",crate::shape_for_source(&t.source)))})
            });
        }
        let seat = if surf_seat.is_some() || bicycle { None }
            else { state.seats.iter().find_map(|seat| furniture_seat_target(&frame, *seat, actor, height)
                .map(|offset| (offset, seat.height, seat.yaw))) };
        if let Some(instance) = state.instances.get_mut(&actor.id) {
            instance
                .motion
                .advance_with_stride(world_foot, actor.facing,
                    scale * if bicycle || surf_seat.is_some() { 1.0 }
                    else { person_height(kind) / crate::human_locomotion::REFERENCE_HEIGHT },
                    time.delta_secs(), if bicycle || surf_seat.is_some() { 1.0 }
                    else { crate::human_locomotion::stride_scale(kind) });
            {
                let person = &mut instance.person;
                person.run_blend = instance.motion.run_weight;
                person.bicycle = bicycle;
                person.surfing = surf_seat.is_some();
                person.surf_half_width = surf_seat.map_or(0.25, |(seat, mount_scale)|
                    seat.half_width * mount_scale / (frame.tile_size.y * 2. * PLAYER_PRESENTATION_SCALE));
                commands.entity(instance.shadow).insert(if person.surfing {
                    Visibility::Hidden
                } else { Visibility::Inherited });
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
                    let mut next = model_transform(
                        &frame,
                        actor,
                        height,
                        instance.motion.yaw,
                        scale * person_height(kind) / person.rig.height,
                    );
                    next.scale.x *= crate::npc_appearance::look(kind).width;
                    next.translation += person.sitting.offset * person.sitting.eased_weight();
                    if let Some(yaw) = person.sitting.yaw {
                        next.rotation = next.rotation.slerp(Quat::from_rotation_y(yaw), person.sitting.eased_weight());
                    }
                    transform.set_if_neq(next);
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
                let ground = resolved_footing_height(&frame, actor_ground_point(&frame, actor), &footing.footing_heights)
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
                                &frame,
                                actor,
                                height,
                                motion.yaw,
                                scale * person_height(kind) / imported.height,
                            ),),
                        ModeledActor,
                    ))
                    .id();
                commands.entity(entity).insert(model_transform(&frame, actor, height, motion.yaw,
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
                    bicycle,
                    uniform,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut images,
                    &mut binds,
                );
                person.surfing = surf_seat.is_some();
                person.surf_half_width = surf_seat.map_or(0.25, |(seat, mount_scale)|
                    seat.half_width * mount_scale / (frame.tile_size.y * 2. * PLAYER_PRESENTATION_SCALE));
                let shadow = commands
                    .spawn((
                        (Mesh3d(state.shadow_mesh.as_ref().unwrap().clone()),
MeshMaterial3d(state.shadow_material.as_ref().unwrap().clone()),
Transform::from_xyz(0.0,
                                if hop_weight > 0.0 {
                                    (resolved_footing_height(&frame, actor_ground_point(&frame, actor), &footing.footing_heights)
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
                        uniform,
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
    // Use the same animated skin surface and smoothed turn as the mount.
    // Following only its neutral height leaves the rider floating or clipping
    // when the Pokémon breathes, swims or changes direction.
    let ModeledActors { props, instances, .. } = &mut *state;
    if let (Some((seat, _)), Some(mount), Some(rider)) = (mount_seat,
        props.get_mut(&VisualActorId::SurfMount), instances.get(&VisualActorId::Player)) {
        if let Ok(rider_root) = transforms.get(rider.entity) {
            let rotation = rider_root.rotation;
            let contact = mount.skin.as_mut().map(|skin| {
                skin.rig.skin_matrices_into(&skin.pose, &mut skin.matrices)
                    .expect("validated Surf mount skin");
                seat.animated_point(skin.rig, &skin.matrices)
            }).unwrap_or(seat.point);
            let position = if let Ok(mut mount_root) = transforms.get_mut(mount.entity) {
                mount_root.rotation = rotation;
                Some(mount_root.transform_point(contact))
            } else { None };
            if let Some(position) = position
                && let Ok(mut rider_root) = transforms.get_mut(rider.entity) {
                rider_root.translation = position;
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
    include!("new_bark_actors_connection_tests.rs");
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
    fn seating_bits(seats: &[crate::seating::Seat]) -> Vec<([u32;4],u32,Option<u32>)> {
        seats.iter().map(|seat| (seat.rect.map(f32::to_bits),seat.height.to_bits(),
            seat.yaw.map(f32::to_bits))).collect()
    }
    #[test]
    fn seating_refresh_preserves_source_drawings_scale_profiles_and_crop_only_reuse() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        let document: crate::live_profiles::Document = serde_json::from_str(
            include_str!("../../../modpacks/voxel-view/profiles.json")).unwrap();
        let stool=document.objects.iter().find(|object|object.name=="Shared lab round stool").unwrap();
        let mut furniture=VisualWorldFrame::default();
        furniture.map_id="ElmsLab".into();
        furniture.grid_size=UVec2::new(stool.tiles[0].len() as u32+2,stool.tiles.len() as u32+2);
        furniture.tile_size=Vec2::splat(8.);
        for row in 0..furniture.grid_size.y { for column in 0..furniture.grid_size.x {
            let mut source=VisualTileSource { tileset_id:stool.tileset.as_str().into(),
                metatile_id:0,subtile_column:(column%4) as u8,subtile_row:(row%4) as u8,
                tile_index:stool.ground };
            if (row as usize)<stool.tiles.len() && (column as usize)<stool.tiles[0].len() {
                let y=row as usize;let x=column as usize;
                source.metatile_id=stool.metatiles.as_ref().map_or(stool.metatile,|blocks|
                    blocks[(usize::from(stool.origin[1])+y)/4][(usize::from(stool.origin[0])+x)/4]);
                source.subtile_column=((usize::from(stool.origin[0])+x)%4) as u8;
                source.subtile_row=((usize::from(stool.origin[1])+y)%4) as u8;
                source.tile_index=stool.tiles[y][x];
            }
            furniture.tiles.push(VisualTile {column,row,source,texture:Handle::default(),
                animation_frames:None,priority:false});
        }}
        let mut park=VisualWorldFrame::default();
        park.map_id="NationalPark".into();park.grid_size=UVec2::new(4,4);
        park.tile_size=Vec2::splat(8.);
        for row in 0..4 { for column in 0..4 {
            park.tiles.push(VisualTile {column,row,texture:Handle::default(),priority:false,
                animation_frames:None,source:VisualTileSource {tileset_id:"park".into(),
                    metatile_id:if row<3 {0x0e}else{0x01},subtile_column:column as u8,
                    subtile_row:row as u8,tile_index:if row<3 {0x07+row as u16*0x10+column as u16}else{0}}});
        }}
        for mut frame in [furniture,park] {
            let mut state=ModeledActors::default();
            assert!(state.refresh_seats(&frame,1,None));
            assert_eq!(state.seats.len(),1);
            let original=seating_bits(&state.seats);
            if frame.map_id.as_ref()=="NationalPark" {
                let bench=crate::exterior_models::model(crate::exterior_models::Kind::ParkBench);
                let cushion=crate::seating::cushion_height(&bench.surface.positions,&bench.surface.indices)
                    .unwrap_or(bench.max[1]);
                let expected=(cushion-bench.min[1])*1.5*frame.tile_size.y/(bench.max[1]-bench.min[1]);
                assert_eq!(state.seats[0].height.to_bits(),expected.to_bits());
                assert_eq!(state.seats[0].rect,[0.,0.,4.,2.]);
            }
            assert!(!state.refresh_seats(&frame,1,None));
            frame.center+=Vec2::new(15.,-7.);
            frame.grid_origin+=IVec2::new(2,1);
            frame.viewport_size+=Vec2::splat(20.);
            frame.terrain_revision+=1;
            assert!(!state.refresh_seats(&frame,1,None));
            assert_eq!(seating_bits(&state.seats),original);
            // Evidence other than labels is retained; an altered drawing must
            // invalidate the complete authored chair/bench exactly as before.
            frame.tiles[0].source.tile_index^=0x80;frame.terrain_revision+=1;
            assert!(state.refresh_seats(&frame,1,None));
            assert!(state.seats.is_empty());
            assert_eq!(seating_bits(&state.seats),seating_bits(&crate::mesh::seating_surfaces(&frame,None)));
            frame.tiles[0].source.tile_index^=0x80;frame.terrain_revision+=1;
            assert!(state.refresh_seats(&frame,1,None));
            assert_eq!(seating_bits(&state.seats),original);
            frame.tile_size*=2.;
            assert!(state.refresh_seats(&frame,1,None));
            assert_eq!(seating_bits(&state.seats),seating_bits(&crate::mesh::seating_surfaces(&frame,None)));
            assert!(state.refresh_seats(&frame,2,Some(&document)));
            assert_eq!(seating_bits(&state.seats),seating_bits(&crate::mesh::seating_surfaces(&frame,Some(&document))));
            frame.tiles[0].priority=true;frame.terrain_revision+=1;
            assert!(state.refresh_seats(&frame,2,Some(&document)));
            assert_eq!(seating_bits(&state.seats),seating_bits(&crate::mesh::seating_surfaces(&frame,Some(&document))));
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
        assert_eq!(furniture_seat_target(&frame, seat, &player, 0.), Some(Vec3::new(0., 5., -16.)));
        let mut npc = player.clone();
        npc.id = VisualActorId::Object(7);
        assert!(furniture_seat_target(&frame, seat, &npc, 0.).is_none(), "NPC must retain its authored tile");
        npc.center.y += 16.;
        assert_eq!(furniture_seat_target(&frame, seat, &npc, 0.), Some(Vec3::new(0., 5., 0.)));
        frame.actors = vec![player.clone(), npc];
        assert!(furniture_seat_target(&frame, seat, &player, 0.).is_none(), "occupied seat must not be claimed");
        frame.actors[1].source_id = "growlithe".into();
        assert!(furniture_seat_target(&frame, seat, &player, 0.).is_none(), "animal occupants also keep their seat");
    }

    #[test]
    fn swimmers_show_head_and_shoulders_above_authenticated_water_only() {
        let mut a = actor();
        a.id = VisualActorId::Object(7);
        a.center = Vec2::ZERO;
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
            a.source_id=if kind==CharacterKind::SwimmerGuy {"swimmer_guy"}else{"swimmer_girl"}.into();
            let total = character_scale(&a, f.tile_size.y) * person_height(kind);
            let above = total - swimmer_depth(&f, &a, kind);
            assert!(above > total * 0.25 && above < total * 0.40,
                "only the upper head/shoulder portion should clear water");
            assert_eq!(modeled_character_height(&f,&a),Some(above),
                "occluder interest follows the visible head and shoulders");
            let mut player = a.clone(); player.id = VisualActorId::Player;
            assert_eq!(swimmer_depth(&f, &player, kind), 0.);
            f.tiles[0].source.tile_index = 0x2c;
            assert_eq!(swimmer_depth(&f, &a, kind), 0., "land swimmer keeps full stature");
            assert_eq!(modeled_character_height(&f,&a),Some(total));
            f.tiles[0].source.tile_index = 0x14;
        }
        assert_eq!(swimmer_depth(&f, &a, CharacterKind::Trainer), 0.);
        assert_eq!(a.center, Vec2::ZERO, "presentation does not move map position");
    }

    fn actor_test_app() -> App {
        fn wait_for_actor_assets(
            frame: Res<VisualWorldFrame>, status: Res<VoxelViewStatus>,
            mut state: ResMut<ModeledActors>,
            mut dimensions: ResMut<crystal_render_api::VisualPokemonDimensions>,
        ) {
            if !frame.active || !status.active || !enabled_map(&frame.map_id) { return; }
            // Synthetic source-art fixtures use one model unit per metre.
            // Production supplies these measurements from its loaded pack.
            for actor in &frame.actors {
                if let Some(rig) = prop_kind(actor).and_then(world_pokemon_rig) {
                    let species = rig.species.name().to_ascii_uppercase();
                    dimensions.meters.entry(species.clone()).or_insert_with(||
                        crate::battle_layout::reference_span(&species, rig.neutral_bounds.0, rig.neutral_bounds.1));
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
                let ModeledActors { asset_warmup, people_gpu, .. } = &mut *state;
                // Another test may initialize the global task pool. Exercise the
                // actual production preparation/publication path in either case,
                // and wait for its result before asserting against this frame.
                while asset_warmup.pending(&frame, people_gpu) {
                    assert!(std::time::Instant::now() < deadline, "actor assets did not settle");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
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
            .init_resource::<crystal_render_api::VisualPokemonDimensions>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>()
            .add_systems(Update, (wait_for_actor_assets, sync).chain());
        app
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn battle_people_prepare_after_the_field_publication_is_cleared() {
        let mut app = actor_test_app();
        app.add_plugins(MinimalPlugins);
        *app.world_mut().resource_mut::<VisualWorldFrame>() = VisualWorldFrame::default();
        app.world_mut().resource_mut::<VoxelViewStatus>().active = false;
        app.insert_resource(crate::VoxelViewSettings { enabled: true, ..default() });
        app.insert_resource(crystal_render_api::VisualBattleFrame {
            active: true, map_id: "VioletGym".into(),
            trainers: [Some(crystal_render_api::VisualBattleTrainer {
                source_id: "falkner".into(), visible: false,
            }), None], ..default()
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        loop {
            app.update();
            let state = app.world().resource::<ModeledActors>();
            let key = (person_name(CharacterKind::Falkner), CharacterKind::Falkner);
            if state.people_gpu.0.contains_key(&key)
                || state.people_gpu.2.lock().expect("prepared appearances").contains_key(&key) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "battle trainer preparation never started");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(!app.world().resource::<VisualWorldFrame>().active);
        assert!(app.world().resource::<ModeledActors>().instances.is_empty());
    }

    #[test]
    #[ignore = "requires supplied character rigs and paintings"]
    fn link_uniform_changes_refresh_only_the_affected_actor_and_preserve_the_frame() {
        use bevy::{asset::RenderAssetUsages,render::render_resource::{Extent3d,TextureDimension,TextureFormat}};
        for source in ["link_receptionist","lass","youngster","cooltrainer_m","super_nerd","officer","kimono_girl"] {
        let mut app=actor_test_app();let mut colors=Vec::new();
        for c in [[80,192,48,255],[48,104,224,255]] {
            colors.push(app.world_mut().resource_mut::<Assets<Image>>().add(Image::new_fill(
                Extent3d{width:16,height:16,depth_or_array_layers:1},TextureDimension::D2,
                &c,TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::all())));
        }
        let mut receptionist=actor();receptionist.id=VisualActorId::Object(7);
        receptionist.source_id=source.into();receptionist.texture=colors[0].clone();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.push(receptionist);
        let before=app.world().resource::<VisualWorldFrame>().clone();app.update();
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),before);
        let state=app.world().resource::<ModeledActors>();
        let player=state.instances[&VisualActorId::Player].entity;
        let first=state.instances[&VisualActorId::Object(7)].entity;
        assert_eq!(state.instances[&VisualActorId::Object(7)].uniform,Some([80,192,48]));
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].texture=colors[1].clone();
        let before=app.world().resource::<VisualWorldFrame>().clone();app.update();
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),before);
        let state=app.world().resource::<ModeledActors>();
        let second=state.instances[&VisualActorId::Object(7)].entity;
        assert_ne!(first,second);assert_eq!(state.instances[&VisualActorId::Player].entity,player);
        assert_eq!(state.instances[&VisualActorId::Object(7)].uniform,Some([48,104,224]));
        assert!(app.world().get_entity(first).is_err());app.update();
        assert_eq!(app.world().resource::<ModeledActors>().instances[&VisualActorId::Object(7)].entity,second,
            "ordinary animation frames must not rebuild the same uniform");
        }
    }

    #[test]
    fn aquatic_presentation_requires_real_water_and_unraised_support() {
        let mut a = actor(); a.id = VisualActorId::Object(7); a.center = Vec2::ZERO;
        a.source_id = "species:MAGIKARP:icon_monster".into();
        let mut frame = VisualWorldFrame::default();
        frame.grid_size = UVec2::ONE; frame.viewport_size = Vec2::splat(8.); frame.tile_size = Vec2::splat(8.);
        frame.tiles.push(crystal_render_api::VisualTile {
            column:0,row:0,texture:Handle::default(),animation_frames:None,priority:false,
            source:crystal_render_api::VisualTileSource { tileset_id:"kanto".into(),metatile_id:0x43,
                subtile_column:0,subtile_row:0,tile_index:0x14 },
        });
        for name in ["MAGIKARP", "LAPRAS", "TOTODILE", "TENTACOOL", "MANTINE", "VAPOREON"] {
            a.source_id=format!("species:{name}:icon_monster").into();
            let rig=crate::species_rig::for_species(name).unwrap();
            assert!(pokemon_swims_here(&frame,&a,rig,0.), "{name}");
            assert!(!pokemon_swims_here(&frame,&a,rig,2.), "{name} sank through a bridge");
        }
        for name in ["PIDGEOT", "GASTLY", "KOFFING", "PIKACHU"] {
            a.source_id=format!("species:{name}:icon_monster").into();
            assert!(!pokemon_swims_here(&frame,&a,crate::species_rig::for_species(name).unwrap(),0.),
                "{name} must not be forced to swim while above water");
        }
        // Native standing-water bodies can share an elevated terrace datum.
        // An additional bridge/deck above that same water must remain land.
        a.source_id = "species:GYARADOS:icon_gyarados".into();
        let aquatic = crate::species_rig::for_species("GYARADOS").unwrap();
        frame.terrace_datum = Some(std::sync::Arc::new(crystal_render_api::VisualTerraceDatum {walkable_mineral:Default::default(),
            origin: IVec2::ZERO, size: UVec2::ONE, levels: vec![[1.;4]].into(),
            water_barriers: vec![].into(), open_water: None,
        }));
        let water = crate::profile::JUMP_LEDGE_HEIGHT;
        assert!(pokemon_swims_here(&frame,&a,aquatic,water), "elevated lake was treated as land");
        assert!(!pokemon_swims_here(&frame,&a,aquatic,water+2.), "deck above an elevated lake sank");
        frame.terrace_datum = None;
        let rig=crate::species_rig::for_species("MAGIKARP").unwrap();
        frame.tiles[0].source.tile_index=0x2c;
        assert!(!pokemon_swims_here(&frame,&a,rig,0.));
        frame.tiles[0].source.tile_index=0x14;
        a.source_id="display:MAGIKARP:icon_monster".into();
        assert!(!pokemon_swims_here(&frame,&a,rig,0.));
        a.source_id="species:MAGIKARP:icon_monster".into();
        a.center=Vec2::splat(100.);
        assert!(!pokemon_swims_here(&frame,&a,rig,0.), "unknown terrain is not water");
    }

    #[test]
    #[ignore = "requires supplied actor assets"]
    fn ledyba_world_actor_publishes_its_flight_membranes_with_matching_skin() {
        let mut app = actor_test_app();
        {
            let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.actors[0].id = VisualActorId::Object(7);
            frame.actors[0].source_id = "species:LEDYBA:icon_monster".into();
        }
        for _ in 0..3 { app.update(); }
        let state = app.world().resource::<ModeledActors>();
        let instance = &state.props[&VisualActorId::Object(7)];
        let skin = instance.skin.as_ref().unwrap();
        let canonical = crate::species_rig::for_species("LEDYBA").unwrap();
        assert!(skin.rig.neutral.positions.len() > canonical.neutral.positions.len());
        let mesh = app.world().get::<Mesh3d>(instance.entity).unwrap();
        let mesh = app.world().resource::<Assets<Mesh>>().get(&mesh.0).unwrap();
        assert_eq!(mesh.count_vertices(), skin.rig.neutral.positions.len());
        let Some(bevy::mesh::VertexAttributeValues::Uint16x4(joints)) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX) else { panic!("flight mesh lacks skin"); };
        for part in skin.rig.anatomy.iter().filter(|part| part.name.starts_with("Wing /")) {
            assert!(part.vertices.clone().all(|v| joints[v][0] as usize >= canonical.joints.len()));
        }
    }

    #[test]
    #[ignore = "requires supplied actor assets"]
    fn magikarp_switches_between_land_flop_and_water_swim_without_moving_gameplay() {
        let mut app=actor_test_app();
        {
            let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.grid_size=UVec2::ONE;frame.viewport_size=Vec2::splat(8.);
            frame.actors[0].id=VisualActorId::Object(7);frame.actors[0].center=Vec2::ZERO;
            frame.actors[0].source_id="species:MAGIKARP:icon_monster".into();
            frame.tiles.push(crystal_render_api::VisualTile {column:0,row:0,texture:Handle::default(),
                animation_frames:None,priority:false,source:crystal_render_api::VisualTileSource {
                    tileset_id:"kanto".into(),metatile_id:0x43,subtile_column:0,subtile_row:0,tile_index:0x2c }});
        }
        app.world_mut().resource_mut::<TerrainRevisionCache>().footing_heights=vec![0.];
        for water in [false,true,false] {
            app.world_mut().resource_mut::<VisualWorldFrame>().tiles[0].source.tile_index=if water {0x14}else{0x2c};
            let before=app.world().resource::<VisualWorldFrame>().clone();
            for _ in 0..3 {
                app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(1./60.));
                app.update();
            }
            let state=app.world().resource::<ModeledActors>();
            let instance=&state.props[&VisualActorId::Object(7)];let skin=instance.skin.as_ref().unwrap();
            let root=app.world().get::<Transform>(instance.entity).unwrap();
            let immersion=if water {surf_riding::SurfSeat::for_rig(skin.rig).waterline*root.scale.y}else{0.};
            assert!((root.translation.y-(0.04-immersion)).abs()<0.0001);
            let side_tilt=(skin.pose[0].rotation*Vec3::Y).x.abs();
            if water {assert!(side_tilt<0.01,"fish kept flopping in water");}
            else {assert!(side_tilt>0.5,"fish floated above land");}
            assert_eq!(*app.world().resource::<VisualWorldFrame>(),before,
                "presentation changed authoritative actors or terrain");
        }
    }

    #[test]
    #[ignore = "requires supplied rider rigs and paintings"]
    fn surf_riders_stay_on_the_animated_mount_through_turns_and_dismount() {
        for female in [false, true] {
            let mut app = actor_test_app();
            app.insert_resource(crystal_render_api::VisualSurfRider { female });
            // Feet/inches from the same source format used by the pack.
            for (species, inches) in [("GOLDEEN", 24.), ("LAPRAS", 98.),
                ("TOTODILE", 24.), ("PIKACHU", 16.), ("GYARADOS", 256.)] {
                app.world_mut().resource_mut::<crystal_render_api::VisualPokemonDimensions>()
                    .meters.insert(species.into(), inches * 0.0254);
            }
            for species in ["LAPRAS", "TOTODILE", "PIKACHU", "GYARADOS", "GOLDEEN"] {
                {
                    let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
                    frame.actors.truncate(1);
                    frame.actors[0].source_id = if species == "PIKACHU" {
                        "surfing_pikachu"
                    } else { "surf" }.into();
                    let mut mount = frame.actors[0].clone();
                    mount.id = VisualActorId::SurfMount;
                    mount.source_id = format!("species:{species}:icon_monster").into();
                    frame.actors.push(mount);
                }
                for facing in [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y] {
                    for _ in 0..4 {
                        app.world_mut().resource_mut::<Time>()
                            .advance_by(std::time::Duration::from_secs_f32(1. / 60.));
                        for actor in &mut app.world_mut().resource_mut::<VisualWorldFrame>().actors {
                            actor.facing = Some(facing);
                            actor.center += facing * 0.1;
                        }
                        app.update();
                    }
                    let world = app.world();
                    let state = world.resource::<ModeledActors>();
                    let rider = &state.instances[&VisualActorId::Player];
                    let mount = &state.props[&VisualActorId::SurfMount];
                    assert!(rider.person.surfing);
                    assert_eq!(rider.kind, if female { CharacterKind::TrainerFemale } else { CharacterKind::Trainer });
                    assert!(!state.props.contains_key(&VisualActorId::Player), "replace the generic Surf prop");
                    let skin = mount.skin.as_ref().unwrap();
                    assert!(skin.pose[0].translation.y.abs()
                        < (skin.rig.neutral_bounds.1.y - skin.rig.neutral_bounds.0.y) * 0.03,
                        "Surf mounts swim at the waterline instead of hovering");
                    let mut expected_swim = vec![Transform::IDENTITY; skin.rig.joints.len()];
                    skin.rig.sample_swimming(skin.seconds, skin.motion.phase,
                        skin.motion.walk_weight, &mut expected_swim);
                    for (actual, expected) in skin.pose.iter().zip(&expected_swim) {
                        assert!(actual.rotation.abs_diff_eq(expected.rotation, 1e-6));
                        assert!(actual.translation.abs_diff_eq(expected.translation, 1e-6));
                    }
                    let seat = surf_riding::SurfSeat::for_rig(skin.rig);
                    let mount_root = world.get::<Transform>(mount.entity).unwrap();
                    assert!(seat.waterline > 0. && seat.waterline < seat.point.y,
                        "{species} must intersect the water below its seated rider: {seat:?}");
                    assert!((mount_root.translation.y + seat.waterline * mount_root.scale.y - 0.04).abs() < 0.0001,
                        "the body-local waterline, rather than its lowest vertex, meets the water plane");
                    let meters = world.resource::<crystal_render_api::VisualPokemonDimensions>().meters[species];
                    let reference = crate::battle_layout::reference_span(species,
                        skin.rig.neutral_bounds.0, skin.rig.neutral_bounds.1);
                    let world_units_per_meter = world.resource::<VisualWorldFrame>().tile_size.y * 2.;
                    assert!((mount_root.scale.y * reference / world_units_per_meter - meters).abs() < 0.00001,
                        "mounting must preserve the Pokédex dimension");
                    assert_eq!(mount_root.scale, Vec3::splat(mount_root.scale.y));
                    let rider_root = world.get::<Transform>(rider.entity).unwrap();
                    let contact = mount_root.transform_point(seat.animated_point(skin.rig, &skin.matrices));
                    assert!(rider_root.translation.abs_diff_eq(contact, 0.0001));
                    assert!(rider_root.rotation.dot(mount_root.rotation).abs() > 0.9999);
                    assert_eq!(world.get::<Visibility>(rider.shadow), Some(&Visibility::Hidden));
                }
                let mounted_scale = {
                    let state = app.world().resource::<ModeledActors>();
                    app.world().get::<Transform>(state.props[&VisualActorId::SurfMount].entity).unwrap().scale
                };
                {
                    let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
                    frame.actors[0].source_id = if female { "kris" } else { "chris" }.into();
                    frame.actors[1].id = VisualActorId::PartyFollower;
                }
                app.update();
                let state = app.world().resource::<ModeledActors>();
                let following_scale = app.world().get::<Transform>(state.props[&VisualActorId::PartyFollower].entity).unwrap().scale;
                assert_eq!(mounted_scale, following_scale, "Surf must never resize {species}");
            }
            {
                let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
                frame.actors.truncate(1);
                frame.actors[0].source_id = if female { "kris" } else { "chris" }.into();
            }
            app.update();
            let state = app.world().resource::<ModeledActors>();
            assert!(!state.props.contains_key(&VisualActorId::SurfMount));
            let rider = &state.instances[&VisualActorId::Player];
            assert!(!rider.person.surfing);
            assert_eq!(app.world().get::<Visibility>(rider.shadow), Some(&Visibility::Inherited));
        }
    }

    #[test]
    fn modeled_pokemon_keep_lit_pigment_and_cast_grounding_shadows() {
        let mut app=actor_test_app();
        let mut follower=actor();follower.id=VisualActorId::PartyFollower;
        follower.source_id="species:TOTODILE:icon_monster".into();
        let original=follower.clone();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors=vec![follower];
        for map in ["GoldenrodCity","ElmsLab","UnionCave1F"] {
            app.world_mut().resource_mut::<VisualWorldFrame>().map_id=map.into();
            app.update();
            let state=app.world().resource::<ModeledActors>();
            let instance=&state.props[&VisualActorId::PartyFollower];
            assert!(instance.skin.is_some(),"preserve production animation");
            assert!(app.world().get::<NotShadowReceiver>(instance.entity).is_some(),
                "small painted Pokemon must remain readable in cast shade");
            assert!(app.world().get::<NotShadowCaster>(instance.entity).is_none(),
                "the Pokemon still casts a real grounding shadow");
            let handle=&app.world().get::<MeshMaterial3d<StandardMaterial>>(instance.entity).unwrap().0;
            let material=app.world().resource::<Assets<StandardMaterial>>().get(handle).unwrap();
            assert!(!material.unlit,"retain real directional contour shading");
            assert_eq!(material.base_color,Color::WHITE,"keep the canonical model pigment");
            assert_eq!(material.emissive,LinearRgba::BLACK,"readability must not make the Pokemon luminous");
            assert_eq!(app.world().resource::<VisualWorldFrame>().actors[0],original);
        }
    }
    #[test]
    fn pokemon_motion_followers_use_travel_and_ignore_camera_scroll_and_warps() {
        for name in ["PIKACHU", "LUGIA", "GLIGAR"] {
            let mut app=actor_test_app();
            let mut follower=actor();follower.id=VisualActorId::PartyFollower;
            follower.source_id=format!("species:{name}:icon_monster").into();
            app.world_mut().resource_mut::<VisualWorldFrame>().actors=vec![follower];
            app.update();
            let root=app.world().resource::<ModeledActors>().props[&VisualActorId::PartyFollower].entity;
            let original=app.world().get::<Mesh3d>(root).unwrap().0.clone();
            for tick in 0..8 {
                app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].center.x+=1.;
                app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
                app.update();
                let state=app.world().resource::<ModeledActors>();
                let skin=state.props[&VisualActorId::PartyFollower].skin.as_ref().unwrap();
                assert!(skin.motion.phase>0.,"{name} {tick}");
            }
            let phase=app.world().resource::<ModeledActors>().props[&VisualActorId::PartyFollower].skin.as_ref().unwrap().motion.phase;
            {
                let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
                frame.center+=Vec2::new(8.,4.);frame.actors[0].center+=Vec2::new(8.,4.);
            }
            app.update();
            assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::PartyFollower].skin.as_ref().unwrap().motion.phase,phase);
            for _ in 0..60 {
                app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));app.update();
            }
            assert_eq!(app.world().resource::<ModeledActors>().props[&VisualActorId::PartyFollower].skin.as_ref().unwrap().motion.walk_weight,0.);
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].center.x+=500.;
            app.update();
            let state=app.world().resource::<ModeledActors>();let skin=state.props[&VisualActorId::PartyFollower].skin.as_ref().unwrap();
            assert_eq!(skin.motion.phase,0.);assert_eq!(skin.motion.walk_weight,0.);
            assert_eq!(app.world().get::<Mesh3d>(root).unwrap().0,original);
        }
    }

    #[test]
    fn animated_pokemon_bounds_survive_bevys_mesh_insertion_pass() {
        use bevy::camera::{primitives::{Aabb, MeshAabb}, visibility::{NoAutoAabb, calculate_bounds}};
        let mut app = actor_test_app();
        let mut pokemon = actor();
        pokemon.id = VisualActorId::Object(10);
        pokemon.source_id = "species:FURRET:icon_monster".into();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors = vec![pokemon.clone()];
        app.update();
        let root = app.world().resource::<ModeledActors>().props[&pokemon.id].entity;
        let envelope = *app.world().get::<Aabb>(root).unwrap();
        let mesh = app.world().get::<Mesh3d>(root).unwrap().0.clone();
        let neutral = app.world().resource::<Assets<Mesh>>().get(&mesh).unwrap().get_aabb().unwrap();
        assert_ne!(envelope, neutral, "locomotion bounds must cover more than the neutral mesh");
        app.add_systems(PostUpdate, calculate_bounds);
        app.update();
        assert_eq!(*app.world().get::<Aabb>(root).unwrap(), envelope);
        // Reproduce the engine overwrite that the production marker prevents.
        app.world_mut().entity_mut(root).remove::<NoAutoAabb>();
        app.world_mut().get_mut::<Mesh3d>(root).unwrap().set_changed();
        app.update();
        assert_eq!(*app.world().get::<Aabb>(root).unwrap(), neutral);
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
        assert_eq!(a_skin.joints.len(),26); // canonical 14 plus four complete three-joint leg chains
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
        let follower_root=app.world().get::<Transform>(state.props[&follower.id].entity).unwrap();
        assert_eq!(follower_root.scale,Vec3::splat(app.world().resource::<VisualWorldFrame>().tile_size.y*2.0));
        assert_eq!(root.scale, follower_root.scale, "NPC and follower retain the same physical size");
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
    fn canonical_pokemon_displays_are_inert_while_living_species_keep_pack_scale() {
        for (label,species,art,height,meters) in [
            ("big_lapras","LAPRAS","big_lapras",1.52,2.4892),
            ("big_snorlax","SNORLAX","big_snorlax",1.48,2.1082),
            ("big_onix","ONIX","big_onix",1.72,8.7884),
            ("icon_lapras","LAPRAS","icon_lapras",0.82,2.4892),
            ("icon_snorlax","SNORLAX","icon_snorlax",0.82,2.1082),
            ("display:LAPRAS:big_lapras","LAPRAS","big_lapras",1.52,2.4892),
            ("display:BLASTOISE:icon_squirtle","BLASTOISE","icon_squirtle",0.82,1.6002),
            ("display:PIKACHU:surfing_pikachu","PIKACHU","icon_pikachu",0.82,0.4064),
        ] {
            let mut app=actor_test_app();
            let mut display=actor();display.id=VisualActorId::Object(51);
            display.source_id=label.into();display.center=Vec2::new(8.,8.);
            let mut living=display.clone();living.id=VisualActorId::Object(52);
            living.source_id=format!("species:{species}:{art}").into();
            living.center.x+=24.;
            app.world_mut().resource_mut::<VisualWorldFrame>().actors=vec![display,living];
            app.world_mut().resource_mut::<crystal_render_api::VisualPokemonDimensions>()
                .meters.insert(species.into(),meters);
            app.update();
            app.world_mut().resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(0.2));
            app.update();
            let world=app.world();let state=world.resource::<ModeledActors>();
            let display=&state.props[&VisualActorId::Object(51)];
            let living=&state.props[&VisualActorId::Object(52)];
            assert!(display.skin.is_none(),"{label} stays an inert display");
            assert!(world.get::<bevy::mesh::skinning::SkinnedMesh>(display.entity).is_none());
            let skin=living.skin.as_ref().expect("identified creature uses its canonical rig");
            assert!(skin.seconds>0.);
            assert!(skin.pose.iter().zip(&skin.rig.joints).any(|(pose,joint)|pose.rotation!=joint.bind.rotation));
            let units=world.resource::<VisualWorldFrame>().tile_size.y*2.;
            let display_mesh=actor_props::mesh(display.kind);
            let low=display_mesh.positions.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min);
            let high=display_mesh.positions.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max);
            let toy=world.get::<Transform>(display.entity).unwrap();
            assert_eq!(toy.scale,Vec3::splat(toy.scale.y));
            assert!(((high-low)*toy.scale.y/units-height).abs()<0.00001);
            let actual=world.get::<Transform>(living.entity).unwrap();
            let span=crate::battle_layout::reference_span(species,skin.rig.neutral_bounds.0,skin.rig.neutral_bounds.1);
            assert!((span*actual.scale.y/units-meters).abs()<0.00001);
            for entity in [display.entity,living.entity] {
                assert!((world.get::<Transform>(entity).unwrap().translation.y-0.04).abs()<0.00001);
                assert!(world.get::<NotShadowReceiver>(entity).is_some());
            }
        }
    }

    #[test]
    fn world_pokemon_rig_aliases_preserve_exact_canonical_vertex_order() {
        for &kind in actor_props::ALL_KINDS {
            if let Some(rig)=world_pokemon_rig(kind) {
                assert_eq!(actor_props::mesh(kind).positions,rig.neutral.positions,"{}",kind.label());
            }
        }
        for &kind in actor_props::ALL_KINDS {
            if kind.label().starts_with("icon_") {
                assert!(world_pokemon_rig(kind).is_none(),"shared icon art is not a living species: {}",kind.label());
            }
        }
        assert!(world_pokemon_rig(actor_props::PropKind::IconBird).is_none());
        assert!(world_pokemon_rig(actor_props::PropKind::IconFish).is_none());
    }
    #[test]
    fn uphill_hop_moves_the_character_root_and_keeps_its_shadow_grounded() {
        use crystal_render_api::{VisualPlayerTraversal, VisualTraversalStep};
        let mut app = actor_test_app();
        let from = Vec2::new(0.0, -12.0);
        let to = Vec2::new(0.0, -4.0);
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
            assert_eq!(root.translation.z, -(from.lerp(to, progress).y + 8.0));
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
    #[ignore = "requires supplied mounted player/NPC rigs and paintings"]
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
            for (id, x) in [(20, 64.), (21, 88.)] {
                let mut biker = frame.actors[0].clone();
                biker.id = VisualActorId::Object(id);
                biker.source_id = "biker".into();
                biker.center.x += x;
                frame.actors.push(biker);
            }
        }
        app.update();
        {
            let state = app.world().resource::<ModeledActors>();
            for id in [VisualActorId::Object(20), VisualActorId::Object(21)] {
                let rider = &state.instances[&id];
                assert_eq!(rider.kind, CharacterKind::Biker);
                assert!(rider.person.bicycle, "NPC must spawn in its riding pose");
                assert_eq!(state.props[&id].kind, actor_props::PropKind::BikerBike);
                assert_eq!(app.world().get::<Transform>(rider.entity).unwrap().translation,
                    app.world().get::<Transform>(state.props[&id].entity).unwrap().translation);
            }
            let first = &state.props[&VisualActorId::Object(20)];
            let second = &state.props[&VisualActorId::Object(21)];
            assert_eq!(app.world().get::<Mesh3d>(first.entity).unwrap().0,
                app.world().get::<Mesh3d>(second.entity).unwrap().0);
            assert_ne!(app.world().get::<Children>(first.entity).unwrap()[0],
                app.world().get::<Children>(second.entity).unwrap()[0]);
        }
        for step in 0..24 {
            app.world_mut().resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(1./60.));
            {
                let mut frame = app.world_mut().resource_mut::<VisualWorldFrame>();
                frame.actors[0].center.x += 0.3;
                frame.actors[0].facing = Some(Vec2::NEG_Y);
                frame.actors[2].center.x += 0.15;
                frame.actors[2].facing = Some(Vec2::NEG_Y);
            }
            let published = app.world().resource::<VisualWorldFrame>().actors.clone();
            app.update();
            let state = app.world().resource::<ModeledActors>();
            assert_eq!(app.world().resource::<VisualWorldFrame>().actors, published);
            for id in [VisualActorId::Player,VisualActorId::Object(19),
                VisualActorId::Object(20),VisualActorId::Object(21)] {
                let rider = &state.instances[&id];
                assert!(rider.person.bicycle);
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
            assert_eq!(state.instances[&VisualActorId::Object(21)].motion.phase,0.);
            assert!(state.instances[&VisualActorId::Player].motion.phase>0.);
            assert!(state.instances[&VisualActorId::Object(20)].motion.phase>0.);
            assert_ne!(state.instances[&VisualActorId::Player].motion.phase,
                state.instances[&VisualActorId::Object(20)].motion.phase);
            if step==0 {
                let rider = &state.instances[&VisualActorId::Player];
                assert!((rider.motion.yaw-facing_yaw(Vec2::NEG_Y)).abs()>0.1,
                    "test must exercise an interpolated turn rather than its settled endpoint");
            }
        }
        let bike = state_bike_entity(&app, VisualActorId::Object(20));
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[2].source_id = "youngster".into();
        app.update();
        assert!(app.world().get_entity(bike).is_err(), "sprite changes must remove the NPC bike");
        assert!(!app.world().resource::<ModeledActors>().instances[&VisualActorId::Object(20)].person.bicycle);
        let bike = state_bike_entity(&app, VisualActorId::Object(21));
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.retain(|actor| actor.id != VisualActorId::Object(21));
        app.update();
        assert!(app.world().get_entity(bike).is_err(), "hidden NPCs must remove their bikes");
        assert!(!app.world().resource::<ModeledActors>().instances.contains_key(&VisualActorId::Object(21)));
    }

    fn state_bike_entity(app: &App, id: VisualActorId) -> Entity {
        app.world().resource::<ModeledActors>().props[&id].entity
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
            // Map transitions retire the previous appearance; this test must
            // prove every source uploads correctly without requiring a leaked
            // catalog of all previously visited character meshes.
            assert_eq!(app.world().resource::<ModeledActors>().people_gpu.0.len(), 1,
                "stale character upload after switching to {source}");
        }
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
                &frame,
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
        let frame = actor_test_app().world().resource::<VisualWorldFrame>().clone();
        let transform = model_transform(&frame, &actor, 2.0, facing_yaw(Vec2::X), 16.0);
        assert_eq!(transform.translation, Vec3::new(20.0, 2.04, -30.0));
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
        let frame = app.world().resource::<VisualWorldFrame>();
        assert_eq!(p.translation,visual_point_to_voxel(actor_ground_point(frame, &player),0.04));
        assert_eq!(n.translation,visual_point_to_voxel(actor_ground_point(frame, &npc),0.04));
        assert_eq!(app.world().resource::<VisualWorldFrame>().actors[0],player);
    }

    #[test]
    fn bicycle_and_rider_share_the_centered_anchor_on_the_first_frame() {
        for source in ["chris_bike", "kris_bike"] {
            let mut app = actor_test_app();
            app.world_mut().resource_mut::<VisualWorldFrame>().actors[0].source_id = source.into();
            app.update();
            let state = app.world().resource::<ModeledActors>();
            let rider = app.world().get::<Transform>(state.instances[&VisualActorId::Player].entity).unwrap();
            let bike = app.world().get::<Transform>(state.props[&VisualActorId::Player].entity).unwrap();
            assert_eq!(rider.translation, bike.translation, "{source} cannot wait a frame to align");
            assert_eq!(rider.translation, Vec3::new(20., 0.04, -30.));
        }
    }

    #[test]
    fn external_map_edge_characters_stand_inside_the_original_floor_bounds() {
        let explicit = std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path = explicit.as_ref().map(std::path::PathBuf::from).unwrap_or_else(||
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        assert!(explicit.is_none() || path.is_file());
        if !path.is_file() { return; }
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let mut app = actor_test_app();
        for map in ["RadioTower3F", "RadioTower4F", "RadioTower5F", "PlayersHouse2F", "NewBarkTown"] {
            let data = &loaded.pack().data().maps[map];
            let size = UVec2::new(data.attributes.width as u32 * 2, data.attributes.height as u32 * 2);
            for scale in [1., 4.] {
                let mut frame = app.world().resource::<VisualWorldFrame>().clone();
                frame.map_id = map.into();
                frame.source_map_size_core_tiles = Some(size);
                frame.grid_size = size * 2;
                frame.tile_size = Vec2::splat(8. * scale);
                frame.viewport_size = frame.grid_size.as_vec2() * frame.tile_size;
                frame.center = Vec2::new(23., -41.);
                frame.actors.clear();
                let half = frame.viewport_size * 0.5;
                let mut expected = Vec::new();
                for (i, (x, y)) in [(0, 0), (size.x - 1, 0), (0, size.y - 1), (size.x - 1, size.y - 1)]
                    .into_iter().enumerate()
                {
                    let mut a = actor();
                    a.id = match i { 0 => VisualActorId::Player, 1 => VisualActorId::RemotePlayer(1),
                        _ => VisualActorId::Object(i as u32) };
                    a.size = Vec2::splat(16. * scale);
                    a.center = frame.center + Vec2::new(-half.x + (x as f32 * 2. + 1.) * frame.tile_size.x,
                        half.y - (y as f32 * 2. + 1.) * frame.tile_size.y);
                    a.facing = Some([Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y][i]);
                    expected.push((a.id, a.center));
                    frame.actors.push(a);
                }
                app.world_mut().resource_mut::<TerrainRevisionCache>().footing_heights =
                    vec![0.; (frame.grid_size.x * frame.grid_size.y) as usize];
                *app.world_mut().resource_mut::<VisualWorldFrame>() = frame.clone();
                app.update();
                let state = app.world().resource::<ModeledActors>();
                for (id, center) in expected {
                    let root = app.world().get::<Transform>(state.instances[&id].entity).unwrap();
                    assert!((root.translation - visual_point_to_voxel(center, 0.04)).length() < 0.001,
                        "{map} {id:?} must stand at the core tile center, independently of facing and scale");
                    let p = Vec2::new(root.translation.x, -root.translation.z) - frame.center;
                    assert!(p.abs().cmple(half - frame.tile_size + Vec2::splat(0.001)).all(),
                        "{map} {id:?} needs half a gameplay tile of floor on every side");
                }
                assert_eq!(*app.world().resource::<VisualWorldFrame>(), frame,
                    "map extent and source presentation must remain unchanged");
            }
        }
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
    fn bedroom_dolls_use_complete_table_support_without_raising_living_pokemon() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        let mut app=actor_test_app();let mut frame=app.world().resource::<VisualWorldFrame>().clone();
        frame.map_id="PlayersHouse2F".into();frame.grid_size=UVec2::new(6,4);
        frame.viewport_size=Vec2::new(48.,32.);frame.tiles.clear();frame.actors.clear();
        let drawing=[[16,17,17,18],[32,33,33,34],[48,49,49,50]];
        for row in 0..4 {for column in 0..6 {
            let tile_index=if row<3&&column<4 {drawing[row][column]}else{1};
            frame.tiles.push(VisualTile{column:column as u32,row:row as u32,
                source:VisualTileSource{tileset_id:"players_room".into(),metatile_id:if row<3&&column<4 {7}else{0},
                    subtile_column:(column%4)as u8,subtile_row:(row%4)as u8,tile_index},
                texture:default(),priority:false,animation_frames:None});
        }}
        for (id,source,center) in [
            (31,"display:PIKACHU:surfing_pikachu",Vec2::new(-12.,12.)),
            (32,"display:LAPRAS:icon_lapras",Vec2::new(-4.,4.)),
            (33,"display:SNORLAX:big_snorlax",Vec2::new(20.,-4.)),
            (34,"species:PIKACHU:icon_pikachu",Vec2::new(-12.,12.)),
        ] {
            let mut doll=actor();doll.id=VisualActorId::Object(id);doll.source_id=source.into();
            doll.center=center;frame.actors.push(doll);
        }
        {
            let mut footing=app.world_mut().resource_mut::<TerrainRevisionCache>();
            footing.built_frame=Some(frame.clone());footing.footing_heights=vec![2.;24];
        }
        *app.world_mut().resource_mut::<VisualWorldFrame>()=frame.clone();app.update();
        for (id,height) in [(31,10.04),(32,10.04),(33,2.04),(34,2.04)] {
            let state=app.world().resource::<ModeledActors>();
            let transform=app.world().get::<Transform>(state.props[&VisualActorId::Object(id)].entity).unwrap();
            assert!((transform.translation.y-height).abs()<1e-5,"{id} at {}",transform.translation.y);
        }
        // A changed or incomplete drawing supplies no phantom tabletop.
        app.world_mut().resource_mut::<TerrainRevisionCache>().built_frame.as_mut().unwrap()
            .tiles[0].source.tile_index=255;
        app.update();
        for id in [31,32] {
            let state=app.world().resource::<ModeledActors>();
            let transform=app.world().get::<Transform>(state.props[&VisualActorId::Object(id)].entity).unwrap();
            assert!((transform.translation.y-2.04).abs()<1e-5);
        }
        assert_eq!(app.world().resource::<TerrainRevisionCache>().footing_heights,vec![2.;24]);
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),frame);
    }

    #[test]
    fn elm_starter_balls_rest_on_bare_table_and_follow_object_visibility() {
        use crystal_render_api::{VisualTile,VisualTileSource};
        let mut app=actor_test_app();let mut frame=app.world().resource::<VisualWorldFrame>().clone();
        frame.map_id="ElmsLab".into();frame.grid_size=UVec2::new(8,5);frame.viewport_size=Vec2::new(64.,40.);
        frame.actors.clear();frame.tiles.clear();
        let drawing=[[5,6,6,6,6,7],[21,22,22,22,22,23],[37,38,38,38,38,39]];
        for y in 0..5 {for x in 0..8 {
            let mut source=VisualTileSource{tileset_id:"lab".into(),metatile_id:1,
                subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:0x10};
            if y<3&&x<6 {
                source.metatile_id=[[6,7],[10,11]][(y+2)/4][x/4];
                source.subtile_row=((y+2)%4)as u8;source.tile_index=drawing[y][x];
            }
            frame.tiles.push(VisualTile{column:x as u32,row:y as u32,source,texture:default(),priority:false,animation_frames:None});
        }}
        for (i,column) in [1,3,5].into_iter().enumerate() {
            let mut ball=actor();ball.id=VisualActorId::Object(20+i as u32);ball.source_id="poke_ball".into();
            ball.center=Vec2::new(-32.+(column as f32+0.5)*8.,8.);frame.actors.push(ball);
        }
        app.world_mut().resource_mut::<TerrainRevisionCache>().built_frame=Some(frame.clone());
        app.world_mut().resource_mut::<TerrainRevisionCache>().footing_heights=vec![6.;40];
        *app.world_mut().resource_mut::<VisualWorldFrame>()=frame.clone();app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props.len(),3);
        for ball in &frame.actors {
            let entity=app.world().resource::<ModeledActors>().props[&ball.id].entity;
            let transform=app.world().get::<Transform>(entity).unwrap();
            assert_eq!(transform.translation,visual_point_to_voxel(ball.center,10.04));
        }
        assert_eq!(*app.world().resource::<VisualWorldFrame>(),frame);
        let removed=app.world().resource::<ModeledActors>().props[&frame.actors[0].id].entity;
        app.world_mut().resource_mut::<VisualWorldFrame>().actors.remove(0);app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props.len(),2);
        assert!(app.world().get_entity(removed).is_err());
        assert_eq!(app.world().resource::<TerrainRevisionCache>().footing_heights,vec![6.;40]);
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
    fn native_stone_palette_updates_without_changing_movement_or_leaking_indoors() {
        use bevy::{asset::RenderAssetUsages,render::render_resource::{Extent3d,TextureDimension,TextureFormat}};
        let mut app=actor_test_app();
        let image=Image::new_fill(Extent3d {width:4,height:4,depth_or_array_layers:1},
            TextureDimension::D2,&[165,123,24,255],TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default());
        let texture=app.world_mut().resource_mut::<Assets<Image>>().add(image);
        let id=VisualActorId::Object(19);let mut prop=actor();
        prop.id=id;prop.source_id="rock".into();prop.texture=texture.clone();
        {
            let mut frame=app.world_mut().resource_mut::<VisualWorldFrame>();
            frame.map_id="CianwoodCity".into();frame.actors.push(prop);
            frame.tiles.push(crystal_render_api::VisualTile {column:0,row:0,
                source:crystal_render_api::VisualTileSource {tileset_id:"johto".into(),
                    metatile_id:0,subtile_column:0,subtile_row:0,tile_index:6},
                texture:default(),animation_frames:None,priority:false});
        }
        app.update();
        let first=app.world().resource::<ModeledActors>().props[&id].entity;
        assert_eq!(app.world().resource::<ModeledActors>().props[&id].stone,Some((CaveStoneFinish::Native([165,123,24]),0)));
        let count=app.world().resource::<Assets<Mesh>>().len();
        app.world_mut().resource_mut::<VisualWorldFrame>().actors[1].center.x+=8.;app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props[&id].entity,first);
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(),count);
        for p in app.world_mut().resource_mut::<Assets<Image>>().get_mut(&texture).unwrap()
            .data.as_mut().unwrap().chunks_exact_mut(4) {p.copy_from_slice(&[120,80,160,255]);}
        app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props[&id].stone,Some((CaveStoneFinish::Native([120,80,160]),0)));
        assert!(app.world().get_entity(first).is_err());
        app.world_mut().resource_mut::<VisualWorldFrame>().map_id="PlayersHouse1F".into();app.update();
        assert_eq!(app.world().resource::<ModeledActors>().props[&id].stone,None);
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
    #[ignore = "requires supplied human GLBs"]
    fn character_cadence_keeps_native_stance_feet_planted() {
        validate_human_cadence();
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
            // The shorter walking cycle can advance up to 1.09 radians on
            // the first fast frame, before the smooth run blend catches up.
            assert!(step > 0.0 && step < 1.10, "phase jumped by {step}");
            assert!((state.run_weight - old_run).abs() < 0.13);
            assert_eq!(state.last_world_foot, position);
            if frame == 230 {
                assert!(state.run_weight > 0.999);
            }
        }
        assert!(state.run_weight < 0.01);
    }

}
