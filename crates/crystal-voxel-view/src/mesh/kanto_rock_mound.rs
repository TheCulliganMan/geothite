//! The complete isolated Kanto rock mound, including scripted road closures.
use super::*;

const LABEL: &str = "world-exterior/kanto-rock-mound";
const DRAWING: [u16; 16] = [
    0x1e, 1, 1, 2, 0x27, 0x11, 0x11, 0x24, 0x27, 0x11, 0x11, 0x24, 0x36, 0x37, 0x37, 0x34,
];

fn matches(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    x: usize,
    y: usize,
) -> bool {
    x + 4 <= g.width
        && y + 4 <= g.height
        && (0..16).all(|i| {
            let at = (y + i / 4) * g.width + x + i % 4;
            let s = &cells[at].source;
            s.tileset_id.as_ref() == "kanto"
                && s.metatile_id == 0x7a
                && s.subtile_column as usize == i % 4
                && s.subtile_row as usize == i / 4
                && s.tile_index == DRAWING[i]
                && matches!(
                    shapes[at],
                    CellShape::Flat | CellShape::PlaneAt { height: 0. }
                )
        })
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    origin: [i32; 2],
    images: Option<&TerrainImageSamples>,
) {
    let Some(images) = images else {
        return;
    };
    // Prefer the native plain road, including when no adjacent-town halo exists.
    let ground = [(0x31, 0x39), (1, 0x23)]
        .into_iter()
        .find_map(|(block, art)| {
            cells.iter().zip(shapes).position(|(tile, shape)| {
                tile.source.tileset_id.as_ref() == "kanto"
                    && tile.source.metatile_id == block
                    && tile.source.tile_index == art
                    && matches!(shape, CellShape::Flat | CellShape::PlaneAt { height: 0. })
            })
        });
    let Some(ground) = ground else {
        return;
    };
    for y in 0..g.height.saturating_sub(3) {
        for x in 0..g.width.saturating_sub(3) {
            if !matches(cells, shapes, g, x, y) || !clear(claimed, g, [x, y, 4, 4]) {
                continue;
            }
            let tiles = (0..16)
                .map(|i| cells[(y + i / 4) * g.width + x + i % 4])
                .collect::<Vec<_>>();
            if !tiles.iter().all(|tile| {
                tile_rgba(images, tile)
                    .is_ok_and(|p| p.len() == 256 && p.chunks_exact(4).all(|p| p[3] == 255))
            }) {
                continue;
            }
            let Some(paint) = fence_palette(images, &tiles) else {
                continue;
            };
            floor(mesh, cells, shapes, g, [x, y, 4, 4], ground, claimed);
            let (w, _, n, _) = g.bounds(x, y);
            // One shared pile recipe, built only from the existing closed boulder.
            // Its source plot and authoritative support remain unchanged.
            for (j, (cx, cz, rise, radius)) in [
                (1.15, 1.20, 1.10, 1.05),
                (2.75, 1.45, 1.35, 1.05),
                (2.0, 2.75, 0.95, 1.08),
            ]
            .into_iter()
            .enumerate()
            {
                let m = variant_model(
                    Kind::ShoreBoulder,
                    [
                        origin[0] + x as i32 + j as i32 * 2,
                        origin[1] + y as i32 + j as i32 * 3,
                    ],
                );
                let first = mesh.solid.positions.len();
                m.append_fitted(
                    &mut mesh.solid,
                    [
                        w + (cx - radius) * g.tile_width,
                        w + (cx + radius) * g.tile_width,
                        n + (cz - radius) * g.tile_height,
                        n + (cz + radius) * g.tile_height,
                    ],
                    0.,
                    rise * g.tile_height / (m.max[1] - m.min[1]),
                    None,
                );
                for i in first..mesh.solid.positions.len() {
                    let at = m.surface.positions[i - first];
                    let normal = m.surface.normals[i - first];
                    let (a, b, t) = match m.vertex_material(i - first) {
                        "stone_light" => (paint[1], paint[2], 0.20),
                        "stone_dark" => (paint[0], paint[1], 0.75),
                        _ => (paint[1], paint[1], 0.),
                    };
                    let color: [f32; 3] = std::array::from_fn(|c| a[c] * (1. - t) + b[c] * t);
                    let luma = color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722;
                    let gain = 0.72 + 0.22 * mesh.solid.normals[i][1].max(0.);
                    for c in 0..3 {
                        mesh.solid.colors[i][c] = (color[c] * 0.45 + luma * 0.55) * gain;
                    }
                    let across = if normal[0].abs() > normal[2].abs() {
                        at[2] - m.min[2]
                    } else {
                        at[0] - m.min[0]
                    };
                    mesh.solid.uvs[i] = [
                        across,
                        crate::scenery_finish::TOWN_PAINT_BASE
                            + 96.
                            + if normal[1].abs() > 0.5 {
                                at[2] - m.min[2]
                            } else {
                                at[1] - m.min[1]
                            },
                    ];
                }
            }
            mark_authored_rect(mesh, g, [x, y, 4, 4], LABEL);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        Vec<VisualTile>,
        Vec<CellShape>,
        GridGeometry,
        TerrainImageSamples,
    ) {
        let mut assets = Assets::<Image>::default();
        let texture = assets.add(Image::default());
        let g = GridGeometry {
            width: 8,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let tiles = (0..32)
            .map(|i| {
                let (x, y) = (i % 8, i / 8);
                let rock = x < 4;
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: texture.clone(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "kanto".into(),
                        metatile_id: if rock { 0x7a } else { 1 },
                        subtile_column: (x % 4) as u8,
                        subtile_row: y as u8,
                        tile_index: if rock { DRAWING[y * 4 + x] } else { 0x23 },
                    },
                }
            })
            .collect();
        let mut images = TerrainImageSamples::default();
        images.pixels.insert(
            texture.id(),
            TileImageSample::Rgba(
                [
                    [25, 25, 25, 255],
                    [85, 65, 40, 255],
                    [180, 135, 70, 255],
                    [240, 235, 210, 255],
                ]
                .repeat(16)
                .into_iter()
                .flatten()
                .collect(),
            ),
        );
        (tiles, vec![CellShape::Flat; 32], g, images)
    }
    fn render(
        tiles: &[VisualTile],
        shapes: &[CellShape],
        g: &GridGeometry,
        images: Option<&TerrainImageSamples>,
    ) -> (TerrainMeshData, Vec<bool>) {
        let mut mesh = TerrainMeshData {
            footing_heights: vec![2.75; tiles.len()],
            authored_cells: vec![None; tiles.len()],
            ..Default::default()
        };
        let mut claimed = vec![false; tiles.len()];
        append(
            &mut mesh,
            &tiles.iter().collect::<Vec<_>>(),
            shapes,
            g,
            &mut claimed,
            [0, 0],
            images,
        );
        (mesh, claimed)
    }
    #[test]
    fn closed_shared_pile_replaces_whole_drawing_and_preserves_support_and_live_paint() {
        let (tiles, shapes, g, images) = fixture();
        let (day, claims) = render(&tiles, &shapes, &g, Some(&images));
        assert_eq!(claims.iter().filter(|&&v| v).count(), 16);
        assert_eq!(day.footing_heights, vec![2.75; 32]);
        let mut isolated = tiles.clone();
        for tile in &mut isolated {
            if tile.column >= 4 {
                tile.source.metatile_id = 0x31;
                tile.source.tile_index = 0x39;
            }
        }
        let (road, road_claims) = render(&isolated, &shapes, &g, Some(&images));
        assert_eq!(road_claims, claims);
        assert_eq!(road.solid, day.solid);
        assert_eq!(road.footing_heights, day.footing_heights);
        let mut repeated = day.clone();
        let mut used = claims.clone();
        append(
            &mut repeated,
            &tiles.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &mut used,
            [0, 0],
            Some(&images),
        );
        assert_eq!(repeated, day);
        assert!(day.solid.positions.iter().all(|p| p[0] >= 0.
            && p[0] <= 32.
            && p[2] >= 0.
            && p[2] <= 32.
            && p[1] >= 0.
            && p[1] <= 10.81));
        assert!(day.solid.positions.iter().any(|p| p[1] > 7.));
        assert!(day.solid.indices.len() / 3 < 4000);
        assert!(
            day.solid
                .uvs
                .iter()
                .all(|&uv| crate::scenery_finish::town_paint_kind(uv) == Some(3))
        );
        let mut night = images.clone();
        for sample in night.pixels.values_mut() {
            if let TileImageSample::Rgba(p) = sample {
                for px in p.chunks_exact_mut(4) {
                    px[0] /= 2;
                    px[1] /= 2;
                }
            }
        }
        let (dark, dark_claims) = render(&tiles, &shapes, &g, Some(&night));
        assert_eq!(dark_claims, claims);
        assert_eq!(dark.solid.positions, day.solid.positions);
        assert_eq!(dark.solid.indices, day.solid.indices);
        assert_eq!(dark.solid.uvs, day.solid.uvs);
        assert_ne!(dark.solid.colors, day.solid.colors);
    }
    #[test]
    fn cleared_altered_cropped_foreign_raised_or_unpainted_art_keeps_native_fallback() {
        let (tiles, shapes, g, images) = fixture();
        for i in 0..16 {
            for fault in 0..4 {
                let mut bad = tiles.clone();
                let at = i / 4 * 8 + i % 4;
                match fault {
                    0 => bad[at].source.tile_index ^= 1,
                    1 => bad[at].source.metatile_id = 0x1a,
                    2 => bad[at].source.tileset_id = "johto".into(),
                    _ => bad[at].source.subtile_row ^= 1,
                }
                let (mesh, claims) = render(&bad, &shapes, &g, Some(&images));
                assert!(mesh.solid.positions.is_empty() && claims.iter().all(|v| !*v));
            }
        }
        let cropped = GridGeometry { height: 3, ..g };
        assert!(
            render(&tiles, &shapes, &cropped, Some(&images))
                .0
                .solid
                .positions
                .is_empty()
        );
        let mut no_ground = tiles.clone();
        for tile in &mut no_ground {
            if tile.column >= 4 {
                tile.source.tileset_id = "johto".into();
            }
        }
        assert!(
            render(&no_ground, &shapes, &g, Some(&images))
                .0
                .solid
                .positions
                .is_empty()
        );
        let mut translucent = images.clone();
        for sample in translucent.pixels.values_mut() {
            if let TileImageSample::Rgba(p) = sample {
                p[3] = 128;
            }
        }
        assert!(
            render(&tiles, &shapes, &g, Some(&translucent))
                .0
                .solid
                .positions
                .is_empty()
        );
        let mut raised = shapes.clone();
        raised[0] = CellShape::PlaneAt { height: 4. };
        for (shapes, images) in [(&raised[..], Some(&images)), (&shapes[..], None)] {
            let (mesh, claims) = render(&tiles, shapes, &g, images);
            assert!(mesh.solid.positions.is_empty() && claims.iter().all(|v| !*v));
        }
    }
}
