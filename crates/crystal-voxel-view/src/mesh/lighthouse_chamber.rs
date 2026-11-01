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
    part: String,
    material: String,
    positions: Vec<f32>,
    normals: Vec<f32>,
    indices: Vec<u32>,
    base_color: [f32; 4],
}
struct Model {
    surface: SurfaceMeshData,
    pixels: Vec<[usize;2]>,
}
fn material_pixel(asset:Asset,material:&str)->Option<[usize;2]> {
    Some(match (asset,material) {
        (Asset::TeaTable,"linen") => [8,5],
        (Asset::TeaTable,"linen_edge"|"wood_cut"|"walnut") => [8,3],
        (Asset::TeaTable,"wood_highlight") => [8,2],
        (Asset::TeaTable,"brass"|"brass_shadow"|"glaze_light"|"sea_glaze") => [8,16],
        (Asset::TeaTable,"tea") => [14,17],
        (Asset::KeeperCot,"linen"|"linen_edge"|"sage_blanket") => [4,12],
        (Asset::KeeperCot,"walnut"|"wood_cut") => [1,3],
        (Asset::KeeperCot,"wood_highlight") => [1,9],
        (Asset::KeeperCot,"dark_joinery") => [0,0],
        (Asset::RedStool,"terracotta"|"wood_highlight") => [5,5],
        (Asset::RedStool,"cushion_edge"|"walnut"|"wood_cut") => [3,4],
        _=>return None,
    })
}
fn part_pixel(asset:Asset,material:&str,part:&str)->Option<[usize;2]> {
    if asset==Asset::TeaTable && part.starts_with("Cup ") {
        return match material {
            "glaze_light"|"sea_glaze"=>Some([10,9]),
            "tea"=>Some([10,10]),
            _=>None,
        };
    }
    material_pixel(asset,material)
}
impl Model {
    fn parse<'a>(source: impl Into<crate::model_storage::Source<'a>>,asset:Asset) -> Result<Self, String> {
        let export: Export = crate::model_storage::parse(source)?;
        let mut surface = SurfaceMeshData::default();
        let mut pixels=Vec::new();
        for p in export.primitives {
            let pixel=part_pixel(asset,&p.material,&p.part).ok_or_else(||format!("unknown lighthouse material {}",p.material))?;
            let count = p.positions.len() / 3;
            pixels.extend(vec![pixel;count]);
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
        Ok(Self { surface,pixels })
    }
}
fn model(asset: Asset) -> &'static Model {
    static MODELS: OnceLock<[Model; 3]> = OnceLock::new();
    &MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!(
                "models/lighthouse_chamber/tea_table.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/lighthouse_chamber/keeper_cot.mesh.json"
            ),
            crate::model_storage::include_model!(
                "models/lighthouse_chamber/red_stool.mesh.json"
            ),
        ]
        .into_iter().zip(Asset::ALL).map(|(s,a)| Model::parse(s,a).expect("validated lighthouse chamber model")).collect::<Vec<_>>().try_into().ok().expect("three lighthouse models")
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
    let model=model(asset);
    let source = &model.surface;
    let base = mesh.textured.positions.len() as u32;
    for ((&point, &normal), &pixel) in source
        .positions
        .iter()
        .zip(&source.normals)
        .zip(&model.pixels)
    {
        mesh.textured
            .positions
            .push((origin + Vec3::from_array(point) * scale).to_array());
        let normal = (Vec3::from_array(normal) / scale).normalize();
        mesh.textured.normals.push(normal.to_array());
        mesh.textured
            .colors
            .push(crate::interior_models::authored_face_color([1.;4], normal));
        let uv=g.uv(p.column+pixel[0]/8,p.row+pixel[1]/8);
        mesh.textured.uvs.push([uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
            uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]);
    }
    mesh.textured
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
    #[test]
    fn external_keeper_furniture_uses_its_own_native_palette_roles() {
        let path=std::env::var_os("CRYSTAL_RENDER_TEST_PACK").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../content-packs/text-tui.crystalpack"));
        if !path.is_file(){return;}
        let loaded=crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pixels=image::load_from_memory(&loaded.pack().runtime_files()["gfx/tilesets/lighthouse.png"]).unwrap().to_luma8();
        for (material,part,value) in [("sea_glaze","Cup faceted bowl",255),("tea","Cup tea inset",0)] {
            let p=part_pixel(Asset::TeaTable,material,part).unwrap();let (_,_,_,tile)=expected(Asset::TeaTable,p[0]/8,p[1]/8);
            assert_eq!(pixels.get_pixel((tile as u32%16)*8+(p[0]%8)as u32,
                (tile as u32/16)*8+(p[1]%8)as u32)[0],value);
            assert_ne!(p,part_pixel(Asset::TeaTable,material,"Teapot faceted body").unwrap());
        }
        for (asset,material,value) in [
            (Asset::TeaTable,"linen",170),(Asset::TeaTable,"linen_edge",85),
            (Asset::TeaTable,"brass",170),(Asset::TeaTable,"tea",0),
            (Asset::KeeperCot,"linen",255),(Asset::KeeperCot,"sage_blanket",255),
            (Asset::KeeperCot,"wood_highlight",170),(Asset::KeeperCot,"walnut",85),
            (Asset::RedStool,"terracotta",170),(Asset::RedStool,"cushion_edge",85),
        ] {
            let p=if asset==Asset::TeaTable&&material=="tea" {
                part_pixel(asset,material,"Cup tea inset").unwrap()
            }else{material_pixel(asset,material).unwrap()};
            let (_,_,_,tile)=expected(asset,p[0]/8,p[1]/8);
            assert_eq!(pixels.get_pixel((tile as u32%16)*8+(p[0]%8)as u32,
                (tile as u32/16)*8+(p[1]%8)as u32)[0],value,"{asset:?} {material}");
            let t=object(asset);let g=grid();let placement=resolve(&t,MAP,&g).remove(0);
            let mut mesh=TerrainMeshData::default();append(&mut mesh,&g,&placement,asset);
            let uv=g.uv(placement.column+p[0]/8,placement.row+p[1]/8);
            assert!(mesh.textured.uvs.contains(&[uv.0+(uv.1-uv.0)*((p[0]%8)as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*((p[1]%8)as f32+0.5)/8.]));
        }
    }
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
                animation_frames: None,
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
            assert!(mesh.textured.positions.iter().all(|v| v[0] >= w
                && v[0] <= e
                && v[2] >= n
                && v[2] <= s
                && v[1] >= 0.
                && v[1] <= p.rise_pixels));
            assert!(mesh.solid.positions.is_empty());
            let floor_vertices=p.width*p.height*4;
            assert!(mesh.textured.positions[..floor_vertices].iter().all(|v|v[1]==0.));
            assert_eq!(mesh.textured.indices.len(), p.width * p.height * 6+model(asset).surface.indices.len());
            let mut shades: Vec<_> = mesh
                .textured
                .colors
                .iter()
                .skip(floor_vertices)
                .map(|lit|{assert_eq!(lit[3],1.);lit[0]})
                .collect();
            shades.sort_by(f32::total_cmp);
            assert!(
                shades.last().unwrap() - shades[0] > 0.2,
                "authored furniture needs face contrast in the baked-shade terrain pass"
            );
        }
    }
}
