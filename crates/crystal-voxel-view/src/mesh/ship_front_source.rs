//! Complete lower-deck room fronts, including each opening and corner void.
//! The fingerprints are identity guards, never game-content copies or collision
//! rules. Geometry and source ownership are separate from the guarded rectangle.
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
    West,
    East,
}
impl Kind {
    pub(super) const ALL: [Self; 2] = [Self::West, Self::East];
    pub(super) fn anchor(self) -> [i32; 2] {
        [if self == Self::West { 12 } else { 36 }, 12]
    }
    pub(super) fn fingerprint(self) -> u64 {
        match self {
            Self::West => 0x0b6467b629e9e383,
            Self::East => 0x412b19233c4b0cd3,
        }
    }
    pub(super) fn entrance(self) -> usize {
        if self == Self::West { 8 } else { 12 }
    }
    pub(super) fn owns(self, x: usize, y: usize) -> bool {
        x < 24
            && y < 4
            && !(self.entrance()..self.entrance() + 4).contains(&x)
            && !(y == 1 && (x == 1 || x == 22))
    }
    pub(super) fn label(self) -> &'static str {
        "ship/lower-deck-room-front"
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Match {
    pub kind: Kind,
    pub column: usize,
    pub row: usize,
    pub origin: [i32; 2],
}
impl Match {
    pub(super) fn guard_indices(self, width: usize) -> impl Iterator<Item = usize> {
        (0..4).flat_map(move |y| (0..24).map(move |x| (self.row + y) * width + self.column + x))
    }
    pub(super) fn indices(self, width: usize) -> impl Iterator<Item = usize> {
        (0..4).flat_map(move |y| {
            (0..24).filter_map(move |x| {
                self.kind
                    .owns(x, y)
                    .then_some((self.row + y) * width + self.column + x)
            })
        })
    }
    pub(super) fn coherent(self, cells: &[Identity<'_>], width: usize, height: usize) -> bool {
        if width.checked_mul(height) != Some(cells.len())
            || self.column.checked_add(24).is_none_or(|v| v > width)
            || self.row.checked_add(4).is_none_or(|v| v > height)
            || [
                self.column as i64 + i64::from(self.origin[0]),
                self.row as i64 + i64::from(self.origin[1]),
            ] != self.kind.anchor().map(i64::from)
        {
            return false;
        }
        let mut hash = 0xcbf29ce484222325u64;
        for i in self.guard_indices(width) {
            let s = &cells[i];
            if s.tileset != "lighthouse" || s.column >= 4 || s.row >= 4 {
                return false;
            }
            for byte in [
                s.metatile as u8,
                (s.metatile >> 8) as u8,
                s.column,
                s.row,
                s.tile as u8,
                (s.tile >> 8) as u8,
            ] {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        hash == self.kind.fingerprint()
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
    if map != "FastShipB1F"
        || width.checked_mul(height) != Some(cells.len())
        || blocked.len() != cells.len()
    {
        return Vec::new();
    }
    Kind::ALL
        .into_iter()
        .filter_map(|kind| {
            let anchor = kind.anchor();
            let column = usize::try_from(i64::from(anchor[0]) - i64::from(origin[0])).ok()?;
            let row = usize::try_from(i64::from(anchor[1]) - i64::from(origin[1])).ok()?;
            let p = Match {
                kind,
                column,
                row,
                origin,
            };
            // A custom profile at the doorway or void owns the whole assembly's
            // interpretation, although those cells are never architecture claims.
            (p.coherent(cells, width, height) && !p.guard_indices(width).any(|i| blocked[i]))
                .then_some(p)
        })
        .collect()
}
