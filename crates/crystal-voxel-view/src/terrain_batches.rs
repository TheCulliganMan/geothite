//! Exact static geometry in bounded spatial draws. Bevy culls each draw against
//! each camera and shadow frustum; no camera/player-dependent geometry is built.
use crate::mesh::SurfaceMeshData;
use bevy::{prelude::*, render::primitives::Aabb};

/// Authoritative source-map rectangle in the mesh's local X/Z coordinates.
/// This is independent of the generated terrain's (possibly padded) extent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AuthenticBounds {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum AuthenticDomain {
    /// No valid authoritative rectangle was supplied. Bounds alone cannot
    /// establish that this geometry belongs to the actual source map.
    #[default]
    Unspecified,
    Inside,
    /// Proven to contain no positive-area source-map surface. Boundary edges
    /// may be shared with Inside, but no triangle surface is duplicated.
    Outside,
    /// A crossing triangle could not be split without changing its semantics.
    /// Preserve it in the overworld and keep it as battle rejection evidence.
    Unresolved,
}

pub(super) struct PreparedBatch {
    pub mesh: Mesh,
    pub bounds: Option<Aabb>,
    pub authentic_domain: AuthenticDomain,
}

impl From<Mesh> for PreparedBatch {
    fn from(mesh: Mesh) -> Self {
        // Compute on the terrain worker, before render extraction moves the
        // CPU buffers. Never scan full vertex buffers when applying a build.
        let bounds = mesh.compute_aabb();
        Self {
            mesh,
            bounds,
            authentic_domain: AuthenticDomain::Unspecified,
        }
    }
}

const CELLS_PER_BATCH: f32 = 16.0;
const MAX_BATCHES_PER_AXIS: usize = 8;
const MIN_TRIANGLES: usize = 4096;

pub(super) fn prepare(source: SurfaceMeshData, tile_size: Vec2) -> Vec<PreparedBatch> {
    prepare_with_authentic_bounds(source, tile_size, None)
}

/// Split only source-map boundary crossings, then spatially batch each domain
/// separately. All three domains still render in the overworld; battle scenery
/// can omit proven Outside batches without mistaking Unresolved for padding.
/// At most three domains, each with at most 8 x 8 spatial batches, are produced.
/// CPU geometry still moves into render-only assets through into_mesh().
pub(super) fn prepare_with_authentic_bounds(
    source: SurfaceMeshData,
    tile_size: Vec2,
    authentic_bounds: Option<AuthenticBounds>,
) -> Vec<PreparedBatch> {
    // A native-only control for paired measurements of this same renderer.
    // Authentic-domain separation remains necessary even when disabled.
    #[cfg(target_arch = "wasm32")]
    let spatial_batching = true;
    #[cfg(not(target_arch = "wasm32"))]
    let spatial_batching = {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("CRYSTAL_TERRAIN_BATCHING").as_deref() != Ok("off"))
    };
    separate_authentic_domains(source, authentic_bounds)
        .into_iter()
        .flat_map(|(domain, data)| {
            let batches = if spatial_batching {
                partition(data, tile_size)
            } else {
                vec![data]
            };
            batches.into_iter().map(move |data| {
                let mut batch = PreparedBatch::from(data.into_mesh());
                batch.authentic_domain = domain;
                batch
            })
        })
        .collect()
}

fn valid_topology(source: &SurfaceMeshData) -> bool {
    let n = source.positions.len();
    source.indices.len() % 3 == 0
        && source.normals.len() == n
        && source.uvs.len() == n
        && source.colors.len() == n
        && source.indices.iter().all(|&i| (i as usize) < n)
        && source
            .cutaway_ranges
            .iter()
            .all(|r| r.start <= r.end && r.end <= n)
        && source.positions.iter().flatten().all(|v| v.is_finite())
}

/// A clipping corner retains original attributes verbatim until an edge is
/// actually cut. New positions/UVs/colors interpolate in f64 and round to f32.
/// Supported flat normals keep their bits; differing normals cannot be split
/// faithfully because Bevy normalizes each corner in the vertex shader.
#[derive(Clone, Copy, Debug)]
struct ClipCorner {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
}

impl ClipCorner {
    fn from_source(source: &SurfaceMeshData, i: usize) -> Self {
        Self {
            position: source.positions[i],
            normal: source.normals[i],
            uv: source.uvs[i],
            color: source.colors[i],
        }
    }

    fn finite(self) -> bool {
        self.position
            .iter()
            .chain(&self.normal)
            .chain(&self.uv)
            .chain(&self.color)
            .all(|v| v.is_finite())
    }

