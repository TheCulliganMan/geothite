//! Hand-reviewed complete park side borders, including corner and lawn backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::mesh) enum Turn {
    North,
    South,
}

pub(super) fn drawing(block: u16) -> Option<([u16; 16], bool, Option<Turn>)> {
    Some(match block {
        0x10 => (
            [
                5, 0x2b, 0x25, 0x26, 5, 0x1b, 0x35, 0x36, 5, 0x43, 1, 1, 5, 0x53, 1, 1,
            ],
            false,
            Some(Turn::North),
        ),
        0x12 => (
            [
                0x25, 0x26, 0x2b, 5, 0x35, 0x36, 0x1b, 5, 1, 1, 0x43, 5, 1, 1, 0x53, 5,
            ],
            true,
            Some(Turn::North),
        ),
        0x14 => (
            [5, 0x1b, 1, 1, 5, 0x43, 1, 1, 5, 0x43, 1, 1, 5, 0x53, 1, 1],
            false,
            None,
        ),
        0x16 => (
            [1, 1, 0x1b, 5, 1, 1, 0x43, 5, 1, 1, 0x43, 5, 1, 1, 0x53, 5],
            true,
            None,
        ),
        0x18 => (
            [
                5, 0x1b, 1, 1, 5, 0x43, 1, 1, 5, 0x53, 0x23, 0x24, 5, 0x2b, 0x33, 0x34,
            ],
            false,
            Some(Turn::South),
        ),
        0x1a => (
            [
                1, 1, 0x1b, 5, 1, 1, 0x43, 5, 0x23, 0x24, 0x53, 5, 0x33, 0x34, 0x2b, 5,
            ],
            true,
            Some(Turn::South),
        ),
        _ => return None,
    })
}

pub(super) fn matches_cell(
    block: u16,
    x: usize,
    y: usize,
    source: (&str, u16, u8, u8, u16),
) -> bool {
    drawing(block).is_some_and(|(tiles, _, _)| {
        x < 4 && y < 4 && source == ("park", block, x as u8, y as u8, tiles[y * 4 + x])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn park_side_fence_guards_cover_every_corner_and_backing_cell() {
        for block in [0x10, 0x12, 0x14, 0x16, 0x18, 0x1a] {
            let (tiles, east, turn) = drawing(block).unwrap();
            assert_eq!(east, matches!(block, 0x12 | 0x16 | 0x1a));
            assert_eq!(turn.is_none(), matches!(block, 0x14 | 0x16));
            for y in 0..4 {
                for x in 0..4 {
                    let s = ("park", block, x as u8, y as u8, tiles[y * 4 + x]);
                    assert!(matches_cell(block, x, y, s));
                    for wrong in [
                        ("johto", s.1, s.2, s.3, s.4),
                        (s.0, s.1 ^ 1, s.2, s.3, s.4),
                        (s.0, s.1, s.2 ^ 1, s.3, s.4),
                        (s.0, s.1, s.2, s.3 ^ 1, s.4),
                        (s.0, s.1, s.2, s.3, s.4 ^ 1),
                    ] {
                        assert!(!matches_cell(block, x, y, wrong));
                    }
                }
            }
        }
        assert!(drawing(0x15).is_none());
        assert!(!matches_cell(0x14, 4, 0, ("park", 0x14, 4, 0, 5)));
    }
}
