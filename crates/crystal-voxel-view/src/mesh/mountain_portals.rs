//! Complete native mountain entrance drawings, with an open recessed mouth.
//! The source still owns footing and warps; this only replaces their artwork.
use super::*;

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    origin: [i32; 2],
) {
    for (i, cell) in cells.iter().enumerate() {
        let s = &cell.source;
        let (metatile, column, art, label) = match s.tileset_id.as_ref() {
            "johto" => (0x73, 0, [[0x46, 0x47], [0x56, 0x57]], "johto/mountain_cave"),
            "johto_modern" => (0x73, 2, [[0x45, 0x22], [0x27, 0x28]], "johto/mountain_cave"),
            // The tall northern cliff embeds the same complete mouth in
            // block 70; block 06 supplies the separate low mound entrance.
            "kanto" if matches!(s.metatile_id,0x06|0x70) => (
                s.metatile_id,
                0,
                [[0x48, 0x49], [0x58, 0x59]],
                "outdoor/kanto_mountain_cave",
            ),
            _ => continue,
        };
        if s.metatile_id != metatile || s.subtile_column != column || s.subtile_row != 2 {
            continue;
        }
        let (x, y) = (i % g.width, i / g.width);
        if x + 2 > g.width || y + 2 > g.height {
            continue;
        }
        let complete = (0..2).all(|dy| {
            (0..2).all(|dx| {
                let at = (y + dy) * g.width + x + dx;
                let other = &cells[at].source;
                !claimed[at]
                    && other.tileset_id == s.tileset_id
                    && other.metatile_id == s.metatile_id
                    && usize::from(other.subtile_column) == usize::from(column) + dx
                    && usize::from(other.subtile_row) == 2 + dy
                    && other.tile_index == art[dy][dx]
            })
        });
        if !complete {
            continue;
        }
        let (w, _, n, _) = g.bounds(x, y);
        let (_, e, _, south) = g.bounds(x + 1, y + 1);
        // Use the same resolved local mountain datum, including stacked tiers.
        let mut h = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
        for row in y.saturating_sub(1)..=(y + 1).min(g.height - 1) {
            for col in x.saturating_sub(1)..=(x + 2).min(g.width - 1) {
                if matches!(
                    shapes[row * g.width + col],
                    CellShape::RaisedTop {
                        solid: SolidKind::Bank,
                        ..
                    } | CellShape::LedgeBand { .. }
                ) {
                    h = h.max(shapes[row * g.width + col].surface_height(g.tile_height));
                }
            }
        }
        let t = g.tile_height;
        let side = t * 0.15;
        rock_block(
            &mut mesh.solid,
            [w, w + side, 0., h, n, south],
            g,
            origin,
            1.,
        );
        rock_block(
            &mut mesh.solid,
            [e - side, e, 0., h, n, south],
            g,
            origin,
            -1.,
        );
        let span = e - w - side * 2.;
        let arch = [
            (0., 0.68),
            (0.22, 0.84),
            (0.5, 0.90),
            (0.78, 0.84),
            (1., 0.68),
        ];
        for segment in arch.windows(2) {
            let (x0, y0) = (w + side + span * segment[0].0, h * segment[0].1);
            let (x1, y1) = (w + side + span * segment[1].0, h * segment[1].1);
            for (face, points) in [
                [
                    [x0, y0, south],
                    [x1, y1, south],
                    [x1, h, south],
                    [x0, h, south],
                ],
                [[x1, y1, n], [x0, y0, n], [x0, h, n], [x1, h, n]],
                [[x0, h, n], [x0, h, south], [x1, h, south], [x1, h, n]],
                [[x0, y0, south], [x0, y0, n], [x1, y1, n], [x1, y1, south]],
            ]
            .into_iter()
            .enumerate()
            {
                let p = points.map(Vec3::from_array);
                let normal = (p[1] - p[0]).cross(p[3] - p[0]).normalize().to_array();
                let first = mesh.solid.colors.len();
                outdoor_stone::append_canyon(&mut mesh.solid, points, normal, g, origin);
                if face == 2 {
                    // Keep the whole source perimeter at the neighboring rock
                    // datum. A raised perimeter tears away from the mountain.
                    // Only the cap's interior receives this closed mineral crest.
                    for p in &mut mesh.solid.positions[first..] {
                        let u = ((p[0] - w - side) / span).clamp(0., 1.);
                        let v = ((p[2] - n) / (south - n)).clamp(0., 1.);
                        let crest = u.min(1. - u).min(v.min(1. - v)) * 2.;
                        p[1] += t * 0.16 * crest;
                    }
                }
                if face == 3 {
                    shade_tunnel(&mut mesh.solid, first, n, south);
                }
            }
        }
        // Only the recessed vertical wall is dark. A closed dark cuboid
        // exposes its top above the mineral-deformed arch as a black cap seam.
        // Keep its upper edge inside the recess too: the mineral finish moves
        // the arch's intrados, while this vertical dark wall stays planar.
        for segment in arch.windows(2) {
            let x0 = w + side + span * segment[0].0;
            let x1 = w + side + span * segment[1].0;
            append_quad(
                &mut mesh.solid,
                [
                    [x0, 0., n + t * 0.10],
                    [x1, 0., n + t * 0.10],
                    [x1, (h * segment[1].1 - t * 0.30).max(0.), n + t * 0.10],
                    [x0, (h * segment[0].1 - t * 0.30).max(0.), n + t * 0.10],
                ],
                [0., 0., 1.],
                [[0., 0.]; 4],
                [0.0015, 0.0012, 0.0008, 1.],
            );
        }
        // A visible threshold fades into the same dark recess. A uniformly
        // sunlit floor and lining made the opening read as a gray flat panel.
        let floor = t * 0.015;
        append_quad_colors(
            &mut mesh.solid,
            [
                [w + side, floor, n],
                [w + side, floor, south],
                [e - side, floor, south],
                [e - side, floor, n],
            ],
            [0., 1., 0.],
            [[0., 0.]; 4],
            [
                [0.008, 0.006, 0.004, 1.],
                [0.16, 0.115, 0.065, 1.],
                [0.16, 0.115, 0.065, 1.],
                [0.008, 0.006, 0.004, 1.],
            ],
        );
        for dy in 0..2 {
            for dx in 0..2 {
                let at = (y + dy) * g.width + x + dx;
                claimed[at] = true;
                mesh.authored_cells[at] = Some(label);
            }
        }
    }
}

