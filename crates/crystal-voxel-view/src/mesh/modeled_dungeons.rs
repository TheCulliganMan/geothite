//! Complete source drawings mapped to original dungeon volumes.
//!
//! All matches are render-only and require a same-tileset ground sample. The
//! resolver sees immutable sources before live profiles mask them. Missing,
//! clipped, recolored-with-new-identities and unsupported drawings fall back.
use super::*;
#[path = "cave_hatch.rs"]
mod cave_hatch;
#[path = "cave_rim.rs"]
mod cave_rim;
#[path = "cave_sill.rs"]
mod cave_sill;
#[path = "cave_stones.rs"]
mod cave_stones;
#[path = "cave_boulder_fields.rs"]
mod cave_boulder_fields;
pub(super) use cave_boulder_fields::append_all;

pub(super) fn movable_stone(frozen: bool, dark: bool, variant: u8) -> SurfaceMeshData {
    let mut mesh = SurfaceMeshData::default();
    cave_stones::append(&mut mesh, [-0.5,0.5,-0.5,0.5], 0., 1., frozen, dark,
        [17+variant as i32*11, 29]);
    let top = mesh.positions.iter().map(|p| p[1]).fold(0., f32::max);
    for p in &mut mesh.positions { p[1] *= 0.64/top; }
    for ids in mesh.indices.chunks_exact(3) {
        let p = [ids[0],ids[1],ids[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        let normal = (p[1]-p[0]).cross(p[2]-p[0]).normalize();
        let a = normal.abs();
        for &i in ids {
            mesh.normals[i as usize] = normal.to_array();
            let p = mesh.positions[i as usize];
            let uv = if a.y>=a.x && a.y>=a.z { [p[0],p[2]] }
                else if a.x>=a.z { [p[2],p[1]] } else { [p[0],p[1]] };
            mesh.uvs[i as usize] = uv.map(|v| v*0.22+variant as f32*0.19);
        }
    }
    mesh
}
#[path = "dungeon_extension.rs"]
mod extension;
#[path = "frozen_wall.rs"]
pub(super) mod frozen_wall;
#[path = "ice_pit.rs"]
mod ice_pit;
#[path = "lighthouse_masonry.rs"]
mod lighthouse;
#[path = "lighthouse_chamber.rs"]
mod lighthouse_chamber;
#[path = "lighthouse_stairs.rs"]
mod lighthouse_stairs;
#[path = "rocket_persian.rs"]
mod rocket_persian;
#[path = "special_rooms.rs"]
mod rooms;
use crate::dungeon_models::{Kind, model};
use crate::live_profiles::Document;

#[derive(Clone, Copy, Debug)]
enum Form {
    // Retain roof floor ownership from the map-scoped resolver.
    Room(rooms::Detail, rooms::FloorStyle),
    Special(extension::Detail),
    Model(Kind),
    GymBoulder,
    SeafoamBoulder,
    NativeGymGuardian,
    SproutGuardian,
    RocketPersian,
    TowerTimber,
    GymOrb,
    RocketWall { open: [bool; 4], blue:usize },
    Lighthouse(lighthouse::Detail),
    LighthouseChamber(lighthouse_chamber::Asset),
    LighthouseStair { down: bool },
    CaveCorner(crate::cave::DiagonalCorner),
    CaveHatch,
    CaveShaft,
    CavePortal,
    CaveSill,
    CaveRim,
    IceHatch,
    IceSill,
    FrozenWall,
    IcePit,
}
#[derive(Clone, Debug)]
pub(super) struct Placement {
    column: usize,
    row: usize,
    width: usize,
    height: usize,
    ground: usize,
    form: Form,
    rise_pixels: f32,
    depth_pixels: f32,
    front_rows: f32,
}
impl Placement {
    pub(super) fn room_wall(&self) -> bool {
        matches!(self.form,
            Form::Model(Kind::TimberWall | Kind::PortholeBulkhead) | Form::TowerTimber
            | Form::Room(rooms::Detail::Wall(_) | rooms::Detail::WallSegment { .. }, _)
            | Form::Special(extension::Detail::VioletWall | extension::Detail::VermilionPanel
                | extension::Detail::SaffronWall | extension::Detail::MahoganyWall
                | extension::Detail::FuchsiaWall
                | extension::Detail::PassageNetwork { .. }
                | extension::Detail::SaffronNetwork { .. }
                | extension::Detail::Model(crate::dungeon_models::ExtensionKind::LeagueWall
                    | crate::dungeon_models::ExtensionKind::ShipBulkhead))
            | Form::Lighthouse(_) | Form::RocketWall { .. })
    }
    pub(super) fn source_row(&self) -> usize { self.row }
    pub(super) fn append_room_wall(&self, mesh:&mut TerrainMeshData,g:&GridGeometry,
        cells:&[&VisualTile],origin:[i32;2]) {
        let mut p=self.clone();
        p.form=match p.form {
            Form::Room(rooms::Detail::WallSegment { material, .. },style)=>
                Form::Room(rooms::Detail::WallSegment { material,open:[true;4] },style),
            Form::Special(extension::Detail::PassageNetwork { basement,fade,.. })=>
                Form::Special(extension::Detail::PassageNetwork { basement,fade,open:[true;4] }),
            Form::Special(extension::Detail::SaffronNetwork { .. })=>
                Form::Special(extension::Detail::SaffronNetwork { open:[true;4] }),
            Form::RocketWall { blue,.. }=>Form::RocketWall { open:[true;4],blue },
            other=>other,
        };
        append(mesh,g,&p,cells,origin);
    }
    pub(super) fn cuts_foundation(&self) -> bool {
        matches!(self.form, Form::IcePit | Form::CaveShaft)
    }
    pub(super) fn floor_sample_and_label(&self) -> (usize, &'static str) {
        (self.ground, self.kind_label())
    }
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| (self.row + y) * width + self.column + x)
        })
    }
    pub(super) fn kind_label(&self) -> &'static str {
        match self.form {
            Form::Room(detail, _) => detail.label(),
            Form::Special(detail) => detail.label(),
            Form::Model(k) => k.label(),
            Form::GymBoulder | Form::SeafoamBoulder => Kind::CaveBoulder.label(),
            Form::NativeGymGuardian | Form::SproutGuardian => Kind::TowerGuardian.label(),
            Form::RocketPersian => "dungeon:rocket-persian-statue",
            Form::TowerTimber => Kind::TimberWall.label(),
            Form::GymOrb => Kind::LeaguePodium.label(),
            Form::RocketWall { .. } => "dungeon:rocket-wall-network",
            Form::Lighthouse(detail) => detail.label(),
            Form::LighthouseChamber(asset) => asset.label(),
            Form::LighthouseStair { down: false } => "lighthouse:stone-stairs-up",
            Form::LighthouseStair { down: true } => "lighthouse:stone-stairs-down",
            Form::CaveCorner(_) => "dungeon:cave-diagonal-corner",
            Form::CaveHatch | Form::CaveShaft => "dungeon:cave-floor-hatch",
            Form::CavePortal => "dungeon:cave-daylight-exit",
            Form::CaveSill => "dungeon:cave-exit-sill",
            Form::CaveRim => "dungeon:cave-rock-rim",
            Form::IceHatch => "dungeon:ice-floor-hatch",
            Form::IceSill => "dungeon:ice-exit-sill",
            Form::FrozenWall => "dungeon:connected-frozen-enclosure",
            Form::IcePit => "dungeon:ice-floor-pit",
        }
    }
}
fn ground(map: &str, cells: &[&VisualTile], tileset: &str, tile_index: u16) -> Option<usize> {
    cells.iter().position(|t| {
        t.source.tileset_id.as_ref() == tileset
            && t.source.tile_index == tile_index
            && matches!(
                shape_for_source_on_map(map, &t.source),
                CellShape::Flat | CellShape::Water | CellShape::PlaneAt { height: 0.0 }
            )
    })
}
/// Existing local classifiers verify drawing identity; this additional check
/// rejects phase-shifted fragments and Frankenstein groups across block seams.
fn phase_consistent(cells: &[&VisualTile], g: &GridGeometry, p: TreePlacement) -> bool {
    let anchor = &cells[p.row * g.width + p.column].source;
    (0..p.height).all(|y| {
        (0..p.width).all(|x| {
            let s = &cells[(p.row + y) * g.width + p.column + x].source;
            s.tileset_id == anchor.tileset_id
                && usize::from(s.subtile_column) == (usize::from(anchor.subtile_column) + x) % 4
                && usize::from(s.subtile_row) == (usize::from(anchor.subtile_row) + y) % 4
                && ((usize::from(anchor.subtile_column) + x) / 4 != 0
                    || (usize::from(anchor.subtile_row) + y) / 4 != 0
                    || s.metatile_id == anchor.metatile_id)
        })
    })
}
fn live_gym_orb(map: &str, tileset: &str) -> bool {
    match tileset {
        "elite_four_room" => matches!(map, "VioletGym" | "BlackthornGym1F" | "MahoganyGym" | "AzaleaGym" | "GoldenrodGym"),
        "tower" => matches!(map, "PewterGym" | "CianwoodGym" | "EcruteakGym"),
        _ => false,
    }
}
pub(super) fn tin_tower_guardian_map(map:&str)->bool {
    matches!(map,"TinTower1F"|"TinTower2F"|"TinTower3F"|"TinTower4F"|"TinTower5F"
        |"TinTower6F"|"TinTower7F"|"TinTower8F"|"TinTower9F")
}
pub(super) fn lighthouse_map(map:&str)->bool {lighthouse::applies(map)}
struct Resolver<'a> {
    map: &'a str,
    cells: &'a [&'a VisualTile],
    g: &'a GridGeometry,
    claimed: Vec<bool>,
    out: Vec<Placement>,
}
impl Resolver<'_> {
    fn add(&mut self, p: Placement) {
        if p.indices(self.g.width).any(|i| self.claimed[i]) {
            return;
        }
        for i in p.indices(self.g.width) {
            self.claimed[i] = true;
        }
        self.out.push(p);
    }
    fn groups(
        &mut self,
        kind: Kind,
        floor: u16,
        rise: f32,
        depth: f32,
        groups: Vec<TreePlacement>,
    ) {
        for p in groups {
            if !phase_consistent(self.cells, self.g, p) {
                continue;
            }
            let tileset = self.cells[p.row * self.g.width + p.column]
                .source
                .tileset_id
                .as_ref();
            let Some(ground) = ground(self.map, self.cells, tileset, floor) else {
                continue;
            };
            self.add(Placement {
                column: p.column,
                row: p.row,
                width: p.width,
                height: p.height,
                ground,
                form: if kind == Kind::CaveBoulder && self.map == "SeafoamGym"
                    && tileset == "cave" && p.width == 2 && p.height == 2 {
                    Form::SeafoamBoulder
                } else if kind == Kind::CaveBoulder
                    && matches!(self.map, "OlivineGym" | "PewterGym" | "CianwoodGym" | "BlackthornGym1F")
                    && p.width == 2 && p.height == 2 {
                    Form::GymBoulder
                } else if kind == Kind::TowerGuardian && (matches!(self.map,
                    "PewterGym"|"FuchsiaGym"|"ViridianGym"|"CeruleanGym"|"CeladonGym"|"VermilionGym")
                    || tin_tower_guardian_map(self.map) || self.map=="BurnedTower1F") {
                    Form::NativeGymGuardian
                } else if kind == Kind::TowerGuardian
                    && matches!(self.map, "SproutTower1F" | "SproutTower2F" | "SproutTower3F") {
                    Form::SproutGuardian
                } else if kind == Kind::LeaguePodium && live_gym_orb(self.map, tileset) {
                    Form::GymOrb
                } else if kind == Kind::WarningBeacon && self.map == "TeamRocketBaseB1F"
                    && tileset == "underground" {
                    Form::RocketPersian
                } else if kind == Kind::TimberWall
                    && (tin_tower_guardian_map(self.map)
                        ||matches!(self.map, "CianwoodGym" | "EcruteakGym" | "SproutTower1F" | "SproutTower2F" | "SproutTower3F" | "BurnedTower1F")) {
                    Form::TowerTimber
                } else {
                    Form::Model(kind)
                },
                rise_pixels: if kind == Kind::CaveBoulder && self.map == "SeafoamGym" {12.} else {rise},
                depth_pixels: if kind == Kind::CaveBoulder && self.map == "SeafoamGym" {14.} else {depth},
                front_rows: p.height as f32,
            });
        }
    }
}

