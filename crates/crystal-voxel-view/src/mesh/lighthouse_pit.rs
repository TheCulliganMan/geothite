//! Native fall shafts below the unchanged lighthouse floor and warp datum.
use super::*;

fn complete(cells: &[&VisualTile], g: &GridGeometry, column: usize, row: usize) -> bool {
    if row < 2
        || column + 4 > g.width
        || row + 2 > g.height
        || g.width.checked_mul(g.height) != Some(cells.len())
    {
        return false;
    }
    (0..4).all(|y| {
        (0..4).all(|x| {
            let s = &cells[(row - 2 + y) * g.width + column + x].source;
            s.tileset_id.as_ref() == "lighthouse"
                && s.metatile_id == 0x28
                && s.subtile_column as usize == x
                && s.subtile_row as usize == y
                && s.tile_index
                    == if y >= 2 {
                        0x01
                    } else if (x + y) % 2 == 0 {
                        0x2e
                    } else {
                        0x2f
                    }
        })
    })
}

pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    if !lighthouse_map(r.map) || r.cells.len() != r.g.width * r.g.height {
        return;
    }
    for row in 2..r.g.height {
        for column in 0..r.g.width {
            let s = &r.cells[row * r.g.width + column].source;
            if s.subtile_column != 0 || s.subtile_row != 2 || !complete(r.cells, r.g, column, row) {
                continue;
            }
            r.add(Placement {
                column,
                row,
                width: 4,
                height: 2,
                ground: (row - 2) * r.g.width + column,
                form: Form::LighthousePit,
                rise_pixels: 0.,
                depth_pixels: 16.,
                front_rows: 2.,
            });
        }
    }
}

/// Floor-level sight targets for nearby complete openings. These never resize
/// the player's capsule or grant reveal eligibility to otherwise opaque art.
pub(super) fn reveal_targets(frame: &VisualWorldFrame, foot: Vec3) -> Vec<Vec3> {
    if !lighthouse_map(&frame.map_id)
        || !foot.is_finite()
        || !frame.center.is_finite()
        || !frame.tile_size.is_finite()
        || frame.tile_size.min_element() <= 0.
    {
        return Vec::new();
    }
    let (width, height) = (frame.grid_size.x as usize, frame.grid_size.y as usize);
    if width.checked_mul(height) != Some(frame.tiles.len()) || width < 4 || height < 4 {
        return Vec::new();
    }
    let mut indexed = vec![None; frame.tiles.len()];
    for tile in &frame.tiles {
        let (x, y) = (tile.column as usize, tile.row as usize);
        if x >= width || y >= height || indexed[y * width + x].replace(tile).is_some() {
            return Vec::new();
        }
    }
    let Some(cells) = indexed.into_iter().collect::<Option<Vec<_>>>() else {
        return Vec::new();
    };
    let g = GridGeometry {
        width,
        height,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: frame.center.x - width as f32 * frame.tile_size.x * 0.5,
        origin_z: -frame.center.y - height as f32 * frame.tile_size.y * 0.5,
    };
    let mut targets = Vec::new();
    for row in 2..height {
        for column in 0..width {
            let source = &cells[row * width + column].source;
            if source.subtile_column != 0
                || source.subtile_row != 2
                || !complete(&cells, &g, column, row)
            {
                continue;
            }
            let (w, _, n, _) = g.bounds(column, row);
            let (e, s) = (w + 4. * g.tile_width, n + 2. * g.tile_height);
            let center = Vec3::new((w + e) * 0.5, 0.04, (n + s) * 0.5);
            let dx = (foot.x - center.x) / (g.tile_width * 4.);
            let dz = (foot.z - center.z) / (g.tile_height * 4.);
            if dx * dx + dz * dz > 1. {
                continue;
            }
            let inset = g.tile_width.min(g.tile_height) * 0.15;
            targets.extend([
                center,
                Vec3::new(w + inset, 0.04, n + inset),
                Vec3::new(e - inset, 0.04, n + inset),
                Vec3::new(e - inset, 0.04, s - inset),
                Vec3::new(w + inset, 0.04, s - inset),
            ]);
        }
    }
    targets
}

fn paint(g: &GridGeometry, column: usize, row: usize, x: usize, y: usize) -> [f32; 2] {
    let uv = g.uv(column, row);
    [
        uv.0 + (uv.1 - uv.0) * (x as f32 + 0.5) / 8.,
        uv.2 + (uv.3 - uv.2) * (y as f32 + 0.5) / 8.,
    ]
}

