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
    PassageNetwork { open: [bool; 4] },
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

pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    let (map, cells, g) = (r.map, r.cells, r.g);
    let Some(first) = cells.first() else {
        return;
    };
    let ts = first.source.tileset_id.as_ref();
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
    if passage(map) && ts == "underground" {
        if let Some(floor) = ground(map, cells, ts, 0x10) {
            // Same exact source vocabulary as the established underground
            // boundary; no teleport pad, stair or floor cell is a wall.
            let wall: Vec<_> = cells
                .iter()
                .map(|t| {
                    let s = &t.source;
                    s.tileset_id.as_ref() == ts
                        && matches!(s.metatile_id,0x08..=0x0e|0x10..=0x14|0x25|0x26|0x2c|0x2d|0x2f)
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
                    Detail::PassageNetwork { open },
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
    } else {
        0.
    };
    if !matches!(
        detail,
        Detail::Floor(_)
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
        Detail::PassageNetwork { open } => {
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
        super::super::append(&mut mesh, &g, p, &cells);
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
