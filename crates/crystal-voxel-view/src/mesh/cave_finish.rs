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

pub(in crate::mesh) fn color(
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
        // Match the curved scarp's horizontal knots. The weathering pass
        // bends these shared edges, so different subdivisions leave slivers
        // even when their original endpoints are identical.
        let u = u as f32 / 4.0;
        let v = v as f32 / 4.0;
        let a = Vec3::from_array(points[0]).lerp(Vec3::from_array(points[3]), u);
        let b = Vec3::from_array(points[1]).lerp(Vec3::from_array(points[2]), u);
        a.lerp(b, v).to_array()
    };
    for v in 0..4 {
        for u in 0..4 {
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
            // Authenticated six-over-sixteen cave flights share the same dust
            // transition as their lower landing. Keep vertical rock excluded.
            || out.normals[i][1] < 0.93 { continue; }
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

// Round only a proven convex shelf corner: one raised quadrant surrounded
// by three known lower-floor quadrants. The source walking center stays
// outside the radius, and the lower seam stays fixed on its original datum.
fn rounded_shelf_corner(
    p: Vec3, cells: &[&VisualTile], g: &GridGeometry,
) -> (Vec3, f32) {
    let height=crate::cave::CAVE_SHELF_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
    if p.y<=0. || p.y>height+0.001 { return (Vec3::ZERO,1.); }
    let x=(p.x-g.origin_x)/g.tile_width;
    let z=(p.z-g.origin_z)/g.tile_height;
    let cx=x.round() as i32;
    let cz=z.round() as i32;
    let radius=0.4;
    if cx<=0 || cz<=0 || cx>=g.width as i32 || cz>=g.height as i32
        || (x-cx as f32).abs()>radius || (z-cz as f32).abs()>radius {
        return (Vec3::ZERO,1.);
    }
    let indices=[(cx-1,cz-1),(cx,cz-1),(cx-1,cz),(cx,cz)]
        .map(|(x,z)|z as usize*g.width+x as usize);
    let tileset=&cells[indices[0]].source.tileset_id;
    let mut raised=[false;4];
    for (quadrant,index) in indices.into_iter().enumerate() {
        if &cells[index].source.tileset_id!=tileset {return (Vec3::ZERO,1.);}
        match crate::cave::cave_shape(&cells[index].source) {
            Some(shape) if shape.surface_height(SOURCE_TILE_HEIGHT)==crate::cave::CAVE_SHELF_HEIGHT => {
                raised[quadrant]=true;
            }
            Some(CellShape::Flat) if cells[index].source.tile_index==0x16 => {},
            _ => return (Vec3::ZERO,1.),
        }
    }
    // Both ends of a bank need an arc. A solitary raised quadrant is a
    // convex lip; a solitary lower quadrant is a recessed inside corner.
    // Two raised quadrants form a straight seam or a diagonal contact and
    // cannot authorize a corner fill.
    let count=raised.into_iter().filter(|s|*s).count();
    let quadrant=match count {
        1 => raised.iter().position(|s|*s).unwrap(),
        3 => raised.iter().position(|s|!*s).unwrap(),
        _ => return (Vec3::ZERO,1.),
    };
    let sx=if quadrant%2==0 {-1.}else{1.};
    let sz=if quadrant<2 {-1.}else{1.};
    let u=(x-cx as f32)*sx;
    let v=(z-cz as f32)*sz;
    // Existing shoulder vertices can sit slightly outside the footprint.
    // Applying the same arc to them keeps the cap and curved body connected.
    // For a recessed corner, cap samples lie in the three rock quadrants.
    // Project the adjacent boundary, then feather that displacement into the
    // rock. Projecting the entire cap directly onto the floor-side circle
    // would jump at the support boundary and could fold the plateau.
    let sample=if count==3 {bevy::prelude::Vec2::new(u.max(0.),v.max(0.))}
        else {bevy::prelude::Vec2::new(u,v)};
    let offset=sample-bevy::prelude::Vec2::splat(radius);
    let distance=offset.length();
    if distance<=radius {return (Vec3::ZERO,1.);}
    let rounded=bevy::prelude::Vec2::splat(radius)+offset*(radius/distance);
    let t=(p.y/height).clamp(0.,1.);
    let feather=|a:f32| {let w=(1.+a.min(0.)/radius).clamp(0.,1.);w*w*(3.-2.*w)};
    let blend=t*t*(3.-2.*t)*if count==3 {feather(u)*feather(v)}else{1.};
    (Vec3::new((rounded.x-sample.x)*sx*g.tile_width,0.,(rounded.y-sample.y)*sz*g.tile_height)*blend,
        ((u.max(0.)+v.max(0.))/(2.*radius)).clamp(0.,1.))
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
        let p = Vec3::from_array(out.positions[i]);
        if p.y <= 0. || p.y > height+0.001 { continue; }
        let kind=(out.uvs[i][1]/100000.).round();
        let (rounding, lip_weight)=if out.uvs[i][0]>90000. && matches!(kind,4. | 6. | 7.) {
            rounded_shelf_corner(p,cells,g)
        } else {(Vec3::ZERO,1.)};
        let on_lip=(p.y-height).abs()<0.001;
        if !on_lip && rounding==Vec3::ZERO {continue;}
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
                    if on_edge && on_lip {
                        if dx != 0 { direction.x = dx as f32; } else { direction.z = dz as f32; }
                    }
                }
            }
        }
        let relief = 0.10 + cave_cliff::slab((x + origin[0] as f32) * 0.74 + 19.,
            (z + origin[1] as f32) * 0.74 - 23.) * 0.08;
        let delta = rounding + direction * Vec3::new(g.tile_width, 0., g.tile_height) * relief * lip_weight;
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
                (pair[0] + across).to_array(),
                (pair[1] + across).to_array(),
                (pair[1] - across).to_array(),
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
    fn cave_floor_fractures_and_chips_face_up_at_both_walking_datums() {
        let g = geometry();
        for height in [0., crate::cave::CAVE_SHELF_HEIGHT] {
            let mut found = false;
            for x in 0..32 {
                let mut mesh = SurfaceMeshData::default();
                floor_detail(&mut mesh,
                    [[0.,height,0.], [0.,height,8.], [8.,height,8.], [8.,height,0.]],
                    [0.,1.,0.], &source(0x16), &g, [x,0]);
                if mesh.indices.is_empty() { continue; }
                found = true;
                for triangle in mesh.indices.chunks_exact(3) {
                    let p: [Vec3;3] = std::array::from_fn(|i|
                        Vec3::from_array(mesh.positions[triangle[i] as usize]));
                    let winding = (p[1]-p[0]).cross(p[2]-p[0]);
                    assert!(winding.y > 0., "floor detail must face the cave camera");
                    for &index in triangle {
                        assert!(winding.dot(Vec3::from_array(mesh.normals[index as usize])) > 0.);
                    }
                    assert!(p.iter().all(|p| p.y >= height));
                }
            }
            assert!(found, "exercise a seeded floor fracture");
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
            let mut ramp=original.clone();
            let slope=Vec3::new(0.,16.,6.).normalize().to_array();
            ramp.normals.fill(slope);
            let ramp_positions=ramp.positions.clone();
            blend_wall_feet(&mut ramp,&cells,&g,[0,0]);
            assert_eq!(ramp.colors,mesh.colors,"ramp dust must close against the floor at its landing");
            assert_eq!(ramp.positions,ramp_positions);
            assert!(ramp.normals.iter().all(|n|*n==slope));
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
    fn cave_weathered_cap_and_scarp_share_every_lip_sample() {
        let g = GridGeometry { height: 2, ..geometry() };
        let tiles: Vec<_> = (0..2).map(|row| VisualTile {
            column: 0, row, source: source(if row == 0 { 0x01 } else { 0x16 }),
            texture: Default::default(), animation_frames: None, priority: false,
        }).collect();
        let cells: Vec<_> = tiles.iter().collect();
        let shapes = [CellShape::RaisedTop { height: 6., solid: SolidKind::Bank }, CellShape::Flat];
        let mut mesh = TerrainMeshData::default();
        modeled_dungeons::append_terrain(&mut mesh, "SilverCaveRoom3", &cells,
            &shapes, &g, &mut [false; 2], [0, 0]);
        weather_shelf_lips(&mut mesh.solid, &cells, &g, [0, 0]);
        let edge = |kind: f32| {
            let mut points: Vec<_> = mesh.solid.positions.iter().zip(&mesh.solid.uvs)
                .filter(|(p, uv)| p[1] == 6. && p[2] >= 8.
                    && (uv[1] / 100000.).round() == kind)
                .map(|(p, _)| *p).collect();
            points.sort_by(|a,b| a[0].total_cmp(&b[0]));
            points.dedup();
            points
        };
        let cap = edge(4.);
        let body = edge(6.);
        assert_eq!(cap.len(), 5, "the cap must sample the curved side's complete lip");
        assert_eq!(cap, body, "weathering must not open cap-to-body slivers");
    }

    #[test]
    fn convex_shelf_corners_round_shared_caps_and_bodies_but_keep_support_and_unknown_edges() {
        let g=GridGeometry {width:3,height:3,..geometry()};
        for quadrant in 0..4 {
            let raised=if quadrant<2 {quadrant}else{quadrant+1};
            let mut tiles:Vec<_>=(0..9).map(|i|VisualTile {
                column:(i%3)as u32,row:(i/3)as u32,
                source:source(if i==raised {0x01}else{0x16}),
                texture:Default::default(),animation_frames:None,priority:false,
            }).collect();
            let refs:Vec<_>=tiles.iter().collect();
            let corner=Vec3::new(8.,6.,8.);
            let (delta,weight)=rounded_shelf_corner(corner,&refs,&g);
            assert!(delta.length()>1. && delta.length()<1.5);
            assert_eq!(delta.y,0.);
            assert_eq!(weight,0.);
            assert_eq!(delta.x.signum(),if quadrant%2==0 {-1.}else{1.});
            assert_eq!(delta.z.signum(),if quadrant<2 {-1.}else{1.});
            let (middle,_)=rounded_shelf_corner(Vec3::new(8.,3.,8.),&refs,&g);
            assert!((middle-delta*0.5).length()<0.00001);
            assert_eq!(rounded_shelf_corner(Vec3::new(8.,0.,8.),&refs,&g).0,Vec3::ZERO);
            let center=Vec3::new((raised%3)as f32*8.+4.,6.,(raised/3)as f32*8.+4.);
            assert_eq!(rounded_shelf_corner(center,&refs,&g).0,Vec3::ZERO);
            let mut mesh=SurfaceMeshData::default();
            mesh.positions=vec![corner.to_array();2];
            mesh.uvs=vec![cave_cliff::floor_uv(corner,&g,[0,0],false),
                cave_cliff::stone_uv(corner,&g,[0,0],false)];
            weather_shelf_lips(&mut mesh,&refs,&g,[0,0]);
            assert_eq!(mesh.positions[0],mesh.positions[1]);
            assert_eq!(mesh.positions[0],(corner+delta).to_array());
            // A pool or missing source drawing cannot authorize cutting back
            // a corner. This keeps shorelines and unpublished halo seams intact.
            let other=if raised==0 {1}else{0};
            for unknown in [0x14,0xff] {
                tiles[other].source.tile_index=unknown;
                assert_eq!(rounded_shelf_corner(corner,&tiles.iter().collect::<Vec<_>>(),&g).0,Vec3::ZERO);
            }
        }
    }

    #[test]
    fn recessed_shelf_corners_blend_both_faces_without_raising_the_floor() {
        for scale in [1.,2.] {
            let g=GridGeometry {width:3,height:3,tile_width:8.*scale,tile_height:8.*scale,
                origin_x:-17.,origin_z:29.};
            for quadrant in 0..4 {
                let lower=if quadrant<2 {quadrant}else{quadrant+1};
                let mut tiles:Vec<_>=(0..9).map(|i|VisualTile {
                    column:(i%3)as u32,row:(i/3)as u32,
                    source:source(if i==lower {0x16}else{0x01}),
                    texture:Default::default(),animation_frames:None,priority:false,
                }).collect();
                let refs:Vec<_>=tiles.iter().collect();
                let corner=Vec3::new(g.origin_x+g.tile_width,6.*scale,g.origin_z+g.tile_height);
                let (delta,weight)=rounded_shelf_corner(corner,&refs,&g);
                assert!(delta.length()>scale && delta.length()<1.5*scale);
                assert_eq!(delta.y,0.);
                assert_eq!(weight,0.);
                assert_eq!(delta.x.signum(),if quadrant%2==0 {-1.}else{1.});
                assert_eq!(delta.z.signum(),if quadrant<2 {-1.}else{1.});
                // Both walking datums and every tile center remain exact.
                for (i,tile) in tiles.iter().enumerate() {
                    let y=if i==lower {0.}else{6.*scale};
                    let center=Vec3::new(g.origin_x+(tile.column as f32+0.5)*g.tile_width,
                        y,g.origin_z+(tile.row as f32+0.5)*g.tile_height);
                    assert_eq!(rounded_shelf_corner(center,&refs,&g).0,Vec3::ZERO);
                }
                assert_eq!(rounded_shelf_corner(Vec3::new(corner.x,0.,corner.z),&refs,&g).0,Vec3::ZERO);
                let mut mesh=SurfaceMeshData::default();
                mesh.positions=vec![corner.to_array();2];
                mesh.uvs=vec![cave_cliff::floor_uv(corner,&g,[0,0],false),
                    cave_cliff::stone_uv(corner,&g,[0,0],false)];
                weather_shelf_lips(&mut mesh,&refs,&g,[0,0]);
                assert_eq!(mesh.positions[0],mesh.positions[1]);
                assert_eq!(mesh.positions[0],(corner+delta).to_array());
                // The arc must meet its unchanged support continuously on
                // the rock side, not snap at the edge of the rounding field.
                let sx=delta.x.signum();
                let sz=delta.z.signum();
                for axis in [Vec3::X*sx,Vec3::Z*sz] {
                    let edge=corner-axis*g.tile_width*0.4;
                    assert!(rounded_shelf_corner(edge,&refs,&g).0.length()<0.00001);
                    assert!(rounded_shelf_corner(edge+axis*g.tile_width*0.0001,&refs,&g).0.length()<0.00001);
                }
                // Exercise the real cap tessellation in all three rock
                // quadrants; every triangle must stay upward and nonzero.
                let mut cap=SurfaceMeshData::default();
                for tile in tiles.iter().filter(|t|t.source.tile_index==0x01) {
                    for v in 0..4 {for u in 0..4 {
                        let at=|u:f32,v:f32| [g.origin_x+(tile.column as f32+u/4.)*g.tile_width,
                            6.*scale,g.origin_z+(tile.row as f32+v/4.)*g.tile_height];
                        let points=[at(u as f32,v as f32),at(u as f32,v as f32+1.),
                            at(u as f32+1.,v as f32+1.),at(u as f32+1.,v as f32)];
                        append_quad_colors(&mut cap,points,[0.,1.,0.],
                            points.map(|p|cave_cliff::floor_uv(Vec3::from_array(p),&g,[0,0],false)),[[0.4;4];4]);
                    }}
                }
                weather_shelf_lips(&mut cap,&refs,&g,[0,0]);
                for tri in cap.indices.chunks_exact(3) {
                    let p:[Vec3;3]=std::array::from_fn(|i|Vec3::from_array(cap.positions[tri[i] as usize]));
                    assert!((p[1]-p[0]).cross(p[2]-p[0]).y>0.,"folded recessed shelf cap");
                }
                for unknown in [0x14,0xff] {
                    tiles[lower].source.tile_index=unknown;
                    assert_eq!(rounded_shelf_corner(corner,&tiles.iter().collect::<Vec<_>>(),&g).0,Vec3::ZERO);
                }
            }
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
            assert!(!out.indices.is_empty());
            for triangle in out.indices.chunks_exact(3) {
                let [a, b, c] = std::array::from_fn(|i|
                    Vec3::from_array(out.positions[triangle[i] as usize]));
                assert!((b-a).cross(c-a).y > 0., "floor triangles must remain nondegenerate and face up");
            }
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
