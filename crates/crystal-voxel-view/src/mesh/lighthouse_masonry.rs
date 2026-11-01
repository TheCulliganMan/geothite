//! Original joined ashlar for complete lighthouse masonry/window drawings.
//!
//! These four native blocks contain WALL/WINDOW quadrants only. No reachable
//! floor, stair, pit, counter, actor or doorway source belongs to the matcher.
//! Exact whole-block ownership controls the visible rubble skirt. Masonry shells
//! stay closed because neighboring blocks can fade independently; a changed
//! block remains on its existing renderer in its entirety.
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
    moss: bool,
    /// North, east, south, west rubble skirts. Owned neighbors omit shared
    /// rubble; independently fading masonry bodies always retain closed ends.
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

pub(super) fn applies(map: &str) -> bool {
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
            candidates.push((column, row, asset, turns, block != 0x3e));
        }
    }
    for (column, row, asset, turns, moss) in candidates {
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
                moss,
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
    let cutaway_start = mesh.textured.positions.len();
    let sample=|pixel:[usize;2]| {
        let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
        [uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
         uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]
    };
    for part in &model(detail.asset).parts {
        let side = world_side(part.side, detail);
        // Each block owns an independent reveal range. Keep buried masonry
        // ends: revealing a neighbor must expose a closed wall, not its hollow
        // interior. Only the low rubble skirt culls shared sides below.
        let source = &part.surface;
        let base = mesh.textured.positions.len() as u32;
        for (&point, &normal) in source.positions.iter().zip(&source.normals) {
            let mut point = Vec3::from_array(point) - Vec3::new(0.5, 0., 0.5);
            let mut normal = Vec3::from_array(normal);
            // Recess the masonry backing behind the native low rubble skirt.
            // The rocks themselves remain inside the original wall footprint.
            if detail.moss {
                let height = if detail.asset == Asset::Ashlar {0.5} else {0.25};
                let inset = 0.90 + 0.10 * (point.y / height).clamp(0.,1.);
                point.x *= inset;
                point.z *= inset;
            }
            if detail.reflect {
                point.x = -point.x;
                normal.x = -normal.x;
            }
            point = rotate(point, detail.turns) + Vec3::new(0.5, 0., 0.5);
            normal = rotate(normal, detail.turns);
            mesh.textured
                .positions
                .push((origin + point * scale).to_array());
            mesh.textured
                .normals
                .push((normal / scale).normalize().to_array());
        }
        if detail.moss {
            // Exported faces have separate vertices per triangle. Recompute
            // normals after recessing the backing so narrow window reveals
            // retain correct lighting and winding.
            for tri in source.indices.chunks_exact(3) {
                let mut ids = [tri[0] as usize + base as usize, tri[1] as usize + base as usize,
                    tri[2] as usize + base as usize];
                if detail.reflect { ids.swap(1,2); }
                let [a,b,c] = ids.map(|i|Vec3::from_array(mesh.textured.positions[i]));
                let normal = (b-a).cross(c-a).normalize().to_array();
                for i in ids { mesh.textured.normals[i] = normal; }
            }
        }
        for (i,color) in source.colors.iter().enumerate() {
            // The authored model's cream paint is not lighthouse stone. Read
            // charcoal from this complete block's upper ashlar, and glass/frame
            // from its own window half. Retain subtle modeled stone variation.
            let glass=color[2]>color[1]&&color[1]>color[0];
            let frame=color[0]>0.4&&color[2]<0.3;
            let normal=Vec3::from_array(mesh.textured.normals[base as usize+i]);
            let bevel=Vec3::from_array(source.normals[i]).abs().max_element()<0.98;
            let pixel=if glass {[3,19]}else if frame {[1,17]}
                else if side<4&&bevel {[3,11]}else{[3,3]};
            let tone=if glass||frame {1.}else{(color[0]/0.63).clamp(0.8,1.12)};
            mesh.textured.uvs.push(sample(pixel));
            mesh.textured.colors.push(crate::interior_models::authored_face_color(
                [tone,tone,tone,1.],normal));
        }
        for tri in source.indices.chunks_exact(3) {
            let [a, b, c] = [tri[0] + base, tri[1] + base, tri[2] + base];
            mesh.textured
                .indices
                .extend(if detail.reflect { [a, c, b] } else { [a, b, c] });
        }
    }
    if detail.moss {
        append_moss_rocks(&mut mesh.textured, g, p, detail);
    }
    // Only successfully owned masonry participates. The floor sample,
    // stairs, pits, furniture, and unknown-source fallback stay opaque.
    if mesh.textured.positions.len() > cutaway_start {
        mesh.textured
            .cutaway_ranges
            .push(cutaway_start..mesh.textured.positions.len());
    }
}

