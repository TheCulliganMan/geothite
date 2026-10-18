//! A frozen instance of the terrain that actually supported a checked encounter.
//! No terrain reconstruction, source sprite copies, or borrowed world fade state.
use bevy::camera::primitives::MeshAabb;
use crate::{
    TerrainRevisionCache, VoxelMaterial,
    battle_layout::{BattleBody, BattleSceneLayout, CAMERA_FOV},
    battle_view::BattleViewStatus,
};
use bevy::{
    asset::RenderAssetUsages,
    asset::AssetId,
    prelude::*,
    camera::{primitives::Aabb, visibility::RenderLayers},
};
use crystal_render_api::{
    BattleFlashMode, BattleLocationFrame, VisualBattleFrame, VisualBattleLocation, VisualWorldFrame,
};
use std::collections::HashMap;
mod walking_ground;
mod retained_correspondence;
mod water_stance;
mod floor_palette;
use walking_ground::WalkingGround;

const BATTLE_LAYER: usize = 29;
/// Four retained-world units per source pixel and sixteen pixels per core tile
/// give 64 retained units per tile. This uniform conversion gives four here.
const RETAINED_SCALE: f32 = 1.0 / 16.0;
const SOURCE_CELL_SIZE: f32 = 32.0;
const BORDER_MARGIN: f32 = 0.25;
const LOW_SHOT_ELEVATIONS: [f32; 7] = [6.0, 12.0, 18.0, 24.0, 32.0, 40.0, 48.0];
const HIGH_SHOT_ELEVATIONS: [f32; 3] = [60.0, 68.0, 75.0];
const RESIDUAL_OPACITY: f32 = 0.20;
const SHOT_YAWS: [f32; 8] = [70.0, 90.0, 110.0, 140.0, -70.0, -90.0, -110.0, -140.0];

fn review_shot_yaw(value: Option<&str>) -> Option<f32> {
    let yaw=value?.parse::<f32>().ok()?;
    SHOT_YAWS.contains(&yaw).then_some(yaw)
}

/// Native art review can select an existing establishing angle. Admission,
/// retained scenery, support and full-body fitting still use the normal proof.
fn requested_review_yaw() -> Option<f32> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static YAW: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
        *YAW.get_or_init(|| {
            let yaw=review_shot_yaw(std::env::var("CRYSTAL_BATTLE_REVIEW_YAW").ok().as_deref());
            if let Some(yaw)=yaw { trace(format_args!("native review yaw selected: {yaw}")); }
            yaw
        })
    }
    #[cfg(target_arch = "wasm32")]
    { None }
}

/// Reward the smaller visible silhouette first. A large partner must not hide
/// a tiny opponent's poor framing behind its own screen coverage. Long, low
/// animals count by their full silhouette rather than standing height alone.
/// This only ranks already proven shots; feet, scale and map geometry stay fixed.
fn actor_readability_score(extents: &[Vec2]) -> f32 {
    let smallest = extents.iter().map(|extent| extent.max_element())
        .fold(f32::INFINITY, f32::min);
    if !smallest.is_finite() || smallest <= 0.0 { return 0.0; }
    smallest.min(0.18) * 1000.0
}

// An upper bound, not a quality threshold: 32 backdrop cells, maximal
// silhouette reward, readable-facing bonus and the lowest candidate elevation.
// Every other score term is nonpositive. Keep this conservative if scoring changes.
fn maximum_shot_score() -> f32 {
    33.0_f32.ln() * 20.0 + 180.0 + 4.0 - 3.0
}


/// Screen coverage of actual tall scenery behind the fighters. A canopy may
/// frame the shot even when its base/center is offscreen. Count a fixed grid
/// once so huge distant walls and overlapping mesh batches cannot dominate.
#[cfg(test)]
fn backdrop_cells(bounds: Bounds, layout: &BattleSceneLayout, viewport: Vec2, pair_depth: f32) -> u32 {
    backdrop_cells_projected(bounds, layout, viewport, pair_depth, layout.camera.to_matrix().inverse())
}

fn backdrop_cells_projected(bounds: Bounds, layout: &BattleSceneLayout, viewport: Vec2, pair_depth: f32, view: Mat4) -> u32 {
    let forward = *layout.camera.forward();
    let center = (bounds.min + bounds.max) * 0.5;
    if (center - layout.camera.translation).dot(forward) <= pair_depth + 0.5 {
        return 0;
    }
    let mut lo = Vec2::splat(f32::INFINITY);
    let mut hi = Vec2::splat(f32::NEG_INFINITY);
    for i in 0..8 {
        let point = Vec3::new(
            if i & 1 == 0 { bounds.min.x } else { bounds.max.x },
            if i & 2 == 0 { bounds.min.y } else { bounds.max.y },
            if i & 4 == 0 { bounds.min.z } else { bounds.max.z });
        if (point - layout.camera.translation).dot(forward) <= 0.01 { return 0; }
        let local = view.transform_point3(point);
        let tangent = (CAMERA_FOV * 0.5).tan();
        let point = Vec2::new(
            0.5 + local.x / (-local.z * 2.0 * tangent * (viewport.x / viewport.y)),
            0.5 - local.y / (-local.z * 2.0 * tangent));
        if !point.is_finite() { return 0; }
        lo = lo.min(point);
        hi = hi.max(point);
    }
    let band = layout.visible_y_range();
    let top = band.x;
    let bottom = band.x + (band.y - band.x) * 0.60;
    let mut cells = 0;
    for row in 0..4 { for column in 0..8 {
        let sample = Vec2::new(0.05 + (column as f32 + 0.5) * 0.90 / 8.,
            top + (row as f32 + 0.5) * (bottom - top) / 4.);
        if sample.cmpge(lo).all() && sample.cmple(hi).all() {
            cells |= 1 << (row * 8 + column);
        }
    }}
    cells
}

fn camera_trial_requested(value: Option<&str>) -> bool {
    value == Some("1")
}

fn camera_trial_enabled() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| {
            camera_trial_requested(std::env::var("CRYSTAL_BATTLE_CAMERA_TRIAL").ok().as_deref())
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

fn trace(message: std::fmt::Arguments<'_>) {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var_os("CRYSTAL_ENCOUNTER_TRACE").is_some() {
        eprintln!("encounter terrain: {message}");
    }
    #[cfg(target_arch = "wasm32")]
    let _ = message;
}

/// An infinite/repeated backdrop is presentation fill, never authored acreage.
#[derive(Component)]
pub(super) struct SyntheticTerrainApron;
#[derive(Component)]
pub(super) struct EncounterTerrainRoot;

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}

/// Immutable scenery bounds, indexed once when the encounter is frozen.
/// Branch bounds only reject impossible hits; leaves retain the exact sight
/// predicate used by the original flat scan.
#[derive(Default)]
struct StaticBlockers {
    bounds: Vec<Bounds>,
    indices: Vec<usize>,
    nodes: Vec<BlockerNode>,
}
impl std::ops::Deref for StaticBlockers {
    type Target = [Bounds];
    fn deref(&self) -> &Self::Target { &self.bounds }
}
struct BlockerNode {
    bounds: Bounds,
    range: std::ops::Range<usize>,
    children: Option<[usize; 2]>,
}
impl StaticBlockers {
    fn visible_or_shadow(&self, camera: Transform, viewport: Vec2, far: f32, ground: f32) -> bool {
        !self.nodes.is_empty() && self.query_view(0, camera, viewport, far, ground)
    }

    fn query_view(&self, index: usize, camera: Transform, viewport: Vec2, far: f32, ground: f32) -> bool {
        let node = &self.nodes[index];
        // A union contains every descendant and its shadow extrusion. Pad the
        // broad phase so floating-point grazing cannot discard a real leaf.
        // Final leaf checks are exactly the original admission predicate.
        let pad = Vec3::splat(0.001);
        let bounds = Bounds { min: node.bounds.min - pad, max: node.bounds.max + pad };
        if !visible_bounds(bounds, camera, viewport, far)
            && !visible_bounds(bounds.shadow_volume(ground), camera, viewport, far) { return false; }
        if let Some([left, right]) = node.children {
            return self.query_view(left, camera, viewport, far, ground)
                || self.query_view(right, camera, viewport, far, ground);
        }
        self.indices[node.range.clone()].iter().any(|&index| {
            let bounds = self.bounds[index];
            visible_bounds(bounds, camera, viewport, far)
                || visible_bounds(bounds.shadow_volume(ground), camera, viewport, far)
        })
    }

    fn new(bounds: Vec<Bounds>) -> Self {
        let indices = (0..bounds.len()).collect();
        let mut out = Self { bounds, indices, nodes: Vec::new() };
        if !out.bounds.is_empty() { out.build(0..out.bounds.len()); }
        out
    }

    fn build(&mut self, range: std::ops::Range<usize>) -> usize {
        let bounds = self.indices[range.clone()].iter().copied()
            .map(|index| self.bounds[index]).reduce(Bounds::union).unwrap();
        let node = self.nodes.len();
        self.nodes.push(BlockerNode { bounds, range: range.clone(), children: None });
        if range.len() > 16 {
            let extent = bounds.max - bounds.min;
            let axis = if extent.x >= extent.y && extent.x >= extent.z { 0 }
                else if extent.y >= extent.z { 1 } else { 2 };
            let boxes = &self.bounds;
            self.indices[range.clone()].sort_unstable_by(|&a, &b| {
                let center = |index: usize| (boxes[index].min[axis] + boxes[index].max[axis]) * 0.5;
                center(a).total_cmp(&center(b)).then(a.cmp(&b))
            });
            let middle = range.start + range.len() / 2;
            let left = self.build(range.start..middle);
            let right = self.build(middle..range.end);
            self.nodes[node].children = Some([left, right]);
        }
        node
    }

    fn blocker(&self, start: Vec3, end: Vec3, min_height: f32) -> Option<&Bounds> {
        if self.nodes.is_empty() { return None; }
        self.query(0, start, end, min_height)
    }

    fn visible_grass_blocks(&self, start: Vec3, end: Vec3, camera: Transform, cutoff: f32) -> bool {
        !self.nodes.is_empty() && self.query_grass(0, start, end, camera, cutoff)
    }

    fn query_grass(&self, index: usize, start: Vec3, end: Vec3, camera: Transform, cutoff: f32) -> bool {
        let node = &self.nodes[index];
        if !node.bounds.segment(start, end) || grass_bounds_discarded(node.bounds, camera, cutoff) {
            return false;
        }
        if let Some([left, right]) = node.children {
            return self.query_grass(left, start, end, camera, cutoff)
                || self.query_grass(right, start, end, camera, cutoff);
        }
        self.indices[node.range.clone()].iter().any(|&index| {
            let bounds = self.bounds[index];
            bounds.segment(start, end) && !grass_bounds_discarded(bounds, camera, cutoff)
        })
    }

    fn query(&self, index: usize, start: Vec3, end: Vec3, min_height: f32) -> Option<&Bounds> {
        let node = &self.nodes[index];
        if node.bounds.max.y <= min_height || !node.bounds.segment(start, end) { return None; }
        if let Some([left, right]) = node.children {
            return self.query(left, start, end, min_height)
                .or_else(|| self.query(right, start, end, min_height));
        }
        self.indices[node.range.clone()].iter().map(|&index| &self.bounds[index])
            .find(|bounds| bounds.max.y > min_height && bounds.segment(start, end))
    }

    #[cfg(test)]
    fn push(&mut self, bounds: Bounds) {
        let mut all = std::mem::take(&mut self.bounds);
        all.push(bounds);
        *self = Self::new(all);
    }

    #[cfg(test)]
    fn clear(&mut self) { *self = Self::default(); }
}
impl Bounds {
    fn from_aabb(aabb: Aabb, transform: Transform) -> Option<Self> {
        let min: Vec3 = (aabb.center - aabb.half_extents).into();
        let max: Vec3 = (aabb.center + aabb.half_extents).into();
        let mut out = Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        };
        for i in 0..8 {
            let p = transform.transform_point(Vec3::new(
                if i & 1 == 0 { min.x } else { max.x },
                if i & 2 == 0 { min.y } else { max.y },
                if i & 4 == 0 { min.z } else { max.z },
            ));
            if !p.is_finite() {
                return None;
            }
            out.min = out.min.min(p);
            out.max = out.max.max(p);
        }
        Some(out)
    }
    fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    fn segment(self, start: Vec3, end: Vec3) -> bool {
        // Most retained scenery lies nowhere near this sight line. Reject its
        // disjoint bounding box before the three-axis division/interval test.
        // The complete segment envelope contains the clipped interval below,
        // so this cannot turn a possible occluder into a clear sight line.
        if self.max.cmplt(start.min(end)).any() || self.min.cmpgt(start.max(end)).any() {
            return false;
        }
        let delta = end - start;
        let mut near: f32 = 0.001;
        let mut far: f32 = 0.995;
        for axis in 0..3 {
            if delta[axis].abs() < 0.000001 {
                if start[axis] < self.min[axis] || start[axis] > self.max[axis] {
                    return false;
                }
            } else {
                let a = (self.min[axis] - start[axis]) / delta[axis];
                let b = (self.max[axis] - start[axis]) / delta[axis];
                near = near.max(a.min(b));
                far = far.min(a.max(b));
                if near > far {
                    return false;
                }
            }
        }
        true
    }
    fn shadow_volume(self, ground: f32) -> Self {
        // Battle sun is at (6,12,8), aimed at the origin. Include the complete
        // caster and its downward extrusion, not only its ground contact.
        let height = (self.max.y - ground).max(0.0);
        Self {
            min: Vec3::new(
                self.min.x - height * 0.5,
                ground.min(self.min.y),
                self.min.z - height * (8.0 / 12.0),
            ),
            max: self.max,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct AuthenticBounds {
    min: Vec2,
    max: Vec2,
}
impl AuthenticBounds {
    fn contains(self, bounds: Bounds) -> bool {
        let min = Vec2::new(bounds.min.x, bounds.min.z);
        let max = Vec2::new(bounds.max.x, bounds.max.z);
        min.cmpge(self.min).all() && max.cmple(self.max).all()
    }
    /// Partition coverage, never geometry. The complete GPU mesh stays intact;
    /// every potentially observable exterior piece must still fail the view
    /// or shadow proof. A crossing bound must not label its known interior as
    /// missing terrain too.
    fn partition_coverage(self, bounds: Bounds) -> (Option<Bounds>, Vec<Bounds>) {
        let lo = Vec2::new(bounds.min.x, bounds.min.z).max(self.min);
        let hi = Vec2::new(bounds.max.x, bounds.max.z).min(self.max);
        if hi.cmplt(lo).any() { return (None, vec![bounds]); }
        let mut outside = Vec::with_capacity(4);
        for (low, high, present) in [
            (Vec2::new(bounds.min.x, bounds.min.z), Vec2::new(lo.x, bounds.max.z), lo.x > bounds.min.x),
            (Vec2::new(hi.x, bounds.min.z), Vec2::new(bounds.max.x, bounds.max.z), bounds.max.x > hi.x),
            (Vec2::new(lo.x, bounds.min.z), Vec2::new(hi.x, lo.y), lo.y > bounds.min.z),
            (Vec2::new(lo.x, hi.y), Vec2::new(hi.x, bounds.max.z), bounds.max.z > hi.y),
        ] {
            // A vertical wall has zero thickness in X or Z but still has a
            // real surface and casts shadows. Do not lose those exterior slabs.
            if present {
                outside.push(Bounds {
                    min: Vec3::new(low.x, bounds.min.y, low.y),
                    max: Vec3::new(high.x, bounds.max.y, high.y),
                });
            }
        }
        (Some(Bounds {
            min: Vec3::new(lo.x, bounds.min.y, lo.y),
            max: Vec3::new(hi.x, bounds.max.y, hi.y),
        }), outside)
    }
    fn point(self, p: Vec3) -> bool {
        let p = Vec2::new(p.x, p.z);
        p.cmpge(self.min + Vec2::splat(BORDER_MARGIN)).all()
            && p.cmple(self.max - Vec2::splat(BORDER_MARGIN)).all()
    }
    fn camera_limit(self, camera: Transform, viewport: Vec2) -> Option<f32> {
        if !self.point(camera.translation) || !viewport.is_finite() || viewport.min_element() <= 0.0
        {
            return None;
        }
        let tangent = (CAMERA_FOV * 0.5).tan();
        let aspect = viewport.x / viewport.y;
        let mut limit = f32::INFINITY;
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                let ray = camera.rotation * Vec3::new(x * tangent * aspect, y * tangent, -1.0);
                for (axis, component) in [(0, ray.x), (1, ray.z)] {
                    let origin = if axis == 0 {
                        camera.translation.x
                    } else {
                        camera.translation.z
                    };
                    if component > 0.000001 {
                        limit = limit.min((self.max[axis] - BORDER_MARGIN - origin) / component);
                    } else if component < -0.000001 {
                        limit = limit.min((self.min[axis] + BORDER_MARGIN - origin) / component);
                    }
                }
            }
        }
        (limit.is_finite() && limit > 0.0).then_some(limit)
    }
}

struct FrozenLeaf {
    mesh: Handle<Mesh>,
    material: Handle<VoxelMaterial>,
    transform: Transform,
    aabb: Aabb,
    bounds: Bounds,
    object: Option<Entity>,
    coverage: Option<std::sync::Arc<[Aabb]>>,
    sight_coverage: Option<std::sync::Arc<[Aabb]>>,
    grass_coverage: Option<std::sync::Arc<[Aabb]>>,
}
struct FrozenMaterial {
    alpha_cutoff: Option<f32>,
    handle: Handle<VoxelMaterial>,
    object: Option<Entity>,
}
struct FrozenObject {
    bounds: Bounds,
    backdrop: Vec<Bounds>,
    surface: Option<(crate::occluder_fade::RetainedSurface, Transform)>,
    faded: bool,
    opacity: f32,
}
impl FrozenObject {
    fn blocks(&self, start: Vec3, end: Vec3) -> bool {
        self.bounds.segment(start, end) && self.surface.as_ref().map_or(true,
            |(surface, pose)| surface.blocks(*pose, start, end))
    }
}

fn advance_opacity(opacity: f32, faded: bool, seconds: f32) -> f32 {
    let target = if faded { RESIDUAL_OPACITY } else { 1.0 };
    let next = target + (opacity - target) * (-12.0 * seconds.max(0.0)).exp();
    if (next - target).abs() < 0.001 {
        target
    } else {
        next
    }
}

fn retained_alpha_mode(opacity: f32, cutoff: Option<f32>) -> AlphaMode {
    match opacity_mode(opacity) {
        AlphaMode::Opaque => cutoff.map_or(AlphaMode::Opaque, AlphaMode::Mask),
        mode => mode,
    }
}

fn opacity_mode(opacity: f32) -> AlphaMode {
    if opacity >= 1.0 {
        AlphaMode::Opaque
    } else {
        AlphaMode::Blend
    }
}
struct FrozenScene {
    derived_stage: bool,
    /// Original source feet remain immutable when Pokémon staging moves onto
    /// another verified floor plot. Source actors and controller pose stay put.
    source_anchors: [Vec3; 2],
    root: Entity,
    anchors: [Vec3; 2],
    authentic: AuthenticBounds,
    omitted: StaticBlockers,
    static_blockers: StaticBlockers,
    grass_lens_blockers: StaticBlockers,
    objects: HashMap<Entity, FrozenObject>,
    backdrop_bounds: Vec<Bounds>,
    materials: Vec<FrozenMaterial>,
    images: Vec<Handle<Image>>,
    // Only encounter-owned copies of the two mutable flower domains.
    meshes: Vec<Handle<Mesh>>,
    ground: f32,
    walking: Option<WalkingGround>,
    water: bool,
    bodies: [Option<BattleBody>; 2],
    clearance: [Vec4; 2],
    clearance_shape: [Vec4; 2],
    grass_contact: [Vec4; 2],
    contact_shape: [Vec4; 2],
}

