//! Facility rooms keep one continuous green wall behind rear furniture.
use super::*;
use bevy::prelude::UVec2;

fn drawing(block: u16) -> Option<[[u16; 4]; 4]> {
    Some(match block {
        1 => [[0x22; 4], [0x22; 4], [1, 0x26, 1, 0x26], [0x26, 1, 0x26, 1]],
        2 => [
            [0x40, 0x42, 0x40, 0x42],
            [0xa, 0xb, 0xa, 0xb],
            [0xa, 0xb, 0xa, 0xb],
            [0x1a, 0x1b, 0x1a, 0x1b],
        ],
        6 => [
            [0x40, 0x42, 0x40, 0x42],
            [0x28, 0x29, 0x28, 0x29],
            [0x28, 0x29, 0x28, 0x29],
            [0x2a, 0x2b, 0x28, 0x29],
        ],
        0x19 => [
            [0x22; 4],
            [0x22; 4],
            [0x23, 0x24, 0x24, 0x25],
            [0x57, 0x58, 0x58, 0x59],
        ],
        0x1a => [
            [0x22; 4],
            [0x40, 0x42, 0x22, 0x22],
            [0x50, 0x52, 0x1c, 0x1c],
            [0x50, 0x52, 0x1c, 0x1c],
        ],
        0x1d => [
            [0x22, 0x22, 0x32, 0x33],
            [0x22, 0x22, 0x43, 0x44],
            [1, 0x26, 1, 0x26],
            [0x26, 1, 0x26, 1],
        ],
        0x1e => [
            [0x22; 4],
            [0x22, 0x22, 0x40, 0x42],
            [1, 0x26, 0x50, 0x52],
            [0x26, 1, 0x50, 0x52],
        ],
        0x35 => [
            [0x22; 4],
            [0x22; 4],
            [0x0e, 0x0f, 1, 0x26],
            [0x1e, 0x1f, 0x26, 1],
        ],
        0x21 => [
            [0x22; 4], [0x40, 0x42, 0x22, 0x22],
            [0x50, 0x52, 1, 0x26], [0x50, 0x52, 0x26, 1],
        ],
        4 => [
            [1, 0x26, 0x10, 0x11], [0x26, 1, 0x20, 0x21],
            [1, 0x26, 1, 0x26], [0x26, 1, 0x26, 1],
        ],
        0x3c => [
            [0x40, 0x42, 0x40, 0x41], [0x50, 0x52, 0x4c, 0x4d],
            [0x50, 0x52, 1, 0x26], [0x50, 0x52, 0x26, 1],
        ],
        0x3d => [
            [0x41, 0x42, 0x40, 0x42], [0x4d, 0x4e, 0x50, 0x52],
            [1, 0x26, 0x50, 0x52], [0x26, 1, 0x50, 0x52],
        ],
        0x3b => [
            [0x22; 4],
            [0x22, 0x22, 0x5c, 0x5d],
            [1, 0x26, 0x5e, 0x5f],
            [0x26, 1, 0x16, 0x17],
        ],
        _ => return None,
    })
}

