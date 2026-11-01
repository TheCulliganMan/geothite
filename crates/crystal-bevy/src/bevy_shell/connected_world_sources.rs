//! Resolve scenery by resident acreage rather than a fixed number of maps.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use anyhow::Result;
use bevy::prelude::{IVec2, UVec2};

use super::{
    VisibleMapCatalogEntry, VisualWorldWindow, connected_render_map_placements_to_depth,
    connected_render_map_placements_with_window,
};

// Include complete drawings and their neighboring source context before they
// reach the resident edge. The regular four-hop context remains unchanged.
const SOURCE_CONTEXT: i32 = 16;

pub(super) fn placements<'a, M: VisibleMapCatalogEntry>(
    active: &str,
    maps: &'a [M],
    window: VisualWorldWindow,
) -> Result<Vec<(&'a crate::RuntimeMapCatalogSnapshot, i32, i32)>> {
    let nearby = connected_render_map_placements_to_depth(active, maps, Some(4))?;
    let known = nearby
        .iter()
        .map(|(map, _, _)| map.map_name.as_str())
        .collect::<HashSet<_>>();
    // Traversal reads only placement metadata. Canonical per-map visitation
    // makes cycles finite without loading distant tilesets or building meshes.
    // Stop at the physical coverage edge on every hop. A long game-map loop
    // can fold back into these coordinates; that does not make its far end
    // adjacent scenery when the path to it already left the resident area.
    let component = connected_render_map_placements_with_window(
        active,
        maps,
        None,
        Some(VisualWorldWindow {
            origin: window.origin - IVec2::splat(SOURCE_CONTEXT),
            size: window.size + UVec2::splat(SOURCE_CONTEXT as u32 * 2),
        }),
    )?;
    let distant = component
        .into_iter()
        .filter(|(map, _, _)| !known.contains(map.map_name.as_str()));
    // Active acreage wins first. Existing near-neighbor overlap priority stays
    // intact because source lookup searches the remaining entries in reverse.
    Ok(nearby
        .iter()
        .take(1)
        .copied()
        .chain(distant)
        .chain(nearby.iter().skip(1).copied())
        .collect())
}

/// Include distant connected edits in the retained field's invalidation key.
/// This is metadata only; it must agree with speculative source publication.
pub(super) fn block_identity<M: VisibleMapCatalogEntry>(active: &str, maps: &[M]) -> u64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    active.hash(&mut hash);
    match connected_render_map_placements_to_depth(active, maps, None) {
        Ok(sources) => {
            for (map, x, y) in sources {
                map.map_name.hash(&mut hash);
                map.blocks.hash(&mut hash);
                (x, y).hash(&mut hash);
            }
        }
        Err(error) => error.to_string().hash(&mut hash),
    }
    hash.finish()
}

#[cfg(test)]
mod tests {
    use bevy::prelude::UVec2;
    use crystal_core::map::{MapAttributes, MapConnection, MapEvents, MapSceneTable};

    use super::*;

    fn chain(direction: &str, count: usize) -> Vec<crate::RuntimeMapCatalogSnapshot> {
        let opposite = match direction {
            "north" => "south",
            "south" => "north",
            "east" => "west",
            "west" => "east",
            _ => panic!("test requires a cardinal connection"),
        };
        (0..count)
            .map(|index| {
                let mut connections = Vec::new();
                for (target, direction) in [
                    (index.checked_sub(1), opposite),
                    ((index + 1 < count).then_some(index + 1), direction),
                ] {
                    if let Some(target) = target {
                        connections.push(MapConnection {
                            direction: direction.into(),
                            target_map: format!("Region{target}"),
                            offset: 0,
                        });
                    }
                }
                crate::RuntimeMapCatalogSnapshot {
                    map_name: format!("Region{index}"),
                    id: format!("Region{index}"),
                    attributes: MapAttributes {
                        tileset_name: "johto".into(),
                        border_block: 0,
                        width: 1,
                        height: 1,
                        connections,
                        time_of_day: None,
                        phone_service: 0,
                        phone_flag: false,
                        environment: Some("route".into()),
                        location: None,
                        music: None,
                        palette: None,
                        fishing_group: None,
                        map_constant: None,
                        map_group_constant: None,
                        blocks_label: None,
                        map_scripts_label: None,
                        map_events_label: None,
                        connection_flags: None,
                    },
                    metadata: None,
                    scenes: MapSceneTable::default(),
                    events: MapEvents::default(),
                    objects: Vec::new(),
                    blocks: vec![index as u16],
                }
            })
            .collect()
    }

    #[test]
    fn distant_regions_are_resident_before_crossing_in_every_direction() {
        let window = VisualWorldWindow {
            origin: IVec2::splat(-96),
            size: UVec2::splat(216),
        };
        for direction in ["north", "south", "east", "west"] {
            let maps = chain(direction, 12);
            let before = placements("Region0", &maps, window).expect("resident coverage");
            assert_eq!(
                before.len(),
                maps.len(),
                "preload all short regions {direction}"
            );
            assert_eq!(before.first().expect("active map").0.map_name, "Region0");
            let (_, shift_x, shift_y) = before
                .iter()
                .find(|(map, _, _)| map.map_name == "Region1")
                .expect("adjacent region");
            let shift = IVec2::new(*shift_x, *shift_y);
            let after = placements(
                "Region1",
                &maps,
                VisualWorldWindow {
                    origin: window.origin - shift,
                    ..window
                },
            )
            .expect("same resident coverage after crossing");
            for (map, x, y) in before {
                let (_, next_x, next_y) = after
                    .iter()
                    .find(|(next, _, _)| next.map_name == map.map_name)
                    .expect("looking across a seam must not introduce a region");
                assert_eq!(IVec2::new(*next_x, *next_y) + shift, IVec2::new(x, y));
            }
        }
    }

