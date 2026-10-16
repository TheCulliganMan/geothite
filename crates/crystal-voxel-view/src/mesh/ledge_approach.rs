//! A jump bank rises across its four authored rows and drops at the south lip.
//! Keep the native footprint and compose the rise with existing end tapers.
use super::new_bark::meadow_noise;
use super::*;

fn bank(source: &VisualTileSource) -> bool {
    source.tileset_id.as_ref() == "johto"
        && source.subtile_column < 4
        && source.subtile_row < 4
        && matches!(shape_for_source(source),
            CellShape::RaisedTop { height, solid: SolidKind::Bank }
            | CellShape::LedgeBand { height, face: LedgeFace::South, .. }
            if height == crate::profile::JUMP_LEDGE_HEIGHT)
}

/// Broad phase for detached source groves. The shaping pass can inspect the
/// adjacent cell, half-unit base probes and the quarter-cell south lip. Two
/// full cells conservatively contain all of those lookups and tolerances.
pub(super) fn grove_bank_regions(cells: &[VisualTile], g: &GridGeometry) -> Vec<[f32;4]> {
    cells.iter().enumerate().filter(|(_,cell)| bank(&cell.source)).map(|(i,_)| {
        let (w,e,n,s) = g.bounds(i%g.width,i/g.width);
        [w-g.tile_width*2.,e+g.tile_width*2.,n-g.tile_height*2.,s+g.tile_height*2.]
    }).collect()
}
pub(super) fn grove_may_touch_bank(bounds: (Vec3,Vec3), regions: &[[f32;4]]) -> bool {
    if !bounds.0.is_finite() || !bounds.1.is_finite() { return true; }
    regions.iter().any(|&[w,e,n,s]|
        bounds.1.x>=w && bounds.0.x<=e && bounds.1.z>=n && bounds.0.z<=s)
}

/// Heights at west/north, west/south, east/south, east/north after the
/// authored four-row rise and optional terminal taper.
pub(super) fn cap_heights(
    source: &VisualTileSource,
    shape: CellShape,
    tile_height: f32,
) -> Option<[f32; 4]> {
    if !bank(source) {
        return None;
    }
    let scale = tile_height / SOURCE_TILE_HEIGHT;
    let [west, east] = match shape {
        CellShape::RampEast {
            west_height,
            east_height,
        } => [west_height * scale, east_height * scale],
        _ => [crate::profile::JUMP_LEDGE_HEIGHT * scale; 2],
    };
    let north = source.subtile_row as f32 / 4.;
    let south = (source.subtile_row as f32 + 1.) / 4.;
    Some([west * north, west * south, east * south, east * north])
}

