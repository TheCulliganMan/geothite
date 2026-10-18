//! Live blue/cream ceramic floors, without repainting facility furnishings.
use super::*;

pub(super) fn is_map(map: &str) -> bool {
    matches!(
        map,
        "MrPokemonsHouse" | "RuinsOfAlphResearchCenter" | "TeamRocketBaseB2F" | "TeamRocketBaseB3F" | "PowerPlant"
    )
}

pub(super) fn source(map: &str, s: &VisualTileSource) -> bool {
    if !is_map(map)
        || s.tileset_id.as_ref() != "facility"
        || s.subtile_column >= 4
        || s.subtile_row >= 4
    {
        return false;
    }
    // Only the actual checker fragments in each complete source block qualify.
    // Wall, stair, carpet, machine and cupboard pixels never become ceramic.
    let (blue, cream): (u16, u16) = match s.metatile_id {
        0x01 | 0x1d | 0x1f | 0x2d | 0x31 | 0x32 | 0x36 => (0xa500, 0x5a00),
        0x03 | 0x0c | 0x20 | 0x26 | 0x34 => (0x00a5, 0x005a),
        0x04 | 0x05 => (0xa521, 0x5a12),
        0x07 => (0xa5a5, 0x5a5a),
        0x08 | 0x28 | 0x29 => (0xa000, 0x5000),
        0x09 | 0x21 | 0x35 | 0x3c => (0x8400, 0x4800),
        0x0a | 0x15 | 0x1e | 0x3b | 0x3d => (0x2100, 0x1200),
        0x0d | 0x22 | 0x2b | 0x2f => (0x8484, 0x4848),
        0x0e | 0x10 | 0x14 | 0x27 | 0x33 | 0x37 | 0x38 => (0x2121, 0x1212),
        0x11 | 0x3a | 0x3f => (0x0021, 0x0012),
        0x23 | 0x3e => (0x0084, 0x0048),
        0x24 => (0x21a5, 0x125a),
        0x25 | 0x2a => (0x84a5, 0x485a),
        0x30 => (0xffff, 0),
        _ => return false,
    };
    let mask = match s.tile_index {
        0x01 => blue,
        0x26 => cream,
        _ => return false,
    };
    mask & (1 << (s.subtile_row * 4 + s.subtile_column)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn johto_facility_floor_matches_every_external_source_phase_and_rejects_edits() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack required");
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let bytes = &loaded.pack().runtime_files()["data/tilesets/facility_metatiles.bin"];
        for map in [
            "MrPokemonsHouse",
            "RuinsOfAlphResearchCenter",
            "TeamRocketBaseB2F",
            "TeamRocketBaseB3F",
            "PowerPlant",
        ] {
            for (block, tiles) in bytes.chunks_exact(16).enumerate() {
                for (phase, &tile) in tiles.iter().enumerate() {
                    let mut s = VisualTileSource {
                        tileset_id: "facility".into(),
                        metatile_id: block as u16,
                        subtile_column: (phase % 4) as u8,
                        subtile_row: (phase / 4) as u8,
                        tile_index: tile as u16,
                    };
                    assert_eq!(
                        source(map, &s),
                        matches!(tile, 1 | 0x26),
                        "{map} block{block:x} phase{phase}"
                    );
                    assert!(!source("UnreviewedFacilityRoom", &s));
                    s.tile_index = 0x91;
                    assert!(!source(map, &s));
                    s.tile_index = tile as u16;
                    s.subtile_row = 4;
                    assert!(!source(map, &s));
                }
            }
        }
    }

    #[test]
    fn johto_facility_floor_keeps_live_blue_cream_fields_support_and_custom_ownership() {
        for map in [
            "MrPokemonsHouse",
            "RuinsOfAlphResearchCenter",
            "TeamRocketBaseB2F",
            "TeamRocketBaseB3F",
            "PowerPlant",
        ] {
            let g = GridGeometry {
                width: 4,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let tiles = (0..16)
                .map(|i| VisualTile {
                    column: (i % 4) as u32,
                    row: (i / 4) as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "facility".into(),
                        metatile_id: 7,
                        subtile_column: (i % 4) as u8,
                        subtile_row: (i / 4) as u8,
                        tile_index: if 0xa5a5u16 & (1 << i) != 0 { 1 } else { 0x26 },
                    },
                })
                .collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let mut original = TerrainMeshData {
                footing_heights: vec![3.25; 16],
                authored_cells: vec![None; 16],
                ..Default::default()
            };
            for i in 0..16 {
                let x = (i % 4) as f32 * 8.;
                let z = (i / 4) as f32 * 8.;
                append_top(
                    &mut original.textured,
                    [x, x + 8., z, z + 8.],
                    3.25,
                    g.uv(i % 4, i / 4),
                );
            }
            let before = tiles.clone();
            let mut mesh = original.clone();
            assert_eq!(finish_surfaces(&mut mesh, map, &refs, &g, [-3, 7]), 16);
            assert_eq!(tiles, before);
            assert_eq!(mesh.footing_heights, original.footing_heights);
            assert_eq!(mesh.authored_cells, original.authored_cells);
            assert!(mesh.solid.positions.is_empty());
            for (p, uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
                assert!(p.iter().all(|v| v.is_finite()));
                assert!(
                    p[0] >= 0.
                        && p[0] <= 32.
                        && p[2] >= 0.
                        && p[2] <= 32.
                        && p[1] >= 3.202
                        && p[1] <= 3.25
                );
                // A constant live swatch retains the blue/cream checker fields
                // while removing the source's one-pixel blue/white dither.
                let x = (uv[0] * 32.).floor() as usize;
                let z = (uv[1] * 32.).floor() as usize;
                assert_eq!(z % 8, 0);
                assert!(matches!(x % 8, 0 | 1));
            }
            for triangle in mesh.textured.indices.chunks_exact(3) {
                let uv = mesh.textured.uvs[triangle[0] as usize];
                assert!(triangle
                    .iter()
                    .all(|&i| mesh.textured.uvs[i as usize] == uv));
                let center = triangle.iter().fold([0.; 2], |a, &i| {
                    let p = mesh.textured.positions[i as usize];
                    [a[0] + p[0] / 3., a[1] + p[2] / 3.]
                });
                let column = (center[0] / 8.).floor() as usize;
                let row = (center[1] / 8.).floor() as usize;
                assert_eq!(
                    (uv[0] * 4.).floor() as usize,
                    column,
                    "retain this cell's live color donor"
                );
                assert_eq!((uv[1] * 4.).floor() as usize, row);
            }
            let custom = Document {
                objects: vec![crate::live_profiles::Object {
                    name: "User checker floor".into(),
                    tileset: "facility".into(),
                    map: Some(map.into()),
                    maps: None,
                    metatile: 7,
                    metatiles: None,
                    origin: [0, 0],
                    tiles: vec![vec![1]],
                    ground: 1,
                    top_pixels: 0,
                    depth_pixels: 0.,
                    footing_pixels: None,
                    mask: Default::default(),
                    parts: vec![],
                }],
                atmosphere: None,
            };
            let mut overridden = original;
            assert_eq!(
                finish_surfaces_with_profiles(
                    &mut overridden,
                    map,
                    &refs,
                    &g,
                    [0, 0],
                    Some(&custom)
                ),
                15,
                "the custom drawing owns source phase (0,0), not all eight blue phases"
            );
            assert_eq!(&overridden.textured.indices[..6], &[0, 1, 2, 0, 2, 3]);
            assert_eq!(overridden.footing_heights, vec![3.25; 16]);
        }
    }
}

