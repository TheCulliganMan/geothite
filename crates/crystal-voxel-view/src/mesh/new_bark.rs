//! Original modeled scenery for a source-verified connected Johto slice.
//!
//! The pack remains authoritative: source cells place scenery, the original
//! ground/footing stays in place, and this module never reads or edits collision.
use super::*;
use crate::new_bark_models::{ModelKind, model};

const MAP: &str = "NewBarkTown";
pub(super) use crate::new_bark_models::supports_map;

const TRADITIONAL_DOOR_TILES: [[u16; 2]; 2] = [[0x27, 0x28], [0x29, 0x2a]];
const DOOR_TILES: [[u16; 2]; 2] = [[0x37, 0x38], [0x39, 0x3a]];

#[derive(Clone, Copy, Debug, PartialEq)]
struct BuildingModel {
    kind: ModelKind,
    /// Threshold center in native 8px source-cell units, measured from west.
    door_column: usize,
}

fn matches_drawing(
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    placement: BuildingPlacement,
    blocks: &[&[u16]],
) -> bool {
    matches_drawing_offset(cells, geometry, placement, blocks, 0)
}

/// Traditional roofs begin below a two-cell strip of background tree crowns.
/// Validate those cells' original metatile phase while leaving them unclaimed.
fn matches_drawing_offset(
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    placement: BuildingPlacement,
    blocks: &[&[u16]],
    skip_top: usize,
) -> bool {
    placement.width == blocks[0].len() * 4
        && placement.height + skip_top == blocks.len() * 4
        && placement.column + placement.width <= geometry.width
        && placement.row + placement.height <= geometry.height
        && (0..placement.height).all(|y| {
            (0..placement.width).all(|x| {
                let source =
                    &cells[(placement.row + y) * geometry.width + placement.column + x].source;
                let sy = y + skip_top;
                source.tileset_id.as_ref() == "johto"
                    && source.metatile_id == blocks[sy / 4][x / 4]
                    && usize::from(source.subtile_column) == x % 4
                    && usize::from(source.subtile_row) == sy % 4
            })
        })
}

fn building_model(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    placement: BuildingPlacement,
) -> Option<BuildingModel> {
    if !supports_map(map) {
        return None;
    }
    // Exact map + complete native drawing + doorway vocabulary. Roof IDs alone
    // never replace a prop, cropped building, stair platform or unrelated map.
    let (kind, door_column, traditional) = if map == MAP
        && (matches_drawing(
            cells,
            geometry,
            placement,
            &[&[0x18, 0x1f, 0x19], &[0x1c, 0x77, 0x1e]],
        ) || matches_drawing(
            cells,
            geometry,
            placement,
            &[&[0x18, 0x1f, 0x19], &[0x1c, 0x1d, 0x1e]],
        )) {
        (ModelKind::Lab, 5, false)
    } else if map == MAP
        && matches_drawing(cells, geometry, placement, &[&[0x18, 0x19], &[0x16, 0x1e]])
    {
        (ModelKind::PlayerHouse, 3, false)
    } else if matches!(
        map,
        "NewBarkTown" | "CherrygroveCity" | "Route30" | "VioletCity"
    ) && matches_drawing(cells, geometry, placement, &[&[0x14, 0x15]])
    {
        (ModelKind::House, 3, false)
    } else if matches!(map, "CherrygroveCity" | "VioletCity")
        && matches_drawing(cells, geometry, placement, &[&[0x18, 0x19], &[0x1a, 0x17]])
    {
        (ModelKind::Mart, 3, false)
    } else if matches!(map, "CherrygroveCity" | "VioletCity")
        && matches_drawing(cells, geometry, placement, &[&[0x18, 0x19], &[0x1a, 0x1b]])
    {
        (ModelKind::Pokecenter, 3, false)
    } else if map == "Route29" && matches_drawing(cells, geometry, placement, &[&[0x1a, 0x11]]) {
        // This is the complete native border-gate drawing, not a cropped roof.
        (ModelKind::RouteGate, 3, false)
    } else if map == "VioletCity"
        && matches_drawing(
            cells,
            geometry,
            placement,
            &[&[0x18, 0x1f, 0x19], &[0x1c, 0x1d, 0x1e]],
        )
    {
        (ModelKind::VioletGym, 5, false)
    } else if map == "VioletCity"
        && matches_drawing_offset(
            cells,
            geometry,
            placement,
            &[&[0x2c, 0x2d], &[0x2e, 0x2f]],
            2,
        )
    {
        (ModelKind::TraditionalHouse, 3, true)
    } else if map == "VioletCity"
        && matches_drawing_offset(
            cells,
            geometry,
            placement,
            &[&[0x2c, 0x2a, 0x2d], &[0x26, 0x27, 0x2f]],
            2,
        )
    {
        (ModelKind::TraditionalHouse, 5, true)
    } else if map == "VioletCity"
        && matches_drawing_offset(
            cells,
            geometry,
            placement,
            &[&[0x2c, 0x2d], &[0x22, 0x23], &[0x28, 0x29]],
            2,
        )
    {
        (ModelKind::SproutTower, 3, true)
    } else {
        return None;
    };
    let art = if traditional {
        TRADITIONAL_DOOR_TILES
    } else {
        DOOR_TILES
    };
    let doorway_matches = (0..2).all(|y| {
        (0..2).all(|x| {
            cells[(placement.row + placement.height - 2 + y) * geometry.width
                + placement.column
                + door_column
                - 1
                + x]
                .source
                .tile_index
                == art[y][x]
        })
    });
    doorway_matches.then_some(BuildingModel { kind, door_column })
}

/// Add only source-complete drawings absent from the generic roof catalog.
/// Sprout's platform stays separate; Route31's east-facing gate keeps its
/// faithful existing renderer until an east-entry model is separately verified.
pub(super) fn building_placements(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
) -> Vec<BuildingPlacement> {
    let mut placements = outdoor_building_placements(cells, geometry);
    let extra = match map {
        "Route29" => Some((0x1a, 0, 8, 4)),
        "VioletCity" => Some((0x2c, 2, 8, 10)),
        _ => None,
    };
    if let Some((first, skip, width, height)) = extra {
        for tile in cells {
            if tile.source.tileset_id.as_ref() != "johto"
                || tile.source.metatile_id != first
                || tile.source.subtile_column != 0
                || usize::from(tile.source.subtile_row) != skip
            {
                continue;
            }
            let placement = BuildingPlacement {
                column: tile.column as usize,
                row: tile.row as usize,
                width,
                height,
                roof_rows: height - 2,
                ground_tile_index: 0x06,
            };
            if building_model(map, cells, geometry, placement).is_some() {
                placements.push(placement);
            }
        }
    }
    if map == "Route29" {
        let mut complete = Vec::new();
        for tile in cells {
            if tile.source.tileset_id.as_ref() != "johto"
                || tile.source.metatile_id != 0x08
                || tile.source.subtile_column != 0
                || tile.source.subtile_row != 0
            {
                continue;
            }
            let p = BuildingPlacement {
                column: tile.column as usize,
                row: tile.row as usize,
                width: 8,
                height: 8,
                roof_rows: 4,
                ground_tile_index: 0x06,
            };
            if building_model(map, cells, geometry, p).is_some() {
                complete.push(p);
            }
        }
        placements.retain(|p| {
            !complete
                .iter()
                .any(|whole| p.column == whole.column && p.row == whole.row + 4 && p.height == 4)
        });
        placements.extend(complete);
    }
    placements.sort_by_key(|p| (p.row, p.column));
    placements.dedup();
    placements
}

fn sprout_platform(cells: &[&VisualTile], geometry: &GridGeometry, p: BuildingPlacement) -> bool {
    const ART: [[u16; 8]; 4] = [
        [0x3b, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x3d],
        [0x4b, 0x4c, 0x9a, 0x9a, 0x4c, 0x4c, 0x4c, 0x4d],
        [0x06; 8],
        [0x06; 8],
    ];
    matches_drawing(cells, geometry, p, &[&[0x74, 0x75]])
        && (0..4).all(|y| {
            (0..8).all(|x| {
                cells[(p.row + y) * geometry.width + p.column + x]
                    .source
                    .tile_index
                    == ART[y][x]
            })
        })
}

fn append_sprout_platform(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    p: BuildingPlacement,
    claimed: &mut [bool],
) -> bool {
    let Some(ground) = authored_ground_cell(cells, shapes, 0x06) else {
        return false;
    };
    // The platform's native walking support is zero. Its old generic building
    // fold was an elevated roof, which hid an actor's legs. Keep the original
    // datum, full path rows and source-footing buffer completely unchanged.
    for y in 0..4 {
        for x in 0..8 {
            claimed[(p.row + y) * g.width + p.column + x] = true;
        }
    }
    append_ground(
        mesh,
        cells,
        shapes,
        g,
        p.column,
        p.row + 2,
        8,
        2,
        ground,
        claimed,
    );
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let unit = g.tile_height;
    let e = w + 8.0 * g.tile_width;
    let south = n + 2.0 * unit;
    // Sixteen quiet transverse planks; their tops, including the entrance,
    // sit exactly on the unchanged walking datum rather than above the actor.
    for row in 0..8 {
        let z0 = n + row as f32 * unit * 0.25;
        let z1 = z0 + unit * 0.25;
        sign_box(
            &mut mesh.solid,
            [w, e, -unit * 0.18, 0.0, z0, z1],
            if row % 3 == 0 {
                [0.49, 0.37, 0.22, 1.0]
            } else {
                [0.54, 0.42, 0.27, 1.0]
            },
        );
        if row > 0 {
            sign_box(
                &mut mesh.solid,
                [w, e, 0.001, 0.004, z0 - unit * 0.008, z0 + unit * 0.008],
                [0.37, 0.29, 0.19, 1.0],
            );
        }
    }
    let open_w = w + 2.0 * g.tile_width;
    let open_e = w + 4.0 * g.tile_width;
    // Inlaid threshold matches the native 9a/9a stair opening. It is flush.
    for row in 0..3 {
        let z0 = south - unit * 0.65 + row as f32 * unit * 0.21;
        sign_box(
            &mut mesh.solid,
            [open_w, open_e, 0.004, 0.008, z0, z0 + unit * 0.18],
            [0.61, 0.61, 0.48, 1.0],
        );
    }
    // Low cream piers and brown rails only on the source's outer edge bands.
    // The exact two-cell central opening remains unobstructed.
    for x in [w + unit * 0.20, e - unit * 0.20] {
        for z in [n + unit * 0.18, south - unit * 0.18] {
            sign_box(
                &mut mesh.solid,
                [
                    x - unit * 0.12,
                    x + unit * 0.12,
                    -unit * 0.20,
                    unit * 0.68,
                    z - unit * 0.12,
                    z + unit * 0.12,
                ],
                [0.67, 0.66, 0.52, 1.0],
            );
            sign_box(
                &mut mesh.solid,
                [
                    x - unit * 0.16,
                    x + unit * 0.16,
                    unit * 0.65,
                    unit * 0.73,
                    z - unit * 0.16,
                    z + unit * 0.16,
                ],
                [0.78, 0.76, 0.61, 1.0],
            );
        }
        for y in [unit * 0.29, unit * 0.56] {
            sign_box(
                &mut mesh.solid,
                [
                    x - unit * 0.055,
                    x + unit * 0.055,
                    y,
                    y + unit * 0.08,
                    n + unit * 0.18,
                    south - unit * 0.18,
                ],
                [0.32, 0.24, 0.16, 1.0],
            );
        }
    }
    for (a, b) in [
        (w + unit * 0.22, open_w - unit * 0.10),
        (open_e + unit * 0.10, e - unit * 0.22),
    ] {
        for y in [unit * 0.29, unit * 0.56] {
            sign_box(
                &mut mesh.solid,
                [
                    a,
                    b,
                    y,
                    y + unit * 0.08,
                    south - unit * 0.22,
                    south - unit * 0.11,
                ],
                [0.32, 0.24, 0.16, 1.0],
            );
        }
        let x = if a < w + unit { b } else { a };
        sign_box(
            &mut mesh.solid,
            [
                x - unit * 0.10,
                x + unit * 0.10,
                -unit * 0.15,
                unit * 0.65,
                south - unit * 0.27,
                south - unit * 0.07,
            ],
            [0.67, 0.66, 0.52, 1.0],
        );
    }
    true
}

pub(super) fn append_building(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    map: &str,
    placement: BuildingPlacement,
    claimed: &mut [bool],
) -> bool {
    if map == "VioletCity" && sprout_platform(cells, geometry, placement) {
        let appended = append_sprout_platform(mesh, cells, shapes, geometry, placement, claimed);
        if appended {
            mark_authored_rect(
                mesh,
                geometry,
                [
                    placement.column,
                    placement.row,
                    placement.width,
                    placement.height,
                ],
                "johto/sprout_forecourt",
            );
        }
        return appended;
    }
    let Some(descriptor) = building_model(map, cells, geometry, placement) else {
        return false;
    };
    let Some(ground) = authored_ground_cell(cells, shapes, placement.ground_tile_index) else {
        return false;
    };
    append_ground(
        mesh,
        cells,
        shapes,
        geometry,
        placement.column,
        placement.row,
        placement.width,
        placement.height,
        ground,
        claimed,
    );
    let (west, _, north, _) = geometry.bounds(placement.column, placement.row);
    let east = west + placement.width as f32 * geometry.tile_width;
    let south = north + placement.height as f32 * geometry.tile_height;
    let cutaway_start = mesh.solid.positions.len();
    model(descriptor.kind).append_fitted(
        &mut mesh.solid,
        [west, east, north, south],
        0.0,
        geometry.tile_height * 2.0,
        Some(west + descriptor.door_column as f32 * geometry.tile_width),
    );
    // Mark only the successfully appended building. The backing was emitted
    // above; later terrain, trees and props must keep their ordinary depth.
    mesh.solid
        .cutaway_ranges
        .push(cutaway_start..mesh.solid.positions.len());
    mark_authored_rect(
        mesh,
        geometry,
        [
            placement.column,
            placement.row,
            placement.width,
            placement.height,
        ],
        descriptor.kind.label(),
    );
    true
}

#[derive(Clone, Copy)]
struct SignPlacement {
    column: usize,
    row: usize,
    ground: usize,
}

fn sign_placements(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
) -> Vec<SignPlacement> {
    let mut placements = Vec::new();
    for tile in cells {
        let source = &tile.source;
        let (origin, ground) = match source.metatile_id {
            0x45 => (0, 0x06),
            0x47 => (2, 0x05),
            0x78 => (2, 0x06),
            _ => continue,
        };
        if source.tileset_id.as_ref() != "johto"
            || source.subtile_column != origin
            || source.subtile_row != origin
        {
            continue;
        }
        let (column, row) = (tile.column as usize, tile.row as usize);
        if column + 1 >= geometry.width || row + 1 >= geometry.height {
            continue;
        }
        let art = [[0x4e, 0x4f], [0x5e, 0x5f]];
        if !(0..2).all(|y| {
            (0..2).all(|x| {
                let s = &cells[(row + y) * geometry.width + column + x].source;
                s.tileset_id.as_ref() == "johto"
                    && s.metatile_id == source.metatile_id
                    && s.subtile_column == origin + x as u8
                    && s.subtile_row == origin + y as u8
                    && s.tile_index == art[y][x]
            })
        }) {
            continue;
        }
        if let Some(ground) = authored_ground_cell(cells, shapes, ground) {
            placements.push(SignPlacement {
                column,
                row,
                ground,
            });
        }
    }
    placements
}

pub(super) fn append_signs(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    claimed: &mut [bool],
) {
    if !supports_map(map) {
        return;
    }
    for placement in sign_placements(cells, shapes, geometry) {
        if (0..2).any(|y| {
            (0..2).any(|x| claimed[(placement.row + y) * geometry.width + placement.column + x])
        }) {
            continue;
        }
        append_ground(
            mesh,
            cells,
            shapes,
            geometry,
            placement.column,
            placement.row,
            2,
            2,
            placement.ground,
            claimed,
        );
        let (w, _, n, _) = geometry.bounds(placement.column, placement.row);
        let e = w + 2.0 * geometry.tile_width;
        let south = n + 2.0 * geometry.tile_height;
        let unit = geometry.tile_height;
        let cx = (w + e) * 0.5;
        let z = south - unit * 0.22;
        // Narrow post and inset cream board stay entirely within the exact
        // source 2x2 sign plot. Text/interactions remain owned by the game.
        sign_box(
            &mut mesh.solid,
            [
                cx - unit * 0.12,
                cx + unit * 0.12,
                0.0,
                unit * 1.25,
                z - unit * 0.11,
                z + unit * 0.11,
            ],
            [0.34, 0.25, 0.16, 1.0],
        );
        sign_box(
            &mut mesh.solid,
            [
                w + unit * 0.10,
                e - unit * 0.10,
                unit * 0.72,
                unit * 1.64,
                z - unit * 0.13,
                z + unit * 0.13,
            ],
            [0.48, 0.35, 0.22, 1.0],
        );
        sign_box(
            &mut mesh.solid,
            [
                w + unit * 0.24,
                e - unit * 0.24,
                unit * 0.84,
                unit * 1.52,
                z + unit * 0.131,
                z + unit * 0.15,
            ],
            [0.80, 0.75, 0.59, 1.0],
        );
        for (dy, length) in [(1.30, 0.88), (1.13, 0.67), (0.98, 0.76)] {
            sign_box(
                &mut mesh.solid,
                [
                    cx - unit * length * 0.5,
                    cx + unit * length * 0.5,
                    unit * dy,
                    unit * (dy + 0.034),
                    z + unit * 0.151,
                    z + unit * 0.153,
                ],
                [0.44, 0.46, 0.34, 1.0],
            );
        }
        mark_authored_rect(
            mesh,
            geometry,
            [placement.column, placement.row, 2, 2],
            "johto/signboard",
        );
    }
}

