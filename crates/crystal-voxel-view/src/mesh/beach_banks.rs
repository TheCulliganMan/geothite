//! Closed sand shoulders below native coastal walking caps. The existing
//! liquid, dry cap, source claims and actor support are never moved.
use super::*;
use bevy::prelude::{IVec2, Vec2, Vec3};

const SEGMENTS: usize = 6;
const PROFILE: [(f32, f32); 6] = [
    (0., 0.),
    (0.28, 0.18),
    (0.72, 0.70),
    (1., 1.),
    (1., 1.12),
    (0., 1.12),
];

fn coastal(map: &str) -> bool {
    crate::environment_fx::coastal_map(map)
        || matches!(
            map,
            "CinnabarIsland"
                | "PalletTown"
                | "VermilionCity"
                | "Route12"
                | "Route13"
                | "Route14"
                | "Route15"
        )
}
fn sand(source: &VisualTileSource) -> bool {
    matches!(
        (source.tileset_id.as_ref(), source.tile_index),
        ("kanto", 0x23) | ("johto" | "johto_modern", 0x06)
    ) || kanto_shore::source_bank(source)
        || (matches!(source.tileset_id.as_ref(), "johto" | "johto_modern")
            && shape_for_source(source) == CellShape::ShoreBand)
}
fn standing_water(tile: &VisualTile, shape: CellShape) -> bool {
    (shape == CellShape::Water
        || matches!(shape,CellShape::PlaneAt{height} if height==crate::profile::WATER_HEIGHT))
        && !tile.priority
        && matches!(
            tile.source.tileset_id.as_ref(),
            "kanto" | "johto" | "johto_modern"
        )
        && tile.source.tile_index == 0x14
}
fn neighbor(g: &GridGeometry, x: usize, z: usize, dx: i32, dz: i32) -> Option<usize> {
    let (x, z) = (x as i32 + dx, z as i32 + dz);
    (x >= 0 && z >= 0 && x < g.width as i32 && z < g.height as i32)
        .then(|| z as usize * g.width + x as usize)
}

