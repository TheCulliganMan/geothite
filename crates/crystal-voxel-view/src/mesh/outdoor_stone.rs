//! World-anchored mineral grain, weathering and moss on native outdoor stone.
use super::*;
use bevy::prelude::{Vec2, Vec3Swizzles};

/// Outdoor rock erodes beneath its source cap. A bounded downward field
/// preserves planar fractures even where the ice crown field is positive.
fn crest_depth(x: f32, z: f32) -> f32 {
    // Large mineral beds carry the silhouette. A second short-period height
    // field turned every broad plane into small humps. Keep one linear field
    // and let the separate cleft field cut the beds instead of swelling them.
    // Keep the bed field below the support clamp on ordinary high caps.
    // Deep relief saturated that clamp across broad patches and bent the
    // surrounding wall into padded shoulders. The separate clefts retain
    // the deep broken cuts without turning the whole bed into a mound.
    0.75 + super::cave_cliff::slab(x * 0.16, z * 0.16) * 1.40
}

fn cap_cleft(x: f32, z: f32) -> f32 {
    let fracture = super::cave_cliff::slab(
        x * 0.32 + z * 0.08 + 29., z * 0.27 - x * 0.05 - 71.);
    // Linear fissure walls cut through broad mineral beds. Sample authored
    // coordinates: moving the fissure with each bevel ring bent the line
    // through the wall and reversed extra wall-plane projections.
    // The cap grid has six intervals per tile. A narrow notch could fall
    // between its samples and read as isolated dimples; wider linear sides
    // make each break span several samples without adding another grid.
    (1. - fracture.abs() / 0.12).clamp(0., 1.) * 0.9
}

// Broad, piecewise-planar mountain beds with shallow resolved joints. The
// old deep erosion multiplied by several radial fades made whole caps swell
// into rounded mounds, even though the underlying slab field was linear.
fn mountain_bed_depth(x: f32, z: f32) -> f32 {
    let bed=super::cave_cliff::slab(x*0.22+7.,z*0.22-19.);
    let joint=super::cave_cliff::slab(x*0.31+z*0.09+29.,z*0.27-x*0.05-71.);
    0.24+bed*0.30+(1.-joint.abs()/0.10).clamp(0.,1.)*0.06
}

fn mineral_bevel(inward: f32, distance: f32, along: f32, depth: f32, width: f32) -> Vec2 {
    if distance >= width || depth <= -width || depth >= width {
        return Vec2::ZERO;
    }
    // A shorter rock can meet a taller wall. Continue its cap deformation
    // into that wall and fade above the join; a hard cap-height cutoff tears
    // the shared vertical samples apart immediately above the shorter rock.
    let above = (-depth / width).max(0.);
    let fade = 1. - above * above * (3. - 2. * above);
    let depth = depth.max(0.);
    // Beyond a finite edge endpoint, the wall-plane distance alone stays
    // small even as the actual edge recedes. Using it after a radial cutoff
    // switched the full bevel on abruptly and folded neighboring cap samples.
    let inward = if along == 0. || along == 1. {
        inward.max(distance)
    } else { inward };
    // Project onto the mineral cut itself. Scaling a radial vector by an
    // octagonal distance still gives a curved mapping inside each octant:
    // neighbouring height rings bow into the padded shoulders seen in the
    // side view. One diagonal plane keeps those rings on a straight facet.
    // Match the previous edge's recession, retaining the broad cap bed.
    let cut = width * (2. - std::f32::consts::SQRT_2);
    let mut projected = Vec2::new(inward.max(0.), depth);
    if projected.x + projected.y < cut {
        let along_cut = ((inward - depth + cut) * 0.5).clamp(0., cut);
        projected = Vec2::new(along_cut, cut - along_cut);
    }
    Vec2::new(projected.x - inward, depth - projected.y) * fade
}

fn combined_rim_shift(rim: [f32; 4]) -> Vec2 {
    let shift = Vec2::new(rim[0] - rim[1], rim[2] - rim[3]);
    let length = shift.length();
    if length == 0. { return shift; }
    // Perpendicular bevels describe one shoulder. Adding both full radial
    // offsets doubled its diagonal gradient and inverted reentrant caps.
    // Preserve the strongest face's radius; approach the single-face field
    // continuously as the other contribution vanishes.
    shift * (shift.x.abs().max(shift.y.abs()) / length)
}

fn adjoining_support(distance: f32, reach: f32, floor: f32) -> f32 {
    let remaining = (1. - (distance / reach).powi(2)).max(0.);
    floor * remaining * remaining
}

/// Sculpt native outdoor rock masses with shared source-coordinate fields.
/// Exposed rims erode; floor contacts and planted-tree seats stay anchored.
pub(super) fn finish_rock_geometry(mesh: &mut TerrainMeshData, map: &str,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry, origin: [i32; 2],
    datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    {
        let _trace = crate::loading_trace::span("region.rock_sculpt_rims", || serde_json::json!({}));
        sculpt_rock_rims_on_map(mesh, map, cells, shapes, g, origin);
    }
    retain_mineral_cap_ownership(mesh);
    {
        let _trace = crate::loading_trace::span("region.rock_round_contacts", || serde_json::json!({}));
        super::rock_bases::round_geometry(mesh, map, cells, shapes, g, origin);
    }
    super::outdoor_mountains::finish(mesh, map, cells, shapes, g, origin,datum);
    // Project backing from the final cap, including its mountain relief.
    super::outdoor_mountains::append_reveal_floor(mesh, map, cells, shapes, g, origin);
    {
        let _trace = crate::loading_trace::span("region.rock_finish_normals", || serde_json::json!({}));
        mineral_normals(&mut mesh.solid);
    }
    finish_mountain_cap_pigment(&mut mesh.solid, map);
}

pub(super) fn retain_mineral_cap_ownership(mesh: &mut TerrainMeshData) {
    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    // Normals still identify the authored faces here. Later smoothing and
    // mountain relief can tilt a wall upward without turning it into a cap.
    for (i,(normal,uv)) in mesh.solid.normals.iter().zip(&mesh.solid.uvs).enumerate() {
        if uv[0]>=-9000. || normal[1]<0.9 {continue;}
        if let Some(last)=ranges.last_mut() && last.end==i {last.end=i+1;}
        else {ranges.push(i..i+1);}
    }
    mesh.mineral_cap_ranges=Some(ranges);
}

fn finish_mountain_cap_pigment(surface: &mut SurfaceMeshData, map: &str) {
    if map != "BlackthornCity" { return; }
    // Canyon's original vertex pigment contains horizontal strata and baked
    // directional shading. Across a cap those bands become broad dark blobs,
    // suggesting rounded mounds even on the shallow planar geometry. Retain
    // the sediment faces, and let the existing mineral material and lighting
    // describe the upper beds without this second, false relief field.
    let reference = Vec3::new(0.35, 0.245, 0.12);
    for ((normal, uv), color) in surface.normals.iter().zip(&surface.uvs)
        .zip(&mut surface.colors)
    {
        if uv[0] >= -9000. || uv[1] >= -30000. { continue; }
        let t = ((normal[1] - 0.35) / 0.45).clamp(0., 1.);
        let weight = t * t * (3. - 2. * t);
        let pigment = Vec3::new(color[0], color[1], color[2]).lerp(reference, weight);
        color[0] = pigment.x;
        color[1] = pigment.y;
        color[2] = pigment.z;
    }
}

pub(super) fn sculpt_rock_rims(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    sculpt_rock_rims_with_edge_index(mesh, cells, shapes, g, origin, true, 0.48);
}

pub(super) fn sculpt_rock_rims_on_map(mesh: &mut TerrainMeshData, map: &str,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry, origin: [i32;2]) {
    // Blackthorn's short adjoining beds cannot absorb the old near-half-tile
    // lateral crest recession on top of both corner cuts. Retain broad caps
    // and keep their projected samples ordered through these narrow joins.
    let recession=if map=="BlackthornCity" {0.0}else{0.48};
    sculpt_rock_rims_with_edge_index(mesh,cells,shapes,g,origin,true,recession);
}

#[cfg(test)]
pub(super) fn sculpt_rock_rims_reference(mesh: &mut TerrainMeshData,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry, origin: [i32; 2]) {
    sculpt_rock_rims_with_edge_index(mesh, cells, shapes, g, origin, false, 0.48);
}

#[cfg(test)]
pub(super) fn sculpt_rock_rims_reference_on_map(mesh: &mut TerrainMeshData, map: &str,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry, origin: [i32;2]) {
    sculpt_rock_rims_with_edge_index(mesh,cells,shapes,g,origin,false,
        if map=="BlackthornCity" {0.0}else{0.48});
}

struct CapTriangle {
    indices: [u32; 3],
    center: [f32; 3],
    retained: bool,
}
#[derive(Default)]
struct CapTriangleIndex {
    triangles: Vec<CapTriangle>,
    cells: bevy::platform::collections::HashMap<(isize, isize), Vec<usize>>,
}
impl CapTriangleIndex {
    fn absorb(&mut self, surface: &mut SurfaceMeshData, g: &GridGeometry) {
        let indices = std::mem::take(&mut surface.indices);
        for triangle in indices.chunks_exact(3) {
            let ids = [triangle[0], triangle[1], triangle[2]];
            let eligible = surface.uvs[ids[0] as usize][0] < -9000.;
            let mut center = [0.; 3];
            if eligible {
                // Preserve the reference pass's division and accumulation order.
                for i in ids {
                    for axis in 0..3 { center[axis] += surface.positions[i as usize][axis] / 3.; }
                }
                if center[0].is_finite() && center[2].is_finite() {
                    let cell = (((center[0] - g.origin_x) / g.tile_width).floor() as isize,
                        ((center[2] - g.origin_z) / g.tile_height).floor() as isize);
                    self.cells.entry(cell).or_default().push(self.triangles.len());
                }
            }
            self.triangles.push(CapTriangle { indices: ids, center, retained: true });
        }
    }
    fn remove_cap(&mut self, surface: &SurfaceMeshData, g: &GridGeometry,
        x: usize, z: usize, h: f32) {
        let (w, e, n, south) = g.bounds(x, z);
        // Neighbor buckets keep cell-boundary rounding conservative. The
        // original strict bounds and height tests still decide every removal.
        for dz in -1..=1 {
            for dx in -1..=1 {
                if let Some(candidates) = self.cells.get(&(x as isize + dx, z as isize + dz)) {
                    for &i in candidates {
                        let triangle = &mut self.triangles[i];
                        if !triangle.retained { continue; }
                        let center = triangle.center;
                        if center[0] > w && center[0] < e && center[2] > n && center[2] < south
                            && triangle.indices.iter().all(|&i|
                                (surface.positions[i as usize][1] - h).abs() < 0.001) {
                            triangle.retained = false;
                        }
                    }
                }
            }
        }
    }
    fn finish(self) -> Vec<u32> {
        self.triangles.into_iter().filter(|triangle| triangle.retained)
            .flat_map(|triangle| triangle.indices).collect()
    }
}

