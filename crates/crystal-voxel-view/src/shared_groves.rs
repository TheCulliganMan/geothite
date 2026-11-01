//! Share exact, source-seeded grove geometry across identical preparations.
//! Placement, directional neighbors, colors and independent fades stay intact.
use crate::{
    mesh::{SurfaceMeshData, TerrainMeshData},
    occluder_fade::{self, PreparedGroup, WeakPreparedGroup},
};
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    family: crate::foliage::GroveFamily,
    bounds: [u32; 4],
    base: u32,
    tile: u32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
}
// Bound lookup metadata, without retaining retired CPU/BVH/GPU geometry.
// Live instances own their shared handles independently of cache eviction.
const CAPACITY: usize = 64;
static TEMPLATES: OnceLock<Mutex<VecDeque<(Key, Vec<WeakPreparedGroup>)>>> = OnceLock::new();
static DIAGNOSTIC_EAGER_COLLISION: OnceLock<bool> = OnceLock::new();

/// Select the pre-optimization collision path before the first grove is
/// prepared. A startup-only control keeps same-binary fidelity comparisons
/// independent of source/artwork changes between compiled builds.
pub fn configure_grove_collision_diagnostics(eager: bool) -> Result<(), String> {
    DIAGNOSTIC_EAGER_COLLISION.set(eager)
        .map_err(|_| "grove collision diagnostics already configured".to_owned())
}

pub(crate) fn prepare(
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
) -> Vec<PreparedGroup> {
    prepare_family(bounds,base,tile,coordinates,neighbors,crate::foliage::GroveFamily::JohtoPine)
}

pub(crate) fn prepare_family(bounds:[f32;4],base:f32,tile:f32,coordinates:[i32;2],neighbors:[bool;4],
    family:crate::foliage::GroveFamily) -> Vec<PreparedGroup> {
    let key = Key {
        family,
        bounds: bounds.map(f32::to_bits),
        base: base.to_bits(),
        tile: tile.to_bits(),
        coordinates,
        neighbors,
    };
    cached_prepare(TEMPLATES.get_or_init(|| Mutex::new(VecDeque::new())), key, || {
        // Generate at the actual placement. Centering/scaling a canonical tree
        // changes f32 branch normals, world-anchored understory and seed variation.
        let mut source = TerrainMeshData::default();
        crate::foliage::append_family_grove(&mut source,bounds,base,tile,coordinates,neighbors,family);
        prepare_surface(source.solid)
    })
}

fn lookup(cache: &mut VecDeque<(Key, Vec<WeakPreparedGroup>)>, key: Key) -> Option<Vec<PreparedGroup>> {
    let index = cache.iter().position(|(candidate, _)| *candidate == key)?;
    let entry = cache.remove(index).unwrap();
    let groups = entry.1.iter().map(WeakPreparedGroup::upgrade).collect::<Option<Vec<_>>>()?;
    cache.push_back(entry);
    Some(groups)
}

fn cached_prepare(
    cache: &Mutex<VecDeque<(Key, Vec<WeakPreparedGroup>)>>,
    key: Key,
    build: impl FnOnce() -> Vec<PreparedGroup>,
) -> Vec<PreparedGroup> {
    let existing = lookup(&mut cache.lock().unwrap(), key);
    let _trace = crate::loading_trace::span("tree.shared_instance", || serde_json::json!({
        "coordinates":key.coordinates,"neighbors":key.neighbors,"cache_hit":existing.is_some()}));
    if let Some(groups) = existing { return groups; }
    // Never hold the cache lock during generation/BVH construction: native
    // background jobs build independent recipes concurrently.
    let groups = build();
    let mut cache = cache.lock().unwrap();
    // Another native job may have completed this exact recipe while we built.
    if let Some(existing) = lookup(&mut cache, key) { return existing; }
    if cache.len() == CAPACITY { cache.pop_front(); }
    cache.push_back((key, groups.iter().map(WeakPreparedGroup::new).collect()));
    groups
}