pub(super) fn append(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let (e, s) = (w + 4. * g.tile_width, n + 2. * g.tile_height);
    let lip = g.tile_width.min(g.tile_height) * 0.08;
    let (iw, ie, near, far) = (w + lip, e - lip, n + lip, s - lip);
    let bevel = -g.tile_height * 0.1;
    let bottom = -g.tile_height * 2.;
    // All three pigments belong to this complete block, including its upper
    // floor drawing. No neighboring wall/atlas-border pixels cross the rim.
    let rim = paint(g, p.column, p.row - 2, 3, 3);
    let stone = paint(g, p.column, p.row - 2, 0, 0);
    let black = paint(g, p.column, p.row, 0, 0);
    for points in [
        [[w, 0., n], [iw, bevel, near], [ie, bevel, near], [e, 0., n]],
        [[e, 0., n], [ie, bevel, near], [ie, bevel, far], [e, 0., s]],
        [[e, 0., s], [ie, bevel, far], [iw, bevel, far], [w, 0., s]],
        [[w, 0., s], [iw, bevel, far], [iw, bevel, near], [w, 0., n]],
    ] {
        let [a, b, c, _] = points.map(Vec3::from_array);
        let normal = (b - a).cross(c - a).normalize().to_array();
        append_quad(&mut mesh.textured, points, normal, [rim; 4], [1.; 4]);
    }
    for (a, b, normal) in [
        ([iw, near], [ie, near], [0., 0., 1.]),
        ([ie, near], [ie, far], [-1., 0., 0.]),
        ([ie, far], [iw, far], [0., 0., -1.]),
        ([iw, far], [iw, near], [1., 0., 0.]),
    ] {
        append_quad(
            &mut mesh.textured,
            [
                [a[0], bevel, a[1]],
                [a[0], bottom, a[1]],
                [b[0], bottom, b[1]],
                [b[0], bevel, b[1]],
            ],
            normal,
            [stone; 4],
            [1.; 4],
        );
    }
    append_quad(
        &mut mesh.textured,
        [
            [iw, bottom, near],
            [iw, bottom, far],
            [ie, bottom, far],
            [ie, bottom, near],
        ],
        [0., 1., 0.],
        [black; 4],
        [1.; 4],
    );
    // The mouth stays open. Neither support heights nor cutaway ownership are
    // changed: authored fall input, not this visual mesh, owns the transition.
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tiles() -> Vec<VisualTile> {
        (0..16)
            .map(|i| {
                let (x, y) = (i % 4, i / 4);
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "lighthouse".into(),
                        metatile_id: 0x28,
                        subtile_column: x as u8,
                        subtile_row: y as u8,
                        tile_index: if y >= 2 {
                            1
                        } else if (x + y) % 2 == 0 {
                            0x2e
                        } else {
                            0x2f
                        },
                    },
                }
            })
            .collect()
    }
    fn grid(scale: f32) -> GridGeometry {
        GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8. * scale,
            tile_height: 8. * scale,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    #[test]
    fn lighthouse_pit_requires_complete_block_and_reserves_only_the_opening() {
        let t = tiles();
        let g = grid(1.);
        let refs = t.iter().collect::<Vec<_>>();
        let resolved = resolve("OlivineLighthouse2F", &refs, &g, None);
        let pits = resolved
            .iter()
            .filter(|p| matches!(p.form, Form::LighthousePit))
            .collect::<Vec<_>>();
        assert_eq!(pits.len(), 1);
        assert!(pits[0].cuts_foundation());
        assert!(!pits[0].room_wall());
        assert_eq!(
            pits[0].indices(4).collect::<Vec<_>>(),
            (8..16).collect::<Vec<_>>()
        );
        for i in 0..16 {
            for field in 0..4 {
                let mut changed = t.clone();
                let s = &mut changed[i].source;
                match field {
                    0 => s.tile_index = 0xffff,
                    1 => s.metatile_id = 0x27,
                    2 => s.tileset_id = "custom_lighthouse".into(),
                    _ => s.subtile_row = (s.subtile_row + 1) % 4,
                }
                assert!(
                    !resolve(
                        "OlivineLighthouse2F",
                        &changed.iter().collect::<Vec<_>>(),
                        &g,
                        None
                    )
                    .iter()
                    .any(|p| matches!(p.form, Form::LighthousePit)),
                    "cell {i}, field {field}"
                );
            }
        }
        assert!(
            !resolve("TinTower2F", &refs, &g, None)
                .iter()
                .any(|p| matches!(p.form, Form::LighthousePit))
        );
        let clipped = GridGeometry { height: 3, ..g };
        assert!(
            !resolve("OlivineLighthouse2F", &refs[..12], &clipped, None)
                .iter()
                .any(|p| matches!(p.form, Form::LighthousePit))
        );
    }
    #[test]
    fn lighthouse_pit_has_inward_wound_closed_shaft_and_no_floor_bridge() {
        for scale in [0.5, 1., 2.] {
            let t = tiles();
            let refs = t.iter().collect::<Vec<_>>();
            let g = grid(scale);
            let resolved = resolve("OlivineLighthouse2F", &refs, &g, None);
            let p = resolved
                .iter()
                .find(|p| matches!(p.form, Form::LighthousePit))
                .unwrap();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; 16],
                ..Default::default()
            };
            append(&mut mesh, &g, p);
            assert_eq!(mesh.footing_heights, vec![2.75; 16]);
            assert!(mesh.solid.positions.is_empty());
            assert!(mesh.textured.cutaway_ranges.is_empty());
            assert!(mesh.depth_coverage_textured_ranges.is_empty());
            assert_eq!(mesh.textured.indices.len(), 18 * 3);
            assert!(mesh.textured.positions.iter().all(|v| v[1] <= 0.));
            assert_eq!(
                mesh.textured
                    .positions
                    .iter()
                    .map(|v| v[1])
                    .fold(f32::INFINITY, f32::min),
                -16. * scale
            );
            let mut edges = std::collections::BTreeMap::new();
            for tri in mesh.textured.indices.chunks_exact(3) {
                let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                let [a, b, c] = ids.map(|i| Vec3::from_array(mesh.textured.positions[i]));
                assert!(
                    (b - a)
                        .cross(c - a)
                        .dot(Vec3::from_array(mesh.textured.normals[ids[0]]))
                        > 0.
                );
                assert!(
                    ids.iter()
                        .all(|&i| mesh.textured.uvs[i] == mesh.textured.uvs[ids[0]])
                );
                assert!(
                    !(a.y == 0. && b.y == 0. && c.y == 0.),
                    "no surface bridges the mouth"
                );
                for (a, b) in [(a, b), (b, c), (c, a)] {
                    let mut key = [
                        a.to_array().map(|v| (v * 10000.).round() as i64),
                        b.to_array().map(|v| (v * 10000.).round() as i64),
                    ];
                    key.sort();
                    *edges.entry(key).or_insert(0usize) += 1;
                }
            }
            let boundary = edges.iter().filter(|(_, n)| **n == 1).collect::<Vec<_>>();
            assert_eq!(boundary.len(), 4, "only the upper opening is unclosed");
            assert!(
                boundary
                    .iter()
                    .all(|(edge, _)| edge[0][1] == 0 && edge[1][1] == 0)
            );
            assert!(edges.values().all(|&n| n == 1 || n == 2));
        }
    }
    #[test]
    fn lighthouse_pit_reveal_targets_follow_complete_nearby_source_and_world_offset() {
        let mut frame = VisualWorldFrame {
            active: true,
            map_id: "OlivineLighthouse2F".into(),
            grid_size: bevy::prelude::UVec2::new(4, 4),
            tile_size: bevy::prelude::Vec2::splat(8.),
            center: bevy::prelude::Vec2::new(100., -40.),
            tiles: tiles(),
            ..Default::default()
        };
        let foot = Vec3::new(76., 0.04, 48.);
        let targets = reveal_targets(&frame, foot);
        assert_eq!(targets.len(), 5);
        assert_eq!(targets[0], Vec3::new(100., 0.04, 48.));
        assert!(targets.iter().all(|p| p.x > 84.
            && p.x < 116.
            && p.z > 40.
            && p.z < 56.
            && p.y == 0.04));
        frame.tiles.reverse();
        assert_eq!(reveal_targets(&frame, foot), targets);
        assert!(reveal_targets(&frame, Vec3::new(-1000., 0., 0.)).is_empty());
        assert!(reveal_targets(&frame, Vec3::splat(f32::NAN)).is_empty());
        frame.map_id = "TinTower2F".into();
        assert!(reveal_targets(&frame, foot).is_empty());
        frame.map_id = "OlivineLighthouse2F".into();
        for i in 0..16 {
            let old = frame.tiles[i].source.tile_index;
            frame.tiles[i].source.tile_index = 0xffff;
            assert!(
                reveal_targets(&frame, foot).is_empty(),
                "changed source cell {i}"
            );
            frame.tiles[i].source.tile_index = old;
        }
        frame.tiles[0] = frame.tiles[1].clone();
        assert!(reveal_targets(&frame, foot).is_empty());
    }
    #[test]
    fn lighthouse_pit_full_scene_keeps_floor_overlays_out_of_the_shaft() {
        let mut tiles = tiles();
        let mut images = Assets::<Image>::default();
        let mut image = Image::default();
        image.texture_descriptor.size.width = 8;
        image.texture_descriptor.size.height = 8;
        image.data = Some(vec![255; 8 * 8 * 4]);
        let texture = images.add(image);
        for tile in &mut tiles {
            tile.texture = texture.clone();
        }
        let frame = VisualWorldFrame {
            active: true,
            map_id: "OlivineLighthouse2F".into(),
            grid_size: bevy::prelude::UVec2::new(4, 4),
            tile_size: bevy::prelude::Vec2::splat(8.),
            viewport_size: bevy::prelude::Vec2::splat(32.),
            map_texture: texture,
            tiles,
            ..Default::default()
        };
        let mesh = crate::mesh::audit_terrain_mesh_with_images(&frame, &images).unwrap();
        assert!(mesh.authored_cells[..8].iter().all(Option::is_none));
        assert!(
            mesh.authored_cells[8..]
                .iter()
                .all(|v| *v == Some("lighthouse:recessed-pit"))
        );
        assert_eq!(mesh.footing_heights, vec![0.; 16]);
        for (x, z) in [(-8., 4.), (0., 8.), (8., 12.)] {
            let mut heights = Vec::new();
            for surface in [&mesh.textured, &mesh.solid] {
                for tri in surface.indices.chunks_exact(3) {
                    let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| surface.positions[i as usize]);
                    let den = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
                    if den.abs() < 1e-6 {
                        continue;
                    }
                    let u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / den;
                    let v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / den;
                    if u >= -1e-5 && v >= -1e-5 && u + v <= 1. + 1e-5 {
                        heights.push(u * a[1] + v * b[1] + (1. - u - v) * c[1]);
                    }
                }
            }
            assert!(!heights.is_empty(), "the recessed bottom remains closed");
            assert!(
                heights.iter().all(|h| (*h + 16.).abs() < 1e-3),
                "floor bridges the opening at ({x},{z}): {heights:?}"
            );
        }
    }
    #[test]
    #[ignore = "requires the user-supplied external pack"]
    fn external_lighthouse_pit_admits_six_native_openings_and_keeps_support() {
        let path =
            std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("explicit external pack required");
        let loaded =
            crystal_assets::read_loaded_verified_compiled_game_pack(std::path::Path::new(&path))
                .unwrap();
        let pack = loaded.pack();
        let art = &pack.runtime_files()["data/tilesets/lighthouse_metatiles.bin"];
        let png = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/lighthouse.png"])
            .unwrap()
            .to_luma8();
        for (tile, x, y, value) in [(1, 0, 0, 0), (0x2e, 0, 0, 85), (0x2e, 3, 3, 170)] {
            assert_eq!(
                png.get_pixel((tile % 16 * 8 + x) as u32, (tile / 16 * 8 + y) as u32)[0],
                value
            );
        }
        for level in 1..=6 {
            let name = format!("OlivineLighthouse{level}F");
            let map = &pack.data().maps[&name];
            let (w, h) = (
                map.attributes.width as usize * 4,
                map.attributes.height as usize * 4,
            );
            let tiles = (0..w * h)
                .map(|i| {
                    let (x, y) = (i % w, i / w);
                    let block = map.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: "lighthouse".into(),
                            metatile_id: block,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: art[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let resolved = resolve(&name, &refs, &g, None);
            let pits = resolved
                .iter()
                .filter(|p| matches!(p.form, Form::LighthousePit))
                .collect::<Vec<_>>();
            assert_eq!(pits.len(), [0, 1, 1, 2, 1, 1][level - 1], "{name}");
            for p in pits {
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![2.75; w * h],
                    ..Default::default()
                };
                super::super::append(&mut mesh, &g, p, &refs, [0, 0]);
                assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
                assert_eq!(p.indices(w).count(), 8);
                assert_eq!(mesh.textured.indices.len(), 54);
                assert_eq!(refs[p.ground].source.tile_index, 0x2e);
                assert!(p.indices(w).all(|i| refs[i].source.tile_index == 1));
            }
        }
    }
}
