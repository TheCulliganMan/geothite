//! Whole authored occluders fade without cutting holes or rebuilding meshes.
//! Source ranges own fade state. A triangle BVH tests real wall surfaces, never
//! the empty interior of a concave object's bounding box. Render leaves merely
//! improve transparent sorting; every leaf of an object has identical opacity.
#[cfg(feature = "terrain-worker")]
#[path = "terrain_worker/occluder_wire.rs"]
pub(super) mod worker_wire;
use crate::{
    TerrainRevisionCache, VOXEL_RENDER_LAYER, VoxelMaterial, VoxelViewStatus, VoxelWorldCamera,
    interior_cutaway::CutawayUniform, mesh::SurfaceMeshData,
};
use bevy::{
    prelude::*,
    camera::{primitives::Aabb, visibility::RenderLayers},
};
use std::ops::Range;
use std::sync::{Arc, Mutex, OnceLock, Weak};

static LEGACY_TREE_VERTEX_WELD: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
/// Startup-only same-binary QA control. Legacy keeps every original merged
/// palette vertex; both modes retain the exact same ordered triangle attributes.
pub fn configure_tree_vertex_weld_diagnostics(legacy: bool) {
    LEGACY_TREE_VERTEX_WELD.store(legacy, std::sync::atomic::Ordering::Relaxed);
}


const RESIDUAL_OPACITY: f32 = 0.20;
// Common screen coverage keeps layered scenery faint without accumulating
// opacity over the player. Preserve enough form to locate the obstruction.
const BUILDING_REVEAL_OPACITY: f32 = 0.18;
const FOLIAGE_REVEAL_OPACITY: f32 = 0.24;
const EXIT_HOLD_SECONDS: f32 = 0.12;
const FADE_RATE: f32 = 14.0;
const RESTORE_RATE: f32 = 8.0;
// A group ordinal can never reach usize::MAX in an allocated vertex table.
// Avoid Option<usize>'s extra tag for every vertex in a large scene.
const UNOWNED: usize = usize::MAX;
const UNOWNED_VERTEX: u32 = u32::MAX;
// Larger leaves reduce build passes and node storage. Every leaf still tests
// all its original triangles exactly; no collision or render detail is removed.
const BVH_LEAF_TRIANGLES: usize = 64;
const MAX_RENDER_LEAVES: usize = 16;
const RENDER_LEAF_TRIANGLES: usize = 2048;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct FadeMeshes;

pub(super) fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        sync.in_set(FadeMeshes).after(bevy::camera::CameraUpdateSystems)
            .after(bevy::transform::TransformSystems::Propagate)
            .before(bevy::asset::AssetEventSystems)
            .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
    );
}

#[derive(Clone)]
pub(super) struct PreparedGroup {
    source_range: Range<usize>,
    surface: Arc<CollisionSurface>,
    leaves: Vec<PreparedLeaf>,
    opaque_leaf: Option<PreparedLeaf>,
    overlay: Option<Box<PreparedGroup>>,
    transform: Transform,
    coverage_fade: bool,
    // Depth is independent of coverage: closed layered alpha can write depth
    // without the pixel mask used by source cabinets.
    depth_coverage_fade: bool,
    residual_opacity: f32,
    camera_wall: Option<[f32; 3]>,
    runtime_lazy_sorted: bool,
}
/// Layered palette foliage shares one visibility sample across all needles.
/// Keep ordinary walls and their existing translucent reveal unchanged.
pub(super) fn use_coverage_fade(groups: &mut [PreparedGroup]) {
    set_coverage_fade(groups, true);
}
pub(super) fn set_coverage_fade(groups: &mut [PreparedGroup], coverage: bool) {
    for group in groups {
        group.coverage_fade = coverage;
        if let Some(overlay) = &mut group.overlay {
            set_coverage_fade(std::slice::from_mut(overlay.as_mut()), coverage);
        }
    }
}
/// Only exact authored opt-ins change reveal style; neighboring walls keep
/// their existing blend. Linked captions inherit the owning object's sample.
pub(super) fn use_source_coverage_fade(groups: &mut [PreparedGroup], ranges: &[Range<usize>], residual: f32) {
    fn set_residual(group: &mut PreparedGroup, residual: f32) {
        group.residual_opacity = residual;
        if let Some(overlay) = &mut group.overlay { set_residual(overlay, residual); }
    }
    for group in groups {
        if ranges.contains(&group.source_range) {
            set_coverage_fade(std::slice::from_mut(group), true);
            set_residual(group, residual);
        }
    }
}
fn set_residual(groups: &mut [PreparedGroup], opacity: f32) {
    for group in groups {
        group.residual_opacity=opacity;
        if let Some(overlay)=&mut group.overlay {
            set_residual(std::slice::from_mut(overlay.as_mut()),opacity);
        }
    }
}
/// Outdoor buildings retain a faint complete form while the player is behind
/// them. Room camera walls keep their separate fully open cutaway policy.
pub(super) fn use_building_reveal(groups: &mut [PreparedGroup], ranges: &[Range<usize>]) {
    // Retain a smooth silhouette instead of speckling the street with the
    // building's opaque source pigments. Depth rejection keeps hidden layers
    // ordered; only buildings already opted into depth coverage receive it.
    // In particular the forest gate's existing coverage policy stays intact.
    use_source_coverage_fade(groups,ranges,BUILDING_REVEAL_OPACITY);
    for group in groups {
        if group.depth_coverage_fade && ranges.contains(&group.source_range) {
            set_coverage_fade(std::slice::from_mut(group),false);
        }
    }
}
/// A slightly stronger crown remains legible against similarly colored ground.
/// This changes presentation only; all source leaves and shadows stay intact.
pub(super) fn use_foliage_reveal(groups: &mut [PreparedGroup], coverage: bool) {
    set_coverage_fade(groups,coverage);
    set_residual(groups,FOLIAGE_REVEAL_OPACITY);
}

/// Closed source cabinets retain depth ordering while a common pixel mask
/// reveals the player. Opt in by exact owned range, preserving other fades.
pub(super) fn use_source_depth_coverage_fade(groups: &mut [PreparedGroup], ranges: &[Range<usize>]) {
    fn enable(group: &mut PreparedGroup) {
        group.coverage_fade=true;
        group.depth_coverage_fade=true;
        if let Some(overlay)=&mut group.overlay {enable(overlay);}
    }
    for group in groups {
        if ranges.contains(&group.source_range) {enable(group);}
    }
}

/// Camera-facing room cutaway is independent of the player's occlusion rays.
/// Every fragment of the near wall disappears, including its cap and back.
pub(super) fn use_camera_walls(groups:&mut [PreparedGroup],planes:&[(Range<usize>,[f32;3])]) {
    for group in groups {
        if let Some((_,plane))=planes.iter().find(|(range,_)|*range==group.source_range) {
            group.camera_wall=Some(*plane);
            group.coverage_fade=true;
            group.residual_opacity=0.;
        }
    }
}

fn outside_room_wall(plane:[f32;3],eye:Vec3)->bool {
    plane[0]*eye.x+plane[1]*eye.z<plane[2]-0.01
}

#[cfg(test)]
mod room_camera_tests {
    use super::*;
    #[test]
    fn near_walls_hide_as_the_camera_orbits_and_inside_cameras_keep_walls() {
        let west=[1.,0.,0.];let east=[-1.,0.,-128.];let front=[0.,-1.,-96.];
        let inside=Vec3::new(64.,24.,48.);
        for plane in [west,east,front] {assert!(!outside_room_wall(plane,inside));}
        let south_east=Vec3::new(180.,180.,180.);
        assert!(outside_room_wall(front,south_east));
        assert!(outside_room_wall(east,south_east));
        assert!(!outside_room_wall(west,south_east));
        let north_west=Vec3::new(-80.,180.,-80.);
        assert!(outside_room_wall(west,north_west));
        assert!(!outside_room_wall(front,north_west));
        assert!(!outside_room_wall(east,north_west));
        // Camera height does not alter which side of the wall it occupies.
        assert_eq!(outside_room_wall(front,south_east),outside_room_wall(front,south_east.with_y(2.)));
    }
}
/// Exact source bounds, available before any mesh is uploaded.
pub(super) fn prepared_bounds(groups: &[PreparedGroup]) -> Option<(Vec3, Vec3)> {
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    fn include(group: &PreparedGroup, low: &mut Vec3, high: &mut Vec3) {
        if let Some(bounds) = group.surface.bounds {
            for i in 0..8 {
                let p = Vec3::new(
                    if i&1 == 0 { bounds.min.x } else { bounds.max.x },
                    if i&2 == 0 { bounds.min.y } else { bounds.max.y },
                    if i&4 == 0 { bounds.min.z } else { bounds.max.z });
                let p = group.transform.transform_point(p);
                *low = low.min(p); *high = high.max(p);
            }
        }
        if let Some(overlay) = &group.overlay { include(overlay, low, high); }
    }
    for group in groups { include(group, &mut low, &mut high); }
    (low.is_finite() && high.is_finite()).then_some((low, high))
}
#[derive(Clone)]
struct PreparedLeaf {
    mesh: Mesh,
    uploaded: Option<Handle<Mesh>>,
    center: Vec3,
    bounds: Aabb,
    shared: Option<Arc<Mutex<SharedLeaf>>>,
}
struct SharedLeaf { mesh: Option<Mesh>, uploaded: Option<Handle<Mesh>> }
// Instances own their shared geometry. The recipe lookup cache holds only
// weak references, so leaving a region releases its CPU/GPU resources.
#[derive(Component)]
struct SharedGeometry(#[allow(dead_code)] Arc<Mutex<SharedLeaf>>);
// Sorted geometry is needed only while its object fades. Retain the exact
// prepared mesh, but avoid uploading a second copy for every opaque object.
#[derive(Component)]
struct DeferredMesh(Option<PreparedLeaf>);
/// Runtime trees retain exact sorted geometry without dormant transform children.
#[derive(Component)]
struct LazySortedLeaves(Vec<PreparedLeaf>);
pub(crate) fn use_lazy_sorted_entities(groups: &mut [PreparedGroup]) {
    for group in groups { group.runtime_lazy_sorted = true; }
}
pub(super) struct WeakPreparedGroup {
    source_range: Range<usize>,
    surface: Weak<CollisionSurface>,
    leaves: Vec<(Vec3, Aabb, Weak<Mutex<SharedLeaf>>)>,
    opaque_leaf: Option<(Vec3, Aabb, Weak<Mutex<SharedLeaf>>)>,
    overlay: Option<Box<WeakPreparedGroup>>,
    transform: Transform,
    coverage_fade: bool,
    depth_coverage_fade: bool,
    residual_opacity: f32,
    camera_wall: Option<[f32; 3]>,
    runtime_lazy_sorted: bool,
}
impl WeakPreparedGroup {
    pub(super) fn new(group: &PreparedGroup) -> Self {
        Self {
            source_range: group.source_range.clone(),
            surface: Arc::downgrade(&group.surface),
            leaves: group.leaves.iter().map(|leaf| (leaf.center, leaf.bounds,
                Arc::downgrade(leaf.shared.as_ref().expect("cached geometry is shared")))).collect(),
            opaque_leaf: group.opaque_leaf.as_ref().map(|leaf| (leaf.center, leaf.bounds,
                Arc::downgrade(leaf.shared.as_ref().expect("cached opaque geometry is shared")))),
            overlay: group.overlay.as_ref().map(|group| Box::new(Self::new(group))),
            transform: group.transform,
            coverage_fade: group.coverage_fade,
            depth_coverage_fade: group.depth_coverage_fade,
            residual_opacity: group.residual_opacity,
            camera_wall: group.camera_wall,
            runtime_lazy_sorted: group.runtime_lazy_sorted,
        }
    }
    pub(super) fn upgrade(&self) -> Option<PreparedGroup> {
        Some(PreparedGroup {
            source_range: self.source_range.clone(),
            coverage_fade: self.coverage_fade,
            depth_coverage_fade: self.depth_coverage_fade,
            residual_opacity: self.residual_opacity,
            camera_wall: self.camera_wall,
            runtime_lazy_sorted: self.runtime_lazy_sorted,
            surface: self.surface.upgrade()?,
            leaves: self.leaves.iter().map(|(center, bounds, shared)| Some(PreparedLeaf {
                mesh: empty_mesh(), uploaded: None, center: *center, bounds: *bounds,
                shared: Some(shared.upgrade()?),
            })).collect::<Option<Vec<_>>>()?,
            opaque_leaf: match &self.opaque_leaf {
                Some((center, bounds, shared)) => Some(PreparedLeaf {
                    mesh: empty_mesh(), uploaded: None, center: *center, bounds: *bounds,
                    shared: Some(shared.upgrade()?),
                }),
                None => None,
            },
            overlay: match &self.overlay {
                Some(group) => Some(Box::new(group.upgrade()?)),
                None => None,
            },
            transform: self.transform,
        })
    }
}
fn empty_mesh() -> Mesh {
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::RENDER_WORLD)
}
fn upload_leaf(leaf: &mut PreparedLeaf, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    if let Some(shared) = &leaf.shared {
        let mut shared = shared.lock().unwrap();
        let _trace = crate::loading_trace::span("tree.upload_or_reuse", || serde_json::json!({
            "shared": true, "cache_hit": shared.uploaded.is_some(),
            "vertices": shared.mesh.as_ref().map_or(0, Mesh::count_vertices)}));
        if let Some(handle) = &shared.uploaded { return handle.clone(); }
        let handle = meshes.add(shared.mesh.take().expect("shared mesh uploads once"));
        shared.uploaded = Some(handle.clone());
        return handle;
    }
    let _trace = crate::loading_trace::span("tree.upload_or_reuse", || serde_json::json!({
        "shared": false, "cache_hit": leaf.uploaded.is_some(), "vertices": leaf.mesh.count_vertices()}));
    leaf.uploaded.get_or_insert_with(|| meshes.add(std::mem::replace(&mut leaf.mesh, empty_mesh()))).clone()
}
/// Prepare the exact sorted alternatives for a frozen view without exposing
/// them in the overworld. Shared prototype uploads remain shared and cached.
pub(super) fn prepare_retention_meshes(world: &mut World, root: Entity) {
    let mut pending = vec![root];
    while let Some(entity) = pending.pop() {
        if let Some(leaves) = world.entity_mut(entity).take::<LazySortedLeaves>() {
            let material = world.get::<OccluderGroup>(entity).unwrap().opaque.clone();
            let global = world.get::<GlobalTransform>(entity).copied().unwrap_or_default();
            for prepared in leaves.0 {
                let local = Transform::from_translation(prepared.center);
                let child = world.spawn((MeshMaterial3d(material.clone()), local,
                    global.mul_transform(local), Visibility::Hidden, prepared.bounds,
                    RenderLayers::layer(VOXEL_RENDER_LAYER), SortedObjectLeaf,
                    DeferredMesh(Some(prepared)))).id();
                world.entity_mut(entity).add_child(child);
            }
        }
        if let Some(children) = world.get::<Children>(entity) {
            pending.extend(children.iter());
        }
        let prepared = world.get_mut::<DeferredMesh>(entity).and_then(|mut deferred| deferred.0.take());
        if let Some(mut prepared) = prepared {
            upload_leaf(&mut prepared, &mut world.resource_mut::<Assets<Mesh>>());
            if let Some(mut deferred) = world.get_mut::<DeferredMesh>(entity) {
                deferred.0 = Some(prepared);
            }
        }
    }
}

pub(super) fn retained_mesh(world: &World, entity: Entity) -> Option<Handle<Mesh>> {
    if let Some(mesh) = world.get::<Mesh3d>(entity) { return Some(mesh.0.clone()); }
    let prepared = world.get::<DeferredMesh>(entity)?.0.as_ref()?;
    prepared.uploaded.clone().or_else(|| prepared.shared.as_ref()
        .and_then(|shared| shared.lock().unwrap().uploaded.clone()))
}

/// Convert a canonical object to shared immutable geometry. Instance state and
/// transforms remain independent; Bevy can batch equal mesh/material handles.
pub(crate) fn share_groups(groups: &mut [PreparedGroup]) {
    for group in groups {
        for leaf in group.leaves.iter_mut().chain(group.opaque_leaf.iter_mut()) {
            if leaf.shared.is_none() {
                leaf.shared = Some(Arc::new(Mutex::new(SharedLeaf {
                    mesh: Some(std::mem::replace(&mut leaf.mesh, empty_mesh())), uploaded: leaf.uploaded.take(),
                })));
            }
        }
        if let Some(overlay) = &mut group.overlay {
            share_groups(std::slice::from_mut(overlay));
        }
    }
}
pub(crate) fn place_groups(groups: &mut [PreparedGroup], transform: Transform) {
    for group in groups { group.transform = transform; }
}

/// Stage complete leaves for render extraction before the remaining groves build.
/// Bounds, collision triangles and fade ownership remain with the source group.
pub(crate) fn stage_group_meshes(groups: &mut [PreparedGroup], meshes: &mut Assets<Mesh>) {
    for group in groups {
        for leaf in group.leaves.iter_mut().chain(group.opaque_leaf.iter_mut()) {
            upload_leaf(leaf, meshes);
        }
        if let Some(overlay) = &mut group.overlay {
            stage_group_meshes(std::slice::from_mut(overlay), meshes);
        }
    }
}

/// The caller must prove the owning material does not consume constant UV0.
/// Linked textured overlays retain their own coordinates and material contract.
pub(super) fn compact_palette_group_uv0(groups: &mut [PreparedGroup]) {
    for group in groups {
        for leaf in group.leaves.iter_mut().chain(group.opaque_leaf.iter_mut()) {
            crate::compact_vertices::remove_unused_zero_uv0(&mut leaf.mesh);
        }
        // Only the shared tree's merged opaque GPU draw is welded. The source
        // collision surface and distance-sorted fading leaves keep their exact
        // original vertex/index ownership and all triangles.
        if !LEGACY_TREE_VERTEX_WELD.load(std::sync::atomic::Ordering::Relaxed) {
            if let Some(opaque) = group.opaque_leaf.as_mut() {
                weld_opaque_palette_tree_vertices(&mut opaque.mesh);
            }
        }
    }
}