    #[test]
    fn remote_regions_do_not_load_art_and_nearby_overlap_priority_is_preserved() {
        let maps = chain("east", 80);
        let old = connected_render_map_placements_to_depth("Region0", &maps, Some(4))
            .expect("regular neighboring context");
        let sources = placements(
            "Region0",
            &maps,
            VisualWorldWindow {
                origin: IVec2::splat(-16),
                size: UVec2::splat(64),
            },
        )
        .expect("bounded source coverage");
        assert_eq!(
            sources.len(),
            16,
            "only source acreage and context are resolved"
        );
        let tail = &sources[sources.len() - old.len() + 1..];
        assert_eq!(
            tail.iter()
                .map(|(map, _, _)| &map.map_name)
                .collect::<Vec<_>>(),
            old.iter()
                .skip(1)
                .map(|(map, _, _)| &map.map_name)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_distant_loop_cannot_fold_unrelated_scenery_into_the_resident_window() {
        let mut maps = chain("east", 7);
        for index in [4, 6] {
            maps[index].attributes.width = 100;
            maps[index].blocks = vec![index as u16; 100];
        }
        maps[5]
            .attributes
            .connections
            .last_mut()
            .expect("forward connection")
            .direction = "west".into();
        let complete = connected_render_map_placements_to_depth("Region0", &maps, None)
            .expect("complete metadata component");
        assert!(
            complete
                .iter()
                .any(|(map, x, _)| map.map_name == "Region6" && *x == 16)
        );
        let sources = placements(
            "Region0",
            &maps,
            VisualWorldWindow {
                origin: IVec2::splat(-16),
                size: UVec2::splat(64),
            },
        )
        .expect("physical resident neighborhood");
        assert!(!sources.iter().any(|(map, _, _)| map.map_name == "Region6"));
    }

    #[test]
    fn edits_beyond_four_connections_invalidate_retained_source_art() {
        let mut maps = chain("east", 12);
        let old_neighborhood =
            super::super::visible_map_block_identity_from_catalog("Region0", &maps);
        let old_coverage = block_identity("Region0", &maps);
        maps[9].blocks[0] += 1;
        assert_eq!(
            super::super::visible_map_block_identity_from_catalog("Region0", &maps),
            old_neighborhood
        );
        assert_ne!(block_identity("Region0", &maps), old_coverage);
    }

    #[test]
    #[cfg(feature = "voxel-view")]
    fn pack_regions_inside_the_preload_window_are_not_limited_by_connection_count() {
        use std::path::PathBuf;

        use crystal_assets::AssetRoot;

        use super::super::{
            BevyShellConfig, BevyShellStart, CrystalRuntime, initialize_bevy_runtime_shell,
        };

        let root = AssetRoot::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository root"),
        );
        let pack = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.repository_root
                    .join("content-packs/text-tui.crystalpack")
            });
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&pack)
            .expect("verified external pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded)
            .expect("runtime from external pack");
        let shell = initialize_bevy_runtime_shell(
            root,
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("trainer spawn"),
                map_name: "Route29".into(),
                tile_x: 5,
                tile_y: 9,
            },
            BevyShellConfig::default(),
        )
        .expect("production controller");
        let snapshot = shell.shell.snapshot().expect("read-only map catalog");
        let mut extended = Vec::new();
        for map in &snapshot.maps {
            let window = VisualWorldWindow {
                origin: IVec2::new(
                    i32::from(map.attributes.width) * 2 - 108,
                    i32::from(map.attributes.height) * 2 - 108,
                ),
                size: UVec2::splat(216),
            };
            let old =
                connected_render_map_placements_to_depth(&map.map_name, &snapshot.maps, Some(4))
                    .expect("pack connection context");
            let sources =
                placements(&map.map_name, &snapshot.maps, window).expect("pack resident coverage");
            for (source, x, y) in sources {
                if !old
                    .iter()
                    .any(|(old, _, _)| old.map_name == source.map_name)
                {
                    let origin = IVec2::new(x, y);
                    let size = IVec2::new(
                        i32::from(source.attributes.width) * 4,
                        i32::from(source.attributes.height) * 4,
                    );
                    if origin.cmplt(window.origin + window.size.as_ivec2()).all()
                        && (origin + size).cmpgt(window.origin).all()
                    {
                        extended.push((map.map_name.as_str(), source.map_name.as_str()));
                    }
                }
            }
        }
        assert!(
            !extended.is_empty(),
            "pack must exercise scenery beyond four connections"
        );
        eprintln!(
            "resident regions previously truncated: {:?}",
            &extended[..extended.len().min(8)]
        );
    }
}
