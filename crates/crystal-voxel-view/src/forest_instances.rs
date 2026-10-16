//! Runtime forest geometry is a bounded library, not a world-sized mesh cache.
//! Full modeled prototypes share GPU handles and collision trees. Placement,
//! height, rotation, visibility and cutaway state belong to each instance.
use crate::{
    foliage::{self, GroveFamily},
    mesh::TerrainMeshData,
    occluder_fade::{self, PreparedGroup},
};
use bevy::prelude::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
};

static EAGER_SORTED_ENTITIES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Startup-only QA control; geometry, collisions and draw policy stay identical.
pub fn configure_lazy_tree_entities_diagnostics(eager: bool) {
    EAGER_SORTED_ENTITIES.store(eager, std::sync::atomic::Ordering::Relaxed);
}

const VARIANTS: usize = 12; // three branch architectures, four height/phase recipes
const CAPACITY: usize = 128; // all 120 family/density/variant combinations fit
const PROTOTYPE_TILE: f32 = 16.;
const PROTOTYPE_BOUNDS: [f32; 4] = [-16., 16., -16., 16.];
struct PrototypeGeometry {
    groups: Vec<PreparedGroup>,
    trunk_bounds: Option<(Vec3, Vec3)>,
}
type Prototype = Arc<PrototypeGeometry>;
type Cell = Arc<OnceLock<Prototype>>;
type GroundCell = Arc<OnceLock<Arc<Vec<PreparedGroup>>>>;
#[derive(Default)]
pub(crate) struct Library {
    entries: Mutex<VecDeque<(Key, Cell)>>,
    ground: Mutex<VecDeque<(u8, GroundCell)>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Key {
    family: u8,
    density: u8,
    variant: u8,
}
impl Key {
    fn new(family: GroveFamily, coordinates: [i32; 2], neighbors: [bool; 4]) -> Self {
        let (shape, height, _) = foliage::instance_variation(coordinates);
        let bucket = (((height - 0.9) / 0.7 * 4.).floor() as usize).min(3);
        let density = neighbors.iter().filter(|&&v| v).count() as u8;
        Self {
            family: if family == GroveFamily::JohtoPine {
                0
            } else {
                1
            },
            density,
            variant: (shape * 4 + bucket) as u8,
        }
    }
}

// Every source plot retains a canopy, including the interior of continuous blocks.
fn seeds() -> &'static [[i32; 2]; VARIANTS] {
    static SEEDS: OnceLock<[[i32; 2]; VARIANTS]> = OnceLock::new();
    SEEDS.get_or_init(|| {
        let mut result = [[0; 2]; VARIANTS];
        let mut found = [false; VARIANTS];
        for z in 0..256 {
            for x in 0..256 {
                let key = Key::new(GroveFamily::JohtoPine, [x,z], [true;4]);
                let variant = key.variant as usize;
                if !found[variant] {
                    result[variant] = [x,z];
                    found[variant] = true;
                }
                if found.iter().all(|v| *v) { return result; }
            }
        }
        panic!("forest prototype seeds must cover every authored shape/height class")
    })
}

impl Library {
    fn prototype(&self, key: Key, family: GroveFamily) -> Prototype {
        let cell = {
            let mut library = self.entries.lock().unwrap();
            if let Some(index) = library.iter().position(|(candidate, _)| *candidate == key) {
                let entry = library.remove(index).unwrap();
                let cell = entry.1.clone();
                library.push_back(entry);
                cell
            } else {
                if library.len() == CAPACITY {
                    library.pop_front();
                }
                let cell = Arc::new(OnceLock::new());
                library.push_back((key, cell.clone()));
                cell
            }
        };
        let _trace = crate::loading_trace::span("tree.prototype", || {
            serde_json::json!({
        "family":family.label(),"density":key.density,"variant":key.variant,
        "cache_hit":cell.get().is_some()})
        });
        cell.get_or_init(|| {
            let mut mesh = TerrainMeshData::default();
            let neighbors = std::array::from_fn(|i| i < key.density as usize);
            foliage::append_family_canopy(
                &mut mesh,
                PROTOTYPE_BOUNDS,
                0.,
                PROTOTYPE_TILE,
                seeds()[key.variant as usize],
                neighbors,
                family,
            );
            let trunk_bounds = Some({
                let count = if family == GroveFamily::JohtoPine { 13 } else { 17 };
                mesh.solid.positions[..count].iter().fold(
                    (Vec3::splat(f32::INFINITY),Vec3::splat(f32::NEG_INFINITY)),
                    |(low,high), &p| (low.min(Vec3::from_array(p)),high.max(Vec3::from_array(p))),
                )
            });
            Arc::new(PrototypeGeometry {
                groups: crate::shared_groves::prepare_surface(mesh.solid), trunk_bounds,
            })
        })
        .clone()
    }

