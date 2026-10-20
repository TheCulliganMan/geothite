//! Exact static geometry in bounded spatial draws. Bevy culls each draw against
//! each camera and shadow frustum; no camera/player-dependent geometry is built.
use bevy::camera::primitives::MeshAabb;
use crate::mesh::SurfaceMeshData;
use bevy::{prelude::*, camera::primitives::Aabb};

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
    pub rooted_grass: bool,
    pub bounds: Option<Aabb>,
    pub authentic_domain: AuthenticDomain,
    pub coverage: Option<std::sync::Arc<[Aabb]>>,
    pub sight_coverage: Option<std::sync::Arc<[Aabb]>>,
    pub grass_coverage: Option<std::sync::Arc<[Aabb]>>,
}

/// Bounded original-triangle coverage, published before CPU geometry retires.
/// The handle ties this evidence to exactly the immutable GPU mesh it describes.
#[derive(Component, Clone)]
pub(super) struct TerrainBatchCoverage {
    pub mesh: bevy::asset::AssetId<Mesh>,
    pub bounds: std::sync::Arc<[Aabb]>,
    pub sight_bounds: std::sync::Arc<[Aabb]>,
    pub grass_bounds: std::sync::Arc<[Aabb]>,
}

pub(super) fn triangle_coverage(mesh: &Mesh) -> Option<std::sync::Arc<[Aabb]>> {
    triangle_coverage_inner(mesh, 0)
}
pub(super) fn triangle_sight_coverage(mesh: &Mesh) -> Option<std::sync::Arc<[Aabb]>> {
    triangle_coverage_inner(mesh, 1)
}

pub(super) fn triangle_grass_coverage(mesh: &Mesh) -> Option<std::sync::Arc<[Aabb]>> {
    triangle_coverage_inner(mesh, 2)
}

fn triangle_coverage_inner(mesh: &Mesh, kind: u8) -> Option<std::sync::Arc<[Aabb]>> {
    let sight = kind != 0;
    use bevy::mesh::VertexAttributeValues;
    use std::collections::BTreeMap;
    if mesh.primitive_topology() != bevy::render::render_resource::PrimitiveTopology::TriangleList {
        return None;
    }
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return None; };
    let indices = mesh.indices()?;
    if indices.is_empty() || indices.len() % 3 != 0 { return None; }
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uvs)) => Some(uvs),
        _ => None,
    };
    // One core tile per bucket. A long triangle retains its whole bound,
    // rather than pretending its center proves all of its source acreage.
    let mut buckets = BTreeMap::<(i32, i32, i32), (Vec3, Vec3)>::new();
    let mut indices = indices.iter();
    while let Some(a) = indices.next() {
        let triangle = [a, indices.next()?, indices.next()?];
        // The battle shader opens its proven full-body aperture in tagged
        // native grass. It cannot hide opaque cavern walls or architecture.
        let grass = uvs.is_some_and(|uvs| triangle.iter().all(|index|
            uvs.get(*index).is_some_and(|uv| uv[0] == 84000.0)));
        if (kind == 1 && grass) || (kind == 2 && !grass) { continue; }
        let points = triangle.map(|index| positions.get(index).map(|p| Vec3::from_array(*p)));
        let points = [points[0]?, points[1]?, points[2]?];
        if points.iter().any(|p| !p.is_finite()) { return None; }
        let min = points[0].min(points[1]).min(points[2]);
        let max = points[0].max(points[1]).max(points[2]);
        // Sight needs narrower, height-separated bounds: merging a low bank
        // with a wall in the same ground tile invents an opaque volume above
        // the water. Every original triangle still contributes its whole AABB.
        let cell = if sight { Vec3::new(16.0, 8.0, 16.0) } else { Vec3::splat(64.0) };
        let center = (min * 0.5 + max * 0.5) / cell;
        let key = (center.x.floor() as i32,
            if sight { center.y.floor() as i32 } else { 0 },
            center.z.floor() as i32);
        buckets.entry(key).and_modify(|bounds| {
            bounds.0 = bounds.0.min(min);
            bounds.1 = bounds.1.max(max);
        }).or_insert((min, max));
        if buckets.len() > 4096 { return None; }
    }
    Some(buckets.into_values().map(|(min, max)| Aabb::from_min_max(min, max)).collect())
}

