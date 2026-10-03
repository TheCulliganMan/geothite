//! A frozen instance of the terrain that actually supported a checked encounter.
//! No terrain reconstruction, source sprite copies, or borrowed world fade state.
use crate::{
    TerrainRevisionCache, VoxelMaterial,
    battle_layout::{BattleBody, BattleSceneLayout, CAMERA_FOV},
    battle_view::BattleViewStatus,
};
use bevy::{
    asset::AssetId,
    prelude::*,
    render::{primitives::Aabb, view::RenderLayers},
};
use crystal_render_api::{
    BattleFlashMode, BattleLocationFrame, VisualBattleFrame, VisualBattleLocation, VisualWorldFrame,
};
use std::collections::HashMap;

const BATTLE_LAYER: usize = 29;
/// Four retained-world units per source pixel and sixteen pixels per core tile
/// give 64 retained units per tile. This uniform conversion gives four here.
const RETAINED_SCALE: f32 = 1.0 / 16.0;
const SOURCE_CELL_SIZE: f32 = 32.0;
const BORDER_MARGIN: f32 = 0.25;
const RESIDUAL_OPACITY: f32 = 0.20;

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
}
struct FrozenMaterial {
    handle: Handle<VoxelMaterial>,
    object: Option<Entity>,
}
struct FrozenObject {
    bounds: Bounds,
    faded: bool,
    opacity: f32,
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

fn opacity_mode(opacity: f32) -> AlphaMode {
    if opacity >= 1.0 {
        AlphaMode::Opaque
    } else {
        AlphaMode::Blend
    }
}
struct FrozenScene {
    root: Entity,
    anchors: [Vec3; 2],
    authentic: AuthenticBounds,
    omitted: Vec<Bounds>,
    objects: HashMap<Entity, FrozenObject>,
    materials: Vec<FrozenMaterial>,
    images: Vec<Handle<Image>>,
    // Only encounter-owned copies of the two mutable flower domains.
    meshes: Vec<Handle<Mesh>>,
    ground: f32,
}

#[derive(Resource, Default)]
pub(super) struct EncounterTerrain {
    generation: Option<u64>,
    frozen: Option<FrozenScene>,
    accepted: bool,
    attempted_roots: Option<[Entity; 3]>,
    fog: Option<(f32, f32)>,
    reason: Option<&'static str>,
}
impl EncounterTerrain {
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

