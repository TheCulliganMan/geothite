//! Exact static geometry in bounded spatial draws. Bevy culls each draw against
//! each camera and shadow frustum; no camera/player-dependent geometry is built.
use crate::mesh::SurfaceMeshData;
use bevy::{prelude::*, render::primitives::Aabb};

pub(super) struct PreparedBatch {
    pub mesh: Mesh,
    pub bounds: Option<Aabb>,
}

impl From<Mesh> for PreparedBatch {
    fn from(mesh: Mesh) -> Self {
        // Compute on the terrain worker, before render extraction moves the
        // CPU buffers. Never scan full vertex buffers when applying a build.
        let bounds = mesh.compute_aabb();
        Self { mesh, bounds }
    }
}

const CELLS_PER_BATCH: f32 = 16.0;
const MAX_BATCHES_PER_AXIS: usize = 8;
const MIN_TRIANGLES: usize = 4096;

pub(super) fn prepare(source: SurfaceMeshData, tile_size: Vec2) -> Vec<PreparedBatch> {
    // A native-only control for paired measurements of this same renderer.
    // Browser builds always use the production partition.
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        if !*ENABLED
            .get_or_init(|| std::env::var("CRYSTAL_TERRAIN_BATCHING").as_deref() != Ok("off"))
        {
            return vec![source.into_mesh().into()];
        }
    }
    partition(source, tile_size)
        .into_iter()
        .map(|data| data.into_mesh().into())
        .collect()
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
        let batches = partition(source, Vec2::splat(8.0))
            .into_iter()
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
