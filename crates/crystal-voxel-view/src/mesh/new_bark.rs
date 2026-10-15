//! Original modeled scenery for a source-verified connected Johto slice.
//!
//! The pack remains authoritative: source cells place scenery, the original
//! ground/footing stays in place, and this module never reads or edits collision.
use super::*;
use bevy::prelude::Vec2;
use crate::new_bark_models::{ModelKind, model};

const MAP: &str = "NewBarkTown";
pub(super) use crate::new_bark_models::supports_map;

/// Outdoor grove coverage is independent of the narrower verified building kit.
pub(super) fn supports_grove(map: &str) -> bool {
    map.starts_with("Route") || map.ends_with("City") || map.ends_with("Town")
        || matches!(map, "LakeOfRage" | "RuinsOfAlphOutside" | "SilverCaveOutside" | "IlexForest")
}


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
    append_building_with_samples(mesh,cells,shapes,geometry,map,placement,claimed,None)
}

pub(super) fn service_wall_color(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement,
    images: &TerrainImageSamples) -> Option<[[f32;4];2]> {
    let mut colors=std::collections::BTreeMap::<[u8;3],usize>::new();
    // Only the two complete brick courses above the door/plaque. Exclude
    // timber edge posts, roof paint, signage and the floor/doorway palette.
    for row in p.row+p.roof_rows..p.row+p.roof_rows+2 {
        for column in p.column+2..p.column+p.width-2 {
            for pixel in tile_rgba(images,cells[row*g.width+column]).ok()?.chunks_exact(4) {
                if pixel[3]==255 {*colors.entry([pixel[0],pixel[1],pixel[2]]).or_default()+=1;}
            }
        }
    }
    if !(2..=4).contains(&colors.len()) {return None;}
    // The colored brick field, rather than its pale mortar, supplies paint.
    // Neutral source walls remain neutral; no town-specific RGB is invented.
    let (&color,_)=colors.iter().max_by_key(|(c,n)| {
        (c.iter().max().unwrap()-c.iter().min().unwrap(),**n)
    })?;
    let (&mortar,_)=colors.iter().max_by_key(|(c,_)|
        u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722)?;
    let linear=|color:[u8;3]| {
        let rgb=color.map(|v| {let s=f32::from(v)/255.;if s<=0.04045 {s/12.92} else {((s+0.055)/1.055).powf(2.4)}});
        [rgb[0],rgb[1],rgb[2],1.]
    };
    Some([linear(color),linear(mortar)])
}

