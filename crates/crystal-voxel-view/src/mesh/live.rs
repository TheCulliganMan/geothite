//! Whole-drawing folds selected by the reloadable profile document.
use super::*;
use crate::live_profiles::{Document, Mask, Object, Part};

pub(super) struct Placement<'a> {
    object: &'a Object,
    column: usize,
    row: usize,
    ground: usize,
}

impl Placement<'_> {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.object.tiles.len()).flat_map(move |y| {
            (0..self.object.tiles[0].len()).map(move |x| (self.row + y) * width + self.column + x)
        })
    }
}

pub(super) fn resolve<'a>(
    cells: &[&VisualTile],
    width: usize,
    height: usize,
    map: &str,
    document: Option<&'a Document>,
) -> Vec<Placement<'a>> {
    let mut placements = Vec::new();
    let Some(document) = document else {
        return placements;
    };
    let trace = std::env::var("CRYSTAL_VOXEL_TRACE_PROFILES").ok();
    let mut claimed = vec![false; cells.len()];
    for object in &document.objects {
        if object.map.as_deref().is_some_and(|id| id != map)
            || object.maps.as_ref().is_some_and(|ids| !ids.iter().any(|id| id == map))
        {
            continue;
        }
        let tracing = trace.as_ref().is_some_and(|filter| object.name.contains(filter));
        let before = placements.len();
        let Some(ground) = cells.iter().position(|tile| {
            tile.source.tileset_id.as_ref() == object.tileset
                && tile.source.tile_index == object.ground
                && matches!(
                    shape_for_source_on_map(map, &tile.source),
                    CellShape::Flat | CellShape::Water
                )
        }) else {
            if tracing {
                eprintln!("profile placement: map={map} name={:?} skipped=no-flat-water-ground tile={}", object.name, object.ground);
            }
            continue;
        };
        let (w, h) = (object.tiles[0].len(), object.tiles.len());
        if w > width || h > height {
            if tracing {
                eprintln!("profile placement: map={map} name={:?} skipped=drawing-larger-than-grid", object.name);
            }
            continue;
        }
        for row in 0..=height - h {
            for column in 0..=width - w {
                let matches = (0..h).all(|y| {
                    (0..w).all(|x| {
                        let index = (row + y) * width + column + x;
                        let source = &cells[index].source;
                        !claimed[index]
                            && source.tileset_id.as_ref() == object.tileset
                            && source.metatile_id
                                == object.metatiles.as_ref().map_or(object.metatile, |blocks| {
                                    blocks[(usize::from(object.origin[1]) + y) / 4]
                                        [(usize::from(object.origin[0]) + x) / 4]
                                })
                            && usize::from(source.subtile_column)
                                == (usize::from(object.origin[0]) + x) % 4
                            && usize::from(source.subtile_row)
                                == (usize::from(object.origin[1]) + y) % 4
                            && source.tile_index == object.tiles[y][x]
                    })
                });
                if matches {
                    let placement = Placement {
                        object,
                        column,
                        row,
                        ground,
                    };
                    for index in placement.indices(width) {
                        claimed[index] = true;
                    }
                    placements.push(placement);
                }
            }
        }
        if tracing {
            eprintln!("profile placement: map={map} name={:?} matches={} ground-cell={ground}", object.name, placements.len() - before);
        }
    }
    placements
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    geometry: &GridGeometry,
    placement: &Placement<'_>,
    cells: &[&VisualTile],
    images: &TerrainImageSamples,
) -> Result<(), TerrainMeshError> {
    let object = placement.object;
    let width = object.tiles[0].len() * 8;
    let height = object.tiles.len() * 8;
    let ground_uv = geometry.uv(
        placement.ground % geometry.width,
        placement.ground / geometry.width,
    );
    for index in placement.indices(geometry.width) {
        let (x0, x1, z0, z1) = geometry.bounds(index % geometry.width, index / geometry.width);
        append_top(&mut mesh.textured, [x0, x1, z0, z1], 0.0, ground_uv);
    }
    if let Some(height) = object.footing_pixels {
        for index in placement.indices(geometry.width) {
            mesh.footing_heights[index] = height * geometry.tile_height / 8.0;
        }
    }
    let ground = tile_rgba(images, cells[placement.ground])?;
    let colors: Vec<_> = ground
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2], p[3]])
        .collect();
    let mut background = vec![false; width * height];
    for ty in 0..object.tiles.len() {
        for tx in 0..object.tiles[0].len() {
            let index = (placement.row + ty) * geometry.width + placement.column + tx;
            let rgba = tile_rgba(images, cells[index])?;
            for y in 0..8 {
                for x in 0..8 {
                    let i = (y * 8 + x) * 4;
                    background[(ty * 8 + y) * width + tx * 8 + x] = rgba[i + 3] == 0
                        || colors.contains(&[rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]);
                }
            }
        }
    }
    let default_part = Part {
        rect: [0, 0, width, height],
        top_pixels: object.top_pixels,
        depth_pixels: object.depth_pixels,
        base_pixels: 0.0,
        height_pixels: None,
        bevel_pixels: None,
        offset_pixels: [0.0, 0.0],
        mask: object.mask,
    };
    let parts = if object.parts.is_empty() {
        std::slice::from_ref(&default_part)
    } else {
        &object.parts
    };
    for part in parts {
        append_part(mesh, geometry, placement, part, &background, width);
    }
    Ok(())
}

