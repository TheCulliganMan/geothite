//! Preserve exact native building provenance in an appended authored art bank.
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, ensure};
use crystal_assets::AssetRoot;
use crystal_render_api::VisualTileSource;

/// Falls copies all 128 JohtoModern blocks before appending its snow blocks.
/// Their logical glyph addresses and referenced palettes remain unchanged, so
/// these cells are still original Modern building/paving sources. Imported
/// snow retains its own identity and cannot acquire a Modern building profile.
pub(super) fn tileset_for_block(tileset: &str, block: u16) -> &str {
    match (tileset, block) {
        ("mn_international_falls", 0..=127) => "johto_modern",
        ("mn_ely_snow", _) => "kanto",
        _ => tileset,
    }
}

/// Winter encounter blades and their snow backing share the native grass
/// geometry. Their colored atlas pixels remain owned by the original tileset.
fn winter_grass_source(tileset: &str, block: u16) -> Option<(&'static str, u16, u8)> {
    let snow = match tileset {
        "mn_winter_outdoor" | "mn_winter_gunflint" => 0x3c,
        "mn_winter_north_shore" => 0x83,
        _ => return None,
    };
    if block == snow {
        Some(("johto", 1, 5))
    } else if block == snow + 1 {
        Some(("johto", 3, 4))
    } else {
        None
    }
}

#[derive(Default)]
pub(super) struct Projection<'a> {
    names: HashMap<&'a str, Arc<str>>,
    forest: Option<[u8; 16]>,
    kanto_bridge_posts: Option<[bool; 2]>,
}

impl<'a> Projection<'a> {
    /// Keep the real colored texture while identifying its original geometry.
    /// Native snow canopy is a color remap of the exact forest05 drawing.
    /// Read that donor's glyph identities from the supplied runtime pack once
    /// per terrain rebuild; no copied pixel or glyph catalog is embedded here.
    pub(super) fn source(
        &mut self,
        root: &AssetRoot,
        tileset: &'a str,
        block: u16,
        column: usize,
        row: usize,
        glyph: u8,
    ) -> Result<VisualTileSource> {
        ensure!(
            column < 4 && row < 4,
            "authored source must address one native metatile"
        );
        let block = if tileset == "kanto" && matches!(block, 0x80 | 0x82) {
            if self.kanto_bridge_posts.is_none() {
                let bytes = crate::read_runtime_asset(
                    root.runtime_assets()
                        .join("data/tilesets/kanto_metatiles.bin"),
                )?;
                self.kanto_bridge_posts = Some([
                    same_drawing(&bytes, 0x80, 0x1b),
                    same_drawing(&bytes, 0x82, 0x5f),
                ]);
            }
            let index = usize::from(block == 0x82);
            if self
                .kanto_bridge_posts
                .context("loaded Kanto bridge drawings")?[index]
            {
                [0x1b, 0x5f][index]
            } else {
                block
            }
        } else {
            block
        };
        let canopy = matches!(
            (tileset, block),
            ("mn_international_falls", 0x81)
                | ("mn_winter_outdoor" | "mn_winter_gunflint", 0x3f)
                | ("mn_ely_snow", 0x83)
        );
        let (name, block, glyph) = if let Some(source) = winter_grass_source(tileset, block) {
            source
        } else if canopy {
            if self.forest.is_none() {
                let bytes = crate::read_runtime_asset(
                    root.runtime_assets()
                        .join("data/tilesets/forest_metatiles.bin"),
                )?;
                self.forest = Some(
                    bytes
                        .get(5 * 16..6 * 16)
                        .context("native forest05 donor")?
                        .try_into()?,
                );
            }
            let donor = self.forest.as_ref().context("loaded forest05 donor")?;
            ("forest", 5, donor[row * 4 + column])
        } else {
            (tileset_for_block(tileset, block), block, glyph)
        };
        let name = self
            .names
            .entry(name)
            .or_insert_with(|| Arc::from(name))
            .clone();
        Ok(VisualTileSource {
            tileset_id: name,
            metatile_id: block,
            subtile_column: column as u8,
            subtile_row: row as u8,
            tile_index: u16::from(glyph),
        })
    }
}

fn same_drawing(bytes: &[u8], authored: usize, native: usize) -> bool {
    bytes
        .get(authored * 16..(authored + 1) * 16)
        .zip(bytes.get(native * 16..(native + 1) * 16))
        .is_some_and(|(authored, native)| authored == native)
}

#[cfg(all(test, feature = "voxel-view"))]
mod tests {
    use std::path::PathBuf;

    use bevy::prelude::Handle;
    use crystal_assets::{AssetRoot, read_verified_compiled_game_pack};
    use crystal_render_api::VisualTile;
    use crystal_voxel_view::{CellCoverageKind, audit_cell_coverage_on_map};

    use super::{Projection, same_drawing, tileset_for_block, winter_grass_source};

