//! Mountain relief inside permanent outdoor rock footprints. Walking support,
//! jump banks, planted trees and cave interiors keep their authored geometry.
use super::*;

const REACH: i32 = 6;

fn permanent(source: &VisualTileSource, shape: CellShape) -> bool {
    // The complete native entrance producer owns these two-by-two mouths.
    // Leave a real opening in the relief field above their warp footing.
    if source.metatile_id == 0x73
        && source.subtile_row >= 2
        && ((source.tileset_id.as_ref() == "johto" && source.subtile_column < 2)
            || (source.tileset_id.as_ref() == "johto_modern" && source.subtile_column >= 2))
    {
        return false;
    }
    if source.tileset_id.as_ref() == "kanto"
        && source.metatile_id == 0x06
        && source.subtile_row >= 2
        && source.subtile_column < 2
    {
        return false;
    }
    let family = match source.tileset_id.as_ref() {
        "johto" | "johto_modern" => {
            source.metatile_id == 0x0a
                || ((0x68..=0x73).contains(&source.metatile_id)
                    && !matches!(source.metatile_id, 0x6a | 0x6c))
        }
        "kanto" => crate::kanto_cliff::kanto_cliff_shape(source).is_some_and(|native| {
            native.surface_height(SOURCE_TILE_HEIGHT) >= crate::kanto_cliff::KANTO_CLIFF_HEIGHT
        }),
        _ => false,
    };
    family
        && matches!(shape, CellShape::RaisedTop { solid: SolidKind::Bank, height }
        | CellShape::LedgeBand { height, .. } if height >= crate::profile::MOUNTAIN_CLIFF_HEIGHT)
}

struct Relief {
    width: usize,
    height: usize,
    peaks: Vec<f32>,
}

impl Relief {
    fn new(
        cells: &[&VisualTile],
        shapes: &[CellShape],
        g: &GridGeometry,
        origin: [i32; 2],
    ) -> Self {
        let rock: Vec<_> = cells
            .iter()
            .zip(shapes)
            .map(|(cell, &shape)| permanent(&cell.source, shape))
            .collect();
        let mut peaks = vec![0.; (g.width + 1) * (g.height + 1)];
        for z in 0..=g.height {
            for x in 0..=g.width {
                // Distance to the nearest real opening, including paths, stairs,
                // portals and tree seats. A viewport cut is not a mountain edge.
                let mut distance = REACH as f32;
                for dz in -REACH..REACH {
                    for dx in -REACH..REACH {
                        let (cx, cz) = (x as i32 + dx, z as i32 + dz);
                        if cx < 0 || cz < 0 || cx >= g.width as i32 || cz >= g.height as i32 {
                            continue;
                        }
                        if rock[cz as usize * g.width + cx as usize] {
                            continue;
                        }
                        let px = (-(dx as f32) - 1.).max(dx as f32).max(0.);
                        let pz = (-(dz as f32) - 1.).max(dz as f32).max(0.);
                        distance = distance.min(px.hypot(pz));
                    }
                }
                // Large intersecting planar beds form unequal ridges rather than
                // a cone repeated at every metatile. World coordinates preserve
                // the formation through connected-map origins and camera crops.
                let wx = x as f32 + origin[0] as f32;
                let wz = z as f32 + origin[1] as f32;
                let bed = super::cave_cliff::slab(wx * 0.105 + 17., wz * 0.105 - 31.);
                let ridge = 2.5 + bed.abs() * 3.;
                peaks[z * (g.width + 1) + x] = distance.min(ridge) * 1.85 * g.tile_height;
            }
        }
        Self {
            width: g.width,
            height: g.height,
            peaks,
        }
    }

