//! Complete authored rock rims with a beveled opening and continuous lower floor.
use super::*;

const DRAWING: [[u16; 4]; 4] = [
    [0x05, 0x24, 0x24, 0x07],
    [0x15, 0x16, 0x16, 0x17],
    [0x15, 0x16, 0x16, 0x17],
    [0x25, 0x26, 0x26, 0x27],
];

pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    for row in 0..r.g.height.saturating_sub(3) {
        for column in 0..r.g.width.saturating_sub(3) {
            let source = &r.cells[row * r.g.width + column].source;
            if !matches!(source.tileset_id.as_ref(), "cave" | "dark_cave")
                || source.metatile_id != 0x31
                || source.subtile_column != 0
                || source.subtile_row != 0
            {
                continue;
            }
            let complete = (0..4).all(|y| {
                (0..4).all(|x| {
                    let s = &r.cells[(row + y) * r.g.width + column + x].source;
                    s.tileset_id == source.tileset_id
                        && s.metatile_id == 0x31
                        && usize::from(s.subtile_column) == x
                        && usize::from(s.subtile_row) == y
                        && s.tile_index == DRAWING[y][x]
                })
            });
            if complete {
                r.add(Placement {
                    column,
                    row,
                    width: 4,
                    height: 4,
                    ground: (row + 1) * r.g.width + column + 1,
                    form: Form::CaveRim,
                    rise_pixels: crate::cave::CAVE_SHELF_HEIGHT,
                    depth_pixels: 32.,
                    front_rows: 4.,
                });
            }
        }
    }
}

