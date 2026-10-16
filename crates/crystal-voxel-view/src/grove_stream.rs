//! Stream shared forest instances ahead of the visible camera/shadow region.
use crate::{TerrainRevisionCache, VoxelMaterial, mesh::StreamedGroves, occluder_fade};
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::tasks::{AsyncComputeTaskPool, Task, futures_lite::future};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(super) struct Stream {
    source: Arc<StreamedGroves>,
    library: Arc<crate::forest_instances::Library>,
    bounds: Vec<(Vec3, Vec3)>,
    active: HashMap<usize, Entity>,
    visible: HashSet<usize>,
    stable: Option<StableSelection>,
    #[cfg(not(target_arch = "wasm32"))]
    jobs: HashMap<usize, Task<Vec<occluder_fade::PreparedGroup>>>,
}
#[derive(Clone, Copy, PartialEq)]
struct CameraKey {
    matrix: Mat4,
    offset: Vec3,
    sun: Option<Vec3>,
    ground: f32,
}
struct StableSelection {
    key: CameraKey,
    interest: HashSet<usize>,
    camera_count: Option<usize>,
}
impl StableSelection {
    fn for_key(&self, key: CameraKey) -> Option<&HashSet<usize>> {
        (self.key == key).then_some(&self.interest)
    }
}
impl Stream {
    pub(super) fn new(source: Arc<StreamedGroves>, library: Arc<crate::forest_instances::Library>) -> Self {
        let bounds = (0..source.len()).map(|i| source.bounds(i)).collect();
        Self {
            source,
            library,
            bounds,
            active: HashMap::new(),
            visible: HashSet::new(),
            stable: None,
            #[cfg(not(target_arch = "wasm32"))]
            jobs: HashMap::new(),
        }
    }
    pub(super) fn missing_bounds(&self) -> impl Iterator<Item = (Vec3, Vec3)> + '_ {
        self.bounds.iter().enumerate().filter_map(|(index, bounds)|
            (!self.active.contains_key(&index)).then_some(*bounds))
    }
}
fn corners(low: Vec3, high: Vec3) -> [Vec3; 8] {
    std::array::from_fn(|i| {
        Vec3::new(
            if i & 1 == 0 { low.x } else { high.x },
            if i & 2 == 0 { low.y } else { high.y },
            if i & 4 == 0 { low.z } else { high.z },
        )
    })
}
fn visible(low: Vec3, high: Vec3, matrix: Mat4) -> bool {
    visible_points(corners(low, high), matrix)
}
fn visible_points<const N: usize>(corners: [Vec3; N], matrix: Mat4) -> bool {
    visible_points_margin(corners, matrix, 1.15)
}
fn visible_points_margin<const N: usize>(corners: [Vec3; N], matrix: Mat4, margin: f32) -> bool {
    let points = corners.map(|p| matrix * p.extend(1.));
    if points.iter().any(|p| !p.is_finite()) {
        return true;
    }
    // A small view margin avoids edge churn during tile/camera interpolation.
    let rejected = points.iter().all(|p| p.x < -p.w * margin)
        || points.iter().all(|p| p.x > p.w * margin)
        || points.iter().all(|p| p.y < -p.w * margin)
        || points.iter().all(|p| p.y > p.w * margin)
        || points.iter().all(|p| p.z < 0.)
        || points.iter().all(|p| p.z > p.w);
    if rejected { return false; }
    // Plane tests alone admit diagonal boxes/sweeps whose projected bounding
    // rectangle overlaps the viewport while their convex footprint misses it.
    // Perspective projection preserves convexity when every point has w > 0.
    // Keep near-plane crossings conservative rather than dividing through them.
    if N > 16 || points.iter().any(|p| p.w <= 0.) { return true; }
    projected_overlap_margin(points.map(|p| p.truncate().truncate() / p.w), margin)
}

#[cfg(test)]
fn projected_overlap<const N: usize>(points: [Vec2; N]) -> bool {
    projected_overlap_margin(points, 1.15)
}

