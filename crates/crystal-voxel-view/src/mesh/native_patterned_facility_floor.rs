//! Source-authenticated patterned floors in Centers and the Battle Tower.
use super::*;

pub(super) fn is_map(map: &str) -> bool {
    map == "Pokecenter2F"
        || kanto_native_floor_map(map)
            && (map.ends_with("Pokecenter1F") || map.ends_with("Pokecenter2FBeta"))
        || matches!(map, "BattleTower1F" | "BattleTowerHallway"
            | "BattleTowerBattleRoom" | "BattleTowerElevator")
}

pub(super) fn source(map: &str, s: &VisualTileSource) -> bool {
    if !is_map(map) || s.tile_index != 0x11
        || s.subtile_column >= 4 || s.subtile_row >= 4 { return false; }
    let mask: u16 = match s.tileset_id.as_ref() {
        "pokecenter" if !map.starts_with("BattleTower") => match s.metatile_id {
            0x02 => 0xc000,
            0x03 | 0x08 | 0x09 | 0x0f | 0x16 | 0x28 | 0x32
                | 0x35 | 0x37 | 0x39 | 0x3b => 0x3000,
            0x04 => 0xffff,
            0x05 | 0x06 | 0x07 | 0x0a | 0x13 | 0x15 | 0x2b | 0x30 | 0x34 | 0x36 => 0xf000,
            0x0c => 0xfff3,
            0x0d => 0x3333,
            0x0e | 0x14 | 0x38 => 0xfccc,
            0x10 | 0x12 | 0x27 | 0x2f => 0xccff,
            0x11 | 0x2e => 0x33ff,
            0x29 => 0xffcc,
            0x2a => 0xe000,
            0x2d => 0xff00,
            _ => 0,
        },
        "battle_tower_inside" if map.starts_with("BattleTower") => match s.metatile_id {
            0x01 => 0xffff,
            0x02 | 0x03 | 0x04 | 0x07 | 0x28 | 0x29 => 0xf000,
            0x05 => 0xfccc,
            0x06 | 0x09 => 0xccff,
            0x08 => 0x33ff,
            0x14 => 0x8888,
            0x15 | 0x16 | 0x19 => 0x3333,
            0x17 => 0x37ff,
            0x18 => 0xcccc,
            0x1a => 0xceff,
            0x1c => 0xf99f,
            0x24 => 0x0fff,
            0x2a => 0x8000,
            0x2b => 0x3000,
            _ => 0,
        },
        _ => 0,
    };
    mask & (1 << (s.subtile_row * 4 + s.subtile_column)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterned_facility_floor_masks_match_every_external_source_phase() {
        let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else { return; };
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        for (map, atlas) in [("Pokecenter2F", "pokecenter"),
            ("BattleTowerHallway", "battle_tower_inside")] {
            let bytes = &pack.pack().runtime_files()[&format!("data/tilesets/{atlas}_metatiles.bin")];
            for (block, tiles) in bytes.chunks_exact(16).enumerate() {
                for (phase, &tile) in tiles.iter().enumerate() {
                    let mut s = VisualTileSource { tileset_id: atlas.into(), metatile_id: block as u16,
                        subtile_column: (phase % 4) as u8, subtile_row: (phase / 4) as u8, tile_index: 0x11 };
                    assert_eq!(source(map, &s), tile == 0x11, "{atlas} block {block:x} phase {phase}");
                    assert!(!source("UnknownFacility", &s));
                    s.tile_index = 0x91; assert!(!source(map, &s));
                    s.tile_index = 0x11; s.subtile_row = 4; assert!(!source(map, &s));
                }
            }
        }
    }

    #[test]
    fn patterned_facility_floors_retain_full_motif_support_and_cell_bounds() {
        for (map, atlas, block) in [("Pokecenter2F", "pokecenter", 4),
            ("PewterPokecenter2FBeta", "pokecenter", 4),
            ("VioletPokecenter1F", "pokecenter", 4),
            ("BattleTower1F", "battle_tower_inside", 1)] {
            let g = GridGeometry { width: 2, height: 2, tile_width: 8., tile_height: 8., origin_x: -8., origin_z: -8. };
            let tiles = (0..4).map(|i| VisualTile { column: (i % 2) as u32, row: (i / 2) as u32,
                texture: Default::default(), animation_frames: None, priority: false,
                source: VisualTileSource { tileset_id: atlas.into(), metatile_id: block,
                    subtile_column: (i % 2) as u8, subtile_row: (i / 2) as u8, tile_index: 0x11 }
            }).collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let mut mesh = TerrainMeshData { footing_heights: vec![3.25; 4], authored_cells: vec![None; 4], ..Default::default() };
            for i in 0..4 {
                let x = -8. + (i % 2) as f32 * 8.; let z = -8. + (i / 2) as f32 * 8.;
                append_top(&mut mesh.textured, [x, x + 8., z, z + 8.], 3.25, g.uv(i % 2, i / 2));
            }
            let original = mesh.clone();
            assert_eq!(finish_surfaces(&mut mesh, map, &refs, &g, [-3, 7]), 4);
            assert_eq!(mesh.footing_heights, vec![3.25; 4]); assert_eq!(mesh.authored_cells, vec![None; 4]);
            assert!(mesh.solid.positions.is_empty());
            for (p, uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
                assert!(p[0] >= -8. && p[0] <= 8. && p[2] >= -8. && p[2] <= 8.);
                assert!(p[1] >= 3.202 && p[1] <= 3.25);
                assert!((uv[0] - (p[0] + 8.) / 16.).abs() < 0.0001);
                assert!((uv[1] - (p[2] + 8.) / 16.).abs() < 0.0001);
            }
            assert!(mesh.textured.indices.chunks_exact(3).any(|t| {
                let u = [t[0],t[1],t[2]].map(|i| mesh.textured.uvs[i as usize]); u[0] != u[1] && u[1] != u[2]
            }), "{map}: retain live drawing instead of two constant swatches");
            let custom = Document { objects: vec![crate::live_profiles::Object {
                name: "User facility floor".into(), tileset: atlas.into(), map: Some(map.into()), maps: None,
                metatile: block, metatiles: None, origin: [0,0], tiles: vec![vec![0x11]], ground: 0x11,
                top_pixels: 0, depth_pixels: 0., footing_pixels: None, mask: Default::default(), parts: vec![],
            }], atmosphere: None };
            let mut overridden = original;
            assert_eq!(finish_surfaces_with_profiles(&mut overridden, map, &refs, &g, [-3,7], Some(&custom)), 3,
                "{map}: custom source floor keeps first refusal");
            assert_eq!(overridden.footing_heights, vec![3.25;4]);
            assert_eq!(overridden.authored_cells, vec![None;4]);
        }
    }
}