    #[test]
    fn winter_grass_and_snow_use_the_native_grounded_grass_pair() {
        for (tileset, snow) in [
            ("mn_winter_outdoor", 0x3c),
            ("mn_winter_gunflint", 0x3c),
            ("mn_winter_north_shore", 0x83),
        ] {
            assert_eq!(winter_grass_source(tileset, snow), Some(("johto", 1, 5)));
            assert_eq!(
                winter_grass_source(tileset, snow + 1),
                Some(("johto", 3, 4))
            );
            let root = AssetRoot::new(std::env::temp_dir());
            let mut projection = Projection::default();
            let mut tiles = Vec::new();
            for row in 0..4 {
                for column in 0..8 {
                    let block = if column < 4 { snow } else { snow + 1 };
                    tiles.push(VisualTile {
                        column: column as u32,
                        row: row as u32,
                        source: projection
                            .source(&root, tileset, block, column % 4, row, 224)
                            .expect("winter source projection"),
                        texture: Handle::default(),
                        animation_frames: None,
                        priority: false,
                    });
                }
            }
            let coverage = audit_cell_coverage_on_map("MN_R12", &tiles, 8, 4)
                .expect("grounded winter grass coverage");
            for row in 0..4 {
                assert!(
                    coverage[row * 8..row * 8 + 4]
                        .iter()
                        .all(|kind| *kind == CellCoverageKind::Flat)
                );
                assert!(
                    coverage[row * 8 + 4..row * 8 + 8]
                        .iter()
                        .all(|kind| *kind == CellCoverageKind::Cutout)
                );
            }
            assert_eq!(winter_grass_source(tileset, snow + 2), None);
            assert_eq!(winter_grass_source(tileset, 3), None);
        }
        assert_eq!(winter_grass_source("johto", 0x3d), None);
    }

    #[test]
    fn custom_bridge_art_must_match_the_complete_native_drawing() {
        let mut bytes = vec![0; 0x83 * 16];
        assert!(same_drawing(&bytes, 0x80, 0x1b));
        bytes[0x80 * 16 + 15] = 1;
        assert!(!same_drawing(&bytes, 0x80, 0x1b));
        assert!(!same_drawing(&bytes[..0x80 * 16 + 15], 0x80, 0x1b));
    }

