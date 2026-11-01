//! Complete Radio Tower flights; support follows the selected native tread mesh.
use super::*;

// The selected flight was authored with a 2:1 run/rise. Fitting its full
// rail envelope to22.4px over a16px run doubled that slope and hid down
// treads at the normal45degree view. Keep the visible flight near its authored
// proportions; other rooms retain their independently fitted staircase sizes.
const FLIGHT_HEIGHT: f32 = 12.;
const TREAD_SCALE: f32 = FLIGHT_HEIGHT / 22.4;
const DOWN_BASE: f32 = -16. * TREAD_SCALE;

fn room(map: &str) -> bool {
    matches!(
        map,
        "RadioTower1F" | "RadioTower2F" | "RadioTower3F" | "RadioTower4F" | "RadioTower5F"
    )
}
fn local(s: &VisualTileSource) -> Option<(u8, u8, bool)> {
    if s.tileset_id.as_ref() != "radio_tower" || s.subtile_row >= 2 {
        return None;
    }
    let (start, down) = match s.metatile_id {
        3 => (2, false),
        0x0c => (0, false),
        0x0d => (2, true),
        0x1d => (0, true),
        _ => return None,
    };
    let x = s.subtile_column.checked_sub(start)?;
    if x >= 2 {
        return None;
    }
    let art = if down {
        [[0x0e, 0x0f], [0x1e, 0x1f]]
    } else {
        [[0x0c, 0x0d], [0x1c, 0x1d]]
    };
    (s.tile_index == art[s.subtile_row as usize][x as usize]).then_some((x, s.subtile_row, down))
}
pub(super) fn stair(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<bool> {
    if !room(map)
        || p.kind != ModelKind::StairFlight
        || p.width != 2
        || p.height != 2
        || p.column + 2 > g.width
        || p.row + 2 > g.height
        || cells.len() != g.width * g.height
    {
        return None;
    }
    let (_, _, down) = local(&cells[p.row * g.width + p.column].source)?;
    (p.base_pixels == if down { DOWN_BASE } else { 0. }
        && p.depth_pixels == 16.
        && p.height_pixels == FLIGHT_HEIGHT
        && p.indices(g.width)
            .enumerate()
            .all(|(j, i)| local(&cells[i].source) == Some(((j % 2) as u8, (j / 2) as u8, down))))
    .then_some(down)
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if !room(map) {
        return;
    }
    let Some(ground) = ground_sample(cells, map, "radio_tower", 1) else {
        return;
    };
    for group in grouped_flat_card_placements(cells, g, 1, false, |s| {
        local(s).map(|(x, y, _)| (x, y, 2, 2))
    }) {
        let down = local(&cells[group.row * g.width + group.column].source)
            .unwrap()
            .2;
        let p = Placement {
            kind: ModelKind::StairFlight,
            column: group.column,
            row: group.row,
            width: 2,
            height: 2,
            ground,
            depth_pixels: 16.,
            height_pixels: FLIGHT_HEIGHT,
            base_pixels: if down { DOWN_BASE } else { 0. },
            footing_pixels: None,
        };
        if stair(map, &p, cells, g).is_none() || p.indices(g.width).any(|i| claimed[i]) {
            continue;
        }
        for i in p.indices(g.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
}
fn pixels(down: bool) -> [[usize; 2]; 5] {
    if down {
        [[5, 4], [5, 4], [2, 13], [2, 10], [2, 10]]
    } else {
        [[2, 1], [2, 1], [7, 5], [2, 4], [2, 8]]
    }
}
fn treads() -> Option<&'static SurfaceMeshData> {
    static TREADS: OnceLock<Option<SurfaceMeshData>> = OnceLock::new();
    TREADS
        .get_or_init(|| {
            crate::interior_models::selected_model(ModelKind::StairFlight)?.native_stair_treads()
        })
        .as_ref()
}
fn sample(down: bool, x: f32, z: f32) -> Option<f32> {
    let mesh = treads()?;
    let q = if down {
        [z * 16., (1. - x) * 16.]
    } else {
        [(1. - z) * 16., x * 16.]
    };
    mesh.indices
        .chunks_exact(3)
        .filter_map(|tri| {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| mesh.positions[i as usize]);
            let v = [b[0] - a[0], b[2] - a[2]];
            let w = [c[0] - a[0], c[2] - a[2]];
            let d = [q[0] - a[0], q[1] - a[2]];
            let det = v[0] * w[1] - v[1] * w[0];
            if det.abs() < 0.000001 {
                return None;
            }
            let u = (d[0] * w[1] - d[1] * w[0]) / det;
            let v = (v[0] * d[1] - v[1] * d[0]) / det;
            (u >= -0.00001 && v >= -0.00001 && u + v <= 1.00001)
                .then_some(a[1] + u * (b[1] - a[1]) + v * (c[1] - a[1]))
        })
        .max_by(f32::total_cmp)
        .map(|y| y * TREAD_SCALE + if down { DOWN_BASE } else { 0. })
}
fn center_height(down: bool, j: usize) -> Option<f32> {
    sample(
        down,
        ((j % 2) as f32 + 0.5) / 2.,
        ((j / 2) as f32 + 0.5) / 2.,
    )
}
pub(super) fn well_uv(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<(f32, f32, f32, f32)> {
    if stair(map, p, cells, g) != Some(true) {
        return None;
    }
    let uv = g.uv(p.column, p.row);
    let point = (uv.0 + (uv.1 - uv.0) / 16., uv.2 + (uv.3 - uv.2) / 16.);
    Some((point.0, point.0, point.1, point.1))
}
pub(super) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    [w, e, n, s]: [f32; 4],
    selected: &crate::interior_models::Model,
) {
    let down = stair(map, p, cells, g).expect("complete Radio flight");
    let out = &mut mesh.textured;
    let first = out.positions.len();
    let bounds = if down { [n, s, -e, -w] } else { [-s, -n, w, e] };
    assert!(selected.append_native_stair(
        out,
        bounds,
        p.base_pixels * g.tile_height / 8.,
        FLIGHT_HEIGHT * g.tile_height / 8.,
        native_source_swatches(p, g, pixels(down))
    ));
    let rotate = |v: [f32; 3]| {
        if down {
            [-v[2], v[1], v[0]]
        } else {
            [v[2], v[1], -v[0]]
        }
    };
    for point in &mut out.positions[first..] {
        *point = rotate(*point);
    }
    for (normal, color) in out.normals[first..]
        .iter_mut()
        .zip(&mut out.colors[first..])
    {
        *normal = rotate(*normal);
        *color = crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal));
    }
    if mesh.footing_heights.len() == g.width * g.height {
        for (j, i) in p.indices(g.width).enumerate() {
            if let Some(y) = center_height(down, j) {
                mesh.footing_heights[i] = y * g.tile_height / 8.;
            }
        }
    }
}
/// Correct the coarse cell sample only when all four support values prove that
/// this complete native flight was rendered. Custom/source fallback stays flat.
pub(super) fn support_correction(
    frame: &VisualWorldFrame,
    cell: bevy::prelude::UVec2,
    foot: bevy::prelude::Vec2,
    resolved: &[f32],
) -> f32 {
    if !room(&frame.map_id) {
        return 0.;
    }
    let g = GridGeometry {
        width: frame.grid_size.x as usize,
        height: frame.grid_size.y as usize,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: 0.,
        origin_z: 0.,
    };
    if cell.x as usize >= g.width
        || cell.y as usize >= g.height
        || g.tile_width <= 0.
        || g.tile_height <= 0.
    {
        return 0.;
    }
    let i = cell.y as usize * g.width + cell.x as usize;
    // Most actor queries are ordinary floor. Avoid constructing a dense source
    // view until the queried cell can actually belong to a native flight.
    let tile = frame
        .tiles
        .get(i)
        .filter(|t| t.column == cell.x && t.row == cell.y)
        .or_else(|| {
            frame
                .tiles
                .iter()
                .find(|t| t.column == cell.x && t.row == cell.y)
        });
    let Some((x, y, down)) = tile.and_then(|t| local(&t.source)) else {
        return 0.;
    };
    let mut order = vec![None; g.width * g.height];
    for tile in &frame.tiles {
        if tile.column as usize >= g.width || tile.row as usize >= g.height {
            return 0.;
        }
        let i = tile.row as usize * g.width + tile.column as usize;
        if order[i].is_some() {
            return 0.;
        }
        order[i] = Some(tile);
    }
    let Some(cells) = order.into_iter().collect::<Option<Vec<_>>>() else {
        return 0.;
    };
    let (Some(col), Some(row)) = (
        (cell.x as usize).checked_sub(x as usize),
        (cell.y as usize).checked_sub(y as usize),
    ) else {
        return 0.;
    };
    if col + 2 > g.width || row + 2 > g.height {
        return 0.;
    }
    let scale = g.tile_height / 8.;
    for j in 0..4 {
        let k = (row + j / 2) * g.width + col + j % 2;
        if local(&cells[k].source) != Some(((j % 2) as u8, (j / 2) as u8, down)) {
            return 0.;
        }
        let Some(expected) = center_height(down, j) else {
            return 0.;
        };
        if resolved
            .get(k)
            .is_none_or(|h| !h.is_finite() || (*h - expected * scale).abs() > 0.001)
        {
            return 0.;
        }
    }
    let size = frame.grid_size.as_vec2() * frame.tile_size;
    let x = (foot.x - frame.center.x + size.x * 0.5) / g.tile_width;
    let z = (frame.center.y + size.y * 0.5 - foot.y) / g.tile_height;
    sample(
        down,
        ((x - col as f32) / 2.).clamp(0., 1.),
        ((z - row as f32) / 2.).clamp(0., 1.),
    )
    .map_or(0., |h| h * scale - resolved[i])
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn have_pack() -> bool {
        let explicit = std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path = explicit
            .as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        assert!(
            explicit.is_none() || path.is_file(),
            "missing explicit test pack"
        );
        path.is_file()
    }
    pub(super) fn packed(map: &str) -> (Vec<VisualTile>, GridGeometry) {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let module = &pack.pack().data().maps[map];
        let layout = &pack.pack().runtime_files()["data/tilesets/radio_tower_metatiles.bin"];
        let (w, h) = (
            module.attributes.width as usize * 4,
            module.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let x = i % w;
                let y = i / w;
                let block = module.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "radio_tower".into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: layout[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
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
                origin_x: 3.,
                origin_z: -7.,
            },
        )
    }
    #[test]
    fn radio_stair_all_twelve_complete_flights_have_native_geometry_and_owned_support() {
        if !have_pack() {
            return;
        }
        for (map, count) in [
            ("RadioTower1F", 1),
            ("RadioTower2F", 2),
            ("RadioTower3F", 3),
            ("RadioTower4F", 4),
            ("RadioTower5F", 2),
        ] {
            let (mut tiles, mut g) = packed(map);
            for (tw, th) in [(8., 8.), (5., 9.), (11., 6.)] {
                g.tile_width = tw;
                g.tile_height = th;
                let cells = tiles.iter().collect::<Vec<_>>();
                let p = resolve(map, &cells, &g, None);
                let flights = p
                    .iter()
                    .filter(|p| stair(map, p, &cells, &g).is_some())
                    .collect::<Vec<_>>();
                assert_eq!(flights.len(), count, "{map}");
                for p in flights {
                    let down = stair(map, p, &cells, &g).unwrap();
                    assert_eq!(p.row, 0, "each flight must match a rear staircase warp");
                    assert_eq!(p.indices(g.width).count(), 4);
                    let mut mesh = TerrainMeshData {
                        footing_heights: vec![31.; g.width * g.height],
                        ..Default::default()
                    };
                    let mut claimed = vec![false; g.width * g.height];
                    assert!(super::super::append(
                        &mut mesh,
                        map,
                        &cells,
                        &g,
                        p,
                        &mut claimed,
                        [0, 0]
                    ));
                    assert_eq!(claimed.iter().filter(|&&b| b).count(), 4);
                    for (i, h) in mesh.footing_heights.iter().enumerate() {
                        if !p.indices(g.width).any(|j| j == i) {
                            assert_eq!(*h, 31.);
                        }
                    }
                    for (j, i) in p.indices(g.width).enumerate() {
                        assert_eq!(
                            mesh.footing_heights[i],
                            center_height(down, j).unwrap() * th / 8.
                        );
                    }
                    let [w, e, n, s] = [
                        g.origin_x + p.column as f32 * tw,
                        g.origin_x + (p.column + 2) as f32 * tw,
                        g.origin_z,
                        g.origin_z + 2. * th,
                    ];
                    for v in &mesh.textured.positions {
                        assert!(
                            v[0] >= w - 0.001
                                && v[0] <= e + 0.001
                                && v[2] >= n - 0.001
                                && v[2] <= s + 0.001
                        );
                        assert!(
                            v[1] >= p.base_pixels * th / 8. - 0.001
                                && v[1] <= (p.base_pixels + FLIGHT_HEIGHT) * th / 8. + 0.001
                        );
                    }
                    assert!(
                        mesh.textured
                            .normals
                            .iter()
                            .all(|v| (Vec3::from_array(*v).length() - 1.).abs() < 0.001)
                    );
                    assert!(stair("LavRadioTower1F", p, &cells, &g).is_none());
                    let mut custom = vec![false; g.width * g.height];
                    custom[p.row * g.width + p.column] = true;
                    let mut out = Vec::new();
                    candidates(map, &cells, &g, &mut custom, &mut out);
                    assert!(!out.iter().any(|q| q.column == p.column && q.row == p.row));
                }
            }
            let cells = tiles.iter().collect::<Vec<_>>();
            let p = resolve(map, &cells, &g, None);
            let flights = p
                .into_iter()
                .filter(|p| stair(map, p, &cells, &g).is_some())
                .collect::<Vec<_>>();
            for p in flights {
                for i in p.indices(g.width) {
                    tiles[i].source.tile_index ^= 1;
                    let refs = tiles.iter().collect::<Vec<_>>();
                    assert!(stair(map, &p, &refs, &g).is_none());
                    assert!(
                        !resolve(map, &refs, &g, None)
                            .iter()
                            .any(|q| q.column == p.column
                                && q.row == p.row
                                && q.kind == ModelKind::StairFlight)
                    );
                    tiles[i].source.tile_index ^= 1;
                }
            }
        }
        let (tiles, g) = packed("LavRadioTower1F");
        assert!(
            !resolve(
                "LavRadioTower1F",
                &tiles.iter().collect::<Vec<_>>(),
                &g,
                None
            )
            .iter()
            .any(|p| p.kind == ModelKind::StairFlight)
        );
    }
    #[test]
    fn radio_stair_rendered_pitch_keeps_tread_width_at_the_normal_camera_angle() {
        if !have_pack() {
            return;
        }
        let (tiles, g) = packed("RadioTower2F");
        let cells = tiles.iter().collect::<Vec<_>>();
        let flights = resolve("RadioTower2F", &cells, &g, None);
        for p in flights
            .iter()
            .filter(|p| stair("RadioTower2F", p, &cells, &g).is_some())
        {
            let selected = crate::interior_models::selected_model(ModelKind::StairFlight).unwrap();
            let mut mesh = TerrainMeshData::default();
            append(
                &mut mesh,
                "RadioTower2F",
                p,
                &cells,
                &g,
                [0., 16., 0., 16.],
                selected,
            );
            let stone = native_source_swatches(
                p,
                &g,
                pixels(stair("RadioTower2F", p, &cells, &g).unwrap()),
            )[2];
            let mut levels = mesh
                .textured
                .positions
                .iter()
                .zip(&mesh.textured.normals)
                .zip(&mesh.textured.uvs)
                .filter(|((_, n), uv)| n[1] > 0.9999 && **uv == stone)
                .map(|((v, _), _)| (v[1] * 1000.).round() as i32)
                .collect::<Vec<_>>();
            levels.sort();
            levels.dedup();
            assert_eq!(levels.len(), 8, "eight separate flat treads");
            for pair in levels.windows(2) {
                let rise = (pair[1] - pair[0]) as f32 / 1000.;
                assert!(
                    (0.9..1.1).contains(&rise),
                    "native2px run needs a roughly1px rise: {rise}"
                );
                let pitch = crate::camera::CAMERA_PITCH_DEGREES.to_radians();
                let exposed = 2. * pitch.sin() - rise * pitch.cos();
                assert!(
                    exposed > 0.6,
                    "nearer tread must leave a visible strip at normal camera pitch"
                );
            }
        }
    }
    #[test]
    fn radio_stair_source_paint_and_warp_endpoints_match_the_external_pack() {
        if !have_pack() {
            return;
        }
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let png = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/radio_tower.png"])
            .unwrap()
            .to_luma8();
        let palette = &pack.data().tilesets["radio_tower"].palette_map;
        let mut total = 0;
        for map in [
            "RadioTower1F",
            "RadioTower2F",
            "RadioTower3F",
            "RadioTower4F",
            "RadioTower5F",
        ] {
            let (tiles, g) = packed(map);
            let cells = tiles.iter().collect::<Vec<_>>();
            let p = resolve(map, &cells, &g, None);
            let flights = p
                .iter()
                .filter(|p| stair(map, p, &cells, &g).is_some())
                .collect::<Vec<_>>();
            for warp in &pack.data().maps[map].events.warps {
                let i = warp.y as usize * 2 * g.width + warp.x as usize * 2;
                let is_stair = local(&tiles[i].source).is_some();
                assert_eq!(
                    flights.iter().any(|p| p.indices(g.width).any(|j| j == i)),
                    is_stair,
                    "{map}: exits stay separate"
                );
                total += usize::from(is_stair);
            }
            for p in flights {
                let down = stair(map, p, &cells, &g).unwrap();
                for (pixel, value) in pixels(down).into_iter().zip([85, 85, 255, 170, 170]) {
                    let raw = tiles[(p.row + pixel[1] / 8) * g.width + p.column + pixel[0] / 8]
                        .source
                        .tile_index as usize;
                    assert_eq!(palette[raw], 0);
                    assert_eq!(
                        png.get_pixel(
                            (raw % 16 * 8 + pixel[0] % 8) as u32,
                            (raw / 16 * 8 + pixel[1] % 8) as u32
                        )
                        .0[0],
                        value
                    );
                }
                // Black at the bottom of the descending well, never pink floor.
                assert_eq!(well_uv(map, p, &cells, &g).is_some(), down);
                let mut cropped = tiles.clone();
                cropped.remove(p.row * g.width + p.column);
                assert!(stair(map, p, &cropped.iter().collect::<Vec<_>>(), &g).is_none());
            }
        }
        assert_eq!(total, 12);
    }
    #[test]
    fn radio_stair_contact_matches_actual_rendered_tops_and_rejects_fallback_support() {
        if !have_pack() {
            return;
        }
        use bevy::prelude::{IVec2, UVec2, Vec2};
        let (tiles, mut g) = packed("RadioTower2F");
        for (tw, th) in [(8., 8.), (5., 9.), (11., 6.)] {
            g.tile_width = tw;
            g.tile_height = th;
            let cells = tiles.iter().collect::<Vec<_>>();
            let p = resolve("RadioTower2F", &cells, &g, None);
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.; g.width * g.height],
                ..Default::default()
            };
            let mut claimed = vec![false; g.width * g.height];
            for p in p
                .iter()
                .filter(|p| stair("RadioTower2F", p, &cells, &g).is_some())
            {
                assert!(super::super::append(
                    &mut mesh,
                    "RadioTower2F",
                    &cells,
                    &g,
                    p,
                    &mut claimed,
                    [0, 0]
                ));
            }
            let mut frame = VisualWorldFrame {
                map_id: "RadioTower2F".into(),
                tiles: tiles.clone(),
                grid_size: UVec2::new(g.width as u32, g.height as u32),
                tile_size: Vec2::new(tw, th),
                center: Vec2::new(17., -9.),
                grid_origin: IVec2::new(-4, -6),
                ..Default::default()
            };
            for p in p
                .iter()
                .filter(|p| stair("RadioTower2F", p, &cells, &g).is_some())
            {
                let down = stair("RadioTower2F", p, &cells, &g).unwrap();
                for z in [0.03, 0.13, 0.27, 0.39, 0.51, 0.63, 0.77, 0.89, 1.] {
                    let gx = p.column as f32 + 1.;
                    let gy = p.row as f32 + 2. * z;
                    let foot = frame.center
                        + Vec2::new(
                            (gx - g.width as f32 * 0.5) * tw,
                            (g.height as f32 * 0.5 - gy) * th,
                        );
                    let expected = sample(down, 0.5, z).unwrap() * th / 8.;
                    let height = crate::footing::resolved_footing_height(
                        &frame,
                        foot,
                        &mesh.footing_heights,
                    )
                    .unwrap();
                    assert!(
                        (height - expected).abs() < 0.001,
                        "tread contact {down} {z}: {height} vs {expected}"
                    );
                    // Independently intersect the actual rendered triangle at this world point.
                    let q = [g.origin_x + gx * tw, g.origin_z + gy * th];
                    let rendered = mesh
                        .textured
                        .indices
                        .chunks_exact(3)
                        .filter_map(|tri| {
                            let [a, b, c] = [tri[0], tri[1], tri[2]]
                                .map(|i| mesh.textured.positions[i as usize]);
                            if tri
                                .iter()
                                .any(|&i| mesh.textured.normals[i as usize][1] <= 0.01)
                            {
                                return None;
                            }
                            let v = [b[0] - a[0], b[2] - a[2]];
                            let w = [c[0] - a[0], c[2] - a[2]];
                            let d = [q[0] - a[0], q[1] - a[2]];
                            let det = v[0] * w[1] - v[1] * w[0];
                            if det.abs() < 0.000001 {
                                return None;
                            }
                            let u = (d[0] * w[1] - d[1] * w[0]) / det;
                            let v = (v[0] * d[1] - v[1] * d[0]) / det;
                            let y = a[1] + u * (b[1] - a[1]) + v * (c[1] - a[1]);
                            (u >= -0.00001
                                && v >= -0.00001
                                && u + v <= 1.00001
                                && (y - expected).abs() < 0.001)
                                .then_some(y)
                        })
                        .next();
                    assert!(
                        rendered.is_some(),
                        "support must lie on a rendered horizontal face"
                    );
                    assert_eq!(
                        crate::footing::resolved_footing_height(
                            &frame,
                            foot,
                            &vec![0.; g.width * g.height]
                        ),
                        Some(0.)
                    );
                }
            }
            frame.tiles.reverse();
            let p = p
                .iter()
                .find(|p| stair("RadioTower2F", p, &cells, &g).is_some())
                .unwrap();
            let cell = UVec2::new(p.column as u32, 0);
            let foot = frame.center
                + Vec2::new(
                    (p.column as f32 + 0.5 - g.width as f32 * 0.5) * tw,
                    (g.height as f32 * 0.5 - 0.5) * th,
                );
            assert_eq!(
                support_correction(&frame, cell, foot, &mesh.footing_heights),
                0.,
                "cell center survives dense reordering"
            );
            // An altered cell or a different owner must disable the point lift.
            let original = frame.tiles.clone();
            for j in 0..4 {
                let x = p.column + j % 2;
                let y = p.row + j / 2;
                let tile = frame
                    .tiles
                    .iter_mut()
                    .find(|t| t.column == x as u32 && t.row == y as u32)
                    .unwrap();
                tile.source.tile_index ^= 1;
                assert_eq!(
                    support_correction(&frame, cell, foot, &mesh.footing_heights),
                    0.
                );
                frame.tiles = original.clone();
                let mut heights = mesh.footing_heights.clone();
                heights[y * g.width + x] += 1.;
                assert_eq!(support_correction(&frame, cell, foot, &heights), 0.);
            }
            frame.map_id = "LavRadioTower1F".into();
            assert_eq!(
                support_correction(&frame, cell, foot, &mesh.footing_heights),
                0.
            );
            frame.map_id = "RadioTower2F".into();
            frame.tiles.pop();
            assert_eq!(
                support_correction(&frame, cell, foot, &mesh.footing_heights),
                0.
            );
        }
    }
}

