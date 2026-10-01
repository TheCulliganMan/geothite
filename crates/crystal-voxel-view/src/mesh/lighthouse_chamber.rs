//! The 6F keeper room has a tea table, low stool and cot, not beacon machinery.
//! Every source picture is proved in full before any part is claimed. These
//! render-only assemblies preserve the native walkable stool and all footing.
use super::*;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Asset {
    TeaTable,
    KeeperCot,
    RedStool,
}
impl Asset {
    const ALL: [Self; 3] = [Self::TeaTable, Self::KeeperCot, Self::RedStool];
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::TeaTable => "dungeon:lighthouse-tea-table",
            Self::KeeperCot => "dungeon:lighthouse-keeper-cot",
            Self::RedStool => "dungeon:lighthouse-red-stool",
        }
    }
    fn size(self) -> (usize, usize, f32) {
        match self {
            Self::TeaTable => (4, 6, 17.),
            Self::KeeperCot => (2, 4, 12.),
            Self::RedStool => (2, 2, 6.),
        }
    }
}
fn expected(asset: Asset, x: usize, y: usize) -> (u16, u8, u8, u16) {
    match asset {
        Asset::TeaTable => {
            let tiles = [
                [0x09, 0x0a, 0x0a, 0x0c],
                [0x19, 0x1a, 0x2c, 0x1c],
                [0x19, 0x40, 0x41, 0x1c],
                [0x19, 0x50, 0x51, 0x1c],
                [0x14, 0x82, 0x82, 0x35],
                [0x0b, 0x80, 0x81, 0x0b],
            ];
            (
                if y < 2 { 0x06 } else { 0x2d },
                x as u8,
                ((y + 2) % 4) as u8,
                tiles[y][x],
            )
        }
        Asset::KeeperCot => (
            0x36,
            (x + 2) as u8,
            y as u8,
            [[0x46, 0x47], [0x56, 0x57], [0x83, 0x84], [0x85, 0x86]][y][x],
        ),
        Asset::RedStool => (
            0x08,
            x as u8,
            (y + 2) as u8,
            [[0x07, 0x08], [0x17, 0x18]][y][x],
        ),
    }
}
fn source_matches(source: &VisualTileSource, asset: Asset, x: usize, y: usize) -> bool {
    let (block, sx, sy, tile) = expected(asset, x, y);
    source.tileset_id.as_ref() == "lighthouse"
        && source.metatile_id == block
        && source.subtile_column == sx
        && source.subtile_row == sy
        && source.tile_index == tile
}
fn matches_at(r: &Resolver<'_>, asset: Asset, column: usize, row: usize) -> bool {
    let (w, h, _) = asset.size();
    column + w <= r.g.width
        && row + h <= r.g.height
        && (0..h).all(|y| {
            (0..w).all(|x| {
                let i = (row + y) * r.g.width + column + x;
                !r.claimed[i] && source_matches(&r.cells[i].source, asset, x, y)
            })
        })
}
pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    if r.map != "OlivineLighthouse6F" || r.g.width == 0 || r.cells.len() != r.g.width * r.g.height {
        return;
    }
    let Some(ground) = r.cells.iter().position(|tile| {
        let s = &tile.source;
        s.tileset_id.as_ref() == "lighthouse"
            && s.metatile_id == 0x0b
            && s.subtile_column < 4
            && s.subtile_row < 4
            && s.tile_index
                == if (s.subtile_column + s.subtile_row) % 2 == 0 {
                    0x0d
                } else {
                    0x1d
                }
            && matches!(
                shape_for_source_on_map(r.map, s),
                CellShape::Flat | CellShape::PlaneAt { height: 0.0 }
            )
    }) else {
        return;
    };
    for asset in Asset::ALL {
        let (width, height, rise_pixels) = asset.size();
        for row in 0..r.g.height {
            for column in 0..r.g.width {
                if matches_at(r, asset, column, row) {
                    r.add(Placement {
                        column,
                        row,
                        width,
                        height,
                        ground,
                        form: Form::LighthouseChamber(asset),
                        rise_pixels,
                        depth_pixels: height as f32 * 8.,
                        front_rows: height as f32,
                    });
                }
            }
        }
    }
}
#[derive(Deserialize)]
struct Export {
    primitives: Vec<Primitive>,
}
#[derive(Deserialize)]
struct Primitive {
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
struct Model {
    surface: SurfaceMeshData,
}
impl Model {
    fn parse(source: &str) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(source)?;
        let mut surface = SurfaceMeshData::default();
        for p in export.primitives {
            let count = p.positions.len() / 3;
            if count == 0
                || p.positions.len() % 3 != 0
                || p.normals.len() != p.positions.len()
                || p.indices.is_empty()
                || p.indices.len() % 3 != 0
                || p.indices.iter().any(|&i| i as usize >= count)
                || p.positions
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                || p.normals.iter().any(|v| !v.is_finite())
                || p.base_color
                    .iter()
                    .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                || p.base_color[3] != 1.
            {
                return Err("invalid lighthouse chamber primitive".into());
            }
            let base = surface.positions.len() as u32;
            surface
                .positions
                .extend(p.positions.chunks_exact(3).map(|v| [v[0], v[1], v[2]]));
            for n in p.normals.chunks_exact(3) {
                let n = Vec3::new(n[0], n[1], n[2]);
                if (n.length_squared() - 1.).abs() > 0.001 {
                    return Err("invalid lighthouse chamber normal".into());
                }
                surface.normals.push(n.to_array());
            }
            surface.uvs.extend(vec![[0.; 2]; count]);
            surface.colors.extend(vec![p.base_color; count]);
            surface
                .indices
                .extend(p.indices.into_iter().map(|i| base + i));
        }
        if surface.positions.is_empty() {
            return Err("empty lighthouse chamber model".into());
        }
        Ok(Self { surface })
    }
}
fn model(asset: Asset) -> &'static Model {
    static MODELS: OnceLock<[Model; 3]> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            include_str!("../../models/lighthouse_chamber/tea_table.mesh.json"),
            include_str!("../../models/lighthouse_chamber/keeper_cot.mesh.json"),
            include_str!("../../models/lighthouse_chamber/red_stool.mesh.json"),
        ]
        .map(|s| Model::parse(s).expect("validated lighthouse chamber model"))
    })[asset as usize]
}
pub(super) fn append(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement, asset: Asset) {
    let uv = g.uv(p.ground % g.width, p.ground / g.width);
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            uv,
        );
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let scale = Vec3::new(
        p.width as f32 * g.tile_width,
        p.rise_pixels * g.tile_height / SOURCE_TILE_HEIGHT,
        p.height as f32 * g.tile_height,
    );
    let origin = Vec3::new(w, 0., n);
    let source = &model(asset).surface;
    let base = mesh.solid.positions.len() as u32;
    for ((&point, &normal), &color) in source
        .positions
        .iter()
        .zip(&source.normals)
        .zip(&source.colors)
    {
        mesh.solid
            .positions
            .push((origin + Vec3::from_array(point) * scale).to_array());
        let normal = (Vec3::from_array(normal) / scale).normalize();
        mesh.solid.normals.push(normal.to_array());
        mesh.solid
            .colors
            .push(crate::interior_models::authored_face_color(color, normal));
    }
    mesh.solid.uvs.extend_from_slice(&source.uvs);
    mesh.solid
        .indices
        .extend(source.indices.iter().map(|i| i + base));
    // Furniture and floor remain opaque and never join the masonry reveal mask.
    // In particular the native FLOOR stool does not acquire actor support.
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    const MAP: &str = "OlivineLighthouse6F";
    fn grid() -> GridGeometry {
        GridGeometry {
            width: 12,
            height: 12,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: -13.,
            origin_z: 5.,
        }
    }
    fn floor() -> Vec<VisualTile> {
        (0..144)
            .map(|i| VisualTile {
                column: (i % 12) as u32,
                row: (i / 12) as u32,
                texture: Default::default(),
                priority: false,
                source: VisualTileSource {
                    tileset_id: Arc::from("lighthouse"),
                    metatile_id: 0x0b,
                    subtile_column: (i % 4) as u8,
                    subtile_row: (i / 12 % 4) as u8,
                    tile_index: if (i % 12 + i / 12) % 2 == 0 {
                        0x0d
                    } else {
                        0x1d
                    },
                },
            })
            .collect()
    }
    fn object(asset: Asset) -> Vec<VisualTile> {
        let mut cells = floor();
        let (w, h, _) = asset.size();
        for y in 0..h {
            for x in 0..w {
                let (block, sx, sy, tile) = expected(asset, x, y);
                let s = &mut cells[(y + 3) * 12 + x + 4].source;
                s.metatile_id = block;
                s.subtile_column = sx;
                s.subtile_row = sy;
                s.tile_index = tile;
            }
        }
        cells
    }
    fn resolve(cells: &[VisualTile], map: &str, g: &GridGeometry) -> Vec<Placement> {
        let refs: Vec<_> = cells.iter().collect();
        let mut r = Resolver {
            map,
            cells: &refs,
            g,
            claimed: vec![false; cells.len()],
            out: vec![],
        };
        resolve_into(&mut r);
        r.out
    }
    #[test]
    fn lighthouse_chamber_exact_complete_pictures_reject_every_changed_cell() {
        let g = grid();
        for asset in Asset::ALL {
            let t = object(asset);
            let p = resolve(&t, MAP, &g);
            assert_eq!(p.len(), 1);
            assert_eq!(p[0].kind_label(), asset.label());
            let (w, h, _) = asset.size();
            assert_eq!(p[0].indices(g.width).count(), w * h);
            for i in p[0].indices(g.width) {
                for field in 0..5 {
                    let mut bad = t.clone();
                    let s = &mut bad[i].source;
                    match field {
                        0 => s.tile_index ^= 1,
                        1 => s.metatile_id ^= 1,
                        2 => s.subtile_column ^= 1,
                        3 => s.subtile_row ^= 1,
                        _ => s.tileset_id = Arc::from("custom_lighthouse"),
                    }
                    assert!(
                        resolve(&bad, MAP, &g).is_empty(),
                        "{asset:?} cell {i} field {field}"
                    );
                }
            }
            for map in [
                "FastShipCabins_NNW_NNE_NE",
                "OlivineLighthouse1F",
                "OlivineLighthouse5F",
                "OlivineLighthouse6FBeta",
            ] {
                assert!(resolve(&t, map, &g).is_empty());
            }
            let mut missing_floor = t.clone();
            for tile in &mut missing_floor {
                if tile.source.metatile_id == 0x0b {
                    tile.source.metatile_id = 0xff;
                }
            }
            assert!(resolve(&missing_floor, MAP, &g).is_empty());
        }
    }
    #[test]
    fn lighthouse_chamber_claims_crop_and_footing_are_safe() {
        let g = grid();
        for asset in Asset::ALL {
            let t = object(asset);
            let refs: Vec<_> = t.iter().collect();
            let p = resolve(&t, MAP, &g).remove(0);
            for i in p.indices(g.width) {
                let mut r = Resolver {
                    map: MAP,
                    cells: &refs,
                    g: &g,
                    claimed: vec![false; t.len()],
                    out: vec![],
                };
                r.claimed[i] = true;
                resolve_into(&mut r);
                assert!(r.out.is_empty());
            }
            for (w, h) in [
                (p.column + p.width - 1, g.height),
                (g.width, p.row + p.height - 1),
            ] {
                let mut cropped = Vec::new();
                for y in 0..h {
                    for x in 0..w {
                        cropped.push(t[y * g.width + x].clone());
                    }
                }
                let cg = GridGeometry {
                    width: w,
                    height: h,
                    ..g
                };
                assert!(resolve(&cropped, MAP, &cg).is_empty());
            }
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![2.25; g.width * g.height];
            mesh.solid.cutaway_ranges = vec![0..0];
            let footing = mesh.footing_heights.clone();
            let ranges = mesh.solid.cutaway_ranges.clone();
            append(&mut mesh, &g, &p, asset);
            assert_eq!(mesh.footing_heights, footing);
            assert_eq!(mesh.solid.cutaway_ranges, ranges);
            let (w, _, n, _) = g.bounds(p.column, p.row);
            let e = w + p.width as f32 * g.tile_width;
            let s = n + p.height as f32 * g.tile_height;
            assert!(mesh.solid.positions.iter().all(|v| v[0] >= w
                && v[0] <= e
                && v[2] >= n
                && v[2] <= s
                && v[1] >= 0.
                && v[1] <= p.rise_pixels));
            assert!(mesh.textured.positions.iter().all(|v| v[1] == 0.));
            assert_eq!(mesh.textured.indices.len(), p.width * p.height * 6);
            let source_colors = &model(asset).surface.colors;
            let mut shades: Vec<_> = mesh
                .solid
                .colors
                .iter()
                .zip(source_colors)
                .filter_map(|(lit, base)| {
                    assert_eq!(lit[3], base[3]);
                    (base[0] > 0.00001).then_some(lit[0] / base[0])
                })
                .collect();
            shades.sort_by(f32::total_cmp);
            assert!(
                shades.last().unwrap() - shades[0] > 0.2,
                "authored furniture needs face contrast in the baked-shade terrain pass"
            );
        }
    }
}