fn cap_triangle(
    out: &mut SurfaceMeshData,
    p: [Vec3; 3],
    color: [f32; 4],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]);
    if normal.length_squared() < 1e-8 {
        return;
    }
    // The first edge of each outer cap triangle meets a scarp sampled every
    // quarter tile. Match those samples before weathering: a sparse cap over
    // a dense face otherwise leaves gaps along the displaced rock lip.
    let axis_aligned = (p[0].x - p[1].x).abs() < 0.00001 || (p[0].z - p[1].z).abs() < 0.00001;
    let divisions = if axis_aligned {
        (((p[1].x - p[0].x).abs() / g.tile_width).max((p[1].z - p[0].z).abs() / g.tile_height) * 4.)
            .ceil()
            .max(1.) as usize
    } else {
        1
    };
    for i in 0..divisions {
        let mut triangle = [
            p[0].lerp(p[1], i as f32 / divisions as f32),
            p[0].lerp(p[1], (i + 1) as f32 / divisions as f32),
            p[2],
        ];
        if normal.y < 0. {
            triangle.swap(1, 2);
        }
        let first = out.positions.len() as u32;
        out.positions.extend(triangle.map(|p| p.to_array()));
        out.normals.extend([[0., 1., 0.]; 3]);
        out.uvs
            .extend(triangle.map(|p| cave_cliff::floor_uv(p, g, origin, false)));
        out.colors
            .extend(triangle.map(|p| cave_cliff::pigment(color, p, g, origin, false)));
        out.indices.extend([first, first + 1, first + 2]);
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    origin: [i32; 2],
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + 4. * g.tile_width;
    let s = n + 4. * g.tile_height;
    let h = crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    let x0 = w + g.tile_width;
    let x1 = e - g.tile_width;
    let z0 = n + g.tile_height;
    let z1 = s - g.tile_height;
    let bevel = g.tile_width * 0.175;
    let upper = [
        Vec3::new(x0, h, z0 - bevel),
        Vec3::new(x1, h, z0 - bevel),
        Vec3::new(x1 + bevel, h, z0),
        Vec3::new(x1 + bevel, h, z1),
        Vec3::new(x1, h, z1 + bevel),
        Vec3::new(x0, h, z1 + bevel),
        Vec3::new(x0 - bevel, h, z1),
        Vec3::new(x0 - bevel, h, z0),
    ];
    let lower = [
        Vec3::new(x0, 0., z0),
        Vec3::new(x1, 0., z0),
        Vec3::new(x1, 0., z0),
        Vec3::new(x1, 0., z1),
        Vec3::new(x1, 0., z1),
        Vec3::new(x0, 0., z1),
        Vec3::new(x0, 0., z1),
        Vec3::new(x0, 0., z0),
    ];
    let outer = [
        Vec3::new(w, h, n),
        Vec3::new(e, h, n),
        Vec3::new(e, h, n),
        Vec3::new(e, h, s),
        Vec3::new(e, h, s),
        Vec3::new(w, h, s),
        Vec3::new(w, h, s),
        Vec3::new(w, h, n),
    ];
    let dark = cells[p.ground].source.tileset_id.as_ref() == "dark_cave";
    let color = if dark {
        [0.31, 0.37, 0.39, 1.]
    } else {
        [0.34, 0.345, 0.31, 1.]
    };
    new_bark::cave_finish::append(
        &mut mesh.solid,
        [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]],
        [0., 1., 0.],
        &cells[p.ground].source,
        g,
        origin,
    );
    // Split each opening edge and vary its shoulder in map coordinates.
    // Only the blocked rock lip changes: the entire lower opening stays on
    // its original boundary and both source height datums remain exact.
    let split = |points: [Vec3; 8]| -> [Vec3; 16] {
        std::array::from_fn(|i| points[i / 2].lerp(points[(i / 2 + 1) % 8], (i % 2) as f32 * 0.5))
    };
    let lip: [Vec3; 16] = split(upper).map(|p| {
        let x = (p.x - g.origin_x) / g.tile_width + origin[0] as f32;
        let z = (p.z - g.origin_z) / g.tile_height + origin[1] as f32;
        let relief = 1.55 + cave_cliff::slab(x * 0.9 + 29., z * 0.9 - 17.) * 0.50;
        let inside = Vec3::new(p.x.clamp(x0, x1), h, p.z.clamp(z0, z1));
        inside + (p - inside) * relief
    });
    let foot = split(lower);
    let cap_edge = split(outer);
    for i in 0..16 {
        let j = (i + 1) % 16;
        cap_triangle(
            &mut mesh.solid,
            [cap_edge[i], cap_edge[j], lip[j]],
            color,
            g,
            origin,
        );
        cap_triangle(
            &mut mesh.solid,
            [cap_edge[i], lip[j], lip[i]],
            color,
            g,
            origin,
        );
        let first = mesh.solid.positions.len();
        cave_cliff::quad_with_point(
            &mut mesh.solid,
            [foot[i], foot[j], lip[j], lip[i]].map(|p| p.to_array()),
            color,
            g,
            origin,
            |p, _, _| p,
            false,
        );
        for k in first..mesh.solid.positions.len() {
            mesh.solid.uvs[k] =
                cave_cliff::stone_uv(Vec3::from_array(mesh.solid.positions[k]), g, origin, false);
        }
    }
    let mut faces = Vec::new();
    for (a, b, normal) in [
        (outer[0], outer[1], [0., 0., -1.]),
        (outer[2], outer[3], [1., 0., 0.]),
        (outer[4], outer[5], [0., 0., 1.]),
        (outer[6], outer[7], [-1., 0., 0.]),
    ] {
        let bottom = |p: Vec3| Vec3::new(p.x, 0., p.z);
        faces.push((
            [a, b, bottom(b), bottom(a)].map(|p| p.to_array()),
            normal,
            color,
        ));
    }
    cave_cliff::scarps(&mut mesh.solid, faces, g, origin);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_cave_rim_keeps_lower_floor_and_bevels_only_the_blocked_stone() {
        for tileset in ["cave", "dark_cave"] {
            let mut t = super::super::tests::tiles(4, 4, tileset);
            for y in 0..4 {
                for x in 0..4 {
                    let source = &mut t[y * 4 + x].source;
                    source.metatile_id = 0x31;
                    source.subtile_column = x as u8;
                    source.subtile_row = y as u8;
                    source.tile_index = DRAWING[y][x];
                }
            }
            let g = super::super::tests::grid(4, 4);
            let refs = t.iter().collect::<Vec<_>>();
            let placements = super::super::resolve("UnionCave1F", &refs, &g, None);
            assert_eq!(placements.len(), 1);
            assert_eq!(placements[0].kind_label(), "dungeon:cave-rock-rim");
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![0.; 16];
            super::super::append(&mut mesh, &g, &placements[0], &refs, [0, 0]);
            assert_eq!(mesh.footing_heights, vec![0.; 16]);
            assert!(mesh.textured.positions.is_empty());
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| (0. ..=6.).contains(&p[1])
                        && (-3. ..=35.).contains(&p[0])
                        && (-3. ..=35.).contains(&p[2])
                        && (p[1] != 0. && p[1] != 6.
                            || ((0. ..=32.).contains(&p[0]) && (0. ..=32.).contains(&p[2]))))
            );
            for corner in [[0., 6., 0.], [32., 6., 0.], [32., 6., 32.], [0., 6., 32.]] {
                assert!(mesh.solid.positions.contains(&corner));
            }
            // The inner shoulder is broader than the old uniform 1.4-pixel
            // bevel and varies along its edge, rather than tracing a box.
            let shoulder_widths = mesh
                .solid
                .positions
                .iter()
                .filter_map(|p| {
                    (p[1] == 6. && p[0] > 4. && p[0] < 8. && (8. ..=24.).contains(&p[2]))
                        .then_some(8. - p[0])
                })
                .collect::<Vec<_>>();
            let narrow = shoulder_widths
                .iter()
                .copied()
                .fold(f32::INFINITY, f32::min);
            let broad = shoulder_widths
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(narrow > 1.4 && broad < 3.);
            assert!(broad - narrow > 0.05);
            // Every top-of-scarp edge sample must also belong to the cap.
            // Equal endpoints alone do not close a weathered T-junction.
            let cap_points = mesh
                .solid
                .positions
                .iter()
                .zip(&mesh.solid.uvs)
                .filter_map(|(p, uv)| ((uv[1] / 100000.).round() == 4.).then_some(p))
                .collect::<Vec<_>>();
            for (p, uv) in mesh.solid.positions.iter().zip(&mesh.solid.uvs) {
                if p[1] == 6.
                    && (uv[1] / 100000.).round() == 6.
                    && (p[0] == 0. || p[0] == 32. || p[2] == 0. || p[2] == 32.)
                {
                    assert!(cap_points.iter().any(|q| Vec3::from_array(**q).distance(Vec3::from_array(*p)) < 0.00001));
                }
            }
            // Every raised point inside the original 2x2 floor is forbidden.
            assert!(!mesh.solid.positions.iter().any(|p| p[1] > 0.
                && p[0] > 8.
                && p[0] < 24.
                && p[2] > 8.
                && p[2] < 24.));
            for tri in mesh.solid.indices.chunks_exact(3) {
                let p = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
                assert!(normal.is_finite());
                assert!(normal.dot(Vec3::from_array(mesh.solid.normals[tri[0] as usize])) > 0.8);
            }
            // Scrolling must translate the authored rim, not resample its
            // weathering into another silhouette or tear cap/body seams.
            let mut shifted_grid = g;
            shifted_grid.origin_x -= 16.;
            shifted_grid.origin_z += 8.;
            let mut shifted = TerrainMeshData::default();
            super::super::append(&mut shifted, &shifted_grid, &placements[0], &refs, [0, 0]);
            assert_eq!(shifted.solid.indices, mesh.solid.indices);
            for (a, b) in shifted.solid.positions.iter().zip(&mesh.solid.positions) {
                assert!(
                    (Vec3::from_array(*a) - Vec3::from_array(*b) - Vec3::new(-16., 0., 8.))
                        .length()
                        < 0.0001
                );
            }
            drop(refs);
            t[5].source.tile_index = 0x01;
            let refs = t.iter().collect::<Vec<_>>();
            assert!(
                !super::super::resolve("UnionCave1F", &refs, &g, None)
                    .iter()
                    .any(|p| matches!(p.form, Form::CaveRim))
            );
        }
    }
}
