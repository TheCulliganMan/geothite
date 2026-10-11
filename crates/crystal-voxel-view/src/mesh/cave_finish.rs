//! Continuous cave terrain. Enclosing rock shares the outside relief field;
//! walking datums, ladders, water and unknown art retain their source geometry.
use super::*;

fn eligible(
    source: &VisualTileSource,
    normal: [f32; 3],
    points: &[[f32; 3]; 4],
    g: &GridGeometry,
) -> bool {
    if source.subtile_column >= 4 || source.subtile_row >= 4 {
        return false;
    }
    let Some(shape) = crate::cave::cave_shape(source) else {
        return false;
    };
    let limit = crate::cave::rock_height(source) * g.tile_height / SOURCE_TILE_HEIGHT;
    if points.iter().any(|p| p[1] < -0.001 || p[1] > limit + 0.001) {
        return false;
    }
    match shape {
        CellShape::Flat => {
            // Caps and implied side closures deliberately borrow the verified
            // plain floor sample. Its UV identifies the paint, not the world
            // cell's elevation. Keep the cap's existing geometry and datum.
            source.tile_index == 0x16 && (normal[1] > 0.999 || normal[1].abs() < 0.001)
        }
        CellShape::RaisedTop { .. } | CellShape::LedgeBand { .. } => {
            normal[1] > 0.999 || normal[1].abs() < 0.001
        }
        _ => false,
    }
}

/// Only complete vertical faces spanning the authored lower-floor/shelf
/// interval can join the shallow scarp contour. Caps and ramps stay separate.
pub(in crate::mesh) fn shallow_face(
    points: &[[f32;3];4], normal: [f32;3], source: &VisualTileSource, g: &GridGeometry,
) -> Option<[f32;4]> {
    if !eligible(source,normal,points,g) || normal[1].abs()>0.001
        || !((normal[0].abs()==1. && normal[2]==0.) || (normal[2].abs()==1. && normal[0]==0.)) { return None; }
    let height=crate::cave::CAVE_SHELF_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
    if !points.iter().all(|p|p[1]==0. || p[1]==height)
        || !points.iter().any(|p|p[1]==0.) || !points.iter().any(|p|p[1]==height) { return None; }
    Some(if source.tileset_id.as_ref()=="dark_cave" { [0.31*0.86,0.37*0.86,0.39*0.86,1.] }
        else {[0.34*0.86,0.345*0.86,0.31*0.86,1.]})
}

/// Borrowed enclosing caps share the authenticated corner contour too.
pub(in crate::mesh) fn enclosing_color(points: &[[f32;3];4], normal: [f32;3], source: &VisualTileSource, g: &GridGeometry) -> Option<[f32;4]> {
    (eligible(source,normal,points,g) && points.iter().any(|p|p[1]>=32.*g.tile_height/SOURCE_TILE_HEIGHT))
        .then_some(if source.tileset_id.as_ref()=="dark_cave" {[0.31,0.37,0.39,1.]} else {[0.34,0.345,0.31,1.]})
}