#[derive(Resource, Default)]
pub(super) struct EncounterTerrain {
    shot_score: Option<f32>,
    generation: Option<u64>,
    frozen: Option<FrozenScene>,
    accepted: bool,
    attempted_roots: Option<[Entity; 3]>,
    fog: Option<(f32, f32)>,
    reason: Option<&'static str>,
}
impl EncounterTerrain {
    pub(super) fn set_visible_apertures(&mut self, camera: Transform, bodies: [Option<(BattleBody, Transform)>; 2], contacts: [Option<(BattleBody, Transform)>; 2]) {
        let Some(scene) = self.frozen.as_mut().filter(|_| self.accepted) else { return; };
        for (index, body) in bodies.into_iter().enumerate() {
            let (clearance, shape) = body.map_or((Vec4::ZERO, Vec4::ZERO),
                |(body, pose)| grass_body_aperture(body, pose, camera));
            scene.clearance[index] = clearance;
            scene.clearance_shape[index] = shape;
            scene.grass_contact[index] = contacts[index].map_or(Vec4::ZERO, |(body, pose)| {
                let mut contact = grass_foot_contact(body, pose);
                contact.y = scene.anchors[index].y;
                contact
            });
            scene.contact_shape[index] = contacts[index].map_or(Vec4::ZERO,
                |(body, pose)| battle_contact_shape(body, pose));
        }
    }
    pub(super) fn generation(&self) -> Option<u64> {
        self.generation
    }
    /// Changes when the actual asynchronous terrain becomes ready too.
    pub(super) fn layout_key(&self) -> (Option<u64>, bool) {
        (self.generation, self.frozen.is_some())
    }
    pub(super) fn anchors(&self) -> Option<[Vec3; 2]> {
        self.frozen.as_ref().map(|f| f.anchors)
    }
    pub(super) fn accepted(&self) -> bool {
        self.accepted
    }
    pub(super) fn supported_bodies(
        &self,
        bodies: [Option<BattleBody>; 2],
        species: [Option<&str>; 2],
    ) -> Option<[Option<BattleBody>; 2]> {
        if !self.frozen.as_ref().is_some_and(|scene| scene.water) {
            return Some(bodies);
        }
        water_stance::supported_bodies(bodies, species)
    }
    pub(super) fn fog_range(&self) -> Option<(f32, f32)> {
        self.accepted.then_some(self.fog).flatten()
    }
    pub(super) fn reason(&self) -> Option<&'static str> {
        self.reason
    }
    pub(super) fn reject_layout(&mut self, reason: &'static str) {
        self.accepted = false;
        self.fog = None;
        self.reason = Some(reason);
    }

    /// Check transient source offsets against the same fixed terrain and body
    /// envelope. Failure uses the original source presentation for that frame;
    /// it never moves the camera, scales a species, or rebuilds geometry.
    pub(super) fn permits_displacements(&self, layout: &BattleSceneLayout, offsets: [Vec2; 2]) -> bool {
        let Some(scene) = self.frozen.as_ref().filter(|_| self.accepted) else { return true; };
        let Some(ground) = &scene.walking else { return true; };
        scene.bodies.iter().zip(layout.body_poses).enumerate().all(|(index, (body, pose))| {
            let (Some(body), Some(mut pose)) = (body, pose) else { return true; };
            pose.translation += layout.source_displacement(offsets[index]);
            ground.contains(*body, pose, scene.anchors[index].y)
        })
    }

    /// A finite genuine neighborhood is eligible only when every visible ray
    /// and every possible omitted shadow stays inside the retained evidence.
    pub(super) fn constrain_layout(
        &mut self,
        layout: &mut BattleSceneLayout,
        bodies: [Option<BattleBody>; 2],
        viewport: Vec2,
    ) -> bool {
        self.constrain_layout_with_trial(layout, bodies, viewport, camera_trial_enabled())
    }

    fn constrain_layout_with_trial(
        &mut self,
        layout: &mut BattleSceneLayout,
        bodies: [Option<BattleBody>; 2],
        viewport: Vec2,
        trial: bool,
    ) -> bool {
        // Retain the older explicit camera trial as an additional fallback.
        // Normal battle shots use `establish_camera` and the same terrain,
        // complete-body and frustum proofs below.
        if trial
            && let Some(mut candidate) = layout.three_quarter_camera(bodies)
            && self.constrain_camera(&mut candidate, bodies, viewport)
        {
            *layout = candidate;
            return true;
        }
        if self.constrain_camera(layout, bodies, viewport) {
            return true;
        }
        // Keep the existing primary and edge views as conservative fallbacks. At an authentic map
        // edge, try at most two fixed lateral views through the identical
        // proof below. Missing floor, scale or omitted geometry never relaxes.
        for clockwise in [false, true] {
            let Some(mut candidate) = layout.side_camera(bodies, clockwise) else {
                continue;
            };
            if self.constrain_camera(&mut candidate, bodies, viewport) {
                *layout = candidate;
                return true;
            }
        }
        false
    }

    /// Choose a proven view, preferring low angles, that shows the location behind the
    /// pair. Tall source geometry counts; flat ground and reeds cannot crowd
    /// out a city's buildings. The chosen side stays fixed during this battle.
    pub(super) fn establish_camera(&mut self, layout: &mut BattleSceneLayout, bodies: [Option<BattleBody>; 2], viewport: Vec2) -> bool {
        let _trace = crate::loading_trace::span("battle.establish_camera", || serde_json::json!({
            "derived_stage": self.frozen.as_ref().is_some_and(|scene| scene.derived_stage),
            "viewport": [viewport.x, viewport.y],
        }));
        // Search every supported stage at ordinary elevations first. A higher
        // view is a fallback only when that complete search finds no low shot.
        let original = layout.clone();
        for elevations in [&LOW_SHOT_ELEVATIONS[..], &HIGH_SHOT_ELEVATIONS[..]] {
            let mut candidate = original.clone();
            if self.establish_camera_tier(&mut candidate, bodies, viewport, elevations) {
                *layout = candidate;
                return true;
            }
        }
        false
    }

    fn establish_camera_tier(&mut self, layout: &mut BattleSceneLayout, bodies: [Option<BattleBody>; 2], viewport: Vec2, elevations: &[f32]) -> bool {
        let mut best = if self.establish_camera_at(layout, bodies, viewport, elevations) {
            let scene = self.frozen.as_ref().unwrap();
            let grass = scene.walking.as_ref().map_or(0, |ground| ground.grass_contacts(scene.anchors));
            Some((self.shot_score.unwrap() + grass as f32 * 40.0, layout.clone(), scene.anchors))
        } else { None };
        let pairs = self.frozen.as_ref().filter(|scene| scene.derived_stage)
            .and_then(|scene| scene.walking.as_ref().map(|ground| ground.nearby_stage_pairs(scene.source_anchors, bodies)))
            .unwrap_or_default();
        let mut unsupported_stages = 0_usize;
        let mut checked_stages = 0_usize;
        for anchors in pairs {
            let distance = anchors[0].distance(self.frozen.as_ref().unwrap().source_anchors[0]);
            let grass = self.frozen.as_ref().unwrap().walking.as_ref()
                .map_or(0, |ground| ground.grass_contacts(anchors));
            if best.as_ref().is_some_and(|(score, _, _)|
                maximum_shot_score() + grass as f32 * 40.0 - distance * 2.0 <= *score - 0.02) {
                continue;
            }
            let Some(candidate) = BattleSceneLayout::for_map_bodies(bodies, anchors, viewport) else { continue; };
            let mut candidate = candidate.reserve_ui(bodies, layout.visible_y_range());
            let scene = self.frozen.as_mut().unwrap();
            let Some(ground) = &scene.walking else { break; };
            if !bodies.into_iter().zip(candidate.body_poses).zip(anchors)
                .all(|((body, pose), anchor)| body.zip(pose).is_none_or(|(body, pose)| ground.contains(body, pose, anchor.y))) {
                unsupported_stages += 1;
                continue;
            }
            checked_stages += 1;
            scene.anchors = anchors;
            if self.establish_camera_at(&mut candidate, bodies, viewport, elevations) {
                let scene = self.frozen.as_ref().unwrap();
                let grass = scene.walking.as_ref().map_or(0, |ground| ground.grass_contacts(anchors));
                let score = self.shot_score.unwrap() - distance * 2.0 + grass as f32 * 40.0;
                if best.as_ref().is_none_or(|(previous, _, _)| score > *previous + 0.01) {
                    best = Some((score, candidate, anchors));
                }
            }
        }
        trace(format_args!("stage search: checked={checked_stages} unsupported={unsupported_stages} bodies={bodies:?}"));
        if let Some((_, mut candidate, anchors)) = best {
            self.frozen.as_mut().unwrap().anchors = anchors;
            if self.constrain_camera(&mut candidate, bodies, viewport) {
                *layout = candidate;
                return true;
            }
        }
        if let Some(scene) = &mut self.frozen { scene.anchors = scene.source_anchors; }
        false
    }

    fn establish_camera_at(&mut self, layout: &mut BattleSceneLayout, bodies: [Option<BattleBody>; 2], viewport: Vec2, elevations: &[f32]) -> bool {
        self.shot_score = None;
        let mut best: Option<(f32, BattleSceneLayout)> = None;
        let review_yaw=requested_review_yaw();
        for (yaw, elevation) in elevations.iter().copied().flat_map(|elevation|
            SHOT_YAWS.map(|yaw| (yaw, elevation))) {
            if review_yaw.is_some_and(|selected| selected!=yaw) { continue; }
            let mut compact = layout.clone();
            compact.distance = 0.0;
            let Some(original) = compact.battle_shot(bodies, yaw.to_radians(), elevation.to_radians()) else { continue; };
            let readable = original.readable_facing(bodies);
            // A readable body angle must not force a poorer backdrop. Score
            // both poses; retain the original when it gives the clearer place.
            for (mut candidate, facing_bonus) in std::iter::once((original, 0.0))
                .chain(readable.into_iter().map(|layout| (layout, 4.0))) {
            if !self.constrain_camera(&mut candidate, bodies, viewport) {
                trace(format_args!("shot rejection: yaw={yaw} elevation={elevation} reason={:?}", self.reason));
                continue;
            }
            let actor_extents: Vec<Vec2> = bodies.into_iter().zip(candidate.body_poses)
                .filter_map(|(body, pose)| body.zip(pose))
                .map(|(mut body, pose)| {
                    body.visual_bounds = None;
                    let mut lo = Vec2::splat(f32::INFINITY);
                    let mut hi = Vec2::splat(f32::NEG_INFINITY);
                    for point in body.corners(pose) {
                        let point = candidate.project_point(point, viewport);
                        lo = lo.min(point);
                        hi = hi.max(point);
                    }
                    hi - lo
                }).collect();
            let visible_height = actor_extents.iter().map(|extent| extent.y).fold(0.0_f32, f32::max);
            if visible_height < 0.15 { continue; }
            let scene = self.frozen.as_ref().unwrap();
            let forward = *candidate.camera.forward();
            let pair = (candidate.hit_anchors[0] + candidate.hit_anchors[1]) * 0.5;
            let pair_depth = (pair - candidate.camera.translation).dot(forward);
            // Projection is invariant across every source piece in this shot.
            // Once all 32 cells are covered, further pieces cannot change its
            // score. Keep the exhaustive shot/ground search and admission proof.
            let view = candidate.camera.to_matrix().inverse();
            let mut scenery_cells = 0_u32;
            for bounds in scene.backdrop_bounds.iter().copied()
                .chain(scene.objects.values().filter(|object| !object.faded
                    && object.bounds.max.y - object.bounds.min.y > 0.8)
                    .flat_map(|object| object.backdrop.iter().copied())) {
                if visible_bounds(bounds, candidate.camera, viewport, candidate.far) {
                    scenery_cells |= backdrop_cells_projected(bounds, &candidate, viewport, pair_depth, view);
                    if scenery_cells == u32::MAX { break; }
                }
            }
            let scenery_score = scenery_cells.count_ones() as f32;
            // Never reward an architectural background at the cost of looking
            // through a faded building or shrinking the pair into the distance.
            let foreground = scene.objects.values().filter(|object| object.faded
                && visible_bounds(object.bounds, candidate.camera, viewport, candidate.far)).count();
            // Both animals face along the encounter corridor. A view far
            // behind either one hides its face and foreshortens the fight;
            // prefer readable profiles before rewarding more distant walls.
            // Rear views remain available when the actual map requires them.
            let rear_view = yaw.to_radians().cos().abs();
            let score = scenery_score.ln_1p() * 20.0 - foreground as f32 * 1000.0
                - rear_view * 50.0 - candidate.distance * 0.5 - elevation * 0.5 + facing_bonus
                + actor_readability_score(&actor_extents);
            if best.as_ref().is_none_or(|(previous, _)| score > *previous + 0.01) {
                best = Some((score, candidate));
            }
            }
        }
        if let Some((score, mut candidate)) = best {
            if self.constrain_camera(&mut candidate, bodies, viewport) {
                self.shot_score = Some(score);
                *layout = candidate;
                return true;
            }
        }
        self.accepted = false;
        self.fog = None;
        self.reason = Some(if elevations[0] > 48.0 { "no readable higher shot on checked ground" } else { "no readable low shot on checked ground" });
        false
    }

    pub(super) fn constrain_camera(
        &mut self,
        layout: &mut BattleSceneLayout,
        bodies: [Option<BattleBody>; 2],
        viewport: Vec2,
    ) -> bool {
        self.accepted = false;
        self.fog = None;
        let Some(scene) = &mut self.frozen else {
            return false;
        };
        // Match the fragment shader: wholly discarded foreground blades cannot
        // obstruct a low shot. Crossing or background bounds remain blockers.
        let grass_cutoff = foreground_grass_cutoff(bodies, layout);
        let grass_blocks = |end| scene.grass_lens_blockers.visible_grass_blocks(
            layout.camera.translation, end, layout.camera, grass_cutoff);
        if grass_blocks(layout.camera.translation + *layout.camera.forward() * 0.30) {
            self.reason = Some("battle camera intersects retained grass near the lens");
            return false;
        }
        let composition = (layout.hit_anchors[0] + layout.hit_anchors[1]) * 0.5;
        if grass_blocks(layout.camera.translation.lerp(composition, 0.60)) {
            self.reason = Some("foreground grass obscures the battle composition");
            return false;
        }
        let Some(limit) = scene.authentic.camera_limit(layout.camera, viewport) else {
            self.reason = Some("battle camera leaves the authentic retained neighborhood");
            return false;
        };
        if scene.derived_stage && (scene.static_blockers.blocker(
            layout.hit_anchors[0], layout.hit_anchors[1],
            scene.anchors[0].y.min(scene.anchors[1].y) + 0.25).is_some()
            || scene.objects.values().any(|object| object.blocks(layout.hit_anchors[0], layout.hit_anchors[1]))) {
            self.reason = Some("scenery separates the battlers");
            return false;
        }
        for fraction in [0.25, 0.5, 0.75] {
            let point = layout.hit_anchors[0].lerp(layout.hit_anchors[1], fraction);
            if scene.static_blockers.blocker(layout.camera.translation, point, point.y * 0.5).is_some()
                || scene.objects.values().any(|object| object.blocks(layout.camera.translation, point)) {
                self.reason = Some("foreground scenery divides the battle composition");
                return false;
            }
        }
        for (index, body) in bodies.into_iter().enumerate() {
            let Some(body) = body else { continue; };
            let upper = layout.hit_anchors[index] + Vec3::Y * body.reference_height * body.scale.y * 0.25;
            let mut sight_points = vec![layout.hit_anchors[index], upper,
                layout.hit_anchors[index] + Vec3::Y * 0.5];
            if let Some(pose) = layout.body_poses[index] {
                // Center rays miss a shelf clipping a wing or tail. Check the
                // neutral body's edges as well, above the contact floor.
                sight_points.extend(body.sight_corners(pose).into_iter()
                    .filter(|point| point.y > scene.anchors[index].y + 0.25));
            }
            if scene.objects.values().any(|object|
                object.bounds.max.y > scene.anchors[index].y + 0.25
                    && sight_points.iter().any(|&point| object.blocks(layout.camera.translation, point))) {
                self.reason = Some("foreground scenery obstructs the battle shot");
                return false;
            }
            if let Some(bounds) = sight_points.iter().find_map(|&point|
                scene.static_blockers.blocker(layout.camera.translation, point, scene.anchors[index].y + 0.25))
            {
                trace(format_args!("static sight rejection: actor={index} bounds={bounds:?} eye={:?} anchor={:?}",
                    layout.camera.translation, layout.hit_anchors[index]));
                self.reason = Some("static terrain occludes a battler");
                return false;
            }
        }
        let view = layout.camera.rotation.inverse();
        let mut deepest: f32 = 0.0;
        let mut radial_distance: f32 = 0.0;
        for (index, (body, pose)) in bodies.into_iter().zip(layout.body_poses).enumerate() {
            if let (Some(body), Some(pose)) = (body, pose) {
                if scene.walking.as_ref().is_some_and(|ground| !ground.contains(body, pose, scene.anchors[index].y)) {
                    self.reason = Some("complete animated battler exceeds checked source ground");
                    return false;
                }
                for point in body.corners(pose) {
                    if !scene.authentic.point(point) {
                        self.reason = Some("battler exceeds authentic terrain bounds");
                        return false;
                    }
                    deepest = deepest.max(-(view * (point - layout.camera.translation)).z);
                    radial_distance = radial_distance.max(point.distance(layout.camera.translation));
                }
            }
        }
        // Perspective near/far planes measure view-axis depth, not radial
        // distance to off-axis wing tips. Use the same units as camera_limit
        // and the projection, retaining every animated corner and rear margin.
        let maximum_far = limit.min(deepest + 32.0).min(layout.far);
        if !deepest.is_finite() || deepest <= 0.0 || maximum_far < deepest + 2.0 {
            self.reason = Some("authentic terrain is too small for the complete battlers");
            return false;
        }
        // Keep literal buildings, trees and cavern walls visible behind the
        // pair. Prefer the deeper authentic view, reducing depth only when
        // omitted source geometry or its shadow would enter that view.
        let far = [32.0, 20.0, 8.0].into_iter()
            .map(|depth| maximum_far.min(deepest + depth))
            .find(|&far| !scene.omitted.visible_or_shadow(layout.camera, viewport, far, scene.ground));
        let Some(far) = far else {
            self.reason = Some("omitted border or mutable terrain could affect the battle view");
            return false;
        };
        // Bevy fog is radial while the retained frustum is axial. Fog must
        // finish inside that proven range and start beyond every wing tip.
        if far <= radial_distance + 0.6 {
            self.reason = Some("authentic terrain cannot keep complete battlers clear of fog");
            return false;
        }
        trace(format_args!(
            "fit camera={:?} authentic={:?} viewport={viewport:?} body_depth={deepest} far={far}",
            layout.camera, scene.authentic
        ));
        // Each instance or authored group owns one opacity. An AABB test is
        // conservative for outdoor trees; it never removes a surface fragment.
        for object in scene.objects.values_mut() {
            object.faded = layout.hit_anchors.iter().any(|&point| {
                object.blocks(layout.camera.translation, point)
                    || object
                        .blocks(layout.camera.translation, point + Vec3::Y * 0.5)
            });
        }
        for index in 0..2 {
            let aperture = match (bodies[index], layout.body_poses[index]) {
                (Some(body), Some(pose)) => grass_body_aperture(body, pose, layout.camera),
                _ => (Vec4::ZERO, Vec4::ZERO),
            };
            scene.clearance[index] = aperture.0;
            scene.clearance_shape[index] = aperture.1;
            scene.grass_contact[index] = match (bodies[index], layout.body_poses[index]) {
                (Some(body), Some(pose)) => grass_foot_contact(body, pose),
                _ => Vec4::ZERO,
            };
            scene.contact_shape[index] = match (bodies[index], layout.body_poses[index]) {
                (Some(body), Some(pose)) => battle_contact_shape(body, pose),
                _ => Vec4::ZERO,
            };
        }
        scene.bodies = bodies;
        layout.far = far;
        self.fog = Some(((radial_distance + 8.0).min(far - 0.5), far - 0.1));
        self.reason = None;
        self.accepted = true;
        true
    }
}

/// Conservative camera-plane rejection. False positives cause a fallback,
/// never a visible hole. The far plane also bounds the opaque fog horizon.
fn visible_bounds(bounds: Bounds, camera: Transform, viewport: Vec2, far: f32) -> bool {
    let view = camera.rotation.inverse();
    let tangent = (CAMERA_FOV * 0.5).tan();
    let horizontal = tangent * viewport.x / viewport.y;
    let points: [Vec3; 8] = std::array::from_fn(|i| {
        view * (Vec3::new(
            if i & 1 == 0 {
                bounds.min.x
            } else {
                bounds.max.x
            },
            if i & 2 == 0 {
                bounds.min.y
            } else {
                bounds.max.y
            },
            if i & 4 == 0 {
                bounds.min.z
            } else {
                bounds.max.z
            },
        ) - camera.translation)
    });
    let outside = |plane: fn(Vec3, f32, f32) -> f32| {
        points.iter().all(|&p| plane(p, horizontal, tangent) < 0.0)
    };
    !(points.iter().all(|p| -p.z < 0.1)
        || points.iter().all(|p| -p.z > far)
        || outside(|p, h, _| -p.z * h + p.x)
        || outside(|p, h, _| -p.z * h - p.x)
        || outside(|p, _, v| -p.z * v + p.y)
        || outside(|p, _, v| -p.z * v - p.y))
}

fn strict_foot(frame: &VisualWorldFrame, heights: &[f32], foot: Vec2) -> Option<Vec3> {
    // resolved_footing_height intentionally falls back to zero off-grid for the
    // overworld. An encounter must prove an in-range, actually built support.
    let sample = crate::tile_at_visual_point(frame, foot + Vec2::Y * 0.01)?;
    let count = (frame.grid_size.x as usize).checked_mul(frame.grid_size.y as usize)?;
    if heights.len() != count {
        return None;
    }
    let index = (sample.y as usize)
        .checked_mul(frame.grid_size.x as usize)?
        .checked_add(sample.x as usize)?;
    let height = *heights.get(index)?;
    height
        .is_finite()
        .then(|| crate::visual_point_to_voxel(foot, height))
}

#[cfg(test)]
fn resolve_context(
    location: &VisualBattleLocation,
    cache: &TerrainRevisionCache,
) -> Result<([Vec3; 2], Transform, AuthenticBounds), &'static str> {
    resolve_context_with_images(location, cache, None)
}

fn resolve_context_with_images(
    location: &VisualBattleLocation,
    cache: &TerrainRevisionCache,
    images: Option<&Assets<Image>>,
) -> Result<([Vec3; 2], Transform, AuthenticBounds), &'static str> {
    let anchors = location
        .anchors
        .as_ref()
        .ok_or("encounter has no checked terrain anchors")?;
    let built = cache
        .built_frame
        .as_ref()
        .ok_or("actual terrain has not finished building")?;
    if built.source_map_size_core_tiles != location.source_map_size_core_tiles {
        return Err("built source acreage does not match the checked encounter");
    }
    let original = cache
        .built_source_texture
        .as_ref()
        .unwrap_or(&built.map_texture);
    if original != &anchors.terrain.map_texture {
        return Err("terrain atlas provenance does not match");
    }
    let mut evidence = anchors.terrain.clone();
    // A build owns an immutable atlas copy. Its original handle is recorded at
    // submission, so this substitution does not weaken any geometry evidence.
    evidence.map_texture = built.map_texture.clone();
    let correspondence = if evidence.matches_built_frame(built) {
        None
    } else if matches!(location.target, crystal_render_api::VisualBattleTarget::WalkingGrass { .. } | crystal_render_api::VisualBattleTarget::SurfWater { .. } | crystal_render_api::VisualBattleTarget::Trainer { .. } | crystal_render_api::VisualBattleTarget::FishingWater { .. } | crystal_render_api::VisualBattleTarget::ScriptedGround { .. }) {
        // A scrolling overworld or animated shoreline can retain an older mesh grid. Prove every
        // overlapping source cell against those actual built inputs, then use
        // the same rigid translation as the live terrain renderer. A desired
        // asynchronous cache key never participates in this correspondence.
        Some(retained_correspondence::resolve_with_images(
            &evidence, built, cache.built_profiles.as_deref(), images,
        )?)
    } else {
        return Err("actual built terrain does not match encounter evidence");
    };
    let center = correspondence.as_ref().map_or(built.center, |c| c.center);
    if built.tile_size != Vec2::splat(SOURCE_CELL_SIZE) {
        return Err("unsupported source terrain scale");
    }
    let map = location
        .source_map_size_core_tiles
        .ok_or("source map acreage is unknown")?;
    if map.min_element() == 0 || map.max_element() > i32::MAX as u32 / 2 {
        return Err("invalid source map extent");
    }
    for point in [location.source.core_tile, location.target.core_tile()] {
        if point.cmplt(IVec2::ZERO).any() || point.cmpge(map.as_ivec2()).any() {
            return Err("encounter core tile is outside source map");
        }
    }
    // Footing indices belong to the actual built grid. Translate only for
    // lookup and return both feet to the witnessed world coordinates once.
    let sample_delta = built.center - center;
    let restore = Vec3::new(sample_delta.x, 0.0, -sample_delta.y);
    let feet = [anchors.source_foot, anchors.target_foot].map(|foot| {
        strict_foot(built, &cache.built_footing_heights, foot + sample_delta)
            .map(|point| point - restore)
    });
    let feet = [
        feet[0].ok_or("source footing is outside actual built terrain")?,
        feet[1].ok_or("target footing is outside actual built terrain")?,
    ];
    let midpoint = (feet[0] + feet[1]) * 0.5;
    let rebase = Transform::from_translation(-midpoint * RETAINED_SCALE)
        .with_scale(Vec3::splat(RETAINED_SCALE));
    let grid = built.grid_size.as_vec2();
    let start = built.grid_origin.as_vec2();
    let mut lo = start.max(Vec2::ZERO);
    let mut hi = (start + grid).min(map.as_vec2() * 2.0);
    if let Some(correspondence) = correspondence {
        lo = lo.max(correspondence.min.as_vec2());
        hi = hi.min(correspondence.max.as_vec2());
    }
    if hi.cmple(lo).any() {
        return Err("source map and actual terrain do not intersect");
    }
    let northwest = Vec2::new(center.x, -center.y) - grid * built.tile_size * 0.5;
    let transform_xz = |p: Vec2| (p - Vec2::new(midpoint.x, midpoint.z)) * RETAINED_SCALE;
    let authentic = AuthenticBounds {
        min: transform_xz(northwest + (lo - start) * built.tile_size),
        max: transform_xz(northwest + (hi - start) * built.tile_size),
    };
    let feet = feet.map(|p| rebase.transform_point(p));
    if feet.iter().any(|&p| !authentic.point(p)) {
        return Err("encounter feet lack authentic source support");
    }
    Ok((
        feet,
        rebase.mul_transform(Transform::from_xyz(center.x, 0.0, -center.y)),
        authentic,
    ))
}

