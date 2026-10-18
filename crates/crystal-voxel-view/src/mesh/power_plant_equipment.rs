//! Native Power Plant machinery: roof drawings above upright control faces.
//! Source collision and all walkable conduit strips remain authoritative.
use super::*;

fn drawing(block: u16) -> Option<[[u16; 4]; 4]> {
    Some(match block {
        0x12 => [
            [69, 69, 70, 69],
            [85, 85, 86, 85],
            [10, 11, 75, 10],
            [26, 27, 91, 26],
        ],
        0x16 => [
            [69, 70, 69, 59],
            [85, 86, 85, 59],
            [74, 75, 74, 59],
            [90, 91, 90, 59],
        ],
        _ => return None,
    })
}

fn resolve(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    customized: &[bool],
    reserved: &[Option<&'static str>],
) -> Vec<(usize, usize)> {
    if frame.map_id.as_ref() != "PowerPlant"
        || cells.len() != g.width * g.height
        || customized.len() != cells.len()
        || reserved.len() != cells.len()
    {
        return Vec::new();
    }
    let mut out = Vec::new();
    for y in 0..g.height.saturating_sub(3) {
        for x in 0..g.width.saturating_sub(3) {
            let s = &cells[y * g.width + x].source;
            let Some(art) = drawing(s.metatile_id) else {
                continue;
            };
            if s.subtile_column != 0 || s.subtile_row != 0 {
                continue;
            }
            if !(0..4).all(|dy| {
                (0..4).all(|dx| {
                    let i = (y + dy) * g.width + x + dx;
                    let t = &cells[i].source;
                    !customized[i]
                        && reserved[i].is_none()
                        && t.tileset_id.as_ref() == "facility"
                        && t.metatile_id == s.metatile_id
                        && t.subtile_column == dx as u8
                        && t.subtile_row == dy as u8
                        && t.tile_index == art[dy][dx]
                })
            }) {
                continue;
            }
            out.push((x, y));
        }
    }
    out
}

// Select an actual neutral midtone from the source control faces. White
// reflections, black outlines, blue floor and gold track accents are not metal
// field paint, even when they occupy the center of a source tile.
fn gray_field(samples: impl IntoIterator<Item = ([u8; 4], [f32; 2])>) -> Option<[f32; 2]> {
    let mut colors = std::collections::BTreeMap::new();
    for (rgba, uv) in samples {
        let low = *rgba[..3].iter().min().unwrap();
        let high = *rgba[..3].iter().max().unwrap();
        if rgba[3] != 255 || low < 96 || high > 208 || high - low > 12 {
            continue;
        }
        let entry = colors.entry(rgba).or_insert((0usize, uv));
        entry.0 += 1;
    }
    colors.into_values().max_by_key(|v| v.0).map(|v| v.1)
}
fn metal_field(
    cells: &[&VisualTile],
    g: &GridGeometry,
    x: usize,
    y: usize,
    images: Option<&TerrainImageSamples>,
) -> [f32; 2] {
    let fallback = g.uv(x, y + 2);
    let fallback = [
        fallback.0 + (fallback.1 - fallback.0) / 16.,
        fallback.2 + (fallback.3 - fallback.2) / 16.,
    ];
    let Some(images) = images else {
        return fallback;
    };
    let mut samples = Vec::new();
    for row in 2..4 {
        for col in 0..4 {
            let tile = cells[(y + row) * g.width + x + col];
            let Some(TileImageSample::Rgba(pixels)) = images.pixels.get(&tile.texture.id()) else {
                continue;
            };
            let uv = g.uv(x + col, y + row);
            for (i, p) in pixels.chunks_exact(4).enumerate() {
                samples.push((
                    [p[0], p[1], p[2], p[3]],
                    [
                        uv.0 + (uv.1 - uv.0) * (i % 8) as f32 / 8. + (uv.1 - uv.0) / 16.,
                        uv.2 + (uv.3 - uv.2) * (i / 8) as f32 / 8. + (uv.3 - uv.2) / 16.,
                    ],
                ));
            }
        }
    }
    gray_field(samples).unwrap_or(fallback)
}

/// A shallow chamfer catches light without changing the source footprint.
fn housing(out: &mut SurfaceMeshData, b: [f32; 6], bevel: f32, paint: [f32; 2]) {
    let [w, e, bottom, top, n, s] = b;
    let lower = [
        [w, bottom, n],
        [e, bottom, n],
        [e, bottom, s],
        [w, bottom, s],
    ];
    let rim = [
        [w, top - bevel, n],
        [e, top - bevel, n],
        [e, top - bevel, s],
        [w, top - bevel, s],
    ];
    let roof = [
        [w + bevel, top, n + bevel],
        [e - bevel, top, n + bevel],
        [e - bevel, top, s - bevel],
        [w + bevel, top, s - bevel],
    ];
    for i in 0..4 {
        let j = (i + 1) % 4;
        for p in [
            [lower[j], lower[i], rim[i], rim[j]],
            [rim[j], rim[i], roof[i], roof[j]],
        ] {
            let n = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
                .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
                .normalize()
                .to_array();
            append_quad(out, p, n, [paint; 4], TEXTURED_SHADE);
        }
    }
    append_quad(
        out,
        [lower[0], lower[1], lower[2], lower[3]],
        [0., -1., 0.],
        [paint; 4],
        TEXTURED_SHADE,
    );
    append_quad(
        out,
        [roof[0], roof[3], roof[2], roof[1]],
        [0., 1., 0.],
        [paint; 4],
        TEXTURED_SHADE,
    );
}

pub(in crate::mesh) fn finish(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
    images: Option<&TerrainImageSamples>,
) {
    if frame.map_id.as_ref() != "PowerPlant" {
        return;
    }
    let mut customized = vec![false; cells.len()];
    for p in live::resolve(cells, g.width, g.height, &frame.map_id, profiles) {
        for i in p.indices(g.width) {
            customized[i] = true;
        }
    }
    let plots = resolve(frame, cells, g, &customized, &mesh.authored_cells);
    if plots.is_empty() {
        return;
    }
    let mut owned = vec![false; cells.len()];
    for &(x, y) in &plots {
        for dy in 0..4 {
            for dx in 0..4 {
                owned[(y + dy) * g.width + x + dx] = true;
            }
        }
    }
    for surface in [
        &mut mesh.textured,
        &mut mesh.solid,
        &mut mesh.animated_textured,
        &mut mesh.animated_solid,
    ] {
        let old = std::mem::take(&mut surface.indices);
        for tri in old.chunks_exact(3) {
            let p = tri
                .iter()
                .map(|&i| Vec3::from_array(surface.positions[i as usize]))
                .sum::<Vec3>()
                / 3.;
            let x = ((p.x - g.origin_x) / g.tile_width).floor() as i32;
            let y = ((p.z - g.origin_z) / g.tile_height).floor() as i32;
            let fold = x >= 0
                && y >= 0
                && (x as usize) < g.width
                && (y as usize) < g.height
                && owned[y as usize * g.width + x as usize]
                && tri.iter().all(|&i| {
                    surface.positions[i as usize][1].abs() < 0.001
                        && surface.normals[i as usize][1] > 0.99
                });
            if !fold {
                surface.indices.extend_from_slice(tri);
            }
        }
    }
    let scale = g.tile_height / 8.;
    for (x, y) in plots {
        let out = &mut mesh.textured;
        let (w, _, n, _) = g.bounds(x, y);
        let e = w + 4. * g.tile_width;
        let s = n + 4. * g.tile_height;
        let paint = metal_field(cells, g, x, y, images);
        // The low machine bed fills the original blocked plot. Its service
        // ledge is machinery, not a newly walkable strip of ordinary floor.
        let first = out.positions.len();
        super::super::modeled_interiors::architecture_box(
            out,
            [w, e, 0., 4. * scale, n, s],
            [1.; 3],
        );
        out.uvs[first..].fill(paint);
        // The shallow bed is never a foreground occluder. Keep this closed
        // support outside the housing reveal, as with source cabinet floors.
        let start = out.positions.len();
        let front = n + 2. * g.tile_height;
        housing(
            out,
            [w, e, 4. * scale, 20. * scale, n, front],
            0.35 * scale,
            paint,
        );
        // Top two source rows are the roof, the lower two are upright controls.
        // A small inset keeps the live surfaces on the actual chamfered shell.
        let bevel = 0.35 * scale;
        for row in 0..2 {
            for col in 0..4 {
                let uv = g.uv(x + col, y + row);
                let west = w + bevel + (e - w - 2. * bevel) * col as f32 / 4.;
                let east = w + bevel + (e - w - 2. * bevel) * (col + 1) as f32 / 4.;
                let north = n + bevel + (front - n - 2. * bevel) * row as f32 / 2.;
                let south = n + bevel + (front - n - 2. * bevel) * (row + 1) as f32 / 2.;
                append_quad(
                    out,
                    [
                        [west, 20.002 * scale, north],
                        [west, 20.002 * scale, south],
                        [east, 20.002 * scale, south],
                        [east, 20.002 * scale, north],
                    ],
                    [0., 1., 0.],
                    [[uv.0, uv.2], [uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2]],
                    TEXTURED_SHADE,
                );
                let uv = g.uv(x + col, y + row + 2);
                let west = w + (e - w) * col as f32 / 4.;
                let east = w + (e - w) * (col + 1) as f32 / 4.;
                let top = (20. * scale - bevel) - (16. * scale - bevel) * row as f32 / 2.;
                let bottom = (20. * scale - bevel) - (16. * scale - bevel) * (row + 1) as f32 / 2.;
                append_quad(
                    out,
                    [
                        [west, bottom, front + 0.002 * scale],
                        [east, bottom, front + 0.002 * scale],
                        [east, top, front + 0.002 * scale],
                        [west, top, front + 0.002 * scale],
                    ],
                    [0., 0., 1.],
                    [[uv.0, uv.3], [uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2]],
                    TEXTURED_SHADE,
                );
            }
        }
        let range = start..out.positions.len();
        out.cutaway_ranges.push(range.clone());
        // Smooth alpha keeps the player readable without a pixel mask. The
        // layered policy retains depth writing for the closed body/panels,
        // while the bed and floor outside this exact range remain opaque.
        mesh.layered_alpha_textured_ranges.push(range);
        for dy in 0..4 {
            for dx in 0..4 {
                mesh.authored_cells[(y + dy) * g.width + x + dx] =
                    Some("facility:power-plant-machine-bed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn power_plant_equipment_field_uses_source_gray_and_rejects_white_ink_and_gold() {
        let mut samples = vec![([248, 248, 248, 255], [0., 0.]); 100];
        samples.extend(vec![([0, 0, 0, 255], [0., 0.]); 100]);
        samples.extend(vec![([80, 80, 80, 255], [0., 0.]); 100]);
        samples.extend(vec![([160, 128, 48, 255], [0., 0.]); 100]);
        samples.extend(vec![([168, 168, 168, 255], [0.25, 0.75]); 8]);
        samples.push(([120, 120, 120, 255], [0.5, 0.5]));
        assert_eq!(gray_field(samples), Some([0.25, 0.75]));
        assert_eq!(
            gray_field([
                ([248, 248, 248, 255], [0., 0.]),
                ([96, 96, 96, 0], [0., 0.])
            ]),
            None
        );
    }
    fn fixture() -> (VisualWorldFrame, GridGeometry, TerrainMeshData) {
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(
            std::path::PathBuf::from(std::env::var_os("CRYSTAL_RENDER_TEST_PACK").unwrap()),
        )
        .unwrap();
        let data = &loaded.pack().data().maps["PowerPlant"];
        let art = &loaded.pack().runtime_files()["data/tilesets/facility_metatiles.bin"];
        let (w, h) = (
            data.attributes.width as usize * 4,
            data.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let b = data.blocks[y / 4 * (w / 4) + x / 4];
                VisualTileSource {
                    tileset_id: "facility".into(),
                    metatile_id: b,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                }
            })
            .collect();
        let mut frame = super::super::super::tests::frame(w as u32, h as u32, tiles);
        frame.map_id = "PowerPlant".into();
        let g = GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![2.75; w * h],
            authored_cells: vec![None; w * h],
            ..Default::default()
        };
        for y in 0..h {
            for x in 0..w {
                append_top(&mut mesh.textured, g.bounds(x, y).into(), 0., g.uv(x, y));
            }
        }
        (frame, g, mesh)
    }
    #[test]
    fn external_power_plant_equipment_has_twelve_closed_beds_roofs_and_upright_controls() {
        let (frame, g, base) = fixture();
        let original = frame.clone();
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let plots = resolve(
            &frame,
            &cells,
            &g,
            &vec![false; cells.len()],
            &base.authored_cells,
        );
        assert_eq!(plots.len(), 12);
        let mut mesh = base.clone();
        finish(&mut mesh, &frame, &cells, &g, None, None);
        assert_eq!(frame, original);
        assert_eq!(mesh.footing_heights, base.footing_heights);
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|&&v| v == Some("facility:power-plant-machine-bed"))
                .count(),
            192
        );
        assert_eq!(mesh.textured.cutaway_ranges.len(), 12);
        assert_eq!(
            mesh.layered_alpha_textured_ranges, mesh.textured.cutaway_ranges,
            "all twelve closed units keep smooth alpha with depth ordering"
        );
        assert!(mesh.depth_coverage_textured_ranges.is_empty(),
            "machine housings must not stipple the player or companion");
        for ((x, y), range) in plots.iter().zip(&mesh.textured.cutaway_ranges) {
            assert!(
                mesh.textured.positions[range.clone()]
                    .iter()
                    .all(|p| p[1] >= 4.),
                "only the housing/panels reveal, never the bed below them"
            );
            let bed = range.start - 24..range.start;
            assert!(
                mesh.textured.positions[bed.clone()]
                    .iter()
                    .all(|p| p[1] >= 0. && p[1] <= 4.)
            );
            assert!(
                mesh.textured
                    .cutaway_ranges
                    .iter()
                    .all(|r| r.end <= bed.start || r.start >= bed.end),
                "the closed low bed remains opaque as foreground hardware fades"
            );
            let mut roof_area = 0.;
            let mut face_area = 0.;
            for tri in mesh
                .textured
                .indices
                .chunks_exact(3)
                .filter(|t| t.iter().all(|&i| range.contains(&(i as usize))))
            {
                let p = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
                let area = (p[1] - p[0]).cross(p[2] - p[0]);
                assert!(area.dot(n) > 0.);
                if p.iter().all(|p| (p.y - 20.002).abs() < 0.001) {
                    roof_area += area.length() / 2.;
                }
                if p.iter()
                    .all(|p| (p.z - ((*y + 2) as f32 * 8. + 0.002)).abs() < 0.001)
                {
                    face_area += area.length() / 2.;
                }
                for p in p {
                    assert!(p.x >= *x as f32 * 8. - 0.001 && p.x <= (*x + 4) as f32 * 8. + 0.001);
                    assert!(p.y >= 0. && p.y <= 20.003);
                }
            }
            assert!((roof_area - (32. - 0.7) * (16. - 0.7)).abs() < 0.01);
            assert!((face_area - 32. * (16. - 0.35)).abs() < 0.01);
        }
        for (i, t) in frame
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.source.metatile_id == 0x0b)
        {
            assert_eq!(
                mesh.authored_cells[i], None,
                "walkable red/white conduit is not machinery"
            );
            assert_eq!(mesh.footing_heights[i], base.footing_heights[i]);
            assert_eq!(t.source.tileset_id.as_ref(), "facility");
        }
    }
    #[test]
    fn external_power_plant_equipment_yields_whole_plots_to_edits_and_custom_ownership() {
        let (mut frame, g, mut base) = fixture();
        let original = frame.clone();
        let plots = resolve(
            &frame,
            &frame.tiles.iter().collect::<Vec<_>>(),
            &g,
            &vec![false; frame.tiles.len()],
            &base.authored_cells,
        );
        for &(x, y) in &plots {
            let i = y * g.width + x;
            frame.tiles[i].source.tile_index ^= 1;
            assert_eq!(
                resolve(
                    &frame,
                    &frame.tiles.iter().collect::<Vec<_>>(),
                    &g,
                    &vec![false; frame.tiles.len()],
                    &base.authored_cells
                )
                .len(),
                11
            );
            frame.tiles[i].source = original.tiles[i].source.clone();
            let mut custom = vec![false; frame.tiles.len()];
            custom[i] = true;
            assert_eq!(
                resolve(
                    &frame,
                    &frame.tiles.iter().collect::<Vec<_>>(),
                    &g,
                    &custom,
                    &base.authored_cells
                )
                .len(),
                11
            );
            base.authored_cells[i] = Some("custom");
            assert_eq!(
                resolve(
                    &frame,
                    &frame.tiles.iter().collect::<Vec<_>>(),
                    &g,
                    &vec![false; frame.tiles.len()],
                    &base.authored_cells
                )
                .len(),
                11
            );
            base.authored_cells[i] = None;
        }
        frame.map_id = "TeamRocketBaseB2F".into();
        let mut mesh = base.clone();
        finish(
            &mut mesh,
            &frame,
            &frame.tiles.iter().collect::<Vec<_>>(),
            &g,
            None,
            None,
        );
        assert_eq!(mesh, base);
    }
}
