use super::*;
use std::sync::Arc;
fn tile(x: usize, y: usize, block: u16, art: u16) -> VisualTile {
    VisualTile {
        animation_frames: None,
        column: x as u32,
        row: y as u32,
        source: VisualTileSource {
            tileset_id: Arc::from("traditional_house"),
            metatile_id: block,
            subtile_column: (x % 4) as u8,
            subtile_row: (y % 4) as u8,
            tile_index: art,
        },
        texture: Default::default(),
        priority: false,
    }
}
fn floor_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let art = [
        [0x44, 0x45, 0x45, 0x46],
        [0x54, 0x55, 0x55, 0x56],
        [0x45, 0x46, 0x44, 0x45],
        [0x55, 0x56, 0x54, 0x55],
    ];
    (
        (0..16)
            .map(|i| tile(i % 4, i / 4, 4, art[i / 4][i % 4]))
            .collect(),
        GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -7.,
            origin_z: 3.,
        },
    )
}
#[test]
fn continuous_tatami_finishes_all_sixteen_cells_without_height_or_object_credit() {
    let (t, g) = floor_fixture();
    let c: Vec<_> = t.iter().collect();
    let mut mesh = TerrainMeshData::default();
    mesh.authored_cells = vec![None; 16];
    mesh.footing_heights = vec![0.; 16];
    for y in 0..4 {
        for x in 0..4 {
            append_top(&mut mesh.textured, g.bounds(x, y).into(), 0., g.uv(x, y));
        }
    }
    assert_eq!(
        modeled_interiors::finish_surfaces(&mut mesh, "KurtsHouse", &c, &g, [0, 0]),
        16
    );
    assert!(mesh.textured.indices.is_empty());
    assert!(mesh.solid.positions.iter().all(|p| p[1] == 0.));
    assert_eq!(mesh.authored_cells, vec![None; 16]);
    assert_eq!(mesh.footing_heights, vec![0.; 16]);
}
#[test]
fn tatami_rejects_changed_drawing_phase_custom_height_and_distorted_uv() {
    let (t, g) = floor_fixture();
    let c: Vec<_> = t.iter().collect();
    assert_eq!(
        tatami_floor_mask("DanceTheater", &c, &g, [0, 0], &[])
            .iter()
            .filter(|&&v| v)
            .count(),
        16
    );
    assert!(
        tatami_floor_mask("DanceTheater", &c, &g, [1, 0], &[])
            .iter()
            .all(|v| !*v)
    );
    for i in 0..16 {
        let mut bad = t.clone();
        bad[i].source.tile_index ^= 1;
        let c: Vec<_> = bad.iter().collect();
        assert!(
            tatami_floor_mask("DanceTheater", &c, &g, [0, 0], &[])
                .iter()
                .all(|v| !*v)
        );
        let mut excluded = vec![false; 16];
        excluded[i] = true;
        assert!(
            tatami_floor_mask(
                "KurtsHouse",
                &t.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
                &excluded
            )
            .iter()
            .all(|v| !*v)
        );
    }
    for mode in 0..4 {
        let mut mesh = TerrainMeshData::default();
        append_top(
            &mut mesh.textured,
            g.bounds(3, 0).into(),
            if mode == 0 { 8. } else { 0. },
            if mode == 1 { g.uv(1, 0) } else { g.uv(3, 0) },
        );
        if mode == 2 {
            mesh.textured.uvs[0][0] += 0.02;
        }
        if mode == 3 {
            mesh.textured.normals[0] = [0., 0., 1.];
        }
        let before = mesh.textured.clone();
        assert_eq!(
            modeled_interiors::finish_surfaces(&mut mesh, "KurtsHouse", &c, &g, [0, 0]),
            0
        );
        assert_eq!(mesh.textured.positions, before.positions);
        assert_eq!(mesh.textured.indices, before.indices);
    }
}
fn theater_fixture() -> (Vec<VisualTile>, GridGeometry) {
    let g = GridGeometry {
        width: 24,
        height: 12,
        tile_width: 8.,
        tile_height: 8.,
        origin_x: -96.,
        origin_z: -48.,
    };
    let tatami = [
        [0x44, 0x45, 0x45, 0x46],
        [0x54, 0x55, 0x55, 0x56],
        [0x45, 0x46, 0x44, 0x45],
        [0x55, 0x56, 0x54, 0x55],
    ];
    let mut t = Vec::new();
    for y in 0..12 {
        for x in 0..24 {
            let b = if y < 4 {
                0x2d
            } else if y < 8 {
                0x2c
            } else if y < 10 {
                if x < 4 {
                    0x2e
                } else if x >= 20 {
                    0x2f
                } else {
                    0x30
                }
            } else {
                4
            };
            let a = match y {
                0 => {
                    if x % 2 == 0 {
                        0x4e
                    } else {
                        0x4f
                    }
                }
                1 => 0x5e,
                2..=8 => 0x50,
                9 => {
                    if matches!(x, 2 | 3 | 20 | 21) {
                        0x5f
                    } else {
                        0x0f
                    }
                }
                _ => tatami[y % 4][x % 4],
            };
            t.push(tile(x, y, b, a));
        }
    }
    (t, g)
}
#[test]
fn theater_atomic_ownership_keeps_native_footing_and_floor_accounting() {
    let (t, g) = theater_fixture();
    let c: Vec<_> = t.iter().collect();
    let out = resolve("DanceTheater", &c, &g, [0, 0], None, &vec![false; t.len()]);
    assert_eq!(out.len(), 1);
    let mut mesh = TerrainMeshData::default();
    mesh.authored_cells = vec![None; t.len()];
    mesh.footing_heights = (0..t.len())
        .map(|i| if i / g.width < 10 { 8. } else { 0. })
        .collect();
    let footing = mesh.footing_heights.clone();
    let mut claimed = vec![false; t.len()];
    assert!(append(&mut mesh, &c, &g, &out[0], &mut claimed));
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 240);
    assert_eq!(
        mesh.authored_cells.iter().filter(|v| v.is_some()).count(),
        72
    );
    assert_eq!(mesh.footing_heights, footing);
    assert!(mesh.textured.positions.is_empty());
    assert!(mesh.solid.cutaway_ranges.is_empty());
    assert!(mesh.solid.positions.iter().all(|p| p[0] >= -96.001
        && p[0] <= 96.001
        && p[2] >= -48.001
        && p[2] <= 32.001
        && p[1] >= 0.
        && p[1] <= 24.001));
    // Actor-facing floor rows retain a single, exactly 8px visible top datum.
    assert!(
        mesh.solid
            .positions
            .iter()
            .filter(|p| p[2] > -32. && p[1] > 7.81)
            .all(|p| (p[1] - 8.).abs() < 0.0001)
    );
    let before = mesh.solid.positions.len();
    assert!(!append(&mut mesh, &c, &g, &out[0], &mut claimed));
    assert_eq!(mesh.solid.positions.len(), before);
    let mut stale = t.clone();
    stale[9 * 24 + 2].source.tile_index = 0x0f;
    let mut empty = TerrainMeshData::default();
    assert!(!append(
        &mut empty,
        &stale.iter().collect::<Vec<_>>(),
        &g,
        &out[0],
        &mut vec![false; t.len()]
    ));
    assert!(empty.solid.indices.is_empty());
}
#[test]
fn custom_stage_profile_has_first_refusal_even_with_same_native_art() {
    let (t, g) = theater_fixture();
    let c: Vec<_> = t.iter().collect();
    let custom:Document=serde_json::from_str(r#"{"objects":[{"name":"My stage screen","map":"DanceTheater","tileset":"traditional_house","metatile":45,"origin":[0,0],"tiles":[[78,79],[94,94]],"ground":69,"top_pixels":8,"depth_pixels":4}]}"#).unwrap();
    assert!(
        resolve(
            "DanceTheater",
            &c,
            &g,
            [0, 0],
            Some(&custom),
            &vec![false; t.len()]
        )
        .is_empty()
    );
}
