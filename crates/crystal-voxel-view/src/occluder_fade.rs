//! Whole authored occluders fade without cutting holes or rebuilding meshes.
//! Source ranges own fade state. A triangle BVH tests real wall surfaces, never
//! the empty interior of a concave object's bounding box. Render leaves merely
//! improve transparent sorting; every leaf of an object has identical opacity.
use crate::{
    TerrainRevisionCache, VOXEL_RENDER_LAYER, VoxelMaterial, VoxelViewStatus, VoxelWorldCamera,
    interior_cutaway::CutawayUniform, mesh::SurfaceMeshData,
};
use bevy::{
    prelude::*,
    render::{primitives::Aabb, view::RenderLayers},
};
use std::ops::Range;

const RESIDUAL_OPACITY: f32 = 0.20;
const EXIT_HOLD_SECONDS: f32 = 0.12;
const FADE_RATE: f32 = 14.0;
const RESTORE_RATE: f32 = 8.0;
const BVH_LEAF_TRIANGLES: usize = 12;
const MAX_RENDER_LEAVES: usize = 16;
const RENDER_LEAF_TRIANGLES: usize = 2048;

pub(super) fn register(app: &mut App) {
    app.add_systems(
        PostUpdate,
        sync.after(bevy::render::camera::CameraUpdateSystem)
            .after(bevy::transform::TransformSystem::TransformPropagate)
            .before(bevy::render::view::VisibilitySystems::VisibilityPropagate),
    );
}

