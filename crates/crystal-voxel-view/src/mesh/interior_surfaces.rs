// Source-proven architectural finishes. These are surfaces, not object coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InteriorFloor {
    Oak,
    Tatami,
    Ceramic,
    Stone,
    Marble,
    LighthouseSlate,
    LighthouseChecker,
    GymRose,
    GymGreen,
    GymTimber,
}
impl InteriorFloor {
    fn palette(self) -> [f32; 3] {
        match self {
            Self::Oak => [0.52, 0.40, 0.27],
            Self::Tatami => [0.60, 0.59, 0.37],
            Self::Ceramic => [0.66, 0.72, 0.69],
            Self::Stone => [0.58, 0.60, 0.57],
            Self::Marble => [0.72, 0.69, 0.61],
            Self::LighthouseSlate => [0.38, 0.43, 0.44],
            Self::LighthouseChecker => [0.54, 0.56, 0.51],
            Self::GymRose => [0.69, 0.60, 0.56],
            Self::GymGreen => [0.42, 0.50, 0.34],
            Self::GymTimber => [0.59, 0.48, 0.31],
        }
    }
}
fn interior_floor(map: &str, source: &VisualTileSource) -> Option<InteriorFloor> {
    use InteriorFloor::*;
    let style = match (source.tileset_id.as_ref(), source.tile_index) {
        _ if traditional_room::tatami_source(map, source) => Tatami,
        _ if gym_floor_source(map, source).is_some() => gym_floor_source(map, source)?,
        ("players_room" | "players_house" | "house", 0x01) => Oak,
        ("traditional_house", 0x50 | 0x44 | 0x45 | 0x54 | 0x55) => Tatami,
        ("traditional_house", 0x01) => Oak,
        ("lab", 0x10 | 0x24) => Oak,
        ("mart", 0x01 | 0x48) | ("pokecenter", 0x11) => Ceramic,
        ("mansion", 0x01 | 0x11) => Marble,
        ("gate", 0x01)
        | ("facility", 0x01 | 0x26)
        | ("radio_tower", 0x01)
        | ("train_station", 0x3d)
        | ("battle_tower_inside", 0x11)
        | ("pokecom_center", 0x3d | 0x11) => Stone,
        ("game_corner", 0x01) => {
            if crate::cafe::is_cafe_map(map) {
                Oak
            } else {
                Stone
            }
        }
        ("lighthouse", _) if lighthouse_floor_source(map, source) => LighthouseSlate,
        ("lighthouse", _) if lighthouse_chamber_floor_source(map, source) => LighthouseChecker,
        _ => return None,
    };
    matches!(
        shape_for_source_on_map(map, source),
        CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
    )
    .then_some(style)
}
fn material_quad(mesh: &mut SurfaceMeshData, bounds: [f32; 4], height: f32, color: [f32; 3]) {
    let [x0, x1, z0, z1] = bounds;
    if x1 <= x0 || z1 <= z0 {
        return;
    }
    append_quad(
        mesh,
        [
            [x0, height, z0],
            [x0, height, z1],
            [x1, height, z1],
            [x1, height, z0],
        ],
        [0.0, 1.0, 0.0],
        [[0.0, 0.0]; 4],
        [color[0], color[1], color[2], 1.0],
    );
}
fn tint(color: [f32; 3], amount: f32) -> [f32; 3] {
    color.map(|v| (v + amount).clamp(0.0, 1.0))
}
fn floor_cell(
    mesh: &mut SurfaceMeshData,
    b: [f32; 4],
    height: f32,
    style: InteriorFloor,
    world: [i32; 2],
) {
    let [x0, x1, z0, z1] = b;
    let w = x1 - x0;
    let d = z1 - z0;
    let [column, row] = world;
    let palette = style.palette();
    if gym_floor_style(style) {
        gym_floor_cell(mesh, b, height, style, world);
    } else if style == InteriorFloor::LighthouseChecker {
        lighthouse_chamber_floor_cell(mesh, b, height, world, [false; 4]);
    } else if style == InteriorFloor::LighthouseSlate {
        lighthouse_floor_cell(mesh, b, height, world);
    } else if style == InteriorFloor::Oak {
        // Wide boards with staggered end joints. The low-contrast palette and
        // sparse joints avoid the old high-frequency striped source texture.
        for half in 0..2 {
            let plank = row * 2 + half;
            let za = z0 + d * half as f32 * 0.5;
            let zb = za + d * 0.5;
            let offset = (plank.rem_euclid(2)) * 2;
            let board = (column - offset).div_euclid(4);
            let variation = ((board * 13 + plank * 7).rem_euclid(7) as f32 - 3.0) * 0.007;
            let seam = d * 0.018;
            let joint = if (column - offset).rem_euclid(4) == 0 {
                w * 0.018
            } else {
                0.0
            };
            let color = tint(palette, variation);
            let dark = tint(palette, -0.10);
            material_quad(mesh, [x0, x1, za, za + seam], height, dark);
            if joint > 0.0 {
                material_quad(mesh, [x0, x0 + joint, za + seam, zb], height, dark);
            }
            material_quad(mesh, [x0 + joint, x1, za + seam, zb], height, color);
        }
    } else if style == InteriorFloor::Tatami {
        let mat_x = column.div_euclid(2);
        let mat_z = row.div_euclid(4);
        let variation = ((mat_x + mat_z).rem_euclid(2) as f32 - 0.5) * 0.035;
        let color = tint(palette, variation);
        let hem = [0.29, 0.35, 0.25];
        let west = if column.rem_euclid(2) == 0 {
            w * 0.06
        } else {
            0.0
        };
        let north = if row.rem_euclid(4) == 0 {
            d * 0.06
        } else {
            0.0
        };
        if north > 0.0 {
            material_quad(mesh, [x0, x1, z0, z0 + north], height, hem);
        }
        if west > 0.0 {
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, hem);
        }
        for line in 0..4 {
            let a = z0 + north + (d - north) * line as f32 / 4.0;
            let b = z0 + north + (d - north) * (line + 1) as f32 / 4.0;
            material_quad(
                mesh,
                [x0 + west, x1, a, b],
                height,
                tint(color, if line % 2 == 0 { 0.005 } else { -0.005 }),
            );
        }
    } else {
        let variation =
            ((column.div_euclid(2) * 3 + row.div_euclid(2) * 5).rem_euclid(5) as f32 - 2.0) * 0.008;
        let color = tint(palette, variation);
        let grout = tint(palette, -0.085);
        let west = if column.rem_euclid(2) == 0 {
            w * 0.025
        } else {
            0.0
        };
        let north = if row.rem_euclid(2) == 0 {
            d * 0.025
        } else {
            0.0
        };
        if north > 0.0 {
            material_quad(mesh, [x0, x1, z0, z0 + north], height, grout);
        }
        if west > 0.0 {
            material_quad(mesh, [x0, x0 + west, z0 + north, z1], height, grout);
        }
        material_quad(mesh, [x0 + west, x1, z0 + north, z1], height, color);
    }
}
// Uses precisely the same grid phase and palette as the established room floor.
pub(super) fn cable_club_floor_cell(mesh: &mut SurfaceMeshData, bounds: [f32; 4], world: [i32; 2]) {
    floor_cell(mesh, bounds, 0.0, InteriorFloor::Ceramic, world);
}
fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.0001
}