    fn prepare_ground(&self, key: Key, bounds: [f32;4], base: f32, tile: f32,
        coordinates: [i32;2]) -> Vec<PreparedGroup> {
        let cell = {
            let mut entries = self.ground.lock().unwrap();
            if let Some((_,cell)) = entries.iter().find(|(variant,_)| *variant == key.variant) {
                cell.clone()
            } else {
                if entries.len() == VARIANTS { entries.pop_front(); }
                let cell = Arc::new(OnceLock::new());
                entries.push_back((key.variant,cell.clone()));
                cell
            }
        };
        let prototype = cell.get_or_init(|| {
            let mut mesh = TerrainMeshData::default();
            crate::understory::append(&mut mesh.solid,PROTOTYPE_BOUNDS,0.,PROTOTYPE_TILE,
                seeds()[key.variant as usize]);
            mesh.solid.cutaway_ranges.push(0..mesh.solid.positions.len());
            Arc::new(crate::shared_groves::prepare_surface(mesh.solid))
        });
        Self::place_ground(prototype, bounds, base, tile, coordinates)
    }

    fn place_ground(prototype: &Arc<Vec<PreparedGroup>>, bounds: [f32;4], base: f32,
        tile: f32, coordinates: [i32;2]) -> Vec<PreparedGroup> {
        let (_,_,turn) = foliage::instance_variation([coordinates[1] ^ 0x317b, coordinates[0]]);
        // A half turn preserves even rectangular source plots. Ground cover
        // never inherits the canopy's lean, height, shift or crown fitting.
        let turn = if turn < std::f32::consts::PI { 0. } else { std::f32::consts::PI };
        let mut groups = prototype.as_ref().clone();
        occluder_fade::place_groups(&mut groups,Transform {
            translation: Vec3::new((bounds[0]+bounds[1])*0.5,base,(bounds[2]+bounds[3])*0.5),
            rotation: Quat::from_rotation_y(turn),
            scale: Vec3::new((bounds[1]-bounds[0])/32.,
                (tile/PROTOTYPE_TILE).min((bounds[1]-bounds[0]).min(bounds[3]-bounds[2])/32.),
                (bounds[3]-bounds[2])/32.),
        });
        groups
    }

    pub(crate) fn prepare(
        &self,
        bounds: [f32; 4],
        base: f32,
        tile: f32,
        coordinates: [i32; 2],
        neighbors: [bool; 4],
        family: GroveFamily,
    ) -> Vec<PreparedGroup> {
        let ground = None;
        self.prepare_with_ground(bounds, base, tile, coordinates, neighbors, family, ground)
    }

    pub(crate) fn prepare_with_ground(&self, bounds: [f32;4], base: f32, tile: f32,
        coordinates: [i32;2], neighbors: [bool;4], family: GroveFamily,
        ground_bounds: Option<[f32;4]>) -> Vec<PreparedGroup> {
        let key = Key::new(family, coordinates, neighbors);
        let source = self.prototype(key, family);
        let mut groups = Self::place_canopy(key, &source, bounds, base, tile, coordinates);
        if let Some(ground_bounds) = ground_bounds {
            groups.extend(self.prepare_ground(key,ground_bounds,base,tile,coordinates));
        }
        Self::finish_instances(&mut groups);
        groups
    }