/// The north wall of the first bank row disappears into the level floor.
/// Remove it before adding relief; flattening its Y afterward leaves the
/// outward earth relief and rim grass drawn across the walking surface.
pub(super) fn obsolete_toe_wall(
    positions: &[[f32; 3]; 4],
    normal: [f32; 3],
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> bool {
    if normal[2] > -0.999 || normal[1].abs() > 0.001 || normal[0].abs() > 0.001 {
        return false;
    }
    let center = positions
        .iter()
        .fold(Vec3::ZERO, |a, p| a + Vec3::from_array(*p))
        * 0.25;
    let x = ((center.x - g.origin_x) / g.tile_width).floor() as isize;
    let z = ((center.z + 0.001 - g.origin_z) / g.tile_height).floor() as isize;
    if x < 0 || z < 0 || x >= g.width as isize || z >= g.height as isize {
        return false;
    }
    let source = &cells[z as usize * g.width + x as usize].source;
    let (_, _, north, _) = g.bounds(x as usize, z as usize);
    let h = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    bank(source)
        && source.subtile_row == 0
        && positions
            .iter()
            .all(|p| (p[2] - north).abs() < 0.001 && p[1] >= 0. && p[1] <= h + 0.001)
}

pub(super) fn shape_approaches(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    shapes: &[CellShape],
    origin: [i32; 2],
) {
    if !cells.iter().any(|cell| bank(&cell.source)) {
        // Kanto banks retain source ownership for directed terrace replacement.
        // Without authored edges they keep their original geometry; consume
        // only the temporary tag before the material/batching boundary.
        for surface in [&mut mesh.solid,&mut mesh.textured,
            &mut mesh.animated_solid,&mut mesh.animated_textured] {
            for uv in &mut surface.uvs {
                if uv[0]==BANK_OWNER_UV {*uv=[0.;2];}
            }
        }
        return;
    }
    for (i, cell) in cells.iter().enumerate() {
        if bank(&cell.source) {
            mesh.footing_heights[i] *= (cell.source.subtile_row as f32 + 0.5) / 4.0;
        }
    }
    for surface in [
        &mut mesh.textured,
        &mut mesh.solid,
        &mut mesh.animated_textured,
        &mut mesh.animated_solid,
    ] {
        shape_surface(surface, cells, g);
    }
    // The original constant-height bank run omitted faces between adjacent
    // jump drawings. Each sawtooth reset now needs its own closed south lip.
    for (i, cell) in cells.iter().enumerate() {
        if !bank(&cell.source) || cell.source.subtile_row != 3 {
            continue;
        }
        let x = i % g.width;
        let z = i / g.width;
        if z + 1 >= g.height {
            continue;
        }
        let next = cells[i + g.width];
        if !bank(&next.source) || next.source.subtile_row != 0 {
            continue;
        }
        let (w, e, _, south) = g.bounds(x, z);
        let scale = g.tile_height / SOURCE_TILE_HEIGHT;
        let [west, east] = match shapes[i] {
            CellShape::RampEast {
                west_height,
                east_height,
            } => [west_height * scale, east_height * scale],
            _ => [crate::profile::JUMP_LEDGE_HEIGHT * scale; 2],
        };
        let p = [
            [e, 0.0, south],
            [e, east, south],
            [w, west, south],
            [w, 0.0, south],
        ];
        if !new_bark::ledge_finish::append(&mut mesh.solid, p, [0.0, 0.0, 1.0], g, origin) {
            append_solid_quad(&mut mesh.solid, p, [0.0, 0.0, 1.0], [0.29, 0.19, 0.08, 1.0]);
        }
    }
    soften_lips(mesh, cells, g, shapes, origin);
}

// Temporary source ownership for relieved earth and its grass. The slope
// pass consumes this marker before meshes reach material or batching code.
pub(super) const BANK_OWNER_UV: f32 = -4096.;

pub(super) fn mark_bank_surface(s: &mut SurfaceMeshData, first: usize, owner: usize) {
    s.uvs[first..].fill([BANK_OWNER_UV, owner as f32]);
}

fn shape_surface(s: &mut SurfaceMeshData, cells: &[&VisualTile], g: &GridGeometry) {
    let original = s.positions.clone();
    let mut changed = vec![false; original.len()];
    let mut toe_blend = vec![None; original.len()];
    let mut cap_normals = vec![None; original.len()];
    let h = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / 8.0;
    for triangle in s.indices.chunks_exact(3) {
        let ids = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        let p = ids.map(|i| Vec3::from_array(original[i]));
        let n = Vec3::from_array(s.normals[ids[0]]);
        let source_owner = ids.iter().all(|&i| s.uvs[i][0] == BANK_OWNER_UV)
            .then(|| s.uvs[ids[0]][1] as usize)
            .filter(|&i| i < cells.len() && bank(&cells[i].source));
        let center = (p[0] + p[1] + p[2]) / 3.0 - n * 0.001;
        let x = ((center.x - g.origin_x) / g.tile_width).floor() as isize;
        let z = ((center.z - g.origin_z) / g.tile_height).floor() as isize;
        if source_owner.is_none()
            && (x < 0 || z < 0 || x >= g.width as isize || z >= g.height as isize)
        {
            continue;
        }
        let mut owner = source_owner.unwrap_or_else(|| z as usize * g.width + x as usize);
        // A relieved cliff face can protrude across the bank's cell boundary.
        // Its centroid may therefore belong to the bank even though its high
        // vertices belong to the cliff. Lower its base and retain its top.
        let mut boundary_wall = source_owner.is_none()
            && n.y.abs() < 0.9 && p.iter().any(|v| v.y > h + 2.0);
        // Native side bands can carry a painted normal which points into the
        // neighboring cell. Resolve a boundary wall from either side of its
        // plane rather than leaving the old vertical back of a bank standing.
        if !bank(&cells[owner].source) {
            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x + dx;
                let nz = z + dz;
                if nx < 0 || nz < 0 || nx >= g.width as isize || nz >= g.height as isize {
                    continue;
                }
                let candidate = nz as usize * g.width + nx as usize;
                let (w, e, n, south) = g.bounds(nx as usize, nz as usize);
                let on_plane = if dx != 0 {
                    p.iter()
                        .all(|v| (v.x - if dx < 0 { e } else { w }).abs() < 0.002)
                } else {
                    p.iter()
                        .all(|v| (v.z - if dz < 0 { south } else { n }).abs() < 0.002)
                };
                if on_plane && bank(&cells[candidate].source) {
                    owner = candidate;
                    boundary_wall = true;
                    break;
                }
            }
        }
        // Sculpted cliffs lean away from their base, so their triangles need
        // not lie in one grid plane. Their low boundary vertices still identify
        // the bank which formerly concealed the bottom of the wall.
        if !bank(&cells[owner].source) {
            'base: for v in p.iter().filter(|v| (v.y - h).abs() < 0.05) {
                for dx in [-0.5, 0.0, 0.5] {
                    for dz in [-0.5, 0.0, 0.5] {
                        let bx = ((v.x + dx - g.origin_x) / g.tile_width).floor() as isize;
                        let bz = ((v.z + dz - g.origin_z) / g.tile_height).floor() as isize;
                        if bx < 0 || bz < 0 || bx >= g.width as isize || bz >= g.height as isize {
                            continue;
                        }
                        let candidate = bz as usize * g.width + bx as usize;
                        if bank(&cells[candidate].source) {
                            owner = candidate;
                            boundary_wall = true;
                            break 'base;
                        }
                    }
                }
            }
        }
        let cell = cells[owner];
        if !bank(&cell.source)
            || p.iter()
                .any(|p| p.y < -0.05 || (!boundary_wall && p.y > h + 2.0))
        {
            continue;
        }
        let north = g.origin_z
            + ((owner / g.width) as f32 - cell.source.subtile_row as f32) * g.tile_height;
        for i in ids {
            let mut v = Vec3::from_array(original[i]);
            let rise = ((v.z - north) / (4.0 * g.tile_height)).clamp(0.0, 1.0);
            if !boundary_wall && n.y > 0.9 {
                // Transform the source normal by the ramp's differential.
                // Triangle-size averaging makes a shared cap corner shade
                // differently before and after material tessellation.
                let native = Vec3::from_array(s.normals[i]);
                let vertical_scale = if v.y <= h { rise } else { 1. };
                cap_normals[i] = Some(Vec3::new(
                    native.x * vertical_scale,
                    native.y,
                    native.z * vertical_scale - native.y * v.y.min(h) / (4.0 * g.tile_height),
                ).normalize());
            }
            v.y = if v.y <= h {
                v.y * rise
            } else if v.y <= h + 0.05 {
                // Ground finishing sits a tiny distance above its source
                // plane. It must follow the same ramp as the native cap;
                // treating this offset as a tall wall left floating strips.
                h * rise + (v.y - h)
            } else if boundary_wall {
                v.y
            } else {
                v.y + h * (rise - 1.0)
            };
            s.positions[i] = v.to_array();
            changed[i] = v.y != original[i][1];
            if n.y > 0.9 && original[i][1] >= h - 0.05 && !boundary_wall {
                // The bank meets level lawn at its north toe. Blend shading
                // through the first source row, keeping the authored linear
                // rise and the sharp south drop unchanged.
                let t = ((v.z - north) / g.tile_height).clamp(0., 1.);
                toe_blend[i] = Some(t * t * (3. - 2. * t));
            }
        }
    }
    for uv in &mut s.uvs {
        if uv[0] == BANK_OWNER_UV {
            *uv = [0.; 2];
        }
    }
    refresh_normals(s, &changed);
    for (i, normal) in cap_normals.into_iter().enumerate() {
        if let Some(normal) = normal {
            s.normals[i] = normal.to_array();
        }
    }
    for (i, blend) in toe_blend.into_iter().enumerate() {
        if let Some(t) = blend {
            s.normals[i] = Vec3::Y.lerp(Vec3::from_array(s.normals[i]), t)
                .normalize().to_array();
        }
    }
}

