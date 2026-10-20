//! Conservative camera-receiver shadow culling. No cascade or mesh changes.
//! Verified native outdoor maps use the filter; browser remains opt-in.
use crate::{VoxelMaterial, VoxelSun, VoxelWorldCamera, VOXEL_RENDER_LAYER};
use bevy::{
    camera::{
        primitives::{Aabb, Frustum},
        visibility::{CascadesVisibleEntities, RenderLayers},
    },
    light::ShadowFilteringMethod,
    math::DVec3,
    prelude::*,
};
use crystal_render_api::VisualWorldFrame;
use std::{collections::HashMap, sync::OnceLock};

static DIAGNOSTIC_ENABLED: OnceLock<bool> = OnceLock::new();

/// Startup-only QA control for same-build shadow comparisons. It never enables
/// reduced geometry, different cascades or a different shadow filtering method.
pub fn configure_shadow_receiver_diagnostics(enabled: bool) -> Result<(), String> {
    DIAGNOSTIC_ENABLED.set(enabled)
        .map_err(|_| "shadow receiver diagnostics already configured".to_owned())
}

fn enabled() -> bool {
    DIAGNOSTIC_ENABLED.get().copied().unwrap_or_else(|| {
        #[cfg(not(target_arch = "wasm32"))]
        { std::env::var("CRYSTAL_SHADOW_RECEIVERS").as_deref() != Ok("off") }
        #[cfg(target_arch = "wasm32")]
        { false }
    })
}

fn trace_enabled() -> bool {
    static TRACE: OnceLock<bool> = OnceLock::new();
    *TRACE.get_or_init(|| {
        #[cfg(not(target_arch = "wasm32"))]
        { std::env::var_os("CRYSTAL_SHADOW_RECEIVER_TRACE").is_some() }
        #[cfg(target_arch = "wasm32")]
        { false }
    })
}

// Gaussian uses a fixed 5x5 footprint; Hardware2x2 is smaller. Temporal uses
// a texel-size-dependent randomized radius and needs its own proof bound.
fn supported_filter(filter: Option<&ShadowFilteringMethod>) -> bool {
    matches!(filter.copied().unwrap_or_default(),
        ShadowFilteringMethod::Hardware2x2 | ShadowFilteringMethod::Gaussian)
}

fn corners(bounds: &Aabb, transform: &GlobalTransform) -> [Vec3; 8] {
    std::array::from_fn(|i| {
        let sign = Vec3::new(
            if i & 1 == 0 { -1.0 } else { 1.0 },
            if i & 2 == 0 { -1.0 } else { 1.0 },
            if i & 4 == 0 { -1.0 } else { 1.0 },
        );
        transform.affine().transform_point3(
            Vec3::from(bounds.center) + Vec3::from(bounds.half_extents) * sign)
    })
}

fn proven_empty(mesh: Option<&Mesh>) -> bool {
    mesh.is_some_and(|mesh| {
        mesh.try_attributes().is_ok_and(|mut attributes|
            attributes.all(|(_, values)| values.is_empty()))
            && mesh.try_indices_option().is_ok_and(|indices|
                indices.is_none_or(|indices| indices.is_empty()))
    })
}

#[derive(Clone, Copy, PartialEq)]
struct ReceiverInput {
    planes: [[f64; 4]; 8],
    sun: DVec3,
    bottom: f64,
    top: f64,
}

struct Decision {
    points: [Vec3; 8],
    keep: bool,
    seen: u64,
}
#[derive(Default)]
struct CachedCascade {
    input: Option<ReceiverInput>,
    decisions: HashMap<Entity, Decision>,
    seen: u64,
}
impl CachedCascade {
    fn set_input(&mut self, input: ReceiverInput, frame: u64) {
        if self.input != Some(input) {
            self.decisions.clear();
            self.input = Some(input);
        }
        self.seen = frame;
    }
    fn keep(&mut self, entity: Entity, points: [Vec3; 8], frame: u64) -> (bool, bool) {
        if let Some(previous) = self.decisions.get_mut(&entity) {
            if previous.points == points {
                previous.seen = frame;
                return (previous.keep, true);
            }
        }
        let input = self.input.expect("receiver inputs precede caster decisions");
        let keep = may_reach(points, input.planes, input.sun, input.bottom, input.top);
        self.decisions.insert(
            entity,
            Decision {
                points,
                keep,
                seen: frame,
            },
        );
        (keep, false)
    }
    fn finish(&mut self, frame: u64) {
        self.decisions.retain(|_, decision| decision.seen == frame);
    }
}
#[derive(Default)]
struct DecisionCache {
    frame: u64,
    cascades: HashMap<(Entity, Entity, usize), CachedCascade>,
}

