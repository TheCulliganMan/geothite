//! The facility's complete paired shelves: cupboard left, books right.
use super::*;

const DRAWING: [[u16; 4]; 4] = [
    [0x40, 0x42, 0x40, 0x42],
    [0x28, 0x29, 0x28, 0x29],
    [0x28, 0x29, 0x28, 0x29],
    [0x2a, 0x2b, 0x28, 0x29],
];
pub(super) fn source(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if !matches!(
        map,
        "MrPokemonsHouse" | "PowerPlant" | "RuinsOfAlphResearchCenter" | "TeamRocketBaseB3F"
    ) || p.kind != ModelKind::Bookcase
        || p.height != 4
        || !matches!(p.width, 2 | 4)
        || cells.len() != g.width * g.height
        || p.column.checked_add(p.width).is_none_or(|x| x > g.width)
        || p.row.checked_add(4).is_none_or(|y| y > g.height)
    {
        return false;
    }
    let start = cells[p.row * g.width + p.column].source.subtile_column as usize;
    if start % 2 != 0 || start + p.width > 4 {
        return false;
    }
    p.indices(g.width).enumerate().all(|(j, i)| {
        let s = &cells[i].source;
        let x = start + j % p.width;
        let y = j / p.width;
        s.tileset_id.as_ref() == "facility"
            && s.metatile_id == 6
            && s.subtile_column as usize == x
            && s.subtile_row as usize == y
            && s.tile_index == DRAWING[y][x]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn map_fixture(
        map: &str,
        loaded: &crystal_assets::LoadedCompiledGamePack,
    ) -> (Vec<VisualTile>, GridGeometry) {
        let pack = loaded.pack();
        let data = &pack.data().maps[map];
        let art = &pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
        let (w, h) = (
            data.attributes.width as usize * 4,
            data.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let b = data.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "facility".into(),
                        metatile_id: b,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect();
        (
            tiles,
            GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }
    #[test]
    fn external_facility_bookcases_keep_source_wood_left_cupboards_right_books_and_support() {
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()),
        )
        .unwrap();
        let mut total = 0;
        let mut cupboards = 0;
        let mut open = 0;
        for (map, expected) in [
            ("MrPokemonsHouse", 16),
            ("PowerPlant", 16),
            ("RuinsOfAlphResearchCenter", 16),
            ("TeamRocketBaseB3F", 64),
        ] {
            let (tiles, g) = map_fixture(map, &loaded);
            let original = tiles.clone();
            let cells = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(map, &cells, &g, None)
                .into_iter()
                .filter(|p| p.kind == ModelKind::Bookcase)
                .collect::<Vec<_>>();
            assert_eq!(
                placements.iter().map(|p| p.width * p.height).sum::<usize>(),
                expected
            );
            let mut mesh = TerrainMeshData {
                footing_heights: vec![1.25; tiles.len()],
                authored_cells: vec![None; tiles.len()],
                ..Default::default()
            };
            let mut claimed = vec![false; tiles.len()];
            for p in placements {
                assert!(source(map, &p, &cells, &g));
                assert_eq!(
                    native_bookcase_source(map, &p, &cells, &g),
                    Some([[3, 10], [3, 12]])
                );
                for unit in 0..p.width / 2 {
                    let i = (p.row + 3) * g.width + p.column + unit * 2;
                    let closed = native_bookcase_closed_base(&p, &cells, &g, unit);
                    assert_eq!(closed, cells[i].source.subtile_column == 0);
                    if closed {
                        cupboards += 1;
                        assert_eq!(
                            [cells[i].source.tile_index, cells[i + 1].source.tile_index],
                            [0x2a, 0x2b]
                        );
                    } else {
                        open += 1;
                        assert_eq!(
                            [cells[i].source.tile_index, cells[i + 1].source.tile_index],
                            [0x28, 0x29]
                        );
                    }
                }
                assert!(append(&mut mesh, map, &cells, &g, &p, &mut claimed, [0, 0]));
            }
            assert_eq!(claimed.iter().filter(|&&v| v).count(), expected);
            total += expected;
            assert!(
                mesh.solid.indices.is_empty(),
                "source material ownership covers cabinet and book surfaces"
            );
            assert!(!mesh.textured.indices.is_empty());
            assert_eq!(mesh.footing_heights, vec![1.25; tiles.len()]);
            assert_eq!(tiles, original);
            for tri in mesh.textured.indices.chunks_exact(3) {
                let p = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                assert!(p.iter().all(|p| p.is_finite()));
            }
        }
        assert_eq!((total, cupboards, open), (112, 7, 7));
    }
    #[test]
    fn external_facility_bookcase_native_finish_rejects_changed_partial_and_other_room_drawings() {
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()),
        )
        .unwrap();
        let (mut tiles, g) = map_fixture("PowerPlant", &loaded);
        let p = resolve("PowerPlant", &tiles.iter().collect::<Vec<_>>(), &g, None)
            .into_iter()
            .find(|p| p.kind == ModelKind::Bookcase)
            .unwrap();
        for i in p.indices(g.width) {
            tiles[i].source.tile_index ^= 1;
            assert!(!source(
                "PowerPlant",
                &p,
                &tiles.iter().collect::<Vec<_>>(),
                &g
            ));
            tiles[i].source.tile_index ^= 1;
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        assert!(!source("UnreviewedFacilityRoom", &p, &refs, &g));
        let mut clipped = p.clone();
        clipped.height = 3;
        assert!(!source("PowerPlant", &clipped, &refs, &g));
        let mut invalid = p.clone();
        invalid.column = g.width;
        assert!(!source("PowerPlant", &invalid, &refs, &g));
    }
}
