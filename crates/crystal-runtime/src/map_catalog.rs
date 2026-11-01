//! Share unchanged pack maps while preserving authoritative block writes.

use std::sync::Arc;

use crystal_core::state::GameState;
use crystal_core::world::map::OverworldMapData;

use crate::{CrystalRuntime, RuntimeMapCatalogSnapshot};

impl CrystalRuntime {
    pub(super) fn map_catalog_snapshot(
        &self,
        active_map: &OverworldMapData,
        state: &GameState,
    ) -> Vec<Arc<RuntimeMapCatalogSnapshot>> {
        self.map_catalog
            .iter()
            .map(|base| {
                let map_name = base.map_name.as_str();
                if active_map.name == map_name {
                    // Live blocks take precedence over saved overrides, even
                    // when a callback has restored the original pack blocks.
                    if base.blocks == active_map.metatile_ids {
                        return Arc::clone(base);
                    }
                    let mut snapshot = (**base).clone();
                    snapshot.blocks.clone_from(&active_map.metatile_ids);
                    Arc::new(snapshot)
                } else if let Some(overrides) = state.map_block_overrides.get(map_name) {
                    // Connected maps can expose writes from a previous visit.
                    let mut snapshot = (**base).clone();
                    for ((x, y), block_id) in overrides {
                        let index = usize::from(*y) * usize::from(snapshot.attributes.width)
                            + usize::from(*x);
                        snapshot.blocks[index] = *block_id;
                    }
                    Arc::new(snapshot)
                } else {
                    Arc::clone(base)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::sync::Arc;

    use crystal_core::state::GameState;
    use crystal_core::world::map::OverworldMapData;

    use crate::test_support;

    #[test]
    fn map_snapshots_share_originals_and_isolate_live_and_neighbor_writes() {
        let (path, _, mut runtime) =
            test_support::load_minimal_compiled_runtime("shared-map-snapshots");
        let base = Arc::clone(&runtime.map_catalog[0]);
        let mut neighbor = (*base).clone();
        neighbor.map_name = "Neighbor".to_string();
        let neighbor = Arc::new(neighbor);
        runtime.map_catalog = vec![Arc::clone(&base), Arc::clone(&neighbor)];
        let mut active = OverworldMapData {
            name: base.map_name.clone(),
            width: base.attributes.width,
            height: base.attributes.height,
            border_block: u16::from(base.attributes.border_block),
            connections: base.attributes.connections.clone(),
            metatile_ids: base.blocks.clone(),
        };
        let mut state = GameState::default();
        let original = runtime.map_catalog_snapshot(&active, &state);
        assert!(Arc::ptr_eq(&original[0], &base));
        assert!(Arc::ptr_eq(&original[1], &neighbor));

        active.metatile_ids[0] = 1;
        state
            .map_block_overrides
            .insert(neighbor.map_name.clone(), BTreeMap::from([((0, 0), 2)]));
        let changed = runtime.map_catalog_snapshot(&active, &state);
        assert_eq!(changed[0].blocks[0], 1);
        assert_eq!(changed[1].blocks[0], 2);
        assert!(!Arc::ptr_eq(&changed[0], &base));
        assert!(!Arc::ptr_eq(&changed[1], &neighbor));
        assert_eq!(original[0].blocks, base.blocks);
        assert_eq!(original[1].blocks, neighbor.blocks);
        assert_eq!(base.blocks[0], 0);
        assert_eq!(neighbor.blocks[0], 0);

        // A callback can restore the live map before its persisted overrides
        // are rewritten. Rendering must continue to use the live map.
        active.metatile_ids.clone_from(&base.blocks);
        state
            .map_block_overrides
            .insert(base.map_name.clone(), BTreeMap::from([((0, 0), 3)]));
        let restored = runtime.map_catalog_snapshot(&active, &state);
        assert!(Arc::ptr_eq(&restored[0], &base));
        assert_eq!(restored[1].blocks[0], 2);
        assert_eq!(changed[0].blocks[0], 1);
        fs::remove_dir_all(path).expect("remove shared map fixture");
    }
}