fn projected_overlap_margin<const N: usize>(mut points: [Vec2; N], margin: f32) -> bool {
    if points.iter().any(|p| !p.is_finite()) { return true; }
    if points.iter().any(|p| p.x.abs() <= margin && p.y.abs() <= margin) { return true; }
    if N < 2 { return false; }
    points.sort_unstable_by(|a,b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    let mut hull = [Vec2::ZERO; 32];
    let mut len = 0;
    for p in points {
        while len >= 2 && (hull[len-1]-hull[len-2]).perp_dot(p-hull[len-1]) <= 0. { len -= 1; }
        hull[len] = p; len += 1;
    }
    let lower = len;
    for &p in points[..N-1].iter().rev() {
        while len > lower && (hull[len-1]-hull[len-2]).perp_dot(p-hull[len-1]) <= 0. { len -= 1; }
        hull[len] = p; len += 1;
    }
    len -= 1;
    for i in 0..len {
        let edge = hull[(i+1)%len] - hull[i];
        let axis = Vec2::new(-edge.y, edge.x);
        let radius = margin * (axis.x.abs()+axis.y.abs());
        let (low,high) = hull[..len].iter().map(|p|p.dot(axis))
            .fold((f32::INFINITY,f32::NEG_INFINITY),|(lo,hi),v|(lo.min(v),hi.max(v)));
        let tolerance = 1e-5 * radius.max(low.abs()).max(high.abs()).max(1.);
        if low > radius+tolerance || high < -radius-tolerance { return false; }
    }
    true
}
#[cfg(test)]
fn needed(low: Vec3, high: Vec3, matrix: Mat4, sun: Option<Vec3>, ground: f32) -> bool {
    needed_margin(low, high, matrix, sun, ground, 1.15)
}
fn needed_margin(low: Vec3, high: Vec3, matrix: Mat4, sun: Option<Vec3>, ground: f32, margin: f32) -> bool {
    if visible_points_margin(corners(low, high), matrix, margin) {
        return true;
    }
    let Some(sun) = sun else {
        return false;
    };
    if sun.y <= 0.001 {
        return true;
    }
    // Include off-camera casters whose full shadow can reach visible ground.
    // Project the complete conservative tree box toward the lowest support
    // plane; this never substitutes lower-detail geometry or proxy shadows.
    let source = corners(low, high);
    let swept = std::array::from_fn::<_, 16, _>(|i| {
        let p = source[i % 8];
        if i < 8 { p } else { p - sun * ((p.y - ground) / sun.y) }
    });
    // Test the swept box's actual support points. Enclosing that sweep in
    // another axis-aligned box invents high, distant corners and selects
    // offscreen trees whose shadow volume never enters the camera frustum.
    visible_points_margin(swept, matrix, margin)
}
pub(super) fn sync(
    frame: &crystal_render_api::VisualWorldFrame,
    matrix: Mat4,
    sun: Option<Vec3>,
    cache: &mut TerrainRevisionCache,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
) -> bool {
    let (Some(_built), Some(_), Some(_)) = (
        &cache.built_frame,
        cache.instances_root,
        cache.solid_material.as_ref(),
    ) else {
        return false;
    };
    let offset = crate::retained_terrain_transform(frame, cache)
        .map_or(Vec3::ZERO, |transform| transform.translation);
    let ground = cache
        .built_footing_heights
        .iter()
        .copied()
        .filter(|h| h.is_finite())
        .fold(0., f32::min)
        - frame.tile_size.y;
    let Some(stream) = &mut cache.groves else {
        return false;
    };
    let _trace = crate::loading_trace::span("region.stream_visible_groves", || {
        serde_json::json!({
        "map":frame.map_id.as_ref(),"available":stream.source.len(),"resident":stream.active.len()})
    });
    let key = CameraKey { matrix, offset, sun, ground };
    let cached = stream.stable.as_ref().and_then(|selection| selection.for_key(key));
    let cache_hit = cached.is_some();
    // Prepare ahead of the view, and retain a wider band to avoid destroying
    // and rebuilding exact meshes when the camera reverses near an edge.
    let interest = cached.cloned().unwrap_or_else(|| stream.bounds.iter().enumerate()
        .filter_map(|(i, (low, high))| {
            let margin = if stream.active.contains_key(&i) { 1.45 } else { 1.30 };
            needed_margin(*low + offset, *high + offset, matrix, sun, ground, margin).then_some(i)
        })
        .collect());
    let mut camera_count = if cache_hit {
        stream.stable.as_ref().and_then(|selection| selection.camera_count)
    } else { None };
    let _interest_trace = crate::loading_trace::span("region.grove_interest", || {
        let camera = *camera_count.get_or_insert_with(|| stream.bounds.iter()
            .filter(|(low, high)| visible(*low + offset, *high + offset, matrix)).count());
        serde_json::json!({
        "map":frame.map_id.as_ref(),"available":stream.source.len(),"requested":interest.len(),"camera":camera,"sun":sun.map(|v| v.to_array()),"ground":ground,"resident":stream.active.len(),"selection_cache_hit":cache_hit,
        "view_projection":matrix.to_cols_array(),"source_offset":offset.to_array()})
    });
    let pending = sync_interest(cache, interest, commands, meshes, materials, |low, high|
        needed_margin(low+offset, high+offset, matrix, sun, ground, 1.45),
        |low, high| camera_priority(low + offset, high + offset, matrix));
    let stream = cache.groves.as_mut().unwrap();
    // Preloading retains assets, not extra draw/shadow work. Only the original
    // camera/shadow footprint is visible; ready outer-band trees stay hidden.
    for (&index, &entity) in &stream.active {
        let (low, high) = stream.bounds[index];
        let draw = needed_margin(low + offset, high + offset, matrix, sun, ground, 1.15);
        if draw != stream.visible.contains(&index) {
            commands.entity(entity).insert(if draw { Visibility::Inherited } else { Visibility::Hidden });
            if draw { stream.visible.insert(index); } else { stream.visible.remove(&index); }
        }
    }
    if pending {
        stream.stable = None;
    } else if !cache_hit {
        // Preparation can tighten conservative recipe bounds. Publish only
        // the final active set after every requested object has settled.
        if camera_count.is_some() {
            camera_count = Some(stream.bounds.iter()
                .filter(|(low, high)| visible(*low + offset, *high + offset, matrix)).count());
        }
        stream.stable = Some(StableSelection {
            key, interest: stream.active.keys().copied().collect(), camera_count,
        });
    } else if let Some(selection) = &mut stream.stable {
        selection.camera_count = camera_count;
    }
    pending
}

/// Warm the existing source grove recipes for the authentic battle neighborhood,
/// including trees outside the old overhead camera. Uses the same bounded jobs
/// and exact models as normal play; no alternate scenery is assembled.
pub(super) fn warm_battle(
    cache: &mut TerrainRevisionCache,
    area: Rect,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
) -> bool {
    if let Some(stream) = &mut cache.groves { stream.stable = None; }
    let Some(stream) = &cache.groves else { return false; };
    if cache.instances_root.is_none() || cache.solid_material.is_none() { return true; }
    let interest = battle_interest(&stream.bounds, area);
    let pending = sync_interest(cache, interest, commands, meshes, materials, |low, high|
        high.x >= area.min.x && low.x <= area.max.x && high.z >= area.min.y && low.z <= area.max.y,
        |low, high| ((low + high).xz() * 0.5 - area.center()).length_squared());
    // The battle camera needs the entire selected neighborhood, including
    // preloaded trees hidden from the normal overhead camera.
    let stream = cache.groves.as_mut().unwrap();
    for (&index, &entity) in &stream.active {
        if stream.visible.insert(index) {
            commands.entity(entity).insert(Visibility::Inherited);
        }
    }
    pending
}

fn battle_interest(bounds: &[(Vec3, Vec3)], area: Rect) -> HashSet<usize> {
    bounds.iter().enumerate().filter_map(|(index, (low, high))| {
        (high.x >= area.min.x && low.x <= area.max.x
            && high.z >= area.min.y && low.z <= area.max.y).then_some(index)
    }).collect()
}

// Rank by the camera's projected footprint, independently of map coordinates.
// Viewport trees precede the preload band even when their box centers lie outside.
fn camera_priority(low: Vec3, high: Vec3, matrix: Mat4) -> f32 {
    let center = matrix * ((low + high) * 0.5).extend(1.);
    let distance = if center.is_finite() && center.w > 0. {
        (center.truncate().truncate() / center.w).length_squared().min(1e6)
    } else { 1e6 };
    let band = if visible(low, high, matrix) { 0. } else { 2e6 };
    band + distance
}

#[cfg(not(target_arch = "wasm32"))]
const CACHED_PUBLICATION_BUDGET: std::time::Duration = std::time::Duration::from_millis(4);
#[cfg(not(target_arch = "wasm32"))]
const MAX_CACHED_PUBLICATIONS: usize = 64;

#[cfg(not(target_arch = "wasm32"))]
fn cached_publication_allowed(elapsed: std::time::Duration, published: usize) -> bool {
    elapsed < CACHED_PUBLICATION_BUDGET && published < MAX_CACHED_PUBLICATIONS
}

#[cfg(not(target_arch = "wasm32"))]
fn publish_native_instance(stream: &mut Stream, index: usize,
    groups: Vec<occluder_fade::PreparedGroup>, root: Entity, opaque: &Handle<VoxelMaterial>,
    commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<VoxelMaterial>,
    needed_now: &impl Fn(Vec3, Vec3) -> bool) -> bool {
    if let Some(bounds) = occluder_fade::prepared_bounds(&groups) {
        stream.bounds[index] = bounds;
        if !needed_now(bounds.0, bounds.1) { return false; }
    }
    let tree = commands.spawn((Transform::default(), Visibility::default())).id();
    commands.entity(root).add_child(tree);
    occluder_fade::spawn_groups(commands, meshes, materials, tree, groups, opaque.clone(), None);
    stream.active.insert(index, tree);
    stream.visible.insert(index);
    true
}

fn sync_interest(
    cache: &mut TerrainRevisionCache,
    mut interest: HashSet<usize>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<VoxelMaterial>,
    needed_now: impl Fn(Vec3, Vec3) -> bool,
    priority: impl Fn(Vec3, Vec3) -> f32,
) -> bool {
    let (Some(root), Some(opaque)) = (cache.instances_root, cache.solid_material.clone()) else { return false; };
    let Some(stream) = &mut cache.groves else { return false; };
    let retire: Vec<_> = stream
        .active
        .keys()
        .copied()
        .filter(|i| !interest.contains(i))
        .collect();
    for index in retire {
        stream.visible.remove(&index);
        commands
            .entity(stream.active.remove(&index).unwrap())
            .despawn();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let publication_started = bevy::platform::time::Instant::now();
        let pool = AsyncComputeTaskPool::get();
        // Match the pool that actually executes these jobs. Four queued trees
        // underutilized larger native pools while oversubscribing tiny ones.
        let cold_job_limit = pool.thread_num().clamp(1,64);
        stream.jobs.retain(|index, _| interest.contains(index));
        let mut ready = Vec::new();
        let mut queued: Vec<_> = stream.jobs.keys().copied().collect();
        queued.sort_unstable_by(|&a, &b| priority(stream.bounds[a].0, stream.bounds[a].1)
            .total_cmp(&priority(stream.bounds[b].0, stream.bounds[b].1)).then(a.cmp(&b)));
        for index in queued {
            let task = stream.jobs.get_mut(&index).unwrap();
            if let Some(groups) = future::block_on(future::poll_once(task)) {
                ready.push((index, groups));
                if ready.len() == 2 { break; }
            }
        }
        for (index, groups) in ready {
            stream.jobs.remove(&index);
            if !publish_native_instance(stream,index,groups,root,&opaque,commands,meshes,
                materials,&needed_now) { interest.remove(&index); }
        }
        // Completed shared prototypes need no worker or geometry build. The
        // old two-tree cap made a newly exposed 285-tree view take 143 frames,
        // even when all its buffers were already shared and ready. Publish
        // cached placements in camera order under a measured CPU budget.
        let mut candidates: Vec<_> = interest.iter().copied()
            .filter(|i| !stream.active.contains_key(i)).collect();
        candidates.sort_unstable_by(|&a,&b| priority(stream.bounds[a].0,stream.bounds[a].1)
            .total_cmp(&priority(stream.bounds[b].0,stream.bounds[b].1)).then(a.cmp(&b)));
        let mut cached_published = 0;
        for index in candidates {
            if !cached_publication_allowed(publication_started.elapsed(), cached_published) { break; }
            let Some(groups) = stream.source.prepare_if_cached(index,&stream.library) else { continue; };
            stream.jobs.remove(&index);
            if publish_native_instance(stream,index,groups,root,&opaque,commands,meshes,
                materials,&needed_now) { cached_published += 1; }
            else { interest.remove(&index); }
        }
        drop(crate::loading_trace::span("region.grove_cached_publication", || serde_json::json!({
            "published":cached_published,"elapsed_ms":publication_started.elapsed().as_secs_f64()*1000.,
            "budget_ms":CACHED_PUBLICATION_BUDGET.as_secs_f64()*1000.,
            "cold_workers":pool.thread_num(),"cold_job_limit":cold_job_limit})));
        let mut missing: Vec<_> = interest
            .iter()
            .copied()
            .filter(|i| !stream.active.contains_key(i) && !stream.jobs.contains_key(i))
            .collect();
        missing.sort_unstable_by(|&a, &b| {
            priority(stream.bounds[a].0, stream.bounds[a].1)
                .total_cmp(&priority(stream.bounds[b].0, stream.bounds[b].1))
                .then(a.cmp(&b))
        });
        for index in missing
            .into_iter()
            .take(cold_job_limit.saturating_sub(stream.jobs.len()))
        {
            let source = stream.source.clone();
            let library = stream.library.clone();
            stream.jobs.insert(
                index,
                pool.spawn(async move { source.prepare(index, &library) }),
            );
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let start = bevy::platform::time::Instant::now();
        let mut missing: Vec<_> = interest
            .iter()
            .copied()
            .filter(|i| !stream.active.contains_key(i))
            .collect();
        missing.sort_unstable_by(|&a, &b| {
            priority(stream.bounds[a].0, stream.bounds[a].1)
                .total_cmp(&priority(stream.bounds[b].0, stream.bounds[b].1))
                .then(a.cmp(&b))
        });
        for index in missing {
            let groups = stream.source.prepare(index, &stream.library);
            if let Some(bounds) = occluder_fade::prepared_bounds(&groups) { stream.bounds[index] = bounds; }
            let bounds = stream.bounds[index];
            if needed_now(bounds.0, bounds.1) {
                let tree = commands.spawn((Transform::default(), Visibility::default())).id();
                commands.entity(root).add_child(tree);
                occluder_fade::spawn_groups(commands, meshes, materials, tree, groups, opaque.clone(), None);
                stream.active.insert(index, tree);
            stream.visible.insert(index);
            } else { interest.remove(&index); }
            if start.elapsed().as_secs_f64() >= 0.004 {
                break;
            }
        }
    }
    interest.iter().any(|i| !stream.active.contains_key(i))
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cached_publication_stops_at_cpu_deadline_and_has_a_finite_fast_path_cap() {
        assert!(cached_publication_allowed(std::time::Duration::ZERO,0));
        assert!(cached_publication_allowed(CACHED_PUBLICATION_BUDGET
            - std::time::Duration::from_nanos(1),MAX_CACHED_PUBLICATIONS-1));
        assert!(!cached_publication_allowed(CACHED_PUBLICATION_BUDGET,0));
        assert!(!cached_publication_allowed(std::time::Duration::ZERO,MAX_CACHED_PUBLICATIONS));
        assert!(!cached_publication_allowed(CACHED_PUBLICATION_BUDGET*2,0));
    }

    #[test]
    fn preload_and_retention_bands_surround_the_visible_camera() {
        let matrix = Mat4::orthographic_rh(-10., 10., -10., 10., 0.1, 100.)
            * Mat4::look_at_rh(Vec3::new(0., 0., 20.), Vec3::ZERO, Vec3::Y);
        let bounds = |x| (Vec3::new(x, -0.1, -0.1), Vec3::new(x + 0.1, 0.1, 0.1));
        let (lo, hi) = bounds(12.);
        assert!(!visible(lo, hi, matrix));
        assert!(needed_margin(lo, hi, matrix, None, 0., 1.30));
        let (lo, hi) = bounds(14.);
        assert!(!needed_margin(lo, hi, matrix, None, 0., 1.30));
        assert!(needed_margin(lo, hi, matrix, None, 0., 1.45));
        let (lo, hi) = bounds(16.);
        assert!(!needed_margin(lo, hi, matrix, None, 0., 1.45));
    }

    #[test]
    fn queue_priority_follows_camera_instead_of_map_origin() {
        let matrix = Mat4::orthographic_rh(-10., 10., -10., 10., 0.1, 100.)
            * Mat4::look_at_rh(Vec3::new(100., 0., 20.), Vec3::new(100., 0., 0.), Vec3::Y);
        let priority = |x| camera_priority(Vec3::new(x, -1., -1.), Vec3::new(x + 1., 1., 1.), matrix);
        assert!(priority(100.) < priority(90.));
        assert!(priority(90.) < priority(86.));
        assert!(priority(100.) < priority(0.));
    }

    #[test]
    fn completed_selection_reuses_visibility_and_invalidates_every_camera_dependency() {
        let matrix = Mat4::orthographic_rh(-10., 10., -10., 10., 0.1, 100.)
            * Mat4::look_at_rh(Vec3::new(0., 20., 20.), Vec3::ZERO, Vec3::Y);
        let key = CameraKey { matrix, offset: Vec3::ZERO,
            sun: Some(Vec3::new(1., 1., 0.).normalize()), ground: 0. };
        let bounds = [(Vec3::splat(-1.), Vec3::splat(1.)),
            (Vec3::new(14., 0., -1.), Vec3::new(16., 8., 1.))];
        let interest: HashSet<_> = bounds.iter().enumerate().filter_map(|(i,(lo,hi))|
            needed(*lo, *hi, key.matrix, key.sun, key.ground).then_some(i)).collect();
        assert_eq!(interest, HashSet::from([0, 1]), "retain the off-camera shadow caster");
        let selection = StableSelection { key, interest, camera_count: Some(1) };
        assert_eq!(selection.for_key(key), Some(&selection.interest));
        for changed in [CameraKey { matrix: Mat4::IDENTITY, ..key },
            CameraKey { offset: Vec3::X, ..key }, CameraKey { sun: None, ..key },
            CameraKey { ground: -32., ..key }] {
            assert!(selection.for_key(changed).is_none(), "changed visibility inputs must recompute");
        }
    }
    #[test]
    fn projected_diagonal_sweep_outside_view_is_not_resident() {
        let points = [Vec2::new(1.1,3.),Vec2::new(3.,1.1),Vec2::new(4.,2.1),Vec2::new(2.1,4.)];
        // Its axis-aligned projection overlaps the upper-right viewport corner,
        // but every support point lies beyond the diagonal x+y=4.1.
        assert!(!projected_overlap(points));
    }
    #[test]
    fn projected_coverage_without_inside_corners_is_retained() {
        assert!(projected_overlap([Vec2::new(-3.,-3.),Vec2::new(3.,-3.),Vec2::new(3.,3.),Vec2::new(-3.,3.)]));
        assert!(projected_overlap([Vec2::new(-2.,0.),Vec2::new(2.,0.)]));
        assert!(projected_overlap([Vec2::new(1.15,0.),Vec2::new(2.,1.),Vec2::new(2.,-1.)]));
    }
    #[test]
    fn near_plane_crossing_keeps_conservative_residency() {
        let projection = Mat4::perspective_rh(1.0,1.0,0.1,100.);
        assert!(visible(Vec3::new(-1.,-1.,-1.),Vec3::new(1.,1.,1.),projection));
    }
    use super::*;
    #[test]
    fn battle_neighborhood_includes_real_trees_hidden_from_the_overhead_camera() {
        let matrix = Mat4::orthographic_rh(-10., 10., -10., 10., 0.1, 100.)
            * Mat4::look_at_rh(Vec3::new(0., 20., 20.), Vec3::ZERO, Vec3::Y);
        let bounds = [
            (Vec3::splat(-1.), Vec3::splat(1.)),
            (Vec3::new(14., 0., -1.), Vec3::new(16., 8., 1.)),
            (Vec3::new(200., 0., 200.), Vec3::new(204., 8., 204.)),
        ];
        assert!(!visible(bounds[1].0, bounds[1].1, matrix));
        assert_eq!(battle_interest(&bounds, Rect::from_corners(Vec2::new(-20., -10.), Vec2::new(20., 10.))),
            HashSet::from([0, 1]));
    }
    #[test]
    fn camera_interest_excludes_remote_trees_and_keeps_shadow_casters() {
        let matrix = Mat4::orthographic_rh(-10., 10., -10., 10., 0.1, 100.)
            * Mat4::look_at_rh(Vec3::new(0., 20., 20.), Vec3::ZERO, Vec3::Y);
        assert!(needed(Vec3::splat(-1.), Vec3::splat(1.), matrix, None, 1.));
        assert!(!needed(
            Vec3::new(100., 0., 100.),
            Vec3::new(101., 5., 101.),
            matrix,
            None,
            1.
        ));
        assert!(needed(
            Vec3::new(20., 0., 0.),
            Vec3::new(22., 30., 2.),
            matrix,
            Some(Vec3::new(1., 1., 0.)),
            1.
        ));
    }
}
