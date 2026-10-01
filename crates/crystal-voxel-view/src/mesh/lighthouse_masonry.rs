//! Original joined ashlar for complete lighthouse masonry/window drawings.
//!
//! These four native blocks contain WALL/WINDOW quadrants only. No reachable
//! floor, stair, pit, counter, actor or doorway source belongs to the matcher.
//! Exact whole-block ownership is established before neighboring sides are
//! culled; a changed block remains on its existing renderer in its entirety.
use super::*;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Asset {
    Ashlar,
    SingleWindow,
    DoubleWindow,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Detail {
    asset: Asset,
    /// North, east, south, west. Only successfully owned neighbors close sides.
    open: [bool; 4],
    turns: u8,
    /// Keep the native single window on the west/north half after orientation.
    reflect: bool,
}
impl Detail {
    pub(super) fn label(self) -> &'static str {
        match self.asset {
            Asset::Ashlar => "dungeon:lighthouse-masonry",
            Asset::SingleWindow | Asset::DoubleWindow => "dungeon:lighthouse-window-masonry",
        }
    }
}

fn applies(map: &str) -> bool {
    matches!(
        map,
        "OlivineLighthouse1F"
            | "OlivineLighthouse2F"
            | "OlivineLighthouse3F"
            | "OlivineLighthouse4F"
            | "OlivineLighthouse5F"
            | "OlivineLighthouse6F"
    )
}

/// A small semantic motif, not an exported source catalog. Both brick courses,
/// the complete window half and the block's phase are checked for every cell.
fn expected(block: u16, x: usize, y: usize) -> Option<u16> {
    if x >= 4 || y >= 4 || !matches!(block, 0x29 | 0x3c | 0x3d | 0x3e) {
        return None;
    }
    Some(match y {
        0 => [0x5e, 0x5f][x % 2],
        1 => [0x4a, 0x4b][x % 2],
        2 if block == 0x3e => [0x5e, 0x5f][x % 2],
        2 if block == 0x29 || (block == 0x3d && x < 2) => 0x4e,
        2 => [0x5a, 0x5b][x % 2],
        3 if block == 0x3e => [0x4a, 0x4b][x % 2],
        _ => [0x4c, 0x4d][x % 2],
    })
}

fn block_at(cells: &[&VisualTile], g: &GridGeometry, column: usize, row: usize) -> Option<u16> {
    if column.checked_add(4)? > g.width || row.checked_add(4)? > g.height {
        return None;
    }
    let first = &cells[row * g.width + column].source;
    let block = first.metatile_id;
    expected(block, 0, 0)?;
    (0..4)
        .all(|y| {
            (0..4).all(|x| {
                let s = &cells[(row + y) * g.width + column + x].source;
                s.tileset_id.as_ref() == "lighthouse"
                    && s.metatile_id == block
                    && usize::from(s.subtile_column) == x
                    && usize::from(s.subtile_row) == y
                    && Some(s.tile_index) == expected(block, x, y)
            })
        })
        .then_some(block)
}

fn face_cells(g: &GridGeometry, x: usize, y: usize, side: usize) -> Vec<usize> {
    match side {
        0 if y > 0 => (x..x + 4).map(|xx| (y - 1) * g.width + xx).collect(),
        1 if x + 4 < g.width => (y..y + 4).map(|yy| yy * g.width + x + 4).collect(),
        2 if y + 4 < g.height => (x..x + 4).map(|xx| (y + 4) * g.width + xx).collect(),
        3 if x > 0 => (y..y + 4).map(|yy| yy * g.width + x - 1).collect(),
        _ => Vec::new(),
    }
}

fn window_turns(r: &Resolver<'_>, x: usize, y: usize) -> Option<u8> {
    // Prefer the room floor. The 1F southwest window faces a native void pocket:
    // exact blank source can orient that window too, but missing viewport data
    // cannot invent an orientation. Ties retain the native south-facing drawing.
    let score = |side| {
        face_cells(r.g, x, y, side)
            .iter()
            .map(|&i| {
                let s = &r.cells[i].source;
                if s.tileset_id.as_ref() != "lighthouse" {
                    return 0;
                }
                if matches!(s.tile_index, 0x2e | 0x2f | 0x0d | 0x1d)
                    && matches!(
                        shape_for_source_on_map(r.map, s),
                        CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
                    )
                {
                    8
                } else if s.metatile_id == 0x0c && s.tile_index == 0x01 {
                    1
                } else {
                    0
                }
            })
            .sum::<usize>()
    };
    let mut best = None;
    for side in [2, 0, 1, 3] {
        let value = score(side);
        if value > best.map_or(0, |(_, value)| value) {
            best = Some((side, value));
        }
    }
    best.map(|(side, _)| ((side + 2) % 4) as u8)
}

pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    if !applies(r.map) || r.g.width == 0 || r.cells.len() != r.g.width * r.g.height {
        return;
    }
    let Some(sample) = r.cells.iter().position(|tile| {
        let s = &tile.source;
        s.tileset_id.as_ref() == "lighthouse"
            && s.metatile_id == 0x27
            && s.subtile_column < 4
            && s.subtile_row < 4
            && s.tile_index == 0x2e
            && (s.subtile_column + s.subtile_row) % 2 == 0
            && matches!(
                shape_for_source_on_map(r.map, s),
                CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
            )
    }) else {
        return;
    };
    let mut candidates = Vec::new();
    let mut owned = vec![false; r.cells.len()];
    for row in 0..r.g.height {
        for column in 0..r.g.width {
            let Some(block) = block_at(r.cells, r.g, column, row) else {
                continue;
            };
            if (0..4).any(|y| (0..4).any(|x| r.claimed[(row + y) * r.g.width + column + x])) {
                continue;
            }
            let asset = match block {
                0x29 => Asset::DoubleWindow,
                0x3d => Asset::SingleWindow,
                _ => Asset::Ashlar,
            };
            let turns = if asset == Asset::Ashlar {
                0
            } else {
                let Some(turns) = window_turns(r, column, row) else {
                    continue;
                };
                turns
            };
            owned[row * r.g.width + column] = true;
            candidates.push((column, row, asset, turns));
        }
    }
    for (column, row, asset, turns) in candidates {
        let neighbors = [
            row.checked_sub(4).map(|y| y * r.g.width + column),
            (column + 4 < r.g.width).then_some(row * r.g.width + column + 4),
            (row + 4 < r.g.height).then_some((row + 4) * r.g.width + column),
            column.checked_sub(4).map(|x| row * r.g.width + x),
        ];
        let open = neighbors.map(|i| !i.is_some_and(|i| owned[i]));
        r.add(Placement {
            column,
            row,
            width: 4,
            height: 4,
            ground: sample,
            form: Form::Lighthouse(Detail {
                asset,
                open,
                turns,
                reflect: turns >= 2,
            }),
            rise_pixels: 32.0,
            depth_pixels: 32.0,
            front_rows: 4.0,
        });
    }
}

#[derive(Deserialize)]
struct Export {
    primitives: Vec<Primitive>,
}
#[derive(Deserialize)]
struct Primitive {
    side: u8,
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
struct Part {
    side: u8,
    surface: SurfaceMeshData,
}
struct Model {
    parts: Vec<Part>,
}
impl Model {
    fn parse<'a>(source: impl Into<crate::model_storage::Source<'a>>) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(source)?;
        let mut parts = Vec::new();
        let mut roles = [false; 6];
        for p in export.primitives {
            let count = p.positions.len() / 3;
            if p.side >= 6
                || count == 0
                || p.positions.len() % 3 != 0
                || p.normals.len() != p.positions.len()
                || p.indices.is_empty()
                || p.indices.len() % 3 != 0
                || p.indices.iter().any(|&i| i as usize >= count)
                || p.positions
                    .iter()
                    .any(|&v| !v.is_finite() || !(0.0..=1.0).contains(&v))
                || p.normals.iter().any(|v| !v.is_finite())
                || p.base_color
                    .iter()
                    .any(|&v| !v.is_finite() || !(0.0..=1.0).contains(&v))
                || p.base_color[3] != 1.0
            {
                return Err("invalid lighthouse masonry primitive".into());
            }
            let mut surface = SurfaceMeshData::default();
            for v in p.positions.chunks_exact(3) {
                surface.positions.push([v[0], v[1], v[2]]);
            }
            for v in p.normals.chunks_exact(3) {
                let n = Vec3::new(v[0], v[1], v[2]);
                if (n.length_squared() - 1.0).abs() > 0.001 {
                    return Err("invalid lighthouse normal".into());
                }
                surface.normals.push(n.to_array());
            }
            surface.uvs = vec![[0.; 2]; count];
            surface.colors = vec![p.base_color; count];
            surface.indices = p.indices;
            roles[p.side as usize] = true;
            parts.push(Part {
                side: p.side,
                surface,
            });
        }
        if !roles.iter().all(|v| *v) {
            return Err("incomplete lighthouse masonry shell".into());
        }
        Ok(Self { parts })
    }
}
fn model(asset: Asset) -> &'static Model {
    static MODELS: OnceLock<[Model; 3]> = OnceLock::new();
    let models = MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/lighthouse_masonry/ashlar.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/lighthouse_masonry/window_single.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/lighthouse_masonry/window_double.mesh.json"
            ),
        ]
        .map(|source| Model::parse(source).expect("validated lighthouse masonry model"))
    });
    &models[match asset {
        Asset::Ashlar => 0,
        Asset::SingleWindow => 1,
        Asset::DoubleWindow => 2,
    }]
}
fn rotate(mut v: Vec3, turns: u8) -> Vec3 {
    for _ in 0..turns {
        v = Vec3::new(-v.z, v.y, v.x);
    }
    v
}
fn world_side(side: u8, detail: Detail) -> u8 {
    let side = if detail.reflect && side < 4 {
        (4 - side) % 4
    } else {
        side
    };
    if side < 4 {
        (side + detail.turns) % 4
    } else {
        side
    }
}