/// Share only fully identical palette vertices, retaining each triangle's
/// original order, winding and attribute bits. No float arithmetic or epsilon
/// comparisons are involved; signed zero and distinct NaN payloads stay apart.
/// Unsupported attributes and unavailable/malformed CPU buffers remain intact.
fn weld_opaque_palette_tree_vertices(mesh: &mut Mesh) -> usize {
    use bevy::mesh::{Indices, VertexAttributeValues};
    if mesh.primitive_topology() != bevy::mesh::PrimitiveTopology::TriangleList {
        return 0;
    }
    let Ok(attributes) = mesh.try_attributes() else { return 0; };
    if attributes.into_iter().any(|(attribute, _)|
        ![Mesh::ATTRIBUTE_POSITION.id, Mesh::ATTRIBUTE_NORMAL.id, Mesh::ATTRIBUTE_COLOR.id]
            .contains(&attribute.id)) {
        return 0;
    }
    let (firsts, remap, original_count) = {
        let Ok(VertexAttributeValues::Float32x3(positions)) = mesh.try_attribute(Mesh::ATTRIBUTE_POSITION) else { return 0; };
        let Ok(VertexAttributeValues::Float32x3(normals)) = mesh.try_attribute(Mesh::ATTRIBUTE_NORMAL) else { return 0; };
        let Ok(colors) = mesh.try_attribute(Mesh::ATTRIBUTE_COLOR) else { return 0; };
        let color_len = match colors {
            VertexAttributeValues::Float32x3(values) => values.len(),
            VertexAttributeValues::Float32x4(values) => values.len(),
            _ => return 0,
        };
        let count = positions.len();
        let Ok(indices) = mesh.try_indices() else { return 0; };
        if normals.len() != count || color_len != count || count > u32::MAX as usize
            || indices.len() % 3 != 0 || indices.iter().any(|index| index >= count) {
            return 0;
        }
        // Fixed-size keys avoid a heap allocation for each source vertex.
        // Foldhash is the engine's ordinary fast lookup hasher; equality still
        // compares every raw attribute bit, so hash collisions cannot merge art.
        let mut seen = bevy::platform::collections::HashMap::<[u32; 10], u32>::with_capacity(count);
        let mut firsts = Vec::with_capacity(count);
        let mut remap = Vec::with_capacity(count);
        for i in 0..count {
            let mut key = [0u32; 10];
            key[..3].copy_from_slice(&positions[i].map(f32::to_bits));
            key[3..6].copy_from_slice(&normals[i].map(f32::to_bits));
            match colors {
                VertexAttributeValues::Float32x3(values) => key[6..9].copy_from_slice(&values[i].map(f32::to_bits)),
                VertexAttributeValues::Float32x4(values) => key[6..10].copy_from_slice(&values[i].map(f32::to_bits)),
                _ => unreachable!(),
            }
            let next = firsts.len() as u32;
            let index = *seen.entry(key).or_insert_with(|| { firsts.push(i); next });
            remap.push(index);
        }
        (firsts, remap, count)
    };
    if firsts.len() == original_count { return 0; }
    // Stable first occurrences are increasing, so compaction can happen in
    // place without allocating a second copy of any vertex attribute buffer.
    fn compact<T: Copy>(values: &mut Vec<T>, firsts: &[usize]) {
        for (next, &old) in firsts.iter().enumerate() { values[next] = values[old]; }
        values.truncate(firsts.len());
    }
    for attribute in [Mesh::ATTRIBUTE_POSITION, Mesh::ATTRIBUTE_NORMAL, Mesh::ATTRIBUTE_COLOR] {
        match mesh.attribute_mut(attribute).unwrap() {
            VertexAttributeValues::Float32x3(values) => compact(values, &firsts),
            VertexAttributeValues::Float32x4(values) => compact(values, &firsts),
            _ => unreachable!(),
        }
    }
    let indices = mesh.remove_indices().unwrap();
    let indices = if indices.iter().all(|old| remap[old] <= u32::from(u16::MAX)) {
        Indices::U16(indices.iter().map(|old| remap[old] as u16).collect())
    } else {
        Indices::U32(indices.iter().map(|old| remap[old]).collect())
    };
    mesh.insert_indices(indices);
    #[cfg(target_arch = "wasm32")]
    crate::browser_memory::align_indices(mesh);
    original_count - firsts.len()
}

#[derive(Clone, Copy, Debug)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}
impl Bounds {
    fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
    fn include(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }
    fn center(self) -> Vec3 {
        self.min * 0.5 + self.max * 0.5
    }
    fn segment(self, start: Vec3, delta: Vec3) -> bool {
        let mut near: f32 = 0.0;
        let mut far: f32 = 0.999;
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
}
#[derive(Clone, Debug)]
struct Node {
    bounds: Bounds,
    triangles: Range<usize>,
    children: [usize; 2],
}
struct CollisionSurface {
    bounds: Option<Bounds>,
    tree: OnceLock<TriangleTree>,
    source: Mutex<Option<SurfaceMeshData>>,
}
impl CollisionSurface {
    fn ready(tree: TriangleTree) -> Self {
        Self { bounds: tree.nodes.first().map(|node| node.bounds),
            tree: OnceLock::from(tree), source: Mutex::new(None) }
    }
    fn deferred(data: &SurfaceMeshData) -> Self {
        let mut bounds = Bounds::empty();
        for &i in &data.indices { bounds.include(data.positions[i as usize].into()); }
        // Collision only consumes positions and indices. Retain those exact
        // buffers, not duplicate painting/normal data, until a real ray needs
        // the tree. Both buffers are released immediately after construction.
        let source = SurfaceMeshData { positions: data.positions.clone(),
            indices: data.indices.clone(), ..Default::default() };
        Self { bounds: (!data.indices.is_empty()).then_some(bounds),
            tree: OnceLock::new(), source: Mutex::new(Some(source)) }
    }
    fn blocks(&self, start: Vec3, end: Vec3) -> bool {
        if !self.bounds.is_some_and(|bounds| bounds.segment(start, end - start)) { return false; }
        self.tree.get_or_init(|| {
            let source = self.source.lock().unwrap().take().expect("retained collision source");
            TriangleTree::new(&source)
        }).blocks(start, end)
    }
}

#[derive(Clone, Debug, Default)]
struct TriangleTree {
    // Keep indexed source positions rather than storing three copied positions
    // for every triangle. This is exact collision geometry, not a proxy mesh.
    positions: Vec<Vec3>,
    triangles: TriangleIndices,
    nodes: Vec<Node>,
}
struct BuildTriangle {
    bounds: Bounds,
    center: [f64; 3],
}
// Collision ownership needs the original corners, but each complete grove
// fits in 16-bit local indices. Keep large source objects lossless too.
#[derive(Clone, Debug)]
enum TriangleIndices {
    Packed { triangles: Vec<[u16; 2]>, exceptions: Vec<[u16; 3]> },
    Narrow(Vec<[u16; 3]>),
    Wide(Vec<[u32; 3]>),
}
impl Default for TriangleIndices {
    fn default() -> Self { Self::Narrow(Vec::new()) }
}
impl TriangleIndices {
    #[cfg(test)]
    fn new(indices: &[u32]) -> Self {
        let mut result = Self::for_build(indices);
        result.compact();
        result
    }
    fn for_build(indices: &[u32]) -> Self {
        if indices.iter().all(|&i| i <= u16::MAX as u32) {
            let narrow: Vec<[u16; 3]> = indices.chunks_exact(3)
                .map(|t| [t[0] as u16, t[1] as u16, t[2] as u16]).collect();
            Self::Narrow(narrow)
        } else {
            Self::Wide(indices.chunks_exact(3).map(|t| t.try_into().unwrap()).collect())
        }
    }
    fn compact(&mut self) {
        if let Self::Narrow(source) = self {
            if let Some(packed) = Self::pack(source) { *self = packed; }
        }
    }
    fn pack(source: &[[u16; 3]]) -> Option<Self> {
        let mut triangles = Vec::with_capacity(source.len());
        let mut exceptions = Vec::new();
        for &[first, second, third] in source {
            let offsets = second.checked_sub(first).zip(third.checked_sub(first));
            if first != u16::MAX && offsets.is_some_and(|(a, b)| a <= 255 && b <= 255) {
                let (a, b) = offsets.unwrap();
                triangles.push([first, a | (b << 8)]);
            } else {
                let index = u16::try_from(exceptions.len()).ok()?;
                triangles.push([u16::MAX, index]);
                exceptions.push([first, second, third]);
            }
        }
        if triangles.len() * 4 + exceptions.len() * 6 >= source.len() * 6 {
            return None;
        }
        exceptions.shrink_to_fit();
        Some(Self::Packed { triangles, exceptions })
    }
    fn unpack(triangle: [u16; 2], exceptions: &[[u16; 3]]) -> [u32; 3] {
        let [first, offsets] = triangle;
        if first == u16::MAX {
            exceptions[usize::from(offsets)].map(u32::from)
        } else {
            [u32::from(first), u32::from(first) + u32::from(offsets & 255),
                u32::from(first) + u32::from(offsets >> 8)]
        }
    }
    fn len(&self) -> usize {
        match self { Self::Packed { triangles, .. } => triangles.len(), Self::Narrow(v) => v.len(), Self::Wide(v) => v.len() }
    }
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn triangle(&self, index: usize) -> [u32; 3] {
        match self { Self::Packed { triangles, exceptions } => Self::unpack(triangles[index], exceptions),
            Self::Narrow(v) => v[index].map(u32::from), Self::Wide(v) => v[index] }
    }
    #[cfg(test)]
    fn iter(&self) -> impl Iterator<Item = [u32; 3]> + '_ {
        (0..self.len()).map(|index| self.triangle(index))
    }
    #[cfg(test)]
    fn storage_bytes(&self) -> usize {
        match self {
            Self::Packed { triangles, exceptions } => triangles.len() * 4 + exceptions.len() * 6,
            Self::Narrow(v) => v.len() * std::mem::size_of::<[u16; 3]>(),
            Self::Wide(v) => v.len() * std::mem::size_of::<[u32; 3]>(),
        }
    }
    fn partition(&mut self, range: Range<usize>, positions: &[Vec3], axis: usize) {
        fn partition<I: Copy + Into<u32>>(triangles: &mut [[I; 3]], positions: &[Vec3], axis: usize) {
            triangles.select_nth_unstable_by(triangles.len() / 2, |a, b| {
                let center = |t: &[I; 3]| {
                    f64::from(positions[t[0].into() as usize][axis])
                        + f64::from(positions[t[1].into() as usize][axis])
                        + f64::from(positions[t[2].into() as usize][axis])
                };
                center(a).total_cmp(&center(b))
            });
        }
        match self {
            Self::Packed { triangles, exceptions } => {
                let selected = &mut triangles[range];
                selected.select_nth_unstable_by(selected.len() / 2, |a, b| {
                    let center = |t: &[u16; 2]| Self::unpack(*t, exceptions).into_iter()
                        .map(|i| f64::from(positions[i as usize][axis])).sum::<f64>();
                    center(a).total_cmp(&center(b))
                });
            }
            Self::Narrow(v) => partition(&mut v[range], positions, axis),
            Self::Wide(v) => partition(&mut v[range], positions, axis),
        }
    }
}
impl TriangleTree {
    fn new(data: &SurfaceMeshData) -> Self {
        let _trace = crate::loading_trace::span("occluder.collision_bvh", || serde_json::json!({"vertices": data.positions.len(), "triangles": data.indices.len()/3}));
        // Face normals and colors require separate render vertices, but the
        // collision tree can share positions with identical float bits.
        // Keep signed zero and every original corner; never quantize geometry.
        // Geometry keys are exact float bits, not network identifiers. Use
        // Bevy's game-data hasher instead of hashing every corner with SipHash.
        // Indices follow source insertion order, independent of hash buckets.
        let mut unique = bevy::platform::collections::HashMap::<[u32; 3], u32>::new();
        let mut positions = Vec::new();
        let mut remap = Vec::with_capacity(data.positions.len());
        for &position in &data.positions {
            let index = *unique.entry(position.map(f32::to_bits)).or_insert_with(|| {
                let index = positions.len() as u32;
                positions.push(Vec3::from_array(position));
                index
            });
            remap.push(index);
        }
        let indices = data.indices.iter().map(|&i| remap[i as usize]).collect::<Vec<_>>();
        drop(unique);
        drop(remap);
        positions.shrink_to_fit();
        let mut tree = Self {
            positions,
            triangles: TriangleIndices::for_build(&indices),
            nodes: Vec::new(),
        };
        drop(indices);
        if !tree.triangles.is_empty() {
            tree.build_cached();
        }
        // Build on direct indices. Decoding compressed triples at every
        // partition comparison and bounds visit repeats work at each level.
        // Retain the same compact representation once construction finishes.
        tree.triangles.compact();
        tree
    }
    fn build_cached(&mut self) {
        // Every ancestor partition asks for the same triangle centroid and
        // corner bounds. Calculate them once, then partition small index IDs.
        // Scratch metadata is dropped before publishing the collision tree.
        let metadata = (0..self.triangles.len()).map(|index| {
            let points = self.triangles.triangle(index).map(|i| self.positions[i as usize]);
            let mut bounds = Bounds::empty();
            for point in points { bounds.include(point); }
            BuildTriangle { bounds, center:std::array::from_fn(|axis|
                f64::from(points[0][axis])+f64::from(points[1][axis])+f64::from(points[2][axis])) }
        }).collect::<Vec<_>>();
        let mut order = (0..self.triangles.len()).collect::<Vec<_>>();
        self.build_cached_node(&mut order, &metadata, 0);
        self.triangles = match &self.triangles {
            TriangleIndices::Narrow(source) => TriangleIndices::Narrow(order.iter().map(|&i|source[i]).collect()),
            TriangleIndices::Wide(source) => TriangleIndices::Wide(order.iter().map(|&i|source[i]).collect()),
            TriangleIndices::Packed { .. } => unreachable!("construction starts with direct indices"),
        };
    }
    fn build_cached_node(&mut self, order: &mut [usize], metadata: &[BuildTriangle], start: usize) -> usize {
        let mut bounds = Bounds::empty();
        for &index in order.iter() {
            bounds.include(metadata[index].bounds.min);
            bounds.include(metadata[index].bounds.max);
        }
        let node = self.nodes.len();
        self.nodes.push(Node { bounds, triangles:start..start+order.len(), children:[UNOWNED;2] });
        if order.len() > BVH_LEAF_TRIANGLES {
            let size = bounds.max-bounds.min;
            let axis = if size.x>=size.y && size.x>=size.z {0} else if size.y>=size.z {1} else {2};
            let middle = order.len()/2;
            order.select_nth_unstable_by(middle, |&a,&b| metadata[a].center[axis].total_cmp(&metadata[b].center[axis]));
            let (left,right) = order.split_at_mut(middle);
            let a = self.build_cached_node(left, metadata, start);
            let b = self.build_cached_node(right, metadata, start+middle);
            self.nodes[node].children = [a,b];
        }
        node
    }
    #[cfg(test)]
    fn build(&mut self, range: Range<usize>) -> usize {
        let mut bounds = Bounds::empty();
        for index in range.clone() {
            for i in self.triangles.triangle(index) {
                bounds.include(self.positions[i as usize]);
            }
        }
        let node = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            triangles: range.clone(),
            children: [UNOWNED; 2],
        });
        if range.len() > BVH_LEAF_TRIANGLES {
            let size = bounds.max - bounds.min;
            let axis = if size.x >= size.y && size.x >= size.z {
                0
            } else if size.y >= size.z {
                1
            } else {
                2
            };
            // Only the median partition is needed for the two child bounds.
            // Sorting every descendant repeatedly adds work without improving
            // exact triangle intersection tests or the rendered geometry.
            self.triangles.partition(range.clone(), &self.positions, axis);
            let mid = range.start + range.len() / 2;
            let left = self.build(range.start..mid);
            let right = self.build(mid..range.end);
            self.nodes[node].children = [left, right];
        }
        node
    }
    fn blocks(&self, start: Vec3, end: Vec3) -> bool {
        !self.nodes.is_empty() && self.hit_node(0, start, end - start)
    }
    fn hit_node(&self, index: usize, start: Vec3, delta: Vec3) -> bool {
        let node = &self.nodes[index];
        if !node.bounds.segment(start, delta) {
            return false;
        }
        if node.children[0] != UNOWNED {
            self.hit_node(node.children[0], start, delta)
                || self.hit_node(node.children[1], start, delta)
        } else {
            node.triangles.clone().any(|index| segment_triangle(start, delta,
                self.triangles.triangle(index).map(|i| self.positions[i as usize])))
        }
    }
}
fn segment_triangle(start: Vec3, delta: Vec3, [a, b, c]: [Vec3; 3]) -> bool {
    // Two-sided intersection: eligibility is a source-owned surface, independent
    // of which side the camera sees. Degenerate triangles never block a ray.
    let e1 = b - a;
    let e2 = c - a;
    let p = delta.cross(e2);
    let determinant = e1.dot(p);
    if determinant.abs() <= 0.000001 {
        return false;
    }
    let inverse = 1.0 / determinant;
    let s = start - a;
    let u = s.dot(p) * inverse;
    if !(-0.00001..=1.00001).contains(&u) {
        return false;
    }
    let q = s.cross(e1);
    let v = delta.dot(q) * inverse;
    if v < -0.00001 || u + v > 1.00001 {
        return false;
    }
    let t = e2.dot(q) * inverse;
    t > 0.00001 && t < 0.999
}

/// Keep touching ranges separate: they are consecutive source objects. Only
/// overlapping ownership is united. Unmarked and mixed triangles stay opaque.
fn source_groups(data: &SurfaceMeshData) -> Vec<Range<usize>> {
    let mut ranges = data
        .cutaway_ranges
        .iter()
        .filter(|r| r.start < r.end)
        .cloned()
        .collect::<Vec<_>>();
    ranges.sort_unstable_by_key(|r| (r.start, r.end));
    let mut groups: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(last) = groups.last_mut()
            && range.start < last.end
        {
            last.end = last.end.max(range.end);
        } else {
            groups.push(range);
        }
    }
    groups
}

