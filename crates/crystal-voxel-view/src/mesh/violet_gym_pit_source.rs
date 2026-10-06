//! Source proof for Violet's two connected recesses. Collision is never input
//! to the matcher: $19's drawn lip/descending shade and $17's void establish
//! the shape. Only the already blocked half of $19 and the void are owned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Identity<'a> {
    pub tileset: &'a str,
    pub metatile: u16,
    pub column: u8,
    pub row: u8,
    pub tile: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Edge {
    pub cell: usize,
    /// North, east, south, west; normals point into the recess.
    pub side: usize,
    /// A rim needs source evidence for the adjoining platform. Missing canvas
    /// data and the map's unlit outer void never invent a coping/walkway.
    pub rim: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Plan {
    pub cells: Vec<usize>,
    pub edges: Vec<Edge>,
}
fn expected(block: u16, row: usize) -> Option<u16> {
    match block {
        0x17 => Some(0x00),
        0x19 => Some([0x01, 0x01, 0x13, 0x12][row]),
        _ => None,
    }
}
fn complete(cells: &[Identity<'_>], width: usize, height: usize, x: usize, y: usize) -> bool {
    if x + 4 > width || y + 4 > height {
        return false;
    }
    let block = cells[y * width + x].metatile;
    (0..4).all(|dy| {
        (0..4).all(|dx| {
            let s = cells[(y + dy) * width + x + dx];
            s.tileset == "elite_four_room"
                && s.metatile == block
                && usize::from(s.column) == dx
                && usize::from(s.row) == dy
                && expected(block, dy) == Some(s.tile)
        })
    })
}
fn platform(s: &Identity<'_>) -> bool {
    if s.tileset != "elite_four_room" || s.column >= 4 || s.row >= 4 {
        return false;
    }
    match s.metatile {
        0x19 => s.row < 2 && s.tile == 0x01,
        0x2d => s.tile == 0x01,
        // The south end meets the two source plaque/statue blocks. They have
        // actual backing at the platform datum; keep their models untouched.
        0x33 if s.row < 2 => {
            s.tile == [[0x55, 0x56], [0x57, 0x58]][s.row as usize][s.column as usize % 2]
        }
        0x33 => s.tile == 0x01,
        0x1d | 0x1e => {
            let statue = if s.metatile == 0x1d {
                s.column < 2
            } else {
                s.column >= 2
            };
            s.tile
                == if statue {
                    [[0x20, 0x21], [0x30, 0x31], [0x22, 0x23], [0x32, 0x33]][s.row as usize]
                        [s.column as usize % 2]
                } else {
                    0x01
                }
        }
        _ => false,
    }
}
pub(super) fn resolve(
    map: &str,
    cells: &[Identity<'_>],
    width: usize,
    height: usize,
    reserved: &[bool],
) -> Plan {
    let mut out = Plan {
        cells: Vec::new(),
        edges: Vec::new(),
    };
    if map != "VioletGym"
        || width == 0
        || width.checked_mul(height) != Some(cells.len())
        || reserved.len() != cells.len()
    {
        return out;
    }
    let mut owned = vec![false; cells.len()];
    for y in 0..height {
        for x in 0..width {
            if !complete(cells, width, height, x, y) {
                continue;
            }
            // The complete native drawing, including its unclaimed walkable half,
            // must remain intact. A custom floor/edge override owns that drawing.
            if (0..4).any(|dy| (0..4).any(|dx| reserved[(y + dy) * width + x + dx])) {
                continue;
            }
            let first = if cells[y * width + x].metatile == 0x19 {
                2
            } else {
                0
            };
            for dy in first..4 {
                for dx in 0..4 {
                    owned[(y + dy) * width + x + dx] = true;
                }
            }
        }
    }
    for i in 0..cells.len() {
        if !owned[i] {
            continue;
        }
        out.cells.push(i);
        let (x, y) = (i % width, i / width);
        let neighbors = [
            y.checked_sub(1).map(|y| y * width + x),
            (x + 1 < width).then_some(i + 1),
            (y + 1 < height).then_some(i + width),
            x.checked_sub(1).map(|x| y * width + x),
        ];
        for (side, other) in neighbors.into_iter().enumerate() {
            if other.is_some_and(|j| owned[j]) {
                continue;
            }
            let rim = other.is_some_and(|j| !reserved[j] && platform(&cells[j]));
            out.edges.push(Edge { cell: i, side, rim });
        }
    }
    out
}
