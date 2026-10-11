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
        let (_, unselected) = build_animated_flowers(&frame, &images, &[]).unwrap();
        assert_eq!(
            unselected.count_vertices(),
            0,
            "source art cannot bypass an override"
        );
        let (_, selected) = build_animated_flowers(&frame, &images, &[[2, 0]]).unwrap();
        assert!(selected.count_vertices() > 0);
        frame.map_id = "TinTower1F".into();
        let (_, unrelated) = build_animated_flowers(&frame, &images, &[[2, 0]]).unwrap();
        assert_eq!(
            unrelated.count_vertices(),
            0,
            "cached anchors cannot create a pillar on another map"
        );
    }

    #[test]
    fn sprout_pillar_moves_with_native_frames_and_keeps_base_normals_and_footing() {
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
            assert!(!mesh.animated_solid.indices.is_empty());
            assert!(
                mesh.animated_solid
                    .positions
                    .iter()
                    .all(|p| p[1] >= 0. && p[1] <= 48.001)
            );
            assert!(
                mesh.animated_solid
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length_squared() - 1.).abs() < 0.0001)
            );
            meshes.push(mesh.animated_solid);
        }
        assert_eq!(meshes[0].indices, meshes[1].indices);
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

pub(super) fn animated_source(map: &str, s: &VisualTileSource) -> bool {
    matches!(map, "SproutTower1F" | "SproutTower2F" | "SproutTower3F") && source_local(s).is_some()
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
    let rise = 48. * scale;
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
    crate::dungeon_models::model(crate::dungeon_models::Kind::TimberColumn).append(
        out,
        [
            w + 4. * scale,
            east - 4. * scale,
            south - 18. * scale,
            south - 2. * scale,
        ],
        0.,
        rise,
    );
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