fn sculpt_rock_rims_with_edge_index(mesh: &mut TerrainMeshData,
    cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry,
    origin: [i32; 2], indexed: bool, crest_recession: f32) {
    if !mesh.solid.uvs.iter().any(|uv| uv[0] < -9000.0) {
        return;
    }
    // Native cap chamfers leave tiny triangular openings when their square
    // side walls are sculpted. Complete the cap before the shared rim warp;
    // its restored corners are then eroded together with their wall vertices.
    let mut cap_triangles: Option<CapTriangleIndex> = None;
    for (i, cell) in cells.iter().enumerate() {
        if !matches!(cell.source.tileset_id.as_ref(), "johto" | "johto_modern")
            || cell.source.metatile_id != 0x0a
        {
            continue;
        }
        let corner = match (cell.source.subtile_column, cell.source.subtile_row) {
            (0, 0) => 0,
            (3, 0) => 1,
            (0, 3) => 2,
            (3, 3) => 3,
            _ => continue,
        };
        let x = i % g.width;
        let z = i / g.width;
        if !rock_platform_corner_clips(cells, shapes, g, x, z)[corner] {
            continue;
        }
        let (w, e, n, south) = g.bounds(x, z);
        let h = shapes[i].surface_height(g.tile_height);
        // Restored corner faces inherit the source cap's mineral family.
        // A generic gray patch would reopen a visible material seam on the
        // Route26 ochre shoulders even though their geometry is closed.
        let canyon = mesh.solid.positions.iter().zip(&mesh.solid.uvs).any(|(p, uv)|
            uv[0] < -9000. && uv[1] < -30000.
                && p[0] >= w && p[0] <= e && p[2] >= n && p[2] <= south
                && (p[1]-h).abs() < 0.001);
        let append_corner = if canyon { append_canyon } else { append };
        // The native clipped cap is a triangle fan. Warping its unequal
        // edge subdivisions beside a regularly sampled wall opens cracks
        // even when their original endpoints coincide. Replace this corner
        // cell with one regular cap before sculpting both surfaces.
        let surface = &mut mesh.solid;
        if indexed {
            let triangles = cap_triangles.get_or_insert_with(|| {
                let mut triangles = CapTriangleIndex::default();
                triangles.absorb(surface, g);
                triangles
            });
            triangles.remove_cap(surface, g, x, z, h);
        } else {
            let positions = &surface.positions;
            let uvs = &surface.uvs;
            let indices = surface
                .indices
                .chunks_exact(3)
                .filter(|tri| {
                    // Most scene triangles are foliage or ground. They cannot
                    // belong to this native rock cap; avoid loading their points
                    // and calculating a centroid for every restored corner.
                    if !(uvs[tri[0] as usize][0] < -9000.) {
                        return true;
                    }
                    let q = [tri[0], tri[1], tri[2]].map(|i| positions[i as usize]);
                    if !q.iter().all(|p| (p[1] - h).abs() < 0.001) {
                        return true;
                    }
                    let center = q.iter().fold([0.; 3], |mut a, p| {
                        for axis in 0..3 {
                            a[axis] += p[axis] / 3.;
                        }
                        a
                    });
                    !(center[0] > w
                        && center[0] < e
                        && center[2] > n
                        && center[2] < south)
                })
                .flat_map(|tri| tri.iter().copied())
                .collect();
            surface.indices = indices;
        }
        append_corner(
            surface,
            [[w, h, n], [w, h, south], [e, h, south], [e, h, n]],
            Vec3::Y.to_array(),
            g,
            origin,
        );
        // Complete the two side sections beneath the restored corner too.
        // A diagonal native chamfer alone cannot close a square wall rim.
        let sides = match corner {
            0 => [0, 3],
            1 => [1, 3],
            2 => [0, 2],
            _ => [1, 2],
        };
        for side in sides {
            let (neighbor, normal, top_a, top_b) = match side {
                0 => (
                    (x.wrapping_sub(1), z),
                    Vec3::NEG_X,
                    [w, h, n],
                    [w, h, south],
                ),
                1 => ((x + 1, z), Vec3::X, [e, h, south], [e, h, n]),
                2 => ((x, z + 1), Vec3::Z, [w, h, south], [e, h, south]),
                _ => ((x, z.wrapping_sub(1)), Vec3::NEG_Z, [e, h, n], [w, h, n]),
            };
            if neighbor.0 >= g.width || neighbor.1 >= g.height {
                continue;
            }
            let neighbor_index = neighbor.1 * g.width + neighbor.0;
            let source = &cells[neighbor_index].source;
            let bottom_at = |p: [f32; 3]| {
                let shape = shapes[neighbor_index];
                let mut bottom = shape.surface_height(g.tile_height).max(0.0);
                if source.tileset_id.as_ref() == "johto"
                    && matches!(shape_for_source(source),
                        CellShape::RaisedTop { height, solid: SolidKind::Bank }
                        | CellShape::LedgeBand { height, face: LedgeFace::South, .. }
                        if height == crate::profile::JUMP_LEDGE_HEIGHT)
                {
                    if let CellShape::RampEast {
                        west_height,
                        east_height,
                    } = shape
                    {
                        let west = g.origin_x + neighbor.0 as f32 * g.tile_width;
                        let along = ((p[0] - west) / g.tile_width).clamp(0.0, 1.0);
                        bottom = (west_height * (1.0 - along) + east_height * along)
                            * g.tile_height
                            / SOURCE_TILE_HEIGHT;
                    }
                    let north = g.origin_z
                        + (neighbor.1 as f32 - source.subtile_row as f32) * g.tile_height;
                    bottom *= ((p[2] - north) / (g.tile_height * 4.0)).clamp(0.0, 1.0);
                }
                bottom
            };
            if bottom_at(top_a).min(bottom_at(top_b)) >= h - 0.01 {
                continue;
            }
            let mut bottom_a = top_a;
            bottom_a[1] = bottom_at(top_a);
            let mut bottom_b = top_b;
            bottom_b[1] = bottom_at(top_b);
            append_corner(
                &mut mesh.solid,
                [bottom_b, top_b, top_a, bottom_a],
                normal.to_array(),
                g,
                origin,
            );
        }
        if let Some(triangles) = &mut cap_triangles {
            triangles.absorb(&mut mesh.solid, g);
        }
    }
    if let Some(triangles) = cap_triangles { mesh.solid.indices = triangles.finish(); }
    let rock_height = |i: usize, cell: &VisualTile| {
        let johto = matches!(cell.source.tileset_id.as_ref(), "johto" | "johto_modern")
            && (cell.source.metatile_id == 0x0a
                || (0x68..=0x73).contains(&cell.source.metatile_id));
        let kanto = cell.source.tileset_id.as_ref()=="kanto"
            && super::outdoor_mountains::permanent(&cell.source,shapes[i]);
        // Both atlases use the same closed mineral shoulders. Source-proven
        // Kanto mountain ink must not remain an unshaped textured box; low
        // jump banks, shore rocks and entrance floors are separate owners.
        if !johto && !kanto {
            return None;
        }
        match shapes[i] {
            CellShape::RaisedTop {
                height,
                solid: SolidKind::Bank,
            }
            | CellShape::LedgeBand { height, .. } => Some(height * g.tile_height / 8.0),
            _ => None,
        }
    };
    if !cells.iter().enumerate().any(|(i, cell)| rock_height(i, cell).is_some()) {
        return;
    }
    let mut edges = Vec::new();
    let mut ledge_joints = Vec::new();
    let mut corners = Vec::new();
    let mut inside_joins = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let Some(height) = rock_height(i, cell) else {
            continue;
        };
        let x = i % g.width;
        let z = i / g.width;
        let (w, e, n, s) = g.bounds(x, z);
        // A lower native rock can meet a canyon shoulder with a different
        // cap tessellation. Retain the narrow contact profile there; broad
        // erosion is reserved for shoulders above ordinary ground.
        let rock_contact = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .any(|(dx, dz)| {
                let nx = x as isize + dx;
                let nz = z as isize + dz;
                if nx < 0 || nz < 0 || nx >= g.width as isize || nz >= g.height as isize {
                    return false;
                }
                let ni = nz as usize * g.width + nx as usize;
                rock_height(ni, cells[ni]).is_some_and(|h| (h - height).abs() > 0.01)
            });
        let mut exposed = [false; 4];
        let mut neighbors = [height; 4];
        for (direction, (dx, dz, a, b, normal)) in [
            (-1, 0, Vec2::new(w, n), Vec2::new(w, s), Vec2::NEG_X),
            (1, 0, Vec2::new(e, n), Vec2::new(e, s), Vec2::X),
            (0, -1, Vec2::new(w, n), Vec2::new(e, n), Vec2::NEG_Y),
            (0, 1, Vec2::new(w, s), Vec2::new(e, s), Vec2::Y),
        ]
        .into_iter()
        .enumerate()
        {
            let nx = x as isize + dx;
            let nz = z as isize + dz;
            // A viewport cut is not an exposed rock edge.
            if nx < 0 || nz < 0 || nx >= g.width as isize || nz >= g.height as isize {
                continue;
            }
            let neighbor = shapes[nz as usize * g.width + nx as usize];
            neighbors[direction] = neighbor.surface_height(g.tile_height).max(0.0);
            if neighbors[direction] >= height - 0.01 {
                continue;
            }
            // A sawtooth reset beside this wall is a real presentation
            // junction. Keep it anchored even when the rock face continues.
            let neighbor_source = &cells[nz as usize * g.width + nx as usize].source;
            if normal.x != 0.
                && neighbor_source.tileset_id.as_ref() == "johto"
                && neighbor_source.subtile_row == 3
                && matches!(shape_for_source(neighbor_source),
                    CellShape::LedgeBand { height, face: LedgeFace::South, .. }
                    if height == crate::profile::JUMP_LEDGE_HEIGHT)
            {
                ledge_joints.push(b);
            }
            exposed[direction] = true;
            neighbors[direction] = neighbor.surface_height(g.tile_height).max(0.0);
            edges.push((
                a,
                b,
                normal,
                height,
                neighbors[direction],
                if cell.source.metatile_id == 0x0a {
                    0.95
                } else if rock_contact {
                    0.48
                } else {
                    // A broad sandstone shoulder, with the same closed cap/
                    // wall projection as bedrock. Adjoining floors still
                    // limit its radius and anchor the lower wall.
                    0.90
                },
            ));
        }
        for (side_x, side_z, corner, inward) in [
            (0, 2, Vec2::new(w, n), Vec2::new(1., 1.)),
            (1, 2, Vec2::new(e, n), Vec2::new(-1., 1.)),
            (0, 3, Vec2::new(w, s), Vec2::new(1., -1.)),
            (1, 3, Vec2::new(e, s), Vec2::new(-1., -1.)),
        ] {
            let diagonal_x = x as isize - inward.x as isize;
            let diagonal_z = z as isize - inward.y as isize;
            if diagonal_x < 0 || diagonal_z < 0
                || diagonal_x >= g.width as isize || diagonal_z >= g.height as isize {
                continue;
            }
            let diagonal_height = shapes[diagonal_z as usize * g.width + diagonal_x as usize]
                .surface_height(g.tile_height);
            if diagonal_height >= height - 0.01 { continue; }
            if !exposed[side_x] && !exposed[side_z]
                && (neighbors[side_x] - height).abs() < 0.01
                && (neighbors[side_z] - height).abs() < 0.01 {
                // Three joined shelf quadrants surround a lower fourth one.
                // Both exposed walls end at this inside corner. Applying
                // each finite bevel through that endpoint rolls the shared
                // vertical edge into a narrow folded strip.
                inside_joins.push((corner, height));
                continue;
            }
            if !exposed[side_x] || !exposed[side_z] { continue; }
            let run = |dx: isize, dz: isize| {
                let mut count = 1;
                for step in 1..6 {
                    let nx = x as isize + dx * step;
                    let nz = z as isize + dz * step;
                    if nx < 0 || nz < 0 || nx >= g.width as isize || nz >= g.height as isize {
                        break;
                    }
                    let ni = nz as usize * g.width + nx as usize;
                    if rock_height(ni, cells[ni]).is_none_or(|h| (h - height).abs() > 0.01) {
                        break;
                    }
                    count += 1;
                }
                count as f32
            };
            let source_corner = corner
                + Vec2::new(origin[0] as f32 * g.tile_width - g.origin_x,
                    origin[1] as f32 * g.tile_height - g.origin_z);
            let variation = new_bark::meadow_noise(
                source_corner.x / g.tile_width * 0.5 + 71.,
                source_corner.y / g.tile_height * 0.5 - 19.) + 0.5;
            let radius = (g.tile_width.min(g.tile_height) * (1.4 + variation * 0.5))
                // Opposing diagonal cuts must not overlap across narrow
                // masses: selecting either displacement would fold the cap.
                .min(run(inward.x as isize, 0) * g.tile_width * 0.49)
                .min(run(0, inward.y as isize) * g.tile_height * 0.49);
            // A native corner of an isolated 0x0a drawing over level ground
            // is a free rock toe, not a six-unit jump-bank junction. Interior
            // source fragments and any raised adjoining floor retain the
            // conservative closure datum.
            let native_phase = match (side_x, side_z) {
                (0, 2) => (0, 0),
                (1, 2) => (3, 0),
                (0, 3) => (0, 3),
                _ => (3, 3),
            };
            let free_corner = cell.source.metatile_id == 0x0a
                && (cell.source.subtile_column, cell.source.subtile_row) == native_phase
                && neighbors[side_x] == 0. && neighbors[side_z] == 0.
                && shapes[diagonal_z as usize * g.width + diagonal_x as usize]
                    .surface_height(g.tile_height) == 0.;
            let datum = (if free_corner { 0. } else {
                crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT
            })
                .max(neighbors[side_x])
                .max(neighbors[side_z]);
            if height > datum + 0.01 {
                corners.push((corner, inward, radius, datum, height));
            }
        }
    }
    // Preserve seats for non-native planted artwork. The native 6a/6c corner
    // courses contain rock; flattening them exposes square shelves in cliffs.
    let planted: Vec<_> = cells.iter().enumerate().filter_map(|(i, cell)| {
        (rock_height(i, cell).is_some()
            && matches!(cell.source.metatile_id, 0x6a | 0x6c)
            && !super::outdoor_mountains::native_rock_corner(&cell.source))
            .then(|| g.bounds(i % g.width, i / g.width))
    }).collect();
    // An authored entrance has its own closed arch and side blocks. Keep the
    // rock/portal seating joint fixed instead of weathering those members
    // apart. Require the complete native mouth, not a tile number alone.
    let portals: Vec<_> = cells.iter().enumerate().filter_map(|(i,cell)| {
        let source=&cell.source;
        if source.tileset_id.as_ref()!="kanto" || !matches!(source.metatile_id,0x06|0x70)
            || source.subtile_column!=0 || source.subtile_row!=2 {return None;}
        let (x,z)=(i%g.width,i/g.width);
        if x+2>g.width || z+2>g.height {return None;}
        let art=[[0x48,0x49],[0x58,0x59]];
        if !(0..2).all(|dz|(0..2).all(|dx| {
            let s=&cells[(z+dz)*g.width+x+dx].source;
            s.tileset_id==source.tileset_id && s.metatile_id==source.metatile_id
                && s.subtile_column==dx as u8 && s.subtile_row==2+dz as u8 && s.tile_index==art[dz][dx]
        })) {return None;}
        let (w,_,n,_)=g.bounds(x,z);let (_,e,_,s)=g.bounds(x+1,z+1);
        Some((w,e,n,s))
    }).collect();
    let edges = join_wall_edges(edges, &ledge_joints);
    let edge_candidates = if indexed { rock_edge_candidates(&edges, g) } else { Vec::new() };
    let all_edges: Vec<_> = (0..edges.len()).collect();
    let s = &mut mesh.solid;
    // A sixth-cell wall/cap sample can differ by a float ULP. Mountain
    // coordinates need a tolerance comfortably below their mesh spacing,
    // rather than a decimal boundary that splits the same authored sample.
    let sample_scale=if crest_recession<0.48 {100.}else{10000.};
    // Authored cap samples include the identical endpoints on adjoining
    // wall faces. Classify before any bevel lowers their height; using the
    // rounded height here misclassified the cap shoulder as lower wall.
    let cap_samples=if crest_recession<0.48 {
        s.positions.iter().zip(&s.normals).zip(&s.uvs)
            .filter(|((_,n),uv)|uv[0] < -9000. && n[1]>0.9)
            .map(|((p,_),_)|p.map(|v|(v*sample_scale).round() as i64))
            .collect::<std::collections::HashSet<_>>()
    } else {std::collections::HashSet::new()};
    let wall_samples=if crest_recession<0.48 {
        s.positions.iter().zip(&s.normals).zip(&s.uvs)
            .filter(|((_,n),uv)|uv[0] < -9000. && n[1]<=0.9)
            .map(|((p,_),_)|p.map(|v|(v*sample_scale).round() as i64))
            .collect::<std::collections::HashSet<_>>()
    } else {std::collections::HashSet::new()};
    let mut shared_positions = HashMap::<[i64; 3], ([f32; 3], [bool; 3])>::new();
    for i in 0..s.positions.len() {
        if s.uvs[i][0] >= -9000.0 {
            continue;
        }
        let p = Vec3::from_array(s.positions[i]);
        let candidates = if indexed {
            let x = ((p.x - g.origin_x) / g.tile_width).floor()
                .clamp(0., g.width as f32 - 1.) as usize;
            let z = ((p.z - g.origin_z) / g.tile_height).floor()
                .clamp(0., g.height as f32 - 1.) as usize;
            &edge_candidates[z * g.width + x]
        } else { &all_edges };
        let nearby_edges = candidates.iter().map(|&i| &edges[i]);
        // Cap and wall grids interpolate the same authored sample through
        // different float expressions. A one-ULP difference can cross a steep
        // shoulder constraint and open a slit. Reuse the position field at
        // source precision, without joining the distinct material/normal data.
        let source_key = s.positions[i].map(|value| (value * sample_scale).round() as i64);
        let source_point = Vec3::new(
            p.x + origin[0] as f32 * g.tile_width - g.origin_x,
            p.y,
            p.z + origin[1] as f32 * g.tile_height - g.origin_z,
        );
        // The nearest adjoining floor constrains every part of the warp,
        // including a perpendicular rim reaching around a convex corner.
        let support = nearby_edges.clone()
            .map(|&(a, b, _, _, bottom, _)| {
                let along = ((p.xz() - a).dot(b - a) / (b - a).length_squared()).clamp(0., 1.);
                // The bevel reaches well beyond the thin wall relief skin.
                // A hard skin-width support cutoff let adjacent samples of
                // a shorter rock warp through its taller neighbor's seat.
                adjoining_support(p.xz().distance(a.lerp(b, along)),
                    g.tile_width.min(g.tile_height), bottom)
            })
            .fold(0.0_f32, f32::max);
        let mut rim_shift = [0.0_f32; 4];
        let mut rim_drop = [0.0_f32; 4];
        let mut wall_inset = [0.0_f32; 4];
        let mut wall_weather = None;
        for &(a, b, normal, height, bottom, rim_scale) in nearby_edges.clone() {
            let along = ((p.xz() - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
            let distance = p.xz().distance(a.lerp(b, along));
            // Choose form from the source edge, not the vertex material.
            // Stone/canyon vertices sharing a join must receive one warp.
            // One authored edge owns the bevel radius. Sampling vertex height
            // or its distance inside the cap inflated each ring independently,
            // producing pillowy shoulders instead of a shared mineral cut.
            let edge_point = a.lerp(b, along) + Vec2::new(
                origin[0] as f32 * g.tile_width - g.origin_x,
                origin[1] as f32 * g.tile_height - g.origin_z,
            );
            // Opposing parallel rims share this along-edge field. Different
            // radii across a narrow shelf would drag its protected centerline.
            let along_source = if normal.x == 0. {
                edge_point.x / g.tile_width
            } else {
                edge_point.y / g.tile_height
            };
            let rim_variation = 0.85 + super::cave_cliff::slab(
                along_source * 0.20, 17.,
            ) * 0.15;
            // Keep the mineral cut narrow enough to retain a broad flat bed.
            // A nearly tile-wide shoulder reads as a rolled mound even when
            // its samples lie on angular planes.
            let rim_width = (g.tile_height * rim_scale * rim_variation * 0.55)
                .min(height - bottom.max(support))
                .max(0.001);
            let direction = if normal.x < 0.0 {
                0
            } else if normal.x > 0.0 {
                1
            } else if normal.y < 0.0 {
                2
            } else {
                3
            };
            // Project the cap/wall corner onto one convex mineral plane.
            // The shared cut keeps straight height rings instead of a rolled
            // shoulder while preserving the authored edge recession.
            let depth = height - p.y;
            let delta = mineral_bevel((p.xz() - a).dot(-normal), distance,
                along, depth, rim_width);
            rim_shift[direction] = rim_shift[direction].max(delta.x);
            rim_drop[direction] = rim_drop[direction].max(-delta.y);
            // Native 0x0a rock edges carry the 0.95 rim scale. Above level
            // ground their lower faces can weather down to the actual floor;
            // a global jump-bank datum left a rigid box beneath every rock.
            // Preserve the full bank datum for canyon/ledge contacts and the
            // highest neighboring support for every source edge.
            let free_rock = rim_scale == 0.95 && bottom == 0.;
            let datum = (if free_rock { 0. } else {
                crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT
            })
                .max(bottom)
                .max(support);
            if height > datum + 0.01 && p.y > datum && p.y < height {
                let vertical = (p.y - datum) / (height - datum);
                let edge = (1.0 - distance / (g.tile_width * 0.14)).max(0.0).powi(2);
                if edge == 0. { continue; }
                // Anchor real run ends, not every internal source-tile seam.
                // A continuous wall shares broad weathering across its cells.
                let end_distance = along.min(1. - along) * a.distance(b);
                let join = (end_distance / g.tile_width).clamp(0., 1.);
                let join = join * join * (3. - 2. * join);
                if join == 0. { continue; }
                // A broad mineral face between two angled shoulders avoids
                // the inflated, rounded profile of a sinusoidal recess.
                let face = (vertical / 0.22).min(1.)
                    * ((1. - vertical) / 0.30).min(1.);
                // Broad stepped courses give the stone actual broken beds,
                // rather than relying on painted strata across one wall.
                // Use the cave rock's piecewise shoulder/slab profile in
                // source space; the deepest recess retains the old bound.
                // The mineral fields depend on the vertex, not its edge.
                // Evaluate only participating edges and share the result at
                // corners. Remote walls contribute exactly zero displacement.
                let (weather, recess) = *wall_weather.get_or_insert_with(|| {
                    let fracture = super::cave_cliff::slab(
                        source_point.x / g.tile_width * 0.65 + source_point.y / g.tile_height * 0.38,
                        source_point.z / g.tile_height * 0.65,
                    );
                    let bed = (source_point.y / g.tile_height * SOURCE_TILE_HEIGHT
                        + super::cave_cliff::slab(
                            source_point.x / g.tile_width * 0.24,
                            source_point.z / g.tile_height * 0.24,
                        ) * 3.2) / 7.5;
                    let course = bed.rem_euclid(1.);
                    let shoulder = ((course - 0.10) / 0.18).clamp(0., 1.);
                    let slope = 1. - ((course - 0.46) / 0.54).clamp(0., 1.);
                    (0.60 + fracture.abs() * 1.50, 1. - shoulder * slope)
                });
                let inset = face
                    * edge
                    * join
                    * weather
                    * g.tile_width
                    * (0.23 + 0.09 * recess);
                wall_inset[direction] = wall_inset[direction].max(inset);
            }
        }
        // Break exposed convex corners into mineral planes, using Ice Path's
        // diagonal compression. Keep positive cap area instead of collapsing
        // it onto the cut line, and share the warp with every wall vertex.
        let rim = combined_rim_shift(rim_shift);
        let rounded = p + Vec3::new(
            rim.x + wall_inset[0] - wall_inset[1],
            -rim_drop.into_iter().fold(0.0_f32, f32::max),
            rim.y + wall_inset[2] - wall_inset[3],
        );
        let mut shoulder = Vec2::ZERO;
        for &(corner, inward, radius, datum, height) in &corners {
            let datum = datum.max(support);
            // Decide ownership before other rims move this wall. A shorter
            // adjoining rock can push the shared sample just outside the
            // taller corner; dropping its entire corner cut there creates
            // a sharp discontinuity. Include the native wall's relief skin,
            // and clamp the rounded projection at the corner planes.
            let original_local = (p.xz() - corner) * inward;
            let local = ((rounded.xz() - corner) * inward).max(Vec2::ZERO);
            if height <= datum + 0.01
                || original_local.x < -g.tile_width * 0.10
                || original_local.y < -g.tile_height * 0.10
                || local.x > radius
                || local.y > radius
                || p.y <= datum
                || p.y > height + 0.01
            {
                continue;
            }
            let remaining = radius - local.x - local.y;
            if remaining <= 0. {
                continue;
            }
            let bank_height = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
            let upper_datum = if datum == 0. { bank_height } else { datum };
            let rise = ((p.y - upper_datum) / (height - upper_datum)).clamp(0., 1.);
            // Free toes receive a small, closed lower facet. Return to the
            // established shoulder field at the bank-height ring so changing
            // the foot cannot amplify the upper wall or disturb its cap.
            let toe = if datum == 0. && p.y < bank_height {
                (std::f32::consts::PI * p.y / bank_height).sin().powi(2) * 0.12
            } else { 0. };
            let fade = rise * rise * (3. - 2. * rise) + toe;
            let displacement = inward * remaining * 0.4 * fade;
            if displacement.length_squared() > shoulder.length_squared() {
                shoulder = displacement;
            }
        }
        let rounded = rounded + Vec3::new(shoulder.x, 0., shoulder.y);
        // Sample the already rounded coordinates. Adding a recess based on
        // the old square edge can reverse the compressed fillet vertices.
        let crest_point = rounded
            + Vec3::new(
                origin[0] as f32 * g.tile_width - g.origin_x,
                0.,
                origin[1] as f32 * g.tile_height - g.origin_z,
            );
        let mut crest_inset = [0.0_f32; 4];
        let mut crest_relief = 0.0_f32;
        let mut crest_erosion = None;
        for &(a, b, normal, height, bottom, _) in nearby_edges {
            let along = ((rounded.xz() - a).dot(b - a) / (b - a).length_squared()).clamp(0., 1.);
            let distance = rounded.xz().distance(a.lerp(b, along));
            let datum = (crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT)
                .max(bottom)
                .max(support);
            let direction = if normal.x < 0. {
                0
            } else if normal.x > 0. {
                1
            } else if normal.y < 0. {
                2
            } else {
                3
            };
            // Recess the upper silhouette with broad map-space erosion.
            // Apply one position field to both cap and wall; fade to zero
            // at the ground datum, real run ends, and the plateau interior.
            if height > datum + 0.01 && rounded.y > datum && rounded.y <= height + 0.01 {
                let rise = ((rounded.y - datum) / (height - datum)).clamp(0., 1.);
                let rise = rise * rise * (3. - 2. * rise);
                let interior = (1. - distance / (g.tile_width * 1.2)).max(0.).powi(2);
                if interior == 0. { continue; }
                let terminal = |endpoint: Vec2, distance: f32| {
                    let free_corner = corners.iter().any(|&(corner, _, _, _, top)| {
                        corner.distance(endpoint) < 0.001 && (top - height).abs() < 0.001
                    });
                    if free_corner {
                        1.
                    } else {
                        (distance / g.tile_width).clamp(0., 1.)
                    }
                };
                let join = terminal(a, along * a.distance(b))
                    .min(terminal(b, (1. - along) * a.distance(b)));
                let join = join * join * (3. - 2. * join);
                if join == 0. { continue; }
                // Every participating edge samples the same rounded source
                // point. Remote edges contribute zero; compute the fracture
                // field once instead of once for every wall in the scene.
                let erosion = *crest_erosion.get_or_insert_with(|| {
                    let fracture = super::cave_cliff::slab(
                        crest_point.x / g.tile_width * 0.45,
                        crest_point.z / g.tile_height * 0.45,
                    );
                    0.65 + fracture * 0.35
                });
                // Preserve lower wall beds. The cap and its coincident wall
                // endpoint share their authored sample classification, so
                // the upper cut cannot split that geometric seam.
                let recession=if cap_samples.contains(&source_key) {crest_recession}else{0.48};
                let recess = g.tile_width * recession * interior * rise * join * erosion;
                crest_inset[direction] = crest_inset[direction].max(recess);
                crest_relief = crest_relief.max(interior * rise * join);
            }
        }
        // Broad planar fractures run through the whole blocked mass, fading
        // above real support and around planted-tree seats. Outdoor stone uses
        // downward erosion rather than clipping Ice Path's raised crowns.
        let datum = (crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT)
            .max(support);
        let rise = ((rounded.y - datum) / (g.tile_height * 2.)).clamp(0., 1.);
        let tree_seat = planted.iter().map(|&(w, e, n, south)| {
            let nearest = Vec2::new(rounded.x.clamp(w, e), rounded.z.clamp(n, south));
            let distance = rounded.xz().distance(nearest) / g.tile_width;
            let fade = distance.clamp(0., 1.);
            fade * fade * (3. - 2. * fade)
        }).fold(1.0_f32, f32::min);
        let mass_relief = rise * rise * (3. - 2. * rise) * tree_seat;
        let relief = (0.55 * mass_relief).max(0.75 * crest_relief * tree_seat);
        let crest_height = if relief > 0. {
            let source_x = source_point.x / g.tile_width;
            let source_z = source_point.z / g.tile_height;
            let legacy=-(crest_depth(source_x,source_z)+cap_cleft(source_x,source_z))
                *g.tile_height*relief;
            let erosion=if cap_samples.contains(&source_key) {
                // Authored cap and coincident wall endpoints share this one
                // shallow bed, without edge-distance-dependent swelling.
                // Keep any higher existing contact: lowering a sheltered
                // corner below its last wall ring would reverse that face.
                let bed=-mountain_bed_depth(source_x,source_z)*g.tile_height*tree_seat;
                if wall_samples.contains(&source_key) {bed.max(legacy)}else{bed}
            } else {
                legacy
            };
            erosion.max(-(rounded.y - datum).max(0.) * 0.35)
        } else {
            0.
        };
        let sculpted = rounded + Vec3::new(
            crest_inset[0] - crest_inset[1], crest_height,
            crest_inset[2] - crest_inset[3]);
        // Preserve the common vertical inside edge and its cap endpoint.
        // Use source positions and one field for every participating face;
        // wall and cap tessellation must not choose different constraints.
        let join_weight = inside_joins.iter().filter(|(_, height)| p.y <= *height + 0.01)
            .map(|(corner, _)| {
                let t = (p.xz().distance(*corner) / (g.tile_width.min(g.tile_height) * 2.))
                    .clamp(0., 1.);
                t * t * (3. - 2. * t)
            }).fold(1.0_f32, f32::min);
        let portal_weight=portals.iter().map(|&(w,e,n,s)| {
            let nearest=Vec2::new(p.x.clamp(w,e),p.z.clamp(n,s));
            let t=(p.xz().distance(nearest)/g.tile_width-0.25).clamp(0.,1.);
            t*t*(3.-2.*t)
        }).fold(1.0_f32,f32::min);
        s.positions[i] = p.lerp(sculpted, join_weight*portal_weight).to_array();
        // Exact source floor contacts, protected inside edges and unexposed
        // viewport interiors keep their own authored coordinates. Only the
        // deformed shoulder samples need a common evaluated position.
        if s.positions[i] != p.to_array() {
            let changed = [0, 1, 2].map(|axis| s.positions[i][axis] != p[axis]);
            let (position, shared_axes) = shared_positions.entry(source_key)
                .or_insert((s.positions[i], changed));
            for axis in 0..3 {
                if changed[axis] && shared_axes[axis] {
                    s.positions[i][axis] = position[axis];
                } else if changed[axis] {
                    position[axis] = s.positions[i][axis];
                    shared_axes[axis] = true;
                }
            }
        }
    }
}

#[cfg(test)]
fn soften_rock_rims(mesh: &mut TerrainMeshData, cells: &[&VisualTile],
    shapes: &[CellShape], g: &GridGeometry, origin: [i32; 2]) {
    sculpt_rock_rims(mesh, cells, shapes, g, origin);
    mineral_normals(&mut mesh.solid);
}

type RockEdge = (Vec2, Vec2, Vec2, f32, f32, f32);

fn rock_edge_candidates(edges: &[RockEdge], g: &GridGeometry) -> Vec<Vec<usize>> {
    let mut candidates = vec![Vec::new(); g.width * g.height];
    // The longest edge field reaches 1.2 tiles from the rounded position.
    // Include four source tiles to cover the preceding rim, wall inset and
    // convex-corner shifts as well. Remote edges contribute exactly zero;
    // scanning every map edge at every vertex made broad route reviews slow.
    let reach = g.tile_width.max(g.tile_height) * 4.;
    for (i, &(a, b, ..)) in edges.iter().enumerate() {
        let low = a.min(b) - Vec2::splat(reach);
        let high = a.max(b) + Vec2::splat(reach);
        let column = |x: f32| ((x - g.origin_x) / g.tile_width).floor()
            .clamp(0., g.width as f32 - 1.) as usize;
        let row = |z: f32| ((z - g.origin_z) / g.tile_height).floor()
            .clamp(0., g.height as f32 - 1.) as usize;
        for z in row(low.y)..=row(high.y) {
            for x in column(low.x)..=column(high.x) {
                // Preserve the original edge order for identical floats.
                candidates[z * g.width + x].push(i);
            }
        }
    }
    candidates
}

/// Join only collinear, touching source edges with identical rock and floor
/// datums and shoulder profiles. Height changes and material contacts remain
/// independently anchored ends.
fn join_wall_edges(mut edges: Vec<RockEdge>, joints: &[Vec2]) -> Vec<RockEdge> {
    let key = |e: &RockEdge| {
        let vertical = e.0.x == e.1.x;
        (
            e.2.x.to_bits(),
            e.2.y.to_bits(),
            e.3.to_bits(),
            e.4.to_bits(),
            e.5.to_bits(),
            if vertical {
                e.0.x.to_bits()
            } else {
                e.0.y.to_bits()
            },
        )
    };
    edges.sort_by(|a, b| {
        key(a).cmp(&key(b)).then_with(|| {
            let start = |e: &RockEdge| if e.0.x == e.1.x { e.0.y } else { e.0.x };
            start(a).total_cmp(&start(b))
        })
    });
    let mut runs: Vec<RockEdge> = Vec::new();
    for edge in edges {
        if let Some(previous) = runs.last_mut() {
            if key(previous) == key(&edge)
                && previous.1.distance(edge.0) < 0.001
                && !joints.iter().any(|p| p.distance(edge.0) < 0.001)
            {
                previous.1 = edge.1;
                continue;
            }
        }
        runs.push(edge);
    }
    runs
}

pub(super) fn smooth_normals(s: &mut SurfaceMeshData) {
    joined_normals(s, -1.);
}

pub(super) fn mineral_normals(s: &mut SurfaceMeshData) {
    if !s.uvs.iter().any(|uv| uv[0] < -9000.) { return; }
    // Share lighting across coplanar source patches, but keep the normals on
    // either side of a fractured bed. Unconditional averaging made the actual
    // planar cuts read as swollen, rounded earth.
    joined_normals(s, 25_f32.to_radians().cos());
}

fn joined_normals(s: &mut SurfaceMeshData, crease: f32) {
    let key = |p: [f32; 3]| p.map(|v| (v * 10000.).round() as i64);
    let mut faces = HashMap::<[i64; 3], Vec<(Vec3, f32, bool)>>::new();
    let mut own = vec![Vec3::ZERO; s.positions.len()];
    for tri in s.indices.chunks_exact(3) {
        let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if s.uvs[ids[0]][0] >= -9000. { continue; }
        let p = ids.map(|i| Vec3::from_array(s.positions[i]));
        let cross = (p[1] - p[0]).cross(p[2] - p[0]);
        if cross.length_squared() < 1e-10 { continue; }
        let normal = cross.normalize();
        for corner in 0..3 {
            let a = p[(corner + 1) % 3] - p[corner];
            let b = p[(corner + 2) % 3] - p[corner];
            let angle = a.cross(b).length().atan2(a.dot(b));
            own[ids[corner]] += normal * angle;
            faces.entry(key(s.positions[ids[corner]])).or_default()
                .push((normal, angle, s.normals[ids[corner]][1] > 0.9));
        }
    }
    for i in 0..s.positions.len() {
        if s.uvs[i][0] >= -9000. || own[i].length_squared() < 1e-10 { continue; }
        let reference = own[i].normalize();
        if let Some(adjacent) = faces.get(&key(s.positions[i])) {
            // Native cap/wall shoulder seams describe one continuous bevel.
            // Keep them smooth; the crease rule applies inside mineral caps.
            let shoulder = adjacent.iter().any(|(_, _, cap)| !cap)
                && adjacent.iter().any(|(_, _, cap)| *cap);
            let normal = adjacent.iter()
                .filter(|(normal, _, _)| shoulder || normal.dot(reference) >= crease)
                .fold(Vec3::ZERO, |sum, (normal, angle, _)| sum + *normal * *angle);
            if normal.length_squared() > 0.0001 {
                s.normals[i] = normal.normalize().to_array();
            }
        }
    }
}

fn noise(p: Vec3) -> f32 {
    let cell = p.floor();
    let f = p - cell;
    let f = f * f * (Vec3::splat(3.0) - f * 2.0);
    let hash = |x: i32, y: i32, z: i32| {
        let mut h = (x as u32).wrapping_mul(0x9e3779b9)
            ^ (y as u32).wrapping_mul(0x85ebca6b)
            ^ (z as u32).wrapping_mul(0xc2b2ae35);
        h ^= h >> 16;
        h = h.wrapping_mul(0x7feb352d);
        (h & 65535) as f32 / 65535.0
    };
    let mut value = 0.0;
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                let weight = |t: f32, i: i32| if i == 0 { 1.0 - t } else { t };
                value += hash(cell.x as i32 + x, cell.y as i32 + y, cell.z as i32 + z)
                    * weight(f.x, x)
                    * weight(f.y, y)
                    * weight(f.z, z);
            }
        }
    }
    value
}

