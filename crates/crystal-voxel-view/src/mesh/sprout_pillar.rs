//! Complete Sprout pillar drawing and pose driven by the live pack frames.
use super::*;

pub(super) fn source_local(s: &VisualTileSource) -> Option<(u8, u8, usize, usize)> {
    if s.tileset_id.as_ref() != "tower" || s.subtile_column >= 4 || s.subtile_row >= 4 {
        return None;
    }
    let (x, y) = match (s.metatile_id, s.subtile_column, s.subtile_row) {
        (0x10, 2..=3, y) => (s.subtile_column - 2, y),
        (0x11, 0..=1, y) => (s.subtile_column + 2, y),
        (0x14, 2..=3, y @ 0..=1) => (s.subtile_column - 2, y + 4),
        (0x15, 0..=1, y @ 0..=1) => (s.subtile_column + 2, y + 4),
        _ => return None,
    };
    let art = [
        [0x2d, 0x2e, 0x2e, 0x2f],
        [0x3d, 0x3e, 0x3e, 0x3f],
        [0x3c, 0x3e, 0x01, 0x2c],
        [0x4d, 0x4e, 0x4e, 0x4f],
        [0x4d, 0x4e, 0x4e, 0x4f],
        [0x5d, 0x5e, 0x5e, 0x5f],
    ];
    (s.tile_index == art[y as usize][x as usize]).then_some((x, y, 4, 6))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fixture() -> (Vec<VisualTile>, GridGeometry, Vec<Handle<Image>>) {
        let mut images = Assets::<Image>::default();
        let frames: Vec<_> = (0..5).map(|_| images.add(Image::default())).collect();
        let g = GridGeometry {
            width: 8,
            height: 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut cells: Vec<_> = (0..64)
            .map(|i| VisualTile {
                column: (i % 8) as u32,
                row: (i / 8) as u32,
                texture: frames[2].clone(),
                priority: false,
                animation_frames: None,
                source: VisualTileSource {
                    tileset_id: Arc::from("tower"),
                    metatile_id: 9,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 8 % 4) as u8,
                    tile_index: 2,
                },
            })
            .collect();
        let art = [
            [0x2d, 0x2e, 0x2e, 0x2f],
            [0x3d, 0x3e, 0x3e, 0x3f],
            [0x3c, 0x3e, 0x01, 0x2c],
            [0x4d, 0x4e, 0x4e, 0x4f],
            [0x4d, 0x4e, 0x4e, 0x4f],
            [0x5d, 0x5e, 0x5e, 0x5f],
        ];
        for y in 0..6 {
            for x in 0..4 {
                let tile = &mut cells[y * 8 + x + 2];
                tile.source.metatile_id = match (x < 2, y < 4) {
                    (true, true) => 0x10,
                    (false, true) => 0x11,
                    (true, false) => 0x14,
                    _ => 0x15,
                };
                tile.source.subtile_column = ((x + 2) % 4) as u8;
                tile.source.subtile_row = (y % 4) as u8;
                tile.source.tile_index = art[y][x];
                tile.animation_frames = Some(frames.clone().into());
            }
        }
        (cells, g, frames)
    }

    #[test]
    fn sprout_pillar_square_matches_projected_top_and_front_and_keeps_closed_grained_faces() {
        for scale in [0.5, 1., 2.] {
            let mut mesh = SurfaceMeshData::default();
            timber(&mut mesh, [0., 0.], scale, [0.4, 0.6], [0.2, 0.3]);
            assert!(mesh.positions.iter().all(|p| p[0].abs() <= 12.001 * scale
                && p[2].abs() <= 12.001 * scale
                && p[1] >= 0.
                && p[1] <= 24.001 * scale));
            assert!(mesh.positions.iter().any(|p| p[1] == 0.));
            assert!(mesh.positions.iter().any(|p| p[1] == 24. * scale));
            for (p, n) in mesh.positions.iter().zip(&mesh.normals) {
                if n[1].abs() < 0.1 {
                    assert!(p[0] * n[0] + p[2] * n[2] > 0., "outward square sides");
                }
            }
            let mut edges = std::collections::HashMap::new();
            let key = |p: [f32; 3]| p.map(|v| (v / scale * 10000.).round() as i32);
            for tri in mesh.indices.chunks_exact(3) {
                let a = Vec3::from_array(mesh.positions[tri[0] as usize]);
                let b = Vec3::from_array(mesh.positions[tri[1] as usize]);
                let c = Vec3::from_array(mesh.positions[tri[2] as usize]);
                assert!(
                    (b - a)
                        .cross(c - a)
                        .dot(Vec3::from_array(mesh.normals[tri[0] as usize]))
                        > 0.
                );
                for (i, j) in [(0, 1), (1, 2), (2, 0)] {
                    let mut pair = [
                        key(mesh.positions[tri[i] as usize]),
                        key(mesh.positions[tri[j] as usize]),
                    ];
                    pair.sort();
                    *edges.entry(pair).or_insert(0) += 1;
                }
            }
            assert!(
                edges.values().all(|&count| count == 2),
                "square timber has a closed surface"
            );
            assert!(mesh.colors.iter().all(|c| c[0] >= c[1] && c[3] == 1.));
            let shades: Vec<_> = mesh
                .positions
                .iter()
                .zip(&mesh.colors)
                .filter(|(p, _)| p[1] > 4. * scale && p[1] < 43. * scale)
                .map(|(_, c)| c[0])
                .collect();
            assert!(
                shades.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                    - shades.iter().copied().fold(f32::INFINITY, f32::min)
                    > 0.1
            );
        }
    }

    #[test]
    fn sprout_pillar_animation_requires_selected_ownership_and_correct_map() {
        let (cells, _, _) = fixture();
        let mut frame = crystal_render_api::VisualWorldFrame {
            map_id: "SproutTower1F".into(),
            grid_size: bevy::prelude::UVec2::splat(8),
            tile_size: bevy::prelude::Vec2::splat(8.),
            viewport_size: bevy::prelude::Vec2::splat(64.),
            tiles: cells,
            ..Default::default()
        };
        let images = Assets::<Image>::default();
        let (unselected, solid) = build_animated_flowers(&frame, &images, &[]).unwrap();
        assert_eq!(
            unselected.count_vertices() + solid.count_vertices(),
            0,
            "source art cannot bypass an override"
        );
        let (selected, _) = build_animated_flowers(&frame, &images, &[[2, 0]]).unwrap();
        assert!(selected.count_vertices() > 0);
        for level in 1..=9 {
            frame.map_id=format!("TinTower{level}F").into();
            let (selected,_) = build_animated_flowers(&frame,&images,&[[2,0]]).unwrap();
            assert!(selected.count_vertices()>0);
            let (unselected,_) = build_animated_flowers(&frame,&images,&[]).unwrap();
            assert_eq!(unselected.count_vertices(),0,"Tin custom ownership must also exclude the pillar");
        }
        frame.map_id = "TinTowerRoof".into();
        let (unrelated, solid) = build_animated_flowers(&frame, &images, &[[2, 0]]).unwrap();
        assert_eq!(
            unrelated.count_vertices() + solid.count_vertices(),
            0,
            "cached anchors cannot create a pillar on another map"
        );
    }

    #[test]
    fn sprout_pillar_live_palette_moves_with_frames_and_keeps_base_normals_and_footing() {
        let (mut cells, g, frames) = fixture();
        let mut meshes = Vec::new();
        for frame in [0, 4] {
            for tile in &mut cells {
                tile.texture = frames[frame].clone();
            }
            let refs: Vec<_> = cells.iter().collect();
            let p = modeled_dungeons::resolve("SproutTower1F", &refs, &g, None);
            let pillar = p
                .iter()
                .find(|p| p.kind_label() == "special:sprout-swaying-pillar")
                .unwrap();
            assert_eq!(pillar.indices(8).count(), 24);
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.5; 64],
                ..Default::default()
            };
            modeled_dungeons::append(&mut mesh, &g, pillar, &refs, [0, 0]);
            assert_eq!(mesh.animated_pillars, vec![[2, 0]]);
            assert!(mesh.footing_heights.iter().all(|h| *h == 2.5));
            assert!(!mesh.animated_textured.indices.is_empty());
            assert!(mesh.animated_solid.indices.is_empty());
            for pixel in [[14, 26], [12, 6]] {
                let uv = g.uv(2 + pixel[0] / 8, pixel[1] / 8);
                assert!(mesh.animated_textured.uvs.contains(&[
                    uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                    uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.,
                ]));
            }
            assert!(
                mesh.animated_textured
                    .positions
                    .iter()
                    .all(|p| p[1] >= 0. && p[1] <= 48.001)
            );
            assert!(
                mesh.animated_textured
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length_squared() - 1.).abs() < 0.0001)
            );
            meshes.push(mesh.animated_textured);
        }
        assert_eq!(meshes[0].indices, meshes[1].indices);
        assert_eq!(meshes[0].uvs, meshes[1].uvs);
        assert!(
            meshes[0]
                .positions
                .iter()
                .zip(&meshes[1].positions)
                .any(|(a, b)| (a[0] - b[0]).abs() > 1.)
        );
        for (a, b) in meshes[0].positions.iter().zip(&meshes[1].positions) {
            if a[1] == 0. {
                assert_eq!(a, b, "pillar base must remain anchored");
            }
        }
        cells[2].source.tile_index ^= 1;
        let mut rejected = SurfaceMeshData::default();
        append(&mut rejected, &g, &cells.iter().collect::<Vec<_>>(), [2, 0]);
        assert!(rejected.positions.is_empty());
        assert!(
            !modeled_dungeons::resolve(
                "SproutTower1F",
                &cells.iter().collect::<Vec<_>>(),
                &g,
                None
            )
            .iter()
            .any(|p| p.kind_label() == "special:sprout-swaying-pillar")
        );
    }
}

