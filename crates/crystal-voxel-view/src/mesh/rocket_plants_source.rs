//! Complete native Rocket B1F plant drawings and their local floor context.
//! Source bindings are independent of rendering, collision, and event state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Identity<'a> {
    pub tileset: &'a str,
    pub metatile: u16,
    pub column: u8,
    pub row: u8,
    pub tile: u16,
}

pub(super) const MAP: &str = "TeamRocketBaseB1F";
const DRAWING: [[u16; 2]; 3] = [[0x1e, 0x1f], [0x2e, 0x2f], [0x3e, 0x3f]];

#[derive(Clone, Copy, Debug)]
pub(super) struct Match {
    /// Complete 4x4 block guard; ownership is only its 2x3 plant drawing.
    pub column: usize,
    pub row: usize,
    metatile: u16,
}
impl Match {
    pub(super) fn plant_column(self) -> usize {
        self.column + if self.metatile == 0x29 { 2 } else { 0 }
    }
    pub(super) fn indices(self, width: usize) -> impl Iterator<Item = usize> {
        (0..3).flat_map(move |y| {
            (0..2).map(move |x| (self.row + y) * width + self.plant_column() + x)
        })
    }
    pub(super) fn guard_indices(self, width: usize) -> impl Iterator<Item = usize> {
        (0..4).flat_map(move |y| (0..4).map(move |x| (self.row + y) * width + self.column + x))
    }
    /// A native 0x10 diamond floor immediately below this very plant.
    pub(super) fn ground(self, width: usize) -> usize {
        (self.row + 3) * width + self.plant_column()
    }
    pub(super) fn coherent(self, cells: &[Identity<'_>], width: usize, height: usize) -> bool {
        width.checked_mul(height) == Some(cells.len())
            && self.column.checked_add(4).is_some_and(|x| x <= width)
            && self.row.checked_add(4).is_some_and(|y| y <= height)
            && (0..4).all(|y| {
                (0..4).all(|x| {
                    let s = &cells[(self.row + y) * width + self.column + x];
                    let offset = if self.metatile == 0x29 { 2 } else { 0 };
                    let tile = if (offset..offset + 2).contains(&x) && y < 3 {
                        DRAWING[y][x - offset]
                    } else {
                        0x10
                    };
                    s.tileset == "underground"
                        && s.metatile == self.metatile
                        && usize::from(s.column) == x
                        && usize::from(s.row) == y
                        && s.tile == tile
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
    if map != MAP
        || width == 0
        || width.checked_mul(height) != Some(cells.len())
        || blocked.len() != cells.len()
    {
        return Vec::new();
    }
    [(0x29, [24, 24]), (0x2a, [28, 24])]
        .into_iter()
        .filter_map(|(metatile, anchor)| {
            let column = usize::try_from(anchor[0] - i64::from(origin[0])).ok()?;
            let row = usize::try_from(anchor[1] - i64::from(origin[1])).ok()?;
            let p = Match {
                column,
                row,
                metatile,
            };
            (p.coherent(cells, width, height) && p.guard_indices(width).all(|i| !blocked[i]))
                .then_some(p)
        })
        .collect()
}