    fn intersect(self, other: Self, axis: usize, value: f32) -> Self {
        if self.position[axis] == value {
            return self;
        }
        if other.position[axis] == value {
            return other;
        }
        // Canonical endpoint order and f64 arithmetic make shared-edge cuts
        // identical even when adjacent triangles traverse the edge oppositely.
        let (a, b) = if self.position[axis] < other.position[axis] {
            (self, other)
        } else {
            (other, self)
        };
        let t = (f64::from(value) - f64::from(a.position[axis]))
            / (f64::from(b.position[axis]) - f64::from(a.position[axis]));
        fn mix<const N: usize>(a: [f32; N], b: [f32; N], t: f64) -> [f32; N] {
            std::array::from_fn(|i| {
                if a[i].to_bits() == b[i].to_bits() {
                    a[i]
                } else {
                    (f64::from(a[i]) * (1.0 - t) + f64::from(b[i]) * t) as f32
                }
            })
        }
        let mut corner = Self {
            position: mix(a.position, b.position, t),
            normal: mix(a.normal, b.normal, t),
            uv: mix(a.uv, b.uv, t),
            color: mix(a.color, b.color, t),
        };
        // Plane membership is exact, including non-integral source bounds.
        corner.position[axis] = value;
        corner
    }
}

fn signed_plane_distance(p: [f32; 3], plane: (usize, f32, bool)) -> f64 {
    let (axis, value, greater) = plane;
    let d = f64::from(p[axis]) - f64::from(value);
    if greater { d } else { -d }
}

fn split_polygon(
    polygon: Vec<ClipCorner>,
    plane: (usize, f32, bool),
) -> (Vec<ClipCorner>, Vec<ClipCorner>) {
    if polygon
        .iter()
        .all(|p| signed_plane_distance(p.position, plane) >= 0.0)
    {
        return (polygon, Vec::new());
    }
    if polygon
        .iter()
        .all(|p| signed_plane_distance(p.position, plane) <= 0.0)
    {
        return (Vec::new(), polygon);
    }
    let mut inside = Vec::with_capacity(polygon.len() + 1);
    let mut outside = Vec::with_capacity(polygon.len() + 1);
    for i in 0..polygon.len() {
        let a = polygon[i];
        let b = polygon[(i + 1) % polygon.len()];
        let da = signed_plane_distance(a.position, plane);
        let db = signed_plane_distance(b.position, plane);
        if da >= 0.0 {
            inside.push(a);
        }
        if da <= 0.0 {
            outside.push(a);
        }
        if (da < 0.0 && db > 0.0) || (da > 0.0 && db < 0.0) {
            let cut = a.intersect(b, plane.0, plane.1);
            inside.push(cut);
            outside.push(cut);
        }
    }
    (inside, outside)
}

fn mark_vertex(data: &mut SurfaceMeshData, next: usize) {
    if let Some(range) = data.cutaway_ranges.last_mut()
        && range.end == next
    {
        range.end += 1;
    } else {
        data.cutaway_ranges.push(next..next + 1);
    }
}

fn append_polygon(data: &mut SurfaceMeshData, polygon: &[ClipCorner], marked: bool) {
    if polygon.len() < 3 {
        return;
    }
    let base = data.positions.len() as u32;
    for corner in polygon {
        let next = data.positions.len();
        data.positions.push(corner.position);
        data.normals.push(corner.normal);
        data.uvs.push(corner.uv);
        data.colors.push(corner.color);
        if marked {
            mark_vertex(data, next);
        }
    }
    // A convex clipped polygon keeps the source triangle's winding. Adjacent
    // polygon edges overlap only on their zero-area shared boundary.
    for i in 1..polygon.len() as u32 - 1 {
        data.indices.extend([base, base + i, base + i + 1]);
    }
}

