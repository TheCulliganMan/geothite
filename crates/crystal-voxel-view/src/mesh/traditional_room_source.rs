//! Exact immutable source bindings; render-only, no collision/event mutation.
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
    WiseCourse,
    WiseJoint,
    WiseReturn,
    BarnRail,
    Theater,
}
#[derive(Clone, Debug)]
pub(super) struct Match {
    pub column: usize,
    pub row: usize,
    pub width: usize,
    pub height: usize,
    pub kind: Kind,
    pub ground: usize,
    anchor: [i32; 2],
    map: &'static str,
}
impl Match {
    pub(super) fn owns(&self, x: usize, y: usize) -> bool {
        x < self.width
            && y < self.height
            && match self.kind {
                Kind::WiseCourse => y < 2,
                Kind::WiseJoint => y < 2 || x >= 2,
                Kind::WiseReturn => x >= 2 && y >= 2,
                Kind::BarnRail => y >= 2,
                Kind::Theater => true,
            }
    }
    pub(super) fn indices(&self, w: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).filter_map(move |x| {
                self.owns(x, y)
                    .then_some((self.row + y) * w + self.column + x)
            })
        })
    }
    pub(super) fn guard_indices(&self, w: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height)
            .flat_map(move |y| (0..self.width).map(move |x| (self.row + y) * w + self.column + x))
    }
    pub(super) fn object_label(&self, x: usize, y: usize) -> Option<&'static str> {
        if !self.owns(x, y) {
            return None;
        }
        match self.kind {
            Kind::Theater if (2..9).contains(&y) => None, // 168 floor finishes, not objects.
            Kind::Theater if y < 2 => Some("traditional:theater-paper-backdrop"),
            Kind::Theater => Some("traditional:theater-stage-fascia"),
            Kind::BarnRail => Some("traditional:barn-slatted-rail"),
            _ => Some("traditional:wise-trio-slatted-divider"),
        }
    }
    pub(super) fn coherent(&self, c: &[Identity<'_>], w: usize, h: usize) -> bool {
        w.checked_mul(h) == Some(c.len())
            && self.ground < c.len()
            && valid_ground(self.map, &c[self.ground])
            && self.column.checked_add(self.width).is_some_and(|v| v <= w)
            && self.row.checked_add(self.height).is_some_and(|v| v <= h)
            && (0..self.height).all(|y| {
                (0..self.width).all(|x| {
                    let s = &c[(self.row + y) * w + self.column + x];
                    let (block, tile) = expected(self.kind, x, y);
                    s.tileset == "traditional_house"
                        && s.metatile == block
                        && s.tile == tile
                        && i32::from(s.column) == (self.anchor[0] + x as i32).rem_euclid(4)
                        && i32::from(s.row) == (self.anchor[1] + y as i32).rem_euclid(4)
                })
            })
    }
}
fn expected(kind: Kind, x: usize, y: usize) -> (u16, u16) {
    let rail = if y % 2 == 0 { 0x40 } else { 0x41 };
    match kind {
        Kind::WiseCourse => (0x28, if y < 2 { rail } else { 0x50 }),
        Kind::WiseJoint => (0x37, if y < 2 || x >= 2 { rail } else { 0x50 }),
        Kind::WiseReturn => (0x38, if y >= 2 && x >= 2 { rail } else { 0x50 }),
        Kind::BarnRail => (0x26, if y >= 2 { rail } else { 0x01 }),
        Kind::Theater => {
            let block = if y < 4 {
                0x2d
            } else if y < 8 {
                0x2c
            } else if x < 4 {
                0x2e
            } else if x >= 20 {
                0x2f
            } else {
                0x30
            };
            let tile = match y {
                0 => {
                    if x % 2 == 0 {
                        0x4e
                    } else {
                        0x4f
                    }
                }
                1 => 0x5e,
                9 => {
                    if matches!(x, 2 | 3 | 20 | 21) {
                        0x5f
                    } else {
                        0x0f
                    }
                }
                _ => 0x50,
            };
            (block, tile)
        }
    }
}
pub(super) fn valid_ground(map: &str, s: &Identity<'_>) -> bool {
    if s.tileset != "traditional_house" || s.column >= 4 || s.row >= 4 {
        return false;
    }
    match map {
        "WiseTriosRoom" => s.metatile == 0x1f && s.tile == 0x50,
        "Route39Barn" => s.metatile == 0x20 && s.tile == 0x01,
        // This is actual zero-height audience floor. Elevated stage 50 never
        // supplies the missing ground of the retired incorrect cushion path.
        "DanceTheater" => tatami_cell(map, s),
        _ => false,
    }
}
pub(super) fn tatami_cell(map: &str, s: &Identity<'_>) -> bool {
    matches!(map, "DanceTheater" | "KurtsHouse")
        && s.tileset == "traditional_house"
        && s.metatile == 0x04
        && s.column < 4
        && s.row < 4
        && s.tile
            == [
                [0x44, 0x45, 0x45, 0x46],
                [0x54, 0x55, 0x55, 0x56],
                [0x45, 0x46, 0x44, 0x45],
                [0x55, 0x56, 0x54, 0x55],
            ][usize::from(s.row)][usize::from(s.column)]
}
pub(super) fn resolve(
    map: &str,
    c: &[Identity<'_>],
    w: usize,
    h: usize,
    origin: [i32; 2],
    reserved: &[bool],
) -> Vec<Match> {
    if w == 0 || w.checked_mul(h) != Some(c.len()) || reserved.len() != c.len() {
        return Vec::new();
    }
    let Some(ground) = c
        .iter()
        .enumerate()
        .find_map(|(i, s)| (!reserved[i] && valid_ground(map, s)).then_some(i))
    else {
        return Vec::new();
    };
    let specs: &[(Kind, [i32; 2])] = match map {
        "WiseTriosRoom" => &[
            (Kind::WiseCourse, [0, 4]),
            (Kind::WiseJoint, [4, 4]),
            (Kind::WiseReturn, [4, 8]),
            (Kind::WiseCourse, [0, 12]),
            (Kind::WiseCourse, [4, 12]),
        ],
        "Route39Barn" => &[
            (Kind::BarnRail, [0, 8]),
            (Kind::BarnRail, [4, 8]),
            (Kind::BarnRail, [12, 8]),
        ],
        "DanceTheater" => &[(Kind::Theater, [0, 0])],
        _ => return Vec::new(),
    };
    let map = match map {
        "WiseTriosRoom" => "WiseTriosRoom",
        "Route39Barn" => "Route39Barn",
        _ => "DanceTheater",
    };
    let mut out = Vec::new();
    for &(kind, anchor) in specs {
        let x = i64::from(anchor[0]) - i64::from(origin[0]);
        let y = i64::from(anchor[1]) - i64::from(origin[1]);
        if x < 0 || y < 0 {
            continue;
        }
        let (width, height) = if kind == Kind::Theater {
            (24, 10)
        } else {
            (4, 4)
        };
        let p = Match {
            column: x as usize,
            row: y as usize,
            width,
            height,
            kind,
            ground,
            anchor,
            map,
        };
        if p.coherent(c, w, h) && !p.guard_indices(w).any(|i| reserved[i]) {
            out.push(p);
        }
    }
    out
}
