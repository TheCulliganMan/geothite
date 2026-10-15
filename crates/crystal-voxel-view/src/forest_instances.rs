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

const VARIANTS: usize = 12; // three branch architectures, four height/phase recipes
const CAPACITY: usize = 160; // all 144 family/density/variant combinations fit
const PROTOTYPE_TILE: f32 = 16.;
const PROTOTYPE_BOUNDS: [f32; 4] = [-16., 16., -16., 16.];
type Prototype = Arc<Vec<PreparedGroup>>;
type Cell = Arc<OnceLock<Prototype>>;
#[derive(Default)]
pub(crate) struct Library {
    entries: Mutex<VecDeque<(Key, Cell)>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Key {
    family: u8,
    density: u8,
    variant: u8,
    sparse: bool,
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
            sparse: density == 4
                && (coordinates[0].rem_euclid(4) != 0 || coordinates[1].rem_euclid(4) != 0),
        }
    }
}

// Find stable source seeds for each authored architecture and height class.
// Dense interior plots retain the existing sparse-understory distribution.
fn seeds(sparse: bool) -> &'static [[i32; 2]; VARIANTS] {
    static SEEDS: OnceLock<[[[i32; 2]; VARIANTS]; 2]> = OnceLock::new();
    &SEEDS.get_or_init(|| {
        std::array::from_fn(|sparse| {
            let mut result = [[0; 2]; VARIANTS];
            let mut found = [false; VARIANTS];
            for z in 0..256 {
                for x in 0..256 {
                    let is_sparse = x % 4 != 0 || z % 4 != 0;
                    if is_sparse != (sparse == 1) {
                        continue;
                    }
                    let key = Key::new(GroveFamily::JohtoPine, [x, z], [true; 4]);
                    let variant = key.variant as usize;
                    if !found[variant] {
                        result[variant] = [x, z];
                        found[variant] = true;
                    }
                    if found.iter().all(|v| *v) {
                        return result;
                    }
                }
            }
            panic!("forest prototype source seeds must cover every authored shape/height class")
        })
    })[usize::from(sparse)]
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
        "sparse":key.sparse,"cache_hit":cell.get().is_some()})
        });
        cell.get_or_init(|| {
            let mut mesh = TerrainMeshData::default();
            let neighbors = std::array::from_fn(|i| i < key.density as usize);
            foliage::append_family_grove(
                &mut mesh,
                PROTOTYPE_BOUNDS,
                0.,
                PROTOTYPE_TILE,
                seeds(key.sparse)[key.variant as usize],
                neighbors,
                family,
            );
            Arc::new(crate::shared_groves::prepare_surface(mesh.solid))
        })
        .clone()
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
        let key = Key::new(family, coordinates, neighbors);
        let source = self.prototype(key, family);
        let (_, height, turn) = foliage::instance_variation(coordinates);
        let (_, prototype_height, prototype_turn) =
            foliage::instance_variation(seeds(key.sparse)[key.variant as usize]);
        let rotation = Quat::from_rotation_y(turn - prototype_turn);
        let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
        // The shared mesh already contains its seed's trunk offset. Replace it
        // with this placement's offset, independent of prototype and rotation.
        let prototype_offset = if key.sparse { Vec2::ZERO } else {
            foliage::grove_trunk_offset(seeds(false)[key.variant as usize], 32.)
        };
        let prototype_root = Vec3::new(prototype_offset.x, 0., prototype_offset.y);
        let offset = if key.sparse { Vec2::ZERO } else {
            foliage::grove_trunk_offset(coordinates, spacing)
        };
        let scale = Vec3::new(
            (bounds[1] - bounds[0]) / 32.,
            tile / PROTOTYPE_TILE * height / prototype_height,
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
        if let Some((low, high)) = occluder_fade::prepared_bounds(&source) {
            let margin = if key.sparse {
                (tile * 0.5 - spacing * 0.2).max(0.)
            } else {
                spacing * 0.75
            };
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
        transform.translation += Vec3::new(offset.x, 0., offset.y)
            - rotation * (transform.scale * prototype_root);
        let _trace = crate::loading_trace::span("tree.instance", || {
            serde_json::json!({
        "coordinates":coordinates,"variant":key.variant,"density":key.density,"sparse":key.sparse})
        });
        let mut groups = source.as_ref().clone();
        occluder_fade::place_groups(&mut groups, transform);
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                for sparse in [false, true] {
                    if sparse && density != 4 { continue; }
                    for &coordinates in seeds(sparse) {
                        let mut mesh = TerrainMeshData::default();
                        foliage::append_family_grove(&mut mesh, PROTOTYPE_BOUNDS, 0.,
                            PROTOTYPE_TILE, coordinates,
                            std::array::from_fn(|i| i < density), family);
                        assert_eq!(mesh.solid.uvs.len(), mesh.solid.positions.len());
                        assert!(mesh.solid.uvs.iter().all(|&uv| uv == [0.; 2]),
                            "{family:?}/{density}/{sparse}/{coordinates:?} requires UV shader behavior");
                        max_vertices = max_vertices.max(mesh.solid.positions.len());
                        indices += mesh.solid.indices.len();
                        count += 1;
                    }
                }
            }
        }
        assert_eq!(count, 144);
        println!("{count} full tree prototypes: max {max_vertices} vertices, {indices} indices; all UV0 zero");
    }

    #[test]
    fn library_has_complete_variety_and_bounded_world_independent_keys() {
        for sparse in [false, true] {
            for (variant, &coordinates) in seeds(sparse).iter().enumerate() {
                let key = Key::new(GroveFamily::JohtoPine, coordinates, [true; 4]);
                assert_eq!(key.variant as usize, variant);
                assert_eq!(key.sparse, sparse);
            }
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
        assert_eq!(keys.len(), 144);
        assert!(keys.len() <= CAPACITY);
    }
    #[test]
    fn many_tree_placements_upload_one_opaque_mesh_and_share_the_material() {
        let library = Library::default();
        let mut world = World::new();
        let root = world
            .spawn((Transform::default(), Visibility::Inherited))
            .id();
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<crate::VoxelMaterial>::default();
        let opaque = materials.add(crate::solid_terrain_material());
        let mut queue = bevy::ecs::world::CommandQueue::default();
        let key = Key::new(GroveFamily::JohtoPine, [4, 8], [false; 4]);
        let coordinates = (-64..64).flat_map(|z| (-64..64).map(move |x| [x, z]))
            .filter(|&c| Key::new(GroveFamily::JohtoPine, c, [false; 4]) == key)
            .take(64).collect::<Vec<_>>();
        assert_eq!(coordinates.len(), 64);
        let mut expected_roots = Vec::new();
        {
            let mut commands = Commands::new(&mut queue, &world);
            for (i, &coordinates) in coordinates.iter().enumerate() {
                let x = i as f32 * 40.;
                let offset = foliage::grove_trunk_offset(coordinates, 32.);
                expected_roots.push(Vec3::new(x + 16. + offset.x, 3., 16. + offset.y));
                let groups = library.prepare(
                    [x, x + 32., 0., 32.],
                    3.,
                    16.,
                    coordinates,
                    [false; 4],
                    GroveFamily::JohtoPine,
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
        let offset = foliage::grove_trunk_offset(seeds(false)[key.variant as usize], 32.);
        let prototype_root = Vec3::new(offset.x, 0., offset.y);
        let mut parents = world.query_filtered::<&ChildOf, With<Mesh3d>>();
        for parent in parents.iter(&world) {
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
        assert_eq!(draws.len(), 64);
        assert!(
            draws
                .iter()
                .all(|(mesh, material)| mesh.0 == draws[0].0.0 && material.0 == opaque)
        );
        assert_eq!(
            meshes.len(),
            1,
            "placement count must not duplicate uploaded tree geometry"
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
        assert!(occluder_fade::prepared_bounds(&a).is_some());
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