pub(super) fn resolve(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    profiles: Option<&Document>,
) -> Vec<Placement> {
    let mut r = Resolver {
        map,
        cells,
        g,
        claimed: vec![false; cells.len()],
        out: Vec::new(),
    };
    // Complete 2x2 drawings, not collision-derived boulder fields. The light
    // and dark atlases retain separate authored materials.
    for (tileset, kind) in [
        ("cave", Kind::CaveBoulder),
        ("dark_cave", Kind::DarkBoulder),
    ] {
        let groups = grouped_flat_card_placements(cells, g, 0x16, false, |s| {
            (s.tileset_id.as_ref() == tileset)
                .then(|| crate::cave::small_rock_local(s))
                .flatten()
                .map(|(x, y)| (x, y, 2, 2))
        });
        r.groups(kind, 0x16, 8.0, 10.0, groups);
    }
    for p in grouped_flat_card_placements(cells, g, 0x16, false, |s| {
        crate::cave::exit_sill_local(s).map(|(x, y)| (x, y, 2, 1))
    }) {
        if !phase_consistent(cells, g, p) {
            continue;
        }
        let tileset = cells[p.row * g.width + p.column].source.tileset_id.as_ref();
        let Some(ground) = ground(map, cells, tileset, 0x16) else {
            continue;
        };
        r.add(Placement {
            column: p.column,
            row: p.row,
            width: 2,
            height: 1,
            ground,
            form: Form::CaveSill,
            rise_pixels: 1.5,
            depth_pixels: 8.0,
            front_rows: 1.0,
        });
    }
    for p in grouped_flat_card_placements(cells, g, 0x16, false, |s| {
        crate::cave::daylight_exit_local(s).map(|(x, y)| (x, y, 2, 2))
    }) {
        if !phase_consistent(cells, g, p) { continue; }
        let tileset = cells[p.row*g.width+p.column].source.tileset_id.as_ref();
        let Some(ground) = ground(map, cells, tileset, 0x16) else { continue; };
        r.add(Placement { column:p.column, row:p.row, width:2, height:2,
            ground, form:Form::CavePortal, rise_pixels:0., depth_pixels:16., front_rows:2. });

    }
    // Complete drawings own the original footprint and support. Upward
    // rails stand above it; downward flights enter a recessed shaft.
    for p in grouped_flat_card_placements(cells, g, 0x16, false, |s| {
        crate::cave::flat_ladder_local(s).map(|(x, y)| (x, y, 2, 2))
    }) {
        if !phase_consistent(cells, g, p) {
            continue;
        }
        let tileset = cells[p.row * g.width + p.column].source.tileset_id.as_ref();
        let Some(ground) = ground(map, cells, tileset, 0x16) else {
            continue;
        };
        r.add(Placement {
            column: p.column,
            row: p.row,
            width: 2,
            height: 2,
            ground,
            form: if matches!(cells[p.row*g.width+p.column].source.metatile_id, 0x0b | 0x15 | 0x1f) {
                Form::CaveShaft
            } else { Form::CaveHatch },
            rise_pixels: 0.0,
            depth_pixels: 16.0,
            front_rows: 2.0,
        });
    }
    if map != "HallOfFame" {
        for p in grouped_flat_card_placements(cells, g, crate::ice_path::CAVE_GROUND_TILE, false, |s| {
            crate::ice_path::exit_sill_local(s).map(|(x,y)| (x,y,2,1))
        }) {
            if !phase_consistent(cells,g,p) { continue; }
            let Some(ground) = ground(map,cells,"ice_path",crate::ice_path::CAVE_GROUND_TILE) else { continue; };
            r.add(Placement { column:p.column,row:p.row,width:2,height:1,ground,
                form:Form::IceSill,rise_pixels:1.5,depth_pixels:8.,front_rows:1. });
        }
        for p in
            grouped_flat_card_placements(cells, g, crate::ice_path::CAVE_GROUND_TILE, false, |s| {
                crate::ice_path::flat_ladder_local(s).map(|(x, y)| (x, y, 2, 2))
            })
        {
            if !phase_consistent(cells, g, p) {
                continue;
            }
            let Some(ground) = ground(map, cells, "ice_path", crate::ice_path::CAVE_GROUND_TILE)
            else {
                continue;
            };
            r.add(Placement {
                column: p.column,
                row: p.row,
                width: 2,
                height: 2,
                ground,
                form: Form::IceHatch,
                rise_pixels: 0.0,
                depth_pixels: 16.0,
                front_rows: 2.0,
            });
        }
    }
    if map.starts_with("IcePath") {
        for p in grouped_flat_card_placements(cells, g, 0x19, false, |s| {
            crate::ice_path::pit_local(s).map(|(x, y)| (x, y, 2, 2))
        }) {
            if !phase_consistent(cells, g, p) {
                continue;
            }
            let Some(ground) = ground(map, cells, "ice_path", 0x19) else {
                continue;
            };
            r.add(Placement {
                column: p.column,
                row: p.row,
                width: 2,
                height: 2,
                ground,
                form: Form::IcePit,
                rise_pixels: 0.0,
                depth_pixels: 16.0,
                front_rows: 2.0,
            });
        }
    }
    for base in [
        crate::ice_path::BoulderBase::CaveGround,
        crate::ice_path::BoulderBase::SmoothIce,
        crate::ice_path::BoulderBase::UpperRight,
    ] {
        let floor = if base == crate::ice_path::BoulderBase::SmoothIce {
            crate::ice_path::SMOOTH_ICE_TILE
        } else {
            crate::ice_path::CAVE_GROUND_TILE
        };
        let groups = grouped_flat_card_placements(cells, g, floor, false, |s| {
            crate::ice_path::boulder_local(s, base).map(|(x, y)| (x, y, 2, 2))
        });
        r.groups(Kind::IceBoulder, floor, 8.0, 12.0, groups);
    }
    for edge in [
        crate::ice_path::EdgeRockKind::Left,
        crate::ice_path::EdgeRockKind::Right,
        crate::ice_path::EdgeRockKind::Single,
    ] {
        let groups = grouped_flat_card_placements(cells, g, 0x19, false, |s| {
            crate::ice_path::edge_rock_local(s)
                .and_then(|(e, x, y)| (e == edge).then_some((x, y, 2, 2)))
        });
        r.groups(Kind::IceBoulder, 0x19, 12.0, 12.0, groups);
    }
    let before = r.out.len();
    let groups = grouped_flat_card_placements(cells, g, 0x19, false, |s| {
        crate::ice_path::rock_mass_local(s).map(|(x, y)| (x, y, 4, 4))
    });
    r.groups(Kind::IceMass, 0x19, 16.0, 16.0, groups);
    for p in &mut r.out[before..] {
        if map.starts_with("IcePath") {
            p.form = Form::FrozenWall;
            p.rise_pixels = 48.0;
            p.depth_pixels = 32.0;
            p.front_rows = 4.0;
        } else {
            p.front_rows = 2.0;
        }
    }
    if matches!(map,"SproutTower1F"|"SproutTower2F"|"SproutTower3F") {
        let statues=grouped_flat_card_placements(cells,g,crate::tower::TOWER_FLOOR_TILE,false,
            crate::tower::sprout_statue_local);
        r.groups(Kind::TowerGuardian,crate::tower::TOWER_FLOOR_TILE,24.,10.,statues);
    }
    if matches!(map, "PewterGym" | "CianwoodGym") {
        r.groups(
            Kind::CaveBoulder,
            crate::tower::TOWER_FLOOR_TILE,
            10.0,
            12.0,
            tower_boulder_placements(cells, g),
        );
    }
    if matches!(map, "CianwoodGym" | "EcruteakGym" | "PewterGym" | "BurnedTower1F") || tin_tower_guardian_map(map) {
        let guardians = grouped_flat_card_placements(cells,g,crate::tower::TOWER_FLOOR_TILE,false,|s| {
            if s.tileset_id.as_ref()!="tower" || s.subtile_row>=4 { return None; }
            let (ox, lower) = match s.metatile_id {
                0x19 if map=="PewterGym" => (0, [0x26, 0x27]),
                0x1a if map=="PewterGym" => (2, [0x26, 0x27]),
                0x21 if map!="PewterGym" => (0, [0x12, 0x13]),
                0x22 if map!="PewterGym" => (2, [0x12, 0x13]),
                _ => return None,
            };
            if s.subtile_column<ox || s.subtile_column>=ox+2 {return None;}
            let x=s.subtile_column-ox; let y=s.subtile_row;
            let expected=[[0x22,0x23],[0x32,0x33],lower,[0x36,0x37]][y as usize][x as usize];
            (s.tile_index==expected).then_some((x,y,2,4))
        });
        r.groups(Kind::TowerGuardian,crate::tower::TOWER_FLOOR_TILE,24.0,10.0,guardians);
    }
    r.groups(
        Kind::AlphGuardian,
        crate::ruins_of_alph::FLOOR_TILE,
        28.0,
        10.0,
        ruins_statue_placements(cells, g),
    );
    if map == "OlivineGym" {
        r.groups(
            Kind::CaveBoulder,
            crate::olivine_gym::GROUND_TILE,
            12.0,
            12.0,
            olivine_gym_boulder_placements(cells, g),
        );
        r.groups(
            Kind::TowerGuardian,
            crate::olivine_gym::GROUND_TILE,
            28.0,
            10.0,
            olivine_gym_statue_placements(map, cells, g),
        );
    }
    if crate::elite_four_room::supports_boulder_map(map) {
        r.groups(
            Kind::CaveBoulder,
            crate::elite_four_room::FLOOR_TILE,
            12.0,
            12.0,
            elite_four_room_boulder_placements(cells, g),
        );
    }
    // Every gym statue matcher is map+tileset+source scoped. Shared tilesets
    // never turn unrelated shop or arcade fixtures into statues.
    for (floor, groups) in [
        (
            crate::cerulean_gym::DECK_TILE,
            cerulean_statue_placements(cells, g),
        ),
        (
            crate::fuchsia_gym::FLOOR_TILE,
            fuchsia_statue_placements(map, cells, g),
        ),
        (
            crate::celadon_gym::STATUE_FLOOR_TILE,
            celadon_statue_placements(map, cells, g),
        ),
        (
            crate::vermilion_gym::FLOOR_TILE,
            vermilion_statue_placements(map, cells, g),
        ),
        (
            crate::viridian_gym::FLOOR_TILE,
            viridian_statue_placements(map, cells, g),
        ),
    ] {
        r.groups(Kind::TowerGuardian, floor, 28.0, 10.0, groups);
    }
    if cells
        .first()
        .is_some_and(|t| t.source.tileset_id.as_ref() == "elite_four_room")
    {
        let statues = grouped_flat_card_placements(cells, g, 0x01, false, |source| {
            let group = crate::violet_gym::card_group(map, source)?;
            if group.height != 4 {
                return None;
            }
            let art = [[0x20, 0x21], [0x30, 0x31], [0x22, 0x23], [0x32, 0x33]];
            (source.tile_index == art[group.local_row as usize][group.local_column as usize])
                .then_some((group.local_column, group.local_row, 2, 4))
        });
        r.groups(Kind::LeaguePodium, 0x01, 28.0, 10.0, statues);
        let plaques = grouped_flat_card_placements(cells, g, 0x01, false, |source| {
            let group = crate::violet_gym::card_group(map, source)?;
            if group.height != 2 {
                return None;
            }
            let art = [[0x55, 0x56], [0x57, 0x58]];
            (source.tile_index == art[group.local_row as usize][group.local_column as usize])
                .then_some((group.local_column, group.local_row, 2, 2))
        });
        r.groups(Kind::GymPlaque, 0x01, 14.0, 5.0, plaques);
    }
    if map == "VermilionGym" {
        let guardians = grouped_flat_card_placements(cells, g, 0x01, false, |s| {
            if s.tileset_id.as_ref() != "game_corner" || s.subtile_row >= 4 {
                return None;
            }
            let x = match (s.metatile_id, s.subtile_column) {
                (0x1d, 2..=3) => s.subtile_column - 2,
                (0x1e, 0..=1) => s.subtile_column,
                _ => return None,
            };
            let art = [[0x42, 0x43], [0x52, 0x53], [0x44, 0x45], [0x54, 0x55]];
            (s.tile_index == art[s.subtile_row as usize][x as usize])
                .then_some((x, s.subtile_row, 2, 4))
        });
        r.groups(Kind::TowerGuardian, 0x01, 24.0, 10.0, guardians);
        let bins = grouped_flat_card_placements(cells, g, 0x01, false, |s| {
            if s.tileset_id.as_ref() != "game_corner"
                || s.metatile_id != 0x1f
                || s.subtile_column < 2
                || s.subtile_row < 2
            {
                return None;
            }
            let (x, y) = (s.subtile_column - 2, s.subtile_row - 2);
            (s.tile_index == [[0x46, 0x47], [0x56, 0x57]][y as usize][x as usize])
                .then_some((x, y, 2, 2))
        });
        r.groups(Kind::GymBin, 0x01, 12.0, 12.0, bins);
    }
    if map.starts_with("FastShip") {
        // Complete stools and both berth drawings belong to ship_rooms, which
        // validates the whole drawing and yields to every live custom profile.
        r.groups(
            Kind::ShipRack,
            crate::ship::CABIN_FLOOR_TILE,
            28.0,
            10.0,
            ship_rack_placements(map, cells, g),
        );
        r.groups(
            Kind::ShipBarrel,
            crate::ship::CABIN_FLOOR_TILE,
            14.0,
            12.0,
            ship_barrel_placements(map, cells, g),
        );
    }
    if let Some(floor) = crate::warehouse::floor_tile(map) {
        r.groups(
            Kind::WarehouseCrate,
            floor,
            12.0,
            12.0,
            warehouse_crate_placements(map, cells, g),
        );
    }
    let timber_walls =
        grouped_flat_card_placements(cells, g, crate::tower::TOWER_FLOOR_TILE, false, |source| {
            match crate::tower::tower_shape(source) {
                Some(CellShape::FacadeBand {
                    band_from_top,
                    band_count: 2,
                    ..
                }) => Some((source.subtile_column, band_from_top, 4, 2)),
                _ => None,
            }
        });
    r.groups(
        Kind::TimberWall,
        crate::tower::TOWER_FLOOR_TILE,
        32.0,
        3.0,
        timber_walls,
    );
    lighthouse::resolve_into(&mut r);
    lighthouse_chamber::resolve_into(&mut r);
    lighthouse_stairs::resolve_into(&mut r);
    // Reuse live whole-drawing resolution rather than copy/export its catalog.
    // Shape-specific vocabulary below guards the semantic conversion; profile
    // names alone are never authority to reinterpret a user-edited drawing.
    for live in live::resolve(cells, g.width, g.height, map, profiles) {
        let o = live.object;
        let descriptor = match o.tileset.as_str() {
            "tower"
                if matches!(o.metatile, 0x12 | 0x13)
                    && o.tiles
                        == vec![
                            vec![0x28, 0x29],
                            vec![0x38, 0x39],
                            vec![0x2a, 0x2b],
                            vec![0x3a, 0x3b],
                        ] =>
            {
                Some((Kind::LeaguePodium, 28.0, 10.0))
            }
            "elite_four_room"
                if matches!(o.metatile, 0x25 | 0x26)
                    && o.tiles
                        == vec![
                            vec![0x20, 0x21],
                            vec![0x30, 0x31],
                            vec![0x22, 0x23],
                            vec![0x32, 0x33],
                        ] =>
            {
                Some((Kind::LeaguePodium, 28.0, 10.0))
            }
            "lighthouse"
                if map.starts_with("FastShipCabins")
                    && matches!(o.metatile, 0x09 | 0x0a | 0x38)
                    && o.tiles.len() == 2
                    && o.tiles[0].len() == 4
                    && o.origin == [0, 0]
                    && live.indices(g.width).all(|i| {
                        matches!(
                            crate::ship::shape(map, &cells[i].source),
                            Some(CellShape::FacadeBand { band_count: 2, .. })
                        )
                    }) =>
            {
                Some((Kind::PortholeBulkhead, 16.0, 3.0))
            }
            _ => None,
        };
        if let Some((kind, rise, depth)) = descriptor {
            r.add(Placement {
                column: live.column,
                row: live.row,
                width: o.tiles[0].len(),
                height: o.tiles.len(),
                ground: live.ground,
                form: if kind == Kind::LeaguePodium && live_gym_orb(map, o.tileset.as_str()) {
                    Form::GymOrb
                } else { Form::Model(kind) },
                rise_pixels: rise,
                depth_pixels: depth,
                front_rows: o.tiles.len() as f32,
            });
        }
    }
    // The alarm sensor is a Persian statue on a pedestal, not a warning lamp.
    // Its upper source rows also close the neighboring wall ends.
    let alarm = grouped_flat_card_placements(cells, g, 0x10, false, |s| {
        if map != "TeamRocketBaseB1F"
            || s.tileset_id.as_ref() != "underground"
            || s.metatile_id != 0x04
            || s.subtile_column >= 2
            || s.subtile_row >= 4
        {
            return None;
        }
        let d = [[0x45, 0x46], [0x55, 0x56], [0x09, 0x19], [0x30, 0x31]];
        (s.tile_index == d[s.subtile_row as usize][s.subtile_column as usize]).then_some((
            s.subtile_column,
            s.subtile_row,
            2,
            4,
        ))
    });
    r.groups(Kind::WarningBeacon, 0x10, 28.0, 8.0, alarm);
    if let (Some(ground),Some(blue)) = (ground(map,cells,"underground",crate::rocket_base::FLOOR_TILE),
        cells.iter().position(|t|crate::rocket_base::is_wall_cell(map,&t.source)
            &&crate::rocket_base::is_lower_face(&t.source))) {
        let wall: Vec<_> = cells
            .iter()
            .map(|t| crate::rocket_base::is_wall_cell(map, &t.source))
            .collect();
        let closes: Vec<_> = cells
            .iter()
            .zip(&wall)
            .map(|(t, &w)| w || crate::rocket_base::closes_wall_edge(map, &t.source))
            .collect();
        for (i, &occupied) in wall.iter().enumerate() {
            if !occupied {
                continue;
            }
            let (x, y) = (i % g.width, i / g.width);
            let open = [
                y == 0 || !closes[i - g.width],
                x + 1 == g.width || !closes[i + 1],
                y + 1 == g.height || !closes[i + g.width],
                x == 0 || !closes[i - 1],
            ];
            r.add(Placement {
                column: x,
                row: y,
                width: 1,
                height: 1,
                ground,
                form: Form::RocketWall { open,blue },
                rise_pixels: crate::rocket_base::WALL_HEIGHT,
                depth_pixels: 8.0,
                front_rows: 1.0,
            });
        }
    }
    for p in diagonal_cave_corner_placements(cells, g) {
        let tileset = cells[p.row * g.width + p.column].source.tileset_id.as_ref();
        if let Some(ground) = ground(map, cells, tileset, 0x16) {
            r.add(Placement {
                column: p.column,
                row: p.row,
                width: 2,
                height: 2,
                ground,
                form: Form::CaveCorner(p.corner),
                rise_pixels: crate::cave::rock_height(&cells[p.row * g.width + p.column].source),
                depth_pixels: 16.0,
                front_rows: 2.0,
            });
        }
    }
    if tin_tower_guardian_map(map)||matches!(map, "SproutTower1F" | "SproutTower2F" | "SproutTower3F") {
        if let Some(ground) = ground(map, cells, "tower", crate::tower::TOWER_FLOOR_TILE) {
            let ladders = grouped_flat_card_placements(cells, g, crate::tower::TOWER_FLOOR_TILE, false, |s| {
                if s.tileset_id.as_ref() != "tower" || s.subtile_column >= 4 || s.subtile_row >= 4 { return None; }
                let (offset, down) = match s.metatile_id {
                    0x1f => (0, true), 0x20 => (0, false),
                    0x02 => (2, true), 0x03 => (2, false), _ => return None,
                };
                if s.subtile_column < offset || s.subtile_column >= offset+2
                    || s.subtile_row < offset || s.subtile_row >= offset+2 { return None; }
                let x=s.subtile_column-offset; let y=s.subtile_row-offset;
                let tile = if down { [[0x0e,0x0f],[0x1e,0x1f]] } else { [[0x0c,0x0d],[0x1c,0x1d]] }
                    [y as usize][x as usize];
                (s.tile_index == tile).then_some((x,y,2,2))
            });
            for p in ladders {
                r.add(Placement { column:p.column,row:p.row,width:2,height:2,ground,
                    form:Form::Room(rooms::Detail::TowerLadder { down: matches!(cells[p.row*g.width+p.column].source.metatile_id,0x02|0x1f) }, rooms::FloorStyle::Default),
                    rise_pixels:24.,depth_pixels:16.,front_rows:2. });
            }
        }
    }
    cave_rim::resolve_into(&mut r);
    rooms::resolve_into(&mut r);
    extension::resolve_into(&mut r, profiles);
    r.out
}