/// A detached grove has no shared vertices with the remaining terrain. Keep
/// its final bank deformation identical without updating global footing twice
/// or appending another copy of the bank's closure surfaces.
pub(super) fn finish_detached_grove(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    shapes: &[CellShape],
    origin: [i32; 2],
) {
    finish_detached_grove_with_claims(mesh, cells, g, shapes, origin, None);
}

pub(super) fn finish_detached_grove_with_claims(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    shapes: &[CellShape],
    origin: [i32; 2],
    claims: Option<&DeferredClaims>,
) {
    if !cells.iter().any(|cell| bank(&cell.source)) {
        return;
    }
    shape_surface(&mut mesh.solid, cells, g);
    soften_lips_with_claims(mesh, cells, g, shapes, origin, claims);
}

fn refresh_normals(s: &mut SurfaceMeshData, changed: &[bool]) {
    if !changed.iter().any(|&moved| moved) {
        return;
    }
    let mut normals = vec![Vec3::ZERO; s.positions.len()];
    for triangle in s.indices.chunks_exact(3) {
        let ids = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        if !ids.iter().any(|i| changed[*i]) {
            continue;
        }
        let p = ids.map(|i| Vec3::from_array(s.positions[i]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for i in ids {
            normals[i] += n;
        }
    }
    for (i, n) in normals.iter().enumerate() {
        if n.length_squared() > 1e-12 {
            s.normals[i] = n.normalize().to_array();
        }
    }
}

/// A shallow rim recess shared by the cap and its earth face. Half-cell
/// linear knots occur in both cap and face tessellations, so different edge
/// subdivisions still follow exactly the same curve.
// Share linear half-cell knots between the coarse cap and finer earth face.
// A curved function sampled separately on those surfaces would open cracks.
// Keep run ends, modeled contacts, ground vertices and cell centers anchored.
fn soften_lips(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    shapes: &[CellShape],
    origin: [i32; 2],
) {
    soften_lips_with_claims(mesh, cells, g, shapes, origin, None);
}

fn soften_lips_with_claims(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    shapes: &[CellShape],
    origin: [i32; 2],
    claims: Option<&DeferredClaims>,
) {
    let full = |i: usize| {
        bank(&cells[i].source)
            && cells[i].source.subtile_row == 3
            && !matches!(shapes[i], CellShape::RampEast { .. })
            && claims.map_or_else(|| mesh
                .authored_cells
                .get(i)
                .is_none_or(|claim| claim.is_none()), |claims| claims.unclaimed(i))
    };
    let interior = |i: usize| {
        let col = i % g.width;
        col > 0 && col + 1 < g.width && full(i) && full(i - 1) && full(i + 1)
    };
    let mut rims = vec![None; cells.len()];
    for i in 0..cells.len() {
        if !interior(i) {
            continue;
        }
        let col = i % g.width;
        let row = i / g.width;
        let (w, e, _, south) = g.bounds(col, row);
        let amplitude = |fraction: f32| {
            let x = col as f32 + fraction + origin[0] as f32;
            let z = row as f32 + 1. + origin[1] as f32;
            // A broad worn edge with smaller source-anchored irregularity.
            // Nearly constant recession still read as a machined straight
            // strip at 45 degrees. Shared half-cell knots keep the cap and
            // detailed face sealed while revealing shallow natural notches.
            g.tile_height * 0.10 * (0.55 + (x * 1.31 + z * 0.73).sin() * 0.28
                + meadow_noise(x * 0.83 + 17., z * 0.71 - 29.) * 0.30)
        };
        rims[i] = Some((
            w,
            e,
            south,
            [
                if interior(i - 1) { amplitude(0.) } else { 0. },
                amplitude(0.5),
                if interior(i + 1) { amplitude(1.) } else { 0. },
            ],
        ));
    }
    if rims.iter().all(Option::is_none) {
        return;
    }
    let h = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
    for s in [
        &mut mesh.solid,
        &mut mesh.textured,
        &mut mesh.animated_solid,
        &mut mesh.animated_textured,
    ] {
        for (i, p) in s.positions.iter_mut().enumerate() {
            if p[1] <= 0. || p[1] > h + g.tile_height * 0.5 {
                continue;
            }
            let x = ((p[0] - g.origin_x) / g.tile_width).floor() as isize;
            let z = ((p[2] - g.origin_z - 0.001) / g.tile_height).floor() as isize;
            if x < 0 || z < 0 || x >= g.width as isize {
                continue;
            }
            let rim = [z, z - 1]
                .into_iter()
                .filter(|&row| row >= 0 && row < g.height as isize)
                .filter_map(|row| rims[row as usize * g.width + x as usize])
                .find(|&(_, _, south, _)| p[2] <= south + g.tile_height * 0.25);
            let Some((w, e, south, knots)) = rim else {
                continue;
            };
            let across = ((p[0] - w) / (e - w) * 2.).clamp(0., 2.);
            let segment = (across.floor() as usize).min(1);
            let t = across - segment as f32;
            let amplitude = knots[segment] * (1. - t) + knots[segment + 1] * t;
            let edge = (1. - (south - p[2]).max(0.) / (g.tile_height * 0.5)).clamp(0., 1.);
            let rise = (p[1] / h).clamp(0., 1.);
            let shift = amplitude * edge * rise * rise;
            let dx = (knots[segment + 1] - knots[segment]) * 2. / (e - w);
            let dy = if p[1] <= h { 2. * rise / h } else { 0. };
            let dz = if edge > 0. && p[2] <= south { 2. / g.tile_height } else { 0. };
            let a = -dx * edge * rise * rise;
            let b = -amplitude * edge * dy;
            let c = 1. - amplitude * dz * rise * rise;
            let normal = Vec3::from_array(s.normals[i]);
            // Cofactor of the lip warp: preserve one source-coordinate
            // normal field across coarse caps and detailed earth faces.
            s.normals[i] = Vec3::new(
                c * normal.x - a * normal.z,
                c * normal.y - b * normal.z,
                normal.z,
            ).normalize().to_array();
            p[2] -= shift;
        }
    }
}

#[cfg(test)]
mod normal_fields {
    use super::*;

    #[test]
    fn grove_bank_broad_phase_keeps_exact_near_and_far_surfaces() {
        let frame = super::super::tests::frame(4,4,(0..4).flat_map(|row|
            (0..4).map(move |column| super::super::tests::source_with_tile(
                0x56,column,row,if row==3 {0x4c} else {0x06}))).collect());
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|cell|shape_for_source(&cell.source)).collect::<Vec<_>>();
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:-16.,origin_z:-16.};
        let regions=grove_bank_regions(&frame.tiles,&g);
        assert!(!regions.is_empty());
        for bounds in [[-8.,8.,-8.,8.],[1000.,1016.,1000.,1016.]] {
            let mut raw=TerrainMeshData::default();
            crate::foliage::append_grove_tree(&mut raw,bounds,3.,8.,[8,12],[true,false,true,false]);
            let low=raw.solid.positions.iter().map(|p|Vec3::from_array(*p)).fold(Vec3::splat(f32::INFINITY),Vec3::min);
            let high=raw.solid.positions.iter().map(|p|Vec3::from_array(*p)).fold(Vec3::splat(f32::NEG_INFINITY),Vec3::max);
            let touches=grove_may_touch_bank((low,high),&regions);
            assert_eq!(touches,bounds[0]<100.);
            let mut reference=raw.clone();
            finish_detached_grove(&mut reference,&cells,&g,&shapes,[0,0]);
            if touches {finish_detached_grove(&mut raw,&cells,&g,&shapes,[0,0]);}
            assert_eq!(raw.solid.positions.iter().flatten().map(|v|v.to_bits()).collect::<Vec<_>>(),
                reference.solid.positions.iter().flatten().map(|v|v.to_bits()).collect::<Vec<_>>());
            assert_eq!(raw.solid.normals.iter().flatten().map(|v|v.to_bits()).collect::<Vec<_>>(),
                reference.solid.normals.iter().flatten().map(|v|v.to_bits()).collect::<Vec<_>>());
            assert_eq!(raw.solid.indices,reference.solid.indices);
        }
    }

    #[test]
    fn shared_cap_normals_do_not_depend_on_material_tessellation() {
        let frame = super::super::tests::frame(4, 4, (0..4).flat_map(|row| {
            (0..4).map(move |column| super::super::tests::source_with_tile(
                0x56, column, row, if row == 3 { 0x4c } else { 0x06 },
            ))
        }).collect());
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|cell| shape_for_source(&cell.source)).collect();
        let g = GridGeometry {
            width: 4, height: 4, tile_width: 8., tile_height: 8.,
            origin_x: -16., origin_z: -16.,
        };
        for height in [6., 6.012] {
            for origin in [[0, 0], [20, -11]] {
                let build = |steps: usize| {
                    let mut mesh = TerrainMeshData::default();
                    for row in 0..4 {
                        for column in 0..4 {
                            let (w, e, n, south) = g.bounds(column, row);
                            for z in 0..steps {
                                for x in 0..steps {
                                    let x0 = w + (e - w) * x as f32 / steps as f32;
                                    let x1 = if x + 1 == steps { e }
                                        else { w + (e - w) * (x + 1) as f32 / steps as f32 };
                                    let z0 = n + (south - n) * z as f32 / steps as f32;
                                    let z1 = if z + 1 == steps { south }
                                        else { n + (south - n) * (z + 1) as f32 / steps as f32 };
                                    append_quad_colors(&mut mesh.solid,
                                        [[x0, height, z0], [x0, height, z1],
                                            [x1, height, z1], [x1, height, z0]],
                                        [0., 1., 0.], [[0.; 2]; 4], [[1.; 4]; 4]);
                                }
                            }
                        }
                    }
                    shape_surface(&mut mesh.solid, &cells, &g);
                    soften_lips(&mut mesh, &cells, &g, &shapes, origin);
                    mesh
                };
                let coarse = build(1);
                let detailed = build(6);
                for (position, normal) in coarse.solid.positions.iter().zip(&coarse.solid.normals) {
                    assert!(detailed.solid.positions.iter().zip(&detailed.solid.normals)
                        .any(|(p, n)| position == p && normal == n),
                        "shared source corner {position:?} must retain its normal across detail levels");
                    assert!((Vec3::from_array(*normal).length() - 1.).abs() < 0.00001);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn undirected_kanto_bank_clears_only_temporary_material_ownership() {
        let mut source=super::super::tests::source_with_tile(0x07,0,0,0x2c);
        source.tileset_id="kanto".into();
        let frame=super::super::tests::frame(1,1,vec![source]);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let g=GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData {footing_heights:vec![6.],..Default::default()};
        append_solid_quad(&mut mesh.solid,[[0.,0.,0.],[0.,6.,0.],[8.,6.,0.],[8.,0.,0.]],
            [0.,0.,-1.],[0.4,0.3,0.2,1.]);
        mark_bank_surface(&mut mesh.solid,0,0);
        let before=mesh.solid.clone();
        shape_approaches(&mut mesh,&cells,&g,&[CellShape::Flat],[0,0]);
        assert_eq!(mesh.solid.positions,before.positions);assert_eq!(mesh.solid.normals,before.normals);
        assert_eq!(mesh.solid.colors,before.colors);assert_eq!(mesh.solid.indices,before.indices);
        assert!(mesh.solid.uvs.iter().all(|uv|*uv==[0.;2]));assert_eq!(mesh.footing_heights,[6.]);
    }

    #[test]
    fn relieved_bank_sod_follows_its_source_without_extra_upright_foliage() {
        let frame = super::super::tests::frame(2, 4, (0..4).flat_map(|row| {
            [super::super::tests::source_with_tile(0x56, 1, row,
                if row == 3 { 0x4c } else { 0x06 }),
             super::super::tests::source_with_tile(0x01, 0, 0, 0x01)]
        }).collect());
        let cells: Vec<_> = frame.tiles.iter().collect();
        for (ox, oz) in [(0., 0.), (-80., 120.)] {
            let g = GridGeometry { width: 2, height: 4, tile_width: 8., tile_height: 8.,
                origin_x: ox, origin_z: oz };
            for row in 0..4 {
                let (w, e, north, south) = g.bounds(0, row);
                for (x, normal) in [(w, [-1., 0., 0.]), (e, [1., 0., 0.])] {
                    let mut surface = SurfaceMeshData::default();
                    let mut p = [[x, 0., north], [x, 6., north],
                        [x, 6., south], [x, 0., south]];
                    if normal[0] < 0. { p.reverse(); }
                    assert!(new_bark::ledge_finish::append(&mut surface, p, normal, &g, [10, -7]));
                    let original = surface.positions.clone();
                    assert!(original.iter().all(|p| (0. ..=8.).contains(&p[1])),
                        "rock masses stay low instead of adding upright grass curtains");
                    mark_bank_surface(&mut surface, 0, row * 2);
                    shape_surface(&mut surface, &cells, &g);
                    for (before, after) in original.iter().zip(&surface.positions) {
                        let rise = ((before[2] - oz) / 32.).clamp(0., 1.);
                        let expected = if before[1] <= 6. { before[1] * rise }
                            else { before[1] + 6. * (rise - 1.) };
                        assert!((after[1] - expected).abs() < 0.00001,
                            "source row {row}, normal {normal:?}: {before:?} became {after:?}");
                        assert_eq!([after[0], after[2]], [before[0], before[2]]);
                    }
                    assert!(surface.uvs.iter().all(|uv| *uv == [0.; 2]));
                }
            }
        }
    }

    #[test]
    fn only_authored_zero_height_bank_toes_lose_their_old_back_wall() {
        let source = super::super::tests::source_with_tile(0x57, 1, 0, 5);
        let f = super::super::tests::frame(1, 1, vec![source]);
        let cells: Vec<_> = f.tiles.iter().collect();
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let p = [[0., 0., 0.], [0., 6., 0.], [8., 6., 0.], [8., 0., 0.]];
        assert!(obsolete_toe_wall(&p, [0., 0., -1.], &cells, &g));
        let mut mesh = TerrainMeshData {
            footing_heights: vec![6.],
            ..Default::default()
        };
        append_quad_colors(
            &mut mesh.textured,
            p,
            [0., 0., -1.],
            [[0.5; 2]; 4],
            [[1.; 4]; 4],
        );
        new_bark::polish_world_surfaces(&mut mesh, "Route26", &cells, &g, [0, 0]);
        assert!(mesh.textured.positions.is_empty());
        assert!(
            mesh.solid.positions.is_empty(),
            "obsolete wall cannot gain floor relief or rim grass"
        );
        assert_eq!(mesh.footing_heights, [6.]);
        assert!(
            !obsolete_toe_wall(&p, [0., 0., 1.], &cells, &g),
            "front drop stays"
        );
        assert!(
            !obsolete_toe_wall(&p, [1., 0., 0.], &cells, &g),
            "side contact stays"
        );
        let tall = p.map(|p| [p[0], p[1] * 2., p[2]]);
        assert!(
            !obsolete_toe_wall(&tall, [0., 0., -1.], &cells, &g),
            "cliff stays"
        );
        for row in 1..4 {
            let other = super::super::tests::frame(
                1,
                1,
                vec![super::super::tests::source_with_tile(0x57, 1, row, 5)],
            );
            assert!(!obsolete_toe_wall(
                &p,
                [0., 0., -1.],
                &other.tiles.iter().collect::<Vec<_>>(),
                &g
            ));
        }
        let flat = super::super::tests::frame(
            1,
            1,
            vec![super::super::tests::source_with_tile(0, 0, 0, 5)],
        );
        assert!(!obsolete_toe_wall(
            &p,
            [0., 0., -1.],
            &flat.tiles.iter().collect::<Vec<_>>(),
            &g
        ));
    }

    #[test]
    fn irregular_lip_seals_differently_sampled_cap_and_face_and_keeps_floor_centers() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 20];
        for row in 0..4 {
            for col in 1..4 {
                sources[row * 5 + col] = super::super::tests::source_with_tile(
                    0x57,
                    1,
                    row as u8,
                    if row == 3 { 0x4c } else { 5 },
                );
            }
        }
        let f = super::super::tests::frame(5, 4, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 5,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![5.25; 20],
            ..Default::default()
        };
        for w in [16., 20.] {
            append_solid_quad(
                &mut mesh.solid,
                [
                    [w, 5.25, 28.],
                    [w, 6., 32.],
                    [w + 4., 6., 32.],
                    [w + 4., 5.25, 28.],
                ],
                [0., 1., 0.],
                [1.; 4],
            );
        }
        assert!(new_bark::ledge_finish::append(
            &mut mesh.solid,
            [
                [24., 0., 32.],
                [24., 6., 32.],
                [16., 6., 32.],
                [16., 0., 32.]
            ],
            [0., 0., 1.],
            &g,
            [0, 0]
        ));
        let before = mesh.clone();
        soften_lips(&mut mesh, &cells, &g, &shapes, [0, 0]);
        assert_eq!(mesh.footing_heights, before.footing_heights);
        let tip = mesh.solid.positions[2][2];
        assert!(tip < 31.7 && tip > 31.2, "bounded organic rim inset");
        for (a, b) in before.solid.positions.iter().zip(&mesh.solid.positions) {
            assert_eq!(a[1], b[1], "rim shaping cannot change y");
            if a[1] == 0. || a[2] == 28. || a[0] == 16. || a[0] == 24. {
                assert_eq!(a, b, "floor, walking centers and run ends stay fixed");
            }
            if a[1] == 6. && (a[2] - 32.).abs() < 0.0001 {
                let t = ((a[0] - 20.).abs() / 4.).clamp(0., 1.);
                let expected = tip * (1. - t) + 32. * t;
                assert!(
                    (b[2] - expected).abs() < 0.0001,
                    "fine earth-face samples must follow the coarse cap edge exactly"
                );
            }
            assert!(Vec3::from_array(*b).is_finite());
        }
        let mut shifted = before.clone();
        for p in &mut shifted.solid.positions {
            p[0] -= 8.;
        }
        let translated = GridGeometry { origin_x: -8., ..g };
        soften_lips(&mut shifted, &cells, &translated, &shapes, [0, 0]);
        for (a, b) in mesh.solid.positions.iter().zip(&shifted.solid.positions) {
            assert!(
                (a[0] - 8. - b[0]).abs() < 0.0001 && a[1] == b[1] && (a[2] - b[2]).abs() < 0.0001
            );
        }
        let mut claimed = before.clone();
        claimed.authored_cells = vec![None; cells.len()];
        claimed.authored_cells[17] = Some("authored/object");
        soften_lips(&mut claimed, &cells, &g, &shapes, [0, 0]);
        assert_eq!(
            claimed.solid.positions, before.solid.positions,
            "modeled object ownership must stop the rim field"
        );
    }

    #[test]
    fn consecutive_jump_drawings_have_a_closed_internal_sawtooth_drop() {
        let sources = (0..8)
            .map(|row| {
                super::super::tests::source_with_tile(
                    0x57,
                    1,
                    row % 4,
                    if row % 4 == 3 { 0x4c } else { 5 },
                )
            })
            .collect();
        let f = super::super::tests::frame(1, 8, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 1,
            height: 8,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![6.; 8],
            ..Default::default()
        };
        for row in 0..8 {
            let (w, e, n, s) = g.bounds(0, row);
            append_top_shaded(&mut mesh.solid, [w, e, n, s], 6., (0., 1., 0., 1.), [1.; 4]);
        }
        shape_approaches(&mut mesh, &cells, &g, &shapes, [0, 0]);
        // An interior ray through the reset must hit a south-facing wall,
        // rather than showing the background between one tip and the next toe.
        let point = bevy::prelude::Vec2::new(4., 3.);
        let cross = |a: bevy::prelude::Vec2, b: bevy::prelude::Vec2| a.x * b.y - a.y * b.x;
        assert!(mesh.solid.indices.chunks_exact(3).any(|tri| {
            if mesh.solid.normals[tri[0] as usize][2] < 0.5 {
                return false;
            }
            let p = [tri[0], tri[1], tri[2]]
                .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
            if p.iter().any(|p| (p.z - 32.).abs() > g.tile_height * 0.25) {
                return false;
            }
            let [a, b, c] = p.map(|p| bevy::prelude::Vec2::new(p.x, p.y));
            let area = cross(b - a, c - a);
            if area.abs() < 0.0001 {
                return false;
            }
            let u = cross(b - point, c - point) / area;
            let v = cross(c - point, a - point) / area;
            u >= -0.0001 && v >= -0.0001 && u + v <= 1.0001
        }));
        assert_eq!(
            mesh.footing_heights,
            [0.75, 2.25, 3.75, 5.25, 0.75, 2.25, 3.75, 5.25]
        );
    }

    #[test]
    fn four_row_bank_is_a_continuous_sawtooth_with_a_sharp_front_drop() {
        let mut sources = vec![super::super::tests::source_with_tile(0, 0, 0, 5)];
        for row in 0..4 {
            sources.push(super::super::tests::source_with_tile(
                0x57,
                1,
                row,
                if row == 3 { 0x4c } else { 5 },
            ));
        }
        let f = super::super::tests::frame(1, 5, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let g = GridGeometry {
            width: 1,
            height: 5,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0, 6.0, 6.0, 6.0, 6.0],
            ..Default::default()
        };
        for row in 1..5 {
            let (w, e, n, s) = g.bounds(0, row);
            append_top_shaded(
                &mut mesh.solid,
                [w, e, n, s],
                6.0,
                (0.0, 1.0, 0.0, 1.0),
                [1.0; 4],
            );
        }
        let top_count = mesh.solid.positions.len();
        // A painted wall normal can identify the neighbor rather than its bank.
        append_solid_quad(
            &mut mesh.solid,
            [
                [0.0, 0.0, 8.0],
                [0.0, 6.0, 8.0],
                [8.0, 6.0, 8.0],
                [8.0, 0.0, 8.0],
            ],
            [0.0, 0.0, 1.0],
            [1.0; 4],
        );
        append_solid_quad(
            &mut mesh.solid,
            [
                [8.0, 0.0, 40.0],
                [8.0, 6.0, 40.0],
                [0.0, 6.0, 40.0],
                [0.0, 0.0, 40.0],
            ],
            [0.0, 0.0, 1.0],
            [1.0; 4],
        );
        shape_approaches(
            &mut mesh,
            &cells,
            &g,
            &cells
                .iter()
                .map(|c| shape_for_source(&c.source))
                .collect::<Vec<_>>(),
            [0, 0],
        );
        for row in 0..4 {
            let i = row + 1;
            let corners = cap_heights(
                &cells[i].source,
                shape_for_source(&cells[i].source),
                g.tile_height,
            )
            .unwrap();
            for (corner, height) in corners.into_iter().enumerate() {
                assert!(
                    (height - mesh.solid.positions[row * 4 + corner][1]).abs() < 0.0001,
                    "blend edge heights must match the rendered approach"
                );
            }
        }
        for (p, n) in mesh.solid.positions[..top_count]
            .iter()
            .zip(&mesh.solid.normals)
        {
            assert!((p[1] - 6.0 * (p[2] - 8.0) / 32.0).abs() < 0.0001);
            if p[2] == 8. {
                assert_eq!(*n, Vec3::Y.to_array(), "toe lighting must meet the level lawn");
            } else {
                assert!(n[1] > 0.9 && n[2] < 0.0);
            }
            assert!((Vec3::from_array(*n).length() - 1.).abs() < 0.0001);
        }
        assert!(
            mesh.solid.positions[top_count..top_count + 4]
                .iter()
                .all(|p| p[1] == 0.0)
        );
        assert_eq!(mesh.solid.positions[top_count + 5][1], 6.0);
        assert_eq!(mesh.solid.positions[top_count + 4][1], 0.0);
        assert!(mesh.solid.normals[top_count + 4][2] > 0.999,
            "toe shading cannot soften the sharp front earth face");
        assert_eq!(mesh.footing_heights, [0.0, 0.75, 2.25, 3.75, 5.25]);
    }
}

#[cfg(test)]
mod cliff_join_tests {
    use super::*;

    #[test]
    fn adjoining_cliff_keeps_its_top_and_closes_against_the_sloping_bank() {
        let sources = (0..4)
            .flat_map(|row| {
                [
                    super::super::tests::source_with_tile(
                        0x57,
                        1,
                        row,
                        if row == 3 { 0x4c } else { 5 },
                    ),
                    super::super::tests::source_with_tile(0, 0, 0, 5),
                ]
            })
            .collect();
        let f = super::super::tests::frame(2, 4, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let g = GridGeometry {
            width: 2,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![6.0; 8],
            ..Default::default()
        };
        // Relief protrudes into the bank, so centroid ownership alone must
        // not cause its base vertices to stay at the old constant height.
        append_solid_quad(
            &mut mesh.solid,
            [
                [7.8, 6.0, 0.0],
                [7.8, 16.0, 0.0],
                [7.8, 16.0, 8.0],
                [7.8, 6.0, 8.0],
            ],
            [-1.0, 0.0, 0.0],
            [1.0; 4],
        );
        // A small painted verge extends into the neighboring flat cell and
        // sits just above the bank. Its cap must follow the bank too.
        append_solid_quad(
            &mut mesh.solid,
            [
                [8.0, 6.012, 0.0],
                [8.0, 6.012, 8.0],
                [8.2, 6.012, 8.0],
                [8.2, 6.012, 0.0],
            ],
            [0.0, 1.0, 0.0],
            [1.0; 4],
        );
        shape_approaches(
            &mut mesh,
            &cells,
            &g,
            &cells
                .iter()
                .map(|c| shape_for_source(&c.source))
                .collect::<Vec<_>>(),
            [0, 0],
        );
        assert_eq!(
            mesh.solid.positions[..4]
                .iter()
                .map(|p| p[1])
                .collect::<Vec<_>>(),
            [0.0, 16.0, 16.0, 1.5]
        );
        for p in &mesh.solid.positions[4..] {
            assert!((p[1] - (0.012 + 6.0 * p[2] / 32.0)).abs() < 0.00001);
        }
    }
}
