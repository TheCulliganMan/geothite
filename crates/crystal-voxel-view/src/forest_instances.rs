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

// Density is fixed before the first terrain job so shared prototype metadata
// cannot mix policies. It is independent of authored model complexity.
static DENSE_UNDERSTORY: OnceLock<bool> = OnceLock::new();
pub(crate) fn dense_understory() -> bool {
    *DENSE_UNDERSTORY.get_or_init(|| {
        #[cfg(target_arch = "wasm32")]
        { false }
        #[cfg(not(target_arch = "wasm32"))]
        { std::env::var_os("CRYSTAL_UNDERSTORY_FULL_DETAIL").is_some() }
    })
}
pub(crate) fn validate_understory_density(dense: bool) -> Result<(), String> {
    if DENSE_UNDERSTORY.get().is_some_and(|current| *current != dense) {
        Err("terrain worker understory density configuration mismatch".into())
    } else { Ok(()) }
}
/// Startup-only density choice, also checked at the worker boundary.
pub fn configure_understory_density(dense: bool) -> Result<(), String> {
    if let Some(current) = DENSE_UNDERSTORY.get() {
        return if *current == dense { Ok(()) }
            else { Err("terrain worker understory density configuration mismatch".into()) };
    }
    DENSE_UNDERSTORY.set(dense)
        .map_err(|_| "understory density already configured".into())
}

static EAGER_SORTED_ENTITIES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Startup-only QA control; geometry, collisions and draw policy stay identical.
pub fn configure_lazy_tree_entities_diagnostics(eager: bool) {
    EAGER_SORTED_ENTITIES.store(eager, std::sync::atomic::Ordering::Relaxed);
}

const VARIANTS: usize = 12; // three branch architectures, four height/phase recipes
const CAPACITY: usize = 144; // 120 evergreen/ancient combinations, twelve deciduous and twelve park recipes
const PROTOTYPE_TILE: f32 = 16.;
const PROTOTYPE_BOUNDS: [f32; 4] = [-16., 16., -16., 16.];
struct PrototypeGeometry {
    groups: Vec<PreparedGroup>,
    trunk_bounds: Option<(Vec3, Vec3)>,
    root_outline: Vec<Vec2>,
}
type Prototype = Arc<PrototypeGeometry>;
type Cell = Arc<OnceLock<Prototype>>;
type GroundCell = Arc<OnceLock<Arc<Vec<PreparedGroup>>>>;

fn root_outline(roots: &[[f32;3]]) -> Vec<Vec2> {
    let mut points=roots.iter().map(|p|Vec2::new(p[0],p[2])).collect::<Vec<_>>();
    points.sort_by(|a,b|a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    points.dedup();
    if points.len()<3 {return points;}
    let mut hull:Vec<Vec2>=Vec::new();
    for &p in &points {
        while hull.len()>=2 && (hull[hull.len()-1]-hull[hull.len()-2])
            .perp_dot(p-hull[hull.len()-1])<=0. {hull.pop();}
        hull.push(p);
    }
    let lower=hull.len();
    for &p in points.iter().rev().skip(1) {
        while hull.len()>lower && (hull[hull.len()-1]-hull[hull.len()-2])
            .perp_dot(p-hull[hull.len()-1])<=0. {hull.pop();}
        hull.push(p);
    }
    hull.pop();
    hull
}

fn root_fit(outline:&[Vec2],prototype_root:Vec3,rotation:Quat,scale:Vec3,
    offset:Vec2,bounds:[f32;4])->f32 {
    let mut fit=1.0_f32;
    for p in outline {
        let p=rotation*(scale*(Vec3::new(p.x,0.,p.y)-prototype_root));
        for (v,offset,radius) in [(p.x,offset.x,(bounds[1]-bounds[0])*0.5),
            (p.z,offset.y,(bounds[3]-bounds[2])*0.5)] {
            if v.abs()>0. {fit=fit.min((radius-v.signum()*offset)/v.abs());}
        }
    }
    fit
}
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
        // Ordinary height uses its own age seed. Grouping by the older
        // variation height could squash a mature prototype into a sapling,
        // flattening foliage along with the trunk.
        let age=if matches!(family,GroveFamily::JohtoPine|GroveFamily::KantoDeciduous) {
            foliage::non_ancient_height_factor(coordinates)-0.5
        } else {(height-0.9)/0.7};
        let bucket=((age*4.).floor() as usize).min(3);
        let density = if matches!(family,GroveFamily::KantoDeciduous|GroveFamily::ParkBroadleaf) {0} else {neighbors.iter().filter(|&&v| v).count() as u8};
        Self {
            family: match family {GroveFamily::JohtoPine=>0,GroveFamily::AncientForest=>1,GroveFamily::KantoDeciduous=>2,GroveFamily::ParkBroadleaf=>3},
            density,
            variant: (shape * 4 + bucket) as u8,
        }
    }
}