/// A projected envelope fits the actual complete animation more closely than
/// a world bounding sphere, avoiding circular clearings around slender bodies.
// Contact shading belongs on the retained blades too: a flat ground shadow
// alone is hidden by tall grass, making a grounded body appear suspended.
fn grass_foot_contact(mut body: BattleBody, pose: Transform) -> Vec4 {
    body.visual_bounds = None;
    let points = body.corners(pose);
    let min = points.iter().copied().fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = points.iter().copied().fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let center = (min + max) * 0.5;
    Vec4::new(center.x, min.y, center.z,
        (Vec2::new(max.x - min.x, max.z - min.z).length() * 0.55).max(0.04))
}

// Keep long/thin bodies' contact patches narrow, independently of camera yaw
// and full animation bounds. The shape follows the neutral sculpture's axes.
fn battle_contact_shape(body: BattleBody, pose: Transform) -> Vec4 {
    let extent = (body.max - body.min) * pose.scale;
    let axis = pose.rotation * Vec3::X;
    let axis = Vec2::new(axis.x, axis.z).try_normalize().unwrap_or(Vec2::X);
    Vec4::new(axis.x, axis.y, (extent.x * 0.60).max(0.04), (extent.z * 0.60).max(0.04))
}

fn foreground_grass_cutoff(bodies: [Option<BattleBody>; 2], layout: &BattleSceneLayout) -> f32 {
    let view = layout.camera.compute_affine().inverse();
    bodies.into_iter().zip(layout.body_poses).filter_map(|(body, pose)| {
        let (clearance, shape) = grass_body_aperture(body?, pose?, layout.camera);
        (clearance.w > 0.0).then(|| view.transform_point3(clearance.truncate()).z
            - if shape.z > 0.0 { shape.z } else { clearance.w })
    }).fold(f32::INFINITY, f32::min)
}

fn grass_bounds_discarded(bounds: Bounds, camera: Transform, cutoff: f32) -> bool {
    let view = camera.compute_affine().inverse();
    (0..8).all(|i| view.transform_point3(Vec3::new(
        if i & 1 == 0 { bounds.min.x } else { bounds.max.x },
        if i & 2 == 0 { bounds.min.y } else { bounds.max.y },
        if i & 4 == 0 { bounds.min.z } else { bounds.max.z },
    )).z > cutoff)
}

fn grass_body_aperture(body: BattleBody, pose: Transform, camera: Transform) -> (Vec4, Vec4) {
    let points = body.corners(pose);
    let min = points.iter().copied().fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = points.iter().copied().fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let center = (min + max) * 0.5;
    let radius = (max - min).length() * 0.5 + 0.25;
    let view = camera.compute_affine().inverse();
    let center_view = view.transform_point3(center);
    let mut lo = Vec2::splat(f32::INFINITY);
    let mut hi = Vec2::splat(f32::NEG_INFINITY);
    let mut depth = 0.0_f32;
    for point in points {
        let point = view.transform_point3(point);
        if !point.is_finite() || point.z >= -0.001 {
            return (center.extend(radius), Vec4::ZERO);
        }
        let projected = point.truncate() * center_view.z / point.z;
        depth = depth.max((point.z - center_view.z).abs());
        lo = lo.min(projected);
        hi = hi.max(projected);
    }
    let projected_center = (lo + hi) * 0.5;
    let radii = ((hi - lo) * 0.5 + Vec2::splat(0.15)) * std::f32::consts::SQRT_2;
    let center = camera.transform_point(Vec3::new(projected_center.x, projected_center.y, center_view.z));
    // Keep grass behind the actual depth envelope. The old bounding sphere
    // removed extra foliage behind slender, tall bodies. Preserve a shallow
    // layer of existing blade roots rather than exposing a bare oval floor.
    // Retain a real foreground fringe over the lower body. Compare its world
    // height to the neutral feet, not height above each individual blade root:
    // neighboring roots can have different elevations on the source terrain.
    let mut neutral = body;
    neutral.visual_bounds = None;
    let neutral_points = neutral.corners(pose);
    let foot = neutral_points.iter().map(|point| point.y).fold(f32::INFINITY, f32::min);
    let top = neutral_points.iter().map(|point| point.y).fold(f32::NEG_INFINITY, f32::max);
    let root_cover = ((top - foot) * 0.22).min(0.32);
    (center.extend(radius), Vec4::new(radii.x, radii.y, depth + 0.15, foot + root_cover))
}

fn collect_leaves(
    world: &World,
    entity: Entity,
    pose: Transform,
    object: Option<Entity>,
    out: &mut Vec<FrozenLeaf>,
) -> Result<(), &'static str> {
    if world.get_entity(entity).is_err() {
        return Err("actual terrain hierarchy is not ready");
    }
    if world.get::<SyntheticTerrainApron>(entity).is_some() {
        return Ok(());
    }
    if world.get::<crate::occluder_fade::OpaqueObjectLeaf>(entity).is_some() {
        return Ok(());
    }
    // The build-time clip proves that these triangles contain no actual source
    // acreage. Their aggregate AABB may span it, but their geometry never does.
    // They remain in the overworld and never become a battle caster or surface.
    if world.get::<crate::terrain_batches::AuthenticDomain>(entity)
        == Some(&crate::terrain_batches::AuthenticDomain::Outside)
    {
        return Ok(());
    }
    if let Some(mesh) = crate::occluder_fade::retained_mesh(world, entity) {
        let material = world
            .get::<MeshMaterial3d<VoxelMaterial>>(entity)
            .ok_or("terrain leaf has no voxel material")?;
        let aabb = world.get::<Aabb>(entity).copied().or_else(|| {
            world
                .get_resource::<Assets<Mesh>>()?
                .get(&mesh)?
                .get_aabb()
        });
        let Some(aabb) = aabb else {
            return Err("terrain leaf bounds have not been published");
        };
        let bounds = Bounds::from_aabb(aabb, pose).ok_or("terrain leaf has nonfinite bounds")?;
        out.push(FrozenLeaf {
            mesh: mesh.clone(),
            material: material.0.clone(),
            transform: pose,
            aabb,
            bounds,
            object,
            coverage: world.get::<crate::terrain_batches::TerrainBatchCoverage>(entity)
                .filter(|coverage| coverage.mesh == mesh.id())
                .map(|coverage| coverage.bounds.clone()),
            grass_coverage: world.get::<crate::terrain_batches::TerrainBatchCoverage>(entity)
                .filter(|coverage| coverage.mesh == mesh.id())
                .map(|coverage| coverage.grass_bounds.clone()),
            sight_coverage: world.get::<crate::terrain_batches::TerrainBatchCoverage>(entity)
                .filter(|coverage| coverage.mesh == mesh.id())
                .map(|coverage| coverage.sight_bounds.clone()),
        });
    }
    if let Some(children) = world.get::<Children>(entity) {
        for child in children.iter() {
            let local = world
                .get::<Transform>(child)
                .copied()
                .ok_or("terrain child lacks a local transform")?;
            collect_leaves(world, child, pose.mul_transform(local), object, out)?;
        }
    }
    Ok(())
}

/// Exclusive only while pinning a new generation; held frames touch no assets.
/// Schedule after sync_voxel_view and before the anchored layout selection.
// Asset publication can complete without replacing the terrain roots. Do not
// latch a temporary miss as a permanent rejection of this encounter's map.
fn preparation_pending(reason: &str) -> bool {
    matches!(reason,
        "actual battle groves are warming"
        | "actual terrain hierarchy is not ready"
        | "terrain leaf bounds have not been published"
        | "actual terrain instances are not ready"
        | "actual static terrain is not ready"
        | "actual animated terrain is not ready"
        | "actual animated mesh handle is missing"
        | "actual animated terrain CPU mesh is unavailable"
        | "voxel materials are unavailable"
        | "terrain images are unavailable"
        | "retained terrain material is unavailable"
        | "retained terrain image is unavailable")
}

pub(super) fn prepare(world: &mut World) {
    let location = world
        .get_resource::<BattleLocationFrame>()
        .and_then(|f| f.location.clone());
    let generation = location.as_ref().map(|l| l.generation);
    if world
        .get_resource::<EncounterTerrain>()
        .is_some_and(|state| {
            state.generation == generation && (state.frozen.is_some() || generation.is_none())
        })
    {
        return;
    }
    let mut state = world
        .remove_resource::<EncounterTerrain>()
        .unwrap_or_default();
    if state.generation != generation {
        if let Some(frozen) = state.frozen.take() {
            if let Ok(root) = world.get_entity_mut(frozen.root) {
                root.despawn();
            }
            if let Some(mut assets) = world.get_resource_mut::<Assets<Mesh>>() {
                for mesh in frozen.meshes {
                    assets.remove(mesh.id());
                }
            }
            if let Some(mut assets) = world.get_resource_mut::<Assets<VoxelMaterial>>() {
                for material in frozen.materials {
                    assets.remove(material.handle.id());
                }
            }
            if let Some(mut assets) = world.get_resource_mut::<Assets<Image>>() {
                for image in frozen.images {
                    assets.remove(image.id());
                }
            }
        }
        state = EncounterTerrain {
            generation,
            ..default()
        };
    }
    if state.frozen.is_none() {
        if let Some(location) = location {
            let ready = world
                .get_resource::<TerrainRevisionCache>()
                .ok_or("voxel terrain cache is unavailable")
                .and_then(|cache| {
                    let (_, pose, authentic) = resolve_context_with_images(&location, cache, world.get_resource::<Assets<Image>>())?;
                    Ok(([
                        cache
                            .instances_root
                            .ok_or("terrain instances are not ready")?,
                        cache
                            .textured_entity
                            .ok_or("textured terrain is not ready")?,
                        cache.solid_entity.ok_or("solid terrain is not ready")?,
                    ], pose, authentic))
                });
            match ready {
                Ok((roots, pose, authentic)) if state.attempted_roots != Some(roots) => {
                    if warm_streamed_groves(world, pose, authentic) {
                        state.reason = Some("actual battle groves are warming");
                    } else {
                        crate::occluder_fade::prepare_retention_meshes(world, roots[0]);
                        state.attempted_roots = Some(roots);
                        match freeze(world, &location) {
                        Ok(frozen) => {
                            state.frozen = Some(frozen);
                            state.reason = None;
                        }
                        Err(reason) => {
                            if preparation_pending(reason) {
                                state.attempted_roots = None;
                            }
                            state.reason = Some(reason);
                        }
                        }
                    }
                }
                Ok(_) => {}
                Err(reason) => {
                    state.reason = Some(reason);
                }
            }
        }
    }
    world.insert_resource(state);
}

fn warm_streamed_groves(world: &mut World, pose: Transform, authentic: AuthenticBounds) -> bool {
    let _trace = crate::loading_trace::span("battle.warm_groves", || serde_json::json!({}));
    if !world.resource::<TerrainRevisionCache>().groves.is_some() { return false; }
    let inverse = pose.to_matrix().inverse();
    let local = |point: Vec2| {
        let point = inverse.transform_point3(Vec3::new(point.x, 0.0, point.y));
        Vec2::new(point.x, point.z)
    };
    let area = Rect::from_corners(local(authentic.min), local(authentic.max));
    let mut queue = bevy::ecs::world::CommandQueue::default();
    let pending = world.resource_scope(|world, mut cache: Mut<TerrainRevisionCache>| {
        world.resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                let mut commands = Commands::new(&mut queue, world);
                crate::grove_stream::warm_battle(&mut cache, area, &mut commands, &mut meshes, &mut materials)
            })
        })
    });
    queue.apply(world);
    pending
}

/// Select complete original triangles from a source terrain batch.
/// Vertex streams stay byte-identical, including UVs, normals and cutaway data.
/// Unreferenced outside vertices cannot draw; the copied AABB uses selected
/// indices only. A straddling triangle is omitted and remains in the existing
/// view/shadow proof, so this never invents a clipped edge or fills missing land.
fn select_authentic_triangles(
    source: &Mesh,
    pose: Transform,
    authentic: AuthenticBounds,
) -> Result<(Option<(Mesh, Aabb, Bounds)>, Vec<Bounds>), &'static str> {
    use bevy::render::{
        mesh::{Indices, VertexAttributeValues},

        render_resource::PrimitiveTopology,
    };
    use std::collections::BTreeMap;
    let Some(VertexAttributeValues::Float32x3(positions)) =
        source.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Err("terrain batch positions are unavailable");
    };
    let indices = source
        .indices()
        .ok_or("terrain batch requires indexed triangles")?;
    if source.primitive_topology() != PrimitiveTopology::TriangleList
        || indices.len() % 3 != 0
        || source
            .attributes()
            .any(|(_, values)| values.len() != positions.len())
        || indices.iter().any(|i| i >= positions.len())
    {
        return Err("terrain batch has invalid triangle attributes");
    }
    let mut kept = Vec::<u32>::new();
    let mut local = Bounds {
        min: Vec3::splat(f32::INFINITY),
        max: Vec3::splat(f32::NEG_INFINITY),
    };
    // Four battle units are one core tile. Spatial buckets prevent distant
    // excluded flowers from forming one false AABB across the genuine corridor.
    // Complete actual triangle bounds remain in each bucket, including straddles.
    const OMITTED_BUCKET_WIDTH: f32 = 4.0;
    const MAX_OMITTED_BUCKETS: usize = 4096;
    let mut excluded = BTreeMap::<(i32, i32), Bounds>::new();
    let mut indices_iter = indices.iter();
    while let Some(a) = indices_iter.next() {
        let triangle = [
            a,
            indices_iter.next().unwrap(),
            indices_iter.next().unwrap(),
        ];
        let points = triangle.map(|i| Vec3::from_array(positions[i]));
        let world = points.map(|p| pose.transform_point(p));
        if world.iter().any(|p| !p.is_finite()) {
            return Err("terrain batch has nonfinite vertices");
        }
        let bounds = Bounds {
            min: world[0].min(world[1]).min(world[2]),
            max: world[0].max(world[1]).max(world[2]),
        };
        if authentic.contains(bounds) {
            kept.extend(triangle.map(|i| i as u32));
            for point in points {
                local.min = local.min.min(point);
                local.max = local.max.max(point);
            }
        } else {
            let center = (bounds.min * 0.5 + bounds.max * 0.5) / OMITTED_BUCKET_WIDTH;
            let key = (center.x.floor() as i32, center.z.floor() as i32);
            excluded
                .entry(key)
                .and_modify(|current| *current = current.union(bounds))
                .or_insert(bounds);
            if excluded.len() > MAX_OMITTED_BUCKETS {
                return Err("terrain batch omission coverage exceeds its bound");
            }
        }
    }
    let selected = if kept.is_empty() {
        None
    } else {
        let mut copy = source.clone();
        // Preserve the original index format and sequence; every other vertex
        // attribute remains untouched. The frozen copy is extracted only once.
        copy.insert_indices(match indices {
            Indices::U16(_) => Indices::U16(kept.into_iter().map(|i| i as u16).collect()),
            Indices::U32(_) => Indices::U32(kept),
        });
        #[cfg(target_arch = "wasm32")]
        crate::browser_memory::align_indices(&mut copy);
        copy.asset_usage = RenderAssetUsages::RENDER_WORLD;
        let aabb = Aabb::from_min_max(local.min, local.max);
        let bounds =
            Bounds::from_aabb(aabb, pose).ok_or("selected terrain batch bounds are invalid")?;
        Some((copy, aabb, bounds))
    };
    Ok((selected, excluded.into_values().collect()))
}

