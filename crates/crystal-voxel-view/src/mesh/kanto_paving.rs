//! Walkable Kanto paving: complete source drawings, live paint, recessed joints.
//! The original collision and standing datum remain authoritative.
use super::*;

const LABEL: &str = "outdoor/kanto-paving";
const BRIDGE_LABEL: &str = "outdoor/kanto-brick-bridge";
fn drawing(block: u16) -> Option<[u16; 16]> {
    Some(match block {
        // The authored brick deck retains its original LADDER permission.
        0x05 => [0x4b; 16],
        0x22 => [
            0x21, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x21, 0x23,
            0x23, 0x10,
        ],
        0x23 => [
            0x20, 0x20, 0x20, 0x20, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x23, 0x20, 0x20,
            0x20, 0x20,
        ],
        0x26 => [
            0x10, 0x23, 0x23, 0x21, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23,
            0x23, 0x21,
        ],
        0x27 => [
            0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23,
            0x23, 0x10,
        ],
        0x44 => [
            0x10, 0x23, 0x23, 0x21, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x21, 0x20,
            0x20, 0x20,
        ],
        0x45 => [
            0x21, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x20, 0x20,
            0x20, 0x21,
        ],
        0x48 => [
            0x20, 0x20, 0x20, 0x21, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x21, 0x23,
            0x23, 0x10,
        ],
        0x49 => [
            0x21, 0x20, 0x20, 0x20, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23, 0x23, 0x23, 0x10, 0x23,
            0x23, 0x21,
        ],
        0x5d => [
            0x21, 0x23, 0x23, 0x21, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23, 0x23, 0x10, 0x10, 0x23,
            0x23, 0x10,
        ],
        _ => return None,
    })
}

