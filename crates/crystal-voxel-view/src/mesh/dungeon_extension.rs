//! Original full-volume special-environment fixtures, source-driven and render-only.
//! Puzzle lettering remains the actual live atlas on stone; no glyph catalog is
//! copied here and no model lookup is based on gameplay collision permissions.
use super::*;
use crate::dungeon_models::{ExtensionKind as Asset, extension_model};

#[derive(Clone, Copy, Debug)]
pub(super) enum Floor {
    Stone,
    Wood,
    Ceramic,
    Carpet,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Detail {
    Model(Asset),
    GlyphFrieze,
    PuzzleDais,
    FloorGlyph,
    TeleportPad,
    LeagueBarrier,
    Floor(Floor),
    PassageNetwork { open: [bool; 4], basement: bool },
    CeruleanBridgeDeck,
    CeruleanBridgeFace,
    StoreBasementLift,
    StoreBasementStair,
    SproutPillar,
    SproutAltar,
    VioletWall,
    VermilionPanel,
    MahoganyWall,
    MahoganyPartition,
    BlackthornStairs,
    BlackthornCrossing,
    BlackthornChannel { lava:bool },
    FuchsiaWall,
    ShipCompound,
    RoofSlope { west: f32, east: f32 },
    RoofRidge,
}
impl Detail {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Model(a) => a.label(),
            Self::GlyphFrieze => "special:live-glyph-stone-frieze",
            Self::PuzzleDais => "special:live-puzzle-dais",
            Self::FloorGlyph => "special:live-inscription-paving",
            Self::TeleportPad => "special:live-teleport-inlay",
            Self::LeagueBarrier => "special:league-balustrade",
            Self::Floor(Floor::Stone) => "special:stone-paving",
            Self::Floor(Floor::Wood) => "special:timber-deck",
            Self::Floor(Floor::Ceramic) => "special:ceramic-paving",
            Self::Floor(Floor::Carpet) => "special:champion-carpet",
            Self::PassageNetwork { .. } => "special:connected-passage-walls",
            Self::CeruleanBridgeDeck => "special:cerulean-brick-bridge",
            Self::CeruleanBridgeFace => "special:cerulean-wood-bridge-face",
            Self::StoreBasementLift => "special:store-basement-lift-front",
            Self::StoreBasementStair => "special:store-basement-warehouse-stair",
            Self::SproutPillar => "special:sprout-swaying-pillar",
            Self::SproutAltar => "special:sprout-framed-altar",
            Self::VioletWall => "special:violet-north-wall",
            Self::VermilionPanel => "special:vermilion-north-panel",
            Self::MahoganyWall => "special:mahogany-panel-wall",
            Self::MahoganyPartition => "special:mahogany-stone-partition",
            Self::BlackthornStairs => "special:blackthorn-stair-flight",
            Self::BlackthornCrossing => "special:blackthorn-filled-rock-crossing",
            Self::BlackthornChannel {lava:true} => "special:blackthorn-lava-channel",
            Self::BlackthornChannel {lava:false} => "special:blackthorn-channel-lip",
            Self::FuchsiaWall => "special:fuchsia-north-wall",
            Self::ShipCompound => "special:ship-corridor-bulkhead",
            Self::RoofSlope { .. } => "special:tower-pitched-tile-roof",
            Self::RoofRidge => "special:tower-ridge-tiles",
        }
    }
}
fn word_room(ts: &str) -> bool {
    matches!(
        ts,
        "kabuto_word_room" | "omanyte_word_room" | "aerodactyl_word_room" | "ho_oh_word_room"
    )
}
fn ruins(ts: &str) -> bool {
    ts == "ruins_of_alph" || word_room(ts)
}
fn league(map: &str) -> bool {
    matches!(
        map,
        "WillsRoom" | "KogasRoom" | "BrunosRoom" | "KarensRoom" | "LancesRoom"
    )
}
fn passage(map: &str) -> bool {
    matches!(
        map,
        "UndergroundPath" | "SaffronGym" | "OlivinePortPassage" | "VermilionPortPassage"
    )
}
// Tile identities for repeated cap, post, floor and opening quadrants. These
// let a cropped known block reconstruct only unseen identities for its guard;
// every visible identity is still checked against the complete fingerprint.
fn basement_wall_tile(block: u16, column: usize, row: usize) -> u16 {
    let quadrants = match block {
        0x08 => [0, 1, 2, 3], 0x09 => [4, 4, 3, 3],
        0x0a => [5, 0, 3, 2], 0x0c => [2, 3, 2, 3],
        0x0e => [3, 2, 3, 2], 0x10 => [2, 3, 6, 4],
        0x11 => [3, 3, 4, 4], 0x12 => [3, 2, 4, 6],
        0x13 => [7, 7, 3, 3], 0x18 => [3, 0, 3, 2],
        0x19 => [0, 3, 2, 3], 0x1a => [3, 2, 3, 6],
        0x1b => [2, 3, 6, 3], 0x1c => [3, 1, 3, 3],
        0x1d => [4, 4, 3, 3], 0x1f => [6, 4, 3, 3],
        0x25 => [5, 8, 3, 3], 0x26 => [8, 1, 3, 3],
        0x2f => [4, 4, 9, 9], _ => unreachable!("guarded basement block"),
    };
    const DRAWINGS: [[u16; 4]; 10] = [
        [0x03, 0x05, 0x0c, 0x0d], [0x03, 0x04, 0x13, 0x14],
        [0x0c, 0x0d, 0x0c, 0x0d], [0x10; 4],
        [0x04, 0x04, 0x14, 0x14], [0x04, 0x05, 0x14, 0x15],
        [0x03, 0x05, 0x13, 0x15], [0x03, 0x05, 0x03, 0x05],
        [0x02, 0x02, 0x12, 0x12], [0x01; 4],
    ];
    DRAWINGS[quadrants[row / 2 * 2 + column / 2]][row % 2 * 2 + column % 2]
}
fn basement_wall_block(cells: &[&VisualTile], g: &GridGeometry, i: usize) -> bool {
    if g.width == 0 || cells.len() != g.width * g.height || i >= cells.len() { return false; }
    let source = &cells[i].source;
    let expected: u64 = match source.metatile_id {
        0x08 => 0x6fa8327d13468ab2,
        0x09 => 0x367ae28b0ad0a1dd,
        0x0a => 0x0d029a27b84a3a92,
        0x0c => 0x833e398853a4267d,
        0x0e => 0xf3a7824ec4e2284d,
        0x10 => 0xdaccc89a0cade7b5,
        0x11 => 0xd9d7b1dcac07993d,
        0x12 => 0x9fbbf7d3094d8015,
        0x13 => 0x840cf6880bdd5095,
        0x18 => 0x82c85b87fb80e81a,
        0x19 => 0x090d6e51c93a1dea,
        0x1a => 0x0cb5174b88675be5,
        0x1b => 0xbdc33fb1cd887f85,
        0x1c => 0x85b0eb5aa76005ed,
        0x1d => 0x5c417647f699abcd,
        0x1f => 0x71e8c598be4cbd7d,
        0x25 => 0x580059b1bf04a041,
        0x26 => 0xfa319c05ebc64319,
        0x2f => 0x6f04d84b32af815d,
        _ => return false,
    };
    if source.subtile_column >= 4 || source.subtile_row >= 4 { return false; }
    let x = (i % g.width) as isize - source.subtile_column as isize;
    let y = (i / g.width) as isize - source.subtile_row as isize;
    let mut hash = 0xcbf29ce484222325u64;
    for dy in 0..4 { for dx in 0..4 {
        let (cx, cy) = (x + dx as isize, y + dy as isize);
        let (block, column, row, tile) = if cx >= 0 && cy >= 0 && cx < g.width as isize && cy < g.height as isize {
            let s = &cells[cy as usize * g.width + cx as usize].source;
            if s.tileset_id.as_ref() != "underground" { return false; }
            (s.metatile_id, s.subtile_column, s.subtile_row, s.tile_index)
        } else {
            (source.metatile_id, dx as u8, dy as u8, basement_wall_tile(source.metatile_id, dx, dy))
        };
        for b in [block as u8, (block >> 8) as u8, column, row, tile as u8, (tile >> 8) as u8] {
            hash = (hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }}
    hash == expected
}
fn cerulean_bridge_detail(s: &VisualTileSource) -> Option<(Detail, u16)> {
    if s.tileset_id.as_ref() != "port" || s.subtile_column >= 4 || s.subtile_row >= 4 { return None; }
    // The gray perimeter/lobby is 0x0a. The pink brick walkway is 0x05,
    // including the floor beside the two statues and above the entry carpet.
    if s.tile_index == 0x05 && matches!(s.metatile_id,
        0x0c | 0x14 | 0x26 | 0x27 | 0x29 | 0x2d | 0x32 | 0x33 | 0x34 | 0x39) {
        Some((Detail::CeruleanBridgeDeck, 0x14))
    } else if s.tile_index == 0x0b && s.subtile_row == 0 && matches!(s.metatile_id, 0x36..=0x39) {
        Some((Detail::CeruleanBridgeFace, 0x14))
    } else { None }
}
fn cerulean_water_source(s: &VisualTileSource) -> bool {
    if s.tileset_id.as_ref() != "port" || s.tile_index != 0x14
        || s.subtile_column >= 4 || s.subtile_row >= 4 { return false; }
    let mask: u16 = match s.metatile_id {
        0x08 | 0x36 => 0xfff0,
        0x0a => 0xffff,
        0x0c => 0x00ff,
        0x14 | 0x38 => 0x3330,
        0x27 => 0xf000,
        0x29 => 0x3000,
        0x2d => 0x0033,
        0x2e => 0xcccc,
        0x2f => 0x3333,
        0x34 | 0x37 => 0xccc0,
        0x39 => 0x00f0,
        _ => return false,
    };
    mask & (1 << (s.subtile_row * 4 + s.subtile_column)) != 0
}
fn harbor(map: &str) -> bool {
    matches!(map, "OlivinePort" | "VermilionPort")
}
fn groups(
    r: &mut Resolver<'_>,
    parts: Vec<TreePlacement>,
    detail: Detail,
    floor: u16,
    rise: f32,
    depth: f32,
) {
    for p in parts {
        if !phase_consistent(r.cells, r.g, p) {
            continue;
        }
        let ts = r.cells[p.row * r.g.width + p.column]
            .source
            .tileset_id
            .as_ref();
        let Some(ground) = ground(r.map, r.cells, ts, floor) else {
            continue;
        };
        r.add(Placement {
            column: p.column,
            row: p.row,
            width: p.width,
            height: p.height,
            ground,
            form: Form::Special(detail),
            rise_pixels: rise,
            depth_pixels: depth,
            front_rows: p.height as f32,
        });
    }
}
fn matching_rect(
    cells: &[&VisualTile],
    g: &GridGeometry,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    f: impl Fn(&VisualTileSource, usize, usize) -> bool,
) -> bool {
    x + w <= g.width
        && y + h <= g.height
        && (0..h).all(|dy| (0..w).all(|dx| f(&cells[(y + dy) * g.width + x + dx].source, dx, dy)))
}
fn add_rect(
    r: &mut Resolver<'_>,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    ground: usize,
    detail: Detail,
    rise: f32,
    depth: f32,
) {
    r.add(Placement {
        column: x,
        row: y,
        width: w,
        height: h,
        ground,
        form: Form::Special(detail),
        rise_pixels: rise,
        depth_pixels: depth,
        front_rows: h as f32,
    });
}

// These are complete native architectural drawings, not arbitrary wall tiles.
fn violet_wall_local(s: &VisualTileSource) -> Option<(u8,u8,usize,usize)> {
    (s.tileset_id.as_ref()=="elite_four_room" && s.metatile_id==0x15
        && s.subtile_column<4 && s.subtile_row<2 && s.tile_index==0x10)
        .then_some((s.subtile_column,s.subtile_row,4,2))
}
fn sprout_altar_local(s: &VisualTileSource) -> Option<(u8,u8,usize,usize)> {
    if s.tileset_id.as_ref()!="tower" || s.subtile_row>=2 {return None;}
    let x=match (s.metatile_id,s.subtile_column) {
        (0x2e,2..=3)=>s.subtile_column-2,
        (0x2f,0..=1)=>s.subtile_column+2,
        _=>return None,
    };
    let art=[[0x11,0x48,0x49,0x11],[0x21,0x58,0x59,0x21]];
    (s.tile_index==art[s.subtile_row as usize][x as usize])
        .then_some((x,s.subtile_row,4,2))
}

pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    let (map, cells, g) = (r.map, r.cells, r.g);
    let Some(first) = cells.first() else {
        return;
    };
    let ts = first.source.tileset_id.as_ref();
    if map=="VioletGym" {
        let walls=grouped_flat_card_placements(cells,g,0x01,false,violet_wall_local);
        groups(r,walls,Detail::VioletWall,0x01,16.,4.);
    }
    if map=="SproutTower3F" {
        let altar=grouped_flat_card_placements(cells,g,0x02,false,sprout_altar_local);
        groups(r,altar,Detail::SproutAltar,0x02,16.,4.);
    }
    if map=="FuchsiaGym" {
        let walls=grouped_flat_card_placements(cells,g,0x10,false,crate::fuchsia_gym::north_wall_local);
        groups(r,walls,Detail::FuchsiaWall,0x10,16.,4.);
    }
    if map == "BlackthornGym1F" {
        let stairs = grouped_flat_card_placements(cells,g,0x01,false,
            |s|crate::elite_four_room::blackthorn_fixture_local(s,0));
        groups(r,stairs,Detail::BlackthornStairs,0x01,8.,16.);
        let rocks = grouped_flat_card_placements(cells,g,0x01,false,
            |s|crate::elite_four_room::blackthorn_fixture_local(s,1));
        r.groups(Kind::CaveBoulder,0x01,12.,12.,rocks);
        let crossings = grouped_flat_card_placements(cells,g,0x01,false,
            |s|crate::elite_four_room::blackthorn_fixture_local(s,2));
        groups(r,crossings,Detail::BlackthornCrossing,0x01,0.,16.);
        for (i,tile) in cells.iter().enumerate() {
            if let Some(lava)=crate::elite_four_room::blackthorn_channel_source(&tile.source) {
                add_rect(r,i%g.width,i/g.width,1,1,i,Detail::BlackthornChannel {lava},0.,8.);
            }
        }
    }
    if map == "MahoganyGym" {
        let walls = grouped_flat_card_placements(cells, g, 0x01, false,
            crate::elite_four_room::mahogany_wall_local);
        groups(r, walls, Detail::MahoganyWall, 0x01, 24., 8.);
        let partitions = grouped_flat_card_placements(cells, g, 0x01, false,
            crate::elite_four_room::mahogany_partition_local);
        groups(r, partitions, Detail::MahoganyPartition, 0x01, 13., 16.);
    }
    if map == "VermilionGym" {
        let panels = grouped_flat_card_placements(cells, g, 0x01, false, crate::vermilion::panel_local);
        groups(r, panels, Detail::VermilionPanel, 0x01, 32., 6.);
    }
    if matches!(map, "SproutTower1F" | "SproutTower2F" | "SproutTower3F") {
        let pillars = grouped_flat_card_placements(cells, g, 0x02, false,
            super::super::sprout_pillar::source_local);
        groups(r, pillars, Detail::SproutPillar, 0x02, 48., 16.);
    }
    if ruins(ts) {
        // Repeated carved guardians flank puzzle consoles as well as the
        // central chamber. Exact art bands, phase and map atlas stay required.
        if ts == "ruins_of_alph" {
            let statues = grouped_flat_card_placements(cells, g, 0x02, false, |s| {
                if s.tileset_id.as_ref() != ts
                    || !matches!(s.metatile_id, 0x01 | 0x02 | 0x18 | 0x19 | 0x34 | 0x35)
                {
                    return None;
                }
                match s.tile_index {
                    0x0e => Some((0, 0, 2, 4)),
                    0x0f => Some((1, 0, 2, 4)),
                    0x1e => Some((0, 1, 2, 4)),
                    0x1f => Some((1, 1, 2, 4)),
                    0x2e => Some((0, 2, 2, 4)),
                    0x2f => Some((1, 2, 2, 4)),
                    0x3e => Some((0, 3, 2, 4)),
                    0x3f => Some((1, 3, 2, 4)),
                    _ => None,
                }
            });
            r.groups(Kind::AlphGuardian, 0x02, 28., 10., statues);
            let panels = grouped_flat_card_placements(cells, g, 0x02, false, |s| {
                if s.tileset_id.as_ref() != ts || s.subtile_row >= 2 {
                    return None;
                }
                let x = match s.metatile_id {
                    0x01 | 0x18 if s.subtile_column >= 2 => s.subtile_column - 2,
                    0x02 | 0x19 if s.subtile_column < 2 => s.subtile_column,
                    _ => return None,
                };
                let art = if matches!(s.metatile_id, 0x01 | 0x18) {
                    [[0x06, 0x06], [0x47, 0x48]]
                } else {
                    [[0x06, 0x06], [0x49, 0x4a]]
                };
                (s.tile_index == art[s.subtile_row as usize][x as usize]).then_some((
                    x,
                    s.subtile_row,
                    2,
                    2,
                ))
            });
            groups(r, panels, Detail::PuzzleDais, 0x02, 10., 12.);
        }
        if let Some(floor) = ground(map, cells, ts, 0x02) {
            // A repeated cornice tile immediately over an actual glyph is a
            // complete two-course frieze. Copy no letters: the live source cell
            // is later sampled at its native 8px size on the recessed face.
            for y in 0..g.height.saturating_sub(1) {
                let mut x = 0;
                while x < g.width {
                    let s = &cells[y * g.width + x].source;
                    let valid = |xx: usize| {
                        let a = &cells[y * g.width + xx].source;
                        let b = &cells[(y + 1) * g.width + xx].source;
                        a.tileset_id.as_ref() == ts
                            && b.tileset_id == a.tileset_id
                            && a.tile_index == 0x06
                            && a.subtile_row % 2 == 0
                            && b.subtile_row == a.subtile_row + 1
                            && b.subtile_column == a.subtile_column
                            && b.metatile_id == a.metatile_id
                            && matches!(b.tile_index, 0x14..=0x45 | 0x4b | 0x4c)
                    };
                    if s.tileset_id.as_ref() != ts || !valid(x) {
                        x += 1;
                        continue;
                    }
                    let start = x;
                    while x < g.width && valid(x) {
                        x += 1;
                    }
                    add_rect(
                        r,
                        start,
                        y,
                        x - start,
                        2,
                        floor,
                        Detail::GlyphFrieze,
                        16.,
                        4.,
                    );
                }
            }
        }
        if word_room(ts) {
            let letters = grouped_flat_card_placements(cells, g, 0x02, false, |s| {
                if s.tileset_id.as_ref() != ts || !(0x80..=0xdf).contains(&s.tile_index) {
                    return None;
                }
                Some((
                    (s.tile_index % 2) as u8,
                    ((s.tile_index / 16) % 2) as u8,
                    2,
                    2,
                ))
            });
            // Require each four-tile letter to stay within the same 16-column
            // atlas band, rather than assemble unrelated glyph halves.
            let letters = letters
                .into_iter()
                .filter(|p| {
                    let a = cells[p.row * g.width + p.column].source.tile_index;
                    [
                        cells[p.row * g.width + p.column + 1].source.tile_index,
                        cells[(p.row + 1) * g.width + p.column].source.tile_index,
                        cells[(p.row + 1) * g.width + p.column + 1]
                            .source
                            .tile_index,
                    ] == [a + 1, a + 16, a + 17]
                })
                .collect();
            groups(r, letters, Detail::FloorGlyph, 0x02, 1., 16.);
        }
    }
    if league(map) && ts == "elite_four_room" {
        let wall = grouped_flat_card_placements(cells, g, 0x01, false, |s| {
            (s.tileset_id.as_ref() == ts
                && s.metatile_id == 0x15
                && s.subtile_row < 2
                && s.tile_index == 0x10)
                .then_some((s.subtile_column, s.subtile_row, 4, 2))
        });
        groups(r, wall, Detail::Model(Asset::LeagueWall), 0x01, 16., 5.);
        let barriers =
            grouped_flat_card_placements(cells, g, 0x01, false, |s| {
                if s.tileset_id.as_ref() != ts || s.metatile_id != 0x2a {
                    return None;
                }
                let art = [
                    [0x25, 0x27, 0x27, 0x26],
                    [0x35, 0x37, 0x37, 0x36],
                    [0x10; 4],
                    [0x10; 4],
                ];
                (s.tile_index == art[s.subtile_row as usize][s.subtile_column as usize])
                    .then_some((s.subtile_column, s.subtile_row, 4, 4))
            });
        groups(r, barriers, Detail::LeagueBarrier, 0x01, 16., 32.);
    }
    if map == "LancesRoom" && ts == "champions_room" {
        let dragons = grouped_flat_card_placements(cells, g, 0x97, false, |s| {
            let x = match s.metatile_id {
                0x35 if s.subtile_column < 2 => s.subtile_column,
                0x36 if s.subtile_column >= 2 => s.subtile_column - 2,
                _ => return None,
            };
            (s.tileset_id.as_ref() == ts
                && s.tile_index == 0x98 + u16::from(s.subtile_row) * 2 + u16::from(x))
            .then_some((x, s.subtile_row, 2, 4))
        });
        groups(
            r,
            dragons,
            Detail::Model(Asset::ChampionDragon),
            0x97,
            30.,
            10.,
        );
    }
    if map == "HallOfFame" && ts == "ice_path" {
        let console =
            grouped_flat_card_placements(cells, g, crate::hall_of_fame::FLOOR_TILE, false, |s| {
                crate::hall_of_fame::console_local(s).and_then(|(x, y)| {
                    (s.tile_index
                        == [[0x26, 0x27], [0x36, 0x37], [0x45, 0x46]][y as usize][x as usize])
                        .then_some((x, y, 2, 3))
                })
            });
        groups(
            r,
            console,
            Detail::Model(Asset::HallOfFameTerminal),
            crate::hall_of_fame::FLOOR_TILE,
            22.,
            10.,
        );
    }
    if map.starts_with("FastShip") && ts == "lighthouse" {
        let floor = if map == "FastShip1F" { 0x04 } else { 0x0d };
        let wall_height = crate::ship::visual_wall_height(map);
        if matches!(map, "FastShip1F" | "FastShipB1F") {
            let compounds = grouped_flat_card_placements(cells, g, floor, false, |s| {
                if s.tileset_id.as_ref() != ts || s.subtile_row == 0 {
                    return None;
                }
                let valid = if map == "FastShip1F" {
                    matches!(s.metatile_id, 0x05 | 0x0f | 0x13 | 0x19)
                } else {
                    s.metatile_id == 0x05
                };
                if !valid {
                    return None;
                }
                let valid_art = match s.subtile_row {
                    1 => matches!(s.tile_index, 0x11 | 0x22 | 0x32),
                    2 => matches!(s.tile_index, 0x02 | 0x03 | 0x10 | 0x0e | 0x0f | 0x20 | 0x21),
                    3 => matches!(s.tile_index, 0x12 | 0x1e | 0x1f | 0x30 | 0x31),
                    _ => false,
                };
                valid_art.then_some((s.subtile_column, s.subtile_row - 1, 4, 3))
            });
            groups(r, compounds, Detail::ShipCompound, floor, wall_height, 24.);
        }
        let doors = grouped_flat_card_placements(cells, g, floor, false, |s| {
            if s.tileset_id.as_ref() != ts || !matches!(s.metatile_id, 0x09 | 0x19 | 0x37) {
                return None;
            }
            match s.tile_index {
                0x0e => Some((0, 0, 2, 2)),
                0x0f => Some((1, 0, 2, 2)),
                0x1e => Some((0, 1, 2, 2)),
                0x1f => Some((1, 1, 2, 2)),
                _ => None,
            }
        });
        groups(r, doors, Detail::Model(Asset::ShipDoor), floor, wall_height, 3.);
        let plains = grouped_flat_card_placements(cells, g, floor, false, |s| {
            if s.tileset_id.as_ref() != ts
                || !matches!(
                    s.metatile_id,
                    0x05 | 0x09
                        | 0x0a
                        | 0x0f
                        | 0x13
                        | 0x19
                        | 0x1c
                        | 0x1d
                        | 0x1e
                        | 0x1f
                        | 0x26
                        | 0x2f
                        | 0x34
                        | 0x37
                        | 0x38
                )
            {
                return None;
            }
            match (s.subtile_row % 2, s.tile_index) {
                (0, 0x10) => Some((0, 0, 1, 2)),
                (1, 0x12) => Some((0, 1, 1, 2)),
                _ => None,
            }
        });
        groups(
            r,
            plains,
            Detail::Model(Asset::ShipBulkhead),
            floor,
            wall_height,
            3.,
        );
    }
    if map == "CeruleanGym" && ts == "port" {
        for (i, tile) in cells.iter().enumerate() {
            if let Some((detail, _)) = cerulean_bridge_detail(&tile.source)
            {
                // Brick decks carry their own material. Upright wooden faces
                // also expose water, so require its actual source sample rather
                // than substituting a blue patch when the viewport omits it.
                let floor = if matches!(detail, Detail::CeruleanBridgeDeck) {
                    i
                } else {
                    let Some(water) = cells.iter().position(|t| cerulean_water_source(&t.source)) else { continue; };
                    water
                };
                add_rect(r, i % g.width, i / g.width, 1, 1, floor, detail, 3., 8.);
            }
        }
    }
    if map == "GoldenrodDeptStoreB1F" && ts == "underground" {
        let lift = grouped_flat_card_placements(cells, g, 0x10, false, |s| {
            if s.tileset_id.as_ref() != "underground" || s.subtile_row >= 2
                || s.tile_index != [0x02, 0x12][s.subtile_row as usize] { return None; }
            let column = match (s.metatile_id, s.subtile_column) {
                (0x25, 2) => 0, (0x25, 3) => 1,
                (0x26, 0) => 2, (0x26, 1) => 3,
                _ => return None,
            };
            Some((column, s.subtile_row, 4, 2))
        });
        groups(r, lift, Detail::StoreBasementLift, 0x10, 16., 3.);
        let stair = grouped_flat_card_placements(cells, g, 0x10, false, |s| {
            if s.tileset_id.as_ref() != "underground" || s.metatile_id != 0x03
                || s.subtile_column < 2 || s.subtile_column > 3 || s.subtile_row >= 2 { return None; }
            let x = s.subtile_column - 2;
            (s.tile_index == [[0x2c, 0x2d], [0x3c, 0x3d]][s.subtile_row as usize][x as usize])
                .then_some((x, s.subtile_row, 2, 2))
        });
        groups(r, stair, Detail::StoreBasementStair, 0x10, 16., 16.);
    }
    if (passage(map) || map == "GoldenrodDeptStoreB1F") && ts == "underground" {
        if let Some(floor) = ground(map, cells, ts, 0x10) {
            // Same exact source vocabulary as the established underground
            // boundary; no teleport pad, stair or floor cell is a wall.
            let wall: Vec<_> = cells
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let s = &t.source;
                    s.tileset_id.as_ref() == ts
                        && (matches!(s.metatile_id,0x08..=0x0e|0x10..=0x14|0x25|0x26|0x2c|0x2d|0x2f)
                            || (map == "GoldenrodDeptStoreB1F" && matches!(s.metatile_id, 0x18..=0x1f)))
                        && (map != "GoldenrodDeptStoreB1F" || basement_wall_block(cells, g, i))
                        && matches!(s.tile_index,0x03..=0x05|0x0c|0x0d|0x13..=0x15)
                })
                .collect();
            for (i, &occupied) in wall.iter().enumerate() {
                if !occupied {
                    continue;
                }
                let (x, y) = (i % g.width, i / g.width);
                let open = [
                    y > 0 && !wall[i - g.width],
                    x + 1 < g.width && !wall[i + 1],
                    y + 1 < g.height && !wall[i + g.width],
                    x > 0 && !wall[i - 1],
                ];
                add_rect(
                    r,
                    x,
                    y,
                    1,
                    1,
                    floor,
                    Detail::PassageNetwork { open, basement: map == "GoldenrodDeptStoreB1F" },
                    16.,
                    8.,
                );
            }
        }
        if map == "SaffronGym" {
            let statues = grouped_flat_card_placements(cells, g, 0x10, false, |s| {
                if s.tileset_id.as_ref() != ts || s.metatile_id != 0x36 || s.subtile_column >= 2 {
                    return None;
                }
                let art = [[0x07, 0x08], [0x17, 0x18], [0x09, 0x19], [0x30, 0x31]];
                (s.tile_index == art[s.subtile_row as usize][s.subtile_column as usize])
                    .then_some((s.subtile_column, s.subtile_row, 2, 4))
            });
            r.groups(Kind::TowerGuardian, 0x10, 28., 9., statues);
            let pads = grouped_flat_card_placements(cells, g, 0x10, false, |s| {
                if s.tileset_id.as_ref() != ts
                    || s.metatile_id != 0x0f
                    || s.subtile_column < 2
                    || s.subtile_row < 2
                {
                    return None;
                }
                let (x, y) = (s.subtile_column - 2, s.subtile_row - 2);
                (s.tile_index == [[0x22, 0x23], [0x32, 0x33]][y as usize][x as usize])
                    .then_some((x, y, 2, 2))
            });
            groups(r, pads, Detail::TeleportPad, 0x10, 1., 16.);
        }
    }
    if harbor(map) && ts == "port" {
        let bollards = grouped_flat_card_placements(cells, g, 0x14, false, |s| {
            if s.tileset_id.as_ref() != ts || matches!(s.metatile_id, 0x18..=0x1f) {
                return None;
            }
            let (x, y) = (s.subtile_column % 2, s.subtile_row % 2);
            (s.tile_index == [[0x01, 0x02], [0x11, 0x12]][y as usize][x as usize])
                .then_some((x, y, 2, 2))
        });
        groups(
            r,
            bollards,
            Detail::Model(Asset::HarborBollard),
            0x14,
            12.,
            10.,
        );
        let rails = grouped_flat_card_placements(cells, g, 0x14, false, |s| {
            (s.tileset_id.as_ref() == ts
                && matches!(s.metatile_id, 0x08 | 0x27)
                && s.tile_index == 0x22)
                .then_some((s.subtile_column, 0, 4, 1))
        });
        groups(r, rails, Detail::Model(Asset::DockRailing), 0x14, 8., 2.);
        if let Some(floor) = ground(map, cells, ts, 0x14) {
            for (i, tile) in cells.iter().enumerate() {
                let s = &tile.source;
                if s.metatile_id != 0x18 || s.subtile_column != 0 || s.subtile_row != 0 {
                    continue;
                }
                let (x, y) = (i % g.width, i / g.width);
                if matching_rect(cells, g, x, y, 16, 8, |s, dx, dy| {
                    s.tileset_id.as_ref() == ts
                        && s.metatile_id == 0x18 + (dy / 4 * 4 + dx / 4) as u16
                        && s.subtile_column as usize == dx % 4
                        && s.subtile_row as usize == dy % 4
                        && ferry_anchor(dx, dy).is_none_or(|tile| s.tile_index == tile)
                }) {
                    add_rect(
                        r,
                        x,
                        y,
                        16,
                        8,
                        floor,
                        Detail::Model(Asset::HarborFerry),
                        28.,
                        64.,
                    );
                }
            }
        }
    }
    if map == "TinTowerRoof" && ts == "tower" {
        resolve_roof(r);
    }
    // Flat semantic floors are modeled below their existing zero-height datum.
    // Repeated brick, carpet and deck cells are complete one-cell materials;
    // clue letters, warp pads, stairs and black void are never generalized.
    for (i, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        if s.tileset_id.as_ref() != ts {
            continue;
        }
        let floor = if ruins(ts) && matches!(s.tile_index, 0x02 | 0x03) {
            Some(Floor::Stone)
        } else if passage(map) && s.tile_index == 0x10 {
            Some(Floor::Ceramic)
        } else if harbor(map) && matches!(s.tile_index, 0x05 | 0x31) {
            Some(Floor::Stone)
        } else if map.starts_with("FastShip") && s.tile_index == 0x04 {
            Some(Floor::Wood)
        } else if map == "TinTowerRoof"
            && s.tile_index == 0x02
            && matches!(s.metatile_id, 0x02 | 0x09 | 0x2d)
        {
            Some(Floor::Wood)
        } else if league(map) && ts == "elite_four_room" && s.tile_index == 0x01 {
            Some(Floor::Stone)
        } else if map == "LancesRoom" && ts == "champions_room" && s.metatile_id == 0x31 {
            Some(Floor::Carpet)
        } else {
            None
        };
        let Some(floor) = floor else {
            continue;
        };
        if shape_for_source_on_map(map, s)
            .surface_height(g.tile_height)
            .abs()
            > 0.001
        {
            continue;
        }
        add_rect(
            r,
            i % g.width,
            i / g.width,
            1,
            1,
            i,
            Detail::Floor(floor),
            1.,
            8.,
        );
    }
}