fn freeze(world: &mut World, location: &VisualBattleLocation) -> Result<FrozenScene, &'static str> {
    let _trace = crate::loading_trace::span("battle.freeze_terrain", || serde_json::json!({
        "map": location.source.map_id.as_ref(),
        "generation": location.generation,
    }));
    let cache = world
        .get_resource::<TerrainRevisionCache>()
        .ok_or("voxel terrain cache is unavailable")?;
    let (anchors, root_pose, authentic) = resolve_context_with_images(location, cache, world.get_resource::<Assets<Image>>())?;
    let walking = WalkingGround::from_context(location, cache, root_pose)?;
    let mut leaves = Vec::new();
    let mut object_surfaces = HashMap::new();
    let mut omitted = Vec::new();
    if let Some(stream) = &cache.groves {
        for (min, max) in stream.missing_bounds() {
            omitted.push(Bounds::from_aabb(Aabb::from_min_max(min, max), root_pose)
                .ok_or("unloaded source grove has invalid coverage")?);
        }
    }
    let instances = cache
        .instances_root
        .ok_or("actual terrain instances are not ready")?;
    if let Some(children) = world.get::<Children>(instances) {
        for child in children.iter() {
            let local = world
                .get::<Transform>(child)
                .copied()
                .ok_or("terrain instance lacks a transform")?;
            if let Some(surface) = crate::occluder_fade::retained_surface(world, child) {
                object_surfaces.insert(child, (surface, root_pose.mul_transform(local)));
            }
            collect_leaves(
                world,
                child,
                root_pose.mul_transform(local),
                Some(child),
                &mut leaves,
            )?;
        }
    }
    for root in [cache.textured_entity, cache.solid_entity] {
        collect_leaves(
            world,
            root.ok_or("actual static terrain is not ready")?,
            root_pose,
            None,
            &mut leaves,
        )?;
    }
    // Static handles are immutable. Only the two flower domains are rewritten
    // under stable world handles, so copy their exact current CPU meshes once.
    let mut mutable_mesh_copies = HashMap::<AssetId<Mesh>, Mesh>::new();
    for (entity, mesh, aabb) in [
        (
            cache.animated_textured_entity,
            cache.animated_textured_mesh.as_ref(),
            cache.built_animated_bounds[0],
        ),
        (
            cache.animated_solid_entity,
            cache.animated_solid_mesh.as_ref(),
            cache.built_animated_bounds[1],
        ),
    ] {
        let Some(aabb) = aabb else {
            continue;
        };
        Bounds::from_aabb(aabb, root_pose).ok_or("animated terrain bounds are invalid")?;
        let entity = entity.ok_or("actual animated terrain is not ready")?;
        let mesh = mesh.ok_or("actual animated mesh handle is missing")?;
        if world.get::<Mesh3d>(entity).map(|handle| &handle.0) != Some(mesh) {
            return Err("actual animated terrain handle does not match its cache");
        }
        let material = world
            .get::<MeshMaterial3d<VoxelMaterial>>(entity)
            .ok_or("actual animated terrain material is missing")?
            .0.clone();
        let source = world
            .get_resource::<Assets<Mesh>>()
            .and_then(|assets| assets.get(mesh))
            .ok_or("actual animated terrain CPU mesh is unavailable")?;
        let (selected, excluded) = select_authentic_triangles(source, root_pose, authentic)?;
        trace(format_args!(
            "animated selection kept={} omitted_buckets={}",
            selected.is_some(),
            excluded.len()
        ));
        omitted.extend(excluded);
        let Some((copy, aabb, bounds)) = selected else {
            continue;
        };
        let sight_coverage = crate::terrain_batches::triangle_sight_coverage(&copy);
        let grass_coverage = crate::terrain_batches::triangle_grass_coverage(&copy);
        mutable_mesh_copies.insert(mesh.id(), copy);
        leaves.push(FrozenLeaf {
            mesh: mesh.clone(),
            material,
            transform: root_pose,
            aabb,
            bounds,
            object: None,
            coverage: None,
            sight_coverage,
            grass_coverage,
        });
    }
    let mut kept = Vec::new();
    for mut leaf in leaves {
        if authentic.contains(leaf.bounds) {
            kept.push(leaf);
        } else {
            // A batch's union can cross the witnessed neighborhood even when
            // its real near wall/floor is completely inside. Reuse the exact
            // original GPU mesh. The small build-time coverage separates its
            // uncertain pieces for the full frustum AND shadow proof below.
            // If any such piece could affect this view, the camera is rejected.
            // No faces are cut or redrawn, and object groups remain whole.
            if leaf.object.is_none() && let Some(coverage) = &leaf.coverage {
                let mut inside: Option<Bounds> = None;
                for aabb in coverage.iter().copied() {
                    let bounds = Bounds::from_aabb(aabb, leaf.transform)
                        .ok_or("terrain batch coverage has nonfinite bounds")?;
                    let (known, exterior) = authentic.partition_coverage(bounds);
                    omitted.extend(exterior);
                    if let Some(bounds) = known {
                        inside = Some(inside.map_or(bounds, |previous| previous.union(bounds)));
                    }
                }
                if let Some(bounds) = inside {
                    trace(format_args!("retained original border batch mesh={:?} coverage={} bounds={bounds:?}", leaf.mesh.id(), coverage.len()));
                    leaf.bounds = bounds;
                    kept.push(leaf);
                }
                continue;
            }
            trace(format_args!(
                "omitted border mesh={:?} object={:?} bounds={:?}",
                leaf.mesh.id(),
                leaf.object,
                leaf.bounds
            ));
            omitted.push(leaf.bounds);
        }
    }
    if kept.is_empty() {
        return Err("no complete authentic terrain leaves are retained");
    }
    let mut objects = HashMap::<Entity, FrozenObject>::new();
    let mut static_blockers = Vec::new();
    let mut grass_lens_blockers = Vec::new();
    let mut ground = anchors[0].y.min(anchors[1].y);
    for leaf in &kept {
        ground = ground.min(leaf.bounds.min.y);
        for aabb in leaf.grass_coverage.iter().flat_map(|coverage| coverage.iter()).copied() {
            let bounds = Bounds::from_aabb(aabb, leaf.transform)
                .ok_or("grass camera coverage has nonfinite bounds")?;
            if let Some(bounds) = authentic.partition_coverage(bounds).0 {
                let pad = Vec3::splat(0.15);
                grass_lens_blockers.push(Bounds { min: bounds.min - pad, max: bounds.max + pad });
            }
        }
        if leaf.object.is_none() {
            if let Some(coverage) = &leaf.sight_coverage {
                for aabb in coverage.iter().copied() {
                    let bounds = Bounds::from_aabb(aabb, leaf.transform)
                        .ok_or("terrain sight coverage has nonfinite bounds")?;
                    if let Some(bounds) = authentic.partition_coverage(bounds).0 {
                        static_blockers.push(bounds);
                    }
                }
            } else {
                static_blockers.push(leaf.bounds);
            }
        }
        if let Some(id) = leaf.object {
            objects
                .entry(id)
                .and_modify(|o| {
                    o.bounds = o.bounds.union(leaf.bounds);
                    o.backdrop.push(leaf.bounds);
                })
                .or_insert(FrozenObject {
                    bounds: leaf.bounds,
                    backdrop: vec![leaf.bounds],
                    surface: object_surfaces.get(&id).cloned(),
                    faded: false,
                    opacity: 1.0,
                });
        }
    }
    // Validate all material/image inputs before creating any retained assets.
    let source_materials = world
        .get_resource::<Assets<VoxelMaterial>>()
        .ok_or("voxel materials are unavailable")?;
    let source_images = world
        .get_resource::<Assets<Image>>()
        .ok_or("terrain images are unavailable")?;
    let mut material_copies =
        HashMap::<(AssetId<VoxelMaterial>, Option<Entity>), VoxelMaterial>::new();
    let mut image_copies = HashMap::<AssetId<Image>, Image>::new();
    for leaf in &kept {
        let key = (leaf.material.id(), leaf.object);
        if material_copies.contains_key(&key) {
            continue;
        }
        let mut material = source_materials
            .get(&leaf.material)
            .ok_or("retained terrain material is unavailable")?
            .clone();
        if material.extension.water_enabled.x > 0.5 {
            let built = cache.built_frame.as_ref().ok_or("actual terrain has not finished building")?;
            let mode = material.extension.water_enabled.x;
            material.extension.water_enabled = crate::cave_water::encounter_registration(
                built, root_pose.translation, RETAINED_SCALE,
            );
            material.extension.water_enabled.x = mode;
        }
        if let Some(texture) = &material.base.base_color_texture {
            if !image_copies.contains_key(&texture.id()) {
                image_copies.insert(
                    texture.id(),
                    source_images
                        .get(texture)
                        .ok_or("retained terrain image is unavailable")?
                        .clone(),
                );
            }
        }
        let cutoff = match material.base.alpha_mode {
            AlphaMode::Mask(v) => Some(v),
            AlphaMode::Blend if material.extension.cutaway.fade.w > 0.0 => Some(material.extension.cutaway.fade.w),
            _ => None,
        };
        material.extension.cutaway = default();
        // Terrain pixels include LCD display zoom; battler sculptures do not.
        // Normalize only blade height, leaving source roots, ground and scenery intact.
        material.extension.grass_rustle.contact = Vec4::ZERO;
        material.extension.grass_rustle.height_scale = cache.built_frame.as_ref()
            .map_or(Vec4::ZERO, |frame| crate::grass_rustle::retained_height_scale(frame.tile_size.y));
        material.base.alpha_mode = cutoff.map_or(AlphaMode::Opaque, AlphaMode::Mask);
        // Any transparency inherited from world reveal is reset. Whole-object
        // battle reveal is computed from this encounter camera independently.
        // A wall's temporary reveal must reset, but a leaf's authored cutout
        // is part of its geometry and must survive the frozen encounter.
        if !matches!(material.base.alpha_mode, AlphaMode::Mask(_)) {
            material.base.alpha_mode = AlphaMode::Opaque;
        }
        material_copies.insert(key, material);
    }
    // The encounter owns this atlas copy. A narrowly proven floor grade makes
    // the bright repeating gym floor recede without repainting source art,
    // props, pits, actors or move effects. Use the actual built grid rather
    // than a shifted live frame or the earlier encounter witness.
    if let Some(built) = cache.built_frame.as_ref() {
        if let Some(atlas) = image_copies.get_mut(&built.map_texture.id()) {
            let graded = floor_palette::grade_violet_floor(
                built, cache.built_profiles.as_deref(), atlas,
            );
            if graded != 0 {
                trace(format_args!("encounter-owned Violet floor grade: {graded} source cells"));
            }
        }
    }
    let image_handles: HashMap<_, _> = image_copies
        .into_iter()
        .map(|(id, image)| (id, world.resource_mut::<Assets<Image>>().add(image)))
        .collect();
    let mut frozen_materials = Vec::new();
    let material_handles: HashMap<_, _> = material_copies
        .into_iter()
        .map(|(key, mut material)| {
            if let Some(texture) = &mut material.base.base_color_texture {
                if let Some(handle) = image_handles.get(&texture.id()) {
                    *texture = handle.clone();
                }
            }
            let alpha_cutoff = match material.base.alpha_mode { AlphaMode::Mask(v) => Some(v), _ => None };
            let handle = world.resource_mut::<Assets<VoxelMaterial>>().add(material);
            frozen_materials.push(FrozenMaterial {
                alpha_cutoff,
                handle: handle.clone(),
                object: key.1,
            });
            (key, handle)
        })
        .collect();
    // All fallible evidence/material/image checks are finished before assets
    // are created. Their private handles are retained until generation retirement.
    let mutable_mesh_handles: HashMap<_, _> = mutable_mesh_copies
        .into_iter()
        .map(|(id, mesh)| (id, world.resource_mut::<Assets<Mesh>>().add(mesh)))
        .collect();
    let root = world
        .spawn((
            (Transform::default(),
Visibility::Hidden,),
            EncounterTerrainRoot,
        ))
        .id();
    // Static draws may contain an entire room, including its floor, walls and
    // small fixtures. Rank the original spatial coverage rather than that
    // draw's enclosing box, so existing cans, pillars and rock formations can
    // contribute to the shot. Geometry and collision evidence stay unchanged.
    let mut backdrop_bounds = Vec::new();
    for leaf in kept.iter().filter(|leaf| leaf.object.is_none()) {
        if let Some(coverage) = &leaf.sight_coverage {
            for aabb in coverage.iter().copied() {
                let bounds = Bounds::from_aabb(aabb, leaf.transform)
                    .ok_or("backdrop source coverage has invalid bounds")?;
                if let Some(bounds) = authentic.partition_coverage(bounds).0
                    && bounds.max.y - bounds.min.y > 0.8 {
                    backdrop_bounds.push(bounds);
                }
            }
        } else if leaf.bounds.max.y - leaf.bounds.min.y > 0.8 {
            backdrop_bounds.push(leaf.bounds);
        }
    }
    for leaf in kept {
        let child = world
            .spawn((
                (Mesh3d(mutable_mesh_handles
                        .get(&leaf.mesh.id())
                        .cloned()
                        .unwrap_or(leaf.mesh)),
MeshMaterial3d(material_handles[&(leaf.material.id(), leaf.object)].clone()),
leaf.transform,),
                leaf.aabb,
                RenderLayers::layer(BATTLE_LAYER),
            ))
            .id();
        world.entity_mut(root).add_child(child);
    }
    trace(format_args!("grass camera coverage: {} bounds, maximum world height {}",
        grass_lens_blockers.len(), grass_lens_blockers.iter().map(|bounds| bounds.max.y).fold(f32::NEG_INFINITY, f32::max)));
    Ok(FrozenScene {
        root,
        anchors,
        authentic,
        omitted: StaticBlockers::new(omitted),
        static_blockers: StaticBlockers::new(static_blockers),
        grass_lens_blockers: StaticBlockers::new(grass_lens_blockers),
        objects,
        backdrop_bounds,
        materials: frozen_materials,
        images: image_handles.into_values().collect(),
        meshes: mutable_mesh_handles.into_values().collect(),
        ground,
        walking,
        water: matches!(location.target, crystal_render_api::VisualBattleTarget::SurfWater { .. }),
        bodies: [None; 2],
        derived_stage: matches!(location.target, crystal_render_api::VisualBattleTarget::ScriptedGround { .. } | crystal_render_api::VisualBattleTarget::Trainer { .. } | crystal_render_api::VisualBattleTarget::WalkingGrass { .. }),
        source_anchors: anchors,
        clearance: [Vec4::ZERO; 2],
        clearance_shape: [Vec4::ZERO; 2], grass_contact: [Vec4::ZERO; 2], contact_shape: [Vec4::ZERO; 2],
    })
}

/// Schedule after the battle view has made its final source/model visibility
/// decision. Source fallback hides this hierarchy without retiring its assets.
pub(super) fn sync_grass_mask_materials(
    state: Res<EncounterTerrain>,
    mask: Res<crate::battle_view::BattleGrassMask>,
    materials: Option<ResMut<Assets<VoxelMaterial>>>,
) {
    let Some(scene) = &state.frozen else { return; };
    let Some(mut materials) = materials else { return; };
    let info = Vec4::new(if mask.active && state.accepted { 1.0 } else { 0.0 }, mask.revision as f32, 0.0, 0.0);
    for retained in &scene.materials {
        if materials.get(&retained.handle).is_some_and(|material|
            material.extension.battle_grass_mask_info != info
                || material.extension.battle_grass_mask != mask.image) {
            if let Some(mut material) = materials.get_mut(&retained.handle) {
                material.extension.battle_grass_mask = mask.image.clone();
                material.extension.battle_grass_mask_info = info;
            }
        }
    }
}