fn boulder_datum(p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> f32 {
    if !matches!(p.form, Form::SeafoamBoulder | Form::Model(Kind::CaveBoulder | Kind::DarkBoulder)) {
        return 0.;
    }
    // Prefer floor paint from this exact metatile over a neighboring shelf.
    // A lower-floor rock beside a step must not borrow the step's height.
    let source = &cells[p.row*g.width+p.column].source;
    let bx = p.column as isize - source.subtile_column as isize;
    let bz = p.row as isize - source.subtile_row as isize;
    let mut local_floor = [false;2];
    for z in bz.max(0)..(bz+4).min(g.height as isize) {
        for x in bx.max(0)..(bx+4).min(g.width as isize) {
            let s = &cells[z as usize*g.width+x as usize].source;
            if s.tileset_id != source.tileset_id || s.metatile_id != source.metatile_id
                || x-s.subtile_column as isize != bx || z-s.subtile_row as isize != bz { continue; }
            if s.tile_index == 0x16 { local_floor[0] = true; }
            if s.tile_index == 0x01 { local_floor[1] = true; }
        }
    }
    match local_floor {
        [true,false] => return 0.,
        [false,true] => return crate::cave::CAVE_SHELF_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT,
        _ => {}
    }
    let mut shelf = 0.;
    let mut count = 0;
    for y in p.row.saturating_sub(1)..=(p.row + p.height).min(g.height - 1) {
        for x in p.column.saturating_sub(1)..=(p.column + p.width).min(g.width - 1) {
            if x >= p.column && x < p.column + p.width && y >= p.row && y < p.row + p.height {
                continue;
            }
            let Some(shape) = crate::cave::cave_shape(&cells[y * g.width + x].source) else {
                continue;
            };
            let h = shape.surface_height(g.tile_height);
            if h > 0. && h <= crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT {
                shelf = f32::max(shelf, h);
                count += 1;
            }
        }
    }
    if count >= 2 { shelf } else { 0. }
}

/// Adjoining terrain must meet the floor emitted beneath a complete rock,
/// rather than the flat sprite that the placement replaces. This is used only
/// for mesh joins, never for authoritative player support or collision.
pub(super) fn joined_surface_shapes(
    placements: &[Placement], cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry,
) -> Vec<CellShape> {
    let mut joined = shapes.to_vec();
    for p in placements {
        if matches!(p.form, Form::SeafoamBoulder | Form::Model(Kind::CaveBoulder | Kind::DarkBoulder | Kind::IceBoulder))
            && matches!(cells[p.row*g.width+p.column].source.tileset_id.as_ref(), "cave"|"dark_cave"|"ice_path") {
            let height = boulder_datum(p, cells, g) * SOURCE_TILE_HEIGHT / g.tile_height;
            for i in p.indices(g.width) { joined[i] = CellShape::PlaneAt { height }; }
        }
    }
    joined
}

fn append_open_gym_bin(out:&mut SurfaceMeshData,b:[f32;4],height:f32) {
    let [w,e,n,s]=b;let center=[(w+e)*0.5,(n+s)*0.5];
    let radius=(e-w).min(s-n)*0.5;let scale=height/12.;let thickness=0.7*scale;
    let bottom=radius*0.83;let floor=scale;
    let inner_bottom=bottom+(radius-bottom)*floor/height-thickness;
    let ring=|r:f32,y:f32| (0..16).map(|j| {
        let angle=j as f32*std::f32::consts::TAU/16.;
        [center[0]+r*angle.cos(),y,center[1]+r*angle.sin()]
    }).collect::<Vec<_>>();
    let outer_low=ring(bottom,0.);let outer_high=ring(radius,height);
    let inner_low=ring(inner_bottom,floor);let inner_high=ring(radius-thickness,height);
    let mut face=|points:[[f32;3];4],color:[f32;4]| {
        let p=points.map(Vec3::from_array);let normal=(p[1]-p[0]).cross(p[2]-p[0]).normalize().to_array();
        append_quad(out,points,normal,[[0.;2];4],color);
    };
    for j in 0..16 {let k=(j+1)%16;
        face([outer_low[k],outer_low[j],outer_high[j],outer_high[k]],[0.34,0.39,0.40,1.]);
        face([inner_low[j],inner_low[k],inner_high[k],inner_high[j]],[0.11,0.13,0.14,1.]);
        face([outer_high[k],outer_high[j],inner_high[j],inner_high[k]],[0.47,0.52,0.54,1.]);
    }
    drop(face);
    for j in 0..16 {let k=(j+1)%16;
        for (points,normal,color) in [
            ([[center[0],0.,center[1]],outer_low[j],outer_low[k]],[0.,-1.,0.],[0.34,0.39,0.40,1.]),
            ([[center[0],floor,center[1]],inner_low[k],inner_low[j]],[0.,1.,0.],[0.045,0.055,0.06,1.]),
        ] {let first=out.positions.len()as u32;out.positions.extend(points);
            out.normals.extend([normal;3]);out.uvs.extend([[0.;2];3]);out.colors.extend([color;3]);
            out.indices.extend([first,first+1,first+2]);}
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    origin: [i32; 2],
) {
    append_internal(mesh, g, p, cells, origin, true);
}

fn append_internal(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    origin: [i32; 2],
    separate_boulder: bool,
) {
    if let Form::LighthouseChamber(asset) = p.form {
        lighthouse_chamber::append(mesh, g, p, asset);
        return;
    }
    if let Form::LighthouseStair { down } = p.form {
        lighthouse_stairs::append(mesh,g,p,down);
        return;
    }
    if let Form::Lighthouse(detail) = p.form {
        lighthouse::append(mesh, g, p, detail);
        return;
    }
    if let Form::Room(detail, floor_style) = p.form {
        rooms::append_with_floor(mesh, g, p, cells, detail, floor_style);
        return;
    }
    if let Form::Special(detail) = p.form {
        extension::append(mesh, g, p, cells, detail);
        return;
    }
    if matches!(p.form, Form::CaveRim) {
        cave_rim::append(mesh, g, p, cells, origin);
        return;
    }
    if matches!(p.form, Form::IcePit) {
        ice_pit::append(&mut mesh.solid, g, p);
        return;
    }
    if matches!(p.form, Form::CavePortal) {
        cave_hatch::append_portal(mesh, g, p, cells, origin);
        return;
    }
    if matches!(p.form, Form::CaveHatch | Form::CaveShaft | Form::IceHatch) {
        let datum = crate::cave::cave_shape(&cells[p.row*g.width+p.column].source)
            .map(|shape|shape.surface_height(g.tile_height)).unwrap_or(0.);
        for i in p.indices(g.width).filter(|_| !matches!(p.form, Form::CaveShaft)) {
            let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
            let floor=[[w,datum,n],[w,datum,s],[e,datum,s],[e,datum,n]];
            if matches!(p.form,Form::IceHatch) {
                ice_ground::append_blended(&mut mesh.solid,floor,false,g,origin,cells);
            } else {
                new_bark::cave_finish::append(&mut mesh.solid,floor,[0.,1.,0.],&cells[p.ground].source,g,origin);
            }
        }
        if matches!(p.form, Form::CaveShaft) {
            cave_hatch::append_shaft(mesh, g, p, cells, datum, origin);
            return;
        }
        let first=mesh.solid.positions.len();
        cave_hatch::append(mesh,g,p,
            cells[p.row*g.width+p.column].source.tileset_id.as_ref()=="dark_cave",
            matches!(p.form,Form::IceHatch),origin);
        for point in &mut mesh.solid.positions[first..] { point[1]+=datum; }
        return;
    }
    let base = boulder_datum(p, cells, g);
    let floor_uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        if matches!(p.form, Form::FrozenWall | Form::CaveCorner(_)) {
            let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
            let floor = [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]];
            // The wall's revealed substrate is real mineral ground, not an
            // atlas patch or the background apron. This adds no walk support.
            if matches!(p.form, Form::FrozenWall) {
                ice_ground::append_blended(&mut mesh.solid, floor, false, g, origin,cells);
            } else {
                new_bark::cave_finish::append(&mut mesh.solid, floor, [0., 1., 0.],
                    &cells[p.ground].source, g, origin);
            }
            continue;
        }
        if matches!(p.form, Form::SeafoamBoulder | Form::Model(Kind::CaveBoulder | Kind::DarkBoulder | Kind::IceBoulder))
            && matches!(cells[p.row*g.width+p.column].source.tileset_id.as_ref(), "cave"|"dark_cave"|"ice_path") {
            let (w, e, n, s) = g.bounds(i % g.width, i / g.width);
            let floor = [[w, base, n], [w, base, s], [e, base, s], [e, base, n]];
            let source = &cells[p.ground].source;
            if source.tileset_id.as_ref() == "ice_path" {
                ice_ground::append_blended(&mut mesh.solid, floor,
                    source.tile_index == crate::ice_path::SMOOTH_ICE_TILE, g, origin,cells);
            } else {
                new_bark::cave_finish::append(&mut mesh.solid, floor, [0.,1.,0.], source, g, origin);
            }
            continue;
        }
        if matches!(p.form, Form::CaveSill) {
            let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
            if new_bark::cave_finish::append(&mut mesh.solid,
                [[w,base,n],[w,base,s],[e,base,s],[e,base,n]], [0.,1.,0.],
                &cells[p.ground].source,g,origin) {
                continue;
            }
        }
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            base,
            floor_uv,
        );
    }
    if !separate_boulder {
        // A connected field supplies the rock above these exact floor caps.
        return;
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.front_rows * g.tile_height;
    let rise = p.rise_pixels * g.tile_height / SOURCE_TILE_HEIGHT;
    match p.form {
        Form::RocketPersian => rocket_persian::append(&mut mesh.textured,g,p,
            [w,e,s-p.depth_pixels*g.tile_height/SOURCE_TILE_HEIGHT,s]),
        Form::CaveSill | Form::IceSill => cave_sill::append(
            &mut mesh.solid,
            g,
            p,
            cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "dark_cave",
            matches!(p.form,Form::IceSill),
            origin,
        ),
        Form::CaveHatch | Form::CaveShaft | Form::CavePortal | Form::IceHatch | Form::IcePit => {
            unreachable!("floor recesses handled above")
        }
        Form::Room(..) => unreachable!("room placements handled above"),
        Form::Special(_) => unreachable!("special placements handled above"),
        Form::FrozenWall => frozen_wall::append(&mut mesh.solid, cells, g, p, origin),
        Form::Model(kind @ (Kind::CaveBoulder | Kind::DarkBoulder | Kind::IceBoulder))
            if matches!(
                cells[p.row * g.width + p.column].source.tileset_id.as_ref(),
                "cave" | "dark_cave" | "ice_path"
            ) =>
        {
            let first=mesh.solid.positions.len();
            cave_stones::append(
                &mut mesh.solid,
                [
                    w,
                    e,
                    s - p.depth_pixels * g.tile_height / SOURCE_TILE_HEIGHT,
                    s,
                ],
                base,
                rise,
                matches!(kind, Kind::IceBoulder),
                matches!(kind, Kind::DarkBoulder),
                [origin[0] + p.column as i32, origin[1] + p.row as i32],
            );
            for i in first..mesh.solid.positions.len() {
                mesh.solid.uvs[i]=cave_cliff::stone_uv(Vec3::from_array(mesh.solid.positions[i]),g,origin,matches!(kind,Kind::IceBoulder));
            }
        }
        Form::NativeGymGuardian | Form::SproutGuardian => {
            let bounds = [w, e, s - p.depth_pixels * g.tile_height / SOURCE_TILE_HEIGHT, s];
            let source_uv = |pixel: [usize; 2]| {
                let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
                [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
            };
            if matches!(p.form, Form::SproutGuardian) {
                // The rear guardians are painted over the north wall. Their
                // foreground source claim must not punch a hole in that wall.
                let source=&cells[p.row*g.width+p.column].source;
                let wall_column=match source.metatile_id {
                    0x2e=>Some(p.column+2),
                    0x2f=>p.column.checked_sub(1),
                    _=>None,
                };
                if let Some(column)=wall_column.filter(|&x|x<g.width) {
                    let wall=&cells[p.row*g.width+column].source;
                    if wall.tileset_id==source.tileset_id && wall.metatile_id==source.metatile_id
                        && wall.subtile_row==0 && wall.tile_index==0x11 {
                        let south=s-2.*g.tile_height;
                        append_tower_backing(&mut mesh.textured,g,column,p.row,
                            [w,e,south-3.*g.tile_height/SOURCE_TILE_HEIGHT,south]);
                    }
                }
                extension::append_sprout_guardian(
                    &mut mesh.textured, bounds, rise, source_uv);
            } else {
                model(Kind::TowerGuardian).append_pewter_guardian(
                    &mut mesh.textured, bounds, rise, source_uv);
            }
        }
        Form::GymOrb => model(Kind::LeaguePodium).append_gym_orb(
            &mut mesh.textured,
            [w, e, s - p.depth_pixels * g.tile_height / SOURCE_TILE_HEIGHT, s],
            rise,
            |pixel| {
                let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
                [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
            },
        ),
        Form::TowerTimber => model(Kind::TimberWall).append_tower_timber(
            &mut mesh.textured,
            [w, e, s - p.depth_pixels * g.tile_height / SOURCE_TILE_HEIGHT, s],
            rise,
            |pixel| {
                let pixel = match pixel {
                    [0, 0] => [0, 0], [0, 2] => if cells[p.row*g.width+p.column].source.tile_index==0x11 {[2,3]}else{[1,3]},
                    [0, 1] => [0, 1], [4, 0] => [2, 0],
                    _ => unreachable!("validated timber materials"),
                };
                let uv = g.uv(p.column, p.row);
                [uv.0 + (uv.1 - uv.0) * (pixel[0] as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * (pixel[1] as f32 + 0.5) / 8.]
            },
        ),
        Form::GymBoulder | Form::SeafoamBoulder => {
            let first=mesh.textured.positions.len();
            let tower_stone = matches!(p.form, Form::GymBoulder)
                && cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "tower";
            model(Kind::CaveBoulder).append_gym_stone(
            &mut mesh.textured,
            [w, e, s - p.depth_pixels * g.tile_height / SOURCE_TILE_HEIGHT, s],
            rise,
            |pixel| {
                // The tower drawing's pale cream is a small glint. Broad
                // highlight facets use its lighter stone shade, matching the
                // movable Cianwood boulders without repainting the atlas.
                let blackthorn_stone = cells[p.row * g.width + p.column].source.tileset_id.as_ref()
                    == "elite_four_room";
                let pixel = if blackthorn_stone {
                    // Exact 4e/4f/5e/5f drawing: native blue stone shades 2/1.
                    match pixel { [8, 6] => [2, 3], [4, 4] => [2, 2],
                        _ => unreachable!("named canonical gym stone") }
                } else if tower_stone && pixel == [4, 4] { [8, 1] } else { pixel };
                let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
                [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
            },
        );
            if matches!(p.form,Form::SeafoamBoulder) {
                for point in &mut mesh.textured.positions[first..] {point[1]+=base;}
            }
        },
        Form::Model(Kind::GymBin) => append_open_gym_bin(&mut mesh.solid,
            [w,e,s-p.depth_pixels*g.tile_height/SOURCE_TILE_HEIGHT,s],rise),
        Form::Model(kind) => model(kind).append(
            &mut mesh.solid,
            [
                w,
                e,
                s - p.depth_pixels * g.tile_height / SOURCE_TILE_HEIGHT,
                s,
            ],
            0.0,
            rise,
        ),
        Form::RocketWall { open,blue } => {
            let texel=|column,row,x:f32,y:f32| {
                let uv=g.uv(column,row);
                [uv.0+(uv.1-uv.0)*(x+0.5)/8.,uv.2+(uv.3-uv.2)*(y+0.5)/8.]
            };
            append_native_rocket_wall(&mut mesh.textured,[w,e,n,s],rise,open,
                texel(p.column,p.row,0.,0.),texel(blue%g.width,blue/g.width,3.,3.));
        }
        Form::CaveRim => unreachable!("cave rims handled above"),
        Form::Lighthouse(_) => unreachable!("lighthouse placements handled above"),
        Form::LighthouseChamber(_) => unreachable!("lighthouse chamber handled above"),
        Form::LighthouseStair { .. } => unreachable!("lighthouse stairs handled above"),
        Form::CaveCorner(corner) => append_corner(
            &mut mesh.solid,
            [w, e, n, s],
            rise,
            corner,
            cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "dark_cave",
            g,
            origin,
            Some(cells),
        ),
    }
}

/// Match the adjoining ceiling-height timber using a verified native 11 swatch.
pub(super) fn append_tower_backing(mesh:&mut SurfaceMeshData,g:&GridGeometry,
    column:usize,row:usize,bounds:[f32;4]) {
    let uv=g.uv(column,row);
    model(Kind::TimberWall).append_tower_timber(mesh,bounds,
        32.*g.tile_height/SOURCE_TILE_HEIGHT,|pixel| {
            let pixel=match pixel {
                [0,0]=>[0,0], [0,2]=>[2,3], [0,1]=>[0,1], [4,0]=>[2,0],
                _=>unreachable!("validated timber materials"),
            };
            [uv.0+(uv.1-uv.0)*(pixel[0] as f32+0.5)/8.,
             uv.2+(uv.3-uv.2)*(pixel[1] as f32+0.5)/8.]
        });
}

/// Four fixed perimeter vertices and a recessed centre form hand-cut facets.
/// Boundary positions are never jittered, so independently emitted segments
/// share bit-identical edges at corners, tees, caps and changed-height seams.
fn faceted_face(
    out: &mut SurfaceMeshData,
    points: [[f32; 3]; 4],
    normal: [f32; 3],
    color: [f32; 4],
    recess: f32,
) {
    let n = Vec3::from_array(normal);
    let p = points.map(Vec3::from_array);
    let centre = (p[0] + p[1] + p[2] + p[3]) * 0.25 - n * recess;
    for j in 0..4 {
        let a = p[j];
        let b = p[(j + 1) % 4];
        let cross = (b - a).cross(centre - a);
        if cross.length_squared() < 1e-10 {
            continue;
        }
        let actual = cross.normalize();
        // Source rectangles have mixed historical winding; make every actual
        // triangle's winding agree with its expected outward-facing normal.
        let (a, b, actual) = if actual.dot(n) < 0.0 {
            (b, a, -actual)
        } else {
            (a, b, actual)
        };
        let base = out.positions.len() as u32;
        out.positions
            .extend([a.to_array(), b.to_array(), centre.to_array()]);
        out.normals.extend([actual.to_array(); 3]);
        out.uvs.extend([[0.0; 2]; 3]);
        let shade = [1.0, 0.94, 1.045, 0.97][j];
        let c = [
            (color[0] * shade).min(1.0),
            (color[1] * shade).min(1.0),
            (color[2] * shade).min(1.0),
            color[3],
        ];
        out.colors.extend([c; 3]);
        out.indices.extend([base, base + 1, base + 2]);
    }
}
fn append_native_rocket_wall(out:&mut SurfaceMeshData,[w,e,n,s]:[f32;4],h:f32,
    open:[bool;4],cap:[f32;2],blue:[f32;2]) {
    let first=out.positions.len();
    faceted_face(out,[[w,h,n],[w,h,s],[e,h,s],[e,h,n]],Vec3::Y.to_array(),[1.;4],0.);
    out.uvs[first..].fill(cap);
    for (enabled,points,normal) in [
        (open[0],[[e,0.,n],[w,0.,n],[w,h,n],[e,h,n]],-Vec3::Z),
        (open[1],[[e,0.,s],[e,0.,n],[e,h,n],[e,h,s]],Vec3::X),
        (open[2],[[w,0.,s],[e,0.,s],[e,h,s],[w,h,s]],Vec3::Z),
        (open[3],[[w,0.,n],[w,0.,s],[w,h,s],[w,h,n]],-Vec3::X),
    ] {
        if !enabled {continue;}
        let first=out.positions.len();
        faceted_face(out,points,normal.to_array(),
            crate::interior_models::authored_face_color([1.;4],normal),0.);
        out.uvs[first..].fill(blue);
    }
}

fn append_connected_wall(out: &mut SurfaceMeshData, b: [f32; 4], h: f32, open: [bool; 4]) {
    let [w, e, n, s] = b;
    let cap = [0.45, 0.53, 0.51, 1.0];
    faceted_face(
        out,
        [[w, h, n], [w, h, s], [e, h, s], [e, h, n]],
        [0., 1., 0.],
        cap,
        0.0,
    );
    for (enabled, points, normal) in [
        (
            open[0],
            [[e, 0., n], [w, 0., n], [w, h, n], [e, h, n]],
            [0., 0., -1.],
        ),
        (
            open[1],
            [[e, 0., s], [e, 0., n], [e, h, n], [e, h, s]],
            [1., 0., 0.],
        ),
        (
            open[2],
            [[w, 0., s], [e, 0., s], [e, h, s], [w, h, s]],
            [0., 0., 1.],
        ),
        (
            open[3],
            [[w, 0., n], [w, 0., s], [w, h, s], [w, h, n]],
            [-1., 0., 0.],
        ),
    ] {
        if !enabled {
            continue;
        }
        for (lo, hi, color, inset) in [
            (0.0, 0.12, [0.21, 0.28, 0.29, 1.0], 0.0),
            (0.12, 0.82, [0.38, 0.46, 0.43, 1.0], h * 0.022),
            (0.82, 1.0, [0.54, 0.60, 0.55, 1.0], 0.0),
        ] {
            let mut band = points;
            band[0][1] = h * lo;
            band[1][1] = h * lo;
            band[2][1] = h * hi;
            band[3][1] = h * hi;
            faceted_face(out, band, normal, color, inset);
        }
    }
}
fn append_masonry(out: &mut SurfaceMeshData, b: [f32; 4], h: f32, open: [bool; 4]) {
    let [w, e, n, s] = b;
    faceted_face(
        out,
        [[w, h, n], [w, h, s], [e, h, s], [e, h, n]],
        [0., 1., 0.],
        [0.66, 0.63, 0.50, 1.0],
        0.0,
    );
    for (side, enabled) in open.into_iter().enumerate() {
        if !enabled {
            continue;
        }
        for course in 0..4 {
            let low = h * course as f32 / 4.0;
            let high = h * (course + 1) as f32 / 4.0;
            // Four matching longitudinal brick segments preserve every edge;
            // staggered inset facets suggest cut masonry without source pixels.
            for segment in 0..4 {
                let a = segment as f32 / 4.0;
                let b = (segment + 1) as f32 / 4.0;
                let (p, norm) = match side {
                    0 => (
                        [
                            [e - (e - w) * a, low, n],
                            [e - (e - w) * b, low, n],
                            [e - (e - w) * b, high, n],
                            [e - (e - w) * a, high, n],
                        ],
                        [0., 0., -1.],
                    ),
                    1 => (
                        [
                            [e, low, s - (s - n) * a],
                            [e, low, s - (s - n) * b],
                            [e, high, s - (s - n) * b],
                            [e, high, s - (s - n) * a],
                        ],
                        [1., 0., 0.],
                    ),
                    2 => (
                        [
                            [w + (e - w) * a, low, s],
                            [w + (e - w) * b, low, s],
                            [w + (e - w) * b, high, s],
                            [w + (e - w) * a, high, s],
                        ],
                        [0., 0., 1.],
                    ),
                    _ => (
                        [
                            [w, low, n + (s - n) * a],
                            [w, low, n + (s - n) * b],
                            [w, high, n + (s - n) * b],
                            [w, high, n + (s - n) * a],
                        ],
                        [-1., 0., 0.],
                    ),
                };
                let c = if (segment + course) % 3 == 0 {
                    [0.64, 0.62, 0.51, 1.0]
                } else {
                    [0.57, 0.55, 0.46, 1.0]
                };
                faceted_face(out, p, norm, c, h * 0.012);
            }
        }
    }
}
fn append_corner(
    out: &mut SurfaceMeshData,
    b: [f32; 4],
    h: f32,
    corner: crate::cave::DiagonalCorner,
    dark: bool,
    g: &GridGeometry,
    origin: [i32; 2],
    cells: Option<&[&VisualTile]>,
) {
    let [w, e, n, s] = b;
    let corners = cells.map(|cells|cave_cliff::RockCorners::new(cells,g));
    let point = |p: Vec3,g: &GridGeometry,o: [i32;2]| {
        let mut shaped=p;
        let shelf=crate::cave::CAVE_SHELF_HEIGHT*g.tile_height/SOURCE_TILE_HEIGHT;
        if p.y>0. {
            let u=((p.x-w)/(e-w)).clamp(0.,1.);
            let v=((p.z-n)/(s-n)).clamp(0.,1.);
            let diagonal=match corner {
                crate::cave::DiagonalCorner::SouthEast=>u+v-1.,
                crate::cave::DiagonalCorner::SouthWest=>v-u,
            };
            let band=(1.-diagonal.abs()/0.125).clamp(0.,1.);
            let rounding=if p.y<shelf {
                let t=p.y/shelf;
                16.*t*t*(1.-t)*(1.-t)
            } else {
                let rise=((p.y-shelf)/(2.*g.tile_height)).clamp(0.,1.);
                rise*rise*(3.-2.*rise)
            };
            // A bowed diagonal replaces the flat triangular sheet through
            // the wall body, not only at its foot. The same field rounds the
            // cap edge. Tapering at both ends preserves adjoining closures,
            // and the narrow band keeps the cap's interior from folding.
            // The exact floor contact and six-pixel walking course remain.
            let recess=rounding*4.*u*(1.-u)*band*0.12;
            let inward=match corner {
                crate::cave::DiagonalCorner::SouthEast=>Vec3::X+Vec3::Z,
                crate::cave::DiagonalCorner::SouthWest=>-Vec3::X+Vec3::Z,
            };
            shaped+=inward.normalize()*Vec3::new(g.tile_width,0.,g.tile_height)*recess;
        }
        corners.as_ref().map_or_else(||cave_cliff::rock_point(shaped,g,o),|corners|corners.point(shaped,g,o))
    };
    let color = if dark {
        [0.31, 0.37, 0.39, 1.0]
    } else {
        [0.34, 0.345, 0.31, 1.0]
    };
    // The corner foot shares the straight wall's exact source boundary.
    // Flaring it into neighboring floor cells left narrow exposed faces
    // after the foreground wall faded, resembling spikes or false ledges.
    let (cap, face, south, side, _side_normal) = match corner {
        crate::cave::DiagonalCorner::SouthEast => (
            [[e, h, n], [e, h, s], [w, h, s]],
            [
                [e, 0., n],
                [e, h, n],
                [w, h, s],
                [w, 0., s],
            ],
            [[w, 0., s], [w, h, s], [e, h, s], [e, 0., s]],
            [[e, 0., s], [e, h, s], [e, h, n], [e, 0., n]],
            [1., 0., 0.],
        ),
        crate::cave::DiagonalCorner::SouthWest => (
            [[w, h, n], [e, h, s], [w, h, s]],
            [
                [w, 0., n],
                [e, 0., s],
                [e, h, s],
                [w, h, n],
            ],
            [[w, 0., s], [w, h, s], [e, h, s], [e, 0., s]],
            [[w, 0., n], [w, h, n], [w, h, s], [w, 0., s]],
            [-1., 0., 0.],
        ),
    };
    let mut cap = cap;
    if (Vec3::from_array(cap[1]) - Vec3::from_array(cap[0]))
        .cross(Vec3::from_array(cap[2]) - Vec3::from_array(cap[0]))
        .y
        < 0.0
    {
        cap.swap(1, 2);
    }
    cave_cliff::cap_triangle_with_point(out, cap, color, g, origin, point);
    // The triangular body occupies the south side of the diagonal. Face
    // winding must point into the open passage, not back into that body.
    let outward = |p: [[f32;3];4]| [p[3],p[2],p[1],p[0]];
    cave_cliff::quad_with_point(out, outward(face), color, g, origin, point, false);
    for closure in [outward(south), outward(side)] {
        let p = closure.map(Vec3::from_array);
        let normal = (p[1]-p[0]).cross(p[3]-p[0]).normalize();
        let length = (p[3]-p[0]).abs().max_element();
        let count = (length/g.tile_width).round().max(1.) as usize;
        for i in 0..count {
            let lo=i as f32/count as f32;
            let hi=(i+1) as f32/count as f32;
            let segment=[p[0].lerp(p[3],lo),p[1].lerp(p[2],lo),
                p[1].lerp(p[2],hi),p[0].lerp(p[3],hi)];
            // Closures belong only to exposed source edges. A full-height
            // adjoining rock cell already closes the mass; keeping the
            // internal face makes it show through the foreground cutaway.
            // Unknown art, shelves and partial diagonal drawings retain
            // their closure instead of opening holes in the enclosure.
            let probe=(segment[0]+segment[3])*0.5+normal*g.tile_width*0.5;
            let x=((probe.x-g.origin_x)/g.tile_width).floor() as i32;
            let z=((probe.z-g.origin_z)/g.tile_height).floor() as i32;
            let buried=cells.and_then(|cells| {
                (x>=0 && z>=0 && x<g.width as i32 && z<g.height as i32)
                    .then(||cells.get(z as usize*g.width+x as usize)).flatten()
            }).is_some_and(|t| {
                t.source.tileset_id.as_ref()==if dark {"dark_cave"} else {"cave"}
                    && crate::cave::diagonal_corner_local(&t.source).is_none()
                    && matches!(crate::cave::cave_shape(&t.source),
                        Some(CellShape::RaisedTop {height,solid:SolidKind::Bank})
                            if height*g.tile_height/SOURCE_TILE_HEIGHT>=h)
            });
            if !buried {
                cave_cliff::quad_with_point(out,segment.map(|p|p.to_array()),color,g,origin,point,false);
            }
        }
    }
}

fn terrain_material(source: &VisualTileSource, shape: CellShape) -> Option<[f32; 4]> {
    if !matches!(
        shape,
        CellShape::RaisedTop {
            solid: SolidKind::Bank,
            ..
        } | CellShape::LedgeBand { .. }
    ) {
        return None;
    }
    match source.tileset_id.as_ref() {
        "cave" => Some([0.34, 0.345, 0.31, 1.0]),
        "dark_cave" => Some([0.31, 0.37, 0.39, 1.0]),
        "ice_path" => Some([0.38, 0.69, 0.78, 1.0]),
        // Exclude rock-platform $0a here: its clipped corners have a separate
        // authored topology. Flat grassy jump ledges also retain their art.
        "johto" | "johto_modern" if matches!(source.metatile_id, 0x68..=0x73) => {
            Some([0.52, 0.49, 0.36, 1.0])
        }
        "kanto"
            if crate::kanto_cliff::kanto_cliff_shape(source).is_some()
                && shape.surface_height(SOURCE_TILE_HEIGHT) >= 16.0 =>
        {
            Some([0.55, 0.53, 0.42, 1.0])
        }
        _ => None,
    }
}
fn neighbor_edge_heights(shape: CellShape, side: usize, scale: f32) -> [f32; 2] {
    // side is viewed from the current rock cell. The endpoints must use the
    // actual adjoining ramp edge, never its north/high support approximation.
    match shape {
        CellShape::RampNorth {
            north_height,
            south_height,
        } => match side {
            0 => [south_height * scale; 2],
            2 => [north_height * scale; 2],
            1 => [south_height * scale, north_height * scale],
            _ => [north_height * scale, south_height * scale],
        },
        CellShape::RampEast {
            west_height,
            east_height,
        } => match side {
            1 => [west_height * scale; 2],
            3 => [east_height * scale; 2],
            0 => [east_height * scale, west_height * scale],
            _ => [west_height * scale, east_height * scale],
        },
        _ => [shape.surface_height(SOURCE_TILE_HEIGHT) * scale; 2],
    }
}
/// Build joined geological relief on verified enclosing rock banks. The final
/// tier-resolved shapes define the bank datum; nonwalkable rock gains a broken
/// silhouette and overhangs, while walking shelves keep their exact plane.
/// Modeled elevated floor patches get exposed sides beneath their existing caps.
/// Actor support, collision, warps and stairs are untouched. Unknown viewport
/// continuation emits no artificial perimeter skirt.
pub(super) fn append_terrain(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
    origin: [i32; 2],
) {
    let mut shallow_scarps = Vec::new();
    let corners = cave_cliff::RockCorners::new_with_shapes(map,cells,shapes,g);
    // A modeled triangular corner owns its complete south and lateral
    // boundary at the tall rock height. Individual source glyphs on those
    // boundaries still have low/flat gameplay support, which must not make
    // an adjoining straight wall emit another face inside the rock mass.
    // These render-only edge heights never replace the walking shapes.
    let mut corner_edges = vec![[None;4];cells.len()];
    for p in diagonal_cave_corner_placements(cells,g) {
        let source=&cells[p.row*g.width+p.column].source;
        let indices=[p.row*g.width+p.column,p.row*g.width+p.column+1,
            (p.row+1)*g.width+p.column,(p.row+1)*g.width+p.column+1];
        if !indices.iter().all(|&i|claimed[i]
            && cells[i].source.tileset_id==source.tileset_id
            && cells[i].source.metatile_id==source.metatile_id)
            || ground(map,cells,source.tileset_id.as_ref(),0x16).is_none() {
            continue;
        }
        let h=crate::cave::rock_height(source)*g.tile_height/SOURCE_TILE_HEIGHT;
        for x in 0..2 {corner_edges[(p.row+1)*g.width+p.column+x][2]=Some(h);}
        let (column,side)=match p.corner {
            crate::cave::DiagonalCorner::SouthEast=>(p.column+1,1),
            crate::cave::DiagonalCorner::SouthWest=>(p.column,3),
        };
        for y in 0..2 {corner_edges[(p.row+y)*g.width+column][side]=Some(h);}
    }
    for (index, tile) in cells.iter().enumerate() {
        // Authored boulders and ladders already own their floor caps. Their
        // joined elevated support still needs an exposed rock side, otherwise
        // the correct cap floats above a hole beside water or lower ground.
        let modeled_floor = claimed[index]
            && matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave")
            && matches!(shapes[index], CellShape::PlaneAt { height } if height == crate::cave::CAVE_SHELF_HEIGHT);
        if claimed[index] && !modeled_floor {
            continue;
        }
        let den = map == "DragonsDenB1F" && tile.source.tileset_id.as_ref() == "johto";
        let Some(mut color) = terrain_material(&tile.source, shapes[index]).or_else(|| {
            modeled_floor.then_some(if tile.source.tileset_id.as_ref() == "dark_cave" {
                [0.31, 0.37, 0.39, 1.]
            } else { [0.34, 0.345, 0.31, 1.] })
        }) else {
            continue;
        };
        if den { color = [0.34, 0.345, 0.31, 1.]; }
        let h = shapes[index].surface_height(g.tile_height);
        if h <= 0.0 {
            continue;
        }
        let (x, y) = (index % g.width, index / g.width);
        let (w, e, n, s) = g.bounds(x, y);
        let cliff = matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave")
            && h >= 16.0 * g.tile_height / SOURCE_TILE_HEIGHT;
        let canyon = (map.starts_with("Route") || map.ends_with("Town") || map.ends_with("City"))
            && match tile.source.tileset_id.as_ref() {
                "johto" | "johto_modern" => (0x68..=0x73).contains(&tile.source.metatile_id),
                "kanto" => crate::kanto_cliff::kanto_cliff_shape(&tile.source).is_some()
                    && h >= crate::kanto_cliff::KANTO_CLIFF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT,
                _ => false,
            };
        let cap = [[w, h, n], [w, h, s], [e, h, s], [e, h, n]];
        if modeled_floor {
            // The owning model already emitted this exact floor cap.
        } else if den {
            let colors = cap.map(|p| cave_cliff::pigment(color, Vec3::from_array(p), g, origin, false));
            append_quad_colors(&mut mesh.solid, cap, [0.,1.,0.],
                cap.map(|p| cave_cliff::floor_uv(Vec3::from_array(p),g,origin,false)), colors);
        } else if canyon {
            outdoor_stone::append_canyon(&mut mesh.solid, cap, [0.0, 1.0, 0.0], g, origin);
        } else if cliff {
            cave_cliff::quad_with_point(&mut mesh.solid, cap, color, g, origin, |p,g,o|corners.point(p,g,o), false);
            if h >= 32. * g.tile_height / SOURCE_TILE_HEIGHT {
                new_bark::cave_finish::append(&mut mesh.solid,
                    [[w, 0., n], [w, 0., s], [e, 0., s], [e, 0., n]],
                    [0., 1., 0.], &tile.source, g, origin);
            }
        } else if matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave") {
            // Walking shelves are smooth support surfaces. Repeated four-way
            // light/dark triangles made each source cell look like a tile
            // quilt; use continuous map-space mineral color on their plane.
            if !new_bark::cave_finish::append(&mut mesh.solid, cap,
                [0., 1., 0.], &tile.source, g, origin)
            {
                let colors = cap.map(|p|
                    cave_cliff::pigment(color, Vec3::from_array(p), g, origin, false));
                append_quad_colors(&mut mesh.solid, cap, [0., 1., 0.],
                    cap.map(|p|cave_cliff::floor_uv(Vec3::from_array(p),g,origin,false)), colors);
            }
            new_bark::cave_finish::floor_detail(
                &mut mesh.solid,
                cap,
                [0., 1., 0.],
                &tile.source,
                g,
                origin,
            );
        } else {
            faceted_face(&mut mesh.solid, cap, [0., 1., 0.], color, 0.0);
        }
        let neighbors = [
            y.checked_sub(1).map(|r| r * g.width + x),
            (x + 1 < g.width).then_some(index + 1),
            (y + 1 < g.height).then_some(index + g.width),
            x.checked_sub(1).map(|c| y * g.width + c),
        ];
        for (side, neighbor) in neighbors.into_iter().enumerate() {
            let Some(ni) = neighbor else {
                continue;
            };
            let bottom = corner_edges[ni][(side+2)%4].map_or_else(
                ||neighbor_edge_heights(shapes[ni],side,g.tile_height/SOURCE_TILE_HEIGHT),
                |height|[height;2]);
            // A changing ramp may meet the cap at only one endpoint. Keep the
            // shared diagonal and emit a single triangle at that termination.
            let low = [bottom[0].min(h), bottom[1].min(h)];
            if low[0] >= h && low[1] >= h {
                continue;
            }
            let (points, normal) = match side {
                0 => (
                    [[e, low[0], n], [w, low[1], n], [w, h, n], [e, h, n]],
                    [0., 0., -1.],
                ),
                1 => (
                    [[e, low[0], s], [e, low[1], n], [e, h, n], [e, h, s]],
                    [1., 0., 0.],
                ),
                2 => (
                    [[w, low[0], s], [e, low[1], s], [e, h, s], [w, h, s]],
                    [0., 0., 1.],
                ),
                _ => (
                    [[w, low[0], n], [w, low[1], s], [w, h, s], [w, h, n]],
                    [-1., 0., 0.],
                ),
            };
            let face_color = [color[0] * 0.86, color[1] * 0.86, color[2] * 0.86, 1.0];
            if den {
                let colors = points.map(|p| cave_cliff::pigment(face_color, Vec3::from_array(p), g, origin, false));
                append_quad_colors(&mut mesh.solid, points, normal,
                    points.map(|p| cave_cliff::stone_uv(Vec3::from_array(p),g,origin,false)), colors);
            } else if canyon {
                outdoor_stone::append_canyon(&mut mesh.solid, points, normal, g, origin);
            } else if cliff {
                cave_cliff::quad_with_point(&mut mesh.solid, points, face_color, g, origin, |p,g,o|corners.point(p,g,o), false);
            } else if matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave") {
                if low == [0., 0.]
                    && h == crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT
                {
                    shallow_scarps.push((points, normal, face_color));
                } else {
                    // Sloping stair boundaries must meet their exact ramp.
                    cave_cliff::quad_with_point(&mut mesh.solid, points, face_color,
                        g, origin, |p,_,_| p, false);
                }
            } else {
                faceted_face(
                    &mut mesh.solid,
                    points,
                    normal,
                    face_color,
                    (h - low[0].min(low[1])).min(g.tile_width) * 0.05,
                );
            }
            // Native plain floor/shelf samples only: no fragments in water,
            // stairs, ladders, authored objects or unknown viewport halos.
            if matches!(tile.source.tileset_id.as_ref(), "cave" | "dark_cave")
                && cells[ni].source.tileset_id == tile.source.tileset_id
                && matches!(cells[ni].source.tile_index, 0x01 | 0x16)
                && !matches!(cells[ni].source.metatile_id, 0x3d | 0x3e)
                && matches!(shapes[ni], CellShape::Flat | CellShape::RaisedTop { .. })
                && bottom[0] == bottom[1]
                && bottom[0] <= crate::cave::CAVE_SHELF_HEIGHT * g.tile_height / SOURCE_TILE_HEIGHT
            {
                cave_stones::edge_debris(
                    &mut mesh.solid, [w, e, n, s], low[0], h, side,
                    tile.source.tileset_id.as_ref() == "dark_cave", false, g, origin,
                    [origin[0] + x as i32, origin[1] + y as i32],
                );
            }
        }
        if !modeled_floor {
            claimed[index] = true;
            mark_authored_rect(mesh, g, [x, y, 1, 1], "dungeon:connected-rock-terrain");
        }
    }
    cave_cliff::scarps(&mut mesh.solid, shallow_scarps, g, origin);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn kanto_outdoor_cliffs_use_ochre_mineral_without_mutating_footing() {
        for block in [0x2a, 0x2c] {
            let mut t = tiles(4, 4, "kanto");
            for tile in &mut t {
                tile.source.metatile_id = block;
                tile.source.tile_index = 0x11;
            }
            let cells = t.iter().collect::<Vec<_>>();
            let g = grid(4, 4);
            let shapes = cells.iter().map(|t| shape_for_source(&t.source)).collect::<Vec<_>>();
            for map in ["Route3", "Route4", "Route9", "LavenderTown"] {
                let mut mesh = TerrainMeshData::default();
                mesh.authored_cells = vec![None; 16];
                mesh.footing_heights = vec![16.; 16];
                let mut claimed = vec![false; 16];
                append_terrain(&mut mesh, map, &cells, &shapes, &g, &mut claimed, [13, 20]);
                assert_eq!(mesh.footing_heights, vec![16.; 16]);
                assert!(claimed.iter().all(|v| *v));
                assert!(mesh.textured.positions.is_empty());
                assert!(!mesh.solid.indices.is_empty());
                assert!(mesh.solid.colors.iter().any(|c| c[0] > c[2] * 1.6),
                    "outdoor Kanto rock must retain ochre rather than pale gray paint");
                assert!(mesh.solid.positions.iter().flatten().all(|v| v.is_finite()));
            }
        }
    }

    #[test]
    fn kanto_cave_warps_distinguish_upward_flights_and_downward_shafts() {
        let g=grid(4,4);
        for tileset in ["cave","dark_cave"] {
            for (block,art,datum,down) in [
                (0x07,[[0x2a,0x2b],[0x3a,0x3b]],0.,false),
                (0x14,[[0x2a,0x2b],[0x3a,0x3b]],0.,false),
                (0x0b,[[0x22,0x23],[0x32,0x33]],0.,true),
                (0x15,[[0x22,0x23],[0x32,0x33]],0.,true),
                (0x1b,[[0x28,0x29],[0x38,0x39]],6.,false),
                (0x1f,[[0x20,0x21],[0x30,0x31]],6.,true),
            ] {
                let mut t=tiles(4,4,tileset);
                for tile in &mut t {tile.source.metatile_id=0x02;tile.source.tile_index=0x16;}
                for y in 0..2 {for x in 0..2 {
                    let source=&mut t[(y+2)*4+x+2].source;
                    source.metatile_id=block;source.tile_index=art[y][x];
                }}
                for map in ["DiglettsCave","MountMoon","RockTunnel1F",
                    "RockTunnelB1F","TohjoFalls","VictoryRoad","UnionCave1F"] {
                    let cells:Vec<_>=t.iter().collect();
                    let placements=resolve(map,&cells,&g,None);
                    let p=placements.iter().find(|p|p.kind_label()=="dungeon:cave-floor-hatch").unwrap();
                    assert_eq!((p.column,p.row,p.width,p.height),(2,2,2,2));
                    assert_eq!(p.cuts_foundation(),down);
                    let mut mesh=TerrainMeshData{footing_heights:vec![datum;16],..Default::default()};
                    append(&mut mesh,&g,p,&cells,[0,0]);
                    assert_eq!(mesh.footing_heights,vec![datum;16]);
                    assert!(mesh.textured.positions.is_empty());
                    let min=mesh.solid.positions.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min);
                    let max=mesh.solid.positions.iter().map(|p|p[1]).fold(f32::NEG_INFINITY,f32::max);
                    if down {
                        assert_eq!(min,datum-13.);
                        assert!(max>=datum+5. && max<datum+5.2);
                        // Neither the lip nor a substrate cap can bridge the shaft.
                        let center=Vec3::new(24.,datum,24.);
                        for face in mesh.solid.positions.chunks_exact(4) {
                            if face.iter().all(|p|p[1]==datum) {
                                let lo=face.iter().fold(Vec3::splat(f32::INFINITY),|v,p|v.min(Vec3::from_array(*p)));
                                let hi=face.iter().fold(Vec3::splat(f32::NEG_INFINITY),|v,p|v.max(Vec3::from_array(*p)));
                                assert!(!(center.x>lo.x && center.x<hi.x && center.z>lo.z && center.z<hi.z));
                            }
                        }
                    } else { assert_eq!(min,datum); assert!(max>datum+26. && max<datum+28.); }
                }
                t[10].source.tile_index^=1;
                assert!(!resolve("MountMoon",&t.iter().collect::<Vec<_>>(),&g,None)
                    .iter().any(|p|p.kind_label()=="dungeon:cave-floor-hatch"));
            }
        }
    }

    #[test]
    fn cave_ladder_and_exit_substrates_share_the_surrounding_mineral_floor() {
        let g=grid(4,4);
        for tileset in ["cave","dark_cave"] {
            let mut t=tiles(4,4,tileset);
            for tile in &mut t {tile.source.metatile_id=0x02;tile.source.tile_index=0x16;}
            let cells:Vec<_>=t.iter().collect();
            for form in [Form::CaveHatch,Form::CaveSill] {
                let p=Placement {column:2,row:2,width:2,
                    height:if matches!(form,Form::CaveHatch){2}else{1},
                    ground:0,form,rise_pixels:6.,depth_pixels:16.,front_rows:2.};
                let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],..Default::default()};
                append(&mut mesh,&g,&p,&cells,[17,53]);
                assert!(mesh.textured.indices.is_empty(),
                    "modeled cave floors must not retain isolated flat atlas patches");
                assert_eq!(mesh.footing_heights,vec![0.;16]);
                let floor:Vec<_>=mesh.solid.positions.iter().zip(&mesh.solid.uvs)
                    .take(p.indices(g.width).count()*64)
                    .filter(|(_,uv)|uv[0]>90000. && (uv[1]/100000.).round()==4.)
                    .map(|(p,_)|p).collect();
                assert!(floor.len()>=p.indices(g.width).count()*64);
                assert!(floor.iter().all(|p|p[1]==0.),"finish must preserve the authored substrate datum");
                for i in p.indices(g.width) {
                    let (w,e,n,s)=g.bounds(i%g.width,i/g.width);
                    for point in [[w,0.,n],[w,0.,s],[e,0.,s],[e,0.,n]] {
                        assert!(floor.iter().any(|p|**p==point),"floor must close at each original corner");
                    }
                }
            }
        }
    }

    #[test]
    fn gym_orb_live_palette_requires_complete_drawings_and_preserves_floor() {
        for (tileset, blocks, art, maps, floor) in [
            ("elite_four_room", vec![(0x1d,0),(0x1e,2),(0x25,2),(0x26,0)],
             [[0x20,0x21],[0x30,0x31],[0x22,0x23],[0x32,0x33]],
             vec!["VioletGym","BlackthornGym1F","MahoganyGym","AzaleaGym","GoldenrodGym"], 1),
            ("tower", vec![(0x12,0),(0x13,2)],
             [[0x28,0x29],[0x38,0x39],[0x2a,0x2b],[0x3a,0x3b]],
             vec!["PewterGym","CianwoodGym","EcruteakGym"], 2),
        ] {
            for (block, ox) in blocks {
                let mut t = tiles(4,4,tileset);
                for tile in &mut t { tile.source.tile_index = floor; tile.source.metatile_id = if tileset == "tower" {9} else {0x2d}; }
                for y in 0..4 { for x in 0..2 {
                    let s = &mut t[y*4+ox+x].source;
                    s.metatile_id = block; s.tile_index = art[y][x];
                }}
                // Tower and elite25/26 forms are authored live-profile folds.
                // Exercise that production boundary rather than asking a
                // profile-free resolver to invent a matching drawing.
                let profiles: Document = serde_json::from_value(serde_json::json!({
                    "objects":[{"name":"scoped gym orb fixture","tileset":tileset,
                        "maps":maps,"metatile":block,"origin":[ox,0],
                        "tiles":art,"ground":floor,"top_pixels":0,"depth_pixels":10.}]
                })).unwrap();
                let g = grid(4,4);
                for &map in &maps {
                    let refs = t.iter().collect::<Vec<_>>();
                    let selected = resolve(map,&refs,&g,Some(&profiles));
                    let p = selected.iter().find(|p|matches!(p.form,Form::GymOrb))
                        .unwrap_or_else(|| panic!("missing live orb: {map} {tileset} block{block:02x}"));
                    assert_eq!(p.indices(4).count(),8);
                    assert!(p.indices(4).all(|i|(ox..ox+2).contains(&(i%4))));
                    let mut mesh = TerrainMeshData {footing_heights:vec![0.;16],..Default::default()};
                    append(&mut mesh,&g,p,&refs,[0,0]);
                    assert_eq!(mesh.footing_heights,vec![0.;16]);
                    assert!(mesh.textured.positions.iter().any(|v|v[1]>27.9));
                    for pixel in [[4,4],[7,3],[8,18],[8,16],[4,20]] {
                        let uv = g.uv(ox+pixel[0]/8,pixel[1]/8);
                        assert!(mesh.textured.uvs.contains(&[
                            uv.0+(uv.1-uv.0)*((pixel[0]%8) as f32+0.5)/8.,
                            uv.2+(uv.3-uv.2)*((pixel[1]%8) as f32+0.5)/8.,
                        ]));
                    }
                    for i in p.indices(4) { for field in 0..5 {
                        let mut altered = t.clone(); let s = &mut altered[i].source;
                        match field { 0=>s.metatile_id^=0x80, 1=>s.tile_index^=0x80,
                            2=>s.subtile_column^=0x80, 3=>s.subtile_row^=0x80,
                            _=>s.tileset_id="custom".into() }
                        assert!(resolve(map,&altered.iter().collect::<Vec<_>>(),&g,Some(&profiles))
                            .iter().all(|p|p.kind_label()!=Kind::LeaguePodium.label()));
                    }}
                }
                for map in ["OtherRoom","SproutTower1F","CeladonGameCorner"] {
                    assert!(resolve(map,&t.iter().collect::<Vec<_>>(),&g,Some(&profiles))
                        .iter().all(|p|!matches!(p.form,Form::GymOrb)));
                }
            }
        }
    }
    #[test]
    fn tin_tower_guardians_require_the_actual_complete_drawing_and_preserve_floor() {
        let g=grid(4,4);let mut t=tiles(4,4,"tower");
        for tile in &mut t {tile.source.metatile_id=9;tile.source.tile_index=2;}
        let art=[[0x22,0x23],[0x32,0x33],[0x12,0x13],[0x36,0x37]];
        for y in 0..4 {for x in 0..2 {
            let s=&mut t[y*4+x].source;s.metatile_id=0x21;s.tile_index=art[y][x];
        }}
        let refs:Vec<_>=t.iter().collect();
        for map in ["TinTower1F","TinTower2F","TinTower3F","TinTower4F","TinTower5F",
            "TinTower6F","TinTower7F","TinTower8F","TinTower9F","BurnedTower1F"] {
            let selected=resolve(map,&refs,&g,None);
            let p=selected.iter().find(|p|matches!(p.form,Form::NativeGymGuardian)).unwrap();
            assert_eq!(p.indices(4).count(),8);
            let mut mesh=TerrainMeshData {footing_heights:vec![0.;16],..Default::default()};
            append(&mut mesh,&g,p,&refs,[0,0]);
            assert_eq!(mesh.footing_heights,vec![0.;16]);
            assert!(mesh.textured.positions.iter().any(|p|p[1]>20.));
        }
        for map in ["TinTowerRoof","SproutTower1F","PlayersHouse1F"] {
            assert!(!resolve(map,&refs,&g,None).iter().any(|p|matches!(p.form,Form::NativeGymGuardian)));
        }
        t[0].source.tile_index^=1;
        assert!(!resolve("TinTower1F",&t.iter().collect::<Vec<_>>(),&g,None).iter()
            .any(|p|matches!(p.form,Form::NativeGymGuardian)));
    }
    #[test]
    fn sprout_rear_statues_keep_ceiling_wall_behind_them_without_claiming_neighbors() {
        for (block,offset) in [(0x2e,0),(0x2f,2)] {
            let g=grid(4,4);let mut t=tiles(4,4,"tower");
            let art=if offset==0 {
                [[0x42,0x43,0x11,0x48],[0x5a,0x5b,0x21,0x58],
                 [0x4c,0x5c,2,2],[0x36,0x37,2,2]]
            } else {
                [[0x49,0x11,0x42,0x43],[0x59,0x21,0x5a,0x5b],
                 [2,2,0x4c,0x5c],[2,2,0x36,0x37]]
            };
            for y in 0..4 {for x in 0..4 {
                t[y*4+x].source.metatile_id=block;t[y*4+x].source.tile_index=art[y][x];
            }}
            let refs=t.iter().collect::<Vec<_>>();
            let placements=resolve("SproutTower3F",&refs,&g,None);
            let p=placements.iter().find(|p|matches!(p.form,Form::SproutGuardian)).unwrap();
            assert_eq!(p.indices(4).count(),8);
            assert!(p.indices(4).all(|i|(offset..offset+2).contains(&(i%4))));
            let mut mesh=TerrainMeshData{footing_heights:vec![2.5;16],..Default::default()};
            append(&mut mesh,&g,p,&refs,[0,0]);
            let south=g.bounds(offset,1).3;
            assert!(mesh.textured.positions.iter().any(|v|v[1]>31.9));
            assert!(mesh.textured.positions.iter().filter(|v|v[1]>24.)
                .any(|v|v[2]>south-3.1));
            assert!(mesh.textured.positions.iter().filter(|v|v[1]>24.).all(|v|v[2]<=south+0.01));
            assert_eq!(mesh.footing_heights,vec![2.5;16]);
            // A changed adjacent drawing cannot supply invented wall paint.
            let wall_column=if offset==0 {2}else{1};t[wall_column].source.tile_index=2;
            let refs=t.iter().collect::<Vec<_>>();
            let mut changed=TerrainMeshData::default();append(&mut changed,&g,p,&refs,[0,0]);
            assert!(changed.textured.positions.iter().all(|v|v[1]<=24.));
        }
    }
    #[test]
    fn sprout_true_statues_exclude_entrance_carpet_and_require_complete_source() {
        let g=grid(8,4);
        let mut t=tiles(8,4,"tower");
        for tile in &mut t {tile.source.metatile_id=9;tile.source.tile_index=2;}
        let art=[[0x4a,0x4b],[0x5a,0x5b],[0x4c,0x5c],[0x36,0x37]];
        for y in 0..4 {for x in 0..2 {
            let s=&mut t[y*8+x].source;s.metatile_id=0x30;s.tile_index=art[y][x];
        }}
        for y in 2..4 {for x in 0..4 {
            let s=&mut t[y*8+x+4].source;s.metatile_id=0x26;
            s.tile_index=[[0x07,0x08,0x08,0x09],[0x17,0x18,0x18,0x19]][y-2][x];
            assert!(crate::tower::entrance_carpet_source(s));
        }}
        let refs:Vec<_>=t.iter().collect();
        let selected=resolve("SproutTower1F",&refs,&g,None);
        assert_eq!(selected.len(),1);
        assert_eq!(selected[0].indices(8).count(),8);
        assert!(selected[0].indices(8).all(|i|i%8<2));
        let mut mesh=TerrainMeshData {footing_heights:vec![0.;32],..Default::default()};
        append(&mut mesh,&g,&selected[0],&refs,[0,0]);
        assert!(!mesh.textured.indices.is_empty());
        assert_eq!(mesh.footing_heights,vec![0.;32]);
        assert!(mesh.textured.positions.iter().all(|p|p[1]>=0. && p[1]<=24.));
        assert!(resolve("TinTower1F",&refs,&g,None).is_empty());
        t[0].source.tile_index^=1;
        assert!(resolve("SproutTower1F",&t.iter().collect::<Vec<_>>(),&g,None).is_empty());
    }

    #[test]
    fn moving_cave_stones_are_ground_rooted_and_keep_a_consistent_game_object_extent() {
        for (frozen,dark) in [(false,false),(false,true),(true,false)] {
            for variant in 0..4 {
                let mesh=movable_stone(frozen,dark,variant);
                assert!(!mesh.indices.is_empty());
                let min=mesh.positions.iter().map(|p| p[1]).fold(f32::INFINITY,f32::min);
                let max=mesh.positions.iter().map(|p| p[1]).fold(0.,f32::max);
                assert_eq!(min,0.);
                assert!((max-0.64).abs()<0.00001);
                assert!(mesh.positions.iter().all(|p| p.iter().all(|v| v.is_finite())
                    && p[0].abs()<0.5 && p[2].abs()<0.5));
                assert!(mesh.normals.iter().all(|n| (Vec3::from_array(*n).length()-1.).abs()<0.00001));
                assert!(mesh.uvs.iter().flatten().all(|v| v.is_finite() && v.abs()<1.));
                if frozen {
                    let mut edges=std::collections::BTreeMap::new();
                    let key=|p:[f32;3]| p.map(|v|(v*100000.).round() as i32);
                    for ids in mesh.indices.chunks_exact(3) {
                        for i in 0..3 {
                            let mut edge=[key(mesh.positions[ids[i] as usize]),
                                key(mesh.positions[ids[(i+1)%3] as usize])];
                            edge.sort();
                            *edges.entry(edge).or_insert(0)+=1;
                        }
                    }
                    assert!(edges.values().all(|&n|n==2),"open moving stone {variant}");
                }
            }
        }
    }
    pub(super) fn grid(w: usize, h: usize) -> GridGeometry {
        GridGeometry {
            width: w,
            height: h,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        }
    }
    pub(super) fn tiles(w: usize, h: usize, tileset: &str) -> Vec<VisualTile> {
        (0..w * h)
            .map(|i| VisualTile {
                animation_frames: None,
                column: (i % w) as u32,
                row: (i / w) as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from(tileset),
                    metatile_id: 0x01,
                    subtile_column: (i % w % 4) as u8,
                    subtile_row: (i / w % 4) as u8,
                    tile_index: 0x16,
                },
                texture: Default::default(),
                priority: false,
            })
            .collect()
    }
    fn rock_fixture() -> Vec<VisualTile> {
        let mut t = tiles(4, 4, "cave");
        for y in 0..2 {
            for x in 0..2 {
                let s = &mut t[y * 4 + x].source;
                s.metatile_id = 0x18;
                s.tile_index = [[0x0c, 0x0d], [0x1c, 0x1d]][y][x];
            }
        }
        t
    }
    #[test]
    fn gym_stone_palette_requires_complete_room_scoped_drawings() {
        for (map, tileset, block, drawing, floor) in [
            ("OlivineGym", "champions_room", 0x11, [[0x46, 0x47], [0x56, 0x57]], crate::olivine_gym::GROUND_TILE),
            ("PewterGym", "tower", 0x32, [[0x04, 0x05], [0x14, 0x15]], crate::tower::TOWER_FLOOR_TILE),
            ("CianwoodGym", "tower", 0x32, [[0x04, 0x05], [0x14, 0x15]], crate::tower::TOWER_FLOOR_TILE),
        ] {
            let mut t = tiles(4, 4, tileset);
            for cell in &mut t { cell.source.tile_index = floor; }
            for y in 0..2 { for x in 0..2 {
                let s = &mut t[y * 4 + x].source;
                s.metatile_id = block;
                s.tile_index = drawing[y][x];
            }}
            let g = grid(4, 4);
            let p = resolve(map, &t.iter().collect::<Vec<_>>(), &g, None);
            assert_eq!(p.iter().filter(|p| matches!(p.form, Form::GymBoulder)).count(), 1);
            let rock = p.iter().find(|p| matches!(p.form, Form::GymBoulder)).unwrap();
            let mut mesh = TerrainMeshData { footing_heights: vec![7.; 16], ..Default::default() };
            append(&mut mesh, &g, rock, &t.iter().collect::<Vec<_>>(), [0, 0]);
            let sample = |pixel: [usize; 2]| {
                let uv = g.uv(pixel[0] / 8, pixel[1] / 8);
                [uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
                 uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.]
            };
            assert!(mesh.textured.uvs.contains(&sample([8, 6])));
            let highlight = if tileset == "tower" { [8, 1] } else { [4, 4] };
            assert!(mesh.textured.uvs.contains(&sample(highlight)));
            if tileset == "tower" {
                assert!(!mesh.textured.uvs.contains(&sample([4, 4])));
            }
            assert_eq!(mesh.footing_heights, vec![7.; 16]);
            for other in ["SproutTower1F", "OtherRoom"] {
                assert!(resolve(other, &t.iter().collect::<Vec<_>>(), &g, None)
                    .iter().all(|p| !matches!(p.form, Form::GymBoulder)));
            }
            for index in [0, 1, 4, 5] {
                let mut altered = t.clone();
                altered[index].source.tile_index ^= 0x80;
                assert!(resolve(map, &altered.iter().collect::<Vec<_>>(), &g, None)
                    .iter().all(|p| !matches!(p.form, Form::GymBoulder)));
            }
        }
    }
    #[test]
    fn seafoam_boulders_keep_full_volume_native_palette_and_floor_datum() {
        let t=rock_fixture();let g=grid(4,4);let cells=t.iter().collect::<Vec<_>>();
        let placements=resolve("SeafoamGym",&cells,&g,None);
        let rock=placements.iter().find(|p|matches!(p.form,Form::SeafoamBoulder)).unwrap();
        assert_eq!((rock.rise_pixels,rock.depth_pixels),(12.,14.));
        let base=boulder_datum(rock,&cells,&g);
        let mut expected=SurfaceMeshData::default();
        model(Kind::CaveBoulder).append(&mut expected,[0.,16.,2.,16.],base,12.);
        let mut mesh=TerrainMeshData {footing_heights:vec![7.;16],..Default::default()};
        append(&mut mesh,&g,rock,&cells,[0,0]);
        assert_eq!(mesh.textured.positions,expected.positions);
        assert_eq!(mesh.textured.normals,expected.normals);
        assert_eq!(mesh.textured.indices,expected.indices);
        assert_eq!(mesh.footing_heights,vec![7.;16]);
        for pixel in [[8,6],[4,4]] {
            let uv=g.uv(pixel[0]/8,pixel[1]/8);
            assert!(mesh.textured.uvs.contains(&[
                uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]));
        }
        for map in ["UnionCave1F","OtherRoom"] {
            assert!(resolve(map,&cells,&g,None).iter().all(|p|!matches!(p.form,Form::SeafoamBoulder)));
        }
        for i in [0,1,4,5] {
            let mut bad=t.clone();bad[i].source.tile_index=0xff;
            assert!(resolve("SeafoamGym",&bad.iter().collect::<Vec<_>>(),&g,None)
                .iter().all(|p|!matches!(p.form,Form::SeafoamBoulder)));
        }
    }
    #[test]
    fn cianwood_static_boulders_use_complete_3d_rocks() {
        let mut t = tiles(4, 4, "tower");
        for tile in &mut t { tile.source.tile_index = crate::tower::TOWER_FLOOR_TILE; }
        for y in 0..4 {
            for x in 0..2 {
                let s = &mut t[y * 4 + x].source;
                s.metatile_id = 0x35;
                s.tile_index = [[0x04, 0x05], [0x14, 0x15]][y % 2][x];
            }
        }
        let g = grid(4, 4);
        let cells: Vec<_> = t.iter().collect();
        let p = resolve("CianwoodGym", &cells, &g, None);
        assert_eq!(p.len(), 2);
        assert!(p.iter().all(|p| p.kind_label() == "dungeon:cave-boulder"));
        let mut mesh = TerrainMeshData { footing_heights: vec![0.0; 16], ..Default::default() };
        for rock in &p { append(&mut mesh, &g, rock, &cells, [0, 0]); }
        assert!(mesh.textured.positions.iter().any(|p| p[1] > 8.0));
        assert_eq!(mesh.footing_heights, vec![0.0; 16]);
        assert!(resolve("SproutTower1F", &cells, &g, None).is_empty());
        t[0].source.tile_index = 0xff;
        assert_eq!(resolve("CianwoodGym", &t.iter().collect::<Vec<_>>(), &g, None).len(), 1);
    }
    #[test]
    fn rocket_persian_requires_its_complete_sensor_drawing_and_preserves_support() {
        let mut t=tiles(4,4,"underground");
        for tile in &mut t {tile.source.tile_index=0x10;}
        let art=[[0x45,0x46],[0x55,0x56],[0x09,0x19],[0x30,0x31]];
        for y in 0..4 {for x in 0..2 {
            let s=&mut t[y*4+x].source;s.metatile_id=4;s.tile_index=art[y][x];
        }}
        let g=grid(4,4);let refs=t.iter().collect::<Vec<_>>();
        let placements=resolve("TeamRocketBaseB1F",&refs,&g,None);
        let statue=placements.iter().find(|p|matches!(p.form,Form::RocketPersian)).unwrap();
        assert_eq!(statue.indices(4).count(),8);
        assert_eq!((statue.rise_pixels,statue.depth_pixels),(28.,8.));
        let mut surface=TerrainMeshData{footing_heights:vec![72.;16],..Default::default()};
        append(&mut surface,&g,statue,&refs,[0,0]);
        assert!(surface.solid.positions.is_empty());
        assert_eq!(surface.footing_heights,vec![72.;16]);
        assert!(surface.textured.positions.iter().any(|v|v[1]==28.));
        for i in statue.indices(4) {for field in 0..5 {
            let mut altered=t.clone();let s=&mut altered[i].source;
            match field {0=>s.metatile_id^=0x80,1=>s.tile_index^=0x80,
                2=>s.subtile_column^=0x80,3=>s.subtile_row^=0x80,_=>s.tileset_id="other".into()}
            assert!(!resolve("TeamRocketBaseB1F",&altered.iter().collect::<Vec<_>>(),&g,None)
                .iter().any(|p|matches!(p.form,Form::RocketPersian)));
        }}
        for map in ["TeamRocketBaseB2F","GoldenrodUnderground","PlayersHouse1F"] {
            assert!(!resolve(map,&refs,&g,None).iter().any(|p|matches!(p.form,Form::RocketPersian)));
        }
    }
    #[test]
    fn pewter_guardians_require_all_eight_source_cells_and_preserve_adjacent_floor() {
        for (block, ox) in [(0x19, 0), (0x1a, 2)] {
            let mut t = tiles(4, 4, "tower");
            for tile in &mut t { tile.source.tile_index = crate::tower::TOWER_FLOOR_TILE; }
            let art = [[0x22, 0x23], [0x32, 0x33], [0x26, 0x27], [0x36, 0x37]];
            for y in 0..4 { for x in 0..2 {
                let s = &mut t[y * 4 + ox + x].source;
                s.metatile_id = block;
                s.tile_index = art[y][x];
            }}
            let g = grid(4, 4);
            let refs = t.iter().collect::<Vec<_>>();
            let p = resolve("PewterGym", &refs, &g, None);
            assert_eq!(p.len(), 1);
            assert_eq!(p[0].kind_label(), "dungeon:tower-guardian");
            assert_eq!(p[0].indices(4).count(), 8);
            assert!(p[0].indices(4).all(|i| (ox..ox + 2).contains(&(i % 4))));
            let mut mesh = TerrainMeshData { footing_heights: vec![72.; 16], ..Default::default() };
            append(&mut mesh, &g, &p[0], &refs, [0, 0]);
            assert_eq!(mesh.footing_heights, vec![72.; 16]);
            assert!(mesh.textured.positions.iter().any(|v| v[1] > 20.));
            for i in p[0].indices(4) { for field in 0..5 {
                let mut altered = t.clone();
                let s = &mut altered[i].source;
                match field { 0 => s.metatile_id ^= 0x80, 1 => s.tile_index ^= 0x80,
                    2 => s.subtile_column ^= 0x80, 3 => s.subtile_row ^= 0x80,
                    _ => s.tileset_id = "other".into() }
                assert!(resolve("PewterGym", &altered.iter().collect::<Vec<_>>(), &g, None).is_empty());
            }}
            for other in ["SproutTower1F", "CianwoodGym", "GoldenrodGameCorner"] {
                assert!(resolve(other, &refs, &g, None).is_empty());
            }
        }
    }
    #[test]
    fn cianwood_rear_guardians_claim_the_complete_statue_without_adjacent_wall() {
        for map in ["CianwoodGym", "EcruteakGym"] {
            for (block, ox) in [(0x21, 0), (0x22, 2)] {
                let mut t = tiles(4, 4, "tower");
                for tile in &mut t {
                    tile.source.metatile_id = 9;
                    tile.source.tile_index = 2;
                }
                let art = [[0x22, 0x23], [0x32, 0x33], [0x12, 0x13], [0x36, 0x37]];
                for y in 0..4 {
                    for x in 0..2 {
                        t[y * 4 + ox + x].source.metatile_id = block;
                        t[y * 4 + ox + x].source.tile_index = art[y][x];
                    }
                }
                let g = grid(4, 4);
                let cells: Vec<_> = t.iter().collect();
                let p = resolve(map, &cells, &g, None);
                assert_eq!(p.len(), 1);
                assert_eq!(p[0].kind_label(), "dungeon:tower-guardian");
                assert_eq!(p[0].indices(4).count(), 8);
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![0.0; 16],
                    ..Default::default()
                };
                append(&mut mesh, &g, &p[0], &cells, [0, 0]);
                assert_eq!(mesh.footing_heights, vec![0.0; 16]);
                assert!(mesh.solid.positions.iter().any(|p| p[1] > 20.0));
                t[ox].source.tile_index = 0xff;
                assert!(resolve(map, &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
            }
        }
    }
    #[test]
    fn cave_floor_hatches_require_complete_drawings_and_keep_flat_support() {
        for (block, drawing) in [
            (0x07, [[0x2a, 0x2b], [0x3a, 0x3b]]),
            (0x0b, [[0x22, 0x23], [0x32, 0x33]]),
            (0x14, [[0x2a, 0x2b], [0x3a, 0x3b]]),
            (0x15, [[0x22, 0x23], [0x32, 0x33]]),
            (0x1b, [[0x28, 0x29], [0x38, 0x39]]),
            (0x1f, [[0x20, 0x21], [0x30, 0x31]]),
        ] {
            for tileset in ["cave", "dark_cave"] {
                let mut t = tiles(4, 4, tileset);
                for y in 0..2 {
                    for x in 0..2 {
                        let cell = &mut t[(y + 2) * 4 + x + 2].source;
                        cell.metatile_id = block;
                        cell.tile_index = drawing[y][x];
                    }
                }
                let cells: Vec<_> = t.iter().collect();
                let g = grid(4, 4);
                let p = resolve("UnionCave1F", &cells, &g, None);
                assert_eq!(p.len(), 1);
                assert_eq!(p[0].kind_label(), "dungeon:cave-floor-hatch");
                assert_eq!(p[0].indices(4).collect::<Vec<_>>(), vec![10, 11, 14, 15]);
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![0.0; 16],
                    ..TerrainMeshData::default()
                };
                append(&mut mesh, &g, &p[0], &cells, [0, 0]);
                assert_eq!(mesh.footing_heights, vec![0.0; 16]);
                assert!(
                    mesh.textured.positions.is_empty(),
                    "complete hatch replaces all pixel planes"
                );
                let datum = if matches!(block,0x1b|0x1f) { 6. } else { 0. };
                let down = matches!(block,0x0b|0x15|0x1f);
                assert!(mesh.solid.positions.iter().all(|p| p[1] >= datum-if down {13.} else {0.}));
                let top=mesh.solid.positions.iter().map(|p|p[1]).fold(datum,f32::max);
                if down {assert!(top>=datum+5. && top<datum+5.2);} else {assert!(top>datum+26. && top<datum+28.);}
                assert!(mesh.solid.positions.iter().any(|p|p[1]==datum));
                assert!(mesh.solid.normals.iter().all(|n| Vec3::from_array(*n).is_finite()
                    && (Vec3::from_array(*n).length()-1.).abs()<1e-5));
                assert_eq!(p[0].cuts_foundation(),down,"only descending ladders cut the foundation");
                t[15].source.tile_index = 0xff;
                assert!(resolve("UnionCave1F", &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
            }
        }
    }

    #[test]
    fn ice_floor_hatches_keep_source_scope_and_exclude_hall_of_fame() {
        for (block, art) in [
            (0x03, [[0x0a, 0x0b], [0x1a, 0x1b]]),
            (0x07, [[0x2a, 0x2b], [0x3a, 0x3b]]),
            (0x37, [[0x0a, 0x0b], [0x1a, 0x1b]]),
            (0x38, [[0x2a, 0x2b], [0x3a, 0x3b]]),
        ] {
            let mut t = tiles(4, 4, "ice_path");
            for cell in &mut t {
                cell.source.tile_index = 0x19;
            }
            for y in 0..2 {
                for x in 0..2 {
                    let cell = &mut t[(y + 2) * 4 + x + 2].source;
                    cell.metatile_id = block;
                    cell.tile_index = art[y][x];
                }
            }
            let cells: Vec<_> = t.iter().collect();
            let g = grid(4, 4);
            let p = resolve("IcePathB3F", &cells, &g, None);
            assert_eq!(p.len(), 1);
            assert_eq!(p[0].kind_label(), "dungeon:ice-floor-hatch");
            assert!(resolve("HallOfFame", &cells, &g, None).is_empty());
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.0; 16],
                ..TerrainMeshData::default()
            };
            append(&mut mesh, &g, &p[0], &cells, [0, 0]);
            assert_eq!(mesh.footing_heights, vec![0.0; 16]);
            assert!(mesh.solid.positions.iter().all(|p| p[1] >= 0.0));
            let top=mesh.solid.positions.iter().map(|p|p[1]).fold(0.,f32::max);
            assert!(top>26. && top<28.,"the ice flight shares the reviewed cave ladder scale");
            assert!(mesh.solid.positions.iter().all(|p|p[0]>=16. && p[0]<=32.
                && p[2]>=16. && p[2]<=32.),"keep the complete native hatch footprint");
            t[15].source.metatile_id = 0xff;
            assert!(resolve("IcePathB3F", &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
        }
    }

    #[test]
    fn rock_height_prefers_its_own_authored_floor_over_a_neighboring_step() {
        for (floor, expected) in [(0x16,0.),(0x01,6.)] {
            let mut t=rock_fixture();
            for i in [2,6] { t[i].source.tile_index=0x01; }
            // One complete local floor sample suffices; neighboring lit paint
            // belongs to a different source block and cannot override it.
            t[10].source.metatile_id=0x18;
            t[10].source.tile_index=floor;
            let cells: Vec<_>=t.iter().collect();
            let g=grid(4,4);
            let p=resolve("UnionCave1F",&cells,&g,None);
            assert_eq!(boulder_datum(&p[0],&cells,&g),expected);
        }
    }
    #[test]
    fn cave_boulders_join_the_neighboring_shelf_without_changing_support() {
        let mut t = rock_fixture();
        for index in [2, 6] {
            t[index].source.tile_index = 0x01;
        }
        let cells = t.iter().collect::<Vec<_>>();
        let g = grid(4, 4);
        let p = resolve("UnionCave1F", &cells, &g, None);
        assert_eq!(p.len(), 1);
        assert_eq!(boulder_datum(&p[0], &cells, &g), 6.);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![0.; 16];
        append(&mut mesh, &g, &p[0], &cells, [0, 0]);
        assert!(mesh.textured.positions.is_empty());
        assert!(mesh.solid.positions.iter().any(|p| p[1] == 6.));
        let shapes: Vec<_> = cells.iter().map(|t| shape_for_source_on_map("UnionCave1F", &t.source)).collect();
        let joined = joined_surface_shapes(&p, &cells, &shapes, &g);
        for i in p[0].indices(g.width) {
            assert_eq!(joined[i].surface_height(g.tile_height), 6.);
            assert_eq!(shapes[i].surface_height(g.tile_height), 0.);
        }
        assert!(mesh.solid.positions.iter().all(|p| p[1] >= 6.));
        assert_eq!(mesh.footing_heights, vec![0.; 16]);
        let mut terrain = TerrainMeshData::default();
        let mut occupied = vec![false;16];
        for i in p[0].indices(g.width) { occupied[i] = true; }
        append_terrain(&mut terrain, "UnionCave1F", &cells, &joined, &g, &mut occupied, [0,0]);
        assert!(!terrain.solid.positions.iter().zip(&terrain.solid.normals).any(|(v,n)|
            v[0] == 16. && v[2] < 16. && v[1] < 6. && n[0] < -0.99),
            "shelf emitted a false downward face against the raised rock floor");
    }
    #[test]
    fn modeled_cave_shelf_floor_closes_its_exposed_side_without_duplicating_the_cap() {
        let t = tiles(2, 1, "cave");
        let cells = t.iter().collect::<Vec<_>>();
        let g = grid(2, 1);
        let shapes = [CellShape::PlaneAt { height: 6. }, CellShape::Flat];
        let mut claimed = [true, false];
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![6., 0.];
        append_terrain(&mut mesh, "UnionCave1F", &cells, &shapes, &g, &mut claimed, [0, 0]);
        assert!(!mesh.solid.indices.is_empty());
        for point in [[8., 0., 0.], [8., 0., 8.], [8., 6., 0.], [8., 6., 8.]] {
            assert!(mesh.solid.positions.contains(&point));
        }
        // The weathered shoulder can overhang the cell edge, but never the
        // lower walking center. Its floor and upper support joins stay exact.
        assert!(mesh.solid.positions.iter().all(|p| p[0] > 7. && p[0] < 12. && (0. ..=6.).contains(&p[1])));
        assert!(mesh.solid.positions.iter().filter(|p| p[1] == 0. || p[1] == 6.).all(|p| p[0] == 8.));
        assert!(mesh.solid.positions.iter().any(|p| p[0] > 8. && p[1] == 4.));
        assert!(mesh.solid.uvs.iter().all(|uv| (uv[1] / 100000.).round() == 6.));
        assert_eq!(mesh.footing_heights, vec![6., 0.]);
        assert_eq!(claimed, [true, false]);
    }
    #[test]
    fn ice_boulder_floor_uses_the_continuous_ground_or_ice_plane() {
        for (block, floor) in [(0x21,0x19),(0x2c,0xc6)] {
            let mut t = tiles(4,4,"ice_path");
            for tile in &mut t { tile.source.metatile_id=0x02; tile.source.tile_index=floor; }
            for y in 0..2 { for x in 0..2 {
                t[y*4+x].source.metatile_id=block;
                t[y*4+x].source.tile_index=[[0x82,0x83],[0x92,0x93]][y][x];
            }}
            let cells: Vec<_> = t.iter().collect();
            let g=grid(4,4);
            let p=resolve("IcePathB1F",&cells,&g,None);
            assert_eq!(p.len(),1);
            let mut mesh=TerrainMeshData::default();
            append(&mut mesh,&g,&p[0],&cells,[0,0]);
            assert!(mesh.textured.positions.is_empty());
            assert!(mesh.solid.positions.iter().all(|p|p[1]>=0.));
            assert!(mesh.solid.positions.iter().zip(&mesh.solid.normals).zip(&mesh.solid.uvs)
                .any(|((p,n),uv)|p[1]==0. && *n==[0.,1.,0.] &&
                    if floor==0x19 { (uv[1]/100000.).round()==5. } else { (uv[1]/100000.).round()==8. }));
        }
    }
    #[test]
    fn complete_rocks_reserve_only_their_four_native_cells() {
        let t = rock_fixture();
        let cells = t.iter().collect::<Vec<_>>();
        let g = grid(4, 4);
        let p = resolve("UnionCave1F", &cells, &g, None);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].indices(4).collect::<Vec<_>>(), vec![0, 1, 4, 5]);
        assert_eq!(p[0].kind_label(), "dungeon:cave-boulder");
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![3.0; 16];
        let original = mesh.footing_heights.clone();
        append(&mut mesh, &g, &p[0], &cells, [0, 0]);
        assert!(!mesh.solid.indices.is_empty());
        assert_eq!(mesh.footing_heights, original);
        // The complete rock and its continuous mineral floor have separate tags.
        assert!(mesh.solid.uvs.iter().all(|uv| uv[0]>90000. && matches!((uv[1]/100000.).round() as u8,4|6)));
        assert!(mesh.solid.uvs.iter().any(|uv| (uv[1]/100000.).round()==4.));
        assert!(mesh.textured.positions.is_empty());
    }
    #[test]
    fn clipped_corrupt_phase_and_missing_ground_rocks_fall_back() {
        let mut t = rock_fixture();
        t[5].source.tile_index = 0xff;
        assert!(
            resolve(
                "UnionCave1F",
                &t.iter().collect::<Vec<_>>(),
                &grid(4, 4),
                None
            )
            .is_empty()
        );
        let mut t = rock_fixture();
        t[5].source.subtile_column = 3;
        assert!(
            resolve(
                "UnionCave1F",
                &t.iter().collect::<Vec<_>>(),
                &grid(4, 4),
                None
            )
            .is_empty()
        );
        let mut t = rock_fixture();
        for a in &mut t {
            if a.source.tile_index == 0x16 {
                a.source.tile_index = 0xfe;
            }
        }
        assert!(
            resolve(
                "UnionCave1F",
                &t.iter().collect::<Vec<_>>(),
                &grid(4, 4),
                None
            )
            .is_empty()
        );
        let mut t = tiles(2, 1, "cave");
        for (x, a) in t.iter_mut().enumerate() {
            a.source.metatile_id = 0x18;
            a.source.tile_index = 0x0c + x as u16;
        }
        assert!(
            resolve(
                "UnionCave1F",
                &t.iter().collect::<Vec<_>>(),
                &grid(2, 1),
                None
            )
            .is_empty()
        );
    }
    #[test]
    fn warehouse_keeps_four_independent_crates_and_map_scope() {
        let mut t = tiles(5, 4, "underground");
        for y in 0..4 {
            for x in 0..4 {
                let s = &mut t[y * 5 + x].source;
                s.metatile_id = 0x0b;
                s.tile_index = [[0x43, 0x44], [0x53, 0x54]][y % 2][x % 2];
            }
        }
        for y in 0..4 {
            t[y * 5 + 4].source.tile_index = 0x01;
        }
        let c = t.iter().collect::<Vec<_>>();
        assert_eq!(
            resolve("GoldenrodDeptStoreB1F", &c, &grid(5, 4), None).len(),
            4
        );
        assert!(resolve("UndergroundPath", &c, &grid(5, 4), None).is_empty());
    }
    #[test]
    fn all_sixteen_wall_topologies_have_no_internal_side_faces() {
        for mask in 0u8..16 {
            let open = std::array::from_fn(|i| mask & (1 << i) != 0);
            let mut m = SurfaceMeshData::default();
            append_connected_wall(&mut m, [0., 8., 0., 8.], 16., open);
            assert_eq!(m.indices.len() / 3, 4 + 12 * mask.count_ones() as usize);
            assert!(
                m.normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.0).abs() < 1e-4)
            );
            let mut native=SurfaceMeshData::default();
            append_native_rocket_wall(&mut native,[0.,8.,0.,8.],32.,open,[0.1,0.2],[0.3,0.4]);
            assert_eq!(native.indices.len()/3,4+4*mask.count_ones()as usize);
            for (normal,uv) in native.normals.iter().zip(&native.uvs) {
                assert_eq!(*uv,if normal[1]>0.9 {[0.1,0.2]}else{[0.3,0.4]},
                    "black cap and blue exposed vertical faces have distinct native source roles");
            }
        }
    }
    #[test]
    fn shallow_terrace_faces_form_one_weathered_bank_without_changing_caps_or_support() {
        let mut t = tiles(2,2,"cave");
        for tile in &mut t { tile.source.tile_index=0xff; }
        for tile in &mut t[..2] { tile.source.tile_index=0x01; }
        let c = t.iter().collect::<Vec<_>>();
        let g = grid(2,2);
        let shapes = [CellShape::RaisedTop {height:6.,solid:SolidKind::Bank},
            CellShape::RaisedTop {height:6.,solid:SolidKind::Bank},CellShape::Flat,CellShape::Flat];
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights=vec![6.,6.,0.,0.];
        append_terrain(&mut mesh,"UnionCaveB2F",&c,&shapes,&g,&mut [false;4],[13,20]);
        assert_eq!(mesh.footing_heights,vec![6.,6.,0.,0.]);
        let face: Vec<_> = mesh.solid.positions.iter().zip(&mesh.solid.normals)
            .filter(|(_,n)| n[2]>0.5).map(|(p,_)| *p).collect();
        assert!(!face.is_empty());
        // The broader sloping shoulder extends beyond the old vertical
        // edge, while its lower recess and both walking datums stay exact.
        assert!(face.iter().all(|p| (6.9..=11.).contains(&p[2]) && p[1]>=0. && p[1]<=6.));
        assert!(face.iter().any(|p| p[1]==4. && p[2]>8.));
        assert!(face.iter().any(|p| p[0]==8. && p[1]==2. && p[2]<8.));
        // The tile seam belongs to the continuous recess. Only real contour
        // ends and the floor/cap joins retain the original edge plane.
        for y in [0.,2.,4.,6.] {
            for x in [0.,16.] {
                assert!(face.iter().any(|p| *p==[x,y,8.]));
            }
        }
        assert!(face.iter().filter(|p| p[1]==0. || p[1]==6.).all(|p| p[2]==8.));
        assert!(mesh.solid.positions.iter().zip(&mesh.solid.normals)
            .filter(|(_,n)| n[1]>0.99999).all(|(p,_)| p[1]==6.));
    }

    #[test]
    fn cave_edge_fragments_require_dry_source_and_preserve_support() {
        let g = grid(2, 1);
        let shapes = [
            CellShape::RaisedTop { height: 40., solid: SolidKind::Bank },
            CellShape::Flat,
        ];
        let mut saw_fragments = false;
        for x in 0..16 {
            let render = |block, tile_index| {
                let mut t = tiles(2, 1, "cave");
                t[0].source.tile_index = 0x26;
                t[1].source.metatile_id = block;
                t[1].source.tile_index = tile_index;
                let cells = t.iter().collect::<Vec<_>>();
                let mut m = TerrainMeshData::default();
                m.footing_heights = vec![40., 0.];
                let mut claimed = vec![false; 2];
                append_terrain(&mut m, "UnionCave1F", &cells, &shapes, &g, &mut claimed, [x, 20]);
                assert_eq!(m.footing_heights, vec![40., 0.]);
                assert_eq!(claimed, vec![true, false]);
                m.solid
            };
            let dry = render(0x01, 0x16);
            let water = render(0x3d, 0x16);
            let ladder = render(0x07, 0x2a);
            let unknown = render(0x01, 0xff);
            assert_eq!(water.positions, unknown.positions);
            assert_eq!(ladder.positions, unknown.positions);
            assert!(dry.indices.len() >= water.indices.len());
            saw_fragments |= dry.indices.len() > water.indices.len();
        }
        assert!(saw_fragments);
    }
    #[test]
    fn tiered_terrain_preserves_caps_hides_shared_sides_and_does_not_skirt_viewports() {
        let t = tiles(3, 1, "cave");
        let c = t.iter().collect::<Vec<_>>();
        let g = grid(3, 1);
        let shapes = [
            CellShape::RaisedTop {
                height: 32.,
                solid: SolidKind::Bank,
            },
            CellShape::RaisedTop {
                height: 16.,
                solid: SolidKind::Bank,
            },
            CellShape::Flat,
        ];
        let mut m = TerrainMeshData::default();
        m.footing_heights = vec![32., 16., 0.];
        let before = m.footing_heights.clone();
        let mut claimed = vec![false; 3];
        append_terrain(&mut m, "UnionCave1F", &c, &shapes, &g, &mut claimed, [0, 0]);
        assert_eq!(claimed, vec![true, true, false]);
        assert_eq!(m.footing_heights, before);
        // Count the rock domain separately from the floor substrate. Floor
        // refinement may change without adding hidden sides or viewport skirts.
        let domain = |tri: &[u32], tag: f32| tri.iter().all(|&i|
            (m.solid.uvs[i as usize][1] / 100000.).round() == tag);
        assert_eq!(m.solid.indices.chunks_exact(3).filter(|tri| domain(tri, 2.)).count(),
            192, "two caps and two exposed east faces; no hidden or viewport sides");
        assert!(m.solid.indices.chunks_exact(3).any(|tri| domain(tri, 4.)
            && tri.iter().all(|&i| m.solid.positions[i as usize][1] == 0.)),
            "the enclosing-bank substrate remains on the lower floor datum");
        assert!(m.solid.positions.iter().all(|p| p[1] >= 0.0));
    }
    #[test]
    fn enclosing_bank_keeps_relief_even_when_footing_cache_records_its_cap() {
        let mut t = tiles(3, 1, "dark_cave");
        // Native ceiling rock is bank evidence. Tile16 is a walking floor
        // and intentionally keeps an exact cap under the current crown rules.
        t[0].source.metatile_id = 0x2e;
        t[0].source.tile_index = 0x26;
        let c = t.iter().collect::<Vec<_>>();
        let g = grid(3, 1);
        let shapes = [
            CellShape::RaisedTop {
                height: 48.,
                solid: SolidKind::Bank,
            },
            CellShape::RaisedTop {
                height: 6.,
                solid: SolidKind::Bank,
            },
            CellShape::Flat,
        ];
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![48., 6., 0.];
        let mut claimed = vec![false; 3];
        append_terrain(
            &mut mesh,
            "DarkCaveVioletEntrance",
            &c,
            &shapes,
            &g,
            &mut claimed,
            [10, 20],
        );
        // The cache includes blocked bank caps too; it is not evidence that
        // every bank roof is a walkable shelf. Real enclosing rock must remain.
        assert!(mesh.solid.positions.iter().any(|p| p[1] > 48.5));
        assert_eq!(mesh.footing_heights, vec![48., 6., 0.]);
        assert!(mesh.solid.positions.iter().zip(&mesh.solid.uvs)
            .any(|(p, uv)| p[1] == 0. && p[0] < 8. && (uv[1] / 100000.).round() == 4.));
        for p in [[8., 6., 0.], [8., 6., 8.], [16., 6., 8.], [16., 6., 0.]] {
            assert!(mesh.solid.positions.contains(&p));
        }
        assert_eq!(claimed, vec![true, true, false]);
    }
    #[test]
    fn stair_ramp_ladder_and_unknown_cells_remain_unclaimed() {
        let t = tiles(3, 1, "cave");
        let c = t.iter().collect::<Vec<_>>();
        let shapes = [
            CellShape::RampNorth {
                north_height: 16.,
                south_height: 0.,
            },
            CellShape::PlaneAt { height: 0. },
            CellShape::Flat,
        ];
        let mut m = TerrainMeshData::default();
        let mut claimed = vec![false; 3];
        append_terrain(
            &mut m,
            "UnionCave1F",
            &c,
            &shapes,
            &grid(3, 1),
            &mut claimed,
            [0, 0],
        );
        assert_eq!(claimed, vec![false; 3]);
        assert!(m.solid.positions.is_empty());
    }
    #[test]
    fn kanto_fuchsia_guardians_own_the_real_blocks_in_the_complete_builder() {
        use bevy::prelude::{Assets,Image,Vec2,UVec2};
        use crystal_render_api::VisualWorldFrame;
        for (block,left) in [(26,2),(27,0)] {
            let mut t=tiles(4,4,"lab");let art=[[76,77],[92,93],[78,79],[94,95]];
            let mut assets=Assets::<Image>::default();let texture=assets.add(Image::default());
            for (i,tile) in t.iter_mut().enumerate() {let x=i%4;let y=i/4;
                tile.texture=texture.clone();tile.source.metatile_id=block;
                tile.source.tile_index=if (left..left+2).contains(&x) {art[y][x-left]}else{16};}
            let refs=t.iter().collect::<Vec<_>>();let g=grid(4,4);
            assert!(super::super::modeled_interiors::resolve("FuchsiaGym",&refs,&g,None).is_empty(),
                "blank walking floor and guardian art cannot become lab machines");
            let placements=resolve("FuchsiaGym",&refs,&g,None);
            let guardian=placements.iter().find(|p|matches!(p.form,Form::NativeGymGuardian)).unwrap();
            assert_eq!(guardian.indices(4).count(),8);
            let mut surface=TerrainMeshData{footing_heights:vec![3.;16],..Default::default()};
            append(&mut surface,&g,guardian,&refs,[0,0]);assert!(surface.solid.indices.is_empty());
            assert_eq!(surface.footing_heights,vec![3.;16]);
            let frame=VisualWorldFrame{active:true,map_id:"FuchsiaGym".into(),map_texture:texture,
                tile_size:Vec2::splat(8.),viewport_size:Vec2::splat(32.),grid_size:UVec2::new(4,4),tiles:t,
                ..Default::default()};
            let samples=TerrainImageSamples::default();
            let mesh=super::super::build_terrain_mesh_internal(&frame,Some(&samples),true,
                Some(super::super::modeled_interiors::built_in_profiles()),None,None).unwrap();
            for i in 0..16 {assert_eq!(mesh.authored_cells[i],
                if (left..left+2).contains(&(i%4)) {Some("dungeon:tower-guardian")}else{None});}
        }
    }
    #[test]
    fn vermilion_entrance_guardians_require_complete_drawings_and_keep_neighbor_floor() {
        for (block, left) in [(0x1d, 2), (0x1e, 0)] {
            let mut t = tiles(4, 4, "game_corner");
            let art = [[0x42, 0x43], [0x52, 0x53], [0x44, 0x45], [0x54, 0x55]];
            for y in 0..4 { for x in 0..4 {
                t[y * 4 + x].source.metatile_id = block;
                t[y * 4 + x].source.tile_index = if x >= left && x < left + 2 {
                    art[y][x - left]
                } else { 0x01 };
            }}
            let g = grid(4, 4);
            let p = resolve("VermilionGym", &t.iter().collect::<Vec<_>>(), &g, None);
            assert_eq!(p.len(), 1);
            assert_eq!(p[0].kind_label(), "dungeon:tower-guardian");
            assert_eq!(p[0].indices(4).count(), 8);
            assert!(p[0].indices(4).all(|i| i % 4 >= left && i % 4 < left + 2));
            assert!(resolve("GoldenrodGameCorner", &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
            t[left].source.tile_index ^= 1;
            assert!(resolve("VermilionGym", &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
        }
    }
    #[test]
    fn kanto_puzzle_cans_have_hollow_interiors_and_closed_thin_walls() {
        for scale in [0.5,1.,2.] {
            let mut mesh=SurfaceMeshData::default();
            append_open_gym_bin(&mut mesh,[0.,16.*scale,0.,12.*scale],12.*scale);
            let mut edges=std::collections::BTreeMap::new();
            for tri in mesh.indices.chunks_exact(3) {
                let points=[tri[0],tri[1],tri[2]].map(|i|mesh.positions[i as usize]);
                let [a,b,c]=points.map(Vec3::from_array);
                assert!((b-a).cross(c-a).dot(Vec3::from_array(mesh.normals[tri[0]as usize]))>0.);
                for j in 0..3 {let mut edge=[points[j].map(f32::to_bits),points[(j+1)%3].map(f32::to_bits)];
                    edge.sort();*edges.entry(edge).or_insert(0usize)+=1;}
                if points.iter().all(|p|p[1]==12.*scale) {
                    assert!(points.iter().all(|p|((p[0]-8.*scale).powi(2)+(p[2]-6.*scale).powi(2)).sqrt()>5.*scale),
                        "the top is a rim, with an open centre");
                }
            }
            assert!(edges.values().all(|&n|n==2),"the hollow wall and bottom form a closed shell");
            assert!(mesh.positions.iter().any(|p|*p==[8.*scale,scale,6.*scale]),"interior bottom stays deep inside the can");
        }
    }
    #[test]
    fn changed_puzzle_bin_source_removes_the_mesh_without_touching_neighbors() {
        let mut t = tiles(4, 4, "game_corner");
        for a in &mut t {
            a.source.tile_index = 0x01;
        }
        for y in 0..2 {
            for x in 0..2 {
                let source = &mut t[(y + 2) * 4 + x + 2].source;
                source.metatile_id = 0x1f;
                source.tile_index = [[0x46, 0x47], [0x56, 0x57]][y][x];
            }
        }
        let g = grid(4, 4);
        let p = resolve("VermilionGym", &t.iter().collect::<Vec<_>>(), &g, None);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].kind_label(), "dungeon:gym-bin");
        assert_eq!(p[0].indices(4).collect::<Vec<_>>(), vec![10, 11, 14, 15]);
        assert!(
            resolve(
                "GoldenrodGameCorner",
                &t.iter().collect::<Vec<_>>(),
                &g,
                None
            )
            .is_empty()
        );
        t[10].source.metatile_id = 1;
        t[10].source.tile_index = 1;
        assert!(resolve("VermilionGym", &t.iter().collect::<Vec<_>>(), &g, None).is_empty());
    }
    #[test]
    fn lighthouse_requires_complete_masonry_and_a_known_native_floor_sample() {
        let mut t = tiles(5, 4, "lighthouse");
        for y in 0..4 {
            for x in 0..4 {
                let source = &mut t[y * 5 + x].source;
                source.metatile_id = 0x3e;
                source.tile_index = [[0x5e, 0x5f, 0x5e, 0x5f], [0x4a, 0x4b, 0x4a, 0x4b]][y % 2][x];
            }
        }
        for y in 0..4 {
            let source = &mut t[y * 5 + 4].source;
            source.metatile_id = 0x27;
            source.tile_index = 0x2e;
        }
        let g = grid(5, 4);
        let p = resolve(
            "OlivineLighthouse1F",
            &t.iter().collect::<Vec<_>>(),
            &g,
            None,
        );
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].kind_label(), "dungeon:lighthouse-masonry");
        t[7].source.tile_index = 0xff;
        assert!(
            resolve(
                "OlivineLighthouse1F",
                &t.iter().collect::<Vec<_>>(),
                &g,
                None
            )
            .is_empty()
        );
    }
    #[test]
    fn external_burned_tower_perimeter_is_upright_and_keeps_fall_openings_free() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        if !path.is_file(){return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
        let map=&pack.data().maps["BurnedTower1F"];
        let art=&pack.runtime_files()["data/tilesets/tower_metatiles.bin"];
        let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
        let t=(0..w*h).map(|i| {
            let (x,y)=(i%w,i/w);let b=map.blocks[y/4*(w/4)+x/4];
            VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                source:VisualTileSource{tileset_id:"tower".into(),metatile_id:b,subtile_column:(x%4)as u8,
                    subtile_row:(y%4)as u8,tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
        }).collect::<Vec<_>>();
        let g=grid(w,h);let refs=t.iter().collect::<Vec<_>>();
        let walls=super::super::modeled_interiors::resolve("BurnedTower1F",&refs,&g,None);
        let mut claimed=vec![false;w*h];let mut mesh=TerrainMeshData{footing_heights:vec![3.;w*h],..Default::default()};
        for p in walls.iter().filter(|p|p.kind==crate::interior_models::ModelKind::WallTraditional) {
            assert!(super::super::modeled_interiors::append(&mut mesh,"BurnedTower1F",&refs,&g,p,&mut claimed,[0,0]));
        }
        let side=t.iter().enumerate().filter(|(_,t)|t.source.tile_index==0x41).map(|(i,_)|i).collect::<Vec<_>>();
        assert!(!side.is_empty());assert!(side.into_iter().all(|i|claimed[i]));
        assert!(mesh.textured.positions.iter().any(|p|p[1]>=32.));
        assert_eq!(mesh.footing_heights,vec![3.;w*h]);
        for warp in map.events.warps.iter().filter(|w|w.target_map.contains("BURNED_TOWER_B1F")) {
            for dy in 0..2 {for dx in 0..2 {
                assert!(!claimed[(warp.y as usize*2+dy)*w+warp.x as usize*2+dx],"fall opening stays free");
            }}
        }
        let objects=resolve("BurnedTower1F",&refs,&g,None);
        assert_eq!(objects.iter().filter(|p|matches!(p.form,Form::NativeGymGuardian)).count(),2);
        assert!(objects.iter().filter(|p|p.kind_label()==Kind::TimberWall.label()).all(|p|matches!(p.form,Form::TowerTimber)));
    }
    #[test]
    fn external_tin_tower_native_walls_ladders_and_custom_ownership_cover_all_nine_floors() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        if !path.is_file(){return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();let pack=loaded.pack();
        let art=&pack.runtime_files()["data/tilesets/tower_metatiles.bin"];
        let png=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/tower.png"]).unwrap().to_luma8();
        for level in 1..=9 {
            let name=format!("TinTower{level}F");let map=&pack.data().maps[&name];
            let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
            let t=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let b=map.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"tower".into(),metatile_id:b,subtile_column:(x%4)as u8,
                        subtile_row:(y%4)as u8,tile_index:art[b as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let refs=t.iter().collect::<Vec<_>>();let g=grid(w,h);let placements=resolve(&name,&refs,&g,None);
            let pillars=placements.iter().filter(|p|matches!(p.form,Form::Special(extension::Detail::TinPillar))).collect::<Vec<_>>();
            assert_eq!(pillars.len(),1,"{name} must model its complete center pillar");
            let mut pillar_mesh=TerrainMeshData{footing_heights:vec![5.;w*h],..Default::default()};
            append(&mut pillar_mesh,&g,pillars[0],&refs,[0,0]);
            assert_eq!(pillars[0].indices(w).count(),24);
            assert!(!pillar_mesh.animated_textured.indices.is_empty());
            assert_eq!(pillar_mesh.animated_pillars,vec![[pillars[0].column,pillars[0].row]]);
            assert_eq!(pillar_mesh.footing_heights,vec![5.;w*h]);
            let interiors=super::super::modeled_interiors::resolve(&name,&refs,&g,None);
            let mut reserved=vec![false;w*h];for p in &interiors {for i in p.indices(w){reserved[i]=true;}}
            let mut expected=0;
            for warp in &map.events.warps {
                let (x,y)=(warp.x as usize*2,warp.y as usize*2);let s=&t[y*w+x].source;
                if matches!(s.metatile_id,2|3|0x1f|0x20)&&matches!(s.tile_index,0x0c|0x0e) {
                    expected+=1;
                    assert!(placements.iter().any(|p|p.column==x&&p.row==y
                        &&matches!(p.form,Form::Room(rooms::Detail::TowerLadder{..},_))),"{name} warp {},{}",warp.x,warp.y);
                }
            }
            let ladders=placements.iter().filter(|p|matches!(p.form,Form::Room(rooms::Detail::TowerLadder{..},_))).collect::<Vec<_>>();
            assert_eq!(expected,[1,2,2,4,4,2,1,0,1][level-1],"{name}: upper-floor teleport pads are not ladders");
            assert_eq!(ladders.len(),expected);
            for p in ladders {
                assert!(p.indices(w).all(|i|!reserved[i]),"perimeters cannot steal a stair warp");
                let down=matches!(p.form,Form::Room(rooms::Detail::TowerLadder{down:true},_));
                let samples=if down {[[0,0],[0,1],[2,2]]}else{[[1,5],[2,11],[0,0]]};
                for (pixel,value) in samples.into_iter().zip([170,85,0]) {
                    let s=&t[(p.row+pixel[1]/8)*w+p.column+pixel[0]/8].source;
                    assert_eq!(png.get_pixel((s.tile_index as u32%16)*8+(pixel[0]%8)as u32,
                        (s.tile_index as u32/16)*8+(pixel[1]%8)as u32)[0],value,"own ladder palette role");
                }
                let mut mesh=TerrainMeshData{footing_heights:vec![5.;w*h],..Default::default()};
                append(&mut mesh,&g,p,&refs,[0,0]);assert!(mesh.solid.positions.is_empty());
                assert_eq!(mesh.footing_heights,vec![5.;w*h]);
                assert_eq!(mesh.textured.positions.iter().any(|v|v[1]<0.),down);
            }
            if let Some(p)=interiors.iter().find(|p|p.kind==crate::interior_models::ModelKind::WallTraditional) {
                let s=&t[p.row*w+p.column].source;
                let mut custom=super::super::modeled_interiors::built_in_profiles().objects[0].clone();
                custom.name="Custom Tin perimeter".into();custom.map=Some(name.clone());custom.maps=None;
                custom.tileset="tower".into();custom.metatile=s.metatile_id;custom.metatiles=None;
                custom.origin=[s.subtile_column,s.subtile_row];custom.ground=2;custom.top_pixels=0;
                custom.depth_pixels=3.;custom.footing_pixels=None;custom.parts.clear();
                custom.tiles=(0..p.height).map(|y|(0..p.width).map(|x|t[(p.row+y)*w+p.column+x].source.tile_index).collect()).collect();
                let doc=Document{objects:vec![custom],atmosphere:None};
                assert!(!super::super::modeled_interiors::resolve(&name,&refs,&g,Some(&doc)).iter()
                    .any(|v|v.indices(w).any(|i|p.indices(w).any(|j|i==j))),"custom source keeps ownership");
            }
        }
    }
    #[test]
    fn tin_tower_threshold_and_grating_wall_courses_join_without_claiming_floor() {
        let mut t=tiles(12,4,"tower");
        for y in 0..4 {for x in 0..12 {
            let s=&mut t[y*12+x].source;
            s.metatile_id=[0x05,0x18,0x3b][x/4];
            s.subtile_column=(x%4)as u8;s.subtile_row=y as u8;
            s.tile_index=if y<2 {[0x11,0x21][y]}else{2};
        }}
        let g=grid(12,4);let refs=t.iter().collect::<Vec<_>>();
        let p=resolve("TinTower6F",&refs,&g,None);
        assert_eq!(p.len(),3);
        for (n,p) in p.iter().enumerate() {
            assert!(matches!(p.form,Form::TowerTimber));
            assert_eq!((p.column,p.row,p.width,p.height),(n*4,0,4,2));
            assert_eq!((p.rise_pixels,p.depth_pixels,p.front_rows),(32.,3.,2.));
            assert!(p.indices(12).all(|i|i/12<2));
        }
        // A changed native face must not be silently filled over a profile.
        t[4].source.tile_index=2;
        let changed=resolve("TinTower6F",&t.iter().collect::<Vec<_>>(),&g,None);
        assert_eq!(changed.len(),2);
        assert!(changed.iter().all(|p|p.column!=4));
    }
    #[test]
    fn tower_timber_live_palette_consumes_two_face_rows_and_leaves_floor_free() {
        let mut t = tiles(4, 4, "tower");
        for y in 0..4 {
            for x in 0..4 {
                let source = &mut t[y * 4 + x].source;
                source.metatile_id = 0x05;
                source.tile_index = if y == 0 {
                    0x11
                } else if y == 1 {
                    0x21
                } else {
                    0x02
                };
            }
        }
        let p = resolve(
            "SproutTower1F",
            &t.iter().collect::<Vec<_>>(),
            &grid(4, 4),
            None,
        );
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].kind_label(), "dungeon:timber-wall");
        assert_eq!(
            p[0].indices(4).collect::<Vec<_>>(),
            (0..8).collect::<Vec<_>>()
        );
        assert!(matches!(p[0].form, Form::TowerTimber));
        let g = grid(4, 4);
        let refs = t.iter().collect::<Vec<_>>();
        let mut mesh = TerrainMeshData { footing_heights: vec![0.; 16], ..Default::default() };
        append(&mut mesh, &g, &p[0], &refs, [0, 0]);
        assert!(!mesh.textured.indices.is_empty());
        assert!(mesh.textured.positions.iter().any(|v| v[1] > 31.9));
        assert_eq!(mesh.footing_heights, vec![0.; 16]);
        let uv = g.uv(0, 0);
        for pixel in [[0, 0], [2, 3], [0, 1], [2, 0]] {
            assert!(mesh.textured.uvs.contains(&[
                uv.0 + (uv.1 - uv.0) * (pixel[0] as f32 + 0.5) / 8.,
                uv.2 + (uv.3 - uv.2) * (pixel[1] as f32 + 0.5) / 8.,
            ]));
        }
        assert!(!mesh.textured.uvs.contains(&[uv.0+(uv.1-uv.0)*1.5/8.,
            uv.2+(uv.3-uv.2)*3.5/8.]),"native white glint cannot paint the full rail");
        for name in (1..=9).map(|level|format!("TinTower{level}F")).chain(["BurnedTower1F".into()]) {
            let tin=resolve(&name,&refs,&g,None);
            assert_eq!(tin.len(),1);assert!(matches!(tin[0].form,Form::TowerTimber));
        }
        let ordinary = resolve("TinTowerRoof", &refs, &g, None);
        assert!(ordinary.iter().all(|p| !matches!(p.form, Form::TowerTimber)));
    }
    #[test]
    fn sprout_ladders_require_whole_source_drawings_and_keep_support() {
        for (block, art, down) in [(0x1f, [[0x0e,0x0f],[0x1e,0x1f]], true), (0x20, [[0x0c,0x0d],[0x1c,0x1d]], false)] {
            let mut t = tiles(4,4,"tower");
            for tile in &mut t { tile.source.metatile_id=9; tile.source.tile_index=2; }
            for y in 0..2 { for x in 0..2 {
                let s=&mut t[y*4+x].source; s.metatile_id=block; s.tile_index=art[y][x];
            }}
            let g=grid(4,4);
            for map in ["SproutTower1F","SproutTower2F","SproutTower3F"] {
                let cells=t.iter().collect::<Vec<_>>();
                let p=resolve(map,&cells,&g,None);
                assert_eq!(p.len(),1);
                assert_eq!(p[0].indices(4).collect::<Vec<_>>(),vec![0,1,4,5]);
                let mut mesh=TerrainMeshData::default(); mesh.footing_heights=vec![0.;16];
                append(&mut mesh,&g,&p[0],&cells,[0,0]);
                assert_eq!(mesh.footing_heights,vec![0.;16]);
                assert!(mesh.solid.positions.is_empty(),"ladder uses its own native palette");
                assert!(mesh.textured.positions.iter().all(|v| v[0]>=0. && v[0]<=16. && v[2]>=0. && v[2]<=16.));
                assert_eq!(mesh.textured.positions.iter().any(|v|v[1]<0.),down);
                assert_eq!(mesh.textured.positions.iter().any(|v|v[1]>20.),!down);
            }
            for level in 1..=9 {
                let tin=resolve(&format!("TinTower{level}F"),&t.iter().collect::<Vec<_>>(),&g,None);
                assert_eq!(tin.iter().filter(|p|matches!(p.form,Form::Room(rooms::Detail::TowerLadder{..},_))).count(),1);
            }
            assert!(resolve("TinTowerRoof",&t.iter().collect::<Vec<_>>(),&g,None).iter()
                .all(|p|!matches!(p.form,Form::Room(rooms::Detail::TowerLadder{..},_))));
            for i in [0,1,4,5] {
                let mut changed=t.clone(); changed[i].source.tile_index^=0x80;
                assert!(resolve("SproutTower1F",&changed.iter().collect::<Vec<_>>(),&g,None).is_empty());
            }
        }
    }
    #[test]
    fn sprout_northeast_ladders_bind_the_lower_right_quadrant_only() {
        for (block, art, label) in [(2, [[0x0e,0x0f],[0x1e,0x1f]], "tower:timber-ladder-down"), (3, [[0x0c,0x0d],[0x1c,0x1d]], "tower:timber-ladder-up")] {
            let mut t=tiles(4,4,"tower");
            for tile in &mut t { tile.source.metatile_id=9; tile.source.tile_index=2; }
            for y in 0..2 { for x in 0..2 {
                let s=&mut t[(y+2)*4+x+2].source; s.metatile_id=block; s.tile_index=art[y][x];
            }}
            let g=grid(4,4); let cells=t.iter().collect::<Vec<_>>();
            let p=resolve("SproutTower1F",&cells,&g,None);
            assert_eq!(p.len(),1); assert_eq!(p[0].kind_label(),label);
            assert_eq!(p[0].indices(4).collect::<Vec<_>>(),vec![10,11,14,15]);
            t[10].source.subtile_column=0;
            assert!(resolve("SproutTower1F",&t.iter().collect::<Vec<_>>(),&g,None).is_empty());
        }
    }
    #[test]
    fn cave_straight_walls_join_the_complete_diagonal_corner_height() {
        for (corner,block,ox,art) in [
            (crate::cave::DiagonalCorner::SouthEast,0x10,2,[[0x0a,0x26],[0x17,0x0a]]),
            (crate::cave::DiagonalCorner::SouthWest,0x11,0,[[0x26,0x0b],[0x0b,0x15]]),
        ] {
            for dark in [false,true] {
                let g=grid(8,8);
                let mut t=tiles(8,8,if dark {"dark_cave"} else {"cave"});
                for tile in &mut t {tile.source.metatile_id=2;tile.source.tile_index=0x16;}
                for y in 0..2 {for x in 0..2 {
                    let src=&mut t[(y+2)*8+x+2].source;
                    src.metatile_id=block;src.subtile_column=ox+x as u8;
                    src.subtile_row=2+y as u8;src.tile_index=art[y][x];
                }}
                let side=match corner {crate::cave::DiagonalCorner::SouthEast=>4,
                    crate::cave::DiagonalCorner::SouthWest=>1};
                let walls=[4*8+2,4*8+3,2*8+side,3*8+side];
                for i in walls {t[i].source.metatile_id=0x2e;t[i].source.tile_index=0x26;}
                let refs=t.iter().collect::<Vec<_>>();
                assert_eq!(diagonal_cave_corner_placements(&refs,&g).len(),1);
                let shapes=t.iter().map(|t|crate::cave::cave_shape(&t.source)
                    .unwrap_or(CellShape::Flat)).collect::<Vec<_>>();
                let mut claimed=vec![true;64];for i in walls {claimed[i]=false;}
                let mut actual=TerrainMeshData::default();
                append_terrain(&mut actual,"SilverCaveRoom3",&refs,&shapes,&g,&mut claimed.clone(),[0,0]);
                let mut joined_shapes=shapes.clone();
                let h=if dark {48.} else {40.};
                for y in 2..4 {for x in 2..4 {joined_shapes[y*8+x]=CellShape::RaisedTop {
                    height:h,solid:SolidKind::Bank};}}
                let mut joined=TerrainMeshData::default();
                append_terrain(&mut joined,"SilverCaveRoom3",&refs,&joined_shapes,&g,&mut claimed,[0,0]);
                assert_eq!(actual.solid.indices.len(),joined.solid.indices.len(),
                    "straight wall emits buried faces against a complete tall corner");
                assert_eq!(actual.solid.positions,joined.solid.positions);
                let mut broken=t.clone();broken[2*8+2].source.tile_index^=0x80;
                let refs=broken.iter().collect::<Vec<_>>();
                let mut fallback=TerrainMeshData::default();
                let mut claimed=vec![true;64];for i in walls {claimed[i]=false;}
                append_terrain(&mut fallback,"SilverCaveRoom3",&refs,&shapes,&g,&mut claimed,[0,0]);
                assert!(fallback.solid.indices.len()>joined.solid.indices.len(),
                    "partial source drawings must retain the straight wall closure");
            }
        }
    }

    #[test]
    fn cave_corner_omits_only_closures_buried_in_proven_rock() {
        let g = grid(6,6);
        for corner in [crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest] {
            for dark in [false,true] {
                let mut t = tiles(6,6,if dark {"dark_cave"} else {"cave"});
                for tile in &mut t {
                    tile.source.metatile_id=0x2e;
                    tile.source.tile_index=0x26;
                }
                let refs=t.iter().collect::<Vec<_>>();
                let h=if dark {48.} else {40.};
                let mut closed=SurfaceMeshData::default();
                append_corner(&mut closed,[16.,32.,16.,32.],h,corner,dark,&g,[0,0],None);
                let mut joined=SurfaceMeshData::default();
                append_corner(&mut joined,[16.,32.,16.,32.],h,corner,dark,&g,[0,0],Some(&refs));
                assert_eq!(closed.indices.len()-joined.indices.len(),2*8*(h as usize/2)*6,
                    "only the two buried side closures should be omitted");
                // The cap and exposed diagonal still reach the real wall height.
                assert!(joined.positions.iter().any(|p|p[1]>=h));
                assert!(joined.positions.iter().any(|p|p[1]==0.));
                let mut interrupted=t.clone();
                let column=match corner {
                    crate::cave::DiagonalCorner::SouthEast=>4,
                    crate::cave::DiagonalCorner::SouthWest=>1,
                };
                let open=&mut interrupted[2*6+column].source;
                open.metatile_id=0x02;
                open.tile_index=0x16;
                let refs=interrupted.iter().collect::<Vec<_>>();
                let mut partial=SurfaceMeshData::default();
                append_corner(&mut partial,[16.,32.,16.,32.],h,corner,dark,&g,[0,0],Some(&refs));
                assert_eq!(partial.indices.len()-joined.indices.len(),4*(h as usize/2)*6,
                    "a one-cell opening must retain that half of the side closure");
                for tile in &mut t {tile.source.tileset_id="custom".into();}
                let refs=t.iter().collect::<Vec<_>>();
                let mut unknown=SurfaceMeshData::default();
                append_corner(&mut unknown,[16.,32.,16.,32.],h,corner,dark,&g,[0,0],Some(&refs));
                assert_eq!(unknown.indices.len(),closed.indices.len(),
                    "unknown neighboring art must not open a corner body");
            }
        }
    }

    #[test]
    fn cave_corner_lower_diagonal_is_recessed_with_an_exact_ground_foot() {
        for corner in [crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest] {
            let mut mesh=SurfaceMeshData::default();
            append_corner(&mut mesh,[0.,16.,0.,16.],40.,corner,false,&grid(2,2),[0,0],None);
            let distance=|p:&[f32;3]|match corner {
                crate::cave::DiagonalCorner::SouthEast=>p[0]+p[2]-16.,
                crate::cave::DiagonalCorner::SouthWest=>p[2]-p[0],
            };
            assert!(mesh.positions.iter().any(|p|p[1]==2. && p[0]>6. && p[0]<10.
                && distance(p)>0.5),"the lower face must have an actual curved recess");
            assert!(mesh.positions.iter().any(|p|p[1]==0. && p[0]==8. && p[2]==8.),
                "the original diagonal floor contact must stay exact");
            assert!(mesh.positions.iter().any(|p|p[1]==6. && p[0]==8. && p[2]==8.),
                "the unchanged course must join above the recess");
        }
    }

    #[test]
    fn cave_corner_diagonal_body_and_cap_share_rounding_above_the_walking_course() {
        let g=grid(2,2);
        for corner in [crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest] {
            for h in [40.,48.] {
                let mut mesh=SurfaceMeshData::default();
                append_corner(&mut mesh,[0.,16.,0.,16.],h,corner,h==48.,&g,[0,0],None);
                let inward=match corner {
                    crate::cave::DiagonalCorner::SouthEast=>(Vec3::X+Vec3::Z).normalize(),
                    crate::cave::DiagonalCorner::SouthWest=>(-Vec3::X+Vec3::Z).normalize(),
                };
                for y in [22.,h] {
                    let original=Vec3::new(8.,y,8.);
                    let planar=cave_cliff::rock_point(original,&g,[0,0]);
                    let rounded=cave_cliff::rock_point(original+inward*0.96,&g,[0,0]);
                    assert!(mesh.positions.iter().any(|p|Vec3::from_array(*p).distance(rounded)<0.001),
                        "the diagonal body and cap must share a bowed center at {y}");
                    assert!(!mesh.positions.iter().any(|p|Vec3::from_array(*p).distance(planar)<0.001),
                        "the tall corner must not retain its planar diagonal sheet");
                }
                for original in [Vec3::new(8.,0.,8.),Vec3::new(8.,6.,8.)] {
                    assert!(mesh.positions.iter().any(|p|Vec3::from_array(*p).distance(original)<0.001));
                }
            }
        }
    }

    #[test]
    fn cave_rounded_diagonal_caps_and_faces_keep_outward_winding_across_map_coordinates() {
        let g=grid(2,2);
        let mut origins=vec![[15,32],[17,53],[79,143]];
        for z in (0..144).step_by(8) { for x in (0..80).step_by(8) {
            origins.push([x,z]);
        }}
        for corner in [crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest] {
            let outward=match corner {
                crate::cave::DiagonalCorner::SouthEast=>-Vec3::X-Vec3::Z,
                crate::cave::DiagonalCorner::SouthWest=>Vec3::X-Vec3::Z,
            };
            for h in [40.,48.] { for &origin in &origins {
                let mut mesh=SurfaceMeshData::default();
                append_corner(&mut mesh,[0.,16.,0.,16.],h,corner,h==48.,&g,origin,None);
                // The eight subdivisions of the triangular cap produce 64
                // triangles; the following range is the exposed diagonal.
                let cap_indices=8*8*3;
                let diagonal_indices=8*(h as usize/2)*6;
                for (i,triangle) in mesh.indices[..cap_indices+diagonal_indices]
                    .chunks_exact(3).enumerate() {
                    let p: [Vec3;3]=std::array::from_fn(|j|
                        Vec3::from_array(mesh.positions[triangle[j] as usize]));
                    let n=(p[1]-p[0]).cross(p[2]-p[0]);
                    let expected=if i<64 {Vec3::Y} else {outward};
                    assert!(n.dot(expected)>0.,
                        "rounded corner folds at {origin:?}, height {h}, triangle {i}: {n:?}");
                }
            }}
        }
    }

    #[test]
    fn cave_corner_caps_and_sides_close_at_tall_source_heights() {
        for corner in [crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest] {
            for h in [40.,48.] {for origin in [[0,0],[17,53],[79,143]] {
                let mut mesh=SurfaceMeshData::default();
                append_corner(&mut mesh,[0.,16.,0.,16.],h,corner,h==48.,&grid(2,2),origin,None);
                let key=|p:[f32;3]|p.map(|v|(v*10000.).round() as i32);
                let mut edges=std::collections::BTreeMap::new();
                for tri in mesh.indices.chunks_exact(3) {
                    for i in 0..3 {
                        let mut edge=[key(mesh.positions[tri[i] as usize]),
                            key(mesh.positions[tri[(i+1)%3] as usize])];
                        edge.sort();*edges.entry(edge).or_insert(0)+=1;
                    }
                }
                for (edge,count) in edges {
                    assert!(count==2 || (count==1 && edge.iter().all(|p|p[1]==0)),
                        "open or overlapping corner seam at height {h}, origin {origin:?}: {edge:?}, {count}");
                }
            }}
        }
    }

    #[test]
    fn cave_corner_bases_stay_inside_the_authored_half_block() {
        for corner in [crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest] {
            for height in [40., 48.] {
                let mut mesh = SurfaceMeshData::default();
                append_corner(&mut mesh, [0.,16.,0.,16.], height, corner,
                    false, &grid(2,2), [0,0], None);
                for p in mesh.positions.iter().filter(|p|p[1]<=6.) {
                    assert!((0. ..=16.).contains(&p[0]) && (0. ..=16.).contains(&p[2]),
                        "corner base projects past the adjoining wall into the floor: {p:?}");
                    let inside = match corner {
                        crate::cave::DiagonalCorner::SouthEast => p[0]+p[2]>=16.-0.001,
                        crate::cave::DiagonalCorner::SouthWest => p[2]>=p[0]-0.001,
                    };
                    assert!(inside, "corner foot expands outside its authored diagonal: {p:?}");
                }
            }
        }
    }

    #[test]
    fn cave_corner_mirrors_keep_upward_caps_and_nonzero_outward_normals() {
        for corner in [
            crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest,
        ] {
            let mut m = SurfaceMeshData::default();
            append_corner(
                &mut m,
                [0., 16., 0., 16.],
                16.,
                corner,
                false,
                &grid(2, 2),
                [0, 0],
                None,
            );
            for ids in m.indices.chunks_exact(3) {
                let p = ids
                    .iter()
                    .map(|&i| Vec3::from_array(m.positions[i as usize]))
                    .collect::<Vec<_>>();
                let n = Vec3::from_array(m.normals[ids[0] as usize]);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.0);
                if p.iter().all(|p|p.y<=6.) {
                    let center=(p[0]+p[1]+p[2])/3.;
                    let expected=if (center.z-16.).abs()<0.001 {Vec3::Z}
                        else if (center.x-16.).abs()<0.001 {Vec3::X}
                        else if center.x.abs()<0.001 {-Vec3::X}
                        else {match corner {
                            crate::cave::DiagonalCorner::SouthEast=>-Vec3::X-Vec3::Z,
                            crate::cave::DiagonalCorner::SouthWest=>Vec3::X-Vec3::Z,
                        }};
                    assert!(n.dot(expected)>0.,"corner face points into the rock: {center:?} {n:?}");
                }
            }
        }
    }
    #[test]
    fn ramp_neighbor_uses_exact_touching_edge_heights() {
        let n = CellShape::RampNorth {
            north_height: 16.,
            south_height: 8.,
        };
        assert_eq!(neighbor_edge_heights(n, 0, 1.), [8., 8.]);
        assert_eq!(neighbor_edge_heights(n, 2, 1.), [16., 16.]);
        let e = CellShape::RampEast {
            west_height: 2.,
            east_height: 10.,
        };
        assert_eq!(neighbor_edge_heights(e, 1, 1.), [2., 2.]);
        assert_eq!(neighbor_edge_heights(e, 3, 1.), [10., 10.]);
    }
    #[test]
    fn ship_room_adapter_does_not_reclaim_partial_full_beds_or_stools() {
        let g = GridGeometry {
            width: 4,
            height: 6,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        for (block, row_offset, drawing) in [
            (0x36, 0, [[0x46, 0x47], [0x56, 0x57]]),
            (0x38, 2, [[0x46, 0x47], [0x56, 0x57]]),
            (0x07, 2, [[0x07, 0x08], [0x17, 0x18]]),
        ] {
            let mut cells: Vec<_> = (0..24)
                .map(|i| VisualTile {
                    animation_frames: None,
                    column: (i % 4) as u32,
                    row: (i / 4) as u32,
                    texture: Default::default(),
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: std::sync::Arc::from("lighthouse"),
                        metatile_id: 0x0b,
                        subtile_column: (i % 4) as u8,
                        subtile_row: (i / 4 % 4) as u8,
                        tile_index: if (i % 4 + i / 4) % 2 == 0 { 0x0d } else { 0x1d },
                    },
                })
                .collect();
            for y in 0..2 {
                for x in 0..2 {
                    cells[y * 4 + x].source = VisualTileSource {
                        tileset_id: std::sync::Arc::from("lighthouse"),
                        metatile_id: block,
                        subtile_column: (x + 2) as u8,
                        subtile_row: (y + row_offset) as u8,
                        tile_index: drawing[y][x],
                    };
                }
            }
            let out = resolve(
                "FastShipCabins_NNW_NNE_NE",
                &cells.iter().collect::<Vec<_>>(),
                &g,
                None,
            );
            assert_eq!(
                out.iter()
                    .filter(|p| matches!(p.form, Form::Model(Kind::ShipBunk | Kind::ShipStool)))
                    .count(),
                0,
                "neither incomplete berth variant may bypass complete-source ownership"
            );
        }
    }
}
