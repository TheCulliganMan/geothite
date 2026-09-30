//! Complete source drawings mapped to original dungeon volumes.
//!
//! All matches are render-only and require a same-tileset ground sample. The
//! resolver sees immutable sources before live profiles mask them. Missing,
//! clipped, recolored-with-new-identities and unsupported drawings fall back.
use super::*;
use crate::dungeon_models::{Kind, model};
use crate::live_profiles::Document;

#[derive(Clone, Copy, Debug)]
enum Form {
    Model(Kind),
    RocketWall { open: [bool; 4] },
    LighthouseWall { open: [bool; 4] },
    CaveCorner(crate::cave::DiagonalCorner),
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
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..self.height).flat_map(move |y| {
            (0..self.width).map(move |x| (self.row + y) * width + self.column + x)
        })
    }
    pub(super) fn kind_label(&self) -> &'static str {
        match self.form {
            Form::Model(k) => k.label(),
            Form::RocketWall { .. } => "dungeon:rocket-wall-network",
            Form::LighthouseWall { .. } => "dungeon:lighthouse-masonry",
            Form::CaveCorner(_) => "dungeon:cave-diagonal-corner",
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
                form: Form::Model(kind),
                rise_pixels: rise,
                depth_pixels: depth,
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
        p.front_rows = 2.0;
    }
    r.groups(
        Kind::TowerGuardian,
        crate::tower::TOWER_FLOOR_TILE,
        16.0,
        10.0,
        tower_statue_placements(cells, g),
    );
    if map == "PewterGym" {
        r.groups(
            Kind::CaveBoulder,
            crate::tower::TOWER_FLOOR_TILE,
            10.0,
            12.0,
            tower_boulder_placements(cells, g),
        );
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
    if matches!(map, "VioletGym" | "MahoganyGym" | "BlackthornGym1F") {
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
        r.groups(
            Kind::ShipStool,
            crate::ship::CABIN_FLOOR_TILE,
            10.0,
            12.0,
            ship_stool_placements(map, cells, g),
        );
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
        let bunks =
            grouped_flat_card_placements(cells, g, crate::ship::CABIN_FLOOR_TILE, false, |s| {
                crate::ship::bunk_shape(map, s)
                    .map(|_| (s.subtile_column % 2, s.subtile_row % 2, 2, 2))
            });
        r.groups(
            Kind::ShipBunk,
            crate::ship::CABIN_FLOOR_TILE,
            7.0,
            16.0,
            bunks,
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
        16.0,
        3.0,
        timber_walls,
    );
    // Complete lighthouse side courses require four genuine floor neighbors
    // on a flank. Preserve mixed corners and doorway/window blocks unchanged.
    let lighthouse_walls = grouped_flat_card_placements(cells, g, 0x2e, false, |source| {
        if source.tileset_id.as_ref() != "lighthouse" || source.metatile_id != 0x3e {
            return None;
        }
        let art = [[0x5e, 0x5f, 0x5e, 0x5f], [0x4a, 0x4b, 0x4a, 0x4b]];
        (source.tile_index == art[source.subtile_row as usize % 2][source.subtile_column as usize])
            .then_some((source.subtile_column, source.subtile_row, 4, 4))
    });
    for p in lighthouse_walls {
        if !phase_consistent(cells, g, p) {
            continue;
        }
        let Some(ground) = ground(map, cells, "lighthouse", 0x2e) else {
            continue;
        };
        let floor_at = |x: usize, y: usize| {
            let source = &cells[y * g.width + x].source;
            source.tileset_id.as_ref() == "lighthouse" && source.metatile_id == 0x27
        };
        if (p.row > 0 && (0..4).any(|x| floor_at(p.column + x, p.row - 1)))
            || (p.row + 4 < g.height && (0..4).any(|x| floor_at(p.column + x, p.row + 4)))
        {
            continue;
        }
        let west = p.column > 0 && (0..4).all(|y| floor_at(p.column - 1, p.row + y));
        let east = p.column + 4 < g.width && (0..4).all(|y| floor_at(p.column + 4, p.row + y));
        if !west && !east {
            continue;
        }
        r.add(Placement {
            column: p.column,
            row: p.row,
            width: 4,
            height: 4,
            ground,
            form: Form::LighthouseWall {
                open: [false, east, false, west],
            },
            rise_pixels: 32.0,
            depth_pixels: 32.0,
            front_rows: 4.0,
        });
    }
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
                form: Form::Model(kind),
                rise_pixels: rise,
                depth_pixels: depth,
                front_rows: o.tiles.len() as f32,
            });
        }
    }
    // The alarm is a complete four-row source drawing; its wall-contact rows
    // continue to close neighboring wall ends without filling its floor rows.
    let alarm = grouped_flat_card_placements(cells, g, 0x10, false, |s| {
        if map != "TeamRocketBaseB1F"
            || s.tileset_id.as_ref() != "underground"
            || s.metatile_id != 0x04
            || s.subtile_column >= 2
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
    if let Some(ground) = ground(map, cells, "underground", crate::rocket_base::FLOOR_TILE) {
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
                form: Form::RocketWall { open },
                rise_pixels: 16.0,
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
                rise_pixels: 16.0,
                depth_pixels: 16.0,
                front_rows: 2.0,
            });
        }
    }
    r.out
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
) {
    let floor_uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.0,
            floor_uv,
        );
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.front_rows * g.tile_height;
    let rise = p.rise_pixels * g.tile_height / SOURCE_TILE_HEIGHT;
    match p.form {
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
        Form::RocketWall { open } => {
            append_connected_wall(&mut mesh.solid, [w, e, n, s], rise, open)
        }
        Form::LighthouseWall { open } => append_masonry(&mut mesh.solid, [w, e, n, s], rise, open),
        Form::CaveCorner(corner) => append_corner(
            &mut mesh.solid,
            [w, e, n, s],
            rise,
            g.tile_width * 0.5,
            corner,
            cells[p.row * g.width + p.column].source.tileset_id.as_ref() == "dark_cave",
        ),
    }
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
    bevel: f32,
    corner: crate::cave::DiagonalCorner,
    dark: bool,
) {
    let [w, e, n, s] = b;
    let color = if dark {
        [0.35, 0.40, 0.42, 1.0]
    } else {
        [0.51, 0.51, 0.42, 1.0]
    };
    let (cap, face, south, side, side_normal) = match corner {
        crate::cave::DiagonalCorner::SouthEast => (
            [[e, h, n], [e, h, s], [w, h, s]],
            [
                [e - bevel, 0., n - bevel],
                [e, h, n],
                [w, h, s],
                [w - bevel, 0., s - bevel],
            ],
            [[w - bevel, 0., s - bevel], [w, h, s], [e, h, s], [e, 0., s]],
            [[e, 0., s], [e, h, s], [e, h, n], [e - bevel, 0., n - bevel]],
            [1., 0., 0.],
        ),
        crate::cave::DiagonalCorner::SouthWest => (
            [[w, h, n], [e, h, s], [w, h, s]],
            [
                [w + bevel, 0., n - bevel],
                [e + bevel, 0., s - bevel],
                [e, h, s],
                [w, h, n],
            ],
            [[w, 0., s], [w, h, s], [e, h, s], [e + bevel, 0., s - bevel]],
            [[w + bevel, 0., n - bevel], [w, h, n], [w, h, s], [w, 0., s]],
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
    let base = out.positions.len() as u32;
    out.positions.extend(cap);
    out.normals.extend([[0., 1., 0.]; 3]);
    out.uvs.extend([[0.; 2]; 3]);
    out.colors.extend([color; 3]);
    out.indices.extend([base, base + 1, base + 2]);
    let normal = (Vec3::from_array(face[1]) - Vec3::from_array(face[0]))
        .cross(Vec3::from_array(face[2]) - Vec3::from_array(face[0]))
        .normalize()
        .to_array();
    faceted_face(out, face, normal, color, h * 0.025);
    faceted_face(out, south, [0., 0., 1.], color, h * 0.014);
    faceted_face(out, side, side_normal, color, h * 0.014);
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
        "cave" => Some([0.48, 0.48, 0.39, 1.0]),
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
/// Replace verified rock bank cells with a continuous, sealed faceted surface.
/// This consumes the FINAL tier-resolved shapes, never raw source heights. No
/// height, actor support, collision, warp or stair state is modified. Unknown
/// viewport continuation emits no artificial perimeter skirt.
pub(super) fn append_terrain(
    mesh: &mut TerrainMeshData,
    _map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    claimed: &mut [bool],
) {
    for (index, tile) in cells.iter().enumerate() {
        if claimed[index] {
            continue;
        }
        let Some(color) = terrain_material(&tile.source, shapes[index]) else {
            continue;
        };
        let h = shapes[index].surface_height(g.tile_height);
        if h <= 0.0 {
            continue;
        }
        let (x, y) = (index % g.width, index / g.width);
        let (w, e, n, s) = g.bounds(x, y);
        faceted_face(
            &mut mesh.solid,
            [[w, h, n], [w, h, s], [e, h, s], [e, h, n]],
            [0., 1., 0.],
            color,
            0.0,
        );
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
            let bottom =
                neighbor_edge_heights(shapes[ni], side, g.tile_height / SOURCE_TILE_HEIGHT);
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
            faceted_face(
                &mut mesh.solid,
                points,
                normal,
                face_color,
                (h - low[0].min(low[1])).min(g.tile_width) * 0.05,
            );
        }
        claimed[index] = true;
        mark_authored_rect(mesh, g, [x, y, 1, 1], "dungeon:connected-rock-terrain");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn grid(w: usize, h: usize) -> GridGeometry {
        GridGeometry {
            width: w,
            height: h,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        }
    }
    fn tiles(w: usize, h: usize, tileset: &str) -> Vec<VisualTile> {
        (0..w * h)
            .map(|i| VisualTile {
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
        append(&mut mesh, &g, &p[0], &cells);
        assert!(!mesh.solid.indices.is_empty());
        assert_eq!(mesh.footing_heights, original);
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
        }
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
        append_terrain(&mut m, "UnionCave1F", &c, &shapes, &g, &mut claimed);
        assert_eq!(claimed, vec![true, true, false]);
        assert_eq!(m.footing_heights, before);
        assert_eq!(
            m.solid.indices.len() / 3,
            16,
            "two caps and two exposed east faces only"
        );
        assert!(
            m.solid
                .positions
                .iter()
                .filter(|p| p[0] == 8.0)
                .all(|p| p[1] >= 16.0)
        );
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
        );
        assert_eq!(claimed, vec![false; 3]);
        assert!(m.solid.positions.is_empty());
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
    fn lighthouse_requires_complete_masonry_and_a_known_four_cell_floor_flank() {
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
        t[9].source.metatile_id = 0x00;
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
    fn tower_walls_consume_only_two_face_rows_and_leave_floor_free() {
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
    }
    #[test]
    fn cave_corner_mirrors_keep_upward_caps_and_nonzero_outward_normals() {
        for corner in [
            crate::cave::DiagonalCorner::SouthEast,
            crate::cave::DiagonalCorner::SouthWest,
        ] {
            let mut m = SurfaceMeshData::default();
            append_corner(&mut m, [0., 16., 0., 16.], 16., 4., corner, false);
            for ids in m.indices.chunks_exact(3) {
                let p = ids
                    .iter()
                    .map(|&i| Vec3::from_array(m.positions[i as usize]))
                    .collect::<Vec<_>>();
                let n = Vec3::from_array(m.normals[ids[0] as usize]);
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.0);
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
}