/// Called once on the terrain build worker, before static spatial batching.
/// Copy indexed attributes exactly, retaining complete triangle order/winding.
pub(super) fn prepare(source: SurfaceMeshData) -> (SurfaceMeshData, Vec<PreparedGroup>) {
    prepare_collision_mode(source, false, &[])
}
pub(super) fn prepare_grove(source: SurfaceMeshData) -> (SurfaceMeshData, Vec<PreparedGroup>) {
    prepare_collision_mode(source, true, &[])
}
fn prepare_collision_mode(mut source: SurfaceMeshData, deferred: bool,
    layered_ranges:&[Range<usize>]) -> (SurfaceMeshData, Vec<PreparedGroup>) {
    let n = source.positions.len();
    if source.indices.len() % 3 != 0
        || source.normals.len() != n
        || source.uvs.len() != n
        || source.colors.len() != n
        || source.indices.iter().any(|&i| i as usize >= n)
        || source
            .cutaway_ranges
            .iter()
            .any(|r| r.start > r.end || r.end > n)
        || source
            .positions
            .iter()
            .any(|&p| !Vec3::from_array(p).is_finite())
    {
        return (source, Vec::new());
    }
    let ranges = source_groups(&source);
    if ranges.is_empty() {
        return (source, Vec::new());
    }
    if ranges.len() >= UNOWNED_VERTEX as usize {
        return (source, Vec::new());
    }
    source.compact_build_storage();
    // Source indices already use u32. Avoid a native-width owner ordinal per
    // vertex: the connected outdoor halo can contain millions of vertices.
    let mut owner = vec![UNOWNED_VERTEX; n];
    for (group, range) in ranges.iter().enumerate() {
        owner[range.clone()].fill(group as u32);
    }
    let bucket_for = |triangle: &[u32]| {
        let group = owner[triangle[0] as usize];
        if group != UNOWNED_VERTEX && triangle.iter().all(|&i| owner[i as usize] == group) {
            group as usize + 1
        } else {
            0
        }
    };
    let mut sizes = vec![0; ranges.len() + 1];
    for triangle in source.indices.chunks_exact(3) {
        sizes[bucket_for(triangle)] += 3;
    }
    let mut buckets: Vec<Vec<u32>> = sizes.into_iter().map(Vec::with_capacity).collect();
    for triangle in source.indices.chunks_exact(3) {
        buckets[bucket_for(triangle)].extend_from_slice(triangle);
    }
    let mut remap = vec![u32::MAX; n];
    let mut extracted = buckets
        .into_iter()
        .map(|indices| select(&source, Some(&owner), &mut remap, indices));
    let opaque = extracted.next().unwrap();
    let selected: Vec<_> = extracted
        .zip(ranges)
        .filter(|(data, _)| !data.indices.is_empty())
        .collect();
    // Selection owns every referenced attribute now. Release the full source
    // before BVHs and transparent draw leaves add their own geometry storage.
    drop(source);
    drop(owner);
    drop(remap);
    let groups = selected.into_iter()
        .map(|(data, source_range)| {
            let surface = if deferred { CollisionSurface::deferred(&data) }
                else { CollisionSurface::ready(TriangleTree::new(&data)) };
            let layered=layered_ranges.contains(&source_range);
            let leaves = if layered {
                // Small layered tabletop objects need vertical as well as lateral
                // sorting. Their exact owner/BVH and one opaque draw are retained.
                render_leaves_with_limits(data, 64, 256, true)
            } else { render_leaves(data) };
            let opaque_leaf = merged_opaque_leaf(&leaves);
            PreparedGroup {
                source_range,
                surface: Arc::new(surface),
                leaves,
                opaque_leaf,
                overlay: None,
                transform: Transform::default(),
                coverage_fade: false,
                depth_coverage_fade: layered,
                residual_opacity: RESIDUAL_OPACITY,
                camera_wall: None,
                runtime_lazy_sorted: false,
            }
        })
        .collect();
    (opaque, groups)
}
/// Links are emitted by the successful authored append, not inferred from
/// proximity. Unknown or malformed links leave both original draws intact.
pub(super) fn prepare_linked(
    textured: SurfaceMeshData,
    solid: SurfaceMeshData,
    links: Vec<(usize, usize)>,
) -> (
    SurfaceMeshData,
    SurfaceMeshData,
    Vec<PreparedGroup>,
    Vec<PreparedGroup>,
) {
    prepare_linked_with_layered_alpha(textured, solid, links, &[])
}

pub(super) fn prepare_linked_with_layered_alpha(
    textured: SurfaceMeshData, solid: SurfaceMeshData, links: Vec<(usize,usize)>,
    layered_textured_ranges:&[Range<usize>],
) -> (SurfaceMeshData,SurfaceMeshData,Vec<PreparedGroup>,Vec<PreparedGroup>) {
    let (textured, mut textured_groups) = prepare_collision_mode(textured,false,layered_textured_ranges);
    let (solid, mut solid_groups) = prepare(solid);
    for (solid_anchor, textured_anchor) in links {
        let Some(group) = solid_groups
            .iter_mut()
            .find(|g| g.source_range.contains(&solid_anchor))
        else {
            continue;
        };
        if group.overlay.is_some() {
            continue;
        }
        if let Some(index) = textured_groups
            .iter()
            .position(|g| g.source_range.contains(&textured_anchor))
        {
            group.overlay = Some(Box::new(textured_groups.swap_remove(index)));
        }
    }
    (textured, solid, textured_groups, solid_groups)
}

fn select(
    source: &SurfaceMeshData,
    // None means every vertex belongs to this whole authored fade object.
    owner: Option<&[u32]>,
    remap: &mut [u32],
    mut indices: Vec<u32>,
) -> SurfaceMeshData {
    let mut touched = Vec::new();
    for i in &mut indices {
        let old = *i as usize;
        if remap[old] == u32::MAX {
            let next = touched.len();
            remap[old] = next as u32;
            touched.push(old);
        }
        *i = remap[old];
    }
    // Reuse the index bucket and size attribute arrays exactly. Incremental
    // growth previously duplicated the bucket and doubled each attribute's
    // allocation at its largest growth step, exhausting browser linear memory.
    let mut data = SurfaceMeshData {
        positions: Vec::with_capacity(touched.len()),
        normals: Vec::with_capacity(touched.len()),
        uvs: Vec::with_capacity(touched.len()),
        colors: Vec::with_capacity(touched.len()),
        indices,
        ..Default::default()
    };
    for &old in &touched {
        let next = data.positions.len();
        data.positions.push(source.positions[old]);
        data.normals.push(source.normals[old]);
        data.uvs.push(source.uvs[old]);
        data.colors.push(source.colors[old]);
        if owner.is_none_or(|owners| owners[old] != UNOWNED_VERTEX) {
            if let Some(range) = data.cutaway_ranges.last_mut()
                && range.end == next
            {
                range.end += 1;
            } else {
                data.cutaway_ranges.push(next..next + 1);
            }
        }
    }
    for old in touched {
        remap[old] = u32::MAX;
    }
    data
}
fn render_leaves(data: SurfaceMeshData) -> Vec<PreparedLeaf> {
    render_leaves_with_limits(data,RENDER_LEAF_TRIANGLES,MAX_RENDER_LEAVES,false)
}
fn render_leaves_with_limits(mut data: SurfaceMeshData,triangles_per_leaf:usize,
    max_leaves:usize,vertical:bool) -> Vec<PreparedLeaf> {
    let _trace = crate::loading_trace::span("occluder.prepare_draw_leaves", || serde_json::json!({"vertices": data.positions.len(), "triangles": data.indices.len()/3}));
    // The source index buffer is no longer needed after the exact BVH build.
    // Move it through sorting/splitting instead of keeping several full copies.
    let mut chunks = vec![std::mem::take(&mut data.indices)];
    while chunks.len() < max_leaves {
        let candidate = chunks
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                if c.len() / 3 > triangles_per_leaf {
                    return true;
                }
                if c.len() / 3 < 24 {
                    return false;
                }
                let mut bounds = Bounds::empty();
                for &i in c.iter() {
                    bounds.include(data.positions[i as usize].into());
                }
                let extent = bounds.max - bounds.min;
                extent.x.max(extent.z) > 32.0
            })
            .max_by_key(|(_, c)| c.len())
            .map(|(i, _)| i);
        let Some(index) = candidate else {
            break;
        };
        let mut indices = chunks.swap_remove(index);
        let mut bounds = Bounds::empty();
        for &i in &indices {
            bounds.include(data.positions[i as usize].into());
        }
        let extent = bounds.max - bounds.min;
        let axis = if vertical && extent.y>extent.x.max(extent.z) {1}
            else if extent.x >= extent.z { 0 } else { 2 };
        let (triangles, remainder) = indices.as_chunks_mut::<3>();
        debug_assert!(remainder.is_empty());
        // Leaves need a spatial median, not a complete triangle ordering.
        // Partition once in linear time; sorting each level made dense groves
        // spend most of their loading time comparing the same triangle centers.
        // Both halves retain every original triangle and attribute.
        triangles.select_nth_unstable_by(triangles.len() / 2, |a, b| {
            let center = |t: &[u32; 3]| {
                t.iter()
                    .map(|&i| f64::from(data.positions[i as usize][axis]))
                    .sum::<f64>()
            };
            center(a).total_cmp(&center(b))
        });
        let split = (triangles.len() / 2) * 3;
        let right = indices.split_off(split);
        chunks.push(indices);
        chunks.push(right);
    }
    let mut remap = vec![u32::MAX; data.positions.len()];
    chunks
        .into_iter()
        .map(|indices| {
            let mut leaf = select(&data, None, &mut remap, indices);
            let mut bounds = Bounds::empty();
            for &p in &leaf.positions {
                bounds.include(p.into());
            }
            // Exact world geometry, centered local coordinates for Bevy's transparent
            // distance sorter. No camera-dependent mesh or index uploads are needed.
            let center = bounds.center();
            for p in &mut leaf.positions {
                *p = (Vec3::from_array(*p) - center).to_array();
            }
            let local_bounds = Aabb::from_min_max(bounds.min - center, bounds.max - center);
            PreparedLeaf {
                mesh: leaf.into_compact_mesh(),
                uploaded: None,
                center,
                bounds: local_bounds,
                shared: None,
            }
        })
        .collect()
}

/// Opaque objects need one draw, not the distance-sorted draws required by
/// transparency. Concatenate the existing leaves without changing their
/// triangles or non-position attributes. Original leaf centers are restored
/// into the group coordinate system; the complete collision BVH stays shared.
fn merged_opaque_leaf(leaves: &[PreparedLeaf]) -> Option<PreparedLeaf> {
    use bevy::mesh::{Indices, VertexAttributeValues};
    if leaves.len() < 2 { return None; }
    let mut merged: Option<Mesh> = None;
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for leaf in leaves {
        let mut mesh = leaf.mesh.clone();
        let VertexAttributeValues::Float32x3(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)? else { return None; };
        for position in positions {
            let point = Vec3::from_array(*position) + leaf.center;
            low = low.min(point); high = high.max(point);
            *position = point.to_array();
        }
        let indices = mesh.try_indices().ok()?.iter().map(|index| index as u32).collect();
        mesh.insert_indices(Indices::U32(indices));
        if let Some(merged) = &mut merged {
            if !merged.attributes().map(|(attribute, _)| attribute.id)
                .eq(mesh.attributes().map(|(attribute, _)| attribute.id)) { return None; }
            merged.merge(&mesh).ok()?;
        }
        else { merged = Some(mesh); }
    }
    let mut merged = merged?;
    // Mesh::merge needs a common index format while offsets accumulate. Restore
    // the compact format afterward: the opaque draw is used in both the main
    // and shadow passes, and merging must not double its index bandwidth.
    if let Some(Indices::U32(indices)) = merged.indices()
        && indices.iter().all(|&index| index <= u16::MAX as u32)
    {
        let Some(Indices::U32(indices)) = merged.remove_indices() else { unreachable!() };
        merged.insert_indices(Indices::U16(indices.into_iter().map(|index| index as u16).collect()));
        #[cfg(target_arch = "wasm32")]
        crate::browser_memory::align_indices(&mut merged);
    }
    Some(PreparedLeaf { mesh: merged, uploaded: None, center: Vec3::ZERO,
        bounds: Aabb::from_min_max(low, high), shared: None })
}

#[derive(Clone, Copy, Debug)]
struct Fade {
    opacity: f32,
    hold: f32,
    residual_opacity: f32,
}
impl Default for Fade {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            hold: 0.0,
            residual_opacity: RESIDUAL_OPACITY,
        }
    }
}
impl Fade {
    fn advance(&mut self, blocked: bool, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let ease = |current: f32, target: f32, rate: f32, seconds: f32| {
            target + (current - target) * (-rate * seconds).exp()
        };
        if blocked {
            self.hold = EXIT_HOLD_SECONDS;
            self.opacity = ease(self.opacity, self.residual_opacity, FADE_RATE, dt);
        } else {
            let held = dt.min(self.hold);
            self.hold = (self.hold - dt).max(0.0);
            self.opacity = ease(self.opacity, self.residual_opacity, FADE_RATE, held);
            self.opacity = ease(self.opacity, 1.0, RESTORE_RATE, dt - held);
        }
        if (self.opacity - 1.0).abs() < 0.0001 {
            self.opacity = 1.0;
        }
        if (self.opacity - self.residual_opacity).abs() < 0.0001 {
            self.opacity = self.residual_opacity;
        }
    }
}
#[derive(Component)]
struct OccluderGroup {
    surface: Arc<CollisionSurface>,
    overlay_surface: Option<Arc<CollisionSurface>>,
    overlay_materials: Option<(Handle<VoxelMaterial>, Handle<VoxelMaterial>)>,
    fade: Fade,
    blocked: bool,
    opaque: Handle<VoxelMaterial>,
    transparent: Handle<VoxelMaterial>,
    transparent_ready: bool,
    blending: bool,
    coverage_fade: bool,
    depth_coverage_fade: bool,
    camera_wall: Option<[f32;3]>,
}

#[derive(Clone)]
pub(super) struct RetainedSurface {
    surface: Arc<CollisionSurface>,
    overlay: Option<Arc<CollisionSurface>>,
}
impl RetainedSurface {
    pub(super) fn blocks(&self, pose: Transform, start: Vec3, end: Vec3) -> bool {
        let inverse = pose.compute_affine().inverse();
        let start = inverse.transform_point3(start);
        let end = inverse.transform_point3(end);
        self.surface.blocks(start, end)
            || self.overlay.as_ref().is_some_and(|surface| surface.blocks(start, end))
    }
}
pub(super) fn retained_surface(world: &World, entity: Entity) -> Option<RetainedSurface> {
    let group = world.get::<OccluderGroup>(entity)?;
    Some(RetainedSurface { surface: group.surface.clone(), overlay: group.overlay_surface.clone() })
}

#[derive(Component)]
struct OverlayLeaf;

#[derive(Component)]
pub(super) struct OpaqueObjectLeaf;
/// A sorted render alternative. These remain the retention leaves so battle
/// border and omitted-shadow proofs keep their original spatial granularity.
#[derive(Component)]
pub(super) struct SortedObjectLeaf;

fn blended(
        materials: &mut Assets<VoxelMaterial>,
        opaque: &Handle<VoxelMaterial>,
        coverage_fade: bool,
        depth_coverage_fade: bool,
    ) -> Handle<VoxelMaterial> {
        let mut material = materials
            .get(opaque)
            .expect("terrain material validated before group upload")
            .clone();
        // The stock shadow shader retains the original base alpha and geometry.
        let cutoff = match material.base.alpha_mode {
            AlphaMode::Mask(v) => v,
            _ if coverage_fade => -1.0,
            _ => 0.0,
        };
        material.base.alpha_mode = if depth_coverage_fade&&coverage_fade {AlphaMode::Mask(0.5)}else{AlphaMode::Blend};
        material.extension.smooth_depth_fade=depth_coverage_fade&&!coverage_fade;
        material.extension.cutaway = CutawayUniform::default();
        material.extension.cutaway.fade.w = cutoff;
        materials.add(material)
    }

