//! The park's source-complete gatehouses and their native north/west entrances.
use super::kanto_craft::{block, ceramic_tile, face, house_roof_paint, mix};
use super::*;

pub(in crate::mesh) const LABEL: &str = "world-exterior/park_gate";
const BLOCKS: [[u16; 3]; 3] = [[0x28, 0x29, 0x2a], [0x2c, 0x2d, 0x2e], [0x30, 0x31, 0x32]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entrance {
    North,
    West,
}

fn drawing_tile(x: usize, y: usize) -> u16 {
    let course = match y {
        0 => 0x10,
        1..=4 => 0x20,
        5 => 0x30,
        6..=10 => 0x40,
        _ => 0x50,
    };
    course
        + if x == 0 {
            0
        } else if x == 11 {
            2
        } else {
            1
        }
}

/// Native north and east thresholds have different orientations. Inspect the
/// whole four-by-four drawing before finishing its eight cloth cells.
pub(in crate::mesh) fn threshold(
    cells: &[&VisualTile],
    g: &GridGeometry,
    x: usize,
    y: usize,
) -> Option<[usize; 4]> {
    if x + 4 > g.width || y + 4 > g.height || cells.len() != g.width * g.height {
        return None;
    }
    let first = &cells[y * g.width + x].source;
    let block = first.metatile_id;
    if first.tileset_id.as_ref() != "park" || !matches!(block, 0x0a | 0x0b) {
        return None;
    }
    let complete = (0..4).all(|dy| {
        (0..4).all(|dx| {
            let s = &cells[(y + dy) * g.width + x + dx].source;
            let cloth = if block == 0x0a { dy >= 2 } else { dx >= 2 };
            let tile = if cloth {
                0x4f
            } else if dy % 2 == 0 {
                if dx % 2 == 0 { 0 } else { 0x16 }
            } else if dx % 2 == 0 {
                6
            } else {
                0
            };
            s.tileset_id.as_ref() == "park"
                && s.metatile_id == block
                && s.tile_index == tile
                && s.subtile_column as usize == dx
                && s.subtile_row as usize == dy
        })
    });
    complete.then_some(if block == 0x0a {
        [0, 2, 4, 2]
    } else {
        [2, 0, 2, 4]
    })
}

fn entrance(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> Option<Entrance> {
    let north = p.row >= 4 && threshold(cells, g, p.column + 4, p.row - 4) == Some([0, 2, 4, 2]);
    let west = p.column >= 4 && threshold(cells, g, p.column - 4, p.row + 8) == Some([2, 0, 2, 4]);
    match (north, west) {
        (true, false) => Some(Entrance::North),
        (false, true) => Some(Entrance::West),
        _ => None,
    }
}

pub(in crate::mesh) fn matches(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
) -> bool {
    p.width == 12
        && p.height == 12
        && p.roof_rows == 6
        && p.ground_tile_index == 0
        && p.column + 12 <= g.width
        && p.row + 12 <= g.height
        && cells.len() == g.width * g.height
        && entrance(cells, g, p).is_some()
        && (0..12).all(|y| {
            (0..12).all(|x| {
                let s = &cells[(p.row + y) * g.width + p.column + x].source;
                s.tileset_id.as_ref() == "park"
                    && s.metatile_id == BLOCKS[y / 4][x / 4]
                    && s.subtile_column as usize == x % 4
                    && s.subtile_row as usize == y % 4
                    && s.tile_index == drawing_tile(x, y)
            })
        })
}

pub(super) fn placements(cells: &[&VisualTile], g: &GridGeometry) -> Vec<BuildingPlacement> {
    cells
        .iter()
        .filter_map(|tile| {
            let s = &tile.source;
            if s.tileset_id.as_ref() != "park"
                || s.metatile_id != 0x28
                || s.subtile_column != 0
                || s.subtile_row != 0
            {
                return None;
            }
            let p = BuildingPlacement {
                column: tile.column as usize,
                row: tile.row as usize,
                width: 12,
                height: 12,
                roof_rows: 6,
                ground_tile_index: 0,
            };
            matches(cells, g, p).then_some(p)
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Paint {
    roof: [f32; 3],
    wall: [f32; 3],
    mortar: [f32; 3],
    trim: [f32; 3],
    shadow: [f32; 3],
}

fn pigment(tile: &VisualTile, images: &TerrainImageSamples) -> Option<Vec<[f32; 3]>> {
    let rgba = tile_rgba(images, tile).ok()?;
    if rgba.len() != 256 || rgba.chunks_exact(4).any(|p| p[3] != 255) {
        return None;
    }
    let mut colors = rgba
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2]])
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if !(2..=4).contains(&colors.len()) {
        return None;
    }
    colors.sort_by_key(|c| u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722);
    Some(
        colors
            .into_iter()
            .map(|c| {
                c.map(|v| {
                    let s = v as f32 / 255.;
                    if s <= 0.04045 {
                        s / 12.92
                    } else {
                        ((s + 0.055) / 1.055).powf(2.4)
                    }
                })
            })
            .collect(),
    )
}

fn paint(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
    images: &TerrainImageSamples,
) -> Option<Paint> {
    let roof = pigment(cells[(p.row + 2) * g.width + p.column + 5], images)?;
    let wall = pigment(cells[(p.row + 7) * g.width + p.column + 5], images)?;
    let roof = house_roof_paint(mix(roof[0], *roof.last()?, 0.35));
    let mortar = *wall.last()?;
    let wall = mix(wall[usize::from(wall.len() > 2)], mortar, 0.62);
    let luminance = wall[0] * 0.2126 + wall[1] * 0.7152 + wall[2] * 0.0722;
    let wall = mix(wall, [luminance * 1.02, luminance, luminance * 0.96], 0.55);
    Some(Paint {
        roof,
        wall,
        mortar,
        trim: mix(wall, mortar, 0.55),
        shadow: mix(roof, wall, 0.12).map(|v| v * 0.18),
    })
}

fn painted_part(out: &mut SurfaceMeshData, b: [f32; 6], bevel: f32, color: [f32; 3], unit: f32) {
    block(out, b, bevel, color, -1., unit);
}

fn roof_shell(out: &mut SurfaceMeshData, b: [f32; 4], eave: f32, peak: f32, unit: f32, p: Paint) {
    let [w, e, n, s] = b;
    let middle = (n + s) * 0.5;
    let profile = [
        (n, eave - unit * 0.16),
        (s, eave - unit * 0.16),
        (s, eave),
        (middle, peak),
        (n, eave),
    ];
    let rings = [w, e].map(|x| profile.map(|(z, y)| Vec3::new(x, y, z)));
    let center = Vec3::new((w + e) * 0.5, (eave * 2. + peak) / 3. - unit * 0.04, middle);
    face(out, rings[0].to_vec(), center, p.wall, -1., unit);
    face(out, rings[1].to_vec(), center, p.wall, -1., unit);
    for i in 0..5 {
        let j = (i + 1) % 5;
        face(
            out,
            vec![rings[0][i], rings[0][j], rings[1][j], rings[1][i]],
            center,
            if i == 0 { p.trim } else { p.roof },
            -1.,
            unit,
        );
    }
}

fn geometry(out: &mut SurfaceMeshData, b: [f32; 4], unit: f32, p: Paint, entrance: Entrance) {
    let first = out.positions.len();
    let [bw, be, bn, bs] = b;
    let dx = (be - bw) / 12.;
    let (w, e, n, s) = (
        bw + dx * 0.35,
        be - dx * 0.35,
        bn + unit * 0.35,
        bs - unit * 0.35,
    );
    let (door_w, door_e) = match entrance {
        Entrance::North => (bw + dx * 4., bw + dx * 8.),
        // After rotation, this opening meets the west mat's last four source
        // rows. Keep a narrow real corner jamb inside the original footprint.
        Entrance::West => (bw + dx * 0.55, bw + dx * 4.),
    };
    let (door_h, wall_h) = (unit * 2.9, unit * 3.8);
    let recess = n + unit * 1.2;
    // Build the recess on the local north wall, then orient the whole closed
    // structure to its source threshold. Gameplay warps on the original mat.
    for bounds in [
        [w, door_w, 0., wall_h, n, s],
        [door_e, e, 0., wall_h, n, s],
        [door_w, door_e, 0., wall_h, recess, s],
        [door_w, door_e, door_h, wall_h, n, recess],
    ] {
        painted_part(out, bounds, unit * 0.025, p.mortar, unit);
    }
    painted_part(
        out,
        [
            door_w,
            door_e,
            unit * 0.04,
            door_h,
            recess - unit * 0.018,
            recess,
        ],
        0.,
        p.shadow,
        unit,
    );
    // Staggered shallow, closed masonry has physical arrises on every side.
    // Clip bricks at the portal, rather than laying decorative faces over it.
    let rows = 9;
    for row in 0..rows {
        let lo = row as f32 * wall_h / rows as f32 + unit * 0.025;
        let hi = (row + 1) as f32 * wall_h / rows as f32 - unit * 0.025;
        for side in 0..4 {
            let (start, end) = if side < 2 { (w, e) } else { (n, s) };
            let length = (end - start) / 10.;
            for brick in -1..11 {
                let a = (start + (brick as f32 + (row % 2) as f32 * 0.5) * length).max(start);
                let z = (start + (brick as f32 + 1. + (row % 2) as f32 * 0.5) * length).min(end);
                if z - a <= unit * 0.06 {
                    continue;
                }
                let spans = if side == 0 && lo < door_h {
                    [[a, z.min(door_w)], [a.max(door_e), z]]
                } else {
                    [[a, z], [0., 0.]]
                };
                for [a, z] in spans {
                    if z - a <= unit * 0.06 {
                        continue;
                    }
                    let color = p.wall.map(|v| {
                        v * (0.96
                            + ((row * 7 + (brick + 1) as usize * 3 + side) % 7) as f32 * 0.012)
                    });
                    let gap = unit * 0.018;
                    let b = match side {
                        0 => [a + gap, z - gap, lo, hi, n - unit * 0.035, n + unit * 0.01],
                        1 => [a + gap, z - gap, lo, hi, s - unit * 0.01, s + unit * 0.035],
                        2 => [w - unit * 0.035, w + unit * 0.01, lo, hi, a + gap, z - gap],
                        _ => [e - unit * 0.01, e + unit * 0.035, lo, hi, a + gap, z - gap],
                    };
                    painted_part(out, b, unit * 0.015, color, unit);
                }
            }
        }
    }
    for bounds in [
        [
            door_w - dx * 0.18,
            door_w,
            0.,
            door_h + unit * 0.15,
            n - unit * 0.07,
            n + unit * 0.12,
        ],
        [
            door_e,
            door_e + dx * 0.18,
            0.,
            door_h + unit * 0.15,
            n - unit * 0.07,
            n + unit * 0.12,
        ],
        [
            door_w - dx * 0.18,
            door_e + dx * 0.18,
            door_h,
            door_h + unit * 0.22,
            n - unit * 0.07,
            n + unit * 0.12,
        ],
    ] {
        painted_part(out, bounds, unit * 0.035, p.trim, unit);
    }
    let roof = [
        bw + dx * 0.06,
        be - dx * 0.06,
        bn + unit * 0.04,
        bs - unit * 0.04,
    ];
    let (eave, peak) = (wall_h + unit * 0.12, wall_h + unit * 1.92);
    roof_shell(out, roof, eave, peak, unit, p);
    let middle = (roof[2] + roof[3]) * 0.5;
    let columns = 24;
    for side in 0..2 {
        let edge = roof[2 + side];
        for column in 0..columns {
            let x0 = roof[0] + (roof[1] - roof[0]) * column as f32 / columns as f32;
            let x1 = roof[0] + (roof[1] - roof[0]) * (column + 1) as f32 / columns as f32;
            for course in 0..3 {
                let section = |i| {
                    let t = i as f32 / 3.;
                    let y = eave + (peak - eave) * t + unit * 0.025;
                    let z = edge + (middle - edge) * t;
                    [Vec3::new(x0, y, z), Vec3::new(x1, y, z)]
                };
                ceramic_tile(
                    out,
                    [section(course), section(course + 1)],
                    unit,
                    unit * 0.035,
                    p.roof.map(|v| {
                        v * (0.96 + ((column * 7 + course * 3 + side) % 9) as f32 * 0.012)
                    }),
                );
            }
        }
    }
    painted_part(
        out,
        [
            roof[0],
            roof[1],
            peak,
            peak + unit * 0.13,
            middle - unit * 0.10,
            middle + unit * 0.10,
        ],
        unit * 0.025,
        p.roof,
        unit,
    );
    for i in first..out.positions.len() {
        if entrance == Entrance::West {
            let [x, y, z] = out.positions[i];
            let width = be - bw;
            let depth = bs - bn;
            out.positions[i] = [
                bw + (z - bn) * width / depth,
                y,
                bs - (x - bw) * depth / width,
            ];
            let [nx, ny, nz] = out.normals[i];
            out.normals[i] = Vec3::new(nz * depth / width, ny, -nx * width / depth)
                .normalize()
                .to_array();
        }
        // These pigments already include the current day/night palette.
        out.uvs[i] = crate::scenery_finish::SOURCE_PAINT_UV;
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    p: BuildingPlacement,
    claimed: &mut [bool],
    images: Option<&TerrainImageSamples>,
) -> bool {
    let rect = [p.column, p.row, p.width, p.height];
    if !matches(cells, g, p) || !clear(claimed, g, rect) {
        return false;
    }
    let Some(entrance) = entrance(cells, g, p) else {
        return false;
    };
    let Some(sample) = structure_ground(cells, shapes, "park", 0) else {
        return false;
    };
    let Some(paint) = images.and_then(|images| paint(cells, g, p, images)) else {
        return false;
    };
    floor(mesh, cells, shapes, g, rect, sample, claimed);
    let b = bounds(g, rect);
    let first = mesh.solid.positions.len();
    geometry(&mut mesh.solid, b, g.tile_height, paint, entrance);
    let range = first..mesh.solid.positions.len();
    mesh.solid.cutaway_ranges.push(range.clone());
    mesh.coverage_fade_ranges.push(range.clone());
    mesh.depth_coverage_solid_ranges.push(range.clone());
    mesh.rigid_buildings
        .push(crate::mesh::rigid_buildings::Building {
            vertices: range,
            doorway: match entrance {
                Entrance::North => Vec2::new((b[0] + b[1]) * 0.5, b[2] - g.tile_height),
                Entrance::West => Vec2::new(b[0] - g.tile_width, b[2] + g.tile_height * 10.),
            },
        });
    mark_authored_rect(mesh, g, rect, LABEL);
    true
}

#[cfg(test)]
#[path = "park_gate_tests.rs"]
mod tests;
