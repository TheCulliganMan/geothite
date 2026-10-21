//! Actual source banks exercise atlas, orientation and viewport registration.

use std::collections::HashMap;

use bevy::prelude::{Assets, Handle, Image};
use crystal_render_api::{VisualTile, VisualTileSource};

use super::{LABEL, finish};
use crate::mesh::{
    Direction, GridGeometry, SurfaceMeshData, TerrainImageSamples, TerrainMeshData,
    TileImageSample, append_textured_shoreline, append_top,
};
use crate::profile::{CellShape, shape_for_source};

#[test]
#[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
fn external_johto_shore_finishes_native_west_and_south_banks_in_both_atlases() {
    let path = std::env::var("CRYSTAL_RENDER_TEST_PACK").expect("set the completed external pack");
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path)
        .expect("load the verified external pack");
    let pack = loaded.pack();
    for map in ["Route34", "Route40", "CherrygroveCity", "EcruteakCity"] {
        let module = &pack.data().maps[map];
        let family = module.attributes.tileset_name.as_str();
        let layout = &pack.runtime_files()[&format!("data/tilesets/{family}_metatiles.bin")];
        let sheet =
            image::load_from_memory(&pack.runtime_files()[&format!("gfx/tilesets/{family}.png")])
                .expect("read the native tilesheet")
                .to_rgba8();
        let width = module.attributes.width as usize * 4;
        let height = module.attributes.height as usize * 4;
        let mut handles = Assets::<Image>::default();
        let mut textures = HashMap::<u16, Handle<Image>>::new();
        let mut images = TerrainImageSamples::default();
        let mut tiles = Vec::with_capacity(width * height);
        let mut expected = Vec::with_capacity(width * height);
        for row in 0..height {
            for column in 0..width {
                let block = module.blocks[row / 4 * (width / 4) + column / 4];
                let tile = layout[block as usize * 16 + row % 4 * 4 + column % 4] as u16;
                let texture = textures
                    .entry(tile)
                    .or_insert_with(|| {
                        let texture = handles.add(Image::default());
                        let mut rgba = image::imageops::crop_imm(
                            &sheet,
                            u32::from(tile % 16) * 8,
                            u32::from(tile / 16) * 8,
                            8,
                            8,
                        )
                        .to_image()
                        .into_raw();
                        // Preserve the actual source ink while making its
                        // fourth water swatch brighter than either stone role.
                        // Sorting all four colors would incorrectly paint blue
                        // water onto the bank, even if its geometry passed.
                        for pixel in rgba.as_chunks_mut::<4>().0 {
                            *pixel = match pixel[0] {
                                0 => [25, 25, 25, 255],
                                85 => [165, 123, 24, 255],
                                170 => [198, 148, 57, 255],
                                255 => [170, 220, 255, 255],
                                _ => panic!("native tile must use its four source ink levels"),
                            };
                        }
                        images
                            .pixels
                            .insert(texture.id(), TileImageSample::Rgba(rgba));
                        texture
                    })
                    .clone();
                expected.push(
                    matches!(block, 0x54 | 0x55 | 0x58 | 0x59 | 0x76 | 0x79 | 0x7a) && tile != 0x14,
                );
                tiles.push(VisualTile {
                    column: column as u32,
                    row: row as u32,
                    source: VisualTileSource {
                        tileset_id: family.into(),
                        metatile_id: block,
                        subtile_column: (column % 4) as u8,
                        subtile_row: (row % 4) as u8,
                        tile_index: tile,
                    },
                    texture,
                    animation_frames: None,
                    priority: false,
                });
            }
        }
        let cells: Vec<_> = tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let count = expected.iter().filter(|&&bank| bank).count();
        assert!(count > 0, "{map} must actually exercise native bank art");
        let stone = super::palette(&cells, &shapes, family, &images)
            .expect("native bank roles provide a stone palette");
        assert!(
            stone.iter().all(|color| color[0] > color[2] * 3.),
            "{map} must not select its brighter blue water as stone paint"
        );
        assert!(
            expected
                .iter()
                .enumerate()
                .all(|(i, &bank)| !bank || shapes[i] == CellShape::ShoreBand),
            "{map} must classify its actual shore drawing before finishing its material"
        );
        if map == "Route34" || map == "Route40" {
            assert!(
                !tiles.iter().any(|c| c.source.tile_index == 0x3d
                    && shape_for_source(&c.source) == CellShape::ShoreBand),
                "the regression must work without an east-bank palette donor"
            );
        }
        let mut previous = None;
        for origin in [[0., 0.], [-24., 40.]] {
            let g = GridGeometry {
                width,
                height,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: origin[0],
                origin_z: origin[1],
            };
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; tiles.len()],
                footing_heights: shapes
                    .iter()
                    .map(|shape| shape.surface_height(8.))
                    .collect(),
                ..Default::default()
            };
            let mut water = SurfaceMeshData::default();
            for (i, &bank) in expected.iter().enumerate() {
                let (x, y) = (i % width, i / width);
                if bank {
                    append_top(&mut mesh.textured, g.bounds(x, y).into(), 0., g.uv(x, y));
                    for direction in Direction::ALL {
                        let (dx, dz) = direction.offset();
                        let (nx, ny) = (x as isize + dx, y as isize + dz);
                        if nx >= 0
                            && ny >= 0
                            && nx < width as isize
                            && ny < height as isize
                            && shapes[ny as usize * width + nx as usize] == CellShape::Water
                        {
                            append_textured_shoreline(
                                &mut mesh.textured,
                                &g,
                                x,
                                y,
                                direction,
                                g.bounds(x, y).into(),
                                CellShape::Water.surface_height(8.),
                                0.,
                            );
                        }
                    }
                } else if shapes[i] == CellShape::Water {
                    append_top(
                        &mut mesh.textured,
                        g.bounds(x, y).into(),
                        shapes[i].surface_height(8.),
                        g.uv(x, y),
                    );
                    append_top(
                        &mut water,
                        g.bounds(x, y).into(),
                        shapes[i].surface_height(8.),
                        g.uv(x, y),
                    );
                }
            }
            let footing = mesh.footing_heights.clone();
            finish(&mut mesh, map, &cells, &shapes, &g, [0, 0], &images);
            assert_eq!(
                mesh.footing_heights, footing,
                "{map} preserves all source support"
            );
            assert_eq!(
                mesh.authored_cells
                    .iter()
                    .filter(|c| **c == Some(LABEL))
                    .count(),
                count
            );
            assert!(
                mesh.authored_cells
                    .iter()
                    .enumerate()
                    .all(|(i, c)| c.is_none() || expected[i])
            );
            assert_eq!(
                mesh.textured.positions, water.positions,
                "{map} leaves only native water in the atlas"
            );
            assert_eq!(mesh.textured.uvs, water.uvs);
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| p.iter().all(|v| v.is_finite())
                        && p[1] >= CellShape::Water.surface_height(8.)
                        && p[1] <= 0.)
            );
            if let Some(uvs) = previous {
                assert_eq!(mesh.solid.uvs, uvs);
            }
            previous = Some(mesh.solid.uvs);
        }
        println!(
            "{map}: {count} source banks finished with native water, support and scrolling preserved"
        );
    }
}