pub(super) fn append(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement, detail: Detail) {
    let floor_uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            floor_uv,
        );
    }
    let (west, _, north, _) = g.bounds(p.column, p.row);
    let scale = Vec3::new(
        p.width as f32 * g.tile_width,
        p.rise_pixels * g.tile_height / SOURCE_TILE_HEIGHT,
        p.height as f32 * g.tile_height,
    );
    let origin = Vec3::new(west, 0., north);
    let cutaway_start = mesh.solid.positions.len();
    for part in &model(detail.asset).parts {
        let side = world_side(part.side, detail);
        if side < 4 && !detail.open[side as usize] {
            continue;
        }
        let source = &part.surface;
        let base = mesh.solid.positions.len() as u32;
        for (&point, &normal) in source.positions.iter().zip(&source.normals) {
            let mut point = Vec3::from_array(point) - Vec3::new(0.5, 0., 0.5);
            let mut normal = Vec3::from_array(normal);
            if detail.reflect {
                point.x = -point.x;
                normal.x = -normal.x;
            }
            point = rotate(point, detail.turns) + Vec3::new(0.5, 0., 0.5);
            normal = rotate(normal, detail.turns);
            mesh.solid
                .positions
                .push((origin + point * scale).to_array());
            mesh.solid
                .normals
                .push((normal / scale).normalize().to_array());
        }
        mesh.solid.uvs.extend_from_slice(&source.uvs);
        mesh.solid.colors.extend_from_slice(&source.colors);
        for tri in source.indices.chunks_exact(3) {
            let [a, b, c] = [tri[0] + base, tri[1] + base, tri[2] + base];
            mesh.solid
                .indices
                .extend(if detail.reflect { [a, c, b] } else { [a, b, c] });
        }
    }
    // Only successfully owned masonry participates. The floor sample,
    // stairs, pits, furniture, and unknown-source fallback stay opaque.
    if mesh.solid.positions.len() > cutaway_start {
        mesh.solid
            .cutaway_ranges
            .push(cutaway_start..mesh.solid.positions.len());
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
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    fn floor(w: usize, h: usize) -> Vec<VisualTile> {
        (0..w * h)
            .map(|i| VisualTile {
                column: (i % w) as u32,
                row: (i / w) as u32,
                source: VisualTileSource {
                    tileset_id: Arc::from("lighthouse"),
                    metatile_id: 0x27,
                    subtile_column: (i % w % 4) as u8,
                    subtile_row: (i / w % 4) as u8,
                    tile_index: if (i % w + i / w) % 2 == 0 { 0x2e } else { 0x2f },
                },
                texture: Default::default(),
                priority: false,
            })
            .collect()
    }
    fn block(t: &mut [VisualTile], w: usize, x: usize, y: usize, id: u16) {
        for dy in 0..4 {
            for dx in 0..4 {
                t[(y + dy) * w + x + dx].source = VisualTileSource {
                    tileset_id: Arc::from("lighthouse"),
                    metatile_id: id,
                    subtile_column: dx as u8,
                    subtile_row: dy as u8,
                    tile_index: expected(id, dx, dy).unwrap(),
                };
            }
        }
    }
    fn resolve(t: &[VisualTile], g: &GridGeometry, map: &str) -> Vec<Placement> {
        let cells = t.iter().collect::<Vec<_>>();
        let mut r = Resolver {
            map,
            cells: &cells,
            g,
            claimed: vec![false; t.len()],
            out: Vec::new(),
        };
        resolve_into(&mut r);
        r.out
    }
    fn detail(p: &Placement) -> Detail {
        let Form::Lighthouse(d) = p.form else {
            panic!("wrong form")
        };
        d
    }
    #[test]
    fn all_six_lighthouse_floors_accept_only_four_complete_wall_drawings() {
        let g = grid(12, 12);
        for id in [0x29, 0x3c, 0x3d, 0x3e] {
            let mut t = floor(12, 12);
            block(&mut t, 12, 4, 4, id);
            for level in 1..=6 {
                let p = resolve(&t, &g, &format!("OlivineLighthouse{level}F"));
                assert_eq!(p.len(), 1);
                assert_eq!(p[0].indices(g.width).count(), 16);
                assert!(p[0].indices(g.width).all(|i| t[i].source.metatile_id == id));
            }
            for map in [
                "FastShip1F",
                "FastShipCabins_NNW_NNE_NE",
                "OlivineLighthouse7F",
                "OlivineLighthouse1FBeta",
                "OlivineCity",
            ] {
                assert!(resolve(&t, &g, map).is_empty(), "{map}");
            }
        }
    }
    #[test]
    fn every_changed_cell_rejects_the_whole_wall_or_window() {
        let g = grid(12, 12);
        for id in [0x29, 0x3c, 0x3d, 0x3e] {
            for dy in 0..4 {
                for dx in 0..4 {
                    let mut t = floor(12, 12);
                    block(&mut t, 12, 4, 4, id);
                    t[(4 + dy) * 12 + 4 + dx].source.tile_index = 0xff;
                    assert!(
                        resolve(&t, &g, "OlivineLighthouse3F").is_empty(),
                        "{id:x} {dx},{dy}"
                    );
                }
            }
        }
    }
    #[test]
    fn cropped_shifted_mixed_atlas_and_missing_native_ground_fall_back() {
        let g = grid(12, 12);
        let mut original = floor(12, 12);
        block(&mut original, 12, 4, 4, 0x3d);
        for kind in 0..4 {
            let mut t = original.clone();
            let s = &mut t[4 * 12 + 4].source;
            match kind {
                0 => s.subtile_column = 1,
                1 => s.subtile_row = 1,
                2 => s.tileset_id = Arc::from("other"),
                _ => s.metatile_id = 0x3c,
            }
            assert!(resolve(&t, &g, "OlivineLighthouse3F").is_empty());
        }
        let mut t = original.clone();
        for t in &mut t {
            if t.source.metatile_id == 0x27 {
                t.source.tile_index = 0xff;
            }
        }
        assert!(resolve(&t, &g, "OlivineLighthouse3F").is_empty());
        let cells = original.iter().collect::<Vec<_>>();
        assert_eq!(block_at(&cells, &g, 4, 4), Some(0x3d));
        assert!(block_at(&cells, &grid(7, 12), 4, 4).is_none());
        assert!(block_at(&cells, &grid(12, 7), 4, 4).is_none());
        assert!(block_at(&cells, &g, 5, 4).is_none());
    }
    #[test]
    fn every_end_corner_straight_tee_cross_and_island_has_exact_shared_sides() {
        let g = grid(20, 20);
        for mask in 0..16 {
            let mut t = floor(20, 20);
            block(&mut t, 20, 8, 8, 0x3e);
            for (side, (x, y)) in [(8, 4), (12, 8), (8, 12), (4, 8)].into_iter().enumerate() {
                if mask & (1 << side) == 0 {
                    block(&mut t, 20, x, y, 0x3c);
                }
            }
            let p = resolve(&t, &g, "OlivineLighthouse4F");
            let center = p.iter().find(|p| p.column == 8 && p.row == 8).unwrap();
            assert_eq!(
                detail(center).open,
                std::array::from_fn(|side| mask & (1 << side) != 0)
            );
        }
    }
    #[test]
    fn a_claimed_cell_rejects_its_entire_wall_and_cannot_hide_a_neighbors_end() {
        let g = grid(16, 12);
        let mut t = floor(16, 12);
        block(&mut t, 16, 4, 4, 0x3e);
        block(&mut t, 16, 8, 4, 0x3e);
        let cells = t.iter().collect::<Vec<_>>();
        let mut r = Resolver {
            map: "OlivineLighthouse4F",
            cells: &cells,
            g: &g,
            claimed: vec![false; t.len()],
            out: Vec::new(),
        };
        r.claimed[5 * 16 + 5] = true;
        resolve_into(&mut r);
        assert_eq!(r.out.len(), 1);
        assert_eq!(r.out[0].column, 8);
        assert!(detail(&r.out[0]).open[3]);
    }
    #[test]
    fn windows_face_each_available_room_direction_and_keep_the_native_half() {
        let g = grid(12, 12);
        for side in 0..4 {
            let mut t = floor(12, 12);
            block(&mut t, 12, 4, 4, 0x3d);
            for tile in &mut t {
                if tile.source.metatile_id == 0x27 {
                    tile.source.metatile_id = 0;
                    tile.source.tile_index = 0;
                }
            }
            for i in face_cells(&g, 4, 4, side) {
                let s = &mut t[i].source;
                s.metatile_id = 0x27;
                s.tile_index = if (s.subtile_column + s.subtile_row) % 2 == 0 {
                    0x2e
                } else {
                    0x2f
                };
            }
            let p = resolve(&t, &g, "OlivineLighthouse3F");
            assert_eq!(p.len(), 1);
            let d = detail(&p[0]);
            assert_eq!(world_side(2, d), side as u8);
            assert_eq!(d.reflect, d.turns >= 2);
        }
    }
    #[test]
    fn native_void_can_orient_a_window_but_unknown_or_cropped_context_cannot() {
        let g = grid(8, 8);
        let mut t = floor(8, 8);
        block(&mut t, 8, 0, 4, 0x3d);
        // Floor sample exists away from the north/south/west/east window flank.
        for i in face_cells(&g, 0, 4, 0)
            .into_iter()
            .chain(face_cells(&g, 0, 4, 1))
        {
            t[i].source.metatile_id = 0;
            t[i].source.tile_index = 0;
        }
        assert!(resolve(&t, &g, "OlivineLighthouse1F").is_empty());
        for i in face_cells(&g, 0, 4, 0) {
            t[i].source.metatile_id = 0x0c;
            t[i].source.tile_index = 1;
        }
        let p = resolve(&t, &g, "OlivineLighthouse1F");
        assert_eq!(p.len(), 1);
        assert_eq!(world_side(2, detail(&p[0])), 0);
    }
    #[test]
    fn modeled_volume_stays_inside_wall_plot_and_preserves_floor_and_footing() {
        let g = grid(12, 12);
        for id in [0x29, 0x3c, 0x3d, 0x3e] {
            let mut t = floor(12, 12);
            block(&mut t, 12, 4, 4, id);
            let p = resolve(&t, &g, "OlivineLighthouse6F");
            for turns in 0..4 {
                let mut d = detail(&p[0]);
                d.turns = turns;
                d.reflect = turns >= 2;
                let mut mesh = TerrainMeshData::default();
                mesh.footing_heights = vec![1.25; t.len()];
                append(&mut mesh, &g, &p[0], d);
                assert_eq!(mesh.footing_heights, vec![1.25; t.len()]);
                assert_eq!(mesh.textured.indices.len() / 6, 16);
                assert!(mesh.textured.cutaway_ranges.is_empty());
                assert_eq!(
                    mesh.solid.cutaway_ranges,
                    vec![0..mesh.solid.positions.len()]
                );
                assert!(mesh
                    .solid
                    .positions
                    .iter()
                    .all(|p| (32.0..=64.0).contains(&p[0])
                        && (0.0..=32.0).contains(&p[1])
                        && (32.0..=64.0).contains(&p[2])));
                assert!(mesh.solid.indices.len() / 3 <= 1100);
                for tri in mesh.solid.indices.chunks_exact(3) {
                    let a = Vec3::from_array(mesh.solid.positions[tri[0] as usize]);
                    let b = Vec3::from_array(mesh.solid.positions[tri[1] as usize]);
                    let c = Vec3::from_array(mesh.solid.positions[tri[2] as usize]);
                    let n = (b - a).cross(c - a);
                    assert!(n.length_squared() > 1e-8);
                    assert!(tri
                        .iter()
                        .all(|&i| n.dot(Vec3::from_array(mesh.solid.normals[i as usize])) > 0.0));
                }
            }
        }
    }
}