fn sign_box(mesh: &mut SurfaceMeshData, bounds: [f32; 6], color: [f32; 4]) {
    let [w, e, low, high, n, s] = bounds;
    for (p, normal, shade) in [
        (
            [[w, high, n], [w, high, s], [e, high, s], [e, high, n]],
            [0.0, 1.0, 0.0],
            1.0,
        ),
        (
            [[w, low, n], [e, low, n], [e, low, s], [w, low, s]],
            [0.0, -1.0, 0.0],
            0.65,
        ),
        (
            [[w, low, n], [w, high, n], [e, high, n], [e, low, n]],
            [0.0, 0.0, -1.0],
            0.72,
        ),
        (
            [[e, low, s], [e, high, s], [w, high, s], [w, low, s]],
            [0.0, 0.0, 1.0],
            0.91,
        ),
        (
            [[w, low, s], [w, high, s], [w, high, n], [w, low, n]],
            [-1.0, 0.0, 0.0],
            0.80,
        ),
        (
            [[e, low, n], [e, high, n], [e, high, s], [e, low, s]],
            [1.0, 0.0, 0.0],
            0.75,
        ),
    ] {
        append_solid_quad(
            mesh,
            p,
            normal,
            [color[0] * shade, color[1] * shade, color[2] * shade, 1.0],
        );
    }
}

fn mixed_tree_origins(metatile: u16) -> &'static [(u8, u8)] {
    match metatile {
        0x5b => &[(0, 0)],
        0x5c => &[(0, 0), (2, 0), (0, 2)],
        0x5d => &[(0, 0), (2, 0)],
        0x5e => &[(0, 0), (2, 0), (2, 2)],
        0x5f => &[(2, 0)],
        0x60 => &[(0, 0), (0, 2)],
        0x61 => &[(0, 0), (2, 0), (0, 2), (2, 2)],
        0x62 => &[(2, 0), (2, 2)],
        0x63 => &[(2, 2)],
        0x64 => &[(0, 0), (0, 2), (2, 2)],
        0x65 => &[(0, 2), (2, 2)],
        0x66 => &[(2, 0), (0, 2), (2, 2)],
        0x67 => &[(0, 2)],
        _ => &[],
    }
}

/// Complete small-tree drawings embedded in mixed route blocks. These blocks
/// also contain lawn and Cut trees; an ID or collision class alone is not
/// sufficient. Every accepted crown has the exact 2x2 art and source phase.
fn complete_mixed_tree(
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    placement: TreePlacement,
) -> bool {
    if placement.width != 2
        || placement.height != 2
        || placement.column + 2 > geometry.width
        || placement.row + 2 > geometry.height
    {
        return false;
    }
    let first = &cells[placement.row * geometry.width + placement.column].source;
    if first.tileset_id.as_ref() != "johto"
        || !mixed_tree_origins(first.metatile_id)
            .contains(&(first.subtile_column, first.subtile_row))
    {
        return false;
    }
    let art = [[0x1e, 0x1f], [0x3e, 0x3f]];
    (0..2).all(|y| {
        (0..2).all(|x| {
            let source = &cells[(placement.row + y) * geometry.width + placement.column + x].source;
            source.tileset_id == first.tileset_id
                && source.metatile_id == first.metatile_id
                && source.subtile_column == first.subtile_column + x as u8
                && source.subtile_row == first.subtile_row + y as u8
                && source.tile_index == art[y][x]
        })
    })
}

pub(super) fn tree_placements(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
) -> Vec<TreePlacement> {
    let mut result = complete_tree_placements(cells, geometry);
    if !supports_map(map) {
        return result;
    }
    for tile in cells {
        let placement = TreePlacement {
            column: tile.column as usize,
            row: tile.row as usize,
            width: 2,
            height: 2,
            ground_tile_index: 0x05,
            ground_metatile_id: None,
            base_height: 0.0,
            rounded: true,
            outline_mask: true,
            remove_all_ground: false,
            card_thickness: 0.0,
        };
        if complete_mixed_tree(cells, geometry, placement)
            && !result.iter().any(|p| {
                p.column == placement.column
                    && p.row == placement.row
                    && p.width == 2
                    && p.height == 2
            })
        {
            result.push(placement);
        }
    }
    result.sort_by_key(|p| (p.row, p.column));
    result
}

fn modeled_tree_ground(
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    placement: TreePlacement,
) -> Option<usize> {
    if !supports_map(map)
        || placement.width != 2
        || !matches!(placement.height, 2 | 4)
        || !(0..placement.height).all(|y| {
            (0..placement.width).all(|x| {
                let index = (placement.row + y) * geometry.width + placement.column + x;
                let source = &cells[index].source;
                let expected = if y == 0 {
                    0x1e
                } else if y + 1 == placement.height {
                    0x3e
                } else {
                    0x2e
                } + x as u16;
                source.tileset_id.as_ref() == "johto"
                    && source.tile_index == expected
                    && (shape_for_source(source).solid_kind() == SolidKind::Tree
                        || complete_mixed_tree(cells, geometry, placement))
            })
        })
    {
        return None;
    }
    authored_ground_cell(cells, shapes, placement.ground_tile_index)
}

/// Reserve complete source trees and signboards that can actually be replaced.
/// Otherwise preserve the existing profile, including clipped views with no ground sample.
pub(super) fn preferred_scenery_cells(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
) -> Vec<bool> {
    let mut reserved = vec![false; cells.len()];
    if !supports_map(map) {
        return reserved;
    }
    let shapes: Vec<_> = cells
        .iter()
        .map(|tile| shape_for_source(&tile.source))
        .collect();
    for placement in tree_placements(map, cells, geometry) {
        if modeled_tree_ground(map, cells, &shapes, geometry, placement).is_none() {
            continue;
        }
        for y in 0..placement.height {
            for x in 0..placement.width {
                reserved[(placement.row + y) * geometry.width + placement.column + x] = true;
            }
        }
    }
    for placement in sign_placements(cells, &shapes, geometry) {
        for y in 0..2 {
            for x in 0..2 {
                reserved[(placement.row + y) * geometry.width + placement.column + x] = true;
            }
        }
    }
    // Route29's legacy live profile contains one pixel tuft per source phase.
    // Complete modeled grass must claim those exact cells before masking.
    if authored_ground_cell(cells, &shapes, 0x05).is_some() {
        for (index, tile) in cells.iter().enumerate() {
            if grass_source(&tile.source) {
                reserved[index] = true;
            }
        }
    }
    reserved
}

/// The source halo is much wider than the playable camera focus. Preserve
/// close foliage detail, use a silhouette-matched authored LOD farther out.
/// Every original object remains, with identical fit bounds and footing.
fn foliage_lod(g: &GridGeometry, x: f32, z: f32, radius_cells: f32) -> bool {
    !crate::new_bark_models::scenery_full_detail()
        && ((x / g.tile_width).powi(2) + (z / g.tile_height).powi(2)) > radius_cells.powi(2)
}

/// Adjacent recognized tree plots allow overlapping crowns inside continuous groves.
pub(super) fn grove_neighbors(cells: &[&VisualTile], g: &GridGeometry, map: &str, p: TreePlacement) -> [bool; 4] {
    let plots = tree_placements(map, cells, g);
    let contains = |x: i32, y: i32| plots.iter().any(|q| x >= q.column as i32 && x < (q.column+q.width) as i32
        && y >= q.row as i32 && y < (q.row+q.height) as i32);
    let x=p.column as i32; let y=p.row as i32;
    [contains(x-1,y+p.height as i32/2),contains(x+p.width as i32,y+p.height as i32/2),
        contains(x+p.width as i32/2,y-1),contains(x+p.width as i32/2,y+p.height as i32)]
}

pub(super) fn append_tree(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    map: &str,
    placement: TreePlacement,
    claimed: &mut [bool],
    map_origin: [i32; 2],
) -> bool {
    let Some(ground) = modeled_tree_ground(map, cells, shapes, geometry, placement) else {
        return false;
    };
    if (0..placement.height).any(|y| {
        (0..placement.width)
            .any(|x| claimed[(placement.row + y) * geometry.width + placement.column + x])
    }) {
        return false;
    }
    append_ground(
        mesh,
        cells,
        shapes,
        geometry,
        placement.column,
        placement.row,
        placement.width,
        placement.height,
        ground,
        claimed,
    );
    let (west, _, north, _) = geometry.bounds(placement.column, placement.row);
    let width = placement.width as f32 * geometry.tile_width;
    let south = north + placement.height as f32 * geometry.tile_height;
    let depth = width.min(placement.height as f32 * geometry.tile_height);
    let kind = if foliage_lod(geometry, west + width * 0.5, south - depth * 0.5, 14.0) {
        ModelKind::TreeLod
    } else { ModelKind::Tree };
    crate::foliage::append_world_tree(mesh, kind, [west, west + width, south - depth, south],
        placement.base_height, geometry.tile_height,
        [map_origin[0] + placement.column as i32, map_origin[1] + placement.row as i32],
        grove_neighbors(cells, geometry, map, placement));
    mark_authored_rect(
        mesh,
        geometry,
        [
            placement.column,
            placement.row,
            placement.width,
            placement.height,
        ],
        "johto/tree",
    );
    true
}

#[allow(clippy::too_many_arguments)]
fn append_ground(
    mesh: &mut TerrainMeshData,
    _cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    column: usize,
    row: usize,
    width: usize,
    height: usize,
    ground: usize,
    claimed: &mut [bool],
) {
    for y in row..row + height {
        for x in column..column + width {
            let (west, east, north, south) = geometry.bounds(x, y);
            append_top(
                &mut mesh.textured,
                [west, east, north, south],
                shapes[ground].surface_height(geometry.tile_height),
                geometry.uv(ground % geometry.width, ground / geometry.width),
            );
            claimed[y * geometry.width + x] = true;
        }
    }
}

pub(super) fn append_flower(
    target: &mut SurfaceMeshData,
    map: &str,
    tile: &VisualTile,
    geometry: &GridGeometry,
    base_height: f32,
) -> bool {
    if !supports_map(map)
        || tile.source.tileset_id.as_ref() != "johto"
        || tile.source.metatile_id != 0x04
        || !matches!(
            (tile.source.subtile_column, tile.source.subtile_row),
            (1, 1) | (3, 1) | (0, 2) | (2, 2)
        )
        || crate::flower::flower_shape(&tile.source).is_none()
    {
        return false;
    }
    let (west, east, north, south) = geometry.bounds(tile.column as usize, tile.row as usize);
    let flowers = model(ModelKind::Flowers);
    flowers.append_fitted(
        target,
        [
            west + geometry.tile_width * 0.08,
            east - geometry.tile_width * 0.08,
            north + geometry.tile_height * 0.08,
            south - geometry.tile_height * 0.08,
        ],
        base_height,
        geometry.tile_height * 0.65 / (flowers.max[1] - flowers.min[1]),
        None,
    );
    true
}

/// Original folded-blade tuft, emitted only for the verified complete Johto
/// grass-cell identity. It changes presentation, never encounter/collision data.
fn grass_source(source: &VisualTileSource) -> bool {
    source.tileset_id.as_ref() == "johto"
        && source.metatile_id == 0x03
        && source.tile_index == 0x04
        && source.subtile_column < 4
        && source.subtile_row < 4
}