    #[test]
    #[ignore = "requires CRYSTAL_INTERIOR_PACK containing International Falls"]
    fn speculative_falls_scene_keeps_native_profiles_without_advancing_gameplay() {
        use super::super::{
            BevyShellConfig, BevyShellStart, RenderedTilesetArt, TilePosition, destination_preload,
            initialize_bevy_runtime_shell,
        };
        use bevy::prelude::{Assets, Image};
        use crystal_assets::read_loaded_verified_compiled_game_pack;
        use crystal_runtime::CrystalRuntime;

        let path = PathBuf::from(std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("Falls pack"));
        let root = AssetRoot::new(path.parent().expect("pack directory"));
        let runtime = CrystalRuntime::from_loaded_compiled_pack(
            &root,
            read_loaded_verified_compiled_game_pack(&path).expect("verified pack"),
        )
        .expect("runtime");
        let shell = initialize_bevy_runtime_shell(
            root,
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime
                    .title_new_game_spawn_identifier()
                    .expect("HOME preserved"),
                map_name: "MnInternationalFalls".into(),
                tile_x: 9,
                tile_y: 8,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                render_test_hour: Some(12),
                ..Default::default()
            },
        )
        .expect("source fixture");
        let snapshot = shell.shell.snapshot().expect("authoritative source");
        let before = snapshot.state_checksum.clone();
        let map = snapshot
            .maps
            .iter()
            .find(|m| m.map_name == "MnInternationalFalls")
            .expect("town");
        let frame = destination_preload::source_frame(
            &snapshot,
            &shell,
            map,
            TilePosition::new(9, 8),
            "day",
            &mut RenderedTilesetArt::default(),
            &mut Assets::<Image>::default(),
        )
        .expect("production preloaded scene");
        frame.validate().expect("complete native scene");
        assert!(frame.tiles.iter().any(
            |t| t.source.tileset_id.as_ref() == "johto_modern" && t.source.metatile_id == 0x18
        ));
        assert!(
            frame
                .tiles
                .iter()
                .any(|t| t.source.tileset_id.as_ref() == "forest" && t.source.metatile_id == 5)
        );
        assert!(
            !frame
                .tiles
                .iter()
                .any(|t| t.source.tileset_id.as_ref() == "mn_international_falls"
                    && t.source.metatile_id == 0x81),
            "streamed terrain must not replace modeled canopy with a flat fallback"
        );
        let city = snapshot
            .maps
            .iter()
            .find(|map| map.map_name == "MnStPaul")
            .expect("city metadata");
        assert!(
            destination_preload::source_frame(
                &snapshot,
                &shell,
                city,
                TilePosition::new(26, 52),
                "day",
                &mut RenderedTilesetArt::default(),
                &mut Assets::<Image>::default(),
            )
            .is_err(),
            "bridges must retain their live three-layer source preparation"
        );
        let kanto = crate::read_runtime_asset(
            shell
                .asset_root
                .runtime_assets()
                .join("data/tilesets/kanto_metatiles.bin"),
        )
        .expect("real native bridge art");
        let mut projection = Projection::default();
        for (authored, native) in [(0x80, 0x1b), (0x82, 0x5f)] {
            assert!(same_drawing(&kanto, authored, native));
            for row in 0..4 {
                for column in 0..4 {
                    let glyph = kanto[authored * 16 + row * 4 + column];
                    let projected = projection
                        .source(
                            &shell.asset_root,
                            "kanto",
                            authored as u16,
                            column,
                            row,
                            glyph,
                        )
                        .expect("live bridge post provenance");
                    assert_eq!(projected.metatile_id, native as u16);
                    assert_eq!(projected.tile_index, u16::from(glyph));
                }
            }
        }
        assert_eq!(
            shell
                .shell
                .snapshot()
                .expect("after presentation prep")
                .state_checksum,
            before
        );
    }

    #[test]
    #[ignore = "requires CRYSTAL_INTERIOR_PACK containing International Falls"]
    fn appended_winter_bank_keeps_both_complete_native_buildings_modeled() {
        let pack = read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_INTERIOR_PACK").expect("Falls pack"),
        ))
        .expect("verified native pack");
        let map = &pack.data().maps["MnInternationalFalls"];
        let files = pack.runtime_files();
        let native = &files["data/tilesets/johto_modern_metatiles.bin"];
        let alias = &files["data/tilesets/mn_international_falls_metatiles.bin"];
        assert_eq!(native.len(), 128 * 16);
        assert_eq!(
            &alias[..native.len()],
            native,
            "the original building bank must remain exact"
        );
        let original = &pack.data().tilesets["johto_modern"].palette_map;
        let expanded = &pack.data().tilesets["mn_international_falls"].palette_map;
        let original_pixels = &files["gfx/tilesets/johto_modern.2bpp"];
        let expanded_pixels = &files["gfx/tilesets/mn_international_falls.2bpp"];
        for glyph in native
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
        {
            let palette = original[usize::from(glyph)];
            assert_eq!(
                palette,
                expanded[usize::from(glyph)],
                "referenced native palette must stay exact"
            );
            let slot = |count: usize| {
                if palette & 8 == 0 {
                    usize::from(glyph)
                } else {
                    usize::from(glyph & 127) + count / 2
                }
            };
            let a = slot(original_pixels.len() / 16) * 16;
            let b = slot(expanded_pixels.len() / 16) * 16;
            assert_eq!(
                &original_pixels[a..a + 16],
                &expanded_pixels[b..b + 16],
                "referenced native indexed pixels must stay exact"
            );
        }
        let (width, height) = (
            usize::from(map.attributes.width) * 4,
            usize::from(map.attributes.height) * 4,
        );
        let forest = &files["data/tilesets/forest_metatiles.bin"][5 * 16..6 * 16];
        assert_eq!(
            forest,
            &files["data/tilesets/mn_border_pines_metatiles.bin"][5 * 16..6 * 16],
            "snow donor retains native forest05 geometry"
        );
        let mut projection = Projection {
            forest: Some(forest.try_into().expect("native donor")),
            ..Default::default()
        };
        let root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
        let mut tiles = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let block = map.blocks[y / 4 * usize::from(map.attributes.width) + x / 4];
                let glyph = alias[usize::from(block) * 16 + y % 4 * 4 + x % 4];
                tiles.push(VisualTile {
                    column: x as u32,
                    row: y as u32,
                    source: projection
                        .source(
                            &root,
                            &map.attributes.tileset_name,
                            block,
                            x % 4,
                            y % 4,
                            glyph,
                        )
                        .expect("authenticated visual source"),
                    texture: Handle::default(),
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let coverage = audit_cell_coverage_on_map(&map.id, &tiles, width, height)
            .expect("native modeled coverage");
        for (x, y) in [(4 * 4, 2 * 4), (11 * 4, 3 * 4)] {
            assert!(
                (y..y + 8).all(|row| (x..x + 8)
                    .all(|column| coverage[row * width + column] == CellCoverageKind::Building)),
                "the complete clinic and Mart must be modeled, including roof and facade"
            );
        }
        assert_eq!(
            tileset_for_block("mn_international_falls", 128),
            "mn_international_falls"
        );
        let canopy = &tiles[0];
        assert_eq!(
            (canopy.source.tileset_id.as_ref(), canopy.source.metatile_id),
            ("forest", 5)
        );
        assert_eq!(
            coverage[0],
            CellCoverageKind::Tree,
            "snow canopy must stand as a native tree, not lie on the floor"
        );
        // The previous alias-only identities made both complete source drawings
        // invisible to the native building catalog. Retain that failing control.
        for tile in &mut tiles {
            tile.source.tileset_id = "mn_international_falls".into();
        }
        let rejected = audit_cell_coverage_on_map(&map.id, &tiles, width, height)
            .expect("unrecognized-bank control");
        assert!((8..16).all(|row| {
            (16..24).all(|column| rejected[row * width + column] != CellCoverageKind::Building)
        }));
    }
}