#[cfg(test)]
mod scene_tests {
    use super::tests::{have_pack, packed};
    use super::*;
    use bevy::prelude::{UVec2, Vec2};
    #[test]
    fn radio_stair_full_scene_keeps_descending_treads_clear_of_floor_overlays() {
        if !have_pack() {
            return;
        }
        let (mut tiles, g) = packed("RadioTower2F");
        let mut images = Assets::<Image>::default();
        let mut image = Image::default();
        image.texture_descriptor.size.width = 8;
        image.texture_descriptor.size.height = 8;
        image.data = Some(vec![255; 8 * 8 * 4]);
        let texture = images.add(image);
        for tile in &mut tiles {
            tile.texture = texture.clone();
        }
        let size = Vec2::new(g.width as f32 * 8., g.height as f32 * 8.);
        let frame = VisualWorldFrame {
            active: true,
            map_id: "RadioTower2F".into(),
            grid_size: UVec2::new(g.width as u32, g.height as u32),
            tile_size: Vec2::splat(8.),
            viewport_size: size,
            map_texture: texture,
            tiles,
            ..Default::default()
        };
        let mesh = crate::mesh::audit_terrain_mesh_with_images(&frame, &images).unwrap();
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        for p in resolve("RadioTower2F", &cells, &g, None)
            .iter()
            .filter(|p| stair("RadioTower2F", p, &cells, &g) == Some(true))
        {
            for step in 0..8 {
                let z = (step as f32 + 0.35) / 8.;
                let q = Vec2::new(
                    -size.x * 0.5 + (p.column as f32 + 1.) * 8.,
                    -size.y * 0.5 + z * 16.,
                );
                let expected = sample(true, 0.5, z).unwrap();
                let highest = [&mesh.textured, &mesh.solid]
                    .into_iter()
                    .flat_map(|surface| {
                        surface.indices.chunks_exact(3).filter_map(move |tri| {
                            let [a, b, c] = [tri[0], tri[1], tri[2]]
                                .map(|i| Vec3::from_array(surface.positions[i as usize]));
                            if surface.normals[tri[0] as usize][1] <= 0.01 {
                                return None;
                            }
                            let area = (b - a).cross(c - a).y;
                            if area.abs() < 0.000001 {
                                return None;
                            }
                            // Signed subtriangle areas give an independent vertical intersection.
                            let point = Vec3::new(q.x, 0., q.y);
                            let weights = [
                                (b - point).cross(c - point).y,
                                (c - point).cross(a - point).y,
                                (a - point).cross(b - point).y,
                            ]
                            .map(|v| v / area);
                            weights
                                .iter()
                                .all(|&v| v >= -0.00001 && v <= 1.00001)
                                .then_some(weights[0] * a.y + weights[1] * b.y + weights[2] * c.y)
                        })
                    })
                    .max_by(f32::total_cmp)
                    .unwrap();
                assert!(
                    (highest - expected).abs() < 0.001,
                    "tread {step}: an opaque surface at {highest} covers {expected}"
                );
            }
        }
    }
}