/// Compute the three independent coverage domains in one original-index scan.
/// Invalid/excessive domains retire separately: excluded grass must not poison
/// opaque sight evidence, and a large coarse domain may exceed its own cap.
fn triangle_coverages(mesh: &Mesh) -> [Option<std::sync::Arc<[Aabb]>>; 3] {
    use bevy::mesh::VertexAttributeValues;
    use std::collections::BTreeMap;
    let _trace = crate::loading_trace::span("region.triangle_coverage", || serde_json::json!({
        "triangles": mesh.indices().map_or(0, |indices| indices.len() / 3),
    }));
    if mesh.primitive_topology() != bevy::render::render_resource::PrimitiveTopology::TriangleList {
        return [None, None, None];
    }
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else { return [None, None, None]; };
    let Some(indices) = mesh.indices() else { return [None, None, None]; };
    if indices.is_empty() || indices.len() % 3 != 0 { return [None, None, None]; }
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uvs)) => Some(uvs),
        _ => None,
    };
    let mut buckets: [Option<BTreeMap<(i32, i32, i32), (Vec3, Vec3)>>; 3] =
        std::array::from_fn(|_| Some(BTreeMap::new()));
    let mut indices = indices.iter();
    while let Some(a) = indices.next() {
        let (Some(b), Some(c)) = (indices.next(), indices.next()) else { return [None, None, None]; };
        let triangle = [a, b, c];
        let grass = uvs.is_some_and(|uvs| triangle.iter().all(|&index|
            uvs.get(index).is_some_and(|uv| uv[0] == 84000.0)));
        let fine_domain = if grass { 2 } else { 1 };
        if buckets[0].is_none() && buckets[fine_domain].is_none() { continue; }
        let points = triangle.map(|index| positions.get(index).map(|p| Vec3::from_array(*p)));
        let points = match points {
            [Some(a), Some(b), Some(c)] if a.is_finite() && b.is_finite() && c.is_finite() => [a, b, c],
            _ => { buckets[0] = None; buckets[fine_domain] = None; continue; }
        };
        let min = points[0].min(points[1]).min(points[2]);
        let max = points[0].max(points[1]).max(points[2]);
        for domain in [0, fine_domain] {
            let Some(bucket) = buckets[domain].as_mut() else { continue; };
            let cell = if domain == 0 { Vec3::splat(64.0) } else { Vec3::new(16.0, 8.0, 16.0) };
            let center = (min * 0.5 + max * 0.5) / cell;
            let key = (center.x.floor() as i32,
                if domain == 0 { 0 } else { center.y.floor() as i32 },
                center.z.floor() as i32);
            bucket.entry(key).and_modify(|bounds| {
                bounds.0 = bounds.0.min(min);
                bounds.1 = bounds.1.max(max);
            }).or_insert((min, max));
            if bucket.len() > 4096 { buckets[domain] = None; }
        }
    }
    buckets.map(|bucket| bucket.map(|bucket|
        bucket.into_values().map(|(min, max)| Aabb::from_min_max(min, max)).collect()))
}

/// Match the vertex shader's tag test, including mixed tagged triangles.
/// Compute before CPU buffers retire; publication never scans vertex arrays.
pub(super) fn has_rooted_grass(mesh: &Mesh) -> bool {
    matches!(mesh.attribute(Mesh::ATTRIBUTE_UV_0),
        Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs))
            if uvs.iter().any(|uv| (uv[0] - 84000.0).abs() < 0.5))
}

impl From<Mesh> for PreparedBatch {
    fn from(mesh: Mesh) -> Self {
        // Compute on the terrain worker, before render extraction moves the
        // CPU buffers. Never scan full vertex buffers when applying a build.
        let bounds = mesh.get_aabb();
        let rooted_grass = has_rooted_grass(&mesh);
        let [coverage, sight_coverage, grass_coverage] = triangle_coverages(&mesh);
        Self {
            mesh,
            rooted_grass,
            bounds,
            authentic_domain: AuthenticDomain::Unspecified,
            coverage,
            sight_coverage,
            grass_coverage,
        }
    }
}

const CELLS_PER_BATCH: f32 = 8.0;
const MAX_BATCHES_PER_AXIS: usize = 16;
const MIN_TRIANGLES: usize = 4096;

pub(super) fn prepare(source: SurfaceMeshData, tile_size: Vec2) -> Vec<PreparedBatch> {
    prepare_with_authentic_bounds(source, tile_size, None)
}

fn convert_spatial_batch(domain: AuthenticDomain, data: SurfaceMeshData) -> PreparedBatch {
    let mut batch = PreparedBatch::from(data.into_compact_mesh());
    batch.authentic_domain = domain;
    batch
}

#[cfg(not(target_arch = "wasm32"))]
fn convert_spatial_batches_on_pool(batches: Vec<(AuthenticDomain, SurfaceMeshData)>,
    pool: &bevy::tasks::TaskPool) -> Vec<PreparedBatch> {
    let workers=pool.thread_num().max(1).min(batches.len());
    if workers==0 {return Vec::new();}
    let count=batches.len();
    let pending=std::sync::Mutex::new(batches.into_iter().enumerate()
        .collect::<std::collections::VecDeque<_>>());
    // Bevy scope() ticks this pool's executor while awaiting its children.
    // A terrain task therefore cannot starve its own one-thread pool. Do not
    // replace this with block_on() over ordinary spawned task handles.
    let output=pool.scope(|scope| {
        for _ in 0..workers {
            let pending=&pending;
            scope.spawn(async move {
                let mut completed=Vec::new();
                loop {
                    let work={pending.lock().unwrap().pop_front()};
                    let Some((index,(domain,data)))=work else {break;};
                    completed.push((index,convert_spatial_batch(domain,data)));
                }
                completed
            });
        }
    });
    let mut ordered=output.into_iter().flatten().collect::<Vec<_>>();
    assert_eq!(ordered.len(),count,"every owned terrain batch must complete");
    ordered.sort_unstable_by_key(|(index,_)|*index);
    ordered.into_iter().map(|(_,batch)|batch).collect()
}

