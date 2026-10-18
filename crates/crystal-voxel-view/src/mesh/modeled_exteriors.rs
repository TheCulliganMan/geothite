//! Source-complete world exterior kits. Authoritative tiles, phases, native
//! ground and door seams select art; collision and simulation are never read.
use super::*;
use bevy::prelude::Vec2;
#[path = "city_facades.rs"]
mod city_facades;
#[path = "connected_north_gate.rs"]
mod connected_north_gate;
#[path = "tower_gate.rs"]
pub(super) mod tower_gate;
#[path = "radio_tower_paint.rs"]
mod radio_tower_paint;
#[path = "modern_gym_paint.rs"]
mod modern_gym_paint;
#[path = "modern_department_paint.rs"]
mod modern_department_paint;
#[path = "ecruteak_houses.rs"]
mod ecruteak_houses;
#[path = "kanto_brick_office.rs"]
mod kanto_brick_office;
#[path = "kanto_craft.rs"]
mod kanto_craft;
#[path = "kanto_gym.rs"]
mod kanto_gym;
#[path = "kanto_house.rs"]
mod kanto_house;
#[path = "kanto_flat_house.rs"]
mod kanto_flat_house;
#[path = "kanto_museum.rs"]
mod kanto_museum;
pub(super) fn portal_block(mesh: &mut SurfaceMeshData, b: [f32; 6], color: [f32; 3]) {
    city_facades::block(mesh, b, color);
}
#[path = "folded_roofs.rs"]
pub(super) mod folded_roofs;
#[path = "kanto_boundary_rocks.rs"]
mod kanto_boundary_rocks;
#[path = "kanto_capped_posts.rs"]
mod kanto_capped_posts;
#[path = "kanto_yard_fence.rs"]
mod kanto_yard_fence;
#[path = "kanto_rock_mound.rs"]
mod kanto_rock_mound;
#[path = "park_hoop_fence.rs"]
pub(super) mod park_hoop_fence;
#[path = "structure_extensions.rs"]
mod structure_extensions;
#[path = "park_flowers.rs"]
pub(super) mod park_flowers;
use crate::exterior_models::{Kind, model, variant_model};
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
    GroveTree,
    KantoBrickOffice,
    KantoGym,
    KantoHouse,
    KantoFlatHouse,
    KantoMuseum,
}
impl Asset {
    fn label(self) -> &'static str {
        match self {
            Self::World(k) => k.label(),
            Self::KantoBrickOffice => "outdoor/kanto_brick_office",
            Self::KantoGym => "outdoor/kanto_gym",
            Self::KantoHouse => "outdoor/kanto_house",
            Self::KantoFlatHouse => "outdoor/kanto_flat_house",
            Self::KantoMuseum => "outdoor/kanto_museum",
            Self::GroveTree => "johto/tree",
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
    north_aperture: Option<[f32;2]>,
}
fn outdoor(tileset: &str) -> bool {
    matches!(
        tileset,
        "johto" | "johto_modern" | "kanto" | "forest" | "park" | "battle_tower_outside"
    )
}

/// A same-number ground tile from a connected map's different tileset is not
/// a valid underlay. Do not guess a material when the live source lacks one.
// A structure borrows pigment from a level source floor before considering
// the same drawing on a remote bank. Tree/root backing keeps its own policy.
pub(super) fn structure_ground(cells: &[&VisualTile], shapes: &[CellShape],
    tileset: &str, tile: u16) -> Option<usize> {
    if !matches!(tileset,"johto"|"johto_modern") {
        return ground(cells,shapes,tileset,tile,None);
    }
    cells.iter().zip(shapes).position(|(c,s)|
        c.source.tileset_id.as_ref()==tileset && c.source.tile_index==tile
            && matches!(s,CellShape::Flat|CellShape::PlaneAt{height:0.}))
        .or_else(||ground(cells,shapes,tileset,tile,None))
}

fn ground(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    tileset: &str,
    tile: u16,
    metatile: Option<u16>,
) -> Option<usize> {
    if tileset=="kanto" {
        // A bank cap can share the lawn tile but is not a native floor paint
        // source. Prefer real flat ground so its underlay also gets finished.
        if let Some(index)=cells.iter().zip(shapes).position(|(c,s)|
            c.source.tileset_id.as_ref()==tileset && c.source.tile_index==tile
                && metatile.is_none_or(|m|c.source.metatile_id==m)
                && matches!(s,CellShape::Flat|CellShape::PlaneAt{height:0.}|CellShape::Water)) {
            return Some(index);
        }
    }
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
// A building beside a native plaza continues its paving under the model.
// Civic plots can also continue independently verified dry courtyards under their
// replacement model. Borrow only a complete immediately adjacent drawing;
// a distant lawn elsewhere on the map must not become a green foundation strip.
fn courtyard_floor_source(cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,
    x:usize,y:usize,include_dry:bool)->Option<usize> {
    let i=y*g.width+x;let s=&cells.get(i)?.source;
    if s.tileset_id.as_ref()!="kanto"
        || !(s.tile_index==0x39 || s.metatile_id==0x01 && s.tile_index==0x23)
        || !(s.metatile_id==0x7b || include_dry && matches!(s.metatile_id,0x01|0x31|0x77))
        || s.subtile_column>3 || s.subtile_row>3 {return None;}
    let (Some(bx),Some(by))=(x.checked_sub(s.subtile_column as usize),y.checked_sub(s.subtile_row as usize)) else {return None;};
    if bx+4>g.width || by+4>g.height {return None;}
    let valid=(0..16).all(|j| {
        let at=(by+j/4)*g.width+bx+j%4;let Some(t)=cells.get(at).map(|c|&c.source) else {return false;};
        let tile=if s.metatile_id==0x01 {if matches!(j,4|14) {0x39} else {0x23}}
            else if s.metatile_id==0x7b {if (j/4+j%4)%2==0 {0x30} else {0x39}}
            else if s.metatile_id==0x31 || j<8 {0x39} else if j<12 {0x0e} else {0x55};
        t.tileset_id.as_ref()=="kanto" && t.metatile_id==s.metatile_id && t.tile_index==tile
            && t.subtile_column as usize==j%4 && t.subtile_row as usize==j/4
    });
    (valid && matches!(shapes.get(i),Some(CellShape::Flat|CellShape::PlaneAt{height:0.}))).then_some(i)
}
fn courtyard_floor_sample(cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry,
    p: BuildingPlacement, include_dry: bool) -> Option<usize> {
    for y in p.row.saturating_sub(1)..(p.row+p.height+1).min(g.height) {
        for x in p.column.saturating_sub(1)..(p.column+p.width+1).min(g.width) {
            if (p.row..p.row+p.height).contains(&y) && (p.column..p.column+p.width).contains(&x) {continue;}
            if let Some(i)=courtyard_floor_source(cells,shapes,g,x,y,include_dry) {return Some(i);}
        }
    }
    None
}
// A wide native street can continue beneath an ordinary house. A narrow door
// path, corner contact or distant road cannot repaint a genuine grass yard.
fn house_frontage_floor_sample(cells:&[&VisualTile],shapes:&[CellShape],g:&GridGeometry,
    p:BuildingPlacement,images:Option<&TerrainImageSamples>)->Option<usize> {
    let images=images?;let y=p.row+p.height;
    if y>=g.height || p.column+p.width>g.width {return None;}
    let mut first=None;let mut checked=std::collections::BTreeSet::new();
    for x in p.column..p.column+p.width {
        let i=courtyard_floor_source(cells,shapes,g,x,y,true)?;
        let s=&cells[i].source;
        let bx=x-s.subtile_column as usize;let by=y-s.subtile_row as usize;
        if checked.insert((bx,by)) && !(0..16).all(|j| {
            tile_rgba(images,cells[(by+j/4)*g.width+bx+j%4]).is_ok_and(|rgba|
                rgba.len()==256 && rgba.chunks_exact(4).all(|p|p[3]==255))
        }) {return None;}
        first.get_or_insert(i);
    }
    first
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
                // A remote Kanto bank supplies lawn paint, never the height
                // of a building's replacement floor. The terrace field owns it.
                if cells[sample].source.tileset_id.as_ref()=="kanto"
                    && matches!(shapes[sample],CellShape::RaisedTop {solid:SolidKind::Bank,..}|CellShape::LedgeBand{..}) {0.}
                    else{shapes[sample].surface_height(g.tile_height)},
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
    let connected_gate=connected_north_gate::matches(cells,g,p);
    let tower_gate=tower_gate::matches(cells,g,p);
    if !connected_gate && !phases(cells, g, p) {
        return None;
    }
    let first = &cells[p.row * g.width + p.column].source;
    let catalogued = connected_gate || tower_gate || BUILDING_TEMPLATES
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
            if connected_gate {
                east=true;
                Asset::World(Kind::EastGate)
            } else if drawing(cells, g, p, "johto", &[&[0x08, 0x09], &[0x10, 0x11]], 0)
                || drawing(cells, g, p, "johto", &[&[0x18, 0x19], &[0x10, 0x11]], 0)
            {
                east = true;
                Asset::World(Kind::EastGate)
            } else if drawing(cells,g,p,"johto",&[&[0x08,0x09],&[0x1a,0x11]],0)
                && door.is_some() {
                Asset::Johto(RouteGate)
            } else if first.metatile_id == 0x08 && p.height == 28 {
                depth = Some(p.width as f32);
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
            } else if drawing(cells, g, p, "johto", &[&[0x08, 0x09], &[0x16, 0x1e]], 0) {
                Asset::Johto(PlayerHouse)
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
            if drawing(cells,g,p,"johto_modern",&[&[0x08,0x09],&[0x1c,0x1e]],0) {
                east=true;
                Asset::World(Kind::EastGate)
            } else if door.is_some()
                && drawing(cells,g,p,"johto_modern",&[&[0x18,0x19],&[0x1a,0x1b]],0) {
                Asset::Johto(Johto::Pokecenter)
            } else if door.is_some()
                && drawing(cells,g,p,"johto_modern",&[&[0x18,0x19],&[0x1a,0x33]],0) {
                Asset::Johto(Johto::Mart)
            } else if drawing(cells,g,p,"johto_modern",&[&[0x14,0x15]],0) && door.is_some() {
                Asset::Johto(Johto::House)
            } else {
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
        }
        "kanto" if kanto_museum::matches(cells,g,p) && door.is_none() => Asset::KantoMuseum,
        "kanto" if kanto_flat_house::matches(cells,g,p) && door.is_none() => Asset::KantoFlatHouse,
        "kanto" if kanto_house::matches(cells,g,p) && door.is_some() => Asset::KantoHouse,
        "kanto" if kanto_gym::matches(cells,g,p)
            && door==Some((p.width-3) as f32) => Asset::KantoGym,
        "kanto" if kanto_brick_office::matches(cells,g,p) => Asset::KantoBrickOffice,
        "kanto" if kanto_brick_office::double_entrance_candidate(cells,g,p) => return None,
        "kanto" => {
            let kind = match first.metatile_id {
                0x02 => return None, // This drawing has one authenticated shared producer.
                0x30 => Kind::KantoFlatHouse,
                0x38 => return None, // Source-bound cottages own both native window courses.
                0x75 => return None, // One complete source-bound museum producer.
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
                0x0c if p.height==12 && at(0,4)==0x75 => return None,
                0x0c if drawing(cells,g,p,"kanto",&[&[0x0c,0x0e],&[0x10,0x66]],0) => return None,
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
        "battle_tower_outside" if tower_gate => Asset::World(Kind::EastGate),
        "battle_tower_outside" if p.width == 20 && p.height == 16 && first.metatile_id == 0x08 => {
            Asset::World(Kind::BattleTower)
        }
        _ => return None,
    };
    if door.is_none() && matches!(asset, Asset::World(Kind::KantoHouse | Kind::KantoFlatHouse)) {
        asset = Asset::World(Kind::KantoBlank);
    }
    let north_aperture=(asset==Asset::World(Kind::EastGate)
        && (tower_gate || super::outdoor_entry_mat::north_approach(cells,g,p))).then_some([2.,6.]);
    if north_aperture.is_some() {east=false;}
    // Large frontless side gates have a transverse passage. The exact native
    // bottom four source rows locate that opening; never add a south door.
    Some(Building {
        asset,
        door: if north_aperture.is_some() {Some(4.)} else if east {
            Some(p.height as f32 - 2.0)
        } else {
            door
        },
        east,
        depth_cells: depth,
        north_aperture,
    })
}

pub(super) fn building_placements(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Vec<BuildingPlacement> {
    let mut out = new_bark::building_placements(map, cells, g);
    // A complete native rock drawing must join the surrounding canyon. The
    // old fitted mesa kit turned Blackthorn's ochre formation into a pale
    // isolated pedestal and prevented the shared rock finish from touching it.
    out.retain(|p| !structure_extensions::native_mesa(cells, g, *p));
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
        if drawing(cells, g, p, "johto", &[&[0x08, 0x09], &[0x10, 0x11]], 0)
            || connected_north_gate::matches(cells,g,p) {
            out.push(p);
        }
    }
    for tile in cells {
        let s=&tile.source;
        if s.tileset_id.as_ref()!="battle_tower_outside" || s.metatile_id!=0x2e
            || s.subtile_column!=2 || s.subtile_row!=0 {continue;}
        let p=BuildingPlacement{column:tile.column as usize,row:tile.row as usize,
            width:8,height:8,roof_rows:4,ground_tile_index:6};
        if tower_gate::matches(cells,g,p) {out.push(p);}
    }
    // A connected complete gate supersedes the old facade-only placement.
    // Keep one asset instance when both sides of the map seam are visible.
    let whole_gates=out.iter().copied().filter(|&p|
        drawing(cells,g,p,"johto",&[&[0x08,0x09],&[0x1a,0x11]],0)
        || drawing(cells,g,p,"johto",&[&[0x18,0x19],&[0x1a,0x11]],0))
        .collect::<Vec<_>>();
    out.retain(|p| !whole_gates.iter().any(|whole|
        p.column==whole.column && p.row==whole.row+4 && p.width==8 && p.height==4));
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

pub(super) fn cabin_roof_palette(cells: &[&VisualTile], g: &GridGeometry,
    p: BuildingPlacement, images: &TerrainImageSamples) -> Option<[[f32;4];3]> {
    let mut colors = std::collections::BTreeSet::new();
    // Complete source roof interior, excluding its transparent/ground corners.
    for row in p.row..p.row+p.roof_rows {
        let inset=if p.width>4 {2}else{1};
        for column in p.column+inset..p.column+p.width-inset {
            let tile = cells[row*g.width+column];
            // Kanto roof blocks also contain blank ground/window cells.
            // Only the pitched/flat roof-art courses authorize roof paint.
            if tile.source.tileset_id.as_ref() == "kanto"
                && !matches!(tile.source.tile_index, 0x05..=0x09 | 0x15..=0x19
                    | 0x4c..=0x4e | 0x5a..=0x5f) {
                continue;
            }
            for pixel in tile_rgba(images,tile).ok()?.chunks_exact(4) {
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

fn ecruteak_facade_paint(cells: &[&VisualTile], g: &GridGeometry,
    p: BuildingPlacement, images: &TerrainImageSamples) -> Option<ecruteak_houses::FacadePaint> {
    let mut colors=std::collections::BTreeSet::new();
    // Complete central facade only. Roof, outside ground and transparent
    // corner art cannot supply plaster or timber roles.
    for row in p.row+p.roof_rows..p.row+p.height {
        for column in p.column+2..p.column+p.width-2 {
            let tile=cells[row*g.width+column];
            if tile.source.tileset_id.as_ref()!="johto" {return None;}
            for pixel in tile_rgba(images,tile).ok()?.chunks_exact(4) {
                if pixel[3]==255 {colors.insert([pixel[0],pixel[1],pixel[2]]);}
            }
        }
    }
    if !(3..=4).contains(&colors.len()) {return None;}
    let mut colors=colors.into_iter().collect::<Vec<_>>();
    colors.sort_by_key(|c|u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722);
    let linear=|c:[u8;3]|c.map(|v| {let s=f32::from(v)/255.;if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)}});
    Some(ecruteak_houses::FacadePaint {
        wall:linear(colors[colors.len()-1]),timber:linear(colors[colors.len()-2]),
        shadow:linear(colors[colors.len()-3]),window:linear(colors[0]),
    })
}

fn forest_gate_wall_palette(cells: &[&VisualTile], g: &GridGeometry,
    p: BuildingPlacement, images: &TerrainImageSamples, open: bool) -> Option<[[f32;4];6]> {
    // Exact masonry and window drawings in the authenticated forest gate.
    // Sampling the complete facade indiscriminately mixes two source palettes.
    let sample = |index| -> Option<Vec<[u8;3]>> {
        let mut colors=std::collections::BTreeSet::new();
        for row in p.row..p.row+p.height {for column in p.column..p.column+p.width {
            let tile=cells[row*g.width+column];
            if tile.source.tileset_id.as_ref()!="forest" || tile.source.tile_index!=index {continue;}
            for pixel in tile_rgba(images,tile).ok()?.chunks_exact(4) {
                if pixel[3]!=255 {return None;}
                colors.insert([pixel[0],pixel[1],pixel[2]]);
            }
        }}
        if !(2..=4).contains(&colors.len()) {return None;}
        let mut colors:Vec<_>=colors.into_iter().collect();
        colors.sort_by_key(|c|u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722);
        Some(colors)
    };
    let wall=sample(0x24)?;let window=sample(0x04)?;
    let door=if open {sample(0x17)?} else {wall.clone()};
    let linear=|c:[u8;3]| {
        let rgb=c.map(|v| {let s=f32::from(v)/255.;if s<=0.04045 {s/12.92} else {((s+0.055)/1.055).powf(2.4)}});
        [rgb[0],rgb[1],rgb[2],1.]
    };
    Some([linear(wall[usize::from(wall.len()>2)]),linear(*wall.last()?),
        linear(*window.last()?),linear(window[0]),linear(window[window.len()-2]),
        linear(door[usize::from(door.len()>2)])])
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
    let connected_samples=if connected_north_gate::matches(cells,g,p) {
        let flat=|family:&str|cells.iter().zip(shapes).position(|(tile,shape)|
            tile.source.tileset_id.as_ref()==family && tile.source.tile_index==0x06
                && matches!(shape,CellShape::Flat|CellShape::PlaneAt{height:0.}));
        let facade=cells[(p.row+4)*g.width+p.column].source.tileset_id.as_ref();
        let (Some(north),Some(south))=(flat("johto"),flat(facade)) else {return false;};
        Some([north,south])
    } else {None};
    let office_paint=if d.asset==Asset::KantoBrickOffice {
        let Some(paint)=images.and_then(|images|kanto_brick_office::paint(cells,g,p,images)) else{return false;};
        Some(paint)
    }else{None};
    let gym_paint=if d.asset==Asset::KantoGym {
        let Some(paint)=images.and_then(|images|kanto_gym::paint(cells,g,p,images)) else{return false;};
        Some(paint)
    }else{None};
    let house_paint=if d.asset==Asset::KantoHouse {
        let Some(paint)=images.and_then(|images|kanto_house::paint(cells,g,p,images)) else{return false;};
        Some(paint)
    }else{None};
    let flat_house_paint=if d.asset==Asset::KantoFlatHouse {
        let Some(paint)=images.and_then(|images|kanto_flat_house::paint(cells,g,p,images)) else{return false;};
        Some(paint)
    }else{None};
    let museum_paint=if d.asset==Asset::KantoMuseum {
        let Some(paint)=images.and_then(|images|kanto_museum::paint(cells,g,p,images)) else{return false;};
        Some(paint)
    }else{None};
    let modern_source_roof=matches!(map,"GoldenrodCity"|"AzaleaTown") && matches!(d.asset,
        Asset::World(Kind::ModernHouse | Kind::ModernBlank | Kind::ModernShop
            | Kind::ModernCenter | Kind::ModernMart | Kind::ModernGym
            | Kind::ModernStation | Kind::ModernDaycare | Kind::ModernArcade));
    let world_roof = modern_source_roof || d.asset==Asset::World(Kind::EastGate)
        || matches!(d.asset, Asset::World(Kind::KantoHouse | Kind::KantoFlatHouse
        | Kind::KantoMart | Kind::KantoBlank | Kind::ForestGate | Kind::ForestGateClosed));
    let tall_pagoda=d.asset==Asset::World(Kind::TinTower) && p.height==16;
    let ecruteak_roof_palette = if tall_pagoda || (map=="EcruteakCity" && matches!(d.asset,
        Asset::Johto(Johto::TraditionalHouse) | Asset::World(Kind::TinTower))) {
        images.and_then(|images|cabin_roof_palette(cells,g,p,images))
    } else {None};
    let ecruteak_facade = if tall_pagoda || (map=="EcruteakCity" && matches!(d.asset,
        Asset::Johto(Johto::TraditionalHouse) | Asset::World(Kind::TinTower | Kind::BurnedTower))) {
        // The ruined drawing's fifth row still contains gray roof fragments
        // (b8..bf). Its native plaster begins one row below the generic roof.
        let facade_p=if d.asset==Asset::World(Kind::BurnedTower) {
            BuildingPlacement {roof_rows:5,..p}
        } else {p};
        images.and_then(|images|ecruteak_facade_paint(cells,g,facade_p,images))
    } else {None};
    let world_roof_palette = if world_roof {
        images.and_then(|images| cabin_roof_palette(cells,g,p,images))
    } else {None};
    let department_palette=if d.asset==Asset::World(Kind::ModernDepartment) {
        images.and_then(|images|modern_department_paint::palette(cells,g,p,images))
    }else{None};
    let modern_gym=d.asset==Asset::World(Kind::ModernGym);
    let modern_gym_palette=if modern_gym {
        images.and_then(|images|modern_gym_paint::palette(cells,g,p,images))
    } else {None};
    let radio_tower=d.asset==Asset::World(Kind::RadioTower);
    let radio_palette=if radio_tower {
        images.and_then(|images|radio_tower_paint::palette(cells,g,p,images))
    } else {None};
    let forest_gate=matches!(d.asset,Asset::World(Kind::ForestGate | Kind::ForestGateClosed));
    let forest_wall_palette=if forest_gate {
        images.and_then(|images|forest_gate_wall_palette(cells,g,p,images,
            matches!(d.asset,Asset::World(Kind::ForestGate))))
    } else {None};
    if images.is_some() && ((forest_gate
        && (world_roof_palette.is_none() || forest_wall_palette.is_none()))
        || radio_tower && radio_palette.is_none()
        || modern_gym && modern_gym_palette.is_none()) {
        for row in p.row..p.row+p.height {for column in p.column..p.column+p.width {
            let (w,e,n,s)=g.bounds(column,row);
            append_top(&mut mesh.textured,[w,e,n,s],0.,g.uv(column,row));
            claimed[row*g.width+column]=true;
        }}
        return true;
    }
    let source_roof = matches!(d.asset, Asset::Johto(Johto::TraditionalHouse
        | Johto::House | Johto::PlayerHouse | Johto::Pokecenter | Johto::Mart | Johto::VioletGym | Johto::RouteGate));
    let service = matches!(d.asset, Asset::Johto(Johto::Pokecenter | Johto::Mart | Johto::RouteGate));
    let compact = map!="IlexForest" && matches!(d.asset, Asset::Johto(Johto::House));
    let entrance_kind=matches!(d.asset,Asset::Johto(Johto::House|Johto::Pokecenter|Johto::Mart|Johto::RouteGate)) && d.door.is_some();
    let entrance_palette=if entrance_kind {
        images.and_then(|images| super::new_bark::entrance_source_colors(cells,g,p,d.door.unwrap() as usize,images))
    } else {None};
    let (roof_palette, wall_palette) = if source_roof {
        if let Some(images)=images {
            let paint=cabin_roof_palette(cells,g,p,images).and_then(|roof| {
                if service {
                    super::new_bark::service_wall_color(cells,g,p,images)
                        .map(|wall|(roof,Some(wall)))
                } else if compact {
                    super::new_bark::compact_wall_colors(cells,g,p,images)
                        .map(|wall|(roof,Some(wall)))
                } else {Some((roof,None))}
            }).filter(|_| !entrance_kind || entrance_palette.is_some());
            let Some((roof,wall))=paint else {
                // Missing material coverage is shown as exact 2D, never purple
                // prototype paint or a legacy voxel-building substitute.
                for row in p.row..p.row+p.height {for column in p.column..p.column+p.width {
                    let (w,e,n,s)=g.bounds(column,row);
                    append_top(&mut mesh.textured,[w,e,n,s],0.,g.uv(column,row));
                    claimed[row*g.width+column]=true;
                }}
                return true;
            };
            (Some(roof),wall)
        } else {(None,None)}
    } else {(None,None)};
    let tileset = cells[p.row * g.width + p.column].source.tileset_id.as_ref();
    let gate_ground=if d.asset==Asset::KantoBrickOffice && d.door.is_none() {
        kanto_brick_office::side_passage_ground(cells,g,p)
    } else {None};
    let sample=gate_ground.or_else(||if tileset=="kanto" {courtyard_floor_sample(cells,shapes,g,p,
        matches!(d.asset,Asset::KantoGym|Asset::KantoBrickOffice))
        .or_else(||if matches!(d.asset,Asset::KantoHouse|Asset::KantoFlatHouse) {
            house_frontage_floor_sample(cells,shapes,g,p,images)
        } else {None})} else {None})
        .or_else(||structure_ground(cells,shapes,tileset,p.ground_tile_index));
    let Some(sample) = sample else {
        return false;
    };
    let paved=matches!(d.asset,Asset::KantoBrickOffice|Asset::KantoHouse) && images.is_some_and(|images|
        super::kanto_paving::building_underlay(&mut mesh.solid,cells,shapes,g,rect,images,map_origin));
    if paved {
        for y in p.row..p.row+p.height {for x in p.column..p.column+p.width {claimed[y*g.width+x]=true;}}
    } else if let Some([north,south])=connected_samples {
        // A source-complete building may straddle two tilesets. Borrow each
        // half's own flat ground so its normal finish can authenticate it.
        floor(mesh,cells,shapes,g,[p.column,p.row,8,4],north,claimed);
        floor(mesh,cells,shapes,g,[p.column,p.row+4,8,4],south,claimed);
    } else {floor(mesh, cells, shapes, g, rect, sample, claimed);}
    if d.asset==Asset::World(Kind::EastGate) {
        for y in p.row+p.height-4..p.row+p.height {
            for x in p.column+p.width..(p.column+p.width+4).min(g.width) {
                let i=y*g.width+x;let source=&cells[i].source;
                if !claimed[i] && source.tileset_id.as_ref()=="johto_modern"
                    && source.metatile_id==0x0b && matches!(source.tile_index,0x40|0x00|0x06) {
                    floor(mesh,cells,shapes,g,[x,y,1,1],sample,claimed);
                    mesh.authored_cells[i]=Some("world-exterior/east_gate");
                }
            }
        }
    }
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
    if d.asset==Asset::World(Kind::Lighthouse) {
        // The native picture is tall, but the actual tower has a round plan.
        // Centre it on the authored door and fit one diameter in both axes.
        let center=door.unwrap_or((b[0]+b[1])*0.5);
        let m=model(Kind::Lighthouse);
        let aspect=(m.max[2]-m.min[2])/(m.max[0]-m.min[0]);
        let diameter=((center-b[0]).min(b[1]-center)*2.).min((b[3]-b[2])/aspect);
        b=[center-diameter*0.5,center+diameter*0.5,b[3]-diameter*aspect,b[3]];
    }
    let cutaway_start = mesh.solid.positions.len();
    match d.asset {
        Asset::KantoMuseum => kanto_museum::append(&mut mesh.solid,b,g.tile_width,
            museum_paint.expect("authenticated museum paint")),
        Asset::KantoHouse => kanto_house::append(&mut mesh.solid,b,g.tile_width,
            door.expect("authenticated cottage door"),house_paint.expect("authenticated cottage paint")),
        Asset::KantoFlatHouse => kanto_flat_house::append(&mut mesh.solid,b,g.tile_width,
            flat_house_paint.expect("authenticated doorless hall paint")),
        Asset::KantoGym => kanto_gym::append(&mut mesh.solid,b,g.tile_width,
            door.expect("authenticated gym door"),gym_paint.expect("authenticated gym paint")),
        Asset::KantoBrickOffice => {
            let paint=office_paint.expect("authenticated office paint");
            if gate_ground.is_some() {
                kanto_brick_office::append_with_passage(&mut mesh.solid,b,g.tile_width,door,paint,true);
            } else {kanto_brick_office::append(&mut mesh.solid,b,g.tile_width,door,paint);}
        },
        Asset::World(Kind::BurnedTower) if map == "EcruteakCity" => {
            ecruteak_houses::append_burned(&mut mesh.solid, b, g.tile_height, door, ecruteak_facade);
        }
        Asset::World(Kind::TinTower) if map == "EcruteakCity" && p.height == 6 => {
            ecruteak_houses::append_temple(&mut mesh.solid, b, g.tile_height, door, ecruteak_roof_palette, ecruteak_facade);
        }
        Asset::World(kind) => {
            let m = model(kind);
            let scale = if kind == Kind::KantoHouse { 1.25 } else { 2.0 };
            if let Some(aperture)=d.north_aperture {
                assert!(m.append_fitted_north_passage(&mut mesh.solid,b,0.,g.tile_height*scale,
                    aperture.map(|x|b[0]+x*g.tile_width)));
            } else {m.append_fitted(&mut mesh.solid, b, 0.0, g.tile_height * scale,
                if kind==Kind::Lighthouse {None}else{door});}
            if kind==Kind::TinTower && p.height==16 {
                m.recolor_fitted_pagoda(&mut mesh.solid,cutaway_start,ecruteak_roof_palette,
                    ecruteak_facade.map(|p|[p.wall,p.timber,p.shadow]));
            }
            if let Some(mut palette) = world_roof_palette {
                if kind==Kind::EastGate {
                    // The sprite's outline is not black paint on the exposed
                    // backing between the gate's ceramic courses.
                    palette[0]=std::array::from_fn(|i|if i==3 {1.}else{
                        (palette[1][i]*0.7+palette[2][i]*0.3)*0.55});
                }
                if modern_source_roof {
                    m.recolor_fitted_modern_roof(&mut mesh.solid,cutaway_start,palette);
                } else {m.recolor_fitted_roof(&mut mesh.solid, cutaway_start, palette);}
            }
            if let Some(palette)=department_palette {
                m.recolor_fitted_department(&mut mesh.solid,cutaway_start,palette);
            }
            if let Some(palette)=modern_gym_palette {
                m.recolor_fitted_modern_gym(&mut mesh.solid,cutaway_start,palette);
            }
            if let Some(palette)=radio_palette {
                m.recolor_fitted_radio_tower(&mut mesh.solid,cutaway_start,palette);
            }
            if let Some(palette)=forest_wall_palette {
                m.recolor_fitted_forest_wall(&mut mesh.solid,cutaway_start,palette);
            }
            if crate::city_lights::enabled(map) {
                crate::city_lights::mark(&mut mesh.solid,cutaway_start,m,kind);
            }
        }
        Asset::Johto(kind) => {
            if map == "EcruteakCity" && kind == Johto::TraditionalHouse {
                if drawing(cells, g, p, "johto", &[&[0x2c,0x2d], &[0x26,0x2f]], 2) {
                    ecruteak_houses::append_gate(&mut mesh.solid, b, g.tile_height, ecruteak_roof_palette, ecruteak_facade);
                } else {
                    ecruteak_houses::append(&mut mesh.solid, b, g.tile_height, door, ecruteak_roof_palette, ecruteak_facade);
                }
            } else {
                let m=johto_model(kind);
                m.append_fitted(&mut mesh.solid, b, 0.0, g.tile_height * 2.0, door);
                if let Some(palette)=roof_palette {m.recolor_fitted_roof(&mut mesh.solid,cutaway_start,palette);}
                if let Some(palette)=wall_palette {
                    if compact {m.recolor_fitted_compact_wall(&mut mesh.solid,cutaway_start,palette);}
                    else {m.recolor_fitted_service_wall(&mut mesh.solid,cutaway_start,palette);}
                }
                if let Some(palette)=entrance_palette {m.recolor_fitted_entrance(&mut mesh.solid,cutaway_start,palette);}
                if super::new_bark::ordinary_lawn_map(map) {
                    m.light_fitted_source_paint(&mut mesh.solid,cutaway_start);
                }
                if crate::city_lights::enabled(map) {m.mark_city_lights(&mut mesh.solid,cutaway_start);}
            }
        }
        Asset::GroveTree => unreachable!("trees cannot be building placements"),
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
    if map!="IlexForest" && matches!(tileset,"johto"|"johto_modern"|"kanto") {
        mesh.coverage_fade_ranges.push(cutaway_start..mesh.solid.positions.len());
        mesh.depth_coverage_solid_ranges.push(cutaway_start..mesh.solid.positions.len());
    }
    if matches!(d.asset, Asset::World(Kind::ForestGate | Kind::ForestGateClosed)) {
        mesh.coverage_fade_ranges.push(cutaway_start..mesh.solid.positions.len());
    }
    mesh.rigid_buildings.push(super::rigid_buildings::Building {
        vertices: cutaway_start..mesh.solid.positions.len(),
        doorway: if d.north_aperture.is_some() {
            Vec2::new(door.unwrap(), b[2]+g.tile_height*0.5)
        } else if d.east {
            Vec2::new(b[1] - g.tile_width * 0.5, door.unwrap_or((b[2]+b[3])*0.5))
        } else {
            Vec2::new(door.unwrap_or((b[0]+b[1])*0.5), b[3] - g.tile_height * 0.5)
        },
    });
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
        "johto" | "johto_modern" => Asset::GroveTree,
        "kanto" => Asset::GroveTree,
        "forest" => Asset::GroveTree,
        "park" => Asset::World(if p.width == 4 {
            Kind::KantoCanopy
        } else {
            Kind::ParkHedge
        }),
        "battle_tower_outside" => Asset::GroveTree,
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
            if claimed[i] {
                continue;
            }
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
    append_tree_with_groves(mesh, cells, shapes, g, map, p, claimed, map_origin, None)
}

pub(super) fn append_tree_with_groves(
    mesh: &mut TerrainMeshData,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    map: &str,
    p: TreePlacement,
    claimed: &mut [bool],
    map_origin: [i32; 2],
    groves: Option<&mut Vec<super::new_bark::GroveRecipe>>,
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
    match kind {
        Asset::World(k) => {
            // Park trees return above and retain their formal planting grid.
            // Give ordinary Kanto canopies the same stable planting variation
            // as Johto groves without changing their source ground claims.
            if k == Kind::KantoCanopy {
                let offset = crate::foliage::grove_trunk_offset(
                    [map_origin[0] + p.column as i32, map_origin[1] + p.row as i32],
                    depth,
                );
                b[0] += offset.x;
                b[1] += offset.x;
                b[2] += offset.y;
                b[3] += offset.y;
            }
            let m = variant_model(k, [map_origin[0] + p.column as i32, map_origin[1] + p.row as i32]);
            let height_factor = if k == Kind::KantoCanopy {
                crate::foliage::non_ancient_height_factor(
                    [map_origin[0]+p.column as i32,map_origin[1]+p.row as i32])
            } else { 1. };
            let foliage_start = mesh.solid.positions.len();
            m.append_fitted(
                &mut mesh.solid,
                b,
                p.base_height,
                h * height_factor / (m.max[1] - m.min[1]),
                None,
            );
            if k == Kind::KantoCanopy {
                for i in foliage_start..mesh.solid.positions.len() {
                    let color = &mut mesh.solid.colors[i];
                    if color[1] > color[0]*1.2 && color[1] > color[2]*1.1 {
                        let pigment = crate::foliage::leaf_pigment(
                            Vec3::from_array(mesh.solid.positions[i]),g.tile_height*1.5);
                        for channel in 0..3 { color[channel] *= pigment[channel]; }
                    }
                }
            }
        }
        Asset::GroveTree => {
            // Forest's four-cell source drawings reserve a larger blocked plot,
            // not a double-width Johto crown spilling across the walking paths.
            if cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "forest"
                && p.width > 2
            {
                let center = [(b[0] + b[1]) * 0.5, (b[2] + b[3]) * 0.5];
                b = [center[0] - g.tile_width, center[0] + g.tile_width,
                    center[1] - g.tile_height, center[1] + g.tile_height];
            }
            let small_forest = cells[p.row*g.width+p.column].source.tileset_id.as_ref() == "forest"
                && p.width == 2 && p.height == 2;
            let neighbors = if small_forest { [false;4] }
                else { super::new_bark::grove_neighbors(mesh, cells, g, map, p) };
            let recipe = super::new_bark::GroveRecipe {
                family: crate::foliage::GroveFamily::from_drawing(
                    cells[p.row*g.width+p.column].source.tileset_id.as_ref(), p.width, p.height),
                bounds: b,
                base_height: p.base_height,
                tile_height: g.tile_height * if small_forest { 0.5 } else { 1. },
                coordinates: [
                    map_origin[0] + p.column as i32,
                    map_origin[1] + p.row as i32,
                ],
                neighbors,
                understory_bounds: super::new_bark::grove_understory_bounds(map,cells,g,p),
                // Dense leaves retain coverage reveal without alpha stacks.
                coverage_fade: cells[p.row*g.width+p.column].source.tileset_id.as_ref()=="kanto"
                    || (map == "IlexForest"
                        && cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "forest"),
            };
            if let Some(groves) = groves { groves.push(recipe); }
            else { recipe.append(mesh); }
        }
        Asset::Johto(_) | Asset::KantoMuseum | Asset::KantoHouse | Asset::KantoFlatHouse | Asset::KantoBrickOffice | Asset::KantoGym => unreachable!("buildings cannot be tree placements"),
    }
    let label=if kind==Asset::GroveTree {
        crate::foliage::GroveFamily::from_drawing(cells[p.row*g.width+p.column].source.tileset_id.as_ref(), p.width, p.height).label()
    } else {kind.label()};
    mark_authored_rect(mesh, g, rect, label);
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
                    && (matches!(
                        crate::johto_fence::johto_fence_shape(next)
                            .or_else(|| crate::modern_route::modern_route_shape(next)),
                        Some(CellShape::FacadeBand {
                            band_from_top: 1,
                            solid: SolidKind::Fence,
                            ..
                        })
                    ) || (next.tile_index==0x4a && next.subtile_row==1
                        && matches!(next.metatile_id,0x40|0x42)))
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
        // $4a is the plan-view north/south stripe of this same fence.
        // append_johto_vertical_fences owns it, including its source elbows.
        // It must never be claimed as a row of disconnected stone posts.
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
    append_props_with_samples(mesh,cells,shapes,g,claimed,None);
}

pub(super) fn fence_palette(images: &TerrainImageSamples, tiles: &[&VisualTile]) -> Option<[[f32;4];3]> {
    let mut colors=std::collections::BTreeSet::new();
    for tile in tiles {
        for px in tile_rgba(images,tile).ok()?.chunks_exact(4) {
            if px[3]==255 {colors.insert([px[0],px[1],px[2]]);}
        }
    }
    if !(2..=4).contains(&colors.len()) {return None;}
    let mut colors:Vec<_>=colors.into_iter().collect();
    colors.sort_by_key(|c|u32::from(c[0])*2126+u32::from(c[1])*7152+u32::from(c[2])*722);
    let linear=|c:[u8;3]| {
        let rgb=c.map(|v| {let s=f32::from(v)/255.;if s<=0.04045 {s/12.92} else {((s+0.055)/1.055).powf(2.4)}});
        [rgb[0],rgb[1],rgb[2],1.]
    };
    Some([linear(colors[0]),linear(colors[colors.len()-2]),linear(colors[colors.len()-1])])
}

pub(super) fn append_vertical_fence(mesh: &mut TerrainMeshData, g: &GridGeometry,
    index: usize, z: [f32;2], x: f32, palette: [[f32;4];3]) {
    let m=model(Kind::TimberFence);
    let first=mesh.solid.positions.len();
    let depth=g.tile_width*0.12;
    m.append_fitted(&mut mesh.solid,[z[0],z[1],-depth,depth],0.,
        1.35*g.tile_height/(m.max[1]-m.min[1]),None);
    m.recolor_fitted_fence(&mut mesh.solid,first,palette);
    // Rotate the same authored rail bay by 90 degrees; preserve winding.
    for p in &mut mesh.solid.positions[first..] { *p=[x-p[2],p[1],p[0]]; }
    for n in &mut mesh.solid.normals[first..] { *n=[-n[2],n[1],n[0]]; }
    mark_authored_rect(mesh,g,[index%g.width,index/g.width,1,1],Kind::TimberFence.label());
}

pub(super) fn append_props_with_samples(
    mesh: &mut TerrainMeshData, cells: &[&VisualTile], shapes: &[CellShape],
    g: &GridGeometry, claimed: &mut [bool], images: Option<&TerrainImageSamples>,
) {
    append_props_at_origin(mesh, cells, shapes, g, claimed, images, [0; 2]);
}

pub(super) fn append_props_at_origin(
    mesh: &mut TerrainMeshData, cells: &[&VisualTile], shapes: &[CellShape],
    g: &GridGeometry, claimed: &mut [bool], images: Option<&TerrainImageSamples>,
    map_origin: [i32; 2],
) {
    append_props_with_datum(mesh,cells,shapes,g,claimed,images,map_origin,None);
}
pub(super) fn append_props_with_datum(mesh:&mut TerrainMeshData,cells:&[&VisualTile],shapes:&[CellShape],
    g:&GridGeometry,claimed:&mut [bool],images:Option<&TerrainImageSamples>,map_origin:[i32;2],
    datum:Option<&crystal_render_api::VisualTerraceDatum>) {
    kanto_rock_mound::append(mesh, cells, shapes, g, claimed, map_origin, images);
    kanto_boundary_rocks::append_with_datum(mesh, cells, shapes, g, claimed, map_origin, images,datum);
    kanto_capped_posts::append_with_samples(mesh, cells, shapes, g, claimed, images);
    kanto_yard_fence::append(mesh,cells,shapes,g,claimed,images);
    for p in props(cells, g) {
        if !clear(claimed, g, p.rect) {
            continue;
        }
        let palette=if p.kind==Kind::TimberFence {
            images.and_then(|images| fence_palette(images,
                &[cells[p.rect[1]*g.width+p.rect[0]],cells[(p.rect[1]+1)*g.width+p.rect[0]]]))
        } else if p.kind==Kind::RouteSign {
            let tiles=[0,1,g.width,g.width+1].map(|offset| cells[p.rect[1]*g.width+p.rect[0]+offset]);
            images.and_then(|images| fence_palette(images,&tiles))
        } else {None};
        if matches!(p.kind,Kind::TimberFence|Kind::RouteSign) && images.is_some() && palette.is_none() {
            continue; // Keep source drawing when live color coverage is unavailable.
        }
        let s = &cells[p.rect[1] * g.width + p.rect[0]].source;
        let Some(sample) = structure_ground(cells, shapes, s.tileset_id.as_ref(), p.ground) else {
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
        let m = variant_model(p.kind, [map_origin[0] + p.rect[0] as i32, map_origin[1] + p.rect[1] as i32]);
        let first=mesh.solid.positions.len();
        m.append_fitted(
            &mut mesh.solid,
            b,
            0.0,
            p.height * g.tile_height / (m.max[1] - m.min[1]),
            None,
        );
        if let Some(palette)=palette {
            if p.kind==Kind::RouteSign { m.recolor_fitted_sign(&mut mesh.solid,first,palette); }
            else { m.recolor_fitted_fence(&mut mesh.solid,first,palette); }
        }
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
    let m = variant_model(kind, [map_origin[0] + p.column as i32, map_origin[1] + p.row as i32]);
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
        kind.height(g.tile_height),
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
fn kanto_flower_palette(tile:&VisualTile,images:&TerrainImageSamples)->Option<[[f32;4];2]> {
    if tile.source.tileset_id.as_ref()!="kanto" || tile.source.tile_index!=3
        || tile.source.subtile_column>=4 || tile.source.subtile_row>=4 {return None;}
    let rgba=tile_rgba(images,tile).ok()?;
    if rgba.len()!=256 || rgba.chunks_exact(4).any(|p|p[3]!=255) {return None;}
    let colors=rgba.chunks_exact(4).map(|p|[p[0],p[1],p[2]])
        .collect::<std::collections::BTreeSet<_>>();
    // Native frames use two blossom colors over one or two ground colors.
    // Select by pigment, independently of changing animation coverage.
    if !(3..=4).contains(&colors.len()) {return None;}
    let mut colors=colors.into_iter().collect::<Vec<_>>();
    colors.sort_by_key(|c|std::cmp::Reverse(i16::from(c[0])-i16::from(c[1])));
    if colors[..2].iter().any(|c|c[0]<=c[1]) {return None;}
    let mut paint=colors[..2].iter().map(|c| {
        let mut color=[0.,0.,0.,1.];for i in 0..3 {
            let v=f32::from(c[i])/255.;color[i]=if v<=0.04045 {v/12.92}else{((v+0.055)/1.055).powf(2.4)};
        }color
    }).collect::<Vec<_>>();
    paint.sort_by(|a,b|{
        let luma=|c:&[f32;4]|c[0]*0.2126+c[1]*0.7152+c[2]*0.0722;
        luma(a).total_cmp(&luma(b))
    });
    Some([paint[0],paint[1]])
}
pub(super) fn append_flower(
    target: &mut SurfaceMeshData,
    _map: &str,
    tile: &VisualTile,
    g: &GridGeometry,
    base_height: f32,
    origin: [i32; 2],
    images: Option<&TerrainImageSamples>,
) -> bool {
    if !outdoor(tile.source.tileset_id.as_ref())
        || crate::flower::flower_shape(&tile.source).is_none()
    {
        return false;
    }
    let start = target.positions.len();
    let (w, e, n, s) = g.bounds(tile.column as usize, tile.row as usize);
    if matches!(_map, "NationalPark" | "NationalParkBugContest")
        && tile.source.tileset_id.as_ref() == "park"
    {
        park_flowers::append(target, [w, e, n, s], base_height,
            [origin[0] + tile.column as i32, origin[1] + tile.row as i32]);
        crate::flower::mark_sway(target, start, base_height);
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
    crate::flower::mark_sway(target, start, base_height);
    if let Some(palette)=images.and_then(|images|kanto_flower_palette(tile,images)) {
        m.recolor_fitted_blossoms(target,start,palette);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_bound_flat_hall_uses_one_producer_across_towns_and_widths() {
        for width in [8,12,16] {
            let (tiles,g,p,_)=kanto_flat_house::fixture(width);
            let refs=tiles.iter().collect::<Vec<_>>();
            for map in ["CeruleanCity","VermilionCity","OtherTown"] {
                let d=descriptor(map,&refs,&g,p).unwrap();
                assert_eq!(d.asset,Asset::KantoFlatHouse);assert_eq!(d.door,None);
                assert_eq!(d.asset.label(),"outdoor/kanto_flat_house");
            }
        }
    }
    #[test]
    fn fallback_tree_cards_never_overwrite_owned_native_ground() {
        let g = GridGeometry { width:2,height:2,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0. };
        let p = TreePlacement {column:0,row:0,width:2,height:2,ground_tile_index:5,
            ground_metatile_id:None,base_height:0.,rounded:true,outline_mask:true,
            remove_all_ground:false,card_thickness:0.};
        let mut mesh = TerrainMeshData {authored_cells:vec![Some("johto/tree");4],
            footing_heights:vec![0.;4],..Default::default()};
        let mut owned = vec![true;4];
        append_missing_tree_source(&mut mesh,&g,p,&mut owned);
        assert!(mesh.textured.positions.is_empty());
        owned = vec![true,true,false,false];
        append_missing_tree_source(&mut mesh,&g,p,&mut owned);
        assert_eq!(mesh.textured.indices.len(),12);
        assert!(mesh.textured.positions.iter().all(|p|p[2]>=8.));
        assert_eq!(mesh.footing_heights,vec![0.;4]);
        assert_eq!(mesh.authored_cells,vec![Some("johto/tree");4]);
    }
    #[test]
    fn native_kanto_blossoms_keep_shared_geometry_roots_and_non_kanto_fallback() {
        let g=GridGeometry {width:1,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut source=super::super::tests::source_with_tile(4,1,1,3);source.tileset_id="kanto".into();
        let frame=super::super::tests::frame(1,1,vec![source]);let tile=&frame.tiles[0];
        let original=frame.clone();let mut images=TerrainImageSamples::default();
        let day=[[222,255,222,255],[247,82,49,255],[255,156,198,255]];
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba((0..64).flat_map(|i|day[i%3]).collect()));
        let palette=kanto_flower_palette(tile,&images).unwrap();
        let linear=|v:u8|{let v=f32::from(v)/255.;if v<=0.04045 {v/12.92}else{((v+0.055)/1.055).powf(2.4)}};
        assert_eq!(palette[0],[linear(247),linear(82),linear(49),1.]);
        assert_eq!(palette[1],[1.,linear(156),linear(198),1.]);
        let build=|map:&str,tile:&VisualTile,images:Option<&TerrainImageSamples>| {
            let mut out=SurfaceMeshData::default();
            assert!(append_flower(&mut out,map,tile,&g,1.25,[8,12],images));out
        };
        let baseline=build("PewterCity",tile,None);let painted=build("PewterCity",tile,Some(&images));
        assert_eq!(painted,build("UnseenTown",tile,Some(&images)),"one shared source producer, independent of town name");
        assert_eq!(baseline.positions,painted.positions);assert_eq!(baseline.normals,painted.normals);
        assert_eq!(baseline.indices,painted.indices);let mut petals=0;let mut leaves=0;
        for i in 0..painted.positions.len() {
            assert_eq!(baseline.uvs[i][1],painted.uvs[i][1]);
            assert!((painted.positions[i][1]-1.25-painted.uvs[i][1]).abs()<0.00001);
            if painted.uvs[i][0]==crate::flower::SOURCE_BLOOM_TAG {
                petals+=1;assert_ne!(baseline.colors[i],painted.colors[i]);
            } else {leaves+=1;assert_eq!(baseline.colors[i],painted.colors[i]);assert_eq!(baseline.uvs[i],painted.uvs[i]);}
        }
        assert!(petals>0 && leaves>0);assert_eq!(frame,original);
        let night=[[41,82,57,255],[90,123,99,255],[99,49,107,255],[156,90,132,255]];
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba((0..64).flat_map(|i|night[i%4]).collect()));
        let dark=build("PewterCity",tile,Some(&images));
        assert_eq!(dark.positions,painted.positions);assert_eq!(dark.normals,painted.normals);
        assert_eq!(dark.indices,painted.indices);assert_eq!(dark.uvs,painted.uvs);assert_ne!(dark.colors,painted.colors);
        for family in ["johto","johto_modern","forest","park"] {
            let mut other=tile.clone();other.source.tileset_id=family.into();
            assert_eq!(build("Route35",&other,None),build("Route35",&other,Some(&images)));
        }
        assert_eq!(build("PewterCity",tile,Some(&TerrainImageSamples::default())),baseline);
        images.pixels.insert(tile.texture.id(),TileImageSample::Rgba((0..64).flat_map(|i|[i as u8,0,0,255]).collect()));
        assert_eq!(build("PewterCity",tile,Some(&images)),baseline,"unknown palette retains the authored model");
    }
    #[test]
    fn flowers_stay_modeled_without_source_animation_pixels_and_keep_roots_planted() {
        let g = GridGeometry { width: 1, height: 1, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        for tileset in ["johto", "johto_modern", "kanto", "forest", "park"] {
            let mut source = super::super::tests::source_with_tile(4, 1, 1, 3);
            source.tileset_id = tileset.into();
            let frame = super::super::tests::frame(1, 1, vec![source]);
            let mut flowers = SurfaceMeshData::default();
            assert!(append_flower(&mut flowers, "Route35", &frame.tiles[0], &g, 0., [0, 0], None));
            assert!(!flowers.positions.is_empty());
            assert!(flowers.uvs.iter().all(|uv| uv[0] == 86000.));
            assert!(flowers.uvs.iter().any(|uv| uv[1] < 0.001));
            assert!(flowers.uvs.iter().any(|uv| uv[1] > 1.));
            for (p, uv) in flowers.positions.iter().zip(&flowers.uvs) {
                assert!((p[1] - uv[1]).abs() < 0.001);
            }
        }
    }

    #[test]
    fn park_flower_cluster_requires_native_flower_and_keeps_contest_appearance() {
        let mut source = super::super::tests::source_with_tile(0x3c, 1, 1, 3);
        source.tileset_id = "park".into();
        let frame = super::super::tests::frame(1, 1, vec![source]);
        let g = GridGeometry { width: 1, height: 1, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let mut ordinary = SurfaceMeshData::default();
        let mut contest = SurfaceMeshData::default();
        assert!(append_flower(&mut ordinary, "NationalPark", &frame.tiles[0], &g, -2., [20, 40], None));
        assert!(append_flower(&mut contest, "NationalParkBugContest", &frame.tiles[0], &g, -2., [20, 40], None));
        assert_eq!(ordinary, contest);
        let mut lawn = frame.tiles[0].clone();
        lawn.source.tile_index = 1;
        let mut rejected = SurfaceMeshData::default();
        assert!(!append_flower(&mut rejected, "NationalPark", &lawn, &g, -2., [20, 40], None));
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
    fn port_entry_rails_share_the_fence_asset_and_preserve_both_warp_channels() {
        let art = [
            [0x06;8], [0x06;8],
            [0x5a,0x5a,0x40,0x40,0x40,0x40,0x5a,0x5a],
            [0x59,0x59,0x40,0x40,0x40,0x40,0x59,0x59],
        ];
        let sources=(0..32).map(|i| {
            let (x,y)=(i%8,i/8);
            super::super::tests::source_with_tile(if x<4 {0x0c}else{0x0d},
                (x%4) as u8,y as u8,art[y][x])
        }).collect();
        let frame=super::super::tests::frame(8,4,sources);
        let cells:Vec<_>=frame.tiles.iter().collect();
        let g=GridGeometry{width:8,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut images=TerrainImageSamples::default();
        let pixels=(0..64).flat_map(|i|match i%3 {
            0=>[40,40,40,255],1=>[130,130,130,255],_=>[230,230,230,255]}).collect::<Vec<_>>();
        for tile in &cells {images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(pixels.clone()));}
        let mut mesh=TerrainMeshData{footing_heights:vec![0.37;32],authored_cells:vec![None;32],..Default::default()};
        let mut claimed=vec![false;32];
        append_props_with_samples(&mut mesh,&cells,&shapes,&g,&mut claimed,Some(&images));
        assert_eq!(props(&cells,&g).len(),4);
        assert!(!mesh.solid.indices.is_empty());
        assert_eq!(mesh.footing_heights,vec![0.37;32]);
        assert_eq!(mesh.textured.quad_count(),8);
        for y in 0..4 {for x in 0..8 {
            let rail=y>=2 && (x<2 || x>=6);
            assert_eq!(claimed[y*8+x],rail);
            assert_eq!(mesh.authored_cells[y*8+x],rail.then_some("world-exterior/timber_fence"));
        }}
        assert!(mesh.solid.positions.iter().all(|p|p[0]<=16.001 || p[0]>=47.999),
            "both two-cell native warp channels must remain open");
        assert!(mesh.solid.colors.iter().all(|c|(c[0]-c[1]).abs()<0.01 && (c[1]-c[2]).abs()<0.01));
        let mut missing=TerrainMeshData{authored_cells:vec![None;32],..Default::default()};
        let mut unclaimed=vec![false;32];
        append_props_with_samples(&mut missing,&cells,&shapes,&g,&mut unclaimed,Some(&TerrainImageSamples::default()));
        assert!(missing.solid.positions.is_empty() && unclaimed.iter().all(|v|!*v));
        let mut changed=frame.tiles.clone();
        changed[24].source.tile_index=0x06;
        assert_eq!(props(&changed.iter().collect::<Vec<_>>(),&g).len(),3,
            "an incomplete rail pair is never modeled");
    }

    #[test]
    fn johto_fence_stripe_retains_rails_and_live_source_color_in_both_directions() {
        let sources=[(0,0x5a),(1,0x59),(2,0x4a),(3,0x06)].into_iter()
            .map(|(row,tile)| super::super::tests::source_with_tile(0x44,0,row,tile)).collect();
        // $44 owns a straight vertical run; the horizontal donor samples are
        // selected from their own source-complete $40 rail drawing.
        let mut frame=super::super::tests::frame(1,4,sources);
        frame.tiles[0].source.metatile_id=0x40;
        frame.tiles[1].source.metatile_id=0x40;
        let cells:Vec<_>=frame.tiles.iter().collect();
        let g=GridGeometry {width:1,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        assert!(props(&cells,&g).iter().all(|p|p.rect[1]!=2),
            "the vertical rail stripe must not become a disconnected post");
        let mut images=TerrainImageSamples::default();
        let pixels:Vec<_>=(0..64).flat_map(|i| match i%3 {
            0=>[40,40,40,255],1=>[130,130,130,255],_=>[230,230,230,255]}).collect();
        for tile in &cells {images.pixels.insert(tile.texture.id(),TileImageSample::Rgba(pixels.clone()));}
        let mut mesh=TerrainMeshData {authored_cells:vec![None;4],footing_heights:vec![0.;4],..Default::default()};
        let mut claimed=vec![false;4];
        let shapes=vec![CellShape::Flat;4];
        append_props_with_samples(&mut mesh,&cells,&shapes,&g,&mut claimed,Some(&images));
        let horizontal=mesh.solid.positions.len();
        super::super::append_johto_vertical_fences(&mut mesh,&images,&cells,&g,&mut claimed).unwrap();
        assert_eq!(claimed,[true,true,true,false]);
        assert_eq!(mesh.authored_cells[2],Some("world-exterior/timber_fence"));
        assert!(mesh.solid.positions.len()>horizontal);
        assert_eq!(mesh.footing_heights,[0.;4]);
        for c in &mesh.solid.colors {assert!((c[0]-c[1]).abs()<0.01 && (c[1]-c[2]).abs()<0.01,
            "neutral source rails cannot become brown timber: {c:?}");}
        for p in &mesh.solid.positions[horizontal..] {
            assert!(p[0]>=3.039 && p[0]<=4.961 && p[2]>=16. && p[2]<=24.,"{p:?}");
        }
        for n in &mesh.solid.normals[horizontal..] {assert!((Vec3::from_array(*n).length()-1.).abs()<0.001);}
        assert!(fence_palette(&TerrainImageSamples::default(),&cells[..2]).is_none());
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
    fn ruined_tower_palette_excludes_the_fifth_gray_roof_course() {
        let (mut tiles,g,p)=drawing_cells("johto",&[&[0x20,0x21],&[0x37,0x3b]]);
        let mut handles=bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let wall=handles.add(bevy::prelude::Image::default());
        let roof=handles.add(bevy::prelude::Image::default());
        for tile in &mut tiles {tile.texture=if tile.row>=5 {wall.clone()}else{roof.clone()};}
        let mut images=TerrainImageSamples::default();
        images.pixels.insert(roof.id(),TileImageSample::Rgba((0..64).flat_map(|i|[i as u8,0,255,255]).collect()));
        for colors in [
            [[0,0,0],[57,57,57],[165,123,24],[222,255,222]],
            [[0,0,0],[24,16,24],[66,33,41],[123,115,189]],
        ] {
            images.pixels.insert(wall.id(),TileImageSample::Rgba((0..64).flat_map(|i| {
                let c=colors[i%4];[c[0],c[1],c[2],255]
            }).collect()));
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(ecruteak_facade_paint(&cells,&g,p,&images).is_none());
            assert!(ecruteak_facade_paint(&cells,&g,BuildingPlacement {roof_rows:5,..p},&images).is_some());
        }
    }

    #[test]
    fn ecruteak_facade_roles_use_only_complete_live_wall_art_at_each_clock() {
        let (mut tiles,g,mut p)=drawing_cells("johto",&[&[0x2c,0x2d],&[0x26,0x2f]]);
        p.row=2;p.height=6;p.roof_rows=2;
        let mut handles=bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let wall=handles.add(bevy::prelude::Image::default());
        let poison=handles.add(bevy::prelude::Image::default());
        for tile in &mut tiles {tile.texture=if tile.row>=4 && (2..6).contains(&tile.column) {wall.clone()}else{poison.clone()};}
        let cells=tiles.iter().collect::<Vec<_>>();let mut images=TerrainImageSamples::default();
        images.pixels.insert(poison.id(),TileImageSample::Rgba((0..64).flat_map(|i|[i as u8,0,255,255]).collect()));
        let linear=|c:[u8;3]|c.map(|v| {let s=f32::from(v)/255.;if s<=0.04045 {s/12.92}else{((s+0.055)/1.055).powf(2.4)}});
        for colors in [
            [[0,0,0],[57,57,57],[165,123,24],[222,255,222]],
            [[0,0,0],[57,57,57],[165,123,24],[255,255,132]],
            [[0,0,0],[24,16,24],[66,33,41],[123,115,189]],
        ] {
            images.pixels.insert(wall.id(),TileImageSample::Rgba((0..64).flat_map(|i|{
                let c=colors[i%4];[c[0],c[1],c[2],255]
            }).collect()));
            let paint=ecruteak_facade_paint(&cells,&g,p,&images).unwrap();
            assert_eq!(paint.wall,linear(colors[3]));assert_eq!(paint.timber,linear(colors[2]));
            assert_eq!(paint.shadow,linear(colors[1]));assert_eq!(paint.window,linear(colors[0]));
        }
        images.pixels.insert(wall.id(),TileImageSample::Rgba((0..64).flat_map(|i|[i as u8,0,0,255]).collect()));
        assert!(ecruteak_facade_paint(&cells,&g,p,&images).is_none(),"unknown source art cannot acquire guessed facade paint");
        images.pixels.remove(&wall.id());assert!(ecruteak_facade_paint(&cells,&g,p,&images).is_none());
    }

    #[test]
    fn shared_kanto_museum_owns_only_the_complete_doorless_hall_on_any_map() {
        for width in [8,12,16,20] {
            let (mut tiles,g,p,mut images)=kanto_museum::fixture(width);
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(outdoor_building_placements(&cells,&g).contains(&p));
            for map in ["PewterCity","UnseenTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,Asset::KantoMuseum);assert_eq!(d.door,None);
            }
            let shapes=vec![CellShape::Flat;cells.len()];
            let empty=||TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut mesh=empty();let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"UnseenTown",p,&mut claimed,[0,0],Some(&images)));
            assert!(claimed[..width*12].iter().all(|&v|v));assert!(claimed[width*12..].iter().all(|&v|!v));
            assert!(mesh.authored_cells[..width*12].iter().all(|&v|v==Some("outdoor/kanto_museum")));
            assert!(mesh.footing_heights.iter().all(|&v|v==0.));assert_eq!(mesh.rigid_buildings.len(),1);
            assert_eq!(mesh.solid.cutaway_ranges.len(),1);
            images.pixels.remove(&cells[4*width+2].texture.id());let mut rejected=empty();let mut claims=vec![false;cells.len()];
            assert!(!append_building_with_samples(&mut rejected,&cells,&shapes,&g,"UnseenTown",p,&mut claims,[0,0],Some(&images)));
            assert!(rejected.solid.indices.is_empty() && rejected.textured.indices.is_empty());assert!(claims.iter().all(|&v|!v));
            drop(cells);
            for i in 0..width*12 {
                let saved=tiles[i].source.clone();tiles[i].source.tile_index^=1;
                assert!(descriptor("UnseenTown",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());tiles[i].source=saved;
            }
        }
    }

    #[test]
    fn ordinary_house_foundations_continue_complete_wide_native_streets() {
        for flat in [false,true] {for width in [8,16] {
            let (mut base,mut g,p,mut images)=if flat {kanto_flat_house::fixture(width)}else{kanto_house::fixture(width)};
            let ground=base[width*4].clone();
            for y in 5..12 {for x in 0..width {
                let mut tile=ground.clone();tile.column=x as u32;tile.row=y as u32;base.push(tile);
            }}
            g.height=12;
            let build=|tiles:&[VisualTile],images:&TerrainImageSamples| {
                let cells=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;tiles.len()];
                let mut mesh=TerrainMeshData {authored_cells:vec![None;tiles.len()],footing_heights:vec![3.25;tiles.len()],..Default::default()};
                let mut claimed=vec![false;tiles.len()];
                assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"UnseenTown",p,&mut claimed,[0,0],Some(images)));
                assert!(claimed[..width*4].iter().all(|&v|v));assert!(claimed[width*4..].iter().all(|&v|!v));
                assert_eq!(mesh.footing_heights,vec![3.25;tiles.len()]);mesh
            };
            let lawn=build(&base,&images);
            let mut assets=Assets::<Image>::default();let street=assets.add(Image::default());
            images.pixels.insert(street.id(),TileImageSample::Rgba([210,200,170,255].repeat(64)));
            for block in [0x01,0x31,0x77] {
                let mut tiles=base.clone();
                for y in 4..8 {for x in 0..width {
                    let j=(y%4)*4+x%4;let tile=&mut tiles[y*width+x];
                    tile.source.metatile_id=block;tile.source.subtile_column=(x%4) as u8;tile.source.subtile_row=(y%4) as u8;
                    tile.source.tile_index=if block==1 {if matches!(j,4|14) {0x39}else{0x23}}
                        else if block==0x31 || j<8 {0x39}else if j<12 {0x0e}else{0x55};
                    tile.texture=street.clone();
                }}
                let before=tiles.clone();let paved=build(&tiles,&images);
                assert_eq!(paved.solid.positions,lawn.solid.positions);
                assert_eq!(paved.solid.normals,lawn.solid.normals);
                assert_eq!(paved.solid.colors,lawn.solid.colors);
                assert_eq!(paved.solid.indices,lawn.solid.indices);
                assert_eq!(paved.textured.positions,lawn.textured.positions);
                assert_eq!(paved.authored_cells,lawn.authored_cells);
                assert_eq!(tiles,before,"ground selection cannot alter the native house or flower-plot cells");
                assert_eq!(super::new_bark::sampled_cell(&paved.textured.uvs[..4],&g),Some(width*4));
            }
        }}
    }

    #[test]
    fn house_foundation_street_requires_full_frontage_and_complete_opaque_native_blocks() {
        let (mut tiles,mut g,p,mut images)=kanto_house::fixture(8);let ground=tiles[32].clone();
        for y in 5..12 {for x in 0..8 {
            let mut tile=ground.clone();tile.column=x as u32;tile.row=y as u32;tiles.push(tile);
        }}
        g.height=12;let lawn=tiles.clone();
        let mut assets=Assets::<Image>::default();let street=assets.add(Image::default());
        images.pixels.insert(street.id(),TileImageSample::Rgba([210,200,170,255].repeat(64)));
        for y in 4..8 {for x in 0..8 {
            let j=(y%4)*4+x%4;let t=&mut tiles[y*8+x];
            t.source.metatile_id=1;t.source.tile_index=if matches!(j,4|14) {0x39}else{0x23};
            t.source.subtile_column=(x%4) as u8;t.source.subtile_row=(y%4) as u8;t.texture=street.clone();
        }}
        let shapes=vec![CellShape::Flat;tiles.len()];
        assert_eq!(house_frontage_floor_sample(&tiles.iter().collect::<Vec<_>>(),&shapes,&g,p,Some(&images)),Some(32));
        for change in 0..8 {
            let mut altered=tiles.clone();let mut paint=images.clone();let mut geometry=g;let mut levels=shapes.clone();
            match change {
                0=>for y in 4..8 {for x in 4..8 {altered[y*8+x]=lawn[y*8+x].clone();}},
                1=>altered[5*8+1].source.subtile_column^=1,
                2=>altered[5*8+1].source.tileset_id="johto".into(),
                3=>geometry.height=7,
                4=>if let Some(TileImageSample::Rgba(rgba))=paint.pixels.get_mut(&street.id()) {rgba[3]=0;},
                5=>{paint.pixels.remove(&street.id());},
                6=>levels[32]=CellShape::PlaneAt {height:4.},
                _=>for y in 4..8 {for x in 0..8 {
                    altered[(y+4)*8+x]=tiles[y*8+x].clone();altered[y*8+x]=lawn[y*8+x].clone();
                }},
            }
            let cells=altered.iter().collect::<Vec<_>>();
            assert_eq!(house_frontage_floor_sample(&cells,&levels,&geometry,p,Some(&paint)),None,
                "case {change}: a narrow path, changed/foreign/cropped/unpainted drawing, raised donor or remote street is not a full native frontage");
        }
        // A genuine grass frontage keeps the existing lawn donor.
        let cells=lawn.iter().collect::<Vec<_>>();
        assert!(house_frontage_floor_sample(&cells,&shapes,&g,p,Some(&images)).is_none());
        assert_eq!(structure_ground(&cells,&shapes,"kanto",0x2c),Some(32));
    }

    #[test]
    fn kanto_cottage_paved_frontage_removes_lawn_without_changing_the_building() {
        for width in [8,12,16] {
            let (mut tiles,mut g,p,mut images)=kanto_house::fixture(width);
            let ground=tiles[width*4].clone();
            for y in 5..9 {for x in 0..width {
                let mut tile=ground.clone();tile.column=x as u32;tile.row=y as u32;tiles.push(tile);
            }}
            g.height=9;
            let build=|tiles:&[VisualTile],images:&TerrainImageSamples| {
                let cells=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;cells.len()];
                let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
                let mut claimed=vec![false;cells.len()];
                assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"UnseenTown",p,&mut claimed,[0,0],Some(images)));
                assert!(claimed[..width*4].iter().all(|&v|v));assert!(claimed[width*4..].iter().all(|&v|!v));
                assert!(mesh.footing_heights.iter().all(|&h|h==0.));mesh
            };
            let lawn=build(&tiles,&images);
            let mut assets=Assets::<Image>::default();let paving=assets.add(Image::default());
            let rgba=(0..64).flat_map(|i| {
                let (x,y)=(i%8,i/8);let joint=y%4==0 || x==if y<4 {3}else{7};
                if joint {[145,110,45,255]}else{[220,235,210,255]}
            }).collect();
            images.pixels.insert(paving.id(),TileImageSample::Rgba(rgba));
            for y in 4..8 {for x in 0..width {
                let tile=&mut tiles[y*width+x];tile.texture=paving.clone();
                tile.source=VisualTileSource {tileset_id:"kanto".into(),metatile_id:0x55,
                    tile_index:0x5b,subtile_column:(x%4) as u8,subtile_row:(y%4) as u8};
            }}
            let paved=build(&tiles,&images);
            let a=lawn.rigid_buildings[0].vertices.clone();let b=paved.rigid_buildings[0].vertices.clone();
            assert_eq!(lawn.solid.positions[a.clone()],paved.solid.positions[b.clone()]);
            assert_eq!(lawn.solid.colors[a],paved.solid.colors[b]);
            assert!(paved.solid.positions.len()>lawn.solid.positions.len(),"finished brick geometry replaces lawn backing");
            images.pixels.remove(&paving.id());let missing=build(&tiles,&images);
            assert_eq!(missing.solid,lawn.solid,"missing paving art keeps the existing ground fallback");
        }
    }

    #[test]
    fn shared_kanto_house_binds_complete_narrow_and_extended_drawings_on_any_map() {
        for width in [8,12,16] {
            let (mut tiles,g,p,mut images)=kanto_house::fixture(width);
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(outdoor_building_placements(&cells,&g).contains(&p));
            for map in ["LavenderTown","CeruleanCity","PewterCity","UnseenTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,Asset::KantoHouse);assert_eq!(d.door,Some(3.));
            }
            let shapes=vec![CellShape::Flat;cells.len()];
            let empty=||TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut mesh=empty();let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"UnseenTown",p,&mut claimed,[0,0],Some(&images)));
            assert!(claimed[..width*4].iter().all(|&v|v));assert!(claimed[width*4..].iter().all(|&v|!v));
            assert!(mesh.authored_cells[..width*4].iter().all(|&v|v==Some("outdoor/kanto_house")));
            assert!(mesh.footing_heights.iter().all(|&v|v==0.));assert_eq!(mesh.rigid_buildings.len(),1);
            assert_eq!(mesh.solid.cutaway_ranges.len(),1);
            images.pixels.remove(&cells[width+2].texture.id());
            let mut rejected=empty();let mut claims=vec![false;cells.len()];
            assert!(!append_building_with_samples(&mut rejected,&cells,&shapes,&g,"UnseenTown",p,&mut claims,[0,0],Some(&images)));
            assert!(rejected.solid.indices.is_empty() && rejected.textured.indices.is_empty());assert!(claims.iter().all(|&v|!v));
            drop(cells);
            for i in 0..width*4 {
                let saved=tiles[i].source.clone();tiles[i].source.tile_index^=1;
                assert!(descriptor("UnseenTown",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
                tiles[i].source=saved;
            }
        }
    }

    #[test]
    fn shared_kanto_gym_preserves_source_drawing_ownership_and_support() {
        let (mut tiles,g,p,mut images)=kanto_gym::fixture();
        let cells=tiles.iter().collect::<Vec<_>>();
        assert!(outdoor_building_placements(&cells,&g).contains(&p));
        for map in ["PewterCity","VermilionCity","SaffronCity","CeladonCity","FuchsiaCity","ViridianCity","UnseenTown"] {
            let d=descriptor(map,&cells,&g,p).unwrap();assert_eq!(d.asset,Asset::KantoGym);assert_eq!(d.door,Some(9.));
        }
        let shapes=vec![CellShape::Flat;cells.len()];
        let empty=||TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut mesh=empty();let mut claimed=vec![false;cells.len()];
        assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"VermilionCity",p,&mut claimed,[0,0],Some(&images)));
        assert!(claimed[..96].iter().all(|&v|v));assert!(claimed[96..].iter().all(|&v|!v));
        assert!(mesh.authored_cells[..96].iter().all(|&v|v==Some("outdoor/kanto_gym")));
        assert!(mesh.footing_heights.iter().all(|&v|v==0.));assert_eq!(mesh.rigid_buildings.len(),1);
        images.pixels.remove(&cells[4+g.width].texture.id());
        let mut rejected=empty();let mut claims=vec![false;cells.len()];
        assert!(!append_building_with_samples(&mut rejected,&cells,&shapes,&g,"UnseenTown",p,&mut claims,[0,0],Some(&images)));
        assert!(claims.iter().all(|&v|!v));assert!(rejected.solid.indices.is_empty());
        drop(cells);tiles[1].source.subtile_column=0;
        assert!(descriptor("VermilionCity",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
    }

    #[test]
    fn native_plaza_foundations_replace_lawn_without_changing_the_shared_building() {
        let (mut tiles,mut g,p,images)=kanto_gym::fixture_width(16);
        let build=|tiles:&[VisualTile],g:&GridGeometry| {
            let refs=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;tiles.len()];
            let mut mesh=TerrainMeshData {authored_cells:vec![None;tiles.len()],footing_heights:vec![0.;tiles.len()],..Default::default()};
            let mut claimed=vec![false;tiles.len()];
            assert!(append_building_with_samples(&mut mesh,&refs,&shapes,g,"UnseenTown",p,&mut claimed,[0,0],Some(&images)));
            assert!(claimed[..p.width*p.height].iter().all(|&v|v));
            assert!(claimed[p.width*p.height..].iter().all(|&v|!v));
            assert!(mesh.footing_heights.iter().all(|&v|v==0.));mesh
        };
        let lawn=build(&tiles,&g);
        let template=tiles[8*g.width].clone();tiles.truncate(8*g.width);
        for y in 8..12 {for x in 0..g.width {
            let mut tile=template.clone();tile.column=x as u32;tile.row=y as u32;
            tile.source.metatile_id=0x7b;tile.source.subtile_column=(x%4) as u8;
            tile.source.subtile_row=(y%4) as u8;
            tile.source.tile_index=if (x+y)%2==0 {0x30} else {0x39};tiles.push(tile);
        }}
        g.height=12;let paved=build(&tiles,&g);
        assert_eq!(lawn.solid.positions,paved.solid.positions);
        assert_eq!(lawn.solid.normals,paved.solid.normals);
        assert_eq!(lawn.solid.colors,paved.solid.colors);
        assert_eq!(lawn.solid.indices,paved.solid.indices);
        assert_eq!(lawn.textured.positions,paved.textured.positions);
        let refs=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;tiles.len()];
        let sample=courtyard_floor_sample(&refs,&shapes,&g,p,false).unwrap();
        assert_eq!(sample,8*g.width+1,"plaza accepted even outside civic-only dry-ground policy");
        assert_eq!(super::new_bark::sampled_cell(&paved.textured.uvs[..4],&g),Some(sample));
        drop(refs);
        for x in (0..g.width).step_by(4) {tiles[8*g.width+x].source.subtile_column=1;}
        assert_eq!(courtyard_floor_sample(&tiles.iter().collect::<Vec<_>>(),&shapes,&g,p,false),None);
        // Prop-bearing79 is not a paving donor, even if its first sample looks identical.
        for t in &mut tiles[8*g.width..] {t.source.metatile_id=0x79;t.source.subtile_column=(t.column%4) as u8;}
        assert_eq!(courtyard_floor_sample(&tiles.iter().collect::<Vec<_>>(),&shapes,&g,p,false),None);
    }

    #[test]
    fn civic_foundation_borrows_only_a_complete_adjacent_native_courtyard() {
        let (mut tiles,mut g,p,_)=kanto_gym::fixture_width(16);
        let template=tiles[8*g.width].clone();tiles.truncate(8*g.width);
        for y in 8..12 {for x in 0..g.width {
            let mut tile=template.clone();tile.column=x as u32;tile.row=y as u32;
            tile.source.metatile_id=0x77;tile.source.subtile_column=(x%4) as u8;
            tile.source.subtile_row=(y%4) as u8;
            tile.source.tile_index=if y<10 {0x39} else if y==10 {0x0e} else {0x55};
            tiles.push(tile);
        }}
        g.height=12;let shapes=vec![CellShape::Flat;tiles.len()];
        let original=tiles.iter().map(|t|t.source.clone()).collect::<Vec<_>>();
        let refs=tiles.iter().collect::<Vec<_>>();
        assert_eq!(courtyard_floor_sample(&refs,&shapes,&g,p,true),Some(8*g.width));
        assert_eq!(tiles.iter().map(|t|t.source.clone()).collect::<Vec<_>>(),original);
        // Alter every candidate block's authored phase. No remote fallback or partial drawing.
        for x in (0..g.width).step_by(4) {tiles[8*g.width+x].source.subtile_column=1;}
        assert_eq!(courtyard_floor_sample(&tiles.iter().collect::<Vec<_>>(),&shapes,&g,p,true),None);
    }

    #[test]
    fn complete_civic_gym_bays_share_one_producer_at_every_source_width() {
        for width in [8,12,16,20,24] {
            let (tiles,g,p,images)=kanto_gym::fixture_width(width);
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(outdoor_building_placements(&cells,&g).contains(&p));
            for map in ["SaffronCity","CeruleanCity","UnseenTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,Asset::KantoGym);assert_eq!(d.door,Some((width-3) as f32));
            }
            let shapes=vec![CellShape::Flat;cells.len()];
            let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"SaffronCity",p,&mut claimed,[0,0],Some(&images)));
            assert!(claimed[..width*8].iter().all(|&v|v));assert!(claimed[width*8..].iter().all(|&v|!v));
            assert!(mesh.authored_cells[..width*8].iter().all(|&v|v==Some("outdoor/kanto_gym")));
            assert!(mesh.footing_heights.iter().all(|&v|v==0.));assert_eq!(mesh.rigid_buildings.len(),1);
        }
    }

    #[test]
    fn kanto_brick_office_keeps_source_ownership_support_and_missing_paint_fallback() {
        let (mut tiles,mut g,p)=drawing_cells("kanto",&[&[0x20,0x54,0x21],&[0x37,0x3a,0x7e]]);
        let mut assets=Assets::<Image>::default();
        let roof=assets.add(Image::default());let wall=assets.add(Image::default());
        let glass=assets.add(Image::default());let door=assets.add(Image::default());
        let unlit=assets.add(Image::default());
        for (dy,row) in [[0x0b,0x0c],[0x1b,0x1c]].iter().enumerate() {
            for (dx,&tile) in row.iter().enumerate() {tiles[(6+dy)*g.width+4+dx].source.tile_index=tile;}
        }
        for (x,y,index,texture) in [(5,1,0x5e,roof.clone()),(2,5,0x4b,wall.clone()),
            (2,4,0x0a,glass.clone()),(4,7,0x1b,door.clone())] {
            let tile=&mut tiles[y*g.width+x];tile.source.tile_index=index;tile.texture=texture;
        }
        for (x,y,index) in [(1,4,0x32),(2,4,0x0a),(3,4,0x0a),(4,4,0x32),
            (7,4,0x32),(8,4,0x32),(9,4,0x0a),(10,4,0x32),(6,6,0x0a),(7,6,0x0a),(4,6,0x0b)] {
            let tile=&mut tiles[y*g.width+x];tile.source.tile_index=index;
            tile.texture=if index==0x32 {unlit.clone()}else{glass.clone()};
        }
        for x in [5,6] {tiles[4*g.width+x].source.tile_index=0x4b;}
        for x in 0..g.width {
            let mut tile=tiles[0].clone();tile.column=x as u32;tile.row=8;
            tile.source=VisualTileSource {tileset_id:"kanto".into(),metatile_id:0x01,subtile_column:0,subtile_row:0,tile_index:KANTO_GROUND_TILE_INDEX};
            tiles.push(tile);
        }
        g.height=9;
        let mut images=TerrainImageSamples::default();
        for (h,colors) in [(roof,[[70,70,70,255],[150,150,150,255]]),
            (wall.clone(),[[160,110,30,255],[222,255,222,255]]),
            (glass,[[57,57,57,255],[222,255,222,255]]),
            (door,[[160,110,30,255],[160,110,30,255]])] {
            images.pixels.insert(h.id(),TileImageSample::Rgba((0..64).flat_map(|i|colors[i%2]).collect()));
        }
        images.pixels.insert(unlit.id(),TileImageSample::Rgba((0..64).flat_map(|_|[50,40,100,255]).collect()));
        let cells=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;cells.len()];
        let pane_paint=kanto_brick_office::paint(&cells,&g,p,&images).unwrap();
        assert!(pane_paint.windows[0][2]>pane_paint.windows[0][1]);
        assert!(pane_paint.windows[1][1]>pane_paint.windows[1][2]);
        assert_eq!(pane_paint.windows[0],pane_paint.windows[3]);
        assert_eq!(pane_paint.windows[1],pane_paint.windows[8]);
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut claimed=vec![false;cells.len()];
        assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"PalletTown",p,&mut claimed,[0,0],Some(&images)));
        assert!(claimed[..96].iter().all(|&v|v));assert!(claimed[96..].iter().all(|&v|!v));
        assert!(mesh.authored_cells[..96].iter().all(|&v|v==Some("outdoor/kanto_brick_office")));
        assert!(mesh.footing_heights.iter().all(|&v|v==0.));
        assert_eq!(mesh.rigid_buildings.len(),1);
        // Viridian's Trainer House borders the plain street drawing, not lawn.
        // Continue its complete local source under the same shared building.
        drop(cells);
        let template=tiles[8*g.width].clone();tiles.truncate(8*g.width);
        for y in 8..13 {for x in 0..g.width {
            let mut tile=template.clone();tile.column=x as u32;tile.row=y as u32;
            let j=(y%4)*4+x%4;
            tile.source=VisualTileSource {tileset_id:"kanto".into(),metatile_id:0x01,
                subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                tile_index:if y==12 {KANTO_GROUND_TILE_INDEX} else if matches!(j,4|14) {0x39} else {0x23}};
            tiles.push(tile);
        }}
        g.height=13;let cells=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;cells.len()];
        let sample=courtyard_floor_sample(&cells,&shapes,&g,p,true).unwrap();
        assert_eq!(sample,8*g.width);
        assert_eq!(courtyard_floor_sample(&cells,&shapes,&g,p,false),None,
            "ordinary residential plots do not acquire civic dry-ground policy");
        let mut street=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut claims=vec![false;cells.len()];
        assert!(append_building_with_samples(&mut street,&cells,&shapes,&g,"UnseenTown",p,&mut claims,[0,0],Some(&images)));
        assert_eq!(street.solid,mesh.solid,"the shared building stays unchanged");
        assert_eq!(street.textured.positions,mesh.textured.positions);
        assert_eq!(super::new_bark::sampled_cell(&street.textured.uvs[..4],&g),Some(sample));
        assert!(claims[..96].iter().all(|&v|v));assert!(claims[96..].iter().all(|&v|!v));
        assert!(street.footing_heights.iter().all(|&v|v==0.));
        drop(cells);
        for x in (0..g.width).step_by(4) {tiles[8*g.width+x].source.tile_index=0x2c;}
        let cells=tiles.iter().collect::<Vec<_>>();
        assert_eq!(courtyard_floor_sample(&cells,&shapes,&g,p,true),None,
            "partial or altered street drawings retain the original fallback");
        images.pixels.remove(&wall.id());
        let mut rejected=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut claims=vec![false;cells.len()];
        assert!(!append_building_with_samples(&mut rejected,&cells,&shapes,&g,"FuchsiaCity",p,&mut claims,[0,0],Some(&images)));
        assert!(claims.iter().all(|&v|!v));assert!(rejected.solid.indices.is_empty());
    }

    #[test]
    fn doorless_kanto_brick_annex_uses_shared_office_and_keeps_all_ten_source_panes() {
        let (mut tiles,mut g,p)=drawing_cells("kanto",&[&[0x20,0x54,0x21],&[0x37,0x7d,0x7e]]);
        let mut assets=Assets::<Image>::default();let mut images=TerrainImageSamples::default();
        let mut texture=|colors:[[u8;4];2]| {
            let h=assets.add(Image::default());images.pixels.insert(h.id(),TileImageSample::Rgba(
                (0..64).flat_map(|i|colors[i%2]).collect()));h
        };
        let roof=texture([[70,70,70,255],[150,150,150,255]]);
        let wall=texture([[160,110,30,255],[222,255,222,255]]);
        let lit=texture([[57,57,57,255],[222,255,222,255]]);
        let unlit=texture([[57,57,57,255],[80,70,140,255]]);
        for (x,y,index,h) in [(5,1,0x5e,roof),(2,5,0x4b,wall)] {
            tiles[y*g.width+x].source.tile_index=index;tiles[y*g.width+x].texture=h;
        }
        for (i,index) in [0x32,0x0a,0x0a,0x0a,0x32,0x0a,0x0a,0x32,0x0a,0x32].into_iter().enumerate() {
            let tile=&mut tiles[4*g.width+1+i];tile.source.tile_index=index;
            tile.texture=if index==0x32 {unlit.clone()} else {lit.clone()};
        }
        for x in 0..g.width {
            let mut tile=tiles[0].clone();tile.column=x as u32;tile.row=8;
            tile.source=VisualTileSource{tileset_id:"kanto".into(),metatile_id:0x01,
                subtile_column:0,subtile_row:0,tile_index:KANTO_GROUND_TILE_INDEX};tiles.push(tile);
        }
        g.height=9;
        let cells=tiles.iter().collect::<Vec<_>>();
        assert!(outdoor_building_placements(&cells,&g).contains(&p));
        for map in ["PewterCity","UnseenTown","CeladonCity"] {
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::KantoBrickOffice);assert_eq!(d.door,None);
        }
        let paint=kanto_brick_office::paint(&cells,&g,p,&images).unwrap();
        for i in 0..10 {assert_eq!(paint.windows[i],paint.windows[if [0,4,7,9].contains(&i) {0}else{1}]);}
        assert_ne!(paint.windows[0],paint.windows[1]);
        let shapes=vec![CellShape::Flat;cells.len()];
        let empty=||TerrainMeshData{authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut mesh=empty();let mut claims=vec![false;cells.len()];
        assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"PewterCity",p,&mut claims,[0,0],Some(&images)));
        assert!(claims[..96].iter().all(|&v|v));assert!(claims[96..].iter().all(|&v|!v));
        assert!(mesh.authored_cells[..96].iter().all(|&v|v==Some("outdoor/kanto_brick_office")));
        assert!(mesh.footing_heights.iter().all(|&v|v==0.));
        images.pixels.remove(&unlit.id());let mut rejected=empty();let mut claims=vec![false;cells.len()];
        assert!(!append_building_with_samples(&mut rejected,&cells,&shapes,&g,"PewterCity",p,&mut claims,[0,0],Some(&images)));
        assert!(claims.iter().all(|&v|!v));assert!(rejected.solid.indices.is_empty());
        drop(cells);tiles[4*g.width+4].source.subtile_column=1;
        assert!(descriptor("PewterCity",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
    }

    #[test]
    fn kanto_brick_street_blocks_follow_native_width_and_door_bays_in_every_city() {
        for width in [8,12,16,20] {for doorway in [false,true] {
            let blocks=width/4;let mut roof=vec![0x54;blocks];roof[0]=0x20;roof[blocks-1]=0x21;
            let mut wall=vec![0x7d;blocks];wall[0]=0x37;wall[blocks-1]=0x7e;
            let door=if !doorway {None} else if width==8 {wall[0]=0x7c;Some(2)}
                else {wall[blocks-2]=0x3a;Some((blocks-2)*4)};
            let (mut tiles,g,p)=drawing_cells("kanto",&[&roof,&wall]);
            if let Some(x)=door {
                for (dx,dy,index) in [(0,0,0x0b),(1,0,0x0c),(0,1,0x1b),(1,1,0x1c)] {
                    tiles[(6+dy)*g.width+x+dx].source.tile_index=index;
                }
            }
            let cells=tiles.iter().collect::<Vec<_>>();
            for map in ["SaffronCity","PewterCity","UnseenTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,Asset::KantoBrickOffice);
                assert_eq!(d.door,door.map(|x|x as f32+1.));
            }
            drop(cells);tiles[g.width-1].source.subtile_column=0;
            assert!(descriptor("SaffronCity",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
        }}
        for service in [0x72,0x73] {
            let (mut tiles,g,p)=drawing_cells("kanto",&[&[0x20,0x21],&[0x7c,service]]);
            for (dx,dy,index) in [(0,0,0x0b),(1,0,0x0c),(0,1,0x1b),(1,1,0x1c)] {
                tiles[(6+dy)*g.width+2+dx].source.tile_index=index;
            }
            assert_ne!(descriptor("SaffronCity",&tiles.iter().collect::<Vec<_>>(),&g,p).unwrap().asset,Asset::KantoBrickOffice);
        }
    }

    #[test]
    fn twin_entrance_story_facade_keeps_both_doors_all_panes_and_shared_office_identity() {
        let (mut tiles,mut g,p)=drawing_cells("kanto",&[
            &[0x68,0x7f,0x7f,0x69],&[0x68,0x7f,0x7f,0x69],&[0x37,0x53,0x3a,0x7e]]);
        let mut assets=Assets::<Image>::default();
        let brick=assets.add(Image::default());let light=assets.add(Image::default());
        let dark=assets.add(Image::default());let trim=assets.add(Image::default());
        let door=assets.add(Image::default());
        for tile in &mut tiles {
            tile.source.tile_index=0x4b;tile.texture=brick.clone();
            if tile.row%2==0 && tile.column>0 && tile.column+1<g.width as u32 {
                tile.source.tile_index=if tile.column%3==0 {0x32}else{0x0a};
                tile.texture=if tile.column%3==0 {dark.clone()}else{light.clone()};
            }
        }
        tiles[0].source.tile_index=0x0f;tiles[0].texture=trim.clone();
        for x in [6,8] {for (dx,dy,index) in [(0,0,0x0b),(1,0,0x0c),(0,1,0x1b),(1,1,0x1c)] {
            let t=&mut tiles[(p.height-2+dy)*g.width+x+dx];t.source.tile_index=index;
            t.texture=if dy==0 {light.clone()}else{door.clone()};
        }}
        let source_panes=tiles.iter().filter(|t|matches!(t.source.tile_index,0x0a|0x32)).count();
        for x in 0..g.width {
            let mut t=tiles[0].clone();t.column=x as u32;t.row=g.height as u32;
            t.source=VisualTileSource {tileset_id:"kanto".into(),metatile_id:1,
                subtile_column:(x%4)as u8,subtile_row:0,tile_index:KANTO_GROUND_TILE_INDEX};
            tiles.push(t);
        }
        g.height+=1;
        let mut images=TerrainImageSamples::default();
        for (h,colors) in [(brick,[[160,110,30,255],[222,255,222,255]]),
            (light,[[57,57,57,255],[222,255,222,255]]),
            (dark,[[50,40,100,255],[50,40,100,255]]),
            (trim,[[70,70,70,255],[150,150,150,255]]),
            (door,[[160,110,30,255],[160,110,30,255]])] {
            images.pixels.insert(h.id(),TileImageSample::Rgba((0..64).flat_map(|i|colors[i%2]).collect()));
        }
        let before=tiles.clone();let cells=tiles.iter().collect::<Vec<_>>();
        assert!(outdoor_building_placements(&cells,&g).contains(&p));
        for map in ["Route23","UnseenTown"] {
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::KantoBrickOffice);assert_eq!(d.door,Some(7.));
        }
        let paint=kanto_brick_office::paint(&cells,&g,p,&images).unwrap();
        assert_eq!(paint.windows.len(),source_panes,"every native story remains visible");
        let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
        let mut claimed=vec![false;cells.len()];let shapes=vec![CellShape::Flat;cells.len()];
        assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"Route23",p,&mut claimed,[0,0],Some(&images)));
        assert!(mesh.authored_cells[..p.height*g.width].iter().all(|&v|v==Some("outdoor/kanto_brick_office")));
        assert!(mesh.authored_cells[p.height*g.width..].iter().all(Option::is_none));
        assert!(mesh.footing_heights.iter().all(|&h|h==0.));
        let lamps=crate::city_lights::collect(&mesh.solid);
        let mut doors=lamps.iter().filter(|e|(e.position.y-2.38*g.tile_width).abs()<0.05*g.tile_width)
            .map(|e|e.position.x/g.tile_width).collect::<Vec<_>>();
        doors.sort_by(f32::total_cmp);
        assert_eq!(doors.len(),2,"both native door lamps remain separate volumes");
        assert!((doors[0]-7.).abs()<0.01 && (doors[1]-9.).abs()<0.01);
        assert_eq!(tiles,before,"presentation never rewrites the native facade");
        drop(cells);
        tiles[(p.height-1)*g.width+8].source.tile_index=0x1a;
        assert!(descriptor("Route23",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none(),
            "an altered twin door cannot fall through to an unrelated tower");
    }

    #[test]
    fn tall_kanto_retail_uses_shared_brick_only_with_its_native_sign() {
        for blocks in [3,4,5] { for stories in [1,2,4] {
            let mut roof=vec![0x54;blocks];roof[0]=0x20;roof[blocks-1]=0x21;
            let mut upper=vec![0x7f;blocks];upper[0]=0x68;upper[blocks-1]=0x69;
            let mut base=vec![0x7d;blocks];base[0]=0x37;
            base[blocks-1]=0x73;
            base[1]=0x3a;let door=4;
            let mut rows=vec![roof.as_slice()];
            rows.extend(std::iter::repeat_n(upper.as_slice(),stories));rows.push(&base);
            let (mut tiles,g,p)=drawing_cells("kanto",&rows);
            for (dx,dy,index) in [(0,0,0x0b),(1,0,0x0c),(0,1,0x1b),(1,1,0x1c)] {
                tiles[(p.height-2+dy)*g.width+door+dx].source.tile_index=index;
            }
            let sign=(p.height-2)*g.width+p.width-4;
            for (dx,dy,index) in [(0,0,0x44),(1,0,0x45),(0,1,0x4a),(1,1,0x4a)] {
                tiles[sign+dy*g.width+dx].source.tile_index=index;
            }
            for map in ["CeladonCity","SaffronCity","UnseenTown"] {
                let cells=tiles.iter().collect::<Vec<_>>();
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,Asset::KantoBrickOffice);
                assert_eq!(d.door,Some(door as f32+1.));
            }
            for index in [sign,sign+1,sign+g.width,sign+g.width+1] {
                tiles[index].source.tile_index^=1;
                assert!(!super::kanto_brick_office::matches(&tiles.iter().collect::<Vec<_>>(),&g,p));
                tiles[index].source.tile_index^=1;
            }
        }}
    }

    #[test]
    fn stacked_kanto_brick_drawings_share_source_stories_without_shop_awnings() {
        for (blocks,stories) in [(2,1),(3,1),(4,4),(5,6)] {
            let mut roof=vec![0x54;blocks];roof[0]=0x20;roof[blocks-1]=0x21;
            let mut upper=vec![0x7f;blocks];upper[0]=0x68;upper[blocks-1]=0x69;
            let mut base=vec![0x7d;blocks];base[0]=0x37;base[blocks-1]=0x7e;
            if blocks>2 {base[1]=0x3a;}
            let mut rows=vec![roof.as_slice()];
            rows.extend(std::iter::repeat_n(upper.as_slice(),stories));rows.push(&base);
            let (mut tiles,mut g,p)=drawing_cells("kanto",&rows);
            let mut assets=Assets::<Image>::default();
            let brick=assets.add(Image::default());let light=assets.add(Image::default());
            let dark=assets.add(Image::default());
            for tile in &mut tiles {
                tile.source.tile_index=0x4b;tile.texture=brick.clone();
                if tile.row>=4 && tile.row%2==0 && tile.column>0 && tile.column+1<g.width as u32 {
                    tile.source.tile_index=if tile.column%3==0 {0x32} else {0x0a};
                    tile.texture=if tile.column%3==0 {dark.clone()} else {light.clone()};
                }
            }
            tiles[g.width+5].source.tile_index=0x5e;
            if blocks>2 {
                for (dx,dy,index) in [(0,0,0x0b),(1,0,0x0c),(0,1,0x1b),(1,1,0x1c)] {
                    tiles[(g.height-2+dy)*g.width+4+dx].source.tile_index=index;
                }
            }
            for x in 0..g.width {
                let mut tile=tiles[0].clone();tile.column=x as u32;tile.row=g.height as u32;
                tile.source=VisualTileSource {tileset_id:"kanto".into(),metatile_id:0x01,
                    subtile_column:0,subtile_row:0,tile_index:KANTO_GROUND_TILE_INDEX};
                tiles.push(tile);
            }
            g.height+=1;
            let mut images=TerrainImageSamples::default();
            for (h,colors) in [(brick,[[80,70,60,255],[180,190,180,255]]),
                (light.clone(),[[57,57,57,255],[255,255,132,255]]),
                (dark.clone(),[[50,40,100,255],[50,40,100,255]])] {
                images.pixels.insert(h.id(),TileImageSample::Rgba((0..64).flat_map(|i|colors[i%2]).collect()));
            }
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(outdoor_building_placements(&cells,&g).contains(&p));
            for map in ["SaffronCity","CeladonCity","UnseenTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();assert_eq!(d.asset,Asset::KantoBrickOffice);
                assert_eq!(d.door,if blocks>2 {Some(5.)} else {None});
            }
            let shapes=vec![CellShape::Flat;cells.len()];
            let mut mesh=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"SaffronCity",p,&mut claimed,[0,0],Some(&images)));
            assert!(claimed[..g.width*p.height].iter().all(|&v|v));
            assert!(claimed[g.width*p.height..].iter().all(|&v|!v));
            assert!(mesh.authored_cells[..g.width*p.height].iter().all(|&v|v==Some("outdoor/kanto_brick_office")));
            assert!(mesh.footing_heights.iter().all(|&v|v==0.));
            let top=mesh.solid.positions.iter().map(|p|p[1]).fold(0.,f32::max);
            // The shared vent cover projects above the unchanged coping;
            // each native story still contributes exactly the same rise.
            assert!((top-(5.94+stories as f32*3.80)*8.).abs()<0.001);
            assert!(mesh.solid.positions.iter().any(|p|
                (p[1]-(5.70+stories as f32*3.80)*8.).abs()<0.001));
            let triangles=mesh.solid.indices.len()/3;
            assert!(triangles<200_000,"largest supported 20x32 drawing: {triangles} triangles");
            if blocks==4 && stories==4 {assert!(triangles<100_000,"Silph: {triangles} triangles");}
            let source_lights=cells.iter().filter(|t|t.source.tile_index==0x0a).count();
            let model_lights=crate::city_lights::collect(&mesh.solid);
            assert_eq!(model_lights.len(),source_lights+usize::from(blocks>2),
                "every native lamp and glazed door, no dark pane relit");
            let emitters=crate::city_lights::collect(&mesh.solid).into_iter()
                .map(|e|(e.position,e.direction)).collect::<Vec<_>>();
            for (lit_paint,dark_paint) in [
                ([95,150,180,255],[220,230,240,255]),
                ([80,65,120,255],[50,40,100,255]),
            ] {
                // A daylight/dusk build must retain the same lamps for a later
                // RTC transition. Even bright paint cannot relight $32 glass.
                for (h,color) in [(&light,lit_paint),(&dark,dark_paint)] {
                    images.pixels.insert(h.id(),TileImageSample::Rgba((0..64).flat_map(|i|
                        if i%2==0 {[57,57,57,255]}else{color}).collect()));
                }
                let mut repainted=TerrainMeshData {authored_cells:vec![None;cells.len()],
                    footing_heights:vec![0.;cells.len()],..Default::default()};
                let mut repaint_claims=vec![false;cells.len()];
                assert!(append_building_with_samples(&mut repainted,&cells,&shapes,&g,
                    "SaffronCity",p,&mut repaint_claims,[0,0],Some(&images)));
                assert_eq!(repainted.solid.uvs,mesh.solid.uvs,
                    "window emission identity must be independent of the build-time palette");
                assert_eq!(crate::city_lights::collect(&repainted.solid).into_iter()
                    .map(|e|(e.position,e.direction)).collect::<Vec<_>>(),emitters);
            }
            eprintln!("stacked brick {}x{}: {triangles} triangles, {source_lights} source-lit panes",p.width,p.height);
            assert!(mesh.solid.positions.iter().all(|p|p.iter().all(|v|v.is_finite())));
            drop(cells);
            // Reject a missing repeated story or a mismatched source phase.
            tiles[4*g.width].source.metatile_id=0x37;
            assert!(!kanto_brick_office::matches(&tiles.iter().collect::<Vec<_>>(),&g,p));
            tiles[4*g.width].source.metatile_id=0x68;
            tiles[4*g.width].source.subtile_row=1;
            assert!(!kanto_brick_office::matches(&tiles.iter().collect::<Vec<_>>(),&g,p));
        }
    }

    #[test]
    fn shared_kanto_brick_office_uses_the_complete_drawing_across_four_cities() {
        let (mut tiles,g,p)=drawing_cells("kanto",&[&[0x20,0x54,0x21],&[0x37,0x3a,0x7e]]);
        for (dy,row) in [[0x0b,0x0c],[0x1b,0x1c]].iter().enumerate() {
            for (dx,&tile) in row.iter().enumerate() {tiles[(6+dy)*g.width+4+dx].source.tile_index=tile;}
        }
        let cells=tiles.iter().collect::<Vec<_>>();
        assert!(outdoor_building_placements(&cells,&g).contains(&p));
        for map in ["PalletTown","CeladonCity","SaffronCity","FuchsiaCity","UnseenTown"] {
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::KantoBrickOffice);assert_eq!(d.door,Some(5.));
        }
        let changed=tiles[0].source.metatile_id;
        tiles[0].source.metatile_id=0x30;
        assert!(descriptor("FuchsiaCity",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
        tiles[0].source.metatile_id=changed;
        tiles[6*g.width+4].source.tile_index=0xffff;
        assert_ne!(descriptor("FuchsiaCity",&tiles.iter().collect::<Vec<_>>(),&g,p).map(|d|d.asset),Some(Asset::KantoBrickOffice));
    }

    #[test]
    fn connected_north_south_gate_uses_one_asset_and_keeps_its_native_door() {
        for roof in [[0x08,0x09],[0x18,0x19]] {
            let (mut tiles,g,p)=drawing_cells("johto",&[&roof,&[0x1a,0x11]]);
            for (dy,row) in [[0x37,0x38],[0x39,0x3a]].iter().enumerate() {
                for (dx,&tile) in row.iter().enumerate() {tiles[(6+dy)*g.width+2+dx].source.tile_index=tile;}
            }
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(outdoor_building_placements(&cells,&g).contains(&p));
            for map in ["Route29","Route46","TestTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,Asset::Johto(Johto::RouteGate));
                assert_eq!(d.door,Some(3.));assert!(!d.east);
                let all=building_placements(map,&cells,&g);
                assert_eq!(all,vec![p],"one complete shared gate for {map}");
            }
            tiles[0].source.subtile_column=1;
            assert!(descriptor("Route29",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
        }
    }

    #[test]
    fn shared_civic_reference_assets_do_not_depend_on_the_town() {
        for (last,asset) in [(0x1b,Asset::Johto(Johto::Pokecenter)),(0x33,Asset::Johto(Johto::Mart))] {
            let (mut tiles,g,p)=drawing_cells("johto_modern",&[&[0x18,0x19],&[0x1a,last]]);
            for (dy,row) in [[0x37,0x38],[0x39,0x3a]].iter().enumerate() {
                for (dx,&tile) in row.iter().enumerate() {tiles[(6+dy)*g.width+2+dx].source.tile_index=tile;}
            }
            let cells=tiles.iter().collect::<Vec<_>>();
            for map in ["AzaleaTown","GoldenrodCity","Route34","TestTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();
                assert_eq!(d.asset,asset);assert_eq!(d.door,Some(3.));
            }
        }
        let (tiles,g,p)=drawing_cells("johto_modern",&[&[0x08,0x09],&[0x1c,0x1e]]);
        let cells=tiles.iter().collect::<Vec<_>>();
        for map in ["AzaleaTown","Route34","TestTown"] {
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::World(Kind::EastGate));assert!(d.east);assert_eq!(d.door,Some(6.));
        }
    }

    #[test]
    fn modern_compact_pair_is_one_house_with_one_source_entrance() {
        let (mut tiles,mut g,p)=drawing_cells("johto_modern",&[&[0x14,0x15]]);
        let p=BuildingPlacement {roof_rows:2,..p};
        let art=[
            [[0x10,0x11,0x11,0x11],[0x0a,0x0b,0x0b,0x0b],
                [0x1a,0x1b,0x37,0x38],[0x01,0x02,0x39,0x3a]],
            [[0x11,0x11,0x11,0x12],[0x0b,0x0b,0x0b,0x0c],
                [0x1b,0x1b,0x1b,0x1c],[0x02,0x02,0x02,0x16]],
        ];
        for tile in &mut tiles {
            tile.source.tile_index=art[tile.column as usize/4][tile.row as usize][tile.column as usize%4];
        }
        for x in 0..g.width {
            let mut tile=tiles[x].clone();tile.row=4;
            tile.source=super::super::tests::source_with_tile(0x01,0,0,6);
            tile.source.tileset_id="johto_modern".into();tiles.push(tile);
        }
        g.height+=1;
        let mut handles=bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let roof=handles.add(bevy::prelude::Image::default());
        let wall=handles.add(bevy::prelude::Image::default());
        let light=handles.add(bevy::prelude::Image::default());
        let leaf=handles.add(bevy::prelude::Image::default());
        for tile in &mut tiles {tile.texture=match tile.source.tile_index {
            0x37|0x38=>light.clone(),0x39|0x3a=>leaf.clone(),
            _ if tile.row<2=>roof.clone(),_=>wall.clone(),
        };}
        let cells=tiles.iter().collect::<Vec<_>>();
        assert_eq!(building_placements("AzaleaTown",&cells,&g),vec![p]);
        let d=descriptor("AzaleaTown",&cells,&g,p).unwrap();
        assert_eq!(d.asset,Asset::Johto(Johto::House));assert_eq!(d.door,Some(3.));
        let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut images=TerrainImageSamples::default();
        images.pixels.insert(roof.id(),TileImageSample::Rgba((0..64).flat_map(|i|
            [[57,57,57,255],[41,140,255,255],[123,255,255,255]][i%3]).collect()));
        for (plaster,ochre,glass,door) in [
            ([222,255,222,255],[165,123,24,255],[222,255,222,255],[165,123,24,255]),
            ([123,115,189,255],[66,33,41,255],[255,255,132,255],[100,40,95,255]),
        ] {
            images.pixels.insert(wall.id(),TileImageSample::Rgba((0..64).flat_map(|i|
                if i%2==0 {plaster}else{ochre}).collect()));
            images.pixels.insert(light.id(),TileImageSample::Rgba(glass.repeat(64)));
            images.pixels.insert(leaf.id(),TileImageSample::Rgba(door.repeat(64)));
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;cells.len()],
                authored_cells:vec![None;cells.len()],..Default::default()};
            let mut claimed=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"AzaleaTown",p,
                &mut claimed,[0,0],Some(&images)));
            assert_eq!(mesh.rigid_buildings.len(),1);
            assert_eq!(mesh.rigid_buildings[0].doorway,Vec2::new(24.,28.));
            assert!(mesh.authored_cells[..32].iter().all(|label|label.as_deref()==Some("johto/house")));
            assert!(claimed[..32].iter().all(|&c|c));assert!(claimed[32..].iter().all(|&c|!c));
            assert_eq!(mesh.footing_heights,vec![0.;cells.len()]);
            assert_eq!(mesh.coverage_fade_ranges.len(),1);
            assert!(mesh.solid.uvs.iter().any(|&uv|crate::scenery_finish::town_glass_coordinates(uv).is_some()));
            assert!(mesh.solid.positions.iter().all(|p|p[0]>=0. && p[0]<=64. && p[2]>=0. && p[2]<=32.));
        }
        let mut changed=tiles.clone();changed[2*g.width+3].source.tile_index=0;
        let cells=changed.iter().collect::<Vec<_>>();
        assert!(descriptor("AzaleaTown",&cells,&g,p).is_none(),"a missing door cannot acquire a guessed house");
    }

    #[test]
    fn modern_compact_pair_does_not_join_goldenrods_independent_square_neighbors() {
        for blocks in [&[&[0x14,0x13][..]][..],&[&[0x12,0x13][..]][..]] {
            let (tiles,g,_)=drawing_cells("johto_modern",blocks);
            let cells=tiles.iter().collect::<Vec<_>>();
            let placements=building_placements("GoldenrodCity",&cells,&g);
            assert_eq!(placements.len(),2);
            assert!(placements.iter().all(|p|p.width==4 && p.height==4));
        }
    }

    #[test]
    fn unseen_modern_town_roofs_follow_source_paint_without_changing_the_layout() {
        for (blocks,roof_rows) in [
            (vec![vec![0x18,0x19],vec![0x1a,0x1b]],4),
            (vec![vec![0x18,0x19],vec![0x16,0x1e]],4),
            (vec![vec![0x12]],2),
        ] {
            let blocks=blocks.iter().map(Vec::as_slice).collect::<Vec<_>>();
            let (mut tiles,mut g,mut p)=drawing_cells("johto_modern",&blocks);
            p.roof_rows=roof_rows;
            for x in 0..g.width {
                let mut tile=tiles[x].clone();tile.row=g.height as u32;
                tile.source.metatile_id=0x01;tile.source.tile_index=6;
                tiles.push(tile);
            }
            g.height+=1;
            let mut handles=bevy::prelude::Assets::<bevy::prelude::Image>::default();
            let roof=handles.add(bevy::prelude::Image::default());
            let poison=handles.add(bevy::prelude::Image::default());
            for tile in &mut tiles {tile.texture=if (tile.row as usize)<roof_rows {
                roof.clone()}else{poison.clone()};}
            let mut images=TerrainImageSamples::default();
            images.pixels.insert(roof.id(),TileImageSample::Rgba((0..64).flat_map(|i|
                [[57,57,57,255],[41,140,255,255],[123,255,255,255]][i%3]).collect()));
            images.pixels.insert(poison.id(),TileImageSample::Rgba((0..64).flat_map(|i|
                [i as u8,0,0,255]).collect()));
            let cells=tiles.iter().collect::<Vec<_>>();
            let shapes=cells.iter().map(|tile|shape_for_source(&tile.source)).collect::<Vec<_>>();
            for map in ["GoldenrodCity","AzaleaTown"] {
                let render=|samples| {
                    let mut mesh=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
                    let mut claims=vec![false;cells.len()];
                    assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,map,p,
                        &mut claims,[0,0],samples));
                    (mesh,claims)
                };
                let (before,claims)=render(None);
                let (after,new_claims)=render(Some(&images));
                assert_eq!(after.solid.positions,before.solid.positions);
                assert_eq!(after.solid.normals,before.solid.normals);
                assert_eq!(after.solid.indices,before.solid.indices);
                assert_eq!(after.footing_heights,before.footing_heights);
                assert_eq!(after.rigid_buildings,before.rigid_buildings);
                assert_eq!(new_claims,claims);
                let live=after.solid.uvs.iter().zip(&after.solid.colors)
                    .filter(|(uv,_)|**uv==crate::scenery_finish::SOURCE_PAINT_UV).collect::<Vec<_>>();
                assert!(!live.is_empty(),"source paint must reach the real modern kit");
                assert!(live.iter().any(|(_,c)|c[2]>c[0]*2. && c[1]>c[0]),
                    "source cyan must survive Goldenrod's facade variants and roof folds");
            }
        }
    }

    #[test]
    fn common_johto_building_paint_follows_live_source_without_map_exceptions() {
        for (blocks,height,roof_rows,kind) in [
            (vec![vec![0x14,0x15]],4,2,Johto::House),
            (vec![vec![0x18,0x19],vec![0x16,0x1e]],8,4,Johto::PlayerHouse),
            (vec![vec![0x2c,0x2d],vec![0x26,0x2f]],6,2,Johto::TraditionalHouse),
            (vec![vec![0x2c,0x2a,0x2d],vec![0x26,0x27,0x2f]],6,4,Johto::VioletGym),
            (vec![vec![0x18,0x19],vec![0x1a,0x1b]],8,4,Johto::Pokecenter),
            (vec![vec![0x18,0x19],vec![0x1a,0x17]],8,4,Johto::Mart),
        ] {
            let rows=blocks.iter().map(Vec::as_slice).collect::<Vec<_>>();
            let (mut tiles,mut g,mut p)=drawing_cells("johto",&rows);
            p.height=height;p.roof_rows=roof_rows;
            if height==6 {p.row=2;}
            for x in 0..g.width {
                let mut tile=tiles[x].clone();
                tile.column=x as u32;tile.row=g.height as u32;
                tile.source=super::super::tests::source_with_tile(0x01,0,0,6);
                tiles.push(tile);
            }
            g.height+=1;
            let entrance=matches!(kind,Johto::House|Johto::Pokecenter|Johto::Mart);
            if entrance {
                for (row,art) in [[0x37,0x38],[0x39,0x3a]].into_iter().enumerate() {
                    for (column,tile) in art.into_iter().enumerate() {
                        tiles[(p.row+p.height-2+row)*g.width+p.column+2+column].source.tile_index=tile;
                    }
                }
            }
            let mut handles=bevy::prelude::Assets::<bevy::prelude::Image>::default();
            let roof_handle=handles.add(bevy::prelude::Image::default());
            let wall_handle=handles.add(bevy::prelude::Image::default());
            let light_handle=handles.add(bevy::prelude::Image::default());
            let leaf_handle=handles.add(bevy::prelude::Image::default());
            for tile in &mut tiles {tile.texture=if entrance && matches!(tile.source.tile_index,0x37|0x38) {
                light_handle.clone()
            } else if entrance && matches!(tile.source.tile_index,0x39|0x3a) {leaf_handle.clone()}
            else if (p.row..p.row+roof_rows).contains(&(tile.row as usize)) {
                roof_handle.clone()} else {wall_handle.clone()};}
            let mut images=TerrainImageSamples::default();
            images.pixels.insert(roof_handle.id(),TileImageSample::Rgba((0..64).flat_map(|i|[
                [57,57,57,255],[41,140,255,255],[123,255,255,255],[222,255,222,255],
            ][i%4]).collect()));
            images.pixels.insert(wall_handle.id(),TileImageSample::Rgba((0..64).flat_map(|i|
                if i%3==0 {[220,240,205,255]} else {[250,145,195,255]}).collect()));
            images.pixels.insert(light_handle.id(),TileImageSample::Rgba([245,245,80,255].repeat(64)));
            images.pixels.insert(leaf_handle.id(),TileImageSample::Rgba([100,40,95,255].repeat(64)));
            let cells=tiles.iter().collect::<Vec<_>>();
            let shapes=cells.iter().map(|tile|shape_for_source(&tile.source)).collect::<Vec<_>>();
            for map in ["CianwoodCity","OlivineCity","AzaleaTown"] {
                assert_eq!(descriptor(map,&cells,&g,p).unwrap_or_else(||panic!("invalid source fixture {kind:?} on {map}")).asset,Asset::Johto(kind));
                let mut before=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
                let mut before_claims=vec![false;cells.len()];
                assert!(append_building(&mut before,&cells,&shapes,&g,map,p,&mut before_claims,[0,0]));
                let mut after=TerrainMeshData {footing_heights:vec![0.;cells.len()],..Default::default()};
                let mut claims=vec![false;cells.len()];
                assert!(append_building_with_samples(&mut after,&cells,&shapes,&g,map,p,&mut claims,[0,0],Some(&images)));
                assert_eq!(after.solid.positions,before.solid.positions);
                assert_eq!(after.solid.normals,before.solid.normals);
                assert_eq!(after.solid.indices,before.solid.indices);
                assert_eq!(after.footing_heights,before.footing_heights);
                assert_eq!(claims,before_claims);
                assert_eq!(after.coverage_fade_ranges,vec![0..after.solid.positions.len()]);
                assert_eq!(after.depth_coverage_solid_ranges,after.coverage_fade_ranges);
                assert_eq!(before.coverage_fade_ranges,after.coverage_fade_ranges);
                assert_ne!(after.solid.colors,before.solid.colors);
                if entrance {
                    assert!(after.solid.colors.iter().any(|c|c[0]>0.7 && c[1]>0.7 && c[2]<0.15),
                        "{kind:?} must use the source entrance light on {map}");
                    if kind!=Johto::House {
                        assert!(after.solid.colors.iter().any(|c|c[0]>c[1]*4. && c[2]>c[1]*4.),
                            "{kind:?} must retain the source plum door leaf on {map}");
                    }
                }
                assert!(after.solid.uvs.iter().zip(&after.solid.colors).any(|(uv,color)|
                    crate::scenery_finish::town_paint_kind(*uv).is_some() && color[0]<color[1] && color[1]<color[2]),
                    "{kind:?} must use source cyan on {map}");
                let mut fallback=TerrainMeshData::default();
                let mut claims=vec![false;cells.len()];
                assert!(append_building_with_samples(&mut fallback,&cells,&shapes,&g,map,p,&mut claims,[0,0],Some(&TerrainImageSamples::default())));
                assert!(fallback.solid.positions.is_empty());
                assert_eq!(fallback.textured.quad_count(),p.width*p.height);
            }
        }
    }
    #[test]
    fn forest_small_headbutt_keeps_authored_plot_and_small_shared_recipe() {
        const CHILD: &str = "GEOTHITE_SHARED_FOREST_TEST_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "mesh::modeled_exteriors::tests::forest_small_headbutt_keeps_authored_plot_and_small_shared_recipe", "--nocapture"])
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
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;6],authored_cells:vec![None;6],..Default::default()};
        let mut claimed=vec![false;6];
        assert!(append_tree(&mut mesh,&cells,&shapes,&g,"IlexForest",p,&mut claimed,[0,0]));
        assert!(!mesh.solid.positions.is_empty());
        assert!(mesh.scenery.is_empty(), "forest must not load the textured sprite tree catalog");
        assert_eq!(mesh.authored_cells,vec![Some("johto/tree"),Some("johto/tree"),None,
            Some("johto/tree"),Some("johto/tree"),None]);
        assert_eq!(mesh.footing_heights,vec![0.;6]);
        assert_eq!(claimed,vec![true,true,false,true,true,false]);
        let mut expected=TerrainMeshData::default();
        crate::foliage::append_grove_tree(&mut expected,[0.,16.,0.,16.],0.,4.,[0,0],[false;4]);
        assert_eq!(mesh.solid.positions,expected.solid.positions);
        assert_eq!(mesh.solid.colors,expected.solid.colors);
        assert_eq!(mesh.solid.indices,expected.solid.indices);
        // The production deferred path must select this same shared recipe.
        let mut deferred=TerrainMeshData {footing_heights:vec![0.;6],authored_cells:vec![None;6],..Default::default()};
        let mut deferred_claimed=vec![false;6];
        let recipe=super::new_bark::claim_grove(&mut deferred,&cells,&shapes,&g,
            "IlexForest",p,&mut deferred_claimed,[0,0]).expect("shared forest grove");
        recipe.append(&mut deferred);
        assert_eq!(deferred.solid.positions,expected.solid.positions);
        assert_eq!(deferred_claimed,claimed);
        assert_eq!(deferred.authored_cells,mesh.authored_cells);
        let mut incomplete=frame.clone();
        incomplete.tiles[4].source.tile_index=0x05;
        let incomplete_cells:Vec<_>=incomplete.tiles.iter().collect();
        assert!(super::new_bark::claim_grove(&mut TerrainMeshData::default(),
            &incomplete_cells,&shapes,&g,"IlexForest",p,&mut vec![false;6],[0,0]).is_none(),
            "partial tree drawings must not acquire a shared grove claim");
    }

    #[test]
    fn large_forest_drawings_keep_ancient_tree_identity_and_full_plot() {
        let sources=(0..4).flat_map(|y| (0..5).map(move |x| {
            let mut source=super::super::tests::source_with_tile(
                if x < 4 {0x05} else {0x01},x as u8,y as u8,
                if x < 4 {0x0c + y * 0x10 + x} else {0x05});
            source.tileset_id="forest".into(); source
        })).collect();
        let frame=super::super::tests::frame(5,4,sources);
        let g=GridGeometry {width:5,height:4,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let cells:Vec<_>=frame.tiles.iter().collect();
        let shapes:Vec<_>=cells.iter().map(|t|shape_for_source(&t.source)).collect();
        let p=TreePlacement {column:0,row:0,width:4,height:4,ground_tile_index:0x05,
            ground_metatile_id:None,base_height:0.,rounded:true,outline_mask:true,
            remove_all_ground:false,card_thickness:0.};
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;20],authored_cells:vec![None;20],..Default::default()};
        let mut claimed=vec![false;20];
        assert!(append_tree(&mut mesh,&cells,&shapes,&g,"IlexForest",p,&mut claimed,[0,0]));
        let mut expected=TerrainMeshData::default();
        crate::foliage::append_family_canopy(&mut expected,[8.,24.,8.,24.],0.,8.,[0,0],[false;4],
            crate::foliage::GroveFamily::AncientForest);
        assert_eq!(mesh.solid.positions,expected.solid.positions);
        assert_eq!(mesh.textured.quad_count(),16,"the entire original plot retains ground backing");
        assert_eq!(claimed,(0..20).map(|i|i%5<4).collect::<Vec<_>>());
        assert_eq!(mesh.footing_heights,vec![0.;20]);
        assert!(mesh.scenery.is_empty());
        let mut deferred = TerrainMeshData { footing_heights: vec![0.;20],
            authored_cells: vec![None;20], ..Default::default() };
        let mut deferred_claimed = vec![false;20];
        let mut recipes = Vec::new();
        assert!(append_tree_with_groves(&mut deferred, &cells, &shapes, &g,
            "IlexForest", p, &mut deferred_claimed, [0,0], Some(&mut recipes)));
        assert_eq!(recipes.len(), 1);
        assert!(deferred.solid.positions.is_empty(), "defer the complete crown, not its source ground");
        assert!(recipes[0].coverage_fade, "forest foliage must use the same coverage reveal as Johto");
        recipes.pop().unwrap().append(&mut deferred);
        assert_eq!(deferred.solid.positions, mesh.solid.positions);
        assert_eq!(deferred.solid.normals, mesh.solid.normals);
        assert_eq!(deferred.solid.colors, mesh.solid.colors);
        assert_eq!(deferred.solid.indices, mesh.solid.indices);
        assert_eq!(deferred.solid.cutaway_ranges, mesh.solid.cutaway_ranges);
        assert_eq!(deferred.textured.positions, mesh.textured.positions);
        assert_eq!(deferred.textured.indices, mesh.textured.indices);
        assert_eq!(deferred.authored_cells, mesh.authored_cells);
        assert_eq!(deferred.footing_heights, mesh.footing_heights);
        assert_eq!(deferred_claimed, claimed);
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
    fn kanto_underground_entrance_recognizes_its_complete_doorway_facade() {
        for (roof,facade,door) in [(0x38,0x10,5.),(0x0c,0x10,5.),(0x38,0x3c,3.)] {
            let (mut tiles,g,p,images)=kanto_house::tall_fixture(roof,facade);
            let cells=tiles.iter().collect::<Vec<_>>();
            assert!(outdoor_building_placements(&cells,&g).contains(&p));
            for map in ["Route8","PalletTown","UnseenTown"] {
                let d=descriptor(map,&cells,&g,p).unwrap();assert_eq!(d.asset,Asset::KantoHouse);assert_eq!(d.door,Some(door));
            }
            let shapes=vec![CellShape::Flat;cells.len()];
            let mut mesh=TerrainMeshData{authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut claims=vec![false;cells.len()];
            assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"UnseenTown",p,&mut claims,[0,0],Some(&images)));
            assert!(mesh.authored_cells[..64].iter().all(|&x|x==Some("outdoor/kanto_house")));
            assert!(claims[..64].iter().all(|&x|x));assert!(claims[64..].iter().all(|&x|!x));
            assert!(mesh.footing_heights.iter().all(|&h|h==0.));assert_eq!(mesh.rigid_buildings.len(),1);
            drop(cells);tiles[g.width*4].source.metatile_id^=1;
            assert!(descriptor("UnseenTown",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none());
        }
        let (mut tiles,g,p,images)=kanto_house::tall_fixture(0x38,0x10);
        for y in 4..8 {for x in 4..8 {
            let s=&mut tiles[y*g.width+x].source;s.metatile_id=0x12;
            if s.tile_index==0x32 {s.tile_index=0x0a;}
        }}
        let cells=tiles.iter().collect::<Vec<_>>();
        assert!(outdoor_building_placements(&cells,&g).contains(&p));
        for map in ["Route5","UnseenTown"] {
            let d=descriptor(map,&cells,&g,p).unwrap();
            assert_eq!(d.asset,Asset::KantoHouse);assert_eq!(d.door,Some(5.));
        }
        assert!(kanto_house::paint(&cells,&g,p,&images).is_some());
        drop(cells);tiles[4*g.width+4].source.tile_index=0x32;
        assert!(descriptor("UnseenTown",&tiles.iter().collect::<Vec<_>>(),&g,p).is_none(),
            "a changed complete drawing cannot borrow the new variant");
    }

    #[test]
    #[ignore = "requires a completed external CRYSTAL_RENDER_TEST_PACK"]
    fn external_route5_entrance_uses_the_shared_model_with_native_pixels() {
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(
            std::env::var("CRYSTAL_RENDER_TEST_PACK").unwrap()).unwrap();
        let pack=loaded.pack();let data=&pack.data().maps["Route5"];
        let art=&pack.runtime_files()["data/tilesets/kanto_metatiles.bin"];
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/kanto.png"])
            .unwrap().to_luma8();
        let (w,h)=(data.attributes.width as usize*4,data.attributes.height as usize*4);
        let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        let mut assets=Assets::<Image>::default();let mut images=TerrainImageSamples::default();
        let tiles=(0..w*h).map(|i| {
            let (x,y)=(i%w,i/w);let b=data.blocks[y/4*(w/4)+x/4];
            let tile=art[b as usize*16+y%4*4+x%4] as u16;
            let texture=assets.add(Image::default());
            let columns=png.width()/8;
            let rgba=(0..64).flat_map(|j| {
                let v=png.get_pixel((u32::from(tile)%columns)*8+j%8,
                    (u32::from(tile)/columns)*8+j/8)[0];[v,v,v,255]
            }).collect();
            images.pixels.insert(texture.id(),TileImageSample::Rgba(rgba));
            VisualTile{column:x as u32,row:y as u32,texture,priority:false,animation_frames:None,
                source:crystal_render_api::VisualTileSource{tileset_id:"kanto".into(),metatile_id:b,
                    subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,tile_index:tile}}
        }).collect::<Vec<_>>();
        let cells=tiles.iter().collect::<Vec<_>>();
        let placements=outdoor_building_placements(&cells,&g);
        let p=placements.iter().copied().find(|p|
            cells[p.row*w+p.column].source.metatile_id==0x38
                && cells[(p.row+4)*w+p.column+4].source.metatile_id==0x12)
            .expect("Route5 blue-roof bright-window entrance drawing");
        assert!(kanto_house::matches(&cells,&g,p),"{p:?}: complete native entrance rejected");
        assert_eq!(descriptor("Route5",&cells,&g,p).unwrap().asset,Asset::KantoHouse);
        for y in 0..8 {for x in 0..8 {
            let tile=cells[(p.row+y)*w+p.column+x];let rgba=tile_rgba(&images,tile).unwrap();
            if !matches!(tile.source.tile_index,0x4b|0x17|0x1a|0x12|0x22|0x0a|0x32|0x0b|0x1b) {continue;}
            let mut roles=[None;4];
            for (j,pixel) in rgba.chunks_exact(4).enumerate() {
                let role=kanto_craft::source_ink(tile.source.tile_index,j%8,j/8);
                assert!(roles[role].is_none_or(|old|old==pixel[0]),
                    "native tile {:02x} at ({x},{y}) disagrees with material role {role}, pixel {j}",tile.source.tile_index);
                roles[role]=Some(pixel[0]);
            }
        }}
        assert!(kanto_house::paint(&cells,&g,p,&images).is_some(),"native entrance paint rejected");
        let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData{authored_cells:vec![None;w*h],footing_heights:vec![0.;w*h],..Default::default()};
        let mut claims=vec![false;w*h];
        assert!(append_building_with_samples(&mut mesh,&cells,&shapes,&g,"Route5",p,
            &mut claims,[0,0],Some(&images)));
        assert_eq!(mesh.rigid_buildings.len(),1);
        assert!(mesh.solid.positions.iter().any(|p|p[1]>32.));
        assert!(mesh.footing_heights.iter().all(|h|*h==0.));
    }

    #[test]
    fn lighthouse_keeps_a_circular_plan_even_with_an_off_center_native_door() {
        let (mut tiles,g,p)=drawing_cells("johto",&[&[0x08,0x09],&[0x7e,0x7f],
            &[0x13,0x0f],&[0x13,0x0f],&[0x13,0x0f],&[0x13,0x0f],&[0x1a,0x11]]);
        for tile in &mut tiles {tile.source.tile_index=6;}
        for (dy,row) in [[0x37,0x38],[0x39,0x3a]].iter().enumerate() {
            for dx in 0..2 {tiles[(26+dy)*8+2+dx].source.tile_index=row[dx];}
        }
        let cells=tiles.iter().collect::<Vec<_>>();
        let d=descriptor("OlivineCity",&cells,&g,p).unwrap();
        assert_eq!(d.asset,Asset::World(Kind::Lighthouse));assert_eq!(d.depth_cells,Some(8.));
        assert_eq!(d.door,Some(3.));
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;224],authored_cells:vec![None;224],..Default::default()};
        let mut claimed=vec![false;224];let shapes=vec![CellShape::Flat;224];
        assert!(append_building(&mut mesh,&cells,&shapes,&g,"OlivineCity",p,&mut claimed,[0,0]));
        let m=model(Kind::Lighthouse);
        let mut points=Vec::new();let mut outline=std::collections::BTreeSet::new();
        for (i,p) in m.surface.positions.iter().enumerate() {
            if m.vertex_material(i)!="plaster_light" || (p[1]-0.28).abs()>0.0001 {continue;}
            let normal=m.surface.normals[i];
            if normal[1].abs()<0.25 {
                let direction=Vec2::new(p[0],p[2]).normalize();
                assert!(direction.dot(Vec2::new(normal[0],normal[2]).normalize())>0.9999,
                    "curved shaft normals follow the circular section");
            }
            let q=mesh.solid.positions[i];points.push(Vec2::new(q[0],q[2]));outline.insert([(q[0]*1000.).round()as i32,(q[2]*1000.).round()as i32]);
        }
        assert!(outline.len()>=32,"the shaft has a smooth circular outline");
        let lo=points.iter().copied().fold(Vec2::splat(f32::INFINITY),Vec2::min);
        let hi=points.iter().copied().fold(Vec2::splat(f32::NEG_INFINITY),Vec2::max);
        let center=(lo+hi)*0.5;assert!((center.x-24.).abs()<0.001);
        let radii=points.iter().map(|p|p.distance(center)).collect::<Vec<_>>();
        let min=radii.iter().copied().fold(f32::INFINITY,f32::min);
        let max=radii.iter().copied().fold(f32::NEG_INFINITY,f32::max);
        assert!(min>18. && max-min<0.001,"round shaft must survive production fitting: {min}..{max}");
        assert_eq!(mesh.rigid_buildings.last().unwrap().doorway,Vec2::new(24.,220.));
        assert_eq!(mesh.footing_heights,vec![0.;224]);assert!(claimed.iter().all(|v|*v));
    }

    #[test]
    fn mahogany_connection_house_recognizes_its_complete_roof_and_facade() {
        let (mut tiles, g, p) = drawing_cells("johto", &[&[0x08, 0x09], &[0x16, 0x1e]]);
        let d = descriptor("MahoganyTown", &tiles.iter().collect::<Vec<_>>(), &g, p).unwrap();
        assert_eq!(d.asset, Asset::Johto(Johto::PlayerHouse));
        assert!(!d.east);
        tiles[0].source.metatile_id = 0x18;
        assert!(descriptor("MahoganyTown", &tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
    }

    #[test]
    fn east_gate_keeps_side_entry_and_rejects_a_broken_native_phase() {
        let (mut tiles, g, p) = drawing_cells("johto", &[&[0x08, 0x09], &[0x10, 0x11]]);
        let d = descriptor("Route31", &tiles.iter().collect::<Vec<_>>(), &g, p).unwrap();
        assert_eq!(d.asset, Asset::World(Kind::EastGate));
        assert!(d.east);
        assert_eq!(d.north_aperture,None);
        assert_eq!(d.door, Some(6.0));
        tiles[19].source.subtile_row = 3;
        assert!(descriptor("Route31", &tiles.iter().collect::<Vec<_>>(), &g, p).is_none());
    }
    #[test]
    fn north_gate_uses_complete_port_approach_and_keeps_support_and_claims() {
        let (mut tiles,g,mut p)=drawing_cells("johto",&[&[0x0c,0x0d],&[0x18,0x19],&[0x10,0x11]]);
        for y in 0..4 {for x in 0..8 {
            tiles[y*8+x].source.tile_index=if y<2 {6}else if x<2 || x>=6 {
                if y==2 {0x5a}else{0x59}
            }else{0x40};
        }}
        p.row=4;p.height=8;
        let cells=tiles.iter().collect::<Vec<_>>();
        let d=descriptor("OlivineCity",&cells,&g,p).unwrap();
        assert_eq!(d.asset,Asset::World(Kind::EastGate));
        assert!(!d.east);assert_eq!(d.door,Some(4.));assert_eq!(d.north_aperture,Some([2.,6.]));
        assert_eq!(descriptor("UnseenTown",&cells,&g,p).unwrap().north_aperture,Some([2.,6.]));
        let shapes=cells.iter().map(|t|shape_for_source(&t.source)).collect::<Vec<_>>();
        let mut mesh=TerrainMeshData{footing_heights:vec![0.25;96],authored_cells:vec![None;96],..Default::default()};
        let mut claimed=vec![false;96];
        assert!(append_building(&mut mesh,&cells,&shapes,&g,"OlivineCity",p,&mut claimed,[0,0]));
        assert_eq!(mesh.footing_heights,vec![0.25;96]);
        assert!(claimed[..32].iter().all(|v|!*v));
        assert!(claimed[32..].iter().all(|v|*v));
        assert_eq!(mesh.rigid_buildings.last().unwrap().doorway,Vec2::new(32.,36.));
        tiles[0].source.tile_index^=1;
        let d=descriptor("OlivineCity",&tiles.iter().collect::<Vec<_>>(),&g,p).unwrap();
        assert!(d.east);assert_eq!(d.north_aperture,None);
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
    fn forest_gate_roof_follows_live_source_without_changing_its_door_or_plot() {
        for last in [0x24,0x25] {
            let (mut tiles,g,mut p)=drawing_cells("forest",
                &[&[0x1d,0x1e,0x1f],&[0x21,0x22,0x23],&[last,0x26,0x27]]);
            p.ground_tile_index=5;tiles.last_mut().unwrap().source.tile_index=5;
            let wall=6*g.width+2;let window=7*g.width+2;let door=10*g.width+2;
            for (index,tile_id,handle_id) in [(wall,0x24,8201),(window,0x04,8202),(door,0x17,8203)] {
                if index==door && last==0x25 {continue;}
                tiles[index].source.tile_index=tile_id;
                tiles[index].texture=Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(handle_id),std::marker::PhantomData);
            }
            let cells=tiles.iter().collect::<Vec<_>>();let shapes=vec![CellShape::Flat;cells.len()];
            let desc=descriptor("IlexForest",&cells,&g,p).unwrap();
            let Asset::World(kind)=desc.asset else {panic!("forest gate source binding");};
            let mut images=TerrainImageSamples::default();
            let colors=[[10,5,25,255],[70,40,120,255],[130,80,180,255]];
            images.pixels.insert(tiles[0].texture.id(),TileImageSample::Rgba(
                (0..64).flat_map(|i|colors[i%3]).collect()));
            images.pixels.insert(tiles[wall].texture.id(),TileImageSample::Rgba(
                [[35,55,105,255],[110,150,210,255]].repeat(32).into_iter().flatten().collect()));
            images.pixels.insert(tiles[window].texture.id(),TileImageSample::Rgba(
                [[0,0,0,255],[70,60,90,255],[140,130,160,255],[245,220,60,255]]
                    .repeat(16).into_iter().flatten().collect()));
            if kind==Kind::ForestGate {
                images.pixels.insert(tiles[door].texture.id(),TileImageSample::Rgba(
                    [[0,0,0,255],[110,40,70,255],[140,80,120,255],[190,130,170,255]]
                        .repeat(16).into_iter().flatten().collect()));
            }
            let mut before=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
            let mut after=before.clone();
            let mut claimed_before=vec![false;cells.len()];let mut claimed_after=claimed_before.clone();
            assert!(append_building_with_samples(&mut before,&cells,&shapes,&g,"IlexForest",p,&mut claimed_before,[0,0],None));
            assert!(append_building_with_samples(&mut after,&cells,&shapes,&g,"IlexForest",p,&mut claimed_after,[0,0],Some(&images)));
            assert_eq!(before.solid.positions,after.solid.positions);
            assert_eq!(before.solid.normals,after.solid.normals);
            assert_eq!(before.solid.indices,after.solid.indices);
            assert_eq!(before.footing_heights,after.footing_heights);
            assert_eq!(claimed_before,claimed_after);
            assert_eq!(after.coverage_fade_ranges, vec![0..after.solid.positions.len()]);
            let m=model(kind);
            for (i,&color) in after.solid.colors.iter().enumerate() {
                let material=m.vertex_material(i);
                if material.starts_with("roof_") {
                    assert_eq!(after.solid.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                } else if material=="yellow" {
                    assert_eq!(color,before.solid.colors[i],"unrelated hardware keeps authored paint");
                } else {
                    if matches!(material,"glass"|"glass_light"|"glass_dark") {
                        assert!(crate::city_lights::source_window_tag(after.solid.uvs[i]));
                    } else {
                        assert_eq!(after.solid.uvs[i],crate::scenery_finish::SOURCE_PAINT_UV);
                    }
                    if material=="plaster" {assert!(color[2]>color[0]*3.,"live blue masonry");}
                    if material=="glass" {assert!(color[0]>color[2]*3.,"separate yellow window palette");}
                    if material=="forest_gate_door" {assert!(color[0]>color[2]*2.,"separate live door pigment");}
                }
            }
            if kind==Kind::ForestGate {
                assert!((0..after.solid.positions.len()).any(|i|m.vertex_material(i)=="forest_gate_door"));
            }
            let mut missing_samples=vec![None,Some(wall),Some(window)];
            if kind==Kind::ForestGate {missing_samples.push(Some(door));}
            for missing in missing_samples {
                let mut incomplete=images.clone();
                if let Some(index)=missing {incomplete.pixels.remove(&tiles[index].texture.id());}
                else {incomplete=TerrainImageSamples::default();}
                let mut fallback=TerrainMeshData {authored_cells:vec![None;cells.len()],footing_heights:vec![0.;cells.len()],..Default::default()};
                assert!(append_building_with_samples(&mut fallback,&cells,&shapes,&g,"IlexForest",p,&mut vec![false;cells.len()],[0,0],Some(&incomplete)));
                assert!(fallback.solid.positions.is_empty());
                assert!(fallback.coverage_fade_ranges.is_empty());
                assert!(!fallback.textured.positions.is_empty());
            }
        }
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
    fn johto_structure_underlays_prefer_level_paint_and_receive_the_shared_earth_finish() {
        for family in ["johto", "johto_modern"] {
            for paint in [0x05,0x06] {
                let mut tiles=[tile(family,0x50,paint),tile(family,0x01,paint),
                    tile(family,0x18,0x80),tile("different_atlas",0x01,paint)];
                for (i,t) in tiles.iter_mut().enumerate() {t.column=i as u32;}
                let cells=tiles.iter().collect::<Vec<_>>();
                let shapes=[CellShape::RaisedTop{height:6.,solid:SolidKind::Bank},
                    CellShape::Flat,CellShape::Flat,CellShape::Flat];
                let sample=structure_ground(&cells,&shapes,family,paint).unwrap();
                assert_eq!(sample,1,"a remote bank must not raise a structure's backing");
                assert_eq!(ground(&cells,&shapes,family,paint,None),Some(0),
                    "tree/root sample selection is unchanged");
                let g=GridGeometry{width:4,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
                let mut mesh=TerrainMeshData{footing_heights:vec![0.;4],
                    authored_cells:vec![None,None,Some("johto/signboard"),None],..Default::default()};
                let claims_before=mesh.authored_cells.clone();
                let mut claims=vec![false;4];
                floor(&mut mesh,&cells,&shapes,&g,[2,0,1,1],sample,&mut claims);
                assert!(mesh.textured.positions.iter().all(|p|p[1]==0.));
                super::super::new_bark::polish_world_surfaces(&mut mesh,"MahoganyTown",&cells,&g,[0,0]);
                assert!(mesh.textured.positions.is_empty(),"no raw pixel card remains under the model");
                assert!(!mesh.solid.positions.is_empty());
                assert_eq!(mesh.footing_heights,vec![0.;4]);
                assert_eq!(mesh.authored_cells,claims_before);
                assert_eq!(claims,[false,false,true,false]);
                assert_eq!(structure_ground(&cells[3..],&shapes[3..],family,paint),None,
                    "a connected foreign atlas cannot supply floor paint");
            }
        }
    }

    #[test]
    fn kanto_ground_prefers_flat_lawn_over_the_same_tile_on_a_remote_bank() {
        let bank=tile("kanto",0x07,0x2c);let lawn=tile("kanto",0x0a,0x2c);
        let shapes=[CellShape::RaisedTop{height:6.,solid:SolidKind::Bank},CellShape::Flat];
        assert_eq!(ground(&[&bank,&lawn],&shapes,"kanto",0x2c,None),Some(1));
        assert_eq!(ground(&[&bank,&lawn],&shapes,"kanto",0x2c,Some(0x07)),Some(0));
        assert_eq!(ground(&[&bank],&shapes[..1],"kanto",0x2c,None),Some(0));
    }

    #[test]
    fn kanto_building_underlay_borrows_bank_paint_without_borrowing_remote_elevation() {
        let g=GridGeometry{width:2,height:1,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
        for (family,shape,expected) in [
            ("kanto",CellShape::RaisedTop{height:6.,solid:SolidKind::Bank},0.),
            ("johto",CellShape::RaisedTop{height:6.,solid:SolidKind::Bank},6.),
            ("kanto",CellShape::Water,CellShape::Water.surface_height(8.))] {
            let a=tile(family,0x07,0x2c);let b=tile(family,0x02,0x05);
            let mut mesh=TerrainMeshData{footing_heights:vec![0.;2],..Default::default()};
            let mut claims=vec![false;2];floor(&mut mesh,&[&a,&b],&[shape,CellShape::Flat],&g,[1,0,1,1],0,&mut claims);
            assert!(mesh.textured.positions.iter().all(|p|(p[1]-expected).abs()<0.001));
            assert_eq!(mesh.footing_heights,vec![0.;2]);assert_eq!(claims,[false,true]);
            assert_eq!(mesh.textured.uvs,vec![[0.,0.],[0.,1.],[0.5,1.],[0.5,0.]]);
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