#[cfg(test)]
#[test]
fn power_plant_checker_finish_covers_real_source_floor_without_repainting_equipment() {
    use std::collections::HashSet;
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap(),
    ).unwrap();
    let pack = loaded.pack();
    let map = &pack.data().maps["PowerPlant"];
    let art = &pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
    let (w,h) = (map.attributes.width as usize*4, map.attributes.height as usize*4);
    let tiles = (0..w*h).map(|i| {
        let (x,y)=(i%w,i/w); let block=map.blocks[y/4*(w/4)+x/4];
        VisualTile { column:x as u32,row:y as u32,texture:Default::default(),
            animation_frames:None,priority:false,source:VisualTileSource {
                tileset_id:"facility".into(),metatile_id:block,subtile_column:(x%4) as u8,
                subtile_row:(y%4) as u8,tile_index:art[block as usize*16+y%4*4+x%4] as u16,
            }}
    }).collect::<Vec<_>>();
    let g=GridGeometry { width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
    let mut mesh=TerrainMeshData { footing_heights:vec![3.25;w*h],authored_cells:vec![None;w*h],..Default::default() };
    for i in 0..tiles.len() {
        append_top(&mut mesh.textured,g.bounds(i%w,i/w).into(),3.25,g.uv(i%w,i/w));
    }
    let original=mesh.textured.clone(); let before=tiles.clone();
    let floor=tiles.iter().map(|t|source("PowerPlant",&t.source)).collect::<Vec<_>>();
    assert_eq!(floor.iter().filter(|&&x|x).count(),468);
    assert_eq!(tiles.iter().filter(|t|power_plant_aggregate_floor::source("PowerPlant",&t.source)).count(),296);
    assert_eq!(finish_surfaces(&mut mesh,"PowerPlant",&tiles.iter().collect::<Vec<_>>(),&g,[0,0]),468+296);
    assert_eq!(tiles,before);
    assert_eq!(mesh.footing_heights,vec![3.25;w*h]);
    assert_eq!(mesh.authored_cells,vec![None;w*h]);
    let vertices=mesh.textured.positions.iter().zip(&mesh.textured.uvs)
        .map(|(p,u)|(p.map(f32::to_bits),u.map(f32::to_bits))).collect::<HashSet<_>>();
    for (i,is_floor) in floor.iter().enumerate() {
        if *is_floor || power_plant_aggregate_floor::source("PowerPlant",&tiles[i].source) {continue;}
        for j in i*4..i*4+4 {
            assert!(vertices.contains(&(original.positions[j].map(f32::to_bits),original.uvs[j].map(f32::to_bits))),
                "machine, conduit, wall, carpet and furniture drawing remains intact at cell{i}");
        }
    }
}