fn source(
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> Option<usize> {
    let (blocks, size): (&[u16], UVec2) = match frame.map_id.as_ref() {
        "MrPokemonsHouse" => (&[6, 0x19, 1, 2], UVec2::new(8, 8)),
        "RuinsOfAlphResearchCenter" => (&[2, 2, 1, 0x3b], UVec2::new(8, 8)),
        "PowerPlant" => (&[6, 1, 1, 0x19, 0x19, 0x1a, 2, 2, 2, 2], UVec2::new(20, 18)),
        "SilphCo1F" => (&[0x35, 0x35, 0x1e, 1, 1, 1, 0x1d, 1], UVec2::new(16, 8)),
        "TrainerHouseB1F" => (&[2, 1, 2, 0x21, 1], UVec2::new(10, 16)),
        _ => return None,
    };
    let trainer_basement = frame.map_id.as_ref() == "TrainerHouseB1F";
    let exits = if trainer_basement { vec![IVec2::new(9, 4)] } else {
        vec![IVec2::new(2, size.y as i32 - 1), IVec2::new(3, size.y as i32 - 1)]
    };
    if frame.source_map_size_core_tiles != Some(size)
        || g.width.checked_mul(g.height) != Some(cells.len())
        || ![g.origin_x, g.origin_z, g.tile_width, g.tile_height]
            .iter()
            .all(|v| v.is_finite())
        || g.tile_width <= 0.
        || g.tile_height <= 0.
        || frame.boundary_warps.len() != exits.len()
        || !exits.iter().all(|p| frame.boundary_warps.contains(p))
    {
        return None;
    }
    let index = |x: i32, y: i32| {
        let (x, y) = (x - frame.grid_origin.x, y - frame.grid_origin.y);
        (x >= 0 && y >= 0 && (x as usize) < g.width && (y as usize) < g.height)
            .then(|| y as usize * g.width + x as usize)
    };
    // Guard the complete rear drawings, including cabinets. A missing wall
    // sprite behind a complete cupboard is still a continuous room boundary.
    let mut course = blocks.iter().enumerate()
        .map(|(i, &b)| (i as i32 * 4, 0, b)).collect::<Vec<_>>();
    if trainer_basement {
        // Authenticate the actual east stair and both arena joins as well as
        // the rear drawing before replacing its generic perimeter.
        course.extend([(16, 8, 4), (0, 12, 0x3c), (16, 12, 0x3d)]);
    }
    let mut north = Vec::new();
    for (bx, by, block) in course {
        for y in 0..4 {
            for x in 0..4 {
                let i = index(bx + x, by + y)?;
                let s = &cells[i].source;
                if s.tileset_id.as_ref() != "facility"
                    || s.metatile_id != block
                    || s.subtile_column != x as u8
                    || s.subtile_row != y as u8
                    || s.tile_index != drawing(block)?[y as usize][x as usize]
                { return None; }
                north.push(i);
            }
        }
    }
    let custom = Document {
        objects: profiles.map_or_else(Vec::new, |d| {
            d.objects
                .iter()
                .filter(|o| {
                    !built_in_profiles()
                        .objects
                        .iter()
                        .any(|canonical| canonical == *o)
                })
                .cloned()
                .collect()
        }),
        atmosphere: None,
    };
    if live::resolve(cells, g.width, g.height, &frame.map_id, Some(&custom))
        .iter()
        .any(|p| p.indices(g.width).any(|i| north.contains(&i)))
    {
        return None;
    }
    index(if trainer_basement { 4 } else { 8 }, 0)
}

// Fold only the old horizontal wall drawing out of the ground pass. Leave
// source floors, underlays, raised tops and all existing vertex/range identities
// intact. In particular, a floor that merely borrows a wall swatch elsewhere
// is not the original wall cell and must survive.
fn remove_flat_wall_drawing(
    mesh: &mut SurfaceMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    let indices = std::mem::take(&mut mesh.indices);
    mesh.indices = indices
        .chunks_exact(3)
        .filter(|t| {
            let points = [0, 1, 2].map(|j| mesh.positions[t[j] as usize]);
            if t.iter().any(|&i| mesh.normals[i as usize][1] < 0.99)
                || points.iter().any(|p| p[1].abs() > 0.001)
            {
                return true;
            }
            let uv = t.iter().fold([0.; 2], |a, &i| {
                [
                    a[0] + mesh.uvs[i as usize][0] / 3.,
                    a[1] + mesh.uvs[i as usize][1] / 3.,
                ]
            });
            if !uv.iter().all(|v| v.is_finite() && *v >= 0. && *v < 1.) {
                return true;
            }
            let x = (uv[0] * g.width as f32).floor() as usize;
            let y = (uv[1] * g.height as f32).floor() as usize;
            let world = frame.grid_origin + IVec2::new(x as i32, y as i32);
            let source = &cells[y * g.width + x].source;
            if !(0..frame.source_map_size_core_tiles.unwrap().x as i32 * 2).contains(&world.x)
                || !(0..2).contains(&world.y)
                || source.tileset_id.as_ref() != "facility"
                || source.tile_index != 0x22
            {
                return true;
            }
            let (w, e, n, s) = g.bounds(x, y);
            !points.iter().all(|p| {
                p[0] >= w - 0.001 && p[0] <= e + 0.001 && p[2] >= n - 0.001 && p[2] <= s + 0.001
            })
        })
        .flatten()
        .copied()
        .collect();
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> bool {
    let Some(donor) = source(frame, cells, g, profiles) else {
        return false;
    };
    remove_flat_wall_drawing(&mut mesh.textured, frame, cells, g);
    let uv = g.uv(donor % g.width, donor / g.width);
    let sample = |y: f32| {
        [
            uv.0 + (uv.1 - uv.0) * 3.5 / 8.,
            uv.2 + (uv.3 - uv.2) * (y + 0.5) / 8.,
        ]
    };
    let mut template = SurfaceMeshData::default();
    let width = 2. * g.tile_width;
    let depth = 3. * g.tile_height / 8.;
    let height = 4. * g.tile_height;
    // Recess the native light/ink courses into the body. Every front face,
    // including trim, stays behind the physical room boundary.
    let mut bottom = 0.;
    for (top, pixel, recess) in [
        (7.75, 1., 0.),
        (8., 4., 0.2),
        (8.4, 3., 0.05),
        (15.75, 1., 0.),
        (16., 4., 0.2),
        (16.4, 3., 0.05),
        (23.75, 1., 0.),
        (24., 4., 0.2),
        (24.4, 3., 0.05),
        (32., 1., 0.),
    ] {
        let first = template.positions.len();
        architecture_box(
            &mut template,
            [
                0.,
                width,
                bottom * g.tile_height / 8.,
                top * g.tile_height / 8.,
                0.,
                depth - recess * g.tile_height / 8.,
            ],
            [1.; 3],
        );
        for i in first..template.positions.len() {
            template.uvs[i] = sample(
                if template.normals[i][1] > 0.5 && template.positions[i][1] == height {
                    3.
                } else {
                    pixel
                },
            );
        }
        bottom = top;
    }
    let b = [0., width, 0., depth];
    let empty = SurfaceMeshData::default();
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let size = frame.source_map_size_core_tiles.unwrap();
    let east = west + size.x as f32 * 2. * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height + 2. * g.tile_height;
    let south = g.origin_z - frame.grid_origin.y as f32 * g.tile_height + size.y as f32 * 2. * g.tile_height;
    // Overlap only outside the native floor. Both side walls meet the rear
    // course, and the actual two exit tiles remain open in the front course.
    let mut sections = Vec::new();
    if frame.map_id.as_ref() == "SilphCo1F" {
        // The authenticated $1d stair occupies source columns26–27. Its
        // landing stays open instead of acquiring a wall across the stairs.
        sections.push((Edge::North, north, west - depth, west + 26. * g.tile_width));
        sections.push((Edge::North, north, west + 28. * g.tile_width, east + depth));
    } else {
        sections.push((Edge::North, north, west - depth, east + depth));
    }
    if frame.map_id.as_ref() == "TrainerHouseB1F" {
        let z = |row: f32| g.origin_z - frame.grid_origin.y as f32 * g.tile_height + row * g.tile_height;
        // Existing authored gold arena walls own the lower room enclosure.
        // Green returns end at their joins; only the actual east landing opens.
        sections.extend([
            (Edge::West, west, north - depth, z(12.)),
            (Edge::East, east, north - depth, z(8.)),
            (Edge::East, east, z(10.), z(12.)),
        ]);
    } else {
    sections.extend([
        (Edge::West, west, north - depth, south),
        (Edge::East, east, north - depth, south),
        (Edge::South, south, west - depth, west + 4. * g.tile_width),
        (Edge::South, south, west + 8. * g.tile_width, east + depth),
    ]);
    }
    for (edge, at, from, to) in sections {
        let mut start = from;
        while start < to - 0.001 {
            let end = (start + width).min(to);
            append_section(mesh, &empty, &template, b, edge, at, start, end);
            start = end;
        }
    }
    true
}

#[cfg(test)]
#[path = "native_facility_room_shell_tests.rs"]
mod tests;
