//! Complete facility table/chair drawings; native source controls placement.
//! Adjacent chair/floor guards never become table geometry or support height.
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
    SquareTable,
    MeetingTable,
    SideDesk,
    Chair,
}
impl Kind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::SquareTable => "facility:square-document-table",
            Self::MeetingTable => "facility:joined-meeting-table",
            Self::SideDesk => "facility:document-side-desk",
            Self::Chair => "facility:square-back-chair",
        }
    }
    pub(super) fn fitting(self) -> ([f32; 4], f32) {
        match self {
            Self::SquareTable => ([0., 32., 0., 32.], 12.15),
            // The complete 8x8 guard includes its separate chairs and floor.
            // Only the native central 6x4 drawing belongs to this table.
            Self::MeetingTable => ([16., 64., 16., 48.], 9.135),
            Self::SideDesk => ([0., 32., 6., 24.], 12.15),
            // Native actor foot is (8,16); the full chair stays north of it.
            Self::Chair => ([2., 14., 0., 8.], 11.4),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ground {
    Checker,
}
struct Rule {
    map: &'static str,
    tileset: &'static str,
    anchor: Option<[i32; 2]>,
    width: usize,
    height: usize,
    fingerprint: u64,
    rows: &'static [u32],
    kind: Kind,
    ground: Ground,
}
include!("facility_tables_bindings.rs");
pub(super) struct Match {
    rule: &'static Rule,
    pub column: usize,
    pub row: usize,
    pub ground: [usize; 2],
    origin: [i32; 2],
}
impl Match {
    pub(super) fn kind(&self) -> Kind {
        self.rule.kind
    }
    pub(super) fn width(&self) -> usize {
        self.rule.width
    }
    pub(super) fn height(&self) -> usize {
        self.rule.height
    }
    pub(super) fn owns(&self, x: usize, y: usize) -> bool {
        x < self.rule.width && y < self.rule.height && self.rule.rows[y] & (1 << x) != 0
    }
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.rule.height).flat_map(move |y| {
            (0..self.rule.width).filter_map(move |x| {
                self.owns(x, y)
                    .then_some((self.row + y) * width + self.column + x)
            })
        })
    }
    pub(super) fn ground_for(&self, x: usize, y: usize) -> usize {
        let parity = (i64::from(self.origin[0]) + x as i64 + i64::from(self.origin[1]) + y as i64)
            .rem_euclid(2) as usize;
        self.ground[parity]
    }
    pub(super) fn coherent(&self, cells: &[Identity<'_>], width: usize, height: usize) -> bool {
        width.checked_mul(height) == Some(cells.len())
            && self.ground.iter().enumerate().all(|(phase, &i)| {
                i < cells.len() && valid_ground(self.rule.ground, &cells[i], phase)
            })
            && self.rule.anchor.is_none_or(|a| {
                i64::from(a[0]) == self.column as i64 + i64::from(self.origin[0])
                    && i64::from(a[1]) == self.row as i64 + i64::from(self.origin[1])
            })
            && complete(cells, width, height, self.column, self.row, self.rule)
    }
}
fn valid_ground(kind: Ground, s: &Identity<'_>, phase: usize) -> bool {
    if s.column >= 4 || s.row >= 4 {
        return false;
    }
    match kind {
        Ground::Checker => {
            s.tileset == "facility"
                && s.metatile == 0x07
                && (usize::from(s.column) + usize::from(s.row)) % 2 == phase
                && s.tile == [0x01, 0x26][phase]
        }
    }
}
fn identity_hash<'a>(sources: impl Iterator<Item = &'a Identity<'a>>) -> u64 {
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
    r: &Rule,
) -> bool {
    if x.checked_add(r.width).is_none_or(|v| v > width)
        || y.checked_add(r.height).is_none_or(|v| v > height)
    {
        return false;
    }
    let sources = || {
        (0..r.height)
            .flat_map(move |dy| (0..r.width).map(move |dx| &cells[(y + dy) * width + x + dx]))
    };
    sources().all(|s| s.tileset == r.tileset && s.column < 4 && s.row < 4)
        && identity_hash(sources()) == r.fingerprint
}
pub(super) fn resolve(
    map: &str,
    cells: &[Identity<'_>],
    width: usize,
    height: usize,
    origin: [i32; 2],
    reserved: &[bool],
    customized: &[bool],
) -> Vec<Match> {
    if width == 0
        || width.checked_mul(height) != Some(cells.len())
        || reserved.len() != cells.len()
        || customized.len() != cells.len()
    {
        return Vec::new();
    }
    let mut blocked = reserved.to_vec();
    let mut out = Vec::new();
    for rule in RULES.iter().filter(|r| r.map == map) {
        let Some(a) = cells.iter().position(|s| valid_ground(rule.ground, s, 0)) else {
            continue;
        };
        let Some(b) = cells.iter().position(|s| valid_ground(rule.ground, s, 1)) else {
            continue;
        };
        if rule.width > width || rule.height > height {
            continue;
        }
        for row in 0..=height - rule.height {
            for column in 0..=width - rule.width {
                if rule.anchor.is_some_and(|anchor| {
                    i64::from(anchor[0]) != column as i64 + i64::from(origin[0])
                        || i64::from(anchor[1]) != row as i64 + i64::from(origin[1])
                }) {
                    continue;
                }
                if !complete(cells, width, height, column, row, rule) {
                    continue;
                }
                // Source-specific custom geometry anywhere in a guarded assembly,
                // including its native staff opening, wins over the whole assembly.
                if (0..rule.height).any(|dy| {
                    (0..rule.width).any(|dx| customized[(row + dy) * width + column + dx])
                }) {
                    continue;
                }
                let p = Match {
                    rule,
                    column,
                    row,
                    ground: [a, b],
                    origin,
                };
                if p.indices(width).any(|i| blocked[i]) {
                    continue;
                }
                for i in p.indices(width) {
                    blocked[i] = true;
                }
                out.push(p);
            }
        }
    }
    out
}