fn color(p: Vec3, n: Vec3) -> [f32; 4] {
    let broad = noise(p / 19.0);
    let grain = noise(p / 1.7);
    let strata = (p.y * 0.68 + noise(p / 8.0) * 2.5).sin();
    let mineral = 0.8 + broad * 0.3 + (grain - 0.5) * 0.16;
    let seam = ((strata.abs() - 0.08) / 0.2).clamp(0.0, 1.0);
    let mut c = Vec3::new(0.20, 0.205, 0.17) * mineral * (0.84 + seam * 0.16);
    let moss = ((noise(p / 10.0 + Vec3::splat(13.0)) - 0.62) * 3.0).clamp(0.0, 0.24) * n.y.max(0.0);
    c = c.lerp(Vec3::new(0.095, 0.16, 0.035), moss);
    c *= 0.65 + 0.35 * n.dot(Vec3::new(0.45, 0.8, 0.55).normalize()).max(0.0);
    [c.x, c.y, c.z, 1.0]
}

pub(super) fn canyon_color(p: Vec3, n: Vec3) -> [f32; 4] {
    let weather = noise(p / 14.0);
    let band = noise(Vec3::new(p.x / 32.0, p.y / 3.8, p.z / 32.0));
    let grain = noise(p / 1.5);
    let fracture = (p.y * 0.7 + noise(p / 9.0) * 2.0).sin().abs();
    let crease = ((fracture - 0.065) / 0.17).clamp(0.0, 1.0);
    let mut c = Vec3::new(0.30, 0.19, 0.085).lerp(Vec3::new(0.40, 0.30, 0.16), band)
        * (0.8 + weather * 0.3 + (grain - 0.5) * 0.12)
        * (0.65 + crease * 0.35);
    let moss = ((noise(p / 11.0 + Vec3::splat(17.0)) - 0.66) * 3.0).clamp(0.0, 0.18) * n.y.max(0.0);
    c = c.lerp(Vec3::new(0.10, 0.16, 0.035), moss);
    c *= 0.65 + 0.35 * n.dot(Vec3::new(0.45, 0.8, 0.55).normalize()).max(0.0);
    [c.x, c.y, c.z, 1.0]
}

