// Timber Gym perimeters use vertical architecture. Scope complete side-strip
// drawings by map and source without changing the invisible-path puzzle or support.
fn append_cianwood_walls(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if !matches!(map, "CianwoodGym" | "EcruteakGym") {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "tower", crate::tower::TOWER_FLOOR_TILE) else {
        return;
    };
    for (i, t) in cells.iter().enumerate() {
        let s = &t.source;
        let (ox, oy, h) = match s.metatile_id {
            0x04 => (0, 2, 2),
            0x06 => (2, 2, 2),
            0x08 => (0, 0, 4),
            0x0a => (2, 0, 4),
            0x0c | 0x16 => (0, 0, 2),
            0x0e | 0x17 => (2, 0, 2),
            0x21 => (2, 0, 2),
            0x22 => (0, 0, 2),
            _ => continue,
        };
        if s.tileset_id.as_ref() != "tower"
            || s.subtile_column as usize != ox
            || s.subtile_row as usize != oy
        {
            continue;
        }
        let (column, row) = (i % g.width, i / g.width);
        if column + 2 > g.width || row + h > g.height {
            continue;
        }
        if !(0..h).all(|y| {
            (0..2).all(|x| {
                let j = (row + y) * g.width + column + x;
                let t = &cells[j].source;
                !claimed[j]
                    && t.tileset_id == s.tileset_id
                    && t.metatile_id == s.metatile_id
                    && t.subtile_column as usize == ox + x
                    && t.subtile_row as usize == oy + y
                    && t.tile_index
                        == if matches!(s.metatile_id, 0x21 | 0x22) {
                            [0x11, 0x21][y]
                        } else {
                            [0x41, 0x31][x]
                        }
            })
        }) {
            continue;
        }
        let p = Placement {
            kind: ModelKind::WallTraditional,
            column,
            row,
            width: 2,
            height: h,
            ground,
            depth_pixels: if matches!(s.metatile_id, 0x21 | 0x22) {
                3.0
            } else {
                h as f32 * 8.0
            },
            height_pixels: 16.0,
            base_pixels: 0.0,
            footing_pixels: None,
        };
        for j in p.indices(g.width) {
            claimed[j] = true;
        }
        out.push(p);
    }
}

#[cfg(test)]
mod cianwood_wall_tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn perimeter_joins_keep_floor_free_and_reject_altered_drawings() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        for (block, ox, oy, h) in [
            (4, 0, 2, 2),
            (6, 2, 2, 2),
            (8, 0, 0, 4),
            (10, 2, 0, 4),
            (12, 0, 0, 2),
            (14, 2, 0, 2),
            (22, 0, 0, 2),
            (23, 2, 0, 2),
            (33, 2, 0, 2),
            (34, 0, 0, 2),
        ] {
            let mut t: Vec<VisualTile> = (0..16)
                .map(|i| VisualTile {
                    column: (i % 4) as u32,
                    row: (i / 4) as u32,
                    texture: Handle::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: Arc::from("tower"),
                        metatile_id: 9,
                        subtile_column: (i % 4) as u8,
                        subtile_row: (i / 4) as u8,
                        tile_index: 2,
                    },
                })
                .collect();
            for y in oy..oy + h {
                for x in ox..ox + 2 {
                    t[y * 4 + x].source.metatile_id = block;
                    t[y * 4 + x].source.tile_index = if matches!(block, 0x21 | 0x22) {
                        [0x11, 0x21][y]
                    } else {
                        [0x41, 0x31][x - ox]
                    };
                }
            }
            for map in ["CianwoodGym", "EcruteakGym"] {
                let mut p = Vec::new();
                let mut claimed = vec![false; 16];
                append_cianwood_walls(map, &t.iter().collect::<Vec<_>>(), &g, &mut claimed, &mut p);
                assert_eq!(p.len(), 1);
                assert_eq!(claimed.iter().filter(|&&x| x).count(), h * 2);
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![0.0; 16],
                    ..Default::default()
                };
                assert!(append(
                    &mut mesh,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &p[0],
                    &mut vec![false; 16],
                    [0, 0]
                ));
                assert_eq!(mesh.footing_heights, vec![0.0; 16]);
                assert!(mesh.solid.positions.iter().all(|p| p[0] >= ox as f32 * 8.0
                    && p[0] <= (ox + 2) as f32 * 8.0
                    && p[2] >= oy as f32 * 8.0
                    && p[2] <= (oy + h) as f32 * 8.0));
                let original = t[oy * 4 + ox].source.tile_index;
                t[oy * 4 + ox].source.tile_index = 0xff;
                let mut bad = Vec::new();
                append_cianwood_walls(
                    map,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &mut vec![false; 16],
                    &mut bad,
                );
                assert!(bad.is_empty());
                t[oy * 4 + ox].source.tile_index = original;
                append_cianwood_walls(
                    "SproutTower1F",
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &mut vec![false; 16],
                    &mut bad,
                );
                assert!(bad.is_empty());
                let mut reserved = vec![false; 16];
                reserved[oy * 4 + ox] = true;
                append_cianwood_walls(
                    map,
                    &t.iter().collect::<Vec<_>>(),
                    &g,
                    &mut reserved,
                    &mut bad,
                );
                assert!(bad.is_empty());
            }
        }
    }
}

fn append_side_wall(
    mesh: &mut SurfaceMeshData,
    bounds: [f32; 4],
    rise: f32,
    facing: f32,
    tile_width: f32,
) {
    let [west, east, north, south] = bounds;
    let depth = south - north;
    let thickness = tile_width * 0.75;
    let cx = if facing > 0.0 {
        west + tile_width * 0.5
    } else {
        east - tile_width * 0.5
    };
    let cz = (north + south) * 0.5;
    let first = mesh.positions.len();
    crate::dungeon_models::model(crate::dungeon_models::Kind::TimberWall).append(
        mesh,
        [-depth * 0.5, depth * 0.5, -thickness * 0.5, thickness * 0.5],
        0.0,
        rise,
    );
    for p in &mut mesh.positions[first..] {
        let [x, y, z] = *p;
        *p = [cx + facing * z, y, cz - facing * x];
    }
    for (n, c) in mesh.normals[first..]
        .iter_mut()
        .zip(&mut mesh.colors[first..])
    {
        let [x, y, z] = *n;
        let old = crate::interior_models::authored_face_color([1.0; 4], Vec3::from_array(*n))[0];
        *n = [facing * z, y, -facing * x];
        let new = crate::interior_models::authored_face_color([1.0; 4], Vec3::from_array(*n))[0];
        for channel in &mut c[..3] {
            *channel *= new / old;
        }
    }
}