    /// A finite genuine neighborhood is eligible only when every visible ray
    /// and every possible omitted shadow stays inside the retained evidence.
    pub(super) fn constrain_layout(
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
        let Some(limit) = scene.authentic.camera_limit(layout.camera, viewport) else {
            self.reason = Some("battle camera leaves the authentic retained neighborhood");
            return false;
        };
        let view = layout.camera.rotation.inverse();
        let mut deepest: f32 = 0.0;
        let mut body_distance: f32 = 0.0;
        for (body, pose) in bodies.into_iter().zip(layout.body_poses) {
            if let (Some(body), Some(pose)) = (body, pose) {
                for point in body.corners(pose) {
                    if !scene.authentic.point(point) {
                        self.reason = Some("battler exceeds authentic terrain bounds");
                        return false;
                    }
                    deepest = deepest.max(-(view * (point - layout.camera.translation)).z);
                    body_distance = body_distance.max(point.distance(layout.camera.translation));
                }
            }
        }
        let far = limit.min(body_distance + 8.0).min(layout.far);
        trace(format_args!(
            "fit camera={:?} authentic={:?} viewport={viewport:?} body_distance={body_distance} far={far}",
            layout.camera, scene.authentic
        ));
        if !deepest.is_finite() || deepest <= 0.0 || far < body_distance + 2.0 {
            self.reason = Some("authentic terrain is too small for the complete battlers");
            return false;
        }
        if scene.omitted.iter().any(|bounds| {
            let visible = visible_bounds(*bounds, layout.camera, viewport, far);
            let shadow = visible_bounds(
                bounds.shadow_volume(scene.ground),
                layout.camera,
                viewport,
                far,
            );
            if visible || shadow {
                trace(format_args!(
                    "rejected omission bounds={bounds:?} visible={visible} shadow={shadow}"
                ));
            }
            visible || shadow
        }) {
            self.reason = Some("omitted border or mutable terrain could affect the battle view");
            return false;
        }
        // Each instance or authored group owns one opacity. An AABB test is
        // conservative for outdoor trees; it never removes a surface fragment.
        for object in scene.objects.values_mut() {
            object.faded = layout.hit_anchors.iter().any(|&point| {
                object.bounds.segment(layout.camera.translation, point)
                    || object
                        .bounds
                        .segment(layout.camera.translation, point + Vec3::Y * 0.5)
            });
        }
        layout.far = far;
        self.fog = Some((body_distance + 0.5, far - 0.1));
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

fn resolve_context(
    location: &VisualBattleLocation,
    cache: &TerrainRevisionCache,
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
    if !evidence.matches_built_frame(built) {
        return Err("actual built terrain does not match encounter evidence");
    }
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
    let feet = [
        strict_foot(built, &cache.built_footing_heights, anchors.source_foot),
        strict_foot(built, &cache.built_footing_heights, anchors.target_foot),
    ];
    let feet = [
        feet[0].ok_or("source footing is outside actual built terrain")?,
        feet[1].ok_or("target footing is outside actual built terrain")?,
    ];
    let midpoint = (feet[0] + feet[1]) * 0.5;
    let rebase = Transform::from_translation(-midpoint * RETAINED_SCALE)
        .with_scale(Vec3::splat(RETAINED_SCALE));
    let grid = built.grid_size.as_vec2();
    let start = built.grid_origin.as_vec2();
    let lo = start.max(Vec2::ZERO);
    let hi = (start + grid).min(map.as_vec2() * 2.0);
    if hi.cmple(lo).any() {
        return Err("source map and actual terrain do not intersect");
    }
    let northwest = Vec2::new(built.center.x, -built.center.y) - grid * built.tile_size * 0.5;
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
        rebase.mul_transform(crate::terrain_transform(built)),
        authentic,
    ))
}

fn collect_leaves(
    world: &World,
    entity: Entity,
    pose: Transform,
    object: Option<Entity>,
    out: &mut Vec<FrozenLeaf>,
) -> Result<(), &'static str> {
    if world.get_entity(entity).is_none() {
        return Err("actual terrain hierarchy is not ready");
    }
    if world.get::<SyntheticTerrainApron>(entity).is_some() {
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
    if let Some(mesh) = world.get::<Handle<Mesh>>(entity) {
        let material = world
            .get::<Handle<VoxelMaterial>>(entity)
            .ok_or("terrain leaf has no voxel material")?;
        let aabb = world.get::<Aabb>(entity).copied().or_else(|| {
            world
                .get_resource::<Assets<Mesh>>()?
                .get(mesh)?
                .compute_aabb()
        });
        let Some(aabb) = aabb else {
            return Err("terrain leaf bounds have not been published");
        };
        let bounds = Bounds::from_aabb(aabb, pose).ok_or("terrain leaf has nonfinite bounds")?;
        out.push(FrozenLeaf {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: pose,
            aabb,
            bounds,
            object,
        });
    }
    if let Some(children) = world.get::<Children>(entity) {
        for &child in children.iter() {
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
            if let Some(root) = world.get_entity_mut(frozen.root) {
                root.despawn_recursive();
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
                    resolve_context(&location, cache)?;
                    Ok([
                        cache
                            .instances_root
                            .ok_or("terrain instances are not ready")?,
                        cache
                            .textured_entity
                            .ok_or("textured terrain is not ready")?,
                        cache.solid_entity.ok_or("solid terrain is not ready")?,
                    ])
                });
            match ready {
                Ok(roots) if state.attempted_roots != Some(roots) => {
                    state.attempted_roots = Some(roots);
                    match freeze(world, &location) {
                        Ok(frozen) => {
                            state.frozen = Some(frozen);
                            state.reason = None;
                        }
                        Err(reason) => {
                            state.reason = Some(reason);
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

/// Select complete original triangles once from the two mutable CPU meshes.
/// Vertex streams stay byte-identical, including UVs, normals and cutaway data.
/// Unreferenced outside vertices cannot draw; the copied AABB uses selected
/// indices only. A straddling triangle is omitted and remains in the existing
/// view/shadow proof, so this never invents a clipped edge or fills missing land.
fn select_authentic_animated(
    source: &Mesh,
    pose: Transform,
    authentic: AuthenticBounds,
) -> Result<(Option<(Mesh, Aabb, Bounds)>, Vec<Bounds>), &'static str> {
    use bevy::render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    };
    use std::collections::BTreeMap;
    let Some(VertexAttributeValues::Float32x3(positions)) =
        source.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Err("animated terrain positions are unavailable");
    };
    let indices = source
        .indices()
        .ok_or("animated terrain requires indexed triangles")?;
    if source.primitive_topology() != PrimitiveTopology::TriangleList
        || indices.len() % 3 != 0
        || source
            .attributes()
            .any(|(_, values)| values.len() != positions.len())
        || indices.iter().any(|i| i >= positions.len())
    {
        return Err("animated terrain has invalid triangle attributes");
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
            return Err("animated terrain has nonfinite vertices");
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
                return Err("animated terrain omission coverage exceeds its bound");
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
        copy.asset_usage = RenderAssetUsages::RENDER_WORLD;
        let aabb = Aabb::from_min_max(local.min, local.max);
        let bounds =
            Bounds::from_aabb(aabb, pose).ok_or("selected animated terrain bounds are invalid")?;
        Some((copy, aabb, bounds))
    };
    Ok((selected, excluded.into_values().collect()))
}

fn freeze(world: &mut World, location: &VisualBattleLocation) -> Result<FrozenScene, &'static str> {
    let cache = world
        .get_resource::<TerrainRevisionCache>()
        .ok_or("voxel terrain cache is unavailable")?;
    let (anchors, root_pose, authentic) = resolve_context(location, cache)?;
    let mut leaves = Vec::new();
    let mut omitted = Vec::new();
    let instances = cache
        .instances_root
        .ok_or("actual terrain instances are not ready")?;
    if let Some(children) = world.get::<Children>(instances) {
        for &child in children.iter() {
            let local = world
                .get::<Transform>(child)
                .copied()
                .ok_or("terrain instance lacks a transform")?;
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
        if world.get::<Handle<Mesh>>(entity) != Some(mesh) {
            return Err("actual animated terrain handle does not match its cache");
        }
        let material = world
            .get::<Handle<VoxelMaterial>>(entity)
            .ok_or("actual animated terrain material is missing")?
            .clone();
        let source = world
            .get_resource::<Assets<Mesh>>()
            .and_then(|assets| assets.get(mesh))
            .ok_or("actual animated terrain CPU mesh is unavailable")?;
        let (selected, excluded) = select_authentic_animated(source, root_pose, authentic)?;
        trace(format_args!(
            "animated selection kept={} omitted_buckets={}",
            selected.is_some(),
            excluded.len()
        ));
        omitted.extend(excluded);
        let Some((copy, aabb, bounds)) = selected else {
            continue;
        };
        mutable_mesh_copies.insert(mesh.id(), copy);
        leaves.push(FrozenLeaf {
            mesh: mesh.clone(),
            material,
            transform: root_pose,
            aabb,
            bounds,
            object: None,
        });
    }
    let mut kept = Vec::new();
    for leaf in leaves {
        if authentic.contains(leaf.bounds) {
            kept.push(leaf);
        } else {
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
    let mut ground = anchors[0].y.min(anchors[1].y);
    for leaf in &kept {
        ground = ground.min(leaf.bounds.min.y);
        if let Some(id) = leaf.object {
            objects
                .entry(id)
                .and_modify(|o| o.bounds = o.bounds.union(leaf.bounds))
                .or_insert(FrozenObject {
                    bounds: leaf.bounds,
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
        material.extension.cutaway = default();
        // Any transparency inherited from world reveal is reset. Whole-object
        // battle reveal is computed from this encounter camera independently.
        material.base.alpha_mode = AlphaMode::Opaque;
        material_copies.insert(key, material);
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
            let handle = world.resource_mut::<Assets<VoxelMaterial>>().add(material);
            frozen_materials.push(FrozenMaterial {
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
            SpatialBundle {
                visibility: Visibility::Hidden,
                ..default()
            },
            EncounterTerrainRoot,
        ))
        .id();
    for leaf in kept {
        let child = world
            .spawn((
                MaterialMeshBundle::<VoxelMaterial> {
                    mesh: mutable_mesh_handles
                        .get(&leaf.mesh.id())
                        .cloned()
                        .unwrap_or(leaf.mesh),
                    material: material_handles[&(leaf.material.id(), leaf.object)].clone(),
                    transform: leaf.transform,
                    ..default()
                },
                leaf.aabb,
                RenderLayers::layer(BATTLE_LAYER),
            ))
            .id();
        world.entity_mut(root).add_child(child);
    }
    Ok(FrozenScene {
        root,
        anchors,
        authentic,
        omitted,
        objects,
        materials: frozen_materials,
        images: image_handles.into_values().collect(),
        meshes: mutable_mesh_handles.into_values().collect(),
        ground,
    })
}

/// Schedule after the battle view has made its final source/model visibility
/// decision. Source fallback hides this hierarchy without retiring its assets.
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
    let seconds = time.as_ref().map_or(0.0, |time| time.delta_seconds());
    for object in scene.objects.values_mut() {
        object.opacity = advance_opacity(object.opacity, object.faded, seconds);
    }
    for retained in &scene.materials {
        let opacity = retained
            .object
            .and_then(|id| scene.objects.get(&id))
            .map_or(1.0, |object| object.opacity);
        let value = Vec4::new(1.0 - opacity, dark, white, 0.0);
        let alpha_mode = opacity_mode(opacity);
        if materials
            .get(&retained.handle)
            .is_some_and(|m| m.extension.cutaway.fade != value || m.base.alpha_mode != alpha_mode)
        {
            if let Some(material) = materials.get_mut(&retained.handle) {
                material.extension.cutaway.fade = value;
                material.base.alpha_mode = alpha_mode;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            map_texture: Handle::weak_from_u128(51),
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
        retained.built_frame.as_mut().unwrap().map_texture = Handle::weak_from_u128(52);
        assert!(resolve_context(&location, &retained).is_ok());
        retained.built_source_texture = Some(Handle::weak_from_u128(53));
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
                bevy::render::render_asset::RenderAssetUsages::default(),
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
        let root = app.world_mut().spawn(SpatialBundle::default()).id();
        let leaf = app
            .world_mut()
            .spawn((
                MaterialMeshBundle::<VoxelMaterial> {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..default()
                },
                bounds,
                RenderLayers::layer(crate::VOXEL_RENDER_LAYER),
            ))
            .id();
        app.world_mut().entity_mut(root).add_child(leaf);
        let instances = app.world_mut().spawn(SpatialBundle::default()).id();
        let apron = app
            .world_mut()
            .spawn((
                MaterialMeshBundle::<VoxelMaterial> {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..default()
                },
                bounds,
                SyntheticTerrainApron,
                RenderLayers::layer(crate::VOXEL_RENDER_LAYER),
            ))
            .id();
        app.world_mut().entity_mut(instances).add_child(apron);
        let empty = app.world_mut().spawn(SpatialBundle::default()).id();
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
            .get::<Handle<VoxelMaterial>>(instance)
            .unwrap()
            .clone();
        assert_ne!(frozen_material, material);
        assert_eq!(app.world().get::<Handle<Mesh>>(instance), Some(&mesh));
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
            .data
            .clone();
        app.world_mut()
            .resource_mut::<Assets<Image>>()
            .get_mut(&image)
            .unwrap()
            .data
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
                .data,
            frozen_pixels
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
        assert!(app.world().get_entity(root).is_none());
        assert!(app.world().get_entity(instance).is_none());
        assert!(app.world().get_entity(original).is_some());
        assert_eq!(counts(&app), baseline);
        assert_eq!(
            *app.world().get::<RenderLayers>(original).unwrap(),
            RenderLayers::layer(crate::VOXEL_RENDER_LAYER)
        );
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
        let shader = include_str!("voxel_surface.wgsl");
        assert!(shader.contains("mix(shaded, vec3<f32>(0.0), clamp(cutaway.fade.y, 0.0, 1.0))"));
        assert!(shader.contains("mix(darkened, vec3<f32>(1.0), clamp(cutaway.fade.z, 0.0, 1.0))"));
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
        let bounds = mesh.compute_aabb().unwrap();
        let mesh = app
            .world_mut()
            .resource_mut::<Assets<Mesh>>()
            .add(crate::retain_animated_mesh_cpu(mesh));
        let entity = app
            .world_mut()
            .spawn((
                MaterialMeshBundle::<VoxelMaterial> {
                    mesh: mesh.clone(),
                    material,
                    ..default()
                },
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
        use bevy::render::mesh::Indices;
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
    fn mutable_flower_copy_is_byte_exact_and_survives_world_replacement_without_growth() {
        use bevy::render::render_asset::RenderAssetUsages;
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
                .any(|&child| app.world().get::<Handle<Mesh>>(child) == Some(&static_mesh))
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
        use bevy::render::{
            mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
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
                select_authentic_animated(&source, Transform::IDENTITY, authentic).unwrap();
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
        use bevy::render::{
            mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
        };
        let source = Mesh::from(Cuboid::new(32.0, 16.0, 32.0));
        let pose = Transform::from_translation(Vec3::new(20.0, 3.0, -10.0))
            .with_scale(Vec3::splat(1.0 / 16.0));
        let authentic = AuthenticBounds {
            min: Vec2::new(18.0, -12.0),
            max: Vec2::new(22.0, -8.0),
        };
        let (selected, omitted) = select_authentic_animated(&source, pose, authentic).unwrap();
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
        assert!(select_authentic_animated(&invalid, pose, authentic).is_err());
        invalid.insert_indices(Indices::U32(vec![0, 1]));
        assert!(select_authentic_animated(&invalid, pose, authentic).is_err());
    }
}