pub(super) fn append(
    s: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    n: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    append_with_pigment(s, p, n, g, origin, color, 20000.0);
}

pub(super) fn append_canyon(
    s: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    n: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
) {
    append_with_pigment(s, p, n, g, origin, canyon_color, 40000.0);
}

/// Bare water-edge mineral keeps its authenticated source pigment. Reuse the
/// world-seeded fractured shelf and closed wall relief, without meadow moss.
pub(in crate::mesh) fn append_coast(
    s: &mut SurfaceMeshData, p: [[f32; 3]; 4], n: [f32; 3],
    g: &GridGeometry, origin: [i32; 2], palette: [[f32; 3]; 2],
) {
    super::new_bark::ledge_finish::append_shore(s, p, n, g, origin, palette);
}

fn append_with_pigment(
    s: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    n: [f32; 3],
    g: &GridGeometry,
    origin: [i32; 2],
    pigment: impl Fn(Vec3, Vec3) -> [f32; 4],
    texture_tag: f32,
) {
    let mut p = p.map(Vec3::from_array);
    // Some authored faces start with their horizontal edge. Rotate cyclically
    // so interpolation's v axis is vertical, retaining the original winding.
    if n[1].abs() <= 0.3 && (p[3].y - p[0].y).abs() > (p[1].y - p[0].y).abs() {
        p.rotate_left(1);
    }
    let offset = Vec3::new(
        origin[0] as f32 * g.tile_width - g.origin_x,
        0.0,
        origin[1] as f32 * g.tile_height - g.origin_z,
    );
    let normal = Vec3::from_array(n);
    let point = |u: f32, v: f32| {
        let q = p[0].lerp(p[3], u).lerp(p[1].lerp(p[2], u), v);
        if normal.y.abs() > 0.3 || u == 0. || u == 1. || v == 0. || v == 1. {
            // sin(PI) is not exactly zero in f32. Keep shared endpoints
            // bit-identical instead of introducing microscopic wall slits.
            return q;
        }
        // Chipped wall facets keep every cap and neighboring face perimeter
        // fixed, so relief cannot open another terrain seam.
        let relief = (std::f32::consts::PI * u).sin()
            * (std::f32::consts::PI * v).sin()
            * g.tile_width
            * 0.075
            * (0.3 + noise((q + offset) / 6.0) * 0.8);
        q + normal * relief
    };
    // Use common world-height rings on walls. Neighboring faces can start
    // on different ledge floors; fractional subdivisions of each face would
    // otherwise approximate the same curved edge with different chords.
    let wall = normal.y.abs() <= 0.3;
    let ring = g.tile_height / 6.0;
    let low = p.iter().map(|q| q.y).fold(f32::INFINITY, f32::min);
    let high = p.iter().map(|q| q.y).fold(f32::NEG_INFINITY, f32::max);
    let first = (low / ring).floor() as i32;
    // A merged wall can span several source cells. Six samples for its
    // entire width leave T-junctions against the six-per-cell cap borders.
    // Partition aligned walls at shared world-coordinate rings as well.
    let axis = if (p[3].x - p[0].x).abs() >= (p[3].z - p[0].z).abs() { 0 } else { 2 };
    let aligned = wall && (p[0][axis] - p[1][axis]).abs() < 0.0001
        && (p[3][axis] - p[2][axis]).abs() < 0.0001;
    let a = p[0][axis] + offset[axis];
    let b = p[3][axis] + offset[axis];
    let horizontal_ring = (if axis == 0 { g.tile_width } else { g.tile_height }) / 6.;
    let mut columns = vec![(0., a), (1., b)];
    if aligned && (b - a).abs() > 0.0001 {
        for k in (a.min(b) / horizontal_ring).floor() as i32 + 1
            ..(a.max(b) / horizontal_ring).ceil() as i32 {
            let level = k as f32 * horizontal_ring;
            let u = (level - a) / (b - a);
            if u > 0.00001 && u < 0.99999 { columns.push((u, level)); }
        }
        columns.sort_by(|a, b| a.0.total_cmp(&b.0));
    } else {
        columns = (0..=6).map(|i| {
            let u = i as f32 / 6.;
            (u, a + (b - a) * u)
        }).collect();
    }
    let height_rings = if wall {
        let jump = crate::profile::JUMP_LEDGE_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT;
        let epsilon = g.tile_height * 0.000001;
        let mut levels = (first..=(high / ring).ceil() as i32).map(|k| {
            let y = k as f32 * ring;
            let authored = (y / jump).round() * jump;
            if (y - authored).abs() < epsilon { authored } else { y }
        }).collect::<Vec<_>>();
        // The sixth-cell lattice misses six-pixel ledge floors. A wall
        // starting there gains a clamped vertex while its taller neighbor
        // spans past it. Include the same authored floors on every wall so
        // their common edge remains conforming through mineral deformation.
        for k in (low / jump).floor() as i32 + 1..(high / jump).ceil() as i32 {
            levels.push(k as f32 * jump);
        }
        levels.sort_by(f32::total_cmp);
        levels.dedup_by(|a,b| (*a-*b).abs() < epsilon);
        levels
    } else { Vec::new() };
    let rows = if wall { height_rings.len() - 1 } else { 6 };
    for row in 0..rows {
        for interval in columns.windows(2) {
            let [(u, along), (end_u, end_along)] = [interval[0], interval[1]];
            let at = |u: f32, along: f32, level: f32| {
                let bottom = p[0].lerp(p[3], u).y;
                let top = p[1].lerp(p[2], u).y;
                let v = if (top - bottom).abs() > 0.0001 {
                    ((level - bottom) / (top - bottom)).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let mut q = point(u, v);
                if aligned { q[axis] = along - offset[axis]; }
                // Preserve the shared world ring exactly. Reconstructing Y
                // through differently based lerps yields unequal seam bits.
                q.y = level.clamp(bottom.min(top), bottom.max(top));
                q
            };
            let quad = if wall {
                let lower = height_rings[row];
                let upper = height_rings[row + 1];
                let (start, end) = if p[1].y >= p[0].y {
                    (lower, upper)
                } else {
                    (upper, lower)
                };
                [
                    at(u, along, start),
                    at(u, along, end),
                    at(end_u, end_along, end),
                    at(end_u, end_along, start),
                ]
            } else {
                let v = row as f32 / 6.0;
                let step = 1. / 6.;
                [
                    point(u, v),
                    point(u, v + step),
                    point(end_u, v + step),
                    point(end_u, v),
                ]
            };
            let area = (quad[1] - quad[0]).cross(quad[2] - quad[0])
                + (quad[2] - quad[0]).cross(quad[3] - quad[0]);
            if area.length_squared() < 0.0000001 {
                continue;
            }
            let face = area.normalize();
            append_quad_colors(
                s,
                quad.map(|p| p.to_array()),
                face.to_array(),
                quad.map(|p| {
                    let world = p + offset;
                    [-10000.0 - world.x, -texture_tag - world.z]
                }),
                quad.map(|p| pigment(p + offset, face)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjoining_mineral_walls_share_every_authored_ledge_floor_sample() {
        for scale in [0.5,1.,2.] {
            let g=GridGeometry {width:4,height:4,tile_width:8.*scale,tile_height:8.*scale,
                origin_x:-13.*scale,origin_z:0.};
            let (w,e,n,s)=g.bounds(0,0);
            let height=32.*scale;
            let jump=crate::profile::JUMP_LEDGE_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
            for course in 1..=4 {
                let floor=course as f32*jump;
                let mut tall=SurfaceMeshData::default();
                let mut short=SurfaceMeshData::default();
                append_canyon(&mut tall,[[w,0.,n],[w,height,n],[e,height,n],[e,0.,n]],
                    [0.,0.,-1.],&g,[11,-7]);
                append_canyon(&mut short,[[e,floor,n],[e,height,n],[e,height,s],[e,floor,s]],
                    [1.,0.,0.],&g,[11,-7]);
                let samples=|mesh:&SurfaceMeshData|mesh.positions.iter()
                    .filter(|p|(p[0]-e).abs()<0.0001 && (p[2]-n).abs()<0.0001 && p[1]>=floor)
                    .map(|p|p.map(f32::to_bits)).collect::<std::collections::BTreeSet<_>>();
                let tall=samples(&tall);let short=samples(&short);
                assert!(tall.contains(&[e.to_bits(),floor.to_bits(),n.to_bits()]),
                    "the taller wall must meet its neighbor's exact foot point");
                assert_eq!(tall,short,"matching edge samples must survive scale {scale}, course {course}");
            }
        }
    }

    #[test]
    fn kanto_portal_arch_keeps_its_seating_joint_while_remote_mineral_faces_erode() {
        let art=[[0x11,0x11,0x24,0x24],[0x11,0x11,0x24,0x24],
            [0x48,0x49,0x34,0x24],[0x58,0x59,0x37,0x34]];
        let sources=(0..64).map(|i| {
            let (x,z)=(i%8,i/8);
            let portal=(2..6).contains(&x)&&(2..6).contains(&z);
            let rock=(1..7).contains(&x)&&(1..7).contains(&z);
            let mut s=super::super::tests::source_with_tile(
                if portal {0x70}else if rock {0x2c}else{0x31},
                if portal {(x-2)as u8}else{(x%4)as u8},
                if portal {(z-2)as u8}else{(z%4)as u8},
                if portal {art[z-2][x-2]}else if rock {0x11}else{0x39});
            s.tileset_id="kanto".into();s
        }).collect();
        let frame=super::super::tests::frame(8,8,sources);
        let cells=frame.tiles.iter().collect::<Vec<_>>();
        let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
        let g=GridGeometry {width:8,height:8,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut mesh=TerrainMeshData {authored_cells:vec![None;64],..Default::default()};
        super::super::mountain_portals::append(&mut mesh,&cells,&shapes,&g,&mut [false;64],[0,0]);
        let portal_end=mesh.solid.positions.len();
        assert!(portal_end>0,"exercise the actual complete native entrance producer");
        append_canyon(&mut mesh.solid,[[8.,0.,8.],[8.,16.,8.],[8.,16.,56.],[8.,0.,56.]],
            Vec3::NEG_X.to_array(),&g,[0,0]);
        let before=mesh.solid.positions.clone();
        sculpt_rock_rims_on_map(&mut mesh,"Route10North",&cells,&shapes,&g,[0,0]);
        assert_eq!(&mesh.solid.positions[..portal_end],&before[..portal_end],
            "weathering cannot tear the closed arch away from its source seat");
        assert!(mesh.solid.positions[portal_end..].iter().zip(&before[portal_end..]).any(|(a,b)|a!=b),
            "a portal must not disable shaping of the surrounding mountain");
    }

    #[test]
    fn blackthorn_cap_pigment_removes_false_relief_without_repainting_walls_or_other_maps() {
        let mut surface = SurfaceMeshData::default();
        surface.positions = vec![[0., 16., 0.]; 5];
        surface.normals = vec![[0., 1., 0.], [0., 1., 0.], [0., 0., 1.],
            [0., 1., 0.], [0., 1., 0.]];
        surface.uvs = vec![[-10000., -40000.], [-10008., -40000.],
            [-10000., -40000.], [-10000., -20000.], [0., 0.]];
        surface.colors = vec![[0.16, 0.11, 0.05, 1.], [0.39, 0.28, 0.15, 1.],
            [0.18, 0.12, 0.06, 1.], [0.20, 0.205, 0.17, 1.], [0.8, 0.7, 0.6, 1.]];
        let original = surface.clone();
        finish_mountain_cap_pigment(&mut surface, "Route26");
        assert_eq!(surface.colors, original.colors);
        finish_mountain_cap_pigment(&mut surface, "BlackthornCity");
        assert_eq!(surface.colors[0], surface.colors[1], "flat caps must not carry false baked relief");
        assert_eq!(&surface.colors[2..], &original.colors[2..], "retain sediment walls and other materials");
        assert_eq!(surface.positions, original.positions);
        assert_eq!(surface.normals, original.normals);
        assert_eq!(surface.uvs, original.uvs);
    }

    #[test]
    fn merged_wall_border_matches_both_source_cell_caps() {
        let g = GridGeometry { width: 2, height: 1, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let mut caps = SurfaceMeshData::default();
        for west in [0., 8.] {
            append_canyon(&mut caps, [[west,16.,0.], [west,16.,8.],
                [west+8.,16.,8.], [west+8.,16.,0.]], [0.,1.,0.], &g, [0,0]);
        }
        let mut wall = SurfaceMeshData::default();
        append_canyon(&mut wall, [[16.,6.,8.], [16.,16.,8.],
            [0.,16.,8.], [0.,6.,8.]], [0.,0.,1.], &g, [0,0]);
        for point in caps.positions.iter().filter(|p| (p[2]-8.).abs()<0.0001) {
            assert!(wall.positions.iter().any(|q|
                (Vec3::from_array(*point)-Vec3::from_array(*q)).length()<0.00001),
                "merged wall omits cap-border sample {point:?}");
        }
        assert!(wall.positions.iter().all(|p| p[1]>=6. && p[1]<=16.));
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_route_rock_caps_retain_winding_after_the_combined_finish() {
        check_external_rock_caps(&["NewBarkTown", "Route29", "CherrygroveCity", "Route30",
            "Route31", "VioletCity", "Route26", "Route27", "Route43",
            "Route45", "Route46", "LakeOfRage"],100_000);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_blackthorn_angular_contacts_retain_source_caps_and_support() {
        check_external_rock_caps(&["BlackthornCity"],1_000);
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_kanto_mountain_shoulders_retain_caps_and_support() {
        check_external_rock_caps(&["CeruleanCity","Route24","Route25","Route10North",
            "CinnabarIsland"],1_000);
    }

    fn check_external_rock_caps(maps: &[&str], minimum_caps: usize) {
        let path = std::env::var("CRYSTAL_RENDER_TEST_PACK")
            .expect("set CRYSTAL_RENDER_TEST_PACK to a completed local pack");
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let mut total_caps = 0;
        // Include both rocky routes and the early-game town/route chain.
        // These are actual source layouts, rather than repeats of Route26.
        for &map in maps {
            let module = &pack.data().maps[map];
            let tileset = &module.attributes.tileset_name;
            let layout = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let width = module.attributes.width as usize * 4;
            let height = module.attributes.height as usize * 4;
            let sources = (0..height).flat_map(|z| (0..width).map(move |x| {
                let block = module.blocks[z / 4 * (width / 4) + x / 4];
                VisualTileSource {
                    tileset_id: tileset.as_str().into(), metatile_id: block,
                    subtile_column: (x % 4) as u8, subtile_row: (z % 4) as u8,
                    tile_index: layout[block as usize * 16 + z % 4 * 4 + x % 4] as u16,
                }
            })).collect();
            let frame = super::super::tests::frame(width as u32, height as u32, sources);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let mut shapes: Vec<_> = cells.iter().map(|c|
                shape_for_source_on_map(map, &c.source)).collect();
            resolve_rock_platform_tiers(&cells, &mut shapes, width);
            resolve_authored_mountain_tiers(&mut shapes, width, height);
            let g = GridGeometry { width, height, tile_width: 16., tile_height: 16.,
                origin_x: 0., origin_z: 0. };
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; cells.len()];
            mesh.footing_heights = shapes.iter().map(|s| s.surface_height(g.tile_height)).collect();
            for (i, cell) in cells.iter().enumerate() {
                let johto=matches!(cell.source.tileset_id.as_ref(), "johto" | "johto_modern")
                    && (cell.source.metatile_id == 0x0a || (0x68..=0x73).contains(&cell.source.metatile_id));
                let kanto=cell.source.tileset_id.as_ref()=="kanto"
                    && super::super::outdoor_mountains::permanent(&cell.source,shapes[i]);
                if !johto && !kanto {
                    continue;
                }
                let h = shapes[i].surface_height(g.tile_height);
                if h <= 0. { continue; }
                let (x, z) = (i % width, i / width);
                let (w, e, n, south) = g.bounds(x, z);
                let append_face = if johto && cell.source.metatile_id == 0x0a { append } else { append_canyon };
                append_face(&mut mesh.solid, [[w,h,n],[w,h,south],[e,h,south],[e,h,n]],
                    Vec3::Y.to_array(), &g, [0,0]);
                for (nx, nz, a, b, normal) in [
                    (x.wrapping_sub(1),z,[w,h,south],[w,h,n],Vec3::NEG_X),
                    (x+1,z,[e,h,n],[e,h,south],Vec3::X),
                    (x,z.wrapping_sub(1),[w,h,n],[e,h,n],Vec3::NEG_Z),
                    (x,z+1,[e,h,south],[w,h,south],Vec3::Z),
                ] {
                    if nx >= width || nz >= height { continue; }
                    let floor = shapes[nz*width+nx].surface_height(g.tile_height).max(0.);
                    if floor >= h { continue; }
                    let mut low_a = a; low_a[1] = floor;
                    let mut low_b = b; low_b[1] = floor;
                    append_face(&mut mesh.solid, [low_a,a,b,low_b], normal.to_array(), &g, [0,0]);
                }
            }
            // Keep authored face classification through sculpting. UV winding
            // alone mislabels weathered vertical wall projections as caps.
            let original_positions=mesh.solid.positions.clone();
            let original_normals=mesh.solid.normals.clone();
            let mut classified = mesh.clone();
            sculpt_rock_rims_on_map(&mut classified, map, &cells, &shapes, &g, [0,0]);
            let footing = mesh.footing_heights.clone();
            finish_rock_geometry(&mut mesh, map, &cells, &shapes, &g, [0,0],None);
            assert_eq!(mesh.footing_heights, footing, "{map} source support changed");
            assert!(mesh.authored_cells.iter().all(Option::is_none), "{map} claims changed");
            if map=="BlackthornCity" {
                let mut groups=HashMap::<[i64;3],(Vec3,Vec3,bool,bool)>::new();
                for &i in &mesh.solid.indices {
                    let i=i as usize;
                    if i>=original_positions.len() || mesh.solid.uvs[i][0]>=-9000. {continue;}
                    let key=original_positions[i].map(|v|(v*100.).round() as i64);
                    let p=Vec3::from_array(mesh.solid.positions[i]);
                    let cap=original_normals[i][1]>0.9;
                    let entry=groups.entry(key).or_insert((p,p,false,false));
                    entry.0=entry.0.min(p);entry.1=entry.1.max(p);
                    entry.2|=cap;entry.3|=!cap;
                }
                let mut shared=0;
                for (key,(low,high,cap,wall)) in groups {
                    if cap && wall {
                        assert!((high-low).max_element()<0.001,
                            "{map} cap/wall seam split at {key:?}: {low:?}..{high:?}");
                        shared+=1;
                    }
                }
                assert!(shared>100,"the real layout must exercise cap/wall joins");
                println!("{map}: {shared} shared cap/wall samples checked");
            }
            let mut caps = 0;
            for ids in mesh.solid.indices.chunks_exact(3) {
                if mesh.solid.uvs[ids[0] as usize][0] >= -9000. { continue; }
                if classified.solid.normals[ids[0] as usize][1] <= 0.9 { continue; }
                let p = [ids[0],ids[1],ids[2]].map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                let cross = (p[1]-p[0]).cross(p[2]-p[0]);
                if cross.y < -0.0001 {
                    let before=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(classified.solid.positions[i as usize]));
                    let mut circular=classified.clone();
                    super::super::rock_bases::round_geometry(&mut circular,"Route26",&cells,&shapes,&g,[0,0]);
                    let control=[ids[0],ids[1],ids[2]].map(|i|Vec3::from_array(circular.solid.positions[i as usize]));
                    let source=[ids[0],ids[1],ids[2]].map(|i|classified.solid.uvs[i as usize]);
                    panic!("{map} inverted native cap: {p:?}; before base: {before:?}; circular control: {control:?}; source UV: {source:?}");
                }
                caps += 1;
            }
            total_caps += caps;
            println!("{map}: {caps} source rock cap triangles checked");
        }
        assert!(total_caps > minimum_caps, "the external layouts must exercise substantial rock geometry");
    }
    #[test]
    fn cap_mineral_clefts_have_resolved_valleys_between_broad_planes() {
        let tile = 8.;
        let flat = super::super::tests::source_with_tile(0,0,0,5);
        let mut sources = vec![flat;100];
        for z in 1..9 { for x in 1..9 {
            sources[z*10+x] = super::super::tests::source_with_tile(0x70,2,1,0x3c);
        }}
        let frame = super::super::tests::frame(10,10,sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let mut shapes: Vec<_> = cells.iter().map(|c|shape_for_source(&c.source)).collect();
        for z in 1..9 {for x in 1..9 {
            shapes[z*10+x] = CellShape::RaisedTop {height:32.,solid:SolidKind::Bank};
        }}
        let g = GridGeometry {width:10,height:10,tile_width:tile,tile_height:tile,
            origin_x:0.,origin_z:0.};
        let mut valleys = 0;
        let mut resolved_cleft_pairs = 0;
        for origin in [[0,0],[17,43],[-31,-7],[64,-32]] {
            let mut mesh = TerrainMeshData::default();
            for z in 1..9 {for x in 1..9 {
                let (w,e,n,s)=g.bounds(x,z);
                append_canyon(&mut mesh.solid,[[w,32.,n],[w,32.,s],[e,32.,s],[e,32.,n]],
                    Vec3::Y.to_array(),&g,origin);
            }}
            let before=mesh.solid.positions.clone();
            soften_rock_rims(&mut mesh,&cells,&shapes,&g,origin);
            let samples: HashMap<_,_> = before.iter().zip(&mesh.solid.positions)
                .filter(|(p,_)|p[0]>=tile*3. && p[0]<=tile*7.
                    && p[2]>=tile*3. && p[2]<=tile*7.)
                .map(|(p,q)| (((p[0]/tile*6.).round() as i32,
                    (p[2]/tile*6.).round() as i32),q[1])).collect();
            for (&(x,z),&center) in &samples {
                // Compare actual interior vertices with the broad-bed-only
                // surface. A cleft must survive on adjacent mesh samples,
                // rather than becoming isolated pits after tessellation.
                let broad_only = |x: i32, z: i32| {
                    let depth = crest_depth(x as f32 / 6. + origin[0] as f32,
                        z as f32 / 6. + origin[1] as f32) * tile * 0.55;
                    32. - depth.min((32. - crate::profile::JUMP_LEDGE_HEIGHT) * 0.35)
                };
                if broad_only(x,z) - center > tile * 0.08 {
                    for next in [(x+1,z),(x,z+1)] {
                        if samples.get(&next).is_some_and(|&height|
                            broad_only(next.0,next.1) - height > tile * 0.08) {
                            resolved_cleft_pairs += 1;
                        }
                    }
                }
                for (a,b) in [((x-1,z),(x+1,z)),((x,z-1),(x,z+1))] {
                    if let (Some(&a),Some(&b))=(samples.get(&a),samples.get(&b)) {
                        if a.min(b)-center>tile*0.08 { valleys+=1; }
                    }
                }
            }
            for ids in mesh.solid.indices.chunks_exact(3) {
                let p=ids.iter().map(|&i|Vec3::from_array(mesh.solid.positions[i as usize]))
                    .collect::<Vec<_>>();
                assert!((p[1]-p[0]).cross(p[2]-p[0]).y>=-0.0001,
                    "mineral clefts must not fold the cap at source phase {origin:?}");
            }
        }
        assert!(valleys>12,"cap beds need resolved geometric clefts, found {valleys}");
        assert!(resolved_cleft_pairs>24,
            "clefts must span neighboring cap samples, found {resolved_cleft_pairs} pairs");
    }
    #[test]
    fn inside_shelf_join_retains_one_vertical_edge_for_both_walls_and_cap() {
        for tile in [8., 32.] {
            let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
            let mut sources = vec![flat; 25];
            for i in [7, 11, 12] {
                sources[i] = super::super::tests::source_with_tile(0x70, 2, 1, 0x3c);
            }
            let f = super::super::tests::frame(5, 5, sources);
            let cells: Vec<_> = f.tiles.iter().collect();
            let mut shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            for i in [7, 11, 12] { shapes[i] = CellShape::RaisedTop { height: 32., solid: SolidKind::Bank }; }
            let g = GridGeometry { width: 5, height: 5, tile_width: tile,
                tile_height: tile, origin_x: 0., origin_z: 0. };
            let h = tile * 4.; let c = tile * 2.;
            let mut mesh = TerrainMeshData::default();
            append(&mut mesh.solid, [[c,0.,tile],[c,h,tile],[c,h,c],[c,0.,c]],
                Vec3::NEG_X.to_array(), &g, [0,0]);
            append(&mut mesh.solid, [[tile,0.,c],[tile,h,c],[c,h,c],[c,0.,c]],
                Vec3::NEG_Z.to_array(), &g, [0,0]);
            append(&mut mesh.solid, [[c,h,c],[c,h,c+tile],[c+tile,h,c+tile],[c+tile,h,c]],
                Vec3::Y.to_array(), &g, [0,0]);
            let original = mesh.solid.positions.clone();
            soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0,0]);
            let mut shared = 0; let mut weathered = 0;
            for (p,q) in original.iter().zip(&mesh.solid.positions) {
                if p[0] == c && p[2] == c {
                    assert_eq!(p,q,"the inside shelf edge must stay joined at either render scale");
                    shared += 1;
                } else if Vec3::from_array(*p).distance(Vec3::from_array(*q)) > tile * 0.01 {
                    weathered += 1;
                }
            }
            assert!(shared > 12); assert!(weathered > 12);
        }
    }
    #[test]
    fn lower_wall_stays_supported_beyond_the_thin_relief_skin_at_a_floor_join() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 16];
        for (i, x, y) in [(5, 1, 1), (6, 2, 1), (9, 1, 2), (10, 2, 2)] {
            sources[i] = super::super::tests::source_with_tile(0x0a, x, y, 0x3c);
        }
        let frame = super::super::tests::frame(4, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry { width: 4, height: 4, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let mut shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        shapes[4] = CellShape::RaisedTop { height: 10., solid: SolidKind::Bank };
        let h = shapes[9].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        append(&mut mesh.solid,
            [[8., 0., 24.], [8., h, 24.], [8., h, 16.], [8., 0., 16.]],
            Vec3::NEG_X.to_array(), &g, [0, 0]);
        let before = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut checked = 0;
        for (p, q) in before.iter().zip(&mesh.solid.positions) {
            if (p[1] - 8.).abs() < 0.001 && p[2] > 17.2 && p[2] < 17.5 {
                assert_eq!(p, q, "the adjoining seat must support the whole shared bevel region");
                checked += 1;
            }
        }
        assert!(checked > 0);
    }
    #[test]
    fn shorter_rock_bevel_joins_a_taller_wall_without_a_height_cutoff() {
        let below = mineral_bevel(0., 0., 0.5, -0.0099, 8.);
        let above = mineral_bevel(0., 0., 0.5, -0.0101, 8.);
        assert!(below.distance(above) < 0.001,
            "the old shorter-cap cutoff must not separate adjacent wall samples");
        assert!(above.length() > 2.);
        assert!(mineral_bevel(0., 0., 0.5, -7.999, 8.).length() < 0.001);
        assert_eq!(mineral_bevel(0., 0., 0.5, -8., 8.), Vec2::ZERO);
        assert_eq!(mineral_bevel(0., 0., 0.5, -16., 8.), Vec2::ZERO);
    }

    #[test]
    fn mineral_shoulder_height_rings_follow_one_planar_cut() {
        let width = 8.;
        let cut = width * (2. - std::f32::consts::SQRT_2);
        for inward in [0., 0.5, 1., 2.] {
            for depth in [0., 0.5, 1., 2.] {
                let delta = mineral_bevel(inward, inward, 0.5, depth, width);
                let projected = Vec2::new(inward + delta.x, depth - delta.y);
                assert!((projected.x + projected.y - cut).abs() < 0.00001,
                    "height rings must describe a mineral plane, not a rolled fillet: {projected:?}");
                assert!(projected.x >= inward && projected.y >= depth);
            }
        }
        assert_eq!(mineral_bevel(5., 5., 0.5, 2., width), Vec2::ZERO);
        assert_eq!(mineral_bevel(0., width, 0.5, 0., width), Vec2::ZERO);
    }

    #[test]
    fn perpendicular_bevels_preserve_reentrant_route26_cap_winding() {
        let points = [Vec3::new(190.66667, 32., 288.),
            Vec3::new(192., 32., 289.33334), Vec3::new(192., 32., 288.)];
        // Actual production contributions before erosion at this cap corner.
        let shifts = [1.3745648, 1.3331273, 2.189051];
        let sculpted = std::array::from_fn::<_, 3, _>(|i| {
            let rim = combined_rim_shift([0., shifts[i], shifts[i], 0.]);
            points[i] + Vec3::new(rim.x, -shifts[i], rim.y)
        });
        assert!((sculpted[1] - sculpted[0]).cross(sculpted[2] - sculpted[0]).y > 0.1,
            "perpendicular bevels must preserve the cap's winding: {sculpted:?}");
        assert_eq!(combined_rim_shift([0., 2., 0., 0.]), Vec2::new(-2., 0.));
        assert!(combined_rim_shift([0., 2., 0.00001, 0.])
            .distance(Vec2::new(-2., 0.)) < 0.00002);
    }

    #[test]
    fn finite_rim_endpoint_preserves_route26_cap_triangle_winding() {
        // These neighboring source samples folded in the Route26 pack:
        // the outer sample received a full bevel while its neighbor missed
        // the finite edge's radius and stayed fixed.
        let a = Vec2::new(192., 280.);
        let b = Vec2::new(192., 288.);
        let normal = Vec2::X;
        let points = [
            Vec3::new(190.66667, 32., 293.33334),
            Vec3::new(190.66667, 32., 294.6667),
            Vec3::new(192., 32., 294.6667),
        ];
        let sculpted = points.map(|p| {
            let along = ((p.xz() - a).dot(b - a) / (b - a).length_squared())
                .clamp(0., 1.);
            let distance = p.xz().distance(a.lerp(b, along));
            let width = 8. * 0.90 * (0.80 + noise(p / (8. * 0.7)) * 0.40);
            let delta = mineral_bevel((p.xz() - a).dot(-normal), distance,
                along, 0., width);
            p + Vec3::new(-delta.x, delta.y, 0.)
        });
        assert!((sculpted[1] - sculpted[0]).cross(sculpted[2] - sculpted[0]).y > 1.,
            "a finite rim endpoint must keep adjacent cap samples in order: {sculpted:?}");
    }

    #[test]
    fn shoulder_normals_follow_corner_angles_instead_of_triangle_area() {
        for span in [2., 8., 24.] {
            for split in [0.2, 0.5, 0.8] {
                let mut mesh = SurfaceMeshData::default();
                let edge = Vec3::Z * span;
                let side = Vec3::X * span;
                let middle = edge.lerp(side, split);
                for points in [[Vec3::ZERO, edge, middle],
                    [Vec3::ZERO, middle, side], [Vec3::ZERO, side, Vec3::Y * 2.]] {
                    let first = mesh.positions.len() as u32;
                    mesh.positions.extend(points.map(|p| p.to_array()));
                    mesh.normals.extend([[0., 1., 0.]; 3]);
                    mesh.uvs.extend([[-10000., -20000.]; 3]);
                    mesh.indices.extend([first, first + 1, first + 2]);
                }
                smooth_normals(&mut mesh);
                let expected = (Vec3::Y + Vec3::Z).normalize();
                for i in [0, 3, 6] {
                    assert!(Vec3::from_array(mesh.normals[i]).dot(expected) > 0.99999);
                    assert_eq!(mesh.normals[i], mesh.normals[0]);
                }
            }
        }
    }

    #[test]
    fn mineral_fractures_keep_separate_normals_without_opening_source_seams() {
        for authored_normal in [[0., 1., 0.], [0., 0., 1.]] {
        for degrees in [10_f32, 50.] {
            let mut mesh = SurfaceMeshData::default();
            let angle = degrees.to_radians();
            let side = Vec3::new(angle.cos(), angle.sin(), 0.);
            for points in [[Vec3::ZERO, Vec3::Z, Vec3::NEG_X],
                [Vec3::ZERO, side, Vec3::Z]] {
                let first = mesh.positions.len() as u32;
                mesh.positions.extend(points.map(|p| p.to_array()));
                mesh.normals.extend([authored_normal; 3]);
                mesh.uvs.extend([[-10000., -20000.]; 3]);
                mesh.indices.extend([first, first + 1, first + 2]);
            }
            let positions = mesh.positions.clone();
            let indices = mesh.indices.clone();
            mineral_normals(&mut mesh);
            assert_eq!(mesh.positions, positions);
            assert_eq!(mesh.indices, indices);
            let a = Vec3::from_array(mesh.normals[0]);
            let b = Vec3::from_array(mesh.normals[3]);
            if degrees < 25. {
                assert!(a.dot(b) > 0.99999, "shallow source seams must share lighting");
            } else {
                assert!((a.dot(b) - angle.cos()).abs() < 0.00001,
                    "fracture planes must retain their actual angle");
            }
            assert!(a.is_normalized() && b.is_normalized());
        }
        }
    }

    #[test]
    fn native_free_rock_corners_break_the_box_toe_and_seal_both_walls() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 36];
        for row in 1..5 { for col in 1..5 {
            sources[row * 6 + col] = super::super::tests::source_with_tile(
                0x0a, (col - 1) as u8, (row - 1) as u8, 0x3c);
        }}
        let f = super::super::tests::frame(6, 6, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry { width: 6, height: 6, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let h = shapes[7].surface_height(8.);
        let mut original = TerrainMeshData::default();
        for (p, n) in [
            ([[8., 0., 40.], [8., h, 40.], [8., h, 8.], [8., 0., 8.]], Vec3::NEG_X),
            ([[8., 0., 8.], [8., h, 8.], [40., h, 8.], [40., 0., 8.]], Vec3::NEG_Z),
        ] {
            append(&mut original.solid, p, n.to_array(), &g, [0, 0]);
        }
        let mut fragments = f.tiles.clone();
        for cell in &mut fragments {
            if cell.source.metatile_id == 0x0a {
                cell.source.subtile_column = 1;
                cell.source.subtile_row = 1;
            }
        }
        let conservative_cells: Vec<_> = fragments.iter().collect();
        for origin in [[0, 0], [-31, 17], [64, -22]] {
            let mut mesh = original.clone();
            soften_rock_rims(&mut mesh, &cells, &shapes, &g, origin);
            let mut conservative = original.clone();
            soften_rock_rims(&mut conservative, &conservative_cells, &shapes, &g, origin);
            let mut moved = 0;
            let mut shared = HashMap::new();
            for (a, b) in original.solid.positions.iter().zip(&mesh.solid.positions) {
                if a[1] == 0. { assert_eq!(a, b, "floor contact is authoritative"); }
                if a[0] == 8. && a[2] == 8. && a[1] > 1. && a[1] < 6. {
                    assert!(b[0] > a[0] + 0.01 && b[2] > a[2] + 0.01);
                    moved += 1;
                }
                if let Some(previous) = shared.insert(a.map(f32::to_bits), *b) {
                    assert_eq!(previous, *b, "corner walls must share the same compression");
                }
            }
            assert!(moved >= 4, "both walls must lose the rigid square toe");
            for (i, p) in original.solid.positions.iter().enumerate() {
                if p[1] >= 6. {
                    assert_eq!(mesh.solid.positions[i], conservative.solid.positions[i],
                        "toe work must preserve the established upper shoulder");
                }
            }
        }
        let mut adjoining = shapes.clone();
        adjoining[6] = CellShape::RaisedTop { height: 6., solid: SolidKind::Bank };
        let mut guarded = original.clone();
        soften_rock_rims(&mut guarded, &cells, &adjoining, &g, [0, 0]);
        for (a, b) in original.solid.positions.iter().zip(&guarded.solid.positions) {
            if a[0] == 8. && a[2] == 8. && a[1] <= 6. {
                assert_eq!(a, b, "a native corner beside a bank must retain its closure");
            }
        }
    }

    #[test]
    fn free_rock_lower_faces_weather_without_moving_the_floor_or_shared_samples() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 12];
        sources[5] = super::super::tests::source_with_tile(0x0a, 1, 1, 0x3c);
        sources[6] = super::super::tests::source_with_tile(0x0a, 2, 1, 0x3c);
        let f = super::super::tests::frame(4, 3, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry { width: 4, height: 3, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let h = shapes[5].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        for w in [8., 16.] {
            append(&mut mesh.solid, [[w + 8., 0., 16.], [w + 8., h, 16.],
                [w, h, 16.], [w, 0., 16.]], Vec3::Z.to_array(), &g, [0, 0]);
        }
        let before = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut moved = false;
        let mut shared = HashMap::new();
        for (a, b) in before.iter().zip(&mesh.solid.positions) {
            if a[1] == 0. { assert_eq!(a, b, "actual ground contacts stay fixed"); }
            if a[1] > 1. && a[1] < 6. && (a[0] - 16.).abs() < 0.01 {
                moved |= b[2] < a[2] - 0.05;
            }
            if let Some(previous) = shared.insert(a.map(f32::to_bits), *b) {
                assert_eq!(previous, *b, "lower weathering cannot open a wall seam");
            }
        }
        assert!(moved, "a free rock must not retain the old six-unit box at its foot");
    }

    #[test]
    fn plateau_mass_has_relief_with_closed_crests_and_level_tree_seats() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 49];
        for row in 2..5 {
            for col in 2..5 {
                sources[row * 7 + col] = super::super::tests::source_with_tile(0x70, 2, 1, 0x3c);
            }
        }
        let f = super::super::tests::frame(7, 7, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 7,
            height: 7,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let h = shapes[16].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        for row in 2..5 {
            for col in 2..5 {
                let (w, e, n, s) = g.bounds(col, row);
                append_canyon(
                    &mut mesh.solid,
                    [[w, h, n], [w, h, s], [e, h, s], [e, h, n]],
                    Vec3::Y.to_array(),
                    &g,
                    [0, 0],
                );
            }
            let (_, e, n, s) = g.bounds(4, row);
            append_canyon(
                &mut mesh.solid,
                [[e, 0., s], [e, h, s], [e, h, n], [e, 0., n]],
                Vec3::X.to_array(),
                &g,
                [0, 0],
            );
        }
        let unshaped = mesh.clone();
        let before = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut edge = Vec::new();
        let mut center = 0;
        for (a, b) in before.iter().zip(&mesh.solid.positions) {
            if a[1] <= 6. {
                assert_eq!(a, b, "floor contacts stay fixed");
            }
            if (a[0] - 40.).abs() < 0.001 && (a[2] - 28.).abs() < 0.001 && a[1] == h {
                assert!(
                    b[0] < 37.5,
                    "the upper silhouette must recede beyond a narrow fillet"
                );
                edge.push(*b);
            }
            if *a == [28., h, 28.] {
                assert_eq!([a[0], a[2]], [b[0], b[2]], "mass relief keeps its footprint");
                assert!(b[1] < h - 0.01 && b[1] > 6.,
                    "blocked plateau interiors need relief above the floor");
                center += 1;
            }
        }
        assert!(center > 0 && edge.len() >= 2);
        assert!(
            edge.iter().all(|p| *p == edge[0]),
            "cap and wall share one recessed crest"
        );
        assert!(mesh.solid.positions.iter().chain(&mesh.solid.normals)
            .all(|p| Vec3::from_array(*p).is_finite()));
        let mut planted_tiles = f.tiles.clone();
        planted_tiles[24].source.metatile_id = 0x6c;
        planted_tiles[24].source.tile_index = 0x20; // custom planted ink, not the native mineral corner
        let planted_cells: Vec<_> = planted_tiles.iter().collect();
        let planted_shapes: Vec<_> = planted_cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let mut seated = unshaped.clone();
        soften_rock_rims(&mut seated, &planted_cells, &planted_shapes, &g, [0, 0]);
        for (a, b) in before.iter().zip(&seated.solid.positions) {
            if *a == [28., h, 28.] {
                assert_eq!(a, b, "planted trees retain their level rock seat");
            }
        }
        let mut shifted = unshaped;
        for p in &mut shifted.solid.positions { p[0] -= 8.; }
        let scrolled = GridGeometry { origin_x: -8., ..g };
        soften_rock_rims(&mut shifted, &cells, &shapes, &scrolled, [0,0]);
        for (a,b) in mesh.solid.positions.iter().zip(&shifted.solid.positions) {
            assert!((Vec3::from_array(*a)-Vec3::from_array(*b)-Vec3::X*8.).length()<0.0001,
                "fractured cap and wall cannot move when the viewport scrolls");
        }
    }

    #[test]
    fn cropped_rock_interior_keeps_its_fractured_top_without_an_exposed_rim() {
        let rock = super::super::tests::source_with_tile(0x70, 2, 1, 0x3c);
        let f = super::super::tests::frame(3, 3, vec![rock; 9]);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry { width: 3, height: 3, tile_width: 8., tile_height: 8., origin_x: 0., origin_z: 0. };
        let h = shapes[4].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        append_canyon(&mut mesh.solid,
            [[8.,h,8.],[8.,h,16.],[16.,h,16.],[16.,h,8.]],
            Vec3::Y.to_array(), &g, [20,-10]);
        let before = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [20,-10]);
        let mut low = h;
        let mut high = 0.0_f32;
        for (a,b) in before.iter().zip(&mesh.solid.positions) {
            assert_eq!([a[0],a[2]], [b[0],b[2]], "a viewport cut cannot invent an eroded rim");
            assert!(b[1] > 6. && b[1] <= h);
            low = low.min(b[1]);
            high = high.max(b[1]);
        }
        assert!(high-low > 0.05*g.tile_height, "rock interiors must not remain flat slabs");
    }

    #[test]
    fn continuous_wall_erodes_across_tile_seams_and_keeps_floor_contacts() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 18];
        for i in 7..11 {
            sources[i] = super::super::tests::source_with_tile(0x70, 2, 1, 0x3c);
        }
        let f = super::super::tests::frame(6, 3, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 6,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let h = shapes[7].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        for col in 1..5 {
            let w = col as f32 * 8.;
            append_canyon(
                &mut mesh.solid,
                [
                    [w + 8., 0., 16.],
                    [w + 8., h, 16.],
                    [w, h, 16.],
                    [w, 0., 16.],
                ],
                Vec3::Z.to_array(),
                &g,
                [0, 0],
            );
        }
        let before = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut internal_seam = 0;
        let mut duplicates = HashMap::new();
        for (a, b) in before.iter().zip(&mesh.solid.positions) {
            if a[1] <= 6. {
                assert_eq!(a, b, "bank and floor contacts stay anchored");
            }
            if (a[0] - 24.).abs() < 0.001 && (a[1] - 16.).abs() < 0.001 {
                assert!(b[2] < 15., "an internal tile seam must share the erosion");
                internal_seam += 1;
            }
            if let Some(previous) = duplicates.insert(a.map(f32::to_bits), *b) {
                assert_eq!(previous, *b, "shared wall samples cannot open a seam");
            }
            assert!(Vec3::from_array(*b).is_finite());
        }
        assert!(internal_seam >= 2);
        let edges = vec![
            (Vec2::new(0., 0.), Vec2::new(8., 0.), Vec2::Y, 32., 0., 0.9),
            (Vec2::new(8., 0.), Vec2::new(16., 0.), Vec2::Y, 32., 6., 0.9),
            (
                Vec2::new(16., 0.),
                Vec2::new(24., 0.),
                Vec2::Y,
                16.,
                6.,
                0.9,
            ),
        ];
        let uniform = vec![
            (Vec2::new(0., 0.), Vec2::new(8., 0.), Vec2::Y, 32., 0., 0.9),
            (Vec2::new(8., 0.), Vec2::new(16., 0.), Vec2::Y, 32., 0., 0.9),
        ];
        assert_eq!(join_wall_edges(uniform.clone(), &[]).len(), 1);
        assert_eq!(
            join_wall_edges(uniform, &[Vec2::new(8., 0.)]).len(),
            2,
            "authored jump drops remain anchored run ends"
        );
        assert_eq!(
            join_wall_edges(edges, &[]).len(),
            3,
            "different contact heights cannot merge"
        );
    }

    #[test]
    fn canyon_shoulders_are_broad_closed_and_anchored_at_the_floor() {
        check_canyon_shoulders(false);
        check_canyon_shoulders(true);
    }

    fn check_canyon_shoulders(kanto: bool) {
        let source = |block,column,row,tile| {
            let mut source=super::super::tests::source_with_tile(block,column,row,tile);
            if kanto {source.tileset_id="kanto".into();}
            source
        };
        let flat = source(if kanto {0x31}else{0}, 0, 0, if kanto {0x39}else{5});
        let mut sources = vec![flat; 12];
        for i in [5, 6] {
            sources[i] = source(if kanto {0x2c}else{0x70}, 2, 1, if kanto {0x11}else{0x3c});
        }
        let frame = super::super::tests::frame(4, 3, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 4,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let h = shapes[5].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        for w in [8., 16.] {
            append_canyon(
                &mut mesh.solid,
                [[w, h, 8.], [w, h, 16.], [w + 8., h, 16.], [w + 8., h, 8.]],
                Vec3::Y.to_array(),
                &g,
                [0, 0],
            );
        }
        append_canyon(
            &mut mesh.solid,
            [[8., 0., 8.], [8., h, 8.], [8., h, 16.], [8., 0., 16.]],
            Vec3::NEG_X.to_array(),
            &g,
            [0, 0],
        );
        let unshaped = mesh.clone();
        let before = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut moved_inner_cap = false;
        let mut shared = HashMap::new();
        for (p, q) in before.iter().zip(&mesh.solid.positions) {
            if p[1] == 0. {
                assert_eq!(p, q, "floor contacts must stay fixed");
            }
            if p[1] == h {
                assert!(q[1] <= h, "fractures must not raise the authored crest");
            }
            if p[1] == h && p[0] > 11. && p[0] < 13. && p[2] == 12. {
                moved_inner_cap |= q[1] < h - 0.05;
            }
            let key = p.map(f32::to_bits);
            if let Some(previous) = shared.insert(key, *q) {
                assert_eq!(previous, *q, "cap and wall must receive the same shoulder");
            }
            assert!(Vec3::from_array(*q).is_finite());
        }
        assert!(
            moved_inner_cap,
            "rounding must extend beyond the old narrow bevel"
        );
        let mut cap_row: Vec<_> = before
            .iter()
            .zip(&mesh.solid.positions)
            .filter(|(p, _)| p[1] == h && p[2] == 12.)
            .map(|(p, q)| (p[0], q[0]))
            .collect();
        cap_row.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert!(
            cap_row.windows(2).all(|p| p[0].1 <= p[1].1 + 0.0001),
            "broad shoulders must not fold the cap back over itself"
        );
        for tri in mesh.solid.indices.chunks_exact(3) {
            if tri.iter().all(|&i| before[i as usize][1] == h) {
                let [a, b, c] = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                assert!((b - a).cross(c - a).y >= -0.0001,
                    "fractured shoulders cannot invert cap triangles");
            }
        }
        for origin in [[-31, -7], [17, 43], [64, -32]] {
            let mut variant = unshaped.clone();
            soften_rock_rims(&mut variant, &cells, &shapes, &g, origin);
            for (a, b) in before.iter().zip(&variant.solid.positions) {
                if a[1] <= 6. {
                    assert_eq!(a, b, "source phase cannot move floor contacts");
                }
                assert!(Vec3::from_array(*b).is_finite());
            }
            for tri in variant.solid.indices.chunks_exact(3) {
                if tri.iter().all(|&i| before[i as usize][1] == h) {
                    let [a, b, c] = [tri[0], tri[1], tri[2]]
                        .map(|i| Vec3::from_array(variant.solid.positions[i as usize]));
                    assert!((b - a).cross(c - a).y >= -0.0001,
                        "source phase {origin:?} cannot invert a fractured cap: {a:?} {b:?} {c:?}");
                }
            }
        }
    }
    #[test]
    fn broad_canyon_corner_breaks_retain_cap_area_across_source_phases() {
        let mut sources = vec![super::super::tests::source_with_tile(0, 0, 0, 5); 36];
        for z in 1..5 {
            for x in 1..5 {
                sources[z * 6 + x] = super::super::tests::source_with_tile(0x70, 2, 1, 0x3c);
            }
        }
        let frame = super::super::tests::frame(6, 6, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 6, height: 6, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0.,
        };
        let h = shapes[7].surface_height(8.);
        let mut corner_positions = Vec::new();
        for origin in [[0, 0], [-31, -7], [17, 43], [64, -32]] {
            let mut mesh = TerrainMeshData::default();
            for z in 1..5 {
                for x in 1..5 {
                    let (w, e, n, s) = g.bounds(x, z);
                    append_canyon(&mut mesh.solid,
                        [[w,h,n],[w,h,s],[e,h,s],[e,h,n]],
                        Vec3::Y.to_array(), &g, origin);
                }
            }
            let before = mesh.solid.positions.clone();
            soften_rock_rims(&mut mesh, &cells, &shapes, &g, origin);
            let corner = before.iter().position(|p| *p == [8.,h,8.]).unwrap();
            let q = mesh.solid.positions[corner];
            assert!(q[0] > 11. && q[2] > 11., "large corners need broad diagonal breaks");
            corner_positions.push(q);
            for tri in mesh.solid.indices.chunks_exact(3) {
                let [a,b,c] = [tri[0],tri[1],tri[2]]
                    .map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                assert!((b-a).cross(c-a).y > 0.,
                    "source phase {origin:?} must retain positive cap area");
            }
        }
        assert!(corner_positions.windows(2).any(|q|
            (q[0][0] - q[1][0]).abs() > 0.05), "corner outlines should vary across the map");
    }
    #[test]
    fn painted_chamfer_gets_a_closed_corner_before_rim_erosion() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 9];
        sources[4] = super::super::tests::source_with_tile(0x0a, 0, 0, 0x2b);
        sources[3] = super::super::tests::source_with_tile(0x57, 1, 2, 5);
        let frame = super::super::tests::frame(3, 3, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        let g = GridGeometry {
            width: 3,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let h = shapes[4].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        append(
            &mut mesh.solid,
            [[12., h, 8.], [8., h, 12.], [16., h, 16.], [12., h, 8.]],
            Vec3::Y.to_array(),
            &g,
            [20, 30],
        );
        let corner_uv = [-10000. - 168., -20000. - 248.];
        assert!(!mesh.solid.uvs.contains(&corner_uv));
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [20, 30]);
        assert!(mesh.solid.uvs.contains(&corner_uv));
        // Referenced cap borders must use the same sixth-cell samples as
        // their wall. The old fan retained extra half-edge samples which
        // followed different chords after the nonlinear shoulder warp.
        for tri in mesh.solid.indices.chunks_exact(3) {
            let ids = [tri[0], tri[1], tri[2]].map(|i| i as usize);
            if !ids.iter().all(|&i| mesh.solid.normals[i][1] > 0.3) {
                continue;
            }
            for i in ids {
                let uv = mesh.solid.uvs[i];
                let x = (-10000. - uv[0] - 168.) / 8.;
                let z = (-20000. - uv[1] - 248.) / 8.;
                if x.abs() < 0.0001 || (x - 1.).abs() < 0.0001 {
                    assert!((z * 6. - (z * 6.).round()).abs() < 0.002);
                }
                if z.abs() < 0.0001 || (z - 1.).abs() < 0.0001 {
                    assert!((x * 6. - (x * 6.).round()).abs() < 0.002);
                }
            }
        }
        // The restored west wall reaches the neighboring ramp's actual
        // boundary height, rather than its old constant six-pixel plane.
        assert!(mesh
            .solid
            .positions
            .iter()
            .any(|p| (p[0] - 8.).abs() < 0.0001
                && (p[1] - 3.).abs() < 0.0001
                && (p[2] - 8.).abs() < 0.0001));
        assert!(mesh
            .solid
            .positions
            .iter()
            .all(|p| Vec3::from_array(*p).is_finite()));
    }
    #[test]
    fn adjoining_wall_faces_share_height_rings_above_different_ledge_floors() {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut low = SurfaceMeshData::default();
        let mut raised = SurfaceMeshData::default();
        append(
            &mut low,
            [[8., 0., 8.], [8., 16., 8.], [8., 16., 16.], [8., 0., 16.]],
            Vec3::X.to_array(),
            &g,
            [0, 0],
        );
        append(
            &mut raised,
            [[8., 6., 16.], [8., 16., 16.], [8., 16., 24.], [8., 6., 24.]],
            Vec3::X.to_array(),
            &g,
            [0, 0],
        );
        let seam = |mesh: &SurfaceMeshData| {
            let mut points: Vec<_> = mesh
                .positions
                .iter()
                .copied()
                .filter(|p| (p[2] - 16.).abs() < 0.0001 && p[1] > 6.001)
                .collect();
            points.sort_by(|a, b| a[1].total_cmp(&b[1]));
            points.dedup_by(|a, b| (a[1] - b[1]).abs() < 0.0001);
            points
        };
        assert_eq!(
            seam(&low),
            seam(&raised),
            "shared curved wall edges need identical chords despite different floors"
        );
        assert!(seam(&low).len() >= 7);
        let mut horizontal_first = SurfaceMeshData::default();
        append(
            &mut horizontal_first,
            [[8., 0., 16.], [8., 0., 8.], [8., 16., 8.], [8., 16., 16.]],
            Vec3::X.to_array(),
            &g,
            [0, 0],
        );
        assert_eq!(seam(&low), seam(&horizontal_first));
        assert!(!horizontal_first.indices.is_empty());

        let mut reversed = SurfaceMeshData::default();
        append(
            &mut reversed,
            [[8., 16., 8.], [8., 0., 8.], [8., 0., 16.], [8., 16., 16.]],
            Vec3::NEG_X.to_array(),
            &g,
            [0, 0],
        );
        for tri in reversed.indices.chunks_exact(3) {
            let q =
                [tri[0], tri[1], tri[2]].map(|i| Vec3::from_array(reversed.positions[i as usize]));
            assert!(
                (q[1] - q[0]).cross(q[2] - q[0]).dot(Vec3::NEG_X) >= -0.0001,
                "height rings must retain top-to-bottom source winding"
            );
        }
    }
    #[test]
    fn stone_shoulder_stays_attached_to_a_higher_neighbor_floor() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 12];
        sources[5] = super::super::tests::source_with_tile(0x0a, 1, 1, 0x3c);
        sources[6] = super::super::tests::source_with_tile(0x0a, 2, 1, 0x3c);
        let frame = super::super::tests::frame(4, 3, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry {
            width: 4,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        shapes[4] = CellShape::RaisedTop {
            height: 10.,
            solid: SolidKind::Bank,
        };
        let h = shapes[5].surface_height(8.);
        let mut mesh = TerrainMeshData::default();
        append(
            &mut mesh.solid,
            [[8., 0., 8.], [8., h, 8.], [8., h, 16.], [8., 0., 16.]],
            Vec3::X.to_array(),
            &g,
            [0, 0],
        );
        let original = mesh.solid.positions.clone();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut checked = 0;
        for (before, after) in original.iter().zip(&mesh.solid.positions) {
            if (before[0] - 8.).abs() < 1.
                && before[1] <= 10.
                && before[2] > 10.9
                && before[2] < 13.1
            {
                assert_eq!(
                    before, after,
                    "rock shaping cannot pull away below an adjoining higher floor"
                );
                checked += 1;
            }
        }
        assert!(checked >= 6);
    }
    #[test]
    fn exposed_rims_share_cap_wall_vertices_without_eroding_internal_joins_or_bases() {
        let flat = super::super::tests::source_with_tile(0, 0, 0, 5);
        let mut sources = vec![flat; 12];
        sources[5] = super::super::tests::source_with_tile(0x0a, 1, 1, 0x3c);
        sources[6] = super::super::tests::source_with_tile(0x0a, 2, 1, 0x3c);
        let frame = super::super::tests::frame(4, 3, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry {
            width: 4,
            height: 3,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let h = shape_for_source(&cells[5].source).surface_height(8.0);
        assert!(h > 0.0);
        let mut mesh = TerrainMeshData::default();
        append(
            &mut mesh.solid,
            [[8., h, 8.], [8., h, 16.], [24., h, 16.], [24., h, 8.]],
            Vec3::Y.to_array(),
            &g,
            [0, 0],
        );
        append(
            &mut mesh.solid,
            [[8., 0., 8.], [8., h, 8.], [8., h, 16.], [8., 0., 16.]],
            Vec3::NEG_X.to_array(),
            &g,
            [0, 0],
        );
        let stone_end = mesh.solid.positions.len();
        append_canyon(
            &mut mesh.solid,
            [[8., 0., 8.], [8., h, 8.], [8., h, 16.], [8., 0., 16.]],
            Vec3::NEG_X.to_array(),
            &g,
            [0, 0],
        );
        let original = mesh.solid.positions.clone();
        let shapes: Vec<_> = cells
            .iter()
            .map(|cell| shape_for_source(&cell.source))
            .collect();
        soften_rock_rims(&mut mesh, &cells, &shapes, &g, [0, 0]);
        let mut cap_row: Vec<_> = original
            .iter()
            .zip(&mesh.solid.positions)
            .filter(|(p, _)| p[1] == h && p[2] == 12.)
            .map(|(before, after)| (before[0], after[0]))
            .collect();
        cap_row.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert!(cap_row.len() >= 6);
        assert!(
            cap_row.windows(2).all(|p| p[1].1 >= p[0].1 - 0.0001),
            "a broad rock shoulder cannot fold its cap back over itself"
        );
        let wall_vertices = mesh.solid.positions.len() - stone_end;
        assert_eq!(
            &mesh.solid.positions[stone_end - wall_vertices..stone_end],
            &mesh.solid.positions[stone_end..],
            "stone and canyon material joins must receive identical shape warps"
        );
        let rim: Vec<_> = original
            .iter()
            .zip(&mesh.solid.positions)
            .filter(|(p, _)| **p == [8., h, 8.])
            .map(|(_, p)| *p)
            .collect();
        assert!(rim.len() >= 2);
        assert!(rim.iter().all(|p| *p == rim[0]));
        let rim_normals: Vec<_> = original
            .iter()
            .zip(&mesh.solid.normals)
            .filter(|(p, _)| **p == [8., h, 8.])
            .map(|(_, n)| *n)
            .collect();
        assert!(
            rim_normals.iter().all(|n| *n == rim_normals[0]),
            "cap and wall lighting must meet smoothly on a rounded shoulder"
        );
        let joint: Vec<_> = original
            .iter()
            .zip(&mesh.solid.positions)
            .filter(|(p, _)| **p == [16., h, 12.])
            .map(|(_, p)| *p)
            .collect();
        assert!(joint.len() >= 2);
        assert!(
            joint.iter().all(|p| *p == joint[0]),
            "broad shoulders must share the same cap seam"
        );
        assert!(rim[0][0] > 8. && rim[0][2] > 8. && rim[0][1] < h);
        assert!(
            rim[0][0] - 8. > g.tile_height * 0.48 * 0.293 * 1.2,
            "broad shoulders must round farther inward than the narrow fillet"
        );
        assert!(
            original
                .iter()
                .zip(&mesh.solid.positions)
                .any(|(before, after)| {
                    (before[0] - 8.).abs() < 1.
                        && before[1] > 8.
                        && before[1] < h - 4.
                        && before[2] > 10.9
                        && before[2] < 13.1
                        && after[0] > before[0] + 0.1
                }),
            "wall weathering must reshape the middle of the face, not only its rim"
        );
        for (before, after) in original.iter().zip(&mesh.solid.positions) {
            if (before[0] - 8.).abs() < 0.0001
                && ((before[2] - 8.).abs() < 0.0001 || (before[2] - 16.).abs() < 0.0001)
                && before[1] > 0.
                && before[1] <= crate::profile::JUMP_LEDGE_HEIGHT
            {
                assert!(
                    Vec3::from_array(*before).distance(Vec3::from_array(*after)) < 0.0001,
                    "shoulder rounding must stop at the ledge closure height"
                );
            }
            if before[1] == 0. {
                assert_eq!(before, after);
            }
            if before[0] == 16. && before[2] > 10.9 && before[2] < 13.1 {
                assert_eq!(before[0], after[0]);
                assert_eq!(before[2], after[2]);
                assert!(after[1] >= crate::profile::JUMP_LEDGE_HEIGHT);
            }
        }
        assert!(mesh
            .solid
            .normals
            .iter()
            .all(|n| Vec3::from_array(*n).is_finite()));
    }
}