    fn height(&self, x: f32, z: f32) -> f32 {
        if x < 0. || z < 0. || x > self.width as f32 || z > self.height as f32 {
            return 0.;
        }
        let cx = (x.floor() as usize).min(self.width - 1);
        let cz = (z.floor() as usize).min(self.height - 1);
        let (u, v) = (x - cx as f32, z - cz as f32);
        let i = cz * (self.width + 1) + cx;
        let [a, b, c, d] = [
            self.peaks[i],
            self.peaks[i + 1],
            self.peaks[i + self.width + 1],
            self.peaks[i + self.width + 2],
        ];
        // The same diagonal on every cell gives broad planar mineral faces.
        if u + v <= 1. {
            a + (b - a) * u + (c - a) * v
        } else {
            d + (c - d) * (1. - u) + (b - d) * (1. - v)
        }
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    if !new_bark::ordinary_lawn_map(map)
        || g.width == 0
        || g.height == 0
        || !cells
            .iter()
            .zip(shapes)
            .any(|(c, &s)| permanent(&c.source, s))
    {
        return;
    }
    let relief = Relief::new(cells, shapes, g, origin);
    for (i, (position, uv)) in mesh
        .solid
        .positions
        .iter_mut()
        .zip(&mesh.solid.uvs)
        .enumerate()
    {
        if uv[0] >= -9000. {
            continue;
        }
        // Stone UVs retain the original source point through rim sculpting.
        // Use that point on both wall and cap to keep their seams joined.
        let x = (-10000. - uv[0]) / g.tile_width - origin[0] as f32;
        let tag = if uv[1] < -30000. { 40000. } else { 20000. };
        let z = (-tag - uv[1]) / g.tile_height - origin[1] as f32;
        // Folded native cliff courses can extend their source footprint past
        // the exposed wall plane. Anchor every mineral toe at the real ground
        // instead of lifting the entire wall along with the mountain cap.
        let support = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        let rise = relief.height(x, z) * (position[1] / support).clamp(0., 1.);
        position[1] += rise;
        if rise > 0.
            && let Some(color) = mesh.solid.colors.get_mut(i)
        {
            let grain =
                super::cave_cliff::slab((x + origin[0] as f32) * 1.7, (z + origin[1] as f32) * 1.7);
            let brown = [
                0.34 + grain * 0.08,
                0.215 + grain * 0.055,
                0.105 + grain * 0.03,
            ];
            let weight = (rise / g.tile_height).min(1.);
            for c in 0..3 {
                color[c] += (brown[c] - color[c]) * weight;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(tileset: &str, metatile: u16) -> (VisualWorldFrame, GridGeometry) {
        let mut source = super::super::tests::source_with_tile(metatile, 1, 1, 0x3c);
        source.tileset_id = tileset.into();
        let flat = super::super::tests::source_with_tile(1, 0, 0, 5);
        let mut sources = vec![flat; 144];
        for z in 2..10 {
            for x in 2..10 {
                sources[z * 12 + x] = source.clone();
            }
        }
        (
            super::super::tests::frame(12, 12, sources),
            GridGeometry {
                width: 12,
                height: 12,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }

    #[test]
    fn permanent_rock_grows_ridges_inside_its_footprint() {
        let (frame, g) = fixture("johto", 0x0a);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        assert!(field.height(6., 6.) > g.tile_height * 4.);
        for x in 0..=12 {
            for z in [0., 2., 10., 12.] {
                assert_eq!(
                    field.height(x as f32, z),
                    0.,
                    "mountain spreads over its source edge"
                );
            }
        }
        // Shared cap/wall samples are independent of previous rock relief.
        let mut mesh = TerrainMeshData::default();
        mesh.solid.positions = vec![
            [48., 16., 48.],
            [47.9, 16., 48.1],
            [16., 0., 48.],
            [48., 0., 48.],
        ];
        mesh.solid.uvs = vec![
            [-10048., -20048.],
            [-10048., -40048.],
            [-10016., -20048.],
            [-10048., -20048.],
        ];
        mesh.footing_heights = vec![16.; 144];
        let support = mesh.footing_heights.clone();
        finish(&mut mesh, "Route26", &cells, &shapes, &g, [0, 0]);
        assert_eq!(mesh.solid.positions[0][1], mesh.solid.positions[1][1]);
        assert_eq!(mesh.solid.positions[2], [16., 0., 48.]);
        assert_eq!(
            mesh.solid.positions[3],
            [48., 0., 48.],
            "the mountain toe must stay grounded"
        );
        assert_eq!(mesh.footing_heights, support);
    }

    #[test]
    fn caves_jump_ledges_and_tree_seats_are_unchanged() {
        let mut jump = super::super::tests::source_with_tile(0x07, 1, 1, 0x2c);
        jump.tileset_id = "kanto".into();
        assert!(
            !permanent(
                &jump,
                CellShape::RaisedTop {
                    height: 48.,
                    solid: SolidKind::Bank
                }
            ),
            "a resolved upper terrace cannot turn a jump bank into a mountain"
        );
        for (tileset, block, map) in [
            ("johto", 0x0a, "DragonsDenB1F"),
            ("cave", 0x0a, "MountMoon"),
            ("dark_cave", 0x0a, "DarkCaveVioletEntrance"),
            ("johto", 0x4b, "Route26"),
            ("johto", 0x6a, "Route26"),
            ("johto_modern", 0x6c, "Route33"),
            ("kanto", 0x07, "Route1"),
        ] {
            let (frame, g) = fixture(tileset, block);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let mut mesh = TerrainMeshData::default();
            mesh.solid.positions = vec![[48., 16., 48.]];
            mesh.solid.uvs = vec![[-10048., -20048.]];
            let before = mesh.clone();
            finish(&mut mesh, map, &cells, &shapes, &g, [0, 0]);
            assert_eq!(mesh, before, "{tileset}/{block:02x} in {map}");
        }
    }

    #[test]
    fn mountain_openings_and_world_origin_survive_relief() {
        let (mut frame, g) = fixture("johto", 0x0a);
        for z in 5..7 {
            for x in 5..7 {
                frame.tiles[z * 12 + x].source =
                    super::super::tests::source_with_tile(0x73, (x - 5) as u8, (z - 3) as u8, 0x46);
            }
        }
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        assert_eq!(field.height(5.5, 5.5), 0., "cave entrance must stay open");
        // A cropped viewport with a translated origin samples the same mass.
        let original = &cells;
        let cropped: Vec<_> = (1..12)
            .flat_map(|z| (1..12).map(move |x| original[z * 12 + x]))
            .collect();
        let cropped_shapes: Vec<_> = cropped
            .iter()
            .map(|c| shape_for_source(&c.source))
            .collect();
        let cg = GridGeometry {
            width: 11,
            height: 11,
            ..g
        };
        let shifted = Relief::new(&cropped, &cropped_shapes, &cg, [1, 1]);
        for (x, z) in [(3.5, 3.5), (7.5, 7.5), (8.25, 4.5)] {
            assert!((field.height(x, z) - shifted.height(x - 1., z - 1.)).abs() < 0.0001);
        }
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_route26_has_mountains_and_preserves_every_open_tile() {
        let path = std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let module = &pack.data().maps["Route26"];
        let tileset = &module.attributes.tileset_name;
        let layout = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
        let width = module.attributes.width as usize * 4;
        let height = module.attributes.height as usize * 4;
        let sources = (0..height)
            .flat_map(|z| {
                (0..width).map(move |x| {
                    let block = module.blocks[z / 4 * (width / 4) + x / 4];
                    VisualTileSource {
                        tileset_id: tileset.as_str().into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (z % 4) as u8,
                        tile_index: layout[block as usize * 16 + z % 4 * 4 + x % 4] as u16,
                    }
                })
            })
            .collect();
        let frame = super::super::tests::frame(width as u32, height as u32, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width,
            height,
            tile_width: 16.,
            tile_height: 16.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let field = Relief::new(&cells, &shapes, &g, [0, 0]);
        let mut peaks = 0;
        for (i, cell) in cells.iter().enumerate() {
            let x = (i % width) as f32;
            let z = (i / width) as f32;
            if permanent(&cell.source, shapes[i]) {
                peaks += usize::from(field.height(x + 0.5, z + 0.5) > g.tile_height * 2.);
            } else {
                for (dx, dz) in [(0., 0.), (0.5, 0.5), (1., 1.)] {
                    assert_eq!(
                        field.height(x + dx, z + dz),
                        0.,
                        "relief enters open tile {x},{z}"
                    );
                }
            }
        }
        assert!(
            peaks > 100,
            "Route26 should contain substantial mountain relief: {peaks}"
        );
    }
}
