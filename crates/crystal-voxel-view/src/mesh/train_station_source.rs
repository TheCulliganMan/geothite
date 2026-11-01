//! Exact complete station train drawing, including both native doors and the
//! entire adjoining platform. This module never writes game/controller state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Identity<'a> {
    pub tileset: &'a str,
    pub metatile: u16,
    pub column: u8,
    pub row: u8,
    pub tile: u16,
}
pub(super) const WIDTH: usize = 20;
pub(super) const HEIGHT: usize = 8;
const ANCHOR: [i32; 2] = [8, 8];
const FINGERPRINT: u64 = 0x0daee1e9ffd85dd6;
// Each station: 60 body cells, 8 retained live door cells, 12 retained rail
// backing cells. The four platform rows are guards only, never ownership.
const BODY_ROWS: [u32; 4] = [0x7fffe, 0xfffff, 0xf3fcf, 0xa2045];
pub(super) fn is_map(map: &str) -> bool {
    matches!(
        map,
        "GoldenrodMagnetTrainStation" | "SaffronMagnetTrainStation"
    )
}
pub(super) fn valid_ground(s: &Identity<'_>) -> bool {
    s.tileset == "train_station"
        && s.metatile == 0x14
        && s.row == 0
        && s.column < 4
        && s.tile == 0x1f
}
#[derive(Clone, Debug)]
pub(super) struct Match {
    pub column: usize,
    pub row: usize,
    pub ground: usize,
}
impl Match {
    pub(super) fn owns(&self, x: usize, y: usize) -> bool {
        x < WIDTH && y < 4 && BODY_ROWS[y] & (1 << x) != 0
    }
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..4).flat_map(move |y| {
            (0..WIDTH).filter_map(move |x| {
                self.owns(x, y)
                    .then_some((self.row + y) * width + self.column + x)
            })
        })
    }
    pub(super) fn guard_indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..HEIGHT)
            .flat_map(move |y| (0..WIDTH).map(move |x| (self.row + y) * width + self.column + x))
    }
    pub(super) fn coherent(&self, cells: &[Identity<'_>], width: usize, height: usize) -> bool {
        if width.checked_mul(height) != Some(cells.len())
            || self.column.checked_add(WIDTH).is_none_or(|v| v > width)
            || self.row.checked_add(HEIGHT).is_none_or(|v| v > height)
            || self.ground >= cells.len()
            || !valid_ground(&cells[self.ground])
        {
            return false;
        }
        let mut hash = 0xcbf29ce484222325u64;
        for i in self.guard_indices(width) {
            let s = cells[i];
            if s.tileset != "train_station" || s.column >= 4 || s.row >= 4 {
                return false;
            }
            for byte in [
                (s.metatile & 255) as u8,
                (s.metatile >> 8) as u8,
                s.column,
                s.row,
                (s.tile & 255) as u8,
                (s.tile >> 8) as u8,
            ] {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        hash == FINGERPRINT
    }
}
pub(super) fn resolve(
    map: &str,
    cells: &[Identity<'_>],
    width: usize,
    height: usize,
    origin: [i32; 2],
    reserved: &[bool],
) -> Vec<Match> {
    if !is_map(map)
        || width == 0
        || width.checked_mul(height) != Some(cells.len())
        || cells.len() != reserved.len()
    {
        return Vec::new();
    }
    let x = i64::from(ANCHOR[0]) - i64::from(origin[0]);
    let y = i64::from(ANCHOR[1]) - i64::from(origin[1]);
    if x < 0 || y < 0 {
        return Vec::new();
    }
    let Some(ground) = cells.iter().position(valid_ground) else {
        return Vec::new();
    };
    let matched = Match {
        column: x as usize,
        row: y as usize,
        ground,
    };
    if !matched.coherent(cells, width, height)
        || reserved[ground]
        || matched.guard_indices(width).any(|i| reserved[i])
    {
        return Vec::new();
    }
    vec![matched]
}