// Choose the same owner at a shared corner from either side of a connected
// map handoff. Interpolation between these shared knots keeps banks joined.
fn corner_width(frame: &VisualWorldFrame, x: i32, z: i32) -> f32 {
    let owner = [[-1, -1], [0, -1], [-1, 0], [0, 0]]
        .into_iter()
        .filter_map(|[dx, dz]| {
            let (cx, cz) = (x + dx, z + dz);
            if cx < 0 || cz < 0 || cx >= frame.grid_size.x as i32 || cz >= frame.grid_size.y as i32
            {
                return None;
            }
            Some((
                frame.scenery_map_at(cx as u32, cz as u32),
                frame.scenery_grid_origin_at(cx as u32, cz as u32) + IVec2::new(x, z),
            ))
        })
        .min_by(|a, b| {
            a.0.cmp(b.0)
                .then_with(|| a.1.x.cmp(&b.1.x))
                .then_with(|| a.1.y.cmp(&b.1.y))
        });
    let point = owner
        .map_or(IVec2::new(x, z) + frame.grid_origin, |(_, p)| p)
        .as_vec2();
    (0.37
        + new_bark::meadow_noise(point.x * 0.41 + 17., point.y * 0.41 - 31.) * 0.10
        + new_bark::meadow_noise(point.x * 1.3, point.y * 1.3) * 0.025)
        .clamp(0.30, 0.44)
}
fn point(g: &GridGeometry, p: Vec2, height: f32) -> Vec3 {
    Vec3::new(
        g.origin_x + p.x * g.tile_width,
        height,
        g.origin_z + p.y * g.tile_height,
    )
}
fn profile(g: &GridGeometry, p: Vec2, outward: Vec2, width: f32) -> [Vec3; 6] {
    let water = outdoor_water::level(g.tile_height);
    PROFILE.map(|(radius, depth)| point(g, p + outward * (width * radius), water * depth))
}
fn pigment(base: [f32; 4], layer: usize) -> [f32; 4] {
    let wet = 1. - PROFILE[layer].1.min(1.) * 0.20;
    [base[0] * wet, base[1] * wet, base[2] * wet, base[3]]
}
fn triangle(
    mesh: &mut SurfaceMeshData,
    mut p: [Vec3; 3],
    mut colors: [[f32; 4]; 3],
    outward: Vec3,
    uv_origin: Vec2,
    g: &GridGeometry,
    smooth: bool,
    top: &mut Vec<bool>,
) {
    let mut face = (p[1] - p[0]).cross(p[2] - p[0]);
    if face.length_squared() < 1e-12 {
        return;
    }
    if face.dot(outward) < 0. {
        p.swap(1, 2);
        colors.swap(1, 2);
        face = -face;
    }
    let first = mesh.positions.len() as u32;
    mesh.positions.extend(p.map(|p| p.to_array()));
    mesh.normals.extend([face.normalize().to_array(); 3]);
    mesh.colors.extend(colors);
    mesh.uvs.extend(p.map(|p| {
        [
            (p.x - g.origin_x) / g.tile_width + uv_origin.x,
            (p.z - g.origin_z) / g.tile_height + uv_origin.y,
        ]
    }));
    mesh.indices.extend([first, first + 1, first + 2]);
    top.extend([smooth; 3]);
}
fn prism(
    mesh: &mut SurfaceMeshData,
    a: [Vec3; 6],
    b: [Vec3; 6],
    ca: [f32; 4],
    cb: [f32; 4],
    outward: Vec3,
    uv_origin: Vec2,
    g: &GridGeometry,
    top: &mut Vec<bool>,
) {
    for layer in 0..6 {
        let next = (layer + 1) % 6;
        let hint = match layer {
            0..=2 => Vec3::Y + outward,
            3 => outward,
            4 => Vec3::NEG_Y,
            _ => -outward,
        };
        let colors = [
            pigment(ca, layer),
            pigment(cb, layer),
            pigment(cb, next),
            pigment(ca, next),
        ];
        triangle(
            mesh,
            [a[layer], b[layer], b[next]],
            [colors[0], colors[1], colors[2]],
            hint,
            uv_origin,
            g,
            layer < 3,
            top,
        );
        triangle(
            mesh,
            [a[layer], b[next], a[next]],
            [colors[0], colors[2], colors[3]],
            hint,
            uv_origin,
            g,
            layer < 3,
            top,
        );
    }
    let tangent = (b[3] - a[3]).normalize();
    for j in 1..5 {
        triangle(
            mesh,
            [a[0], a[j], a[j + 1]],
            [pigment(ca, 0), pigment(ca, j), pigment(ca, j + 1)],
            -tangent,
            uv_origin,
            g,
            false,
            top,
        );
        triangle(
            mesh,
            [b[0], b[j], b[j + 1]],
            [pigment(cb, 0), pigment(cb, j), pigment(cb, j + 1)],
            tangent,
            uv_origin,
            g,
            false,
            top,
        );
    }
}
fn smooth_tops(mesh: &mut SurfaceMeshData, top: &[bool]) {
    let key = |p: [f32; 3]| p.map(|v| (v * 10000.).round() as i32);
    let mut sums = std::collections::BTreeMap::<[i32; 3], Vec3>::new();
    for ids in mesh.indices.chunks_exact(3) {
        if !top[ids[0] as usize] {
            continue;
        }
        let p = ids
            .iter()
            .map(|&i| Vec3::from_array(mesh.positions[i as usize]))
            .collect::<Vec<_>>();
        let face = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in ids {
            *sums.entry(key(mesh.positions[i as usize])).or_default() += face;
        }
    }
    for (i, &is_top) in top.iter().enumerate() {
        if is_top {
            mesh.normals[i] = sums[&key(mesh.positions[i])].normalize().to_array();
        }
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
) {
    if g.width == 0
        || g.height == 0
        || cells.len() != g.width * g.height
        || shapes.len() != cells.len()
        || mesh.authored_cells.len() != cells.len()
        || mesh.footing_heights.len() != cells.len()
    {
        return;
    }
    let candidate = (0..cells.len())
        .map(|i| {
            sand(&cells[i].source)
                && !cells[i].priority
                && coastal(frame.scenery_map_at((i % g.width) as u32, (i / g.width) as u32))
                && (matches!(shapes[i], CellShape::Flat | CellShape::ShoreBand)
                    || matches!(shapes[i],CellShape::PlaneAt{height} if height==0.))
                && matches!(
                    mesh.authored_cells[i],
                    None | Some("outdoor/kanto-shore" | "outdoor/johto-shore")
                )
                && mesh.footing_heights[i].abs() < g.tile_height * 0.0001
                && Direction::ALL.into_iter().any(|d| {
                    let (dx, dz) = d.offset();
                    neighbor(g, i % g.width, i / g.width, dx as i32, dz as i32)
                        .is_some_and(|j| standing_water(cells[j], shapes[j]))
                })
        })
        .collect::<Vec<_>>();
    if !candidate.iter().any(|&c| c) {
        return;
    }
    // Sample the completed native sand at the exact contact. This retains
    // its current palette, damp edge and shadow, including at night.
    let mut paint = vec![Vec::<(Vec3, [f32; 4], f32)>::new(); cells.len()];
    for ids in mesh.solid.indices.chunks_exact(3) {
        if !ids.iter().all(|&i| {
            let i = i as usize;
            let uv = mesh.solid.uvs[i];
            mesh.solid.positions[i][1].abs() < g.tile_height * 0.0001
                && mesh.solid.normals[i][1] > 0.99
                && uv[0] > 65000.
                && uv[0] < 69000.
                && matches!((-uv[1] / 10000.).round() as u8, 6 | 11)
        }) {
            continue;
        }
        let center = ids
            .iter()
            .map(|&i| Vec3::from_array(mesh.solid.positions[i as usize]))
            .sum::<Vec3>()
            / 3.;
        let (x, z) = (
            ((center.x - g.origin_x) / g.tile_width).floor() as i32,
            ((center.z - g.origin_z) / g.tile_height).floor() as i32,
        );
        if x < 0 || z < 0 || x >= g.width as i32 || z >= g.height as i32 {
            continue;
        }
        let i = z as usize * g.width + x as usize;
        if !candidate[i] {
            continue;
        }
        paint[i].extend(ids.iter().map(|&j| {
            (
                Vec3::from_array(mesh.solid.positions[j as usize]),
                mesh.solid.colors[j as usize],
                (-mesh.solid.uvs[j as usize][1] / 10000.).round(),
            )
        }));
    }
    let mut bank = SurfaceMeshData::default();
    let mut top = Vec::new();
    for i in 0..cells.len() {
        if !candidate[i] || paint[i].is_empty() {
            continue;
        }
        let (x, z) = (i % g.width, i / g.width);
        let source_origin = frame.scenery_grid_origin_at(x as u32, z as u32).as_vec2();
        let uv_origin = Vec2::new(
            66000. + source_origin.x,
            -paint[i][0].2 * 10000. + source_origin.y,
        );
        let color = |p: Vec2| {
            let q = point(g, p, 0.);
            paint[i]
                .iter()
                .min_by(|a, b| a.0.distance_squared(q).total_cmp(&b.0.distance_squared(q)))
                .unwrap()
                .1
        };
        let wet =
            |dx, dz| neighbor(g, x, z, dx, dz).is_some_and(|j| standing_water(cells[j], shapes[j]));
        for (dx, dz, a, b) in [
            (
                0,
                -1,
                Vec2::new(x as f32, z as f32),
                Vec2::new(x as f32 + 1., z as f32),
            ),
            (
                0,
                1,
                Vec2::new(x as f32, z as f32 + 1.),
                Vec2::new(x as f32 + 1., z as f32 + 1.),
            ),
            (
                -1,
                0,
                Vec2::new(x as f32, z as f32),
                Vec2::new(x as f32, z as f32 + 1.),
            ),
            (
                1,
                0,
                Vec2::new(x as f32 + 1., z as f32),
                Vec2::new(x as f32 + 1., z as f32 + 1.),
            ),
        ] {
            if !wet(dx, dz) {
                continue;
            }
            let outward = Vec2::new(dx as f32, dz as f32);
            let (wa, wb) = (
                corner_width(frame, a.x as i32, a.y as i32),
                corner_width(frame, b.x as i32, b.y as i32),
            );
            for segment in 0..SEGMENTS {
                let (t0, t1) = (
                    segment as f32 / SEGMENTS as f32,
                    (segment + 1) as f32 / SEGMENTS as f32,
                );
                let (p, q) = (a.lerp(b, t0), a.lerp(b, t1));
                prism(
                    &mut bank,
                    profile(g, p, outward, wa + (wb - wa) * t0),
                    profile(g, q, outward, wa + (wb - wa) * t1),
                    color(p),
                    color(q),
                    Vec3::new(outward.x, 0., outward.y),
                    uv_origin,
                    g,
                    &mut top,
                );
            }
        }
        for (dx, dz, p) in [
            (-1, -1, Vec2::new(x as f32, z as f32)),
            (1, -1, Vec2::new(x as f32 + 1., z as f32)),
            (1, 1, Vec2::new(x as f32 + 1., z as f32 + 1.)),
            (-1, 1, Vec2::new(x as f32, z as f32 + 1.)),
        ] {
            if !(wet(dx, 0) && wet(0, dz) && wet(dx, dz)) {
                continue;
            }
            let width = corner_width(frame, p.x as i32, p.y as i32);
            let direction = |step: usize| {
                let a = step as f32 * std::f32::consts::FRAC_PI_2 / 4.;
                Vec2::new(dx as f32 * a.cos(), dz as f32 * a.sin())
            };
            for step in 0..4 {
                let (a, b) = (direction(step), direction(step + 1));
                let n = (a + b).normalize();
                prism(
                    &mut bank,
                    profile(g, p, a, width),
                    profile(g, p, b, width),
                    color(p),
                    color(p),
                    Vec3::new(n.x, 0., n.y),
                    uv_origin,
                    g,
                    &mut top,
                );
            }
        }
    }
    smooth_tops(&mut bank, &top);
    let first = mesh.solid.positions.len() as u32;
    mesh.solid.positions.extend(bank.positions);
    mesh.solid.normals.extend(bank.normals);
    mesh.solid.colors.extend(bank.colors);
    mesh.solid.uvs.extend(bank.uvs);
    mesh.solid
        .indices
        .extend(bank.indices.into_iter().map(|i| first + i));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::tests::{frame, source_with_tile};

    fn fixture(
        size: Vec2,
        color: [f32; 4],
    ) -> (
        VisualWorldFrame,
        GridGeometry,
        Vec<CellShape>,
        TerrainMeshData,
    ) {
        let mut sources = (0..9)
            .map(|i| {
                let mut source = source_with_tile(1, 0, 0, if i == 4 { 0x23 } else { 0x14 });
                source.tileset_id = "kanto".into();
                source
            })
            .collect::<Vec<_>>();
        let mut f = frame(3, 3, std::mem::take(&mut sources));
        f.map_id = "Route20".into();
        f.tile_size = size;
        f.grid_origin = IVec2::new(20, 10);
        let first = Handle::Uuid(
            bevy::asset::uuid::Uuid::from_u128(0x101),
            std::marker::PhantomData,
        );
        let second = Handle::Uuid(
            bevy::asset::uuid::Uuid::from_u128(0x102),
            std::marker::PhantomData,
        );
        for tile in &mut f.tiles {
            tile.texture = first.clone();
            tile.animation_frames = Some(vec![first.clone(), second.clone()].into());
        }
        let g = GridGeometry {
            width: 3,
            height: 3,
            tile_width: size.x,
            tile_height: size.y,
            origin_x: 0.,
            origin_z: 0.,
        };
        let shapes = (0..9)
            .map(|i| {
                if i == 4 {
                    CellShape::Flat
                } else {
                    CellShape::Water
                }
            })
            .collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 9],
            footing_heights: shapes.iter().map(|s| s.surface_height(size.y)).collect(),
            ..Default::default()
        };
        let (w, e, n, s) = g.bounds(1, 1);
        let positions = [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]];
        append_quad_colors(
            &mut mesh.solid,
            positions,
            [0., 1., 0.],
            positions.map(|p| [66000. + p[0] / size.x + 20., -60000. + p[2] / size.y + 10.]),
            [color; 4],
        );
        append_top(
            &mut mesh.animated_textured,
            [0., size.x, 0., size.y],
            outdoor_water::level(size.y),
            g.uv(0, 0),
        );
        (f, g, shapes, mesh)
    }

    #[test]
    fn sand_shoulders_close_every_direction_and_corner_below_the_unchanged_walking_cap() {
        for size in [Vec2::splat(8.), Vec2::splat(32.), Vec2::new(32., 16.)] {
            for color in [[0.42, 0.31, 0.16, 1.], [0.02, 0.04, 0.07, 1.]] {
                let (f, g, shapes, mut mesh) = fixture(size, color);
                let original = f.clone();
                let before = mesh.clone();
                finish(
                    &mut mesh,
                    &f,
                    &f.tiles.iter().collect::<Vec<_>>(),
                    &shapes,
                    &g,
                );
                assert_eq!(f, original);
                assert_eq!(mesh.authored_cells, before.authored_cells);
                assert_eq!(mesh.footing_heights, before.footing_heights);
                assert_eq!(mesh.animated_textured, before.animated_textured);
                assert_eq!(
                    &mesh.solid.positions[..4],
                    before.solid.positions.as_slice()
                );
                assert_eq!(&mesh.solid.normals[..4], before.solid.normals.as_slice());
                assert_eq!(&mesh.solid.colors[..4], before.solid.colors.as_slice());
                assert_eq!(&mesh.solid.uvs[..4], before.solid.uvs.as_slice());
                assert_eq!(&mesh.solid.indices[..6], before.solid.indices.as_slice());
                assert!(mesh.solid.positions.len() > 100);
                for p in &mesh.solid.positions[4..] {
                    assert!(p[1] <= 0. && p[1] >= outdoor_water::level(size.y) * 1.12001);
                    let q = Vec2::new(p[0] / size.x, p[2] / size.y);
                    let nearest = q.clamp(Vec2::ONE, Vec2::splat(2.));
                    assert!(
                        q.distance(nearest) <= 0.44001,
                        "the sand toe must stay within its adjacent water cells"
                    );
                }
                for (normal, pigment) in mesh.solid.normals[4..].iter().zip(&mesh.solid.colors[4..])
                {
                    assert!((Vec3::from_array(*normal).length() - 1.).abs() < 0.001);
                    assert!(
                        (0..3)
                            .all(|c| pigment[c] >= color[c] * 0.799
                                && pigment[c] <= color[c] * 1.001),
                        "the new shoulder uses the received sand pigment at day and night"
                    );
                }
                let key = |p: [f32; 3]| p.map(|v| (v * 10000.).round() as i32);
                let mut edges = std::collections::BTreeMap::new();
                let mut volume = 0.;
                for ids in mesh.solid.indices[6..].chunks_exact(3) {
                    let p = ids
                        .iter()
                        .map(|&i| Vec3::from_array(mesh.solid.positions[i as usize]))
                        .collect::<Vec<_>>();
                    let face = (p[1] - p[0]).cross(p[2] - p[0]);
                    let normal = ids
                        .iter()
                        .map(|&i| Vec3::from_array(mesh.solid.normals[i as usize]))
                        .sum::<Vec3>();
                    assert!(face.length_squared() > 1e-12 && face.dot(normal) > 0.);
                    let root = Vec3::new(size.x, 0., size.y);
                    volume += (p[0] - root).dot((p[1] - root).cross(p[2] - root)) / 6.;
                    for [a, b] in [[0, 1], [1, 2], [2, 0]] {
                        let (a, b) = (key(p[a].to_array()), key(p[b].to_array()));
                        *edges
                            .entry(if a < b { (a, b) } else { (b, a) })
                            .or_insert(0) += 1;
                    }
                }
                assert!(
                    volume > 0. && edges.values().all(|&count| count % 2 == 0),
                    "each shoulder must be a closed outward volume"
                );
                assert!(
                    mesh.solid.indices.len() / 3 < 1000,
                    "the shoreline remains part of the bounded terrain batch"
                );
            }
        }
    }

    #[test]
    fn sand_growth_rejects_unproven_water_raised_or_owned_ground_rooms_and_paving() {
        for case in 0..10 {
            let (mut f, g, mut shapes, mut mesh) = fixture(Vec2::splat(8.), [0.4; 4]);
            match case {
                0 => f.map_id = "LakeOfRage".into(),
                1 => f.tiles[4].source.tile_index = 0x2c,
                2 => mesh.authored_cells[4] = Some("fixture/owned"),
                3 => mesh.footing_heights[4] = 8.,
                4 => {
                    for (i, tile) in f.tiles.iter_mut().enumerate() {
                        if i != 4 {
                            tile.source.tile_index = 0x12;
                        }
                    }
                }
                5 => {
                    for tile in &mut f.tiles {
                        if tile.column != 1 || tile.row != 1 {
                            tile.priority = true;
                        }
                    }
                }
                6 => {
                    for tile in &mut f.tiles {
                        if tile.column != 1 || tile.row != 1 {
                            tile.source.tile_index = 0x99;
                        }
                    }
                }
                7 => mesh.solid.uvs.iter_mut().for_each(|uv| uv[1] -= 20000.),
                8 => shapes[4] = CellShape::Water,
                _ => f.tiles[4].source.tileset_id = "port".into(),
            }
            let before = mesh.clone();
            finish(
                &mut mesh,
                &f,
                &f.tiles.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
            );
            assert_eq!(
                mesh, before,
                "unproven shoreline case {case} must retain its complete presentation"
            );
        }
    }

    #[test]
    fn painted_native_shore_border_uses_its_existing_sand_cap_as_the_shoulder_contact() {
        let (mut f, g, mut shapes, mut mesh) = fixture(Vec2::splat(8.), [0.4; 4]);
        f.tiles[4].source.metatile_id = 0x1f;
        f.tiles[4].source.tile_index = 0x33;
        mesh.authored_cells[4] = Some("outdoor/kanto-shore");
        shapes[4] = CellShape::Flat;
        // Route20's actual rendered $1f/$33 caps use received sand role 11,
        // rather than the fixed-palette path role 6 used by the base fixture.
        for uv in &mut mesh.solid.uvs {
            uv[1] -= 50000.;
        }
        assert!(kanto_shore::source_bank(&f.tiles[4].source));
        let before = mesh.clone();
        finish(
            &mut mesh,
            &f,
            &f.tiles.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
        );
        assert!(mesh.solid.positions.len() > before.solid.positions.len());
        assert!(
            mesh.solid.uvs[4..]
                .iter()
                .all(|uv| (-uv[1] / 10000.).round() == 11.),
            "new faces retain the native sand finish and its source-palette lighting"
        );
        assert_eq!(mesh.authored_cells, before.authored_cells);
        assert_eq!(mesh.footing_heights, before.footing_heights);
        assert_eq!(
            &mesh.solid.positions[..4],
            before.solid.positions.as_slice()
        );
    }

    #[test]
    fn paused_water_and_native_level_planes_keep_the_same_physical_sand_bank() {
        let (f, g, shapes, mut a) = fixture(Vec2::splat(8.), [0.4; 4]);
        let mut paused = f.clone();
        let mut b = a.clone();
        let mut planes = shapes.clone();
        for (i, tile) in paused.tiles.iter_mut().enumerate() {
            tile.animation_frames = None;
            planes[i] = CellShape::PlaneAt {
                height: if i == 4 {
                    0.
                } else {
                    crate::profile::WATER_HEIGHT
                },
            };
        }
        finish(&mut a, &f, &f.tiles.iter().collect::<Vec<_>>(), &shapes, &g);
        finish(
            &mut b,
            &paused,
            &paused.tiles.iter().collect::<Vec<_>>(),
            &planes,
            &g,
        );
        assert!(a.solid.positions.len() > 4);
        assert_eq!(
            a, b,
            "physical shores must follow the source water and native level, not animation metadata"
        );
    }

    #[test]
    fn beach_geometry_and_sand_paint_survive_a_connected_map_handoff() {
        let (f, g, shapes, mut a) = fixture(Vec2::splat(8.), [0.4; 4]);
        let mut b = a.clone();
        let mut connected = f.clone();
        connected.map_id = "CinnabarIsland".into();
        let shift = IVec2::new(-40, 8);
        connected.grid_origin += shift;
        connected.source_regions = vec![
            crystal_render_api::VisualSourceRegion {
                map_id: "CinnabarIsland".into(),
                origin: IVec2::ZERO,
                size: bevy::prelude::UVec2::splat(4),
            },
            crystal_render_api::VisualSourceRegion {
                map_id: "Route20".into(),
                origin: shift,
                size: bevy::prelude::UVec2::splat(100),
            },
        ]
        .into();
        finish(&mut a, &f, &f.tiles.iter().collect::<Vec<_>>(), &shapes, &g);
        finish(
            &mut b,
            &connected,
            &connected.tiles.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
        );
        assert!(a.solid.positions.len() > 4);
        assert_eq!(
            a, b,
            "same source-owned sand shoulder, paint and water contact across the handoff"
        );
    }
}
