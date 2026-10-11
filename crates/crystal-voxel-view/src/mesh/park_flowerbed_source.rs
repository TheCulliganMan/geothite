//! Complete native flowerbed: left end, repeated center blocks, right end.
const LEFT: [u16; 16] = [
    0x37, 0x38, 0x38, 0x38, 0x47, 3, 1, 3, 0x47, 3, 3, 3, 0x57, 0x58, 0x58, 0x58,
];
const MIDDLE: [u16; 16] = [
    0x38, 0x38, 0x38, 0x38, 3, 3, 1, 3, 3, 1, 3, 3, 0x58, 0x58, 0x58, 0x58,
];
const RIGHT: [u16; 16] = [
    0x38, 0x38, 0x38, 0x39, 1, 3, 3, 0x49, 3, 1, 3, 0x49, 0x58, 0x58, 0x58, 0x59,
];

pub(super) fn border(x: usize, y: usize, width: usize) -> bool {
    x < width && y < 4 && (x == 0 || x + 1 == width || y == 0 || y == 3)
}
pub(super) fn matches_cell(width: usize, x: usize, y: usize, s: (&str, u16, u8, u8, u16)) -> bool {
    if width < 8 || width % 4 != 0 || x >= width || y >= 4 {
        return false;
    }
    let (block, tiles) = if x < 4 {
        (0x3c, LEFT)
    } else if x + 4 >= width {
        (0x3e, RIGHT)
    } else {
        (0x3d, MIDDLE)
    };
    s == ("park", block, (x % 4) as u8, y as u8, tiles[y * 4 + x % 4])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn park_flowerbed_guard_keeps_all_flowers_outside_border_ownership() {
        for width in [8, 12, 20] {
            let mut count = 0;
            for y in 0..4 {
                for x in 0..width {
                    let (block, tiles) = if x < 4 {
                        (0x3c, LEFT)
                    } else if x + 4 >= width {
                        (0x3e, RIGHT)
                    } else {
                        (0x3d, MIDDLE)
                    };
                    let s = ("park", block, (x % 4) as u8, y as u8, tiles[y * 4 + x % 4]);
                    assert!(matches_cell(width, x, y, s));
                    assert_eq!(border(x, y, width), !matches!(s.4, 1 | 3));
                    count += usize::from(border(x, y, width));
                    for wrong in [
                        ("johto", s.1, s.2, s.3, s.4),
                        (s.0, s.1 ^ 1, s.2, s.3, s.4),
                        (s.0, s.1, s.2 ^ 1, s.3, s.4),
                        (s.0, s.1, s.2, s.3 ^ 1, s.4),
                        (s.0, s.1, s.2, s.3, s.4 ^ 1),
                    ] {
                        assert!(!matches_cell(width, x, y, wrong));
                    }
                }
            }
            assert_eq!(count, width * 2 + 4);
        }
        assert!(!matches_cell(19, 0, 0, ("park", 0x3c, 0, 0, 0x37)));
        assert!(!matches_cell(20, 20, 0, ("park", 0x3e, 0, 0, 0x38)));
    }
}