// Every source plot retains a canopy, including the interior of continuous blocks.
fn seeds(family:GroveFamily) -> &'static [[i32; 2]; VARIANTS] {
    static PINES: OnceLock<[[i32; 2]; VARIANTS]> = OnceLock::new();
    static DECIDUOUS: OnceLock<[[i32; 2]; VARIANTS]> = OnceLock::new();
    static OTHER: OnceLock<[[i32; 2]; VARIANTS]> = OnceLock::new();
    static PARK: OnceLock<[[i32; 2]; VARIANTS]> = OnceLock::new();
    let cache=match family {
        GroveFamily::JohtoPine => &PINES,
        GroveFamily::KantoDeciduous => &DECIDUOUS,
        GroveFamily::AncientForest => &OTHER,
        GroveFamily::ParkBroadleaf => &PARK,
    };
    cache.get_or_init(|| {
        let mut result = [[0; 2]; VARIANTS];
        let mut found = [false; VARIANTS];
        for z in 0..256 {
            for x in 0..256 {
                let key = Key::new(family, [x,z], [true;4]);
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
                if family==GroveFamily::ParkBroadleaf {[-32.,32.,-32.,32.]} else {PROTOTYPE_BOUNDS},
                0.,
                PROTOTYPE_TILE,
                seeds(family)[key.variant as usize],
                neighbors,
                family,
            );
            let roots = {
                let count = if family == GroveFamily::JohtoPine { 13 } else { 17 };
                if family==GroveFamily::ParkBroadleaf {
                    mesh.solid.positions.iter().filter(|p|p[1].abs()<0.000001).copied().collect::<Vec<_>>()
                } else if family==GroveFamily::KantoDeciduous {
                    mesh.solid.positions.iter().filter(|p|p[1]<=PROTOTYPE_TILE*0.18).copied().collect::<Vec<_>>()
                } else {mesh.solid.positions[..count].to_vec()}
            };
            let trunk_bounds = Some({
                roots.iter().fold(
                    (Vec3::splat(f32::INFINITY),Vec3::splat(f32::NEG_INFINITY)),
                    |(low,high), &p| (low.min(Vec3::from_array(p)),high.max(Vec3::from_array(p))),
                )
            });
            let root_outline=if matches!(family,GroveFamily::KantoDeciduous|GroveFamily::ParkBroadleaf) {root_outline(&roots)} else {Vec::new()};
            let groups=crate::shared_groves::prepare_surface(mesh.solid);
            Arc::new(PrototypeGeometry {groups,trunk_bounds,root_outline})
        })
        .clone()
    }

    fn ground_prototype(&self, variant: usize) -> Arc<Vec<PreparedGroup>> {
        let cell = {
            let mut entries = self.ground.lock().unwrap();
            if let Some((_,cell)) = entries.iter().find(|(v,_)| *v == variant as u8) {
                cell.clone()
            } else {
                let cell = Arc::new(OnceLock::new());
                entries.push_back((variant as u8,cell.clone()));
                cell
            }
        };
        cell.get_or_init(|| {
            let mut mesh = crate::understory::prototype(variant);
            mesh.cutaway_ranges.push(0..mesh.positions.len());
            Arc::new(crate::shared_groves::prepare_surface(mesh))
        }).clone()
    }

    fn ground_plants(plot: crate::understory::Plot, base: f32, coordinates: [i32; 2],
        family: GroveFamily) -> Vec<crate::understory::Plant> {
        // Dedicated forests retain their dense floor. Trees, source encounter
        // grass and collision never enter this decorative-plant selection.
        if family == GroveFamily::AncientForest || dense_understory() {
            crate::understory::plants(plot, base, coordinates)
        } else {
            crate::understory::adaptive_plants(plot, base, coordinates)
        }
    }

    fn prepare_ground(&self, plot: crate::understory::Plot, base: f32,
        coordinates: [i32;2], family: GroveFamily) -> Vec<PreparedGroup> {
        Self::ground_plants(plot,base,coordinates,family).into_iter().flat_map(|plant| {
            Self::place_ground(&self.ground_prototype(plant.variant),plant)
        }).collect()
    }

    fn place_ground(prototype: &Arc<Vec<PreparedGroup>>, plant: crate::understory::Plant)
        -> Vec<PreparedGroup> {
        let mut groups = prototype.as_ref().clone();
        occluder_fade::place_groups(&mut groups,plant.transform);
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
        ground_bounds: Option<crate::understory::Plot>) -> Vec<PreparedGroup> {
        let key = Key::new(family, coordinates, neighbors);
        let source = self.prototype(key, family);
        let mut groups = Self::place_canopy(key, &source, bounds, base, tile, coordinates);
        if let Some(ground_bounds) = ground_bounds {
            groups.extend(self.prepare_ground(ground_bounds,base,coordinates,family));
        }
        Self::finish_instances(&mut groups);
        groups
    }

    /// Clone and place completed prototypes only. No initialization, geometry
    /// generation or waits on worker-owned locks occur on this frame-thread path.
    /// Busy metadata is a cache miss; ordinary asynchronous jobs retry it.
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
        ground_bounds: Option<crate::understory::Plot>) -> Option<Vec<PreparedGroup>> {
        let key = Key::new(family, coordinates, neighbors);
        let source = {
            let entries = self.entries.try_lock().ok()?;
            entries.iter().find(|(candidate,_)| *candidate == key)?.1.get()?.clone()
        };
        let plants = ground_bounds.map(|plot|Self::ground_plants(plot,base,coordinates,family))
            .unwrap_or_default();
        let ground = if plants.is_empty() { Vec::new() } else {
            let entries = self.ground.try_lock().ok()?;
            plants.into_iter().map(|plant| {
                let source = entries.iter().find(|(variant,_)| *variant == plant.variant as u8)?.1.get()?.clone();
                Some((source,plant))
            }).collect::<Option<Vec<_>>>()?
        };
        let _trace = crate::loading_trace::span("tree.prepare_cached_instance", ||
            serde_json::json!({"coordinates":coordinates,"variant":key.variant,"density":key.density}));
        let mut groups = Self::place_canopy(key, &source, bounds, base, tile, coordinates);
        for (source,plant) in ground {
            groups.extend(Self::place_ground(&source,plant));
        }
        Self::finish_instances(&mut groups);
        Some(groups)
    }

    fn place_canopy(key: Key, source: &Prototype, bounds: [f32;4], base: f32,
        tile: f32, coordinates: [i32;2]) -> Vec<PreparedGroup> {
        let family = match key.family {0=>GroveFamily::JohtoPine,1=>GroveFamily::AncientForest,2=>GroveFamily::KantoDeciduous,3=>GroveFamily::ParkBroadleaf,_=>unreachable!("validated grove family")};
        let turn = foliage::instance_turn(family,coordinates);
        let prototype_turn = foliage::instance_turn(family,seeds(family)[key.variant as usize]);
        let vertical_ratio = foliage::instance_vertical_ratio(family,coordinates,
            seeds(family)[key.variant as usize], std::array::from_fn(|i| i < key.density as usize));
        let rotation = Quat::from_rotation_y(turn - prototype_turn);
        let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
        // The shared mesh already contains its seed's trunk offset. Replace it
        // with this placement's offset, independent of prototype and rotation.
        let prototype_span = if family==GroveFamily::ParkBroadleaf {64.} else {32.};
        let prototype_offset = if family==GroveFamily::ParkBroadleaf {Vec2::ZERO}
            else {foliage::grove_trunk_offset(seeds(family)[key.variant as usize], 32.)};
        let prototype_root = Vec3::new(prototype_offset.x, 0., prototype_offset.y);
        let offset = if family==GroveFamily::ParkBroadleaf {Vec2::ZERO}
            else {foliage::grove_trunk_offset(coordinates, spacing)};
        let width_ratio=foliage::instance_width_ratio(family,coordinates,seeds(family)[key.variant as usize]);
        let scale = Vec3::new(
            (bounds[1] - bounds[0]) / prototype_span * width_ratio,
            tile / PROTOTYPE_TILE * vertical_ratio,
            (bounds[3] - bounds[2]) / prototype_span * width_ratio,
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
        // Crown growth and yaw retain their family-specific overhang allowance.
        // Fit the shared prototype with one instance scale;
        // never rewrite vertices, normals or indices for placement.
        if let Some((low, high)) = occluder_fade::prepared_bounds(&source.groups) {
            let margin = spacing * foliage::instance_canopy_margin(family,coordinates);
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
        // Fit the real broadleaf root outline at arbitrary yaw. Rotating its
        // enclosing rectangle can otherwise squeeze the entire mature crown.
        // Existing evergreen/ancient footprints retain their four-corner fit.
        if let Some((low,high)) = source.trunk_bounds {
            let corners=std::array::from_fn::<_,4,_>(|i|Vec2::new(
                if i&1==0 {low.x}else{high.x},if i&2==0 {low.z}else{high.z}));
            let outline=if source.root_outline.is_empty() {&corners[..]} else {&source.root_outline};
            let fit=root_fit(outline,prototype_root,rotation,transform.scale,offset,bounds);
            transform.scale.x *= fit;
            transform.scale.z *= fit;
        }
        transform.translation += Vec3::new(offset.x, 0., offset.y)
            - rotation * (transform.scale * prototype_root);
        let _trace = crate::loading_trace::span("tree.instance", || {
            serde_json::json!({
        "coordinates":coordinates,"variant":key.variant,"density":key.density,
        "family":key.family,"base":base,"tile":tile,"bounds":bounds,
        "translation":transform.translation.to_array(),"scale":transform.scale.to_array()})
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
    fn park_instances_keep_full_crowns_mature_height_and_clear_paths_in_twelve_shared_recipes() {
        let library = Library::default();
        let family = GroveFamily::ParkBroadleaf;
        let bounds = [-32.,32.,-32.,32.];
        let mut coordinates = seeds(family).to_vec();
        coordinates.extend([[0,0],[4,8],[-12,20]]);
        let mut heights = Vec::new();
        for at in coordinates {
            let key = Key::new(family,at,[false;4]);
            assert_eq!(key,Key::new(family,at,[true;4]));
            let prototype = library.prototype(key,family);
            assert!(Arc::ptr_eq(&prototype,&library.prototype(key,family)));
            let groups = library.prepare(bounds,3.,16.,at,[false;4],family);
            assert_eq!(groups.len(),1,"one planted tree keeps one reveal owner");
            let triangles = occluder_fade::geometry_signature(&crate::mesh::SurfaceMeshData::default(),groups);
            let mut original = TerrainMeshData::default();
            foliage::append_family_canopy(&mut original,bounds,3.,16.,at,[false;4],family);
            assert_eq!(triangles.len(),original.solid.indices.len()/3,
                "instances retain every full-detail tree triangle");
            let mut top=3.0_f32;
            let mut low=Vec2::splat(f32::INFINITY);
            let mut high=Vec2::splat(f32::NEG_INFINITY);
            for triangle in triangles {
                for vertex in triangle.chunks_exact(12) {
                    let p=Vec3::new(f32::from_bits(vertex[0]),f32::from_bits(vertex[1]),f32::from_bits(vertex[2]));
                    assert!(p.is_finite() && p.y>=2.9999);
                    top=top.max(p.y); low=low.min(Vec2::new(p.x,p.z)); high=high.max(Vec2::new(p.x,p.z));
                    if p.x.abs()>32.0001 || p.z.abs()>32.0001 {
                        assert!(p.y-3.>16.*3.,"branches over a promenade must clear visitors: {p:?}");
                    }
                }
            }
            assert!(top-3.>16.*9.,"shared park trees must retain their mature height");
            assert!((high-low).max_element()>16.*6.,"the crown must remain broad");
            heights.push(top);
        }
        assert_eq!(library.entries.lock().unwrap().len(),12);
        let shortest=heights.iter().copied().fold(f32::INFINITY,f32::min);
        let tallest=heights.iter().copied().fold(f32::NEG_INFINITY,f32::max);
        assert!(tallest-shortest>16.*2.,"instances retain real height variation");
    }

    #[test]
    fn arbitrary_tree_yaw_fits_the_actual_roots_without_false_rectangle_squeezing() {
        let roots=[[1.,0.,0.],[0.,0.,1.],[-1.,0.,0.],[0.,0.,-1.],[0.,0.,0.]];
        let outline=root_outline(&roots);
        assert_eq!(outline.len(),4);
        for step in 0..72 {
            let rotation=Quat::from_rotation_y(step as f32*std::f32::consts::TAU/72.);
            let fit=root_fit(&outline,Vec3::ZERO,rotation,Vec3::ONE,Vec2::ZERO,[-1.,1.,-1.,1.]);
            assert!((fit-1.).abs()<0.00001,"rotating roots already inside their plot must retain crown width");
            for bounds in [[-0.6,0.6,-1.,1.],[-1.,1.,-0.5,0.5]] {
                let offset=Vec2::new(0.12,-0.08);
                let fit=root_fit(&outline,Vec3::ZERO,rotation,Vec3::ONE,offset,bounds);
                for point in roots {
                    let p=rotation*(Vec3::from_array(point)*fit);
                    assert!(p.x+offset.x>=bounds[0]-0.00001 && p.x+offset.x<=bounds[1]+0.00001
                        && p.z+offset.y>=bounds[2]-0.00001 && p.z+offset.y<=bounds[3]+0.00001);
                }
            }
        }
    }

    #[test]
    fn all_broadleaf_root_hulls_protect_native_contacts_at_continuous_yaw() {
        let family=GroveFamily::KantoDeciduous;
        for at in seeds(family) {
            let mut mesh=TerrainMeshData::default();
            foliage::append_family_canopy(&mut mesh,PROTOTYPE_BOUNDS,0.,PROTOTYPE_TILE,*at,[false;4],family);
            let roots=mesh.solid.positions.iter().filter(|p|p[1]<=PROTOTYPE_TILE*0.18).copied().collect::<Vec<_>>();
            let outline=root_outline(&roots);
            assert!(outline.len()>=3 && outline.len()<100);
            let offset=foliage::grove_trunk_offset(*at,32.);
            let origin=Vec3::new(offset.x,0.,offset.y);
            for step in 0..24 {
                let rotation=Quat::from_rotation_y(step as f32*std::f32::consts::TAU/24.);
                let scale=Vec3::new(1.15,1.,0.9);
                let fit=root_fit(&outline,origin,rotation,scale,offset,PROTOTYPE_BOUNDS);
                for p in &roots {
                    let p=rotation*(scale*fit*(Vec3::from_array(*p)-origin))+origin;
                    assert!(p.x>=-16.0001 && p.x<=16.0001 && p.z>=-16.0001 && p.z<=16.0001,
                        "the real modeled root must remain in its source plot at every yaw");
                }
            }
        }
    }


    #[test]
    fn deciduous_placements_share_three_branch_models_in_four_bounded_age_classes() {
        let library=Library::default();let mut keys=Vec::new();
        for z in -12..12 {for x in -12..12 {
            for neighbors in [[false;4],[true;4],[true,false,true,false]] {
                let key=Key::new(GroveFamily::KantoDeciduous,[x,z],neighbors);
                if !keys.contains(&key) {keys.push(key);}
                assert_eq!(key,Key::new(GroveFamily::KantoDeciduous,[x,z],[false;4]));
            }
        }}
        assert_eq!(keys.len(),12,"three branch models need four leaf-preserving age classes");
        let mut signatures=Vec::new();
        for key in keys {
            assert_eq!(Key::new(GroveFamily::KantoDeciduous,
                seeds(GroveFamily::KantoDeciduous)[key.variant as usize],[false;4]),key);
            let source=library.prototype(key,GroveFamily::KantoDeciduous);
            let same=library.prototype(key,GroveFamily::KantoDeciduous);
            assert!(Arc::ptr_eq(&source,&same));
            signatures.push(occluder_fade::geometry_signature(
                &crate::mesh::SurfaceMeshData::default(),source.groups.clone()));
        }
        for (i,signature) in signatures.iter().enumerate() {
            assert!(!signatures[..i].contains(signature));
        }
        assert_eq!(library.entries.lock().unwrap().len(),12);
    }

    #[test]
    fn adding_understory_preserves_every_tree_triangle_and_placement() {
        let library = Library::default();
        for family in [GroveFamily::JohtoPine,GroveFamily::AncientForest,GroveFamily::KantoDeciduous] {
        let tree = library.prepare_with_ground([-16.,16.,-16.,16.],3.,16.,[4,8],
            [true;4],family,None);
        let augmented = library.prepare_with_ground([-16.,16.,-16.,16.],3.,16.,[4,8],
            [true;4],family,Some(crate::understory::Plot::isolated([-32.,32.,-32.,32.])));
        assert!(augmented.len() > tree.len());
        let signature = |groups| occluder_fade::geometry_signature(
            &crate::mesh::SurfaceMeshData::default(),groups);
        assert_eq!(signature(tree.clone()),signature(augmented[..tree.len()].to_vec()),
            "adding ground flora must not alter any existing tree geometry or transform");
        }
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
            GroveFamily::JohtoPine).is_some(), "a canopy-only request does not need ground cover");
        assert!(library.prepare_if_cached_with_ground(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine,Some(crate::understory::Plot::isolated(bounds))).is_none(), "missing requested ground cover must remain asynchronous");
        assert!(library.ground.lock().unwrap().is_empty());
        let expected = library.prepare_with_ground(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine,Some(crate::understory::Plot::isolated(bounds)));
        let count = library.entries.lock().unwrap().len();
        let ground_count = library.ground.lock().unwrap().len();
        let cached = library.prepare_if_cached_with_ground(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine,Some(crate::understory::Plot::isolated(bounds))).unwrap();
        let signature = |groups| occluder_fade::geometry_signature(
            &crate::mesh::SurfaceMeshData::default(),groups);
        assert_eq!(signature(cached),signature(expected));
        assert_eq!(library.entries.lock().unwrap().len(),count);
        assert_eq!(library.ground.lock().unwrap().len(),ground_count);
        assert!(Arc::ptr_eq(&source,&library.prototype(key,GroveFamily::JohtoPine)));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cached_frame_path_defers_busy_worker_metadata_and_recovers_exact_geometry() {
        let library = Library::default();
        let bounds = [-16.,16.,-16.,16.];
        let coordinates = [4,8];
        let neighbors = [true;4];
        let plot = crate::understory::Plot::isolated(bounds);
        let expected = library.prepare_with_ground(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine,Some(plot));
        // Holding the actual worker mutex on this thread makes an accidental
        // blocking lookup deadlock, independent of scheduler timing.
        {
            let _worker = library.entries.lock().unwrap();
            assert!(library.prepare_if_cached(bounds,3.,16.,coordinates,neighbors,
                GroveFamily::JohtoPine).is_none());
        }
        {
            let _worker = library.ground.lock().unwrap();
            assert!(library.prepare_if_cached_with_ground(bounds,3.,16.,coordinates,neighbors,
                GroveFamily::JohtoPine,Some(plot)).is_none());
            assert!(library.prepare_if_cached(bounds,3.,16.,coordinates,neighbors,
                GroveFamily::JohtoPine).is_some(), "canopy-only placements must not touch ground metadata");
        }
        let recovered = library.prepare_if_cached_with_ground(bounds,3.,16.,coordinates,neighbors,
            GroveFamily::JohtoPine,Some(plot)).unwrap();
        let signature = |groups| occluder_fade::geometry_signature(
            &crate::mesh::SurfaceMeshData::default(),groups);
        assert_eq!(signature(expected),signature(recovered));
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
                for &coordinates in seeds(family) {
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
    fn pine_instances_keep_branch_depth_across_young_and_mature_heights() {
        let family=GroveFamily::JohtoPine;
        let mut low=f32::INFINITY;
        let mut high=0.0_f32;
        for z in -64..64 {for x in -64..64 {for neighbors in [[false;4],[true;4]] {
            let coordinates=[x,z];
            let key=Key::new(family,coordinates,neighbors);
            let prototype=seeds(family)[key.variant as usize];
            let ratio=foliage::instance_vertical_ratio(family,coordinates,prototype,neighbors);
            low=low.min(ratio);high=high.max(ratio);
            assert!((0.60..1.70).contains(&ratio),
                "shared foliage must not flatten or stretch its needle volume: {coordinates:?}, {prototype:?}, {ratio}");
        }}}
        println!("pine prototype vertical ratios: {low:.6}..{high:.6}");
    }

    #[test]
    fn deciduous_instances_keep_leaf_depth_across_young_and_mature_heights() {
        let family=GroveFamily::KantoDeciduous;
        let mut low=f32::INFINITY;
        let mut high=0.0_f32;
        for z in -64..64 {for x in -64..64 {
            let at=[x,z];let key=Key::new(family,at,[false;4]);
            let prototype=seeds(family)[key.variant as usize];
            let ratio=foliage::instance_vertical_ratio(family,at,prototype,[false;4]);
            low=low.min(ratio);high=high.max(ratio);
            assert!((0.60..1.70).contains(&ratio),
                "shared broadleaf blades must retain their depth: {at:?}, {prototype:?}, {ratio}");
        }}
        println!("deciduous prototype vertical ratios: {low:.6}..{high:.6}");
    }

    #[test]
    fn library_has_complete_variety_and_bounded_world_independent_keys() {
        for family in [GroveFamily::JohtoPine,GroveFamily::AncientForest] {
        for (variant, &coordinates) in seeds(family).iter().enumerate() {
            let key = Key::new(family, coordinates, [true; 4]);
            assert_eq!(key.variant as usize, variant);
        }}
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
    fn many_tree_and_ground_placements_share_a_bounded_plant_library_and_the_material() {
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
        let mut expected_draws = 64;
        {
            let mut commands = Commands::new(&mut queue, &world);
            for (i, &coordinates) in coordinates.iter().enumerate() {
                let x = i as f32 * 40.;
                let offset = foliage::grove_trunk_offset(coordinates, 32.);
                expected_roots.push(Vec3::new(x + 16. + offset.x, 3., 16. + offset.y));
                expected_draws += Library::ground_plants(
                    crate::understory::Plot::isolated([x,x+32.,0.,32.]),3.,coordinates,GroveFamily::JohtoPine).len();
                let groups = library.prepare_with_ground(
                    [x, x + 32., 0., 32.],
                    3.,
                    16.,
                    coordinates,
                    [true,false,true,false],
                    GroveFamily::JohtoPine,
                    Some(crate::understory::Plot::isolated([x, x + 32., 0., 32.])),
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
        let offset = foliage::grove_trunk_offset(seeds(GroveFamily::JohtoPine)[key.variant as usize], 32.);
        let prototype_root = Vec3::new(offset.x, 0., offset.y);
        let mut parents = world.query::<(&Mesh3d,&ChildOf)>();
        for (mesh,parent) in parents.iter(&world) {
            let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
                meshes.get(&mesh.0).unwrap().attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
                else {panic!("expected plant/tree positions")};
            let bottom=positions.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min);
            let top=positions.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max);
            if top-bottom < PROTOTYPE_TILE {continue;}
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
        assert_eq!(draws.len(), expected_draws);
        assert!(
            draws
                .iter()
                .all(|(_, material)| material.0 == opaque)
        );
        assert_eq!(draws.iter().map(|(mesh,_)| mesh.0.id())
            .collect::<std::collections::HashSet<_>>().len(),meshes.len());
        assert!(meshes.len() > 2 && meshes.len() <= 1 + crate::understory::VARIANTS,
            "placements must share the bounded species/palette library");
        assert!(library.ground.lock().unwrap().len() <= crate::understory::VARIANTS);
    }

    #[test]
    fn shared_pines_retain_full_leader_height_at_every_plot_scale() {
        let library = Library::default();
        for coordinates in seeds(GroveFamily::JohtoPine).iter().copied().chain([[4,8],[-12,4],[29,4]]) {
            for neighbors in [[false;4], [true;4], [true,false,true,false]] {
                for tile in [8.,16.] {
                    let groups = library.prepare([0.,tile*2.,0.,tile*2.],3.,tile,
                        coordinates,neighbors,GroveFamily::JohtoPine);
                    let (low,high) = occluder_fade::prepared_bounds(&groups).unwrap();
                    let mut direct = TerrainMeshData::default();
                    foliage::append_family_canopy(&mut direct,[0.,tile*2.,0.,tile*2.],
                        3.,tile,coordinates,neighbors,GroveFamily::JohtoPine);
                    let leader = direct.solid.positions[12*13][1];
                    assert!((low.y - 3.).abs() < 0.001);
                    assert!(high.y >= leader - 0.001,
                        "runtime prototype compressed the mature leader: {coordinates:?}, {neighbors:?}, {tile}, {high:?}, {leader}");
                    assert!(high.y > 3. + tile * 2.6,
                        "young trees must keep their complete planted height");
                }
            }
        }
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
