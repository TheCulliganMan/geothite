//! Preserve each laboratory's real wall band and the two different pane drawings.
use super::*;

pub(super) fn section_template(
    template: &SurfaceMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placements: &[Placement],
    width: f32,
) -> Option<SurfaceMeshData> {
    let donor = placements
        .iter()
        .find(|p| p.kind == ModelKind::WallClinical && source(map, p, cells, g))?;
    let mut clipped = template.clone();
    let start = g.uv(donor.column, donor.row).0;
    let fraction = width / (2. * g.tile_width);
    // Apertures and final corner runs may be narrower than a whole section.
    // Its two adjacent donor columns have the same plain/checker fields, so
    // crop that continuous source interval rather than squeezing every check.
    for uv in &mut clipped.uvs {
        uv[0] = start + (uv[0] - start) * fraction;
    }
    Some(clipped)
}

pub(super) fn perimeter_template(
    out: &mut SurfaceMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placements: &[Placement],
) -> Option<usize> {
    let donor = placements
        .iter()
        .find(|p| p.kind == ModelKind::WallClinical && source(map, p, cells, g))?;
    let w = g.origin_x + donor.column as f32 * g.tile_width;
    let s = g.origin_z + (donor.row + 2) as f32 * g.tile_height;
    // Ordinary perimeter sections are two source columns wide. Supply exactly
    // that width so copy_section never halves the real checker texel pitch.
    append(
        out,
        g,
        donor,
        [w, w + 2. * g.tile_width, s - 3. * g.tile_height / 8., s],
    );
    Some(donor.row)
}

pub(super) fn source(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if !matches!(map, "OaksLab" | "ElmsLab")
        || p.height != 2
        || p.base_pixels != 0.
        || p.depth_pixels != 3.
        || p.height_pixels != 32.
        || g.width.checked_mul(g.height) != Some(cells.len())
        || p.width > g.width
        || p.height > g.height
        || p.column > g.width - p.width
        || p.row > g.height - p.height
    {
        return false;
    }
    let first = &cells[p.row * g.width + p.column].source;
    let panel = p.kind == ModelKind::WallClinical
        && p.width == 4
        && matches!(first.metatile_id, 8 | 13 | 23)
        && first.subtile_column == 0;
    let window = p.kind == ModelKind::WindowWall
        && matches!(p.width, 2 | 4)
        && matches!(first.metatile_id, 9 | 32)
        && (first.subtile_column == 0 || p.width == 2 && first.subtile_column == 2);
    (panel || window)
        && p.indices(g.width).enumerate().all(|(j, i)| {
            let s = &cells[i].source;
            let x = first.subtile_column as usize + j % p.width;
            let y = j / p.width;
            let tile = if panel {
                [1, 17][y]
            } else if first.metatile_id == 9 {
                [[32, 33, 34, 35], [48, 49, 50, 51]][y][x]
            } else {
                [[32, 33, 32, 33], [48, 49, 48, 49]][y][x]
            };
            s.tileset_id.as_ref() == "lab"
                && s.metatile_id == first.metatile_id
                && s.subtile_column as usize == x
                && s.subtile_row as usize == y
                && s.tile_index == tile
        })
}

pub(super) fn append(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement, b: [f32; 4]) {
    let [w, e, n, s] = b;
    let scale = g.tile_height / 8.;
    let height = 32. * scale;
    let cream = native_source_swatches(p, g, [[0, 0]])[0];
    let first = out.positions.len();
    let index = out.indices.len();
    architecture_box(out, [w, e, 0., height, n, s], [1.; 3]);
    out.uvs[first..].fill(cream);
    // Replace the front rather than overlaying another coplanar drawing.
    out.indices.drain(index + 6..index + 12);
    let shade = crate::interior_models::authored_face_color([1.; 4], Vec3::Z);
    append_quad(
        out,
        [
            [e, 0., s],
            [e, 16. * scale, s],
            [w, 16. * scale, s],
            [w, 0., s],
        ],
        Vec3::Z.to_array(),
        [cream; 4],
        shade,
    );
    if p.kind == ModelKind::WallClinical {
        let columns = ((e - w) / g.tile_width).round().max(1.) as usize;
        for y in 0..2 {
            for x in 0..columns {
                let uv = g.uv(p.column + x % p.width, p.row + y);
                let a = w + (e - w) * x as f32 / columns as f32;
                let b = w + (e - w) * (x + 1) as f32 / columns as f32;
                let top = height - y as f32 * g.tile_height;
                append_quad(
                    out,
                    [
                        [b, top - g.tile_height, s],
                        [b, top, s],
                        [a, top, s],
                        [a, top - g.tile_height, s],
                    ],
                    Vec3::Z.to_array(),
                    [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                    shade,
                );
            }
        }
        return;
    }
    // A shallow frame has closed edges; its front retains the complete source
    // reflection instead of substituting aqua glass and a generic crossbar.
    let sx = (e - w) / (p.width as f32 * 8.);
    for unit in 0..p.width / 2 {
        let left = w + unit as f32 * 16. * sx;
        let right = left + 16. * sx;
        let front = s + 0.4 * scale;
        let edge = native_source_swatches(p, g, [[unit * 16 + 1, 1]])[0];
        let a = left + sx;
        let b = right - sx;
        let low = 22. * scale;
        let high = 31. * scale;
        for (points, normal) in [
            (
                [[a, low, s], [a, low, front], [a, high, front], [a, high, s]],
                Vec3::NEG_X,
            ),
            (
                [[b, low, front], [b, low, s], [b, high, s], [b, high, front]],
                Vec3::X,
            ),
            (
                [
                    [a, high, front],
                    [b, high, front],
                    [b, high, s],
                    [a, high, s],
                ],
                Vec3::Y,
            ),
            (
                [[a, low, s], [b, low, s], [b, low, front], [a, low, front]],
                Vec3::NEG_Y,
            ),
        ] {
            append_quad(
                out,
                points,
                normal.to_array(),
                [edge; 4],
                crate::interior_models::authored_face_color([1.; 4], normal),
            );
        }
        let xs = [0., 1., 8., 15., 16.];
        let ys = [0., 1., 8., 10., 16.];
        for pair_y in ys.windows(2) {
            for pair_x in xs.windows(2) {
                let [x0, x1] = [pair_x[0], pair_x[1]];
                let [y0, y1] = [pair_y[0], pair_y[1]];
                let cx = ((x0 + x1) * 0.5 / 8.) as usize;
                let cy = ((y0 + y1) * 0.5 / 8.) as usize;
                let uv = g.uv(p.column + unit * 2 + cx, p.row + cy);
                let u = |x| uv.0 + (uv.1 - uv.0) * (x - cx as f32 * 8.) / 8.;
                let v = |y| uv.2 + (uv.3 - uv.2) * (y - cy as f32 * 8.) / 8.;
                let z = if x0 >= 1. && x1 <= 15. && y0 >= 1. && y1 <= 10. {
                    front
                } else {
                    s
                };
                append_quad(
                    out,
                    [
                        [left + x1 * sx, height - y1 * scale, z],
                        [left + x1 * sx, height - y0 * scale, z],
                        [left + x0 * sx, height - y0 * scale, z],
                        [left + x0 * sx, height - y1 * scale, z],
                    ],
                    Vec3::Z.to_array(),
                    [
                        [u(x1), v(y1)],
                        [u(x1), v(y0)],
                        [u(x0), v(y0)],
                        [u(x0), v(y1)],
                    ],
                    shade,
                );
            }
        }
    }
}
