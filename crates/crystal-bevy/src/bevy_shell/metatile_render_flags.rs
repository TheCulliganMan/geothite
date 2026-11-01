//! Compile immutable collision strings once for the source tile compositor.
//!
//! Each cache belongs to one TilesetArt and its fixed metatile layout. Map
//! callbacks replace block IDs; they do not mutate the pack's collision rows.

use std::sync::OnceLock;

use anyhow::{Result, bail};
use crystal_runtime::RuntimeTilesetCatalogSnapshot;

fn is_cliff_crown_floor(tileset: &str, block: u16) -> bool {
    // This cliff's walkable crown has the same collision pattern as a facade,
    // but its upper floor must not be drawn in front of an actor walking on it.
    (tileset == "mn_outdoor_johto_stairs" && block == 0x72)
        || super::inland_winter_render::is_cleared_cliff_floor(tileset, block)
}

#[derive(Default)]
pub(super) struct MetatileRenderFlags {
    entries: OnceLock<Vec<Entry>>,
}

#[derive(Clone, Copy)]
enum Entry {
    Missing,
    InvalidLength,
    Present {
        quadrants: [bool; 4],
        foreground: bool,
    },
}

impl MetatileRenderFlags {
    pub(super) fn resolve(
        &self,
        tileset: &RuntimeTilesetCatalogSnapshot,
        metatile_count: usize,
        block: u16,
        sub_x: usize,
        sub_y: usize,
    ) -> Result<(bool, bool)> {
        let entries = self.entries.get_or_init(|| {
            (0..metatile_count)
                .map(|block| {
                    let Some(tokens) = u16::try_from(block)
                        .ok()
                        .and_then(|block| super::tileset_collision_tokens(tileset, block))
                    else {
                        return Entry::Missing;
                    };
                    if tokens.len() != 4 {
                        return Entry::InvalidLength;
                    }
                    Entry::Present {
                        quadrants: std::array::from_fn(|index| {
                            super::priority_collision_token(&tokens[index])
                        }),
                        foreground: tokens[0] == "FLOOR"
                            && tokens[1] == "FLOOR"
                            && !is_cliff_crown_floor(&tileset.tileset_id, block as u16)
                            && tokens[2..4].iter().all(|token| {
                                token == "WALL" || super::priority_collision_token(token)
                            }),
                    }
                })
                .collect()
        });
        match entries
            .get(usize::from(block))
            .copied()
            .unwrap_or(Entry::Missing)
        {
            Entry::Missing => bail!("source collision row"),
            Entry::InvalidLength => bail!("source collision row must have four quadrants"),
            Entry::Present {
                quadrants,
                foreground,
            } => {
                let quadrant = usize::from(sub_y >= 2) * 2 + usize::from(sub_x >= 2);
                Ok((
                    quadrants[quadrant] || (foreground && sub_y >= 1),
                    foreground,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
    use crystal_runtime::{CrystalRuntime, RuntimeTilesetCatalogSnapshot};

    use super::{MetatileRenderFlags, is_cliff_crown_floor};

    #[test]
    fn cached_priority_matches_every_external_pack_collision_quadrant() {
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root.repository_root
                    .join("content-packs/text-tui.crystalpack")
            });
        let loaded = read_loaded_verified_compiled_game_pack(path).expect("load external pack");
        let runtime =
            CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("construct runtime");
        let mut checked = 0;
        for (id, definition) in &runtime.data().tilesets {
            let tileset = RuntimeTilesetCatalogSnapshot {
                tileset_id: id.clone(),
                collision: definition.collision.clone(),
                palette_map: definition.palette_map.clone(),
            };
            let rows = tileset
                .collision
                .iter()
                .map(|(key, tokens)| {
                    (
                        u16::from_str_radix(key, 16).expect("canonical block ID"),
                        tokens,
                    )
                })
                .collect::<Vec<_>>();
            let count = rows
                .iter()
                .map(|(block, _)| usize::from(*block) + 1)
                .max()
                .expect("tileset collision rows");
            let flags = MetatileRenderFlags::default();
            for (block, tokens) in rows {
                assert_eq!(tokens.len(), 4);
                let foreground = tokens[0] == "FLOOR"
                    && tokens[1] == "FLOOR"
                    && !is_cliff_crown_floor(id, block)
                    && tokens[2..4].iter().all(|token| {
                        token == "WALL" || super::super::priority_collision_token(token)
                    });
                for sub_y in 0..4 {
                    for sub_x in 0..4 {
                        let quadrant = (if sub_y < 2 { 0 } else { 2 }) + usize::from(sub_x >= 2);
                        let expected = super::super::priority_collision_token(&tokens[quadrant])
                            || (foreground && sub_y >= 1);
                        assert_eq!(
                            flags
                                .resolve(&tileset, count, block, sub_x, sub_y)
                                .expect("resolve source flags"),
                            (expected, foreground),
                            "tileset {id} block {block} subtile ({sub_x}, {sub_y})"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 1_000, "exercise the production collision catalog");
    }

    #[test]
    fn cliff_crowns_do_not_turn_walkable_floor_into_a_building_eave() {
        let collision = ["FLOOR", "FLOOR", "WALL", "WALL"]
            .map(str::to_owned)
            .to_vec();
        for (id, block) in [
            ("mn_outdoor_johto_stairs", 0x72),
            ("mn_winter_iron_descent", 0x88),
            ("mn_iron_descent_thaw", 0xd0),
            ("mn_iron_descent_thaw", 0xd3),
            ("mn_iron_descent_thaw", 0xd6),
        ] {
            let tileset = RuntimeTilesetCatalogSnapshot {
                tileset_id: id.into(),
                collision: BTreeMap::from([(format!("{block:02x}"), collision.clone())]),
                palette_map: Vec::new(),
            };
            let flags = MetatileRenderFlags::default();
            for y in 0..2 {
                for x in 0..4 {
                    assert_eq!(
                        flags
                            .resolve(&tileset, block as usize + 1, block, x, y)
                            .unwrap(),
                        (false, false),
                        "cleared playable upper floor {id}:{block:02x}"
                    );
                }
            }
            // A normal native facade keeps its existing foreground behavior.
            let ordinary = RuntimeTilesetCatalogSnapshot {
                tileset_id: "johto".into(),
                ..tileset
            };
            assert_eq!(
                MetatileRenderFlags::default()
                    .resolve(&ordinary, block as usize + 1, block, 0, 1)
                    .unwrap(),
                (true, true)
            );
        }
        let fence = RuntimeTilesetCatalogSnapshot {
            tileset_id: "mn_outdoor_johto_stairs".into(),
            collision: BTreeMap::from([("49".into(), collision)]),
            palette_map: Vec::new(),
        };
        assert_eq!(
            MetatileRenderFlags::default()
                .resolve(&fence, 0x4a, 0x49, 0, 1)
                .unwrap(),
            (true, true),
            "the neighboring fence retains its foreground"
        );
    }

    #[test]
    #[ignore = "requires a Northstar review pack"]
    fn route_one_cliff_priority_preserves_native_fence_and_facade_flags() {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("Northstar review pack path"),
        );
        let root = AssetRoot::new(path.parent().expect("pack parent"));
        let loaded = read_loaded_verified_compiled_game_pack(&path).expect("load review pack");
        let runtime = CrystalRuntime::from_loaded_compiled_pack(&root, loaded).expect("runtime");
        let route = &runtime.data().maps["MnR01C00R00"];
        assert_eq!(route.attributes.tileset_name, "mn_outdoor_johto_stairs");
        assert!(
            route.blocks.contains(&0x72),
            "Route 1 must use the cliff crown"
        );
        let definition = &runtime.data().tilesets["mn_outdoor_johto_stairs"];
        let tileset = RuntimeTilesetCatalogSnapshot {
            tileset_id: "mn_outdoor_johto_stairs".into(),
            collision: definition.collision.clone(),
            palette_map: definition.palette_map.clone(),
        };
        let ordinary = RuntimeTilesetCatalogSnapshot {
            tileset_id: "johto".into(),
            ..tileset.clone()
        };
        let flags = MetatileRenderFlags::default();
        let ordinary_flags = MetatileRenderFlags::default();
        for x in 0..4 {
            assert_eq!(
                flags.resolve(&tileset, 0x100, 0x72, x, 1).expect("cliff"),
                (false, false)
            );
        }
        for block in [0x16, 0x1e, 0x49] {
            for y in 0..4 {
                for x in 0..4 {
                    assert_eq!(
                        flags
                            .resolve(&tileset, 0x100, block, x, y)
                            .expect("regional block"),
                        ordinary_flags
                            .resolve(&ordinary, 0x100, block, x, y)
                            .expect("ordinary block"),
                        "facade or fence block {block:02x} at ({x}, {y})"
                    );
                }
            }
        }
    }

    #[test]
    fn malformed_rows_keep_the_source_compositor_errors() {
        let tileset = RuntimeTilesetCatalogSnapshot {
            tileset_id: "test".to_string(),
            collision: BTreeMap::from([("01".to_string(), vec!["FLOOR".to_string()])]),
            palette_map: Vec::new(),
        };
        let flags = MetatileRenderFlags::default();
        assert_eq!(
            flags
                .resolve(&tileset, 2, 0, 0, 0)
                .expect_err("missing row")
                .to_string(),
            "source collision row"
        );
        assert_eq!(
            flags
                .resolve(&tileset, 2, 1, 0, 0)
                .expect_err("short row")
                .to_string(),
            "source collision row must have four quadrants"
        );
    }
}