    /// Clone and place completed prototypes only. No initialization, geometry
    /// generation or waits on another thread's OnceLock occur on this path.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn prepare_if_cached(&self, bounds: [f32;4], base: f32, tile: f32,
        coordinates: [i32;2], neighbors: [bool;4], family: GroveFamily)
        -> Option<Vec<PreparedGroup>> {
        let ground = None;
        self.prepare_if_cached_with_ground(bounds,base,tile,coordinates,neighbors,family,ground)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn prepare_if_cached_with_ground(&self, bounds: [f32;4], base: f32, tile: f32,
        coordinates: [i32;2], neighbors: [bool;4], family: GroveFamily,
        ground_bounds: Option<[f32;4]>) -> Option<Vec<PreparedGroup>> {
        let key = Key::new(family, coordinates, neighbors);
        let source = {
            let entries = self.entries.lock().unwrap();
            entries.iter().find(|(candidate,_)| *candidate == key)?.1.get()?.clone()
        };
        let ground = if ground_bounds.is_some() {
            let entries = self.ground.lock().unwrap();
            Some(entries.iter().find(|(variant,_)| *variant == key.variant)?.1.get()?.clone())
        } else { None };
        let _trace = crate::loading_trace::span("tree.prepare_cached_instance", ||
            serde_json::json!({"coordinates":coordinates,"variant":key.variant,"density":key.density}));
        let mut groups = Self::place_canopy(key, &source, bounds, base, tile, coordinates);
        if let Some(ground) = ground {
            groups.extend(Self::place_ground(&ground,ground_bounds.unwrap(),base,tile,coordinates));
        }
        Self::finish_instances(&mut groups);
        Some(groups)
    }

    fn place_canopy(key: Key, source: &Prototype, bounds: [f32;4], base: f32,
        tile: f32, coordinates: [i32;2]) -> Vec<PreparedGroup> {
        let (_, _, turn) = foliage::instance_variation(coordinates);
        let (_, _, prototype_turn) =
            foliage::instance_variation(seeds()[key.variant as usize]);
        let family = if key.family == 0 { GroveFamily::JohtoPine } else { GroveFamily::AncientForest };
        let vertical_ratio = foliage::instance_vertical_ratio(family,coordinates,
            seeds()[key.variant as usize], std::array::from_fn(|i| i < key.density as usize));
        let rotation = Quat::from_rotation_y(turn - prototype_turn);
        let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
        // The shared mesh already contains its seed's trunk offset. Replace it
        // with this placement's offset, independent of prototype and rotation.
        let prototype_offset = foliage::grove_trunk_offset(seeds()[key.variant as usize], 32.);
        let prototype_root = Vec3::new(prototype_offset.x, 0., prototype_offset.y);
        let offset = foliage::grove_trunk_offset(coordinates, spacing);
        let scale = Vec3::new(
            (bounds[1] - bounds[0]) / 32.,
            tile / PROTOTYPE_TILE * vertical_ratio,
            (bounds[3] - bounds[2]) / 32.,
        );
        let mut transform = Transform {
            translation: Vec3::new(
                (bounds[0] + bounds[1]) * 0.5,
                base,
                (bounds[2] + bounds[3]) * 0.5,
            ),
            rotation,
            scale,
        };
        // Quarter turns and irregular source plots must stay inside the same
        // authored crown margin. Fit the shared prototype with one instance scale;
        // never rewrite vertices, normals or indices for placement.
        if let Some((low, high)) = occluder_fade::prepared_bounds(&source.groups) {
            let margin = spacing * if key.family == 1 { 1.25 } else { 0.75 };
            let mut fit = 1.0_f32;
            for i in 0..8 {
                let p = rotation
                    * (scale
                        * (Vec3::new(
                            if i & 1 == 0 { low.x } else { high.x },
                            if i & 2 == 0 { low.y } else { high.y },
                            if i & 4 == 0 { low.z } else { high.z },
                        ) - prototype_root));
                for (v, offset, radius) in [
                    (p.x, offset.x, (bounds[1] - bounds[0]) * 0.5 + margin),
                    (p.z, offset.y, (bounds[3] - bounds[2]) * 0.5 + margin),
                ] {
                    let available = radius - v.signum() * offset;
                    if v.abs() > available {
                        fit = fit.min(available / v.abs());
                    }
                }
            }
            transform.scale.x *= fit;
            transform.scale.z *= fit;
        }
        // Offset roots retain the same authored collision footprint, including
        // a quarter-turned, irregular flare on a rectangular source plot.
        if let Some((low,high)) = source.trunk_bounds {
            let mut fit = 1.0_f32;
            for i in 0..4 {
                let p = rotation * (transform.scale * (Vec3::new(
                    if i&1==0 {low.x} else {high.x}, 0.,
                    if i&2==0 {low.z} else {high.z}) - prototype_root));
                for (v,offset,radius) in [(p.x,offset.x,(bounds[1]-bounds[0])*0.5),
                    (p.z,offset.y,(bounds[3]-bounds[2])*0.5)] {
                    if v.abs()>0. { fit=fit.min((radius-v.signum()*offset)/v.abs()); }
                }
            }
            transform.scale.x *= fit;
            transform.scale.z *= fit;
        }
        transform.translation += Vec3::new(offset.x, 0., offset.y)
            - rotation * (transform.scale * prototype_root);
        let _trace = crate::loading_trace::span("tree.instance", || {
            serde_json::json!({
        "coordinates":coordinates,"variant":key.variant,"density":key.density})
        });
        let mut groups = source.groups.clone();
        occluder_fade::place_groups(&mut groups, transform);
        groups
    }

    fn finish_instances(groups: &mut [PreparedGroup]) {
        if !EAGER_SORTED_ENTITIES.load(std::sync::atomic::Ordering::Relaxed) {
            occluder_fade::use_lazy_sorted_entities(groups);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adding_understory_preserves_every_ancient_tree_triangle_and_placement() {
        let library = Library::default();
        let tree = library.prepare_with_ground([-16.,16.,-16.,16.],3.,16.,[4,8],
            [true;4],GroveFamily::AncientForest,None);
        let augmented = library.prepare_with_ground([-16.,16.,-16.,16.],3.,16.,[4,8],
            [true;4],GroveFamily::AncientForest,Some([-32.,32.,-32.,32.]));
        assert!(augmented.len() > tree.len());
        let signature = |groups| occluder_fade::geometry_signature(
            &crate::mesh::SurfaceMeshData::default(),groups);
        assert_eq!(signature(tree.clone()),signature(augmented[..tree.len()].to_vec()),
            "adding ground flora must not alter any existing tree geometry or transform");
    }
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cached_preparation_never_initializes_and_retains_exact_shared_geometry() {
        let library = Library::default();
        let bounds = [-16., 16., -16., 16.];
        let coordinates = [4,8];
        let neighbors = [true,true,false,false];
        assert!(library.prepare_if_cached(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine).is_none());
        assert!(library.entries.lock().unwrap().is_empty());
        assert!(library.ground.lock().unwrap().is_empty());
        let key = Key::new(GroveFamily::JohtoPine,coordinates,neighbors);
        let source = library.prototype(key,GroveFamily::JohtoPine);
        assert!(library.prepare_if_cached(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine).is_none(), "missing ground cover must remain asynchronous");
        assert!(library.ground.lock().unwrap().is_empty());
        let expected = library.prepare(bounds,3.,16.,coordinates,neighbors,GroveFamily::JohtoPine);
        let count = library.entries.lock().unwrap().len();
        let ground_count = library.ground.lock().unwrap().len();
        let cached = library.prepare_if_cached(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine).unwrap();
        let signature = |groups| occluder_fade::geometry_signature(
            &crate::mesh::SurfaceMeshData::default(),groups);
        assert_eq!(signature(cached),signature(expected));
        assert_eq!(library.entries.lock().unwrap().len(),count);
        assert_eq!(library.ground.lock().unwrap().len(),ground_count);
        assert!(Arc::ptr_eq(&source,&library.prototype(key,GroveFamily::JohtoPine)));
    }

    #[test]
    fn every_runtime_prototype_uses_the_texture_free_palette_contract() {
        let material = crate::solid_terrain_material();
        assert!(crate::compact_vertices::solid_uv0_is_unused(
            &material.base, material.extension.water_enabled));
        let mut count = 0;
        let mut max_vertices = 0;
        let mut indices = 0;
        for family in [GroveFamily::JohtoPine, GroveFamily::AncientForest] {
            for density in 0..=4 {
                for &coordinates in seeds() {
                        let mut mesh = TerrainMeshData::default();
                        foliage::append_family_canopy(&mut mesh, PROTOTYPE_BOUNDS, 0.,
                            PROTOTYPE_TILE, coordinates,
                            std::array::from_fn(|i| i < density), family);
                        assert!(mesh.solid.positions.len() > 1000,
                            "every authored plot must retain a full tree, including dense interiors");
                        assert!(mesh.solid.positions.iter().any(|p| p[1] > PROTOTYPE_TILE));
                        assert_eq!(mesh.solid.uvs.len(), mesh.solid.positions.len());
                        assert!(mesh.solid.uvs.iter().all(|&uv| uv == [0.; 2]),
                            "{family:?}/{density}/{coordinates:?} requires UV shader behavior");
                        max_vertices = max_vertices.max(mesh.solid.positions.len());
                        indices += mesh.solid.indices.len();
                        count += 1;
                }
            }
        }
        assert_eq!(count, 120);
        println!("{count} full tree prototypes: max {max_vertices} vertices, {indices} indices; all UV0 zero");
    }

    #[test]
    fn library_has_complete_variety_and_bounded_world_independent_keys() {
        for (variant, &coordinates) in seeds().iter().enumerate() {
            let key = Key::new(GroveFamily::JohtoPine, coordinates, [true; 4]);
            assert_eq!(key.variant as usize, variant);
        }
        let mut keys = Vec::new();
        for family in [GroveFamily::JohtoPine, GroveFamily::AncientForest] {
            for z in -40..40 {
                for x in -40..40 {
                    for density in 0..=4 {
                        let key = Key::new(family, [x, z], std::array::from_fn(|i| i < density));
                        if !keys.contains(&key) {
                            keys.push(key);
                        }
                    }
                }
            }
        }
        assert_eq!(keys.len(), 120);
        assert!(keys.len() <= CAPACITY);
    }
    #[test]
    fn many_tree_and_ground_placements_share_two_meshes_and_the_material() {
        let library = Library::default();
        let mut world = World::new();
        let root = world
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<crate::VoxelMaterial>::default();
        let opaque = materials.add(crate::solid_terrain_material());
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let key = Key::new(GroveFamily::JohtoPine, [4, 8], [true,false,true,false]);
        let coordinates = (-64..64).flat_map(|z| (-64..64).map(move |x| [x, z]))
            .filter(|&c| Key::new(GroveFamily::JohtoPine, c, [true,false,true,false]) == key)
            .take(64).collect::<Vec<_>>();
        assert_eq!(coordinates.len(), 64);
        let mut expected_roots = Vec::new();
        {
            let mut commands = Commands::new(&mut queue, &world);
            for (i, &coordinates) in coordinates.iter().enumerate() {
                let x = i as f32 * 40.;
                let offset = foliage::grove_trunk_offset(coordinates, 32.);
                expected_roots.push(Vec3::new(x + 16. + offset.x, 3., 16. + offset.y));
                let groups = library.prepare_with_ground(
                    [x, x + 32., 0., 32.],
                    3.,
                    16.,
                    coordinates,
                    [true,false,true,false],
                    GroveFamily::JohtoPine,
                    Some([x, x + 32., 0., 32.]),
                );
                occluder_fade::spawn_groups(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    root,
                    groups,
                    opaque.clone(),
                    None,
                );
            }
        }
        queue.apply(&mut world);
        let offset = foliage::grove_trunk_offset(seeds()[key.variant as usize], 32.);
        let prototype_root = Vec3::new(offset.x, 0., offset.y);
        let mut parents = world.query::<(&Mesh3d,&ChildOf)>();
        for (mesh,parent) in parents.iter(&world) {
            if meshes.get(&mesh.0).unwrap().count_vertices() <= crate::understory::MAX_VERTICES { continue; }
            // The prototype root is in group coordinates. Mesh children carry
            // only leaf-local centering; placement belongs to their group.
            let group = world.get::<Transform>(parent.parent()).unwrap();
            let placed_root = world.get::<Transform>(root).unwrap()
                .transform_point(group.transform_point(prototype_root));
            let index = expected_roots.iter().position(|expected|
                placed_root.distance(*expected) < 0.001).unwrap_or_else(||
                panic!("shared tree must use its world placement's trunk offset: {placed_root:?}"));
            expected_roots.swap_remove(index);
        }
        assert!(expected_roots.is_empty(), "every placement must own exactly one opaque draw");
        let mut query = world.query::<(&Mesh3d, &MeshMaterial3d<crate::VoxelMaterial>)>();
        let draws = query.iter(&world).collect::<Vec<_>>();
        assert_eq!(draws.len(), 128);
        assert!(
            draws
                .iter()
                .all(|(_, material)| material.0 == opaque)
        );
        assert_eq!(draws.iter().map(|(mesh,_)| mesh.0.id())
            .collect::<std::collections::HashSet<_>>().len(),2);
        assert_eq!(
            meshes.len(),
            2,
            "placement count must not duplicate uploaded tree or ground geometry"
        );
    }

    #[test]
    fn different_world_placements_reuse_one_prototype() {
        let key = Key::new(GroveFamily::JohtoPine, [4, 8], [false; 4]);
        let library = Library::default();
        let a = library.prototype(key, GroveFamily::JohtoPine);
        let b = library.prototype(key, GroveFamily::JohtoPine);
        assert!(Arc::ptr_eq(&a, &b));
        let other_app = Library::default();
        assert!(
            !Arc::ptr_eq(&a, &other_app.prototype(key, GroveFamily::JohtoPine)),
            "GPU handles must belong to one app asset store"
        );
        assert!(occluder_fade::prepared_bounds(&a.groups).is_some());
        let groups = library.prepare(
            [100., 132., 200., 232.],
            7.,
            16.,
            [4, 8],
            [false; 4],
            GroveFamily::JohtoPine,
        );
        let (low, high) = occluder_fade::prepared_bounds(&groups).unwrap();
        assert!(low.x >= 76. && high.x <= 156.);
        assert!(low.z >= 176. && high.z <= 256.);
        assert!(high.y > 7. + 16. * 2.);
    }
}
