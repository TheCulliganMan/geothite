//! Small source motifs only: complete table courses and two joined U bulkheads.
//! This module is dependency-free so its exact selector tests also run via rustc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Asset {
    TeaShort,
    TeaMedium,
    TeaLong,
    TeaMess,
    CaptainsDesk,
    BulkheadU,
    WallTea,
    CaptainsChair,
    StoolEast,
    StoolWest,
    FullBunk,
    JoinedBunk,
}
pub(super) type Identity = (u16, u8, u8, u16);
impl Asset {
    pub(super) const ALL: [Self; 12] = [
        Self::TeaShort,
        Self::TeaMedium,
        Self::TeaLong,
        Self::TeaMess,
        Self::CaptainsDesk,
        Self::BulkheadU,
        Self::WallTea,
        Self::CaptainsChair,
        Self::StoolEast,
        Self::StoolWest,
        Self::FullBunk,
        Self::JoinedBunk,
    ];
    pub(super) fn size(self) -> (usize, usize, f32) {
        match self {
            Self::TeaShort => (4, 4, 14.),
            Self::TeaMedium => (4, 6, 17.),
            Self::TeaLong => (4, 8, 17.),
            Self::TeaMess => (4, 10, 17.),
            Self::CaptainsDesk => (6, 4, 14.),
            Self::BulkheadU => (24, 16, 16.),
            Self::WallTea => (4, 3, 14.),
            Self::CaptainsChair => (2, 3, 24.),
            Self::StoolEast | Self::StoolWest => (2, 2, 7.),
            Self::FullBunk | Self::JoinedBunk => (2, 4, 7.),
        }
    }
    pub(super) fn claim(self, x: usize, y: usize) -> bool {
        let (w, h, _) = self.size();
        x < w && y < h && (self != Self::BulkheadU || y >= 14 || x < 2 || x >= 22)
    }
    pub(super) fn expected(self, x: usize, y: usize) -> Option<Identity> {
        let (w, h, _) = self.size();
        if x >= w || y >= h {
            return None;
        }
        if matches!(self, Self::StoolEast | Self::StoolWest) {
            let (block, offset) = if self == Self::StoolEast { (0x07, 2) } else { (0x08, 0) };
            return Some((block, (x + offset) as u8, (y + 2) as u8,
                [[0x07, 0x08], [0x17, 0x18]][y][x]));
        }
        if matches!(self, Self::FullBunk | Self::JoinedBunk) {
            // The wall-side berth starts below $38's wall and finishes in
            // $39. Both source halves must match before either is claimed.
            let (block, row) = if self == Self::JoinedBunk {
                (if y < 2 { 0x38 } else { 0x39 }, (y + 2) % 4)
            } else {
                (0x36, y)
            };
            return Some((block, (x + 2) as u8, row as u8,
                [[0x46, 0x47], [0x56, 0x57], [0x83, 0x84], [0x85, 0x86]][y][x]));
        }
        if self == Self::WallTea {
            return Some((
                0x37,
                x as u8,
                (y + 1) as u8,
                [
                    [0x09, 0x0a, 0x0a, 0x0c],
                    [0x14, 0x1a, 0x82, 0x35],
                    [0x3b, 0x3c, 0x3b, 0x3c],
                ][y][x],
            ));
        }
        if self == Self::CaptainsChair {
            return Some((
                0x30,
                (x + 2) as u8,
                (y + 1) as u8,
                [[0x54, 0x55], [0x42, 0x43], [0x17, 0x18]][y][x],
            ));
        }
        if self == Self::BulkheadU {
            let block = if y < 12 {
                if x < 4 {
                    0x20
                } else if x >= 20 {
                    0x21
                } else {
                    return None;
                }
            } else if x < 4 {
                0x22
            } else if x >= 20 {
                0x23
            } else {
                0x25
            };
            return Some((
                block,
                (x % 4) as u8,
                (y % 4) as u8,
                divider_tile(block, x % 4, y % 4)?,
            ));
        }
        if self == Self::CaptainsDesk {
            let rows = [
                [0x09, 0x0a, 0x0a, 0x0a, 0x0a, 0x0c],
                [0x19, 0x2c, 0x44, 0x45, 0x2c, 0x1c],
                [0x14, 0x82, 0x82, 0x82, 0x82, 0x35],
                [0x3b, 0x3c, 0x0b, 0x0b, 0x3b, 0x3c],
            ];
            return Some((
                if x < 4 { 0x32 } else { 0x33 },
                (x % 4) as u8,
                y as u8,
                rows[y][x],
            ));
        }
        if y < 2 {
            return Some((
                0x06,
                x as u8,
                (y + 2) as u8,
                [[0x09, 0x0a, 0x0a, 0x0c], [0x19, 0x1a, 0x2c, 0x1c]][y][x],
            ));
        }
        let long = matches!(self, Self::TeaLong | Self::TeaMess);
        if long && y < 6 {
            return Some((
                0x2a,
                x as u8,
                (y - 2) as u8,
                [
                    [0x19, 0x40, 0x41, 0x1c],
                    [0x19, 0x50, 0x51, 0x1c],
                    [0x19, 0x2c, 0x1a, 0x1c],
                    [0x19, 0x2c, 0x2c, 0x1c],
                ][y - 2][x],
            ));
        }
        let local = y - if long { 6 } else { 2 };
        let short = matches!(self, Self::TeaShort | Self::TeaLong);
        let rows = if short {
            [
                [0x14, 0x82, 0x82, 0x35],
                [0x0b, 0x80, 0x81, 0x0b],
                [0; 4],
                [0; 4],
            ]
        } else {
            [
                [0x19, 0x40, 0x41, 0x1c],
                [0x19, 0x50, 0x51, 0x1c],
                [0x14, 0x82, 0x82, 0x35],
                [0x0b, 0x80, 0x81, 0x0b],
            ]
        };
        Some((
            if short { 0x2c } else { 0x2d },
            x as u8,
            local as u8,
            rows[local][x],
        ))
    }
}
fn divider_tile(block: u16, x: usize, y: usize) -> Option<u16> {
    let floor = if (x + y) % 2 == 0 { 0x0d } else { 0x1d };
    Some(match block {
        0x20 => {
            if x < 2 {
                0x15 + x as u16
            } else {
                floor
            }
        }
        0x21 => {
            if x >= 2 {
                0x15 + (x - 2) as u16
            } else {
                floor
            }
        }
        0x22 => {
            if y < 2 {
                if x < 2 { 0x15 + x as u16 } else { floor }
            } else {
                [[0x15, 0x26, 0x33, 0x33], [0x32, 0x11, 0x11, 0x11]][y - 2][x]
            }
        }
        0x23 => {
            if y < 2 {
                if x >= 2 { 0x15 + (x - 2) as u16 } else { floor }
            } else {
                [[0x33, 0x33, 0x25, 0x16], [0x11, 0x11, 0x11, 0x22]][y - 2][x]
            }
        }
        0x25 => {
            if y < 2 {
                floor
            } else if y == 2 {
                0x33
            } else {
                0x11
            }
        }
        _ => return None,
    })
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
pub(super) fn complete(
    asset: Asset,
    width: usize,
    height: usize,
    column: usize,
    row: usize,
    at: impl Fn(usize, usize) -> Option<Identity>,
) -> bool {
    let (w, h, _) = asset.size();
    if column.checked_add(w).is_none_or(|v| v > width)
        || row.checked_add(h).is_none_or(|v| v > height)
    {
        return false;
    }
    (0..h).all(|y| {
        (0..w).all(|x| {
            asset
                .expected(x, y)
                .is_none_or(|s| at(column + x, row + y) == Some(s))
        })
    })
}
#[cfg(test)]
pub(super) fn bulkhead_contains(x: f32, z: f32) -> bool {
    (0.0..=192.001).contains(&x)
        && (0.0..=128.001).contains(&z)
        && (z >= 124.999
            || x <= 8.001
            || x >= 183.999
            || (z <= 112.001 && (x <= 16.001 || x >= 175.999)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_ship_pictures_reject_every_identity_field_change_and_crop() {
        for a in Asset::ALL {
            let (w, h, _) = a.size();
            let at = |x, y| {
                a.expected(x, y)
                    .or(Some((0x0b, (x % 4) as u8, (y % 4) as u8, 0x0d)))
            };
            assert!(complete(a, w, h, 0, 0, at));
            assert!(!complete(a, w - 1, h, 0, 0, at));
            assert!(!complete(a, w, h - 1, 0, 0, at));
            for y in 0..h {
                for x in 0..w {
                    if a.expected(x, y).is_none() {
                        continue;
                    }
                    for field in 0..5 {
                        let change = |xx, yy| {
                            let mut s = at(xx, yy)?;
                            if (xx, yy) == (x, y) {
                                match field {
                                    0 => s.0 ^= 1,
                                    1 => s.1 ^= 1,
                                    2 => s.2 ^= 1,
                                    3 => s.3 ^= 1,
                                    _ => return None,
                                }
                            }
                            Some(s)
                        };
                        assert!(
                            !complete(a, w, h, 0, 0, change),
                            "{a:?} {x},{y} field {field}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn bulkhead_has_exact_120_vertical_88_horizontal_claims_for_two_rooms() {
        let mut v = 0;
        let mut h = 0;
        for y in 0..16 {
            for x in 0..24 {
                if !Asset::BulkheadU.claim(x, y) {
                    continue;
                }
                if y < 14 || x == 0 || x == 23 {
                    v += 2;
                } else {
                    h += 2;
                }
            }
        }
        assert_eq!((v, h), (120, 88));
        assert!(!Asset::BulkheadU.claim(12, 0));
        assert!(!Asset::BulkheadU.claim(12, 13));
        assert!(applies("FastShipB1F"));
        assert!(!applies("FastShipB1FBeta"));
        assert!(!applies("OlivineLighthouse6F"));
    }
}
