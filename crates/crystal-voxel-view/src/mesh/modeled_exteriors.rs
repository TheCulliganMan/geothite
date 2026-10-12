//! Source-complete world exterior kits. Authoritative tiles, phases, native
//! ground and door seams select art; collision and simulation are never read.
use super::*;
#[path = "city_facades.rs"]
mod city_facades;
#[path = "folded_roofs.rs"]
pub(super) mod folded_roofs;
#[path = "kanto_boundary_rocks.rs"]
mod kanto_boundary_rocks;
#[path = "kanto_capped_posts.rs"]
mod kanto_capped_posts;
#[path = "park_hoop_fence.rs"]
pub(super) mod park_hoop_fence;
#[path = "structure_extensions.rs"]
mod structure_extensions;
#[path = "park_flowers.rs"]
pub(super) mod park_flowers;
use crate::exterior_models::{Kind, model};
use crate::new_bark_models::{ModelKind as Johto, model as johto_model};

// Share the existing planting producer through its owning module boundary.
pub(super) fn append_park_underplanting(mesh: &mut SurfaceMeshData,
    bounds: [f32;4], cell: [i32;2]) {
    park_flowers::append_underplanting(mesh,bounds,cell);
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Asset {
    World(Kind),
    Johto(Johto),
    ForestTree,
}
impl Asset {
    fn label(self) -> &'static str {
        match self {
            Self::World(k) => k.label(),
            Self::ForestTree => "world-exterior/forest_tree",
            Self::Johto(k) => match k {
                Johto::House => "johto/house",
                Johto::PlayerHouse => "johto/player_house",
                Johto::Lab => "johto/lab",
                Johto::Pokecenter => "johto/pokecenter",
                Johto::Mart => "johto/mart",
                Johto::RouteGate => "johto/route_gate",
                Johto::TraditionalHouse => "johto/traditional_house",
                Johto::VioletGym => "johto/violet_gym",
                Johto::SproutTower => "johto/sprout_tower",
                Johto::Tree | Johto::TreeLod => "johto/tree",
                Johto::Grass | Johto::GrassLod => "johto/grass",
                Johto::Flowers => "johto/flowers",
            },
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Building {
    asset: Asset,
    door: Option<f32>,
    east: bool,
    depth_cells: Option<f32>,
}
fn outdoor(tileset: &str) -> bool {
    matches!(
        tileset,
        "johto" | "johto_modern" | "kanto" | "forest" | "park" | "battle_tower_outside"
    )
}

/// A same-number ground tile from a connected map's different tileset is not
/// a valid underlay. Do not guess a material when the live source lacks one.
fn ground(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    tileset: &str,
    tile: u16,
    metatile: Option<u16>,
) -> Option<usize> {
    cells
        .iter()
        .zip(shapes)
        .position(|(c, s)| {
            c.source.tileset_id.as_ref() == tileset
                && c.source.tile_index == tile
                && metatile.is_none_or(|m| c.source.metatile_id == m)
                && matches!(
                    s,
                    CellShape::Flat
                        | CellShape::Water
                        | CellShape::RaisedTop {
                            solid: SolidKind::Bank,
                            ..
                        }
                )
        })
        .or_else(|| {
            // Some complete maps deliberately contain no sample of the catalog's
            // preferred path. Use only independently identified ground from the
            // same tileset; never borrow a numbered tile from a connected map.
            if metatile.is_some() {
                return None;
            }
            cells.iter().zip(shapes).position(|(c, shape)| {
                c.source.tileset_id.as_ref() == tileset
                    && matches!(shape, CellShape::Flat | CellShape::PlaneAt { height: 0.0 })
                    && match (tileset, tile, c.source.metatile_id, c.source.tile_index) {
                        ("johto" | "johto_modern", 0x06, 0x02, 0x05) => true,
                        ("kanto", KANTO_GROUND_TILE_INDEX, 0x31 | 0x7b, 0x39) => true,
                        _ => false,
                    }
            })
        })
}
fn clear(c: &[bool], g: &GridGeometry, rect: [usize; 4]) -> bool {
    let [x, y, w, h] = rect;
    x + w <= g.width
        && y + h <= g.height
        && (y..y + h).all(|yy| (x..x + w).all(|xx| !c[yy * g.width + xx]))
}
fn floor(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    rect: [usize; 4],
    sample: usize,
    claimed: &mut [bool],
) {
    let [x, y, w, h] = rect;
    let uv = g.uv(sample % g.width, sample / g.width);
    for yy in y..y + h {
        for xx in x..x + w {
            let (w, e, n, s) = g.bounds(xx, yy);
            append_top(
                &mut mesh.textured,
                [w, e, n, s],
                shapes[sample].surface_height(g.tile_height),
                uv,
            );
            claimed[yy * g.width + xx] = true;
        }
    }
    let _ = cells;
}
fn bounds(g: &GridGeometry, r: [usize; 4]) -> [f32; 4] {
    let [x, y, w, h] = r;
    let (west, _, north, _) = g.bounds(x, y);
    [
        west,
        west + w as f32 * g.tile_width,
        north,
        north + h as f32 * g.tile_height,
    ]
}
fn phases(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> bool {
    if p.width == 0 || p.height == 0 || p.column + p.width > g.width || p.row + p.height > g.height
    {
        return false;
    }
    let first = &cells[p.row * g.width + p.column].source;
    (0..p.height).all(|y| {
        (0..p.width).all(|x| {
            let s = &cells[(p.row + y) * g.width + p.column + x].source;
            let sx = usize::from(first.subtile_column) + x;
            let sy = usize::from(first.subtile_row) + y;
            let bx = x.saturating_sub(sx % 4);
            let by = y.saturating_sub(sy % 4);
            let peer = &cells[(p.row + by) * g.width + p.column + bx].source;
            s.tileset_id == first.tileset_id
                && usize::from(s.subtile_column) == sx % 4
                && usize::from(s.subtile_row) == sy % 4
                && s.metatile_id == peer.metatile_id
        })
    })
}
fn drawing(
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
    tileset: &str,
    rows: &[&[u16]],
    skip: usize,
) -> bool {
    p.width == rows[0].len() * 4
        && p.height + skip == rows.len() * 4
        && p.column + p.width <= g.width
        && p.row + p.height <= g.height
        && (0..p.height).all(|y| {
            (0..p.width).all(|x| {
                let s = &cells[(p.row + y) * g.width + p.column + x].source;
                let sy = y + skip;
                s.tileset_id.as_ref() == tileset
                    && s.metatile_id == rows[sy / 4][x / 4]
                    && s.subtile_column as usize == x % 4
                    && s.subtile_row as usize == sy % 4
            })
        })
}
fn door_column(cells: &[&VisualTile], g: &GridGeometry, p: BuildingPlacement) -> Option<f32> {
    if p.height < 2 {
        return None;
    }
    let tile = |x: usize, y: usize| {
        cells[(p.row + y) * g.width + p.column + x]
            .source
            .tile_index
    };
    let families = [
        [[0x37, 0x38], [0x39, 0x3a]],
        [[0x27, 0x28], [0x29, 0x2a]],
        [[0x18, 0x19], [0x17, 0x17]],
        [[0x0b, 0x0c], [0x1b, 0x1c]],
        [[0x42, 0x43], [0x4a, 0x4a]],
        [[0x44, 0x45], [0x4a, 0x4a]],
        [[0x07, 0x08], [0x17, 0x18]],
    ];
    (0..p.width - 1)
        .find(|&x| {
            families.iter().any(|f| {
                (0..2).all(|y| (0..2).all(|dx| tile(x + dx, p.height - 2 + y) == f[y][dx]))
            })
        })
        .map(|x| (x + 1) as f32)
}
fn descriptor(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: BuildingPlacement,
) -> Option<Building> {
    if !phases(cells, g, p) {
        return None;
    }
    let first = &cells[p.row * g.width + p.column].source;
    let catalogued = BUILDING_TEMPLATES
        .iter()
        .any(|t| drawing(cells, g, p, t.tileset, t.rows, t.skip_top_source_rows))
        || (first.tileset_id.as_ref() == "kanto"
            && outdoor_building_placements(cells, g).contains(&p))
        || drawing(cells, g, p, "johto", &[&[0x08, 0x09], &[0x10, 0x11]], 0)
        || drawing(cells, g, p, "johto", &[&[0x1a, 0x11]], 0)
        || drawing(
            cells,
            g,
            p,
            "johto",
            &[&[0x2c, 0x2d], &[0x22, 0x23], &[0x28, 0x29]],
            2,
        );
    if !catalogued {
        return None;
    }
    let at = |x: usize, y: usize| {
        cells[(p.row + y) * g.width + p.column + x]
            .source
            .metatile_id
    };
    let door = door_column(cells, g, p);
    let mut east = false;
    let mut depth = None;
    let mut asset = match first.tileset_id.as_ref() {
        "johto" => {
            use Johto::*;
            if drawing(cells, g, p, "johto", &[&[0x08, 0x09], &[0x10, 0x11]], 0)
                || drawing(cells, g, p, "johto", &[&[0x18, 0x19], &[0x10, 0x11]], 0)
            {
                east = true;
                Asset::World(Kind::EastGate)
            } else if first.metatile_id == 0x08 && p.height == 28 {
                depth = Some(3.0);
                Asset::World(Kind::Lighthouse)
            } else if first.metatile_id == 0x20 && p.width == 8 && p.height == 8 {
                Asset::World(Kind::BurnedTower)
            } else if first.metatile_id == 0x24 && p.height == 16 {
                Asset::World(Kind::TinTower)
            } else if first.metatile_id == 0x2c
                && p.width == 12
                && p.height == 6
                && map == "EcruteakCity"
            {
                Asset::World(Kind::TinTower)
            } else if first.metatile_id == 0x2c && p.height == 10 {
                Asset::Johto(SproutTower)
            } else if first.metatile_id == 0x2c && p.height == 6 {
                Asset::Johto(if p.width == 12 {
                    VioletGym
                } else {
                    TraditionalHouse
                })
            } else if first.metatile_id == 0x14 && p.width == 8 && p.height == 4 {
                Asset::Johto(House)
            } else if first.metatile_id == 0x1a && p.width == 8 && p.height == 4 && door.is_some() {
                Asset::Johto(RouteGate)
            } else if first.metatile_id == 0x08 && p.height == 8 && at(0, 4) == 0x1c {
                Asset::World(Kind::JohtoBarn)
            } else if first.metatile_id == 0x18 && p.height == 8 {
                match (p.width, at(0, 4), at(p.width - 4, 4)) {
                    (8, 0x16, 0x1e) => Asset::Johto(PlayerHouse),
                    (8, 0x1a, 0x17) => Asset::Johto(Mart),
                    (8, 0x1a, 0x1b) => Asset::Johto(Pokecenter),
                    (8, 0x1a, 0x11) => Asset::Johto(RouteGate),
                    (12, 0x1c, 0x1e) => {
                        Asset::Johto(if map == "NewBarkTown" { Lab } else { VioletGym })
                    }
                    _ => return None,
                }
            } else {
                return None;
            }
        }
        "johto_modern" => {
            let kind = match (first.metatile_id, p.width, p.height) {
                (0x25, 8, 12) => Kind::RadioTower,
                (0x18, 12, 16) => Kind::ModernDepartment,
                (0x12 | 0x13 | 0x15 | 0x67, 4, 4) => Kind::ModernBlank,
                (0x14, 4, 4) => Kind::ModernShop,
                (0x18, 12, 8) => match at(4, 4) {
                    0x1d => Kind::ModernGym,
                    0x0f => Kind::ModernStation,
                    0x2c => Kind::ModernDaycare,
                    0x17 => Kind::ModernArcade,
                    0x23 => Kind::ModernBlank,
                    _ => return None,
                },
                (0x18, 8, 8) => match (at(0, 4), at(4, 4)) {
                    (0x16, 0x1e) => Kind::ModernHouse,
                    (0x1a, 0x33) => Kind::ModernMart,
                    (0x1a, 0x1b) => Kind::ModernCenter,
                    (0x10, 0x11) => {
                        east = true;
                        Kind::EastGate
                    }
                    (0x1c, 0x1e) => Kind::ModernBlank,
                    _ => return None,
                },
                _ => return None,
            };
            Asset::World(kind)
        }
        "kanto" => {
            let kind = match first.metatile_id {
                0x02 => Kind::KantoHouse,
                0x30 => Kind::KantoFlatHouse,
                0x38 => Kind::KantoHouse,
                0x75 => Kind::KantoMuseum,
                0x68 if p.height > 8 => {
                    if map == "CeladonCity" {
                        Kind::KantoMansion
                    } else if map == "LavenderTown" {
                        Kind::LavenderRadioTower
                    } else {
                        Kind::Silph
                    }
                }
                0x68 => Kind::KantoCenter,
                0x0c => {
                    if map == "Route10North" {
                        Kind::PowerPlant
                    } else {
                        Kind::KantoGym
                    }
                }
                0x20 if p.height > 8 => Kind::KantoDepartment,
                0x20 => match at(p.width - 4, p.height - 4) {
                    0x72 => Kind::KantoCenter,
                    0x73 => Kind::KantoMart,
                    _ => {
                        if map == "Route19" {
                            Kind::KantoRouteGate
                        } else if map == "SaffronCity" {
                            Kind::KantoStation
                        } else if map == "CeladonCity" {
                            Kind::KantoArcade
                        } else {
                            Kind::KantoFlatHouse
                        }
                    }
                },
                _ => return None,
            };
            Asset::World(kind)
        }
        "forest" if first.metatile_id == 0x1d && p.width == 12 && p.height == 12 => {
            Asset::World(if at(0, p.height - 4) == 0x25 {
                Kind::ForestGateClosed
            } else {
                Kind::ForestGate
            })
        }
        "battle_tower_outside" if p.width == 20 && p.height == 16 && first.metatile_id == 0x08 => {
            Asset::World(Kind::BattleTower)
        }
        _ => return None,
    };
    if door.is_none() && matches!(asset, Asset::World(Kind::KantoHouse | Kind::KantoFlatHouse)) {
        asset = Asset::World(Kind::KantoBlank);
    }
    // Large frontless side gates have a transverse passage. The exact native
    // bottom four source rows locate that opening; never add a south door.
    Some(Building {
        asset,
        door: if east {
            Some(p.height as f32 - 2.0)
        } else {
            door
        },
        east,
        depth_cells: depth,
    })
}

pub(super) fn building_placements(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Vec<BuildingPlacement> {
    let mut out = new_bark::building_placements(map, cells, g);
    for tile in cells {
        let s = &tile.source;
        if s.tileset_id.as_ref() != "johto"
            || s.metatile_id != 0x08
            || s.subtile_column != 0
            || s.subtile_row != 0
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
        if drawing(cells, g, p, "johto", &[&[0x08, 0x09], &[0x10, 0x11]], 0) {
            out.push(p);
        }
    }
    out.sort_by_key(|p| (p.row, p.column));
    out.dedup();
    out
}

pub(super) fn append_building(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    map: &str,
    p: BuildingPlacement,
    claimed: &mut [bool],
    map_origin: [i32; 2],
) -> bool {
    append_building_with_samples(mesh,cells,shapes,g,map,p,claimed,map_origin,None)
}

fn cabin_roof_palette(cells: &[&VisualTile], g: &GridGeometry,
    p: BuildingPlacement, images: &TerrainImageSamples) -> Option<[[f32;4];3]> {
    let mut colors = std::collections::BTreeSet::new();
    // Complete source roof interior, excluding its transparent/ground corners.
    for row in p.row..p.row+p.roof_rows {
        for column in p.column+2..p.column+p.width-2 {
            for pixel in tile_rgba(images,cells[row*g.width+column]).ok()?.chunks_exact(4) {
                if pixel[3] == 255 { colors.insert([pixel[0],pixel[1],pixel[2]]); }
            }
        }
    }
    // A complete source roof can use all four native palette entries, with
    // the brightest reserved for its border rather than the roof field.
    if !(2..=4).contains(&colors.len()) { return None; }
    let mut colors:Vec<_>=colors.into_iter().collect();
    colors.sort_by_key(|c|u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722);
    let linear = |c:[u8;3]| {
        let rgb=c.map(|v| {let s=f32::from(v)/255.;if s<=0.04045 {s/12.92} else {((s+0.055)/1.055).powf(2.4)}});
        [rgb[0],rgb[1],rgb[2],1.]
    };
    let dark=linear(colors[0]);
    let light=linear(colors[if colors.len()==4 {2} else {colors.len()-1}]);
    let mid=if colors.len()>=3 {linear(colors[1])} else {
        std::array::from_fn(|i|(dark[i]+light[i])*0.5)
    };
    Some([dark,mid,light])
}

pub(super) fn append_building_with_samples(
    mesh: &mut TerrainMeshData, cells: &[&VisualTile], shapes: &[CellShape],
    g: &GridGeometry, map: &str, p: BuildingPlacement, claimed: &mut [bool],
    map_origin: [i32;2], images: Option<&TerrainImageSamples>,
) -> bool {
    if structure_extensions::append_building(mesh, cells, shapes, g, p, claimed, map_origin) {
        return true;
    }
    let Some(d) = descriptor(map, cells, g, p) else {
        return false;
    };
    let rect = [p.column, p.row, p.width, p.height];
    if !clear(claimed, g, rect) {
        return false;
    }
    let roof_palette = if d.asset == Asset::Johto(Johto::TraditionalHouse)
        || (map == "LakeOfRage" && d.asset == Asset::Johto(Johto::House)) {
        if let Some(images)=images {
            let Some(palette)=cabin_roof_palette(cells,g,p,images) else {
                // Missing material coverage is shown as exact 2D, never purple
                // prototype paint or a legacy voxel-building substitute.
                for row in p.row..p.row+p.height {for column in p.column..p.column+p.width {
                    let (w,e,n,s)=g.bounds(column,row);
                    append_top(&mut mesh.textured,[w,e,n,s],0.,g.uv(column,row));
                    claimed[row*g.width+column]=true;
                }}
                return true;
            };
            Some(palette)
        } else {None}
    } else {None};
    let tileset = cells[p.row * g.width + p.column].source.tileset_id.as_ref();
    let Some(sample) = ground(cells, shapes, tileset, p.ground_tile_index, None) else {
        return false;
    };
    floor(mesh, cells, shapes, g, rect, sample, claimed);
    let mut b = bounds(g, rect);
    if let Some(depth) = d.depth_cells {
        b[2] = b[3] - depth * g.tile_height;
    }
    let door = d.door.map(|offset| {
        if d.east {
            b[2] + offset * g.tile_height
        } else {
            b[0] + offset * g.tile_width
        }
    });
    let cutaway_start = mesh.solid.positions.len();
    match d.asset {
        Asset::World(kind) => {
            model(kind).append_fitted(&mut mesh.solid, b, 0.0, g.tile_height * 2.0, door)
        }
        Asset::Johto(kind) => {
            let m=johto_model(kind);
            m.append_fitted(&mut mesh.solid, b, 0.0, g.tile_height * 2.0, door);
            if let Some(palette)=roof_palette {m.recolor_fitted_roof(&mut mesh.solid,cutaway_start,palette);}
        }
        Asset::ForestTree => unreachable!("trees cannot be building placements"),
    }
    if map == "GoldenrodCity"
        && let Asset::World(kind @ (Kind::ModernBlank | Kind::ModernHouse)) = d.asset
    {
        city_facades::finish(
            &mut mesh.solid,
            cutaway_start,
            model(kind),
            b,
            g.tile_height * 2.0,
            door,
            [
                p.column as i32 + map_origin[0],
                p.row as i32 + map_origin[1],
            ],
        );
    }
    if let Asset::World(
        kind @ (Kind::ModernHouse
        | Kind::ModernBlank
        | Kind::ModernShop
        | Kind::ModernMart
        | Kind::ModernGym
        | Kind::ModernStation
        | Kind::ModernDepartment
        | Kind::ModernArcade),
    ) = d.asset
    {
        folded_roofs::append(
            &mut mesh.solid,
            cutaway_start,
            model(kind),
            g.tile_height * 2.0,
            [
                p.column as i32 + map_origin[0],
                p.row as i32 + map_origin[1],
            ],
        );
    }
    // Eligibility belongs only to this successful authored building append.
    // Ground backing and the earlier cave/mesa/forecourt/ice branch stay opaque.
    mesh.solid
        .cutaway_ranges
        .push(cutaway_start..mesh.solid.positions.len());
    mark_authored_rect(mesh, g, rect, d.asset.label());
    true
}

fn tree_kind(cells: &[&VisualTile], g: &GridGeometry, p: TreePlacement) -> Option<Asset> {
    if p.column + p.width > g.width || p.row + p.height > g.height {
        return None;
    }
    let s = &cells[p.row * g.width + p.column].source;
    Some(match s.tileset_id.as_ref() {
        "johto" | "johto_modern" => Asset::Johto(Johto::Tree),
        "kanto" => Asset::World(Kind::KantoCanopy),
        "forest" => Asset::ForestTree,
        "park" => Asset::World(if p.width == 4 {
            Kind::KantoCanopy
        } else {
            Kind::ParkHedge
        }),
        "battle_tower_outside" => Asset::ForestTree,
        _ => return None,
    })
}
pub(super) fn tree_ground(
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    p: TreePlacement,
) -> Option<usize> {
    tree_kind(cells, g, p)?;
    let tileset = cells[p.row * g.width + p.column].source.tileset_id.as_ref();
    ground(
        cells,
        shapes,
        tileset,
        p.ground_tile_index,
        p.ground_metatile_id,
    )
    .or_else(|| {
        // These native maps have complete standard trees and real path
        // backing, but no lawn sample. Unknown maps and atlases still refuse.
        (matches!(map, "Route40" | "RuinsOfAlphOutside")
            && tileset == "johto"
            && p.ground_tile_index == 0x05)
            .then(|| ground(cells, shapes, "johto", 0x06, None))
            .flatten()
    })
}

pub(super) fn park_tree_ground(
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    p: TreePlacement,
) -> Option<usize> {
    if p.width != 4 || p.height != 4
        || p.column + 4 > g.width || p.row + 4 > g.height
        || !(0..4).all(|y| (0..4).all(|x| {
            crate::park::large_tree_local(&cells[(p.row + y) * g.width + p.column + x].source)
                == Some((x as u8, y as u8))
        }))
    {
        return None;
    }
    tree_ground(map, cells, shapes, g, p)
}

/// Missing authored trees stay exact flat source art; reserve against legacy
/// grouped extrusion without inventing a model or an authored floor claim.
pub(super) fn append_missing_tree_source(mesh: &mut TerrainMeshData,
    g: &GridGeometry, p: TreePlacement, claimed: &mut [bool]) {
    for row in p.row..p.row + p.height {
        for column in p.column..p.column + p.width {
            let i = row * g.width + column;
            let (w,e,n,s) = g.bounds(column,row);
            append_top(&mut mesh.textured,[w,e,n,s],p.base_height,g.uv(column,row));
            claimed[i] = true;
        }
    }
}

pub(super) fn append_tree(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    map: &str,
    p: TreePlacement,
    claimed: &mut [bool],
    map_origin: [i32; 2],
) -> bool {
    let Some(kind) = tree_kind(cells, g, p) else {
        return false;
    };
    let park_tree = cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "park"
        && p.width == 4;
    if park_tree && park_tree_ground(map, cells, shapes, g, p).is_none() {
        return false;
    }
    if kind == Asset::World(Kind::ParkHedge) {
        let first = &cells[p.row * g.width + p.column].source;
        let Some((fence, 0, 0)) = crate::park::hoop_fence_local(first) else {
            return false;
        };
        if !(0..2).all(|y| (0..2).all(|x| {
            let s = &cells[(p.row+y)*g.width+p.column+x].source;
            crate::park::hoop_fence_local(s) == Some((fence,x as u8,y as u8))
                && s.metatile_id == first.metatile_id
                && usize::from(s.subtile_column) == usize::from(first.subtile_column)+x
                && usize::from(s.subtile_row) == usize::from(first.subtile_row)+y
        })) {
            return false;
        }
    }
    let rect = [p.column, p.row, p.width, p.height];
    if !clear(claimed, g, rect) {
        return false;
    }
    let Some(sample) = tree_ground(map, cells, shapes, g, p) else {
        return false;
    };
    if kind == Asset::ForestTree && !crate::foliage::world_tree_available([
        map_origin[0] + p.column as i32, map_origin[1] + p.row as i32]) {
        append_missing_tree_source(mesh, g, p, claimed);
        return true;
    }
    floor(mesh, cells, shapes, g, rect, sample, claimed);
    if park_tree {
        crate::foliage::append_park_tree(mesh, bounds(g, rect), p.base_height, g.tile_height,
            [map_origin[0] + p.column as i32, map_origin[1] + p.row as i32]);
        mark_authored_rect(mesh, g, rect, "world-exterior/park_tree");
        return true;
    }
    if kind == Asset::World(Kind::ParkHedge) {
        // The historical kit key classified the native arches as foliage.
        // Retain catalog compatibility while drawing the actual open fence.
        let fence = crate::park::hoop_fence_local(
            &cells[p.row * g.width + p.column].source,
        ).expect("complete park fence drawing").0;
        park_hoop_fence::append(&mut mesh.solid, bounds(g, rect), g.tile_height, fence);
        mark_authored_rect(mesh, g, rect, "world-exterior/park_hoop_fence");
        return true;
    }
    let mut b = bounds(g, rect);
    let depth = (b[1] - b[0]).min(b[3] - b[2]);
    b[2] = b[3] - depth;
    let h = p.height as f32 * g.tile_height;
    if kind == Asset::ForestTree {
        let neighbors = super::new_bark::grove_neighbors(mesh, cells, g, map, p);
        crate::foliage::append_world_tree(
            mesh,
            Johto::Tree,
            b,
            p.base_height,
            g.tile_height,
            [
                map_origin[0] + p.column as i32,
                map_origin[1] + p.row as i32,
            ],
            neighbors,
        );
        mark_authored_rect(mesh, g, rect, kind.label());
        return true;
    }
    match kind {
        Asset::World(k) => {
            let m = model(k);
            m.append_fitted(
                &mut mesh.solid,
                b,
                p.base_height,
                h / (m.max[1] - m.min[1]),
                None,
            )
        }
        Asset::ForestTree => unreachable!("forest tree handled before catalog append"),
        Asset::Johto(_) => {
            let neighbors = super::new_bark::grove_neighbors(mesh, cells, g, map, p);
            crate::foliage::append_grove_tree(
                mesh,
                b,
                p.base_height,
                g.tile_height,
                [
                    map_origin[0] + p.column as i32,
                    map_origin[1] + p.row as i32,
                ],
                neighbors,
            );
        }
    }
    mark_authored_rect(mesh, g, rect, kind.label());
    true
}

#[derive(Clone, Copy)]
struct Prop {
    rect: [usize; 4],
    kind: Kind,
    ground: u16,
    height: f32,
}
fn props(cells: &[&VisualTile], g: &GridGeometry) -> Vec<Prop> {
    let mut out = Vec::new();
    for tile in cells {
        let s = &tile.source;
        if !outdoor(s.tileset_id.as_ref()) {
            continue;
        }
        let x = tile.column as usize;
        let y = tile.row as usize;
        // Exact phase-aware source classifier gives each cuttable crown its
        // own 2x2 drawing, so scripted replacement immediately removes it.
        if let Some(CellShape::FacadeBand {
            band_from_top: 0,
            band_count: 2,
            ground_tile_index,
            solid: SolidKind::CutTree,
            ..
        }) = crate::cut_tree::cut_tree_shape(s)
        {
            let complete=x+2<=g.width && y+2<=g.height && (0..2).all(|dy|(0..2).all(|dx|{
                let ss=&cells[(y+dy)*g.width+x+dx].source;
                ss.metatile_id==s.metatile_id && ss.tileset_id==s.tileset_id && ss.subtile_column==s.subtile_column+dx as u8 && ss.subtile_row==s.subtile_row+dy as u8
                    && matches!(crate::cut_tree::cut_tree_shape(ss),Some(CellShape::FacadeBand{band_from_top,solid:SolidKind::CutTree,..}) if band_from_top==dy as u8)
            }));
            // The leading source column is even for every authored Cut pair.
            if s.subtile_column % 2 == 0 && complete {
                out.push(Prop {
                    rect: [x, y, 2, 2],
                    kind: Kind::CutTree,
                    ground: ground_tile_index,
                    height: 2.1,
                });
            }
        }
        if matches!(s.tileset_id.as_ref(), "johto" | "johto_modern")
            && matches!(s.metatile_id, 0x45 | 0x47 | 0x78 | 0x3d)
            && s.tile_index == 0x4e
            && x + 2 <= g.width
            && y + 2 <= g.height
        {
            let art = [[0x4e, 0x4f], [0x5e, 0x5f]];
            let complete = (0..2).all(|dy| {
                (0..2).all(|dx| {
                    let ss = &cells[(y + dy) * g.width + x + dx].source;
                    ss.tileset_id == s.tileset_id
                        && ss.metatile_id == s.metatile_id
                        && ss.subtile_column == s.subtile_column + dx as u8
                        && ss.subtile_row == s.subtile_row + dy as u8
                        && ss.tile_index == art[dy][dx]
                })
            });
            if complete {
                out.push(Prop {
                    rect: [x, y, 2, 2],
                    kind: Kind::RouteSign,
                    ground: if s.metatile_id == 0x47 { 0x05 } else { 0x06 },
                    height: 2.0,
                });
            }
        }
        if s.tileset_id.as_ref() == "forest"
            && s.metatile_id == 0x20
            && s.subtile_column == 0
            && s.subtile_row == 0
            && x + 2 <= g.width
            && y + 3 <= g.height
        {
            if (0..3).all(|dy| {
                (0..2).all(|dx| {
                    let ss = &cells[(y + dy) * g.width + x + dx].source;
                    ss.tileset_id == s.tileset_id
                        && ss.metatile_id == 0x20
                        && ss.subtile_column == dx as u8
                        && ss.subtile_row == dy as u8
                        && matches!(
                            shape_for_source(ss),
                            CellShape::FacadeBand {
                                solid: SolidKind::Prop,
                                ..
                            }
                        )
                })
            }) {
                out.push(Prop {
                    rect: [x, y, 2, 3],
                    kind: Kind::ForestShrine,
                    ground: 0x05,
                    height: 3.0,
                });
            }
        }
        let rail = crate::johto_fence::johto_fence_shape(s)
            .or_else(|| crate::modern_route::modern_route_shape(s));
        if let Some(CellShape::FacadeBand {
            band_from_top: 0,
            band_count: 2,
            ground_tile_index,
            solid: SolidKind::Fence,
            ..
        }) = rail
        {
            if y + 1 < g.height {
                let next = &cells[(y + 1) * g.width + x].source;
                if next.tileset_id == s.tileset_id
                    && next.metatile_id == s.metatile_id
                    && next.subtile_column == s.subtile_column
                    && next.subtile_row == s.subtile_row + 1
                    && matches!(
                        crate::johto_fence::johto_fence_shape(next)
                            .or_else(|| crate::modern_route::modern_route_shape(next)),
                        Some(CellShape::FacadeBand {
                            band_from_top: 1,
                            solid: SolidKind::Fence,
                            ..
                        })
                    )
                {
                    out.push(Prop {
                        rect: [x, y, 1, 2],
                        kind: Kind::TimberFence,
                        ground: ground_tile_index,
                        height: 1.35,
                    });
                }
            }
        }
        if matches!(s.tileset_id.as_ref(), "johto" | "johto_modern")
            && s.tile_index == 0x4a
            && matches!(s.metatile_id, 0x40 | 0x42 | 0x44 | 0x46 | 0x48 | 0x4a)
        {
            out.push(Prop {
                rect: [x, y, 1, 1],
                kind: Kind::StonePost,
                ground: 0x06,
                height: 1.35,
            });
        }
    }
    for p in park_bench_placements(cells, g) {
        out.push(Prop {
            rect: [p.column, p.row, p.width, p.height],
            kind: Kind::ParkBench,
            ground: p.ground_tile_index,
            height: 1.5,
        });
    }
    out
}
pub(super) fn append_props(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
) {
    kanto_boundary_rocks::append(mesh, cells, shapes, g, claimed);
    kanto_capped_posts::append(mesh, cells, shapes, g, claimed);
    for p in props(cells, g) {
        if !clear(claimed, g, p.rect) {
            continue;
        }
        let s = &cells[p.rect[1] * g.width + p.rect[0]].source;
        let Some(sample) = ground(cells, shapes, s.tileset_id.as_ref(), p.ground, None) else {
            continue;
        };
        floor(mesh, cells, shapes, g, p.rect, sample, claimed);
        let mut b = bounds(g, p.rect);
        if p.kind == Kind::ParkBench {
            // The extra claimed row removes the original painted feet; it
            // does not stretch or move the existing full-volume bench.
            b[3] -= g.tile_height;
        }
        if matches!(p.kind, Kind::TimberFence | Kind::RouteSign) {
            let center = b[3] - g.tile_height * 0.35;
            b[2] = center - g.tile_height * 0.12;
            b[3] = center + g.tile_height * 0.12;
        }
        if p.kind == Kind::RouteSign {
            let center = (b[0] + b[1]) * 0.5;
            b[0] = center - g.tile_width * 0.825;
            b[1] = center + g.tile_width * 0.825;
        }
        let m = model(p.kind);
        m.append_fitted(
            &mut mesh.solid,
            b,
            0.0,
            p.height * g.tile_height / (m.max[1] - m.min[1]),
            None,
        );
        mark_authored_rect(mesh, g, p.rect, p.kind.label());
    }
}

pub(super) fn preferred_cells(map: &str, cells: &[&VisualTile], g: &GridGeometry) -> Vec<bool> {
    let shapes: Vec<_> = cells
        .iter()
        .map(|t| shape_for_source_on_map(map, &t.source))
        .collect();
    let mut reserved = vec![false; cells.len()];
    let mut reserve = |rect: [usize; 4]| {
        let [x, y, w, h] = rect;
        for yy in y..y + h {
            for xx in x..x + w {
                reserved[yy * g.width + xx] = true;
            }
        }
    };
    for p in building_placements(map, cells, g) {
        if structure_extensions::ready(cells, &shapes, g, p) {
            reserve([p.column, p.row, p.width, p.height]);
        } else if descriptor(map, cells, g, p).is_some()
            && ground(
                cells,
                &shapes,
                cells[p.row * g.width + p.column].source.tileset_id.as_ref(),
                p.ground_tile_index,
                None,
            )
            .is_some()
        {
            reserve([p.column, p.row, p.width, p.height]);
        }
    }
    for p in complete_tree_placements(cells, g) {
        if tree_ground(map, cells, &shapes, g, p).is_some() {
            reserve([p.column, p.row, p.width, p.height]);
        }
    }
    for p in props(cells, g) {
        if ground(
            cells,
            &shapes,
            cells[p.rect[1] * g.width + p.rect[0]]
                .source
                .tileset_id
                .as_ref(),
            p.ground,
            None,
        )
        .is_some()
        {
            reserve(p.rect);
        }
    }
    for p in kanto_boundary_rocks::resolve(cells, &shapes, g) {
        reserve(p.rect);
    }
    for p in kanto_capped_posts::resolve(cells, &shapes, g) {
        reserve(p.rect);
    }
    reserved
}

fn canonical_live(object: &crate::live_profiles::Object) -> bool {
    static CANONICAL: std::sync::OnceLock<crate::live_profiles::Document> =
        std::sync::OnceLock::new();
    CANONICAL
        .get_or_init(|| {
            serde_json::from_slice(include_bytes!(
                "../../../../modpacks/voxel-view/profiles.json"
            ))
            .expect("valid shipped profiles")
        })
        .objects
        .iter()
        .any(|candidate| candidate == object)
}

/// The shipped coastal profile duplicates Johto's native $0a stone block.
/// Leave it to the shared bank mesher before live placements reserve cells.
/// User-edited profiles retain their explicit renderer.
pub(super) fn native_stone_profile(object: &crate::live_profiles::Object) -> bool {
    object.tileset == "johto"
        && object.name == "Cherrygrove coastal boulder"
        && canonical_live(object)
}

/// Only source-complete live placements arrive here. Explicit named objects
/// cover coastal rocks whose raster silhouettes do not belong to tree logic.
pub(super) fn append_live(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &live::Placement<'_>,
    map_origin: [i32; 2],
) -> bool {
    if !canonical_live(p.object) {
        return false;
    }
    let small_tree = p.object.tileset == "johto"
        && (p.object.name.starts_with("Route29 small tree ")
            || p.object.name.starts_with("NewBarkTown small tree "));
    let kind = match (p.object.tileset.as_str(), p.object.name.as_str()) {
        _ if small_tree => Kind::CutTree,
        ("johto", "Cherrygrove coastal boulder") => Kind::ShoreBoulder,
        ("kanto", name) if name.starts_with("Cinnabar shoreline rock ") => Kind::ShoreBoulder,
        ("kanto", name) if name.starts_with("CinnabarIsland sign ") => Kind::RouteSign,
        _ => return false,
    };
    let w = p.object.tiles[0].len();
    let h = p.object.tiles.len();
    let rect = [p.column, p.row, w, h];
    if let Some(height) = p.object.footing_pixels {
        for y in p.row..p.row + h {
            for x in p.column..p.column + w {
                mesh.footing_heights[y * g.width + x] = height * g.tile_height / SOURCE_TILE_HEIGHT;
            }
        }
    }
    let ground_uv = g.uv(p.ground % g.width, p.ground / g.width);
    for y in p.row..p.row + h {
        for x in p.column..p.column + w {
            let (a, b, c, d) = g.bounds(x, y);
            append_top(&mut mesh.textured, [a, b, c, d], 0.0, ground_uv);
        }
    }
    let mut b = bounds(g, rect);
    if small_tree {
        crate::foliage::append_grove_tree(mesh, b, 0.0, g.tile_height * 0.5,
            [map_origin[0] + p.column as i32, map_origin[1] + p.row as i32], [false; 4]);
        mark_authored_rect(mesh, g, rect, "johto/small_tree");
        return true;
    }
    let m = model(kind);
    if kind == Kind::RouteSign {
        let center = b[3] - g.tile_height * 0.35;
        b[2] = center - g.tile_height * 0.12;
        b[3] = center + g.tile_height * 0.12;
    }
    m.append_fitted(
        &mut mesh.solid,
        b,
        0.0,
        h as f32 * g.tile_height / (m.max[1] - m.min[1]),
        None,
    );
    mark_authored_rect(mesh, g, rect, kind.label());
    let _ = cells;
    true
}

/// Meadow geometry follows the existing exact grass identities, including
/// Kanto and National Park variants. No grass is inferred from collision.
pub(super) fn append_grass(
    target: &mut SurfaceMeshData,
    _map: &str,
    tile: &VisualTile,
    g: &GridGeometry,
    base_height: f32,
    _origin: [i32; 2],
) -> bool {
    append_connected_grass(target, _map, tile, g, base_height, _origin, [false; 4])
}

pub(super) fn append_connected_grass(
    target: &mut SurfaceMeshData,
    _map: &str,
    tile: &VisualTile,
    g: &GridGeometry,
    base_height: f32,
    _origin: [i32; 2],
    joined: [bool; 4],
) -> bool {
    if !outdoor(tile.source.tileset_id.as_ref())
        || crate::grass::grass_shape(&tile.source).is_none()
    {
        return false;
    }
    let (w, e, n, s) = g.bounds(tile.column as usize, tile.row as usize);
    let tall = tile.source.tileset_id.as_ref() == "park" && tile.source.metatile_id == 0x13;
    let park = tile.source.tileset_id.as_ref() == "park";
    let kind = if park {
        if tall { crate::grass_blades::PatchKind::ParkLong }
        else { crate::grass_blades::PatchKind::ParkShort }
    } else { crate::grass_blades::PatchKind::Meadow };
    crate::grass_blades::append_styled_patch(
        target,
        [w, e, n, s],
        base_height,
        g.tile_height * if tall { 1.25 } else if park { 0.82 } else { 1.0 },
        [
            _origin[0] + tile.column as i32,
            _origin[1] + tile.row as i32,
        ],
        ((w + e) * 0.5 / g.tile_width).hypot((n + s) * 0.5 / g.tile_height) < 10.0,
        joined,
        kind,
    );
    true
}
pub(super) fn append_flower(
    target: &mut SurfaceMeshData,
    _map: &str,
    tile: &VisualTile,
    g: &GridGeometry,
    base_height: f32,
    origin: [i32; 2],
) -> bool {
    if !outdoor(tile.source.tileset_id.as_ref())
        || crate::flower::flower_shape(&tile.source).is_none()
    {
        return false;
    }
    let (w, e, n, s) = g.bounds(tile.column as usize, tile.row as usize);
    if matches!(_map, "NationalPark" | "NationalParkBugContest")
        && tile.source.tileset_id.as_ref() == "park"
    {
        park_flowers::append(target, [w, e, n, s], base_height,
            [origin[0] + tile.column as i32, origin[1] + tile.row as i32]);
        return true;
    }
    let m = johto_model(Johto::Flowers);
    m.append_fitted(
        target,
        [
            w + g.tile_width * 0.08,
            e - g.tile_width * 0.08,
            n + g.tile_height * 0.08,
            s - g.tile_height * 0.08,
        ],
        base_height,
        g.tile_height * 0.65 / (m.max[1] - m.min[1]),
        None,
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn park_flower_cluster_requires_native_flower_and_keeps_contest_appearance() {
        let mut source = super::super::tests::source_with_tile(0x3c, 1, 1, 3);
        source.tileset_id = "park".into();
        let frame = super::super::tests::frame(1, 1, vec![source]);
        let g = GridGeometry { width: 1, height: 1, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let mut ordinary = SurfaceMeshData::default();
        let mut contest = SurfaceMeshData::default();
        assert!(append_flower(&mut ordinary, "NationalPark", &frame.tiles[0], &g, -2., [20, 40]));
        assert!(append_flower(&mut contest, "NationalParkBugContest", &frame.tiles[0], &g, -2., [20, 40]));
        assert_eq!(ordinary, contest);
        let mut lawn = frame.tiles[0].clone();
        lawn.source.tile_index = 1;
        let mut rejected = SurfaceMeshData::default();
        assert!(!append_flower(&mut rejected, "NationalPark", &lawn, &g, -2., [20, 40]));
        assert_eq!(rejected, SurfaceMeshData::default());
    }

    #[test]
    fn park_outer_tree_uses_its_own_crown_only_for_a_complete_native_plot() {
        let sources = (0..20).map(|i| {
            let x = i % 5;
            let y = i / 5;
            let mut source = super::super::tests::source_with_tile(
                if x < 4 { 6 } else { crate::park::PARK_TREE_GROUND_METATILE },
                x.min(3) as u8, y as u8,
                if x < 4 { (0x0c + x + y * 0x10) as u16 } else { 1 });
            source.tileset_id = "park".into();
            source
        }).collect();
        let frame = super::super::tests::frame(5, 4, sources);
        let g = GridGeometry { width: 5, height: 4, tile_width: 8., tile_height: 8., origin_x: 0., origin_z: 0. };
        let cells: Vec<_> = frame.tiles.iter().collect();
        let placements = complete_tree_placements(&cells, &g);
        assert_eq!(placements.len(), 1);
        let shapes = vec![CellShape::Flat; 20];
        let mut mesh = TerrainMeshData { footing_heights: vec![0.; 20], authored_cells: vec![None; 20], ..Default::default() };
        let mut claimed = vec![false; 20];
        assert!(append_tree(&mut mesh, &cells, &shapes, &g, "NationalPark", placements[0], &mut claimed, [0, 0]));
        assert_eq!(mesh.authored_cells[0], Some("world-exterior/park_tree"));
        assert_eq!(mesh.footing_heights, vec![0.; 20]);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 16);
        assert!((0..4).all(|y| !claimed[y * 5 + 4]));
        for y in 0..4 { for x in 0..4 {
            let mut changed = frame.tiles.clone();
            changed[y * 5 + x].source.tile_index ^= 1;
            let cells: Vec<_> = changed.iter().collect();
            let mut refused = TerrainMeshData::default();
            let mut claimed = vec![false; 20];
            assert!(!append_tree(&mut refused, &cells, &shapes, &g, "NationalPark", placements[0], &mut claimed, [0, 0]));
            assert!(refused.solid.positions.is_empty() && refused.textured.positions.is_empty());
            assert!(claimed.iter().all(|&v| !v));
        }}
    }

    #[test]
    fn park_hoop_fences_replace_only_complete_native_drawings() {
        for drawing in [[0x25,0x26,0x35,0x36], [0x23,0x24,0x33,0x34]] {
            let sources=(0..6).map(|i| {
                let x=i%3; let y=i/3;
                let mut s=super::super::tests::source_with_tile(
                    if x<2 {0x11} else {crate::park::PARK_TREE_GROUND_METATILE},
                    x as u8,y as u8,if x<2 {drawing[y*2+x]} else {1});
                s.tileset_id="park".into(); s
            }).collect();
            let frame=super::super::tests::frame(3,2,sources);
            let g=GridGeometry {width:3,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
            let cells:Vec<_>=frame.tiles.iter().collect();
            let placements=complete_tree_placements(&cells,&g);
            assert_eq!(placements.len(),1);
            let shapes=vec![CellShape::Flat;6];
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;6],authored_cells:vec![None;6],..Default::default()};
            let mut claimed=vec![false;6];
            assert!(append_tree(&mut mesh,&cells,&shapes,&g,"NationalPark",placements[0],&mut claimed,[0,0]));
            assert_eq!(claimed,[true,true,false,true,true,false]);
            assert_eq!(mesh.textured.quad_count(),4);
            assert_eq!(mesh.footing_heights,[0.;6]);
            assert_eq!(mesh.authored_cells[0],Some("world-exterior/park_hoop_fence"));
            assert!(!append_tree(&mut mesh,&cells,&shapes,&g,"NationalPark",placements[0],&mut claimed,[0,0]));
            for index in [0,1,3,4] {
                let mut changed=frame.tiles.clone();
                // An otherwise valid tree quadrant cannot fill a fence hole.
                changed[index].source.metatile_id=6;
                changed[index].source.tile_index=0x0c + changed[index].source.subtile_column as u16
                    + changed[index].source.subtile_row as u16 * 0x10;
                assert!(complete_tree_placements(&changed.iter().collect::<Vec<_>>(),&g).is_empty());
                let mut changed=frame.tiles.clone();
                // A duplicate quadrant must not be accepted as a complete arch.
                changed[index].source.tile_index=if index==0 {drawing[1]} else {drawing[0]};
                assert!(complete_tree_placements(&changed.iter().collect::<Vec<_>>(),&g).is_empty());
                let mut changed=frame.tiles.clone();
                changed[index].source.subtile_column+=1;
                assert!(complete_tree_placements(&changed.iter().collect::<Vec<_>>(),&g).is_empty());
            }
        }
    }
    #[test]
    fn park_bench_replaces_all_three_source_rows_without_stretching_or_claiming_backing() {
        let drawing=[7,8,9,10, 0x17,0x18,0x19,0x1a,
            0x27,0x28,0x29,0x2a, 6,0,6,0];
        let sources=drawing.into_iter().enumerate().map(|(i,tile)| {
            let mut s=super::super::tests::source_with_tile(0x0e,(i%4) as u8,(i/4) as u8,tile);
            s.tileset_id="park".into(); s
        }).collect();
        let frame=super::super::tests::frame(4,4,sources);
        let g=GridGeometry {width:4,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let cells:Vec<_>=frame.tiles.iter().collect();
        let shapes:Vec<_>=cells.iter().map(|t|shape_for_source(&t.source)).collect();
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],authored_cells:vec![None;16],..Default::default()};
        let mut claimed=vec![false;16];
        append_props(&mut mesh,&cells,&shapes,&g,&mut claimed);
        assert_eq!(claimed,[vec![true;12],vec![false;4]].concat());
        assert!(mesh.solid.positions.iter().all(|p| (0. ..=32.).contains(&p[0])
            && (0. ..=16.).contains(&p[2])),"replacement coverage must not deepen the bench model");
        assert_eq!(mesh.textured.quad_count(),12);
        super::new_bark::polish_world_surfaces(&mut mesh,"NationalPark",&cells,&g,[0,0]);
        assert!(mesh.textured.positions.is_empty(),"all claimed rows receive clean plaza backing");
        assert_eq!(mesh.footing_heights,vec![0.;16]);
        for index in 8..12 {
            let mut changed=frame.tiles.clone();
            changed[index].source.tile_index=0;
            let cells:Vec<_>=changed.iter().collect();
            let mut mesh=TerrainMeshData::default();
            let mut claimed=vec![false;16];
            append_props(&mut mesh,&cells,&shapes,&g,&mut claimed);
            assert!(claimed.iter().all(|c| !c),"a changed feet row rejects the complete drawing");
        }
    }
    use std::sync::Arc;
    fn drawing_cells(
        tileset: &str,
        blocks: &[&[u16]],
    ) -> (Vec<VisualTile>, GridGeometry, BuildingPlacement) {
        let width = blocks[0].len() * 4;
        let height = blocks.len() * 4;
        let mut cells = Vec::new();
        for y in 0..height {
            for x in 0..width {
                cells.push(VisualTile {
                    animation_frames: None,
                    column: x as u32,
                    row: y as u32,
                    source: VisualTileSource {
                        tileset_id: Arc::from(tileset),
                        metatile_id: blocks[y / 4][x / 4],
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: 0,
                    },
                    texture: Handle::default(),
                    priority: false,
                });
            }
        }
        (
            cells,
            GridGeometry {
                width,
                height,
                tile_width: 8.0,
                tile_height: 8.0,
                origin_x: 0.0,
                origin_z: 0.0,
            },
            BuildingPlacement {
                column: 0,
                row: 0,
                width,
                height,
                roof_rows: 4,
                ground_tile_index: if tileset == "kanto" {
                    KANTO_GROUND_TILE_INDEX
                } else {
                    6
                },
            },
        )
    }
    #[test]
    fn cabin_roof_uses_live_source_swatches_and_rejects_missing_palette_coverage() {
        let frame=super::super::tests::frame(6,1,vec![super::super::tests::source_with_tile(0x2c,0,0,0x05);6]);
        let g=GridGeometry {width:6,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let cells:Vec<_>=frame.tiles.iter().collect();
        let p=BuildingPlacement {column:0,row:0,width:6,height:1,roof_rows:1,ground_tile_index:0x05};
        assert!(cabin_roof_palette(&cells,&g,p,&TerrainImageSamples::default()).is_none());
        let pixels:Vec<_>=(0..64).flat_map(|i|if i%2==0 {[57,57,57,255]} else {[165,123,24,255]}).collect();
        let mut images=TerrainImageSamples::default();
        for tile in &cells {
            images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(pixels.clone()));
        }
        let palette=cabin_roof_palette(&cells,&g,p,&images).unwrap();
        assert!(palette[0][0] < palette[1][0] && palette[1][0] < palette[2][0]);
        assert_eq!(palette[0][0],palette[0][1]);
        assert!(palette[2][0]>palette[2][1] && palette[2][1]>palette[2][2]);
        let cyan:Vec<_>=(0..64).flat_map(|i|[
            [57,57,57,255], [41,140,255,255], [123,255,255,255], [222,255,222,255],
        ][i%4]).collect();
        for tile in &cells {
            images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(cyan.clone()));
        }
        let cyan_palette=cabin_roof_palette(&cells,&g,p,&images).unwrap();
        assert!(cyan_palette[1][2]>cyan_palette[1][1]);
        assert_eq!(cyan_palette[2][1],cyan_palette[2][2]);
        assert!(cyan_palette[2][0]<cyan_palette[2][1]);
        let pixels:Vec<_>=(0..64).flat_map(|i|[i as u8,0,0,255]).collect();
        images.pixels.insert(cells[2].texture.id(),TileImageSample::Rgba(pixels));
        assert!(cabin_roof_palette(&cells,&g,p,&images).is_none(),"unknown paint must not acquire a guessed 3D palette");
    }

    #[test]
    fn missing_forest_asset_retains_exact_2d_tiles_without_substitute_geometry() {
        const CHILD: &str = "GEOTHITE_MISSING_FOREST_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "mesh::modeled_exteriors::tests::missing_forest_asset_retains_exact_2d_tiles_without_substitute_geometry", "--nocapture"])
                .env(CHILD, "1")
                .env("CRYSTAL_OPEN_MODEL_ROOT", std::env::temp_dir().join(format!("geothite-missing-trees-{}",std::process::id())))
                .output().unwrap();
            assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
            return;
        }
        let sources = [0x26,0x27,0x05,0x36,0x37,0x05].into_iter().enumerate().map(|(i,tile)| {
            let mut s = super::super::tests::source_with_tile(
                if i%3<2 {0x06} else {0x01}, (i%3) as u8,(i/3) as u8,tile);
            s.tileset_id="forest".into(); s
        }).collect();
        let frame=super::super::tests::frame(3,2,sources);
        let g=GridGeometry {width:3,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let cells:Vec<_>=frame.tiles.iter().collect();
        let shapes:Vec<_>=cells.iter().map(|t|shape_for_source(&t.source)).collect();
        let p=TreePlacement {column:0,row:0,width:2,height:2,ground_tile_index:0x05,
            ground_metatile_id:None,base_height:0.,rounded:true,outline_mask:true,
            remove_all_ground:false,card_thickness:0.};
        assert!(!crate::foliage::world_tree_available([0,0]));
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;6],authored_cells:vec![None;6],..Default::default()};
        let mut claimed=vec![false;6];
        assert!(append_tree(&mut mesh,&cells,&shapes,&g,"IlexForest",p,&mut claimed,[0,0]));
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.scenery.is_empty());
        assert_eq!(mesh.authored_cells,vec![None;6]);
        assert_eq!(mesh.footing_heights,vec![0.;6]);
        assert_eq!(claimed,vec![true,true,false,true,true,false]);
        let mut expected=SurfaceMeshData::default();
        for row in 0..2 {for column in 0..2 {
            let (w,e,n,s)=g.bounds(column,row);
            append_top(&mut expected,[w,e,n,s],0.,g.uv(column,row));
        }}
        assert_eq!(mesh.textured.positions,expected.positions);
        assert_eq!(mesh.textured.uvs,expected.uvs);
        assert_eq!(mesh.textured.indices,expected.indices);
    }

    #[test]
    fn path_only_tree_maps_require_complete_art_and_same_atlas_backing() {
        let sources = (0..4)
            .flat_map(|row| {
                (0..3).map(move |column| {
                    if column < 2 {
                        super::super::tests::source_with_tile(
                            0x05,
                            column,
                            row,
                            (if row == 0 {
                                0x1e
                            } else if row == 3 {
                                0x3e
                            } else {
                                0x2e
                            }) + u16::from(column),
                        )
                    } else {
                        super::super::tests::source_with_tile(0x01, 0, 0, 0x06)
                    }
                })
            })
            .collect();
        let mut f = super::super::tests::frame(3, 4, sources);
        let g = GridGeometry {
            width: 3,
            height: 4,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        for map in ["Route40", "RuinsOfAlphOutside"] {
            let cells = f.tiles.iter().collect::<Vec<_>>();
            let shapes = cells
                .iter()
                .map(|t| shape_for_source(&t.source))
                .collect::<Vec<_>>();
            let p = complete_tree_placements(&cells, &g)[0];
            let ground = tree_ground(map, &cells, &shapes, &g, p).unwrap();
            assert_eq!(cells[ground].source.tile_index, 0x06);
            assert_eq!(shapes[ground].surface_height(g.tile_height), 0.0);
            assert!(tree_ground("NewBarkTown", &cells, &shapes, &g, p).is_none());
        }
        for tile in &mut f.tiles {
            if tile.source.tile_index == 0x06 {
                tile.source.tileset_id = std::sync::Arc::from("kanto");
            }
        }
        let cells = f.tiles.iter().collect::<Vec<_>>();
        let shapes = cells
            .iter()
            .map(|t| shape_for_source(&t.source))
            .collect::<Vec<_>>();
        let p = complete_tree_placements(&cells, &g)[0];
        assert!(tree_ground("Route40", &cells, &shapes, &g, p).is_none());
    }
    #[test]
    fn east_gate_keeps_side_entry_and_rejects_a_broken_native_phase() {
        let (mut tiles, g, p) = drawing_cells("johto", &[&[0x08, 0x09], &[0x10, 0x11]]);
        let d = descriptor("Route31", &tiles.iter().collect::<Vec<_>>(), &g, p).unwrap();
        assert_eq!(d.asset, Asset::World(Kind::EastGate));
        assert!(d.east);
        assert_eq!(d.door, Some(6.0));
        tiles[19].source.subtile_row = 3;
        assert!(descriptor("Route31", &tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
    }
    #[test]
    fn unknown_facade_and_cropped_building_remain_unclaimed() {
        let (tiles, g, mut p) = drawing_cells("johto_modern", &[&[0x18, 0x19], &[0x16, 0x7f]]);
        assert!(descriptor("GoldenrodCity", &tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
        p.width = 4;
        assert!(descriptor("GoldenrodCity", &tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
    }
    #[test]
    fn modern_and_kanto_architecture_use_distinct_assets() {
        let (a, ga, pa) = drawing_cells("johto_modern", &[&[0x18, 0x19], &[0x1a, 0x1b]]);
        let (b, gb, pb) = drawing_cells("kanto", &[&[0x68, 0x69], &[0x37, 0x73]]);
        assert_eq!(
            descriptor("GoldenrodCity", &a.iter().collect::<Vec<_>>(), &ga, pa)
                .unwrap()
                .asset,
            Asset::World(Kind::ModernCenter)
        );
        assert_eq!(
            descriptor("SaffronCity", &b.iter().collect::<Vec<_>>(), &gb, pb)
                .unwrap()
                .asset,
            Asset::World(Kind::KantoCenter)
        );
    }
    #[test]
    fn kanto_service_models_follow_the_native_poke_and_mart_signs() {
        for (map, terminal, expected) in [
            ("CinnabarIsland", 0x72, Kind::KantoCenter),
            ("SilverCaveOutside", 0x72, Kind::KantoCenter),
            ("CeruleanCity", 0x73, Kind::KantoMart),
        ] {
            let (mut tiles, geometry, placement) =
                drawing_cells("kanto", &[&[0x20, 0x21], &[0x37, terminal]]);
            assert_eq!(
                descriptor(map, &tiles.iter().collect::<Vec<_>>(), &geometry, placement)
                    .unwrap()
                    .asset,
                Asset::World(expected)
            );
            tiles[0].source.subtile_column = 1;
            assert!(
                descriptor(map, &tiles.iter().collect::<Vec<_>>(), &geometry, placement).is_none()
            );
        }
    }
    #[test]
    fn verified_small_tree_profiles_use_leafy_geometry_and_preserve_custom_cards() {
        let doc: crate::live_profiles::Document = serde_json::from_slice(include_bytes!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .unwrap();
        let flat = super::super::tests::source_with_tile(0x02, 0, 0, 5);
        let mut sources = vec![flat; 9];
        for (i, x, z, tile) in [
            (0, 0, 0, 0x1e),
            (1, 1, 0, 0x1f),
            (3, 0, 1, 0x3e),
            (4, 1, 1, 0x3f),
        ] {
            sources[i] = super::super::tests::source_with_tile(0x3c, x, z, tile);
        }
        let frame = super::super::tests::frame(3, 3, sources);
        let cells: Vec<_> = frame.tiles.iter().collect();
        let g = GridGeometry {
            width: 3,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let placements = live::resolve(&cells, 3, 3, "Route29", Some(&doc));
        let p = placements
            .iter()
            .find(|p| p.object.name == "Route29 small tree 3c 0,0")
            .unwrap();
        let mut mesh = TerrainMeshData {
            authored_cells: vec![None; 9],
            footing_heights: vec![0.; 9],
            ..Default::default()
        };
        assert!(append_live(&mut mesh, &cells, &g, p, [20, 30]));
        assert!(mesh.textured.positions.iter().all(|p| p[1] == 0.));
        assert!(mesh.solid.positions.iter().any(|p| p[1] > 12.));
        assert_eq!(mesh.footing_heights, vec![0.; 9]);
        for i in 0..9 {
            assert_eq!(
                mesh.authored_cells[i],
                [0, 1, 3, 4].contains(&i).then_some("johto/small_tree")
            );
        }
        let mut changed = p.object.clone();
        changed.top_pixels += 1;
        let custom = live::Placement {
            object: &changed,
            column: p.column,
            row: p.row,
            ground: p.ground,
        };
        let mut fallback = TerrainMeshData::default();
        assert!(!append_live(&mut fallback, &cells, &g, &custom, [20, 30]));
        assert!(fallback.solid.positions.is_empty());
        let mut changed_cells = frame.tiles.clone();
        changed_cells[4].source.tile_index = 5;
        let changed_refs: Vec<_> = changed_cells.iter().collect();
        assert!(
            !live::resolve(&changed_refs, 3, 3, "Route29", Some(&doc))
                .iter()
                .any(|p| p.object.name == "Route29 small tree 3c 0,0")
        );
    }

    #[test]
    fn cherrygrove_rock_profile_reuses_native_stone_mesh_and_pigment() {
        let doc: crate::live_profiles::Document = serde_json::from_slice(include_bytes!(
            "../../../../modpacks/voxel-view/profiles.json")).unwrap();
        let drawing = [[0x2b,0x2c,0x2c,0x2d], [0x3b,0x3c,0x3c,0x3d],
            [0x3b,0x3c,0x3c,0x3d], [0x4b,0x4c,0x4c,0x4d]];
        let mut sources = vec![super::super::tests::source_with_tile(0x35,0,0,0x14);36];
        for (row, tiles) in drawing.into_iter().enumerate() {
            for (column, tile) in tiles.into_iter().enumerate() {
                sources[(row+1)*6+column+1] = super::super::tests::source_with_tile(
                    0x0a,column as u8,row as u8,tile);
            }
        }
        sources[0] = super::super::tests::source_with_tile(0x02,0,0,0x05);
        let mut f = super::super::tests::frame(6,6,sources);
        f.map_id = std::sync::Arc::from("CherrygroveCity");
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(live::resolve(&cells,6,6,"CherrygroveCity",Some(&doc)).iter()
            .any(|p| native_stone_profile(p.object)), "fixture must exercise the overridden profile");
        let samples = TerrainImageSamples::default();
        let native = build_instanced_terrain_mesh_with_samples(&f,&samples).unwrap();
        let profiled = build_instanced_terrain_mesh_with_profiles(&f,&samples,&doc).unwrap();
        assert!(!native.solid.positions.is_empty());
        assert_eq!(profiled.solid.positions,native.solid.positions);
        assert_eq!(profiled.solid.normals,native.solid.normals);
        assert_eq!(profiled.solid.colors,native.solid.colors);
        assert_eq!(profiled.solid.uvs,native.solid.uvs);
        assert_eq!(profiled.solid.indices,native.solid.indices);
        assert_eq!(profiled.footing_heights,native.footing_heights);
        assert_eq!(profiled.authored_cells,native.authored_cells);
        let stone = |mesh: &TerrainMeshData| -> Vec<_> {
            mesh.solid.uvs.iter().enumerate().filter(|(_,uv)| uv[0] < -9000.)
                .map(|(i,uv)| (mesh.solid.positions[i],mesh.solid.colors[i],*uv)).collect()
        };
        let city_stone = stone(&profiled);
        assert!(!city_stone.is_empty());
        f.map_id = std::sync::Arc::from("Route26");
        let route = build_instanced_terrain_mesh_with_profiles(&f,&samples,&doc).unwrap();
        assert_eq!(stone(&route),city_stone,"identical blocks must share their mineral finish");
        for cell in &mut f.tiles {cell.source.tileset_id=std::sync::Arc::from("johto_modern");}
        let modern = build_instanced_terrain_mesh_with_profiles(&f,&samples,&doc).unwrap();
        assert_eq!(stone(&modern),city_stone,"Johto Modern shares the same $0a drawing");
    }

    #[test]
    fn changing_a_named_live_profile_keeps_its_custom_renderer() {
        let doc: crate::live_profiles::Document = serde_json::from_slice(include_bytes!(
            "../../../../modpacks/voxel-view/profiles.json"
        ))
        .unwrap();
        let mut object = doc
            .objects
            .into_iter()
            .find(|o| o.name == "Cherrygrove coastal boulder")
            .unwrap();
        assert!(canonical_live(&object));
        assert!(native_stone_profile(&object));
        object.tiles[0][0] ^= 1;
        assert!(!canonical_live(&object));
        assert!(!native_stone_profile(&object));
    }
    #[test]
    fn forest_door_block_change_selects_a_closed_gateway_without_a_new_door() {
        for (last, expected) in [(0x24, Kind::ForestGate), (0x25, Kind::ForestGateClosed)] {
            let (a, g, p) = drawing_cells(
                "forest",
                &[
                    &[0x1d, 0x1e, 0x1f],
                    &[0x21, 0x22, 0x23],
                    &[last, 0x26, 0x27],
                ],
            );
            let desc = descriptor("IlexForest", &a.iter().collect::<Vec<_>>(), &g, p).unwrap();
            assert_eq!(desc.asset, Asset::World(expected));
            if last == 0x25 {
                assert!(desc.door.is_none());
            }
        }
    }
    #[test]
    fn ground_sample_must_share_the_native_tileset() {
        let (mut a, g, _) = drawing_cells("johto", &[&[0x01]]);
        a[0].source.tile_index = 6;
        let cells = a.iter().collect::<Vec<_>>();
        let shapes = vec![CellShape::Flat; g.width * g.height];
        assert_eq!(ground(&cells, &shapes, "johto", 6, None), Some(0));
        assert_eq!(ground(&cells, &shapes, "johto_modern", 6, None), None);
    }
    #[test]
    fn doorway_requires_all_four_tiles_not_a_roof_or_half_door() {
        let (mut a, g, p) = drawing_cells("kanto", &[&[0x02, 0x03]]);
        for (dx, dy, t) in [(0, 0, 0x0b), (1, 0, 0x0c), (0, 1, 0x1b), (1, 1, 0x1c)] {
            a[(2 + dy) * g.width + 2 + dx].source.tile_index = t;
        }
        assert_eq!(door_column(&a.iter().collect::<Vec<_>>(), &g, p), Some(3.0));
        a[3 * g.width + 3].source.tile_index = 0;
        assert_eq!(door_column(&a.iter().collect::<Vec<_>>(), &g, p), None);
    }
}

#[cfg(test)]
mod structure_underlay_tests {
    use super::*;
    use std::sync::Arc;
    fn tile(tileset: &str, block: u16, index: u16) -> VisualTile {
        VisualTile {
            animation_frames: None,
            column: 0,
            row: 0,
            source: VisualTileSource {
                tileset_id: Arc::from(tileset),
                metatile_id: block,
                subtile_column: 0,
                subtile_row: 0,
                tile_index: index,
            },
            texture: Handle::default(),
            priority: false,
        }
    }
    #[test]
    fn native_grass_and_kanto_paving_rescue_complete_buildings_without_path_samples() {
        let lawn = tile("johto", 0x02, 0x05);
        let paving = tile("kanto", 0x7b, 0x39);
        assert_eq!(
            ground(&[&lawn], &[CellShape::Flat], "johto", 6, None),
            Some(0)
        );
        assert_eq!(
            ground(
                &[&paving],
                &[CellShape::Flat],
                "kanto",
                KANTO_GROUND_TILE_INDEX,
                None
            ),
            Some(0)
        );
        assert_eq!(
            ground(&[&lawn], &[CellShape::Flat], "johto_modern", 6, None),
            None
        );
        let false_paving = tile("kanto", 0x20, 0x39);
        assert_eq!(
            ground(
                &[&false_paving],
                &[CellShape::Flat],
                "kanto",
                KANTO_GROUND_TILE_INDEX,
                None
            ),
            None
        );
        assert_eq!(
            ground(
                &[&paving],
                &[CellShape::Flat],
                "kanto",
                KANTO_GROUND_TILE_INDEX,
                Some(0x02)
            ),
            None
        );
    }
}
