//! Pure source-identity resolution for the original Gym scenery kit.
//! No collision, actors, scripts, or map-state mutation enters this module.
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
    PlanterLow,
    PlanterRound,
    CentralTree,
    Hedge,
    Maze,
}
impl Kind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::PlanterLow => "gym:leafy-display-planter",
            Self::PlanterRound => "gym:round-display-planter",
            Self::CentralTree => "gym:azalea-broad-tree",
            Self::Hedge => "gym:celadon-round-hedge",
            Self::Maze => "gym:viridian-connected-maze",
        }
    }
    pub(super) fn asset_index(self) -> usize {
        match self {
            Self::PlanterLow => 0,
            Self::PlanterRound => 1,
            Self::CentralTree => 2,
            Self::Hedge => 3,
            Self::Maze => unreachable!(),
        }
    }
    pub(super) fn rise_pixels(self) -> f32 {
        match self {
            Self::PlanterLow => 14.,
            Self::PlanterRound => 16.,
            Self::CentralTree => 28.,
            Self::Hedge => 15.,
            Self::Maze => 12.,
        }
    }
}
struct Object {
    map: &'static str,
    tileset: &'static str,
    width: usize,
    height: usize,
    fingerprint: u64,
    kind: Kind,
}
struct Network {
    anchor: [i32; 2],
    width: usize,
    height: usize,
    fingerprint: u64,
    rows: &'static [u32],
}
include!("gym_scenery_bindings.rs");
#[derive(Clone, Debug)]
pub(super) struct Match {
    pub column: usize,
    pub row: usize,
    pub width: usize,
    pub height: usize,
    pub kind: Kind,
    pub ground: usize,
    // Sparse networks own only wall cells. Guard rectangles also check openings.
    rows: Option<&'static [u32]>,
    map: &'static str,
    tileset: &'static str,
    fingerprint: u64,
}
impl Match {
    pub(super) fn coherent(&self, cells: &[Identity<'_>], width: usize, height: usize) -> bool {
        width.checked_mul(height) == Some(cells.len())
            && self.ground < cells.len()
            && valid_ground(self.map, &cells[self.ground])
            && complete(
                cells,
                width,
                height,
                self.column,
                self.row,
                self.width,
                self.height,
                self.tileset,
                self.fingerprint,
            )
    }
    pub(super) fn owns(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && self.rows.is_none_or(|rows| rows[y] & (1u32 << x) != 0)
    }
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).filter_map(move |x| {
                self.owns(x, y)
                    .then_some((self.row + y) * width + self.column + x)
            })
        })
    }
    /// Exposed N/E/S/W faces; joined cells share the same coplanar top datum.
    pub(super) fn open_mask(&self, x: usize, y: usize) -> u8 {
        u8::from(y == 0 || !self.owns(x, y - 1))
            | (u8::from(!self.owns(x + 1, y)) << 1)
            | (u8::from(!self.owns(x, y + 1)) << 2)
            | (u8::from(x == 0 || !self.owns(x - 1, y)) << 3)
    }
}
fn identity_hash<'a>(sources: impl IntoIterator<Item = &'a Identity<'a>>) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for s in sources {
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
    hash
}
fn complete(
    cells: &[Identity<'_>],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    tileset: &str,
    fingerprint: u64,
) -> bool {
    if x.checked_add(w).is_none_or(|v| v > width) || y.checked_add(h).is_none_or(|v| v > height) {
        return false;
    }
    let sources =
        || (0..h).flat_map(move |dy| (0..w).map(move |dx| &cells[(y + dy) * width + x + dx]));
    sources().all(|s| s.tileset == tileset && s.column < 4 && s.row < 4)
        && identity_hash(sources()) == fingerprint
}
pub(super) fn valid_ground(map: &str, s: &Identity<'_>) -> bool {
    let (tileset, block, tile) = match map {
        "AzaleaGym" => ("elite_four_room", 0x12, 0x1f),
        "GoldenrodGym" => ("elite_four_room", 0x02, 0x03),
        "CeladonGym" => ("train_station", 0x19, 0x57),
        "ViridianGym" => ("train_station", 0x03, 0x3d),
        _ => return false,
    };
    s.tileset == tileset && s.metatile == block && s.tile == tile && s.column < 4 && s.row < 4
}
pub(super) fn resolve(
    map: &str,
    cells: &[Identity<'_>],
    width: usize,
    height: usize,
    origin: [i32; 2],
    reserved: &[bool],
) -> Vec<Match> {
    if width.checked_mul(height) != Some(cells.len()) || cells.len() != reserved.len() || width == 0
    {
        return Vec::new();
    }
    let Some(ground) = cells.iter().position(|s| valid_ground(map, s)) else {
        return Vec::new();
    };
    let mut claimed = reserved.to_vec();
    let mut out = Vec::new();
    for object in OBJECTS.iter().filter(|o| o.map == map) {
        for row in 0..height {
            for column in 0..width {
                if !complete(
                    cells,
                    width,
                    height,
                    column,
                    row,
                    object.width,
                    object.height,
                    object.tileset,
                    object.fingerprint,
                ) {
                    continue;
                }
                let p = Match {
                    column,
                    row,
                    width: object.width,
                    height: object.height,
                    kind: object.kind,
                    ground,
                    rows: None,
                    map: object.map,
                    tileset: object.tileset,
                    fingerprint: object.fingerprint,
                };
                if p.indices(width).any(|i| claimed[i]) {
                    continue;
                }
                for i in p.indices(width) {
                    claimed[i] = true;
                }
                out.push(p);
            }
        }
    }
    if map == "ViridianGym" {
        for n in NETWORKS {
            let x = i64::from(n.anchor[0]) - i64::from(origin[0]);
            let y = i64::from(n.anchor[1]) - i64::from(origin[1]);
            if x < 0 || y < 0 {
                continue;
            }
            let (column, row) = (x as usize, y as usize);
            if !complete(
                cells,
                width,
                height,
                column,
                row,
                n.width,
                n.height,
                "train_station",
                n.fingerprint,
            ) {
                continue;
            }
            let p = Match {
                column,
                row,
                width: n.width,
                height: n.height,
                kind: Kind::Maze,
                ground,
                rows: Some(n.rows),
                map: "ViridianGym",
                tileset: "train_station",
                fingerprint: n.fingerprint,
            };
            // A profile touching the guard's opening is a semantic override too.
            if (0..n.height)
                .any(|dy| (0..n.width).any(|dx| reserved[(row + dy) * width + column + dx]))
                || p.indices(width).any(|i| claimed[i])
            {
                continue;
            }
            for i in p.indices(width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    }
    out
}