/// Split only source-map boundary crossings, then spatially batch each domain
/// separately. All three domains still render in the overworld; battle scenery
/// can omit proven Outside batches without mistaking Unresolved for padding.
/// At most three domains, each with at most 16 x 16 spatial batches, are produced.
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
    let domains=separate_authentic_domains(source, authentic_bounds);
    #[cfg(not(target_arch = "wasm32"))]
    {
        static PARALLEL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let enabled=*PARALLEL.get_or_init(||
            std::env::var("CRYSTAL_TERRAIN_BATCH_PARALLEL").as_deref()!=Ok("off"));
        if enabled && spatial_batching {
            if let Some(pool)=bevy::tasks::AsyncComputeTaskPool::try_get() {
                // Source-domain decisions and spatial ordering stay serial.
                // Only independent compact mesh/coverage conversion is parallel.
                let batches=domains.into_iter().flat_map(|(domain,data)|
                    partition(data,tile_size).into_iter().map(move |data|(domain,data)))
                    .collect::<Vec<_>>();
                let triangles=batches.iter().map(|(_,data)|data.indices.len()/3).sum::<usize>();
                let parallel=batches.len()>=4 && triangles>=16_384;
                let _trace=crate::loading_trace::span("region.convert_spatial_batches", ||
                    serde_json::json!({"batches":batches.len(),"triangles":triangles,
                        "workers":if parallel {pool.thread_num().min(batches.len()).max(1)}else{1},
                        "parallel":parallel}));
                return if parallel {convert_spatial_batches_on_pool(batches,pool)}
                    else {batches.into_iter().map(|(domain,data)|convert_spatial_batch(domain,data)).collect()};
            }
        }
    }
    domains.into_iter().flat_map(|(domain, data)| {
            let batches = if spatial_batching {
                partition(data, tile_size)
            } else {
                vec![data]
            };
            batches.into_iter().map(move |data| convert_spatial_batch(domain,data))
        })
        .collect()
}