pub(super) struct PreparedGroup {
    source_range: Range<usize>,
    surface: TriangleTree,
    leaves: Vec<PreparedLeaf>,
    overlay: Option<Box<PreparedGroup>>,
}
struct PreparedLeaf {
    mesh: Mesh,
    center: Vec3,
    bounds: Aabb,
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
    children: Option<[usize; 2]>,
}
#[derive(Clone, Debug, Default)]
struct TriangleTree {
    triangles: Vec<[Vec3; 3]>,
    nodes: Vec<Node>,
}
impl TriangleTree {
    fn new(data: &SurfaceMeshData) -> Self {
        let mut tree = Self {
            triangles: data
                .indices
                .chunks_exact(3)
                .map(|t| [t[0], t[1], t[2]].map(|i| Vec3::from_array(data.positions[i as usize])))
                .collect(),
            nodes: Vec::new(),
        };
        if !tree.triangles.is_empty() {
            tree.build(0..tree.triangles.len());
        }
        tree
    }
    fn build(&mut self, range: Range<usize>) -> usize {
        let mut bounds = Bounds::empty();
        for triangle in &self.triangles[range.clone()] {
            for &p in triangle {
                bounds.include(p);
            }
        }
        let node = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            triangles: range.clone(),
            children: None,
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
            self.triangles[range.clone()].sort_unstable_by(|a, b| {
                let center = |t: &[Vec3; 3]| {
                    f64::from(t[0][axis]) + f64::from(t[1][axis]) + f64::from(t[2][axis])
                };
                center(a).total_cmp(&center(b))
            });
            let mid = range.start + range.len() / 2;
            let left = self.build(range.start..mid);
            let right = self.build(mid..range.end);
            self.nodes[node].children = Some([left, right]);
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
        if let Some([a, b]) = node.children {
            self.hit_node(a, start, delta) || self.hit_node(b, start, delta)
        } else {
            self.triangles[node.triangles.clone()]
                .iter()
                .any(|&t| segment_triangle(start, delta, t))
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
    let mut owner = vec![None; n];
    for (group, range) in ranges.iter().enumerate() {
        owner[range.clone()].fill(Some(group));
    }
    let mut buckets = vec![Vec::<u32>::new(); ranges.len() + 1];
    for triangle in source.indices.chunks_exact(3) {
        let group = owner[triangle[0] as usize];
        let index = if group.is_some() && triangle.iter().all(|&i| owner[i as usize] == group) {
            group.unwrap() + 1
        } else {
            0
        };
        buckets[index].extend_from_slice(triangle);
    }
    let mut remap = vec![u32::MAX; n];
    let mut extracted = buckets
        .into_iter()
        .map(|indices| select(&source, &owner, &mut remap, indices));
    let opaque = extracted.next().unwrap();
    let groups = extracted
        .zip(ranges)
        .filter(|(data, _)| !data.indices.is_empty())
        .map(|(data, source_range)| {
            let surface = TriangleTree::new(&data);
            let leaves = render_leaves(data);
            PreparedGroup {
                source_range,
                surface,
                leaves,
                overlay: None,
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
    let (textured, mut textured_groups) = prepare(textured);
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
    owner: &[Option<usize>],
    remap: &mut [u32],
    indices: Vec<u32>,
) -> SurfaceMeshData {
    let mut data = SurfaceMeshData::default();
    let mut touched = Vec::new();
    for i in indices {
        let old = i as usize;
        if remap[old] == u32::MAX {
            let next = data.positions.len();
            remap[old] = next as u32;
            touched.push(old);
            data.positions.push(source.positions[old]);
            data.normals.push(source.normals[old]);
            data.uvs.push(source.uvs[old]);
            data.colors.push(source.colors[old]);
            if owner[old].is_some() {
                if let Some(range) = data.cutaway_ranges.last_mut()
                    && range.end == next
                {
                    range.end += 1;
                } else {
                    data.cutaway_ranges.push(next..next + 1);
                }
            }
        }
        data.indices.push(remap[old]);
    }
    for old in touched {
        remap[old] = u32::MAX;
    }
    data
}
fn render_leaves(data: SurfaceMeshData) -> Vec<PreparedLeaf> {
    let mut chunks = vec![data.indices.clone()];
    while chunks.len() < MAX_RENDER_LEAVES {
        let candidate = chunks
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                if c.len() / 3 > RENDER_LEAF_TRIANGLES {
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
        let indices = chunks.swap_remove(index);
        let mut bounds = Bounds::empty();
        for &i in &indices {
            bounds.include(data.positions[i as usize].into());
        }
        let extent = bounds.max - bounds.min;
        let axis = if extent.x >= extent.z { 0 } else { 2 };
        let mut triangles = indices
            .chunks_exact(3)
            .map(|t| [t[0], t[1], t[2]])
            .collect::<Vec<_>>();
        triangles.sort_unstable_by(|a, b| {
            let center = |t: &[u32; 3]| {
                t.iter()
                    .map(|&i| f64::from(data.positions[i as usize][axis]))
                    .sum::<f64>()
            };
            center(a).total_cmp(&center(b))
        });
        let right = triangles.split_off(triangles.len() / 2);
        chunks.push(triangles.into_iter().flatten().collect());
        chunks.push(right.into_iter().flatten().collect());
    }
    let owner = vec![Some(0); data.positions.len()];
    let mut remap = vec![u32::MAX; data.positions.len()];
    chunks
        .into_iter()
        .map(|indices| {
            let mut leaf = select(&data, &owner, &mut remap, indices);
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
                mesh: leaf.into_mesh(),
                center,
                bounds: local_bounds,
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
struct Fade {
    opacity: f32,
    hold: f32,
}
impl Default for Fade {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            hold: 0.0,
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
            self.opacity = ease(self.opacity, RESIDUAL_OPACITY, FADE_RATE, dt);
        } else {
            let held = dt.min(self.hold);
            self.hold = (self.hold - dt).max(0.0);
            self.opacity = ease(self.opacity, RESIDUAL_OPACITY, FADE_RATE, held);
            self.opacity = ease(self.opacity, 1.0, RESTORE_RATE, dt - held);
        }
        if (self.opacity - 1.0).abs() < 0.0001 {
            self.opacity = 1.0;
        }
        if (self.opacity - RESIDUAL_OPACITY).abs() < 0.0001 {
            self.opacity = RESIDUAL_OPACITY;
        }
    }
}
#[derive(Component)]
struct OccluderGroup {
    surface: TriangleTree,
    overlay_surface: Option<TriangleTree>,
    overlay_materials: Option<(Handle<VoxelMaterial>, Handle<VoxelMaterial>)>,
    fade: Fade,
    blocked: bool,
    opaque: Handle<VoxelMaterial>,
    transparent: Handle<VoxelMaterial>,
    blending: bool,
}

#[derive(Component)]
struct OverlayLeaf;

pub(super) fn spawn_groups(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
    root: Entity,
    groups: Vec<PreparedGroup>,
    opaque: Handle<VoxelMaterial>,
    linked_opaque: Option<Handle<VoxelMaterial>>,
) {
    fn blended(
        materials: &mut Assets<VoxelMaterial>,
        opaque: &Handle<VoxelMaterial>,
    ) -> Handle<VoxelMaterial> {
        let mut material = materials
            .get(opaque)
            .expect("terrain material validated before group upload")
            .clone();
        // The stock shadow shader retains the original base alpha and geometry.
        material.base.alpha_mode = AlphaMode::Blend;
        material.extension.cutaway = CutawayUniform::default();
        materials.add(material)
    }
    for PreparedGroup {
        source_range: _,
        surface,
        leaves,
        overlay,
    } in groups
    {
        let transparent = blended(materials, &opaque);
        let (overlay_surface, overlay_leaves) = overlay
            .map(|g| (Some(g.surface), g.leaves))
            .unwrap_or_default();
        let overlay_materials = overlay_surface.as_ref().map(|_| {
            let opaque = linked_opaque
                .as_ref()
                .expect("authored cross-domain link requires its material")
                .clone();
            let transparent = blended(materials, &opaque);
            (opaque, transparent)
        });
        let group = commands
            .spawn((
                SpatialBundle::default(),
                OccluderGroup {
                    surface,
                    overlay_surface,
                    overlay_materials: overlay_materials.clone(),
                    fade: Fade::default(),
                    blocked: false,
                    opaque: opaque.clone(),
                    transparent,
                    blending: false,
                },
            ))
            .id();
        for (domain, leaves) in [leaves, overlay_leaves].into_iter().enumerate() {
            if leaves.is_empty() {
                continue;
            }
            let material = if domain == 0 {
                &opaque
            } else {
                &overlay_materials.as_ref().unwrap().0
            };
            for PreparedLeaf {
                mesh,
                center,
                bounds,
            } in leaves
            {
                let mut leaf = commands.spawn((
                    MaterialMeshBundle::<VoxelMaterial> {
                        mesh: meshes.add(mesh),
                        material: material.clone(),
                        transform: Transform::from_translation(center),
                        ..default()
                    },
                    bounds,
                    RenderLayers::layer(VOXEL_RENDER_LAYER),
                ));
                if domain == 1 {
                    leaf.insert(OverlayLeaf);
                }
                let leaf = leaf.id();
                commands.entity(group).add_child(leaf);
            }
        }
        commands.entity(root).add_child(group);
    }
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
fn sync(
    status: Res<VoxelViewStatus>,
    cache: Res<TerrainRevisionCache>,
    time: Res<Time>,
    cameras: Query<(&Camera, &GlobalTransform), With<VoxelWorldCamera>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut groups: Query<(&mut OccluderGroup, Ref<GlobalTransform>, &Children)>,
    mut leaves: Query<(&mut Handle<VoxelMaterial>, Option<&OverlayLeaf>)>,
    mut previous: Local<Option<(Mat4, CutawayUniform, bool)>>,
) {
    let (camera, active) = cameras
        .get_single()
        .map(|(c, t)| (t.compute_matrix(), c.is_active))
        .unwrap_or((Mat4::IDENTITY, false));
    let uniform = cache
        .solid_material
        .as_ref()
        .and_then(|h| materials.get(h))
        .map(|m| m.extension.cutaway)
        .unwrap_or_default();
    let input = (camera, uniform, status.active && active);
    let changed = previous.as_ref() != Some(&input);
    *previous = Some(input);
    let rays = if input.2 {
        target_rays(uniform, camera)
    } else {
        None
    };
    for (mut group, transform, children) in &mut groups {
        if changed || transform.is_changed() || group.is_added() {
            group.blocked = rays.is_some_and(|(eye, targets)| {
                let local = transform.compute_matrix().inverse();
                local.is_finite()
                    && targets.into_iter().any(|target| {
                        let start = local.transform_point3(eye);
                        let end = local.transform_point3(target);
                        group.surface.blocks(start, end)
                            || group
                                .overlay_surface
                                .as_ref()
                                .is_some_and(|surface| surface.blocks(start, end))
                    })
            });
        }
        let previous_alpha = group.fade.opacity;
        let blocked = group.blocked;
        group.fade.advance(blocked, time.delta_seconds());
        let blending = group.fade.opacity < 1.0;
        if blending != group.blending {
            let material = if blending {
                group.transparent.clone()
            } else {
                group.opaque.clone()
            };
            for &child in children.iter() {
                if let Ok((mut handle, overlay)) = leaves.get_mut(child) {
                    *handle = if overlay.is_some() {
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
        if group.fade.opacity != previous_alpha {
            if let Some(material) = materials.get_mut(&group.transparent) {
                material.extension.cutaway.fade.x = 1.0 - group.fade.opacity;
            }
            if let Some((_, transparent)) = &group.overlay_materials {
                if let Some(material) = materials.get_mut(transparent) {
                    material.extension.cutaway.fade.x = 1.0 - group.fade.opacity;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::{Indices, VertexAttributeValues};
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
    fn bvh_matches_actual_triangles_across_orbit_zoom_and_translated_roots() {
        let mut data = u_wall();
        // More than one BVH leaf, with disjoint scenery and an empty interior.
        for x in -8..=8 {
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
                        .compute_matrix();
                    let uniform = CutawayUniform::for_player(Vec3::ZERO, 16.0).unwrap();
                    let (origin, targets) = target_rays(uniform, camera).unwrap();
                    for target in targets {
                        let expected = tree
                            .triangles
                            .iter()
                            .any(|&t| segment_triangle(origin, target - origin, t));
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
        let VertexAttributeValues::Float32x4(colors) =
            leaf.mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!()
        };
        data.colors = colors.clone();
        let Indices::U32(indices) = leaf.mesh.indices().unwrap() else {
            panic!()
        };
        data.indices = indices.clone();
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
            .add(crate::textured_terrain_material(Handle::weak_from_u128(42)));
        let camera = app
            .world_mut()
            .spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 20.0, 80.0)
                        .looking_at(Vec3::Y * 10.0, Vec3::Y),
                    ..default()
                },
                VoxelWorldCamera,
            ))
            .id();
        let root = app.world_mut().spawn(SpatialBundle::default()).id();
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
                .compute_matrix(),
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
        for _ in 0..120 {
            app.update();
        }
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
        for &leaf in app.world().get::<Children>(group).unwrap().iter() {
            let expected = if app.world().get::<OverlayLeaf>(leaf).is_some() {
                &overlay_handle
            } else {
                &handle
            };
            assert_eq!(
                app.world().get::<Handle<VoxelMaterial>>(leaf).unwrap(),
                expected
            );
        }
        // Moving the retained terrain root moves its precise blocker surface too.
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation
            .z = -40.0;
        app.update();
        assert!(app.world().get::<OccluderGroup>(group).unwrap().blocked);
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
        app.world_mut().entity_mut(root).despawn_recursive();
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
            let root = app.world_mut().spawn(SpatialBundle::default()).id();
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
            app.world_mut().entity_mut(root).despawn_recursive();
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