pub(super) fn pillar_map(map:&str)->bool {
    matches!(map,"SproutTower1F"|"SproutTower2F"|"SproutTower3F")
        ||super::modeled_dungeons::tin_tower_guardian_map(map)
}
pub(super) fn animated_source(map: &str, s: &VisualTileSource) -> bool {
    pillar_map(map) && source_local(s).is_some()
}

pub(super) fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    cells: &[&VisualTile],
    anchor: [usize; 2],
) {
    let [column, row] = anchor;
    if column + 4 > g.width || row + 6 > g.height {
        return;
    }
    // Recheck the complete visible source; an animation update must never
    // create a pillar from an altered drawing or a partial camera crop.
    if !(0..6).all(|y| {
        (0..4).all(|x| {
            source_local(&cells[(row + y) * g.width + column + x].source)
                == Some((x as u8, y as u8, 4, 6))
        })
    }) {
        return;
    }
    let scale = g.tile_height / SOURCE_TILE_HEIGHT;
    let (w, _, _, _) = g.bounds(column, row);
    let (_, _, _, south) = g.bounds(column + 3, row + 5);
    let east = w + 4. * g.tile_width;
    let rise = 24. * scale;
    let mut poses = [0.; 6];
    for (y, pose) in poses.iter_mut().enumerate() {
        *pose = (0..4)
            .find_map(|x| {
                let tile = cells[(row + y) * g.width + column + x];
                let frames = tile.animation_frames.as_ref()?;
                let frame = frames.iter().position(|handle| *handle == tile.texture)?;
                Some((frame.min(4) as f32 - 2.) * scale)
            })
            .unwrap_or(0.);
    }
    let bend = |height: f32| {
        let t = (height / rise).clamp(0., 1.);
        let sample = (1. - t) * 5.;
        let a = (sample.floor() as usize).min(4);
        let fraction = sample - a as f32;
        (poses[a] * (1. - fraction) + poses[a + 1] * fraction) * t
    };
    let first = out.positions.len();
    // The 2D drawing projects a broad square dark top above a full-width
    // amber front. Its 48 drawn pixels include both depth and vertical face;
    // turning all 48 into height creates a narrow, towering column.
    let center = [(w + east) * 0.5, south - 14. * scale];
    let sample = |pixel: [usize; 2]| {
        let native = g.uv(column + pixel[0] / 8, row + pixel[1] / 8);
        [
            native.0
                + (native.1 - native.0) * (pixel[0] % 8) as f32 / 8.
                + (native.1 - native.0) / 16.,
            native.2
                + (native.3 - native.2) * (pixel[1] % 8) as f32 / 8.
                + (native.3 - native.2) / 16.,
        ]
    };
    timber(out, center, scale, sample([14, 26]), sample([12, 6]));
    for i in first..out.positions.len() {
        let height = out.positions[i][1];
        out.positions[i][0] += bend(height);
        let epsilon = 0.02 * scale;
        let slope = (bend(height + epsilon) - bend(height - epsilon)) / (2. * epsilon);
        let old = Vec3::from_array(out.normals[i]);
        let normal = Vec3::new(old.x, old.y - slope * old.x, old.z).normalize();
        let before = crate::interior_models::authored_face_color([1.; 4], old)[0];
        let after = crate::interior_models::authored_face_color([1.; 4], normal)[0];
        out.normals[i] = normal.to_array();
        for channel in &mut out.colors[i][..3] {
            *channel *= after / before;
        }
    }
}