pub(super) fn register(app: &mut App) {
    if !enabled() {
        return;
    }
    app.add_systems(
        PostUpdate,
        prune.after(bevy::light::SimulationLightSystems::CheckLightVisibility),
    );
}

// A shadow receiver p and its caster q satisfy q = p + t * toward_sun,
// t >= 0. For each camera plane n.p+d >= 0, a necessary condition is
// max_box(n.q+d) - t*n.toward_sun >= 0. Intersect these intervals with the
// complete scene's receiver-height slab. Independent plane support maxima
// intentionally allow false positives; an empty interval proves no receiver.
fn may_reach<const N: usize>(
    points: [Vec3; 8],
    planes: [[f64; 4]; N],
    sun: DVec3,
    bottom: f64,
    top: f64,
) -> bool {
    if !sun.is_finite()
        || sun.y <= 0.001
        || !bottom.is_finite()
        || !top.is_finite()
        || bottom > top
        || points.iter().any(|p| !p.is_finite())
        || planes.iter().flatten().any(|v| !v.is_finite())
    {
        return true;
    }
    // Inflate all constraints, including height, to keep near-boundary casters.
    const MARGIN: f64 = 0.1;
    let minimum_y = points
        .iter()
        .map(|p| f64::from(p.y))
        .fold(f64::INFINITY, f64::min);
    let maximum_y = points
        .iter()
        .map(|p| f64::from(p.y))
        .fold(f64::NEG_INFINITY, f64::max);
    let mut low = 0.0_f64.max((minimum_y - top - MARGIN) / sun.y);
    let mut high = (maximum_y - bottom + MARGIN) / sun.y;
    for [x, y, z, d] in planes {
        let normal = DVec3::new(x, y, z);
        let support = points
            .iter()
            .map(|p| normal.dot(p.as_dvec3()) + d)
            .fold(f64::NEG_INFINITY, f64::max)
            + MARGIN;
        let slope = normal.dot(sun);
        if slope > 0.0 {
            high = high.min(support / slope);
        } else if slope < 0.0 {
            low = low.max(support / slope);
        } else if support < 0.0 {
            return false;
        }
    }
    low <= high
}

// Match Bevy's cascade selection and next-cascade overlap sampling. Shadow
// projections are unchanged; these planes restrict only possible receivers.
fn cascade_planes(
    base: [[f64; 4]; 6],
    depth: [f64; 4],
    near: f32,
    far: f32,
    padding: f64,
) -> Option<[[f64; 4]; 8]> {
    if !near.is_finite()
        || !far.is_finite()
        || near < 0.0
        || far < near
        || !padding.is_finite()
        || padding < 0.0
        || depth.iter().any(|v| !v.is_finite())
    {
        return None;
    }
    let [x, y, z, d] = depth;
    let mut planes = [[0.0; 4]; 8];
    planes[..6].copy_from_slice(&base);
    planes[6] = [x, y, z, d - f64::from(near)];
    planes[7] = [-x, -y, -z, f64::from(far) - d];
    Some(planes.map(|[x, y, z, d]| [x, y, z, d + padding * DVec3::new(x, y, z).length()]))
}

fn supported_map(map: &str) -> bool {
    matches!(map, "NewBarkTown" | "Route29" | "CherrygroveCity")
}

