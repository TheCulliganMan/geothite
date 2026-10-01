//! Source-complete world exterior kits. Authoritative tiles, phases, native
//! ground and door seams select art; collision and simulation are never read.
use super::*;
#[path = "kanto_boundary_rocks.rs"]
mod kanto_boundary_rocks;
#[path = "structure_extensions.rs"]
mod structure_extensions;
use crate::exterior_models::{Kind, model};
use crate::new_bark_models::{ModelKind as Johto, model as johto_model};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Asset {
    World(Kind),
    Johto(Johto),
}
impl Asset {
    fn label(self) -> &'static str {
        match self {
            Self::World(k) => k.label(),
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
) -> bool {
    if structure_extensions::append_building(mesh, cells, shapes, g, p, claimed) {
        return true;
    }
    let Some(d) = descriptor(map, cells, g, p) else {
        return false;
    };
    let rect = [p.column, p.row, p.width, p.height];
    if !clear(claimed, g, rect) {
        return false;
    }
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
    match d.asset {
        Asset::World(kind) => {
            model(kind).append_fitted(&mut mesh.solid, b, 0.0, g.tile_height * 2.0, door)
        }
        Asset::Johto(kind) => {
            johto_model(kind).append_fitted(&mut mesh.solid, b, 0.0, g.tile_height * 2.0, door)
        }
    }
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
        "forest" => Asset::World(Kind::ForestConifer),
        "park" => Asset::World(if p.width == 4 {
            Kind::KantoCanopy
        } else {
            Kind::ParkHedge
        }),
        "battle_tower_outside" => Asset::World(Kind::ForestConifer),
        _ => return None,
    })
}
fn tree_ground(
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

pub(super) fn append_tree(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    map: &str,
    p: TreePlacement,
    claimed: &mut [bool],
    _map_origin: [i32; 2],
) -> bool {
    let Some(kind) = tree_kind(cells, g, p) else {
        return false;
    };
    let rect = [p.column, p.row, p.width, p.height];
    if !clear(claimed, g, rect) {
        return false;
    }
    let Some(sample) = tree_ground(map, cells, shapes, g, p) else {
        return false;
    };
    floor(mesh, cells, shapes, g, rect, sample, claimed);
    let mut b = bounds(g, rect);
    let depth = (b[1] - b[0]).min(b[3] - b[2]);
    b[2] = b[3] - depth;
    let h = p.height as f32 * g.tile_height;
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
        Asset::Johto(k) => {
            let m = johto_model(k);
            m.append_fitted(
                &mut mesh.solid,
                b,
                p.base_height,
                h / (m.max[1] - m.min[1]),
                None,
            )
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
                    height: 1.6,
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
        if matches!(p.kind, Kind::TimberFence | Kind::RouteSign) {
            let center = b[3] - g.tile_height * 0.35;
            b[2] = center - g.tile_height * 0.12;
            b[3] = center + g.tile_height * 0.12;
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

/// Only source-complete live placements arrive here. Explicit named objects
/// cover coastal rocks whose raster silhouettes do not belong to tree logic.
pub(super) fn append_live(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    p: &live::Placement<'_>,
) -> bool {
    if !canonical_live(p.object) {
        return false;
    }
    let kind = match (p.object.tileset.as_str(), p.object.name.as_str()) {
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
    let m = model(kind);
    let mut b = bounds(g, rect);
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
    if !outdoor(tile.source.tileset_id.as_ref())
        || crate::grass::grass_shape(&tile.source).is_none()
    {
        return false;
    }
    let (w, e, n, s) = g.bounds(tile.column as usize, tile.row as usize);
    let m = johto_model(Johto::GrassLod);
    let tall = tile.source.tileset_id.as_ref() == "park" && tile.source.metatile_id == 0x13;
    m.append_fitted(
        target,
        [
            w + g.tile_width * 0.06,
            e - g.tile_width * 0.06,
            n + g.tile_height * 0.06,
            s - g.tile_height * 0.06,
        ],
        base_height,
        g.tile_height * (if tall { 1.1 } else { 0.6 }) / (m.max[1] - m.min[1]),
        None,
    );
    true
}
pub(super) fn append_flower(
    target: &mut SurfaceMeshData,
    _map: &str,
    tile: &VisualTile,
    g: &GridGeometry,
    base_height: f32,
) -> bool {
    if !outdoor(tile.source.tileset_id.as_ref())
        || crate::flower::flower_shape(&tile.source).is_none()
    {
        return false;
    }
    let (w, e, n, s) = g.bounds(tile.column as usize, tile.row as usize);
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
        object.tiles[0][0] ^= 1;
        assert!(!canonical_live(&object));
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