pub(super) fn append_grass(
    target: &mut SurfaceMeshData,
    map: &str,
    tile: &VisualTile,
    geometry: &GridGeometry,
    base_height: f32,
    map_origin: [i32; 2],
) -> bool {
    if !supports_map(map) || !grass_source(&tile.source) {
        return false;
    }
    let (w, e, n, s) = geometry.bounds(tile.column as usize, tile.row as usize);
    let seed = lattice(
        map_origin[0] + tile.column as i32,
        map_origin[1] + tile.row as i32,
    );
    let grass = model(
        if foliage_lod(geometry, (w + e) * 0.5, (n + s) * 0.5, 10.0) {
            ModelKind::GrassLod
        } else {
            ModelKind::Grass
        },
    );
    let first = target.positions.len();
    grass.append_fitted(
        target,
        [
            w + geometry.tile_width * 0.08,
            e - geometry.tile_width * 0.08,
            n + geometry.tile_height * 0.08,
            s - geometry.tile_height * 0.08,
        ],
        base_height,
        geometry.tile_height * (0.53 + seed * 0.14) / (grass.max[1] - grass.min[1]),
        None,
    );
    for color in &mut target.colors[first..] {
        for c in &mut color[..3] {
            *c *= 0.92 + seed * 0.13;
        }
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GroundMaterial {
    Lawn,
    Path,
    Water,
    Shore,
    Bank,
    Pavers,
    Stone,
    Ice,
}

/// Only the exposed cells of the two native checker plazas become paving.
/// The southeast quadrant of $79 is a separate prop, including cropped views.
fn kanto_plaza_material(source: &VisualTileSource) -> Option<GroundMaterial> {
    let x = source.subtile_column;
    let y = source.subtile_row;
    if x >= 4 || y >= 4 || (source.metatile_id == 0x79 && x >= 2 && y >= 2) {
        return None;
    }
    let expected = if (x + y) % 2 == 0 { 0x30 } else { 0x39 };
    (matches!(source.metatile_id, 0x79 | 0x7b) && source.tile_index == expected)
        .then_some(GroundMaterial::Pavers)
}

/// A bit is one source cell in a native 4x4 metatile, in row-major order.
/// Masks deliberately omit prop, stair, wall and lettering cells.
fn source_cell_in_mask(source: &VisualTileSource, mask: u16) -> bool {
    source.subtile_column < 4
        && source.subtile_row < 4
        && mask & (1 << (source.subtile_row * 4 + source.subtile_column)) != 0
}

fn ice_path_ground_material(source: &VisualTileSource) -> Option<GroundMaterial> {
    // Each 2x2 pool quadrant has four distinct diagonal-highlight samples.
    // The mixed blocks replace exactly one quadrant with a separate boulder.
    let pool = match source.metatile_id {
        0x1f => 0xffff,
        0x2c => 0xffcc,
        0x2d => 0xccff,
        0x2e => 0xff33,
        0x2f => 0x33ff,
        _ => 0,
    };
    if source_cell_in_mask(source, pool) {
        let expected =
            0xc6 + u16::from(source.subtile_column % 2) + u16::from(source.subtile_row % 2) * 0x10;
        return (source.tile_index == expected).then_some(GroundMaterial::Ice);
    }
    // Exact locations of the four plain ground samples: rock grain $19 and
    // dotted variations $9a/$9b/$aa. Dark rock fragments, ladders, entrances,
    // and the Hall of Fame atlas reuse remain source art.
    let masks: [u16; 4] = match source.metatile_id {
        0x02 => [0x6ed2, 0x812d, 0x0000, 0x1000],
        0x03 => [0x236c, 0x1093, 0x0000, 0x0000],
        0x04 => [0x0000, 0xc4c0, 0x0800, 0x0000],
        0x05 => [0x0000, 0x8df0, 0x4200, 0x3000],
        0x06 => [0x0000, 0x3330, 0x0000, 0x0000],
        0x07 => [0x2370, 0x1001, 0x0084, 0x0008],
        0x08 => [0x0000, 0x4cc4, 0x8008, 0x0000],
        0x09 => [0x8660, 0x5088, 0x0800, 0x2117],
        0x0a => [0x0000, 0x3233, 0x0100, 0x0000],
        0x0c => [0x0000, 0x00c4, 0x0008, 0x0000],
        0x0d => [0x0000, 0x00ff, 0x0000, 0x0000],
        0x0e => [0x0000, 0x0032, 0x0001, 0x0000],
        0x10 => [0x0000, 0x113f, 0x0240, 0x0000],
        0x11 => [0x0000, 0x88cf, 0x0010, 0x4420],
        0x12 => [0x0000, 0x0000, 0x00d2, 0x0004],
        0x13 => [0x0000, 0x0001, 0x0092, 0x0048],
        0x14 => [0x8800, 0x0000, 0x0400, 0x4000],
        0x15 => [0xd300, 0x0c00, 0x0000, 0x2000],
        0x16 => [0x3100, 0x0200, 0x0000, 0x0000],
        0x17 => [0xc421, 0x0846, 0x0000, 0x0098],
        0x18 => [0x8cc4, 0x0008, 0x0000, 0x4000],
        0x1a => [0x2222, 0x0000, 0x0010, 0x0101],
        0x1b => [0x0622, 0xc811, 0x3000, 0x0100],
        0x1c => [0x0080, 0x0040, 0x0000, 0x000c],
        0x1d => [0x0073, 0x0000, 0x008c, 0x0000],
        0x1e => [0x0001, 0x0000, 0x0002, 0x0030],
        0x20 => [0xaec8, 0x0004, 0x0000, 0x5100],
        0x21 => [0xc431, 0x0000, 0x0006, 0x08c8],
        0x22 => [0xb532, 0x0000, 0x0800, 0x4201],
        0x23 => [0x31e6, 0x0000, 0x0201, 0x0018],
        0x24 => [0x0084, 0x0022, 0x0200, 0x0448],
        0x25 => [0x0037, 0x0000, 0x0e00, 0x01c8],
        0x26 => [0x0012, 0x0000, 0x0541, 0x0224],
        0x27 => [0x4eae, 0x0000, 0x0000, 0xa040],
        0x28 => [0x2010, 0x0120, 0x4447, 0x1200],
        0x29 => [0x4fd6, 0x0009, 0x8020, 0x1000],
        0x2a => [0x0020, 0x0000, 0xffc0, 0x0010],
        0x37 => [0x0040, 0x0000, 0x001f, 0x1120],
        0x38 => [0x0220, 0x0008, 0x0005, 0x10d2],
        0x39 => [0x0000, 0x00cc, 0x0000, 0x0000],
        0x3a => [0x0000, 0x0033, 0x0000, 0x0000],
        0x3e => [0x0000, 0x00ef, 0x0010, 0x0000],
        _ => return None,
    };
    let mask = match source.tile_index {
        0x19 => masks[0],
        0x9a => masks[1],
        0x9b => masks[2],
        0xaa => masks[3],
        _ => return None,
    };
    source_cell_in_mask(source, mask).then_some(GroundMaterial::Stone)
}

fn port_water_source(source: &VisualTileSource) -> bool {
    // Water-only cells from the native quay, ship and gym blocks. In
    // particular, the four bollard tiles never become water or pavement.
    let mask = match source.metatile_id {
        0x02 => 0xcccc,
        0x03 => 0x3333,
        0x05 => 0xcc00,
        0x06 => 0xff00,
        0x07 => 0x3300,
        0x08 => 0xfff0,
        0x09 => 0xcccc,
        0x0a => 0xffff,
        0x0b => 0x3333,
        0x0c => 0x00ff,
        0x0d => 0x00cc,
        0x0e => 0x00ff,
        0x0f => 0x0033,
        0x10 => 0x7777,
        0x12 => 0xeeee,
        0x13 => 0x3333,
        0x14 => 0x3330,
        0x15 => 0xfff0,
        0x16 => 0x7770,
        0x17 => 0x0033,
        0x18 => 0x00ff,
        0x19 => 0x0033,
        0x1a => 0x000f,
        0x1b => 0x88ef,
        0x1c => 0xf100,
        0x1d => 0xf000,
        0x1e => 0xf000,
        0x1f => 0xfec8,
        0x20 => 0x3300,
        0x22 => 0xcc00,
        0x23 => 0x3300,
        0x24 => 0xff00,
        0x25 => 0xff00,
        0x27 => 0xf000,
        0x28 => 0xc000,
        0x29 => 0x3000,
        0x2c => 0x00cc,
        0x2d => 0x0033,
        0x2e => 0xcccc,
        0x2f => 0x3333,
        0x30 => 0xff00,
        0x34 => 0xccc0,
        0x36 => 0xfff0,
        0x37 => 0xccc0,
        0x38 => 0x3330,
        0x39 => 0x00f0,
        _ => return false,
    };
    source.tile_index == 0x14 && source_cell_in_mask(source, mask)
}

/// The atlas sample, not an object's plot, identifies the exposed ground under
/// a modeled tree, house, sign or flower. Unknown art stays untouched.
fn ground_material(source: &VisualTileSource) -> Option<GroundMaterial> {
    let tileset = source.tileset_id.as_ref();
    if !matches!(
        tileset,
        "johto"
            | "johto_modern"
            | "kanto"
            | "forest"
            | "park"
            | "battle_tower_outside"
            | "cave"
            | "dark_cave"
            | "ice_path"
            | "port"
    ) {
        return None;
    }
    match shape_for_source(source) {
        CellShape::Flat | CellShape::PlaneAt { height: 0.0 } => {
            match (tileset, source.tile_index) {
                ("kanto", _) if matches!(source.metatile_id, 0x79 | 0x7b) => {
                    kanto_plaza_material(source)
                }
                ("ice_path", _) => ice_path_ground_material(source),
                ("johto" | "johto_modern" | "forest" | "battle_tower_outside", 0x05)
                | ("kanto", 0x2c)
                | ("park", 0x01) => Some(GroundMaterial::Lawn),
                ("johto" | "johto_modern" | "battle_tower_outside", 0x06)
                | ("kanto", 0x0d | 0x39) => Some(GroundMaterial::Path),
                ("johto_modern", 0x2f) if matches!(source.metatile_id, 0x06 | 0x66 | 0x77) => {
                    Some(GroundMaterial::Pavers)
                }
                ("park", 0x00) => Some(GroundMaterial::Pavers),
                ("cave" | "dark_cave", 0x16) => Some(GroundMaterial::Stone),
                _ => None,
            }
        }
        CellShape::Water if tileset == "port" => {
            port_water_source(source).then_some(GroundMaterial::Water)
        }
        CellShape::Water => Some(GroundMaterial::Water),
        CellShape::ShoreBand => Some(GroundMaterial::Shore),
        _ => None,
    }
}

fn cell_at(geometry: &GridGeometry, x: f32, z: f32) -> Option<usize> {
    let column = ((x - geometry.origin_x) / geometry.tile_width).floor();
    let row = ((z - geometry.origin_z) / geometry.tile_height).floor();
    (column >= 0.0 && row >= 0.0 && column < geometry.width as f32 && row < geometry.height as f32)
        .then_some(row as usize * geometry.width + column as usize)
}

fn lattice(x: i32, z: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x8da6_b343) ^ (z as u32).wrapping_mul(0xd816_3841);
    n ^= n >> 13;
    n = n.wrapping_mul(0xcb1a_b31f);
    (n & 0xffff) as f32 / 65535.0
}

/// Continuous, low-amplitude color fields avoid a noisy cell checkerboard.
/// This never displaces vertices or changes the authoritative walking datum.
fn meadow_noise(x: f32, z: f32) -> f32 {
    let (ix, iz) = (x.floor() as i32, z.floor() as i32);
    let (tx, tz) = (x - x.floor(), z - z.floor());
    let (tx, tz) = (tx * tx * (3.0 - 2.0 * tx), tz * tz * (3.0 - 2.0 * tz));
    let a = lattice(ix, iz) * (1.0 - tx) + lattice(ix + 1, iz) * tx;
    let b = lattice(ix, iz + 1) * (1.0 - tx) + lattice(ix + 1, iz + 1) * tx;
    a * (1.0 - tz) + b * tz - 0.5
}

fn ground_color(material: GroundMaterial, p: [f32; 3], geometry: &GridGeometry) -> [f32; 4] {
    let (x, z) = (p[0] / geometry.tile_width, p[2] / geometry.tile_height);
    let broad = meadow_noise(x / 5.0, z / 5.0);
    let fine = meadow_noise(x / 1.7, z / 1.7);
    let (base, variation) = match material {
        GroundMaterial::Lawn => ([0.34, 0.43, 0.22], broad * 0.135 + fine * 0.055),
        GroundMaterial::Path => ([0.66, 0.61, 0.48], broad * 0.060 + fine * 0.035),
        GroundMaterial::Water => ([0.095, 0.30, 0.32], broad * 0.036 + fine * 0.010),
        GroundMaterial::Shore => ([0.45, 0.46, 0.37], broad * 0.032 + fine * 0.024),
        GroundMaterial::Bank => ([0.39, 0.37, 0.29], broad * 0.035 + fine * 0.020),
        GroundMaterial::Pavers => ([0.53, 0.51, 0.45], broad * 0.035 + fine * 0.015),
        GroundMaterial::Stone => ([0.35, 0.36, 0.33], broad * 0.055 + fine * 0.020),
        GroundMaterial::Ice => ([0.38, 0.56, 0.61], broad * 0.040 + fine * 0.015),
    };
    [
        base[0] + variation,
        base[1] + variation,
        base[2] + variation,
        1.0,
    ]
}

fn sampled_cell(uvs: &[[f32; 2]], geometry: &GridGeometry) -> Option<usize> {
    let u = uvs.iter().map(|uv| uv[0]).sum::<f32>() / uvs.len() as f32;
    let v = uvs.iter().map(|uv| uv[1]).sum::<f32>() / uvs.len() as f32;
    if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
        return None;
    }
    Some(
        (v * geometry.height as f32) as usize * geometry.width
            + (u * geometry.width as f32) as usize,
    )
}

fn shallow_bank_source(source: &VisualTileSource) -> bool {
    if source.tileset_id.as_ref() != "johto"
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return false;
    }
    let x = source.subtile_column;
    let y = source.subtile_row;
    let expected = match source.metatile_id {
        0x4b => {
            if y == 3 && x < 2 {
                0x4c
            } else {
                0x05
            }
        }
        0x4e => {
            if x == 0 {
                0x3b
            } else {
                0x05
            }
        }
        0x4f => {
            if x == 3 {
                0x3d
            } else {
                0x05
            }
        }
        0x50 | 0x52 => {
            if y == 3 {
                if x == 0 { 0x4b } else { 0x4c }
            } else if x == 0 {
                0x3b
            } else if source.metatile_id == 0x50 {
                0x06
            } else {
                0x05
            }
        }
        0x51 | 0x53 => {
            if y == 3 {
                if x == 3 { 0x4d } else { 0x4c }
            } else if x == 3 {
                0x3d
            } else if source.metatile_id == 0x51 {
                0x06
            } else {
                0x05
            }
        }
        0x56 => {
            if y == 3 {
                0x4c
            } else {
                0x06
            }
        }
        0x57 => {
            if y == 3 {
                0x4c
            } else {
                0x05
            }
        }
        0x5a => {
            if y == 3 && x < 2 {
                0x4c
            } else {
                0x06
            }
        }
        _ => return false,
    };
    source.tile_index == expected
}

/// Require the complete native $07/$1a/$2f drawing, including $2f's flat
/// eastern half. A clipped or changed block cannot authorize a bank finish.
fn complete_kanto_shallow_bank(index: usize, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    let source = &cells[index].source;
    if source.tileset_id.as_ref() != "kanto"
        || !matches!(source.metatile_id, 0x07 | 0x1a | 0x2f)
        || source.subtile_column >= 4
        || source.subtile_row >= 4
    {
        return false;
    }
    let Some(west) = (index % g.width).checked_sub(usize::from(source.subtile_column)) else {
        return false;
    };
    let Some(north) = (index / g.width).checked_sub(usize::from(source.subtile_row)) else {
        return false;
    };
    west + 4 <= g.width
        && north + 4 <= g.height
        && (0..4).all(|y| {
            (0..4).all(|x| {
                let candidate = &cells[(north + y) * g.width + west + x].source;
                let expected = if y < 3 {
                    if source.metatile_id == 0x1a {
                        0x39
                    } else {
                        0x2c
                    }
                } else if source.metatile_id == 0x2f {
                    [0x37, 0x34, 0x04, 0x04][x]
                } else {
                    0x37
                };
                candidate.tileset_id == source.tileset_id
                    && candidate.metatile_id == source.metatile_id
                    && usize::from(candidate.subtile_column) == x
                    && usize::from(candidate.subtile_row) == y
                    && candidate.tile_index == expected
            })
        })
}

/// Match the native bank mesher's complete cell quads and cropped face UVs.
/// Tops may borrow cap paint across a connected run; exposed sides sample the
/// authored $37/$34 front course (including the east/west fallback). Never
/// recolor a facade, partial/custom quad, deeper cliff, or claimed model cell.
fn kanto_shallow_bank_surface(
    positions: &[[f32; 3]],
    normal: [f32; 3],
    uvs: &[[f32; 2]],
    sample: usize,
    cells: &[&VisualTile],
    g: &GridGeometry,
    authored_cells: &[Option<&str>],
) -> Option<GroundMaterial> {
    if !complete_kanto_shallow_bank(sample, cells, g) {
        return None;
    }
    let center = positions.iter().fold([0.0; 3], |mut sum, p| {
        for a in 0..3 {
            sum[a] += p[a] * 0.25;
        }
        sum
    });
    let world = cell_at(
        g,
        center[0] - normal[0] * g.tile_width * 0.001,
        center[2] - normal[2] * g.tile_height * 0.001,
    )?;
    if !complete_kanto_shallow_bank(world, cells, g)
        || [world, sample]
            .into_iter()
            .any(|i| authored_cells.get(i).is_some_and(Option::is_some))
    {
        return None;
    }
    let height = g.tile_height * crate::profile::JUMP_LEDGE_HEIGHT / SOURCE_TILE_HEIGHT;
    if shape_for_source(&cells[world].source).surface_height(g.tile_height) != height {
        return None;
    }
    let (w, e, n, s) = g.bounds(world % g.width, world / g.width);
    let (u0, u1, v0, v1) = g.uv(sample % g.width, sample / g.width);
    let tile = cells[sample].source.tile_index;
    if normal == [0.0, 1.0, 0.0] {
        // These are append_top_shaded's original vertices and UV order.
        return (positions
            == [
                [w, height, n],
                [w, height, s],
                [e, height, s],
                [e, height, n],
            ]
            && uvs == [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]
            && cells[sample].source.subtile_row < 3)
            .then_some(match tile {
                0x2c => GroundMaterial::Lawn,
                0x39 => GroundMaterial::Path,
                _ => return None,
            });
    }
    if !matches!(tile, 0x37 | 0x34) {
        return None;
    }
    // append_bank_run_side crops the one native course to JUMP_LEDGE_HEIGHT.
    let cv0 = v0 + (v1 - v0) * ((g.tile_height - height) / g.tile_height);
    let cv1 = v0 + (v1 - v0);
    let (expected_positions, expected_uvs) = match normal {
        [0.0, 0.0, 1.0] => (
            [[e, 0.0, s], [e, height, s], [w, height, s], [w, 0.0, s]],
            [[u1, cv1], [u1, cv0], [u0, cv0], [u0, cv1]],
        ),
        [0.0, 0.0, -1.0] => (
            [[w, 0.0, n], [w, height, n], [e, height, n], [e, 0.0, n]],
            [[u0, cv1], [u0, cv0], [u1, cv0], [u1, cv1]],
        ),
        [-1.0, 0.0, 0.0] => (
            [[w, 0.0, s], [w, height, s], [w, height, n], [w, 0.0, n]],
            [[u1, cv1], [u1, cv0], [u0, cv0], [u0, cv1]],
        ),
        [1.0, 0.0, 0.0] => (
            [[e, 0.0, n], [e, height, n], [e, height, s], [e, 0.0, s]],
            [[u0, cv1], [u0, cv0], [u1, cv0], [u1, cv1]],
        ),
        _ => return None,
    };
    (positions == expected_positions && uvs == expected_uvs).then_some(GroundMaterial::Bank)
}