/// Replace only exact flat-floor atlas quads, including verified floor restored
/// beneath objects. Source pictures, unknown strips, stair treads and walls keep
/// their own art. Geometry heights and all actor footing remain unchanged.
/// Returns finished floor quads; deliberately does not mark them as objects.
pub(super) fn finish_surfaces(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
) -> usize {
    finish_surfaces_excluding(mesh, map, cells, g, world_origin, &[])
}
/// Live source profiles have first refusal over the Gym material pass, including
/// profiles whose drawings consist entirely of a recognized native floor.
pub(super) fn finish_surfaces_with_profiles(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
    profiles: Option<&Document>,
) -> usize {
    if (!gym_floor_map(map) && !traditional_room::floor_map(map))
        || g.width.checked_mul(g.height) != Some(cells.len()) {
        return finish_surfaces(mesh, map, cells, g, world_origin);
    }
    let mut excluded = vec![false; cells.len()];
    for placement in live::resolve(cells, g.width, g.height, map, profiles) {
        for index in placement.indices(g.width) {
            excluded[index] = true;
        }
    }
    finish_surfaces_excluding(mesh, map, cells, g, world_origin, &excluded)
}
fn finish_surfaces_excluding(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
    excluded: &[bool],
) -> usize {
    if g.width.checked_mul(g.height) != Some(cells.len())
        || cells.is_empty()
        || (!excluded.is_empty() && excluded.len() != cells.len())
    {
        return 0;
    }
    let gym_ground = gym_floor_underlays(mesh, map, cells, g, world_origin, excluded);
    let tatami_floor = traditional_room::tatami_floor_mask(map, cells, g, world_origin, excluded);
    let source_styles: Vec<_> = cells
        .iter()
        .map(|tile| interior_floor(map, &tile.source))
        .collect();
    if source_styles.iter().all(Option::is_none) {
        return 0;
    }
    let original = std::mem::take(&mut mesh.textured.indices);
    let mut finished = 0;
    for quad in original.chunks(6) {
        let candidate = (|| {
            if quad.len() != 6 || quad[0] != quad[3] || quad[2] != quad[4] {
                return None;
            }
            let ids = [
                quad[0] as usize,
                quad[1] as usize,
                quad[2] as usize,
                quad[5] as usize,
            ];
            if ids.iter().any(|&i| {
                i >= mesh.textured.positions.len() || mesh.textured.normals[i] != [0.0, 1.0, 0.0]
            }) {
                return None;
            }
            let points = ids.map(|i| mesh.textured.positions[i]);
            let uvs = ids.map(|i| mesh.textured.uvs[i]);
            let height = points[0][1];
            if points.iter().any(|p| !near(p[1], height)) {
                return None;
            }
            let x0 = points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
            let x1 = points
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max);
            let z0 = points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
            let z1 = points
                .iter()
                .map(|p| p[2])
                .fold(f32::NEG_INFINITY, f32::max);
            if !near(x1 - x0, g.tile_width) || !near(z1 - z0, g.tile_height) {
                return None;
            }
            let column = ((x0 - g.origin_x) / g.tile_width).round() as i32;
            let row = ((z0 - g.origin_z) / g.tile_height).round() as i32;
            if column < 0 || row < 0 || column >= g.width as i32 || row >= g.height as i32 {
                return None;
            }
            if !near(x0, g.origin_x + column as f32 * g.tile_width)
                || !near(z0, g.origin_z + row as f32 * g.tile_height)
            {
                return None;
            }
            let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
            let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
            let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
            let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
            let sc = ((min_u + max_u) * 0.5 * g.width as f32).floor() as usize;
            let sr = ((min_v + max_v) * 0.5 * g.height as f32).floor() as usize;
            if sc >= g.width || sr >= g.height {
                return None;
            }
            let (u0, u1, v0, v1) = g.uv(sc, sr);
            if !near(min_u, u0) || !near(max_u, u1) || !near(min_v, v0) || !near(max_v, v1) {
                return None;
            }
            // Bounds alone also accept a clipped/distorted sample whose other
            // corners still touch the original limits. Require all four actual
            // atlas corners before replacing source artwork with a finish.
            if [[u0, v0], [u0, v1], [u1, v0], [u1, v1]]
                .iter()
                .any(|corner| {
                    !uvs.iter()
                        .any(|uv| near(uv[0], corner[0]) && near(uv[1], corner[1]))
                })
            {
                return None;
            }
            let destination = row as usize * g.width + column as usize;
            let source_index = sr * g.width + sc;
            if traditional_room::floor_map(map)
                && (cells[destination].source.metatile_id == 0x04
                    || cells[source_index].source.metatile_id == 0x04)
                && (destination != source_index || !tatami_floor[destination] || !near(height, 0.0))
            {
                return None;
            }
            let style = source_styles[sr * g.width + sc].or_else(|| {
                // Old grouped fixtures used a wallpaper sample for masking.
                // It is floor backing only under an actually appended model,
                // never permission to paint over an unknown wall/prop cell.
                let source = &cells[sr * g.width + sc].source;
                let modeled = mesh
                    .authored_cells
                    .get(destination)
                    .is_some_and(Option::is_some);
                if !modeled {
                    return None;
                }
                match (source.tileset_id.as_ref(), source.tile_index) {
                    ("players_room", 0x02) | ("players_house", 0x11) => Some(InteriorFloor::Oak),
                    ("traditional_house", 0x11) => Some(InteriorFloor::Tatami),
                    _ => None,
                }
            })?;
            if gym_floor_style(style)
                && !gym_floor_destination(
                    mesh,
                    map,
                    cells,
                    destination,
                    sr * g.width + sc,
                    style,
                    &gym_ground,
                    excluded,
                )
            {
                return None;
            }
            if style == InteriorFloor::LighthouseSlate
                && !lighthouse_floor_destination(mesh, map, cells, destination)
            {
                return None;
            }
            if style == InteriorFloor::LighthouseChecker
                && !lighthouse_chamber_floor_destination(mesh, map, cells, destination)
            {
                return None;
            }
            Some((
                destination,
                [x0, x1, z0, z1],
                height,
                style,
                [column + world_origin[0], row + world_origin[1]],
            ))
        })();
        if let Some((destination, bounds, height, style, world)) = candidate {
            if style == InteriorFloor::LighthouseChecker {
                let edges = lighthouse_chamber_floor_edges(mesh, map, cells, g, destination);
                lighthouse_chamber_floor_cell(&mut mesh.solid, bounds, height, world, edges);
            } else {
                floor_cell(&mut mesh.solid, bounds, height, style, world);
            }
            finished += 1;
        } else {
            mesh.textured.indices.extend_from_slice(quad);
        }
    }
    if finished > 0 {
        compact_surface(&mut mesh.textured);
    }
    append_known_room_backing(mesh, map, cells, g, world_origin);
    finished
}
fn compact_surface(mesh: &mut SurfaceMeshData) {
    let mut rebuilt = SurfaceMeshData::default();
    let mut mapping = vec![u32::MAX; mesh.positions.len()];
    for &old in &mesh.indices {
        let old = old as usize;
        if mapping[old] == u32::MAX {
            mapping[old] = rebuilt.positions.len() as u32;
            rebuilt.positions.push(mesh.positions[old]);
            rebuilt.normals.push(mesh.normals[old]);
            rebuilt.uvs.push(mesh.uvs[old]);
            rebuilt.colors.push(mesh.colors[old]);
        }
        rebuilt.indices.push(mapping[old]);
    }
    *mesh = rebuilt;
}
fn architecture_box(mesh: &mut SurfaceMeshData, b: [f32; 6], color: [f32; 3]) {
    let [x0, x1, y0, y1, z0, z1] = b;
    for (p, n, shade) in [
        (
            [[x0, y1, z0], [x0, y1, z1], [x1, y1, z1], [x1, y1, z0]],
            [0.0, 1.0, 0.0],
            1.0,
        ),
        (
            [[x1, y0, z1], [x1, y1, z1], [x0, y1, z1], [x0, y0, z1]],
            [0.0, 0.0, 1.0],
            0.88,
        ),
        (
            [[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]],
            [0.0, 0.0, -1.0],
            0.74,
        ),
        (
            [[x0, y0, z1], [x0, y1, z1], [x0, y1, z0], [x0, y0, z0]],
            [-1.0, 0.0, 0.0],
            0.80,
        ),
        (
            [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]],
            [1.0, 0.0, 0.0],
            0.91,
        ),
        (
            [[x0, y0, z1], [x0, y0, z0], [x1, y0, z0], [x1, y0, z1]],
            [0.0, -1.0, 0.0],
            0.60,
        ),
    ] {
        append_quad(
            mesh,
            p,
            n,
            [[0.0, 0.0]; 4],
            [color[0] * shade, color[1] * shade, color[2] * shade, 1.0],
        );
    }
}
fn bedroom_course(cells: &[&VisualTile], g: &GridGeometry) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    if g.width < 16 || g.height < 4 {
        return out;
    }
    for row in 0..=g.height - 4 {
        for column in 0..=g.width - 16 {
            let complete = (0..4).all(|y| {
                (0..16).all(|x| {
                    let s = &cells[(row + y) * g.width + column + x].source;
                    s.tileset_id.as_ref() == "players_room"
                        && s.subtile_column as usize == x % 4
                        && s.subtile_row as usize == y
                        && match x / 4 {
                            0 => matches!(
                                s.metatile_id,
                                0x04 | 0x08 | 0x0b | 0x0e | 0x11 | 0x14 | 0x17
                            ),
                            1 => s.metatile_id == 0x01,
                            2 => s.metatile_id == 0x03,
                            3 => matches!(s.metatile_id, 0x02 | 0x1f | 0x23 | 0x24 | 0x25),
                            _ => false,
                        }
                })
            });
            if complete {
                out.push((column, row, 16));
            }
        }
    }
    out
}
fn append_known_room_backing(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    world_origin: [i32; 2],
) {
    // Existing complete native wall-course matchers are the room blueprint.
    // This makes no wall from collision, an arbitrary flat cell or a viewport edge.
    let mut courses: Vec<_> = ordinary_house::course_origins(cells, g)
        .into_iter()
        .map(|(x, y, w)| (x, y, w, ModelKind::WallDomestic))
        .collect();
    courses.extend(
        players_house::course_origins(cells, g)
            .into_iter()
            .map(|(x, y, w)| (x, y, w, ModelKind::WallDomestic)),
    );
    courses.extend(
        traditional_house::course_origins(cells, g)
            .into_iter()
            .map(|(x, y, w, _)| (x, y, w, ModelKind::WallTraditional)),
    );
    courses.extend(
        bedroom_course(cells, g)
            .into_iter()
            .map(|(x, y, w)| (x, y, w, ModelKind::WallDomestic)),
    );
    for (column, row, width, kind) in courses {
        let tw = g.tile_width;
        let th = g.tile_height;
        let x0 = g.origin_x + column as f32 * tw;
        let x1 = x0 + width as f32 * tw;
        let front = g.origin_z + (row as f32 + 1.80) * th;
        let height = 4.0 * th;
        let mut opening = vec![false; width];
        for x in 0..width {
            opening[x] = (0..4).any(|y| {
                house_stair_local(map, &cells[(row + y) * g.width + column + x].source).is_some()
            });
        }
        // Group matching pairs, preserving the actual stair opening. Existing
        // complete window, shelf and picture models stand in front of this wall.
        let mut x = 0;
        while x < width {
            if opening[x] {
                x += 1;
                continue;
            }
            let w = if x + 1 < width && !opening[x + 1] {
                2
            } else {
                1
            };
            model(kind).append_fitted(
                &mut mesh.solid,
                [
                    x0 + x as f32 * tw,
                    x0 + (x + w) as f32 * tw,
                    front - th * 0.16,
                    front,
                ],
                0.0,
                height,
            );
            x += w;
        }
        // One uninterrupted lintel/cornice connects the panel kit visually.
        architecture_box(
            &mut mesh.solid,
            [
                x0,
                x1,
                height - th * 0.06,
                height + th * 0.10,
                front - th * 0.21,
                front + th * 0.22,
            ],
            [0.45, 0.34, 0.24],
        );
        architecture_box(
            &mut mesh.solid,
            [
                x0,
                x1,
                height + th * 0.10,
                height + th * 0.17,
                front - th * 0.20,
                front + th * 0.20,
            ],
            [0.62, 0.49, 0.33],
        );
        // A shallow below-floor cutaway section is safe only for the complete
        // bedroom blueprint, never for a cropped viewport or a guessed room.
        if map == "PlayersHouse2F"
            && column == 0
            && row == 0
            && width == 16
            && g.width == 16
            && g.height == 12
            && world_origin == [0, 0]
        {
            let north = g.origin_z + 2.0 * th;
            let south = g.origin_z + 12.0 * th;
            let color = [0.35, 0.28, 0.22];
            let depth = th * 0.38;
            for (a, b, c, d) in [
                (x0, x0 + tw * 0.045, north, south),
                (x1 - tw * 0.045, x1, north, south),
                (x0, x1, south - th * 0.045, south),
            ] {
                architecture_box(&mut mesh.solid, [a, b, -depth, -0.001, c, d], color);
            }
        }
    }
}