fn append_part(
    mesh: &mut TerrainMeshData,
    geometry: &GridGeometry,
    placement: &Placement<'_>,
    part: &Part,
    background: &[bool],
    drawing_width: usize,
) {
    let [rx, ry, w, h] = part.rect;
    let mask = match part.mask {
        Mask::None => vec![false; w * h],
        Mask::Ground => {
            let candidates: Vec<_> = (0..h)
                .flat_map(|y| (0..w).map(move |x| background[(ry + y) * drawing_width + rx + x]))
                .collect();
            boundary_connected_mask(w, h, &candidates)
        }
    };
    let on = |x: isize, y: isize| {
        x >= 0
            && y >= 0
            && (x as usize) < w
            && (y as usize) < h
            && !mask[y as usize * w + x as usize]
    };
    let sx = geometry.tile_width / 8.0;
    let sy = geometry.tile_height / 8.0;
    let base = part.base_pixels * sy;
    let front_rows = h - part.top_pixels;
    let rise = part.height_pixels.unwrap_or(front_rows as f32) * sy;
    let top_height = base + rise;
    let front = geometry.origin_z
        + placement.row as f32 * geometry.tile_height
        + (ry as f32 + h as f32 + part.offset_pixels[1]) * sy;
    let back = front - part.depth_pixels * sy;
    let left = geometry.origin_x
        + placement.column as f32 * geometry.tile_width
        + (rx as f32 + part.offset_pixels[0]) * sx;
    let uv = |x: usize, y: usize| {
        let x = rx + x;
        let y = ry + y;
        let (u0, u1, v0, v1) = geometry.uv(placement.column + x / 8, placement.row + y / 8);
        [
            lerp_pixel(u0, u1, x % 8),
            lerp_pixel(u0, u1, x % 8 + 1),
            lerp_pixel(v0, v1, y % 8),
            lerp_pixel(v0, v1, y % 8 + 1),
        ]
    };
    for y in 0..h {
        for x in 0..w {
            if !on(x as isize, y as isize) {
                continue;
            }
            let [u0, u1, v0, v1] = uv(x, y);
            let x0 = left + x as f32 * sx;
            let x1 = x0 + sx;
            let tex = [[u0, v1], [u0, v0], [u1, v0], [u1, v1]];
            if y < part.top_pixels {
                let z0 = back + (front - back) * y as f32 / part.top_pixels as f32;
                let z1 = back + (front - back) * (y + 1) as f32 / part.top_pixels as f32;
                if let Some(bevel) = part.bevel_pixels {
                    // Map every source texel once over a rounded bevel. The cap
                    // meets the ground at the perimeter, so there is no invented
                    // side texture or open vertical edge. Adjacent texels share
                    // identical vertex heights, including the corner slopes.
                    let height = |px: usize, py: usize| {
                        let edge_x = px.min(w - px) as f32;
                        let edge_y = py.min(h - py) as f32;
                        let dx = (1.0 - edge_x / bevel).max(0.0);
                        let dy = (1.0 - edge_y / bevel).max(0.0);
                        base + rise * (1.0 - dx.hypot(dy).min(1.0))
                    };
                    let a = Vec3::new(x0, height(x, y), z0);
                    let b = Vec3::new(x0, height(x, y + 1), z1);
                    let c = Vec3::new(x1, height(x + 1, y + 1), z1);
                    let d = Vec3::new(x1, height(x + 1, y), z0);
                    let normal = (b - a).cross(d - a).normalize().to_array();
                    append_quad(
                        &mut mesh.textured,
                        [a.to_array(), b.to_array(), c.to_array(), d.to_array()],
                        normal,
                        [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                        TEXTURED_SHADE,
                    );
                    continue;
                }
                append_quad(
                    &mut mesh.textured,
                    [
                        [x0, top_height, z0],
                        [x0, top_height, z1],
                        [x1, top_height, z1],
                        [x1, top_height, z0],
                    ],
                    [0.0, 1.0, 0.0],
                    [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
                    TEXTURED_SHADE,
                );
                continue;
            }
            let top = base + rise * (h - y) as f32 / front_rows as f32;
            let bottom = base + rise * (h - y - 1) as f32 / front_rows as f32;
            append_quad(
                &mut mesh.textured,
                [
                    [x1, bottom, front],
                    [x1, top, front],
                    [x0, top, front],
                    [x0, bottom, front],
                ],
                [0.0, 0.0, 1.0],
                [[u1, v1], [u1, v0], [u0, v0], [u0, v1]],
                TEXTURED_SHADE,
            );
            if part.depth_pixels == 0.0 {
                continue;
            }
            append_quad(
                &mut mesh.textured,
                [
                    [x0, bottom, back],
                    [x0, top, back],
                    [x1, top, back],
                    [x1, bottom, back],
                ],
                [0.0, 0.0, -1.0],
                tex,
                [0.68, 0.68, 0.68, 1.0],
            );
            if !on(x as isize - 1, y as isize) {
                append_quad(
                    &mut mesh.textured,
                    [
                        [x0, bottom, front],
                        [x0, top, front],
                        [x0, top, back],
                        [x0, bottom, back],
                    ],
                    [-1.0, 0.0, 0.0],
                    tex,
                    [0.78, 0.78, 0.78, 1.0],
                );
            }
            if !on(x as isize + 1, y as isize) {
                append_quad(
                    &mut mesh.textured,
                    [
                        [x1, bottom, back],
                        [x1, top, back],
                        [x1, top, front],
                        [x1, bottom, front],
                    ],
                    [1.0, 0.0, 0.0],
                    tex,
                    [0.78, 0.78, 0.78, 1.0],
                );
            }
            if !on(x as isize, y as isize - 1) {
                append_quad(
                    &mut mesh.textured,
                    [
                        [x0, top, back],
                        [x0, top, front],
                        [x1, top, front],
                        [x1, top, back],
                    ],
                    [0.0, 1.0, 0.0],
                    tex,
                    TEXTURED_SHADE,
                );
            }
            if !on(x as isize, y as isize + 1) {
                append_quad(
                    &mut mesh.textured,
                    [
                        [x1, bottom, back],
                        [x1, bottom, front],
                        [x0, bottom, front],
                        [x0, bottom, back],
                    ],
                    [0.0, -1.0, 0.0],
                    tex,
                    [0.68, 0.68, 0.68, 1.0],
                );
            }
        }
    }
}