/// A finish on the exact native shallow-bank surfaces, never a new cliff.
/// Explicit source blocks and their paint distinguish these low jumps from
/// doors, tower platforms, deep mountain faces and shoreline reuse.
fn shallow_bank_surface(
    positions: &[[f32; 3]],
    normal: [f32; 3],
    sample: &VisualTileSource,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<GroundMaterial> {
    if sample.tileset_id.as_ref() != "johto" {
        return None;
    }
    let center = positions.iter().fold([0.0; 3], |mut sum, p| {
        for a in 0..3 {
            sum[a] += p[a] * 0.25;
        }
        sum
    });
    let index = cell_at(
        g,
        center[0] - normal[0] * g.tile_width * 0.001,
        center[2] - normal[2] * g.tile_height * 0.001,
    )?;
    let world = &cells[index].source;
    if !shallow_bank_source(world) {
        return None;
    }
    let shallow = matches!(world.metatile_id, 0x4b | 0x50..=0x53 | 0x56 | 0x57 | 0x5a);
    let flat_edge = matches!(world.metatile_id, 0x4e | 0x4f);
    if !shallow && !flat_edge {
        return None;
    }
    let maximum = g.tile_height * crate::profile::JUMP_LEDGE_HEIGHT / SOURCE_TILE_HEIGHT;
    if positions
        .iter()
        .any(|p| p[1] < -0.001 || p[1] > maximum + 0.001)
    {
        return None;
    }
    let shape = shape_for_source(world);
    if normal[1] > 0.01
        && matches!(sample.tile_index, 0x05 | 0x06)
        && matches!(
            shape,
            CellShape::RaisedTop {
                solid: SolidKind::Bank,
                ..
            } | CellShape::LedgeBand { .. }
        )
    {
        return Some(if sample.tile_index == 0x05 {
            GroundMaterial::Lawn
        } else {
            GroundMaterial::Path
        });
    }
    if matches!(sample.tile_index, 0x3b | 0x3d | 0x4b | 0x4c | 0x4d)
        && sample.metatile_id == world.metatile_id
        && shallow_bank_source(sample)
        && (shallow
            || (flat_edge && normal[1] > 0.999 && positions.iter().all(|p| p[1].abs() < 0.001)))
    {
        return Some(GroundMaterial::Bank);
    }
    None
}

fn surface_material(
    map: &str,
    positions: &[[f32; 3]],
    normal: [f32; 3],
    sample: &VisualTileSource,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    authored_cells: &[Option<&str>],
) -> Option<GroundMaterial> {
    if let Some(material) = shallow_bank_surface(positions, normal, sample, cells, geometry) {
        return Some(material);
    }
    let material = ground_material(sample)?;
    let center = positions.iter().fold([0.0; 3], |mut sum, p| {
        for axis in 0..3 {
            sum[axis] += p[axis] * 0.25;
        }
        sum
    });
    // Move a vertical face's probe slightly inside the originating land cell.
    let world = cell_at(
        geometry,
        center[0] - normal[0] * geometry.tile_width * 0.001,
        center[2] - normal[2] * geometry.tile_height * 0.001,
    );
    let water_height = CellShape::Water.surface_height(geometry.tile_height);
    let epsilon = geometry.tile_height * 0.0001;
    if normal[1] > 0.999 {
        let datum = if material == GroundMaterial::Water {
            water_height
        } else {
            0.0
        };
        if positions.iter().any(|p| (p[1] - datum).abs() > epsilon) {
            // Preserve the existing visual-only background apron as well.
            return (world.is_none()
                && matches!(material, GroundMaterial::Lawn | GroundMaterial::Path)
                && positions.iter().all(|p| (p[1] + 0.04).abs() < epsilon))
            .then_some(material);
        }
        let world = world?;
        if cells[world].source.tileset_id != sample.tileset_id {
            return None;
        }
        // Water beds beneath the complete harbor models were authored at
        // this same water datum. Their existing claims prove the underlay;
        // a clipped, unknown or changed prop never gains a claim here.
        let harbor_bed = material == GroundMaterial::Water
            && sample.tileset_id.as_ref() == "port"
            && matches!(
                authored_cells.get(world).copied().flatten(),
                Some("special:harbor-bollard" | "special:dock-railing" | "special:harbor-ferry")
            );
        // The closed Kanto shore-rock adapter proves a complete source drawing
        // and supplies a live same-atlas water sample at the native datum.
        let kanto_rock_bed = material == GroundMaterial::Water
            && sample.tileset_id.as_ref() == "kanto"
            && sample.tile_index == 0x14
            && authored_cells.get(world).copied().flatten()
                == Some("world-exterior/kanto_boundary_shore");
        // Route19's two all-rock blocks are surrounded by native water. The
        // exact binding keeps the land-rock sculpture, but supplies block $43
        // water at the same datum as its neighbors, never a square paving island.
        let route19_rock_bed = map == "Route19"
            && material == GroundMaterial::Water
            && sample.tileset_id.as_ref() == "kanto"
            && sample.metatile_id == 0x43
            && sample.tile_index == 0x14
            && authored_cells.get(world).copied().flatten()
                == Some("world-exterior/kanto_boundary_land")
            && cells[world].source.metatile_id == 0x13
            && cells[world].source.subtile_column < 4
            && cells[world].source.subtile_row < 4
            && cells[world].source.tile_index
                == [[0x2a, 0x2b], [0x3a, 0x3b]]
                    [cells[world].source.subtile_row as usize % 2]
                    [cells[world].source.subtile_column as usize % 2];
        // Other water and shore faces must agree with the actual source cell.
        if matches!(material, GroundMaterial::Water | GroundMaterial::Shore)
            && ground_material(&cells[world].source) != Some(material)
            && !harbor_bed
            && !kanto_rock_bed
            && !route19_rock_bed
        {
            return None;
        }
        Some(material)
    } else if normal[1].abs() < epsilon
        && material == GroundMaterial::Shore
        && world.is_some_and(|i| ground_material(&cells[i].source) == Some(GroundMaterial::Shore))
        && positions
            .iter()
            .all(|p| p[1] <= epsilon && p[1] >= water_height - epsilon)
    {
        Some(GroundMaterial::Shore)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct GroundShadow {
    bounds: [f32; 4],
    tree: bool,
}

impl GroundShadow {
    fn darkness(self, p: [f32; 3], geometry: &GridGeometry) -> f32 {
        let [w, e, n, s] = self.bounds;
        if p[0] <= w || p[0] >= e || p[2] <= n || p[2] >= s {
            return 0.0;
        }
        let cx = (w + e) * 0.5;
        let cz = (n + s) * 0.5;
        if self.tree {
            let x = (p[0] - cx) / ((e - w) * 0.49);
            let z = (p[2] - cz) / ((s - n) * 0.49);
            let r2 = x * x + z * z;
            let canopy = (1.0 - r2).max(0.0).powi(2) * 0.30;
            let trunk = (1.0 - r2 * 18.0).max(0.0) * 0.22;
            canopy + trunk
        } else {
            // Soft AO around the foundation, feathered to zero at the source
            // plot edge. The house's original plot is the hard outer bound.
            let dx = ((p[0] - cx).abs() - (e - w) * 0.38).max(0.0);
            let dz = ((p[2] - cz).abs() - (s - n) * 0.40).max(0.0);
            let distance = (dx * dx + dz * dz).sqrt();
            let edge = ((p[0] - w).min(e - p[0]) / geometry.tile_width)
                .min((p[2] - n).min(s - p[2]) / geometry.tile_height)
                .min(1.0);
            (1.0 - distance / (geometry.tile_width * 0.8)).max(0.0) * 0.25 * edge
        }
    }
}

#[path = "ground_blend.rs"]
mod ground_blend;

struct GroundFinish<'a> {
    forest: bool,
    blend: ground_blend::GroundBlend,
    geometry: &'a GridGeometry,
    map_origin: [i32; 2],
    shadows: Vec<GroundShadow>,
    // Source-cell broad phase makes contact shading O(local occluders), not
    // O(all trees in the published halo) for every ground vertex.
    shadow_cells: Option<Vec<Vec<usize>>>,
}

impl<'a> GroundFinish<'a> {
    fn new(
        map: &str,
        cells: &[&VisualTile],
        geometry: &'a GridGeometry,
        map_origin: [i32; 2],
    ) -> Self {
        let shapes: Vec<_> = cells
            .iter()
            .map(|tile| shape_for_source(&tile.source))
            .collect();
        let mut shadows = Vec::new();
        for placement in tree_placements(map, cells, geometry) {
            if modeled_tree_ground(map, cells, &shapes, geometry, placement).is_none() {
                continue;
            }
            let (west, _, north, _) = geometry.bounds(placement.column, placement.row);
            let width = placement.width as f32 * geometry.tile_width;
            let south = north + placement.height as f32 * geometry.tile_height;
            let depth = width.min(placement.height as f32 * geometry.tile_height);
            shadows.push(GroundShadow {
                bounds: [west, west + width, south - depth, south],
                tree: true,
            });
        }
        for placement in building_placements(map, cells, geometry) {
            if building_model(map, cells, geometry, placement).is_none() {
                continue;
            }
            let (west, _, north, _) = geometry.bounds(placement.column, placement.row);
            shadows.push(GroundShadow {
                bounds: [
                    west,
                    west + placement.width as f32 * geometry.tile_width,
                    north,
                    north + placement.height as f32 * geometry.tile_height,
                ],
                tree: false,
            });
        }
        for placement in sign_placements(cells, &shapes, geometry) {
            let (w, _, n, _) = geometry.bounds(placement.column, placement.row);
            let unit = geometry.tile_height;
            shadows.push(GroundShadow {
                bounds: [
                    w + unit * 0.65,
                    w + unit * 1.35,
                    n + unit * 1.44,
                    n + unit * 1.99,
                ],
                tree: true,
            });
        }
        let mut shadow_cells = vec![Vec::new(); geometry.width * geometry.height];
        for (index, shadow) in shadows.iter().enumerate() {
            let [w, e, n, s] = shadow.bounds;
            let x0 = ((w - geometry.origin_x) / geometry.tile_width)
                .floor()
                .max(0.0) as usize;
            let x1 = ((e - geometry.origin_x) / geometry.tile_width)
                .floor()
                .max(0.0) as usize;
            let y0 = ((n - geometry.origin_z) / geometry.tile_height)
                .floor()
                .max(0.0) as usize;
            let y1 = ((s - geometry.origin_z) / geometry.tile_height)
                .floor()
                .max(0.0) as usize;
            for y in y0..=y1.min(geometry.height.saturating_sub(1)) {
                for x in x0..=x1.min(geometry.width.saturating_sub(1)) {
                    shadow_cells[y * geometry.width + x].push(index);
                }
            }
        }
        let mut materials: Vec<_> = cells.iter().map(|t| ground_material(&t.source)).collect();
        // Modeled trees retain their verified lawn underlay. This also blends
        // the rectangular source plot into a neighboring path.
        for shadow in shadows.iter().filter(|s| s.tree) {
            let [w,e,n,s] = shadow.bounds;
            for row in 0..geometry.height { for col in 0..geometry.width {
                let (x0,x1,z0,z1) = geometry.bounds(col,row);
                if x0>=w && x1<=e && z0>=n && z1<=s {
                    materials[row*geometry.width+col] = Some(GroundMaterial::Lawn);
                }
            } }
        }
        let blend = ground_blend::GroundBlend::new(&materials, geometry);
        Self {
            forest: cells.iter().any(|t| t.source.tileset_id.as_ref() == "forest"),
            blend,
            geometry,
            map_origin,
            shadows,
            shadow_cells: Some(shadow_cells),
        }
    }

    fn darkness(&self, p: [f32; 3]) -> f32 {
        if let Some(index) = &self.shadow_cells {
            return cell_at(self.geometry, p[0], p[2]).map_or(0.0, |cell| {
                index[cell]
                    .iter()
                    .map(|&i| self.shadows[i].darkness(p, self.geometry))
                    .fold(0.0, f32::max)
            });
        }
        self.shadows
            .iter()
            .map(|s| s.darkness(p, self.geometry))
            .fold(0.0, f32::max)
    }

    fn color(&self, material: GroundMaterial, p: [f32; 3]) -> [f32; 4] {
        // Stable map coordinates prevent grain/grass from swimming when the
        // renderer's published grid scrolls by one source cell.
        let map_p = [
            p[0] - self.geometry.origin_x + self.map_origin[0] as f32 * self.geometry.tile_width,
            p[1],
            p[2] - self.geometry.origin_z + self.map_origin[1] as f32 * self.geometry.tile_height,
        ];
        let mut color = ground_color(material, map_p, self.geometry);
        if self.forest && material == GroundMaterial::Lawn {
            let x = map_p[0] / self.geometry.tile_width;
            let z = map_p[2] / self.geometry.tile_height;
            let moss = meadow_noise(x * 0.29, z * 0.29);
            let litter = meadow_noise(x * 1.7 + 19.0, z * 1.7 - 7.0);
            let t = ((moss - 0.35) * 2.2).clamp(0.0, 1.0);
            let soil = [0.25, 0.18, 0.105];
            let green = [0.25, 0.32, 0.16];
            for c in 0..3 { color[c] = (soil[c]*(1.0-t)+green[c]*t) * (0.90+litter*0.18); }
        }
        if matches!(material, GroundMaterial::Lawn | GroundMaterial::Path) {
            if let Some(mixed) = self.blend.color(p, self.geometry, self.map_origin) { color = mixed; }
            let darkness = self.darkness(p);
            for c in &mut color[..3] {
                *c *= 1.0 - darkness;
            }
        }
        color
    }

    fn append_surface(
        &self,
        mesh: &mut SurfaceMeshData,
        material: GroundMaterial,
        positions: [[f32; 3]; 4],
    ) {
        let [w, e, n, s] = face_bounds(&positions);
        let full_cell = (e - w - self.geometry.tile_width).abs() < 0.001
            && (s - n - self.geometry.tile_height).abs() < 0.001;
        if full_cell && material == GroundMaterial::Pavers {
            let column = ((w - self.geometry.origin_x) / self.geometry.tile_width).round() as i32
                + self.map_origin[0];
            let row = ((n - self.geometry.origin_z) / self.geometry.tile_height).round() as i32
                + self.map_origin[1];
            let gap = self.geometry.tile_width * 0.02;
            let inset_w = if column.rem_euclid(2) == 0 { gap } else { 0.0 };
            let inset_n = if row.rem_euclid(2) == 0 { gap } else { 0.0 };
            let mut seam = self.color(material, [(w + e) * 0.5, 0.0, (n + s) * 0.5]);
            for c in &mut seam[..3] {
                *c *= 0.79;
            }
            let rectangles = [
                ([w, w + inset_w, n, s], true),
                ([w + inset_w, e, n, n + inset_n], true),
                ([w + inset_w, e, n + inset_n, s], false),
            ];
            for ([x0, x1, z0, z1], is_seam) in rectangles {
                if x1 <= x0 || z1 <= z0 {
                    continue;
                }
                let p = [[x0, 0.0, z0], [x0, 0.0, z1], [x1, 0.0, z1], [x1, 0.0, z0]];
                append_quad_colors(
                    mesh,
                    p,
                    [0.0, 1.0, 0.0],
                    [[0.0; 2]; 4],
                    if is_seam {
                        [seam; 4]
                    } else {
                        p.map(|p| self.color(material, p))
                    },
                );
            }
            return;
        }
        if !full_cell || !matches!(material, GroundMaterial::Lawn | GroundMaterial::Path) {
            append_quad_colors(
                mesh,
                positions,
                [0.0, 1.0, 0.0],
                [[0.0; 2]; 4],
                positions.map(|p| self.color(material, p)),
            );
            return;
        }
        // Two subdivisions capture a soft contact shadow around a trunk
        // without decals, alpha blending, lifted floors or driver shadows.
        let subdivisions = if self.forest || self.blend.near([(w+e)*0.5,positions[0][1],(n+s)*0.5], self.geometry) { 6 } else { 2 };
        for row in 0..subdivisions {
            for column in 0..subdivisions {
                let fraction = 1.0 / subdivisions as f32;
                let x0 = if column == 0 { w } else { w + (e-w)*column as f32*fraction };
                let x1 = if column+1 == subdivisions { e } else { w + (e-w)*(column+1) as f32*fraction };
                let z0 = if row == 0 { n } else { n + (s-n)*row as f32*fraction };
                let z1 = if row+1 == subdivisions { s } else { n + (s-n)*(row+1) as f32*fraction };
                let y = positions[0][1];
                let p = [[x0, y, z0], [x0, y, z1], [x1, y, z1], [x1, y, z0]];
                append_quad_colors(
                    mesh,
                    p,
                    [0.0, 1.0, 0.0],
                    [[0.0; 2]; 4],
                    p.map(|p| self.color(material, p)),
                );
            }
        }
        self.append_grain(mesh, material, positions);
    }

    fn append_grain(
        &self,
        mesh: &mut SurfaceMeshData,
        material: GroundMaterial,
        positions: [[f32; 3]; 4],
    ) {
        let [w, e, n, s] = face_bounds(&positions);
        let column = ((w - self.geometry.origin_x) / self.geometry.tile_width).round() as i32
            + self.map_origin[0];
        let row = ((n - self.geometry.origin_z) / self.geometry.tile_height).round() as i32
            + self.map_origin[1];
        // Irregular, low-contrast gravel flecks. Lawn gets rarer longer tiny
        // leaves. Uneven position, count, shape and orientation avoid a grid.
        for sample in 0..(if self.forest && material == GroundMaterial::Lawn { 9 } else { 3 }) {
            let seed = lattice(column.wrapping_mul(7) + sample, row.wrapping_mul(13) + 83);
            let threshold = if self.forest && material == GroundMaterial::Lawn { 0.22 } else if material == GroundMaterial::Path {
                0.46
            } else {
                0.90
            };
            if seed < threshold {
                continue;
            }
            let cx = w + (e - w) * (0.16 + lattice(column + sample * 19, row + 109) * 0.68);
            let cz = n + (s - n) * (0.16 + lattice(column + 211, row + sample * 23) * 0.68);
            let angle = lattice(column + 41, row + sample * 31) * std::f32::consts::TAU;
            let rx = (e - w) * (if self.forest && sample % 5 == 0 { 0.12 } else if self.forest { 0.035 + seed * 0.07 } else { 0.018 + seed * 0.036 });
            let rz = (s - n)
                * (if self.forest && sample % 5 == 0 { 0.007 } else if material == GroundMaterial::Path {
                    0.014 + seed * 0.018
                } else {
                    0.012
                });
            let y = positions[0][1] + self.geometry.tile_height * 0.0015;
            let point = |x: f32, z: f32| {
                [
                    cx + angle.cos() * x - angle.sin() * z,
                    y,
                    cz + angle.sin() * x + angle.cos() * z,
                ]
            };
            let p = [
                point(-rx, -rz * 0.3),
                point(-rx * 0.6, rz),
                point(rx, rz * 0.2),
                point(rx * 0.65, -rz),
            ];
            let mut color = self.color(material, [cx, positions[0][1], cz]);
            if self.forest && material == GroundMaterial::Lawn {
                let leaf = if sample % 5 == 0 { [0.20,0.135,0.075] } else if seed > 0.70 { [0.39,0.29,0.14] } else { [0.29,0.205,0.12] };
                color[..3].copy_from_slice(&leaf);
            }
            let contrast = if seed > 0.76 { 1.07 } else { 0.87 };
            for c in &mut color[..3] {
                *c *= contrast;
            }
            append_solid_quad(mesh, p, [0.0, 1.0, 0.0], color);
        }
    }
}

fn face_bounds(positions: &[[f32; 3]]) -> [f32; 4] {
    [
        positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min),
        positions
            .iter()
            .map(|p| p[0])
            .fold(f32::NEG_INFINITY, f32::max),
        positions.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min),
        positions
            .iter()
            .map(|p| p[2])
            .fold(f32::NEG_INFINITY, f32::max),
    ]
}

/// A few broad, quiet highlights break up the water without tiling bright
/// pixels. Every highlight remains fully inside an existing water-cell face.
fn append_water_ripple(
    target: &mut SurfaceMeshData,
    p: &[[f32; 3]],
    geometry: &GridGeometry,
    map_origin: [i32; 2],
) {
    let west = p.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
    let east = p.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
    let north = p.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
    let south = p.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max);
    if east - west < geometry.tile_width * 0.99 || south - north < geometry.tile_height * 0.99 {
        return;
    }
    let x = ((west - geometry.origin_x) / geometry.tile_width).floor() as i32 + map_origin[0];
    let z = ((north - geometry.origin_z) / geometry.tile_height).floor() as i32 + map_origin[1];
    let seed = lattice(x, z);
    if seed < 0.76 {
        return;
    }
    let length = geometry.tile_width * (0.34 + seed * 0.14);
    let cx = (west + east) * 0.5;
    let cz = north + geometry.tile_height * (0.30 + lattice(z + 91, x) * 0.40);
    let thickness = geometry.tile_height * 0.024;
    let y = p[0][1] + geometry.tile_height * 0.001;
    append_solid_quad(
        target,
        [
            [cx - length * 0.5, y, cz],
            [cx - length * 0.28, y, cz + thickness],
            [cx + length * 0.5, y, cz + thickness * 0.2],
            [cx + length * 0.28, y, cz - thickness * 0.6],
        ],
        [0.0, 1.0, 0.0],
        [0.16, 0.37, 0.38, 1.0],
    );
}