#[cfg(test)]
mod surface_finish_tests {
    use super::*;
    use std::sync::Arc;
    fn floor() -> (Vec<VisualTile>, GridGeometry) {
        (
            vec![VisualTile {
                column: 0,
                row: 0,
                texture: Handle::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("players_room"),
                    metatile_id: 0x05,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: 0x01,
                },
            }],
            GridGeometry {
                width: 1,
                height: 1,
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: 0.0,
                origin_z: 0.0,
            },
        )
    }
    #[test]
    fn authored_floor_keeps_height_footing_and_object_evidence_separate() {
        let (tiles, g) = floor();
        let refs: Vec<_> = tiles.iter().collect();
        let mut mesh = TerrainMeshData::default();
        append_top(&mut mesh.textured, [0.0, 8.0, 0.0, 8.0], 3.5, g.uv(0, 0));
        mesh.footing_heights = vec![3.5];
        mesh.authored_cells = vec![None];
        assert_eq!(
            finish_surfaces(&mut mesh, "PlayersHouse2F", &refs, &g, [4, 6]),
            1
        );
        assert!(mesh.textured.indices.is_empty());
        assert_eq!(mesh.footing_heights, vec![3.5]);
        assert_eq!(mesh.authored_cells, vec![None]);
        assert!(mesh.solid.positions.iter().all(|p| near(p[1], 3.5)));
    }
    #[test]
    fn unknown_prop_strip_and_upright_art_are_not_floor_materials() {
        let (mut tiles, g) = floor();
        tiles[0].source.tile_index = 0x30;
        let refs: Vec<_> = tiles.iter().collect();
        let mut mesh = TerrainMeshData::default();
        append_top(&mut mesh.textured, [0.0, 8.0, 0.0, 8.0], 0.0, g.uv(0, 0));
        let before = mesh.textured.clone();
        assert_eq!(
            finish_surfaces(&mut mesh, "PlayersHouse2F", &refs, &g, [0, 0]),
            0
        );
        assert_eq!(mesh.textured, before);
    }
    #[test]
    fn plank_material_world_phase_is_stable_across_viewport_origins() {
        let mut a = SurfaceMeshData::default();
        let mut b = SurfaceMeshData::default();
        floor_cell(
            &mut a,
            [0.0, 8.0, 0.0, 8.0],
            0.0,
            InteriorFloor::Oak,
            [7, 9],
        );
        floor_cell(
            &mut b,
            [32.0, 40.0, 48.0, 56.0],
            0.0,
            InteriorFloor::Oak,
            [7, 9],
        );
        assert_eq!(a.colors, b.colors);
        assert_eq!(a.indices, b.indices);
        assert_eq!(a.positions.len(), b.positions.len());
        for (a, b) in a.positions.iter().zip(&b.positions) {
            assert!(near(a[0] + 32.0, b[0]) && near(a[2] + 48.0, b[2]));
        }
    }
}

include!("lighthouse_floor.rs");

include!("gym_floor.rs");