/// A cave's separate enclosing rock owns everything outside the authored
/// map. Keeping the padded floor batches there creates coplanar mineral
/// surfaces under the reveal, producing stripes when the rock fades away.
/// Sculpted enclosing sides can cross the nominal boundary; their complete
/// source geometry remains intact. Only padded floors belong to the outer cap.
pub(super) fn prepare_enclosed_cave(
    mut source: SurfaceMeshData, tile_size: Vec2, bounds: Option<AuthenticBounds>,
) -> Vec<PreparedBatch> {
    let Some(rect)=bounds.filter(|b| b.min.is_finite() && b.max.is_finite()
        && b.max.x>b.min.x && b.max.y>b.min.y) else {
        return prepare_with_authentic_bounds(source,tile_size,bounds);
    };
    if !valid_topology(&source) || !tile_size.is_finite() || tile_size.min_element()<=0. {
        return prepare_with_authentic_bounds(source,tile_size,bounds);
    }
    let margin=tile_size.max_element()*0.75;
    let enclosing=source.indices.chunks_exact(3).map(|triangle| {
        let outside=triangle.iter().any(|&i| {
            let p=source.positions[i as usize];
            p[0]<rect.min.x || p[0]>rect.max.x || p[2]<rect.min.y || p[2]>rect.max.y
        });
        outside && triangle.iter().all(|&i| {
            let i=i as usize;let p=source.positions[i];let uv=source.uvs[i];
            let local=Vec2::new(p[0],p[2]);
            uv[0]>90000. && matches!((uv[1]/100000.).round() as i32,2|3|10|18)
                && source.normals[i][1].abs()<0.85
                && (local-local.clamp(rect.min,rect.max)).length()<=margin
        })
    }).collect::<Vec<_>>();
    if !enclosing.iter().any(|&keep|keep) {
        let mut batches=prepare_with_authentic_bounds(source,tile_size,bounds);
        batches.retain(|batch|batch.authentic_domain!=AuthenticDomain::Outside);
        return batches;
    }
    let mut marked=vec![false;source.positions.len()];
    for range in &source.cutaway_ranges {marked[range.clone()].fill(true);}
    let mut remap=vec![u32::MAX;source.positions.len()];
    let wall=crate::mesh::select_surface_indices(&source,&marked,&mut remap,
        source.indices.chunks_exact(3).zip(&enclosing).filter(|(_,keep)|**keep)
            .flat_map(|(triangle,_)|triangle.iter().copied()));
    drop(marked);drop(remap);
    // Keep the large remaining attribute buffers owned in place. The normal
    // domain partition compacts their referenced vertices; duplicating the
    // entire terrain here would add an unnecessary peak allocation.
    let mut corner=0;
    source.indices.retain(|_| {let keep=!enclosing[corner/3];corner+=1;keep});
    let mut batches=prepare_with_authentic_bounds(source,tile_size,bounds);
    batches.retain(|batch|batch.authentic_domain!=AuthenticDomain::Outside);
    batches.extend(prepare_with_authentic_bounds(wall,tile_size,bounds));
    batches
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

pub(super) fn separate_authentic_domains(
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
    // Most authored objects lie wholly in one domain. Classify their existing
    // buffer once and move it through unchanged instead of expanding every
    // triangle corner and copying all four attribute streams twice.
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    let mut finite = true;
    for &position in &source.positions {
        let p = Vec2::new(position[0], position[2]);
        if !p.is_finite() {
            finite = false;
            break;
        }
        min = min.min(p);
        max = max.max(p);
    }
    if finite {
        if min.cmpge(bounds.min).all() && max.cmple(bounds.max).all() {
            return vec![(AuthenticDomain::Inside, source)];
        }
        // Strict separation keeps boundary-only triangles in the inside domain.
        if max.x < bounds.min.x || min.x > bounds.max.x
            || max.y < bounds.min.y || min.y > bounds.max.y
        {
            return vec![(AuthenticDomain::Outside, source)];
        }
    }
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
        let base = data.positions.len();
        let mut indices = indices;
        for index in &mut indices {
            let old = *index as usize;
            if remap[old] == u32::MAX {
                remap[old] = (base + touched.len()) as u32;
                touched.push(old);
            }
            *index = remap[old];
        }
        data.positions.reserve_exact(touched.len());
        data.normals.reserve_exact(touched.len());
        data.uvs.reserve_exact(touched.len());
        data.colors.reserve_exact(touched.len());
        for (offset, old) in touched.drain(..).enumerate() {
            data.positions.push(source.positions[old]);
            data.normals.push(source.normals[old]);
            data.uvs.push(source.uvs[old]);
            data.colors.push(source.colors[old]);
            if marked[old] {
                mark_vertex(&mut data, base + offset);
            }
            remap[old] = u32::MAX;
        }
        if data.indices.is_empty() {
            data.indices = indices;
        } else {
            data.indices.reserve_exact(indices.len());
            data.indices.extend(indices);
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
    // Browsers retain the existing memory policy until separately measured.
    #[cfg(target_arch = "wasm32")]
    let cache_buckets = false;
    #[cfg(not(target_arch = "wasm32"))]
    let cache_buckets = {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("CRYSTAL_TERRAIN_BUCKET_CACHE").as_deref() != Ok("off"))
    };
    partition_with_bucket_cache(source, tile_size, cache_buckets)
}

fn partition_with_bucket_cache(source: SurfaceMeshData, tile_size: Vec2, cache_buckets: bool) -> Vec<SurfaceMeshData> {
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
    let _trace = crate::loading_trace::span("region.partition_surface", || serde_json::json!({
        "vertices":vertex_count,"triangles":source.indices.len()/3,
        "buckets":batch_count,"cache_buckets":cache_buckets,
    }));
    let bucket_for = |triangle: &[u32]| {
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
        z * count.x as usize + x
    };
    // Count first: growing large buckets can retain nearly twice the indices
    // and briefly needs both old and new allocations in the browser heap.
    let mut sizes = vec![0; batch_count];
    // At most 16x16 buckets: one byte per triangle avoids repeating the f64
    // centroid calculation without adding a source-sized remap per worker.
    // Release it before allocating compact vertex attributes below.
    let mut triangle_buckets = cache_buckets.then(|| Vec::<u8>::with_capacity(source.indices.len() / 3));
    for triangle in source.indices.chunks_exact(3) {
        let bucket = bucket_for(triangle);
        sizes[bucket] += 3;
        if let Some(cached) = triangle_buckets.as_mut() {
            cached.push(u8::try_from(bucket).expect("spatial bucket fits in one byte"));
        }
    }
    let mut buckets: Vec<Vec<u32>> = sizes.into_iter().map(Vec::with_capacity).collect();
    if let Some(cached) = triangle_buckets {
        for (triangle, bucket) in source.indices.chunks_exact(3).zip(cached) {
            buckets[usize::from(bucket)].extend_from_slice(triangle);
        }
    } else {
        for triangle in source.indices.chunks_exact(3) {
            buckets[bucket_for(triangle)].extend_from_slice(triangle);
        }
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
    for mut indices in buckets.into_iter().filter(|indices| !indices.is_empty()) {
        // Determine the exact referenced vertex set before allocating attributes.
        // Reuse the bucket's index buffer rather than copying every index again.
        for index in &mut indices {
            let old = *index as usize;
            if remap[old] == u32::MAX {
                remap[old] = touched.len() as u32;
                touched.push(old);
            }
            *index = remap[old];
        }
        let count = touched.len();
        let mut batch = SurfaceMeshData {
            positions: Vec::with_capacity(count),
            normals: Vec::with_capacity(count),
            uvs: Vec::with_capacity(count),
            colors: Vec::with_capacity(count),
            indices,
            ..default()
        };
        for (next, old) in touched.drain(..).enumerate() {
            batch.positions.push(source.positions[old]);
            batch.normals.push(source.normals[old]);
            batch.uvs.push(source.uvs[old]);
            batch.colors.push(source.colors[old]);
            if marked[old] {
                mark_vertex(&mut batch, next);
            }
            remap[old] = u32::MAX;
        }
        result.push(batch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::{CameraProjection, primitives::Sphere};
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

    #[test]
    fn rooted_grass_identity_matches_shader_tags_even_on_mixed_triangles() {
        for (tag, expected) in [(0.0, false), (84000.0, true), (83999.75, true),
            (84000.5, false), (86000.0, false)] {
            let mut mesh = crate::actor_quad_mesh();
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,
                vec![[tag, 0.5], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]);
            let batch = PreparedBatch::from(mesh);
            assert_eq!(batch.rooted_grass, expected, "shader tag {tag}");
            assert!(batch.grass_coverage.unwrap().is_empty(),
                "mixed triangles cannot use whole-grass triangle coverage as shader identity");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn assert_prepared_batches_exact(expected:&[PreparedBatch],actual:&[PreparedBatch]) {
        let bounds_bits=|bounds:Aabb| {
            let mut bits=bounds.center.to_array().map(f32::to_bits).to_vec();
            bits.extend(bounds.half_extents.to_array().map(f32::to_bits));bits
        };
        let coverage_bits=|coverage:&Option<std::sync::Arc<[Aabb]>>|
            coverage.as_ref().map(|coverage|coverage.iter().copied().map(&bounds_bits).collect::<Vec<_>>());
        assert_eq!(actual.len(),expected.len());
        for (a,b) in expected.iter().zip(actual) {
            assert_eq!(a.authentic_domain,b.authentic_domain);
            assert_eq!(a.rooted_grass,b.rooted_grass);
            assert_eq!(a.mesh.asset_usage,b.mesh.asset_usage);
            assert_eq!(a.mesh.primitive_topology(),b.mesh.primitive_topology());
            assert_eq!(a.mesh.indices(),b.mesh.indices(),"index values AND widths remain exact");
            let attributes=|mesh:&Mesh|mesh.attributes().map(|(attribute,values)|
                (attribute.id,attribute.format,values.get_bytes().to_vec())).collect::<Vec<_>>();
            assert_eq!(attributes(&a.mesh),attributes(&b.mesh),"all vertex attribute bits and formats");
            assert_eq!(a.bounds.map(&bounds_bits),b.bounds.map(&bounds_bits));
            assert_eq!(coverage_bits(&a.coverage),coverage_bits(&b.coverage));
            assert_eq!(coverage_bits(&a.sight_coverage),coverage_bits(&b.sight_coverage));
            assert_eq!(coverage_bits(&a.grass_coverage),coverage_bits(&b.grass_coverage));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn native_conversion_fixture() -> Vec<(AuthenticDomain,SurfaceMeshData)> {
        let mut source=fixture(72);
        for (index,uvs) in source.uvs.chunks_exact_mut(4).enumerate() {
            if index%3==0 {uvs.fill([84000.,-0.]);}
        }
        let bounds=Some(AuthenticBounds {min:Vec2::splat(64.),max:Vec2::splat(448.)});
        let mut result=separate_authentic_domains(source,bounds).into_iter()
            .flat_map(|(domain,data)|partition(data,Vec2::splat(8.)).into_iter()
                .map(move|data|(domain,data))).collect::<Vec<_>>();
        assert!(result.len()>4);
        // Exercise both compact index widths and the remaining domain labels
        // alongside genuine source-domain and spatial partition decisions.
        result.push((AuthenticDomain::Unresolved,fixture(130)));
        result.push((AuthenticDomain::Unspecified,SurfaceMeshData::default()));
        result
    }
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_parallel_batch_conversion_is_bit_exact_ordered_and_complete() {
        let input=native_conversion_fixture();
        let expected=input.iter().map(|(domain,data)|convert_spatial_batch(*domain,data.clone()))
            .collect::<Vec<_>>();
        assert!(expected.iter().any(|batch|matches!(batch.mesh.indices(),Some(bevy::mesh::Indices::U16(_)))));
        assert!(expected.iter().any(|batch|matches!(batch.mesh.indices(),Some(bevy::mesh::Indices::U32(_)))));
        for workers in [1,3] {
            let pool=bevy::tasks::TaskPoolBuilder::new().num_threads(workers).build();
            let actual=convert_spatial_batches_on_pool(input.clone(),&pool);
            assert_prepared_batches_exact(&expected,&actual);
        }
    }
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_bucket_cache_preserves_boundary_extreme_and_negative_geometry_exactly() {
        for (scale, offset) in [(1., 0.), (1., -2048.), (-1., 1024.), (1.0e20, -2.0e22)] {
            let mut source = fixture(72);
            for position in &mut source.positions {
                position[0] = position[0] * scale + offset;
                position[2] = position[2] * scale - offset;
            }
            let serial = partition_with_bucket_cache(source.clone(), Vec2::splat(8.), false);
            let cached = partition_with_bucket_cache(source, Vec2::splat(8.), true);
            assert!(serial.len() > 1, "fixture must exercise spatial bucketing");
            let convert = |batches: Vec<SurfaceMeshData>| batches.into_iter()
                .map(|data| convert_spatial_batch(AuthenticDomain::Inside, data)).collect::<Vec<_>>();
            assert_prepared_batches_exact(&convert(serial), &convert(cached));
        }
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn native_batch_conversion_nested_in_one_worker_pool_cannot_starve_children() {
        let pool=std::sync::Arc::new(bevy::tasks::TaskPoolBuilder::new().num_threads(1).build());
        assert_eq!(pool.thread_num(),1);
        let input=native_conversion_fixture();
        let expected=input.iter().map(|(domain,data)|convert_spatial_batch(*domain,data.clone()))
            .collect::<Vec<_>>();
        let (send,receive)=std::sync::mpsc::sync_channel(1);
        let owned_pool=pool.clone();
        let task=pool.spawn(async move {
            let output=convert_spatial_batches_on_pool(input,&owned_pool);
            send.send(output).unwrap();
        });
        let actual=match receive.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(output)=>output,
            Err(error)=>{
                // Preserve a useful test failure instead of blocking forever
                // joining the very worker whose starvation is being checked.
                std::mem::forget(task);std::mem::forget(pool);
                panic!("nested single-worker terrain conversion did not complete: {error}");
            }
        };
        bevy::tasks::futures_lite::future::block_on(task);
        assert_prepared_batches_exact(&expected,&actual);
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

    fn assert_fused_coverage_matches_independent_scans(mesh: &Mesh) {
        let bits = |coverage: Option<std::sync::Arc<[Aabb]>>| coverage.map(|bounds|
            bounds.iter().map(|bound| {
                let mut bits = bound.center.to_array().map(f32::to_bits).to_vec();
                bits.extend(bound.half_extents.to_array().map(f32::to_bits));
                bits
            }).collect::<Vec<_>>());
        let expected = [triangle_coverage(mesh), triangle_sight_coverage(mesh), triangle_grass_coverage(mesh)].map(&bits);
        assert_eq!(triangle_coverages(mesh).map(bits), expected,
            "every independent domain keeps exact bound bits, key order and absence semantics");
    }

    #[test]
    fn fused_coverage_preserves_index_width_tags_and_original_mesh_bits() {
        let mut source = fixture(40);
        for (quad, uvs) in source.uvs.chunks_exact_mut(4).enumerate() {
            if quad % 3 == 0 { uvs.fill([84000., -0.0]); }
            else if quad % 3 == 1 { uvs[0][0] = 84000.; }
        }
        for mesh in [source.clone().into_mesh(), source.into_compact_mesh()] {
            assert_fused_coverage_matches_independent_scans(&mesh);
            let attributes = mesh.attributes().map(|(attribute, values)|
                (attribute.id, values.get_bytes().to_vec())).collect::<Vec<_>>();
            let indices = mesh.indices().cloned();
            let batch = PreparedBatch::from(mesh);
            assert_eq!(batch.mesh.attributes().map(|(attribute, values)|
                (attribute.id, values.get_bytes().to_vec())).collect::<Vec<_>>(), attributes);
            assert_eq!(batch.mesh.indices(), indices.as_ref());
        }
    }

    #[test]
    fn fused_coverage_retires_invalid_and_excessive_domains_independently() {
        for grass in [false, true] {
            let mut source = SurfaceMeshData::default();
            append_triangle(&mut source, [[0.,0.,0.], [4.,0.,0.], [0.,0.,4.]], false);
            append_triangle(&mut source, [[80.,0.,80.], [84.,0.,80.], [80.,0.,84.]], false);
            if grass { source.uvs[3..].fill([84000., 0.]); }
            source.positions[3][0] = f32::NAN;
            let mesh = source.into_mesh();
            assert_fused_coverage_matches_independent_scans(&mesh);
            let coverage = triangle_coverages(&mesh);
            assert!(coverage[0].is_none());
            assert!(coverage[if grass { 1 } else { 2 }].is_some(),
                "a face excluded from this domain cannot invalidate its evidence");
        }
        for spacing in [16., 128.] {
            let mut source = SurfaceMeshData::default();
            for i in 0..4100 {
                let x = i as f32 * spacing;
                append_triangle(&mut source, [[x,0.,0.], [x+1.,0.,0.], [x,0.,1.]], false);
            }
            let grass_start = source.uvs.len();
            append_triangle(&mut source, [[0.,12.,0.], [1.,12.,0.], [0.,12.,1.]], false);
            source.uvs[grass_start..].fill([84000., 0.]);
            let mesh = source.into_mesh();
            assert_fused_coverage_matches_independent_scans(&mesh);
            let [full, sight, grass] = triangle_coverages(&mesh);
            assert!(sight.is_none());
            assert_eq!(full.is_some(), spacing == 16.);
            assert_eq!(grass.unwrap().len(), 1, "each coverage domain keeps its own 4096-bucket cap");
        }
    }

    #[test]
    fn fused_coverage_keeps_missing_cpu_attributes_and_malformed_indices_absent() {
        let mut mesh = fixture(4).into_mesh();
        mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0);
        assert_fused_coverage_matches_independent_scans(&mesh);
        mesh.insert_indices(bevy::mesh::Indices::U32(vec![0,1,2,u32::MAX,1,2]));
        assert_fused_coverage_matches_independent_scans(&mesh);
        mesh.insert_indices(bevy::mesh::Indices::U32(vec![0,1]));
        assert_fused_coverage_matches_independent_scans(&mesh);
        mesh.insert_indices(bevy::mesh::Indices::U32(Vec::new()));
        assert_fused_coverage_matches_independent_scans(&mesh);
        mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION);
        assert_fused_coverage_matches_independent_scans(&mesh);
    }

    #[test]
    fn coverage_encloses_every_original_corner_including_long_crossing_faces() {
        let mut data = large_plane();
        append_triangle(&mut data, [[-320.0, 2.0, 0.0], [320.0, 12.0, 0.0], [0.0, 3.0, 40.0]], false);
        append_triangle(&mut data, [[512.0, -4.0, 512.0], [520.0, -2.0, 512.0], [512.0, -4.0, 520.0]], false);
        let mesh = data.clone().into_compact_mesh();
        let coverage = triangle_coverage(&mesh).unwrap();
        for triangle in data.indices.chunks_exact(3) {
            assert!(coverage.iter().any(|bounds| triangle.iter().all(|index| {
                let point = Vec3::from_array(data.positions[*index as usize]);
                let min: Vec3 = (bounds.center - bounds.half_extents).into();
                let max: Vec3 = (bounds.center + bounds.half_extents).into();
                point.cmpge(min).all() && point.cmple(max).all()
            })), "a center bucket must still enclose the entire original face");
        }
        assert_eq!(mesh.asset_usage, bevy::asset::RenderAssetUsages::RENDER_WORLD);
        let mut invalid = data.clone();
        invalid.positions[0][0] = f32::NAN;
        assert!(triangle_coverage(&invalid.into_mesh()).is_none());
        data.indices.push(0);
        assert!(triangle_coverage(&data.into_mesh()).is_none());
    }

    #[test]
    fn sight_coverage_does_not_join_a_low_bank_to_a_neighboring_wall() {
        let mut data = SurfaceMeshData::default();
        append_triangle(&mut data, [[0., 0., 0.], [10., 4., 0.], [0., 0., 10.]], false);
        append_triangle(&mut data, [[32., 0., 0.], [32., 40., 0.], [40., 0., 0.]], false);
        let mesh = data.into_compact_mesh();
        assert_eq!(triangle_coverage(&mesh).unwrap().len(), 1);
        let sight = triangle_sight_coverage(&mesh).unwrap();
        assert_eq!(sight.len(), 2);
        let bank = sight.iter().find(|bounds| bounds.center.x < 16.).unwrap();
        assert_eq!(bank.center.y + bank.half_extents.y, 4.);
        let wall = sight.iter().find(|bounds| bounds.center.x > 16.).unwrap();
        assert_eq!(wall.center.y + wall.half_extents.y, 40.);
    }

    #[test]
    fn sight_coverage_keeps_opaque_walls_and_full_geometry_proof_keeps_revealed_grass() {
        let mut data = SurfaceMeshData::default();
        append_triangle(&mut data, [[0., 0., 0.], [0., 20., 0.], [20., 0., 0.]], false);
        append_triangle(&mut data, [[128., 0., 128.], [128., 20., 128.], [148., 0., 128.]], false);
        data.uvs[3..].fill([84000.0, 0.0]);
        let mesh = data.into_compact_mesh();
        let full = triangle_coverage(&mesh).unwrap();
        let sight = triangle_sight_coverage(&mesh).unwrap();
        assert_eq!(full.len(), 2, "grass remains in authentic geometry and shadow coverage");
        assert_eq!(sight.len(), 1, "only the shader's body aperture avoids a sight blocker");
        let grass = triangle_grass_coverage(&mesh).unwrap();
        assert_eq!(grass.len(), 1, "lens evidence includes only original grass triangles");
        assert_eq!(grass[0].center.y + grass[0].half_extents.y, 20.);
        assert_eq!(grass[0].center.x - grass[0].half_extents.x, 128.);
        let prepared = PreparedBatch::from(mesh.clone());
        assert_eq!(prepared.grass_coverage.unwrap().len(), 1);
        assert_eq!(mesh.asset_usage, bevy::asset::RenderAssetUsages::RENDER_WORLD);
        let min: Vec3 = (sight[0].center - sight[0].half_extents).into();
        let max: Vec3 = (sight[0].center + sight[0].half_extents).into();
        assert_eq!(min, Vec3::ZERO);
        assert_eq!(max, Vec3::new(20., 20., 0.));
    }

    #[test]
    fn enclosed_cave_keeps_sculpted_boundary_sides_and_drops_padding() {
        let bounds=AuthenticBounds {min:Vec2::ZERO,max:Vec2::splat(32.)};
        let mut source=SurfaceMeshData::default();
        let mut quad=|points:[[f32;3];4],normal:[f32;3],uvs:[[f32;2];4],color:[f32;4]| {
            let first=source.positions.len() as u32;
            source.positions.extend(points);source.normals.extend([normal;4]);
            source.uvs.extend(uvs);source.colors.extend([color;4]);
            source.indices.extend([first,first+1,first+2,first,first+2,first+3]);
        };
        for (z,role,normal) in [(-0.25,3.,[0.,0.15,0.988686]),
            (-8.,3.,[0.,0.15,0.988686])] {
            let points=[[8.,0.,z],[16.,0.,z],[16.,48.,z],[8.,48.,z]];
            quad(points,normal,
                points.map(|p|[100000.+p[0],role*100000.+p[2]]),[0.2,0.4,0.5,1.]);
        }
        quad(
            [[8.,0.,-1.],[8.,0.,-0.5],[16.,0.,-0.5],[16.,0.,-1.]],
            [0.,1.,0.],[[100000.,500000.];4],[0.3;4]);
        let batches=prepare_enclosed_cave(source.clone(),Vec2::splat(8.),Some(bounds));
        let mut triangles=0;
        for batch in batches {
            let bevy::mesh::VertexAttributeValues::Float32x3(points)=
                batch.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else {panic!("positions");};
            let bevy::mesh::VertexAttributeValues::Float32x3(normals)=
                batch.mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap() else {panic!("normals");};
            let bevy::mesh::VertexAttributeValues::Float32x2(uvs)=
                batch.mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() else {panic!("uvs");};
            let indices=batch.mesh.indices().unwrap().iter().collect::<Vec<_>>();
            triangles+=indices.len()/3;
            for i in indices {
                let old=source.positions[..4].iter().position(|p|*p==points[i]).unwrap();
                assert_eq!(normals[i],source.normals[old]);
                assert_eq!(uvs[i],source.uvs[old]);
            }
        }
        assert_eq!(triangles,2,"the complete displaced side stays exactly once; far halo and floor padding do not");
    }
    #[test]
    fn enclosed_cave_removes_coplanar_padding_but_preserves_the_complete_source_floor() {
        for scale in [1.,2.] {
            let mut source=large_plane();
            for p in &mut source.positions {p[0]=p[0]*scale-17.;p[1]=0.;p[2]=p[2]*scale+29.;}
            source.normals.fill([0.,1.,0.]);
            let bounds=AuthenticBounds {min:Vec2::new(-17.,29.),
                max:Vec2::new(-17.+4.*scale,29.+3.*scale)};
            let batches=prepare_enclosed_cave(source.clone(),Vec2::splat(8.*scale),Some(bounds));
            assert!(!batches.is_empty());
            let mut area=0.;
            for batch in batches {
                assert_eq!(batch.authentic_domain,AuthenticDomain::Inside);
                let bevy::mesh::VertexAttributeValues::Float32x3(points)=
                    batch.mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else {panic!("positions");};
                assert!(points.iter().all(|p|p[1]==0. && p[0]>=bounds.min.x && p[0]<=bounds.max.x
                    && p[2]>=bounds.min.y && p[2]<=bounds.max.y));
                let indices:Vec<_>=batch.mesh.indices().unwrap().iter().collect();
                for t in indices.chunks_exact(3) {
                    let [a,b,c]=[t[0],t[1],t[2]].map(|i|Vec3::from_array(points[i]));
                    area+=(b-a).cross(c-a).y*0.5;
                }
            }
            assert!((area-12.*scale*scale).abs()<0.0001,
                "every interior floor surface must remain exactly once at the source datum");
            let fallback=prepare_enclosed_cave(source,Vec2::splat(8.*scale),None);
            assert!(fallback.iter().all(|b|b.authentic_domain==AuthenticDomain::Unspecified),
                "without authoritative bounds no padding may be inferred or removed");
        }
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
            let aabb = data.into_mesh().get_aabb().unwrap();
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
    fn compact_static_indices_preserve_geometry_and_fall_back_above_u16() {
        for large in [false, true] {
            let mut data = fixture(1);
            if large {
                data.positions.resize(65537, [0.0; 3]);
                data.normals.resize(65537, [0.0, 1.0, 0.0]);
                data.uvs.resize(65537, [0.0; 2]);
                data.colors.resize(65537, [1.0; 4]);
                data.indices.extend([65535, 65536, 0]);
            }
            let original = data.clone().into_mesh();
            let compact = data.into_compact_mesh();
            for attribute in [Mesh::ATTRIBUTE_POSITION, Mesh::ATTRIBUTE_NORMAL,
                Mesh::ATTRIBUTE_UV_0] {
                assert_eq!(compact.attribute(attribute), original.attribute(attribute));
            }
            let Some(bevy::mesh::VertexAttributeValues::Float32x4(original_colors)) =
                original.attribute(Mesh::ATTRIBUTE_COLOR) else { panic!() };
            let Some(bevy::mesh::VertexAttributeValues::Float32x3(compact_colors)) =
                compact.attribute(Mesh::ATTRIBUTE_COLOR) else { panic!() };
            let restored: Vec<_> = compact_colors.iter().map(|&[r, g, b]| [r, g, b, 1.0])
                .map(|color| color.map(f32::to_bits)).collect();
            assert_eq!(restored, original_colors.iter().map(|color| color.map(f32::to_bits)).collect::<Vec<_>>());
            assert!(compact.attribute(Mesh::ATTRIBUTE_UV_1).is_none(),
                "whole-object fading must not upload the unread legacy eligibility mask");
            assert!(matches!(compact.indices().unwrap(), bevy::mesh::Indices::U32(_)) == large);
            assert_eq!(compact.indices().unwrap().iter().collect::<Vec<_>>(),
                original.indices().unwrap().iter().collect::<Vec<_>>());
        }
    }

    #[test]
    fn whole_domain_classification_reuses_attribute_and_index_buffers() {
        for (x, domain) in [(1.0, AuthenticDomain::Inside), (-3.0, AuthenticDomain::Outside)] {
            let mut source = SurfaceMeshData::default();
            append_triangle(&mut source, [[x, 0.0, 1.0], [x + 0.5, 0.0, 1.0], [x, 0.0, 2.0]], true);
            let expected = triangles(&source);
            let pointers = (source.positions.as_ptr(), source.normals.as_ptr(),
                source.uvs.as_ptr(), source.colors.as_ptr(), source.indices.as_ptr());
            let mut domains = separate_authentic_domains(source, Some(bounds()));
            assert_eq!(domains.len(), 1);
            let (actual_domain, data) = domains.pop().unwrap();
            assert_eq!(actual_domain, domain);
            assert_eq!(triangles(&data), expected);
            assert_eq!((data.positions.as_ptr(), data.normals.as_ptr(),
                data.uvs.as_ptr(), data.colors.as_ptr(), data.indices.as_ptr()), pointers);
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
            // Attribute storage is allocated once at its exact referenced size.
            assert_eq!(batch.positions.capacity(), batch.positions.len());
            assert_eq!(batch.normals.capacity(), batch.normals.len());
            assert_eq!(batch.uvs.capacity(), batch.uvs.len());
            assert_eq!(batch.colors.capacity(), batch.colors.len());
            assert_eq!(batch.indices.capacity(), batch.indices.len());
            for (key, count) in triangles(batch) {
                *actual.entry(key).or_insert(0) += count;
            }
        }
        assert_eq!(actual, expected);
        assert_eq!(batches, partition(source, Vec2::splat(8.0)));
    }

    #[test]
    fn large_builder_slack_is_released_without_changing_any_surface_data() {
        let mut source = fixture(4);
        let expected = source.clone();
        source.positions.reserve_exact(200_000);
        source.normals.reserve_exact(200_000);
        source.uvs.reserve_exact(200_000);
        source.colors.reserve_exact(200_000);
        source.indices.reserve_exact(400_000);
        source.compact_build_storage();
        assert_eq!(source, expected);
        assert_eq!(source.positions.capacity(), source.positions.len());
        assert_eq!(source.normals.capacity(), source.normals.len());
        assert_eq!(source.uvs.capacity(), source.uvs.len());
        assert_eq!(source.colors.capacity(), source.colors.len());
        assert_eq!(source.indices.capacity(), source.indices.len());
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
            let bounds = batch.clone().into_mesh().get_aabb().unwrap();
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
                        ..default()
                    };
                    let frustum = projection.compute_frustum(&camera);
                    let clip_from_world =
                        projection.get_clip_from_view() * camera.to_matrix().inverse();
                    for offset in [Vec3::ZERO, Vec3::new(-73.5, 0.0, 97.25)] {
                        let world_from_local = GlobalTransform::from_translation(offset);
                        let clip_from_local = clip_from_world * world_from_local.to_matrix();
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