fn color(
    p: [f32; 3],
    normal: [f32; 3],
    dark: bool,
    g: &GridGeometry,
    origin: [i32; 2],
) -> [f32; 4] {
    let x = (p[0] - g.origin_x) / g.tile_width + origin[0] as f32;
    let z = (p[2] - g.origin_z) / g.tile_height + origin[1] as f32;
    let y = p[1] / g.tile_height;
    let broad = meadow_noise(x * 0.22, z * 0.22);
    let grain = meadow_noise(x * 2.3 + 17.0, z * 2.3 - 11.0);
    let wall = normal[1].abs() < 0.001;
    if !wall && y > 0.01 {
        let base = if dark {
            [0.31, 0.37, 0.39]
        } else {
            [0.34, 0.345, 0.31]
        };
        return cave_cliff::pigment(
            [base[0], base[1], base[2], 1.],
            Vec3::from_array(p),
            g,
            origin,
            false,
        );
    }
    let base = match (dark, wall) {
        (false, false) => [0.43, 0.405, 0.32],
        (false, true) => [0.34, 0.345, 0.31],
        (true, false) => [0.295, 0.355, 0.38],
        (true, true) => [0.31, 0.37, 0.39],
    };
    // Broad mineral layers follow elevation continuously across folded rows;
    // their gentle warp is seeded in map coordinates, never viewport cells.
    let strata = (y * 9.5 + broad * 2.0).sin() * 0.025;
    let contact = if wall {
        0.76 + 0.24 * (y / 0.7).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let facing = if wall {
        0.82 + 0.12 * (normal[0] * -0.6 + normal[2] * 0.8).max(0.0)
    } else {
        1.0
    };
    // Keep mineral grain below the broad tonal variation; large high-frequency
    // vertex changes read as a repeated triangle pattern across every shelf.
    let variation =
        broad * 0.065 + grain * if wall { 0.015 } else { 0.009 } + if wall { strata } else { 0.0 };
    [
        ((base[0] + variation) * contact * facing).clamp(0.0, 1.0),
        ((base[1] + variation) * contact * facing).clamp(0.0, 1.0),
        ((base[2] + variation) * contact * facing).clamp(0.0, 1.0),
        1.0,
    ]
}

pub(in crate::mesh) fn append(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    normal: [f32; 3],
    source: &VisualTileSource,
    g: &GridGeometry,
    origin: [i32; 2],
) -> bool {
    if !eligible(source, normal, &points, g) {
        return false;
    }
    let dark = source.tileset_id.as_ref() == "dark_cave";
    // Borrowed caps on enclosing ledge bands must share the same relief as
    // the adjacent rock mass and outside continuation. A flat cap underneath
    // those deformed surfaces exposes patches and opens their shared edge.
    if points
        .iter()
        .any(|p| p[1] >= 32. * g.tile_height / SOURCE_TILE_HEIGHT)
    {
        let tint = if dark {
            [0.31, 0.37, 0.39, 1.]
        } else {
            [0.34, 0.345, 0.31, 1.]
        };
        cave_cliff::quad(out, points, tint, g, origin);
        return true;
    }
    let at = |u: usize, v: usize| {
        // Preserve exact outside edges. Every subdivision is on the original
        // plane, including its height; this finish cannot change footing.
        let u = u as f32 / 3.0;
        let v = v as f32 / 3.0;
        let a = Vec3::from_array(points[0]).lerp(Vec3::from_array(points[3]), u);
        let b = Vec3::from_array(points[1]).lerp(Vec3::from_array(points[2]), u);
        a.lerp(b, v).to_array()
    };
    for v in 0..3 {
        for u in 0..3 {
            let patch = [at(u, v), at(u, v + 1), at(u + 1, v + 1), at(u + 1, v)];
            append_quad_colors(
                out,
                patch,
                normal,
                patch.map(|p| if normal[1].abs()<0.001 {
                    cave_cliff::stone_uv(Vec3::from_array(p), g, origin, false)
                } else { cave_cliff::floor_uv(Vec3::from_array(p), g, origin, false) }),
                patch.map(|p| color(p, normal, dark, g, origin)),
            );
        }
    }
    true
}

/// Feather mineral dust from verified enclosing rock and shallow banks onto
/// the lower walking floor. Raised tops never bleed into their own plateau.
/// This changes only pigment: source heights, silhouette and opacity stay exact.
pub(in crate::mesh) fn blend_wall_feet(
    out: &mut SurfaceMeshData, cells: &[&VisualTile], g: &GridGeometry, origin: [i32; 2],
) {
    if g.width == 0 || g.height == 0 { return; }
    for i in 0..out.positions.len() {
        if out.uvs[i][0] < 90000. || (out.uvs[i][1] / 100000.).round() != 4.
            || out.normals[i][1] < 0.999 { continue; }
        let p = Vec3::from_array(out.positions[i]);
        if p.y > crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT + 0.001 { continue; }
        let x = (p.x - g.origin_x) / g.tile_width;
        let z = (p.z - g.origin_z) / g.tile_height;
        let mut distance = f32::INFINITY;
        let mut dark = false;
        let mut shallow = false;
        for row in (z.floor() as i32 - 2)..=(z.floor() as i32 + 2) {
            for column in (x.floor() as i32 - 2)..=(x.floor() as i32 + 2) {
                if row < 0 || column < 0 || row >= g.height as i32 || column >= g.width as i32 { continue; }
                let source = &cells[row as usize * g.width + column as usize].source;
                let Some(CellShape::RaisedTop { height, .. } | CellShape::LedgeBand { height, .. }) = crate::cave::cave_shape(source) else { continue; };
                let bank = height == crate::cave::CAVE_SHELF_HEIGHT;
                if height < 32. && !bank { continue; }
                if bank && p.y >= height * g.tile_height / SOURCE_TILE_HEIGHT - 0.001 { continue; }
                let dx = (column as f32 - x).max(0.).max(x - column as f32 - 1.);
                let dz = (row as f32 - z).max(0.).max(z - row as f32 - 1.);
                let d = dx.hypot(dz);
                if d < distance { distance = d; dark = source.tileset_id.as_ref() == "dark_cave"; shallow = bank; }
            }
        }
        let width = (if shallow { 0.65 } else { 1.15 }) + meadow_noise((x + origin[0] as f32) * 0.6 + 31.,
            (z + origin[1] as f32) * 0.6 - 19.) * 0.25;
        let t = (1. - distance / width).clamp(0., 1.);
        if t == 0. { continue; }
        let tint = if dark { [0.31, 0.37, 0.39, 1.] } else { [0.34, 0.345, 0.31, 1.] };
        let mineral = cave_cliff::pigment(tint, p, g, origin, false);
        let strength = t * t * (3. - 2. * t) * 0.65;
        for channel in 0..3 {
            out.colors[i][channel] += (mineral[channel] - out.colors[i][channel]) * strength;
        }
    }
}

/// Break the silhouette of exposed shelf lips without changing elevation.
/// Shared cap/body vertices use one map-space displacement; walking centers
/// and the lower floor are untouched. Only proven six-pixel shelf edges qualify.
pub(in crate::mesh) fn weather_shelf_lips(
    out: &mut SurfaceMeshData, cells: &[&VisualTile], g: &GridGeometry, origin: [i32; 2],
) {
    if g.width == 0 || g.height == 0 { return; }
    let height = crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    let shelf = |i: usize| crate::cave::cave_shape(&cells[i].source)
        .is_some_and(|s| s.surface_height(SOURCE_TILE_HEIGHT) == crate::cave::CAVE_SHELF_HEIGHT);
    let lower = |i: usize| crate::cave::cave_shape(&cells[i].source)
        .is_some_and(|s| matches!(s, CellShape::Flat | CellShape::Water | CellShape::PlaneAt { height: 0. }));
    let mut moved = vec![false; out.positions.len()];
    for i in 0..out.positions.len() {
        // Eligibility comes from the actual shelf boundary. Borrowed atlas
        // paint and mineral paint must move together at that shared join.
        if (out.positions[i][1] - height).abs() > 0.001 { continue; }
        let p = Vec3::from_array(out.positions[i]);
        let x = (p.x - g.origin_x) / g.tile_width;
        let z = (p.z - g.origin_z) / g.tile_height;
        let mut direction = Vec3::ZERO;
        for row in (z.floor() as i32 - 1)..=(z.floor() as i32) {
            for column in (x.floor() as i32 - 1)..=(x.floor() as i32) {
                if row < 0 || column < 0 || row >= g.height as i32 || column >= g.width as i32 { continue; }
                let index = row as usize * g.width + column as usize;
                if !shelf(index) { continue; }
                for (dx, dz) in [(0,-1),(1,0),(0,1),(-1,0)] {
                    let (cx, cz) = (column + dx, row + dz);
                    if cx < 0 || cz < 0 || cx >= g.width as i32 || cz >= g.height as i32 { continue; }
                    let other = cz as usize * g.width + cx as usize;
                    if cells[index].source.tileset_id != cells[other].source.tileset_id || !lower(other) { continue; }
                    let on_edge = if dx != 0 {
                        (x - (column + i32::from(dx > 0)) as f32).abs() < 0.001
                            && z >= row as f32 - 0.001 && z <= row as f32 + 1.001
                    } else {
                        (z - (row + i32::from(dz > 0)) as f32).abs() < 0.001
                            && x >= column as f32 - 0.001 && x <= column as f32 + 1.001
                    };
                    if on_edge {
                        if dx != 0 { direction.x = dx as f32; } else { direction.z = dz as f32; }
                    }
                }
            }
        }
        let relief = 0.10 + cave_cliff::slab((x + origin[0] as f32) * 0.74 + 19.,
            (z + origin[1] as f32) * 0.74 - 23.) * 0.08;
        let delta = direction * Vec3::new(g.tile_width, 0., g.tile_height) * relief;
        out.positions[i] = (p + delta).to_array();
        moved[i] = delta != Vec3::ZERO;
        if out.uvs[i][0] > 90000. {
            out.uvs[i][0] += delta.x;
            out.uvs[i][1] += delta.z;
        }
    }
    // Keep side facets aligned with the changed silhouette. Horizontal caps
    // retain their exact upward normal and their six-pixel walking datum.
    for triangle in out.indices.chunks_exact(3) {
        if !triangle.iter().any(|&i| moved[i as usize]) { continue; }
        let points: [Vec3;3] = std::array::from_fn(|i| Vec3::from_array(out.positions[triangle[i] as usize]));
        let Some(normal) = (points[1]-points[0]).cross(points[2]-points[0]).try_normalize() else { continue; };
        let light = Vec3::new(-0.4,0.8,0.45).normalize();
        let shade = 0.60 + 0.40 * normal.dot(light).max(0.);
        for &index in triangle {
            let index = index as usize;
            if out.normals[index][1] < 0.999 {
                let old = 0.60 + 0.40 * Vec3::from_array(out.normals[index]).dot(light).max(0.);
                for channel in 0..3 { out.colors[index][channel] *= shade / old; }
                out.normals[index] = normal.to_array();
            }
        }
    }
}

/// Small fractures and stone chips on verified low cave floor planes. These
/// are visual relief only; their source walk datum stays exact underneath.
pub(in crate::mesh) fn floor_detail(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    normal: [f32; 3],
    source: &VisualTileSource,
    g: &GridGeometry,
    origin: [i32; 2],
) {
    if normal[1] < 0.999
        || !eligible(source, normal, &points, g)
        || points[0][1]
            > g.tile_height * crate::cave::CAVE_SHELF_HEIGHT / SOURCE_TILE_HEIGHT + 0.001
    {
        return;
    }
    let [w, e, n, s] = face_bounds(&points);
    if (e - w - g.tile_width).abs() > 0.001 || (s - n - g.tile_height).abs() > 0.001 {
        return;
    }
    let x = ((w - g.origin_x) / g.tile_width).round() as i32 + origin[0];
    let z = ((n - g.origin_z) / g.tile_height).round() as i32 + origin[1];
    let seed = lattice(x.wrapping_mul(7) + 71, z.wrapping_mul(13) + 19);
    if seed < 0.78 {
        return;
    }
    let unit = g.tile_width.min(g.tile_height);
    let root = Vec3::new(
        w + unit * (0.25 + lattice(x + 23, z - 18) * 0.50),
        points[0][1] + unit * 0.001,
        n + unit * (0.25 + lattice(x - 27, z + 32) * 0.50),
    );
    let angle = seed * std::f32::consts::TAU;
    let forward = Vec3::new(angle.cos(), 0., angle.sin());
    let side = Vec3::new(-angle.sin(), 0., angle.cos());
    let line = [
        root - forward * unit * 0.19,
        root + side * unit * 0.04,
        root + forward * unit * 0.22 + side * unit * 0.02,
    ];
    for pair in line.windows(2) {
        let d = (pair[1] - pair[0]).normalize();
        let across = Vec3::new(-d.z, 0., d.x) * unit * 0.004;
        append_solid_quad(
            out,
            [
                (pair[0] - across).to_array(),
                (pair[1] - across).to_array(),
                (pair[1] + across).to_array(),
                (pair[0] + across).to_array(),
            ],
            [0., 1., 0.],
            [0.20, 0.185, 0.155, 1.],
        );
    }
    // A low weathered chip with a raised center, never a gameplay boulder.
    let center = root + side * unit * 0.16;
    let peak = center + Vec3::Y * unit * (0.025 + seed * 0.030);
    let rim: [Vec3; 6] = std::array::from_fn(|i| {
        let a = angle + i as f32 * std::f32::consts::TAU / 6.;
        center + Vec3::new(a.cos() * unit * 0.075, 0., a.sin() * unit * 0.06)
    });
    for i in 0..6 {
        let p = [rim[i], peak, rim[(i + 1) % 6]];
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        let first = out.positions.len() as u32;
        out.positions.extend(p.map(|p| p.to_array()));
        out.normals.extend([normal.to_array(); 3]);
        out.uvs.extend([[0.; 2]; 3]);
        out.colors.extend([[0.34, 0.32, 0.28, 1.]; 3]);
        out.indices.extend([first, first + 1, first + 2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(tile_index: u16) -> VisualTileSource {
        VisualTileSource {
            tileset_id: "cave".into(),
            metatile_id: 0,
            subtile_column: 0,
            subtile_row: 0,
            tile_index,
        }
    }

    fn geometry() -> GridGeometry {
        GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        }
    }

    #[test]
    fn wall_foot_blending_preserves_support_and_is_stable_across_grid_scrolling() {
        for (tileset, wall_tile) in [("cave", 0x26), ("dark_cave", 0x26), ("cave", 0x01), ("dark_cave", 0x01)] {
            let g = GridGeometry { width: 5, height: 1, ..geometry() };
            let mut wall = source(wall_tile); wall.tileset_id = tileset.into();
            wall.metatile_id = if wall_tile == 0x26 { 0x2e } else { 0 };
            let mut floor = source(0x16); floor.tileset_id = tileset.into();
            let tiles: Vec<_> = (0..5).map(|column| VisualTile {
                column, row: 0, source: if column == 0 { wall.clone() } else { floor.clone() },
                texture: Default::default(), animation_frames: None, priority: false,
            }).collect();
            let cells: Vec<_> = tiles.iter().collect();
            let mut mesh = SurfaceMeshData::default();
            for column in [1, 4] {
                let x = column as f32 * 8.;
                let y = if column == 1 { 0. } else { 6. };
                assert!(append(&mut mesh, [[x, y, 0.], [x, y, 8.], [x+8., y, 8.], [x+8., y, 0.]],
                    [0.,1.,0.], &floor, &g, [0,0]));
            }
            let original = mesh.clone();
            blend_wall_feet(&mut mesh, &cells, &g, [0,0]);
            assert_eq!(mesh.positions, original.positions);
            assert_eq!(mesh.normals, original.normals);
            assert_eq!(mesh.indices, original.indices);
            assert_eq!(mesh.uvs, original.uvs);
            assert_ne!(mesh.colors[0], original.colors[0]);
            assert_eq!(mesh.colors.last(), original.colors.last());
            assert!(mesh.colors.iter().all(|c| c[3] == 1.));
            let mut shifted = original.clone();
            for p in &mut shifted.positions { p[0] -= 16.; }
            let scrolled = GridGeometry { origin_x: -16., ..g };
            blend_wall_feet(&mut shifted, &cells, &scrolled, [0,0]);
            assert_eq!(shifted.colors, mesh.colors);
            let mut water = original.clone();
            water.uvs.fill([0.,0.]);
            blend_wall_feet(&mut water, &cells, &g, [0,0]);
            assert_eq!(water.colors, original.colors);
            if wall_tile == 0x01 {
                let mut plateau = original.clone();
                for p in &mut plateau.positions { p[1] = 6.; }
                blend_wall_feet(&mut plateau, &cells, &g, [0,0]);
                assert_eq!(plateau.colors, original.colors);
            }
            let mut unproven = tiles.clone();
            unproven[0].source = source(0x26);
            unproven[0].source.tileset_id = tileset.into();
            unproven[0].source.metatile_id = 0x09;
            let refs: Vec<_> = unproven.iter().collect();
            let mut untouched = original.clone();
            blend_wall_feet(&mut untouched, &refs, &g, [0,0]);
            assert_eq!(untouched.colors, original.colors);
        }
    }

    #[test]
    fn weathered_shelf_lips_join_cap_and_body_without_moving_either_walking_center() {
        let g = GridGeometry { width: 2, height: 1, ..geometry() };
        let cells: Vec<_> = (0..2).map(|column| VisualTile {
            column, row: 0, source: source(if column == 0 { 0x01 } else { 0x16 }),
            texture: Default::default(), animation_frames: None, priority: false,
        }).collect();
        let refs: Vec<_> = cells.iter().collect();
        let mut mesh = SurfaceMeshData::default();
        mesh.positions = vec![[8.,6.,4.], [8.,6.,4.], [4.,6.,4.], [12.,0.,4.], [8.,0.,4.], [8.,6.,4.]];
        mesh.uvs = mesh.positions.iter().enumerate().map(|(i,p)| {
            if i == 5 { [0.2,0.4] }
            else if i == 1 { cave_cliff::stone_uv(Vec3::from_array(*p), &g, [0,0], false) }
            else { cave_cliff::floor_uv(Vec3::from_array(*p), &g, [0,0], false) }
        }).collect();
        let original = mesh.positions.clone();
        weather_shelf_lips(&mut mesh, &refs, &g, [0,0]);
        assert_eq!(mesh.positions[0], mesh.positions[1]);
        assert!(mesh.positions[0][0] > 8. && mesh.positions[0][0] < 10.);
        assert_eq!(mesh.positions[0][1], 6.);
        assert_eq!(mesh.positions[2..5], original[2..5]);
        assert_eq!(mesh.positions[0], mesh.positions[5]);
        assert_eq!(mesh.uvs[5], [0.2,0.4]);
        let mut shifted = mesh.clone();
        shifted.positions = original.iter().map(|p| [p[0]-16., p[1], p[2]]).collect();
        weather_shelf_lips(&mut shifted, &refs, &GridGeometry { origin_x: -16., ..g }, [0,0]);
        for (a,b) in shifted.positions.iter().zip(&mesh.positions) {
            assert!((Vec3::from_array(*a) + Vec3::X*16. - Vec3::from_array(*b)).length() < 0.00001);
        }
    }

    #[test]
    fn shallow_contour_excludes_caps_ramps_water_and_incomplete_intervals() {
        let g=geometry();
        let p=[[0.,0.,8.],[8.,0.,8.],[8.,6.,8.],[0.,6.,8.]];
        assert!(shallow_face(&p,[0.,0.,1.],&source(0x16),&g).is_some());
        assert!(shallow_face(&p,[0.,1.,0.],&source(0x16),&g).is_none());
        assert!(shallow_face(&p,[0.,0.,1.],&source(0x14),&g).is_none());
        let mut incomplete=p; incomplete[0][1]=2.;
        assert!(shallow_face(&incomplete,[0.,0.,1.],&source(0x16),&g).is_none());
        let mut wall=SurfaceMeshData::default();
        append(&mut wall,[[0.,0.,8.],[8.,0.,8.],[8.,16.,8.],[0.,16.,8.]],
            [0.,0.,1.],&source(0x16),&g,[0,0]);
        assert!(wall.uvs.iter().all(|uv|(uv[1]/100000.).round()==6.));
    }
    #[test]
    fn finish_preserves_floor_and_shelf_datums_and_face_extent() {
        let g = geometry();
        for (tile, y) in [(0x16, 0.0), (0x01, 6.0), (0x16, 16.0)] {
            let p = [[0.0, y, 0.0], [0.0, y, 8.0], [8.0, y, 8.0], [8.0, y, 0.0]];
            let mut out = SurfaceMeshData::default();
            assert!(append(
                &mut out,
                p,
                [0.0, 1.0, 0.0],
                &source(tile),
                &g,
                [0, 0]
            ));
            assert!(
                out.positions.iter().all(|v| v[1] == y
                    && (0.0..=8.0).contains(&v[0])
                    && (0.0..=8.0).contains(&v[2]))
            );
            for corner in p {
                assert!(out.positions.contains(&corner));
            }
            assert_eq!(out.indices.len(), 54);
        }
    }

    #[test]
    fn enclosing_cap_and_wall_share_the_outside_rock_contour() {
        let g = geometry();
        let cap = [[0., 40., 0.], [0., 40., 8.], [8., 40., 8.], [8., 40., 0.]];
        let face = [[0., 0., 8.], [0., 0., 0.], [0., 40., 0.], [0., 40., 8.]];
        let mut top = SurfaceMeshData::default();
        let mut wall = SurfaceMeshData::default();
        assert!(append(
            &mut top,
            cap,
            [0., 1., 0.],
            &source(0x16),
            &g,
            [10, 20]
        ));
        assert!(append(
            &mut wall,
            face,
            [-1., 0., 0.],
            &source(0x16),
            &g,
            [10, 20]
        ));
        for z in 0..=4 {
            let p =
                cave_cliff::rock_point(Vec3::new(0., 40., z as f32 * 2.), &g, [10, 20]).to_array();
            assert!(top.positions.contains(&p));
            assert!(wall.positions.contains(&p));
        }
        assert!(top.positions.iter().any(|p| (p[1] - 40.).abs() > 0.1));
        assert!(
            top.normals
                .iter()
                .chain(&wall.normals)
                .all(|n| Vec3::from_array(*n).is_finite())
        );
        assert!(wall.positions.contains(&[0., 0., 0.]));
        assert!(wall.positions.contains(&[0., 0., 8.]));
    }

    #[test]
    fn water_ladders_unknown_art_and_other_tilesets_keep_their_source() {
        let g = geometry();
        let p = [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 8.0],
            [8.0, 0.0, 8.0],
            [8.0, 0.0, 0.0],
        ];
        for tile in [0x14, 0x2a, 0x3a, 0x26, 0x33] {
            assert!(!append(
                &mut SurfaceMeshData::default(),
                p,
                [0.0, 1.0, 0.0],
                &source(tile),
                &g,
                [0, 0]
            ));
        }
        let mut other = source(0x16);
        other.tileset_id = "johto".into();
        assert!(!append(
            &mut SurfaceMeshData::default(),
            p,
            [0.0, 1.0, 0.0],
            &other,
            &g,
            [0, 0]
        ));
    }

    #[test]
    fn mineral_color_is_stable_when_published_grid_scrolls() {
        let a = geometry();
        let mut b = geometry();
        b.origin_x = -8.0;
        let n = [0.0, 1.0, 0.0];
        assert_eq!(
            color([4.0, 0.0, 3.0], n, false, &a, [10, 5]),
            color([-4.0, 0.0, 3.0], n, false, &b, [10, 5])
        );
        assert_ne!(
            color([4.0, 0.0, 3.0], n, false, &a, [10, 5]),
            color([4.0, 0.0, 3.0], n, true, &a, [10, 5])
        );
    }
}