/// The green native rubble is a low, uneven stone skirt below the windows.
/// Read its complete 4c/4d drawings from this block's atlas, including night
/// palettes; solid brick block 3e has no rubble and must remain bare.
fn append_moss_rocks(out: &mut SurfaceMeshData, g: &GridGeometry, p: &Placement, detail: Detail) {
    let (west, _, north, _) = g.bounds(p.column, p.row);
    let tall = detail.asset == Asset::Ashlar;
    let scale = Vec3::new(4. * g.tile_width, g.tile_height * if tall {2.} else {1.}, 4. * g.tile_height);
    for side in 0..4 {
        if !detail.open[side] { continue; }
        for rock in 0..4 {
            let left = rock as f32 / 4.;
            let right = (rock + 1) as f32 / 4.;
            let h0 = [0.75, 0.95, 0.8, 0.9][rock];
            let h1 = [0.95, 0.8, 0.9, 0.75][rock];
            let mut uv = g.uv(p.column + rock, p.row + 3);
            if tall { uv.2 = g.uv(p.column + rock, p.row + 2).2; }
            let front = [[left, 0., 1.0], [right, 0., 1.0],
                [right, h1, 0.995], [left, h0, 0.995]];
            let crown = [[left, h0, 0.995], [right, h1, 0.995],
                [right, h1 * 0.8, 0.90], [left, h0 * 0.8, 0.90]];
            for (points, tex) in [(front, [[uv.0,uv.3],[uv.1,uv.3],[uv.1,uv.2],[uv.0,uv.2]]),
                (crown, [[uv.0,uv.2],[uv.1,uv.2],[uv.1,uv.3],[uv.0,uv.3]])] {
                let points = points.map(|v| {
                    let local = rotate(Vec3::from_array(v) - Vec3::new(0.5,0.,0.5),
                        ((side + 2) % 4) as u8) + Vec3::new(0.5,0.,0.5);
                    (Vec3::new(west,0.,north) + local * scale).to_array()
                });
                let [a,b,c,_] = points.map(Vec3::from_array);
                let normal = (b-a).cross(c-a).normalize();
                append_quad(out, points, normal.to_array(), tex,
                    crate::interior_models::authored_face_color([1.;4], normal));
            }
        }
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
                animation_frames: None,
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
    fn external_lighthouse_family_uses_native_charcoal_and_preserves_wall_ownership() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        if !path.is_file(){return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack=loaded.pack();let art=&pack.runtime_files()["data/tilesets/lighthouse_metatiles.bin"];
        let pixels=image::load_from_memory(&pack.runtime_files()["gfx/tilesets/lighthouse.png"]).unwrap().to_luma8();
        assert_eq!(pixels.get_pixel((0x5e%16)*8+3,(0x5e/16)*8+3)[0],0,"wall body is native charcoal");
        for level in 1..=6 {
            let name=format!("OlivineLighthouse{level}F");let map=&pack.data().maps[&name];
            let (w,h)=(map.attributes.width as usize*4,map.attributes.height as usize*4);
            let tiles=(0..w*h).map(|i| {
                let (x,y)=(i%w,i/w);let block=map.blocks[y/4*(w/4)+x/4];
                VisualTile{column:x as u32,row:y as u32,texture:Default::default(),priority:false,animation_frames:None,
                    source:VisualTileSource{tileset_id:"lighthouse".into(),metatile_id:block,
                        subtile_column:(x%4)as u8,subtile_row:(y%4)as u8,
                        tile_index:art[block as usize*16+y%4*4+x%4]as u16}}
            }).collect::<Vec<_>>();
            let g=grid(w,h);let refs=tiles.iter().collect::<Vec<_>>();let placements=resolve(&tiles,&g,&name);
            assert!(!placements.is_empty(),"{name} must actually resolve native masonry");
            for p in &placements {
                assert_eq!(block_at(&refs,&g,p.column,p.row),Some(tiles[p.row*w+p.column].source.metatile_id));
                let mut mesh=TerrainMeshData{footing_heights:vec![6.;w*h],..Default::default()};
                append(&mut mesh,&g,p,detail(p));
                assert!(mesh.solid.positions.is_empty());
                assert_eq!(mesh.footing_heights,vec![6.;w*h]);
                assert_eq!(mesh.textured.cutaway_ranges[0].start,64,"opaque floor excluded from reveal");
                assert!(mesh.textured.positions.iter().any(|v|v[1]==32.));
            }
        }
    }
    #[test]
    fn independently_revealed_neighbors_leave_closed_masonry_ends() {
        // A single surviving block after all four neighbors fade. Test actual
        // emitted, outward-facing triangles, including rotated window shells.
        let g = grid(12, 12);
        for id in [0x29, 0x3c, 0x3d, 0x3e] {
            let mut tiles = floor(12, 12);
            block(&mut tiles, 12, 4, 4, id);
            let p = resolve(&tiles, &g, "OlivineLighthouse5F").remove(0);
            for turns in 0..4 {
                let mut d = detail(&p);
                d.open = [false; 4];
                d.turns = turns;
                d.reflect = turns >= 2;
                let mut mesh = TerrainMeshData::default();
                append(&mut mesh, &g, &p, d);
                let surface = &mesh.textured;
                let reveal = surface.cutaway_ranges[0].clone();
                assert_eq!(reveal.start, 64, "floor stays outside fading body");
                for outward in [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z] {
                    let across = Vec3::new(-outward.z, 0., outward.x);
                    for offset in [-8., 0., 8.] {
                        let target = Vec3::new(48., 24., 48.) + across * offset;
                        let start = target + outward * 32.;
                        let direction = target - start;
                        let hit = surface.indices.chunks_exact(3).any(|tri| {
                            if !tri.iter().all(|i| reveal.contains(&(*i as usize))) {
                                return false;
                            }
                            let [a,b,c] = [tri[0],tri[1],tri[2]]
                                .map(|i| Vec3::from_array(surface.positions[i as usize]));
                            let e1 = b-a;
                            let e2 = c-a;
                            let h = direction.cross(e2);
                            let determinant = e1.dot(h);
                            if determinant <= 1e-6 { return false; }
                            let relative = start-a;
                            let u = relative.dot(h)/determinant;
                            let q = relative.cross(e1);
                            let v = direction.dot(q)/determinant;
                            let t = e2.dot(q)/determinant;
                            u >= -1e-5 && v >= -1e-5 && u+v <= 1.00001
                                && t > 0. && t < 1.
                        });
                        assert!(hit, "closed outward end: block={id:x} turns={turns} side={outward:?} offset={offset}");
                    }
                }
            }
        }
    }
    #[test]
    fn moss_rocks_follow_native_band_and_cull_joined_faces() {
        let g = grid(16,12);
        for id in [0x29,0x3c,0x3d,0x3e] {
            let mut t = floor(16,12);
            block(&mut t,16,4,4,id);
            block(&mut t,16,8,4,id);
            let ps = resolve(&t,&g,"OlivineLighthouse2F");
            assert_eq!(ps.len(),2);
            for p in &ps {
                let d = detail(p);
                assert_eq!(d.moss,id != 0x3e);
                assert_eq!(d.open.iter().filter(|v|**v).count(),3);
                let mut out = SurfaceMeshData::default();
                if d.moss { append_moss_rocks(&mut out,&g,p,d); }
                assert_eq!(out.positions.len(),if id==0x3e {0}else{3*4*8});
                assert!(out.positions.iter().all(|v| (0.0..if id==0x3c {16.0}else{8.0}).contains(&v[1])));
                assert!(out.normals.iter().all(|n|Vec3::from_array(*n).is_normalized()));
            }
        }
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
                assert!(mesh.solid.positions.is_empty());
                assert_eq!(mesh.textured.cutaway_ranges,vec![64..mesh.textured.positions.len()]);
                assert!(mesh.solid.cutaway_ranges.is_empty());
                let uv=g.uv(p[0].column,p[0].row);
                let body=[uv.0+(uv.1-uv.0)*3.5/8.,uv.2+(uv.3-uv.2)*3.5/8.];
                assert!(mesh.textured.uvs[64..].contains(&body),"stone uses its own charcoal pixel");
                if matches!(id,0x3c|0x3e) {
                    let uv=g.uv(p[0].column,p[0].row+1);
                    let edge=[uv.0+(uv.1-uv.0)*3.5/8.,uv.2+(uv.3-uv.2)*3.5/8.];
                    assert!(mesh.textured.uvs[64..].contains(&edge),"beveled joints keep native gray85");
                    let stone_end = mesh.textured.positions.len() - if d.moss {d.open.iter().filter(|v|**v).count()*32} else {0};
                    for i in 64..stone_end {
                        let n=Vec3::from_array(mesh.textured.normals[i]);
                        if n.abs().max_element()>0.999 {
                            assert_eq!(mesh.textured.uvs[i],body,"broad faces cannot become alternating gray blocks");
                        }
                    }
                }
                assert!(mesh
                    .textured
                    .positions
                    .iter()
                    .all(|p| (32.0..=64.0).contains(&p[0])
                        && (0.0..=32.0).contains(&p[1])
                        && (32.0..=64.0).contains(&p[2])));
                assert!((mesh.textured.indices.len()-96) / 3 <= 1100);
                for tri in mesh.textured.indices.chunks_exact(3) {
                    let a = Vec3::from_array(mesh.textured.positions[tri[0] as usize]);
                    let b = Vec3::from_array(mesh.textured.positions[tri[1] as usize]);
                    let c = Vec3::from_array(mesh.textured.positions[tri[2] as usize]);
                    let n = (b - a).cross(c - a);
                    assert!(n.length_squared() > 1e-8);
                    assert!(tri
                        .iter()
                        .all(|&i| n.dot(Vec3::from_array(mesh.textured.normals[i as usize])) > 0.0));
                }
            }
        }
    }
}