/// A square timber block: broad native dark top and full amber side faces.
/// The source drawing's top depth and front rise are each 24 pixels.
fn timber(
    out: &mut SurfaceMeshData,
    center: [f32; 2],
    scale: f32,
    side_uv: [f32; 2],
    top_uv: [f32; 2],
) {
    let x0 = center[0] - 12. * scale;
    let x1 = center[0] + 12. * scale;
    let z0 = center[1] - 12. * scale;
    let z1 = center[1] + 12. * scale;
    let height = 24. * scale;
    // Four continuous flat sides; subdivisions carry grain and the source's
    // dark mottled foot, without adding collars, a plinth or a bright stripe.
    let edges = [
        ([x0, z0], [x0, z1]),
        ([x0, z1], [x1, z1]),
        ([x1, z1], [x1, z0]),
        ([x1, z0], [x0, z0]),
    ];
    for (b, a) in edges {
        for y in 0..12 {
            for x in 0..12 {
                let point = |u: f32, v: f32| {
                    [
                        a[0] + (b[0] - a[0]) * u,
                        v * height,
                        a[1] + (b[1] - a[1]) * u,
                    ]
                };
                let lo = y as f32 / 12.;
                let hi = (y + 1) as f32 / 12.;
                let left = x as f32 / 12.;
                let right = (x + 1) as f32 / 12.;
                let p = [
                    point(left, lo),
                    point(left, hi),
                    point(right, hi),
                    point(right, lo),
                ];
                let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
                    .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
                    .normalize();
                let colors = p.map(|p| {
                    let u = (p[0] + p[2] - center[0] - center[1]) / scale;
                    let y = p[1] / scale;
                    let grain = (u * 1.6 + (y * 0.13).sin() * 0.5).sin() * 0.07;
                    let foot = if y < 4. {
                        0.30 * ((u * 2.1 + y * 2.7).sin() * 0.5 + 0.5)
                    } else {
                        0.
                    };
                    let pigment = 1. + grain - foot;
                    crate::interior_models::authored_face_color(
                        [pigment, pigment * 0.91, pigment * 0.88, 1.],
                        normal,
                    )
                });
                append_quad_colors(out, p, normal.to_array(), [side_uv; 4], colors);
            }
        }
    }
    // Matching two-pixel cap subdivisions close the same side perimeter.
    for (y, normal, uv) in [(0., -Vec3::Y, side_uv), (height, Vec3::Y, top_uv)] {
        for z in 0..12 {
            for x in 0..12 {
                let w = x0 + (x1 - x0) * x as f32 / 12.;
                let e = x0 + (x1 - x0) * (x + 1) as f32 / 12.;
                let n = z0 + (z1 - z0) * z as f32 / 12.;
                let s = z0 + (z1 - z0) * (z + 1) as f32 / 12.;
                let mut p = [[w, y, n], [w, y, s], [e, y, s], [e, y, n]];
                if normal.y < 0. {
                    p.reverse();
                }
                let color = crate::interior_models::authored_face_color([1.; 4], normal);
                append_quad(out, p, normal.to_array(), [uv; 4], color);
            }
        }
    }
}