fn ferry_anchor(x: usize, y: usize) -> Option<u16> {
    match (x, y) {
        (0, 2) => Some(0x2b),
        (6, 0) => Some(0x1a),
        (8, 1) => Some(0x26),
        (12, 1) => Some(0x2a),
        (0, 4) => Some(0x2b),
        (4, 4) => Some(0x46),
        (8, 4) => Some(0x49),
        (12, 4) => Some(0x4d),
        (1, 6) => Some(0x58),
        (7, 6) => Some(0x5c),
        (11, 6) => Some(0x5e),
        (12, 6) => Some(0x5f),
        (3, 3) => Some(0x33),
        (7, 3) => Some(0x3e),
        (11, 3) => Some(0x42),
        (14, 3) => Some(0x45),
        _ => None,
    }
}
fn resolve_roof(r: &mut Resolver<'_>) {
    let (cells, g) = (r.cells, r.g);
    let mut side = vec![0i8; cells.len()];
    for (i, tile) in cells.iter().enumerate() {
        let s = &tile.source;
        if s.subtile_column != 0 || s.subtile_row != 0 || !matches!(s.metatile_id, 0x1c | 0x23) {
            continue;
        }
        let (x, y) = (i % g.width, i / g.width);
        let block = s.metatile_id;
        let art = if block == 0x1c {
            [0x16, 0x24, 0x16, 0x06]
        } else {
            [0x25, 0x35, 0x34, 0x35]
        };
        if matching_rect(cells, g, x, y, 4, 4, |s, dx, dy| {
            s.tileset_id.as_ref() == "tower"
                && s.metatile_id == block
                && s.subtile_column as usize == dx
                && s.subtile_row as usize == dy
                && s.tile_index == art[dx]
        }) {
            for dy in 0..4 {
                for dx in 0..4 {
                    side[(y + dy) * g.width + x + dx] = if block == 0x1c { -1 } else { 1 };
                }
            }
        }
    }
    for (i, &slope) in side.iter().enumerate() {
        if slope == 0 {
            continue;
        }
        let (x, y) = (i % g.width, i / g.width);
        let edge = if slope < 0 {
            let mut c = x;
            while c < g.width && side[y * g.width + c] == slope {
                c += 1;
            }
            c
        } else {
            let mut c = x;
            while c > 0 && side[y * g.width + c - 1] == slope {
                c -= 1;
            }
            c
        };
        let adjacent = if slope < 0 {
            edge
        } else {
            match edge.checked_sub(1) {
                Some(v) => v,
                None => continue,
            }
        };
        if adjacent >= g.width {
            continue;
        }
        let anchor = &cells[y * g.width + adjacent].source;
        if anchor.tileset_id.as_ref() != "tower"
            || !matches!(anchor.metatile_id, 0x02 | 0x09 | 0x1b | 0x2d)
        {
            continue;
        }
        let edge_x = g.origin_x + edge as f32 * g.tile_width;
        let (w, e, _, _) = g.bounds(x, y);
        let heights = if slope < 0 {
            [-(edge_x - w) * 0.25, -(edge_x - e) * 0.25]
        } else {
            [-(w - edge_x) * 0.25, -(e - edge_x) * 0.25]
        };
        add_rect(
            r,
            x,
            y,
            1,
            1,
            i,
            Detail::RoofSlope {
                west: heights[0],
                east: heights[1],
            },
            1.,
            8.,
        );
    }
    let ridges = grouped_flat_card_placements(cells, g, 0x02, false, |s| {
        if s.tileset_id.as_ref() != "tower" || !matches!(s.metatile_id, 0x1b | 0x2d) {
            return None;
        }
        match s.tile_index {
            0x50 => Some((0, 0, 2, 1)),
            0x51 => Some((1, 0, 2, 1)),
            _ => None,
        }
    });
    groups(r, ridges, Detail::RoofRidge, 0x02, 3., 8.);
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    detail: Detail,
) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.height as f32 * g.tile_height;
    let scale = g.tile_height / SOURCE_TILE_HEIGHT;
    let rise = p.rise_pixels * scale;
    let floor_height = if matches!(
        detail,
        Detail::Model(Asset::HarborBollard | Asset::HarborFerry | Asset::DockRailing)
    ) {
        -2. * scale
    } else if matches!(detail, Detail::StoreBasementStair) {
        -16. * scale
    } else {
        0.
    };
    if !matches!(
        detail,
        Detail::Floor(_)
            | Detail::CeruleanBridgeDeck
            | Detail::CeruleanBridgeFace
            | Detail::BlackthornCrossing
            | Detail::BlackthornChannel {..}
            | Detail::FloorGlyph
            | Detail::TeleportPad
            | Detail::RoofSlope { .. }
            | Detail::RoofRidge
    ) {
        let uv = g.uv(p.ground % g.width, p.ground / g.width);
        for i in p.indices(g.width) {
            append_top(
                &mut mesh.textured,
                g.bounds(i % g.width, i / g.width).into(),
                floor_height,
                uv,
            );
        }
    }
    match detail {
        Detail::VioletWall | Detail::SproutAltar => {
            let timber=matches!(detail,Detail::SproutAltar);
            let body=if timber {[0.52,0.34,0.13,1.]} else {[0.62,0.65,0.73,1.]};
            slab(&mut mesh.solid,[w,e,s-4.*scale,s],0.,rise,body);
            // The actual packed ornament skins a closed wall. Every source
            // tile retains its native size and phase; no picture is invented.
            for y in 0..p.height {for x in 0..p.width {
                let uv=g.uv(p.column+x,p.row+y);
                let x0=w+x as f32*g.tile_width;
                let x1=x0+g.tile_width;
                let top=rise-y as f32*g.tile_height;
                let bottom=top-g.tile_height;
                append_quad(&mut mesh.textured,
                    [[x1,bottom,s+0.01],[x1,top,s+0.01],[x0,top,s+0.01],[x0,bottom,s+0.01]],
                    [0.,0.,1.],[[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2],[uv.0,uv.3]],TEXTURED_SHADE);
            }}
            let trim=if timber {[0.72,0.52,0.23,1.]} else {[0.82,0.81,0.69,1.]};
            slab(&mut mesh.solid,[w,e,s-5.*scale,s+scale],rise-scale,rise+scale,trim);
            slab(&mut mesh.solid,[w,e,s-5.*scale,s+scale],0.,scale,trim);
        }
        Detail::FuchsiaWall => {
            slab(&mut mesh.solid,[w,e,s-4.*scale,s],0.,rise,[0.76,0.78,0.69,1.]);
            slab(&mut mesh.solid,[w,e,s-4.5*scale,s+0.5*scale],14.5*scale,rise,[0.92,0.91,0.82,1.]);
            slab(&mut mesh.solid,[w,e,s,s+0.3*scale],0.,2.*scale,[0.39,0.48,0.42,1.]);
            for column in 0..2 {
                let west=w+(e-w)*column as f32/2.;
                slab(&mut mesh.solid,[west+scale,west+(e-w)/2.-scale,s,s+0.15*scale],
                    3.*scale,13.*scale,[0.68,0.73,0.64,1.]);
            }
        }
        Detail::BlackthornChannel {lava} => {
            let top=if lava {-2.*scale}else{0.};
            slab(&mut mesh.solid,[w,e,n,s],-4.*scale,top,
                if lava {[0.79,0.24,0.14,1.]}else{[0.31,0.42,0.53,1.]});
            // Sample the actual source face at its recessed datum. This keeps
            // lava 38/5b on the production atlas animation path; no new clock,
            // substitute sample, or generic liquid material is introduced.
            append_top(&mut mesh.textured,[w,e,n,s],top+0.001*scale,
                g.uv(p.column,p.row));
        }
        Detail::BlackthornStairs => {
            crate::interior_models::model(crate::interior_models::ModelKind::StairFlight)
                .append_fitted(&mut mesh.solid,[w,e,n,s],0.,rise);
        }
        Detail::BlackthornCrossing => {
            // The callback fills the channel with rubble that can be crossed.
            // Keep every top at the original walking datum, with depth below it.
            slab(&mut mesh.solid,[w,e,n,s],-6.*scale,-0.4*scale,[0.33,0.34,0.31,1.]);
            for row in 0..3 { for column in 0..3 {
                let west = w+(e-w)*column as f32/3.;
                let north = n+(s-n)*row as f32/3.;
                slab(&mut mesh.solid,[west+0.15*scale,west+(e-w)/3.-0.15*scale,
                    north+0.15*scale,north+(s-n)/3.-0.15*scale],-0.4*scale,0.,
                    if (row+column)%2==0 {[0.58,0.59,0.54,1.]}else{[0.49,0.51,0.46,1.]});
            }}
        }
        Detail::MahoganyPartition => {
            crate::dungeon_models::room_model(crate::dungeon_models::RoomAsset::GymStonePartition)
                .append(&mut mesh.solid, [w,e,s-16.*scale,s], 0., rise);
        }
        Detail::MahoganyWall => {
            // Pale square panels under a broad cold-gray cap. Repeated modules
            // meet along the same front plane, with no per-tile vertical blocks.
            let back = s - 8. * scale;
            let face = s - scale;
            slab(&mut mesh.solid, [w, e, back, face], 0., rise, [0.56, 0.61, 0.62, 1.]);
            slab(&mut mesh.solid, [w, e, back - scale, s], 22. * scale, rise,
                [0.78, 0.82, 0.80, 1.]);
            slab(&mut mesh.solid, [w, e, face, s], 17. * scale, 22. * scale,
                [0.67, 0.73, 0.71, 1.]);
            for row in 0..2 {
                for column in 0..p.width {
                    let west = w + column as f32 * g.tile_width;
                    let bottom = (1. + row as f32 * 8.) * scale;
                    slab(&mut mesh.solid,
                        [west + scale * 0.5, west + g.tile_width - scale * 0.5, face, s],
                        bottom, bottom + 7. * scale, [0.77, 0.82, 0.81, 1.]);
                    slab(&mut mesh.solid,
                        [west + 1.4 * scale, west + g.tile_width - 1.4 * scale, s, s + 0.2 * scale],
                        bottom + 0.9 * scale, bottom + 6.1 * scale, [0.59, 0.67, 0.67, 1.]);
                }
            }
        }
        Detail::VermilionPanel => {
            // The native drawing is a recessed dark upper panel, pale lower
            // wall and repeated plinth. Retain that silhouette as closed solids.
            let trim = [0.91, 0.87, 0.78, 1.];
            slab(&mut mesh.solid, [w, e, s - 6. * scale, s - scale], 0., rise,
                [0.68, 0.68, 0.65, 1.]);
            slab(&mut mesh.solid, [w + 2. * scale, e - 2. * scale, s - scale, s - 0.7 * scale],
                16. * scale, 30. * scale, [0.045, 0.055, 0.065, 1.]);
            for (bounds, low, high) in [
                ([w, w + 2. * scale, s - 6. * scale, s], 0., rise),
                ([e - 2. * scale, e, s - 6. * scale, s], 0., rise),
                ([w + 2. * scale, e - 2. * scale, s - scale, s], 30. * scale, rise),
                ([w + 2. * scale, e - 2. * scale, s - scale, s], 14. * scale, 16. * scale),
                ([w, e, s - 8. * scale, s + scale], 0., 3. * scale),
            ] { slab(&mut mesh.solid, bounds, low, high, trim); }
            for x in 0..p.width {
                let west = w + x as f32 * g.tile_width + scale;
                slab(&mut mesh.solid, [west, west + g.tile_width - 2. * scale, s, s + 0.35 * scale],
                    0.6 * scale, 1.6 * scale, [0.46, 0.47, 0.44, 1.]);
            }
        }
        Detail::SproutPillar => {
            let anchor = [p.column, p.row];
            super::super::sprout_pillar::append(&mut mesh.animated_solid, g, cells, anchor);
            mesh.animated_pillars.push(anchor);
        }
        Detail::Model(Asset::HarborFerry) => {
            // The final three source courses depict the hull's upright face.
            // Collapse that illustrated depth while keeping the north docking
            // seam fixed; the real boarding warp is on the preceding gangway.
            extension_model(Asset::HarborFerry).append(
                &mut mesh.solid,
                [w, e, n, n + 40. * scale],
                floor_height,
                28. * scale,
            );
        }
        Detail::Model(kind) => extension_model(kind).append(
            &mut mesh.solid,
            [w, e, s - p.depth_pixels * scale, s],
            floor_height,
            rise,
        ),
        Detail::CeruleanBridgeDeck => {
            // Walkers retain the authored zero-height footing. Keep the modeled
            // bridge thickness below that surface rather than clipping feet.
            slab(&mut mesh.solid, [w, e, n, s], -3. * scale, -0.15 * scale, [0.97, 0.80, 0.62, 1.]);
            let grout = 0.18 * scale;
            for row in 0..2 {
                let north = n + (s - n) * row as f32 * 0.5;
                let south = north + (s - n) * 0.5;
                let pieces = if (p.row * 2 + row) % 2 == 0 { 1 } else { 2 };
                for piece in 0..pieces {
                    let west = w + (e - w) * piece as f32 / pieces as f32;
                    let east = w + (e - w) * (piece + 1) as f32 / pieces as f32;
                    slab(&mut mesh.solid, [west + grout, east - grout, north + grout, south - grout],
                        -0.15 * scale, 0., [0.90, 0.39, 0.43, 1.]);
                }
            }
        }
        Detail::CeruleanBridgeFace => {
            // The painted face belongs against the preceding deck's edge,
            // not eight pixels farther into the water at the source row's end.
            let depth = 1.2 * scale;
            append_top(&mut mesh.textured, [w, e, n, s], 0., g.uv(p.ground % g.width, p.ground / g.width));
            slab(&mut mesh.solid, [w, e, n, n + depth], -3. * scale, 3. * scale, [0.47, 0.34, 0.15, 1.]);
            for x in [w + (e - w) * 0.12, w + (e - w) * 0.62] {
                slab(&mut mesh.solid, [x, x + (e - w) * 0.18, n, n + depth],
                    -6. * scale, 2. * scale, [0.60, 0.46, 0.23, 1.]);
            }
        }
        Detail::StoreBasementLift => {
            crate::interior_models::model(crate::interior_models::ModelKind::GateDoorFrame)
                .append_fitted(&mut mesh.solid, [w, e, s - 3. * scale, s], 0., 16. * scale);
            // Keep the native pink threshold beneath the open frame.
            for i in p.indices(g.width) {
                append_top(&mut mesh.textured, g.bounds(i % g.width, i / g.width).into(),
                    0.01 * scale, g.uv(i % g.width, i / g.width));
            }
        }
        Detail::StoreBasementStair => {
            let first = mesh.solid.positions.len();
            crate::interior_models::model(crate::interior_models::ModelKind::StairFlight)
                .append_fitted(&mut mesh.solid, [n, s, -e, -w], -16. * scale, 16. * scale);
            for point in &mut mesh.solid.positions[first..] {
                *point = [-point[2], point[1], point[0]];
            }
            for (normal, color) in mesh.solid.normals[first..].iter_mut().zip(&mut mesh.solid.colors[first..]) {
                let old = crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal))[0];
                *normal = [-normal[2], normal[1], normal[0]];
                let new = crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal))[0];
                for channel in &mut color[..3] { *channel *= new / old; }
            }
        }
        Detail::GlyphFrieze => {
            extension_model(Asset::RuinsFrieze).append(
                &mut mesh.solid,
                [w, e, s - 4. * scale, s],
                0.,
                16. * scale,
            );
            for x in 0..p.width {
                let uv = g.uv(p.column + x, p.row + 1);
                let x0 = w + x as f32 * g.tile_width;
                let x1 = x0 + g.tile_width;
                // Preserve every glyph pixel at its original size and phase.
                append_quad(
                    &mut mesh.textured,
                    [
                        [x1, 4. * scale, s + 0.01],
                        [x1, 12. * scale, s + 0.01],
                        [x0, 12. * scale, s + 0.01],
                        [x0, 4. * scale, s + 0.01],
                    ],
                    [0., 0., 1.],
                    [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                    TEXTURED_SHADE,
                );
            }
        }
        Detail::PuzzleDais => {
            extension_model(Asset::PuzzleDais).append(
                &mut mesh.solid,
                [w, e, s - 12. * scale, s],
                0.,
                10. * scale,
            );
            for x in 0..p.width {
                let x0 = w + x as f32 * g.tile_width;
                let x1 = x0 + g.tile_width;
                append_top(
                    &mut mesh.textured,
                    [x0, x1, s - 9. * scale, s - 1. * scale],
                    10. * scale + 0.01,
                    g.uv(p.column + x, p.row + 1),
                );
            }
        }
        Detail::FloorGlyph | Detail::TeleportPad => {
            slab(
                &mut mesh.solid,
                [w, e, n, s],
                -1.5 * scale,
                0.,
                [0.60, 0.57, 0.44, 1.],
            );
            for i in p.indices(g.width) {
                append_top(
                    &mut mesh.textured,
                    g.bounds(i % g.width, i / g.width).into(),
                    0.01,
                    g.uv(i % g.width, i / g.width),
                );
            }
        }
        Detail::Floor(kind) => {
            floor_tile(&mut mesh.solid, [w, e, n, s], kind, p.column, p.row, scale)
        }
        Detail::PassageNetwork { open, basement } => {
            if open[2] {
                let mut rear_open = open;
                rear_open[2] = false;
                append_masonry(
                    &mut mesh.solid,
                    [w, e, n, s - 2. * scale],
                    16. * scale,
                    rear_open,
                );
                extension_model(Asset::PassageWall).append(
                    &mut mesh.solid,
                    [w, e, s - 2. * scale, s],
                    0.,
                    16. * scale,
                );
            } else {
                append_masonry(&mut mesh.solid, [w, e, n, s], 16. * scale, open);
            }
            if basement {
                // Preserve the stockroom's blue wainscot as modeled trim.
                let trim = 0.28 * scale;
                let color = [0.18, 0.25, 0.72, 1.];
                for (edge, visible) in open.into_iter().enumerate() {
                    if !visible { continue; }
                    let bounds = match edge {
                        0 => [w, e, n, n + trim],
                        1 => [e - trim, e, n + if open[0] { trim } else { 0. }, s - if open[2] { trim } else { 0. }],
                        2 => [w, e, s - trim, s],
                        _ => [w, w + trim, n + if open[0] { trim } else { 0. }, s - if open[2] { trim } else { 0. }],
                    };
                    slab(&mut mesh.solid, bounds, 1. * scale, 6. * scale, color);
                }
            }
        }
        Detail::LeagueBarrier => {
            slab(
                &mut mesh.solid,
                [w, e, n, s - 4. * scale],
                0.,
                16. * scale,
                [0.45, 0.48, 0.43, 1.],
            );
            extension_model(Asset::LeagueWall).append(
                &mut mesh.solid,
                [w, e, s - 4. * scale, s],
                0.,
                16. * scale,
            );
        }
        Detail::ShipCompound => {
            // Source cap is a real horizontal rear course, not a floating card.
            // Join it to the front panels through a closed original steel shell.
            slab(
                &mut mesh.solid,
                [w, e, n, s - 3. * scale],
                0.,
                rise,
                [0.66, 0.70, 0.65, 1.],
            );
            let mut x = 0;
            while x < p.width {
                let tile = cells[(p.row + 1) * g.width + p.column + x]
                    .source
                    .tile_index;
                let x0 = w + x as f32 * g.tile_width;
                let pair = x + 1 < p.width && matches!(tile, 0x02 | 0x0e);
                let width = if pair { 2 } else { 1 };
                let x1 = x0 + width as f32 * g.tile_width;
                if tile == 0x02 && pair {
                    model(Kind::PortholeBulkhead).append_porthole(
                        &mut mesh.solid,
                        [x0, x1, s - 3. * scale, s],
                        0.,
                        16. * scale,
                        rise,
                    );
                } else {
                    extension_model(if tile == 0x0e && pair {
                        Asset::ShipDoor
                    } else {
                        Asset::ShipBulkhead
                    })
                    .append(
                        &mut mesh.solid,
                        [x0, x1, s - 3. * scale, s],
                        0.,
                        rise,
                    );
                }
                x += width;
            }
        }
        Detail::RoofSlope { west, east } => {
            roof_tile(&mut mesh.solid, [w, e, n, s], west, east, scale)
        }
        Detail::RoofRidge => {
            slab(
                &mut mesh.solid,
                [w, e, n, s],
                -1. * scale,
                0.,
                [0.24, 0.31, 0.35, 1.],
            );
            for segment in 0..8 {
                let a = segment as f32 / 8.;
                let b = (segment + 1) as f32 / 8.;
                let ya = (a * std::f32::consts::PI).sin() * 3. * scale;
                let yb = (b * std::f32::consts::PI).sin() * 3. * scale;
                for (z, normal) in [(n, [0., 0., -1.]), (s, [0., 0., 1.])] {
                    faceted_face(
                        &mut mesh.solid,
                        [
                            [w + (e - w) * a, 0., z],
                            [w + (e - w) * b, 0., z],
                            [w + (e - w) * b, yb, z],
                            [w + (e - w) * a, ya, z],
                        ],
                        normal,
                        [0.35, 0.43, 0.45, 1.],
                        0.,
                    );
                }
                faceted_face(
                    &mut mesh.solid,
                    [
                        [w + (e - w) * a, ya, n],
                        [w + (e - w) * a, ya, s],
                        [w + (e - w) * b, yb, s],
                        [w + (e - w) * b, yb, n],
                    ],
                    [0., 1., 0.],
                    [0.42, 0.51, 0.52, 1.],
                    0.,
                );
            }
        }
    }
}
fn slab(out: &mut SurfaceMeshData, b: [f32; 4], low: f32, high: f32, color: [f32; 4]) {
    let [w, e, n, s] = b;
    for (p, norm) in [
        (
            [[w, high, n], [w, high, s], [e, high, s], [e, high, n]],
            [0., 1., 0.],
        ),
        (
            [[w, low, n], [e, low, n], [e, low, s], [w, low, s]],
            [0., -1., 0.],
        ),
        (
            [[e, low, n], [w, low, n], [w, high, n], [e, high, n]],
            [0., 0., -1.],
        ),
        (
            [[w, low, s], [e, low, s], [e, high, s], [w, high, s]],
            [0., 0., 1.],
        ),
        (
            [[w, low, n], [w, low, s], [w, high, s], [w, high, n]],
            [-1., 0., 0.],
        ),
        (
            [[e, low, s], [e, low, n], [e, high, n], [e, high, s]],
            [1., 0., 0.],
        ),
    ] {
        faceted_face(out, p, norm, color, 0.);
    }
}
fn floor_tile(out: &mut SurfaceMeshData, b: [f32; 4], kind: Floor, x: usize, y: usize, scale: f32) {
    let [w, e, n, s] = b;
    let color = match kind {
        Floor::Stone => {
            if (x + y) % 2 == 0 {
                [0.57, 0.54, 0.43, 1.]
            } else {
                [0.63, 0.59, 0.47, 1.]
            }
        }
        Floor::Wood => [0.46, 0.32, 0.19, 1.],
        Floor::Ceramic => {
            if (x + y) % 2 == 0 {
                [0.60, 0.65, 0.63, 1.]
            } else {
                [0.48, 0.54, 0.53, 1.]
            }
        }
        Floor::Carpet => [0.35, 0.22, 0.33, 1.],
    };
    slab(
        out,
        b,
        -1.3 * scale,
        -0.12 * scale,
        [color[0] * 0.7, color[1] * 0.7, color[2] * 0.7, 1.],
    );
    let count = if matches!(kind, Floor::Wood) { 4 } else { 1 };
    for i in 0..count {
        let z0 = n + (s - n) * i as f32 / count as f32 + 0.04 * scale;
        let z1 = n + (s - n) * (i + 1) as f32 / count as f32 - 0.04 * scale;
        let pad = 0.055 * scale;
        let top = [
            [w + pad, 0., z0 + pad],
            [w + pad, 0., z1 - pad],
            [e - pad, 0., z1 - pad],
            [e - pad, 0., z0 + pad],
        ];
        faceted_face(out, top, [0., 1., 0.], color, 0.);
        for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
            let mut pa = top[a];
            let mut pb = top[b];
            pa[1] = -0.12 * scale;
            pb[1] = -0.12 * scale;
            let normal = [[-1., 0., 0.], [0., 0., 1.], [1., 0., 0.], [0., 0., -1.]][a];
            faceted_face(out, [top[a], top[b], pb, pa], normal, color, 0.);
        }
    }
}
fn roof_tile(out: &mut SurfaceMeshData, b: [f32; 4], west: f32, east: f32, scale: f32) {
    let [w, e, n, s] = b;
    let color = [0.29, 0.40, 0.43, 1.];
    for j in 0..4 {
        let a = j as f32 / 4.;
        let b = (j + 1) as f32 / 4.;
        let x0 = w + (e - w) * a;
        let x1 = w + (e - w) * b;
        let y0 = west + (east - west) * a;
        let y1 = west + (east - west) * b;
        // Low geometric tile courses sit wholly below the walkable ridge datum.
        let normal = Vec3::new(west - east, e - w, 0.).normalize().to_array();
        faceted_face(
            out,
            [[x0, y0, n], [x0, y0, s], [x1, y1, s], [x1, y1, n]],
            normal,
            color,
            0.18 * scale,
        );
    }
    faceted_face(
        out,
        [
            [w, west - scale, n],
            [e, east - scale, n],
            [e, east - scale, s],
            [w, west - scale, s],
        ],
        Vec3::new(east - west, w - e, 0.).normalize().to_array(),
        color,
        0.,
    );
    for (p, norm) in [
        (
            [
                [w, west - 1. * scale, n],
                [w, west - 1. * scale, s],
                [w, west, s],
                [w, west, n],
            ],
            [-1., 0., 0.],
        ),
        (
            [
                [e, east - 1. * scale, s],
                [e, east - 1. * scale, n],
                [e, east, n],
                [e, east, s],
            ],
            [1., 0., 0.],
        ),
        (
            [
                [w, west - 1. * scale, s],
                [e, east - 1. * scale, s],
                [e, east, s],
                [w, west, s],
            ],
            [0., 0., 1.],
        ),
        (
            [
                [e, east - 1. * scale, n],
                [w, west - 1. * scale, n],
                [w, west, n],
                [e, east, n],
            ],
            [0., 0., -1.],
        ),
    ] {
        faceted_face(out, p, norm, color, 0.);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn grid(w: usize, h: usize) -> GridGeometry {
        GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    fn tiles(w: usize, h: usize, ts: &str, floor: u16) -> Vec<VisualTile> {
        (0..w * h)
            .map(|i| VisualTile {
                animation_frames: None,
                column: (i % w) as u32,
                row: (i / w) as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from(ts),
                    metatile_id: 0x25,
                    subtile_column: (i % w % 4) as u8,
                    subtile_row: (i / w % 4) as u8,
                    tile_index: floor,
                },
            })
            .collect()
    }
    #[test]
    fn room_architecture_requires_complete_art_and_preserves_actor_floor() {
        for (map,ts,floor,block,label) in [
            ("VioletGym","elite_four_room",0x01,0x2d,"special:violet-north-wall"),
            ("SproutTower3F","tower",0x02,0x09,"special:sprout-framed-altar"),
        ] {
            let g=grid(8,4);
            let mut t=tiles(8,4,ts,floor);
            for tile in &mut t {tile.source.metatile_id=block;}
            for y in 0..2 {for x in 0..4 {
                let source=&mut t[y*8+x+2].source;
                if map=="VioletGym" {
                    source.metatile_id=0x15;
                    source.subtile_column=x as u8;
                    source.tile_index=0x10;
                } else {
                    source.metatile_id=if x<2 {0x2e} else {0x2f};
                    source.tile_index=[[0x11,0x48,0x49,0x11],[0x21,0x58,0x59,0x21]][y][x];
                }
            }}
            let refs=t.iter().collect::<Vec<_>>();
            let placements=super::super::resolve(map,&refs,&g,None);
            let p=placements.iter().find(|p|p.kind_label()==label).unwrap();
            assert_eq!(p.indices(g.width).count(),8);
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;t.len()],..Default::default()};
            super::super::append(&mut mesh,&g,p,&refs,[0,0]);
            assert_eq!(mesh.footing_heights,vec![0.;t.len()]);
            assert!(mesh.solid.normals.iter().any(|n|n[2] < -0.99));
            assert!(mesh.solid.normals.iter().any(|n|n[0]>0.99));
            assert!(mesh.solid.positions.iter().any(|v|v[1]>=16.));
            assert!(!super::super::resolve("OtherRoom",&refs,&g,None).iter().any(|p|p.kind_label()==label));
            for i in [2,3,4,5,10,11,12,13] {
                let mut bad=t.clone();bad[i].source.tile_index^=0x80;
                assert!(!super::super::resolve(map,&bad.iter().collect::<Vec<_>>(),&g,None).iter().any(|p|p.kind_label()==label));
            }
            let crop=t.chunks(8).flat_map(|row|row[3..].iter().cloned()).collect::<Vec<_>>();
            assert!(!super::super::resolve(map,&crop.iter().collect::<Vec<_>>(),&grid(5,4),None).iter().any(|p|p.kind_label()==label));
        }
    }
    #[test]
    fn word_room_frieze_uses_original_glyph_uvs_on_raised_stone_faces() {
        let mut t = tiles(4, 4, "kabuto_word_room", 0x02);
        for x in 0..4 {
            t[x].source.tile_index = 0x06;
            t[4 + x].source.tile_index = [0x36, 0x1c, 0x22, 0x37][x];
        }
        let cells = t.iter().collect::<Vec<_>>();
        let g = grid(4, 4);
        let placements = super::super::resolve("RuinsOfAlphKabutoWordRoom", &cells, &g, None);
        let p = placements
            .iter()
            .find(|p| p.kind_label() == "special:live-glyph-stone-frieze")
            .unwrap();
        assert_eq!(p.indices(4).collect::<Vec<_>>(), (0..8).collect::<Vec<_>>());
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![0.; 16];
        let before = mesh.footing_heights.clone();
        super::super::append(&mut mesh, &g, p, &cells, [0,0]);
        assert_eq!(mesh.footing_heights, before);
        for x in 0..4 {
            let uv = g.uv(x, 1);
            assert!(mesh.textured.uvs.contains(&[uv.0, uv.2]));
            assert!(mesh.textured.uvs.contains(&[uv.1, uv.3]));
        }
        assert!(mesh.textured.positions.iter().any(|p| p[1] == 12.));
        assert!(!mesh.solid.indices.is_empty());
    }
    #[test]
    fn floor_letters_require_all_four_matching_glyph_quadrants() {
        let mut t = tiles(4, 4, "kabuto_word_room", 0x02);
        for (i, art) in [(0, 0x80), (1, 0x81), (4, 0x90), (5, 0x91)] {
            t[i].source.tile_index = art;
        }
        let count = |t: &Vec<VisualTile>| {
            super::super::resolve(
                "RuinsOfAlphKabutoWordRoom",
                &t.iter().collect::<Vec<_>>(),
                &grid(4, 4),
                None,
            )
            .iter()
            .filter(|p| p.kind_label() == "special:live-inscription-paving")
            .count()
        };
        assert_eq!(count(&t), 1);
        t[5].source.tile_index = 0xa1;
        assert_eq!(count(&t), 0);
    }
    #[test]
    fn ship_corridor_consumes_cap_and_two_faces_but_not_void() {
        let mut t = tiles(5, 4, "lighthouse", 0x04);
        let art = [
            [0x01, 0x01, 0x01, 0x01],
            [0x11, 0x11, 0x11, 0x11],
            [0x02, 0x03, 0x10, 0x10],
            [0x12, 0x12, 0x12, 0x12],
        ];
        for y in 0..4 {
            for x in 0..4 {
                let s = &mut t[y * 5 + x].source;
                s.metatile_id = 0x05;
                s.tile_index = art[y][x];
            }
        }
        let c = t.iter().collect::<Vec<_>>();
        let p = super::super::resolve("FastShip1F", &c, &grid(5, 4), None);
        let p = p
            .iter()
            .find(|p| p.kind_label() == "special:ship-corridor-bulkhead")
            .unwrap();
        assert_eq!((p.row, p.height), (1, 3));
        assert_eq!(p.indices(5).count(), 12);
        assert!(p.indices(5).all(|i| i >= 5));
    }
    #[test]
    fn compound_ferry_requires_eight_complete_blocks_and_bow_cabin_keel_anchors() {
        let mut t = tiles(17, 8, "port", 0x14);
        for y in 0..8 {
            for x in 0..16 {
                let s = &mut t[y * 17 + x].source;
                s.metatile_id = 0x18 + (y / 4 * 4 + x / 4) as u16;
                s.tile_index = ferry_anchor(x, y).unwrap_or(0x14);
            }
        }
        let count = |t: &Vec<VisualTile>| {
            super::super::resolve(
                "OlivinePort",
                &t.iter().collect::<Vec<_>>(),
                &grid(17, 8),
                None,
            )
            .iter()
            .filter(|p| p.kind_label() == "special:harbor-ferry")
            .count()
        };
        assert_eq!(count(&t), 1);
        t[6].source.tile_index = 0x14;
        assert_eq!(count(&t), 0);
    }
    #[test]
    fn passage_network_never_claims_a_teleport_pad_or_stair_drawing() {
        let mut t = tiles(4, 4, "underground", 0x10);
        for y in 0..2 {
            for x in 0..2 {
                let s = &mut t[y * 4 + x].source;
                s.metatile_id = 0x0f;
                s.tile_index = [[0x22, 0x23], [0x32, 0x33]][y][x];
            }
        }
        let p = super::super::resolve(
            "SaffronGym",
            &t.iter().collect::<Vec<_>>(),
            &grid(4, 4),
            None,
        );
        assert!(
            p.iter()
                .all(|p| p.kind_label() != "special:connected-passage-walls")
        );
        assert!(
            p.iter()
                .all(|p| p.indices(4).all(|i| ![0, 1, 4, 5].contains(&i)))
        );
    }
    #[test]
    fn store_basement_passage_network_checks_visible_source_without_crop_fallback() {
        let mut t = tiles(4, 4, "underground", 0x10);
        for (i, tile) in t.iter_mut().enumerate() {
            tile.source.metatile_id = 0x09;
            tile.source.tile_index = [0x04, 0x14, 0x10, 0x10][i / 4];
        }
        let g = grid(4, 4);
        let count = |tiles: &[VisualTile], grid: &GridGeometry| {
            super::super::resolve("GoldenrodDeptStoreB1F", &tiles.iter().collect::<Vec<_>>(), grid, None)
                .into_iter().filter(|p| p.kind_label() == "special:connected-passage-walls")
                .map(|p| {
                    assert!(p.indices(grid.width).all(|i| tiles[i].source.tile_index != 0x10));
                    p.indices(grid.width).count()
                }).sum::<usize>()
        };
        assert_eq!(count(&t, &g), 8);
        // A changed floor sample is also part of the complete drawing guard.
        t[15].source.tile_index = 0x01;
        assert_eq!(count(&t, &g), 0);
        t[15].source.tile_index = 0x10;
        let cropped: Vec<_> = t.chunks(4).flat_map(|row| row[1..].iter().cloned()).collect();
        assert_eq!(count(&cropped, &grid(3, 4)), 6);
        let mut changed_crop = cropped;
        changed_crop[11].source.tile_index = 0x01;
        assert_eq!(count(&changed_crop, &grid(3, 4)), 0);
    }
    #[test]
    fn store_basement_lift_and_warehouse_stair_have_distinct_complete_geometry() {
        let mut t = tiles(8, 4, "underground", 0x10);
        for y in 0..2 {
            for x in 2..6 {
                let s = &mut t[y * 8 + x].source;
                s.metatile_id = if x < 4 { 0x25 } else { 0x26 };
                s.tile_index = [0x02, 0x12][y];
            }
            for x in 6..8 {
                let s = &mut t[y * 8 + x].source;
                s.metatile_id = 0x03;
                s.tile_index = [[0x2c, 0x2d], [0x3c, 0x3d]][y][x - 6];
            }
        }
        let g = grid(8, 4);
        let resolve = |t: &[VisualTile]| super::super::resolve("GoldenrodDeptStoreB1F",
            &t.iter().collect::<Vec<_>>(), &g, None);
        let placements = resolve(&t);
        for (label, count, low, high) in [
            ("special:store-basement-lift-front", 8, 0., 16.),
            ("special:store-basement-warehouse-stair", 4, -16., 0.),
        ] {
            let p = placements.iter().find(|p| p.kind_label() == label).unwrap();
            assert_eq!(p.indices(8).count(), count);
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![2.5; t.len()];
            super::super::append(&mut mesh, &g, p, &t.iter().collect::<Vec<_>>(), [0, 0]);
            assert!(!mesh.solid.indices.is_empty());
            assert!(mesh.footing_heights.iter().all(|h| *h == 2.5));
            assert!(mesh.solid.positions.iter().all(|v| v[1] >= low - 0.001 && v[1] <= high + 0.001));
            assert!(mesh.solid.normals.iter().all(|n| (Vec3::from_array(*n).length_squared() - 1.).abs() < 0.0001));
            if low < 0. {
                assert!(mesh.textured.positions.iter().all(|v| v[1] == -16.));
                assert!(mesh.solid.positions.iter().any(|v| v[1] < -15.));
            }
        }
        t[2].source.tile_index = 0x03;
        assert!(!resolve(&t).iter().any(|p| p.kind_label() == "special:store-basement-lift-front"));
        assert!(resolve(&t).iter().any(|p| p.kind_label() == "special:store-basement-warehouse-stair"));
    }
    #[test]
    fn vermilion_north_panels_keep_central_passage_and_reject_partial_drawings() {
        let g = grid(20, 8);
        let mut t = tiles(20, 8, "game_corner", 0x01);
        let left = [[0x48,0x49,0x49,0x49],[0x58,0x59,0x59,0x59],[0x5c,0x4d,0x4d,0x4d],[0x5e,0x5b,0x5b,0x5b]];
        let right = [[0x49,0x49,0x49,0x4a],[0x59,0x59,0x59,0x5a],[0x4d,0x4d,0x4d,0x5d],[0x5b,0x5b,0x5b,0x5f]];
        for anchor in [0, 12] { for y in 0..4 { for x in 0..8 {
            let s = &mut t[y * 20 + anchor + x].source;
            s.metatile_id = if x < 4 {0x20} else {0x21};
            s.subtile_column = (x % 4) as u8;
            s.tile_index = if x < 4 {left[y][x]} else {right[y][x - 4]};
        }}}
        let refs: Vec<_> = t.iter().collect();
        let panels: Vec<_> = super::super::resolve("VermilionGym", &refs, &g, None).into_iter()
            .filter(|p|p.kind_label()=="special:vermilion-north-panel").collect();
        assert_eq!(panels.len(), 2);
        let mut mesh = TerrainMeshData { footing_heights: vec![0.;160], ..Default::default() };
        for p in &panels {
            assert_eq!(p.indices(20).count(), 32);
            assert!(p.indices(20).all(|i| !(8..12).contains(&(i % 20))));
            super::super::append(&mut mesh, &g, p, &refs, [0,0]);
        }
        assert!(!mesh.solid.indices.is_empty());
        assert!(mesh.solid.positions.iter().all(|p|p[1]>=0. && p[1]<=32.));
        assert!(mesh.footing_heights.iter().all(|h|*h==0.));
        assert!(mesh.solid.normals.iter().all(|n|(Vec3::from_array(*n).length_squared()-1.).abs()<0.0001));
        assert!(!super::super::resolve("GoldenrodGameCorner", &refs, &g, None).iter()
            .any(|p|p.kind_label()=="special:vermilion-north-panel"));
        t[0].source.tile_index ^= 1;
        let remaining = super::super::resolve("VermilionGym", &t.iter().collect::<Vec<_>>(), &g, None);
        let surviving: Vec<_> = remaining.iter().filter(|p|p.kind_label()=="special:vermilion-north-panel").collect();
        assert_eq!(surviving.len(), 1);
        assert_eq!(surviving[0].column, 12);
    }

    #[test]
    fn fuchsia_wall_keeps_invisible_maze_and_rejects_incomplete_courses() {
        let g=grid(4,4);
        let mut t=tiles(4,4,"lab",0x10);
        for cell in &mut t {cell.source.metatile_id=0x0d;}
        for y in 0..2 {for x in 0..4 {t[y*4+x].source.tile_index=[0x01,0x11][y];}}
        t[12].source.metatile_id=0x1d;t[12].source.tile_index=0x37;
        let refs:Vec<_>=t.iter().collect();
        let selected=super::super::resolve("FuchsiaGym",&refs,&g,None);
        let walls:Vec<_>=selected.iter().filter(|p|p.kind_label()=="special:fuchsia-north-wall").collect();
        assert_eq!(walls.len(),1);
        assert_eq!(walls[0].indices(4).count(),8);
        assert!(walls[0].indices(4).all(|i|i<8));
        let mut mesh=TerrainMeshData{footing_heights:vec![0.;16],..Default::default()};
        super::super::append(&mut mesh,&g,walls[0],&refs,[0,0]);
        assert!(!mesh.solid.indices.is_empty());
        assert!(mesh.solid.positions.iter().all(|p|p[1]>=0. && p[1]<=16.));
        assert_eq!(mesh.footing_heights,vec![0.;16]);
        assert!(!super::super::resolve("ElmsLab",&refs,&g,None).iter()
            .any(|p|p.kind_label()=="special:fuchsia-north-wall"));
        t[4].source.tile_index=0x37;
        assert!(!super::super::resolve("FuchsiaGym",&t.iter().collect::<Vec<_>>(),&g,None).iter()
            .any(|p|p.kind_label()=="special:fuchsia-north-wall"));
    }

    #[test]
    fn blackthorn_channel_uses_own_live_atlas_without_a_floor_sample() {
        let g=grid(4,4);
        let mut t=tiles(4,4,"elite_four_room",0x02);
        let art=[2,2,0x38,2,0x38,2,0x5b,2,2,2,2,2,2,0x5b,2,2];
        for (i,tile) in t.iter_mut().enumerate() { tile.source.metatile_id=0x2b;tile.source.tile_index=art[i]; }
        let refs:Vec<_>=t.iter().collect();
        let selected=super::super::resolve("BlackthornGym1F",&refs,&g,None);
        let lava:Vec<_>=selected.iter().filter(|p|p.kind_label()=="special:blackthorn-lava-channel").collect();
        assert_eq!(lava.len(),16);
        for p in lava {
            assert_eq!(p.ground,p.row*4+p.column,"channel must use its own live atlas slot");
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],..Default::default()};
            super::super::append(&mut mesh,&g,p,&refs,[0,0]);
            assert!(!mesh.solid.indices.is_empty());
            assert!(mesh.solid.positions.iter().all(|v|v[1]<=-2.));
            assert!(mesh.textured.positions.iter().all(|v|(v[1]+1.999).abs()<0.0001));
            let (u0, u1, v0, v1) = g.uv(p.column, p.row);
            assert_eq!(mesh.textured.uvs, [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]);
            assert_eq!(mesh.footing_heights,vec![0.;16]);
        }
        assert!(!super::super::resolve("BrunosRoom",&refs,&g,None).iter()
            .any(|p|p.kind_label()=="special:blackthorn-lava-channel"));
        t[0].source.tile_index=0x01;
        let remaining=super::super::resolve("BlackthornGym1F",&t.iter().collect::<Vec<_>>(),&g,None);
        assert_eq!(remaining.iter().filter(|p|p.kind_label()=="special:blackthorn-lava-channel").count(),15);
    }

    #[test]
    fn blackthorn_distinguishes_stairs_rocks_and_walkable_callback_crossings() {
        let g = grid(8,8);
        let mut t = tiles(8,8,"elite_four_room",0x01);
        for cell in &mut t { cell.source.metatile_id=0x2d; }
        for (anchor,block,phase,art) in [
            ([2,2],0x29,[2,2],[[0x2c,0x2d],[0x3c,0x3d]]),
            ([4,0],0x30,[0,0],[[0x4e,0x4f],[0x5e,0x5f]]),
            ([0,4],0x3b,[0,2],[[0x44,0x45],[0x46,0x47]]),
        ] { for y in 0..2 { for x in 0..2 {
            let s=&mut t[(anchor[1]+y)*8+anchor[0]+x].source;
            s.metatile_id=block;s.subtile_column=(phase[0]+x)as u8;
            s.subtile_row=(phase[1]+y)as u8;s.tile_index=art[y][x];
        }}}
        let refs: Vec<_>=t.iter().collect();
        let selected=super::super::resolve("BlackthornGym1F",&refs,&g,None);
        for (label,anchor) in [("special:blackthorn-stair-flight",[2,2]),
            ("dungeon:cave-boulder",[4,0]),("special:blackthorn-filled-rock-crossing",[0,4])] {
            let p=selected.iter().find(|p|p.kind_label()==label).unwrap();
            assert_eq!([p.column,p.row],anchor);
            assert_eq!(p.indices(8).count(),4);
            let mut mesh=TerrainMeshData {footing_heights:vec![3.;64],..Default::default()};
            super::super::append(&mut mesh,&g,p,&refs,[0,0]);
            assert!(!mesh.solid.indices.is_empty());
            assert_eq!(mesh.footing_heights,vec![3.;64]);
            if label=="special:blackthorn-filled-rock-crossing" {
                assert!(mesh.solid.positions.iter().all(|v|v[1]<=0.));
                assert!(mesh.solid.positions.iter().any(|v|v[1] < -5.));
            }
        }
        assert!(!selected.iter().any(|p|p.kind_label()=="dungeon:cave-boulder" && p.column==2 && p.row==2));
        for y in 4..6 { for x in 0..2 {
            let s=&mut t[y*8+x].source;s.metatile_id=0x2d;s.tile_index=0x01;
        }}
        assert!(!super::super::resolve("BlackthornGym1F",&t.iter().collect::<Vec<_>>(),&g,None).iter()
            .any(|p|p.kind_label()=="special:blackthorn-filled-rock-crossing"));
    }

    #[test]
    fn mahogany_wall_consumes_complete_panels_and_preserves_ice_and_footing() {
        let g = grid(8, 8);
        let mut t = tiles(8, 8, "elite_four_room", 0x01);
        for cell in &mut t { cell.source.metatile_id = 0x2d; }
        for y in 0..4 { for x in 0..4 {
            let cell = &mut t[y * 8 + x];
            cell.source.metatile_id = 0x27;
            cell.source.tile_index = [0x50, 0x24, 0x34, 0x34][y];
        }}
        // A neighboring sliding source must not become part of the enclosure.
        t[4].source.metatile_id = 0x39;
        t[4].source.tile_index = 0x05;
        for y in 0..2 { for x in 0..2 {
            let cell = &mut t[(y+4)*8+x+4];
            cell.source.metatile_id = 0x35;
            cell.source.subtile_column = (x+2) as u8;
            cell.source.subtile_row = y as u8;
            cell.source.tile_index = [[0x2a,0x2b],[0x3a,0x3b]][y][x];
        }}
        let refs: Vec<_> = t.iter().collect();
        let placements = super::super::resolve("MahoganyGym", &refs, &g, None);
        let walls: Vec<_> = placements.iter().filter(|p|p.kind_label()=="special:mahogany-panel-wall").collect();
        assert_eq!(walls.len(), 1);
        assert_eq!(walls[0].indices(8).count(), 16);
        assert!(!walls[0].indices(8).any(|i|i==4));
        let mut mesh = TerrainMeshData { footing_heights: vec![0.; 64], ..Default::default() };
        super::super::append(&mut mesh, &g, walls[0], &refs, [0,0]);
        assert!(!mesh.solid.indices.is_empty());
        assert!(mesh.solid.positions.iter().all(|v|v[1]>=0. && v[1]<=24.));
        assert!(mesh.solid.positions.iter().any(|v|v[1]==24.));
        assert!(mesh.footing_heights.iter().all(|h|*h==0.));
        assert!(mesh.solid.normals.iter().all(|n|(Vec3::from_array(*n).length_squared()-1.).abs()<0.0001));
        let partitions: Vec<_> = placements.iter().filter(|p|p.kind_label()=="special:mahogany-stone-partition").collect();
        assert_eq!(partitions.len(),1);
        assert_eq!(partitions[0].indices(8).count(),4);
        let mut stone = TerrainMeshData { footing_heights: vec![0.;64], ..Default::default() };
        super::super::append(&mut stone,&g,partitions[0],&refs,[0,0]);
        assert!(!stone.solid.indices.is_empty());
        assert!(stone.solid.positions.iter().all(|v|v[1]>=0. && v[1]<=13.));
        assert!(stone.footing_heights.iter().all(|h|*h==0.));
        for map in ["VioletGym", "BlackthornGym1F"] {
            assert!(!super::super::resolve(map, &refs, &g, None).iter()
                .any(|p|p.kind_label()=="special:mahogany-panel-wall"));
        }
        t[0].source.tile_index ^= 1;
        assert!(!super::super::resolve("MahoganyGym", &t.iter().collect::<Vec<_>>(), &g, None).iter()
            .any(|p|p.kind_label()=="special:mahogany-panel-wall"));
        t[0].source.tile_index ^= 1;
        let crop: Vec<_> = t[..24].iter().collect();
        assert!(!super::super::resolve("MahoganyGym", &crop, &grid(8,3), None).iter()
            .any(|p|p.kind_label()=="special:mahogany-panel-wall"));
        t[4*8+4].source.tile_index ^= 1;
        assert!(!super::super::resolve("MahoganyGym", &t.iter().collect::<Vec<_>>(), &g, None).iter()
            .any(|p|p.kind_label()=="special:mahogany-stone-partition"));
    }

    #[test]
    fn cerulean_bridge_face_requires_water_and_deck_has_its_own_material() {
        for (block, tile, label) in [
            (0x26, 0x05, "special:cerulean-brick-bridge"),
            (0x36, 0x0b, "special:cerulean-wood-bridge-face"),
        ] {
            let mut t = tiles(1, 1, "port", tile);
            t[0].source.metatile_id = block;
            if tile == 0x0b {
                let absent = super::super::resolve("CeruleanGym", &t.iter().collect::<Vec<_>>(), &grid(1, 1), None);
                assert!(!absent.iter().any(|p| p.kind_label() == label), "no invented water when the sample is absent");
                let mut water = tiles(1, 1, "port", 0x14).remove(0);
                water.column = 1;
                water.source.metatile_id = 0x2e;
                water.source.subtile_column = 2;
                t.push(water);
                t[1].source.metatile_id = 0xffff;
                let invalid = super::super::resolve("CeruleanGym", &t.iter().collect::<Vec<_>>(), &grid(2, 1), None);
                assert!(!invalid.iter().any(|p| p.kind_label() == label), "water tile number alone is insufficient");
                t[1].source.metatile_id = 0x2e;
            }
            let g = grid(t.len(), 1);
            let p = super::super::resolve("CeruleanGym", &t.iter().collect::<Vec<_>>(), &g, None);
            let bridge = p.iter().find(|p| p.kind_label() == label).unwrap();
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![0.; t.len()];
            super::super::append(&mut mesh, &g, bridge, &t.iter().collect::<Vec<_>>(), [0, 0]);
            assert_eq!(mesh.footing_heights, vec![0.; t.len()]);
            assert!(!mesh.solid.indices.is_empty());
            assert!(mesh.solid.positions.iter().any(|v| v[1] < -2.9));
            if tile == 0x05 {
                assert!(mesh.solid.positions.iter().all(|v| v[1] <= 0.));
                assert!(mesh.textured.indices.is_empty(), "no coplanar floor backing beneath bridge bricks");
            }
            assert!(!super::super::resolve("OlivinePort", &t.iter().collect::<Vec<_>>(), &g, None)
                .iter().any(|p| p.kind_label() == label));
            t[0].source.tile_index = 0x22;
            assert!(cerulean_bridge_detail(&t[0].source).is_none(), "pool foam must remain water artwork");
        }
        let mut gray = tiles(1, 1, "port", 0x0a);
        gray[0].source.metatile_id = 0x31;
        assert!(cerulean_bridge_detail(&gray[0].source).is_none(), "gray perimeter must not become a pink walkway");
    }
    #[test]
    fn floor_volume_stays_below_unchanged_actor_datum() {
        for kind in [Floor::Stone, Floor::Wood, Floor::Ceramic, Floor::Carpet] {
            let mut m = SurfaceMeshData::default();
            floor_tile(&mut m, [0., 8., 0., 8.], kind, 0, 0, 1.);
            assert!(m.positions.iter().all(|p| p[1] <= 0.));
            assert!(m.positions.iter().any(|p| p[1] < -1.));
            assert!(
                m.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
            );
        }
    }
    #[test]
    fn roof_tiles_keep_continuous_slope_endpoints_without_raising_walkway() {
        let mut m = SurfaceMeshData::default();
        roof_tile(&mut m, [0., 8., 0., 8.], -4., -2., 1.);
        roof_tile(&mut m, [8., 16., 0., 8.], -2., 0., 1.);
        assert!(m.positions.iter().all(|p| p[1] <= 0.));
        let shared = m
            .positions
            .iter()
            .filter(|p| p[0] == 8.)
            .collect::<Vec<_>>();
        assert!(shared.iter().any(|p| p[1] == -2.));
        assert!(shared.iter().all(|p| p[1] >= -3. && p[1] <= -2.));
    }
}