pub(super) fn spawn_groups(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
    root: Entity,
    groups: Vec<PreparedGroup>,
    opaque: Handle<VoxelMaterial>,
    linked_opaque: Option<Handle<VoxelMaterial>>,
) -> Vec<Entity> {
    let mut spawned_leaves = Vec::new();
    // Opt-in until the native comparison verifies identical surfaces. The
    // solid palette shader does not consume constant zero UVs when neither a
    // base/normal texture nor source water animation is present. Preserve all
    // tagged finishes and every atlas-bearing material.
    let compact_uvs = std::env::var("CRYSTAL_COMPACT_SOLID_UVS").as_deref() == Ok("on");
    for PreparedGroup {
        source_range: _,
        surface,
        leaves,
        opaque_leaf,
        overlay,
        transform,
        coverage_fade,
        depth_coverage_fade,
        residual_opacity,
        camera_wall,
        runtime_lazy_sorted,
    } in groups
    {
        // Only runtime library groups may defer entities; linked authored
        // scenery keeps its returned leaf/domain and overlay contract intact.
        let lazy = runtime_lazy_sorted && opaque_leaf.is_some() && overlay.is_none();
        let transparent = if lazy { Handle::default() }
            else { blended(materials, &opaque, coverage_fade, depth_coverage_fade) };
        let (overlay_surface, overlay_leaves, overlay_opaque) = overlay
            .map(|g| (Some(g.surface), g.leaves, g.opaque_leaf))
            .unwrap_or_default();
        let overlay_materials = overlay_surface.as_ref().map(|_| {
            let opaque = linked_opaque
                .as_ref()
                .expect("authored cross-domain link requires its material")
                .clone();
            let transparent = blended(materials, &opaque, coverage_fade, depth_coverage_fade);
            (opaque, transparent)
        });
        let group = commands
            .spawn((
                (transform, Visibility::default()),
                OccluderGroup {
                    surface,
                    overlay_surface,
                    overlay_materials: overlay_materials.clone(),
                    fade: Fade { residual_opacity, ..Default::default() },
                    blocked: false,
                    opaque: opaque.clone(),
                    transparent,
                    transparent_ready: !lazy,
                    blending: false,
                    coverage_fade,
                    depth_coverage_fade,
                    camera_wall,
                },
            ))
            .id();
        let mut pending_sorted = Vec::new();
        for (domain, (leaves, opaque_leaf)) in [(leaves, opaque_leaf), (overlay_leaves, overlay_opaque)].into_iter().enumerate() {
            if leaves.is_empty() {
                continue;
            }
            let material = if domain == 0 {
                &opaque
            } else {
                &overlay_materials.as_ref().unwrap().0
            };
            let combined = opaque_leaf.is_some();
            for (mut prepared, opaque_only) in leaves.into_iter().map(|leaf| (leaf, false))
                .chain(opaque_leaf.into_iter().map(|leaf| (leaf, true))) {
                if compact_uvs && materials.get(material).is_some_and(|material| {
                    crate::compact_vertices::solid_uv0_is_unused(
                        &material.base, material.extension.water_enabled,
                    )
                }) {
                    crate::compact_vertices::remove_unused_zero_uv0(&mut prepared.mesh);
                }
                let defer = combined && !opaque_only;
                if lazy && defer {
                    pending_sorted.push(prepared);
                    continue;
                }
                let mut leaf = commands.spawn((
                    (MeshMaterial3d(material.clone()),
Transform::from_translation(prepared.center),),
                    if combined && !opaque_only { Visibility::Hidden } else { Visibility::Inherited },
                    prepared.bounds,
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ));
                if defer {
                    leaf.insert(DeferredMesh(Some(prepared)));
                } else {
                    leaf.insert(Mesh3d(upload_leaf(&mut prepared, meshes)));
                    if let Some(shared) = prepared.shared {
                        leaf.insert(SharedGeometry(shared));
                    }
                }
                if domain == 1 {
                    leaf.insert(OverlayLeaf);
                }
                if combined {
                    if opaque_only { leaf.insert(OpaqueObjectLeaf); }
                    else { leaf.insert(SortedObjectLeaf); }
                }
                let leaf = leaf.id();
                spawned_leaves.push(leaf);
                commands.entity(group).add_child(leaf);
            }
        }
        if !pending_sorted.is_empty() {
            commands.entity(group).insert(LazySortedLeaves(pending_sorted));
        }
        commands.entity(root).add_child(group);
    }
    spawned_leaves
}
fn target_rays(uniform: CutawayUniform, camera: Mat4) -> Option<(Vec3, [Vec3; 9])> {
    if uniform.bottom_radius.w <= 0.0 || !camera.is_finite() {
        return None;
    }
    let view = camera.inverse();
    let bottom = uniform.bottom_radius.truncate();
    let top = uniform.top_feather.truncate();
    if !view.is_finite()
        || !bottom.is_finite()
        || !top.is_finite()
        || view.transform_point3(bottom).z >= -0.001
        || view.transform_point3(top).z >= -0.001
    {
        return None;
    }
    let side = camera.x_axis.truncate().normalize_or_zero() * uniform.bottom_radius.w * 0.65;
    let targets = std::array::from_fn(|i| {
        bottom.lerp(top, [0.12, 0.52, 0.92][i / 3]) + side * (i % 3) as f32 - side
    });
    Some((camera.w_axis.truncate(), targets))
}