/// Material-only finishing pass on the existing, source-authored terrain.
/// No cells are added, removed or reclassified. Ground faces may be subdivided
/// for contact shading; their boundary, datum and normals remain unchanged.
pub(super) fn polish_surfaces(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    map_origin: [i32; 2],
) {
    if supports_map(map) {
        polish_world_surfaces(mesh, map, cells, geometry, map_origin);
    }
}

/// Finish known atlas ground throughout the authored world. The source sampler
/// and actual world cell must agree; new object/coverage claims are never made.
pub(super) fn polish_world_surfaces(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    map_origin: [i32; 2],
) {
    let finish = GroundFinish::new(map, cells, geometry, map_origin);
    // The terrain mesher encodes faces as four vertices and six indices,
    // including polygon fans. Refuse unfamiliar topology rather than alter it.
    if mesh.textured.positions.len() % 4 != 0
        || mesh.textured.indices.len() != mesh.textured.positions.len() / 4 * 6
        || mesh
            .textured
            .indices
            .chunks_exact(6)
            .enumerate()
            .any(|(i, face)| {
                let b = (i * 4) as u32;
                face != [b, b + 1, b + 2, b, b + 2, b + 3]
            })
    {
        return;
    }
    let old = std::mem::take(&mut mesh.textured);
    for base in (0..old.positions.len()).step_by(4) {
        let positions: [[f32; 3]; 4] = old.positions[base..base + 4].try_into().unwrap();
        let uvs: [[f32; 2]; 4] = old.uvs[base..base + 4].try_into().unwrap();
        let normal = old.normals[base];
        let material = sampled_cell(&uvs, geometry).and_then(|i| {
            kanto_shallow_bank_surface(
                &positions,
                normal,
                &uvs,
                i,
                cells,
                geometry,
                &mesh.authored_cells,
            )
            .or_else(|| {
                surface_material(
                    map,
                    &positions,
                    normal,
                    &cells[i].source,
                    cells,
                    geometry,
                    &mesh.authored_cells,
                )
            })
        });
        if let Some(material) = material {
            if normal[1] > 0.999 && positions[0][1].abs() < 0.001 {
                finish.append_surface(&mut mesh.solid, material, positions);
            } else {
                append_quad_colors(
                    &mut mesh.solid,
                    positions,
                    normal,
                    [[0.0; 2]; 4],
                    positions.map(|p| finish.color(material, p)),
                );
            }
            if material == GroundMaterial::Water && normal[1] > 0.999 {
                append_water_ripple(&mut mesh.solid, &positions, geometry, map_origin);
            }
        } else {
            append_quad_colors(
                &mut mesh.textured,
                positions,
                normal,
                uvs,
                old.colors[base..base + 4].try_into().unwrap(),
            );
        }
    }
    // Runtime background uses its own repeated texture. Carry those exact
    // four strips into the same material domain, without expanding the apron.
    let background_material = mesh.background.as_ref().and_then(|background| {
        cells
            .iter()
            .find(|tile| tile.texture == background.texture)
            .and_then(|tile| ground_material(&tile.source))
            .filter(|m| matches!(m, GroundMaterial::Lawn | GroundMaterial::Path))
    });
    if let Some(material) = background_material {
        let background = mesh.background.take().unwrap();
        for base in (0..background.mesh.positions.len()).step_by(4) {
            let positions: [[f32; 3]; 4] = background.mesh.positions[base..base + 4]
                .try_into()
                .unwrap();
            append_quad_colors(
                &mut mesh.solid,
                positions,
                background.mesh.normals[base],
                [[0.0; 2]; 4],
                positions.map(|p| finish.color(material, p)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{frame, source_with_tile};
    use super::*;

    fn drawing(blocks: &[&[u16]], door_column: usize) -> VisualWorldFrame {
        let width = blocks[0].len() * 4;
        let height = blocks.len() * 4;
        let sources = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let tile = if y >= height - 2 && (door_column - 1..door_column + 1).contains(&x)
                    {
                        DOOR_TILES[y - (height - 2)][x - (door_column - 1)]
                    } else {
                        0x06
                    };
                    source_with_tile(blocks[y / 4][x / 4], (x % 4) as u8, (y % 4) as u8, tile)
                })
            })
            .collect();
        frame(width as u32, height as u32, sources)
    }

    fn geometry(frame: &VisualWorldFrame) -> GridGeometry {
        GridGeometry {
            width: frame.grid_size.x as usize,
            height: frame.grid_size.y as usize,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: -32.0,
            origin_z: -40.0,
        }
    }

    fn traditional_drawing(blocks: &[&[u16]], door: usize) -> VisualWorldFrame {
        let mut result = drawing(blocks, door);
        let width = result.grid_size.x as usize;
        let height = result.grid_size.y as usize;
        for y in 0..2 {
            for x in 0..2 {
                result.tiles[(height - 2 + y) * width + door - 1 + x]
                    .source
                    .tile_index = TRADITIONAL_DOOR_TILES[y][x];
            }
        }
        result
    }

    #[test]
    fn connected_city_models_require_native_complete_signatures_and_doors() {
        for (map, blocks, door, kind) in [
            (
                "CherrygroveCity",
                &[&[0x18, 0x19][..], &[0x1a, 0x17][..]][..],
                3,
                ModelKind::Mart,
            ),
            (
                "CherrygroveCity",
                &[&[0x18, 0x19][..], &[0x1a, 0x1b][..]][..],
                3,
                ModelKind::Pokecenter,
            ),
            (
                "VioletCity",
                &[&[0x18, 0x19][..], &[0x1a, 0x17][..]][..],
                3,
                ModelKind::Mart,
            ),
            (
                "VioletCity",
                &[&[0x18, 0x1f, 0x19][..], &[0x1c, 0x1d, 0x1e][..]][..],
                5,
                ModelKind::VioletGym,
            ),
            ("Route30", &[&[0x14, 0x15][..]][..], 3, ModelKind::House),
            ("Route29", &[&[0x1a, 0x11][..]][..], 3, ModelKind::RouteGate),
        ] {
            let mut f = drawing(blocks, door);
            let g = geometry(&f);
            let cells: Vec<_> = f.tiles.iter().collect();
            let placements = building_placements(map, &cells, &g);
            assert_eq!(placements.len(), 1, "{map}");
            assert_eq!(
                building_model(map, &cells, &g, placements[0]),
                Some(BuildingModel {
                    kind,
                    door_column: door
                })
            );
            assert_eq!(
                building_model("AzaleaTown", &cells, &g, placements[0]),
                None
            );
            // One changed source phase or missing doorway must leave old art.
            let last = f.tiles.len() - g.width + door - 1;
            f.tiles[last].source.tile_index = 0;
            let cells: Vec<_> = f.tiles.iter().collect();
            assert_eq!(building_model(map, &cells, &g, placements[0]), None);
        }
    }

    #[test]
    fn violet_traditional_plots_preserve_tree_rows_and_native_thresholds() {
        for (blocks, door, kind) in [
            (
                &[&[0x2c, 0x2d][..], &[0x2e, 0x2f][..]][..],
                3,
                ModelKind::TraditionalHouse,
            ),
            (
                &[&[0x2c, 0x2a, 0x2d][..], &[0x26, 0x27, 0x2f][..]][..],
                5,
                ModelKind::TraditionalHouse,
            ),
            (
                &[&[0x2c, 0x2d][..], &[0x22, 0x23][..], &[0x28, 0x29][..]][..],
                3,
                ModelKind::SproutTower,
            ),
        ] {
            let original = traditional_drawing(blocks, door);
            let width = original.grid_size.x;
            let height = original.grid_size.y;
            let mut sources: Vec<_> = original.tiles.into_iter().map(|t| t.source).collect();
            sources.extend((0..width).map(|_| source_with_tile(0x01, 0, 0, 0x06)));
            let f = frame(width, height + 1, sources);
            let g = geometry(&f);
            let cells: Vec<_> = f.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let placements = building_placements("VioletCity", &cells, &g);
            assert_eq!(placements.len(), 1);
            let p = placements[0];
            assert_eq!(p.row, 2, "native crown cells are not part of the building");
            assert_eq!(p.height, g.height - 3);
            assert_eq!(
                building_model("VioletCity", &cells, &g, p),
                Some(BuildingModel {
                    kind,
                    door_column: door
                })
            );
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.0; cells.len()],
                ..Default::default()
            };
            let mut claimed = vec![false; cells.len()];
            // The extra unclaimed path row is a real source ground sample.
            assert!(append_building(
                &mut mesh,
                &cells,
                &shapes,
                &g,
                "VioletCity",
                p,
                &mut claimed
            ));
            assert!(claimed[..g.width * 2].iter().all(|c| !c));
            assert!(
                claimed[g.width * 2..g.width * (g.height - 1)]
                    .iter()
                    .all(|c| *c)
            );
            assert!(claimed[g.width * (g.height - 1)..].iter().all(|c| !c));
            assert_eq!(mesh.footing_heights, vec![0.0; cells.len()]);
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|v| v[0] >= g.origin_x - 0.001
                        && v[0] <= g.origin_x + g.width as f32 * 8.0 + 0.001
                        && v[2] >= g.origin_z + 16.0 - 0.001
                        && v[2] <= g.origin_z + (g.height - 1) as f32 * 8.0 + 0.001)
            );
        }
    }

    #[test]
    fn route29_modeled_grass_owns_every_phase_before_legacy_live_profiles() {
        let sources = (0..4)
            .flat_map(|y| {
                (0..5).map(move |x| {
                    if x < 4 {
                        source_with_tile(0x03, x, y, 0x04)
                    } else {
                        source_with_tile(0x01, 0, 0, 0x05)
                    }
                })
            })
            .collect();
        let mut f = frame(5, 4, sources);
        f.map_id = std::sync::Arc::from("Route29");
        let profiles: crate::live_profiles::Document = serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .unwrap();
        let mesh = build_instanced_terrain_mesh_with_profiles(
            &f,
            &TerrainImageSamples::default(),
            &profiles,
        )
        .unwrap();
        assert_eq!(mesh.textured.quad_count(), 0);
        assert!(mesh.tree_instances.is_empty());
        assert!(
            mesh.solid.positions.len()
                >= model(ModelKind::Grass).surface_mesh().positions.len() * 16
        );
        assert_eq!(mesh.footing_heights, vec![0.0; 20]);
    }

    #[test]
    fn grass_is_source_scoped_low_and_contained_without_footing_changes() {
        let mut f = frame(1, 1, vec![source_with_tile(0x03, 1, 2, 0x04)]);
        let g = geometry(&f);
        let mut mesh = SurfaceMeshData::default();
        assert!(append_grass(
            &mut mesh,
            "Route29",
            &f.tiles[0],
            &g,
            0.0,
            [7, 9]
        ));
        assert!(
            mesh.indices.len() < 300,
            "grass must remain cheap per source cell"
        );
        assert!(mesh.positions.iter().all(|p| p[0] > -32.0
            && p[0] < -24.0
            && p[2] > -40.0
            && p[2] < -32.0
            && p[1] >= 0.0
            && p[1] < 8.0 * 0.68));
        assert!(
            mesh.normals
                .iter()
                .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 0.001)
        );
        assert!(!append_grass(
            &mut mesh,
            "AzaleaTown",
            &f.tiles[0],
            &g,
            0.0,
            [7, 9]
        ));
        f.tiles[0].source.metatile_id = 0x04;
        assert!(!append_grass(
            &mut mesh,
            "Route29",
            &f.tiles[0],
            &g,
            0.0,
            [7, 9]
        ));
    }

    #[test]
    fn sprout_platform_is_a_ground_level_walkway_with_an_open_native_threshold() {
        let art = [
            [0x3b, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x3d],
            [0x4b, 0x4c, 0x9a, 0x9a, 0x4c, 0x4c, 0x4c, 0x4d],
            [0x06; 8],
            [0x06; 8],
        ];
        let sources = (0..4)
            .flat_map(|y| {
                (0..8).map(move |x| {
                    source_with_tile(
                        if x < 4 { 0x74 } else { 0x75 },
                        (x % 4) as u8,
                        y as u8,
                        art[y][x],
                    )
                })
            })
            .collect();
        let mut f = frame(8, 4, sources);
        f.map_id = std::sync::Arc::from("VioletCity");
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let p = building_placements("VioletCity", &cells, &g)[0];
        assert!(sprout_platform(&cells, &g, p));
        let mesh = build_terrain_mesh_with_samples(&f, &TerrainImageSamples::default()).unwrap();
        assert_eq!(mesh.footing_heights, vec![0.0; 32]);
        assert_eq!(
            mesh.textured.quad_count(),
            0,
            "all platform pixel roof/rail art is replaced"
        );
        // Check structural geometry before the ordinary lower-path grain
        // finish. That separate pass adds 0.01–0.02px dust flecks, not rails or
        // a walking deck. The actual landing/treads retain the stricter bound.
        let mut structure = TerrainMeshData {
            footing_heights: vec![0.0; 32],
            ..Default::default()
        };
        let mut claimed = vec![false; 32];
        let shapes: Vec<_> = cells.iter().map(|t| shape_for_source(&t.source)).collect();
        assert!(append_sprout_platform(
            &mut structure,
            &cells,
            &shapes,
            &g,
            p,
            &mut claimed
        ));
        let west = g.origin_x;
        let open_w = west + 16.0;
        let open_e = west + 32.0;
        assert!(
            structure
                .solid
                .positions
                .iter()
                .all(|v| v[1] <= 0.009 || v[0] <= open_w + 0.001 || v[0] >= open_e - 0.001),
            "no elevated structural geometry covers the native two-cell stair/walk channel"
        );
        assert_eq!(structure.footing_heights, vec![0.0; 32]);
        assert!(claimed.iter().all(|v| *v));
        assert!(mesh.solid.positions.iter().any(|v| v[1] == 0.0));
        assert!(
            mesh.solid.positions.iter().all(|v| v[1] < 6.0),
            "no false roof-height deck remains"
        );
        f.tiles[8 + 2].source.tile_index = 0x06;
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(
            !sprout_platform(&cells, &g, p),
            "modified native stair art is not promoted"
        );
    }

    // Literal pack signatures, independent of the finish classifier.
    fn kanto_bank_fixture(blocks: &[u16]) -> VisualWorldFrame {
        let height = blocks.len() * 4 + 2;
        frame(
            6,
            height as u32,
            (0..height)
                .flat_map(|y| {
                    (0..6).map(move |x| {
                        if x == 0 || x == 5 || y == 0 || y == height - 1 {
                            return material_source("unknown_mod", 0, 0, 0, 0);
                        }
                        let block = blocks[(y - 1) / 4];
                        let art = match block {
                            0x07 => [
                                0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c,
                                0x2c, 0x37, 0x37, 0x37, 0x37,
                            ],
                            0x1a => [
                                0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39,
                                0x39, 0x37, 0x37, 0x37, 0x37,
                            ],
                            0x2f => [
                                0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c, 0x2c,
                                0x2c, 0x37, 0x34, 0x04, 0x04,
                            ],
                            _ => unreachable!(),
                        };
                        let sx = x - 1;
                        let sy = (y - 1) % 4;
                        material_source("kanto", block, sx as u8, sy as u8, art[sy * 4 + sx])
                    })
                })
                .collect(),
        )
    }

    fn kanto_bank_geometry(f: &VisualWorldFrame) -> GridGeometry {
        GridGeometry {
            origin_x: -f.viewport_size.x * 0.5,
            origin_z: -f.viewport_size.y * 0.5,
            ..geometry(f)
        }
    }

    #[test]
    fn kanto_shallow_banks_finish_actual_mesher_caps_and_all_four_sides() {
        for (block, cap, top_count, side_count) in [
            (0x07, GroundMaterial::Lawn, 16, 16),
            (0x1a, GroundMaterial::Path, 16, 16),
            (0x2f, GroundMaterial::Lawn, 8, 12),
        ] {
            let f = kanto_bank_fixture(&[block]);
            let g = kanto_bank_geometry(&f);
            let cells: Vec<_> = f.tiles.iter().collect();
            let raw = build_terrain_mesh(&f).unwrap();
            let mut counts = [0; 5];
            let mut mesh = TerrainMeshData {
                footing_heights: raw.footing_heights.clone(),
                authored_cells: raw.authored_cells.clone(),
                ..Default::default()
            };
            for base in (0..raw.textured.positions.len()).step_by(4) {
                let p = &raw.textured.positions[base..base + 4];
                let n = raw.textured.normals[base];
                let uv = &raw.textured.uvs[base..base + 4];
                let sample = sampled_cell(uv, &g).unwrap();
                let material = kanto_shallow_bank_surface(p, n, uv, sample, &cells, &g, &[]);
                if p.iter().all(|v| v[1] == 0.0) {
                    assert_eq!(material, None, "flat ground, including $2f's east half");
                    continue;
                }
                assert_eq!(
                    material,
                    Some(if n[1] == 1.0 {
                        cap
                    } else {
                        GroundMaterial::Bank
                    })
                );
                let direction = match n {
                    [0.0, 1.0, 0.0] => 0,
                    [0.0, 0.0, -1.0] => 1,
                    [0.0, 0.0, 1.0] => 2,
                    [-1.0, 0.0, 0.0] => 3,
                    [1.0, 0.0, 0.0] => 4,
                    _ => panic!("unexpected native bank normal"),
                };
                counts[direction] += 1;
                for axis in 0..3 {
                    let mut changed = p.to_vec();
                    changed[0][axis] += 0.25;
                    assert_eq!(
                        kanto_shallow_bank_surface(&changed, n, uv, sample, &cells, &g, &[]),
                        None,
                        "clipped/custom geometry must retain source art"
                    );
                }
                append_quad_colors(
                    &mut mesh.textured,
                    p.try_into().unwrap(),
                    n,
                    uv.try_into().unwrap(),
                    raw.textured.colors[base..base + 4].try_into().unwrap(),
                );
            }
            assert_eq!(counts[0], top_count);
            assert_eq!(counts[1..].iter().sum::<usize>(), side_count);
            assert!(counts.iter().all(|&count| count > 0));
            // Apply the real finish to every original bank quad together.
            // The existing flat-ground subdivision pass is outside this change.
            let before = mesh.clone();
            polish_world_surfaces(&mut mesh, "Route4", &cells, &g, [0, 0]);
            assert!(mesh.textured.positions.is_empty());
            assert_eq!(mesh.solid.positions, before.textured.positions);
            assert_eq!(mesh.solid.normals, before.textured.normals);
            assert_eq!(mesh.solid.indices, before.textured.indices);
            assert_eq!(mesh.footing_heights, before.footing_heights);
            assert_eq!(mesh.authored_cells, before.authored_cells);
            assert!(mesh.animated_textured.positions.is_empty());
            assert!(mesh.animated_solid.positions.is_empty());
            if block == 0x2f {
                for y in 1..5 {
                    assert_eq!(&mesh.footing_heights[y * 6 + 3..y * 6 + 5], &[0.0, 0.0]);
                }
            }
        }
    }

    #[test]
    fn kanto_shallow_bank_finish_uses_the_actual_connected_run_uv_sample() {
        let f = kanto_bank_fixture(&[0x07, 0x1a]);
        let g = kanto_bank_geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let mesh = build_terrain_mesh(&f).unwrap();
        let mut borrowed = 0;
        for base in (0..mesh.textured.positions.len()).step_by(4) {
            let p = &mesh.textured.positions[base..base + 4];
            let n = mesh.textured.normals[base];
            if n != [0.0, 1.0, 0.0] || p[0][1] == 0.0 {
                continue;
            }
            let uv = &mesh.textured.uvs[base..base + 4];
            let sample = sampled_cell(uv, &g).unwrap();
            let expected = if cells[sample].source.tile_index == 0x2c {
                GroundMaterial::Lawn
            } else {
                GroundMaterial::Path
            };
            assert_eq!(
                kanto_shallow_bank_surface(p, n, uv, sample, &cells, &g, &[]),
                Some(expected)
            );
            let owner = cell_at(&g, (p[0][0] + p[2][0]) * 0.5, (p[0][2] + p[2][2]) * 0.5).unwrap();
            borrowed +=
                usize::from(cells[owner].source.metatile_id != cells[sample].source.metatile_id);
        }
        assert!(
            borrowed > 0,
            "the mesher really borrows cap UVs across a mixed bank run"
        );
    }

    #[test]
    fn kanto_shallow_bank_finish_rejects_changed_clipped_unknown_and_claimed_sources() {
        for block in [0x07, 0x1a, 0x2f] {
            let f = kanto_bank_fixture(&[block]);
            let g = kanto_bank_geometry(&f);
            let source_index = 7;
            assert!(complete_kanto_shallow_bank(
                source_index,
                &f.tiles.iter().collect::<Vec<_>>(),
                &g
            ));
            for y in 1..5 {
                for x in 1..5 {
                    for field in 0..4 {
                        let mut changed = f.clone();
                        let source = &mut changed.tiles[y * 6 + x].source;
                        match field {
                            0 => source.tile_index ^= 1,
                            1 => source.subtile_column = 4,
                            2 => source.subtile_row = 4,
                            _ => source.tileset_id = std::sync::Arc::from("unknown_mod"),
                        }
                        assert!(!complete_kanto_shallow_bank(
                            source_index,
                            &changed.tiles.iter().collect::<Vec<_>>(),
                            &g
                        ));
                    }
                }
            }
            for (width, height, dx, dy) in [(3, 4, 0, 0), (3, 4, 1, 0), (4, 3, 0, 0), (4, 3, 0, 1)]
            {
                let sources = (0..height)
                    .flat_map(|y| {
                        (0..width).map({
                            let f = &f;
                            move |x| f.tiles[(y + 1 + dy) * 6 + x + 1 + dx].source.clone()
                        })
                    })
                    .collect();
                let cropped = frame(width as u32, height as u32, sources);
                let cg = kanto_bank_geometry(&cropped);
                let cells: Vec<_> = cropped.tiles.iter().collect();
                for i in 0..cells.len() {
                    assert!(!complete_kanto_shallow_bank(i, &cells, &cg));
                }
            }
        }
        let f = kanto_bank_fixture(&[0x07]);
        let g = kanto_bank_geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut top = SurfaceMeshData::default();
        append_top(
            &mut top,
            g.bounds(1, 1).into(),
            crate::profile::JUMP_LEDGE_HEIGHT,
            g.uv(1, 1),
        );
        let p = &top.positions;
        let uv = &top.uvs;
        let n = [0.0, 1.0, 0.0];
        for block in [0x06, 0x28, 0x3f, 0x57, 0x79] {
            let mut changed = f.clone();
            for t in &mut changed.tiles {
                t.source.metatile_id = block;
            }
            assert_eq!(
                kanto_shallow_bank_surface(
                    p,
                    n,
                    uv,
                    7,
                    &changed.tiles.iter().collect::<Vec<_>>(),
                    &g,
                    &[]
                ),
                None
            );
        }
        let mut claims = vec![None; cells.len()];
        claims[7] = Some("custom/model");
        assert_eq!(
            kanto_shallow_bank_surface(p, n, uv, 7, &cells, &g, &claims),
            None
        );
        for axis in 0..3 {
            let mut changed = p.clone();
            changed[0][axis] += 0.25;
            assert_eq!(
                kanto_shallow_bank_surface(&changed, n, uv, 7, &cells, &g, &[]),
                None
            );
        }
        for height in [0.0, 3.0, 8.0, 16.0] {
            let mut changed = p.clone();
            for v in &mut changed {
                v[1] = height;
            }
            assert_eq!(
                kanto_shallow_bank_surface(&changed, n, uv, 7, &cells, &g, &[]),
                None
            );
        }
        let mut changed_uv = uv.clone();
        changed_uv[0][0] += 0.01;
        assert_eq!(
            kanto_shallow_bank_surface(p, n, &changed_uv, 7, &cells, &g, &[]),
            None
        );
        assert_eq!(
            kanto_shallow_bank_surface(p, [0.1, 0.99, 0.0], uv, 7, &cells, &g, &[]),
            None
        );
    }

    #[test]
    fn shallow_ledge_finish_preserves_the_native_datum_vertices_and_normals() {
        let mut f = frame(
            4,
            4,
            (0..4)
                .flat_map(|y| {
                    (0..4).map(move |x| {
                        source_with_tile(0x56, x, y, if y == 3 { 0x4c } else { 0x06 })
                    })
                })
                .collect(),
        );
        let before = build_terrain_mesh(&f).unwrap();
        f.map_id = std::sync::Arc::from("Route30");
        let after = build_terrain_mesh(&f).unwrap();
        assert_eq!(before.footing_heights, after.footing_heights);
        for (p, n) in before
            .textured
            .positions
            .iter()
            .zip(&before.textured.normals)
        {
            assert!(
                after
                    .textured
                    .positions
                    .iter()
                    .zip(&after.textured.normals)
                    .chain(after.solid.positions.iter().zip(&after.solid.normals))
                    .any(|(q, m)| p == q && n == m),
                "material finish must preserve every native ledge vertex/normal"
            );
        }
        assert!(
            after
                .solid
                .positions
                .iter()
                .any(|v| (v[1] - 6.0).abs() < 0.001)
        );
        let source = source_with_tile(0x74, 0, 0, 0x3b);
        assert!(
            !shallow_bank_source(&source),
            "tower-side reuse is outside the shallow-bank vocabulary"
        );
        assert!(!shallow_bank_source(&source_with_tile(0x56, 0, 0, 0x37)));
    }

    #[test]
    fn cherrygrove_mixed_shoreline_block_models_three_complete_trees_only() {
        // Actual source drawing east of Cherrygrove Center: three independent
        // crowns and one open lawn quadrant, not a single four-cell-wide tree.
        let art = [
            [0x1e, 0x1f, 0x05, 0x05],
            [0x3e, 0x3f, 0x05, 0x05],
            [0x1e, 0x1f, 0x1e, 0x1f],
            [0x3e, 0x3f, 0x3e, 0x3f],
        ];
        let sources = (0..4)
            .flat_map(|y| {
                (0..4).map(move |x| source_with_tile(0x64, x, y, art[y as usize][x as usize]))
            })
            .collect();
        let mut f = frame(4, 4, sources);
        f.map_id = std::sync::Arc::from("CherrygroveCity");
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let plots = tree_placements("CherrygroveCity", &cells, &g);
        assert_eq!(
            plots.iter().map(|p| (p.column, p.row)).collect::<Vec<_>>(),
            vec![(0, 0), (0, 2), (2, 2)]
        );
        assert!(tree_placements("AzaleaTown", &cells, &g).is_empty());
        let reserved = preferred_scenery_cells("CherrygroveCity", &cells, &g);
        assert_eq!(reserved.iter().filter(|&&v| v).count(), 12);
        assert!(!reserved[2] && !reserved[3] && !reserved[6] && !reserved[7]);
        let mesh = build_terrain_mesh_with_samples(&f, &TerrainImageSamples::default()).unwrap();
        assert!(
            mesh.textured.positions.is_empty(),
            "verified crown art must not remain as bright sprite fragments"
        );
        assert!(
            mesh.solid.positions.len() >= model(ModelKind::Tree).surface_mesh().positions.len() * 3
        );
        assert!(mesh.footing_heights.iter().all(|h| *h == 0.0));
        f.tiles[0].source.tile_index = 0x05;
        let cells: Vec<_> = f.tiles.iter().collect();
        assert_eq!(
            tree_placements("CherrygroveCity", &cells, &g).len(),
            2,
            "altered or incomplete crown remains faithful source art"
        );
        f.tiles[0].source.tile_index = 0x1e;
        f.tiles[1].source.subtile_column = 3;
        let cells: Vec<_> = f.tiles.iter().collect();
        assert_eq!(
            tree_placements("CherrygroveCity", &cells, &g).len(),
            2,
            "source phase is part of the complete signature"
        );
    }

    #[test]
    fn modeled_signs_require_the_complete_verified_art_and_stay_in_the_source_plot() {
        let mut sources = vec![source_with_tile(0x01, 0, 0, 0x06); 12];
        let art = [[0x4e, 0x4f], [0x5e, 0x5f]];
        for y in 0..2 {
            for x in 0..2 {
                sources[y * 4 + x] = source_with_tile(0x45, x as u8, y as u8, art[y][x]);
            }
        }
        let mut frame = frame(4, 3, sources);
        let geometry = geometry(&frame);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
        assert_eq!(sign_placements(&cells, &shapes, &geometry).len(), 1);
        let mut claimed = vec![false; cells.len()];
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0; cells.len()],
            ..Default::default()
        };
        append_signs(
            &mut mesh,
            "AzaleaTown",
            &cells,
            &shapes,
            &geometry,
            &mut claimed,
        );
        assert!(mesh.solid.positions.is_empty());
        append_signs(&mut mesh, MAP, &cells, &shapes, &geometry, &mut claimed);
        assert_eq!(claimed.iter().filter(|&&c| c).count(), 4);
        assert!(mesh.solid.positions.iter().all(|p| p[0] > -32.0
            && p[0] < -16.0
            && p[2] > -40.0
            && p[2] < -24.0
            && p[1] >= 0.0
            && p[1] < 16.0));
        assert!(mesh.footing_heights.iter().all(|&y| y == 0.0));
        frame.tiles[1].source.tile_index = 0x06;
        let cells: Vec<_> = frame.tiles.iter().collect();
        assert!(sign_placements(&cells, &shapes, &geometry).is_empty());
    }

    #[test]
    fn spatial_shadow_lookup_is_color_identical_to_full_scene_scan() {
        let width = 68;
        let height = 66;
        let sources = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    if x < 64 && y < 64 {
                        source_with_tile(
                            0x05,
                            (x % 4) as u8,
                            (y % 4) as u8,
                            (if y % 4 == 0 {
                                0x1e
                            } else if y % 4 == 3 {
                                0x3e
                            } else {
                                0x2e
                            }) + (x % 2) as u16,
                        )
                    } else {
                        source_with_tile(0x01, (x % 4) as u8, (y % 4) as u8, 0x05)
                    }
                })
            })
            .collect();
        let f = frame(width, height, sources);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let fast = GroundFinish::new("Route29", &cells, &g, [0, 0]);
        let reference = GroundFinish {
            forest: false,
            blend: fast.blend.clone(),
            geometry: &g,
            map_origin: [0, 0],
            shadows: fast.shadows.clone(),
            shadow_cells: None,
        };
        assert_eq!(fast.shadows.len(), 512);
        let max_candidates = fast
            .shadow_cells
            .as_ref()
            .unwrap()
            .iter()
            .map(Vec::len)
            .max()
            .unwrap();
        assert!(
            max_candidates <= 4,
            "contact shade must consider local plots, not the entire forest"
        );
        for y in 0..height * 2 + 1 {
            for x in 0..width * 2 + 1 {
                let p = [
                    g.origin_x + x as f32 * 4.0,
                    0.0,
                    g.origin_z + y as f32 * 4.0,
                ];
                assert_eq!(
                    fast.color(GroundMaterial::Lawn, p),
                    reference.color(GroundMaterial::Lawn, p)
                );
            }
        }
    }

    #[test]
    #[ignore = "opt-in full/adaptive geometry and build-time measurement"]
    fn benchmark_dense_modeled_terrain_budget() {
        let width = 68;
        let height = 66;
        let sources = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    if x < 48 && y < 64 {
                        source_with_tile(
                            0x05,
                            (x % 4) as u8,
                            (y % 4) as u8,
                            (if y % 4 == 0 {
                                0x1e
                            } else if y % 4 == 3 {
                                0x3e
                            } else {
                                0x2e
                            }) + (x % 2) as u16,
                        )
                    } else if x < 64 && y < 64 {
                        source_with_tile(0x03, (x % 4) as u8, (y % 4) as u8, 0x04)
                    } else {
                        source_with_tile(0x01, (x % 4) as u8, (y % 4) as u8, 0x05)
                    }
                })
            })
            .collect();
        let mut f = frame(width, height, sources);
        f.map_id = std::sync::Arc::from("Route29");
        let profiles: crate::live_profiles::Document = serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .unwrap();
        let start = std::time::Instant::now();
        let mesh = build_instanced_terrain_mesh_with_profiles(
            &f,
            &TerrainImageSamples::default(),
            &profiles,
        )
        .unwrap();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        assert!(mesh.footing_heights.iter().all(|h| *h == 0.0));
        assert!(mesh.textured.positions.is_empty());
        println!(
            "terrain_budget detail={} grid=68x66 trees=384 grass=1024 vertices={} triangles={} bytes={} build_ms={:.2}",
            if crate::new_bark_models::scenery_full_detail() {
                "full"
            } else {
                "adaptive"
            },
            mesh.solid.positions.len(),
            mesh.solid.indices.len() / 3,
            mesh.solid.positions.len() * 48 + mesh.solid.indices.len() * 4,
            elapsed
        );
    }

    #[test]
    #[ignore = "opt-in comparative CPU benchmark; correctness is covered separately"]
    fn benchmark_dense_forest_contact_shadow_lookup() {
        let width = 68;
        let height = 66;
        let sources = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    if x < 64 && y < 64 {
                        source_with_tile(
                            0x05,
                            (x % 4) as u8,
                            (y % 4) as u8,
                            (if y % 4 == 0 {
                                0x1e
                            } else if y % 4 == 3 {
                                0x3e
                            } else {
                                0x2e
                            }) + (x % 2) as u16,
                        )
                    } else {
                        source_with_tile(0x01, 0, 0, 0x05)
                    }
                })
            })
            .collect();
        let f = frame(width, height, sources);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let fast = GroundFinish::new("Route29", &cells, &g, [0, 0]);
        let reference = GroundFinish {
            forest: false,
            blend: fast.blend.clone(),
            geometry: &g,
            map_origin: [0, 0],
            shadows: fast.shadows.clone(),
            shadow_cells: None,
        };
        let run = |finish: &GroundFinish| {
            let started = std::time::Instant::now();
            let mut checksum = 0.0;
            for _ in 0..10 {
                for y in 0..height * 2 {
                    for x in 0..width * 2 {
                        checksum += std::hint::black_box(finish.color(
                            GroundMaterial::Lawn,
                            [
                                g.origin_x + x as f32 * 4.0,
                                0.0,
                                g.origin_z + y as f32 * 4.0,
                            ],
                        ))[0];
                    }
                }
            }
            (started.elapsed().as_secs_f64() * 1000.0, checksum)
        };
        let old = run(&reference);
        let new = run(&fast);
        assert_eq!(old.1, new.1);
        println!(
            "shadow_lookup trees={} samples={} full_scan_ms={:.2} indexed_ms={:.2} speedup={:.2}",
            fast.shadows.len(),
            width * height * 4 * 10,
            old.0,
            new.0,
            old.0 / new.0
        );
    }

    #[test]
    fn contact_shadows_are_soft_bounded_and_do_not_change_surface_height() {
        let frame = frame(2, 2, vec![source_with_tile(0x01, 0, 0, 0x05); 4]);
        let geometry = geometry(&frame);
        let shadow = GroundShadow {
            bounds: [-32.0, -16.0, -40.0, -24.0],
            tree: true,
        };
        assert!(shadow.darkness([-24.0, 0.0, -32.0], &geometry) > 0.45);
        assert_eq!(shadow.darkness([-32.0, 0.0, -32.0], &geometry), 0.0);
        assert_eq!(shadow.darkness([-15.9, 0.0, -32.0], &geometry), 0.0);
        assert!(
            shadow.darkness([-30.0, 0.0, -32.0], &geometry)
                < shadow.darkness([-28.0, 0.0, -32.0], &geometry)
        );
        let finish = GroundFinish {
            forest: false,
            blend: Default::default(),
            geometry: &geometry,
            map_origin: [0, 0],
            shadows: vec![shadow],
            shadow_cells: None,
        };
        let unshaded = GroundFinish {
            forest: false,
            blend: Default::default(),
            geometry: &geometry,
            map_origin: [0, 0],
            shadows: vec![],
            shadow_cells: None,
        };
        assert!(
            finish.color(GroundMaterial::Lawn, [-24.0, 0.0, -32.0])[0]
                < unshaded.color(GroundMaterial::Lawn, [-24.0, 0.0, -32.0])[0]
        );
        let mut mesh = SurfaceMeshData::default();
        finish.append_surface(
            &mut mesh,
            GroundMaterial::Lawn,
            [
                [-32.0, 0.0, -40.0],
                [-32.0, 0.0, -32.0],
                [-24.0, 0.0, -32.0],
                [-24.0, 0.0, -40.0],
            ],
        );
        assert_eq!(
            &mesh.positions[..16]
                .iter()
                .map(|p| p[1])
                .collect::<Vec<_>>(),
            &vec![0.0; 16]
        );
    }

    #[test]
    fn surface_grain_and_color_remain_anchored_when_the_source_grid_scrolls() {
        let frame = frame(1, 1, vec![source_with_tile(0x01, 0, 0, 0x06)]);
        let geometry = geometry(&frame);
        let finish = GroundFinish {
            forest: false,
            blend: Default::default(),
            geometry: &geometry,
            map_origin: [10, 9],
            shadows: vec![],
            shadow_cells: None,
        };
        let shifted = GroundFinish {
            forest: false,
            blend: Default::default(),
            geometry: &geometry,
            map_origin: [11, 9],
            shadows: vec![],
            shadow_cells: None,
        };
        let p = [-28.0, 0.0, -36.0];
        assert_eq!(
            finish.color(GroundMaterial::Path, p),
            shifted.color(GroundMaterial::Path, [p[0] - 8.0, p[1], p[2]])
        );
        for material in [GroundMaterial::Path, GroundMaterial::Lawn] {
            let mut count = 0;
            for column in 0..20 {
                let w = -32.0 + column as f32 * 8.0;
                let positions = [
                    [w, 0.0, -40.0],
                    [w, 0.0, -32.0],
                    [w + 8.0, 0.0, -32.0],
                    [w + 8.0, 0.0, -40.0],
                ];
                let mut mesh = SurfaceMeshData::default();
                finish.append_grain(&mut mesh, material, positions);
                count += mesh.quad_count();
                assert!(mesh.positions.iter().all(|p| p[0] > w
                    && p[0] < w + 8.0
                    && p[2] > -40.0
                    && p[2] < -32.0
                    && p[1] > 0.0
                    && p[1] < 0.02));
            }
            assert!(count > 0, "expected some irregular surface detail");
        }
    }

    #[test]
    fn forest_floor_is_continuous_world_anchored_and_leaves_walk_datum_intact() {
        let f = frame(2, 2, vec![source_with_tile(0x01,0,0,0x05); 4]);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut floor = GroundFinish::new("IlexForest", &cells, &g, [0,0]);
        floor.forest = true;
        let soil = floor.color(GroundMaterial::Lawn, [-28.0,0.0,-36.0]);
        assert_ne!(soil, ground_color(GroundMaterial::Lawn, [4.0,0.0,4.0], &g));
        let near = floor.color(GroundMaterial::Lawn, [-28.0001,0.0,-36.0]);
        assert!(soil.iter().zip(near).all(|(a,b)| (a-b).abs() < 0.001));
        let mut mesh = SurfaceMeshData::default();
        floor.append_surface(&mut mesh, GroundMaterial::Lawn,
            [[-32.0,0.0,-40.0],[-32.0,0.0,-32.0],[-24.0,0.0,-32.0],[-24.0,0.0,-40.0]]);
        assert!(mesh.positions.iter().all(|p| p[1]>=0.0 && p[1]<0.02));
        assert!(mesh.positions.iter().any(|p| p[1]>0.0), "forest must include leaf litter");
    }

    #[test]
    fn tree_variants_preserve_horizontal_plot_and_ground_anchor() {
        let sources = (0..4)
            .flat_map(|row| {
                (0..3).map(move |column| {
                    if column < 2 {
                        source_with_tile(
                            0x05,
                            column,
                            row,
                            (if row == 0 {
                                0x1e
                            } else if row == 3 {
                                0x3e
                            } else {
                                0x2e
                            }) + column as u16,
                        )
                    } else {
                        source_with_tile(0x01, 0, 0, 0x05)
                    }
                })
            })
            .collect();
        let mut frame = frame(3, 4, sources);
        frame.map_id = std::sync::Arc::from(MAP);
        let a = build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap();
        frame.grid_origin.x += 1;
        let b = build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap();
        assert_ne!(a.solid.positions, b.solid.positions);
        assert_eq!(b, build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap());
        // Different world placements may change the contour, while every
        // elevated tree vertex stays grounded inside the same occupied plot.
        assert!(a.solid.positions.iter().chain(&b.solid.positions)
            .filter(|p| p[1] > 0.02)
            .all(|p| p[0] >= -12.001 && p[0] <= 4.001
                && p[2] >= -0.001 && p[2] <= 16.001
                && p[1] <= 80.0 * 1.601));
        assert_eq!(a.footing_heights, b.footing_heights);
    }

    #[test]
    fn ground_materials_require_the_actual_source_and_preserve_unknown_art() {
        for (tile, expected) in [(0x05, GroundMaterial::Lawn), (0x06, GroundMaterial::Path)] {
            assert_eq!(
                ground_material(&source_with_tile(0x01, 0, 0, tile)),
                Some(expected)
            );
        }
        assert_eq!(
            ground_material(&source_with_tile(0x54, 1, 1, 0x14)),
            Some(GroundMaterial::Water)
        );
        for tile in [0x5c, 0x4c, 0x3d] {
            assert_eq!(
                ground_material(&source_with_tile(0x54, 0, 0, tile)),
                Some(GroundMaterial::Shore)
            );
        }
        assert_eq!(ground_material(&source_with_tile(0x01, 0, 0, 0x07)), None);
        assert_eq!(
            ground_material(&source_with_tile(0x05, 0, 0, 0x05)),
            None,
            "a source tree is not ground, even when a synthetic fixture uses lawn pixels"
        );
        let mut non_johto = source_with_tile(0x01, 0, 0, 0x05);
        non_johto.tileset_id = std::sync::Arc::from("forest");
        assert_eq!(
            ground_material(&non_johto),
            Some(GroundMaterial::Lawn),
            "the source-verified forest underlay shares the meadow finish"
        );
        non_johto.tileset_id = std::sync::Arc::from("unknown_mod");
        assert_eq!(ground_material(&non_johto), None);
    }

    #[test]
    fn finishing_preserves_ground_geometry_footing_and_every_other_map() {
        let mut frame = frame(
            4,
            1,
            vec![
                source_with_tile(0x01, 0, 0, 0x05),
                source_with_tile(0x01, 1, 0, 0x06),
                source_with_tile(0x54, 0, 0, 0x5c),
                source_with_tile(0x54, 1, 0, 0x14),
            ],
        );
        frame.map_id = std::sync::Arc::from(MAP);
        let geometry = geometry(&frame);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0, 0.0, 0.0, -2.0],
            ..Default::default()
        };
        for (i, cell) in cells.iter().enumerate() {
            let (w, e, n, s) = geometry.bounds(i, 0);
            append_top(
                &mut mesh.textured,
                [w, e, n, s],
                shape_for_source(&cell.source).surface_height(geometry.tile_height),
                geometry.uv(i, 0),
            );
        }
        let before = mesh.clone();
        polish_surfaces(&mut mesh, "AzaleaTown", &cells, &geometry, [0, 0]);
        assert_eq!(mesh, before, "unverified maps keep their original finish");
        polish_surfaces(&mut mesh, MAP, &cells, &geometry, [0, 0]);
        assert!(mesh.textured.positions.is_empty());
        assert_eq!(mesh.footing_heights, before.footing_heights);
        for p in before.textured.positions {
            assert!(
                mesh.solid.positions.contains(&p),
                "original ground vertex {p:?} was changed"
            );
        }
        assert!(
            mesh.solid
                .colors
                .iter()
                .all(|color| color[3] == 1.0 && color.iter().all(|v| (0.0..=1.0).contains(v)))
        );
    }

    #[test]
    fn authored_ground_below_objects_changes_but_object_faces_do_not() {
        let frame = frame(
            2,
            1,
            vec![
                source_with_tile(0x05, 0, 0, 0x20),
                source_with_tile(0x01, 0, 0, 0x05),
            ],
        );
        let geometry = geometry(&frame);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let (w, e, n, s) = geometry.bounds(0, 0);
        let mut mesh = TerrainMeshData::default();
        append_top(&mut mesh.textured, [w, e, n, s], 0.0, geometry.uv(1, 0));
        append_top(&mut mesh.textured, [w, e, n, s], 4.0, geometry.uv(1, 0));
        append_top(&mut mesh.textured, [w, e, n, s], 0.0, geometry.uv(0, 0));
        polish_surfaces(&mut mesh, MAP, &cells, &geometry, [0, 0]);
        assert!(
            (4..=7).contains(&mesh.solid.quad_count()),
            "only sampled ground at the ground datum changes"
        );
        assert_eq!(
            mesh.textured.quad_count(),
            2,
            "raised and unknown artwork survives"
        );
    }

    #[test]
    fn shore_caps_and_the_existing_water_drop_share_the_stone_material() {
        let frame = frame(
            2,
            1,
            vec![
                source_with_tile(0x54, 0, 1, 0x3d),
                source_with_tile(0x54, 1, 1, 0x14),
            ],
        );
        let geometry = geometry(&frame);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let mut mesh = TerrainMeshData::default();
        let (w, e, n, s) = geometry.bounds(0, 0);
        append_textured_shoreline(
            &mut mesh.textured,
            &geometry,
            0,
            0,
            Direction::East,
            [w, e, n, s],
            -2.0,
            0.0,
        );
        let original = mesh.textured.positions.clone();
        polish_surfaces(&mut mesh, MAP, &cells, &geometry, [0, 0]);
        assert!(mesh.textured.positions.is_empty());
        assert_eq!(mesh.solid.positions, original);
        assert_eq!(mesh.solid.normals, vec![[1.0, 0.0, 0.0]; 4]);
    }

    #[test]
    fn terrain_color_is_deterministic_continuous_and_water_ripples_stay_inside() {
        let frame = frame(1, 1, vec![source_with_tile(0x54, 1, 1, 0x14)]);
        let geometry = geometry(&frame);
        let p = [17.0, 0.0, -19.0];
        assert_eq!(
            ground_color(GroundMaterial::Lawn, p, &geometry),
            ground_color(GroundMaterial::Lawn, p, &geometry)
        );
        let a = ground_color(GroundMaterial::Lawn, [15.9999, 0.0, 8.0], &geometry);
        let b = ground_color(GroundMaterial::Lawn, [16.0001, 0.0, 8.0], &geometry);
        assert!((a[0] - b[0]).abs() < 0.00001);
        let mut emitted = 0;
        for x in 0..20 {
            let w = x as f32 * 8.0;
            let positions = [
                [w, -2.0, 0.0],
                [w, -2.0, 8.0],
                [w + 8.0, -2.0, 8.0],
                [w + 8.0, -2.0, 0.0],
            ];
            let mut mesh = SurfaceMeshData::default();
            append_water_ripple(&mut mesh, &positions, &geometry, [0, 0]);
            assert!(mesh.positions.iter().all(|p| p[0] > w
                && p[0] < w + 8.0
                && p[2] > 0.0
                && p[2] < 8.0
                && p[1] > -2.0
                && p[1] < -1.99));
            emitted += mesh.quad_count();
        }
        assert!(emitted > 0 && emitted < 10, "ripples should remain sparse");
    }

    #[test]
    fn exact_source_signatures_select_the_four_new_bark_buildings() {
        for (blocks, door, kind) in [
            (
                &[&[0x18, 0x1f, 0x19][..], &[0x1c, 0x77, 0x1e][..]][..],
                5,
                ModelKind::Lab,
            ),
            (
                &[&[0x18, 0x19][..], &[0x16, 0x1e][..]][..],
                3,
                ModelKind::PlayerHouse,
            ),
            (&[&[0x14, 0x15][..]][..], 3, ModelKind::House),
        ] {
            let frame = drawing(blocks, door);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let geometry = geometry(&frame);
            let placements = outdoor_building_placements(&cells, &geometry);
            assert_eq!(placements.len(), 1);
            assert_eq!(
                building_model(MAP, &cells, &geometry, placements[0]),
                Some(BuildingModel {
                    kind,
                    door_column: door
                })
            );
            assert_eq!(
                building_model("Route29", &cells, &geometry, placements[0]),
                None
            );
        }
    }

    #[test]
    fn partial_or_reused_source_art_is_not_enough_to_replace_a_building() {
        let mut frame = drawing(&[&[0x18, 0x19], &[0x16, 0x1e]], 3);
        let geometry = geometry(&frame);
        let placement = BuildingPlacement {
            column: 0,
            row: 0,
            width: 8,
            height: 8,
            roof_rows: 4,
            ground_tile_index: 0x06,
        };
        frame.tiles[7 * 8 + 2].source.tile_index = 0;
        let cells: Vec<_> = frame.tiles.iter().collect();
        assert_eq!(building_model(MAP, &cells, &geometry, placement), None);
        frame.tiles[7 * 8 + 2].source.tile_index = 0x39;
        frame.tiles[1].source.subtile_column = 0;
        let cells: Vec<_> = frame.tiles.iter().collect();
        assert_eq!(building_model(MAP, &cells, &geometry, placement), None);
    }

    #[test]
    fn runtime_path_uses_authored_geometry_without_sampling_building_pixels() {
        let source_frame = drawing(&[&[0x14, 0x15]], 3);
        let mut sources: Vec<_> = source_frame
            .tiles
            .into_iter()
            .map(|tile| tile.source)
            .collect();
        sources.extend((0..8).map(|_| source_with_tile(0x01, 0, 0, 0x06)));
        let mut frame = frame(8, 5, sources);
        frame.map_id = std::sync::Arc::from(MAP);
        // An empty image sampler is intentional: authored replacement geometry
        // needs source identity plus the ground atlas slot, not raster pixels.
        let mesh =
            build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap();
        assert_eq!(mesh.textured.quad_count(), 0);
        assert!(
            mesh.solid.indices.len()
                >= model(ModelKind::House).surface_mesh().indices.len() + 40 * 24
        );
        assert!(
            mesh.solid.positions[..model(ModelKind::House).surface_mesh().positions.len()]
                .iter()
                .all(|p| p[0] >= -32.001 && p[0] <= 32.001 && p[2] >= -20.001 && p[2] <= 12.001)
        );
        assert!(mesh.footing_heights.iter().all(|height| *height == 0.0));
        frame.map_id = std::sync::Arc::from("Route29");
        assert!(matches!(
            build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()),
            Err(TerrainMeshError::MissingMaskImage { .. })
        ));
    }

    #[test]
    fn modeled_trees_take_ownership_before_the_old_live_card_profile() {
        let sources = (0..4)
            .flat_map(|row| {
                (0..3).map(move |column| {
                    if column < 2 {
                        source_with_tile(
                            0x05,
                            column,
                            row,
                            (if row == 0 {
                                0x1e
                            } else if row == 3 {
                                0x3e
                            } else {
                                0x2e
                            }) + column as u16,
                        )
                    } else {
                        source_with_tile(0x01, 0, 0, 0x05)
                    }
                })
            })
            .collect();
        let mut frame = frame(3, 4, sources);
        frame.map_id = std::sync::Arc::from(MAP);
        let profiles: crate::live_profiles::Document = serde_json::from_str(
            r#"{
            "objects":[{"name":"test source tree", "tileset":"johto", "metatile":5,
                "origin":[0,0], "tiles":[[30,31],[46,47],[46,47],[62,63]],
                "ground":5,"top_pixels":0,"depth_pixels":0,"mask":"ground"}]
        }"#,
        )
        .unwrap();
        let mesh = build_instanced_terrain_mesh_with_profiles(
            &frame,
            &TerrainImageSamples::default(),
            &profiles,
        )
        .unwrap();
        assert!(
            mesh.solid.indices.len()
                >= model(ModelKind::Tree).surface_mesh().indices.len() + 12 * 24
        );
        assert_eq!(mesh.textured.quad_count(), 0);
        assert!(mesh.tree_instances.is_empty());
        assert!(mesh.footing_heights.iter().all(|height| *height == 0.0));
    }

    #[test]
    fn modeled_flowers_are_map_scoped_and_stay_inside_the_source_cell() {
        let frame = frame(1, 1, vec![source_with_tile(0x04, 1, 1, 0x03)]);
        let geometry = geometry(&frame);
        let mut mesh = SurfaceMeshData::default();
        assert!(!append_flower(
            &mut mesh,
            "AzaleaTown",
            &frame.tiles[0],
            &geometry,
            0.0
        ));
        assert!(mesh.positions.is_empty());
        assert!(append_flower(
            &mut mesh,
            MAP,
            &frame.tiles[0],
            &geometry,
            0.0
        ));
        assert!(mesh.positions.iter().all(|p| p[0] > -32.0
            && p[0] < -24.0
            && p[2] > -40.0
            && p[2] < -32.0
            && p[1] >= 0.0));
    }

    #[test]
    fn flower_animation_refresh_retains_the_authored_model() {
        let mut frame = frame(
            2,
            1,
            vec![
                source_with_tile(0x04, 1, 1, 0x03),
                source_with_tile(0x01, 1, 0, 0x05),
            ],
        );
        frame.map_id = std::sync::Arc::from(MAP);
        let terrain =
            build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap();
        let (textured, solid) =
            build_animated_flowers(&frame, &Assets::<Image>::default()).unwrap();
        assert_eq!(textured.count_vertices(), 0);
        assert_eq!(
            solid.count_vertices(),
            terrain.animated_solid.positions.len()
        );
        assert_eq!(
            solid.indices().unwrap().len(),
            terrain.animated_solid.indices.len()
        );
        assert!(!terrain.animated_solid.positions.is_empty());
        assert!(terrain.animated_textured.positions.is_empty());
    }

    #[test]
    fn model_placement_leaves_authoritative_footing_unchanged() {
        let frame = drawing(&[&[0x14, 0x15]], 3);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let geometry = geometry(&frame);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0; cells.len()],
            ..Default::default()
        };
        let before = mesh.footing_heights.clone();
        let shapes = vec![CellShape::Flat; cells.len()];
        let mut claimed = vec![false; cells.len()];
        let placement = outdoor_building_placements(&cells, &geometry)[0];
        assert!(append_building(
            &mut mesh,
            &cells,
            &shapes,
            &geometry,
            MAP,
            placement,
            &mut claimed
        ));
        assert_eq!(mesh.footing_heights, before);
        assert!(claimed.iter().all(|claimed| *claimed));
        assert!(!mesh.solid.indices.is_empty());
        assert_eq!(mesh.textured.quad_count(), cells.len());
    }
    #[test]
    fn expanded_ground_finishes_are_source_exact_and_never_object_coverage() {
        let mut source = source_with_tile(0x06, 0, 0, 0x2f);
        source.tileset_id = "johto_modern".into();
        assert_eq!(ground_material(&source), Some(GroundMaterial::Pavers));
        source.tileset_id = "johto".into();
        assert_eq!(
            ground_material(&source),
            None,
            "same number in another atlas is not pavement"
        );
        source.tileset_id = "cave".into();
        source.tile_index = 0x16;
        assert_eq!(ground_material(&source), Some(GroundMaterial::Stone));
        source.tileset_id = "unknown_mod".into();
        assert_eq!(ground_material(&source), None);
    }

    #[test]
    fn paving_finish_preserves_support_and_source_bounds() {
        let mut source = source_with_tile(0x06, 0, 0, 0x2f);
        source.tileset_id = "johto_modern".into();
        let frame = frame(1, 1, vec![source]);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry {
            width: 1,
            height: 1,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0],
            authored_cells: vec![None],
            ..Default::default()
        };
        append_top(&mut mesh.textured, [0.0, 8.0, 0.0, 8.0], 0.0, g.uv(0, 0));
        polish_world_surfaces(&mut mesh, "GoldenrodCity", &cells, &g, [0, 0]);
        assert!(mesh.textured.positions.is_empty());
        assert!(!mesh.solid.positions.is_empty());
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[1] == 0.0 && (0.0..=8.0).contains(&p[0]) && (0.0..=8.0).contains(&p[2]))
        );
        assert_eq!(mesh.footing_heights, vec![0.0]);
        assert_eq!(mesh.authored_cells, vec![None]);
    }

    fn material_source(tileset: &str, block: u16, x: u8, y: u8, tile: u16) -> VisualTileSource {
        let mut source = source_with_tile(block, x, y, tile);
        source.tileset_id = tileset.into();
        source
    }

    #[test]
    fn kanto_plaza_finishes_both_checker_samples_but_preserves_the_prop_quadrant() {
        for block in [0x79, 0x7b] {
            for y in 0..4 {
                for x in 0..4 {
                    let prop = block == 0x79 && x >= 2 && y >= 2;
                    let tile = if prop {
                        0x46 + u16::from(x - 2) + u16::from(y - 2) * 0x10
                    } else if (x + y) % 2 == 0 {
                        0x30
                    } else {
                        0x39
                    };
                    let source = material_source("kanto", block, x, y, tile);
                    assert_eq!(
                        ground_material(&source),
                        (!prop).then_some(GroundMaterial::Pavers),
                        "block {block:x}, source cell ({x}, {y})"
                    );
                    // A changed known tile at the wrong phase is not enough.
                    let mut changed = source.clone();
                    changed.tile_index = if tile == 0x30 { 0x39 } else { 0x30 };
                    assert_eq!(ground_material(&changed), None);
                }
            }
        }
        assert_eq!(
            ground_material(&material_source("kanto", 0x78, 0, 0, 0x30)),
            None
        );
        assert_eq!(
            ground_material(&material_source("unknown_mod", 0x7b, 0, 0, 0x30)),
            None
        );
        assert_eq!(
            ground_material(&material_source("kanto", 0x7b, 4, 0, 0x30)),
            None
        );
    }

    #[test]
    fn ice_pools_finish_every_native_quadrant_and_reject_boulder_reuse() {
        for (block, boulder) in [
            (0x1f, None),
            (0x2c, Some((0, 0))),
            (0x2d, Some((0, 1))),
            (0x2e, Some((1, 0))),
            (0x2f, Some((1, 1))),
        ] {
            for y in 0..4 {
                for x in 0..4 {
                    let is_boulder = boulder == Some((x / 2, y / 2));
                    let tile = if is_boulder { 0x82 } else { 0xc6 }
                        + u16::from(x % 2)
                        + u16::from(y % 2) * 0x10;
                    let source = material_source("ice_path", block, x, y, tile);
                    assert_eq!(
                        ground_material(&source),
                        (!is_boulder).then_some(GroundMaterial::Ice)
                    );
                    let mut changed = source.clone();
                    changed.tile_index = if is_boulder { 0xc6 } else { 0x19 };
                    assert_eq!(ground_material(&changed), None);
                }
            }
        }
        for source in [
            material_source("ice_path", 0x1f, 0, 0, 0xc7),
            material_source("ice_path", 0x30, 0, 0, 0xc6),
            material_source("ice_path", 0x1f, 4, 0, 0xc6),
            material_source("unknown_mod", 0x1f, 0, 0, 0xc6),
        ] {
            assert_eq!(ground_material(&source), None);
        }
    }

    #[test]
    fn ice_floor_variations_keep_ladder_rock_and_changed_cells_as_source_art() {
        for (block, art) in [
            (
                0x02,
                [
                    0x9a, 0x19, 0x9a, 0x9a, 0x19, 0x9a, 0x19, 0x19, 0x9a, 0x19, 0x19, 0x19, 0xaa,
                    0x19, 0x19, 0x9a,
                ],
            ),
            (
                0x14,
                [
                    0xc0, 0xc1, 0xc2, 0xc3, 0xd0, 0xd1, 0xd2, 0xd3, 0xc4, 0xc5, 0x9b, 0x19, 0xd4,
                    0xd5, 0xaa, 0x19,
                ],
            ),
            (
                0x37,
                [
                    0x9b, 0x9b, 0x9b, 0x9b, 0x9b, 0xaa, 0x19, 0xab, 0xaa, 0x12, 0x0a, 0x0b, 0xaa,
                    0x12, 0x1a, 0x1b,
                ],
            ),
        ] {
            for (i, tile) in art.into_iter().enumerate() {
                let source = material_source("ice_path", block, (i % 4) as u8, (i / 4) as u8, tile);
                assert_eq!(
                    ground_material(&source),
                    matches!(tile, 0x19 | 0x9a | 0x9b | 0xaa).then_some(GroundMaterial::Stone)
                );
                let mut changed = source;
                changed.tile_index = if tile == 0x19 { 0x9a } else { 0x19 };
                assert_eq!(ground_material(&changed), None);
            }
        }
        assert_eq!(
            ground_material(&material_source("ice_path", 0x30, 0, 0, 0x9a)),
            None
        );
    }

    #[test]
    fn mixed_plaza_finishing_keeps_source_prop_faces_and_support_untouched() {
        let art = [
            0x30, 0x39, 0x30, 0x39, 0x39, 0x30, 0x39, 0x30, 0x30, 0x39, 0x46, 0x47, 0x39, 0x30,
            0x56, 0x57,
        ];
        let sources = art
            .into_iter()
            .enumerate()
            .map(|(i, tile)| material_source("kanto", 0x79, (i % 4) as u8, (i / 4) as u8, tile))
            .collect();
        let frame = frame(4, 4, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = geometry(&frame);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.0; 16],
            authored_cells: vec![None; 16],
            ..Default::default()
        };
        for i in 0..16 {
            append_top(
                &mut mesh.textured,
                g.bounds(i % 4, i / 4).into(),
                0.0,
                g.uv(i % 4, i / 4),
            );
        }
        let before = mesh.clone();
        polish_world_surfaces(&mut mesh, "LavenderTown", &cells, &g, [0, 0]);
        assert_eq!(mesh.textured.quad_count(), 4);
        for (survivor, source) in [10, 11, 14, 15].into_iter().enumerate() {
            assert_eq!(
                mesh.textured.positions[survivor * 4..survivor * 4 + 4],
                before.textured.positions[source * 4..source * 4 + 4]
            );
            assert_eq!(
                mesh.textured.uvs[survivor * 4..survivor * 4 + 4],
                before.textured.uvs[source * 4..source * 4 + 4]
            );
        }
        assert_eq!(mesh.footing_heights, before.footing_heights);
        assert_eq!(mesh.authored_cells, before.authored_cells);
        assert!(mesh.solid.positions.iter().all(|p| p[1] == 0.0));
        for p in &before.textured.positions {
            assert!(mesh.solid.positions.contains(p) || mesh.textured.positions.contains(p));
        }
    }

    #[test]
    fn port_water_finish_requires_native_cells_and_preserves_bollards_and_hull() {
        for source in [
            material_source("port", 0x0a, 0, 0, 0x14),
            material_source("port", 0x18, 3, 1, 0x14),
            material_source("port", 0x1c, 0, 2, 0x14),
        ] {
            assert_eq!(ground_material(&source), Some(GroundMaterial::Water));
        }
        for source in [
            material_source("port", 0x01, 0, 0, 0x01),
            material_source("port", 0x01, 1, 0, 0x02),
            material_source("port", 0x01, 0, 1, 0x11),
            material_source("port", 0x01, 1, 1, 0x12),
            material_source("port", 0x01, 0, 0, 0x14),
            material_source("port", 0x18, 0, 2, 0x14),
            material_source("port", 0x18, 0, 2, 0x2b),
            material_source("port", 0x0a, 4, 0, 0x14),
            material_source("unknown_mod", 0x0a, 0, 0, 0x14),
        ] {
            assert_eq!(ground_material(&source), None);
        }
    }

    #[test]
    fn harbor_water_underlays_use_existing_model_claims_and_keep_the_water_datum() {
        let frame = frame(
            5,
            1,
            vec![
                material_source("port", 0x0a, 0, 0, 0x14),
                material_source("port", 0x01, 0, 0, 0x01),
                material_source("port", 0x08, 0, 0, 0x22),
                material_source("port", 0x18, 0, 2, 0x2b),
                material_source("port", 0x3f, 0, 0, 0x05),
            ],
        );
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = geometry(&frame);
        let water = CellShape::Water.surface_height(g.tile_height);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![water, 0.0, 0.0, crate::port::SHIP_HEIGHT, 0.0],
            authored_cells: vec![
                None,
                Some("special:harbor-bollard"),
                Some("special:dock-railing"),
                Some("special:harbor-ferry"),
                None,
            ],
            ..Default::default()
        };
        for i in 0..5 {
            append_top(&mut mesh.textured, g.bounds(i, 0).into(), water, g.uv(0, 0));
            // Prop/ship faces and raised surfaces are never ground finishes.
            append_top(&mut mesh.textured, g.bounds(i, 0).into(), 4.0, g.uv(i, 0));
        }
        let before = mesh.clone();
        let mut no_claims = mesh.clone();
        no_claims.authored_cells.fill(None);
        polish_world_surfaces(&mut no_claims, "OlivinePort", &cells, &g, [0, 0]);
        assert_eq!(
            no_claims.textured.quad_count(),
            9,
            "only actual source water can finish without claims"
        );
        polish_world_surfaces(&mut mesh, "OlivinePort", &cells, &g, [0, 0]);
        assert_eq!(
            mesh.textured.quad_count(),
            6,
            "five object faces and the unknown underlay remain"
        );
        assert_eq!(mesh.footing_heights, before.footing_heights);
        assert_eq!(mesh.authored_cells, before.authored_cells);
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| (p[1] - water).abs() <= 0.0081)
        );
        for i in 0..4 {
            for p in &before.textured.positions[i * 8..i * 8 + 4] {
                assert!(
                    mesh.solid.positions.contains(p),
                    "water boundary changed: {p:?}"
                );
            }
        }
    }
    #[test]
    fn kanto_shore_rock_bed_uses_water_finish_only_with_complete_model_claim() {
        let frame = frame(
            3,
            1,
            vec![
                material_source("kanto", 0x01, 0, 0, 0x14),
                material_source("kanto", 0x19, 2, 0, 0x2a),
                material_source("kanto", 0x19, 2, 0, 0x2a),
            ],
        );
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = geometry(&frame);
        let water = CellShape::Water.surface_height(g.tile_height);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![water, 0.0, 0.0],
            authored_cells: vec![None, Some("world-exterior/kanto_boundary_shore"), None],
            ..Default::default()
        };
        for i in 0..3 {
            append_top(&mut mesh.textured, g.bounds(i, 0).into(), water, g.uv(0, 0));
            append_top(&mut mesh.textured, g.bounds(i, 0).into(), 4.0, g.uv(0, 0));
        }
        let before = mesh.clone();
        polish_world_surfaces(&mut mesh, "Route20", &cells, &g, [0, 0]);
        assert_eq!(
            mesh.textured.quad_count(),
            4,
            "raised faces and unclaimed rock underlay remain"
        );
        assert_eq!(mesh.footing_heights, before.footing_heights);
        assert_eq!(mesh.authored_cells, before.authored_cells);
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| (p[1] - water).abs() <= 0.0081)
        );
        for i in 0..2 {
            for p in &before.textured.positions[i * 8..i * 8 + 4] {
                assert!(
                    mesh.solid.positions.contains(p),
                    "native water boundary changed: {p:?}"
                );
            }
        }
    }
    #[test]
    fn route19_water_bed_requires_native_donor_complete_claim_and_rock_identity() {
        let frame = frame(
            3,
            1,
            vec![
                material_source("kanto", 0x43, 0, 0, 0x14),
                material_source("kanto", 0x13, 0, 0, 0x2a),
                material_source("kanto", 0x13, 0, 0, 0x2a),
            ],
        );
        let g = geometry(&frame);
        let water = CellShape::Water.surface_height(g.tile_height);
        let mut original = TerrainMeshData {
            footing_heights: vec![water, 0.0, 0.0],
            authored_cells: vec![None, Some("world-exterior/kanto_boundary_land"), None],
            ..Default::default()
        };
        for i in 0..3 {
            append_top(
                &mut original.textured,
                g.bounds(i, 0).into(),
                water,
                g.uv(0, 0),
            );
            append_top(
                &mut original.textured,
                g.bounds(i, 0).into(),
                0.0,
                g.uv(0, 0),
            );
        }
        for mode in 0..5 {
            let mut frame = frame.clone();
            let mut mesh = original.clone();
            let map = if mode == 1 { "Route10South" } else { "Route19" };
            if mode == 2 {
                frame.tiles[0].source.metatile_id = 0x44;
            }
            if mode == 3 {
                frame.tiles[1].source.tile_index = 0x2b;
            }
            if mode == 4 {
                mesh.authored_cells[1] = None;
            }
            polish_world_surfaces(
                &mut mesh,
                map,
                &frame.tiles.iter().collect::<Vec<_>>(),
                &g,
                [0, 0],
            );
            assert_eq!(
                mesh.textured.quad_count(),
                if mode == 0 { 4 } else { 5 },
                "mode {mode}"
            );
            assert_eq!(mesh.footing_heights, original.footing_heights);
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .all(|p| (p[1] - water).abs() < 0.001)
            );
        }
    }
}