pub(super) fn sync(
    mut state: ResMut<EncounterTerrain>,
    status: Res<BattleViewStatus>,
    frame: Res<VisualBattleFrame>,
    mode: Res<BattleFlashMode>,
    time: Option<Res<Time>>,
    mut roots: Query<&mut Visibility, With<EncounterTerrainRoot>>,
    materials: Option<ResMut<Assets<VoxelMaterial>>>,
) {
    let shown = status.active && state.accepted;
    let Some(scene) = &mut state.frozen else {
        return;
    };
    if let Ok(mut visibility) = roots.get_mut(scene.root) {
        visibility.set_if_neq(if shown {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
    if !shown {
        return;
    }
    let Some(mut materials) = materials else {
        return;
    };
    let (dark, white) =
        crate::battle_view::source_environment_palette(frame.source.as_ref(), *mode);
    let seconds = time.as_ref().map_or(0.0, |time| time.delta_secs());
    for object in scene.objects.values_mut() {
        object.opacity = advance_opacity(object.opacity, object.faded, seconds);
    }
    for retained in &scene.materials {
        let opacity = retained
            .object
            .and_then(|id| scene.objects.get(&id))
            .map_or(1.0, |object| object.opacity);
        let value = Vec4::new(1.0 - opacity, dark, white, retained.alpha_cutoff.unwrap_or(0.0));
        let alpha_mode = retained_alpha_mode(opacity, retained.alpha_cutoff);
        if materials
            .get(&retained.handle)
            .is_some_and(|m| m.extension.cutaway.fade != value || m.base.alpha_mode != alpha_mode || m.extension.battle_clearance != scene.clearance || m.extension.battle_clearance_shape != scene.clearance_shape || m.extension.battle_grass_contact != scene.grass_contact || m.extension.battle_contact_shape != scene.contact_shape)
        {
            if let Some(mut material) = materials.get_mut(&retained.handle) {
                material.extension.cutaway.fade = value;
                material.extension.battle_clearance = scene.clearance;
                material.extension.battle_clearance_shape = scene.clearance_shape;
                material.extension.battle_grass_contact = scene.grass_contact;
                material.extension.battle_contact_shape = scene.contact_shape;
                material.base.alpha_mode = alpha_mode;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn indexed_scenery_preserves_flat_sight_checks_and_fixture_retirement() {
        let mut bounds = Vec::new();
        for x in -8_i32..=8 {
            for z in -8_i32..=8 {
                let min = Vec3::new(x as f32 * 2.0, 0.0, z as f32 * 2.0);
                bounds.push(Bounds { min, max: min + Vec3::new(1.0, 0.25 + (x * z).rem_euclid(7) as f32, 1.0) });
            }
        }
        let mut indexed = StaticBlockers::new(bounds.clone());
        for start in [Vec3::new(-20.0, 4.0, -20.0), Vec3::new(20.0, 0.5, 20.0), Vec3::new(0.0, 8.0, 0.0)] {
            for x in -10..=10 {
                for z in -10..=10 {
                    let end = Vec3::new(x as f32 * 2.0 + 1.0, 1.0, z as f32 * 2.0);
                    for height in [-1.0, 0.25, 1.0, 6.0, 10.0] {
                        let flat = bounds.iter().any(|bounds| bounds.max.y > height && bounds.segment(start, end));
                        assert_eq!(indexed.blocker(start, end, height).is_some(), flat,
                            "indexed sight must retain every flat-scan obstruction: {start:?} -> {end:?}, height={height}");
                    }
                }
            }
        }
        indexed.clear();
        assert!(indexed.blocker(Vec3::NEG_X * 2.0, Vec3::X * 2.0, 0.0).is_none());
        indexed.push(Bounds { min: Vec3::splat(-1.0), max: Vec3::splat(1.0) });
        assert!(indexed.blocker(Vec3::NEG_X * 2.0, Vec3::X * 2.0, 0.0).is_some());
    }

    #[test]
    fn sight_line_bounds_keep_grazing_hits_and_clipped_endpoints() {
        let bounds = Bounds { min: Vec3::splat(-1.0), max: Vec3::splat(1.0) };
        for (start, end, blocked) in [
            (Vec3::new(-2.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), true),
            (Vec3::new(-2.0, 1.0, 0.0), Vec3::new(2.0, 1.0, 0.0), true),
            (Vec3::new(-2.0, 1.001, 0.0), Vec3::new(2.0, 1.001, 0.0), false),
            (Vec3::new(-2.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), false),
            (Vec3::new(-2.0, 2.0, 0.0), Vec3::new(2.0, -2.0, 0.0), true),
            (Vec3::new(-3.0, -3.0, -3.0), Vec3::splat(-2.0), false),
            (Vec3::ZERO, Vec3::ZERO, true),
        ] {
            assert_eq!(bounds.segment(start, end), blocked, "{start:?} -> {end:?}");
            assert_eq!(bounds.segment(end, start), blocked, "reversed {start:?} -> {end:?}");
        }
    }

    #[test]
    fn cached_backdrop_projection_matches_original_corner_projection() {
        let viewport = Vec2::new(1280., 720.);
        for yaw in [-2.4_f32, -1.2, 0., 1.2, 2.4] {
            let mut layout = BattleSceneLayout::default();
            layout.camera = Transform::from_translation(Vec3::new(yaw.sin() * 12., 3., yaw.cos() * 12.))
                .looking_at(Vec3::Y, Vec3::Y);
            let view = layout.camera.to_matrix().inverse();
            for x in -8..8 { for z in -8..8 {
                let bounds = Bounds { min: Vec3::new(x as f32 * 3., 0., z as f32 * 3.),
                    max: Vec3::new(x as f32 * 3. + 2., 9., z as f32 * 3. + 2.) };
                let forward = *layout.camera.forward();
                let mut expected = 0_u32;
                let corners: Vec<_> = (0..8).map(|i| Vec3::new(
                    if i & 1 == 0 { bounds.min.x } else { bounds.max.x },
                    if i & 2 == 0 { bounds.min.y } else { bounds.max.y },
                    if i & 4 == 0 { bounds.min.z } else { bounds.max.z })).collect();
                if (((bounds.min + bounds.max) * 0.5 - layout.camera.translation).dot(forward) > 5.5)
                    && corners.iter().all(|&p| (p - layout.camera.translation).dot(forward) > 0.01) {
                    let projected: Vec<_> = corners.into_iter().map(|p| layout.project_point(p, viewport)).collect();
                    let lo = projected.iter().copied().fold(Vec2::splat(f32::INFINITY), Vec2::min);
                    let hi = projected.iter().copied().fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
                    let band = layout.visible_y_range();
                    for row in 0..4 { for column in 0..8 {
                        let sample = Vec2::new(0.05 + (column as f32 + 0.5) * 0.90 / 8.,
                            band.x + (row as f32 + 0.5) * (band.y - band.x) * 0.60 / 4.);
                        if sample.cmpge(lo).all() && sample.cmple(hi).all() { expected |= 1 << (row * 8 + column); }
                    }}
                }
                assert_eq!(backdrop_cells_projected(bounds, &layout, viewport, 5., view), expected,
                    "cached scoring changed at yaw={yaw} box=({x},{z})");
            }}
        }
    }

    #[test]
    fn backdrop_ranking_counts_visible_canopy_instead_of_offscreen_world_size() {
        let mut layout = BattleSceneLayout::default();
        layout.camera = Transform::from_xyz(0., 1., 5.)
            .looking_at(Vec3::new(0., 1., 0.), Vec3::Y);
        let viewport = Vec2::new(800., 600.);
        let canopy = Bounds { min: Vec3::new(-20., 0., -7.), max: Vec3::new(-2., 8., -5.) };
        let center = layout.project_point((canopy.min + canopy.max) * 0.5, viewport);
        assert!(center.x < 0.05, "fixture canopy center is outside the shot");
        let coverage = backdrop_cells(canopy, &layout, viewport, 5.);
        assert!(coverage != 0, "visible canopy at the frame edge still supplies a background");
        let distant = Bounds { min: canopy.min - Vec3::Z * 200., max: canopy.max - Vec3::Z * 200. };
        assert!(coverage.count_ones() > backdrop_cells(distant, &layout, viewport, 5.).count_ones(),
            "large world dimensions cannot substitute for visible scenery");
        let offscreen = Bounds { min: canopy.min - Vec3::X * 100., max: canopy.max - Vec3::X * 100. };
        assert_eq!(backdrop_cells(offscreen, &layout, viewport, 5.), 0);
        let foreground = Bounds { min: Vec3::new(-2.,0.,2.), max: Vec3::new(2.,8.,3.) };
        assert_eq!(backdrop_cells(foreground, &layout, viewport, 5.), 0,
            "foreground occluders never earn a backdrop reward");
    }

    #[test]
    fn shot_readability_rewards_the_small_opponent_before_an_already_large_partner() {
        let distant = actor_readability_score(&[Vec2::new(0.25, 0.25), Vec2::new(0.05, 0.025)]);
        let larger_partner = actor_readability_score(&[Vec2::new(0.50, 0.50), Vec2::new(0.05, 0.025)]);
        let closer_pair = actor_readability_score(&[Vec2::new(0.25, 0.25), Vec2::new(0.10, 0.05)]);
        assert_eq!(distant, larger_partner);
        assert!(closer_pair > distant + 40.0);
        assert_eq!(actor_readability_score(&[Vec2::splat(0.4); 2]),
            actor_readability_score(&[Vec2::splat(0.8); 2]));
        assert_eq!(actor_readability_score(&[]), 0.0);
    }
    use super::*;
use bevy::camera::primitives::MeshAabb;
    use crystal_render_api::{
        VisualActorId, VisualBattleAnchorFrame, VisualBattleObjectTarget, VisualBattleTarget,
        VisualBattleSourceMovement, VisualBattleSourcePose, VisualBattleTerrainEvidence,
    };
    use std::sync::Arc;

    #[test]
    fn whole_object_fade_is_smooth_and_restores_opaque_depth_without_frame_rate_dependence() {
        assert_eq!(opacity_mode(1.0), AlphaMode::Opaque);
        let first = advance_opacity(1.0, true, 1.0 / 60.0);
        assert!(first < 1.0 && first > RESIDUAL_OPACITY);
        assert_eq!(opacity_mode(first), AlphaMode::Blend);
        let at_30 = (0..15).fold(1.0, |a, _| advance_opacity(a, true, 1.0 / 30.0));
        let at_60 = (0..30).fold(1.0, |a, _| advance_opacity(a, true, 1.0 / 60.0));
        assert!((at_30 - at_60).abs() < 0.00001);
        let faded = (0..60).fold(1.0, |a, _| advance_opacity(a, true, 1.0 / 60.0));
        assert_eq!(faded, RESIDUAL_OPACITY);
        let restored = (0..60).fold(faded, |a, _| advance_opacity(a, false, 1.0 / 60.0));
        assert_eq!(restored, 1.0);
        assert_eq!(opacity_mode(restored), AlphaMode::Opaque);
    }

    fn fixture() -> (VisualWorldFrame, VisualBattleLocation, Vec<f32>) {
        let frame = VisualWorldFrame {
            active: true,
            map_id: Arc::from("Route36"),
            source_map_size_core_tiles: Some(UVec2::new(24, 16)),
            terrain_revision: 73,
            grid_origin: IVec2::new(-8, -8),
            grid_size: UVec2::new(64, 48),
            tile_size: Vec2::splat(32.0),
            viewport_size: Vec2::new(640.0, 576.0),
            center: Vec2::new(100.0, -60.0),
            map_texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(51), std::marker::PhantomData),
            ..default()
        };
        let evidence = VisualBattleTerrainEvidence {
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
        };
        let location = VisualBattleLocation {
            generation: 7,
            source: VisualBattleSourcePose {
                map_id: frame.map_id.clone(),
                source_frame: 999,
                core_tile: IVec2::new(12, 8),
                facing: IVec2::NEG_Y,
                movement: VisualBattleSourceMovement::Normal,
            },
            target: VisualBattleTarget::Object(VisualBattleObjectTarget {
                object_identifier: Arc::from("tree"),
                object_script: Arc::from("SudowoodoScript"),
                core_tile: IVec2::new(12, 7),
                trigger_script: Arc::from("WateredWeirdTreeScript"),
                battle_source_script: Arc::from("WateredWeirdTreeScript"),
                startbattle_command_index: 12,
            }),
            source_map_size_core_tiles: Some(UVec2::new(24, 16)),
            anchors: Some(Arc::new(VisualBattleAnchorFrame {
                terrain: evidence,
                source_actor: VisualActorId::Player,
                target_actor: Some(VisualActorId::Object(0)),
                source_foot: Vec2::new(132.0, -124.0),
                target_foot: Vec2::new(132.0, -60.0),
            })),
        };
        let heights = vec![0.0; (frame.grid_size.x * frame.grid_size.y) as usize];
        (frame, location, heights)
    }
    fn cache(frame: &VisualWorldFrame, heights: Vec<f32>) -> TerrainRevisionCache {
        TerrainRevisionCache {
            built_frame: Some(frame.clone()),
            built_source_texture: Some(frame.map_texture.clone()),
            built_footing_heights: heights,
            ..default()
        }
    }
    fn walking_fixture() -> (VisualWorldFrame, VisualBattleLocation, Vec<f32>) {
        let (frame, mut location, heights) = fixture();
        let source = location.source.core_tile;
        location.target = VisualBattleTarget::WalkingGrass {
            core_tile: source,
            presentation: crystal_render_api::VisualBattleDerivedGrassPlacement {
                step_from_core_tile: source + IVec2::Y,
                witnessed_player_foot: Some(Vec2::new(132.0, -156.0)),
                presentation_core_tile: source - IVec2::Y * 2,
                walkable_core_tiles: (-3..=3).flat_map(|y| (-3..=3).map(move |x| source + IVec2::new(x, y))).collect::<Vec<_>>().into(),
            },
        };
        let anchors = Arc::make_mut(location.anchors.as_mut().unwrap());
        anchors.target_actor = None;
        anchors.target_foot = Vec2::new(132.0, 4.0);
        (frame, location, heights)
    }
    #[test]
    fn indexed_omission_view_matches_flat_geometry_and_shadow_proof() {
        let bounds: Vec<_> = (-12_i32..12).flat_map(|x| (-12_i32..12).map(move |z| Bounds {
            min: Vec3::new(x as f32 * 4., -2., z as f32 * 4.),
            max: Vec3::new(x as f32 * 4. + 0.1, (x + z).abs() as f32 + 0.1, z as f32 * 4. + 2.),
        })).collect();
        let index = StaticBlockers::new(bounds.clone());
        for yaw in 0..24 { for far in [4., 12., 32., 80.] { for ground in [-8., 0., 4.] {
            let angle = yaw as f32 * std::f32::consts::TAU / 24.;
            let camera = Transform::from_xyz(angle.sin() * 60., 5., angle.cos() * 60.)
                .looking_at(Vec3::new(0., 2., 0.), Vec3::Y);
            let viewport = Vec2::new(1280., 720.);
            let flat = bounds.iter().any(|&bounds| visible_bounds(bounds, camera, viewport, far)
                || visible_bounds(bounds.shadow_volume(ground), camera, viewport, far));
            assert_eq!(index.visible_or_shadow(camera, viewport, far, ground), flat,
                "omission proof changed at yaw={yaw} far={far} ground={ground}");
        }}}
        assert!(!StaticBlockers::default().visible_or_shadow(Transform::IDENTITY, Vec2::splat(100.), 10., 0.));
    }

    #[test]
    fn battle_contact_shape_retains_long_thin_footprints_through_yaw() {
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.1, 0.0, -2.0), Vec3::new(0.1, 1.0, 2.0), 1.0);
        for yaw in [0.0_f32, 0.7, std::f32::consts::FRAC_PI_2] {
            let pose = Transform { rotation: Quat::from_rotation_y(yaw), scale: Vec3::new(2.0, 1.0, 0.5), ..default() };
            let shape = battle_contact_shape(body, pose);
            assert!((shape.z - 0.24).abs() < 0.00001);
            assert!((shape.w - 1.20).abs() < 0.00001);
            let axis = Vec2::new(shape.x, shape.y);
            let expected = pose.rotation * Vec3::X;
            assert!((axis - Vec2::new(expected.x, expected.z)).length() < 0.00001);
            body.visual_bounds = Some((Vec3::splat(-10.0), Vec3::splat(10.0)));
            assert_eq!(shape, battle_contact_shape(body, pose), "moving tails and wings must not inflate contact shading");
        }
    }

    #[test]
    fn battle_contact_stays_on_checked_ground_during_lift_and_clears_when_hidden() {
        let (mut app, _, _, _, _) = app_fixture();
        prepare(app.world_mut());
        let mut state = app.world_mut().resource_mut::<EncounterTerrain>();
        state.accepted = true;
        let ground = state.frozen.as_ref().unwrap().anchors[0];
        let neutral = BattleBody::modeled("TEST", Vec3::new(-0.2, 0.0, -0.1), Vec3::new(0.2, 1.0, 0.1), 1.0);
        let pose = Transform::from_translation(ground + Vec3::Y * 3.0);
        let camera = Transform::from_xyz(0.0, 6.0, 10.0).looking_at(ground, Vec3::Y);
        let mut visible = neutral;
        visible.min = Vec3::splat(-5.0); visible.max = Vec3::splat(5.0);
        state.set_visible_apertures(camera, [Some((visible, pose)), None], [Some((neutral, pose)), None]);
        let scene = state.frozen.as_ref().unwrap();
        assert_eq!(scene.grass_contact[0].y, ground.y, "the shadow stays on the checked floor during source animation lift");
        assert!((scene.contact_shape[0].z - 0.24).abs() < 0.00001);
        assert!((scene.contact_shape[0].w - 0.12).abs() < 0.00001);
        state.set_visible_apertures(camera, [None; 2], [None; 2]);
        let scene = state.frozen.as_ref().unwrap();
        assert_eq!(scene.grass_contact, [Vec4::ZERO; 2]);
        assert_eq!(scene.contact_shape, [Vec4::ZERO; 2]);
    }

    #[test]
    fn grass_foot_contact_uses_neutral_ground_and_ignores_animation_envelope() {
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.2, -0.5, -0.1),
            Vec3::new(0.2, 0.5, 0.1), 1.0);
        for ground in [-4.0, 0.0, 3.0] {
            let pose = Transform::from_xyz(2.0, ground + 0.5, -3.0);
            let contact = grass_foot_contact(body, pose);
            assert!((contact.y - ground).abs() < 0.00001);
            assert_eq!(contact.x, 2.0);
            assert_eq!(contact.z, -3.0);
            assert!(contact.w > 0.2 && contact.w < 0.3);
            body.visual_bounds = Some((Vec3::splat(-5.0), Vec3::splat(5.0)));
            assert_eq!(contact, grass_foot_contact(body, pose),
                "moving leaves and attack envelopes must not move the contact patch");
        }
    }

    #[test]
    fn grass_contact_fringe_tracks_world_feet_and_stays_below_small_bodies() {
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.3, -0.5, -0.3), Vec3::new(0.3, 0.5, 0.3), 1.);
        body.visual_bounds = Some((Vec3::splat(-2.), Vec3::splat(3.)));
        let camera = Transform::from_xyz(0., 3., 10.).looking_at(Vec3::ZERO, Vec3::Y);
        for ground in [-4., 0., 3.] {
            let pose = Transform::from_xyz(0., ground + 0.5, 0.);
            let (_, shape) = grass_body_aperture(body, pose, camera);
            assert!((shape.w - (ground + 0.22)).abs() < 0.00001,
                "the actual neutral feet, not the full animation envelope, set the overlap");
            assert!(shape.w < ground + 0.5, "the fringe stays below the body's middle");
        }
    }

    #[test]
    fn grass_aperture_encloses_the_animation_without_a_large_round_clearing() {
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.25, 0.0, -0.25), Vec3::new(0.25, 4.0, 0.25), 4.0);
        body.visual_bounds = Some((Vec3::new(-0.4, -0.2, -0.5), Vec3::new(0.4, 4.5, 0.5)));
        for yaw in [0.0_f32, 90.0, 180.0, 270.0] {
            let rotation = Quat::from_rotation_y(yaw.to_radians());
            let camera = Transform::from_translation(rotation * Vec3::new(0.0, 4.0, 12.0))
                .looking_at(Vec3::Y * 2.0, Vec3::Y);
            let pose = Transform::from_rotation(rotation);
            let (clearance, shape) = grass_body_aperture(body, pose, camera);
            let view = camera.compute_affine().inverse();
            let center = view.transform_point3(clearance.truncate());
            assert!(shape.x < clearance.w * 0.5, "slender bodies need narrower apertures");
            for point in body.corners(pose) {
                let point = view.transform_point3(point);
                let projected = point.truncate() * center.z / point.z;
                let normalized = (projected - center.truncate()) / shape.truncate().truncate();
                assert!(normalized.length() <= 1.0, "the full animation remains clear");
                assert!(point.z >= center.z - shape.z,
                    "every animated point lies ahead of the retained grass depth boundary");
            }
            assert!(shape.z < clearance.w, "depth uses the real envelope, not its bounding sphere");
            assert!((shape.w - 0.32).abs() < 0.00001,
                "a tall rig keeps only a bounded fringe at its feet");
            assert_eq!(pose.scale, Vec3::ONE);
        }
    }

    #[test]
    fn nearby_scripted_staging_keeps_full_bodies_on_real_floor_and_preserves_origin() {
        let (frame, mut location, heights) = walking_fixture();
        let unavailable = location.source.core_tile + IVec2::X;
        let VisualBattleTarget::WalkingGrass { presentation, .. } = &location.target else { panic!() };
        location.target = VisualBattleTarget::ScriptedGround {
            presentation: crystal_render_api::VisualBattleDerivedTrainerPlacement {
                presentation_core_tile: presentation.presentation_core_tile,
                walkable_core_tiles: presentation.walkable_core_tiles.iter().copied()
                    .filter(|tile| *tile != unavailable).collect::<Vec<_>>().into(),
            },
        };
        let origin = location.clone();
        let cache = cache(&frame, heights);
        let (anchors, pose, _) = resolve_context(&location, &cache).unwrap();
        let ground = WalkingGround::from_context(&location, &cache, pose).unwrap().unwrap();
        let bodies = [Some(BattleBody::modeled("TEST", Vec3::new(-2.5, 0.0, -0.5), Vec3::new(2.5, 1.0, 0.5), 1.0)); 2];
        let viewport = Vec2::new(800., 600.);
        let original = BattleSceneLayout::for_map_bodies(bodies, anchors, viewport).unwrap();
        assert!(!ground.contains(bodies[0].unwrap(), original.body_poses[0].unwrap(), anchors[0].y));
        let pairs = ground.nearby_stage_pairs(anchors, bodies);
        assert!(pairs.len() <= 3528);
        assert!(pairs.iter().any(|feet| (feet[1] - feet[0]).length() == 4.0),
            "small bodies can use a closer real-floor pair without being rescaled");
        assert!(pairs.into_iter().any(|feet| {
            feet != anchors && BattleSceneLayout::for_map_bodies(bodies, feet, viewport).is_some_and(|layout|
                (0..2).all(|index| ground.contains(bodies[index].unwrap(), layout.body_poses[index].unwrap(), feet[index].y)))
        }), "an adjacent real floor plot fits the complete animation");
        assert_eq!(location, origin, "derived staging never changes authoritative source metadata");
    }

    #[test]
    fn scripted_ground_requires_real_floor_without_inventing_an_enemy_contact() {
        let (frame, mut location, heights) = walking_fixture();
        let VisualBattleTarget::WalkingGrass { presentation, .. } = &location.target else { panic!() };
        location.target = VisualBattleTarget::ScriptedGround {
            presentation: crystal_render_api::VisualBattleDerivedTrainerPlacement {
                presentation_core_tile: presentation.presentation_core_tile,
                walkable_core_tiles: presentation.walkable_core_tiles.clone(),
            },
        };
        let cache = cache(&frame, heights);
        let (_, pose, _) = resolve_context(&location, &cache).unwrap();
        assert!(WalkingGround::from_context(&location, &cache, pose).unwrap().is_some());
        let mut blocked = location.clone();
        let VisualBattleTarget::ScriptedGround { presentation } = &mut blocked.target else { panic!() };
        presentation.walkable_core_tiles = Arc::from([blocked.source.core_tile]);
        assert!(WalkingGround::from_context(&blocked, &cache, pose).is_err());
        let mut surf = location.clone();
        surf.source.movement = crystal_render_api::VisualBattleSourceMovement::Surf;
        assert!(WalkingGround::from_context(&surf, &cache, pose).is_err());
        let mut stale = location.clone();
        Arc::make_mut(stale.anchors.as_mut().unwrap()).target_foot += Vec2::X;
        assert!(WalkingGround::from_context(&stale, &cache, pose).is_err());
    }

    fn surf_fixture() -> (VisualWorldFrame, VisualBattleLocation, TerrainRevisionCache) {
        let (mut frame, mut location, mut heights) = walking_fixture();
        frame.tiles = (0..frame.grid_size.y).flat_map(|row| (0..frame.grid_size.x).map(move |column| {
            crystal_render_api::VisualTile {
                column, row,
                source: crystal_render_api::VisualTileSource {
                    tileset_id: Arc::from("johto"), metatile_id: 0,
                    subtile_column: (column % 4) as u8, subtile_row: (row % 4) as u8,
                    tile_index: 0x14,
                },
                texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(71), std::marker::PhantomData), animation_frames: None, priority: false,
            }
        })).collect();
        heights.fill(-8.0);
        location.source.movement = VisualBattleSourceMovement::Surf;
        let VisualBattleTarget::WalkingGrass { core_tile, presentation } = location.target else { panic!() };
        location.target = VisualBattleTarget::SurfWater {
            core_tile,
            presentation: crystal_render_api::VisualBattleDerivedSurfPlacement {
                step_from_core_tile: presentation.step_from_core_tile,
                witnessed_player_foot: presentation.witnessed_player_foot,
                presentation_core_tile: presentation.presentation_core_tile,
                water_core_tiles: presentation.walkable_core_tiles,
            },
        };
        Arc::make_mut(location.anchors.as_mut().unwrap()).terrain.tiles = frame.tiles.clone().into();
        let mut retained = cache(&frame, heights);
        retained.built_profiles = Some(Arc::new(crate::live_profiles::Document::default()));
        (frame, location, retained)
    }

    #[test]
    fn surf_support_requires_actual_water_cells_profile_provenance_and_sampled_heights() {
        let (frame, location, retained) = surf_fixture();
        let (feet, pose, _) = resolve_context(&location, &retained).unwrap();
        let body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        let fits = |cache: &TerrainRevisionCache| {
            WalkingGround::from_context(&location, cache, pose).ok().flatten()
                .is_some_and(|ground| ground.contains(body, Transform::from_translation(feet[0]), feet[0].y))
        };
        assert!(fits(&retained));
        assert_eq!(feet[0].y, 0.0, "real sampled water heights are rebased, not replaced by a guessed datum");
        let mut bad = cache(&frame, retained.built_footing_heights.clone());
        assert!(!fits(&bad), "absent actual profile provenance cannot prove water geometry");
        bad.built_profiles = retained.built_profiles.clone();
        let cell = IVec2::new(25,18) - frame.grid_origin;
        let index = cell.y as usize * frame.grid_size.x as usize + cell.x as usize;
        bad.built_frame.as_mut().unwrap().tiles[index].source.tile_index = 0;
        assert!(!fits(&bad), "a bank/flat drawing is not water despite producer collision approval");
        bad.built_frame = Some(frame.clone());
        bad.built_footing_heights[index] += 16.0;
        assert!(!fits(&bad), "one covered cell at another elevation is not a level water surface");
        bad.built_footing_heights = retained.built_footing_heights.clone();
        bad.built_frame.as_mut().unwrap().tiles.reverse();
        assert!(fits(&bad), "valid coordinate order does not become an implicit tile-index identity");
        bad.built_frame.as_mut().unwrap().tiles.pop();
        assert!(!fits(&bad), "an incomplete grid cannot establish water support");
        let mut profile = crate::live_profiles::Document::default();
        profile.objects.push(crate::live_profiles::Object {
            name: "Water override".into(), tileset: "johto".into(), map: Some("Route36".into()), maps: None,
            metatile: 0, metatiles: None, origin: [0,0], tiles: vec![vec![0x14]], ground: 1,
            top_pixels: 0, depth_pixels: 4.0, footing_pixels: Some(8.0),
            mask: crate::live_profiles::Mask::None, parts: Vec::new(),
        });
        bad.built_frame = Some(frame.clone());
        bad.built_profiles = Some(Arc::new(profile.clone()));
        assert!(!fits(&bad), "an applicable custom drawing cannot inherit a Water label");
        profile.objects[0].tiles = vec![vec![1]]; profile.objects[0].ground = 0x14;
        bad.built_profiles = Some(Arc::new(profile.clone()));
        assert!(!fits(&bad), "custom ground samples also make the geometry proof unavailable");
        profile.objects[0].map = Some("DifferentMap".into());
        bad.built_profiles = Some(Arc::new(profile));
        assert!(fits(&bad), "unrelated map profiles do not affect this actual build");
    }

    #[test]
    fn surf_support_keeps_movement_witness_and_body_displacement_inside_checked_water() {
        let (_, location, retained) = surf_fixture();
        let (feet, pose, _) = resolve_context(&location, &retained).unwrap();
        for case in 0..6 {
            let mut bad = location.clone();
            let VisualBattleTarget::SurfWater { core_tile, presentation } = &mut bad.target else { panic!() };
            match case {
                0 => bad.source.movement = VisualBattleSourceMovement::Normal,
                1 => bad.source.movement = VisualBattleSourceMovement::SurfPika,
                2 => *core_tile += IVec2::X,
                3 => presentation.witnessed_player_foot = None,
                4 => presentation.witnessed_player_foot = Some(Vec2::splat(f32::NAN)),
                _ => presentation.step_from_core_tile += IVec2::X,
            }
            assert!(WalkingGround::from_context(&bad, &retained, pose).is_err(), "case{case}");
        }
        let ground = WalkingGround::from_context(&location, &retained, pose).unwrap().unwrap();
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.5,0.0,-0.5), Vec3::new(0.5,1.0,0.5), 1.0);
        let layout = BattleSceneLayout::for_anchored_bodies([Some(body);2], feet, Vec2::new(800.0,600.0)).unwrap();
        let original = layout.body_poses[0].unwrap();
        assert!(ground.contains(body, original, feet[0].y));
        let mut far = original;
        far.translation += layout.source_displacement(Vec2::new(20.0 / crate::battle_layout::SOURCE_PIXEL_WORLD,0.0));
        assert!(!ground.contains(body, far, feet[0].y), "attack offsets may not cross the approved water boundary");
        body.visual_bounds = Some((Vec3::new(-16.0,0.0,-0.5),Vec3::new(16.0,1.0,0.5)));
        assert!(!ground.contains(body, original, feet[0].y), "full animated envelopes cannot be clamped to fit the lake");
    }

    fn trainer_fixture() -> (VisualWorldFrame, VisualBattleLocation, Vec<f32>) {
        let (frame, mut location, heights) = walking_fixture();
        let source = location.source.core_tile;
        let contact_tile = source - IVec2::X;
        location.source.facing = IVec2::NEG_X;
        location.target = VisualBattleTarget::Trainer {
            contact: crystal_render_api::VisualBattleTrainerTarget {
                object_identifier: "TRAINER_OBJECT".into(),
                object_script: "TrainerTable".into(),
                core_tile: contact_tile,
                facing: IVec2::X,
                trainer_class: "BIRD_KEEPER".into(),
                trainer_id: "ABE".into(),
                event_flag: "EVENT_BEAT_BIRD_KEEPER_ABE".into(),
                battle_source_script: "TrainerTable".into(),
                provenance: crystal_render_api::VisualBattleTrainerProvenance::TrainerTable { command_index: 0 },
                witnessed_actor: Some(VisualActorId::Object(0)),
                witnessed_foot: Some(Vec2::new(68.0, -124.0)),
            },
            presentation: crystal_render_api::VisualBattleDerivedTrainerPlacement {
                presentation_core_tile: source + IVec2::Y * 2,
                walkable_core_tiles: (-3..=3)
                    .flat_map(|y| (-3..=3).map(move |x| source + IVec2::new(x, y)))
                    .filter(|tile| *tile != contact_tile).collect::<Vec<_>>().into(),
            },
        };
        let anchors = Arc::make_mut(location.anchors.as_mut().unwrap());
        anchors.target_foot = Vec2::new(132.0, -252.0);
        (frame, location, heights)
    }

    #[test]
    fn trainer_ground_keeps_field_contact_separate_from_derived_pokemon_support() {
        let (frame, location, heights) = trainer_fixture();
        let cache = cache(&frame, heights);
        let (feet, root, _) = resolve_context(&location, &cache).unwrap();
        let ground = WalkingGround::from_context(&location, &cache, root).unwrap().unwrap();
        assert_eq!(location.target.core_tile(), IVec2::new(11, 8));
        assert_eq!(feet, [Vec3::new(0.0, 0.0, -4.0), Vec3::new(0.0, 0.0, 4.0)]);
        assert!(location.anchors.as_ref().unwrap().target_actor.is_none());
        let body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        for foot in feet {
            assert!(ground.contains(body, Transform::from_translation(foot), 0.0));
        }
        assert!(!ground.contains(body, Transform::from_xyz(-4.0, 0.0, -4.0), 0.0),
            "the trainer's occupied floor cannot support a Pokémon body");
    }

    #[test]
    fn trainer_ground_requires_settled_paired_actor_witness_and_safe_corridor() {
        let (frame, location, heights) = trainer_fixture();
        let cache = cache(&frame, heights);
        let (_, root, _) = resolve_context(&location, &cache).unwrap();
        for case in 0..11 {
            let mut bad = location.clone();
            let VisualBattleTarget::Trainer { contact, presentation } = &mut bad.target else { panic!() };
            match case {
                0 => contact.core_tile -= IVec2::X, // pre-approach identity is stale
                1 => contact.facing = IVec2::NEG_X,
                2 => contact.witnessed_actor = None,
                3 => contact.witnessed_foot = None,
                4 => contact.witnessed_actor = Some(VisualActorId::Player),
                5 => contact.witnessed_foot = Some(Vec2::splat(f32::NAN)),
                6 => contact.witnessed_foot = Some(Vec2::new(68.0, -123.0)),
                7 => presentation.walkable_core_tiles = vec![contact.core_tile, bad.source.core_tile,
                    bad.source.core_tile + IVec2::Y, presentation.presentation_core_tile].into(),
                8 => presentation.presentation_core_tile += IVec2::X,
                9 => presentation.walkable_core_tiles = vec![bad.source.core_tile,
                    presentation.presentation_core_tile].into(),
                _ => bad.source.movement = VisualBattleSourceMovement::Surf,
            }
            assert!(WalkingGround::from_context(&bad, &cache, root).is_err(), "case {case}");
        }
    }

    #[test]
    fn scripted_trainer_ground_keeps_real_empty_flag_and_ordered_command_provenance() {
        let (frame, mut location, heights) = trainer_fixture();
        let cache = cache(&frame, heights);
        let (_, root, _) = resolve_context(&location, &cache).unwrap();
        let VisualBattleTarget::Trainer { contact, .. } = &mut location.target else { panic!() };
        contact.event_flag = Arc::from("");
        assert!(WalkingGround::from_context(&location, &cache, root).is_err(),
            "ordinary trainer-table contact still requires its actual defeat flag");
        for (load, start, valid) in [(8, 9, true), (9, 9, false), (10, 9, false)] {
            let VisualBattleTarget::Trainer { contact, .. } = &mut location.target else { panic!() };
            contact.provenance = crystal_render_api::VisualBattleTrainerProvenance::Scripted {
                loadtrainer_command_index: load, startbattle_command_index: start,
            };
            assert_eq!(WalkingGround::from_context(&location, &cache, root).is_ok(), valid);
        }
        let VisualBattleTarget::Trainer { contact, .. } = &mut location.target else { panic!() };
        contact.provenance = crystal_render_api::VisualBattleTrainerProvenance::Scripted {
            loadtrainer_command_index: 8, startbattle_command_index: 9,
        };
        contact.witnessed_actor = None;
        assert!(WalkingGround::from_context(&location, &cache, root).is_err(),
            "script lineage never substitutes for the settled field actor witness");
    }

    #[test]
    fn trainer_ground_preserves_body_envelopes_elevations_and_source_displacement() {
        let (frame, location, mut heights) = trainer_fixture();
        let cell = IVec2::new(24, 22) - frame.grid_origin;
        heights[(cell.y as u32 * frame.grid_size.x + cell.x as u32) as usize] = 8.0;
        let cache = cache(&frame, heights);
        let (feet, root, _) = resolve_context(&location, &cache).unwrap();
        let ground = WalkingGround::from_context(&location, &cache, root).unwrap().unwrap();
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        assert!(ground.contains(body, Transform::from_translation(feet[0]), 0.0));
        assert!(!ground.contains(body, Transform::from_translation(feet[1]), 0.0),
            "the south-center body reaches a real different-height cell");
        body.visual_bounds = Some((Vec3::new(-12.0, 0.0, -0.5), Vec3::new(12.0, 1.0, 0.5)));
        assert!(!ground.contains(body, Transform::from_translation(feet[0]), 0.0));
        let layout = BattleSceneLayout::for_anchored_bodies([Some(body); 2], feet, Vec2::new(800.0, 600.0));
        assert!(layout.is_some(), "camera fit must not silently shrink the large body");
        let layout = layout.unwrap();
        assert_eq!(layout.body_poses[0].unwrap().scale, Vec3::ONE);
        body.visual_bounds = None;
        let layout = BattleSceneLayout::for_anchored_bodies([Some(body); 2], feet, Vec2::new(800.0, 600.0)).unwrap();
        let original = layout.clone();
        let neutral = layout.body_poses[0].unwrap();
        assert!(ground.contains(body, neutral, 0.0));
        let mut small = neutral;
        small.translation += layout.source_displacement(Vec2::new(3.0, 0.0));
        assert!(ground.contains(body, small, 0.0));
        let mut translated = neutral;
        translated.translation += layout.source_displacement(Vec2::new(
            8.0 / crate::battle_layout::SOURCE_PIXEL_WORLD, 0.0,
        ));
        assert!(translated.translation.abs_diff_eq(feet[1], 0.0001));
        assert!(!ground.contains(body, translated, 0.0),
            "a previously valid body must reject displacement onto the raised cell");
        assert_eq!(layout, original);
    }

    fn walking_region(location: &VisualBattleLocation, cache: &TerrainRevisionCache) -> WalkingGround {
        let (_, root, _) = resolve_context(location, cache).unwrap();
        WalkingGround::from_context(location, cache, root).unwrap().unwrap()
    }
    #[test]
    fn walking_support_accepts_the_bounded_wider_witness_and_rejects_beyond_it() {
        let (frame, mut location, heights) = walking_fixture();
        let source = location.source.core_tile;
        let radius = crystal_render_api::WALKING_BATTLE_GROUND_RADIUS;
        let VisualBattleTarget::WalkingGrass { presentation, .. } = &mut location.target else { panic!() };
        presentation.walkable_core_tiles = (-radius..=radius)
            .flat_map(|y| (-radius..=radius).map(move |x| source + IVec2::new(x, y)))
            .collect::<Vec<_>>().into();
        let cache = cache(&frame, heights);
        let (_, root, _) = resolve_context(&location, &cache).unwrap();
        assert!(WalkingGround::from_context(&location, &cache, root).unwrap().is_some());
        let VisualBattleTarget::WalkingGrass { presentation, .. } = &mut location.target else { panic!() };
        let mut tiles = presentation.walkable_core_tiles.to_vec();
        tiles.pop();
        tiles.push(source + IVec2::X * (radius + 1));
        presentation.walkable_core_tiles = tiles.into();
        assert!(WalkingGround::from_context(&location, &cache, root).is_err(),
            "a wider local witness must never admit unbounded map support");
    }

    #[test]
    fn walking_clearance_maps_actual_grid_and_full_animated_body_without_rescaling() {
        let (frame, location, heights) = walking_fixture();
        let cache = cache(&frame, heights);
        let (feet, _, _) = resolve_context(&location, &cache).unwrap();
        assert_eq!(feet, [Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -4.0)]);
        let ground = walking_region(&location, &cache);
        let mut body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        let pose = Transform::from_translation(feet[0]);
        assert!(ground.contains(body, pose, 0.0));
        body.visual_bounds = Some((Vec3::new(-20.0, 0.0, -0.5), Vec3::new(20.0, 1.0, 0.5)));
        assert!(!ground.contains(body, pose, 0.0), "neutral feet cannot authorize the wing envelope");
        assert_eq!(pose.scale, Vec3::ONE);
    }
    #[test]
    fn walking_clearance_rejects_holes_between_approved_tiles() {
        let (frame, mut location, heights) = walking_fixture();
        let VisualBattleTarget::WalkingGrass { presentation, .. } = &mut location.target else { panic!() };
        presentation.walkable_core_tiles = presentation.walkable_core_tiles.iter().copied().filter(|p| *p != IVec2::new(12, 9)).collect::<Vec<_>>().into();
        let cache = cache(&frame, heights);
        let ground = walking_region(&location, &cache);
        let body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        assert!(!ground.contains(body, Transform::from_xyz(0.0, 0.0, 4.0), 0.0), "south-center footprint touches the missing next row");
        assert!(ground.contains(body, Transform::from_xyz(0.0, 0.0, -4.0), 0.0));
    }
    #[test]
    fn walking_clearance_rejects_incompatible_built_elevation_and_invalid_evidence() {
        let (frame, location, mut heights) = walking_fixture();
        let grid = IVec2::new(24, 18) - frame.grid_origin;
        heights[(grid.y as u32 * frame.grid_size.x + grid.x as u32) as usize] = 8.0;
        let cache = cache(&frame, heights);
        let ground = walking_region(&location, &cache);
        let body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        assert!(!ground.contains(body, Transform::from_xyz(0.0, 0.0, 4.0), 0.0));
        let (_, root, _) = resolve_context(&location, &cache).unwrap();
        for case in 0..5 {
            let mut bad = location.clone();
            let VisualBattleTarget::WalkingGrass { core_tile, presentation } = &mut bad.target else { panic!() };
            match case {
                0 => *core_tile += IVec2::X,
                1 => presentation.presentation_core_tile += IVec2::X,
                2 => presentation.witnessed_player_foot = Some(Vec2::splat(f32::NAN)),
                3 => presentation.witnessed_player_foot = Some(Vec2::new(140.0, -156.0)),
                _ => presentation.walkable_core_tiles = vec![IVec2::new(12, 8); 50].into(),
            }
            assert!(WalkingGround::from_context(&bad, &cache, root).is_err(), "case {case}");
        }
    }
    #[test]
    fn walking_clearance_source_offsets_do_not_reframe_or_move_original_supports() {
        let (frame, location, heights) = walking_fixture();
        let cache = cache(&frame, heights);
        let ground = walking_region(&location, &cache);
        let (feet, _, _) = resolve_context(&location, &cache).unwrap();
        let body = BattleBody::modeled("TEST", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0);
        let layout = BattleSceneLayout::for_anchored_bodies([Some(body); 2], feet, Vec2::new(800.0, 600.0)).unwrap();
        let original = layout.clone();
        for index in 0..2 {
            let mut pose = layout.body_poses[index].unwrap();
            pose.translation += layout.source_displacement(Vec2::new(3.0, 4.0));
            assert!(ground.contains(body, pose, feet[index].y));
            pose.translation += layout.source_displacement(Vec2::new(500.0, 0.0));
            assert!(!ground.contains(body, pose, feet[index].y));
        }
        assert_eq!(layout, original);
    }
    #[test]
    fn derived_ground_retained_grid_translation_preserves_source_feet_geometry_and_clearance() {
        use crystal_render_api::{VisualTile, VisualTileSource};
        fn fill(frame: &mut VisualWorldFrame) {
            frame.tiles = (0..frame.grid_size.y)
                .flat_map(|row| (0..frame.grid_size.x).map(move |column| (column, row)))
                .map(|(column, row)| {
                    let source = frame.grid_origin + IVec2::new(column as i32, row as i32);
                    let id = ((source.y + 100) * 256 + source.x + 100) as u16;
                    VisualTile {
                        animation_frames: None,
                        column,
                        row,
                        source: VisualTileSource {
                            tileset_id: "walking-fixture".into(),
                            metatile_id: id,
                            subtile_column: 0,
                            subtile_row: 0,
                            tile_index: id,
                        },
                        texture: Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(u128::from(id) + 1000), std::marker::PhantomData),
                        priority: false,
                    }
                })
                .collect();
        }
        for (mut frame, mut location, heights) in [walking_fixture(), trainer_fixture()] {
            fill(&mut frame);
            Arc::make_mut(location.anchors.as_mut().unwrap())
                .terrain
                .tiles = frame.tiles.clone().into();
            let retained = cache(&frame, heights);
            let (original_feet, original_pose, original_bounds) =
                resolve_context(&location, &retained).unwrap();
            let body = BattleBody::modeled(
                "TEST",
                Vec3::new(-0.5, 0.0, -0.5),
                Vec3::new(0.5, 1.0, 0.5),
                1.0,
            );
            for shift in [
                IVec2::new(2, -2),
                IVec2::new(-2, 2),
                IVec2::X * 3,
                IVec2::NEG_Y * 3,
            ] {
                let mut live = frame.clone();
                live.grid_origin += shift;
                live.center += Vec2::new(27.0, -23.0);
                fill(&mut live);
                let foot_delta = live.center - frame.center
                    + Vec2::new(-shift.x as f32, shift.y as f32) * frame.tile_size;
                let mut moved = location.clone();
                let anchors = Arc::make_mut(moved.anchors.as_mut().unwrap());
                anchors.terrain.grid_origin = live.grid_origin;
                anchors.terrain.center = live.center;
                anchors.terrain.tiles = live.tiles.clone().into();
                anchors.source_foot += foot_delta;
                anchors.target_foot += foot_delta;
                match &mut moved.target {
                    VisualBattleTarget::WalkingGrass { presentation, .. } => {
                        *presentation.witnessed_player_foot.as_mut().unwrap() += foot_delta
                    }
                    VisualBattleTarget::Trainer { contact, .. } => {
                        *contact.witnessed_foot.as_mut().unwrap() += foot_delta
                    }
                    _ => panic!("expected derived placement fixture"),
                }
                let (feet, pose, bounds) = resolve_context(&moved, &retained).unwrap();
                assert_eq!(feet, original_feet);
                assert_eq!(pose, original_pose);
                assert_eq!(bounds.min, original_bounds.min);
                assert_eq!(bounds.max, original_bounds.max);
                let ground = walking_region(&moved, &retained);
                for foot in feet {
                    assert!(ground.contains(body, Transform::from_translation(foot), foot.y));
                }
                // A desired key or matching live scene still cannot replace an altered actual tile.
                let mut bad = cache(&frame, retained.built_footing_heights.clone());
                let tile = &mut bad.built_frame.as_mut().unwrap().tiles
                    [(24 * frame.grid_size.x + 32) as usize];
                tile.source.tile_index ^= 1;
                bad.key = Some(crate::TerrainCacheKey::from_frame(&live));
                assert!(resolve_context(&moved, &bad).is_err());
            }
        }
    }
    #[test]
    fn strict_support_rejects_off_grid_truncation_and_nonfinite_heights() {
        let (frame, _, mut heights) = fixture();
        assert_eq!(
            strict_foot(&frame, &heights, Vec2::new(132.0, -124.0)),
            Some(Vec3::new(132.0, 0.0, 124.0))
        );
        let outside = frame.center + Vec2::X * frame.grid_size.x as f32 * frame.tile_size.x;
        assert_eq!(
            crate::resolved_footing_height(&frame, outside, &heights),
            Some(0.0)
        );
        assert!(strict_foot(&frame, &heights, outside).is_none());
        let point = Vec2::new(132.0, -124.0);
        let tile = crate::tile_at_visual_point(&frame, point + Vec2::Y * 0.01).unwrap();
        heights[(tile.y * frame.grid_size.x + tile.x) as usize] = f32::NAN;
        assert!(strict_foot(&frame, &heights, point).is_none());
        heights.pop();
        assert!(strict_foot(&frame, &heights, Vec2::new(132.0, -60.0)).is_none());
    }
    #[test]
    fn exact_build_evidence_and_uniform_rebase_preserve_four_units_per_core_tile() {
        let (frame, location, heights) = fixture();
        let mut retained = cache(&frame, heights);
        let (anchors, pose, authentic) = resolve_context(&location, &retained).unwrap();
        assert_eq!(
            anchors,
            [Vec3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, -2.0)]
        );
        assert_eq!(pose.scale, Vec3::splat(1.0 / 16.0));
        assert_eq!(authentic.min, Vec2::new(-50.0, -34.0));
        assert_eq!(authentic.max, Vec2::new(46.0, 30.0));
        // An atlas copy is legitimate only with its original submission handle.
        retained.built_frame.as_mut().unwrap().map_texture = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(52), std::marker::PhantomData);
        assert!(resolve_context(&location, &retained).is_ok());
        retained.built_source_texture = Some(Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(53), std::marker::PhantomData));
        assert!(resolve_context(&location, &retained).is_err());
        retained.built_source_texture = Some(frame.map_texture.clone());
        retained.built_frame.as_mut().unwrap().grid_origin += IVec2::X;
        retained.key = Some(crate::TerrainCacheKey::from_frame(&frame));
        assert!(
            resolve_context(&location, &retained).is_err(),
            "a matching desired key cannot authorize stale built geometry"
        );
    }
    #[test]
    fn fishing_water_contact_keeps_real_support_and_rejects_missing_target_height() {
        let (frame, mut location, mut heights) = fixture();
        location.target = VisualBattleTarget::FishingWater {
            core_tile: IVec2::new(12, 7),
        };
        let anchors = Arc::make_mut(location.anchors.as_mut().unwrap());
        anchors.target_actor = None;
        let target = crate::tile_at_visual_point(
            &frame,
            anchors.target_foot + Vec2::Y * 0.01,
        ).unwrap();
        let index = (target.y * frame.grid_size.x + target.x) as usize;
        heights[index] = 16.0;
        let mut retained = cache(&frame, heights);
        let (feet, _, _) = resolve_context(&location, &retained).unwrap();
        assert_eq!(feet, [Vec3::new(0.0, -0.5, 2.0), Vec3::new(0.0, 0.5, -2.0)]);
        assert!(location.anchors.as_ref().unwrap().target_actor.is_none());
        retained.built_footing_heights[index] = f32::NAN;
        assert_eq!(
            resolve_context(&location, &retained).unwrap_err(),
            "target footing is outside actual built terrain"
        );
        retained.built_footing_heights.pop();
        assert!(resolve_context(&location, &retained).is_err());
    }
    #[test]
    fn elevation_and_nonzero_frame_centers_share_one_transform() {
        let (frame, location, mut heights) = fixture();
        let a = location.anchors.as_ref().unwrap();
        for (foot, height) in [(a.source_foot, 16.0), (a.target_foot, 48.0)] {
            let tile = crate::tile_at_visual_point(&frame, foot + Vec2::Y * 0.01).unwrap();
            heights[(tile.y * frame.grid_size.x + tile.x) as usize] = height;
        }
        let (feet, pose, _) = resolve_context(&location, &cache(&frame, heights)).unwrap();
        assert_eq!(feet, [Vec3::new(0.0, -1.0, 2.0), Vec3::new(0.0, 1.0, -2.0)]);
        // Mesh positions are grid-local; footing positions are already visual.
        assert_eq!(pose.transform_point(Vec3::new(32.0, 16.0, 64.0)), feet[0]);
    }
    #[test]
    fn tall_battler_uses_a_higher_shot_only_when_lower_views_cannot_fit() {
        let bodies = [
            Some(BattleBody::modeled("TALL_WIDE", Vec3::new(-8.0, 0.0, -4.0), Vec3::new(8.0, 12.0, 4.0), 1.0)),
            Some(BattleBody::modeled("SMALL", Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 2.0, 1.0), 1.0)),
        ];
        let anchors = [Vec3::new(0.0, 0.0, -4.0), Vec3::new(0.0, 0.0, 4.0)];
        let viewport = Vec2::new(1000.0, 700.0);
        let original = BattleSceneLayout::for_anchored_bodies(bodies, anchors, viewport).unwrap();
        // Witness the complete frustum of one real higher shot. Its minimal
        // source rectangle must suffice; unrelated extra ground is unnecessary.
        let mut compact = original.clone();compact.distance = 0.0;
        let mut higher = compact.battle_shot(bodies, 90.0_f32.to_radians(), 75.0_f32.to_radians()).unwrap();
        let inverse = higher.camera.rotation.inverse();
        let depth = bodies.into_iter().zip(higher.body_poses)
            .flat_map(|(body, pose)| body.unwrap().corners(pose.unwrap()))
            .map(|point| -(inverse * (point-higher.camera.translation)).z).fold(0.0_f32, f32::max);
        let radial = bodies.into_iter().zip(higher.body_poses)
            .flat_map(|(body, pose)| body.unwrap().corners(pose.unwrap()))
            .map(|point| point.distance(higher.camera.translation)).fold(0.0_f32, f32::max);
        let required_far = (depth+2.1).max(radial+0.7);
        let tangent = (CAMERA_FOV*0.5).tan();
        let eye = Vec2::new(higher.camera.translation.x, higher.camera.translation.z);
        let (mut min, mut max) = (eye, eye);
        for x in [-1.0, 1.0] {for y in [-1.0, 1.0] {
            let ray = higher.camera.rotation * Vec3::new(x*tangent*viewport.x/viewport.y, y*tangent, -1.0);
            let point = higher.camera.translation + ray*required_far;
            let point = Vec2::new(point.x, point.z);
            min = min.min(point);max = max.max(point);
        }}
        let authentic = AuthenticBounds {min:min-Vec2::splat(BORDER_MARGIN+0.01),max:max+Vec2::splat(BORDER_MARGIN+0.01)};
        let mut state = EncounterTerrain {
            frozen: Some(FrozenScene {
                root: Entity::from_raw_u32(1).unwrap(), anchors,
                authentic,
                omitted: StaticBlockers::default(), static_blockers: StaticBlockers::default(), grass_lens_blockers: StaticBlockers::default(), objects: HashMap::new(), backdrop_bounds: Vec::new(), materials: Vec::new(),
                images: Vec::new(), meshes: Vec::new(), ground: 0.0,
                walking: None, water: false, bodies: [None, None], clearance: [Vec4::ZERO; 2], clearance_shape: [Vec4::ZERO; 2], grass_contact: [Vec4::ZERO; 2], contact_shape: [Vec4::ZERO; 2], derived_stage: false, source_anchors: anchors,
            }),
            ..default()
        };
        assert!(state.constrain_camera(&mut higher, bodies, viewport), "the independently witnessed higher shot must fit: {:?}", state.reason());
        for elevation in [6.0_f32, 12.0, 18.0, 24.0, 32.0, 40.0, 48.0] {
            for yaw in [70.0_f32, 90.0, 110.0, 140.0, -70.0, -90.0, -110.0, -140.0] {
                let mut compact = original.clone();compact.distance = 0.0;
                let shot = compact.battle_shot(bodies, yaw.to_radians(), elevation.to_radians()).unwrap();
                for mut candidate in std::iter::once(shot.clone()).chain(shot.readable_facing(bodies)) {
                    assert!(!state.constrain_camera(&mut candidate, bodies, viewport), "fixture unexpectedly admits a low shot");
                }
            }
        }
        let mut layout = original.clone();
        assert!(state.establish_camera(&mut layout, bodies, viewport), "{:?}", state.reason());
        assert_eq!(layout.origins, original.origins);
        for (body, pose) in bodies.into_iter().zip(layout.body_poses) {
            let body = body.unwrap();let pose = pose.unwrap();
            assert_eq!(pose.scale, Vec3::ONE);
            for point in body.corners(pose) {
                assert!(state.frozen.as_ref().unwrap().authentic.point(point));
            }
        }
        assert!((-layout.camera.forward().y).asin().to_degrees() > 48.0);
        state.frozen.as_mut().unwrap().authentic = AuthenticBounds { min: Vec2::splat(-80.0), max: Vec2::splat(80.0) };
        let mut lower = original.clone();
        assert!(state.establish_camera_at(&mut lower, bodies, viewport, &LOW_SHOT_ELEVATIONS));
        let mut chosen = original.clone();
        assert!(state.establish_camera(&mut chosen, bodies, viewport));
        assert_eq!(chosen.camera, lower.camera, "a valid low shot must take precedence");
        // Removing the full wing's original ground remains a rejection.
        state.frozen.as_mut().unwrap().authentic = AuthenticBounds { min: Vec2::splat(-5.0), max: Vec2::splat(5.0) };
        assert!(!state.establish_camera(&mut original.clone(), bodies, viewport));
    }

    #[test]
    fn wide_battler_far_plane_uses_view_depth_and_keeps_every_corner_inside() {
        let bodies = [
            Some(BattleBody::modeled("WIDE", Vec3::new(-8.0, 0.0, -0.5), Vec3::new(8.0, 3.0, 0.5), 1.0)),
            Some(BattleBody::modeled("SMALL", Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 1.0, 0.5), 1.0)),
        ];
        let anchors = [Vec3::ZERO, Vec3::new(0.0, 0.0, 4.0)];
        let viewport = Vec2::new(1600.0, 600.0);
        let mut layout = BattleSceneLayout::for_anchored_bodies(bodies, anchors, viewport).unwrap();
        layout.camera = Transform::from_xyz(0.0, 4.0, 20.0)
            .looking_at(Vec3::new(0.0, 4.0, 0.0), Vec3::Y);
        layout.far = 23.0;
        let original = layout.clone();
        let mut state = EncounterTerrain {
            frozen: Some(FrozenScene {
                root: Entity::from_raw_u32(1).unwrap(), anchors,
                authentic: AuthenticBounds { min: Vec2::splat(-40.0), max: Vec2::splat(40.0) },
                omitted: StaticBlockers::default(), static_blockers: StaticBlockers::default(), grass_lens_blockers: StaticBlockers::default(), objects: HashMap::new(), backdrop_bounds: Vec::new(), materials: Vec::new(),
                images: Vec::new(), meshes: Vec::new(), ground: 0.0,
                walking: None, water: false, bodies: [None, None], clearance: [Vec4::ZERO; 2], clearance_shape: [Vec4::ZERO; 2], grass_contact: [Vec4::ZERO; 2], contact_shape: [Vec4::ZERO; 2], derived_stage: false, source_anchors: anchors,
            }),
            ..default()
        };
        assert!(bodies.into_iter().zip(layout.body_poses).flat_map(|(body, pose)| body.unwrap().corners(pose.unwrap()))
            .any(|point| point.distance(layout.camera.translation) + 2.0 > layout.far),
            "the fixture must distinguish radial distance from clipping depth");
        assert!(state.constrain_camera(&mut layout, bodies, viewport), "{:?}", state.reason());
        assert_eq!(layout.body_poses, original.body_poses);
        let projection = Mat4::perspective_rh(CAMERA_FOV, viewport.x / viewport.y, 0.1, layout.far);
        let view = layout.camera.to_matrix().inverse();
        for (body, pose) in bodies.into_iter().zip(layout.body_poses) {
            for point in body.unwrap().corners(pose.unwrap()) {
                let clip = projection * view * point.extend(1.0);
                let ndc = clip.truncate() / clip.w;
                assert!(point.distance(layout.camera.translation) < state.fog_range().unwrap().0, "fog must not hide a framed wing tip");
                assert!(ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0 && (0.0..=1.0).contains(&ndc.z), "{ndc:?}");
            }
        }
        let mut fogged_bodies = bodies;
        let mut fogged = bodies[0].unwrap();fogged.min.x = -12.0;fogged.max.x = 12.0;
        fogged_bodies[0] = Some(fogged);
        assert!(!state.constrain_camera(&mut original.clone(), fogged_bodies, viewport));
        assert_eq!(state.reason(), Some("authentic terrain cannot keep complete battlers clear of fog"));
        let mut clipped = original.clone();
        clipped.far = 20.0;
        assert!(!state.constrain_camera(&mut clipped, bodies, viewport), "far plane must not clip a wing");
        state.frozen.as_mut().unwrap().authentic.max.x = 10.0;
        assert!(!state.constrain_camera(&mut original.clone(), bodies, viewport), "source terrain must still contain every wing corner");
    }

    #[test]
    fn border_crossing_leaf_and_offscreen_shadow_are_conservatively_rejected() {
        let authentic = AuthenticBounds {
            min: Vec2::splat(-20.0),
            max: Vec2::splat(20.0),
        };
        let crossing = Bounds {
            min: Vec3::new(19.0, 0.0, 0.0),
            max: Vec3::new(21.0, 2.0, 2.0),
        };
        assert!(!authentic.contains(crossing));
        let camera = Transform::from_xyz(0.0, 5.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y);
        let viewport = Vec2::new(16.0, 9.0);
        let caster = Bounds {
            min: Vec3::new(8.0, 10.0, 8.0),
            max: Vec3::new(9.0, 20.0, 9.0),
        };
        assert!(!visible_bounds(caster, camera, viewport, 25.0));
        assert!(visible_bounds(
            caster.shadow_volume(0.0),
            camera,
            viewport,
            25.0
        ));
        assert!(authentic.camera_limit(camera, viewport).unwrap() < 100.0);
        assert!(
            authentic
                .camera_limit(Transform::from_xyz(21.0, 5.0, 0.0), viewport)
                .is_none()
        );
    }

    fn app_fixture() -> (
        App,
        Entity,
        Handle<Mesh>,
        Handle<VoxelMaterial>,
        Handle<Image>,
    ) {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<BattleViewStatus>()
            .init_resource::<VisualBattleFrame>()
            .init_resource::<BattleFlashMode>()
            .init_resource::<EncounterTerrain>()
            .add_systems(Update, sync);
        let (mut frame, mut location, heights) = fixture();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_fill(
                bevy::render::render_resource::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                &[10, 20, 30, 255],
                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                bevy::asset::RenderAssetUsages::default(),
            ));
        frame.map_texture = image.clone();
        Arc::make_mut(location.anchors.as_mut().unwrap())
            .terrain
            .map_texture = image.clone();
        let material = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::textured_terrain_material(image.clone()));
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(512.0, 1.0, 512.0));
        let bounds = Aabb::from_min_max(
            Vec3::new(-256.0, -0.5, -256.0),
            Vec3::new(256.0, 0.5, 256.0),
        );
        let root = app.world_mut().spawn((Transform::default(), Visibility::default())).id();
        let leaf = app
            .world_mut()
            .spawn((
                (Mesh3d(mesh.clone()),
MeshMaterial3d(material.clone()),),
                bounds,
                RenderLayers::layer(crate::VOXEL_RENDER_LAYER),
            ))
            .id();
        app.world_mut().entity_mut(root).add_child(leaf);
        let instances = app.world_mut().spawn((Transform::default(), Visibility::default())).id();
        let apron = app
            .world_mut()
            .spawn((
                (Mesh3d(mesh.clone()),
MeshMaterial3d(material.clone()),),
                bounds,
                SyntheticTerrainApron,
                RenderLayers::layer(crate::VOXEL_RENDER_LAYER),
            ))
            .id();
        app.world_mut().entity_mut(instances).add_child(apron);
        let empty = app.world_mut().spawn((Transform::default(), Visibility::default())).id();
        let mut retained = cache(&frame, heights);
        retained.instances_root = Some(instances);
        retained.textured_entity = Some(root);
        retained.solid_entity = Some(empty);
        app.insert_resource(retained)
            .insert_resource(BattleLocationFrame {
                location: Some(Arc::new(location)),
            });
        (app, leaf, mesh, material, image)
    }
    #[test]
    fn map_preparation_retries_published_material_without_replacing_source_roots() {
        let (mut app, _, _, material, _) = app_fixture();
        let original = app.world_mut().resource_mut::<Assets<VoxelMaterial>>()
            .remove(material.id()).unwrap();
        prepare(app.world_mut());
        let state = app.world().resource::<EncounterTerrain>();
        assert!(state.frozen.is_none());
        assert_eq!(state.reason(), Some("retained terrain material is unavailable"));
        assert!(state.attempted_roots.is_none(), "pending assets must be retried on the same roots");
        let generation = state.generation();
        app.world_mut().resource_mut::<Assets<VoxelMaterial>>()
            .insert(material.id(), original).unwrap();
        prepare(app.world_mut());
        let state = app.world().resource::<EncounterTerrain>();
        assert!(state.frozen.is_some(), "published source material must admit the real map");
        assert_eq!(state.generation(), generation);
        assert!(state.reason().is_none());
        assert!(!preparation_pending("actual built terrain does not match encounter evidence"));
        assert!(!preparation_pending("terrain leaf has nonfinite bounds"));
    }

    #[test]
    fn retained_faded_world_crown_recovers_mask_and_keeps_it_through_opacity_updates() {
        let (mut app, _, _, material, _) = app_fixture();
        {
            let mut materials = app.world_mut().resource_mut::<Assets<VoxelMaterial>>();
            let mut source = materials.get_mut(&material).unwrap();
            source.base.alpha_mode = AlphaMode::Blend;
            source.extension.cutaway.fade = Vec4::new(0.8, 0.0, 0.0, 0.63);
        }
        prepare(app.world_mut());
        let frozen = app.world().resource::<EncounterTerrain>().frozen.as_ref().unwrap();
        assert_eq!(frozen.materials[0].alpha_cutoff, Some(0.63));
        let copy = app.world().resource::<Assets<VoxelMaterial>>().get(&frozen.materials[0].handle).unwrap();
        assert_eq!(copy.base.alpha_mode, AlphaMode::Mask(0.63));
        assert_eq!(retained_alpha_mode(1.0, Some(0.63)), AlphaMode::Mask(0.63));
        assert_eq!(retained_alpha_mode(0.2, Some(0.63)), AlphaMode::Blend);
    }

    #[test]
    fn retained_encounter_preserves_cutout_material_and_alpha_pixels() {
        let (mut app, _, _, material, image)=app_fixture();
        app.world_mut().resource_mut::<Assets<VoxelMaterial>>().get_mut(&material).unwrap().base.alpha_mode=AlphaMode::Mask(0.5);
        app.world_mut().resource_mut::<Assets<Image>>().get_mut(&image).unwrap().data.as_mut().expect("CPU image pixels")[3]=0;
        prepare(app.world_mut());
        let root=app.world().resource::<EncounterTerrain>().frozen.as_ref().unwrap().root;
        let child=app.world().get::<Children>(root).unwrap()[0];
        let copied=&app.world().get::<MeshMaterial3d<VoxelMaterial>>(child).unwrap().0;
        let material=app.world().resource::<Assets<VoxelMaterial>>().get(copied).unwrap();
        assert_eq!(material.base.alpha_mode,AlphaMode::Mask(0.5));
        let copied_image=material.base.base_color_texture.as_ref().unwrap();
        assert_ne!(copied_image,&image);
        assert_eq!(app.world().resource::<Assets<Image>>().get(copied_image).unwrap().data.as_ref().expect("CPU image pixels")[3],0);
    }

    #[test]
    fn handles_are_reused_assets_are_frozen_once_and_retirement_cleans_only_the_instances() {
        let (mut app, original, mesh, material, image) = app_fixture();
        let counts = |app: &App| {
            (
                app.world().resource::<Assets<Mesh>>().len(),
                app.world().resource::<Assets<VoxelMaterial>>().len(),
                app.world().resource::<Assets<Image>>().len(),
            )
        };
        let baseline = counts(&app);
        prepare(app.world_mut());
        let state = app.world().resource::<EncounterTerrain>();
        let frozen = state.frozen.as_ref().unwrap();
        let root = frozen.root;
        let children = app.world().get::<Children>(root).unwrap();
        assert_eq!(children.len(), 1, "the synthetic apron is never instanced");
        let instance = children[0];
        let frozen_material = app
            .world()
            .get::<MeshMaterial3d<VoxelMaterial>>(instance)
            .unwrap()
            .0
            .clone();
        assert_ne!(frozen_material, material);
        assert_eq!(app.world().get::<Mesh3d>(instance).map(|mesh| &mesh.0), Some(&mesh));
        assert_eq!(
            *app.world().get::<RenderLayers>(instance).unwrap(),
            RenderLayers::layer(29)
        );
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Hidden
        );
        let frozen_image = app
            .world()
            .resource::<Assets<VoxelMaterial>>()
            .get(&frozen_material)
            .unwrap()
            .base
            .base_color_texture
            .clone()
            .unwrap();
        assert_ne!(frozen_image, image);
        let retained_counts = counts(&app);
        assert_eq!(
            retained_counts,
            (baseline.0, baseline.1 + 1, baseline.2 + 1)
        );
        let frozen_pixels = app
            .world()
            .resource::<Assets<Image>>()
            .get(&frozen_image)
            .unwrap()
            .data.as_ref().expect("CPU image pixels")
            .clone();
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .get_mut(&image)
            .unwrap()
            .data.as_mut().expect("CPU image pixels")
            .fill(91);
        app.world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .get_mut(&material)
            .unwrap()
            .extension
            .cutaway
            .fade
            .x = 0.8;
        for _ in 0..30 {
            prepare(app.world_mut());
            app.update();
        }
        assert_eq!(counts(&app), retained_counts);
        assert_eq!(
            app.world()
                .resource::<Assets<Image>>()
                .get(&frozen_image)
                .unwrap()
                .data.as_ref().expect("CPU image pixels").as_slice(),
            frozen_pixels.as_slice()
        );
        assert_eq!(
            app.world()
                .resource::<Assets<VoxelMaterial>>()
                .get(&frozen_material)
                .unwrap()
                .extension
                .cutaway
                .fade
                .x,
            0.0
        );
        // Temporary source fallback and terminal narration retain all assets.
        app.world_mut().resource_mut::<EncounterTerrain>().accepted = true;
        app.world_mut().resource_mut::<BattleViewStatus>().active = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Visible
        );
        app.world_mut().resource_mut::<BattleViewStatus>().active = false;
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .use_source_scene = true;
        app.update();
        prepare(app.world_mut());
        assert_eq!(
            *app.world().get::<Visibility>(root).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(counts(&app), retained_counts);
        app.world_mut()
            .resource_mut::<BattleLocationFrame>()
            .location = None;
        prepare(app.world_mut());
        assert!(app.world().get_entity(root).is_err());
        assert!(app.world().get_entity(instance).is_err());
        assert!(app.world().get_entity(original).is_ok());
        assert_eq!(counts(&app), baseline);
        assert_eq!(
            *app.world().get::<RenderLayers>(original).unwrap(),
            RenderLayers::layer(crate::VOXEL_RENDER_LAYER)
        );
    }
    #[test]
    fn grass_camera_admission_retains_crossing_and_background_blades() {
        let camera = Transform::IDENTITY;
        let bounds = |near, far| Bounds {
            min: Vec3::new(-1.0, -1.0, far),
            max: Vec3::new(1.0, 1.0, near),
        };
        assert!(grass_bounds_discarded(bounds(-1.0, -2.0), camera, -3.0));
        assert!(!grass_bounds_discarded(bounds(-2.0, -4.0), camera, -3.0));
        assert!(!grass_bounds_discarded(bounds(-4.0, -5.0), camera, -3.0));
        assert!(!grass_bounds_discarded(bounds(-1.0, -2.0), camera, f32::INFINITY));
        // Strict comparison matches the fragment shader at the cut plane.
        assert!(!grass_bounds_discarded(bounds(-2.0, -3.0), camera, -3.0));
        let rotated = Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI));
        assert!(grass_bounds_discarded(bounds(-4.0, -5.0), rotated, -3.0));
    }

    #[test]
    fn finite_side_camera_fits_real_rigs_without_relaxing_authentic_or_omission_limits() {
        let cyndaquil = crate::species_rig::for_species("CYNDAQUIL").unwrap();
        let bird = crate::species_rig::for_species("PIDGEOTTO").unwrap();
        let bird_min = bird.neutral.positions.iter().map(|p| Vec3::from_array(*p))
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let bird_max = bird.neutral.positions.iter().map(|p| Vec3::from_array(*p))
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        // Sizes are the current pack values used by this two-party fixture.
        let bodies = [
            ("CYNDAQUIL", 0.508, cyndaquil.neutral_bounds, cyndaquil.animated_bounds),
            ("PIDGEOTTO", 1.0922, (bird_min, bird_max), bird.animated_bounds),
        ].map(|(species, meters, (min, max), envelope)| {
            let scale = crate::battle_layout::model_scale(species, Some(meters), min, max).unwrap();
            let mut body = BattleBody::modeled(species, min, max, scale);
            body.visual_bounds = Some(envelope);
            Some(body)
        });
        let anchors = [Vec3::new(0.0, 0.0, -4.0), Vec3::new(0.0, 0.0, 4.0)];
        let viewport = Vec2::new(800.0, 600.0);
        let primary = BattleSceneLayout::for_anchored_bodies(bodies, anchors, viewport).unwrap();
        let mut state = EncounterTerrain {
            frozen: Some(FrozenScene {
                root: Entity::from_raw_u32(1).unwrap(), anchors,
                authentic: AuthenticBounds { min: Vec2::new(-18.0, -12.0), max: Vec2::new(22.0, 52.0) },
                omitted: StaticBlockers::default(), static_blockers: StaticBlockers::default(), grass_lens_blockers: StaticBlockers::default(), objects: HashMap::new(), backdrop_bounds: Vec::new(), materials: Vec::new(),
                images: Vec::new(), meshes: Vec::new(), ground: 0.0,
                walking: None, water: false, bodies: [None, None], clearance: [Vec4::ZERO; 2], clearance_shape: [Vec4::ZERO; 2], grass_contact: [Vec4::ZERO; 2], contact_shape: [Vec4::ZERO; 2], derived_stage: false, source_anchors: anchors,
            }),
            ..default()
        };
        // The full standard bird envelope no longer fits the old narrow
        // fixture. Require rejection there, retaining original size and every
        // animated bound, then exercise the finite side view with enough retained
        // foreground for its full top-down frustum.
        let mut tight=primary.clone();
        assert!(!state.constrain_layout(&mut tight,bodies,viewport));
        assert_eq!(tight,primary);
        state.frozen.as_mut().unwrap().authentic=AuthenticBounds {
            min:Vec2::new(-18.0,-24.0),max:Vec2::new(22.0,52.0),
        };
        let mut layout = primary.clone();
        assert!(!state.constrain_camera(&mut layout, bodies, viewport));
        assert_eq!(layout, primary);
        // Exercise the existing 75-degree fallback directly even when the
        // preferred lower view also fits this fixture's authentic bounds.
        let mut side_accepted = false;
        for clockwise in [false, true] {
            let mut side = primary.side_camera(bodies, clockwise).unwrap();
            if state.constrain_camera(&mut side, bodies, viewport) {
                side_accepted = true;
                assert_eq!(side.body_poses, primary.body_poses);
                assert_eq!(side.hit_anchors, primary.hit_anchors);
            }
        }
        assert!(side_accepted,"{:?}; bodies={bodies:?}; primary={primary:?}",state.reason());
        assert!(state.constrain_layout(&mut layout, bodies, viewport), "{:?}", state.reason());
        assert_ne!(layout.camera, primary.camera);
        assert_eq!(layout.origins, primary.origins);
        assert_eq!(layout.body_poses, primary.body_poses);
        assert_eq!(layout.hit_anchors, primary.hit_anchors);
        let scene = state.frozen.as_ref().unwrap();
        assert!(scene.authentic.camera_limit(layout.camera, viewport).unwrap() >= layout.far);
        for (body, pose) in bodies.into_iter().zip(layout.body_poses) {
            for point in body.unwrap().corners(pose.unwrap()) {
                assert!(scene.authentic.point(point));
                assert!(layout.far >= -(layout.camera.rotation.inverse() * (point - layout.camera.translation)).z + 2.0);
            }
        }
        // Foreground blades are completely discarded by the shader, even
        // when their retained bounds surround the lens.
        let eye = layout.camera.translation;
        state.frozen.as_mut().unwrap().grass_lens_blockers = StaticBlockers::new(vec![Bounds {
            min: eye - Vec3::splat(0.5), max: eye + Vec3::new(0.5, 2.0, 0.5),
        }]);
        let mut grass_trial = layout.clone();
        assert!(state.constrain_camera(&mut grass_trial, bodies, viewport));
        assert_eq!(grass_trial.body_poses, layout.body_poses);
        state.frozen.as_mut().unwrap().grass_lens_blockers = StaticBlockers::new(vec![Bounds {
            min: Vec3::new(eye.x - 0.5, -0.10, eye.z - 0.5),
            max: Vec3::new(eye.x + 0.5, 0.04, eye.z + 0.5),
        }]);
        assert!(state.constrain_camera(&mut grass_trial, bodies, viewport));
        let ahead = eye.lerp((layout.hit_anchors[0] + layout.hit_anchors[1]) * 0.5, 0.35);
        state.frozen.as_mut().unwrap().grass_lens_blockers = StaticBlockers::new(vec![Bounds {
            min: ahead - Vec3::splat(0.2), max: ahead + Vec3::splat(0.2),
        }]);
        assert!(state.constrain_camera(&mut grass_trial, bodies, viewport));
        state.frozen.as_mut().unwrap().grass_lens_blockers = StaticBlockers::default();
        // An omitted object crossing the entire scene blocks every view. No
        // alternate camera may turn that missing evidence into a visible hole.
        state.frozen.as_mut().unwrap().omitted.push(Bounds {
            min: Vec3::splat(-100.0), max: Vec3::splat(100.0),
        });
        let mut rejected = primary.clone();
        assert!(!state.constrain_layout(&mut rejected, bodies, viewport));
        assert_eq!(rejected, primary);
        assert!(!state.accepted());
        assert!(state.fog_range().is_none());
        state.frozen.as_mut().unwrap().omitted.clear();
        state.frozen.as_mut().unwrap().authentic = AuthenticBounds {
            min: Vec2::splat(-100.0), max: Vec2::splat(100.0),
        };
        let mut ordinary = primary.clone();
        assert!(state.constrain_layout_with_trial(&mut ordinary, bodies, viewport, false));
        assert_eq!(ordinary.camera, primary.camera, "ordinary play keeps the established camera");
        assert_eq!(ordinary.body_poses, primary.body_poses);
        assert_eq!(ordinary.hit_anchors, primary.hit_anchors);
        let mut trial = primary.clone();
        assert!(state.constrain_layout_with_trial(&mut trial, bodies, viewport, true));
        let preferred = primary.three_quarter_camera(bodies).unwrap();
        assert_eq!(trial.camera, preferred.camera, "the explicit trial uses the proven lower view");
        assert_eq!(trial.body_poses, primary.body_poses);
        assert_eq!(trial.hit_anchors, primary.hit_anchors);
        let body_depth = bodies.into_iter().zip(trial.body_poses)
            .flat_map(|(body, pose)| body.unwrap().corners(pose.unwrap()))
            .map(|point| -(trial.camera.rotation.inverse() * (point - trial.camera.translation)).z).fold(0.0_f32, f32::max);
        assert!(trial.far >= body_depth + 20.0, "complete map scenery should survive behind the battle");
        let wall = trial.camera.translation.lerp(trial.hit_anchors[0], 0.5);
        state.frozen.as_mut().unwrap().static_blockers.push(Bounds {
            min: wall - Vec3::splat(0.3), max: wall + Vec3::splat(0.3),
        });
        let mut occluded = trial.clone();
        assert!(!state.constrain_camera(&mut occluded, bodies, viewport),
            "genuine terrain cannot be accepted while it hides a Pokémon");
        assert_eq!(state.reason(), Some("static terrain occludes a battler"));
        assert_eq!(occluded.body_poses, trial.body_poses);
        state.frozen.as_mut().unwrap().static_blockers.clear();
        assert!(state.constrain_camera(&mut trial, bodies, viewport));
        let mut neutral = bodies[0].unwrap();
        neutral.visual_bounds = None;
        let corner = neutral.corners(trial.body_poses[0].unwrap()).into_iter()
            .filter(|point| point.y > anchors[0].y + 0.25)
            .max_by(|a, b| a.distance_squared(trial.hit_anchors[0])
                .total_cmp(&b.distance_squared(trial.hit_anchors[0]))).unwrap();
        let edge = trial.camera.translation.lerp(corner, 0.85);
        let edge_blocker = Bounds { min: edge - Vec3::splat(0.02), max: edge + Vec3::splat(0.02) };
        assert!(!edge_blocker.segment(trial.camera.translation, trial.hit_anchors[0]));
        state.frozen.as_mut().unwrap().static_blockers.push(edge_blocker);
        assert!(!state.constrain_camera(&mut trial, bodies, viewport),
            "a clear center cannot excuse scenery clipping a body edge");
        assert_eq!(state.reason(), Some("static terrain occludes a battler"));
        state.frozen.as_mut().unwrap().static_blockers.clear();
        assert!(state.constrain_camera(&mut trial, bodies, viewport));
        // An obstacle beside the neutral sculpture can intersect its temporary
        // source picture bank. Reserve that sight coverage before send-out.
        let mut bank_bodies = bodies;
        bank_bodies[0] = Some(bank_bodies[0].unwrap().with_source_card_envelope(1.0));
        let mut bank_view = BattleSceneLayout::for_anchored_bodies(bank_bodies, anchors, viewport)
            .unwrap().three_quarter_camera(bank_bodies).unwrap();
        assert!(state.constrain_camera(&mut bank_view, bank_bodies, viewport));
        let bank_blocker = bank_bodies[0].unwrap().sight_corners(bank_view.body_poses[0].unwrap())
            .into_iter().filter(|point| point.y > anchors[0].y + 0.25)
            .find_map(|corner| {
                let edge = bank_view.camera.translation.lerp(corner, 0.85);
                let blocker = Bounds { min: edge - Vec3::splat(0.002), max: edge + Vec3::splat(0.002) };
                state.frozen.as_mut().unwrap().static_blockers.push(blocker);
                let clear = state.constrain_camera(&mut bank_view, bodies, viewport);
                state.frozen.as_mut().unwrap().static_blockers.clear();
                clear.then_some(blocker)
            }).expect("fixture requires a source-bank edge outside the neutral sight rays");
        state.frozen.as_mut().unwrap().static_blockers.push(bank_blocker);
        assert!(!state.constrain_camera(&mut bank_view, bank_bodies, viewport),
            "a source picture bank must not bypass the ordinary occlusion proof");
        assert_eq!(state.reason(), Some("static terrain occludes a battler"));
        state.frozen.as_mut().unwrap().static_blockers.clear();
        assert!(state.constrain_camera(&mut trial, bodies, viewport));
        state.frozen.as_mut().unwrap().objects.insert(Entity::from_raw_u32(2).unwrap(), FrozenObject {
            bounds: Bounds { min: wall - Vec3::splat(0.3), max: wall + Vec3::splat(0.3) },
            backdrop: Vec::new(),
            surface: None,
            faded: true, opacity: RESIDUAL_OPACITY,
        });
        assert!(!state.constrain_camera(&mut trial, bodies, viewport),
            "a faded wall is still an obstruction to a readable battle shot");
        assert_eq!(state.reason(), Some("foreground scenery obstructs the battle shot"));
        state.frozen.as_mut().unwrap().objects.clear();
        assert!(state.constrain_camera(&mut trial, bodies, viewport));
        state.frozen.as_mut().unwrap().derived_stage = true;
        let divider = (trial.hit_anchors[0] + trial.hit_anchors[1]) * 0.5;
        state.frozen.as_mut().unwrap().static_blockers.push(Bounds {
            min: divider - Vec3::splat(0.3), max: divider + Vec3::splat(0.3),
        });
        assert!(!state.constrain_camera(&mut trial, bodies, viewport));
        assert_eq!(state.reason(), Some("scenery separates the battlers"));
        state.frozen.as_mut().unwrap().static_blockers.clear();
        state.frozen.as_mut().unwrap().derived_stage = false;
        let foreground = trial.camera.translation.lerp(
            (trial.hit_anchors[0] + trial.hit_anchors[1]) * 0.5, 0.5);
        let divider = Bounds { min: foreground - Vec3::splat(0.3), max: foreground + Vec3::splat(0.3) };
        assert!(trial.hit_anchors.iter().all(|point| !divider.segment(trial.camera.translation, *point)),
            "both fighters remain individually visible in this fixture");
        state.frozen.as_mut().unwrap().static_blockers.push(divider);
        assert!(!state.constrain_camera(&mut trial, bodies, viewport));
        assert_eq!(state.reason(), Some("foreground scenery divides the battle composition"));
        state.frozen.as_mut().unwrap().static_blockers.clear();
        let tiny = [Some(BattleBody::modeled("TEST", Vec3::new(-0.1, 0.0, -0.1), Vec3::new(0.1, 0.2, 0.1), 1.0)); 2];
        let mut too_small = BattleSceneLayout::for_map_bodies(tiny, anchors, viewport).unwrap();
        assert!(!state.establish_camera_at(&mut too_small, tiny, viewport, &LOW_SHOT_ELEVATIONS),
            "physical eligibility alone cannot accept an unreadably distant shot");
        assert!(!state.accepted());
        assert_eq!(state.reason(), Some("no readable low shot on checked ground"));
    }

    #[test]
    fn experimental_camera_requires_the_exact_explicit_startup_flag() {
        for value in [None, Some(""), Some("0"), Some("false"), Some("true"), Some("01")] {
            assert!(!camera_trial_requested(value));
        }
        assert!(camera_trial_requested(Some("1")));
    }

    #[test]
    fn native_review_yaw_only_selects_supported_establishing_angles() {
        for value in [None,Some(""),Some("0"),Some("1"),Some("45"),Some("180"),Some("NaN"),Some("inf")] {
            assert_eq!(review_shot_yaw(value),None,"{value:?} must retain the normal camera search");
        }
        for yaw in [70.,90.,110.,140.,-70.,-90.,-110.,-140.] {
            assert_eq!(review_shot_yaw(Some(&format!("{yaw}"))),Some(yaw));
        }
    }

    #[test]
    fn eligibility_sets_fog_beyond_complete_bodies_and_a_rejected_layout_hides_context() {
        let (mut app, _, _, _, _) = app_fixture();
        prepare(app.world_mut());
        let body = BattleBody::modeled(
            "TEST",
            Vec3::new(-0.5, 0.0, -0.5),
            Vec3::new(0.5, 2.0, 0.5),
            1.0,
        );
        let bodies = [Some(body); 2];
        let viewport = Vec2::new(16.0, 9.0);
        let mut layout = BattleSceneLayout::for_bodies(bodies, viewport);
        let anchors = app
            .world()
            .resource::<EncounterTerrain>()
            .anchors()
            .unwrap();
        layout.origins = anchors;
        layout.hit_anchors = anchors.map(|p| p + Vec3::Y);
        layout.body_poses = anchors.map(|p| Some(Transform::from_translation(p)));
        let mut state = app.world_mut().resource_mut::<EncounterTerrain>();
        assert!(
            state.constrain_layout(&mut layout, bodies, viewport),
            "{:?}",
            state.reason()
        );
        let (start, end) = state.fog_range().unwrap();
        for pose in layout.body_poses.into_iter().flatten() {
            for point in body.corners(pose) {
                let depth =
                    -(layout.camera.rotation.inverse() * (point - layout.camera.translation)).z;
                assert!(depth < start && depth < end);
                assert!(point.distance(layout.camera.translation) < start);
            }
        }
        assert!(layout.far < 100.0);
        state.reject_layout("body fit rejected");
        assert!(!state.accepted());
        assert!(state.fog_range().is_none());
        assert!(state.anchors().is_some());
    }
    #[test]
    fn default_world_palette_is_identity_and_reduced_flash_uses_the_shared_source_rule() {
        let world_material = crate::solid_terrain_material();
        assert_eq!(world_material.extension.cutaway.fade, Vec4::ZERO);
        let shader = include_str!("voxel_surface.wesl");
        assert!(shader.contains("mix(shaded, vec3<f32>(0.0), clamp(terrain.cutaway.fade.y, 0.0, 1.0))"));
        assert!(shader.contains("mix(darkened, vec3<f32>(1.0), clamp(terrain.cutaway.fade.z, 0.0, 1.0))"));
        let source = crystal_render_api::VisualBattleSourceFrame {
            frame: 0,
            bgp: 0,
            battler_bgps: [0xe4; 2],
            battler_palettes: [[[0.0; 4]; 4]; 2],
            battler_textures: std::array::from_fn(|_| Handle::default()),
            battler_offsets: [Vec2::ZERO; 2],
            screen_offset: Vec2::ZERO,
            line_x_offsets: None,
            line_y_offsets: None,
            objects: Vec::new(),
            battler_rows: [None; 2],
        };
        let full =
            crate::battle_view::source_environment_palette(Some(&source), BattleFlashMode::Full);
        let reduced =
            crate::battle_view::source_environment_palette(Some(&source), BattleFlashMode::Reduced);
        assert_eq!(full, (0.0, 1.0));
        assert!(reduced.1 > 0.0 && reduced.1 < full.1);
    }

    fn animated_fixture(app: &mut App, material: Handle<VoxelMaterial>, size: f32) -> Handle<Mesh> {
        let mut mesh = Mesh::from(Cuboid::new(size, 10.0, size));
        let count = mesh.count_vertices();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.17, 0.29, 0.43, 1.0]; count]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.13, 0.91]; count]);
        let bounds = mesh.get_aabb().unwrap();
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(crate::retain_animated_mesh_cpu(mesh));
        let entity = app
            .world_mut()
            .spawn((
                (Mesh3d(mesh.clone()),
MeshMaterial3d(material),),
                bounds,
                RenderLayers::layer(crate::VOXEL_RENDER_LAYER),
            ))
            .id();
        let mut cache = app.world_mut().resource_mut::<TerrainRevisionCache>();
        cache.animated_solid_entity = Some(entity);
        cache.animated_solid_mesh = Some(mesh.clone());
        cache.built_animated_bounds[1] = Some(bounds);
        mesh
    }

    fn mesh_bytes(mesh: &Mesh) -> Vec<u8> {
        use bevy::mesh::Indices;
        let mut bytes = Vec::new();
        for (id, values) in mesh.attributes() {
            bytes.extend(format!("{id:?}").as_bytes());
            bytes.extend(values.get_bytes());
        }
        match mesh.indices() {
            Some(Indices::U16(indices)) => {
                bytes.push(16);
                for index in indices {
                    bytes.extend(index.to_le_bytes());
                }
            }
            Some(Indices::U32(indices)) => {
                bytes.push(32);
                for index in indices {
                    bytes.extend(index.to_le_bytes());
                }
            }
            None => bytes.push(0),
        }
        bytes
    }

    #[test]
    fn crossing_coverage_preserves_all_interior_and_exterior_volume_including_zero_thickness_walls() {
        let authentic = AuthenticBounds { min: Vec2::splat(-2.0), max: Vec2::splat(2.0) };
        for bounds in [
            Bounds { min: Vec3::new(-5.0, -2.0, -4.0), max: Vec3::new(6.0, 12.0, 8.0) },
            Bounds { min: Vec3::new(-5.0, -2.0, 1.0), max: Vec3::new(6.0, 12.0, 1.0) },
            Bounds { min: Vec3::new(1.0, -2.0, -4.0), max: Vec3::new(1.0, 12.0, 8.0) },
        ] {
            let (known, exterior) = authentic.partition_coverage(bounds);
            let known = known.unwrap();
            assert!(authentic.contains(known));
            assert!(!exterior.is_empty());
            for x in 0..=20 {
                for z in 0..=20 {
                    for y in [bounds.min.y, bounds.max.y] {
                        let point = Vec3::new(bounds.min.x + (bounds.max.x - bounds.min.x) * x as f32 / 20.0,
                            y, bounds.min.z + (bounds.max.z - bounds.min.z) * z as f32 / 20.0);
                        assert!(std::iter::once(&known).chain(exterior.iter())
                            .any(|part| point.cmpge(part.min).all() && point.cmple(part.max).all()),
                            "coverage lost a wall or caster at {point:?}");
                    }
                }
            }
            for part in exterior {
                assert!(part.max.x <= authentic.min.x || part.min.x >= authentic.max.x
                    || part.max.z <= authentic.min.y || part.min.z >= authentic.max.y);
                assert_eq!(part.min.y, bounds.min.y);
                assert_eq!(part.max.y, bounds.max.y);
            }
        }
    }

    #[test]
    fn static_border_batch_reuses_gpu_geometry_after_cpu_retirement_with_separate_border_proof() {
        use bevy::mesh::Indices;
        use bevy::render::render_resource::PrimitiveTopology;
        let (mut app, leaf, source, _, _) = app_fixture();
        let location = app.world().resource::<BattleLocationFrame>().location.clone().unwrap();
        let (anchors, pose, authentic) = resolve_context(
            &location, app.world().resource::<TerrainRevisionCache>(),
        ).unwrap();
        let near = anchors[0] + Vec3::Y * 0.04;
        let outside = Vec3::new(authentic.max.x + 20.0, near.y, near.z);
        let positions: Vec<[f32; 3]> = [near, outside].into_iter().flat_map(|center|
            [Vec3::new(-0.3, 0.0, -0.3), Vec3::new(0.3, 0.0, -0.3), Vec3::new(0.0, 0.0, 0.3)]
                .map(|delta| pose.to_matrix().inverse().transform_point3(center + delta).to_array())
        ).collect();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 6]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.17, 0.29, 0.43, 1.0]; 6]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.13, 0.91]; 6]);
        mesh.insert_indices(Indices::U32((0..6).collect()));
        let expected = mesh_bytes(&mesh);
        let coverage = crate::terrain_batches::triangle_coverage(&mesh).unwrap();
        app.world_mut().entity_mut(leaf).insert((mesh.get_aabb().unwrap(),
            crate::terrain_batches::TerrainBatchCoverage { mesh: source.id(), sight_bounds: coverage.clone(), grass_bounds: Default::default(), bounds: coverage }));
        app.world_mut().resource_mut::<Assets<Mesh>>().insert(source.id(), mesh);
        let retired_cpu = app.world_mut().resource_mut::<Assets<Mesh>>().remove_untracked(source.id()).unwrap();
        prepare(app.world_mut());
        let frozen = app.world().resource::<EncounterTerrain>().frozen.as_ref().unwrap();
        assert!(frozen.meshes.is_empty(), "reuse the actual GPU handle, without a second mesh");
        assert!(app.world().get::<Children>(frozen.root).unwrap().iter().any(|child|
            app.world().get::<Mesh3d>(child).is_some_and(|mesh| mesh.0 == source)));
        assert!(app.world().resource::<Assets<Mesh>>().get(&source).is_none());
        assert_eq!(mesh_bytes(&retired_cpu), expected);
        assert_eq!(frozen.omitted.len(), 1);
        assert!(frozen.omitted[0].min.x > authentic.max.x);
        // A handle change cannot borrow an old batch's favorable coverage.
        app.world_mut().get_mut::<crate::terrain_batches::TerrainBatchCoverage>(leaf).unwrap().mesh
            = Handle::<Mesh>::default().id();
        assert_eq!(freeze(app.world_mut(), &location).err(),
            Some("no complete authentic terrain leaves are retained"));
    }

    #[test]
    fn mutable_flower_copy_is_byte_exact_and_survives_world_replacement_without_growth() {
        use bevy::asset::RenderAssetUsages;
        let (mut app, _, static_mesh, material, _) = app_fixture();
        let source = animated_fixture(&mut app, material, 20.0);
        let baseline = app.world().resource::<Assets<Mesh>>().len();
        let assets = app.world().resource::<Assets<Mesh>>();
        let original = assets.get(&source).unwrap();
        assert!(original.asset_usage.contains(RenderAssetUsages::MAIN_WORLD));
        assert!(
            original
                .asset_usage
                .contains(RenderAssetUsages::RENDER_WORLD)
        );
        let expected = mesh_bytes(original);
        let topology = original.primitive_topology();
        prepare(app.world_mut());
        let state = app.world().resource::<EncounterTerrain>();
        let frozen = state.frozen.as_ref().unwrap();
        assert!(frozen.omitted.is_empty());
        assert_eq!(frozen.meshes.len(), 1);
        let copy = frozen.meshes[0].clone();
        let root = frozen.root;
        assert_ne!(copy, source);
        let assets = app.world().resource::<Assets<Mesh>>();
        let frozen_mesh = assets.get(&copy).unwrap();
        assert_eq!(mesh_bytes(frozen_mesh), expected);
        assert_eq!(frozen_mesh.primitive_topology(), topology);
        assert_eq!(frozen_mesh.asset_usage, RenderAssetUsages::RENDER_WORLD);
        assert_eq!(assets.len(), baseline + 1);
        assert!(
            app.world()
                .get::<Children>(root)
                .unwrap()
                .iter()
                .any(|child| app.world().get::<Mesh3d>(child).map(|mesh| &mesh.0) == Some(&static_mesh))
        );
        app.world_mut().resource_mut::<Assets<Mesh>>().insert(
            source.id(),
            crate::retain_animated_mesh_cpu(Mesh::from(Cuboid::new(80.0, 40.0, 80.0))),
        );
        for _ in 0..30 {
            prepare(app.world_mut());
            app.update();
        }
        let assets = app.world().resource::<Assets<Mesh>>();
        assert_eq!(assets.len(), baseline + 1);
        assert_eq!(mesh_bytes(assets.get(&copy).unwrap()), expected);
        assert_ne!(mesh_bytes(assets.get(&source).unwrap()), expected);
        // Temporary source fallback retains the same exact private mesh.
        app.world_mut()
            .resource_mut::<VisualBattleFrame>()
            .use_source_scene = true;
        prepare(app.world_mut());
        app.update();
        assert!(app.world().resource::<Assets<Mesh>>().get(&copy).is_some());
        app.world_mut()
            .resource_mut::<BattleLocationFrame>()
            .location = None;
        prepare(app.world_mut());
        let assets = app.world().resource::<Assets<Mesh>>();
        assert_eq!(assets.len(), baseline);
        assert!(assets.get(&copy).is_none());
        assert!(assets.get(&source).is_some());
        assert!(assets.get(&static_mesh).is_some());
    }

    #[test]
    fn mutable_flower_border_crossing_stays_omitted_and_missing_cpu_data_stays_ineligible() {
        let (mut app, _, _, material, _) = app_fixture();
        animated_fixture(&mut app, material, 4096.0);
        prepare(app.world_mut());
        let frozen = app
            .world()
            .resource::<EncounterTerrain>()
            .frozen
            .as_ref()
            .unwrap();
        assert!(frozen.meshes.is_empty());
        assert!(!frozen.omitted.is_empty());
        assert!(frozen.omitted.iter().any(|&bounds| visible_bounds(
            bounds,
            BattleSceneLayout::default().camera,
            Vec2::new(16.0, 9.0),
            100.0
        )));

        let (mut app, _, _, material, _) = app_fixture();
        let source = animated_fixture(&mut app, material, 20.0);
        app.world_mut()
            .resource_mut::<Assets<Mesh>>()
            .remove(source.id());
        let before = app.world().resource::<Assets<Mesh>>().len();
        prepare(app.world_mut());
        let state = app.world().resource::<EncounterTerrain>();
        assert!(state.frozen.is_none());
        assert_eq!(
            state.reason(),
            Some("actual animated terrain CPU mesh is unavailable")
        );
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), before);
    }

    #[test]
    fn animated_partition_preserves_original_triangle_order_and_attribute_bytes() {
        use bevy::{
            asset::RenderAssetUsages,
            mesh::Indices,
            render::render_resource::PrimitiveTopology,
        };
        for short_indices in [false, true] {
            let mut source = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            let positions = vec![
                [-12.0, 0.0, 0.0],
                [-11.0, 1.0, 0.0],
                [-12.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [12.0, 0.0, 0.0],
                [13.0, 1.0, 0.0],
                [12.0, 0.0, 1.0],
                [3.0, 0.0, 0.0],
                [5.0, 1.0, 0.0],
                [3.0, 0.0, 1.0],
            ];
            let count = positions.len();
            source.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            source.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count]);
            source.insert_attribute(
                Mesh::ATTRIBUTE_UV_0,
                (0..count)
                    .map(|i| [i as f32 * 0.01, -0.0])
                    .collect::<Vec<_>>(),
            );
            source.insert_attribute(
                Mesh::ATTRIBUTE_UV_1,
                (0..count).map(|i| [i as f32, 0.7]).collect::<Vec<_>>(),
            );
            source.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                (0..count)
                    .map(|i| [0.13, i as f32 * 0.031, 0.9, 1.0])
                    .collect::<Vec<_>>(),
            );
            let order = vec![6, 7, 8, 0, 1, 2, 3, 4, 5, 9, 10, 11];
            source.insert_indices(if short_indices {
                Indices::U16(order.iter().map(|&i| i as u16).collect())
            } else {
                Indices::U32(order)
            });
            let authentic = AuthenticBounds {
                min: Vec2::splat(-4.0),
                max: Vec2::splat(4.0),
            };
            let (selected, omitted) =
                select_authentic_triangles(&source, Transform::IDENTITY, authentic).unwrap();
            let (copy, aabb, bounds) = selected.unwrap();
            assert_eq!(
                copy.indices().unwrap().iter().collect::<Vec<_>>(),
                vec![3, 4, 5]
            );
            assert_eq!(
                matches!(copy.indices(), Some(Indices::U16(_))),
                short_indices
            );
            for ((source_id, source_values), (copy_id, copy_values)) in
                source.attributes().zip(copy.attributes())
            {
                assert_eq!(source_id, copy_id);
                assert_eq!(source_values.get_bytes(), copy_values.get_bytes());
            }
            assert_eq!(Vec3::from(aabb.center - aabb.half_extents), Vec3::ZERO);
            assert_eq!(Vec3::from(aabb.center + aabb.half_extents), Vec3::ONE);
            assert!(authentic.contains(bounds));
            assert_eq!(omitted.len(), 3);
            assert!(
                omitted.iter().any(|b| b.min.x == 3.0 && b.max.x == 5.0),
                "straddling source triangles remain covered by the omission proof"
            );
            assert!(
                omitted.iter().all(|b| b.max.x - b.min.x <= 2.0),
                "opposite outside flowers cannot form one AABB across the corridor"
            );
        }
    }

    #[test]
    fn animated_partition_uses_real_transformed_coordinates_and_rejects_invalid_inputs() {
        use bevy::{
            asset::RenderAssetUsages,
            mesh::Indices,
            render::render_resource::PrimitiveTopology,
        };
        let source = Mesh::from(Cuboid::new(32.0, 16.0, 32.0));
        let pose = Transform::from_translation(Vec3::new(20.0, 3.0, -10.0))
            .with_scale(Vec3::splat(1.0 / 16.0));
        let authentic = AuthenticBounds {
            min: Vec2::new(18.0, -12.0),
            max: Vec2::new(22.0, -8.0),
        };
        let (selected, omitted) = select_authentic_triangles(&source, pose, authentic).unwrap();
        let (copy, _, bounds) = selected.unwrap();
        assert_eq!(mesh_bytes(&copy), mesh_bytes(&source));
        assert!(omitted.is_empty());
        assert_eq!(bounds.min, Vec3::new(19.0, 2.5, -11.0));
        assert_eq!(bounds.max, Vec3::new(21.0, 3.5, -9.0));
        let mut invalid = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        invalid.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0]; 3]);
        invalid.insert_indices(Indices::U32(vec![0, 1, 7]));
        assert!(select_authentic_triangles(&invalid, pose, authentic).is_err());
        invalid.insert_indices(Indices::U32(vec![0, 1]));
        assert!(select_authentic_triangles(&invalid, pose, authentic).is_err());
    }
}
