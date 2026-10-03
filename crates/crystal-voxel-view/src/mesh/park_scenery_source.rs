//! Complete park fixtures. This module never reads or mutates collision or events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Identity<'a> {
    pub tileset: &'a str,
    pub metatile: u16,
    pub column: u8,
    pub row: u8,
    pub tile: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    LitterBin,
    PedestalFountain,
    PondBasin,
}
impl Kind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::LitterBin => "park/litter-bin",
            Self::PedestalFountain => "park/pedestal-fountain",
            Self::PondBasin => "park/pond-basin",
        }
    }
    pub(super) fn asset_index(self) -> usize {
        match self {
            Self::LitterBin => 0,
            Self::PedestalFountain => 1,
            Self::PondBasin => 2,
        }
    }
}
// Hand-reviewed whole metatiles, including all native backing and spray cells.
const BIN: [u16; 16] = [
    0x5a, 0x5b, 0x00, 0x16, 0x13, 0x82, 0x06, 0x00, 0x00, 0x16, 0x00, 0x16, 0x06, 0x00, 0x06, 0x00,
];
const DRINK: [u16; 16] = [
    0x00, 0x4c, 0x4d, 0x4e, 0x06, 0x5c, 0x5d, 0x5e, 0x00, 0x16, 0x00, 0x16, 0x06, 0x00, 0x06, 0x00,
];
const POND_W: [u16; 16] = [
    0x14, 0x14, 0x14, 0x5f, 0x14, 0x14, 0x5f, 0x80, 0x14, 0x14, 0x5f, 0x90, 0x14, 0x14, 0x14, 0x5f,
];
const POND_E: [u16; 16] = [
    0x5f, 0x14, 0x14, 0x14, 0x81, 0x5f, 0x14, 0x14, 0x91, 0x5f, 0x14, 0x14, 0x5f, 0x14, 0x14, 0x14,
];
#[derive(Clone, Copy, Debug)]
pub(super) struct Match {
    pub column: usize,
    pub row: usize,
    pub kind: Kind,
}
impl Match {
    pub(super) fn guard_width(self) -> usize {
        if self.kind == Kind::PondBasin { 8 } else { 4 }
    }
    pub(super) fn guard_indices(self, width: usize) -> impl Iterator<Item = usize> {
        (0..4).flat_map(move |y| {
            (0..self.guard_width()).map(move |x| (self.row + y) * width + self.column + x)
        })
    }
    pub(super) fn indices(self, width: usize) -> impl Iterator<Item = usize> {
        let (x, y, w) = match self.kind {
            Kind::LitterBin => (0, 0, 2),
            Kind::PedestalFountain => (1, 0, 3),
            Kind::PondBasin => (3, 1, 2),
        };
        (0..2).flat_map(move |dy| {
            (0..w).map(move |dx| (self.row + y + dy) * width + self.column + x + dx)
        })
    }
    pub(super) fn ground(self, local: usize, width: usize) -> usize {
        // Same source block and palette, preserving the plaza's staggered detail.
        let x = match self.kind {
            Kind::LitterBin => local % 2,
            Kind::PedestalFountain => 1 + local % 3,
            Kind::PondBasin => 0,
        };
        let y = match self.kind {
            Kind::LitterBin => 2 + local / 2,
            Kind::PedestalFountain => 2 + local / 3,
            Kind::PondBasin => 0,
        };
        (self.row + y) * width + self.column + x
    }
    pub(super) fn coherent(self, cells: &[Identity<'_>], width: usize, height: usize) -> bool {
        if width.checked_mul(height) != Some(cells.len())
            || self.column + self.guard_width() > width
            || self.row + 4 > height
        {
            return false;
        }
        (0..4).all(|y| {
            (0..self.guard_width()).all(|x| {
                let s = cells[(self.row + y) * width + self.column + x];
                let (block, art) = match self.kind {
                    Kind::LitterBin => (0x0f, &BIN),
                    Kind::PedestalFountain => (0x2f, &DRINK),
                    Kind::PondBasin => {
                        if x < 4 {
                            (0x3f, &POND_W)
                        } else {
                            (0x33, &POND_E)
                        }
                    }
                };
                s.tileset == "park"
                    && s.metatile == block
                    && s.column as usize == x % 4
                    && s.row as usize == y
                    && s.tile == art[y * 4 + x % 4]
            })
        })
    }
}
pub(super) fn resolve(
    map: &str,
    cells: &[Identity<'_>],
    width: usize,
    height: usize,
    origin: [i32; 2],
    blocked: &[bool],
) -> Vec<Match> {
    if !matches!(
        map,
        "NationalPark" | "NationalParkBugContest" | "SafariZoneBeta"
    ) || width.checked_mul(height) != Some(cells.len())
        || blocked.len() != cells.len()
    {
        return vec![];
    }
    let mut result = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if (x as i64 + origin[0] as i64).rem_euclid(4) != 0
                || (y as i64 + origin[1] as i64).rem_euclid(4) != 0
            {
                continue;
            }
            let s = cells[y * width + x];
            let kind = match s.metatile {
                0x0f => Kind::LitterBin,
                0x2f => Kind::PedestalFountain,
                0x3f if map != "SafariZoneBeta" => Kind::PondBasin,
                _ => continue,
            };
            let p = Match {
                column: x,
                row: y,
                kind,
            };
            if p.coherent(cells, width, height) && p.guard_indices(width).all(|i| !blocked[i]) {
                result.push(p);
            }
        }
    }
    result
}