// Nearby conversation partners need the same clear sight line as the player.
// Keep the interest small and source-backed; props, distant crowds, trees and
// interior camera walls must not make the outdoor neighborhood disappear.
fn nearby_character_targets(frame: &crystal_render_api::VisualWorldFrame,
    heights: &[f32], player: CutawayUniform, camera: Mat4) -> Vec<Vec3> {
    use crystal_render_api::VisualActorId;
    if player.bottom_radius.w<=0. || !crate::cave_water::outdoor(&frame.map_id)
        || frame.map_id.contains("Forest") || !frame.tile_size.is_finite()
        || frame.tile_size.min_element()<=0. {return Vec::new();}
    let scale=frame.tile_size.y*2.;
    let mut interest=Vec::new();
    for actor in &frame.actors {
        let VisualActorId::Object(id)=actor.id else {continue;};
        let Some(height)=crate::new_bark_actors::modeled_character_height(frame,actor)
        else {continue;};
        let ground=crate::footing::actor_ground_point(frame,actor);
        if crate::footing::tile_at_visual_point(frame,ground).is_none() {continue;}
        let Some(support)=crate::resolved_footing_height(frame,ground,heights) else {continue;};
        let foot=crate::visual_point_to_voxel(ground,support+0.04);
        let delta=foot-player.bottom_radius.truncate();
        let distance=delta.length_squared();
        if !foot.is_finite() || distance>(scale*2.25).powi(2) {continue;}
        let Some(mut body)=CutawayUniform::for_player(foot,scale) else {continue;};
        body.top_feather= (foot+Vec3::Y*height).extend(body.top_feather.w);
        if let Some((_,targets))=target_rays(body,camera) {interest.push((distance,id,targets));}
    }
    interest.sort_by(|a,b|a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    interest.into_iter().take(2).flat_map(|(_,_,targets)|targets).collect()
}

// Whole-building fading retains its silhouette; the body aperture prevents
// that residual paint from covering the trainer. Other reveal styles
// keep their existing capsule policy, including foliage and room walls.
fn building_aperture(material: &mut VoxelMaterial, player: CutawayUniform, enabled: bool) {
    if enabled {
        material.extension.cutaway.bottom_radius = player.bottom_radius;
        material.extension.cutaway.top_feather = player.top_feather;
    }
}
fn sync(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    status: Res<VoxelViewStatus>,
    cache: Res<TerrainRevisionCache>,
    frame: Option<Res<crystal_render_api::VisualWorldFrame>>,
    // Occluder opacity is presentation state. It must settle while virtual
    // gameplay time is paused, including strict paired render captures.
    time: Res<Time<Real>>,
    cameras: Query<(&Camera, &GlobalTransform), With<VoxelWorldCamera>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut groups: Query<(Entity, &mut OccluderGroup, Ref<GlobalTransform>, &Children, Option<&mut LazySortedLeaves>)>,
    mut leaves: Query<(&mut MeshMaterial3d<VoxelMaterial>, Option<&OverlayLeaf>,
        Option<&OpaqueObjectLeaf>, Option<&SortedObjectLeaf>, &mut Visibility,
        Option<&mut DeferredMesh>)>,
    mut previous: Local<Option<(Mat4, CutawayUniform, bool)>>,
    mut previous_pit_targets: Local<Vec<Vec3>>,
    mut previous_character_targets: Local<Vec<Vec3>>,
) {
    let _trace = crate::loading_trace::span("frame.occluder_fades", || {
        let (count, blocked, faded, deferred, leaf_entities, pending_leaves) = groups.iter()
            .fold((0,0,0,0,0,0), |(count,blocked,faded,deferred,leaves,pending),
                (_,group,_,children,lazy)|
            (count+1, blocked+usize::from(group.blocked), faded+usize::from(group.fade.opacity < 1.),
                deferred+usize::from(group.surface.tree.get().is_none()),
                leaves+children.len(), pending+lazy.map_or(0,|lazy| lazy.0.len())));
        serde_json::json!({"groups":count,"blocked":blocked,"faded":faded,
            "deferred_collision":deferred,"leaf_entities":leaf_entities,
            "lazy_sorted_leaves":pending_leaves})
    });
    let (camera, active) = cameras
        .single()
        .map(|(c, t)| (t.to_matrix(), c.is_active))
        .unwrap_or((Mat4::IDENTITY, false));
    let uniform = cache
        .solid_material
        .as_ref()
        .and_then(|h| materials.get(h))
        .map(|m| m.extension.cutaway)
        .unwrap_or_default();
    let input = (camera, uniform, status.active && active);
    let mut pit_targets=if input.2&&uniform.bottom_radius.w>0.&&cache.built_profiles.is_some() {
        frame.as_ref().filter(|frame|cache.built_frame.as_ref()
            .is_some_and(|built|built.map_id==frame.map_id))
            .map(|frame|crate::mesh::lighthouse_pit_reveal_targets(frame,uniform.bottom_radius.truncate()))
            .unwrap_or_default()
    } else {Vec::new()};
    if !pit_targets.is_empty() {
        let view=camera.inverse();
        pit_targets.retain(|target|view.is_finite()&&target.is_finite()
            &&view.transform_point3(*target).z < -0.001);
    }
    let character_targets=if input.2 {
        frame.as_ref().filter(|frame|cache.built_frame.as_ref()
            .is_some_and(|built|built.map_id==frame.map_id))
            .map(|frame|nearby_character_targets(frame,&cache.footing_heights,uniform,camera))
            .unwrap_or_default()
    } else {Vec::new()};
    let changed = previous.as_ref() != Some(&input)||*previous_pit_targets!=pit_targets
        ||*previous_character_targets!=character_targets;
    *previous = Some(input);
    (*previous_pit_targets).clone_from(&pit_targets);
    (*previous_character_targets).clone_from(&character_targets);
    let rays = if input.2 {
        target_rays(uniform, camera)
    } else {
        None
    };
    for (group_entity, mut group, transform, children, lazy_sorted) in &mut groups {
        if changed || transform.is_changed() || group.is_added() {
            let outside=group.camera_wall.is_some_and(|plane|input.2&&outside_room_wall(plane,
                transform.to_matrix().inverse().transform_point3(camera.w_axis.truncate())));
            let has_ray_targets=group.camera_wall.is_none()||!pit_targets.is_empty();
            let blocked = outside || (has_ray_targets && rays.is_some_and(|(eye, targets)| {
                let local = transform.to_matrix().inverse();
                let start = local.transform_point3(eye);
                // Camera-plane walls retain their existing outside-room
                // policy. Nearby fall openings independently test actual
                // triangles, even when the player body itself is unobstructed.
                let body=targets.into_iter().filter(|_|group.camera_wall.is_none());
                let people=character_targets.iter().copied().filter(|_|
                    group.camera_wall.is_none() && group.fade.residual_opacity==BUILDING_REVEAL_OPACITY);
                local.is_finite()
                    && body.chain(pit_targets.iter().copied()).chain(people).any(|target| {
                        let end = local.transform_point3(target);
                        group.surface.blocks(start, end)
                            || group
                                .overlay_surface
                                .as_ref()
                                .is_some_and(|surface| surface.blocks(start, end))
                    })
            }));
            if blocked != group.blocked {
                group.blocked = blocked;
            }
        }
        // Settled opaque objects have no fade work. Keep their component
        // unchanged too, rather than running two exponentials and marking a
        // clear object modified on every host update. Ray decisions above
        // still use the complete original surface whenever inputs change.
        if !group.blocked && group.fade.opacity == 1.0 && group.fade.hold == 0.0
            && !group.blending {
            continue;
        }
        let previous_alpha = group.fade.opacity;
        let blocked = group.blocked;
        group.fade.advance(blocked, time.delta_secs());
        let blending = group.fade.opacity < 1.0;
        if blending != group.blending {
            if blending && !group.transparent_ready {
                group.transparent = blended(&mut materials, &group.opaque,
                    group.coverage_fade, group.depth_coverage_fade);
                group.transparent_ready = true;
            }
            if blending {
                if let Some(mut pending) = lazy_sorted {
                    for mut prepared in pending.0.drain(..) {
                        let local = Transform::from_translation(prepared.center);
                        let mesh = upload_leaf(&mut prepared, &mut meshes);
                        let mut leaf = commands.spawn((Mesh3d(mesh),
                            MeshMaterial3d(group.transparent.clone()), local,
                            transform.mul_transform(local), Visibility::Inherited,
                            prepared.bounds, RenderLayers::layer(VOXEL_RENDER_LAYER),
                            SortedObjectLeaf));
                        if let Some(shared) = prepared.shared { leaf.insert(SharedGeometry(shared)); }
                        let leaf = leaf.id();
                        commands.entity(group_entity).add_child(leaf);
                    }
                    commands.entity(group_entity).remove::<LazySortedLeaves>();
                }
            }
            let material = if blending {
                group.transparent.clone()
            } else {
                group.opaque.clone()
            };
            for child in children.iter() {
                if let Ok((mut handle, overlay, opaque_only, sorted, mut visibility, deferred)) = leaves.get_mut(child) {
                    if opaque_only.is_some() {
                        *visibility = if blending { Visibility::Hidden } else { Visibility::Inherited };
                    } else if sorted.is_some() {
                        if blending {
                            if let Some(mut deferred) = deferred {
                                if let Some(mut prepared) = deferred.0.take() {
                                    let mesh = upload_leaf(&mut prepared, &mut meshes);
                                    let mut entity = commands.entity(child);
                                    entity.insert(Mesh3d(mesh)).remove::<DeferredMesh>();
                                    if let Some(shared) = prepared.shared {
                                        entity.insert(SharedGeometry(shared));
                                    }
                                }
                            }
                        }
                        *visibility = if blending { Visibility::Inherited } else { Visibility::Hidden };
                    }
                    handle.0 = if overlay.is_some() {
                        let (opaque, transparent) = group.overlay_materials.as_ref().unwrap();
                        if blending {
                            transparent.clone()
                        } else {
                            opaque.clone()
                        }
                    } else {
                        material.clone()
                    };
                }
            }
            group.blending = blending;
        }
        if group.fade.opacity != previous_alpha || changed {
            let clear_body = (group.coverage_fade || group.depth_coverage_fade)
                && group.fade.residual_opacity == BUILDING_REVEAL_OPACITY
                && group.camera_wall.is_none();
            if let Some(mut material) = materials.get_mut(&group.transparent) {
                material.extension.cutaway.fade.x = 1.0 - group.fade.opacity;
                building_aperture(&mut material, uniform, clear_body);
            }
            if let Some((_, transparent)) = &group.overlay_materials {
                if let Some(mut material) = materials.get_mut(transparent) {
                    material.extension.cutaway.fade.x = 1.0 - group.fade.opacity;
                    building_aperture(&mut material, uniform, clear_body);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    fn nearby_npc_frame() -> crystal_render_api::VisualWorldFrame {
        crystal_render_api::VisualWorldFrame {map_id:"SaffronCity".into(),
            grid_size:UVec2::splat(16),tile_size:Vec2::splat(8.),viewport_size:Vec2::splat(128.),
            actors:vec![crystal_render_api::VisualActor {id:crystal_render_api::VisualActorId::Object(1),
                source_id:"youngster".into(),model_palette:None,texture:default(),
                center:Vec2::new(12.,-8.),size:Vec2::splat(16.),flip_x:false,
                above_priority:false,facing:Some(Vec2::X)}],..default()}
    }
    #[test]
    fn nearby_npc_reveals_a_real_roof_that_misses_every_player_ray() {
        let frame=nearby_npc_frame();let foot=Vec3::new(-20.,0.04,8.);
        let player=CutawayUniform::for_player(foot,16.).unwrap();
        let camera=Transform::from_translation(Vec3::new(-20.,80.,128.))
            .looking_at(foot+Vec3::Y*12.,Vec3::Y).to_matrix();
        let mut roof=SurfaceMeshData::default();
        quad(&mut roof,[[0.,32.,20.],[32.,32.,20.],[32.,32.,60.],[0.,32.,60.]],false);
        let tree=TriangleTree::new(&roof);
        let (eye,body)=target_rays(player,camera).unwrap();
        assert!(!body.into_iter().any(|target|tree.blocks(eye,target)));
        let targets=nearby_character_targets(&frame,&vec![0.;256],player,camera);
        assert_eq!(targets.len(),9);
        assert!(targets.iter().any(|&target|tree.blocks(eye,target)),
            "the nearby source NPC has a real blocked line of sight even when the player is clear");
        assert!(!targets.iter().any(|&target|tree.blocks(target,target+(target-eye))),
            "a roof behind the actor cannot trigger its reveal");
    }
    #[test]
    fn nearby_npc_interest_is_bounded_source_backed_and_tracks_motion_and_support() {
        use crystal_render_api::{VisualActorId,VisualEffectId};
        let mut frame=nearby_npc_frame();let foot=Vec3::new(-20.,0.04,8.);
        let player=CutawayUniform::for_player(foot,16.).unwrap();
        let camera=Transform::from_translation(Vec3::new(-20.,80.,128.))
            .looking_at(foot+Vec3::Y*12.,Vec3::Y).to_matrix();
        let original=frame.actors[0].clone();
        for (id,source,center) in [(VisualActorId::Object(2),"rock",Vec2::new(2.,-8.)),
            (VisualActorId::Object(3),"youngster",Vec2::new(60.,-60.)),
            (VisualActorId::Effect(VisualEffectId::GrassRustle),"youngster",Vec2::ZERO),
            (VisualActorId::RemotePlayer(4),"youngster",Vec2::ZERO),
            (VisualActorId::Object(5),"youngster",Vec2::splat(f32::NAN))] {
            frame.actors.push(crystal_render_api::VisualActor {id,source_id:source.into(),center,..original.clone()});
        }
        let heights=vec![0.;256];
        let first=nearby_character_targets(&frame,&heights,player,camera);assert_eq!(first.len(),9);
        assert!(nearby_character_targets(&frame,&[],player,camera).is_empty());
        assert!(nearby_character_targets(&frame,&vec![40.;256],player,camera).is_empty());
        let mut raised=frame.clone();raised.actors[0].center.x=-4.;
        assert_eq!(nearby_character_targets(&raised,&vec![24.;256],player,camera).len(),9,
            "a nearby character on one raised land shelf remains a conversation partner");
        frame.actors[0].center.x-=1.;
        assert_ne!(nearby_character_targets(&frame,&heights,player,camera),first);
        assert_ne!(nearby_character_targets(&frame,&vec![1.;256],player,camera),
            nearby_character_targets(&frame,&heights,player,camera));
        for (id,x) in [(6,2.),(7,4.)] {
            frame.actors.push(crystal_render_api::VisualActor {id:VisualActorId::Object(id),center:Vec2::new(x,-8.),..original.clone()});
        }
        let selected=nearby_character_targets(&frame,&heights,player,camera);
        assert_eq!(selected.len(),18,"at most two nearby conversation partners");
        for (targets,x) in selected.chunks_exact(9).zip([2.,4.]) {
            assert!((targets.iter().map(|p|p.x).sum::<f32>()/9.-x).abs()<0.001);
        }
        for map in ["PlayersHouse1F","IlexForest"] {
            frame.map_id=map.into();
            assert!(nearby_character_targets(&frame,&heights,player,camera).is_empty());
        }
    }
    #[test]
    fn lighthouse_pit_focus_detects_a_cap_that_does_not_hide_the_player() {
        let mut data=SurfaceMeshData::default();
        quad(&mut data,[[-12.,32.,20.],[32.,32.,20.],[32.,32.,60.],[-12.,32.,60.]],false);
        let tree=TriangleTree::new(&data);
        let foot=Vec3::new(-24.,0.04,8.);
        let camera=Transform::from_translation(Vec3::new(-24.,80.,128.))
            .looking_at(foot+Vec3::Y*12.,Vec3::Y).to_matrix();
        let uniform=CutawayUniform::for_player(foot,16.).unwrap();
        let (eye,body)=target_rays(uniform,camera).unwrap();
        assert!(!body.into_iter().any(|target|tree.blocks(eye,target)),"player alone does not reveal this cap");
        let tiles=(0..16).map(|i| {
            let (x,y)=(i%4,i/4);
            crystal_render_api::VisualTile {column:x as u32,row:y as u32,
                texture:default(),priority:false,animation_frames:None,
                source:crystal_render_api::VisualTileSource {tileset_id:"lighthouse".into(),
                    metatile_id:0x28,subtile_column:x as u8,subtile_row:y as u8,
                    tile_index:if y>=2 {1}else if (x+y)%2==0 {0x2e}else{0x2f}}}
        }).collect();
        let frame=crystal_render_api::VisualWorldFrame {map_id:"OlivineLighthouse2F".into(),
            grid_size:UVec2::new(4,4),tile_size:Vec2::splat(8.),tiles,..default()};
        let pit=crate::mesh::lighthouse_pit_reveal_targets(&frame,foot);
        assert_eq!(pit.len(),5);
        assert!(pit.into_iter().any(|target|tree.blocks(eye,target)),"nearby opening reveals its actual blocker");
        assert_eq!(uniform.bottom_radius.w,10.4,"player body radius stays unchanged");
    }
    #[test]
    fn building_body_aperture_tracks_support_without_changing_other_fades() {
        let mut material = crate::solid_terrain_material();
        material.extension.cutaway.fade = Vec4::new(0.82, 0., 0., -1.);
        let fade = material.extension.cutaway.fade;
        for foot in [Vec3::new(8., 0., 16.), Vec3::new(24., 12., 32.)] {
            let player = CutawayUniform::for_player(foot,16.).unwrap();
            building_aperture(&mut material,player,false);
            if foot.x == 8. {assert_eq!(material.extension.cutaway.bottom_radius,Vec4::ZERO);}
            building_aperture(&mut material,player,true);
            assert_eq!(material.extension.cutaway.bottom_radius,player.bottom_radius);
            assert_eq!(material.extension.cutaway.top_feather,player.top_feather);
            assert_eq!(material.extension.cutaway.fade,fade);
        }
        building_aperture(&mut material,CutawayUniform::default(),true);
        assert_eq!(material.extension.cutaway.bottom_radius,Vec4::ZERO);
        assert_eq!(material.extension.cutaway.fade,fade);
    }
    fn palette_weld_mesh(rgb: bool, wide: bool) -> Mesh {
        use bevy::mesh::{Indices, VertexAttributeValues};
        let mut mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList,
            bevy::asset::RenderAssetUsages::default());
        let mut positions = vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.],
            [0., 0., 0.], [1., 0., 0.], [0., 1., 0.],
            [-0.0, 0., 0.], [0., 0., 0.], [0., 0., 0.]];
        let count = positions.len();
        let mut colors = vec![[0.2, 0.3, 0.4, 1.0]; count];
        colors[7][0] = f32::from_bits(0x7fc00001);
        colors[8][0] = f32::from_bits(0x7fc00002);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, std::mem::take(&mut positions));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; count]);
        if rgb {
            mesh.insert_attribute(bevy::mesh::MeshVertexAttribute {
                format: bevy::render::render_resource::VertexFormat::Float32x3,
                ..Mesh::ATTRIBUTE_COLOR
            }, VertexAttributeValues::Float32x3(colors.into_iter().map(|c| [c[0], c[1], c[2]]).collect()));
        } else { mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors); }
        let indices = vec![0,1,2,3,4,5,6,1,2,7,1,2,8,1,2];
        mesh.insert_indices(if wide { Indices::U32(indices) }
            else { Indices::U16(indices.into_iter().map(|i| i as u16).collect()) });
        mesh
    }
    fn palette_triangle_bits(mesh: &Mesh) -> Vec<[u32; 10]> {
        use bevy::mesh::VertexAttributeValues;
        let VertexAttributeValues::Float32x3(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else { panic!() };
        let VertexAttributeValues::Float32x3(normals) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() else { panic!() };
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap();
        mesh.indices().unwrap().iter().map(|i| {
            let mut key = [0u32; 10];
            key[..3].copy_from_slice(&positions[i].map(f32::to_bits));
            key[3..6].copy_from_slice(&normals[i].map(f32::to_bits));
            match colors {
                VertexAttributeValues::Float32x3(values) => key[6..9].copy_from_slice(&values[i].map(f32::to_bits)),
                VertexAttributeValues::Float32x4(values) => key[6..10].copy_from_slice(&values[i].map(f32::to_bits)),
                _ => panic!(),
            }
            key
        }).collect()
    }

    #[test]
    fn palette_tree_vertex_weld_preserves_every_ordered_triangle_bit_and_float_exception() {
        for rgb in [false, true] { for wide in [false, true] {
            let mut mesh = palette_weld_mesh(rgb, wide);
            let expected = palette_triangle_bits(&mesh);
            let format = mesh.attributes().find(|(a, _)| a.id == Mesh::ATTRIBUTE_COLOR.id).unwrap().0.format;
            assert_eq!(weld_opaque_palette_tree_vertices(&mut mesh), 3);
            assert_eq!(mesh.count_vertices(), 6,
                "signed zero and different NaN payloads must remain independent vertices");
            assert_eq!(palette_triangle_bits(&mesh), expected);
            assert_eq!(mesh.attributes().find(|(a, _)| a.id == Mesh::ATTRIBUTE_COLOR.id).unwrap().0.format, format);
            assert_eq!(weld_opaque_palette_tree_vertices(&mut mesh), 0);
        }}
    }

    #[test]
    fn palette_tree_vertex_weld_leaves_unsupported_and_malformed_meshes_intact() {
        use bevy::mesh::Indices;
        for case in 0..5 {
            let mut mesh = palette_weld_mesh(false, true);
            match case {
                0 => { mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.;2]; 9]); }
                1 => { mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.,0.,1.]; 8]); }
                2 => { mesh.insert_indices(Indices::U32(vec![0, 1, 9])); }
                3 => { mesh.insert_indices(Indices::U32(vec![0, 1])); }
                _ => { mesh.remove_indices(); }
            }
            let before = mesh.clone();
            assert_eq!(weld_opaque_palette_tree_vertices(&mut mesh), 0);
            assert_eq!(format!("{mesh:?}"), format!("{before:?}"),
                "an unsupported or malformed mesh must remain completely unmodified");
        }
        let mut gpu_only = palette_weld_mesh(false, true);
        let _gpu = gpu_only.take_gpu_data().unwrap();
        assert_eq!(weld_opaque_palette_tree_vertices(&mut gpu_only), 0);
    }

    #[test]
    fn frozen_view_publishes_deferred_geometry_once_without_exposing_overworld_alternatives() {
        for shared in [false, true] {
            let mut world = World::new();
            world.init_resource::<Assets<Mesh>>();
            let source = Mesh::from(Cuboid::new(2., 5., 2.));
            let expected = source.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().clone();
            let shared_mesh = shared.then(|| Arc::new(Mutex::new(SharedLeaf {
                mesh: Some(source.clone()), uploaded: None,
            })));
            let root = world.spawn_empty().id();
            let leaves: Vec<_> = (0..2).map(|_| world.spawn((
                Visibility::Hidden,
                DeferredMesh(Some(PreparedLeaf {
                    mesh: source.clone(), uploaded: None, center: Vec3::ZERO,
                    bounds: Aabb::from_min_max(Vec3::new(-1.,0.,-1.),Vec3::new(1.,5.,1.)),
                    shared: shared_mesh.clone(),
                })), SortedObjectLeaf,
            )).id()).collect();
            world.entity_mut(root).add_children(&leaves);
            prepare_retention_meshes(&mut world, root);
            let first: Vec<_> = leaves.iter().map(|&leaf| {
                assert!(world.get::<Mesh3d>(leaf).is_none());
                assert_eq!(world.get::<Visibility>(leaf), Some(&Visibility::Hidden));
                let mesh = retained_mesh(&world, leaf).expect("retained sorted geometry is ready");
                assert_eq!(world.resource::<Assets<Mesh>>().get(&mesh).unwrap()
                    .attribute(Mesh::ATTRIBUTE_POSITION), Some(&expected));
                mesh
            }).collect();
            assert_eq!(world.resource::<Assets<Mesh>>().len(), if shared { 1 } else { 2 });
            prepare_retention_meshes(&mut world, root);
            for (&leaf, original) in leaves.iter().zip(first) {
                assert_eq!(retained_mesh(&world, leaf), Some(original));
            }
            assert_eq!(world.resource::<Assets<Mesh>>().len(), if shared { 1 } else { 2 });
        }
    }

    #[test]
    fn runtime_tree_lazy_children_preserve_first_fade_pose_retention_and_reuse() {
        use bevy::asset::AssetApp;
        for (retain_first, populated_default) in [(false,false),(true,false),(false,true),(true,true)] {
            let mut source = crate::mesh::TerrainMeshData::default();
            crate::foliage::append_grove_tree(&mut source, [0.,32.,0.,32.],
                3.,16.,[4,8],[false;4]);
            let (_, mut groups) = prepare(source.solid);
            assert_eq!(groups.len(), 1);
            let expected: Vec<_> = groups[0].leaves.iter().map(|leaf| (
                Transform::from_translation(leaf.center),
                leaf.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().clone(),
                leaf.mesh.indices().unwrap().clone(),
            )).collect();
            assert!(expected.len() > 1);
            share_groups(&mut groups);
            use_lazy_sorted_entities(&mut groups);
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin,
                bevy::asset::AssetPlugin::default()))
                .init_asset::<Mesh>().init_asset::<VoxelMaterial>()
                .init_resource::<TerrainRevisionCache>().init_resource::<VoxelViewStatus>()
                .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                    std::time::Duration::from_secs_f64(1./60.)));
            register(&mut app);
            if populated_default {
                app.world_mut().resource_mut::<Assets<VoxelMaterial>>()
                    .insert(Handle::<VoxelMaterial>::default().id(), crate::solid_terrain_material())
                    .unwrap();
            }
            let base_material_count = 1 + usize::from(populated_default);
            let root = app.world_mut().spawn((Transform::from_xyz(80.,7.,-20.)
                .with_rotation(Quat::from_rotation_y(0.7))
                .with_scale(Vec3::new(1.2,1.5,0.8)), Visibility::Inherited)).id();
            let material = app.world_mut().resource_mut::<Assets<VoxelMaterial>>()
                .add(crate::solid_terrain_material());
            app.world_mut().resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                    spawn_groups(&mut world.commands(), &mut meshes, &mut materials,
                        root, groups, material.clone(), None);
                });
            });
            app.world_mut().flush(); app.update();
            let group = app.world_mut().query_filtered::<Entity, With<OccluderGroup>>()
                .single(app.world()).unwrap();
            assert_eq!(app.world().get::<Children>(group).unwrap().len(),1);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(),1);
            assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(),base_material_count);
            assert_eq!(app.world_mut().query_filtered::<Entity, With<SortedObjectLeaf>>()
                .iter(app.world()).count(),0);
            if retain_first {
                prepare_retention_meshes(app.world_mut(),root);
                assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(),base_material_count);
                assert_eq!(app.world().resource::<Assets<Mesh>>().len(),1+expected.len());
                let children = app.world().get::<Children>(group).unwrap();
                for child in children.iter().skip(1) {
                    assert!(app.world().get::<Mesh3d>(child).is_none());
                    assert_eq!(app.world().get::<Visibility>(child),Some(&Visibility::Hidden));
                    assert!(retained_mesh(app.world(),child).is_some());
                }
                prepare_retention_meshes(app.world_mut(),root);
                assert_eq!(app.world().get::<Children>(group).unwrap().len(),1+expected.len());
            }
            // A scroll on the first fade must not publish a stale or identity pose.
            app.world_mut().get_mut::<Transform>(root).unwrap().translation.x += 13.;
            app.world_mut().get_mut::<OccluderGroup>(group).unwrap().fade.opacity=0.5;
            app.update();
            let global=*app.world().get::<GlobalTransform>(group).unwrap();
            let sorted: Vec<_> = app.world().get::<Children>(group).unwrap().iter()
                .filter(|child| app.world().get::<SortedObjectLeaf>(*child).is_some()).collect();
            assert_eq!(sorted.len(),expected.len());
            assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(),base_material_count+1);
            let fade_material = &app.world().get::<OccluderGroup>(group).unwrap().transparent;
            assert_ne!(fade_material.id(), Handle::<VoxelMaterial>::default().id());
            assert!(app.world().get::<OccluderGroup>(group).unwrap().transparent_ready);
            let mut first_handles=Vec::new();
            for (&child,(local,positions,indices)) in sorted.iter().zip(&expected) {
                assert_eq!(app.world().get::<GlobalTransform>(child),Some(&global.mul_transform(*local)));
                assert_eq!(app.world().get::<Visibility>(child),Some(&Visibility::Inherited));
                let handle=&app.world().get::<Mesh3d>(child).unwrap().0;
                let mesh=app.world().resource::<Assets<Mesh>>().get(handle).unwrap();
                assert_eq!(mesh.attribute(Mesh::ATTRIBUTE_POSITION),Some(positions));
                assert_eq!(mesh.indices(),Some(indices));
                first_handles.push(handle.clone());
            }
            for _ in 0..180 {app.update();}
            assert_eq!(app.world().get::<OccluderGroup>(group).unwrap().fade.opacity,1.);
            for &child in &sorted { assert_eq!(app.world().get::<Visibility>(child),Some(&Visibility::Hidden)); }
            app.world_mut().get_mut::<OccluderGroup>(group).unwrap().fade.opacity=0.5;
            app.update();
            for (&child,handle) in sorted.iter().zip(&first_handles) {
                assert_eq!(&app.world().get::<Mesh3d>(child).unwrap().0,handle);
            }
            assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(),base_material_count+1);
            app.world_mut().entity_mut(root).despawn();
            app.update();
            assert_eq!(app.world_mut().query_filtered::<Entity, With<OccluderGroup>>()
                .iter(app.world()).count(),0);
            assert_eq!(app.world_mut().query_filtered::<Entity, With<LazySortedLeaves>>()
                .iter(app.world()).count(),0);
        }
    }

    #[test]
    fn opaque_and_sorted_grove_draws_are_exclusive_through_fade_and_restore() {
        use bevy::asset::AssetApp;
        let mut source = crate::mesh::TerrainMeshData::default();
        crate::foliage::append_grove_tree(&mut source, [0.0, 32.0, 0.0, 32.0],
            3.0, 16.0, [4, 8], [false; 4]);
        let (_, groups) = prepare(source.solid);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin,
            bevy::asset::AssetPlugin::default()))
            .init_asset::<Mesh>().init_asset::<VoxelMaterial>()
            .init_resource::<TerrainRevisionCache>().init_resource::<VoxelViewStatus>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0)));
        register(&mut app);
        let root = app.world_mut().spawn((Transform::default(), Visibility::Inherited)).id();
        let material = app.world_mut().resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        app.world_mut().resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
            world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                spawn_groups(&mut world.commands(), &mut meshes, &mut materials, root, groups, material, None);
            });
        });
        app.world_mut().flush(); app.update();
        let counts = |world: &mut World| {
            let mut counts = (0, 0);
            for (visibility, opaque, sorted) in world.query::<(&Visibility, Option<&OpaqueObjectLeaf>, Option<&SortedObjectLeaf>)>().iter(world) {
                if *visibility != Visibility::Hidden {
                    counts.0 += usize::from(opaque.is_some());
                    counts.1 += usize::from(sorted.is_some());
                }
            }
            counts
        };
        assert_eq!(counts(app.world_mut()), (1, 0));
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        assert_eq!(mesh_count, 1, "hidden sorted leaves must not upload at spawn");
        assert!(app.world_mut().query::<&DeferredMesh>().iter(app.world()).count() > 1);
        let material_count = app.world().resource::<Assets<VoxelMaterial>>().len();
        app.world_mut().resource_mut::<Messages<AssetEvent<Mesh>>>().clear();
        {
            let world = app.world_mut();
            let mut query = world.query::<&mut OccluderGroup>();
            for mut group in query.iter_mut(world) { group.fade.opacity = 0.5; }
        }
        app.update();
        let fading = counts(app.world_mut());
        assert_eq!(fading.0, 0); assert!(fading.1 > 1);
        let fading_mesh_count = app.world().resource::<Assets<Mesh>>().len();
        assert_eq!(fading_mesh_count, mesh_count + fading.1);
        let mesh_events = app.world().resource::<Messages<AssetEvent<Mesh>>>();
        assert_eq!(mesh_events.iter_current_update_messages()
            .filter(|event| matches!(event, AssetEvent::Added { .. })).count(), fading.1,
            "fade geometry must publish during the same update as its visibility");
        assert_eq!(app.world_mut().query::<&DeferredMesh>().iter(app.world()).count(), 0);
        assert_eq!(app.world_mut().query_filtered::<&Mesh3d, With<SortedObjectLeaf>>().iter(app.world()).count(), fading.1);
        for _ in 0..180 { app.update(); }
        assert_eq!(counts(app.world_mut()), (1, 0));
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), fading_mesh_count);
        assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(), material_count);
    }

    #[test]
    fn opaque_grove_batch_preserves_every_source_triangle_attribute_and_collision_surface() {
        for coordinates in [[4, 8], [-3, 7]] {
            let mut source = crate::mesh::TerrainMeshData::default();
            crate::foliage::append_grove_tree(&mut source, [0.0, 32.0, 0.0, 32.0],
                3.0, 16.0, coordinates, [false; 4]);
            let (unowned, groups) = prepare(source.solid);
            let expected = geometry_signature(&unowned, groups.clone());
            let batches = groups.into_iter().map(|mut group| {
                assert!(group.leaves.len() > 1, "fixture must exercise sorted-leaf consolidation");
                let collision = group.surface.clone();
                group.leaves = vec![group.opaque_leaf.take().expect("compatible source leaves must batch")];
                assert!(Arc::ptr_eq(&collision, &group.surface));
                group
            }).collect();
            assert_eq!(geometry_signature(&unowned, batches), expected,
                "opaque batching must preserve complete source positions, normals, UVs, colors, winding and triangle multiplicity");
        }
    }

    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use std::collections::BTreeMap;
    fn quad(data: &mut SurfaceMeshData, points: [[f32; 3]; 4], marked: bool) {
        let base = data.positions.len();
        for p in points {
            let n = data.positions.len() as f32;
            data.positions.push(p);
            data.normals.push([0.0, 0.0, 1.0]);
            data.uvs.push([n * 0.1, n * 0.03]);
            data.colors.push([0.2, n * 0.005, 0.7, 1.0]);
        }
        data.indices
            .extend([0, 1, 2, 0, 2, 3].map(|i| base as u32 + i));
        if marked {
            data.cutaway_ranges.push(base..base + 4);
        }
    }
    #[test]
    fn opaque_merge_uses_narrow_indices_only_when_every_offset_fits() {
        use bevy::mesh::Indices;
        for first_count in [65_533, 65_534] {
            let make_leaf = |count: usize, center: Vec3| {
                let source = SurfaceMeshData {
                    positions: vec![[0.25, -0.5, 0.75]; count],
                    normals: vec![[0.0, 1.0, 0.0]; count],
                    uvs: vec![[0.0; 2]; count],
                    colors: vec![[0.137, 0.287, 0.863, 1.0]; count],
                    indices: vec![0, (count - 2) as u32, (count - 1) as u32],
                    ..Default::default()
                };
                PreparedLeaf { mesh: source.into_compact_mesh(), uploaded: None, center,
                    bounds: Aabb::from_min_max(Vec3::ZERO, Vec3::ONE), shared: None }
            };
            let leaves = [make_leaf(first_count, Vec3::X), make_leaf(3, Vec3::Z)];
            let merged = merged_opaque_leaf(&leaves).unwrap();
            let indices = merged.mesh.indices().unwrap();
            assert_eq!(matches!(indices, Indices::U16(_)), first_count == 65_533);
            assert_eq!(indices.iter().collect::<Vec<_>>(),
                [0, first_count - 2, first_count - 1, first_count, first_count + 1, first_count + 2]);
            for (attribute, values) in merged.mesh.attributes() {
                if attribute.id == Mesh::ATTRIBUTE_POSITION.id { continue; }
                let expected = leaves.iter().flat_map(|leaf|
                    leaf.mesh.attribute(attribute.id).unwrap().get_bytes().iter().copied()
                ).collect::<Vec<_>>();
                assert_eq!(values.get_bytes(), expected);
            }
            let Some(VertexAttributeValues::Float32x3(positions)) =
                merged.mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!() };
            for (index, position) in positions.iter().enumerate() {
                let center = if index < first_count { Vec3::X } else { Vec3::Z };
                assert_eq!(position.map(f32::to_bits),
                    (Vec3::new(0.25, -0.5, 0.75) + center).to_array().map(f32::to_bits));
            }
        }
    }

    #[test]
    fn early_palette_compaction_keeps_geometry_tags_and_linked_texture_coordinates() {
        let material = crate::solid_terrain_material();
        assert!(crate::compact_vertices::solid_uv0_is_unused(
            &material.base, material.extension.water_enabled,
        ));
        for uv in [[0.0; 2], [82000.0, 1000.0]] {
            let mut source = wall(0.0);
            source.uvs.fill(uv);
            let (_, mut groups) = prepare(source.clone());
            let (_, mut overlay) = prepare(source);
            groups[0].overlay = Some(Box::new(overlay.pop().unwrap()));
            let leaf = &groups[0].leaves[0];
            let before: Vec<_> = leaf.mesh.attributes()
                .filter(|(attribute, _)| attribute.id != Mesh::ATTRIBUTE_UV_0.id)
                .map(|(attribute, values)| (attribute.id, values.get_bytes().to_vec())).collect();
            let indices = leaf.mesh.indices().cloned();
            let center = leaf.center;
            let bounds = leaf.bounds;
            compact_palette_group_uv0(&mut groups);
            let leaf = &groups[0].leaves[0];
            let after: Vec<_> = leaf.mesh.attributes()
                .filter(|(attribute, _)| attribute.id != Mesh::ATTRIBUTE_UV_0.id)
                .map(|(attribute, values)| (attribute.id, values.get_bytes().to_vec())).collect();
            assert_eq!(before, after);
            assert_eq!(indices, leaf.mesh.indices().cloned());
            assert_eq!(center, leaf.center);
            assert_eq!(bounds, leaf.bounds);
            assert_eq!(leaf.mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_none(), uv == [0.0; 2]);
            assert!(groups[0].overlay.as_ref().unwrap().leaves[0].mesh
                .attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        }
    }

    #[test]
    fn selecting_reuses_indices_and_sizes_exact_attributes_without_changing_ownership() {
        let mut source = SurfaceMeshData::default();
        quad(&mut source, [[0.,0.,0.],[1.,0.,0.],[1.,1.,0.],[0.,1.,0.]], false);
        let indices = vec![3,0,1,3,1,2];
        let allocation = indices.as_ptr();
        let mut remap = vec![u32::MAX;4];
        let selected = select(&source, Some(&[UNOWNED_VERTEX,0,UNOWNED_VERTEX,0]), &mut remap, indices);
        assert_eq!(selected.indices.as_ptr(), allocation, "selection must reuse its large index bucket");
        assert_eq!(selected.indices, [0,1,2,0,2,3]);
        for (new,old) in [3,0,1,2].into_iter().enumerate() {
            assert_eq!(selected.positions[new],source.positions[old]);
            assert_eq!(selected.normals[new],source.normals[old]);
            assert_eq!(selected.uvs[new],source.uvs[old]);
            assert_eq!(selected.colors[new],source.colors[old]);
        }
        assert_eq!(selected.cutaway_ranges,[0..1,2..3]);
        assert_eq!(selected.positions.capacity(),4);
        assert_eq!(selected.normals.capacity(),4);
        assert_eq!(selected.uvs.capacity(),4);
        assert_eq!(selected.colors.capacity(),4);
        assert!(remap.iter().all(|&i|i==u32::MAX));
    }
    #[test]
    fn fading_cutout_crowns_retain_the_source_alpha_cutoff() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(), Visibility::default())).id();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<VoxelMaterial>::default();
        let mut material = crate::solid_terrain_material();
        material.base.alpha_mode = AlphaMode::Mask(0.63);
        let handle = materials.add(material);
        let mut data = SurfaceMeshData::default();
        quad(&mut data, [[0.,0.,0.],[0.,1.,0.],[1.,1.,0.],[1.,0.,0.]], true);
        let (_, groups) = prepare(data);
        let mut queue = bevy::ecs::world::CommandQueue::default();
        { let mut commands = Commands::new(&mut queue, &world);
          spawn_groups(&mut commands, &mut meshes, &mut materials, root, groups, handle, None); }
        queue.apply(&mut world);
        let mut query = world.query::<&OccluderGroup>();
        let group = query.single(&world).unwrap();
        let faded = materials.get(&group.transparent).unwrap();
        assert_eq!(faded.base.alpha_mode, AlphaMode::Blend);
        assert_eq!(faded.extension.cutaway.fade.w, 0.63);
    }

    #[test]
    fn source_cabinet_depth_coverage_survives_cache_and_preserves_other_fades() {
        for opt_in in [false,true] {
            let mut world=World::new();
            let root=world.spawn((Transform::default(),Visibility::default())).id();
            let mut meshes=Assets::<Mesh>::default();
            let mut materials=Assets::<VoxelMaterial>::default();
            let original=crate::solid_terrain_material();
            let handle=materials.add(original.clone());
            let mut data=SurfaceMeshData::default();
            quad(&mut data,[[0.,0.,0.],[0.,1.,0.],[1.,1.,0.],[1.,0.,0.]],true);
            let (_,mut groups)=prepare(data);
            let source=groups[0].source_range.clone();
            use_source_depth_coverage_fade(&mut groups,&[0..999]);
            assert!(!groups[0].depth_coverage_fade);
            if opt_in {use_source_depth_coverage_fade(&mut groups,&[source]);}
            share_groups(&mut groups);
            let cached=WeakPreparedGroup::new(&groups[0]).upgrade().unwrap();
            assert_eq!(cached.depth_coverage_fade,opt_in);
            assert_eq!(cached.coverage_fade,opt_in);
            let mut queue=bevy::ecs::world::CommandQueue::default();
            {let mut commands=Commands::new(&mut queue,&world);
                spawn_groups(&mut commands,&mut meshes,&mut materials,root,vec![cached],handle.clone(),None);}
            queue.apply(&mut world);
            let group=world.query::<&OccluderGroup>().single(&world).unwrap();
            let faded=materials.get(&group.transparent).unwrap();
            assert_eq!(faded.base.alpha_mode,if opt_in {AlphaMode::Mask(0.5)}else{AlphaMode::Blend});
            assert_eq!(faded.extension.cutaway.fade.w,if opt_in {-1.}else{0.});
            assert_eq!(materials.get(&handle).unwrap().base.alpha_mode,original.base.alpha_mode);
        }
    }
    #[test]
    fn palette_crown_coverage_survives_cache_and_keeps_opaque_material_unchanged() {
        for coverage in [false, true] {
            let mut world = World::new();
            let root = world.spawn((Transform::default(), Visibility::default())).id();
            let mut meshes = Assets::<Mesh>::default();
            let mut materials = Assets::<VoxelMaterial>::default();
            let original = crate::solid_terrain_material();
            let handle = materials.add(original.clone());
            let mut data = SurfaceMeshData::default();
            quad(&mut data, [[0.,0.,0.],[0.,1.,0.],[1.,1.,0.],[1.,0.,0.]], true);
            let (_, mut groups) = prepare(data);
            if coverage { use_coverage_fade(&mut groups); }
            share_groups(&mut groups);
            let cached = WeakPreparedGroup::new(&groups[0]).upgrade().unwrap();
            assert_eq!(cached.coverage_fade, coverage);
            let mut queue = bevy::ecs::world::CommandQueue::default();
            { let mut commands = Commands::new(&mut queue, &world);
              spawn_groups(&mut commands, &mut meshes, &mut materials, root, vec![cached], handle.clone(), None); }
            queue.apply(&mut world);
            let mut query = world.query::<&OccluderGroup>();
            let group = query.single(&world).unwrap();
            let faded = materials.get(&group.transparent).unwrap();
            assert_eq!(faded.extension.cutaway.fade.w, if coverage {-1.} else {0.});
            let opaque = materials.get(&handle).unwrap();
            assert_eq!(opaque.base.alpha_mode, original.base.alpha_mode);
            assert_eq!(opaque.base.base_color, original.base.base_color);
            assert_eq!(opaque.extension.cutaway.fade, original.extension.cutaway.fade);
        }
    }

    #[test]
    fn layered_alpha_writes_depth_without_masking_or_mutating_the_opaque_material() {
        let mut materials=Assets::<VoxelMaterial>::default();
        let original=crate::solid_terrain_material();
        let handle=materials.add(original.clone());
        for (coverage,depth) in [(false,false),(false,true),(true,true)] {
            let faded=blended(&mut materials,&handle,coverage,depth);
            let material=materials.get(&faded).unwrap();
            assert_eq!(material.extension.smooth_depth_fade,depth&&!coverage);
            assert_eq!(material.base.alpha_mode,if depth&&coverage {AlphaMode::Mask(0.5)}else{AlphaMode::Blend});
            assert_eq!(material.extension.cutaway.fade.w,if coverage {-1.}else{0.});
        }
        let opaque=materials.get(&handle).unwrap();
        assert_eq!(opaque.base.alpha_mode,original.base.alpha_mode);
        assert!(!opaque.extension.smooth_depth_fade);
    }

    #[test]
    fn exact_layered_source_sorting_preserves_geometry_owner_and_neighbor_defaults() {
        let mut data=SurfaceMeshData::default();
        for x in [0.,4.] {
            let first=data.positions.len();
            for level in 0..160 {
                let y=level as f32*0.25;
                quad(&mut data,[[x,y,0.],[x,y,1.],[x+1.,y,1.],[x+1.,y,0.]],false);
            }
            data.cutaway_ranges.push(first..data.positions.len());
        }
        let range=data.cutaway_ranges[0].clone();
        let (opaque,baseline)=prepare(data.clone());
        let original=geometry_signature(&opaque,baseline.clone());
        let (fine_opaque,_,mut fine,_)=prepare_linked_with_layered_alpha(data.clone(),
            SurfaceMeshData::default(),Vec::new(),std::slice::from_ref(&range));
        assert_eq!(fine.len(),2);assert_eq!(fine[0].source_range,range);
        assert!(fine[0].leaves.len()>baseline[0].leaves.len());
        assert!(fine[0].leaves.len()<=256);assert!(fine[0].opaque_leaf.is_some());
        assert_eq!(fine[1].leaves.len(),baseline[1].leaves.len());
        assert!(fine[0].leaves.iter().map(|l|l.center.y).fold(f32::NEG_INFINITY,f32::max)
            -fine[0].leaves.iter().map(|l|l.center.y).fold(f32::INFINITY,f32::min)>20.,
            "layered faces must partition vertically, not only in X/Z");
        assert_eq!(geometry_signature(&fine_opaque,fine.clone()),original);
        assert_eq!(prepared_bounds(&fine),prepared_bounds(&baseline));
        assert!(fine.iter().all(|g|!g.coverage_fade));
        assert!(fine[0].depth_coverage_fade);assert!(!fine[1].depth_coverage_fade);
        share_groups(&mut fine);
        let cached=WeakPreparedGroup::new(&fine[0]).upgrade().unwrap();
        assert_eq!(cached.source_range,range);
        assert_eq!(cached.leaves.len(),fine[0].leaves.len());
        assert_eq!(cached.residual_opacity,baseline[0].residual_opacity);
        assert!(cached.depth_coverage_fade&&!cached.coverage_fade);
        let (_,_,unmatched,_)=prepare_linked_with_layered_alpha(data,
            SurfaceMeshData::default(),Vec::new(),&[0..9999]);
        assert_eq!(unmatched[0].leaves.len(),baseline[0].leaves.len());
    }

    #[test]
    fn outdoor_reveal_keeps_scenery_legible_and_clears_most_player_pixels() {
        for (foliage,depth) in [(false,false),(false,true),(true,false)] {
            let mut data=SurfaceMeshData::default();
            quad(&mut data,[[0.,0.,0.],[0.,1.,0.],[1.,1.,0.],[1.,0.,0.]],true);
            quad(&mut data,[[2.,0.,0.],[2.,1.,0.],[3.,1.,0.],[3.,0.,0.]],true);
            let (_,mut groups)=prepare(data);
            let bounds=prepared_bounds(&groups);let range=groups[1].source_range.clone();
            let surfaces=groups.iter().map(|g|g.surface.clone()).collect::<Vec<_>>();
            if foliage {use_foliage_reveal(&mut groups,true);}
            else {
                if depth {use_source_depth_coverage_fade(&mut groups,std::slice::from_ref(&range));}
                use_building_reveal(&mut groups,std::slice::from_ref(&range));
            }
            assert_eq!(prepared_bounds(&groups),bounds);
            for (i,group) in groups.iter().enumerate() {assert!(Arc::ptr_eq(&group.surface,&surfaces[i]));}
            assert_eq!(groups[0].residual_opacity,if foliage {FOLIAGE_REVEAL_OPACITY} else {RESIDUAL_OPACITY});
            assert_eq!(groups[1].residual_opacity,if foliage {FOLIAGE_REVEAL_OPACITY} else {BUILDING_REVEAL_OPACITY});
            let mut fade=Fade {residual_opacity:groups[1].residual_opacity,..Default::default()};
            for _ in 0..120 {fade.advance(true,1./60.);}
            assert!(fade.opacity>=0.15 && fade.opacity<=0.25,"retain a ghost with at least75% clear pixels");
            for _ in 0..240 {fade.advance(false,1./60.);}
            assert_eq!(fade.opacity,1.,"restore the whole object after clearing it");
            share_groups(&mut groups);
            let cached=WeakPreparedGroup::new(&groups[1]).upgrade().unwrap();
            assert_eq!(cached.residual_opacity,groups[1].residual_opacity);
            assert_eq!(cached.source_range,range);
            assert_eq!(cached.coverage_fade,foliage||!depth);
            assert_eq!(cached.depth_coverage_fade,depth);
        }
    }

    #[test]
    fn source_coverage_opt_in_preserves_neighboring_walls_and_cached_ownership() {
        let mut data = SurfaceMeshData::default();
        quad(&mut data, [[0.,0.,0.],[0.,1.,0.],[1.,1.,0.],[1.,0.,0.]], true);
        quad(&mut data, [[2.,0.,0.],[2.,1.,0.],[3.,1.,0.],[3.,0.,0.]], true);
        let (_, mut groups) = prepare(data);
        assert_eq!(groups.len(), 2);
        let ranges = groups.iter().map(|g| g.source_range.clone()).collect::<Vec<_>>();
        use_source_coverage_fade(&mut groups, &[ranges[1].clone(), 0..999], 0.);
        assert!(!groups[0].coverage_fade);
        assert!(groups[1].coverage_fade);
        share_groups(&mut groups);
        for (index, group) in groups.iter().enumerate() {
            let cached = WeakPreparedGroup::new(group).upgrade().unwrap();
            assert_eq!(cached.source_range, ranges[index]);
            assert_eq!(cached.coverage_fade, index == 1);
            assert_eq!(cached.residual_opacity, if index == 1 {0.} else {RESIDUAL_OPACITY});
        }
    }

    fn wall(z: f32) -> SurfaceMeshData {
        let mut data = SurfaceMeshData::default();
        quad(
            &mut data,
            [
                [-16.0, 0.0, z],
                [16.0, 0.0, z],
                [16.0, 32.0, z],
                [-16.0, 32.0, z],
            ],
            true,
        );
        data
    }
    fn u_wall() -> SurfaceMeshData {
        let mut data = wall(-16.0);
        quad(
            &mut data,
            [
                [-16.0, 0.0, -16.0],
                [-16.0, 0.0, 16.0],
                [-16.0, 32.0, 16.0],
                [-16.0, 32.0, -16.0],
            ],
            true,
        );
        quad(
            &mut data,
            [
                [16.0, 0.0, 16.0],
                [16.0, 0.0, -16.0],
                [16.0, 32.0, -16.0],
                [16.0, 32.0, 16.0],
            ],
            true,
        );
        data.cutaway_ranges = vec![0..data.positions.len()];
        data
    }
    #[test]
    fn retained_battle_surface_reuses_real_triangles_and_transformed_openings() {
        let tree = Arc::new(CollisionSurface::ready(TriangleTree::new(&u_wall())));
        let retained = RetainedSurface { surface: tree.clone(), overlay: None };
        assert!(Arc::ptr_eq(&tree, &retained.surface));
        let pose = Transform::from_xyz(3.0, 1.0, -2.0)
            .with_rotation(Quat::from_rotation_y(0.7)).with_scale(Vec3::splat(0.0625));
        let target = pose.transform_point(Vec3::new(0., 10., 0.));
        let opening = pose.transform_point(Vec3::new(0., 10., 80.));
        assert!(!retained.blocks(pose, opening, target));
        assert!(retained.blocks(pose, pose.transform_point(Vec3::new(80., 10., 0.)), target));
        let linked = RetainedSurface { surface: tree, overlay: Some(Arc::new(CollisionSurface::ready(TriangleTree::new(&wall(16.))))) };
        assert!(linked.blocks(pose, opening, target), "linked overlay surfaces remain blockers");
    }

    #[test]
    fn deferred_collision_rejects_distant_rays_without_building_and_preserves_concave_openings() {
        let data = u_wall();
        let surface = CollisionSurface::deferred(&data);
        assert!(surface.tree.get().is_none());
        assert!(!surface.blocks(Vec3::splat(1000.), Vec3::splat(1100.)));
        assert!(surface.tree.get().is_none(), "off-object rays must not prepare a tree");
        assert!(surface.source.lock().unwrap().is_some());
        let opening = (Vec3::new(0., 10., 80.), Vec3::new(0., 10., 0.));
        assert!(!surface.blocks(opening.0, opening.1), "bounds are never a substitute for actual triangles");
        let tree = surface.tree.get().expect("a nearby ray initializes exact triangles");
        assert!(surface.source.lock().unwrap().is_none(), "release the redundant source buffers");
        assert!(surface.blocks(Vec3::new(80., 10., 0.), opening.1));
        assert!(std::ptr::eq(tree, surface.tree.get().unwrap()), "reuse the immutable tree");
    }

    #[test]
    fn deferred_grove_collision_preserves_eager_rays_and_every_render_attribute() {
        let mut source = crate::mesh::TerrainMeshData::default();
        crate::foliage::append_grove_tree(&mut source, [0.,32.,0.,32.], 3.,16.,[4,8],[false;4]);
        let (opaque, eager) = prepare(source.solid.clone());
        let (deferred_opaque, deferred) = prepare_grove(source.solid);
        assert_eq!(geometry_signature(&opaque, eager.clone()), geometry_signature(&deferred_opaque, deferred.clone()));
        assert_eq!(prepared_bounds(&eager), prepared_bounds(&deferred));
        assert!(deferred.iter().all(|group| group.surface.tree.get().is_none()));
        for i in 0..2048 {
            let x = (i % 64) as f32 - 16.;
            let y = (i / 64) as f32 * 3.;
            let start = Vec3::new(x,y,-40.);
            let end = Vec3::new(32.-x,y,72.);
            assert_eq!(eager.iter().any(|group| group.surface.blocks(start,end)),
                deferred.iter().any(|group| group.surface.blocks(start,end)), "ray {i}");
        }
    }

    #[test]
    fn concave_source_room_empty_space_does_not_fade_its_walls() {
        let tree = TriangleTree::new(&u_wall());
        let target = Vec3::new(0.0, 10.0, 0.0);
        let eye = Vec3::new(0.0, 10.0, 80.0);
        assert!(
            tree.nodes[0].bounds.segment(eye, target - eye),
            "fixture must defeat an AABB-only test"
        );
        assert!(!tree.blocks(eye, target));
        for eye in [
            Vec3::new(80.0, 10.0, 0.0),
            Vec3::new(-80.0, 10.0, 0.0),
            Vec3::new(0.0, 10.0, -80.0),
        ] {
            assert!(tree.blocks(eye, target));
        }
        assert!(!tree.blocks(Vec3::new(0.0, 80.0, 0.0), target));
        assert!(
            !tree.blocks(Vec3::new(0.0, 10.0, 80.0), Vec3::new(0.0, 10.0, 40.0)),
            "a wall behind the player does not fade"
        );
    }
    #[test]
    fn weak_templates_share_live_instances_and_release_retired_geometry() {
        let (_, mut groups) = prepare(u_wall());
        share_groups(&mut groups);
        let weak: Vec<_> = groups.iter().map(WeakPreparedGroup::new).collect();
        let mut world = World::new();
        let root = world.spawn((Transform::default(), Visibility::default())).id();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<VoxelMaterial>::default();
        let material = materials.add(crate::solid_terrain_material());
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            spawn_groups(&mut commands, &mut meshes, &mut materials, root, groups, material.clone(), None);
        }
        queue.apply(&mut world);
        let originals: Vec<_> = world.query::<&Mesh3d>().iter(&world).map(|mesh| mesh.0.id()).collect();
        let original_count = meshes.len();
        let copies = weak.iter().map(WeakPreparedGroup::upgrade).collect::<Option<Vec<_>>>().unwrap();
        {
            let mut commands = Commands::new(&mut queue, &world);
            spawn_groups(&mut commands, &mut meshes, &mut materials, root, copies, material, None);
        }
        queue.apply(&mut world);
        assert_eq!(meshes.len(), original_count, "a live copy reuses uploaded buffers");
        assert!(world.query::<&Mesh3d>().iter(&world).all(|mesh| originals.contains(&mesh.0.id())));
        world.despawn(root);
        assert!(weak.iter().all(|group| group.upgrade().is_none()),
            "weak recipe metadata must not retain retired geometry or mesh handles");
    }

    #[test]
    fn staging_preserves_exact_leaf_buffers_and_reuses_uploaded_handles() {
        let (_, mut groups) = prepare(u_wall());
        let before = groups[0].leaves[0].mesh.clone();
        let center = groups[0].leaves[0].center;
        let bounds = groups[0].leaves[0].bounds;
        let mut meshes = Assets::<Mesh>::default();
        stage_group_meshes(&mut groups, &mut meshes);
        let leaf = &groups[0].leaves[0];
        let handle = leaf.uploaded.clone().unwrap();
        let uploaded = meshes.get(&handle).unwrap();
        let attributes = |mesh: &Mesh| mesh.attributes()
            .map(|(id, values)| (*id, values.clone())).collect::<Vec<_>>();
        assert_eq!(attributes(&before), attributes(uploaded));
        assert_eq!(before.indices(), uploaded.indices());
        assert_eq!(leaf.mesh.count_vertices(), 0);
        assert_eq!(leaf.center, center);
        assert_eq!(leaf.bounds, bounds);
        let count = meshes.len();
        stage_group_meshes(&mut groups, &mut meshes);
        assert_eq!(meshes.len(), count);
        assert_eq!(groups[0].leaves[0].uploaded.as_ref(), Some(&handle));
    }
    #[test]
    fn repeated_tree_instances_share_geometry_but_keep_transforms_and_fades_independent() {
        let (_, mut canonical) = prepare(u_wall());
        let vertices = canonical[0].leaves[0].mesh.count_vertices();
        share_groups(&mut canonical);
        let surface = canonical[0].surface.clone();
        let mut meshes = Assets::<Mesh>::default();
        let mut handles = Vec::new();
        for index in 0..1000 {
            let mut instance = canonical.clone();
            place_groups(&mut instance, Transform::from_xyz(index as f32*32., 0., 0.));
            assert!(Arc::ptr_eq(&surface, &instance[0].surface));
            assert_eq!(instance[0].transform.translation.x, index as f32*32.);
            stage_group_meshes(&mut instance, &mut meshes);
            handles.push(upload_leaf(&mut instance[0].leaves[0], &mut meshes));
        }
        assert!(handles.iter().all(|h| h == &handles[0]));
        assert_eq!(meshes.len(), canonical[0].leaves.len());
        assert_eq!(meshes.get(&handles[0]).unwrap().count_vertices(), vertices);
        let mut blocked = Fade::default();
        let mut clear = Fade::default();
        blocked.advance(true, 1.);
        clear.advance(false, 1.);
        assert_eq!(blocked.opacity, RESIDUAL_OPACITY);
        assert_eq!(clear.opacity, 1.);
    }
    #[test]
    fn bvh_keeps_source_corner_bits_and_sharing_instead_of_expanding_triangles() {
        let mut source = SurfaceMeshData::default();
        source.positions.push([0.0, 3.0, -0.0]);
        for edge in 0..32 {
            let angle = edge as f32 * std::f32::consts::TAU / 32.0;
            source.positions.push([angle.cos() * 16.0, 3.0, angle.sin() * 16.0]);
            source.indices.extend([0, 1 + edge, 1 + (edge + 1) % 32]);
        }
        source.indices.extend_from_within(0..3);
        source.indices.extend([0, 0, 0]);
        let tree = TriangleTree::new(&source);
        let key = |t: [u32; 3]| {
            t.into_iter().flat_map(|i| source.positions[i as usize].map(f32::to_bits))
                .collect::<Vec<_>>()
        };
        let mut expected: Vec<_> = source.indices.chunks_exact(3)
            .map(|t| key([t[0], t[1], t[2]])).collect();
        let mut actual: Vec<_> = tree.triangles.iter().map(|t| {
            t.into_iter().flat_map(|i| tree.positions[i as usize].to_array().map(f32::to_bits))
                .collect::<Vec<_>>()
        }).collect();
        expected.sort_unstable();
        actual.sort_unstable();
        assert_eq!(actual, expected);
        assert!(tree.positions.len() <= source.positions.len());
        let indexed_bytes = tree.positions.len() * std::mem::size_of::<Vec3>()
            + tree.triangles.storage_bytes();
        assert!(indexed_bytes < tree.triangles.len() * std::mem::size_of::<[Vec3; 3]>());
    }

    #[test]
    fn collision_sharing_preserves_signed_zero_and_every_duplicate_corner() {
        let mut data = SurfaceMeshData::default();
        let corners = [[0.0, 0.0, 1.0], [-0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        data.positions.extend(corners);
        data.positions.extend(corners);
        data.indices = vec![0, 2, 3, 4, 6, 7, 1, 2, 3, 5, 6, 7];
        let tree = TriangleTree::new(&data);
        assert_eq!(tree.positions.len(), 4);
        let expected: Vec<_> = data.indices.iter().flat_map(|&i|
            data.positions[i as usize].map(f32::to_bits)).collect();
        let actual: Vec<_> = tree.triangles.iter().flatten().flat_map(|i|
            tree.positions[i as usize].to_array().map(f32::to_bits)).collect();
        assert_eq!(actual, expected);
        assert!(tree.blocks(Vec3::Y, -Vec3::Y));
    }

    #[test]
    fn collision_indices_are_narrow_only_when_every_original_corner_fits() {
        let indices = [0, u16::MAX as u32, 42];
        let narrow = TriangleIndices::new(&indices);
        assert_eq!(narrow.triangle(0), indices);
        assert_eq!(narrow.storage_bytes(), 6);
        let mut data = SurfaceMeshData::default();
        data.positions = vec![[0.0; 3]; u16::MAX as usize + 2];
        data.positions[0] = [-1.0, 0.0, -1.0];
        data.positions[u16::MAX as usize] = [1.0, 0.0, -1.0];
        data.positions[u16::MAX as usize + 1] = [0.0, 0.0, 1.0];
        data.indices = vec![0, u16::MAX as u32, u16::MAX as u32 + 1];
        let tree = TriangleTree::new(&data);
        assert_eq!(TriangleIndices::new(&data.indices).triangle(0), [0, 65535, 65536]);
        assert_eq!(TriangleIndices::new(&data.indices).storage_bytes(), 12);
        assert!(tree.triangles.storage_bytes() <= 6);
        assert!(tree.blocks(Vec3::Y, -Vec3::Y));
        assert!(!tree.blocks(Vec3::new(2.0, 1.0, 0.0), Vec3::new(2.0, -1.0, 0.0)));
        // Unique positions retain their full local index range after sharing.
        data.positions = (0..=u16::MAX as u32 + 1)
            .map(|i| [i as f32, 2.0, 0.0]).collect();
        data.positions[0] = [-1.0, 0.0, -1.0];
        data.positions[u16::MAX as usize] = [1.0, 0.0, -1.0];
        data.positions[u16::MAX as usize + 1] = [0.0, 0.0, 1.0];
        let wide = TriangleTree::new(&data);
        assert_eq!(wide.triangles.triangle(0), [0, 65535, 65536]);
        assert_eq!(wide.triangles.storage_bytes(), 12);
        assert!(wide.blocks(Vec3::Y, -Vec3::Y));
    }

    #[test]
    fn packed_collision_indices_preserve_exceptions_and_bvh_reordering() {
        let mut source: Vec<[u32; 3]> = (0..20).map(|i| [i * 10, i * 10 + 1, i * 10 + 2]).collect();
        source.extend([[65535, 65534, 0], [500, 100, 900], [0, 256, 300], [0, 255, 255]]);
        let flat: Vec<_> = source.iter().flatten().copied().collect();
        let mut packed = TriangleIndices::new(&flat);
        assert!(packed.storage_bytes() < source.len() * 6);
        assert_eq!(packed.iter().collect::<Vec<_>>(), source);
        let positions: Vec<_> = (0..=65535).map(|i| Vec3::new(i as f32, 0.0, 0.0)).collect();
        let endpoints = (packed.triangle(0), packed.triangle(source.len() - 1));
        packed.partition(1..source.len() - 1, &positions, 0);
        assert_eq!((packed.triangle(0), packed.triangle(source.len() - 1)), endpoints);
        let mut after = packed.iter().collect::<Vec<_>>();
        after.sort_unstable();
        source.sort_unstable();
        assert_eq!(after, source, "partitioning cannot change any original corner");
    }

    #[test]
    fn direct_index_build_matches_legacy_packed_grove_intersections_and_retention() {
        let mut source = crate::mesh::TerrainMeshData::default();
        crate::foliage::append_grove_tree(&mut source, [-12.,8.,-9.,3.], 3., 8., [8,12], [true,false,true,false]);
        let direct = TriangleTree::new(&source.solid);
        let remap = direct.positions.iter().enumerate().map(|(i,p)|
            (p.to_array().map(f32::to_bits), i as u32)).collect::<bevy::platform::collections::HashMap<_,_>>();
        let original = source.solid.indices.iter().map(|&i|
            remap[&source.solid.positions[i as usize].map(f32::to_bits)]).collect::<Vec<_>>();
        let mut legacy = TriangleTree { positions:direct.positions.clone(),
            triangles:TriangleIndices::new(&original), nodes:Vec::new() };
        legacy.build(0..legacy.triangles.len());
        let mut a = direct.triangles.iter().collect::<Vec<_>>();
        let mut b = legacy.triangles.iter().collect::<Vec<_>>();
        a.sort_unstable(); b.sort_unstable();
        assert_eq!(a,b);
        assert_eq!(direct.triangles.storage_bytes(),legacy.triangles.storage_bytes());
        let mut seed = 731u32;
        let mut coordinate = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) as f32 / 16777216. * 80. - 40.
        };
        for _ in 0..2048 {
            let start = Vec3::new(coordinate(),coordinate()+42.,coordinate());
            let end = Vec3::new(coordinate(),coordinate()+12.,coordinate());
            assert_eq!(direct.blocks(start,end),legacy.blocks(start,end), "{start:?} -> {end:?}");
        }
    }

    #[test]
    fn collision_exception_table_overflow_preserves_full_narrow_indices() {
        let source = vec![900, 0, 901].repeat(65537);
        let indices = TriangleIndices::new(&source);
        assert_eq!(indices.len(), 65537);
        assert_eq!(indices.storage_bytes(), 65537 * 6);
        assert!(indices.iter().all(|triangle| triangle == [900, 0, 901]));
    }

    #[test]
    fn bvh_matches_actual_triangles_across_orbit_zoom_and_translated_roots() {
        let mut data = u_wall();
        // More than one BVH leaf, with disjoint scenery and an empty interior.
        for x in -24..=24 {
            quad(
                &mut data,
                [
                    [x as f32 * 16.0, 0.0, -64.0],
                    [x as f32 * 16.0 + 8.0, 0.0, -64.0],
                    [x as f32 * 16.0 + 8.0, 20.0, -64.0],
                    [x as f32 * 16.0, 20.0, -64.0],
                ],
                false,
            );
        }
        let tree = TriangleTree::new(&data);
        assert!(tree.nodes.len() > 1);
        for orbit in 0..128 {
            for distance in [36.0, 80.0, 160.0] {
                for rise in [0.1, 0.5, 1.5] {
                    let angle = orbit as f32 * std::f32::consts::TAU / 128.0;
                    let eye = Vec3::new(
                        angle.sin() * distance,
                        10.0 + distance * rise,
                        angle.cos() * distance,
                    );
                    let camera = Transform::from_translation(eye)
                        .looking_at(Vec3::Y * 10.0, Vec3::Y)
                        .to_matrix();
                    let uniform = CutawayUniform::for_player(Vec3::ZERO, 16.0).unwrap();
                    let (origin, targets) = target_rays(uniform, camera).unwrap();
                    for target in targets {
                        let expected = data.indices.chunks_exact(3).any(|t| {
                            let points = [t[0], t[1], t[2]]
                                .map(|i| Vec3::from_array(data.positions[i as usize]));
                            segment_triangle(origin, target - origin, points)
                        });
                        assert_eq!(tree.blocks(origin, target), expected);
                        let shift = Vec3::new(219.0, 8.0, -397.0);
                        assert_eq!(
                            tree.blocks((origin + shift) - shift, (target + shift) - shift),
                            expected
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn blocked_groups_fade_smoothly_restore_and_repeat_without_sticking() {
        let mut fade = Fade::default();
        for _ in 0..12 {
            let before = fade.opacity;
            fade.advance(true, 1.0 / 60.0);
            assert!(fade.opacity < before && fade.opacity > RESIDUAL_OPACITY);
        }
        assert!(fade.opacity < 0.25);
        for _ in 0..20 {
            for _ in 0..60 {
                fade.advance(true, 1.0 / 60.0);
            }
            assert_eq!(fade.opacity, RESIDUAL_OPACITY);
            fade.advance(false, 0.06);
            assert_eq!(fade.opacity, RESIDUAL_OPACITY);
            fade.advance(true, 0.02);
            assert_eq!(fade.opacity, RESIDUAL_OPACITY);
            let mut previous = fade.opacity;
            for _ in 0..120 {
                fade.advance(false, 1.0 / 60.0);
                assert!(fade.opacity >= previous);
                previous = fade.opacity;
            }
            assert_eq!(fade.opacity, 1.0);
        }
    }
    #[test]
    fn full_gateway_reveal_settles_restores_and_keeps_default_wall_policy() {
        for residual in [0., RESIDUAL_OPACITY] {
            let mut fade = Fade { residual_opacity: residual, ..Default::default() };
            for _ in 0..3 {
                for _ in 0..120 {
                    let previous = fade.opacity;
                    fade.advance(true, 1. / 60.);
                    assert!(fade.opacity <= previous && fade.opacity >= residual);
                }
                assert_eq!(fade.opacity, residual);
                fade.advance(false, EXIT_HOLD_SECONDS / 2.);
                assert_eq!(fade.opacity, residual);
                for _ in 0..120 { fade.advance(false, 1. / 60.); }
                assert_eq!(fade.opacity, 1.);
            }
        }
    }
    #[test]
    fn fade_is_time_step_equivalent_even_when_a_frame_crosses_exit_hold() {
        for chunks in [1, 2, 5, 30, 60, 144] {
            let mut one = Fade::default();
            let mut many = Fade::default();
            for (blocked, seconds) in [
                (true, 0.17),
                (false, 0.07),
                (true, 0.09),
                (false, 0.24),
                (false, 0.31),
            ] {
                one.advance(blocked, seconds);
                for _ in 0..chunks {
                    many.advance(blocked, seconds / chunks as f32);
                }
                assert!(
                    (one.opacity - many.opacity).abs() < 0.000003,
                    "{chunks}: {one:?}, {many:?}"
                );
                assert!((one.hold - many.hold).abs() < 0.000003);
            }
        }
        let mut fade = Fade::default();
        for dt in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
            fade.advance(true, dt);
        }
        assert_eq!(fade.opacity, 1.0);
    }
    fn corners(data: &SurfaceMeshData) -> BTreeMap<Vec<u32>, usize> {
        let mut result = BTreeMap::new();
        for triangle in data.indices.chunks_exact(3) {
            let mut key = Vec::new();
            for &i in triangle {
                let i = i as usize;
                key.extend(data.positions[i].map(f32::to_bits));
                key.extend(data.normals[i].map(f32::to_bits));
                key.extend(data.uvs[i].map(f32::to_bits));
                key.extend(data.colors[i].map(f32::to_bits));
                key.push(u32::from(
                    data.cutaway_ranges.iter().any(|r| r.contains(&i)),
                ));
            }
            *result.entry(key).or_default() += 1;
        }
        result
    }
    fn restore_leaf(leaf: PreparedLeaf) -> SurfaceMeshData {
        let mut data = SurfaceMeshData::default();
        let VertexAttributeValues::Float32x3(positions) =
            leaf.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!()
        };
        data.positions = positions
            .iter()
            .map(|&p| (Vec3::from_array(p) + leaf.center).to_array())
            .collect();
        let VertexAttributeValues::Float32x3(normals) =
            leaf.mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap()
        else {
            panic!()
        };
        data.normals = normals.clone();
        let VertexAttributeValues::Float32x2(uvs) =
            leaf.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!()
        };
        data.uvs = uvs.clone();
        data.colors = match leaf.mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() {
            VertexAttributeValues::Float32x4(colors) => colors.clone(),
            VertexAttributeValues::Float32x3(colors) => colors.iter()
                .map(|&[r, g, b]| [r, g, b, 1.0]).collect(),
            _ => panic!("unexpected color representation"),
        };
        data.indices = leaf.mesh.indices().unwrap().iter().map(|i| i as u32).collect();
        data.cutaway_ranges = vec![0..data.positions.len()];
        data
    }
    #[test]
    fn source_partition_and_sort_origins_reconstruct_every_corner_and_keep_neighbors_opaque() {
        let mut data = SurfaceMeshData::default();
        for x in 0..5200 {
            let x = (x % 260) as f32 * 0.25;
            quad(
                &mut data,
                [
                    [x, 0.0, 20.0],
                    [x + 0.25, 0.0, 20.0],
                    [x + 0.25, 32.0, 20.0],
                    [x, 32.0, 20.0],
                ],
                false,
            );
        }
        data.cutaway_ranges.push(0..data.positions.len());
        let next = data.positions.len();
        quad(
            &mut data,
            [
                [80.0, 0.0, 20.0],
                [90.0, 0.0, 20.0],
                [90.0, 32.0, 20.0],
                [80.0, 32.0, 20.0],
            ],
            true,
        );
        assert_eq!(
            data.cutaway_ranges[1].start, next,
            "touching ranges belong to distinct objects"
        );
        quad(
            &mut data,
            [
                [-2.0, 0.0, -2.0],
                [2.0, 0.0, -2.0],
                [2.0, 0.0, 2.0],
                [-2.0, 0.0, 2.0],
            ],
            false,
        );
        let expected = corners(&data);
        let (opaque, groups) = prepare(data);
        assert_eq!(groups.len(), 2);
        assert_eq!(opaque.indices.len(), 6);
        assert!(opaque.cutaway_ranges.is_empty());
        let mut actual = corners(&opaque);
        for group in groups {
            assert!(group.leaves.len() <= MAX_RENDER_LEAVES);
            for leaf in group.leaves {
                for (key, count) in corners(&restore_leaf(leaf)) {
                    *actual.entry(key).or_default() += count;
                }
            }
        }
        assert_eq!(actual, expected);
    }
    #[test]
    fn malformed_or_unmarked_geometry_is_kept_whole() {
        let mut data = wall(20.0);
        data.cutaway_ranges.clear();
        let (opaque, groups) = prepare(data.clone());
        assert_eq!(opaque, data);
        assert!(groups.is_empty());
        data.cutaway_ranges.push(0..999);
        let (opaque, groups) = prepare(data.clone());
        assert_eq!(opaque, data);
        assert!(groups.is_empty());
    }
    #[test]
    fn ecs_uses_current_camera_and_roots_without_mesh_or_material_growth() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::transform::TransformPlugin))
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<VoxelMaterial>>()
            .init_resource::<TerrainRevisionCache>()
            .init_resource::<VoxelViewStatus>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            ));
        register(&mut app);
        app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
        let mut material = crate::solid_terrain_material();
        material.extension.cutaway = CutawayUniform::for_player(Vec3::ZERO, 16.0).unwrap();
        let handle = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(material);
        app.world_mut()
            .resource_mut::<TerrainRevisionCache>()
            .solid_material = Some(handle.clone());
        let overlay_handle = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::textured_terrain_material(Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(42), std::marker::PhantomData)));
        let camera = app
            .world_mut()
            .spawn((
                (Camera3d::default(), Msaa::Off,
Transform::from_xyz(0.0, 20.0, 80.0)
                        .looking_at(Vec3::Y * 10.0, Vec3::Y),),
                VoxelWorldCamera,
            ))
            .id();
        let root = app.world_mut().spawn((Transform::default(), Visibility::default())).id();
        let opaque = handle.clone();
        let mut overlay = SurfaceMeshData::default();
        quad(
            &mut overlay,
            [
                [10.0, 18.0, 20.1],
                [14.0, 18.0, 20.1],
                [14.0, 22.0, 20.1],
                [10.0, 22.0, 20.1],
            ],
            true,
        );
        let rays = target_rays(
            CutawayUniform::for_player(Vec3::ZERO, 16.0).unwrap(),
            app.world()
                .get::<Transform>(camera)
                .unwrap()
                .to_matrix(),
        )
        .unwrap();
        let overlay_tree = TriangleTree::new(&overlay);
        assert!(
            !rays.1.iter().any(|&end| overlay_tree.blocks(rays.0, end)),
            "caption must miss the player rays"
        );
        let (_, _, textures, groups) = prepare_linked(overlay, wall(20.0), vec![(0, 0)]);
        assert!(textures.is_empty());
        app.world_mut()
            .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                    let mut commands = world.commands();
                    spawn_groups(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        root,
                        groups,
                        opaque,
                        Some(overlay_handle.clone()),
                    );
                });
            });
        app.world_mut().flush();
        app.update();
        let entities = app
            .world_mut()
            .query_filtered::<Entity, With<OccluderGroup>>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 1);
        let group = entities[0];
        let mesh_count = app.world().resource::<Assets<Mesh>>().len();
        let material_count = app.world().resource::<Assets<VoxelMaterial>>().len();
        assert_eq!(
            material_count, 4,
            "two shared opaque domains plus two cached blended materials"
        );
        // Paired captures hold authoritative simulation while camera/material
        // presentation settles. Exercise that real ECS condition here too.
        app.world_mut().resource_mut::<Time<Virtual>>().pause();
        let frozen_world_time = app.world().resource::<Time<Virtual>>().elapsed();
        for _ in 0..120 {
            app.update();
        }
        assert_eq!(app.world().resource::<Time<Virtual>>().elapsed(), frozen_world_time);
        let state = app.world().get::<OccluderGroup>(group).unwrap();
        assert_eq!(state.fade.opacity, RESIDUAL_OPACITY);
        assert!(state.blending);
        assert_eq!(
            app.world()
                .resource::<Assets<VoxelMaterial>>()
                .get(&state.transparent)
                .unwrap()
                .base
                .alpha_mode,
            AlphaMode::Blend
        );
        assert_eq!(
            app.world()
                .resource::<Assets<VoxelMaterial>>()
                .get(&state.transparent)
                .unwrap()
                .base
                .base_color
                .alpha(),
            1.0,
            "shadow base alpha must never fade"
        );
        let overlay = state.overlay_materials.as_ref().unwrap();
        assert_eq!(
            app.world()
                .resource::<Assets<VoxelMaterial>>()
                .get(&overlay.1)
                .unwrap()
                .extension
                .cutaway
                .fade
                .x,
            1.0 - RESIDUAL_OPACITY,
            "caption must fade with housing even when all rays miss it"
        );
        // A camera turn must select real foreground geometry on this same frame.
        *app.world_mut().get_mut::<Transform>(camera).unwrap() =
            Transform::from_xyz(0.0, 20.0, -80.0).looking_at(Vec3::Y * 10.0, Vec3::Y);
        app.update();
        assert!(!app.world().get::<OccluderGroup>(group).unwrap().blocked);
        for _ in 0..120 {
            app.update();
        }
        let state = app.world().get::<OccluderGroup>(group).unwrap();
        assert_eq!(state.fade.opacity, 1.0);
        assert!(!state.blending);
        for leaf in app.world().get::<Children>(group).unwrap().iter() {
            let expected = if app.world().get::<OverlayLeaf>(leaf).is_some() {
                &overlay_handle
            } else {
                &handle
            };
            assert_eq!(
                &app.world().get::<MeshMaterial3d<VoxelMaterial>>(leaf).unwrap().0,
                expected
            );
        }
        #[derive(Resource, Default)]
        struct GroupChanges(usize);
        app.init_resource::<GroupChanges>().add_systems(
            PostUpdate,
            (|groups: Query<(), Changed<OccluderGroup>>, mut changes: ResMut<GroupChanges>| {
                changes.0 = groups.iter().count();
            }).after(sync),
        );
        // Warm the observer's own change tick, then prove a settled update
        // does not mark the retained blocker or request new material work.
        app.update();
        app.update();
        assert_eq!(app.world().resource::<GroupChanges>().0, 0);
        // Moving the retained terrain root moves its precise blocker surface too.
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation
            .z = -40.0;
        app.update();
        assert!(app.world().get::<OccluderGroup>(group).unwrap().blocked);
        assert_eq!(app.world().resource::<GroupChanges>().0, 1);
        for _ in 0..12 {
            app.world_mut().resource_mut::<VoxelViewStatus>().active = false;
            for _ in 0..120 {
                app.update();
            }
            assert_eq!(
                app.world()
                    .get::<OccluderGroup>(group)
                    .unwrap()
                    .fade
                    .opacity,
                1.0
            );
            app.world_mut().resource_mut::<VoxelViewStatus>().active = true;
            for _ in 0..90 {
                app.update();
            }
            assert_eq!(
                app.world()
                    .get::<OccluderGroup>(group)
                    .unwrap()
                    .fade
                    .opacity,
                RESIDUAL_OPACITY
            );
        }
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(
            app.world().resource::<Assets<VoxelMaterial>>().len(),
            material_count
        );
        app.world_mut().entity_mut(root).despawn();
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&OccluderGroup>()
                .iter(app.world())
                .count(),
            0,
            "map replacement removes all old fade state"
        );
    }
    #[test]
    fn replacing_maps_drops_old_meshes_materials_and_fade_state() {
        use bevy::asset::AssetApp;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<VoxelMaterial>();
        let opaque = app
            .world_mut()
            .resource_mut::<Assets<VoxelMaterial>>()
            .add(crate::solid_terrain_material());
        for _ in 0..16 {
            let root = app.world_mut().spawn((Transform::default(), Visibility::default())).id();
            let (_, groups) = prepare(u_wall());
            app.world_mut()
                .resource_scope(|world, mut meshes: Mut<Assets<Mesh>>| {
                    world.resource_scope(|world, mut materials: Mut<Assets<VoxelMaterial>>| {
                        spawn_groups(
                            &mut world.commands(),
                            &mut meshes,
                            &mut materials,
                            root,
                            groups,
                            opaque.clone(),
                            None,
                        );
                    });
                });
            app.world_mut().flush();
            app.update();
            assert_eq!(app.world().resource::<Assets<VoxelMaterial>>().len(), 2);
            assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 1);
            for mut group in app
                .world_mut()
                .query::<&mut OccluderGroup>()
                .iter_mut(app.world_mut())
            {
                assert_eq!(
                    group.fade.opacity, 1.0,
                    "new terrain cannot inherit old fade state"
                );
                group.fade.opacity = RESIDUAL_OPACITY;
            }
            app.world_mut().entity_mut(root).despawn();
            app.update();
            app.update();
            assert_eq!(
                app.world().resource::<Assets<Mesh>>().len(),
                0,
                "old meshes retained across maps"
            );
            assert_eq!(
                app.world().resource::<Assets<VoxelMaterial>>().len(),
                1,
                "old fade materials retained across maps"
            );
        }
    }
    #[test]
    fn domain_links_are_source_owned_and_preserve_unlinked_neighbors() {
        let solid = wall(20.0);
        let mut textured = wall(20.1);
        quad(
            &mut textured,
            [
                [100.0, 0.0, 20.1],
                [116.0, 0.0, 20.1],
                [116.0, 32.0, 20.1],
                [100.0, 32.0, 20.1],
            ],
            true,
        );
        let expected = corners(&textured);
        let (_, _, textures, solids) =
            prepare_linked(textured, solid, vec![(9999, 0), (0, 0), (0, 4)]);
        assert_eq!(solids.len(), 1);
        assert_eq!(
            textures.len(),
            1,
            "unlinked neighbor must remain independent"
        );
        let overlay = solids.into_iter().next().unwrap().overlay.unwrap();
        let mut actual = BTreeMap::new();
        for group in textures.into_iter().chain([*overlay]) {
            for leaf in group.leaves {
                for (key, count) in corners(&restore_leaf(leaf)) {
                    *actual.entry(key).or_default() += count;
                }
            }
        }
        assert_eq!(
            actual, expected,
            "linking must neither drop nor duplicate source triangles"
        );
    }
}

