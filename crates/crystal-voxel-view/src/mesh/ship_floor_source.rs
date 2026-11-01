// Exact native ship floor vocabulary, kept dependency-free for pack checks.
// The 1F timber deck already has its own material; no lighthouse/shared-atlas
// room, void, doorway, ladder, furniture picture or custom source is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Style {
    Cabin,
    Mess,
    Deck,
    Border,
}
impl Style {
    pub(super) fn palette(self) -> [f32; 3] {
        match self {
            Self::Cabin => [0.55, 0.47, 0.43],
            Self::Mess => [0.64, 0.62, 0.55],
            Self::Deck => [0.36, 0.43, 0.44],
            Self::Border => [0.25, 0.34, 0.37],
        }
    }
}
pub(super) fn applies(map: &str) -> bool {
    matches!(
        map,
        "FastShipB1F"
            | "FastShipCabins_NNW_NNE_NE"
            | "FastShipCabins_SE_SSE_CaptainsCabin"
            | "FastShipCabins_SW_SSW_NW"
    )
}
pub(super) fn style(
    map: &str,
    tileset: &str,
    block: u16,
    x: u8,
    y: u8,
    tile: u16,
) -> Option<Style> {
    if !applies(map) || tileset != "lighthouse" || x >= 4 || y >= 4 {
        return None;
    }
    let phase = y * 4 + x;
    let odd = (x + y) % 2 != 0;
    if map == "FastShipB1F" {
        let mask = match block {
            0x27 => 0xffffu16,
            0x2b => 0xfff0,
            0x3a => 0x33ff, // Southeast ladder keeps its original art.
            _ => 0,
        };
        if mask & (1 << phase) != 0 && tile == if odd { 0x2f } else { 0x2e } {
            return Some(Style::Deck);
        }
        if block == 0x2b && y == 0 && tile == if x == 0 { 0x1b } else { 0x3f } {
            return Some(Style::Border);
        }
    }
    // Every mask is only exposed $0d/$1d checker, including floor beside
    // bunks, stools, tea tables and partition walls. Match the map as well as
    // atlas/block/phase; the shared lighthouse atlas is not a room blueprint.
    let lower = map == "FastShipB1F";
    let northwest = map == "FastShipCabins_NNW_NNE_NE";
    let captain = map == "FastShipCabins_SE_SSE_CaptainsCabin";
    let southwest = map == "FastShipCabins_SW_SSW_NW";
    let mask: u16 = match block {
        0x0b => 0xffff,
        0x06 => 0x00ff,
        0x07 => 0x33ff,
        0x08 => 0xccff,
        0x36 => 0x3333,
        0x35 if !captain => 0x33ff,
        0x0a if !lower => 0xff00,
        0x09 if northwest || southwest => 0xff00,
        0x2c if northwest || captain => 0xff00,
        0x1b if captain || southwest => 0x00ff, // South warp carpet is excluded.
        0x38 if !lower => 0x3300,
        0x39 if !lower => 0xff33,
        0x30 if captain => 0x3300,
        0x33 if captain => 0xcccc,
        0x34 if southwest => 0xcc00,
        0x20 if lower => 0xcccc,
        0x21 if lower => 0x3333,
        0x22 if lower => 0x00cc,
        0x23 if lower => 0x0033,
        0x25 if lower => 0x00ff,
        _ => return None,
    };
    (mask & (1 << phase) != 0 && tile == if odd { 0x1d } else { 0x0d }).then_some(if lower {
        Style::Mess
    } else {
        Style::Cabin
    })
}

// Floor beneath these older modeled fixtures is safe only after the complete
// runtime resolver, exact source identity AND successful ownership all agree.
// Doorways and walls are intentionally outside this surface-only addition.
pub(super) fn fixture_source(
    map: &str,
    label: &str,
    tileset: &str,
    block: u16,
    x: u8,
    y: u8,
    tile: u16,
) -> bool {
    if !applies(map) || tileset != "lighthouse" || x >= 4 || y >= 4 {
        return false;
    }
    let storage = matches!(
        map,
        "FastShipCabins_NNW_NNE_NE" | "FastShipCabins_SE_SSE_CaptainsCabin"
    );
    match label {
        "dungeon:ship-barrel" => {
            let local = match block {
                0x2f if storage && x < 2 && y >= 2 => Some((x, y - 2)),
                0x35 if map != "FastShipCabins_SE_SSE_CaptainsCabin" && x >= 2 && y >= 2 => {
                    Some((x - 2, y - 2))
                }
                _ => None,
            };
            local.is_some_and(|(x, y)| tile == [[0x48, 0x49], [0x58, 0x59]][y as usize][x as usize])
        }
        "dungeon:ship-rack" if storage && block == 0x2f && x >= 2 => {
            tile == [[0x13, 0x3e], [0x36, 0x2b], [0x36, 0x2b], [0x3b, 0x3c]][y as usize]
                [(x - 2) as usize]
        }
        "dungeon:ship-bunk" if map != "FastShipB1F" && block == 0x38 && x >= 2 && y >= 2 => {
            tile == [[0x46, 0x47], [0x56, 0x57]][(y - 2) as usize][(x - 2) as usize]
        }
        _ => false,
    }
}