pub(crate) fn prepare_surface(source: SurfaceMeshData) -> Vec<PreparedGroup> {
    let (opaque, mut groups) = if DIAGNOSTIC_EAGER_COLLISION.get().copied().unwrap_or(false) {
        occluder_fade::prepare(source)
    } else { occluder_fade::prepare_grove(source) };
    occluder_fade::use_coverage_fade(&mut groups);
    assert!(
        opaque.indices.is_empty(),
        "a complete source tree owns every triangle"
    );
    let material = crate::solid_terrain_material();
    if crate::compact_vertices::solid_uv0_is_unused(
        &material.base,
        material.extension.water_enabled,
    ) {
        occluder_fade::compact_palette_group_uv0(&mut groups);
    }
    // The worker wire currently carries float attributes. Keep its ABI and
    // browser output unchanged; native render copies use Bevy's own decoder.
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "terrain-worker")))]
    if std::env::var("CRYSTAL_PACKED_GROVE_NORMALS").as_deref() != Ok("off") {
        occluder_fade::pack_palette_group_normals(&mut groups);
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "terrain-worker")))]
    occluder_fade::localize_opaque_triangles(&mut groups);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "terrain-worker")))]
    occluder_fade::localize_opaque_vertex_fetch(&mut groups);
    occluder_fade::share_groups(&mut groups);
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ancient_and_pine_templates_never_share_a_cached_family() {
        use crate::foliage::GroveFamily::{AncientForest,JohtoPine};
        let bounds=[-12.,8.,-9.,3.];
        let pine=prepare_family(bounds,3.,8.,[4,8],[false;4],JohtoPine);
        let forest=prepare_family(bounds,3.,8.,[4,8],[false;4],AncientForest);
        let signature=|groups|crate::occluder_fade::geometry_signature(&SurfaceMeshData::default(),groups);
        assert_ne!(signature(pine.clone()),signature(forest.clone()));
        let again=prepare_family(bounds,3.,8.,[4,8],[false;4],AncientForest);
        assert_eq!(signature(forest),signature(again));
    }
    #[test]
    fn generation_does_not_hold_cache_lock_and_retired_templates_are_rebuilt() {
        let cache = Mutex::new(VecDeque::new());
        let key = Key { family:crate::foliage::GroveFamily::JohtoPine,
            bounds: [-12., 8., -9., 3.].map(f32::to_bits), base: 3f32.to_bits(),
            tile: 8f32.to_bits(), coordinates: [8, 12], neighbors: [false; 4] };
        let builds = std::cell::Cell::new(0);
        let build = || {
            assert!(cache.try_lock().is_ok(), "independent native jobs must not serialize on generation");
            builds.set(builds.get() + 1);
            let mut source = TerrainMeshData::default();
            crate::foliage::append_grove_tree(&mut source, [-12., 8., -9., 3.], 3., 8., [8, 12], [false; 4]);
            prepare_surface(source.solid)
        };
        let first = cached_prepare(&cache, key, build);
        let second = cached_prepare(&cache, key, build);
        assert_eq!(builds.get(), 1, "live geometry must be reused");
        drop(first); drop(second);
        let rebuilt = cached_prepare(&cache, key, build);
        assert_eq!(builds.get(), 2, "lookup metadata must not keep retired geometry alive");
        assert!(!rebuilt.is_empty());
    }

    #[test]
    fn cache_retains_exact_source_seed_neighbors_and_placement_geometry() {
        for coordinates in [[-3, 4], [8, 12], [17, -11]] {
            for neighbors in [
                [false; 4],
                [true, false, true, false],
                [false, true, false, true],
                [true; 4],
            ] {
                let bounds = [-12., 8., -9., 3.];
                let mut source = TerrainMeshData::default();
                crate::foliage::append_grove_tree(
                    &mut source,
                    bounds,
                    3.,
                    8.,
                    coordinates,
                    neighbors,
                );
                let expected = crate::occluder_fade::geometry_signature(
                    &SurfaceMeshData::default(),
                    prepare_surface(source.solid),
                );
                for _ in 0..2 {
                    let actual = crate::occluder_fade::geometry_signature(
                        &SurfaceMeshData::default(),
                        prepare(bounds, 3., 8., coordinates, neighbors),
                    );
                    assert!(
                        actual == expected,
                        "exact source geometry differs for {coordinates:?}/{neighbors:?}"
                    );
                }
            }
        }
    }
}