// Authenticate the existing outline art by palette equality, so time-of-day
// colors can change without accepting a changed drawing or transparent tile.
fn ink(tile: u16, x: usize, y: usize) -> usize {
    match tile {
        0x4b => {
            [
                "00000000", "11111110", "11111110", "00000000", "11101111", "11101111", "00000000",
                "01111110",
            ][y]
                .as_bytes()[x] as usize
                - b'0' as usize
        }
        0x10 => [1, 1, 3, 0, 2, 3, 2, 1][x],
        0x20 => match y {
            0 | 1 | 7 => 1,
            2 => 3,
            3 => 0,
            4 => 2,
            5 => {
                if x < 2 || x > 4 {
                    3
                } else {
                    2
                }
            }
            _ => {
                if (2..6).contains(&x) {
                    3
                } else {
                    2
                }
            }
        },
        0x21 => {
            [
                "11333311", "13200231", "32000023", "32000023", "31200213", "33122133", "13300332",
                "11333311",
            ][y]
                .as_bytes()[x] as usize
                - b'0' as usize
        }
        _ => 0,
    }
}
fn palette(
    cells: &[&VisualTile],
    indices: &[usize; 16],
    images: &TerrainImageSamples,
) -> Option<[[u8; 4]; 4]> {
    let mut colors = [None; 4];
    for &i in indices {
        let rgba = tile_rgba(images, cells[i]).ok()?;
        if rgba.len() != 256 {
            return None;
        }
        for (p, bytes) in rgba.chunks_exact(4).enumerate() {
            let color: [u8; 4] = bytes.try_into().ok()?;
            if color[3] != 255 {
                return None;
            }
            let slot = &mut colors[ink(cells[i].source.tile_index, p % 8, p / 8)];
            if slot.is_some_and(|old| old != color) {
                return None;
            }
            *slot = Some(color);
        }
    }
    if cells[indices[0]].source.metatile_id == 0x05 {
        let a = colors[0]?;
        let b = colors[1]?;
        return (a != b).then_some([a, b, a, b]);
    }
    let colors = colors.map(|c| c.unwrap_or([0; 4]));
    if colors.iter().any(|c| c[3] != 255)
        || (0..4).any(|a| (a + 1..4).any(|b| colors[a] == colors[b]))
    {
        return None;
    }
    Some(colors)
}
fn paint(rgba: [u8; 4], gain: f32) -> [f32; 4] {
    let linear = |v: u8| {
        let s = f32::from(v) / 255.;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    [
        linear(rgba[0]) * gain,
        linear(rgba[1]) * gain,
        linear(rgba[2]) * gain,
        1.,
    ]
}
fn quad(out: &mut SurfaceMeshData, p: [[f32; 3]; 4], color: [f32; 4]) {
    let normal = (Vec3::from_array(p[1]) - Vec3::from_array(p[0]))
        .cross(Vec3::from_array(p[2]) - Vec3::from_array(p[0]))
        .normalize();
    // Terrain is palette-preserving/unlit. Match the existing exterior kit's
    // baked directional finish so chamfers read as volume in that material.
    let shade = 0.62 + 0.38 * normal.dot(Vec3::new(-0.35, 0.85, 0.4).normalize()).max(0.);
    let mut color = color;
    for c in &mut color[..3] {
        *c *= shade;
    }
    append_solid_quad(out, p, normal.to_array(), color);
}
fn slab(out: &mut SurfaceMeshData, b: [f32; 4], color: [f32; 4], scale: f32) {
    let [w, e, n, s] = b;
    let gap = scale * 0.009;
    let bevel = scale * 0.025;
    let low = -scale * 0.018;
    let outer = [
        [w + gap, low, n + gap],
        [w + gap, low, s - gap],
        [e - gap, low, s - gap],
        [e - gap, low, n + gap],
    ];
    let inner = [
        [w + gap + bevel, 0., n + gap + bevel],
        [w + gap + bevel, 0., s - gap - bevel],
        [e - gap - bevel, 0., s - gap - bevel],
        [e - gap - bevel, 0., n + gap + bevel],
    ];
    quad(out, inner, color);
    for j in 0..4 {
        let k = (j + 1) % 4;
        quad(out, [outer[j], outer[k], inner[k], inner[j]], color);
    }
}

// Clip a whole staggered brick, including its bevels, at the source-cell edge.
// Cutting a half-brick adds no false mortar line or bevel at that clipping seam.
fn clipped_slab(
    out: &mut SurfaceMeshData,
    b: [f32; 4],
    clip: [f32; 4],
    color: [f32; 4],
    scale: f32,
) {
    let mut whole = SurfaceMeshData::default();
    slab(&mut whole, b, color, scale);
    for (q, p) in whole.positions.chunks_exact(4).enumerate() {
        let mut points = p.iter().copied().map(Vec3::from_array).collect::<Vec<_>>();
        for (axis, bound, sign) in [
            (0, clip[0], 1.),
            (0, clip[1], -1.),
            (2, clip[2], 1.),
            (2, clip[3], -1.),
        ] {
            let mut clipped = Vec::new();
            for j in 0..points.len() {
                let a = points[j];
                let z = points[(j + 1) % points.len()];
                let da = (a[axis] - bound) * sign;
                let dz = (z[axis] - bound) * sign;
                if da >= 0. {
                    clipped.push(a);
                }
                if (da >= 0.) != (dz >= 0.) {
                    let mut p = a.lerp(z, da / (da - dz));
                    p[axis] = bound;
                    clipped.push(p);
                }
            }
            points = clipped;
        }
        if points.len() < 3 {
            continue;
        }
        // Directed ground interpolates separately across the source cell's
        // NW-SE diagonal. A brick triangle spanning that crease can sink
        // beneath its bedding on a graded bypass, exposing white triangles.
        // Preserve the same two planar domains before the shared deformation.
        for sign in [-1., 1.] {
            let distance = |p: Vec3| {
                sign * ((p.x - clip[0]) / (clip[1] - clip[0])
                    - (p.z - clip[2]) / (clip[3] - clip[2]))
            };
            let mut half = Vec::new();
            for j in 0..points.len() {
                let a = points[j];
                let b = points[(j + 1) % points.len()];
                let da = distance(a);
                let db = distance(b);
                if da >= 0. {
                    half.push(a);
                }
                if (da >= 0.) != (db >= 0.) {
                    let mut p = a.lerp(b, da / (da - db));
                    p.z = clip[2] + (p.x - clip[0]) * (clip[3] - clip[2]) / (clip[1] - clip[0]);
                    half.push(p);
                }
            }
            if half.len() < 3 {
                continue;
            }
            let vertices = half
                .into_iter()
                .map(|p| (p.to_array(), [0.; 2]))
                .collect::<Vec<_>>();
            append_polygon(out, &vertices, whole.normals[q * 4], whole.colors[q * 4]);
        }
    }
}

fn brick_deck(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    i: usize,
    source: &VisualTileSource,
    colors: [[u8; 4]; 4],
) {
    let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
    let scale = g.tile_width.min(g.tile_height);
    quad(
        out,
        [
            [w, -scale * 0.02, n],
            [w, -scale * 0.02, s],
            [e, -scale * 0.02, s],
            [e, -scale * 0.02, n],
        ],
        paint(colors[0], 0.76),
    );
    let height = (s - n) / 3.;
    for row in 0..3 {
        let phase = i32::from(source.subtile_row) * 3 + row;
        let offset = if phase % 2 == 0 { 0. } else { (e - w) * 0.5 };
        for col in -1..1 {
            let x = w + col as f32 * (e - w) + offset;
            // Clipped halves of one brick share pigment, even at a native
            // metatile phase wrap. Viewport translations cannot repaint it.
            let brick = (i32::from(source.subtile_column) + col).rem_euclid(4);
            let gain = 0.91 + (brick + phase).rem_euclid(3) as f32 * 0.015;
            clipped_slab(
                out,
                [
                    x,
                    x + e - w,
                    n + row as f32 * height,
                    n + (row + 1) as f32 * height,
                ],
                [w, e, n, s],
                paint(colors[1], gain),
                scale,
            );
        }
    }
}
fn brick_water_face(
    out: &mut SurfaceMeshData,
    p: [[f32; 3]; 4],
    normal: [f32; 3],
    colors: [[u8; 4]; 4],
    scale: f32,
) {
    let outer = p.map(Vec3::from_array);
    let n = Vec3::from_array(normal);
    let tangent = (outer[3] - outer[0]).normalize();
    let gap = scale * 0.018;
    let inner = [
        outer[0] + tangent * gap + Vec3::Y * gap + n * gap,
        outer[1] + tangent * gap - Vec3::Y * gap + n * gap,
        outer[2] - tangent * gap - Vec3::Y * gap + n * gap,
        outer[3] - tangent * gap + Vec3::Y * gap + n * gap,
    ];
    quad(out, p, paint(colors[0], 0.72));
    quad(
        out,
        inner.map(|point| point.to_array()),
        paint(colors[1], 0.94),
    );
    for j in 0..4 {
        let k = (j + 1) % 4;
        quad(
            out,
            [
                outer[j].to_array(),
                outer[k].to_array(),
                inner[k].to_array(),
                inner[j].to_array(),
            ],
            paint(colors[1], 0.94),
        );
    }
}
fn append(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    i: usize,
    source: &VisualTileSource,
    colors: [[u8; 4]; 4],
) {
    let tile = source.tile_index;
    if tile == 0x4b {
        brick_deck(out, g, i, source, colors);
        return;
    }
    let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
    let scale = g.tile_width.min(g.tile_height);
    quad(
        out,
        [
            [w, -scale * 0.02, n],
            [w, -scale * 0.02, s],
            [e, -scale * 0.02, s],
            [e, -scale * 0.02, n],
        ],
        if tile == 0x23 {
            let a = paint(colors[0], 0.65);
            let b = paint(colors[1], 0.9);
            std::array::from_fn(|j| if j == 3 { 1. } else { a[j] * 0.6 + b[j] * 0.4 })
        } else {
            paint(colors[1], 0.78)
        },
    );
    if tile == 0x23 {
        // Pale walking flags retain their source palette, with modest wear.
        let phase = (source.subtile_column + 2 * source.subtile_row) % 4;
        slab(
            out,
            [w, e, n, s],
            paint(colors[0], 0.90 + f32::from(phase) * 0.01),
            scale,
        );
    } else if tile == 0x21 {
        let center = Vec3::new((w + e) * 0.5, 0., (n + s) * 0.5);
        let ring: Vec<_> = (0..16)
            .map(|j| {
                let a = -(j as f32) * std::f32::consts::TAU / 16.;
                center + Vec3::new(a.cos() * (e - w) * 0.46, 0., a.sin() * (s - n) * 0.46)
            })
            .collect();
        let top: Vec<_> = ring.iter().map(|p| center + (*p - center) * 0.92).collect();
        let inner: Vec<_> = top.iter().map(|p| center + (*p - center) * 0.66).collect();
        let vertices: Vec<_> = inner.iter().map(|p| (p.to_array(), [0.; 2])).collect();
        append_polygon(out, &vertices, [0., 1., 0.], paint(colors[0], 0.90));
        for j in 0..16 {
            let k = (j + 1) % 16;
            quad(
                out,
                [
                    top[j].to_array(),
                    top[k].to_array(),
                    inner[k].to_array(),
                    inner[j].to_array(),
                ],
                paint(colors[1], 0.96),
            );
            quad(
                out,
                [
                    (ring[j] - Vec3::Y * scale * 0.018).to_array(),
                    (ring[k] - Vec3::Y * scale * 0.018).to_array(),
                    top[k].to_array(),
                    top[j].to_array(),
                ],
                paint(colors[1], 0.96),
            );
        }
    } else {
        slab(out, [w, e, n, s], paint(colors[1], 0.96), scale);
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    images: &TerrainImageSamples,
) {
    let old = &mesh.textured;
    if g.width == 0
        || cells.len() != g.width * g.height
        || shapes.len() != cells.len()
        || old.positions.len() % 4 != 0
        || old.indices.len() != old.positions.len() / 4 * 6
        || old.indices.chunks_exact(6).enumerate().any(|(q, indices)| {
            let b = (q * 4) as u32;
            indices != [b, b + 1, b + 2, b, b + 2, b + 3]
        })
    {
        return;
    }
    let mut faces = vec![None; cells.len()];
    let mut face_counts = vec![0; cells.len()];
    let mut bridge_walls = vec![Vec::new(); cells.len()];
    for (q, (p, uv)) in old
        .positions
        .chunks_exact(4)
        .zip(old.uvs.chunks_exact(4))
        .enumerate()
    {
        let Some(i) = new_bark::sampled_cell(uv, g) else {
            continue;
        };
        if cells[i].source.tileset_id.as_ref() == "kanto"
            && cells[i].source.metatile_id == 0x05
            && cells[i].source.tile_index == 0x4b
            && old.normals[q * 4][1] == 0.
            && old.normals[q * 4..q * 4 + 4]
                .iter()
                .all(|n| *n == old.normals[q * 4])
            && kanto_shore::native_face(p, old.normals[q * 4], uv, i, shapes, g)
        {
            bridge_walls[i].push(q);
        }
        let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
        let expected = [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]];
        if p.iter().all(|p| expected.contains(p))
            && expected.iter().all(|v| p.contains(v))
            && old.normals[q * 4..q * 4 + 4].iter().all(|n| n[1] > 0.999)
        {
            faces[i] = Some(q);
            face_counts[i] += 1;
        }
    }
    let mut removed = std::collections::HashSet::new();
    for start in 0..cells.len() {
        let first = &cells[start].source;
        if first.tileset_id.as_ref() != "kanto"
            || first.subtile_column != 0
            || first.subtile_row != 0
        {
            continue;
        }
        let Some(drawing) = drawing(first.metatile_id) else {
            continue;
        };
        let x = start % g.width;
        let y = start / g.width;
        if x + 4 > g.width || y + 4 > g.height {
            continue;
        }
        let indices = std::array::from_fn(|j| (y + j / 4) * g.width + x + j % 4);
        if indices.iter().enumerate().any(|(j, &i)| {
            let s = &cells[i].source;
            let mut wall_normals = std::collections::HashSet::new();
            s.tileset_id != first.tileset_id
                || s.metatile_id != first.metatile_id
                || s.subtile_column as usize != j % 4
                || s.subtile_row as usize != j / 4
                || s.tile_index != drawing[j]
                || !matches!(
                    shapes[i],
                    CellShape::Flat | CellShape::PlaneAt { height: 0. }
                )
                || faces[i].is_none()
                || face_counts[i] != 1
                || faces[i].is_some_and(|q| {
                    mesh.cutaway_links
                        .iter()
                        .any(|&(_, anchor)| anchor / 4 == q)
                })
                || mesh.authored_cells.get(i).is_none_or(|c| c.is_some())
                || bridge_walls[i]
                    .iter()
                    .any(|&q| mesh.cutaway_links.iter().any(|&(_, a)| a / 4 == q))
                || bridge_walls[i]
                    .iter()
                    .any(|&q| !wall_normals.insert(old.normals[q * 4].map(f32::to_bits)))
        }) {
            continue;
        }
        let Some(colors) = palette(cells, &indices, images) else {
            continue;
        };
        for &i in &indices {
            append(&mut mesh.solid, g, i, &cells[i].source, colors);
            mesh.authored_cells[i] = Some(if first.metatile_id == 0x05 {
                BRIDGE_LABEL
            } else {
                LABEL
            });
            removed.insert(faces[i].unwrap());
            for &q in &bridge_walls[i] {
                brick_water_face(
                    &mut mesh.solid,
                    old.positions[q * 4..q * 4 + 4].try_into().unwrap(),
                    old.normals[q * 4],
                    colors,
                    g.tile_width.min(g.tile_height),
                );
                removed.insert(q);
            }
        }
    }
    if removed.is_empty() {
        return;
    }
    let old = std::mem::take(&mut mesh.textured);
    for (_, anchor) in &mut mesh.cutaway_links {
        *anchor -= removed.iter().filter(|&&q| q < *anchor / 4).count() * 4;
    }
    let mut marked = vec![false; old.positions.len()];
    for r in &old.cutaway_ranges {
        marked[r.clone()].fill(true);
    }
    let mut remap = vec![u32::MAX; old.positions.len()];
    mesh.textured = select_surface_indices(
        &old,
        &marked,
        &mut remap,
        old.indices
            .chunks_exact(6)
            .enumerate()
            .filter(|(q, _)| !removed.contains(q))
            .flat_map(|(_, indices)| indices.iter().copied()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    const COLORS: [[u8; 4]; 4] = [
        [238, 246, 220, 255],
        [142, 156, 150, 255],
        [74, 94, 83, 255],
        [22, 32, 28, 255],
    ];
    fn fixture(
        block: u16,
    ) -> (
        Vec<VisualTile>,
        Vec<CellShape>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let g = GridGeometry {
            width: 4,
            height: 4,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -16.,
            origin_z: 24.,
        };
        let mut assets = Assets::<Image>::default();
        let mut samples = TerrainImageSamples::default();
        let mut cells = Vec::new();
        let mut mesh = TerrainMeshData::default();
        for (i, tile) in drawing(block).unwrap().into_iter().enumerate() {
            let texture = assets.add(Image::default());
            let rgba = (0..64)
                .flat_map(|p| COLORS[ink(tile, p % 8, p / 8)])
                .collect();
            samples
                .pixels
                .insert(texture.id(), TileImageSample::Rgba(rgba));
            cells.push(VisualTile {
                column: (i % 4) as u32,
                row: (i / 4) as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("kanto"),
                    metatile_id: block,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 4) as u8,
                    tile_index: tile,
                },
                texture,
                animation_frames: None,
                priority: false,
            });
            let (w, e, n, s) = g.bounds(i % 4, i / 4);
            append_top_shaded(
                &mut mesh.textured,
                [w, e, n, s],
                0.,
                g.uv(i % 4, i / 4),
                [1.; 4],
            );
        }
        mesh.footing_heights = vec![0.; 16];
        mesh.authored_cells = vec![None; 16];
        let shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        (cells, shapes, g, samples, mesh)
    }
    fn bridge_fixture() -> (
        Vec<VisualTile>,
        Vec<CellShape>,
        GridGeometry,
        TerrainImageSamples,
        TerrainMeshData,
    ) {
        let g = GridGeometry {
            width: 6,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -24.,
            origin_z: 32.,
        };
        let mut assets = Assets::<Image>::default();
        let mut images = TerrainImageSamples::default();
        let mut cells = Vec::new();
        let mut shapes = Vec::new();
        let mut mesh = TerrainMeshData::default();
        for i in 0..36 {
            let x = i % 6;
            let y = i / 6;
            let bridge = (1..5).contains(&x) && (1..5).contains(&y);
            let tile = if bridge { 0x4b } else { 0x14 };
            let texture = assets.add(Image::default());
            let rgba = (0..64)
                .flat_map(|p| {
                    if bridge {
                        COLORS[ink(tile, p % 8, p / 8)]
                    } else {
                        [30, 70, 100, 255]
                    }
                })
                .collect();
            images
                .pixels
                .insert(texture.id(), TileImageSample::Rgba(rgba));
            cells.push(VisualTile {
                column: x as u32,
                row: y as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("kanto"),
                    metatile_id: if bridge { 5 } else { 0x43 },
                    subtile_column: if bridge { (x - 1) as u8 } else { 0 },
                    subtile_row: if bridge { (y - 1) as u8 } else { 0 },
                    tile_index: tile,
                },
                texture,
                animation_frames: None,
                priority: false,
            });
            shapes.push(if bridge {
                CellShape::Flat
            } else {
                CellShape::Water
            });
            mesh.footing_heights.push(shapes[i].surface_height(8.));
            mesh.authored_cells.push(None);
            let (w, e, n, s) = g.bounds(x, y);
            append_top_shaded(
                &mut mesh.textured,
                [w, e, n, s],
                shapes[i].surface_height(8.),
                g.uv(x, y),
                [1.; 4],
            );
        }
        for i in 0..36 {
            if cells[i].source.metatile_id != 5 {
                continue;
            }
            let x = i % 6;
            let y = i / 6;
            let (w, e, n, s) = g.bounds(x, y);
            for d in Direction::ALL {
                let (dx, dy) = d.offset();
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0
                    && ny >= 0
                    && nx < 6
                    && ny < 6
                    && shapes[ny as usize * 6 + nx as usize] == CellShape::Water
                {
                    append_textured_shoreline(
                        &mut mesh.textured,
                        &g,
                        x,
                        y,
                        d,
                        [w, e, n, s],
                        -2.,
                        0.,
                    );
                }
            }
        }
        (cells, shapes, g, images, mesh)
    }
    #[test]
    fn brick_bridge_keeps_original_support_and_finishes_all_water_edges() {
        let (cells, shapes, g, images, mut mesh) = bridge_fixture();
        let feet = mesh.footing_heights.clone();
        finish(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &images,
        );
        assert_eq!(mesh.footing_heights, feet);
        assert!(mesh
            .textured
            .uvs
            .chunks_exact(4)
            .all(|uv| new_bark::sampled_cell(uv, &g)
                .is_none_or(|i| cells[i].source.metatile_id != 5)));
        for (i, c) in cells.iter().enumerate() {
            assert_eq!(
                mesh.authored_cells[i],
                (c.source.metatile_id == 5).then_some(BRIDGE_LABEL)
            );
        }
        assert!(mesh
            .solid
            .positions
            .iter()
            .all(|p| p[1] <= 0. && p[1] >= -2.));
        for tri in mesh.solid.indices.chunks_exact(3) {
            let [a, b, c] =
                std::array::from_fn(|j| Vec3::from_array(mesh.solid.positions[tri[j] as usize]));
            let cross = (b - a).cross(c - a);
            if cross.length_squared() > 0.000001 {
                assert!(cross.dot(Vec3::from_array(mesh.solid.normals[tri[0] as usize])) > 0.);
            }
        }
        for normal in [[1., 0., 0.], [-1., 0., 0.], [0., 0., 1.], [0., 0., -1.]] {
            assert!(
                mesh.solid.normals.contains(&normal),
                "water edge {normal:?}"
            );
        }
    }
    #[test]
    fn graded_source_cell_cannot_push_white_bedding_through_its_brick_caps() {
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let source = VisualTileSource {
            tileset_id: Arc::from("kanto"),
            metatile_id: 5,
            subtile_column: 0,
            subtile_row: 0,
            tile_index: 0x4b,
        };
        let mut deck = SurfaceMeshData::default();
        brick_deck(&mut deck, &g, 0, &source, COLORS);
        let grade = |p: Vec3| 6. * (p.x / 8.).min(p.z / 8.);
        for tri in deck.indices.chunks_exact(3) {
            if tri.iter().any(|&i| i < 4) {
                continue;
            }
            let p: [Vec3;3] = std::array::from_fn(|j|Vec3::from_array(deck.positions[tri[j] as usize]));
            if (p[1] - p[0]).cross(p[2] - p[0]).length_squared() < 0.000001 {
                continue;
            }
            let center = (p[0] + p[1] + p[2]) / 3.;
            let top = p.iter().map(|p| p.y + grade(*p)).sum::<f32>() / 3.;
            assert!(
                top >= grade(center) - 0.16 - 0.00001,
                "bedding breaks through at {center:?}"
            );
        }
    }
    #[test]
    fn clipped_staggered_bricks_join_without_a_false_source_cell_seam() {
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut pair = Vec::new();
        for column in 0..2 {
            let source = VisualTileSource {
                tileset_id: Arc::from("kanto"),
                metatile_id: 5,
                subtile_column: column,
                subtile_row: 0,
                tile_index: 0x4b,
            };
            let mut surface = SurfaceMeshData::default();
            brick_deck(&mut surface, &g, column as usize, &source, COLORS);
            assert!(surface.positions.iter().all(|p| p[0] >= column as f32 * 8.
                && p[0] <= (column + 1) as f32 * 8.
                && p[2] >= 0.
                && p[2] <= 8.));
            let edge = surface
                .positions
                .iter()
                .zip(&surface.colors)
                .zip(&surface.normals)
                .filter(|((p, _), _)| {
                    p[0] == 8. && p[2] > g.tile_height / 3. && p[2] < g.tile_height * 2. / 3.
                })
                .map(|((p, c), n)| {
                    (
                        [p[1].to_bits(), p[2].to_bits()],
                        c.map(f32::to_bits),
                        n.map(f32::to_bits),
                    )
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert!(!edge.is_empty());
            pair.push(edge);
        }
        assert_eq!(pair[0], pair[1]);
    }
    #[test]
    fn unknown_brick_art_or_source_phase_does_not_consume_the_bridge() {
        for defect in 0..3 {
            let (mut cells, shapes, g, mut images, mut mesh) = bridge_fixture();
            if defect == 0 {
                cells[8].source.subtile_column = 0;
            } else if defect == 1 {
                mesh.authored_cells[8] = Some("other");
            } else if let TileImageSample::Rgba(p) =
                images.pixels.get_mut(&cells[8].texture.id()).unwrap()
            {
                p[0] ^= 32;
            }
            let old = mesh.textured.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert_eq!(mesh.textured, old);
            assert!(mesh.solid.positions.is_empty());
        }
    }
    #[test]
    fn all_native_floor_drawings_get_bevels_without_raising_standing_datum() {
        for block in [0x22, 0x23, 0x26, 0x27, 0x44, 0x45, 0x48, 0x49, 0x5d] {
            let (cells, shapes, g, images, mut mesh) = fixture(block);
            let feet = mesh.footing_heights.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert!(mesh.textured.positions.is_empty(), "block {block:x}");
            assert!(mesh.authored_cells.iter().all(|c| *c == Some(LABEL)));
            assert_eq!(mesh.footing_heights, feet);
            assert!(mesh.solid.positions.iter().all(|p| p[1] <= 0.
                && p[1] >= -0.16
                && (-16. ..=16.).contains(&p[0])
                && (24. ..=56.).contains(&p[2])));
            assert!(mesh.solid.positions.iter().any(|p| p[1] < 0.));
            assert!(mesh.solid.normals.iter().any(|n| n[1] > 0. && n[1] < 0.999));
            for tri in mesh.solid.indices.chunks_exact(3) {
                let p = tri
                    .iter()
                    .map(|&i| Vec3::from_array(mesh.solid.positions[i as usize]))
                    .collect::<Vec<_>>();
                let cross = (p[1] - p[0]).cross(p[2] - p[0]);
                if cross.length_squared() > 0.000001 {
                    assert!(cross.y > 0., "upward floor winding");
                }
            }
        }
    }
    #[test]
    fn changed_art_phase_claim_height_and_borrowed_sample_reject_the_complete_block() {
        for defect in 0..8 {
            let (mut cells, mut shapes, g, mut images, mut mesh) = fixture(0x44);
            match defect {
                0 => cells[6].source.subtile_column = 1,
                1 => cells[6].source.metatile_id = 0x45,
                2 => cells[6].source.tile_index = 0x39,
                3 => mesh.authored_cells[6] = Some("other"),
                4 => shapes[6] = CellShape::PlaneAt { height: 2. },
                5 => mesh.textured.positions[6 * 4][1] = 2.,
                6 => {
                    if let TileImageSample::Rgba(rgba) =
                        images.pixels.get_mut(&cells[6].texture.id()).unwrap()
                    {
                        rgba[3] = 0;
                    }
                }
                _ => mesh.textured.uvs[6 * 4..6 * 4 + 4].copy_from_slice(&[[0., 0.]; 4]),
            }
            let old = mesh.textured.clone();
            finish(
                &mut mesh,
                &cells.iter().collect::<Vec<_>>(),
                &shapes,
                &g,
                &images,
            );
            assert_eq!(mesh.textured, old, "defect {defect}");
            assert!(mesh.solid.positions.is_empty());
        }
        let (mut cells, shapes, g, images, mut mesh) = fixture(0x44);
        cells.truncate(12);
        let old = mesh.textured.clone();
        finish(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &shapes,
            &g,
            &images,
        );
        assert_eq!(mesh.textured, old);
    }
    #[test]
    fn live_palette_changes_paint_but_not_geometry() {
        let (cells, shapes, g, images, mut day) = fixture(0x27);
        let mut night_images = images.clone();
        for sample in night_images.pixels.values_mut() {
            if let TileImageSample::Rgba(rgba) = sample {
                for p in rgba.chunks_exact_mut(4) {
                    p[0] /= 2;
                    p[1] /= 2;
                    p[2] = (u16::from(p[2]) * 3 / 4) as u8;
                }
            }
        }
        let mut night = day.clone();
        let cells = cells.iter().collect::<Vec<_>>();
        finish(&mut day, &cells, &shapes, &g, &images);
        finish(&mut night, &cells, &shapes, &g, &night_images);
        assert_eq!(day.solid.positions, night.solid.positions);
        assert_eq!(day.footing_heights, night.footing_heights);
        assert_ne!(day.solid.colors, night.solid.colors);
    }
}