pub(super) fn append_building_with_samples(
    mesh: &mut TerrainMeshData, cells: &[&VisualTile], shapes: &[CellShape],
    geometry: &GridGeometry, map: &str, placement: BuildingPlacement,
    claimed: &mut [bool], images: Option<&TerrainImageSamples>,
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
    let source_colors=if matches!(descriptor.kind,ModelKind::House|ModelKind::Pokecenter|ModelKind::Mart) {
        if let Some(images)=images {
            let roof=modeled_exteriors::cabin_roof_palette(cells,geometry,placement,images);
            let colors=if descriptor.kind==ModelKind::House {
                roof.map(|roof|(roof,None))
            } else {
                roof.zip(service_wall_color(cells,geometry,placement,images))
                    .map(|(roof,wall)|(roof,Some(wall)))
            };
            let Some(colors)=colors else {
                // Unavailable source paint stays visible as its original drawing.
                // Never replace it with a guessed red roof or cream facade.
                for row in placement.row..placement.row+placement.height {
                    for column in placement.column..placement.column+placement.width {
                        let (w,e,n,s)=geometry.bounds(column,row);
                        append_top(&mut mesh.textured,[w,e,n,s],0.,geometry.uv(column,row));
                        claimed[row*geometry.width+column]=true;
                    }
                }
                return true;
            };
            Some(colors)
        } else {None}
    } else {None};
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
    if let Some((roof,wall))=source_colors {
        model(descriptor.kind).recolor_fitted_roof(&mut mesh.solid,cutaway_start,roof);
        if let Some(wall)=wall {
            model(descriptor.kind).recolor_fitted_service_wall(&mut mesh.solid,cutaway_start,wall);
        }
    }
    // Mark only the successfully appended building. The backing was emitted
    // above; later terrain, trees and props must keep their ordinary depth.
    mesh.rigid_buildings.push(super::rigid_buildings::Building {
        vertices: cutaway_start..mesh.solid.positions.len(),
        doorway: Vec2::new(west + descriptor.door_column as f32 * geometry.tile_width,
            south - geometry.tile_height * 0.5),
    });
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
        let z = south - unit * 0.35;
        // Use the same compact notice panel as every other verified route
        // sign. Source ownership, ground sample and interactions stay here.
        let model = crate::exterior_models::model(crate::exterior_models::Kind::RouteSign);
        model.append_fitted(
            &mut mesh.solid,
            [cx - unit * 0.825, cx + unit * 0.825,
                z - unit * 0.12, z + unit * 0.12],
            0.0,
            unit * 2.0 / (model.max[1] - model.min[1]),
            None,
        );
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
        0x3c => &[(0, 0)],
        0x3d => &[(2, 0)],
        0x3e => &[(0, 2)],
        0x3f => &[(2, 2)],
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
    let origins: &[(u8, u8)] = match (first.tileset_id.as_ref(), first.metatile_id) {
        ("johto", block) => mixed_tree_origins(block),
        ("johto_modern", 0x2f) => &[(0, 0), (2, 0)],
        ("johto_modern", 0x3b) => &[(0, 2), (2, 2)],
        _ => &[],
    };
    if !origins.contains(&(first.subtile_column, first.subtile_row)) {
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
    if !supports_grove(map) {
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

pub(super) fn modeled_tree_ground(
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    placement: TreePlacement,
) -> Option<usize> {
    if supports_grove(map) && complete_mixed_tree(cells, geometry, placement) {
        return grove_ground_sample(cells, shapes, geometry, placement);
    }
    // Ilex uses its own source drawing, but the same production grove recipe
    // and shared geometry as Johto. Validate all four cells before claiming it.
    if map == "IlexForest" && placement.width == 2 && placement.height == 2
        && placement.column + 2 <= geometry.width && placement.row + 2 <= geometry.height
        && (0..2).all(|y| (0..2).all(|x| {
            let source = &cells[(placement.row + y) * geometry.width + placement.column + x].source;
            source.tileset_id.as_ref() == "forest"
                && source.tile_index == 0x26 + y as u16 * 0x10 + x as u16
        }))
    {
        return modeled_exteriors::tree_ground(map, cells, shapes, geometry, placement);
    }
    if !supports_grove(map)
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
    grove_ground_sample(cells, shapes, geometry, placement)
}

fn grove_ground_sample(cells: &[&VisualTile], shapes: &[CellShape],
    geometry: &GridGeometry, placement: TreePlacement) -> Option<usize> {
    let tileset=&cells[placement.row*geometry.width+placement.column].source.tileset_id;
    cells.iter().zip(shapes).enumerate()
        .filter(|(_, (tile,shape))| tile.source.tileset_id==*tileset
            && tile.source.tile_index==placement.ground_tile_index
            && (matches!(shape,CellShape::Flat)
                || matches!(shape,CellShape::RaisedTop {solid:SolidKind::Bank,..})))
        .min_by_key(|(_, (tile,shape))| (!matches!(shape,CellShape::Flat),tile.row,tile.column))
        .map(|(i,_)|i)
}

/// Reserve complete source trees and signboards that can actually be replaced.
/// Otherwise preserve the existing profile, including clipped views with no ground sample.
pub(super) fn preferred_scenery_cells(
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
) -> Vec<bool> {
    let mut reserved = vec![false; cells.len()];
    if !supports_grove(map) {
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
    for placement in sign_placements(cells, &shapes, geometry)
        .into_iter().filter(|_| supports_map(map))
    {
        for y in 0..2 {
            for x in 0..2 {
                reserved[(placement.row + y) * geometry.width + placement.column + x] = true;
            }
        }
    }
    // Route29's legacy live profile contains one pixel tuft per source phase.
    // Complete modeled grass must claim those exact cells before masking.
    if supports_map(map) && authored_ground_cell(cells, &shapes, 0x05).is_some() {
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
pub(super) fn grove_neighbors(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    map: &str,
    p: TreePlacement,
) -> [bool; 4] {
    let occupied = mesh.tree_neighborhood.get_or_insert_with(|| {
        let mut occupied = vec![false; g.width * g.height];
        for plot in tree_placements(map, cells, g) {
            for row in plot.row..(plot.row + plot.height).min(g.height) {
                for column in plot.column..(plot.column + plot.width).min(g.width) {
                    occupied[row * g.width + column] = true;
                }
            }
        }
        occupied
    });
    let contains = |x: i32, y: i32| {
        x >= 0 && y >= 0 && (x as usize) < g.width && (y as usize) < g.height
            && occupied[y as usize * g.width + x as usize]
    };
    let x = p.column as i32;
    let y = p.row as i32;
    [
        contains(x - 1, y + p.height as i32 / 2),
        contains(x + p.width as i32, y + p.height as i32 / 2),
        contains(x + p.width as i32 / 2, y - 1),
        contains(x + p.width as i32 / 2, y + p.height as i32),
    ]
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
    let Some(grove) = claim_grove(mesh, cells, shapes, geometry, map, placement, claimed, map_origin) else {
        return false;
    };
    grove.append(mesh);
    true
}

/// Retain source placement parameters while the rest of the terrain establishes
/// bank claims. Runtime construction expands one complete grove at a time.
#[derive(Clone)]
pub(super) struct GroveRecipe {
    pub(super) family: crate::foliage::GroveFamily,
    pub(super) bounds: [f32; 4],
    pub(super) base_height: f32,
    pub(super) tile_height: f32,
    pub(super) coordinates: [i32; 2],
    pub(super) neighbors: [bool; 4],
    pub(super) coverage_fade: bool,
}
impl GroveRecipe {
    pub(super) fn coordinates_for_trace(&self) -> [i32;2] { self.coordinates }
    pub(super) fn conservative_bounds(&self) -> (bevy::prelude::Vec3, bevy::prelude::Vec3) {
        let [w, e, n, s] = self.bounds;
        let sparse = self.neighbors.iter().all(|&n| n)
            && (self.coordinates[0].rem_euclid(4) != 0 || self.coordinates[1].rem_euclid(4) != 0);
        // Understory stems reach at most 0.396 tile, including their leaf
        // width, from centers inside 0.29 plot spacing. Do not treat those
        // short fern plots as full-height canopies when selecting visibility.
        let spacing = (e - w).min(s - n);
        let understory_margin = (self.tile_height * 0.5 - spacing * 0.2).max(0.);
        let margin = if sparse { understory_margin }
            else if self.family == crate::foliage::GroveFamily::JohtoPine {
                // Pine generation proportionally fits every crown vertex
                // inside a 0.75-spacing margin. Keep the understory bound
                // and allow the bank lip's <= 0.113-tile recession as well.
                (spacing * 0.75 + self.tile_height * 0.125).max(understory_margin)
            } else { (e - w).max(s - n) * 2. };
        (bevy::prelude::Vec3::new(w - margin, self.base_height - self.tile_height, n - margin),
         bevy::prelude::Vec3::new(e + margin, self.base_height + self.tile_height * if sparse { 1. } else { 9. }, s + margin))
    }
    pub(super) fn prepare_shared(self) -> Vec<crate::occluder_fade::PreparedGroup> {
        let mut groups = crate::shared_groves::prepare_family(self.bounds, self.base_height, self.tile_height,
            self.coordinates, self.neighbors, self.family);
        crate::occluder_fade::set_coverage_fade(&mut groups, self.coverage_fade);
        groups
    }
    pub(super) fn prepare_instanced(self, library: &crate::forest_instances::Library) -> Vec<crate::occluder_fade::PreparedGroup> {
        let mut groups = library.prepare(self.bounds, self.base_height,
            self.tile_height, self.coordinates, self.neighbors, self.family);
        crate::occluder_fade::set_coverage_fade(&mut groups, self.coverage_fade);
        groups
    }
    pub(super) fn on_terrace(mut self, edges: &[crystal_render_api::VisualJumpLedge],
        g: &GridGeometry, origin: [i32;2]) -> Self {
        let [w,e,n,s]=self.bounds;
        self.base_height += super::terraces::Field::new(edges,g,origin)
            .height(bevy::prelude::Vec2::new((w+e)*0.5,(n+s)*0.5));
        self
    }
    pub(super) fn append(self, mesh: &mut TerrainMeshData) {
        crate::foliage::append_family_grove(mesh, self.bounds, self.base_height,
            self.tile_height, self.coordinates, self.neighbors, self.family);
    }
}

pub(super) fn claim_grove(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    geometry: &GridGeometry,
    map: &str,
    placement: TreePlacement,
    claimed: &mut [bool],
    map_origin: [i32; 2],
) -> Option<GroveRecipe> {
    let Some(ground) = modeled_tree_ground(map, cells, shapes, geometry, placement) else {
        return None;
    };
    if (0..placement.height).any(|y| {
        (0..placement.width)
            .any(|x| claimed[(placement.row + y) * geometry.width + placement.column + x])
    }) {
        return None;
    }
    let ground_start = mesh.textured.positions.len();
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
    // A borrowed atlas cell supplies paint, not the elevation of this plot.
    // In a connected halo its first lawn sample may live on a raised bank.
    for p in &mut mesh.textured.positions[ground_start..] { p[1]=placement.base_height; }
    let (west, _, north, _) = geometry.bounds(placement.column, placement.row);
    let width = placement.width as f32 * geometry.tile_width;
    let south = north + placement.height as f32 * geometry.tile_height;
    let depth = width.min(placement.height as f32 * geometry.tile_height);
    let coordinates = [
        map_origin[0] + placement.column as i32,
        map_origin[1] + placement.row as i32,
    ];
    let neighbors = grove_neighbors(mesh, cells, geometry, map, placement);
    let family=crate::foliage::GroveFamily::from_source(map,
        cells[placement.row*geometry.width+placement.column].source.tileset_id.as_ref());
    mark_authored_rect(
        mesh,
        geometry,
        [
            placement.column,
            placement.row,
            placement.width,
            placement.height,
        ],
        family.label(),
    );
    Some(GroveRecipe {
        family,
        bounds: [west, west + width, south - depth, south],
        base_height: placement.base_height,
        tile_height: geometry.tile_height
            * if complete_mixed_tree(cells, geometry, placement) { 0.5 } else { 1. },
        coordinates,
        neighbors,
        coverage_fade: true,
    })
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

/// A modeled tree's ground replaces its original hull. Preserve the land's
/// exposed water contact even though ordinary exposed-side meshing skips it.
pub(super) fn append_grove_water_contacts(
    mesh: &mut TerrainMeshData, cells: &[&VisualTile], g: &GridGeometry, origin: [i32; 2],
) {
    let bottom = CellShape::Water.surface_height(g.tile_height);
    for (i, tile) in cells.iter().enumerate() {
        if !matches!(mesh.authored_cells.get(i).copied().flatten(),
            Some("johto/tree" | "johto/small_tree" | "forest/ancient_tree" | "world-exterior/kanto_canopy"
                | "world-exterior/park_tree"))
            || mesh.footing_heights.get(i).is_none_or(|h| h.abs() > 0.001)
            || !matches!(tile.source.tileset_id.as_ref(),
                "johto" | "johto_modern" | "kanto" | "forest" | "park" | "battle_tower_outside")
        { continue; }
        let x = i % g.width;
        let y = i / g.width;
        let (w, e, n, s) = g.bounds(x, y);
        for direction in Direction::ALL {
            let (dx, dy) = direction.offset();
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= g.width as isize || ny >= g.height as isize { continue; }
            let neighbor = &cells[ny as usize * g.width + nx as usize].source;
            if neighbor.tileset_id != tile.source.tileset_id
                || !matches!(shape_for_source(neighbor), CellShape::Water)
            { continue; }
            let (p, normal) = match direction {
                Direction::North => ([[w,bottom,n],[w,0.,n],[e,0.,n],[e,bottom,n]], [0.,0.,-1.]),
                Direction::South => ([[e,bottom,s],[e,0.,s],[w,0.,s],[w,bottom,s]], [0.,0.,1.]),
                Direction::West => ([[w,bottom,s],[w,0.,s],[w,0.,n],[w,bottom,n]], [-1.,0.,0.]),
                Direction::East => ([[e,bottom,n],[e,0.,n],[e,0.,s],[e,bottom,s]], [1.,0.,0.]),
            };
            // Reuse the same mineral wall and pigment as native shoreline
            // terrain. Its perimeter stays fixed to the existing floor/water.
            outdoor_stone::append_canyon(&mut mesh.solid, p, normal, g, origin);
        }
    }
}

/// Original folded-blade tuft, emitted only for the verified complete Johto
/// grass-cell identity. It changes presentation, never encounter/collision data.
pub(super) fn grass_source(source: &VisualTileSource) -> bool {
    source.tileset_id.as_ref() == "johto"
        && source.metatile_id == 0x03
        && source.tile_index == 0x04
        && source.subtile_column < 4
        && source.subtile_row < 4
}

#[cfg(test)]
pub(super) fn append_grass(
    target: &mut SurfaceMeshData,
    map: &str,
    tile: &VisualTile,
    geometry: &GridGeometry,
    base_height: f32,
    map_origin: [i32; 2],
) -> bool {
    append_connected_grass(target, map, tile, geometry, base_height, map_origin, [false; 4])
}

pub(super) fn append_connected_grass(
    target: &mut SurfaceMeshData,
    map: &str,
    tile: &VisualTile,
    geometry: &GridGeometry,
    base_height: f32,
    map_origin: [i32; 2],
    joined: [bool; 4],
) -> bool {
    if !supports_map(map) || !grass_source(&tile.source) {
        return false;
    }
    let (w, e, n, s) = geometry.bounds(tile.column as usize, tile.row as usize);
    crate::grass_blades::append_patch(
        target,
        [w, e, n, s],
        base_height,
        crate::grass_blades::PatchKind::Meadow.height(geometry.tile_height),
        [
            map_origin[0] + tile.column as i32,
            map_origin[1] + tile.row as i32,
        ],
        !foliage_lod(geometry, (w + e) * 0.5, (n + s) * 0.5, 10.0),
        joined,
    );
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GroundMaterial {
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

pub(super) fn ice_path_ground_material(source: &VisualTileSource) -> Option<GroundMaterial> {
    if source.tile_index == 0x12 {
        let (x, y) = (source.subtile_column, source.subtile_row);
        let native_grain = match source.metatile_id {
            0x10 => matches!((x, y), (3, 1) | (1, 3)),
            0x12 => matches!((x, y), (0, 0) | (3, 0)),
            0x37 => matches!((x, y), (1, 2) | (1, 3)),
            0x38 => matches!((x, y), (0, 2) | (1, 3)),
            _ => false,
        };
        return native_grain.then_some(GroundMaterial::Stone);
    }
    // The fifth dotted ground variation appears at these exact native
    // positions. It is grain beneath props and openings, not another prop.
    if source.tile_index == 0xab {
        let phase = match source.metatile_id {
            0x07 => Some((1, 0)),
            0x12 => Some((1, 1)),
            0x1a => Some((0, 3)),
            0x24 => Some((3, 2)),
            0x29 => Some((1, 3)),
            0x37 => Some((3, 1)),
            _ => None,
        };
        return (phase == Some((source.subtile_column, source.subtile_row)))
            .then_some(GroundMaterial::Stone);
    }
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
                ("kanto", 0x11) if source.metatile_id == 0x29
                    && source.subtile_column < 2 && source.subtile_row < 4 =>
                {
                    Some(GroundMaterial::Stone)
                }
                ("johto" | "johto_modern" | "forest" | "battle_tower_outside", 0x05)
                | ("kanto", 0x2c)
                | ("park", 0x01) => Some(GroundMaterial::Lawn),
                ("johto" | "johto_modern" | "battle_tower_outside", 0x06)
                // Kanto's quiet path alternates blank $23 and speckled $39.
                // Both samples are the same ground; finishing only the
                // speckles leaves pale source-cell patches across the path.
                | ("kanto", 0x0d | 0x23 | 0x39) => Some(GroundMaterial::Path),
                ("johto_modern", 0x2f) if matches!(source.metatile_id, 0x06 | 0x66 | 0x77) => {
                    Some(GroundMaterial::Pavers)
                }
                ("park", _) if crate::park::masonry_ground(source) => Some(GroundMaterial::Pavers),
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
pub(super) fn meadow_noise(x: f32, z: f32) -> f32 {
    let (ix, iz) = (x.floor() as i32, z.floor() as i32);
    let (tx, tz) = (x - x.floor(), z - z.floor());
    let (tx, tz) = (tx * tx * (3.0 - 2.0 * tx), tz * tz * (3.0 - 2.0 * tz));
    let a = lattice(ix, iz) * (1.0 - tx) + lattice(ix + 1, iz) * tx;
    let b = lattice(ix, iz + 1) * (1.0 - tx) + lattice(ix + 1, iz + 1) * tx;
    a * (1.0 - tz) + b * tz - 0.5
}

fn ground_color(material: GroundMaterial, p: [f32; 3], geometry: &GridGeometry) -> [f32; 4] {
    ground_color_with_path_palette(material, p, geometry, None)
}

fn ground_color_with_path_palette(
    material: GroundMaterial,
    p: [f32; 3],
    geometry: &GridGeometry,
    path_palette: Option<[f32; 3]>,
) -> [f32; 4] {
    let (x, z) = (p[0] / geometry.tile_width, p[2] / geometry.tile_height);
    let broad = meadow_noise(x / 5.0, z / 5.0);
    let fine = meadow_noise(x / 1.7, z / 1.7);
    let (base, variation) = match material {
        GroundMaterial::Lawn => ([0.24, 0.37, 0.105], broad * 0.19 + fine * 0.085),
        GroundMaterial::Path => (path_palette.unwrap_or([0.68, 0.52, 0.31]),
            broad * 0.195 + fine * 0.100),
        GroundMaterial::Water => ([0.095, 0.30, 0.32], broad * 0.036 + fine * 0.010),
        GroundMaterial::Shore => ([0.45, 0.46, 0.37], broad * 0.032 + fine * 0.024),
        GroundMaterial::Bank => ([0.39, 0.37, 0.29], broad * 0.035 + fine * 0.020),
        GroundMaterial::Pavers => ([0.53, 0.51, 0.45], broad * 0.035 + fine * 0.015),
        GroundMaterial::Stone => ([0.35, 0.36, 0.33], broad * 0.055 + fine * 0.020),
        GroundMaterial::Ice => ([0.38, 0.56, 0.61], broad * 0.040 + fine * 0.015),
    };
    [
        (base[0] + variation).clamp(0.0, 1.0),
        (base[1] + variation).clamp(0.0, 1.0),
        (base[2] + variation).clamp(0.0, 1.0),
        1.0,
    ]
}

pub(super) fn sampled_cell(uvs: &[[f32; 2]], geometry: &GridGeometry) -> Option<usize> {
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

pub(super) fn shallow_bank_source(source: &VisualTileSource) -> bool {
    if !matches!(source.tileset_id.as_ref(), "johto" | "johto_modern")
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
        0x4c | 0x4e => {
            if x == 0 {
                0x3b
            } else {
                if source.metatile_id == 0x4c { 0x06 } else { 0x05 }
            }
        }
        0x4d | 0x4f => {
            if x == 3 {
                0x3d
            } else {
                if source.metatile_id == 0x4d { 0x06 } else { 0x05 }
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

/// Require the complete native jump-ledge drawing, including $2f's flat
/// eastern half. A clipped or changed block cannot authorize a bank finish.
fn complete_kanto_shallow_bank(index: usize, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    let source = &cells[index].source;
    if source.tileset_id.as_ref() != "kanto"
        || !matches!(source.metatile_id, 0x07 | 0x1a | 0x2f | 0x58..=0x5b)
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
                let expected = if matches!(source.metatile_id, 0x58..=0x5b) {
                    let west = matches!(source.metatile_id, 0x58 | 0x5a);
                    let side = if west { x == 0 } else { x == 3 };
                    if matches!(source.metatile_id, 0x5a | 0x5b) && y == 3 {
                        if side { if west { 0x36 } else { 0x34 } } else { 0x37 }
                    } else if side { if west { 0x27 } else { 0x24 } } else { 0x2c }
                } else if y < 3 {
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
    // The native bank also retains a flat source underlay beneath its front
    // course. Its old $37/$34 ink remained visible along the finished 3D lip.
    // Replace only this exact, complete bank-cell quad with its cap ground.
    if normal == [0., 1., 0.] && matches!(cells[sample].source.tile_index, 0x37 | 0x34 | 0x36 | 0x27 | 0x24) {
        let (w, e, n, s) = g.bounds(world % g.width, world / g.width);
        let (u0, u1, v0, v1) = g.uv(sample % g.width, sample / g.width);
        if positions == [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]]
            && uvs == [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]
            && shape_for_source(&cells[world].source).surface_height(g.tile_height) == height
        {
            return Some(if cells[world].source.metatile_id == 0x1a {
                GroundMaterial::Path
            } else { GroundMaterial::Lawn });
        }
    }
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
            && (cells[sample].source.subtile_row < 3
                || matches!(cells[sample].source.metatile_id, 0x58 | 0x59)))
            .then_some(match tile {
                0x2c => GroundMaterial::Lawn,
                0x39 => GroundMaterial::Path,
                _ => return None,
            });
    }
    if !matches!(tile, 0x37 | 0x34 | 0x36 | 0x27 | 0x24)
        && !(tile == 0x2c && matches!(cells[sample].source.metatile_id, 0x58..=0x5b)) {
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
    if !matches!(sample.tileset_id.as_ref(), "johto" | "johto_modern") {
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

fn owned_rock_surface(positions: &[[f32; 3]], normal: [f32; 3],
    cells: &[&VisualTile], geometry: &GridGeometry) -> bool {
    let center = positions.iter().fold([0.0; 3], |mut sum, p| {
        for axis in 0..3 { sum[axis] += p[axis] * 0.25; }
        sum
    });
    cell_at(geometry, center[0] - normal[0] * 0.01,
        center[2] - normal[2] * 0.01).is_some_and(|i| {
            let source = &cells[i].source;
            matches!(source.tileset_id.as_ref(), "johto" | "johto_modern")
                && source.metatile_id == 0x0a
                && source.subtile_column < 4 && source.subtile_row < 4
                && matches!(shape_for_source(source), CellShape::RaisedTop {
                    solid: SolidKind::Bank, .. })
        })
}

/// Finish the existing Forest ledge hull, including its borrowed cap sample.
/// Its source classifier already distinguishes the ramp opening and side courses.
fn forest_bank_surface(
    positions: &[[f32; 3]], normal: [f32; 3], sample: &VisualTileSource,
    cells: &[&VisualTile], g: &GridGeometry, claims: &[Option<&str>],
) -> Option<GroundMaterial> {
    if sample.tileset_id.as_ref() != "forest" { return None; }
    let center = positions.iter().fold(Vec3::ZERO, |sum,p| sum+Vec3::from_array(*p)) * 0.25;
    let i = cell_at(g,center.x-normal[0]*g.tile_width*0.001,
        center.z-normal[2]*g.tile_height*0.001)?;
    if claims.get(i).is_some_and(Option::is_some) { return None; }
    let shape = crate::forest::forest_ledge_shape(&cells[i].source)?;
    if !matches!(shape,CellShape::RaisedTop {solid:SolidKind::Bank,..}
        | CellShape::LedgeBand {..} | CellShape::RampNorth {..}) { return None; }
    let maximum = g.tile_height*crate::profile::JUMP_LEDGE_HEIGHT/SOURCE_TILE_HEIGHT;
    if positions.iter().any(|p|p[1] < -0.001 || p[1]>maximum+0.001) { return None; }
    Some(if normal[1]>0.01 {GroundMaterial::Lawn} else {GroundMaterial::Bank})
}

fn ordinary_lawn_map(map: &str) -> bool {
    map.strip_prefix("Route").is_some_and(|n|
        !n.is_empty() && n.bytes().all(|c|c.is_ascii_digit()))
        || map.ends_with("Town") || map.ends_with("City")
        || matches!(map,"LakeOfRage"|"RuinsOfAlphOutside"|"SilverCaveOutside")
}

fn lawn_water_bank(
    map: &str, positions: &[[f32;3];4], normal: [f32;3], sample: &VisualTileSource,
    cells: &[&VisualTile], g: &GridGeometry, authored: &[Option<&str>],
) -> bool {
    let outdoor = ordinary_lawn_map(map);
    let land_material = ground_material(sample);
    if !outdoor || normal[1].abs()>0.001
        || !matches!(sample.tileset_id.as_ref(),"johto"|"johto_modern")
        || !matches!(land_material, Some(GroundMaterial::Lawn | GroundMaterial::Path)) {return false;}
    let center=positions.iter().map(|p|Vec3::from_array(*p)).sum::<Vec3>()/4.;
    let n=Vec3::from_array(normal);
    if n.length_squared()<0.99 {return false;}
    let Some(land)=cell_at(g,center.x-normal[0]*g.tile_width*0.001,
        center.z-normal[2]*g.tile_height*0.001) else {return false;};
    let Some(water)=cell_at(g,center.x+normal[0]*g.tile_width*0.001,
        center.z+normal[2]*g.tile_height*0.001) else {return false;};
    let low=CellShape::Water.surface_height(g.tile_height);
    let epsilon=g.tile_height*0.0001;
    cells[land].source.tileset_id==sample.tileset_id
        && ground_material(&cells[land].source)==land_material
        && authored.get(land).is_some_and(|claim|claim.is_none())
        && cells[water].source.tileset_id==sample.tileset_id
        && matches!(shape_for_source(&cells[water].source),CellShape::Water)
        && positions.iter().all(|p|p[1]>=low-epsilon && p[1]<=epsilon)
        && positions.iter().any(|p|(p[1]-low).abs()<epsilon)
        && positions.iter().any(|p|p[1].abs()<epsilon)
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
    if let Some(material) = forest_bank_surface(positions,normal,sample,cells,geometry,authored_cells) {
        return Some(material);
    }
    if (map.starts_with("Route") || map.ends_with("Town") || map.ends_with("City"))
        && owned_rock_surface(positions, normal, cells, geometry) {
        return Some(GroundMaterial::Stone);
    }
    if normal[1] > 0.999 {
        let center = positions.iter().fold([0.0; 3], |mut sum, p| {
            for axis in 0..3 { sum[axis] += p[axis] * 0.25; }
            sum
        });
        if let Some(i) = cell_at(geometry, center[0], center[2]) {
            let owned = &cells[i].source;
            let shape = shape_for_source(owned);
            let height = shape.surface_height(geometry.tile_height);
            if shallow_bank_source(owned)
                && height > 0.0
                && positions.iter().all(|p| (p[1] - height).abs() < 0.001)
                && matches!(shape, CellShape::RaisedTop { solid: SolidKind::Bank, .. }
                    | CellShape::LedgeBand { .. })
            {
                let top = match shape {
                    CellShape::LedgeBand { top_tile_index, .. } => top_tile_index,
                    _ => match owned.tile_index { 0x3b | 0x3d => 0x05, tile => tile },
                };
                if let Some(material) = match top {
                    0x05 => Some(GroundMaterial::Lawn),
                    0x06 => Some(GroundMaterial::Path),
                    _ => None,
                } {
                    return Some(material);
                }
            }
        }
    }
    if (map.starts_with("Route") || map.ends_with("Town") || map.ends_with("City"))
        && matches!(sample.tileset_id.as_ref(), "johto" | "johto_modern")
        && sample.metatile_id == 0x0a
        && sample.subtile_column < 4 && sample.subtile_row < 4
    {
        const ROCK_DRAWING: [u16; 16] = [
            0x2b, 0x2c, 0x2c, 0x2d, 0x3b, 0x3c, 0x3c, 0x3d,
            0x3b, 0x3c, 0x3c, 0x3d, 0x4b, 0x4c, 0x4c, 0x4d,
        ];
        if ROCK_DRAWING.contains(&sample.tile_index) {
            // The exact native drawing identifies this terrain hull even
            // where its folded side course probes the neighboring cliff.
            if positions.iter().all(|p| p[1] >= -0.05) {
                return Some(GroundMaterial::Stone);
            }
        }
    }
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
        // Complete park fixtures replace their basin/spray cells with a
        // same-atlas water underlay. Those cells originally contain spray,
        // so their raw source material cannot identify the replacement bed.
        let park_basin_bed = matches!(map, "NationalPark" | "NationalParkBugContest")
            && material == GroundMaterial::Water
            && sample.tileset_id.as_ref() == "park"
            && sample.tile_index == 0x14
            && authored_cells.get(world).copied().flatten() == Some("park/pond-basin")
            && matches!(cells[world].source.metatile_id, 0x3f | 0x33)
            && matches!(cells[world].source.tile_index, 0x5f | 0x80 | 0x81 | 0x90 | 0x91);
        // The closed Kanto shore-rock adapter proves a complete source drawing
        // and supplies a live same-atlas water sample at the native datum.
        let kanto_rock_bed = material == GroundMaterial::Water
            && sample.tileset_id.as_ref() == "kanto"
            && sample.tile_index == 0x14
            && authored_cells.get(world).copied().flatten()
                == Some("world-exterior/kanto_boundary_shore");
        let buoy_bed = material == GroundMaterial::Water
            && sample.tile_index == 0x14
            && sample.tileset_id == cells[world].source.tileset_id
            && authored_cells.get(world).copied().flatten() == Some(buoy_course::LABEL)
            && buoy_course::source(&cells[world].source);
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
                == [[0x2a, 0x2b], [0x3a, 0x3b]][cells[world].source.subtile_row as usize % 2]
                    [cells[world].source.subtile_column as usize % 2];
        // Other water and shore faces must agree with the actual source cell.
        if matches!(material, GroundMaterial::Water | GroundMaterial::Shore)
            && ground_material(&cells[world].source) != Some(material)
            && !harbor_bed
            && !park_basin_bed
            && !kanto_rock_bed
            && !buoy_bed
            && !route19_rock_bed
        {
            return None;
        }
        Some(material)
    } else if normal[1].abs() < epsilon
        && material == GroundMaterial::Pavers
        && sample.tileset_id.as_ref() == "park"
        && sample.tile_index == 0x15
        && world.is_some_and(|i| crate::park::masonry_ground(&cells[i].source))
        && positions.iter().all(|p| p[1] <= epsilon && p[1] >= water_height - epsilon)
    {
        Some(GroundMaterial::Pavers)
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
    trunk_center: Option<[f32; 2]>,
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
            let [tx, tz] = self.trunk_center.unwrap_or([cx, cz]);
            let trunk_x = (p[0]-tx)/((e-w)*0.49);
            let trunk_z = (p[2]-tz)/((s-n)*0.49);
            let trunk = (1.0-(trunk_x*trunk_x+trunk_z*trunk_z)*18.0).max(0.0)*0.22;
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

#[path = "cave_finish.rs"]
pub(super) mod cave_finish;
#[path = "city_paving.rs"]
mod city_paving;
#[path = "forest_floor.rs"]
mod forest_floor;
#[path = "ground_blend.rs"]
mod ground_blend;
#[path = "shore_blend.rs"]
mod shore_blend;
#[path = "ledge_finish.rs"]
pub(super) mod ledge_finish;
#[path = "ice_rim.rs"]
mod ice_rim;

/// Continue the verified neighboring floor beneath a rounded rock corner.
pub(super) fn append_rock_floor_patches(
    mesh: &mut SurfaceMeshData, map: &str, cells: &[&VisualTile],
    g: &GridGeometry, origin: [i32;2], patches: &[(usize,usize)],
) {
    let finish=GroundFinish::new(map,cells,g,origin);
    for &(cell,ground) in patches {
        let material=if cells[ground].source.tile_index==0x06 {GroundMaterial::Path} else {GroundMaterial::Lawn};
        let (w,e,n,s)=g.bounds(cell%g.width,cell/g.width);
        finish.append_surface(mesh,material,[[w,0.,n],[w,0.,s],[e,0.,s],[e,0.,n]]);
    }
}

struct GroundFinish<'a> {
    cerulean_pool: bool,
    path_palette: Option<[f32; 3]>,
    city_paving: Option<Vec<u8>>,
    forest: bool,
    sunlit_grove: bool,
    blend: ground_blend::GroundBlend,
    shore: shore_blend::ShoreBlend,
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
            let ground = if matches!(map, "NationalPark" | "NationalParkBugContest") {
                super::modeled_exteriors::park_tree_ground(map, cells, &shapes, geometry, placement)
            } else {
                modeled_tree_ground(map, cells, &shapes, geometry, placement)
                    .or_else(|| super::modeled_exteriors::tree_ground(
                        map, cells, &shapes, geometry, placement))
            };
            if ground.is_none() {
                continue;
            }
            let (west, _, north, _) = geometry.bounds(placement.column, placement.row);
            let width = placement.width as f32 * geometry.tile_width;
            let south = north + placement.height as f32 * geometry.tile_height;
            let depth = width.min(placement.height as f32 * geometry.tile_height);
            shadows.push(GroundShadow {
                bounds: [west, west + width, south - depth, south],
                tree: true,
                trunk_center: matches!(cells[placement.row * geometry.width + placement.column]
                    .source.tileset_id.as_ref(), "johto" | "johto_modern").then(|| {
                    let offset = crate::foliage::grove_trunk_offset(
                        [map_origin[0]+placement.column as i32, map_origin[1]+placement.row as i32],
                        width.min(depth),
                    );
                    [west+width*0.5+offset.x, south-depth*0.5+offset.y]
                }),
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
                trunk_center: None,
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
                trunk_center: None,
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
        let mut heights = vec![[0.0; 4]; cells.len()];
        let mut cap_shapes = shapes.clone();
        taper_johto_ledge_ends(cells, &mut cap_shapes, geometry.width);
        // Raised ledge caps use borrowed lawn/path art. Include those proven
        // caps in the seam field at their real tier, so dirt cap edges do not
        // remain hard rectangles while ordinary ground gets a soft transition.
        for (i, cell) in cells.iter().enumerate() {
            if !shallow_bank_source(&cell.source) { continue; }
            let top = match shapes[i] {
                CellShape::LedgeBand { top_tile_index, .. } => top_tile_index,
                CellShape::RaisedTop { solid: SolidKind::Bank, .. } => match cell.source.tile_index {
                    0x3b | 0x3d => 0x05,
                    tile => tile,
                },
                CellShape::Flat => match cell.source.tile_index {
                    0x3b | 0x3d | 0x4b | 0x4c | 0x4d => 0x05,
                    tile => tile,
                },
                _ => continue,
            };
            materials[i] = match top {
                0x05 => Some(GroundMaterial::Lawn),
                0x06 => Some(GroundMaterial::Path),
                _ => materials[i],
            };
            heights[i] = ledge_approach::cap_heights(&cell.source, cap_shapes[i], geometry.tile_height)
                .unwrap_or([shapes[i].surface_height(geometry.tile_height); 4]);
        }
        // Modeled trees retain their verified lawn underlay. This also blends
        // the rectangular source plot into a neighboring path.
        for shadow in shadows.iter().filter(|s| s.tree) {
            let [w, e, n, s] = shadow.bounds;
            for row in 0..geometry.height {
                for col in 0..geometry.width {
                    let (x0, x1, z0, z1) = geometry.bounds(col, row);
                    if x0 >= w && x1 <= e && z0 >= n && z1 <= s {
                        materials[row * geometry.width + col] = Some(GroundMaterial::Lawn);
                    }
                }
            }
        }
        let park_paving = matches!(map, "NationalPark" | "NationalParkBugContest");
        let city_paving = (map == "GoldenrodCity" || park_paving).then(|| {
            let eligible: Vec<_> = cells
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    if park_paving {
                        return crate::park::masonry_ground(&t.source)
                            && materials[i] == Some(GroundMaterial::Pavers);
                    }
                    city_paving::city_cell(
                        t.source.tileset_id.as_ref(),
                        [
                            i as i32 % geometry.width as i32 + map_origin[0],
                            i as i32 / geometry.width as i32 + map_origin[1],
                        ],
                    )
                })
                .collect();
            let local: Vec<_> = materials
                .iter()
                .enumerate()
                .map(|(i, m)| if eligible[i] { *m } else { None })
                .collect();
            let mut flags = city_paving::borders(&local, geometry);
            for (i, flag) in flags.iter_mut().enumerate() {
                if eligible[i] {
                    *flag |= if park_paving { 0x40 } else { 0x80 };
                }
            }
            flags
        });
        let blend = ground_blend::GroundBlend::at_corners(&materials, &heights, geometry);
        Self {
            cerulean_pool: map == "CeruleanGym",
            // New Bark's pale gravel has a cool green cast in the native
            // drawing. Keep Route29's earth and coastal sand separate.
            path_palette: (map == "NewBarkTown").then_some([0.50, 0.54, 0.46]),
            city_paving,
            sunlit_grove: ((map.starts_with("Route") || map == "LakeOfRage")
                && cells.iter().any(|t| matches!(t.source.tileset_id.as_ref(), "johto" | "johto_modern")))
                || matches!(map, "NationalPark" | "NationalParkBugContest"),
            forest: cells
                .iter()
                .any(|t| t.source.tileset_id.as_ref() == "forest"),
            blend,
            shore: shore_blend::ShoreBlend::new(map,cells,geometry),
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

    fn tree_darkness(&self, p: [f32; 3]) -> f32 {
        if let Some(index) = &self.shadow_cells {
            return cell_at(self.geometry, p[0], p[2]).map_or(0., |cell| {
                index[cell].iter().filter(|&&i| self.shadows[i].tree)
                    .map(|&i| self.shadows[i].darkness(p, self.geometry))
                    .fold(0., f32::max)
            });
        }
        self.shadows.iter().filter(|s| s.tree)
            .map(|s| s.darkness(p, self.geometry)).fold(0., f32::max)
    }

    fn color(&self, material: GroundMaterial, p: [f32; 3]) -> [f32; 4] {
        // Stable map coordinates prevent grain/grass from swimming when the
        // renderer's published grid scrolls by one source cell.
        let map_p = [
            p[0] - self.geometry.origin_x + self.map_origin[0] as f32 * self.geometry.tile_width,
            p[1],
            p[2] - self.geometry.origin_z + self.map_origin[1] as f32 * self.geometry.tile_height,
        ];
        let mut color = ground_color_with_path_palette(material, map_p, self.geometry,
            self.path_palette);
        // Water classification has already passed the source and datum guards.
        // Cerulean's blue pool is distinct from outdoor teal water; only its
        // color changes here, never its cells, height or animation ownership.
        if self.cerulean_pool && material == GroundMaterial::Water {
            color[0] = (color[0] - 0.025).clamp(0.,1.);
            color[1] = (color[1] - 0.080).clamp(0.,1.);
            color[2] = (color[2] + 0.420).clamp(0.,1.);
        }

        if self.forest && material == GroundMaterial::Lawn {
            let x = map_p[0] / self.geometry.tile_width;
            let z = map_p[2] / self.geometry.tile_height;
            let moss = meadow_noise(x * 0.18, z * 0.18);
            let litter = meadow_noise(x * 1.7 + 19.0, z * 1.7 - 7.0);
            let t = ((moss + 0.16) * 2.8).clamp(0.0, 1.0);
            let t = t * t * (3.0 - 2.0 * t);
            let soil = [0.31, 0.205, 0.105];
            let green = [0.13, 0.34, 0.075];
            for c in 0..3 {
                color[c] = (soil[c] * (1.0 - t) + green[c] * t) * (0.90 + litter * 0.18);
            }
        }
        if self.sunlit_grove && material == GroundMaterial::Bank {
            color[0] *= 0.80;
            color[1] *= 0.65;
            color[2] *= 0.40;
        }
        if matches!(material, GroundMaterial::Lawn | GroundMaterial::Path) {
            if let Some(mixed) = self.blend.color_with_path_palette(p, self.geometry,
                self.map_origin, self.path_palette) {
                color = mixed;
            }
            let mut darkness = self.darkness(p);
            if self.sunlit_grove {
                let grass = ((0.60 - color[0]) / 0.34).clamp(0.0, 1.0);
                let x = map_p[0] / self.geometry.tile_width;
                let z = map_p[2] / self.geometry.tile_height;
                // Several continuous scales give the forest floor moss,
                // dry soil and small sunlit breaks without a per-tile stencil.
                let moss = meadow_noise(x * 0.63 + 31.0, z * 0.63 - 13.0);
                let grain = meadow_noise(x * 3.1, z * 3.1) * 0.045;
                let soil = ((moss + 0.10) * 1.8).clamp(0.0, 0.65) * grass;
                let earth = [0.34, 0.29, 0.095];
                for c in 0..3 {
                    color[c] = color[c] * (1.0 - soil) + earth[c] * soil + grain;
                }
                let patch = meadow_noise(x * 1.3, z * 1.3)
                    + meadow_noise(x * 0.47 + 17.0, z * 0.47 - 9.0) * 0.55;
                let fleck = ((patch + 0.16) / 0.40).clamp(0.0, 1.0);
                let fleck = fleck * fleck * (3.0 - 2.0 * fleck);
                // Filtered canopy light changes only ground contact shading;
                // the dense trunk contact stays dark and cast shadows remain real.
                darkness = (darkness - 0.30).max(0.0) + darkness.min(0.30) * (1.0 - fleck * 0.48);
            }
            if matches!(material, GroundMaterial::Lawn | GroundMaterial::Path) {
                color=self.shore.color(p,self.geometry,self.map_origin,color);
            }
            for c in &mut color[..3] {
                *c *= 1.0 - darkness;
            }
        }
        color
    }

    fn park_canopy_darkness(&self, p: [f32; 3]) -> f32 {
        // The tree's lawn backing is restricted to its source plot. Canopy
        // shade is a separate presentation field and may fall on nearby
        // masonry without changing that backing or its source ownership.
        let cover = self.shadows.iter().filter(|s| s.tree)
            .filter_map(|s| {
                let [w, e, n, south] = s.bounds;
                let width = e - w;
                let depth = south - n;
                if width < self.geometry.tile_width * 2. { return None; }
                let x = (p[0] - ((w + e) * 0.5 - width * 0.36)) / (width * 0.80);
                let z = (p[2] - ((n + south) * 0.5 - depth * 0.44)) / (depth * 0.75);
                Some((1. - x * x - z * z).max(0.).powi(2))
            }).fold(0.0_f32, f32::max);
        if cover == 0. { return 0.; }
        let x = (p[0] - self.geometry.origin_x) / self.geometry.tile_width
            + self.map_origin[0] as f32;
        let z = (p[2] - self.geometry.origin_z) / self.geometry.tile_height
            + self.map_origin[1] as f32;
        let patch = meadow_noise(x * 1.3, z * 1.3)
            + meadow_noise(x * 0.47 + 17., z * 0.47 - 9.) * 0.55;
        let light = ((patch + 0.16) / 0.40).clamp(0., 1.);
        let light = light * light * (3. - 2. * light);
        cover * 0.22 * (1. - light * 0.70)
    }

    // Only the verified floor is tagged; accent meshes retain their own
    // pigments. A map-space projection survives scrolling and native steps.
    fn tag_floor(&self, mesh: &mut SurfaceMeshData, first: usize, material: GroundMaterial) {
        let role = match material {
            GroundMaterial::Path => 6.0,
            GroundMaterial::Lawn if self.forest => 7.0,
            GroundMaterial::Pavers => 8.0,
            GroundMaterial::Lawn => 9.0,
            _ => return,
        };
        for i in first..mesh.positions.len() {
            let p = mesh.positions[i];
            mesh.uvs[i] = [
                66000.0 + (p[0] - self.geometry.origin_x) / self.geometry.tile_width
                    + self.map_origin[0] as f32,
                -role * 10000.0 + (p[2] - self.geometry.origin_z) / self.geometry.tile_height
                    + self.map_origin[1] as f32,
            ];
        }
    }

    fn append_surface(
        &self,
        mesh: &mut SurfaceMeshData,
        material: GroundMaterial,
        positions: [[f32; 3]; 4],
    ) {
        let first_floor = mesh.positions.len();
        let [w, e, n, s] = face_bounds(&positions);
        let full_cell = (e - w - self.geometry.tile_width).abs() < 0.001
            && (s - n - self.geometry.tile_height).abs() < 0.001;
        if full_cell
            && matches!(material, GroundMaterial::Path | GroundMaterial::Pavers)
            && let Some(borders) = &self.city_paving
        {
            let flags = cell_at(self.geometry, (w + e) * 0.5, (n + s) * 0.5)
                .and_then(|i| borders.get(i))
                .copied()
                .unwrap_or(0);
            if flags & 0xc0 != 0 {
                // Park limestone uses staggered flags with worn light edges,
                // rather than the old two-cell grid of uniform gray squares.
                // Reuse the disjoint, world-anchored limestone partition;
                // source ownership and the original floor remain unchanged.
                let park = flags & 0x40 != 0;
                let first = mesh.positions.len();
                city_paving::append(
                    mesh,
                    positions,
                    self.geometry,
                    self.map_origin,
                    if park { GroundMaterial::Path } else { material },
                    if park { 0x40 } else { flags & 15 },
                );
                if park {
                    for i in first..mesh.positions.len() {
                        let shade = self.park_canopy_darkness(mesh.positions[i]);
                        for (c, gain) in [1.02, 0.96, 0.86].into_iter().enumerate() {
                            mesh.colors[i][c] *= 1. - shade * gain;
                        }
                    }
                }
                self.tag_floor(mesh, first_floor, GroundMaterial::Pavers);
                self.append_accents(mesh, material, positions);
                return;
            }
        }
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
            self.tag_floor(mesh, first_floor, material);
            self.append_accents(mesh, material, positions);
            return;
        }
        let level_rectangle = positions == [
            [w, positions[0][1], n], [w, positions[0][1], s],
            [e, positions[0][1], s], [e, positions[0][1], n],
        ];
        if !level_rectangle || !matches!(material, GroundMaterial::Lawn | GroundMaterial::Path) {
            append_quad_colors(
                mesh,
                positions,
                [0.0, 1.0, 0.0],
                [[0.0; 2]; 4],
                positions.map(|p| self.color(material, p)),
            );
            self.tag_floor(mesh, first_floor, material);
            return;
        }
        // Two subdivisions capture a soft contact shadow around a trunk
        // without decals, alpha blending, lifted floors or driver shadows.
        let subdivisions = if self.forest
            || (self.sunlit_grove
                && self
                    .shadow_cells
                    .as_ref()
                    .and_then(|cells| {
                        cell_at(self.geometry, (w + e) * 0.5, (n + s) * 0.5)
                            .and_then(|i| cells.get(i))
                    })
                    .is_some_and(|shadows| !shadows.is_empty()))
            || self.shore.near([(w+e)*0.5,positions[0][1],(n+s)*0.5],self.geometry)
            || self.blend.near(
                [(w + e) * 0.5, positions[0][1], (n + s) * 0.5],
                self.geometry,
            ) {
            6
        } else {
            2
        };
        let columns = ((e - w) / self.geometry.tile_width * subdivisions as f32).ceil().max(1.0) as usize;
        let rows = ((s - n) / self.geometry.tile_height * subdivisions as f32).ceil().max(1.0) as usize;
        for row in 0..rows {
            for column in 0..columns {
                let fraction_x = 1.0 / columns as f32;
                let fraction_z = 1.0 / rows as f32;
                let x0 = if column == 0 {
                    w
                } else {
                    w + (e - w) * column as f32 * fraction_x
                };
                let x1 = if column + 1 == columns {
                    e
                } else {
                    w + (e - w) * (column + 1) as f32 * fraction_x
                };
                let z0 = if row == 0 {
                    n
                } else {
                    n + (s - n) * row as f32 * fraction_z
                };
                let z1 = if row + 1 == rows {
                    s
                } else {
                    n + (s - n) * (row + 1) as f32 * fraction_z
                };
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
        self.tag_floor(mesh, first_floor, material);
        self.append_grain(mesh, material, positions);
    }

    fn append_accents(
        &self,
        mesh: &mut SurfaceMeshData,
        material: GroundMaterial,
        positions: [[f32; 3]; 4],
    ) {
        if positions.iter().any(|p| (p[1] - positions[0][1]).abs() > 0.001) {
            return;
        }
        let [w, _, n, _] = face_bounds(&positions);
        let column = ((w - self.geometry.origin_x) / self.geometry.tile_width).round() as i32
            + self.map_origin[0];
        let row = ((n - self.geometry.origin_z) / self.geometry.tile_height).round() as i32
            + self.map_origin[1];
        forest_floor::append_scatter(
            mesh, positions, self.geometry, [column, row],
            self.forest || self.sunlit_grove || self.darkness(positions[0]) > 0.1,
            material == GroundMaterial::Lawn,
        );
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
        if self.forest && material == GroundMaterial::Lawn {
            let canopy = (self.tree_darkness([(w+e)*0.5, positions[0][1], (n+s)*0.5])
                / 0.30).clamp(0., 1.);
            forest_floor::append_with_canopy(mesh, positions, self.geometry, [column, row], canopy);
            return;
        }
        self.append_accents(mesh, material, positions);
        // Irregular gravel and small lawn clumps. Stable, uneven positions
        // give grass a soft surface without changing the walking datum.
        let samples = if material == GroundMaterial::Lawn {
            8
        } else {
            3
        };
        for sample in 0..samples {
            let seed = lattice(column.wrapping_mul(7) + sample, row.wrapping_mul(13) + 83);
            let threshold = if material == GroundMaterial::Path {
                0.46
            } else {
                0.56
            };
            if seed < threshold {
                continue;
            }
            let cx = w + (e - w) * (0.16 + lattice(column + sample * 19, row + 109) * 0.68);
            let cz = n + (s - n) * (0.16 + lattice(column + 211, row + sample * 23) * 0.68);
            let angle = lattice(column + 41, row + sample * 31) * std::f32::consts::TAU;
            let rx = (e - w) * (0.018 + seed * 0.036);
            let rz = (s - n)
                * (if material == GroundMaterial::Path {
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
            let contrast = if seed > 0.76 { 1.07 } else { 0.87 };
            for c in &mut color[..3] {
                *c *= contrast;
            }
            append_solid_quad(mesh, p, [0.0, 1.0, 0.0], color);
            if material == GroundMaterial::Lawn {
                let root = Vec3::new(cx, y, cz);
                let height = self.geometry.tile_height * (0.040 + seed * 0.030);
                for blade in 0..3 {
                    let a = angle + blade as f32 * 2.094395;
                    let side = Vec3::new(a.cos(), 0., a.sin()) * self.geometry.tile_width * 0.023;
                    let lean = Vec3::new(-a.sin(), 0., a.cos()) * self.geometry.tile_width * 0.050;
                    let tip = root + Vec3::Y * height + lean;
                    let points = [
                        (root - side).to_array(),
                        (root + side).to_array(),
                        tip.to_array(),
                        (root + lean * 0.4 + Vec3::Y * height * 0.55).to_array(),
                    ];
                    let normal = (Vec3::from_array(points[1]) - Vec3::from_array(points[0]))
                        .cross(Vec3::from_array(points[2]) - Vec3::from_array(points[0]))
                        .normalize()
                        .to_array();
                    let leaf = [color[0] * 1.09, color[1] * 1.06, color[2] * 0.9, 1.];
                    append_solid_quad(mesh, points, normal, leaf);
                    append_solid_quad(
                        mesh,
                        [points[3], points[2], points[1], points[0]],
                        [-normal[0], -normal[1], -normal[2]],
                        leaf,
                    );
                }
            }
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

/// Finish source-authored terrain colors and low decorative ice-edge relief.
/// No cells are reclassified and footing is unchanged. Plain ground faces may
/// be subdivided for contact shading while preserving their boundary and datum.
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
    polish_world_surfaces_with_shapes(mesh, map, cells, geometry, map_origin, &[]);
}

pub(super) fn polish_world_surfaces_with_shapes(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    geometry: &GridGeometry,
    map_origin: [i32; 2],
    shapes: &[CellShape],
) {
    let finish = GroundFinish::new(map, cells, geometry, map_origin);
    let corners = if cave_crowns::enabled(map) && shapes.len() == cells.len() {
        cave_cliff::RockCorners::new_with_shapes(map, cells, shapes, geometry)
    } else {
        cave_cliff::RockCorners::new_on_map(map, cells, geometry)
    };
    let cave_trace = std::env::var_os("CRYSTAL_CAVE_RENDER_TRACE").is_some()
        && cells.first().is_some_and(|t| {
            matches!(
                t.source.tileset_id.as_ref(),
                "cave" | "dark_cave" | "ice_path"
            )
        });
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
        if cave_trace {
            eprintln!("cave finish skipped unfamiliar topology on {map}");
        }
        return;
    }
    let old = std::mem::take(&mut mesh.textured);
    let mut cave_faces = 0;
    let mut shallow_cave_faces = Vec::new();
    for base in (0..old.positions.len()).step_by(4) {
        let positions: [[f32; 3]; 4] = old.positions[base..base + 4].try_into().unwrap();
        let uvs: [[f32; 2]; 4] = old.uvs[base..base + 4].try_into().unwrap();
        let normal = old.normals[base];
        if let Some(i) = sampled_cell(&uvs, geometry).filter(|&i|
            lawn_water_bank(map, &positions, normal, &cells[i].source,
                cells, geometry, &mesh.authored_cells)) {
            let finished = if ground_material(&cells[i].source) == Some(GroundMaterial::Path) {
                ledge_finish::append_bare_soil(&mut mesh.solid, positions, normal, geometry, map_origin)
            } else {
                ledge_finish::append(&mut mesh.solid, positions, normal, geometry, map_origin)
            };
            if finished { continue; }
        }
        let center=positions.iter().map(|p|Vec3::from_array(*p)).sum::<Vec3>()/4.;
        let owned_x=((center.x-geometry.origin_x)/geometry.tile_width).floor() as isize;
        let owned_z=((center.z-geometry.origin_z)/geometry.tile_height).floor() as isize;
        let owned=(owned_x>=0 && owned_z>=0 && owned_x<geometry.width as isize
            && owned_z<geometry.height as isize)
            .then(||owned_z as usize*geometry.width+owned_x as usize);
        if map=="VictoryRoad" && normal[1]>0.3 && normal[1]<0.999
            && owned.is_some_and(|i| {
                let s=&cells[i].source;
                s.tileset_id.as_ref()=="cave" && (s.tile_index==0x16
                    || (matches!(s.metatile_id,0x12|0x30) && matches!(s.tile_index,0x36|0x37)))
            }) {
            let points=positions.map(Vec3::from_array);
            append_quad_colors(&mut mesh.solid,positions,normal,
                points.map(|p|cave_cliff::floor_uv(p,geometry,map_origin,false)),
                points.map(|p|cave_cliff::pigment([0.40,0.40,0.38,1.],p,geometry,map_origin,false)));
            continue;
        }
        if ledge_approach::obsolete_toe_wall(&positions, normal, cells, geometry) {
            continue;
        }
        if sampled_cell(&uvs, geometry)
            .is_some_and(|i| cave_shore::append(mesh, positions, normal, cells, geometry, i, map_origin))
        {
            continue;
        }
        if let Some(color)=sampled_cell(&uvs,geometry).and_then(|i|
            cave_finish::enclosing_color(&positions,normal,&cells[i].source,geometry)) {
            cave_cliff::quad_with_point(&mut mesh.solid,positions,color,geometry,map_origin,
                |p,g,o|corners.point(p,g,o),false);
            cave_faces+=1;
            continue;
        }
        if let Some(color)=sampled_cell(&uvs,geometry).and_then(|i|
            cave_finish::shallow_face(&positions,normal,&cells[i].source,geometry)) {
            shallow_cave_faces.push((positions,normal,color));
            cave_faces+=1;
            continue;
        }
        if sampled_cell(&uvs, geometry).is_some_and(|i| {
            cave_finish::append(
                &mut mesh.solid,
                positions,
                normal,
                &cells[i].source,
                geometry,
                map_origin,
            )
        }) {
            if let Some(i) = sampled_cell(&uvs, geometry) {
                cave_finish::floor_detail(
                    &mut mesh.solid,
                    positions,
                    normal,
                    &cells[i].source,
                    geometry,
                    map_origin,
                );
            }
            cave_faces += 1;
            continue;
        }
        if map=="VictoryRoad" && normal[1]>0.999 && owned.is_some_and(|i| {
            let s=&cells[i].source;
            s.tileset_id.as_ref()=="cave" && s.tile_index==0x16
        }) {
            if center.y>=crate::cave::CAVE_ROCK_HEIGHT*geometry.tile_height/SOURCE_TILE_HEIGHT {
                cave_cliff::quad_with_point(&mut mesh.solid,positions,[0.34,0.345,0.31,1.],geometry,map_origin,
                    |p,g,o|corners.point(p,g,o),false);
            } else {
                let points=positions.map(Vec3::from_array);
                append_quad_colors(&mut mesh.solid,positions,normal,
                    points.map(|p|cave_cliff::floor_uv(p,geometry,map_origin,false)),
                    points.map(|p|cave_cliff::pigment([0.40,0.40,0.38,1.],p,geometry,map_origin,false)));
            }
            continue;
        }
        if map != "HallOfFame"
            && sampled_cell(&uvs, geometry).is_some_and(|i| {
                let x = i % geometry.width;
                let y = i / geometry.width;
                // A borrowed atlas sample is paint, not an edge at a new location.
                if (positions[0][0] - (geometry.origin_x + x as f32 * geometry.tile_width)).abs()
                    > 0.001
                    || (positions[0][2] - (geometry.origin_z + y as f32 * geometry.tile_height))
                        .abs()
                        > 0.001
                {
                    return false;
                }
                let neighbors = std::array::from_fn(|j| {
                    let nx = x as isize+(j%3) as isize-1;
                    let ny = y as isize+(j/3) as isize-1;
                    if nx<0 || ny<0 || nx>=geometry.width as isize || ny>=geometry.height as isize {
                        return None;
                    }
                    let ni = ny as usize*geometry.width+nx as usize;
                    (mesh.authored_cells.get(ni).is_none_or(|claim| claim.is_none())
                        && ice_rim::has_edge(&cells[ni].source))
                        .then_some(&cells[ni].source)
                });
                ice_rim::append(
                    &mut mesh.solid,
                    positions,
                    normal,
                    &cells[i].source,
                    geometry,
                    map_origin,
                    neighbors,
                )
            })
        {
            continue;
        }
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
            if material == GroundMaterial::Water && normal[1] > 0.999
                && sampled_cell(&uvs,geometry).is_some_and(|i|
                    crate::cave_water::live_pool_water(cells[i],map))
            {
                // Keep authenticated pool water on its original atlas UVs
                // so the source-phase shader can draw continuous currents.
                // Painting it into the solid mesh discarded that clock and
                // left only static short ripple bars on a uniform plane.
                append_quad_colors(&mut mesh.textured,positions,normal,uvs,
                    old.colors[base..base+4].try_into().unwrap());
                continue;
            }
            let painted_start = mesh.solid.positions.len();
            if material == GroundMaterial::Shore
                && sampled_cell(&uvs, geometry).is_some_and(|i|
                    crate::forest::forest_shore_shape(&cells[i].source).is_some())
            {
                // The forest pond's exact rocky rim uses a damp native mineral
                // finish, sharing the existing closed water-face geometry.
                let first = mesh.solid.positions.len();
                outdoor_stone::append_coast(&mut mesh.solid, positions, normal,
                    geometry, map_origin, [[0.30,0.33,0.28],[0.18,0.21,0.16]]);
                if normal[1] > 0.999 {
                    let owner = sampled_cell(&uvs, geometry).unwrap();
                    let source = &cells[owner].source;
                    let (w,e,n,s) = geometry.bounds(owner % geometry.width, owner / geometry.width);
                    for vertex in first..mesh.solid.positions.len() {
                        let p = mesh.solid.positions[vertex];
                        let x = ((p[0]-w)/(e-w)).clamp(0.,1.);
                        let z = ((p[2]-n)/(s-n)).clamp(0.,1.);
                        let across = match source.tile_index {
                            0x1b => x, 0x19 => 1.-x, 0x2a => z,
                            0x09 => x*z, 0x0b => (1.-x)*z,
                            _ => 1.,
                        };
                        let mx = (p[0]-geometry.origin_x)/geometry.tile_width + map_origin[0] as f32;
                        let mz = (p[2]-geometry.origin_z)/geometry.tile_height + map_origin[1] as f32;
                        let rough = meadow_noise(mx*2.1+31.,mz*2.1-17.)*0.16;
                        let t = ((across+rough-0.12)/0.62).clamp(0.,1.);
                        let stone = t*t*(3.-2.*t);
                        let humus = finish.color(GroundMaterial::Lawn,p);
                        for c in 0..3 {
                            mesh.solid.colors[vertex][c] = humus[c]*(1.-stone)
                                + mesh.solid.colors[vertex][c]*stone;
                        }
                    }
                    // The dry shoulder shares the forest floor's material
                    // projection, keeping its pigment continuous at the seam.
                    finish.tag_floor(&mut mesh.solid, first, GroundMaterial::Lawn);
                    // Small stones and fallen leaves are retained in the dry
                    // source bank cell. Nothing is placed on swimmable water.
                    forest_floor::append_scatter(&mut mesh.solid, positions, geometry,
                        [owner as i32 % geometry.width as i32 + map_origin[0],
                            owner as i32 / geometry.width as i32 + map_origin[1]], true, false);
                }
                continue;
            }
            if material == GroundMaterial::Shore
                && sampled_cell(&uvs, geometry).is_some_and(|i|
                    matches!(cells[i].source.tileset_id.as_ref(), "johto" | "johto_modern"))
            {
                // Native Johto coasts are ochre rock, not a concrete strip.
                // Share the outdoor mineral finish on their cap and water face.
                outdoor_stone::append_canyon(&mut mesh.solid, positions, normal, geometry, map_origin);
                continue;
            }
            if material == GroundMaterial::Stone
                && (owned_rock_surface(&positions, normal, cells, geometry)
                    || sampled_cell(&uvs, geometry).is_some_and(|i|
                    (matches!(cells[i].source.tileset_id.as_ref(), "johto" | "johto_modern")
                        && cells[i].source.metatile_id == 0x0a)
                    || (cells[i].source.tileset_id.as_ref() == "kanto"
                        && cells[i].source.metatile_id == 0x29
                        && cells[i].source.tile_index == 0x11
                        && cells[i].source.subtile_column < 2
                        && cells[i].source.subtile_row < 4)))
            {
                // The byte-identical $0a drawing shares one mineral finish
                // across towns and routes, including coastal copies.
                // Mountain cap/face samples belong to the same ochre source
                // family. A map-name-dependent slate finish left dark gravel
                // strips through Blackthorn's otherwise joined canyon walls.
                outdoor_stone::append_canyon(&mut mesh.solid, positions, normal, geometry, map_origin);
                continue;
            }
            if map.starts_with("IcePath") && normal[1] > 0.999
                && positions.iter().all(|p| p[1].abs() < 0.001)
                && matches!(material, GroundMaterial::Stone | GroundMaterial::Ice)
                && sampled_cell(&uvs, geometry).is_some_and(|i|
                    cells[i].source.tileset_id.as_ref() == "ice_path")
            {
                ice_ground::append_blended(&mut mesh.solid, positions,
                    material == GroundMaterial::Ice, geometry, map_origin, cells);
                continue;
            }
            let center = positions.iter().fold([0.0;3], |mut a,p| {
                for i in 0..3 { a[i]+=p[i]*0.25; } a
            });
            // A bank run can borrow path paint for its grassy side verge.
            // Resolve the cap by its owned source cell, not the borrowed UV.
            let bank_verge = normal[1]>0.999 && positions.iter().all(|p|(p[1]-positions[0][1]).abs()<0.001)
                && cell_at(geometry,center[0],center[2]).is_some_and(|i| {
                    let source=&cells[i].source;
                    shallow_bank_source(source)
                        && matches!(source.tile_index,0x3b|0x3d)
                        && matches!(shape_for_source(source),CellShape::Flat
                            |CellShape::RaisedTop {solid:SolidKind::Bank,..}
                            |CellShape::LedgeBand {..})
                });
            if bank_verge {
                finish.append_surface(&mut mesh.solid,GroundMaterial::Lawn,positions);
                continue;
            }
            if material == GroundMaterial::Bank && normal[1]>0.999
                && positions.iter().all(|p| (p[1]-positions[0][1]).abs()<0.001)
                && sampled_cell(&uvs, geometry).is_some_and(|i|
                    shallow_bank_source(&cells[i].source))
            {
                finish.append_surface(&mut mesh.solid,GroundMaterial::Lawn,positions);
                continue;
            }
            if material == GroundMaterial::Bank {
                if sampled_cell(&uvs, geometry).is_some_and(|i|
                    forest_bank_surface(&positions, normal, &cells[i].source,
                        cells, geometry, &mesh.authored_cells) == Some(GroundMaterial::Bank))
                    && ledge_finish::append(&mut mesh.solid, positions, normal, geometry, map_origin)
                {
                    continue;
                }
                let owner = sampled_cell(&uvs, geometry).filter(|&i| {
                    cells[i].source.tileset_id.as_ref() == "johto"
                        && shallow_bank_source(&cells[i].source)
                });
                if sampled_cell(&uvs, geometry).is_some_and(|i|
                    complete_kanto_shallow_bank(i, cells, geometry))
                    && ledge_finish::append(&mut mesh.solid, positions, normal, geometry, map_origin)
                {
                    continue;
                }
                if let Some(owner) = owner {
                    let first = mesh.solid.positions.len();
                    if ledge_finish::append(&mut mesh.solid, positions, normal, geometry, map_origin) {
                        super::ledge_approach::mark_bank_surface(&mut mesh.solid, first, owner);
                        continue;
                    }
                }
            }
            // Native raised lawn/path caps need the same fine contact and
            // blending finish as the floor. Verify their source and datum;
            // borrowed wall paint must never authorize a raised surface.
            let native_bank_cap = matches!(material, GroundMaterial::Lawn | GroundMaterial::Path)
                && cell_at(geometry, center[0], center[2]).is_some_and(|i| {
                    let source = &cells[i].source;
                    let shape = shape_for_source(source);
                    let height = shape.surface_height(geometry.tile_height);
                    (shallow_bank_source(source)
                        || crate::forest::forest_ledge_shape(source).is_some())
                        && height > 0.0
                        && matches!(shape, CellShape::RaisedTop { solid: SolidKind::Bank, .. }
                            | CellShape::LedgeBand { .. })
                        && positions.iter().all(|p| (p[1] - height).abs() < 0.001)
                });
            if normal[1] > 0.999
                && (positions[0][1].abs() < 0.001
                    || native_bank_cap
                    || (map == "Route29"
                        && matches!(material, GroundMaterial::Lawn | GroundMaterial::Path)))
            {
                finish.append_surface(&mut mesh.solid, material, positions);
            } else {
                append_quad_colors(
                    &mut mesh.solid,
                    positions,
                    normal,
                    [[0.0; 2]; 4],
                    positions.map(|p| finish.color(material, p)),
                );
                // The approach can have distinct normals at its four
                // corners. Painting unchanged geometry must retain them.
                let start = mesh.solid.normals.len() - 4;
                mesh.solid.normals[start..].copy_from_slice(&old.normals[base..base + 4]);
            }
            if material == GroundMaterial::Water && normal[1] > 0.999 {
                if matches!(map, "NationalPark" | "NationalParkBugContest") {
                    // Proven park water carries stable source coordinates for
                    // moving pigment; the authored floor and atlas stay intact.
                    for i in painted_start..mesh.solid.positions.len() {
                        let p = mesh.solid.positions[i];
                        mesh.solid.uvs[i] = [
                            50000. + (p[0]-geometry.origin_x)/geometry.tile_width + map_origin[0] as f32,
                            -50000. + (p[2]-geometry.origin_z)/geometry.tile_height + map_origin[1] as f32,
                        ];
                    }
                } else {
                let start=mesh.solid.colors.len();
                append_water_ripple(&mut mesh.solid, &positions, geometry, map_origin);
                if finish.cerulean_pool {
                    for color in &mut mesh.solid.colors[start..] {
                        *color=[0.24,0.48,0.88,1.];
                    }
                }
                }
            }
        } else {
            append_quad_colors(
                &mut mesh.textured,
                positions,
                normal,
                uvs,
                old.colors[base..base + 4].try_into().unwrap(),
            );
            let start = mesh.textured.normals.len() - 4;
            mesh.textured.normals[start..].copy_from_slice(&old.normals[base..base + 4]);
        }
    }
    cave_cliff::scarps(&mut mesh.solid,shallow_cave_faces,geometry,map_origin);
    cave_finish::blend_wall_feet(&mut mesh.solid, cells, geometry, map_origin);
    cave_finish::weather_shelf_lips(&mut mesh.solid, cells, geometry, map_origin);
    cave_finish::weather_shelf_lips(&mut mesh.textured, cells, geometry, map_origin);
    if cave_trace {
        eprintln!(
            "cave finish {map}: {cave_faces} of {} faces",
            old.positions.len() / 4
        );
    }
    // Runtime background uses its own repeated texture. Carry those exact
    // four strips into the same material domain, without expanding the apron.
    let background_material = mesh.background.as_ref().and_then(|background| {
        ground_material(&background.source)
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
    use bevy::prelude::IVec2;
    #[test]
    fn ilex_pine_recipe_requires_complete_source_art_and_keeps_footing() {
        let sources=(0..4).flat_map(|row|(0..4).map(move|column| {
            let tile=if row<2 && column<2 {0x26+row*16+column} else {1};
            let mut source=source_with_tile(0,column as u8,row as u8,tile);
            source.tileset_id="forest".into();source
        })).collect();
        let f=frame(4,4,sources);let g=geometry(&f);
        let cells=f.tiles.iter().collect::<Vec<_>>();
        let shapes=vec![CellShape::Flat;16];
        let p=TreePlacement {column:0,row:0,width:2,height:2,ground_tile_index:1,
            ground_metatile_id:None,base_height:0.,rounded:true,outline_mask:true,
            remove_all_ground:false,card_thickness:0.};
        let mut mesh=TerrainMeshData {authored_cells:vec![None;16],footing_heights:vec![0.;16],
            ..Default::default()};
        let mut claimed=vec![false;16];
        let recipe=claim_grove(&mut mesh,&cells,&shapes,&g,"IlexForest",p,&mut claimed,[0,0])
            .expect("complete native forest drawing");
        assert_eq!(recipe.family,crate::foliage::GroveFamily::JohtoPine);
        assert_eq!(claimed.iter().filter(|&&c|c).count(),4);
        assert!(mesh.footing_heights.iter().all(|&height|height==0.));
        assert_eq!(mesh.authored_cells[0],Some("johto/tree"));
        let mut changed=f.tiles.clone();changed[0].source.tile_index=1;
        assert!(modeled_tree_ground("IlexForest",&changed.iter().collect::<Vec<_>>(),&shapes,&g,p).is_none());
    }
    #[test]
    fn grove_ground_borrows_paint_without_borrowing_a_bank_height() {
        let f=frame(4,4,(0..4).flat_map(|y|(0..4).map(move|x| {
            if x>=2 {source_with_tile(0x05,x,y,
                (if y==0 {0x1e} else if y==3 {0x3e} else {0x2e})+(x-2) as u16)}
            else {source_with_tile(if x==0 && y==0 {0x57} else {0x00},x,y,5)}
        })).collect());
        let g=geometry(&f);
        for foreign in [false,true] {
            let mut tiles=f.tiles.clone();
            if foreign {for tile in &mut tiles {
                if tile.column<2 && !(tile.column==0 && tile.row==0) {
                    tile.source.tileset_id="johto_modern".into();
                }
            }}
            let cells=tiles.iter().collect::<Vec<_>>();
            let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
            assert_eq!(shapes[0].surface_height(g.tile_height),6.);
            let plot=tree_placements("Route46",&cells,&g)[0];
            let ground=modeled_tree_ground("Route46",&cells,&shapes,&g,plot).unwrap();
            assert_eq!(ground,if foreign {0} else {1},"prefer level same-atlas lawn");
            let mut mesh=TerrainMeshData {authored_cells:vec![None;16],footing_heights:vec![0.;16],..Default::default()};
            let mut claimed=vec![false;16];
            claim_grove(&mut mesh,&cells,&shapes,&g,"Route46",plot,&mut claimed,[0,0]).unwrap();
            assert!(mesh.textured.positions.iter().all(|p|p[1]==plot.base_height));
            assert!(mesh.footing_heights.iter().all(|&h|h==0.));
            let (u0,u1,v0,v1)=g.uv(ground%4,ground/4);
            assert!(mesh.textured.uvs.iter().all(|uv|
                (u0..=u1).contains(&uv[0]) && (v0..=v1).contains(&uv[1])));
        }
    }

    #[test]
    fn mixed_small_tree_corner_variants_replace_complete_art_only() {
        for (block, column, row) in [(0x3c,0,0),(0x3d,2,0),(0x3e,0,2),(0x3f,2,2)] {
            let f = frame(4,4,(0..4).flat_map(|y|(0..4).map(move|x| {
                let tile = if (column..column+2).contains(&x) && (row..row+2).contains(&y) {
                    [[0x1e,0x1f],[0x3e,0x3f]][(y-row) as usize][(x-column) as usize]
                } else { 5 };
                source_with_tile(block,x,y,tile)
            })).collect());
            let g=geometry(&f);
            let cells=f.tiles.iter().collect::<Vec<_>>();
            let shapes=cells.iter().map(|c|shape_for_source(&c.source)).collect::<Vec<_>>();
            let plots=tree_placements("Route36",&cells,&g);
            assert_eq!(plots.len(),1,"block {block:02x}");
            let mut mesh=TerrainMeshData { authored_cells:vec![None;16],
                footing_heights:vec![0.;16],..Default::default() };
            let mut claimed=vec![false;16];
            let recipe=claim_grove(&mut mesh,&cells,&shapes,&g,"Route36",plots[0],
                &mut claimed,[0,0]).expect("complete mixed corner crown");
            assert_eq!(recipe.tile_height,g.tile_height*0.5);
            assert_eq!(claimed.iter().filter(|&&v|v).count(),4);
            recipe.append(&mut mesh);
            assert!(mesh.solid.positions.iter().any(|p| p[1] > 12.));
            assert!(mesh.footing_heights.iter().all(|&height| height == 0.));
            // All four crown cells become native foliage; the source texture
            // retained underneath it must be lawn, never the old tree card.
            let lawn = f.tiles.iter().position(|tile| tile.source.tile_index == 5).unwrap();
            let (u0,u1,v0,v1) = g.uv(lawn % 4,lawn / 4);
            assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
            assert!(mesh.textured.uvs.iter().all(|uv|
                (u0..=u1).contains(&uv[0]) && (v0..=v1).contains(&uv[1])));
            let mut changed=f.tiles.clone();
            changed[row as usize*4+column as usize].source.tile_index=5;
            assert!(!complete_mixed_tree(&changed.iter().collect::<Vec<_>>(),&g,plots[0]));
        }
    }
    #[test]
    fn modern_mixed_small_trees_replace_both_complete_phases_without_source_cards() {
        let sources = (0..4).flat_map(|y| (0..8).map(move |x| {
            let block = if x < 4 { 0x2f } else { 0x3b };
            let crown = if x < 4 { y < 2 } else { y >= 2 };
            let tile = if crown { [[0x1e, 0x1f], [0x3e, 0x3f]][y % 2][x % 2] } else { 5 };
            let mut source = source_with_tile(block, (x % 4) as u8, y as u8, tile);
            source.tileset_id = "johto_modern".into();
            source
        })).collect();
        let f = frame(8, 4, sources);
        let g = geometry(&f);
        let cells = f.tiles.iter().collect::<Vec<_>>();
        let shapes = cells.iter().map(|c| shape_for_source(&c.source)).collect::<Vec<_>>();
        let plots = tree_placements("AzaleaTown", &cells, &g);
        assert_eq!(plots.len(), 4);
        let mut mesh = TerrainMeshData { authored_cells: vec![None; 32],
            footing_heights: vec![0.; 32], ..Default::default() };
        let mut claimed = vec![false; 32];
        for plot in &plots {
            let recipe = claim_grove(&mut mesh, &cells, &shapes, &g, "AzaleaTown",
                *plot, &mut claimed, [0, 0]).expect("complete native small tree");
            assert_eq!(recipe.tile_height, g.tile_height * 0.5);
            recipe.append(&mut mesh);
        }
        assert_eq!(claimed.iter().filter(|&&c| c).count(), 16);
        assert!(mesh.footing_heights.iter().all(|&h| h == 0.));
        assert!(mesh.solid.positions.iter().any(|p| p[1] > 12.));
        // Original crown glyphs must not remain underneath the native foliage.
        let (u0, u1, v0, v1) = g.uv(4, 0);
        assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
        assert!(mesh.textured.uvs.iter().all(|uv|
            (u0..=u1).contains(&uv[0]) && (v0..=v1).contains(&uv[1])));
        let lower = *plots.iter().find(|p| p.row == 2 && p.column == 4).unwrap();
        let mut changed = f.tiles.clone();
        changed[2 * 8 + 5].source.tile_index = 5;
        assert!(!complete_mixed_tree(&changed.iter().collect::<Vec<_>>(), &g, lower));
        changed = f.tiles.clone();
        changed[2 * 8 + 4].source.subtile_row = 0;
        assert!(!complete_mixed_tree(&changed.iter().collect::<Vec<_>>(), &g, lower));
        let clipped = GridGeometry { height: 3, ..g };
        assert!(!complete_mixed_tree(&cells, &clipped, lower));
    }
    #[test]
    fn runtime_groves_preserve_completed_bank_geometry_claims_and_footing() {
        fn same_geometry(expected: Vec<Vec<u32>>, actual: Vec<Vec<u32>>) {
            assert_eq!(expected.len(), actual.len(), "retain every triangle");
            if let Some(index) = expected.iter().zip(&actual).position(|(a, b)| a != b) {
                assert_eq!(expected[index], actual[index], "triangle {index} differs");
            }
        }
        let profiles: crate::live_profiles::Document = serde_json::from_str(include_str!(
            "../../../../modpacks/voxel-view/profiles.json"
        )).unwrap();
        for (width, height) in [(8_u32, 8_u32), (10, 16)] {
            for origin in [IVec2::ZERO, IVec2::new(-3, 4)] {
                let sources = (0..height).flat_map(|y| (0..width).map(move |x| {
                    if x < width - 4 {
                        source_with_tile(0x05, (x % 2) as u8, (y % 4) as u8,
                            (if y % 4 == 0 { 0x1e } else if y % 4 == 3 { 0x3e } else { 0x2e }) + (x % 2) as u16)
                    } else if x >= width - 4 && y < 4 {
                        source_with_tile(0x56, (x % 4) as u8, y as u8,
                            if y == 3 { 0x4c } else { 0x06 })
                    } else {
                        source_with_tile(0x01, (x % 4) as u8, (y % 4) as u8, 0x05)
                    }
                })).collect();
                let mut f = frame(width, height, sources);
                f.map_id = std::sync::Arc::from("NewBarkTown");
                f.grid_origin = origin;
                let samples = TerrainImageSamples::default();
                let reference = build_instanced_terrain_mesh_with_profiles(&f, &samples, &profiles).unwrap();
                let runtime = build_runtime_terrain_mesh_with_profiles(&f, &samples, &profiles).unwrap();
                let mut deferred = begin_runtime_terrain_mesh_with_profiles(&f, &samples, &profiles).unwrap();
                while !deferred.advance(1) {}
                let deferred = deferred.finish();
                assert_eq!(runtime.terrain, deferred.terrain);
                assert_eq!(runtime.grove_opaque, deferred.grove_opaque);
                let empty = SurfaceMeshData::default();
                let comparison = build_runtime_terrain_mesh_with_profiles(&f, &samples, &profiles).unwrap();
                same_geometry(crate::occluder_fade::geometry_signature(&empty, deferred.grove_groups),
                    crate::occluder_fade::geometry_signature(&empty, comparison.grove_groups));
                assert!(!runtime.grove_groups.is_empty(), "fixture must exercise detached trees");
                assert!(runtime.grove_opaque.is_empty(), "complete grove ownership must survive");
                assert_eq!(reference.authored_cells, runtime.terrain.authored_cells);
                assert_eq!(reference.footing_heights, runtime.terrain.footing_heights);
                let (textured, solid, text_groups, solid_groups) = crate::occluder_fade::prepare_linked(
                    reference.textured, reference.solid, reference.cutaway_links);
                let (next_textured, next_solid, next_text_groups, mut next_solid_groups) =
                    crate::occluder_fade::prepare_linked(runtime.terrain.textured,
                        runtime.terrain.solid, runtime.terrain.cutaway_links);
                next_solid_groups.extend(runtime.grove_groups);
                assert_eq!(solid_groups.len(), next_solid_groups.len(), "retain each independent fade owner");
                same_geometry(crate::occluder_fade::geometry_signature(&textured, text_groups),
                    crate::occluder_fade::geometry_signature(&next_textured, next_text_groups));
                same_geometry(crate::occluder_fade::geometry_signature(&solid, solid_groups),
                    crate::occluder_fade::geometry_signature(&next_solid, next_solid_groups));
            }
        }
    }

    #[test]
    fn forest_ledge_finish_uses_owned_hull_and_rejects_foreign_or_claimed_art() {
        let g=GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut f=frame(1,1,vec![source_with_tile(0x18,1,1,0x05)]);
        f.tiles[0].source.tileset_id="forest".into();
        let cap=[[0.,6.,0.],[0.,6.,8.],[8.,6.,8.],[8.,6.,0.]];
        let wall=[[0.,0.,0.],[0.,6.,0.],[0.,6.,8.],[0.,0.,8.]];
        let cells=vec![&f.tiles[0]];
        let sample=&cells[0].source;
        assert_eq!(forest_bank_surface(&cap,[0.,1.,0.],sample,&cells,&g,&[None]),Some(GroundMaterial::Lawn));
        assert_eq!(forest_bank_surface(&wall,[-1.,0.,0.],sample,&cells,&g,&[None]),Some(GroundMaterial::Bank));
        assert_eq!(forest_bank_surface(&cap,[0.,1.,0.],sample,&cells,&g,&[Some("authored/prop")]),None);
        let high=cap.map(|mut p|{p[1]+=8.;p});
        assert_eq!(forest_bank_surface(&high,[0.,1.,0.],sample,&cells,&g,&[None]),None);
        let mut foreign=sample.clone();foreign.tileset_id="johto".into();
        assert_eq!(forest_bank_surface(&cap,[0.,1.,0.],&foreign,&cells,&g,&[None]),None);
        drop(cells);
        f.tiles[0].source.metatile_id=0x20;
        let cells=vec![&f.tiles[0]];
        assert_eq!(forest_bank_surface(&cap,[0.,1.,0.],&cells[0].source,&cells,&g,&[None]),None);
    }

    #[test]
    fn forest_ledge_relief_and_litter_keep_native_cap_and_floor_contacts() {
        let g=GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut f=frame(1,1,vec![source_with_tile(0x18,1,1,0x05)]);
        f.tiles[0].source.tileset_id="forest".into();
        let cells=vec![&f.tiles[0]];
        let cap=[[0.,6.,0.],[0.,6.,8.],[8.,6.,8.],[8.,6.,0.]];
        let wall=[[0.,0.,8.],[0.,6.,8.],[0.,6.,0.],[0.,0.,0.]];
        let mut mesh=TerrainMeshData {footing_heights:vec![6.],authored_cells:vec![None],..Default::default()};
        let (u0,u1,v0,v1)=g.uv(0,0);
        let uvs=[[u0,v0],[u0,v1],[u1,v1],[u1,v0]];
        append_quad_colors(&mut mesh.textured,cap,[0.,1.,0.],uvs,[[1.;4];4]);
        append_quad_colors(&mut mesh.textured,wall,[-1.,0.,0.],uvs,[[1.;4];4]);
        polish_world_surfaces(&mut mesh,"IlexForest",&cells,&g,[0,0]);
        assert!(mesh.textured.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![6.]);
        assert_eq!(mesh.authored_cells,vec![None]);
        for corner in cap.into_iter().chain(wall) {
            assert!(mesh.solid.positions.contains(&corner),"native ledge perimeter moved");
        }
        assert!(mesh.solid.positions.iter().any(|p|p[0] < -0.01 && p[1]>0. && p[1]<6.),
            "earthen face must have relief inside its fixed perimeter");
        assert!(mesh.solid.positions.iter().any(|p|p[0]>0. && p[0]<8. && p[2]>0. && p[2]<8. && p[1]>6.),
            "native raised cap must receive the forest litter finish");
    }

    #[test]
    fn cave_shelf_return_faces_share_the_front_scarp_finish() {
        let g=GridGeometry {width:3,height:3,tile_width:8.,tile_height:8.,origin_x:-17.,origin_z:29.};
        for tileset in ["cave","dark_cave","johto"] {
            let mut tiles:Vec<_>=(0..9).map(|i|VisualTile {
                column:(i%3)as u32,row:(i/3)as u32,priority:false,
                texture:Handle::default(),animation_frames:None,
                source:VisualTileSource {tileset_id:tileset.into(),metatile_id:0x02,
                    subtile_column:(i%3)as u8,subtile_row:(i/3)as u8,
                    tile_index:if i==4 {0x01}else{0x16}},
            }).collect();
            let mut shapes=vec![CellShape::Flat;9];
            shapes[4]=CellShape::RaisedTop {height:6.,solid:SolidKind::Bank};
            for known in [true,false] {
                tiles[4].source.tile_index=if known {0x01}else{0xff};
                let cells:Vec<_>=tiles.iter().collect();
                let mut mesh=TerrainMeshData {footing_heights:vec![0.;9],..Default::default()};
                append_exposed_sides(&mut mesh,&g,&cells,&shapes,&vec![None;9],1,1);
                if known && tileset!="johto" {
                    assert_eq!(mesh.textured.positions.len(),16);
                    assert!(mesh.solid.positions.is_empty());
                    let joins=mesh.textured.positions.clone();
                    polish_world_surfaces(&mut mesh,"UnionCave1F",&cells,&g,[0,0]);
                    assert!(mesh.textured.positions.is_empty());
                    assert!(mesh.solid.positions.len()>16);
                    assert!(mesh.solid.uvs.iter().all(|uv|(uv[1]/100000.).round()==6.));
                    assert!(mesh.solid.positions.iter().all(|p|p[1]>=0. && p[1]<=6.));
                    for p in joins.into_iter().filter(|p|p[1]==0.) {
                        assert!(mesh.solid.positions.contains(&p),"lost exact floor join");
                    }
                    assert_eq!(mesh.footing_heights,vec![0.;9]);
                } else {
                    assert!(mesh.textured.positions.is_empty());
                    assert_eq!(mesh.solid.positions.len(),16);
                    assert!(mesh.solid.uvs.iter().all(|uv|*uv==[0.,0.]));
                }
            }
        }
    }

    #[test]
    fn tall_cave_return_faces_use_joined_rock_and_participate_in_the_cutaway() {
        let g=GridGeometry {width:3,height:3,tile_width:8.,tile_height:8.,origin_x:-17.,origin_z:29.};
        for tileset in ["cave","dark_cave"] {
            let tiles:Vec<_>=(0..9).map(|i|VisualTile {
                column:(i%3)as u32,row:(i/3)as u32,priority:false,
                texture:Handle::default(),animation_frames:None,
                source:VisualTileSource {tileset_id:tileset.into(),metatile_id:if i==4 {0x2e}else{0x02},
                    subtile_column:(i%3)as u8,subtile_row:(i/3)as u8,
                    tile_index:if i==4 {0x26}else{0x16}},
            }).collect();
            let cells:Vec<_>=tiles.iter().collect();
            let height=crate::cave::rock_height(&cells[4].source);
            let mut shapes=vec![CellShape::Flat;9];
            shapes[4]=CellShape::RaisedTop {height,solid:SolidKind::Bank};
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;9],..Default::default()};
            append_exposed_sides(&mut mesh,&g,&cells,&shapes,&vec![None;9],1,1);
            assert_eq!(mesh.textured.positions.len(),16);
            assert!(mesh.solid.positions.is_empty());
            polish_world_surfaces(&mut mesh,"UnionCave1F",&cells,&g,[0,0]);
            assert!(mesh.textured.positions.is_empty());
            assert!(mesh.solid.positions.len()>16);
            assert!(mesh.solid.uvs.iter().all(|uv|(uv[1]/100000.).round()==2.),
                "an enclosing return face must share the wall reveal category");
            assert!(mesh.solid.normals.iter().all(|n|Vec3::from_array(*n).is_finite()));
            assert_eq!(mesh.footing_heights,vec![0.;9]);
        }
    }

    #[test]
    fn folded_cave_faces_join_across_cells_without_moving_floor_or_lip() {
        let g=GridGeometry {width:2,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let t:Vec<_>=(0..2).map(|x|VisualTile {
            column:x,row:0,priority:false,texture:Handle::default(),animation_frames:None,
            source:VisualTileSource { tileset_id:"cave".into(),metatile_id:0x02,
                subtile_column:x as u8,subtile_row:0,tile_index:0x16 },
        }).collect();
        let cells:Vec<_>=t.iter().collect();
        let mut mesh=TerrainMeshData {footing_heights:vec![6.;2],..Default::default()};
        for x in 0..2 {
            let (w,e,_,s)=g.bounds(x,0);
            let (u0,u1,v0,v1)=g.uv(x,0);
            append_quad_colors(&mut mesh.textured,
                [[w,0.,s],[e,0.,s],[e,6.,s],[w,6.,s]], [0.,0.,1.],
                [[u0,v1],[u1,v1],[u1,v0],[u0,v0]],[[1.;4];4]);
        }
        polish_world_surfaces(&mut mesh,"UnionCave1F",&cells,&g,[0,0]);
        assert!(mesh.textured.positions.is_empty());
        assert_eq!(mesh.footing_heights,vec![6.;2]);
        for x in [0.,16.] { for y in [0.,6.] {
            assert!(mesh.solid.positions.contains(&[x,y,8.]));
        }}
        assert!(mesh.solid.positions.iter().any(|p|p[0]==8. && p[1]>0. && p[1]<6. && p[2]<8.),
            "native tile join incorrectly became a new flat bank end");
        assert!(mesh.solid.positions.iter().all(|p|p[1]>=0. && p[1]<=6.));
        assert!(mesh.solid.uvs.iter().all(|uv|(uv[1]/100000.).round()==6.));
    }

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
    fn compact_house_source_roof_tracks_live_paint_without_changing_plot_or_door() {
        let original=drawing(&[&[0x14,0x15]],3);
        let mut sources:Vec<_>=original.tiles.into_iter().map(|t|t.source).collect();
        sources.extend((0..8).map(|_|source_with_tile(0x01,0,0,0x06)));
        let f=frame(8,5,sources);let g=geometry(&f);
        let cells:Vec<_>=f.tiles.iter().collect();
        let p=building_placements("CherrygroveCity",&cells,&g)[0];
        let shapes:Vec<_>=cells.iter().map(|t|shape_for_source(&t.source)).collect();
        let mut baseline=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut base_claims=vec![false;cells.len()];
        assert!(append_building(&mut baseline,&cells,&shapes,&g,"CherrygroveCity",p,&mut base_claims));
        let mut paints=Vec::new();
        for field in [[55,200,230,255],[180,135,45,255]] {
            let mut images=TerrainImageSamples::default();
            let pixels:Vec<_>=(0..64).flat_map(|i|if i%3==0 {[25,25,25,255]}
                else if i%3==1 {field}else {[235,245,235,255]}).collect();
            for tile in &cells {images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(pixels.clone()));}
            let mut painted=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut claims=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut painted,&cells,&shapes,&g,"CherrygroveCity",p,&mut claims,Some(&images)));
            assert_eq!(painted.solid.positions,baseline.solid.positions);
            assert_eq!(painted.solid.normals,baseline.solid.normals);
            assert_eq!(painted.solid.indices,baseline.solid.indices);
            assert_eq!(painted.footing_heights,baseline.footing_heights);
            assert_eq!(claims,base_claims);
            assert_ne!(painted.solid.colors,baseline.solid.colors);
            paints.push(painted.solid.colors);
        }
        assert_ne!(paints[0],paints[1],"source swatches, not a fixed town RGB, select roof paint");
        let mut fallback=TerrainMeshData::default();let mut claims=vec![false;cells.len()];
        assert!(append_building_with_samples(&mut fallback,&cells,&shapes,&g,"CherrygroveCity",p,&mut claims,Some(&TerrainImageSamples::default())));
        assert!(fallback.solid.positions.is_empty());
        assert_eq!(fallback.textured.quad_count(),p.width*p.height);
    }

    #[test]
    fn service_source_colors_cover_roof_and_masonry_without_changing_claims_or_footing() {
        for last in [0x17,0x1b] {
            let original=drawing(&[&[0x18,0x19],&[0x1a,last]],3);
            let mut sources:Vec<_>=original.tiles.into_iter().map(|t|t.source).collect();
            sources.extend((0..8).map(|_|source_with_tile(0x01,0,0,0x06)));
            let f=frame(8,9,sources);
            let g=geometry(&f);
            let cells:Vec<_>=f.tiles.iter().collect();
            let p=building_placements("CherrygroveCity",&cells,&g)[0];
            let shapes:Vec<_>=cells.iter().map(|t|shape_for_source(&t.source)).collect();
            let mut images=TerrainImageSamples::default();
            let roof:Vec<_>=(0..64).flat_map(|i|match i%4 {
                0=>[20,35,55,255],1=>[30,130,230,255],
                2=>[100,230,240,255],_=>[245,245,245,255]}).collect();
            let wall:Vec<_>=(0..64).flat_map(|i|if i%3==0 {
                [220,240,205,255]} else {[250,145,195,255]}).collect();
            for tile in &cells {images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(
                if (tile.row as usize)<p.roof_rows {roof.clone()} else {wall.clone()}));}
            assert!(modeled_exteriors::cabin_roof_palette(&cells,&g,p,&images).is_some());
            let paint=service_wall_color(&cells,&g,p,&images).unwrap();
            let brick=paint[0];
            assert!(brick[0]>brick[2] && brick[2]>brick[1]);
            assert!(paint[1][1]>paint[1][0] && paint[1][0]>paint[1][2],"source mortar stays separate from pink clay");
            let mut before=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut before_claimed=vec![false;cells.len()];
            assert!(append_building(&mut before,&cells,&shapes,&g,"CherrygroveCity",p,&mut before_claimed));
            let mut after=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut after,&cells,&shapes,&g,"CherrygroveCity",p,&mut claimed,Some(&images)));
            assert_eq!(before.solid.positions,after.solid.positions);
            assert_eq!(before.solid.indices,after.solid.indices);
            assert_eq!(before.solid.normals,after.solid.normals);
            assert_eq!(before.textured,after.textured);
            assert_ne!(before.solid.colors,after.solid.colors);
            assert_eq!(before_claimed,claimed);
            assert_eq!(before.footing_heights,after.footing_heights);
            assert_eq!(before.solid.cutaway_ranges,after.solid.cutaway_ranges);
            images.pixels.remove(&cells[(p.row+p.roof_rows)*g.width+p.column+2].texture.id());
            let mut fallback=TerrainMeshData::default();
            let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut fallback,&cells,&shapes,&g,"CherrygroveCity",p,&mut claimed,Some(&images)));
            assert!(fallback.solid.positions.is_empty(),"missing swatches must not introduce guessed paint");
            assert_eq!(fallback.textured.quad_count(),p.width*p.height);
        }
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
    fn grass_is_source_scoped_tall_and_contained_without_footing_changes() {
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
        assert!(!mesh.indices.is_empty());
        assert!(mesh.positions.iter().all(|p| p[0] > -32.0
            && p[0] < -24.0
            && p[2] > -40.0
            && p[2] < -32.0
            && p[1] >= 0.0
            && p[1] < 8.0 * 3.0));
        assert!(mesh.positions.iter().any(|p| p[1] > 16.0));
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
    #[test]
    fn ice_rim_finishing_preserves_source_and_all_footing_heights() {
        let f = frame(
            4,
            4,
            (0..4)
                .flat_map(|y| {
                    (0..4).map(move |x| {
                        material_source(
                            "ice_path",
                            0x08,
                            x,
                            y,
                            if x < 2 { 0xa8 + u16::from(x) } else { 0x9a },
                        )
                    })
                })
                .collect(),
        );
        let g = GridGeometry {
            origin_x: -(f.grid_size.x as f32 * f.tile_size.x) * 0.5,
            origin_z: -(f.grid_size.y as f32 * f.tile_size.y) * 0.5,
            ..geometry(&f)
        };
        let cells: Vec<_> = f.tiles.iter().collect();
        let source = f.clone();
        let mut mesh = build_terrain_mesh(&f).unwrap();
        let footing = mesh.footing_heights.clone();
        let claims = mesh.authored_cells.clone();
        let textured = mesh.textured.indices.len();
        polish_world_surfaces(&mut mesh, "IcePath1F", &cells, &g, [0, 0]);
        assert!(
            mesh.textured.indices.len() < textured,
            "edge paint becomes ice relief"
        );
        assert_eq!(mesh.footing_heights, footing);
        assert_eq!(mesh.authored_cells, claims);
        assert_eq!(f, source);
    }

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
                            0x58 => [0x27,0x2c,0x2c,0x2c,0x27,0x2c,0x2c,0x2c,0x27,0x2c,0x2c,0x2c,0x27,0x2c,0x2c,0x2c],
                            0x59 => [0x2c,0x2c,0x2c,0x24,0x2c,0x2c,0x2c,0x24,0x2c,0x2c,0x2c,0x24,0x2c,0x2c,0x2c,0x24],
                            0x5a => [0x27,0x2c,0x2c,0x2c,0x27,0x2c,0x2c,0x2c,0x27,0x2c,0x2c,0x2c,0x36,0x37,0x37,0x37],
                            0x5b => [0x2c,0x2c,0x2c,0x24,0x2c,0x2c,0x2c,0x24,0x2c,0x2c,0x2c,0x24,0x37,0x37,0x37,0x34],
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
    fn kanto_mineral_barrier_bases_finish_native_fill_and_owned_rock_underlays() {
        let f = frame(4, 4, (0..4).flat_map(|y| (0..4).map(move |x|
            material_source("kanto", 0x29, x, y, if x < 2 { 0x11 } else { 0x24 })
        )).collect());
        let g = kanto_bank_geometry(&f);
        let cells = f.tiles.iter().collect::<Vec<_>>();
        let shapes = cells.iter().map(|t| shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 16], authored_cells: vec![None; 16],
            ..Default::default()
        };
        let mut claimed = vec![false; 16];
        modeled_exteriors::append_props(&mut mesh, &cells, &shapes, &g, &mut claimed);
        for (i, occupied) in claimed.iter().enumerate() {
            if !occupied {
                let (w, e, n, s) = g.bounds(i % 4, i / 4);
                append_top(&mut mesh.textured, [w, e, n, s], 0., g.uv(i % 4, i / 4));
            }
        }
        let feet = mesh.footing_heights.clone();
        let claims = mesh.authored_cells.clone();
        assert_eq!(claims.iter().filter(|c| **c == Some("world-exterior/kanto_boundary_path")).count(), 8);
        assert!(!mesh.textured.positions.is_empty());
        polish_world_surfaces(&mut mesh, "Route4", &cells, &g, [0, 0]);
        assert!(mesh.textured.positions.is_empty(), "both the exposed fill and the replacement rock bed lose pixel ink");
        assert_eq!(mesh.footing_heights, feet);
        assert_eq!(mesh.authored_cells, claims);
        assert!(mesh.solid.colors.iter().any(|c| c[0] > c[2] * 1.6));
        for (block, x, y, tile) in [(0x28, 0, 0, 0x11), (0x29, 2, 0, 0x11),
            (0x29, 0, 4, 0x11), (0x29, 0, 0, 0x24)] {
            assert_eq!(ground_material(&material_source("kanto", block, x, y, tile)), None);
        }
    }

    #[test]
    fn kanto_shallow_banks_finish_actual_mesher_caps_and_all_four_sides() {
        for (block, cap, top_count, side_count) in [
            (0x07, GroundMaterial::Lawn, 16, 16),
            (0x1a, GroundMaterial::Path, 16, 16),
            (0x2f, GroundMaterial::Lawn, 8, 12),
            (0x58, GroundMaterial::Lawn, 16, 16),
            (0x59, GroundMaterial::Lawn, 16, 16),
            (0x5a, GroundMaterial::Lawn, 16, 16),
            (0x5b, GroundMaterial::Lawn, 16, 16),
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
                    let front_underlay = n == [0., 1., 0.]
                        && matches!(cells[sample].source.tile_index, 0x37 | 0x34 | 0x36 | 0x27 | 0x24);
                    assert_eq!(material, front_underlay.then_some(cap),
                        "only the complete raised front's underlay loses its pixel bank ink");
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
            // Caps retain their support plane; the earthen side finish adds
            // bounded relief and rim grass instead of retaining flat gray walls.
            let before = mesh.clone();
            polish_world_surfaces(&mut mesh, "Route4", &cells, &g, [0, 0]);
            assert!(mesh.textured.positions.is_empty());
            for (&point,&normal) in before.textured.positions.iter().zip(&before.textured.normals) {
                if normal == [0.,1.,0.] {
                    assert!(mesh.solid.positions.iter().zip(&mesh.solid.normals)
                        .any(|(p,n)|*p==point && *n==normal), "native cap support stays exact");
                }
            }
            let min=before.textured.positions.iter().fold([f32::INFINITY;3],|a,p|
                std::array::from_fn(|i|a[i].min(p[i])));
            let max=before.textured.positions.iter().fold([f32::NEG_INFINITY;3],|a,p|
                std::array::from_fn(|i|a[i].max(p[i])));
            assert!(mesh.solid.positions.iter().all(|p|p.iter().all(|v|v.is_finite())
                && p[0]>=min[0]-g.tile_width*0.2 && p[0]<=max[0]+g.tile_width*0.2
                && p[2]>=min[2]-g.tile_height*0.2 && p[2]<=max[2]+g.tile_height*0.2),
                "soil relief stays close to the original face footprint");
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
        for block in [0x07, 0x1a, 0x2f, 0x58, 0x59, 0x5a, 0x5b] {
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
    fn johto_ledge_side_verges_use_owned_ground_with_original_footing() {
        for block in [0x4f,0x51] {
            let mut f = frame(4, 4, (0..4).flat_map(|y| (0..4).map(move |x| {
                let tile=if block==0x51 && y==3 {if x==3 {0x4d} else {0x4c}}
                    else if x==3 {0x3d} else if block==0x51 {0x06} else {0x05};
                source_with_tile(block,x,y,tile)
            })).collect());
            let before = build_terrain_mesh(&f).unwrap();
            f.map_id = std::sync::Arc::from("Route29");
            let after = build_terrain_mesh(&f).unwrap();
            assert_eq!(before.footing_heights, after.footing_heights);
            let mut g = geometry(&f);
            g.origin_x = -f.viewport_size.x * 0.5;
            g.origin_z = -f.viewport_size.y * 0.5;
            let (west, _, _, _) = g.bounds(3, 0);
            let height=shape_for_source(&source_with_tile(block,3,0,0x3d)).surface_height(g.tile_height);
            let border: Vec<_> = after.solid.positions.iter().zip(&after.solid.colors)
                .zip(&after.solid.normals).filter(|((p,_),n)|
                    p[0]>west+0.001 && (p[1]-height * if block==0x51 {
                        ((p[2]-g.origin_z)/(4.0*g.tile_height)).clamp(0.0,1.0)
                    } else {1.0}).abs()<0.05 && n[1]>0.9).collect();
            assert!(!border.is_empty(),"side verge missing for block {block:x}");
            assert!(border.iter().any(|((_,c),_)| c[1]>c[0]+0.04),
                "borrowed path art painted the flat border of block {block:x}");
        }
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
    fn lawn_shore_soil_keeps_all_four_bank_perimeters_and_rejects_unknown_contacts() {
        let mut f=frame(3,3,vec![source_with_tile(0x35,0,0,0x14);9]);
        f.tiles[4].source=source_with_tile(0x01,0,0,0x05);
        let g=geometry(&f);
        let cells:Vec<_>=f.tiles.iter().collect();
        let low=CellShape::Water.surface_height(g.tile_height);
        for direction in Direction::ALL {
            let mut mesh=TerrainMeshData {
                authored_cells:vec![None;9],footing_heights:vec![low;9],..Default::default()
            };
            mesh.footing_heights[4]=0.;
            let (w,e,n,s)=g.bounds(1,1);
            append_textured_shoreline(&mut mesh.textured,&g,1,1,direction,[w,e,n,s],low,0.);
            let p:[[f32;3];4]=mesh.textured.positions[..].try_into().unwrap();
            let normal=mesh.textured.normals[0];
            assert!(lawn_water_bank("LakeOfRage",&p,normal,&cells[4].source,&cells,&g,&mesh.authored_cells));
            for map in ["Route26","Route43","CherrygroveCity","NewBarkTown"] {
                assert!(lawn_water_bank(map,&p,normal,&cells[4].source,&cells,&g,&mesh.authored_cells));
            }
            assert!(!lawn_water_bank("Route43Gate",&p,normal,&cells[4].source,&cells,&g,&mesh.authored_cells));
            let mut claims=mesh.authored_cells.clone();claims[4]=Some("johto/tree");
            assert!(!lawn_water_bank("LakeOfRage",&p,normal,&cells[4].source,&cells,&g,&claims));
            let changed=p.map(|mut v|{v[1]-=g.tile_height;v});
            assert!(!lawn_water_bank("LakeOfRage",&changed,normal,&cells[4].source,&cells,&g,&mesh.authored_cells));
            let footing=mesh.footing_heights.clone();
            let claims=mesh.authored_cells.clone();
            polish_world_surfaces(&mut mesh,"LakeOfRage",&cells,&g,[0,0]);
            assert!(mesh.textured.positions.is_empty());
            assert!(!mesh.solid.indices.is_empty());
            for corner in p {assert!(mesh.solid.positions.contains(&corner));}
            assert_eq!(mesh.footing_heights,footing);
            assert_eq!(mesh.authored_cells,claims);
            let mut changed=f.clone();changed.tiles[4].source.tileset_id=std::sync::Arc::from("unknown");
            let changed_cells:Vec<_>=changed.tiles.iter().collect();
            assert!(!lawn_water_bank("LakeOfRage",&p,normal,&cells[4].source,&changed_cells,&g,&claims));
            changed=f.clone();
            for tile in &mut changed.tiles {tile.source=source_with_tile(0x01,0,0,0x05);}
            let changed_cells:Vec<_>=changed.tiles.iter().collect();
            assert!(!lawn_water_bank("LakeOfRage",&p,normal,&cells[4].source,&changed_cells,&g,&claims));
        }
    }

    #[test]
    fn path_shore_soil_closes_cherrygrove_water_contacts_at_exact_source_datums() {
        let mut f = frame(3, 3, vec![source_with_tile(0x35, 0, 0, 0x14); 9]);
        f.tiles[4].source = source_with_tile(0x00, 0, 0, 0x06);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let low = CellShape::Water.surface_height(g.tile_height);
        for direction in Direction::ALL {
            let mut mesh = TerrainMeshData {
                authored_cells: vec![None; 9], footing_heights: vec![low; 9],
                ..Default::default()
            };
            mesh.footing_heights[4] = 0.;
            let (w, e, n, s) = g.bounds(1, 1);
            append_textured_shoreline(&mut mesh.textured, &g, 1, 1,
                direction, [w, e, n, s], low, 0.);
            let corners = mesh.textured.positions.clone();
            let floor = mesh.footing_heights.clone();
            let claims = mesh.authored_cells.clone();
            polish_world_surfaces(&mut mesh, "CherrygroveCity", &cells, &g, [0, 0]);
            assert!(mesh.textured.positions.is_empty(), "path edge must receive the soil finish");
            for corner in corners { assert!(mesh.solid.positions.contains(&corner)); }
            assert_eq!(mesh.footing_heights, floor);
            assert_eq!(mesh.authored_cells, claims);
            assert!(mesh.solid.positions.iter().all(|p| p[1] >= low && p[1] <= 0.));
        }
    }

    #[test]
    fn lake_cabin_corner_crowns_share_grove_without_sprite_floor_fragments() {
        for (block, crown_row) in [(0x3c, 0usize), (0x3e, 2usize)] {
            let mut sources = (0..4).flat_map(|y| (0..4).map(move |x|
                source_with_tile(block, x, y, 0x05))).collect::<Vec<_>>();
            for (dy, row) in [[0x1e, 0x1f], [0x3e, 0x3f]].iter().enumerate() {
                for (x, &tile) in row.iter().enumerate() {
                    sources[(crown_row+dy)*4+x].tile_index = tile;
                }
            }
            let mut f = frame(4, 4, sources);
            f.map_id = std::sync::Arc::from("LakeOfRage");
            let mut g = geometry(&f);
            g.origin_x = -(g.width as f32 * g.tile_width) * 0.5;
            g.origin_z = -(g.height as f32 * g.tile_height) * 0.5;
            let cells: Vec<_> = f.tiles.iter().collect();
            for map in ["LakeOfRage", "Route43", "CherrygroveCity"] {
                let plots = tree_placements(map, &cells, &g);
                assert_eq!(plots.len(), 1);
                assert_eq!((plots[0].column,plots[0].row,plots[0].width,plots[0].height),
                    (0,crown_row,2,2));
                let reserved = preferred_scenery_cells(map, &cells, &g);
                assert_eq!(reserved.iter().filter(|&&v|v).count(), 4);
            }
            let mut mesh = build_terrain_mesh_with_samples(&f, &TerrainImageSamples::default()).unwrap();
            // Exercise the production outdoor finish even when the test
            // process has authored exterior loading disabled.
            polish_world_surfaces(&mut mesh, "LakeOfRage", &cells, &g, [0,0]);
            assert!(mesh.textured.positions.is_empty(), "block {block:x}: complete corner crown must not retain a sprite square");
            assert!(mesh.footing_heights.iter().all(|&h|h==0.));
            assert!(!mesh.solid.indices.is_empty());
            f.tiles[crown_row*4].source.subtile_column = 1;
            let cells: Vec<_> = f.tiles.iter().collect();
            assert!(tree_placements("LakeOfRage", &cells, &g).is_empty(),
                "changed source phase must retain its original drawing");
        }
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
        assert!(tree_placements("UnionCave1F", &cells, &g).is_empty());
        for map in ["LakeOfRage", "Route43", "Route32", "Route35", "Route42",
            "Route44", "Route45", "Route46", "BlackthornCity", "EcruteakCity", "MahoganyTown"] {
            let lake_plots = tree_placements(map, &cells, &g);
            assert_eq!(lake_plots.iter().map(|p| (p.column,p.row,p.width,p.height)).collect::<Vec<_>>(),
                plots.iter().map(|p| (p.column,p.row,p.width,p.height)).collect::<Vec<_>>(),
                "{map} reuses exact mixed crown signatures");
            let reserved = preferred_scenery_cells(map, &cells, &g);
            assert_eq!(reserved.iter().filter(|&&v| v).count(), 12);
            let finish = GroundFinish::new(map, &cells, &g, [0, 0]);
            assert_eq!(finish.shadows.iter().filter(|s| s.tree).count(), 3);
            assert!(finish.shadows.iter().all(|s| s.trunk_center.is_some()));
        }
        let reserved = preferred_scenery_cells("CherrygroveCity", &cells, &g);
        assert_eq!(reserved.iter().filter(|&&v| v).count(), 12);
        assert!(!reserved[2] && !reserved[3] && !reserved[6] && !reserved[7]);
        let mesh = build_terrain_mesh_with_samples(&f, &TerrainImageSamples::default()).unwrap();
        assert!(
            mesh.textured.positions.is_empty(),
            "verified crown art must not remain as bright sprite fragments"
        );
        assert!(
            mesh.solid.positions.len() >= 3 * 1000
        );
        assert!(mesh.footing_heights.iter().all(|h| *h == 0.0));
        let canopy_height = mesh.solid.positions.iter().map(|p| p[1])
            .fold(0.0_f32, f32::max);
        assert!(canopy_height > g.tile_height * 3.
            && canopy_height <= g.tile_height * 8.,
            "coastal town trees need a broad town crown, not vertically stretched world trees: {canopy_height}");
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
            && p[1] <= 16.001));
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
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: fast.sunlit_grove,
            blend: fast.blend.clone(),
            shore: fast.shore.clone(),
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
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: fast.sunlit_grove,
            blend: fast.blend.clone(),
            shore: fast.shore.clone(),
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
                trunk_center: None,
        };
        let anchored = GroundShadow { trunk_center: Some([-20.0,-28.0]), ..shadow };
        assert!(anchored.darkness([-20.0,0.0,-28.0], &geometry)
            > shadow.darkness([-20.0,0.0,-28.0], &geometry)+0.20,
            "contact shade must follow the displaced trunk");
        assert_eq!(anchored.darkness([-32.0,0.0,-32.0], &geometry),0.0,
            "shadow bounds retain the source underlay");
        assert!(shadow.darkness([-24.0, 0.0, -32.0], &geometry) > 0.45);
        assert_eq!(shadow.darkness([-32.0, 0.0, -32.0], &geometry), 0.0);
        assert_eq!(shadow.darkness([-15.9, 0.0, -32.0], &geometry), 0.0);
        assert!(
            shadow.darkness([-30.0, 0.0, -32.0], &geometry)
                < shadow.darkness([-28.0, 0.0, -32.0], &geometry)
        );
        let finish = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: false,
            blend: Default::default(),
            shore: Default::default(),
            geometry: &geometry,
            map_origin: [0, 0],
            shadows: vec![shadow],
            shadow_cells: None,
        };
        let unshaded = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: false,
            blend: Default::default(),
            shore: Default::default(),
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
    fn route26_native_raised_path_receives_detail_without_lowering_its_cap() {
        let f = frame(1, 1, vec![source_with_tile(0x56, 1, 1, 0x06)]);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let height = shape_for_source(&cells[0].source).surface_height(g.tile_height);
        assert!(height > 0.);
        let (w, e, n, s) = g.bounds(0, 0);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![height],
            ..Default::default()
        };
        append_top(&mut mesh.textured, [w, e, n, s], height, g.uv(0, 0));
        let original = mesh.textured.positions.clone();
        polish_world_surfaces(&mut mesh, "Route26", &cells, &g, [10, 28]);
        assert!(mesh.textured.positions.is_empty());
        assert!(mesh.solid.positions.len() > 4, "raised cap needs fine ground detail");
        assert!(original.iter().all(|p| mesh.solid.positions.contains(p)));
        assert_eq!(mesh.footing_heights, vec![height]);
        assert!(mesh.solid.positions.iter().all(|p| p[1] >= height));
        for offset in [-0.5, 0.5] {
            let mut displaced = TerrainMeshData::default();
            append_top(
                &mut displaced.textured,
                [w, e, n, s],
                height + offset,
                g.uv(0, 0),
            );
            let before = displaced.textured.positions.clone();
            polish_world_surfaces(&mut displaced, "Route26", &cells, &g, [10, 28]);
            assert_eq!(
                displaced.textured.positions.iter().chain(&displaced.solid.positions)
                    .copied().collect::<Vec<_>>(), before,
                "a different raised datum must retain the original coarse geometry"
            );
            assert_eq!(displaced.textured.indices.len() + displaced.solid.indices.len(), 6,
                "borrowed native paint must not authorize fine raised-cap tessellation");
        }
    }

    #[test]
    fn terrace_finish_keeps_level_cap_boundaries_and_sloped_support() {
        let f = frame(2, 2, vec![source_with_tile(0x01, 0, 0, 0x05); 4]);
        let g = geometry(&f);
        let floor = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: true,
            blend: Default::default(),
            shore: Default::default(),
            geometry: &g,
            map_origin: [10, 9],
            shadows: vec![],
            shadow_cells: None,
        };
        let (w, _, n, _) = g.bounds(0, 0);
        let cap = [
            [w, 3., n],
            [w, 3., n + g.tile_height * 0.5],
            [w + g.tile_width * 2., 3., n + g.tile_height * 0.5],
            [w + g.tile_width * 2., 3., n],
        ];
        let mut mesh = SurfaceMeshData::default();
        floor.append_surface(&mut mesh, GroundMaterial::Lawn, cap);
        assert!(cap.iter().all(|p| mesh.positions.contains(p)));
        assert!(mesh.positions.iter().all(|p| p[1] >= 3. && p[1] < 3. + g.tile_height * 0.08
            && p[0] >= w && p[0] <= w + g.tile_width * 2.
            && p[2] >= n && p[2] <= n + g.tile_height * 0.5));
        let slope = [cap[0], [cap[1][0], 0., cap[1][2]],
            [cap[2][0], 0., cap[2][2]], cap[3]];
        let mut sloped = SurfaceMeshData::default();
        floor.append_surface(&mut sloped, GroundMaterial::Lawn, slope);
        assert_eq!(sloped.positions, slope);
    }

    #[test]
    fn route_floor_light_stays_map_anchored_and_preserves_other_surface_materials() {
        let f = frame(2, 2, vec![source_with_tile(0x01, 0, 0, 0x05); 4]);
        let g = geometry(&f);
        let shifted_g = GridGeometry {
            origin_x: g.origin_x - 8.0,
            ..g
        };
        let floor = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: true,
            blend: Default::default(),
            shore: Default::default(),
            geometry: &g,
            map_origin: [10, 9],
            shadows: vec![GroundShadow {
                bounds: [-32., -16., -40., -24.],
                tree: true,
                trunk_center: None,
            }],
            shadow_cells: None,
        };
        let shifted = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: true,
            blend: Default::default(),
            shore: Default::default(),
            geometry: &shifted_g,
            map_origin: [10, 9],
            shadows: vec![GroundShadow {
                bounds: [-40., -24., -40., -24.],
                tree: true,
                trunk_center: None,
            }],
            shadow_cells: None,
        };
        for z in 0..16 {
            for x in 0..16 {
                let p = [-31.5 + x as f32, 0., -39.5 + z as f32];
                for material in [GroundMaterial::Lawn, GroundMaterial::Path] {
                    let a = floor.color(material, p);
                    let b = shifted.color(material, [p[0] - 8., p[1], p[2]]);
                    assert!(a.iter().zip(b).all(|(a, b)| (*a - b).abs() < 0.00001));
                    assert!(a.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
                }
            }
        }
        let p = [-24., 0., -32.];
        let map_p = [p[0] - g.origin_x + 80., 0., p[2] - g.origin_z + 72.];
        for material in [
            GroundMaterial::Water,
            GroundMaterial::Shore,
            GroundMaterial::Stone,
        ] {
            assert_eq!(floor.color(material, p), ground_color(material, map_p, &g));
        }
    }

    #[test]
    fn cerulean_pool_palette_is_scoped_and_changes_no_other_material() {
        let mut native=source_with_tile(0x2e,2,0,0x14);
        native.tileset_id="port".into();
        let f=frame(1,1,vec![native]);let g=geometry(&f);
        let cells=f.tiles.iter().collect::<Vec<_>>();
        let pool=GroundFinish::new("CeruleanGym",&cells,&g,[0,0]);
        let harbor=GroundFinish::new("OlivinePort",&cells,&g,[0,0]);
        let point=[0.,CellShape::Water.surface_height(g.tile_height),0.];
        let blue=pool.color(GroundMaterial::Water,point);
        assert!(blue[2]>blue[1]*2.);
        let map_point=[point[0]-g.origin_x,point[1],point[2]-g.origin_z];
        assert_eq!(harbor.color(GroundMaterial::Water,point),ground_color(GroundMaterial::Water,map_point,&g));
        for material in [GroundMaterial::Stone,GroundMaterial::Path,GroundMaterial::Bank] {
            assert_eq!(pool.color(material,point),harbor.color(material,point));
        }
    }
    #[test]
    fn surface_grain_and_color_remain_anchored_when_the_source_grid_scrolls() {
        let frame = frame(1, 1, vec![source_with_tile(0x01, 0, 0, 0x06)]);
        let geometry = geometry(&frame);
        let finish = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: false,
            blend: Default::default(),
            shore: Default::default(),
            geometry: &geometry,
            map_origin: [10, 9],
            shadows: vec![],
            shadow_cells: None,
        };
        let shifted = GroundFinish {
            cerulean_pool: false,
            path_palette: None,
            city_paving: None,
            forest: false,
            sunlit_grove: false,
            blend: Default::default(),
            shore: Default::default(),
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
                    && p[1]
                        < if material == GroundMaterial::Lawn {
                            0.9
                        } else {
                            // Pebble relief stays below ankle height; the
                            // underlying source plane remains at zero.
                            0.9
                        }));
            }
            assert!(count > 0, "expected some irregular surface detail");
        }
    }

    #[test]
    fn forest_pond_rim_uses_native_mineral_without_moving_source_support() {
        let mut rim = source_with_tile(0x0a,0,0,0x1b);
        rim.tileset_id = "forest".into();
        let mut water = source_with_tile(0x0a,1,0,0x14);
        water.tileset_id = "forest".into();
        let mut f = frame(2,1,vec![rim,water]);
        f.map_id = "IlexForest".into();
        let original = f.clone();
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut mesh = TerrainMeshData { footing_heights: vec![0.,-2.],
            authored_cells: vec![None;2], ..Default::default() };
        let (w,e,n,s) = g.bounds(0,0);
        let corners = [[w,0.,n],[w,0.,s],[e,0.,s],[e,0.,n]];
        append_top(&mut mesh.textured,[w,e,n,s],0.,g.uv(0,0));
        polish_world_surfaces(&mut mesh,"IlexForest",&cells,&g,[0,0]);
        assert!(mesh.textured.positions.is_empty(), "verified rim must not remain pixel art");
        assert_eq!(mesh.footing_heights,vec![0.,-2.]);
        assert_eq!(mesh.authored_cells,vec![None;2]);
        assert!(corners.iter().all(|p| mesh.solid.positions.contains(p)));
        assert!((level_floor_area(&mesh.solid) - f64::from(g.tile_width*g.tile_height)).abs() < 0.001,
            "native bank must retain complete source support beneath its accents");
        assert!(mesh.solid.positions.iter().all(|p| (0. ..0.9).contains(&p[1])));
        assert_eq!(f,original,"source tiles and gameplay state remain authoritative");
    }

    #[test]
    fn outdoor_floor_material_coordinates_are_stable_and_exclude_accents() {
        let f = frame(2, 2, vec![source_with_tile(0x01, 0, 0, 0x05); 4]);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let floor = GroundFinish::new("NewBarkTown", &cells, &g, [-7, 11]);
        let shifted_g = GridGeometry { origin_x: g.origin_x + 64., origin_z: g.origin_z - 32., ..g };
        let shifted = GroundFinish::new("NewBarkTown", &cells, &shifted_g, [-7, 11]);
        for material in [GroundMaterial::Lawn, GroundMaterial::Path, GroundMaterial::Pavers] {
            let (w,e,n,s) = g.bounds(0,0);
            let p = [[w,0.,n],[w,0.,s],[e,0.,s],[e,0.,n]];
            let mut a = SurfaceMeshData::default();
            let mut b = SurfaceMeshData::default();
            floor.append_surface(&mut a, material, p);
            shifted.append_surface(&mut b, material, p.map(|v| [v[0]+64.,v[1],v[2]-32.]));
            assert_eq!(a.uvs, b.uvs, "material grain must not swim with the camera");
            assert_eq!(a.indices, b.indices);
            for (p, uv) in a.positions.iter().zip(&a.uvs) {
                if p[1] == 0. {
                    assert!((65000. ..69000.).contains(&uv[0]) && uv[1] < -55000.);
                } else {
                    assert_eq!(*uv, [0.,0.], "flowers and stones retain their own pigments");
                }
            }
        }
    }

    #[test]
    fn forest_floor_is_continuous_world_anchored_and_leaves_walk_datum_intact() {
        let f = frame(2, 2, vec![source_with_tile(0x01, 0, 0, 0x05); 4]);
        let g = geometry(&f);
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut floor = GroundFinish::new("IlexForest", &cells, &g, [0, 0]);
        floor.forest = true;
        let soil = floor.color(GroundMaterial::Lawn, [-28.0, 0.0, -36.0]);
        assert_ne!(
            soil,
            ground_color(GroundMaterial::Lawn, [4.0, 0.0, 4.0], &g)
        );
        let near = floor.color(GroundMaterial::Lawn, [-28.0001, 0.0, -36.0]);
        assert!(soil.iter().zip(near).all(|(a, b)| (a - b).abs() < 0.001));
        // Litter accumulates at tree contacts; an empty clearing may be bare.
        // Keep the existing litter/footing assertions on an explicit canopy.
        floor.shadows.push(GroundShadow { bounds: [-32.,-24.,-40.,-32.],
            tree: true, trunk_center: Some([-28.,-36.]) });
        floor.shadow_cells = None;
        let mut mesh = SurfaceMeshData::default();
        floor.append_surface(
            &mut mesh,
            GroundMaterial::Lawn,
            [
                [-32.0, 0.0, -40.0],
                [-32.0, 0.0, -32.0],
                [-24.0, 0.0, -32.0],
                [-24.0, 0.0, -40.0],
            ],
        );
        assert!(mesh.positions.iter().all(|p| p[1] >= 0.0 && p[1] < 3.0));
        assert!(
            mesh.positions.iter().any(|p| p[1] > 0.0),
            "forest must include leaf litter"
        );
    }

    #[test]
    fn cached_grove_neighbors_match_source_plot_scans_at_edges_and_interiors() {
        let sources = (0..8).flat_map(|row| (0..6).map(move |column| {
            let y = row % 4;
            let x = column % 2;
            source_with_tile(0x05, x, y,
                (if y == 0 { 0x1e } else if y == 3 { 0x3e } else { 0x2e }) + x as u16)
        })).collect();
        let frame = frame(6, 8, sources);
        let g = geometry(&frame);
        let cells: Vec<_> = frame.tiles.iter().collect();
        for map in ["NewBarkTown", "Route29", "Route30"] {
            let plots = tree_placements(map, &cells, &g);
            assert!(plots.len() > 4);
            let mut mesh = TerrainMeshData::default();
            let mut allocation = None;
            for &p in &plots {
                let contains = |x: i32, y: i32| plots.iter().any(|q| {
                    x >= q.column as i32 && x < (q.column + q.width) as i32
                        && y >= q.row as i32 && y < (q.row + q.height) as i32
                });
                let x = p.column as i32;
                let y = p.row as i32;
                let expected = [contains(x - 1, y + p.height as i32 / 2),
                    contains(x + p.width as i32, y + p.height as i32 / 2),
                    contains(x + p.width as i32 / 2, y - 1),
                    contains(x + p.width as i32 / 2, y + p.height as i32)];
                assert_eq!(grove_neighbors(&mut mesh, &cells, &g, map, p), expected);
                let pointer = mesh.tree_neighborhood.as_ref().unwrap().as_ptr();
                if let Some(previous) = allocation { assert_eq!(pointer, previous); }
                allocation = Some(pointer);
            }
        }
    }

    #[test]
    fn modeled_grove_keeps_closed_water_contacts_without_changing_floor_or_claims() {
        for scale in [1., 4.] {
            let mut f = frame(4, 4, vec![source_with_tile(0x35,0,0,0x14);16]);
            f.tile_size *= scale;
            for (i, tile) in [(5,0x1e),(6,0x1f),(9,0x3e),(10,0x3f)] {
                f.tiles[i].source = source_with_tile(0x61, ((i%4)-1) as u8,
                    ((i/4)-1) as u8, tile);
            }
            let mut g = geometry(&f);
            g.tile_width *= scale;
            g.tile_height *= scale;
            let cells: Vec<_> = f.tiles.iter().collect();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![CellShape::Water.surface_height(g.tile_height);16],
                authored_cells: vec![None;16], ..Default::default()
            };
            for i in [5,6,9,10] {
                mesh.footing_heights[i] = 0.;
                mesh.authored_cells[i] = Some("johto/tree");
            }
            let footing = mesh.footing_heights.clone();
            let claims = mesh.authored_cells.clone();
            append_grove_water_contacts(&mut mesh, &cells, &g, [0,0]);
            assert_eq!(mesh.footing_heights, footing);
            assert_eq!(mesh.authored_cells, claims);
            assert!(!mesh.solid.indices.is_empty());
            let mut contacts = 0;
            for i in [5,6,9,10] {
                for direction in Direction::ALL {
                    let (dx,dy) = direction.offset();
                    let x = i%4;
                    let y = i/4;
                    let neighbor = (y as isize+dy) as usize*4+(x as isize+dx) as usize;
                    if claims[neighbor].is_some() { continue; }
                    contacts += 1;
                    let (w,e,n,s) = g.bounds(x,y);
                    let origin = Vec3::new((w+e)*0.5,-g.tile_height*0.125,(n+s)*0.5);
                    let direction = Vec3::new(dx as f32,0.,dy as f32);
                    let hit = mesh.solid.indices.chunks_exact(3).any(|ids| {
                        let p = [ids[0],ids[1],ids[2]].map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                        let e1 = p[1]-p[0]; let e2 = p[2]-p[0];
                        let h = direction.cross(e2); let det = e1.dot(h);
                        if det.abs() < 1e-7 { return false; }
                        let delta = origin-p[0]; let u = delta.dot(h)/det;
                        let q = delta.cross(e1); let v = direction.dot(q)/det;
                        let distance = e2.dot(q)/det;
                        (0.0..=1.0).contains(&u) && v>=0. && u+v<=1.
                            && distance>0. && distance<g.tile_width*0.75
                    });
                    assert!(hit, "open modeled grove contact at {i}, scale {scale}");
                }
            }
            assert_eq!(contacts,8);
            assert!(mesh.solid.positions.iter().all(|p|
                p[1]>=CellShape::Water.surface_height(g.tile_height)-0.001 && p[1]<=0.001));
            for invalid in 0..3 {
                let mut mesh = TerrainMeshData {
                    footing_heights: footing.clone(), authored_cells: claims.clone(), ..Default::default()
                };
                let mut changed = f.clone();
                match invalid {
                    0 => mesh.authored_cells.fill(Some("custom/tree")),
                    1 => for i in [5,6,9,10] { mesh.footing_heights[i]=g.tile_height; },
                    _ => for i in [0,1,2,3,4,7,8,11,12,13,14,15] {
                        changed.tiles[i].source.tileset_id = std::sync::Arc::from("unknown");
                    },
                }
                append_grove_water_contacts(&mut mesh, &changed.tiles.iter().collect::<Vec<_>>(), &g, [0,0]);
                assert!(mesh.solid.positions.is_empty());
            }
        }
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
        assert_eq!(
            b,
            build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap()
        );
        // The shared pine's elevated limbs and needles may overlap adjacent
        // crowns. Its 13-by-13 fluted trunk stays in the blocked source plot;
        // the former broadleaf generator's vertex count included crown faces.
        assert!(
            a.solid.positions[..13 * 13].iter()
                .chain(&b.solid.positions[..13 * 13])
                .all(|p| p[0] >= -12.001
                    && p[0] <= 4.001
                    && p[2] >= -0.001
                    && p[2] <= 16.001
                    && p[1] >= 0.)
        );
        // Ground flowers can rise above 0.6 outside the blocked tree plot.
        // The pine owns explicit cutaway ranges; measure that geometry rather
        // than treating every elevated ground accent as part of its crown.
        assert!(!a.solid.cutaway_ranges.is_empty() && !b.solid.cutaway_ranges.is_empty());
        let crowns = || [&a.solid, &b.solid].into_iter().flat_map(|mesh|
            mesh.cutaway_ranges.iter().flat_map(|range| mesh.positions[range.clone()].iter()));
        assert!(crowns()
            .filter(|p| p[1] > 0.6)
            .all(|p| p[0] >= -24. && p[0] <= 16.
                && p[2] >= -12. && p[2] <= 28. && p[1] <= 64.),
            "crown exceeds the readable plot margin: {:?}",
            crowns()
                .filter(|p| p[1] > 0.6 && (p[0] < -24. || p[0] > 16.
                    || p[2] < -12. || p[2] > 28. || p[1] > 64.)).take(3).collect::<Vec<_>>());
        assert_eq!(a.footing_heights, b.footing_heights);

        let mut shared = None;
        for map in ["NewBarkTown", "Route29", "CherrygroveCity", "Route30",
            "Route31", "VioletCity", "Route26", "Route43", "LakeOfRage",
            "Route32", "Route35", "Route42", "Route44", "Route45", "Route46",
            "BlackthornCity", "EcruteakCity", "MahoganyTown"] {
            let g = geometry(&frame);
            let cells: Vec<_> = frame.tiles.iter().collect();
            let shapes: Vec<_> = cells.iter().map(|c| shape_for_source(&c.source)).collect();
            let plots = tree_placements(map, &cells, &g);
            assert_eq!(plots.len(), 1, "{map}");
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.; cells.len()], ..Default::default()
            };
            let mut claimed = vec![false; cells.len()];
            assert!(append_tree(&mut mesh, &cells, &shapes, &g, map, plots[0],
                &mut claimed, frame.grid_origin.to_array()), "{map}");
            assert_eq!(claimed.iter().filter(|&&c| c).count(), 8);
            assert!(mesh.footing_heights.iter().all(|&h| h == 0.));
            if let Some(reference) = &shared {
                assert_eq!(&mesh.solid, reference, "{map} must use the shared grove asset");
            } else { shared = Some(mesh.solid); }
        }
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
    fn kanto_quiet_path_finishes_blank_and_speckled_samples_together() {
        let f = frame(4, 4, (0..16).map(|i|
            material_source("kanto", 0x01, i % 4, i / 4,
                if i == 4 || i == 14 { 0x39 } else { 0x23 })
        ).collect());
        let g = geometry(&f);
        let cells = f.tiles.iter().collect::<Vec<_>>();
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 16], authored_cells: vec![None; 16],
            ..Default::default()
        };
        for (i, tile) in cells.iter().enumerate() {
            assert_eq!(ground_material(&tile.source), Some(GroundMaterial::Path));
            let (w, e, n, s) = g.bounds(i % 4, i / 4);
            append_top(&mut mesh.textured, [w, e, n, s], 0., g.uv(i % 4, i / 4));
        }
        let feet = mesh.footing_heights.clone();
        let claims = mesh.authored_cells.clone();
        let corners = mesh.textured.positions.clone();
        polish_world_surfaces(&mut mesh, "Route22", &cells, &g, [0, 0]);
        assert!(mesh.textured.positions.is_empty(), "blank path cells must not retain raster paint");
        assert!(corners.iter().all(|p| mesh.solid.positions.contains(p)));
        assert_eq!(mesh.footing_heights, feet);
        assert_eq!(mesh.authored_cells, claims);
        assert_eq!(ground_material(&material_source("unknown_mod", 0x01, 0, 0, 0x23)), None);
        assert_eq!(ground_material(&material_source("kanto", 0x01, 0, 0, 0x24)), None);
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
            mesh.solid.quad_count() >= 4 && mesh.solid.positions.iter().all(|p| p[1] < 0.6),
            "sampled ground keeps its datum, with only low decorative grass"
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
        append_top(&mut mesh.textured, [w, e, n, s], 0.0, geometry.uv(0, 0));
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
        assert!(mesh.solid.positions.len() > original.len());
        for p in original { assert!(mesh.solid.positions.contains(&p), "native shoreline endpoints stay closed"); }
        assert!(mesh.solid.positions.iter().all(|p| p[1] >= -2.0 && p[1] <= 0.0));
        assert!(mesh.solid.uvs.iter().all(|uv| uv[0] < -9000.0 && uv[1] < -30000.0));
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
    fn runtime_path_uses_authored_geometry_with_live_source_roof_paint() {
        let source_frame = drawing(&[&[0x14, 0x15]], 3);
        let mut sources: Vec<_> = source_frame
            .tiles
            .into_iter()
            .map(|tile| tile.source)
            .collect();
        sources.extend((0..8).map(|_| source_with_tile(0x01, 0, 0, 0x06)));
        let mut frame = frame(8, 5, sources);
        frame.map_id = std::sync::Arc::from(MAP);
        // Replacement geometry keeps the live roof paint. Missing paint must
        // preserve the original drawing rather than invent a building color.
        let missing = build_terrain_mesh_with_samples(&frame, &TerrainImageSamples::default()).unwrap();
        assert_eq!(missing.textured.quad_count(), 32);
        let mut images = TerrainImageSamples::default();
        let roof: Vec<_> = (0..64).flat_map(|i| match i % 4 {
            0 => [20,35,55,255], 1 => [30,130,230,255],
            2 => [100,230,240,255], _ => [245,245,245,255],
        }).collect();
        for tile in &frame.tiles {
            images.pixels.insert(tile.texture.id(), TileImageSample::Rgba(roof.clone()));
        }
        let mesh =
            build_terrain_mesh_with_samples(&frame, &images).unwrap();
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
                >= 1000 + 12 * 24
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
            build_animated_flowers(&frame, &Assets::<Image>::default(), &[]).unwrap();
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

    #[test]
    fn native_rock_shoulders_share_source_pigment_across_routes_without_changing_support() {
        let f = frame(1,1,vec![material_source("johto",0x0a,1,1,0x3c)]);
        let cells: Vec<_> = f.tiles.iter().collect();
        let g = GridGeometry { width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let h = shape_for_source(&cells[0].source).surface_height(8.);
        let mut gray = TerrainMeshData { footing_heights:vec![h],authored_cells:vec![None], ..Default::default() };
        append_top(&mut gray.textured, g.bounds(0,0).into(), h, g.uv(0,0));
        let mut ochre = gray.clone();
        polish_world_surfaces(&mut gray,"Route29",&cells,&g,[0,0]);
        polish_world_surfaces(&mut ochre,"Route26",&cells,&g,[0,0]);
        assert_eq!(gray.footing_heights,ochre.footing_heights);
        assert_eq!(gray.solid.positions,ochre.solid.positions);
        assert_eq!(gray.solid.normals,ochre.solid.normals);
        assert!(!ochre.solid.uvs.is_empty());
        assert!(ochre.solid.uvs.iter().all(|uv| uv[0] < -9000. && uv[1] < -30000.));
        // The byte-identical native drawing owns the material family.
        // A map name alone must not recolor the same shoulder gray.
        assert_eq!(gray.solid.uvs, ochre.solid.uvs);
        assert_eq!(gray.solid.colors, ochre.solid.colors);
    }

    #[test]
    fn park_water_glints_keep_source_phase_and_original_floor() {
        let f = frame(1,1,vec![material_source("park",0x3f,0,0,0x14)]);
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut previous = None;
        for (x,z) in [(0.,0.),(-8.,-16.)] {
            let g = GridGeometry { width:1,height:1,tile_width:8.,tile_height:8.,origin_x:x,origin_z:z };
            let mut mesh = TerrainMeshData { footing_heights:vec![-2.],authored_cells:vec![None], ..Default::default() };
            append_top(&mut mesh.textured,g.bounds(0,0).into(),-2.,g.uv(0,0));
            let before = mesh.textured.positions.clone();
            polish_world_surfaces(&mut mesh,"NationalPark",&cells,&g,[20,24]);
            assert_eq!(mesh.solid.positions,before);
            assert_eq!(mesh.footing_heights,vec![-2.]);
            assert!(mesh.textured.positions.is_empty());
            assert!(mesh.solid.uvs.iter().all(|uv| uv[0]>40000. && uv[0]<60000. && uv[1] < -40000.));
            if let Some(uvs) = previous { assert_eq!(mesh.solid.uvs,uvs); }
            previous = Some(mesh.solid.uvs);
        }
    }

    #[test]
    fn live_pools_keep_original_uvs_geometry_and_support_across_map_families() {
        for (map, tileset, metatile, column, row) in [
            ("LakeOfRage", "johto", 0x35, 0, 0),
            ("Route40", "johto", 0x35, 0, 0),
            ("Route19", "kanto", 0x43, 0, 0),
            ("CinnabarIsland", "kanto", 0x43, 0, 0),
            ("IlexForest", "forest", 0x35, 0, 0),
            ("NationalPark", "park", 0x3f, 0, 0),
            ("SafariZoneBeta", "park", 0x3f, 0, 0),
            ("VermilionPort", "port", 0x0a, 0, 0),
            ("CeruleanGym", "port", 0x2e, 2, 0),
        ] {
        let mut f=frame(1,1,vec![material_source(tileset,metatile,column,row,0x14)]);
        f.map_id=map.into();
        let first=Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(8101),std::marker::PhantomData);
        let second=Handle::<Image>::Uuid(bevy::asset::uuid::Uuid::from_u128(8102),std::marker::PhantomData);
        f.tiles[0].texture=first.clone();f.tiles[0].animation_frames=Some(vec![first,second].into());
        let cells:Vec<_>=f.tiles.iter().collect();
        let g=GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let p=[[0.,-2.,0.],[0.,-2.,8.],[8.,-2.,8.],[8.,-2.,0.]];
        let uv=[[0.,0.],[0.,1.],[1.,1.],[1.,0.]];
        let mut mesh=TerrainMeshData {footing_heights:vec![-2.],authored_cells:vec![None],..Default::default()};
        append_quad_colors(&mut mesh.textured,p,[0.,1.,0.],uv,[[1.;4];4]);
        let source=mesh.textured.clone();
        polish_world_surfaces(&mut mesh,map,&cells,&g,[10,20]);
        assert_eq!(mesh.textured,source,"{map} keeps its live water binding");
        assert!(mesh.solid.positions.is_empty(),"{map} must not bake static ripple bars");
        assert_eq!(mesh.footing_heights,[-2.]);assert_eq!(mesh.authored_cells,[None]);
        }
    }

    #[test]
    fn park_basin_water_underlay_requires_complete_fixture_ownership() {
        let f = frame(1,1,vec![material_source("park",0x3f,3,0,0x5f)]);
        let cells: Vec<_> = f.tiles.iter().collect();
        let g = GridGeometry { width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let p = [[0.,-2.,0.],[0.,-2.,8.],[8.,-2.,8.],[8.,-2.,0.]];
        let water = material_source("park",0x3f,0,0,0x14);
        assert_eq!(surface_material("NationalPark",&p,[0.,1.,0.],&water,&cells,&g,
            &[Some("park/pond-basin")]),Some(GroundMaterial::Water));
        assert_eq!(surface_material("NationalPark",&p,[0.,1.,0.],&water,&cells,&g,
            &[None]),None);
        assert_eq!(surface_material("Route29",&p,[0.,1.,0.],&water,&cells,&g,
            &[Some("park/pond-basin")]),None);
    }

    #[test]
    fn park_masonry_receives_canopy_shade_without_changing_its_partition() {
        let sources = (0..36).map(|i| {
            let x = i % 6;
            let y = i / 6;
            if (1..5).contains(&x) && (1..5).contains(&y) {
                material_source("park", 6, (x - 1) as u8, (y - 1) as u8,
                    (0x0c + x - 1 + (y - 1) * 0x10) as u16)
            } else if y == 0 {
                material_source("park", 0x01, x as u8 % 4, 0, 0)
            } else {
                material_source("park", crate::park::PARK_TREE_GROUND_METATILE, 0, 0, 1)
            }
        }).collect();
        let f = frame(6, 6, sources);
        let cells: Vec<_> = f.tiles.iter().collect();
        let g = GridGeometry { width: 6, height: 6, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let p = [[0., 0., 0.], [0., 0., 8.], [8., 0., 8.], [8., 0., 0.]];
        let mut finish = GroundFinish::new("NationalPark", &cells, &g, [0, 0]);
        assert_eq!(finish.shadows.iter().filter(|s| s.tree).count(), 1);
        let mut shaded = SurfaceMeshData::default();
        finish.append_surface(&mut shaded, GroundMaterial::Pavers, p);
        assert_eq!(finish.park_canopy_darkness([128., 0., 128.]), 0.);
        finish.shadows.clear();
        let mut exposed = SurfaceMeshData::default();
        finish.append_surface(&mut exposed, GroundMaterial::Pavers, p);
        assert_eq!(exposed.positions, shaded.positions);
        assert_eq!(exposed.indices, shaded.indices);
        assert_eq!(exposed.normals, shaded.normals);
        assert!(exposed.colors.iter().zip(&shaded.colors)
            .any(|(a, b)| b[0] < a[0] - 0.01));
        assert!(exposed.colors.iter().zip(&shaded.colors)
            .all(|(a, b)| (0..3).all(|c| b[c] <= a[c] && b[c] >= a[c] * 0.77)));
        for y in 1..5 { for x in 1..5 {
            let mut changed = f.tiles.clone();
            changed[y * 6 + x].source.tile_index ^= 1;
            let cells: Vec<_> = changed.iter().collect();
            let refused = GroundFinish::new("NationalPark", &cells, &g, [0, 0]);
            assert!(refused.shadows.iter().all(|s| !s.tree));
        }}
    }

    #[test]
    fn park_plaza_finishes_all_pavement_samples_at_the_original_floor() {
        let drawing = [0,0x16,0,0x16, 6,0,6,0, 0,0x16,0,0x16, 6,0,6,0];
        let sources = drawing.into_iter().enumerate().map(|(i,tile)|
            material_source("park",0x01,(i%4) as u8,(i/4) as u8,tile)).collect();
        let frame = frame(4,4,sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry { width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.;16], authored_cells:vec![None;16], ..Default::default()
        };
        for i in 0..16 {
            append_top(&mut mesh.textured,g.bounds(i%4,i/4).into(),0.,g.uv(i%4,i/4));
        }
        polish_world_surfaces(&mut mesh,"NationalPark",&cells,&g,[0,0]);
        assert!(mesh.textured.positions.is_empty(),"no checker or pixel-art pavement remains");
        assert!(!mesh.solid.positions.is_empty());
        assert!(mesh.solid.positions.iter().all(|p| (0. ..0.9).contains(&p[1])
            && (0. ..=32.).contains(&p[0]) && (0. ..=32.).contains(&p[2])));
        // Decorations sit above the plane; indexed floor triangles must
        // still partition exactly the original plaza at its original height.
        let area = level_floor_area(&mesh.solid);
        assert!((area-32.*32.).abs()<0.001,
            "staggered park masonry must cover exactly the original plaza");
        assert_eq!(mesh.footing_heights,vec![0.;16]);
        assert_eq!(mesh.authored_cells,vec![None;16]);
    }

    #[test]
    fn park_pool_coping_finishes_its_recessed_wall_without_moving_it() {
        let frame = frame(1,1,vec![material_source("park",0x08,1,0,0x15)]);
        let cells:Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let floor = CellShape::Water.surface_height(g.tile_height);
        let wall = [[0.,floor,0.],[0.,floor,8.],[0.,0.,8.],[0.,0.,0.]];
        let mut mesh = TerrainMeshData {footing_heights:vec![0.],authored_cells:vec![None],..Default::default()};
        let (u0,u1,v0,v1) = g.uv(0,0);
        append_quad_colors(&mut mesh.textured,wall,[-1.,0.,0.],
            [[u0,v0],[u0,v1],[u1,v1],[u1,v0]],[TEXTURED_SHADE;4]);
        polish_world_surfaces(&mut mesh,"NationalPark",&cells,&g,[0,0]);
        assert!(mesh.textured.positions.is_empty());
        assert_eq!(mesh.solid.positions,wall);
        assert_eq!(mesh.solid.normals,vec![[-1.,0.,0.];4]);
        assert_eq!(mesh.footing_heights,vec![0.]);
    }

    fn level_floor_area(mesh: &SurfaceMeshData) -> f64 {
        mesh.indices.chunks_exact(3).filter_map(|indices| {
            let p = [indices[0], indices[1], indices[2]]
                .map(|i| Vec3::from_array(mesh.positions[i as usize]));
            p.iter().all(|p| p.y == 0.0).then(||
                f64::from((p[1] - p[0]).cross(p[2] - p[0]).y.abs()) * 0.5)
        }).sum()
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
                    matches!(tile, 0x12 | 0x19 | 0x9a | 0x9b | 0xaa | 0xab)
                        .then_some(GroundMaterial::Stone)
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
    fn ice_dotted_ground_requires_the_exact_native_phase() {
        for (block, x, y) in [
            (0x10, 3, 1),
            (0x10, 1, 3),
            (0x12, 0, 0),
            (0x12, 3, 0),
            (0x37, 1, 2),
            (0x37, 1, 3),
            (0x38, 0, 2),
            (0x38, 1, 3),
        ] {
            assert_eq!(
                ground_material(&material_source("ice_path", block, x, y, 0x12)),
                Some(GroundMaterial::Stone)
            );
            assert_eq!(
                ground_material(&material_source("ice_path", block, 2, y, 0x12)),
                None
            );
        }
        for (block, x, y) in [
            (0x07, 1, 0),
            (0x12, 1, 1),
            (0x1a, 0, 3),
            (0x24, 3, 2),
            (0x29, 1, 3),
            (0x37, 3, 1),
        ] {
            assert_eq!(
                ground_material(&material_source("ice_path", block, x, y, 0xab)),
                Some(GroundMaterial::Stone)
            );
            assert_eq!(
                ground_material(&material_source("ice_path", block, (x + 1) % 4, y, 0xab)),
                None
            );
            assert_eq!(
                ground_material(&material_source("ice_path", block, x, y, 0xff)),
                None
            );
        }
        assert_eq!(
            ground_material(&material_source("ice_path", 0x30, 1, 0, 0xab)),
            None
        );
        assert_eq!(
            ground_material(&material_source("cave", 0x07, 1, 0, 0xab)),
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
        assert!(mesh.solid.positions.iter().all(|p| (0.0..0.9).contains(&p[1])));
        assert!((level_floor_area(&mesh.solid)
            - 12.0 * f64::from(g.tile_width * g.tile_height)).abs() < 0.001,
            "decoration must retain the full source floor outside the prop quadrant");
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

#[cfg(test)]
mod streamed_grove_bounds_tests {
    use super::*;
    #[test]
    fn streaming_bounds_contain_every_source_leaf_and_branch() {
        for coordinates in [[-7, 3], [0, 0], [4, 8], [15, -19], [1, 2], [3, 5]] {
            for mask in 0u32..16 {
                let neighbors = std::array::from_fn(|i| mask & (1 << i) != 0);
                for tile_height in [8.,32.] {
                    for plot in [[2.,2.], [1.,4.], [4.,1.]] {
                    for family in [crate::foliage::GroveFamily::JohtoPine,crate::foliage::GroveFamily::AncientForest] {
                    let recipe = GroveRecipe { family,
                        bounds:[-tile_height*plot[0],0.,0.,tile_height*plot[1]],
                        base_height:13.,tile_height,coordinates,neighbors,coverage_fade:true };
                    let (low, high) = recipe.conservative_bounds();
                    let mut mesh = TerrainMeshData::default();
                    recipe.append(&mut mesh);
                    for p in mesh.solid.positions {
                        let p = bevy::prelude::Vec3::from_array(p);
                        assert!(p.cmpge(low).all() && p.cmple(high).all(),
                            "{coordinates:?} {neighbors:?} {family:?} {plot:?}: {p:?} outside {low:?}..{high:?}");
                    }
                    }
                    }
                }
            }
        }
    }
}