fn prune(
    frame: Res<VisualWorldFrame>,
    cameras: Query<(Entity, &Camera, &Frustum, &GlobalTransform, Option<&ShadowFilteringMethod>), With<VoxelWorldCamera>>,
    receivers: Query<(
        &Mesh3d,
        Option<&Aabb>,
        &GlobalTransform,
        Option<&RenderLayers>,
        &InheritedVisibility,
        Has<bevy::mesh::morph::MeshMorphWeights>,
        Has<bevy::light::NotShadowReceiver>,
        Has<MeshMaterial3d<VoxelMaterial>>,
    )>,
    meshes: Res<Assets<Mesh>>,
    materials: Option<Res<Assets<VoxelMaterial>>>,
    mut lights: Query<
        (
            Entity,
            &GlobalTransform,
            &DirectionalLight,
            &bevy::light::Cascades,
            &bevy::light::CascadeShadowConfig,
            &mut CascadesVisibleEntities,
        ),
        With<VoxelSun>,
    >,
    mut next_report: Local<f64>,
    time: Res<Time<Real>>,
    mut cache: Local<DecisionCache>,
) {
    if !frame.active || !supported_map(&frame.map_id) {
        cache.cascades.clear();
        return;
    }
    let Ok((camera_entity, camera, frustum, camera_transform, filter)) = cameras.single() else {
        return;
    };
    if !camera.is_active || !supported_filter(filter) {
        cache.cascades.clear();
        return;
    }
    if !lights.iter().any(|(_, _, light, _, _, _)| light.shadow_maps_enabled) {
        cache.cascades.clear();
        return;
    }
    let _trace = crate::loading_trace::span("render.shadow_receivers", || serde_json::json!({
        "map": frame.map_id.as_ref(),
    }));
    // The rooted-grass vertex shader can move a receiver outside its stored
    // Aabb. Until those meshes publish swept bounds, keep stock shadow lists
    // throughout the spring, including its rebound after the source card ends.
    if materials.as_ref().is_some_and(|materials| materials.iter().any(|(_, material)| {
        let contact = material.extension.grass_rustle.contact;
        !contact.is_finite() || (contact.z > 0.0 && contact.w != 0.0)
    })) {
        cache.cascades.clear();
        return;
    }
    let layer = RenderLayers::layer(VOXEL_RENDER_LAYER);
    #[cfg(not(target_arch = "wasm32"))]
    let started = trace_enabled().then(std::time::Instant::now);
    cache.frame = cache.frame.wrapping_add(1);
    let stamp = cache.frame;
    let mut bottom = f64::INFINITY;
    let mut top = f64::NEG_INFINITY;
    for (mesh, bounds, transform, layers, visible, morph, not_receiver, voxel) in &receivers {
        if !visible.get() || !layers.unwrap_or_default().intersects(&layer) {
            continue;
        }
        // Morph bounds are not guaranteed to include the active target.
        // Skinned production actors explicitly provide animated sweep bounds.
        if morph { return; }
        // Sky and standard nonreceivers cannot establish receiver heights.
        // VoxelSurface samples shadows explicitly, so retain its geometry
        // conservatively even if an owner attaches NotShadowReceiver.
        if not_receiver && !voxel { continue; }
        let Some(bounds) = bounds else {
            if proven_empty(meshes.get(&mesh.0)) {
                continue;
            }
            return; // Unknown geometry cannot prove a global receiver-height bound.
        };
        for point in corners(bounds, transform) {
            if !point.is_finite() { return; }
            bottom = bottom.min(f64::from(point.y));
            top = top.max(f64::from(point.y));
        }
    }
    let base_planes = frustum
        .half_spaces
        .map(|p| p.normal_d().to_array().map(f64::from));
    let mut before = 0;
    let view_from_world = camera_transform.to_matrix().inverse();
    if !view_from_world.is_finite() {
        return;
    }
    let depth = (-view_from_world.row(2)).to_array().map(f64::from);
    let mut after = 0;
    let mut hits = 0;
    let mut misses = 0;
    for (light_entity, transform, light, configs, settings, mut visible) in &mut lights {
        // This workspace does not enable experimental_pbr_pcss. An adaptive
        // PCSS footprint would require a separate bound before using this filter.
        if !light.shadow_maps_enabled {
            continue;
        }
        let Some(configs) = configs.cascades.get(&camera_entity) else {
            continue;
        };
        let sun = (*transform.back()).as_dvec3();
        let Some(cascades) = visible.entities.get_mut(&camera_entity) else {
            continue;
        };
        if configs.len() != cascades.len()
            || settings.bounds.len() != cascades.len()
            || !settings.overlap_proportion.is_finite()
            || !(0.0..=1.0).contains(&settings.overlap_proportion)
        {
            continue;
        }
        for (index, cascade) in cascades.iter_mut().enumerate() {
            let texel = f64::from(configs[index].texel_size);
            if !texel.is_finite() || texel < 0.0 {
                continue;
            }
            // Include this cascade's receiver bias, fixed filter and raster edges.
            let padding = texel
                * (f64::from(light.shadow_normal_bias).abs() * 2.0_f64.sqrt() + 8.0)
                + f64::from(light.shadow_depth_bias).abs();
            let near = if index == 0 {
                0.0
            } else {
                (1.0 - settings.overlap_proportion) * settings.bounds[index - 1]
            };
            let Some(planes) =
                cascade_planes(base_planes, depth, near, settings.bounds[index], padding)
            else {
                continue;
            };
            let cached = cache
                .cascades
                .entry((light_entity, camera_entity, index))
                .or_default();
            cached.set_input(
                ReceiverInput {
                    planes,
                    sun,
                    bottom: bottom - padding,
                    top: top + padding,
                },
                stamp,
            );
            before += cascade.entities.len();
            cascade.entities.retain(|entity| {
                let Ok((_, Some(bounds), transform, _, _, _, _, _)) = receivers.get(*entity) else {
                    return true;
                };
                let (keep, hit) =
                    cached.keep(*entity, corners(bounds, transform), stamp);
                if hit {
                    hits += 1;
                } else {
                    misses += 1;
                }
                keep
            });
            cached.finish(stamp);
            after += cascade.entities.len();
        }
    }
    cache.cascades.retain(|_, cascade| cascade.seen == stamp);
    // Bevy allocates cascades per camera. Only the exact world-camera key
    // above was changed; retained battle and image-capture views retain their
    // own caster lists even when they happen to share the same directional sun.
    drop(crate::loading_trace::span("shadow.receivers", || serde_json::json!({
        "before": before, "after": after, "cache_hits": hits, "cache_misses": misses,
    })));
    let now = time.elapsed_secs_f64();
    if trace_enabled() && now >= *next_report {
        *next_report = now + 2.0;
        #[cfg(not(target_arch = "wasm32"))]
        eprintln!(
            "shadow_receiver_culling map={} before={} after={} cache_hits={} cache_misses={} system_ms={:.3}",
            frame.map_id, before, after, hits, misses, started.map_or(0., |started| started.elapsed().as_secs_f64() * 1000.0)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn planes() -> [[f64; 4]; 6] {
        [
            [1., 0., 0., 1.],
            [-1., 0., 0., 1.],
            [0., 1., 0., 1.],
            [0., -1., 0., 1.],
            [0., 0., 1., 1.],
            [0., 0., -1., 1.],
        ]
    }
    struct Fixture {
        app: App,
        sun: Entity,
        camera: Entity,
        capture_camera: Entity,
        reachable: Entity,
        offscreen: Entity,
    }

    fn fixture() -> Fixture {
        use bevy::camera::visibility::VisibleMeshEntities;
        let mut app = App::new();
        app.insert_resource(VisualWorldFrame { active: true, map_id: "Route29".into(), ..default() })
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Time::<Real>::default())
            .add_systems(Update, prune);
        let mut frustum = Frustum::default();
        frustum.half_spaces = planes().map(|p|
            bevy::shape::HalfSpace::new(Vec4::from_array(p.map(|v| v as f32))));
        let camera = app.world_mut().spawn((Camera::default(), frustum,
            GlobalTransform::IDENTITY, VoxelWorldCamera)).id();
        let capture_camera = app.world_mut().spawn(Camera::default()).id();
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::default()));
        let spawn = |app: &mut App, position| app.world_mut().spawn((
            Mesh3d(mesh.clone()), Aabb::from_min_max(Vec3::splat(-0.1), Vec3::splat(0.1)),
            GlobalTransform::from_translation(position),
            InheritedVisibility::VISIBLE, RenderLayers::layer(VOXEL_RENDER_LAYER),
        )).id();
        // A visible ground receiver is below the offscreen overhead caster.
        spawn(&mut app, Vec3::new(0.0, 0.0, -0.5));
        let reachable = spawn(&mut app, Vec3::new(0.0, 10.0, -0.5));
        let offscreen = spawn(&mut app, Vec3::new(100.0, 10.0, -0.5));
        let mut cascades = bevy::light::Cascades::default();
        for camera in [camera, capture_camera] {
            cascades.cascades.insert(camera, vec![bevy::light::cascade::Cascade { texel_size: 0.01, ..default() }]);
        }
        let mut visible = CascadesVisibleEntities::default();
        for camera in [camera, capture_camera] {
            visible.entities.insert(camera, vec![VisibleMeshEntities { entities: vec![reachable, offscreen] }]);
        }
        let sun = app.world_mut().spawn((VoxelSun, DirectionalLight {
            shadow_maps_enabled: true, shadow_normal_bias: 0.0, shadow_depth_bias: 0.0, ..default()
        }, GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))),
            bevy::light::CascadeShadowConfigBuilder { num_cascades: 1, maximum_distance: 2.0, ..default() }.build(),
            cascades, visible)).id();
        Fixture { app, sun, camera, capture_camera, reachable, offscreen }
    }

    #[test]
    fn cherrygrove_uses_the_same_receiver_proof_and_unsupported_maps_keep_stock_lists() {
        for (map, pruned) in [("CherrygroveCity", true), ("PlayersHouse1F", false),
            ("IlexForest", false), ("Route30", false)] {
            let mut f = fixture();
            f.app.world_mut().resource_mut::<VisualWorldFrame>().map_id = map.into();
            f.app.update();
            let visible = f.app.world().get::<CascadesVisibleEntities>(f.sun).unwrap();
            let expected = if pruned { vec![f.reachable] } else { vec![f.reachable, f.offscreen] };
            assert_eq!(visible.entities[&f.camera][0].entities, expected, "{map}");
            assert_eq!(visible.entities[&f.capture_camera][0].entities, [f.reachable, f.offscreen]);
        }
    }

    #[test]
    fn production_system_prunes_only_world_camera_and_preserves_capture_cascades() {
        let mut f = fixture();
        f.app.update();
        let visible = f.app.world().get::<CascadesVisibleEntities>(f.sun).unwrap();
        assert_eq!(visible.entities[&f.camera][0].entities, [f.reachable]);
        assert_eq!(visible.entities[&f.capture_camera][0].entities, [f.reachable, f.offscreen],
            "retained battle/image camera lists must not inherit world-camera pruning");
    }

    #[test]
    fn sky_is_not_a_receiver_and_nonreceiving_objects_still_cast() {
        for sky_has_bounds in [false,true] {
            for voxel_receiver in [false,true] {
                let mut f=fixture();
                let mut frustum=Frustum::default();
                let mut wide=planes();wide[1][3]=12.;wide[3][3]=12.;
                frustum.half_spaces=wide.map(|p|
                    bevy::shape::HalfSpace::new(Vec4::from_array(p.map(|v|v as f32))));
                f.app.world_mut().entity_mut(f.camera).insert(frustum);
                f.app.world_mut().entity_mut(f.sun).insert(GlobalTransform::from(
                    Transform::from_translation(Vec3::new(-10.,10.,0.)).looking_at(Vec3::ZERO,Vec3::Y)));
                for caster in [f.reachable,f.offscreen] {
                    f.app.world_mut().entity_mut(caster).insert(bevy::light::NotShadowReceiver);
                }
                f.app.world_mut().entity_mut(f.offscreen)
                    .insert(GlobalTransform::from_translation(Vec3::new(10.,10.,-0.5)));
                if voxel_receiver {
                    f.app.world_mut().entity_mut(f.offscreen)
                        .insert(MeshMaterial3d::<VoxelMaterial>(Handle::default()));
                }
                let mesh=f.app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::default()));
                let mut sky=f.app.world_mut().spawn((Mesh3d(mesh),GlobalTransform::IDENTITY,
                    InheritedVisibility::VISIBLE,RenderLayers::layer(VOXEL_RENDER_LAYER),
                    bevy::light::NotShadowCaster,bevy::light::NotShadowReceiver));
                if sky_has_bounds {sky.insert(Aabb::from_min_max(Vec3::splat(-3000.),Vec3::splat(3000.)));}
                f.app.update();
                let visible=f.app.world().get::<CascadesVisibleEntities>(f.sun).unwrap();
                let expected=if voxel_receiver {vec![f.reachable,f.offscreen]} else {vec![f.reachable]};
                assert_eq!(visible.entities[&f.camera][0].entities,expected,
                    "nonreceivers must not enlarge the scene slab; VoxelSurface still samples shadows");
                assert_eq!(visible.entities[&f.capture_camera][0].entities,[f.reachable,f.offscreen]);
            }
        }
    }

    #[test]
    fn unsupported_filter_and_unknown_receiver_bounds_keep_stock_shadow_lists() {
        for unknown_bounds in [false, true] {
            let mut f = fixture();
            if unknown_bounds {
                let mesh = f.app.world_mut().resource_mut::<Assets<Mesh>>().add(Mesh::from(Cuboid::default()));
                f.app.world_mut().spawn((Mesh3d(mesh), GlobalTransform::IDENTITY,
                    InheritedVisibility::VISIBLE, RenderLayers::layer(VOXEL_RENDER_LAYER)));
            } else {
                f.app.world_mut().entity_mut(f.camera).insert(ShadowFilteringMethod::Temporal);
            }
            f.app.update();
            let visible = f.app.world().get::<CascadesVisibleEntities>(f.sun).unwrap();
            assert_eq!(visible.entities[&f.camera][0].entities, [f.reachable, f.offscreen]);
        }
    }

    #[test]
    fn active_grass_spring_keeps_stock_shadow_lists_including_rebound() {
        for spring in [-0.3, 0.6] {
            let mut f = fixture();
            let mut materials = Assets::<VoxelMaterial>::default();
            let mut material = crate::solid_terrain_material();
            material.extension.grass_rustle.contact = Vec4::new(0.0, 0.0, 12.0, spring);
            materials.add(material);
            f.app.insert_resource(materials);
            f.app.update();
            let visible = f.app.world().get::<CascadesVisibleEntities>(f.sun).unwrap();
            assert_eq!(visible.entities[&f.camera][0].entities, [f.reachable, f.offscreen]);
        }
    }

    #[test]
    fn cache_revalidates_casters_and_receiver_inputs_and_removes_stale_entries() {
        let input = ReceiverInput {
            planes: cascade_planes(planes(), [0., 0., 1., 0.], 0., 1., 0.).unwrap(),
            sun: DVec3::Y,
            bottom: -1.,
            top: 1.,
        };
        let entity = Entity::PLACEHOLDER;
        let hidden = [Vec3::new(-10., 2., 0.); 8];
        let visible = [Vec3::new(0., 2., 0.5); 8];
        let mut cache = CachedCascade::default();
        cache.set_input(input, 1);
        assert_eq!(cache.keep(entity, hidden, 1), (false, false));
        cache.set_input(input, 2);
        assert_eq!(cache.keep(entity, hidden, 2), (false, true));
        assert_eq!(cache.keep(entity, visible, 2), (true, false));
        for changed in [
            ReceiverInput {
                sun: DVec3::new(1., 1., 0.).normalize(),
                ..input
            },
            ReceiverInput {
                bottom: -2.,
                ..input
            },
            ReceiverInput { top: 2., ..input },
            ReceiverInput {
                planes: input.planes.map(|[x, y, z, d]| [x, y, z, d + 1.]),
                ..input
            },
        ] {
            cache.set_input(changed, 3);
            assert_eq!(
                cache.keep(entity, visible, 3),
                (
                    may_reach(
                        visible,
                        changed.planes,
                        changed.sun,
                        changed.bottom,
                        changed.top
                    ),
                    false
                )
            );
        }
        cache.finish(4);
        assert!(cache.decisions.is_empty());
    }
    #[test]
    fn offscreen_casters_that_reach_receivers_are_preserved() {
        let sun = DVec3::new(0.4, 0.8, 0.3).normalize();
        for x in [-1., -0.5, 0., 0.5, 1.] {
            for y in [-1., 0., 1.] {
                for z in [-1., 0., 1.] {
                    let receiver = DVec3::new(x, y, z);
                    for distance in [0., 1., 10., 1000.] {
                        let caster = (receiver + sun * distance).as_vec3();
                        assert!(
                            may_reach([caster; 8], planes(), sun, -1., 1.),
                            "a real shadow ray reaching the view must survive"
                        );
                    }
                }
            }
        }
        assert!(!may_reach(
            [Vec3::new(-10., 2., 0.); 8],
            planes(),
            sun,
            -1.,
            1.
        ));
        assert!(!may_reach(
            [Vec3::new(10., -2., 0.); 8],
            planes(),
            sun,
            -1.,
            1.
        ));
    }
    #[test]
    fn boundaries_and_unknown_bounds_are_conservative() {
        assert!(may_reach(
            [Vec3::new(1.05, 0., 0.); 8],
            planes(),
            DVec3::Y,
            -1.,
            1.
        ));
        assert!(may_reach([Vec3::NAN; 8], planes(), DVec3::Y, -1., 1.));
        assert!(may_reach([Vec3::ZERO; 8], planes(), DVec3::ZERO, -1., 1.));
        assert!(may_reach([Vec3::ZERO; 8], planes(), DVec3::Y, f64::NAN, 1.));
    }
    #[test]
    fn actual_receiving_rays_survive_camera_rotations_and_box_bounds() {
        for yaw in [0.0_f32, 0.4, 1.7, 3.1] {
            let transform = Mat4::from_rotation_translation(
                Quat::from_rotation_y(yaw),
                Vec3::new(120., -25., 70.),
            );
            let inverse = transform.as_dmat4().inverse();
            let world_planes = planes()
                .map(|p| (inverse.transpose() * bevy::math::DVec4::from_array(p)).to_array());
            for sun in [DVec3::new(0.4, 0.8, 0.3), DVec3::new(-0.6, 0.7, -0.2)] {
                let sun = sun.normalize();
                for receiver in [
                    Vec3::new(-1., -1., -1.),
                    Vec3::ZERO,
                    Vec3::new(1., 1., 1.),
                    Vec3::new(1., -0.5, -1.),
                ] {
                    let receiver = transform.transform_point3(receiver);
                    for t in [0., 10., 1000.] {
                        let caster = (receiver.as_dvec3() + sun * t).as_vec3();
                        let points = std::array::from_fn(|i| {
                            caster
                                + Vec3::new(
                                    if i & 1 == 0 { -0.2 } else { 0.2 },
                                    if i & 2 == 0 { -0.2 } else { 0.2 },
                                    if i & 4 == 0 { -0.2 } else { 0.2 },
                                )
                        });
                        assert!(may_reach(points, world_planes, sun, -26., -24.));
                    }
                }
            }
        }
    }
    #[test]
    fn cascade_depth_limits_preserve_blending_and_bias_footprints() {
        let mut base = planes();
        base[4][3] = 10.;
        base[5][3] = 10.;
        let depth = [0., 0., 1., 0.];
        let first = cascade_planes(base, depth, 0., 2., 0.).unwrap();
        let next = cascade_planes(base, depth, 1.6, 4., 0.).unwrap();
        let overlap = [Vec3::new(0., 10., 1.8); 8];
        assert!(may_reach(overlap, first, DVec3::Y, -1., 1.));
        assert!(may_reach(overlap, next, DVec3::Y, -1., 1.));
        let outside = [Vec3::new(0., 10., 3.); 8];
        assert!(!may_reach(outside, first, DVec3::Y, -1., 1.));
        let biased = cascade_planes(base, depth, 0., 2., 1.1).unwrap();
        assert!(may_reach(outside, biased, DVec3::Y, -2.1, 2.1));
        assert!(cascade_planes(base, depth, f32::NAN, 2., 0.).is_none());
    }
}
