//! Compile immutable collision strings once for the source tile compositor.
//!
//! Each cache belongs to one TilesetArt and its fixed metatile layout. Map
//! callbacks replace block IDs; they do not mutate the pack's collision rows.

use std::sync::OnceLock;

use anyhow::{Result, bail};
use crystal_runtime::RuntimeTilesetCatalogSnapshot;

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

    use super::MetatileRenderFlags;

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