#[cfg(test)]
pub(crate) fn geometry_signature(
    opaque: &SurfaceMeshData, groups: Vec<PreparedGroup>,
) -> Vec<Vec<u32>> {
    let mut result = Vec::new();
    let mut add = |data: &SurfaceMeshData| {
        for tri in data.indices.chunks_exact(3) {
            let mut bits = Vec::new();
            for &i in tri {
                let i = i as usize;
                bits.extend(data.positions[i].map(f32::to_bits));
                bits.extend(data.normals[i].map(f32::to_bits));
                bits.extend(data.uvs[i].map(f32::to_bits));
                bits.extend(data.colors[i].map(f32::to_bits));
            }
            result.push(bits);
        }
    };
    add(opaque);
    for group in groups {
        for leaf in group.leaves {
            // Shared instances keep their canonical CPU mesh in the shared
            // leaf until upload; leaf.mesh is intentionally an empty placeholder.
            let shared = leaf.shared.as_ref().map(|s| s.lock().unwrap());
            let mesh = shared.as_ref().map_or(&leaf.mesh, |s|
                s.mesh.as_ref().expect("geometry signature requires an unuploaded CPU mesh"));
            let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else { panic!() };
            let bevy::mesh::VertexAttributeValues::Float32x3(normals) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() else { panic!() };
            // A compact palette leaf has implicit zero coordinates. Tagged
            // finishes retain their attribute and must still match exactly.
            let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) => uvs.clone(),
                None => vec![[0.0; 2]; positions.len()],
                _ => panic!("unexpected UV0 representation"),
            };
            let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() {
                bevy::mesh::VertexAttributeValues::Float32x4(colors) => colors.clone(),
                bevy::mesh::VertexAttributeValues::Float32x3(colors) => colors.iter()
                    .map(|&[r, g, b]| [r, g, b, 1.0]).collect(),
                _ => panic!("unexpected color representation"),
            };
            let data = SurfaceMeshData {
                positions: positions.iter().map(|&p| group.transform.transform_point(
                    Vec3::from_array(p) + leaf.center).to_array()).collect(),
                normals: if group.transform.rotation == Quat::IDENTITY
                    && group.transform.scale.x > 0.
                    && group.transform.scale.x == group.transform.scale.y
                    && group.transform.scale.x == group.transform.scale.z {
                    normals.clone()
                } else {
                    normals.iter().map(|&n| (group.transform.rotation
                        * (Vec3::from_array(n) / group.transform.scale)).normalize().to_array()).collect()
                }, uvs, colors,
                indices: mesh.indices().unwrap().iter().map(|i| i as u32).collect(),
                ..Default::default()
            };
            add(&data);
        }
    }
    result.sort_unstable();
    result
}