fn shade_tunnel(out: &mut SurfaceMeshData, first: usize, back: f32, front: f32) {
    for (p, c) in out.positions[first..].iter().zip(&mut out.colors[first..]) {
        let daylight = 0.06 + 0.24 * ((p[2] - back) / (front - back)).clamp(0., 1.);
        for value in &mut c[..3] {
            *value *= daylight;
        }
    }
}

fn rock_block(
    out: &mut SurfaceMeshData,
    bounds: [f32; 6],
    g: &GridGeometry,
    origin: [i32; 2],
    inner_normal: f32,
) {
    let mut block = SurfaceMeshData::default();
    super::modeled_exteriors::portal_block(&mut block, bounds, [0.49, 0.36, 0.20]);
    for (p, n) in block
        .positions
        .chunks_exact(4)
        .zip(block.normals.chunks_exact(4))
    {
        let first = out.colors.len();
        outdoor_stone::append_canyon(out, p.try_into().unwrap(), n[0], g, origin);
        if n[0][0] == inner_normal {
            shade_tunnel(out, first, bounds[4], bounds[5]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_portals_replace_four_source_cells_without_changing_footing() {
        for (tileset, metatile, first_column, art) in [
            ("johto", 0x73, 0, [[0x46, 0x47], [0x56, 0x57]]),
            ("johto_modern", 0x73, 2, [[0x45, 0x22], [0x27, 0x28]]),
            ("kanto", 0x06, 0, [[0x48, 0x49], [0x58, 0x59]]),
            ("kanto", 0x70, 0, [[0x48, 0x49], [0x58, 0x59]]),
        ] {
            let mut sources = vec![super::super::tests::source_with_tile(1, 0, 0, 6); 16];
            for y in 0..2 {
                for x in 0..2 {
                    let mut source = super::super::tests::source_with_tile(
                        metatile,
                        first_column + x as u8,
                        2 + y as u8,
                        art[y][x],
                    );
                    source.tileset_id = tileset.into();
                    sources[(y + 1) * 4 + x + 1] = source;
                }
            }
            let mut frame = super::super::tests::frame(4, 4, sources);
            let g = GridGeometry {
                width: 4,
                height: 4,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: -16.,
                origin_z: -16.,
            };
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![7.; 16];
            mesh.authored_cells = vec![None; 16];
            let mut claimed = vec![false; 16];
            let cells = frame.tiles.iter().collect::<Vec<_>>();
            let shapes = cells
                .iter()
                .map(|cell| shape_for_source(&cell.source))
                .collect::<Vec<_>>();
            append(&mut mesh, &cells, &shapes, &g, &mut claimed, [0, 0]);
            assert_eq!(mesh.footing_heights, vec![7.; 16]);
            assert_eq!(claimed.iter().filter(|v| **v).count(), 4);
            assert!(mesh.textured.positions.is_empty());
            assert!(!mesh.solid.indices.is_empty());
            let darkness = mesh
                .solid
                .colors
                .iter()
                .zip(&mesh.solid.normals)
                .filter(|(color, normal)| color[0] < 0.003 && color[1] < 0.003 && normal[2] > 0.99)
                .count();
            assert!(
                darkness >= 16,
                "the complete arched recess needs a dark backdrop"
            );
            for (color, normal) in mesh.solid.colors.iter().zip(&mesh.solid.normals) {
                if color[0] < 0.003 && color[1] < 0.003 {
                    assert_eq!(normal[1], 0., "the dark recess must not expose a black cap");
                }
            }
            let floor = mesh
                .solid
                .positions
                .iter()
                .zip(&mesh.solid.colors)
                .filter(|(p, _)| (p[1] - g.tile_height * 0.015).abs() < 0.001)
                .map(|(_, c)| c[0])
                .collect::<Vec<_>>();
            assert!(
                floor.iter().any(|c| *c < 0.01) && floor.iter().any(|c| *c > 0.1),
                "a lit threshold must lead into a shaded tunnel floor"
            );
            let h = g.tile_height * crate::profile::MOUNTAIN_CLIFF_HEIGHT / SOURCE_TILE_HEIGHT;
            let (_, _, north, _) = g.bounds(1, 1);
            let (_, _, _, south) = g.bounds(2, 2);
            let mut joins = 0;
            for ((p, normal), uv) in mesh
                .solid
                .positions
                .iter()
                .zip(&mesh.solid.normals)
                .zip(&mesh.solid.uvs)
            {
                if uv[0] < -9000.
                    && normal[1] > 0.9
                    && ((p[2] - north).abs() < 0.001 || (p[2] - south).abs() < 0.001)
                {
                    assert!(
                        (p[1] - h).abs() < 0.001,
                        "the mineral crown must meet its neighboring rock exactly"
                    );
                    joins += 1;
                }
            }
            assert!(
                joins > 20,
                "the real tessellated roof perimeter must be exercised"
            );
            frame.tiles[10].source.tile_index ^= 1;
            let mut rejected = TerrainMeshData::default();
            append(
                &mut rejected,
                &frame.tiles.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &mut vec![false; 16],
                [0, 0],
            );
            assert!(
                rejected.solid.positions.is_empty(),
                "a changed or partial source drawing stays untouched"
            );
        }
    }
}