fn separate_authentic_domains(
    source: SurfaceMeshData,
    bounds: Option<AuthenticBounds>,
) -> Vec<(AuthenticDomain, SurfaceMeshData)> {
    if source.indices.is_empty() {
        return Vec::new();
    }
    let Some(bounds) =
        bounds.filter(|b| b.min.is_finite() && b.max.is_finite() && b.min.cmplt(b.max).all())
    else {
        return vec![(AuthenticDomain::Unspecified, source)];
    };
    if !valid_topology(&source) {
        return vec![(AuthenticDomain::Unspecified, source)];
    }
    let planes = [
        (0, bounds.min.x, true),
        (0, bounds.max.x, false),
        (2, bounds.min.y, true),
        (2, bounds.max.y, false),
    ];
    let mut marked = vec![false; source.positions.len()];
    for range in &source.cutaway_ranges {
        marked[range.clone()].fill(true);
    }
    // Original indexed triangles retain their sharing within each domain.
    // Only crossing triangles acquire new interpolated vertices.
    let mut indices: [Vec<u32>; 3] = std::array::from_fn(|_| Vec::new());
    let mut clipped: [SurfaceMeshData; 3] = std::array::from_fn(|_| default());
    for triangle in source.indices.chunks_exact(3) {
        let corners = [triangle[0], triangle[1], triangle[2]]
            .map(|i| ClipCorner::from_source(&source, i as usize));
        if planes.iter().all(|&plane| {
            corners
                .iter()
                .all(|p| signed_plane_distance(p.position, plane) >= 0.0)
        }) {
            indices[0].extend_from_slice(triangle);
        } else if planes.iter().any(|&plane| {
            corners
                .iter()
                .all(|p| signed_plane_distance(p.position, plane) <= 0.0)
                && corners
                    .iter()
                    .any(|p| signed_plane_distance(p.position, plane) < 0.0)
        }) {
            indices[1].extend_from_slice(triangle);
        } else if !corners.iter().all(|c| c.finite())
            || corners
                .iter()
                .any(|c| c.normal.map(f32::to_bits) != corners[0].normal.map(f32::to_bits))
            || triangle
                .iter()
                .any(|&i| marked[i as usize] != marked[triangle[0] as usize])
        {
            // Boolean eligibility has no fractional representation. Differing
            // normals undergo nonlinear vertex-shader normalization. Preserve
            // these triangles whole, and never sanitize an invalid attribute.
            indices[2].extend_from_slice(triangle);
        } else {
            let cutaway = marked[triangle[0] as usize];
            let mut remaining = corners.to_vec();
            for plane in planes {
                let (inside, outside) = split_polygon(remaining, plane);
                append_polygon(&mut clipped[1], &outside, cutaway);
                remaining = inside;
                if remaining.is_empty() {
                    break;
                }
            }
            append_polygon(&mut clipped[0], &remaining, cutaway);
        }
    }
    let mut remap = vec![u32::MAX; source.positions.len()];
    let mut touched = Vec::new();
    let mut result = Vec::with_capacity(3);
    for ((domain, indices), mut data) in [
        AuthenticDomain::Inside,
        AuthenticDomain::Outside,
        AuthenticDomain::Unresolved,
    ]
    .into_iter()
    .zip(indices)
    .zip(clipped)
    {
        for index in indices {
            let old = index as usize;
            if remap[old] == u32::MAX {
                let next = data.positions.len();
                remap[old] = next as u32;
                touched.push(old);
                data.positions.push(source.positions[old]);
                data.normals.push(source.normals[old]);
                data.uvs.push(source.uvs[old]);
                data.colors.push(source.colors[old]);
                if marked[old] {
                    mark_vertex(&mut data, next);
                }
            }
            data.indices.push(remap[old]);
        }
        for old in touched.drain(..) {
            remap[old] = u32::MAX;
        }
        if !data.indices.is_empty() {
            result.push((domain, data));
        }
    }
    result
}

/// Preserve complete indexed triangles, their corner attributes and cutaway
/// eligibility. A long triangle is assigned once and its actual full bounds
/// participate in culling, including portions outside the nominal grid cell.
fn partition(source: SurfaceMeshData, tile_size: Vec2) -> Vec<SurfaceMeshData> {
    if source.indices.is_empty() {
        return Vec::new();
    }
    let vertex_count = source.positions.len();
    if source.indices.len() / 3 < MIN_TRIANGLES
        || source.indices.len() % 3 != 0
        || source.normals.len() != vertex_count
        || source.uvs.len() != vertex_count
        || source.colors.len() != vertex_count
        || source.indices.iter().any(|&i| i as usize >= vertex_count)
        || source
            .cutaway_ranges
            .iter()
            .any(|r| r.start > r.end || r.end > vertex_count)
        || !tile_size.is_finite()
        || tile_size.cmple(Vec2::ZERO).any()
    {
        return vec![source];
    }
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for &position in &source.positions {
        let p = Vec3::from_array(position);
        if !p.is_finite() {
            return vec![source];
        }
        let p = Vec2::new(p.x, p.z);
        min = min.min(p);
        max = max.max(p);
    }
    let extent = max - min;
    if !extent.is_finite() {
        return vec![source];
    }
    let width = (tile_size * CELLS_PER_BATCH).max(extent / MAX_BATCHES_PER_AXIS as f32);
    if !width.is_finite() || width.cmple(Vec2::ZERO).any() {
        return vec![source];
    }
    let count = (extent / width)
        .ceil()
        .as_uvec2()
        .clamp(UVec2::ONE, UVec2::splat(MAX_BATCHES_PER_AXIS as u32));
    if count == UVec2::ONE {
        return vec![source];
    }
    let batch_count = (count.x * count.y) as usize;
    let mut buckets = vec![Vec::<u32>::new(); batch_count];
    for triangle in source.indices.chunks_exact(3) {
        // f64 protects bucket selection for valid extreme f32 coordinates.
        // Stored geometry is copied bit-for-bit; it is never rebased or rounded.
        let center = |axis: usize| {
            triangle
                .iter()
                .map(|&i| f64::from(source.positions[i as usize][axis]))
                .sum::<f64>()
                / 3.0
        };
        let x = (((center(0) - f64::from(min.x)) / f64::from(width.x)).floor() as usize)
            .min(count.x as usize - 1);
        let z = (((center(2) - f64::from(min.y)) / f64::from(width.y)).floor() as usize)
            .min(count.y as usize - 1);
        buckets[z * count.x as usize + x].extend_from_slice(triangle);
    }
    // One shared dense remap, reset only at touched vertices. No per-batch
    // full-grid scans or attribute hashing, and no work on retained frames.
    let mut marked = vec![false; vertex_count];
    for range in &source.cutaway_ranges {
        marked[range.clone()].fill(true);
    }
    let mut remap = vec![u32::MAX; vertex_count];
    let mut touched = Vec::new();
    let mut result = Vec::with_capacity(batch_count);
    for indices in buckets.into_iter().filter(|indices| !indices.is_empty()) {
        let mut batch = SurfaceMeshData {
            indices: Vec::with_capacity(indices.len()),
            ..default()
        };
        for index in indices {
            let old = index as usize;
            if remap[old] == u32::MAX {
                let next = batch.positions.len();
                remap[old] = next as u32;
                touched.push(old);
                batch.positions.push(source.positions[old]);
                batch.normals.push(source.normals[old]);
                batch.uvs.push(source.uvs[old]);
                batch.colors.push(source.colors[old]);
                if marked[old] {
                    if let Some(range) = batch.cutaway_ranges.last_mut()
                        && range.end == next
                    {
                        range.end += 1;
                    } else {
                        batch.cutaway_ranges.push(next..next + 1);
                    }
                }
            }
            batch.indices.push(remap[old]);
        }
        for old in touched.drain(..) {
            remap[old] = u32::MAX;
        }
        result.push(batch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::{camera::CameraProjection, primitives::Sphere};
    use std::collections::BTreeMap;

    // Deliberately indexed, asymmetric and non-flat, with overlapping cutaway
    // ranges. Every vertex carries distinct UV/color/normal bits.
    fn fixture(side: usize) -> SurfaceMeshData {
        let mut data = SurfaceMeshData::default();
        for z in 0..side {
            for x in 0..side {
                let base = data.positions.len() as u32;
                for [dx, dz] in [[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]] {
                    let n = data.positions.len() as f32;
                    data.positions
                        .push([x as f32 * 8.0 + dx, (x % 9) as f32, z as f32 * 8.0 + dz]);
                    data.normals.push([n * 0.0001, 1.0, -0.0]);
                    data.uvs.push([n * 0.00001, -n * 0.0001]);
                    data.colors.push([0.2, n * 0.00001, 0.6, 1.0]);
                }
                data.indices
                    .extend([base, base + 2, base + 1, base, base + 3, base + 2]);
                if (x + z) % 3 == 0 {
                    data.cutaway_ranges.push(base as usize..base as usize + 3);
                    data.cutaway_ranges
                        .push(base as usize + 1..base as usize + 4);
                }
            }
        }
        data
    }

    fn triangles(data: &SurfaceMeshData) -> BTreeMap<Vec<u32>, usize> {
        let mut counts = BTreeMap::new();
        let mut marked = vec![false; data.positions.len()];
        for range in &data.cutaway_ranges {
            marked[range.clone()].fill(true);
        }
        for triangle in data.indices.chunks_exact(3) {
            let mut bits = Vec::new();
            for &index in triangle {
                let i = index as usize;
                bits.extend(data.positions[i].map(f32::to_bits));
                bits.extend(data.normals[i].map(f32::to_bits));
                bits.extend(data.uvs[i].map(f32::to_bits));
                bits.extend(data.colors[i].map(f32::to_bits));
                bits.push(u32::from(marked[i]));
            }
            *counts.entry(bits).or_insert(0) += 1;
        }
        counts
    }

    fn bounds() -> AuthenticBounds {
        AuthenticBounds {
            min: Vec2::ZERO,
            max: Vec2::new(4.0, 3.0),
        }
    }

    fn append_triangle(data: &mut SurfaceMeshData, positions: [[f32; 3]; 3], marked: bool) {
        let base = data.positions.len() as u32;
        for [x, y, z] in positions {
            data.positions.push([x, y, z]);
            data.normals.push([x / 32.0, 1.0 + z / 64.0, -0.0]);
            data.uvs.push([x / 16.0, z / 16.0]);
            data.colors
                .push([0.5 + x / 64.0, 0.5 + z / 64.0, 0.25, 1.0]);
        }
        data.indices.extend([base, base + 1, base + 2]);
        if marked {
            data.cutaway_ranges.push(base as usize..base as usize + 3);
        }
    }

    fn large_plane() -> SurfaceMeshData {
        let mut data = SurfaceMeshData::default();
        let [a, b, c, d] = [[-8.0, -10.0], [12.0, -10.0], [12.0, 14.0], [-8.0, 14.0]]
            .map(|[x, z]| [x, x / 8.0 + z / 4.0, z]);
        append_triangle(&mut data, [a, c, b], true);
        append_triangle(&mut data, [a, d, c], true);
        data.normals.fill([0.125, 1.0, -0.0]);
        data
    }

    fn projected_signed_area(data: &SurfaceMeshData) -> f64 {
        data.indices
            .chunks_exact(3)
            .map(|t| {
                let [a, b, c] =
                    [t[0], t[1], t[2]].map(|i| data.positions[i as usize].map(f64::from));
                ((b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0])) * 0.5
            })
            .sum()
    }

    fn count_strict_coverage(data: &SurfaceMeshData, p: [f64; 2]) -> usize {
        data.indices
            .chunks_exact(3)
            .filter(|t| {
                let vertices =
                    [t[0], t[1], t[2]].map(|i| data.positions[i as usize].map(f64::from));
                let sides = std::array::from_fn::<_, 3, _>(|i| {
                    let a = vertices[i];
                    let b = vertices[(i + 1) % 3];
                    (b[0] - a[0]) * (p[1] - a[2]) - (b[2] - a[2]) * (p[0] - a[0])
                });
                sides.iter().all(|s| *s > 1.0e-9) || sides.iter().all(|s| *s < -1.0e-9)
            })
            .count()
    }

    #[test]
    fn authentic_domains_preserve_uncut_bits_and_own_boundary_surfaces_once() {
        let mut source = SurfaceMeshData::default();
        append_triangle(
            &mut source,
            [[1.0, 0.0, 1.0], [2.0, 0.0, 1.0], [1.0, 0.0, 2.0]],
            false,
        );
        append_triangle(
            &mut source,
            [[-2.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 1.0]],
            true,
        );
        // A nonzero-area vertical face on the source boundary is authentic.
        append_triangle(
            &mut source,
            [[0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 2.0]],
            true,
        );
        // Only an edge touches the rectangle; its surface belongs outside.
        append_triangle(
            &mut source,
            [[-1.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
            false,
        );
        let expected = triangles(&source);
        let domains = separate_authentic_domains(source, Some(bounds()));
        assert_eq!(domains.len(), 2);
        let mut actual = BTreeMap::new();
        for (domain, data) in domains {
            assert_eq!(data.indices.len(), 6);
            for (key, n) in triangles(&data) {
                *actual.entry(key).or_insert(0) += n;
            }
            match domain {
                AuthenticDomain::Inside => assert!(data.positions.iter().all(|p| p[0] >= 0.0)),
                AuthenticDomain::Outside => assert!(data.positions.iter().all(|p| p[0] <= 0.0)),
                other => panic!("unexpected domain {other:?}"),
            }
        }
        assert_eq!(actual, expected);
    }

    #[test]
    fn authentic_clipping_conserves_area_winding_attributes_and_exact_once_coverage() {
        let source = large_plane();
        let original_area = projected_signed_area(&source);
        let domains = separate_authentic_domains(source.clone(), Some(bounds()));
        for i in 0..source.positions.len() {
            assert!(
                domains.iter().any(|(_, data)| {
                    (0..data.positions.len()).any(|j| {
                        data.positions[j].map(f32::to_bits) == source.positions[i].map(f32::to_bits)
                            && data.normals[j].map(f32::to_bits)
                                == source.normals[i].map(f32::to_bits)
                            && data.uvs[j].map(f32::to_bits) == source.uvs[i].map(f32::to_bits)
                            && data.colors[j].map(f32::to_bits)
                                == source.colors[i].map(f32::to_bits)
                    })
                }),
                "an original corner lost its exact attribute bits"
            );
        }
        assert_eq!(domains.len(), 2);
        let inside = &domains
            .iter()
            .find(|(d, _)| *d == AuthenticDomain::Inside)
            .unwrap()
            .1;
        let outside = &domains
            .iter()
            .find(|(d, _)| *d == AuthenticDomain::Outside)
            .unwrap()
            .1;
        assert!((projected_signed_area(inside) + 12.0).abs() < 1.0e-6);
        assert!(
            (projected_signed_area(inside) + projected_signed_area(outside) - original_area).abs()
                < 1.0e-5
        );
        for (domain, data) in &domains {
            let mut eligible = vec![false; data.positions.len()];
            for range in &data.cutaway_ranges {
                eligible[range.clone()].fill(true);
            }
            assert!(eligible.into_iter().all(|b| b));
            for (i, &[x, y, z]) in data.positions.iter().enumerate() {
                assert!((y - (x / 8.0 + z / 4.0)).abs() < 1.0e-6);
                assert_eq!(
                    data.normals[i].map(f32::to_bits),
                    [0.125, 1.0, -0.0_f32].map(f32::to_bits)
                );
                assert!((data.uvs[i][0] - x / 16.0).abs() < 1.0e-6);
                assert!((data.uvs[i][1] - z / 16.0).abs() < 1.0e-6);
                assert!((data.colors[i][0] - (0.5 + x / 64.0)).abs() < 1.0e-6);
                assert!((data.colors[i][1] - (0.5 + z / 64.0)).abs() < 1.0e-6);
                assert_eq!(data.colors[i][2..], [0.25, 1.0]);
                if *domain == AuthenticDomain::Inside {
                    assert!((0.0..=4.0).contains(&x) && (0.0..=3.0).contains(&z));
                }
            }
            for triangle in data.indices.chunks_exact(3) {
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
                    .map(|i| Vec3::from_array(data.positions[i as usize]));
                assert!((b - a).cross(c - a).y >= 0.0, "winding changed");
                if *domain == AuthenticDomain::Outside {
                    let center = (a + b + c) / 3.0;
                    assert!(
                        center.x <= 0.0 || center.x >= 4.0 || center.z <= 0.0 || center.z >= 3.0
                    );
                }
            }
        }
        // Irrational-ish offsets avoid authored and newly introduced edges.
        // Every sample in the complete overworld plane is owned exactly once.
        for z in 0..71 {
            for x in 0..73 {
                let p = [
                    -8.0 + (x as f64 + 0.371) * 20.0 / 73.0,
                    -10.0 + (z as f64 + 0.613) * 24.0 / 71.0,
                ];
                let n_inside = count_strict_coverage(inside, p);
                let n_outside = count_strict_coverage(outside, p);
                assert_eq!(
                    n_inside + n_outside,
                    1,
                    "gap or overlapping surface at {p:?}"
                );
                let authentic = p[0] > 0.0 && p[0] < 4.0 && p[1] > 0.0 && p[1] < 3.0;
                assert_eq!(n_inside, usize::from(authentic));
            }
        }
    }

    #[test]
    fn authentic_clipping_keeps_unsupported_crossings_as_rejection_evidence() {
        let mut source = SurfaceMeshData::default();
        append_triangle(
            &mut source,
            [[-1.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 2.0]],
            false,
        );
        source.normals.fill([0.0, 1.0, 0.0]);
        source.cutaway_ranges.push(0..1);
        let domains = separate_authentic_domains(source.clone(), Some(bounds()));
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].0, AuthenticDomain::Unresolved);
        assert_eq!(triangles(&domains[0].1), triangles(&source));
        source.cutaway_ranges.clear();
        source.uvs[0][0] = f32::NAN;
        let domains = separate_authentic_domains(source.clone(), Some(bounds()));
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].0, AuthenticDomain::Unresolved);
        assert_eq!(triangles(&domains[0].1), triangles(&source));
        source.uvs[0][0] = -1.0 / 16.0;
        source.normals[1][0] = 1.0;
        let domains = separate_authentic_domains(source.clone(), Some(bounds()));
        assert_eq!(domains.len(), 1);
        assert_eq!(
            domains[0].0,
            AuthenticDomain::Unresolved,
            "mixed normals must keep original shading"
        );
        assert_eq!(triangles(&domains[0].1), triangles(&source));
        // Lack of valid rectangle/topology must never manufacture ownership.
        for rectangle in [
            None,
            Some(AuthenticBounds {
                min: Vec2::ZERO,
                max: Vec2::ZERO,
            }),
            Some(AuthenticBounds {
                min: Vec2::splat(f32::NAN),
                max: Vec2::ONE,
            }),
        ] {
            let domains = separate_authentic_domains(source.clone(), rectangle);
            assert_eq!(domains[0].0, AuthenticDomain::Unspecified);
            assert_eq!(triangles(&domains[0].1), triangles(&source));
        }
        source.indices.push(0);
        let domains = separate_authentic_domains(source.clone(), Some(bounds()));
        assert_eq!(domains[0].0, AuthenticDomain::Unspecified);
        assert_eq!(domains[0].1.indices, source.indices);
    }

    #[test]
    fn authentic_shared_edge_intersections_are_direction_independent() {
        let source = large_plane();
        let a = ClipCorner::from_source(&source, 0);
        let b = ClipCorner::from_source(&source, 1);
        for (axis, boundary) in [(0, 0.137), (0, 3.719), (2, 2.813)] {
            let forward = a.intersect(b, axis, boundary);
            let backward = b.intersect(a, axis, boundary);
            assert_eq!(
                forward.position.map(f32::to_bits),
                backward.position.map(f32::to_bits)
            );
            assert_eq!(
                forward.normal.map(f32::to_bits),
                backward.normal.map(f32::to_bits)
            );
            assert_eq!(forward.uv.map(f32::to_bits), backward.uv.map(f32::to_bits));
            assert_eq!(
                forward.color.map(f32::to_bits),
                backward.color.map(f32::to_bits)
            );
            assert_eq!(forward.position[axis], boundary);
        }
    }

    #[test]
    fn authentic_domain_batches_bound_draw_growth_and_keep_real_aabbs() {
        let source = large_plane();
        assert_eq!(partition(source.clone(), Vec2::ONE).len(), 1);
        let domains = separate_authentic_domains(source, Some(bounds()));
        let batches: Vec<_> = domains
            .into_iter()
            .flat_map(|(d, data)| {
                partition(data, Vec2::ONE)
                    .into_iter()
                    .map(move |data| (d, data))
            })
            .collect();
        assert_eq!(
            batches.len(),
            2,
            "two huge triangles need only two domain draws"
        );
        for (domain, data) in batches {
            let aabb = data.into_mesh().compute_aabb().unwrap();
            if domain == AuthenticDomain::Inside {
                assert_eq!(aabb.min().x, 0.0);
                assert_eq!(aabb.max().x, 4.0);
                assert_eq!(aabb.min().z, 0.0);
                assert_eq!(aabb.max().z, 3.0);
            }
        }
        let source = fixture(68);
        let bounds = AuthenticBounds {
            min: Vec2::new(65.0, 57.0),
            max: Vec2::new(471.0, 463.0),
        };
        let domains = separate_authentic_domains(source, Some(bounds));
        let batches: Vec<_> = domains
            .into_iter()
            .flat_map(|(d, data)| {
                partition(data, Vec2::splat(0.0001))
                    .into_iter()
                    .map(move |data| (d, data))
            })
            .collect();
        assert!(batches.len() <= 3 * MAX_BATCHES_PER_AXIS * MAX_BATCHES_PER_AXIS);
        assert!(batches.iter().any(|(d, _)| *d == AuthenticDomain::Inside));
        assert!(batches.iter().any(|(d, _)| *d == AuthenticDomain::Outside));
        for (domain, data) in batches {
            if domain == AuthenticDomain::Inside {
                for [x, _, z] in data.positions {
                    assert!((bounds.min.x..=bounds.max.x).contains(&x));
                    assert!((bounds.min.y..=bounds.max.y).contains(&z));
                }
            }
        }
    }

    #[test]
    fn static_batches_keep_every_ordered_triangle_corner_and_cutaway_bit() {
        let mut source = fixture(68);
        // Complete long triangles must remain, even across multiple cells.
        source
            .indices
            .extend([0, 5000, source.positions.len() as u32 - 1]);
        // Duplicate surfaces and degenerate geometry are retained too.
        source.indices.extend_from_within(0..3);
        source.indices.extend([0, 0, 0]);
        let expected = triangles(&source);
        let batches = partition(source.clone(), Vec2::splat(8.0));
        assert!(batches.len() > 1);
        assert!(batches.len() <= MAX_BATCHES_PER_AXIS * MAX_BATCHES_PER_AXIS);
        let mut actual = BTreeMap::new();
        for batch in &batches {
            for (key, count) in triangles(batch) {
                *actual.entry(key).or_insert(0) += count;
            }
        }
        assert_eq!(actual, expected);
        assert_eq!(batches, partition(source, Vec2::splat(8.0)));
    }

    #[test]
    fn static_batches_are_bounded_and_keep_small_or_uncertain_inputs_whole() {
        let small = fixture(3);
        assert_eq!(partition(small.clone(), Vec2::splat(8.0)), vec![small]);
        let large = fixture(68);
        for tile_size in [
            Vec2::ZERO,
            Vec2::splat(f32::NAN),
            Vec2::splat(f32::INFINITY),
        ] {
            assert_eq!(partition(large.clone(), tile_size), vec![large.clone()]);
        }
        let batches = partition(large.clone(), Vec2::splat(0.0001));
        assert!(batches.len() <= MAX_BATCHES_PER_AXIS * MAX_BATCHES_PER_AXIS);
        let mut nonfinite = large;
        nonfinite.positions[0][0] = f32::INFINITY;
        assert_eq!(
            partition(nonfinite.clone(), Vec2::splat(8.0)),
            vec![nonfinite]
        );
        assert!(partition(SurfaceMeshData::default(), Vec2::splat(8.0)).is_empty());
    }

    #[test]
    fn static_batch_frusta_keep_visible_samples_after_orbit_zoom_and_scroll() {
        let mut source = fixture(68);
        for p in &mut source.positions {
            p[0] -= 272.0;
            p[2] -= 272.0;
        }
        // Test a triangle crossing the eye plane, not just local tile faces.
        let n = source.positions.len() as u32;
        source.positions.extend([
            [-400.0, 0.0, -400.0],
            [400.0, 0.0, -400.0],
            [0.0, 400.0, 400.0],
        ]);
        source.normals.extend([[0.0, 1.0, 0.0]; 3]);
        source.uvs.extend([[0.0, 0.0]; 3]);
        source.colors.extend([[1.0; 4]; 3]);
        source.indices.extend([n, n + 1, n + 2]);
        // Exercise ordinary batches and the complete authentic/exterior union
        // against the same cameras, including the huge crossing triangle.
        let batches = [
            None,
            Some(AuthenticBounds {
                min: Vec2::new(-129.5, -133.25),
                max: Vec2::new(117.75, 103.5),
            }),
        ]
        .into_iter()
        .flat_map(|bounds| separate_authentic_domains(source.clone(), bounds))
        .flat_map(|(_, data)| partition(data, Vec2::splat(8.0)))
        .map(|batch| {
            let bounds = batch.clone().into_mesh().compute_aabb().unwrap();
            (batch, bounds)
        })
        .collect::<Vec<_>>();
        let mut culled_triangles = 0;
        let mut checked_visible_samples = 0;
        for viewport in [Vec2::new(160.0, 144.0), Vec2::new(480.0, 640.0)] {
            for zoom in [0.0, 1.0, 3.0, 5.0] {
                for orbit in [0.0, 0.25, 1.0, 2.0, 3.5, 5.25, 7.0] {
                    let pose = crate::camera::VoxelCameraControls::new(zoom, orbit).pose(viewport);
                    let camera = GlobalTransform::from(pose.transform());
                    let projection = PerspectiveProjection {
                        fov: pose.vertical_fov_radians,
                        near: pose.near,
                        far: pose.far,
                        aspect_ratio: viewport.x / viewport.y,
                    };
                    let frustum = projection.compute_frustum(&camera);
                    let clip_from_world =
                        projection.get_clip_from_view() * camera.compute_matrix().inverse();
                    for offset in [Vec3::ZERO, Vec3::new(-73.5, 0.0, 97.25)] {
                        let world_from_local = GlobalTransform::from_translation(offset);
                        let clip_from_local = clip_from_world * world_from_local.compute_matrix();
                        for (batch, bounds) in &batches {
                            let sphere = Sphere {
                                center: world_from_local.affine().transform_point3a(bounds.center),
                                radius: world_from_local.radius_vec3a(bounds.half_extents),
                            };
                            let visible = frustum.intersects_sphere(&sphere, false)
                                && frustum.intersects_obb(
                                    bounds,
                                    &world_from_local.affine(),
                                    true,
                                    false,
                                );
                            if !visible {
                                culled_triangles += batch.indices.len() / 3;
                            }
                            for triangle in batch.indices.chunks_exact(3) {
                                let [a, b, c] = [triangle[0], triangle[1], triangle[2]]
                                    .map(|i| Vec3::from_array(batch.positions[i as usize]));
                                for p in [a, b, c, (a + b + c) / 3.0, a * 0.2 + b * 0.3 + c * 0.5] {
                                    let p = clip_from_local * p.extend(1.0);
                                    if p.w > 0.0
                                        && p.x.abs() < p.w
                                        && p.y.abs() < p.w
                                        && p.z > 0.0
                                        && p.z < p.w
                                    {
                                        checked_visible_samples += 1;
                                        assert!(visible, "culling discarded an on-screen sample");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(
            culled_triangles > 100_000,
            "fixture must exercise real off-screen exclusion"
        );
        assert!(
            checked_visible_samples > 100_000,
            "fixture must also exercise retained geometry"
        );
    }
}
