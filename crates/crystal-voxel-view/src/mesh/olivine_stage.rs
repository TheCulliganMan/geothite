//! Whole-drawing raised terrace; collision and scripts stay authoritative.
use super::*;
use crate::interior_models::{ModelKind, model};
use crate::live_profiles::Document;

const DRAWING: u64 = 0x7282c18080447b71;
const ANCHOR: [i32; 2] = [4, 4];
const SIZE: [usize; 2] = [12, 8];

fn identity_hash<'a>(sources: impl IntoIterator<Item = &'a VisualTileSource>) -> Option<u64> {
    let mut hash = 0xcbf29ce484222325u64;
    let mut count = 0;
    for s in sources {
        if s.tileset_id.as_ref() != "champions_room" { return None; }
        for byte in [(s.metatile_id & 255) as u8, (s.metatile_id >> 8) as u8,
            s.subtile_column, s.subtile_row, (s.tile_index & 255) as u8,
            (s.tile_index >> 8) as u8] {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
        count += 1;
    }
    (count == SIZE[0] * SIZE[1]).then_some(hash)
}

#[derive(Clone, Copy)]
pub(super) struct Placement { column: usize, row: usize }
impl Placement {
    pub(super) fn indices(&self, width: usize) -> impl Iterator<Item = usize> + '_ {
        (0..8).flat_map(move |y| (0..12).filter_map(move |x| {
            // Two native ground-art corners are not part of the terrace body.
            (!(y == 0 && (x == 0 || x == 11)))
                .then_some((self.row + y) * width + self.column + x)
        }))
    }
}

pub(super) fn resolve(map: &str, cells: &[&VisualTile], g: &GridGeometry,
    origin: [i32; 2], profiles: Option<&Document>, reserved: &[bool]) -> Option<Placement> {
    resolve_drawing(map, cells, g, origin, profiles, reserved, DRAWING)
}

fn resolve_drawing(map: &str, cells: &[&VisualTile], g: &GridGeometry,
    origin: [i32; 2], profiles: Option<&Document>, reserved: &[bool], drawing: u64)
    -> Option<Placement> {
    if map != "OlivineGym" || cells.len() != g.width * g.height
        || reserved.len() != cells.len() { return None; }
    let x = i64::from(ANCHOR[0]) - i64::from(origin[0]);
    let y = i64::from(ANCHOR[1]) - i64::from(origin[1]);
    if x < 0 || y < 0 { return None; }
    let p = Placement { column: x as usize, row: y as usize };
    if p.column + SIZE[0] > g.width || p.row + SIZE[1] > g.height { return None; }
    let sources = (0..8).flat_map(|dy| (0..12).map(move |dx|
        &cells[(p.row + dy) * g.width + p.column + dx].source));
    if identity_hash(sources) != Some(drawing) { return None; }
    let mut blocked = reserved.to_vec();
    for custom in live::resolve(cells, g.width, g.height, map, profiles) {
        for i in custom.indices(g.width) { blocked[i] = true; }
    }
    (!p.indices(g.width).any(|i| blocked[i])).then_some(p)
}

pub(super) fn append(mesh: &mut TerrainMeshData, g: &GridGeometry,
    p: &Placement, claimed: &mut [bool]) -> bool {
    if claimed.len() != g.width * g.height || p.column + 12 > g.width
        || p.row + 8 > g.height || p.indices(g.width).any(|i| claimed[i])
        || mesh.footing_heights.len() != claimed.len()
        || mesh.authored_cells.len() != claimed.len() { return false; }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let stage = model(ModelKind::OlivineStage);
    if !stage.append_olivine_stage(&mut mesh.textured,
        [w, w + 12.0 * g.tile_width, n, n + 8.0 * g.tile_height],
        g.tile_height,|material,point,normal| {
            let pixel=match material {
                0|1=>[16.,16.],2=>[14.,4.],3=>[14.,3.],4=>[15.,0.],
                5=>[34.,51.],6=>[37.,49.],_=>unreachable!("validated Olivine stage materials"),
            };
            let uv=g.uv(p.column+pixel[0]as usize/8,p.row+pixel[1]as usize/8);
            if material<=1 {
                let x=if normal[2].abs()>normal[0].abs() {point[0]} else if normal[1].abs()<0.5 {point[2]} else {point[0]};
                let y=if normal[1].abs()<0.5 {point[1]} else {point[2]};
                [uv.0+(uv.1-uv.0)*x.rem_euclid(8.)/8.,
                 uv.2+(uv.3-uv.2)*y.rem_euclid(8.)/8.]
            } else {
                [uv.0+(uv.1-uv.0)*(pixel[0]%8.+0.5)/8.,
                 uv.2+(uv.3-uv.2)*(pixel[1]%8.+0.5)/8.]
            }
        }) {return false;}
    for i in p.indices(g.width) {
        let x = i % g.width - p.column;
        let y = i / g.width - p.row;
        let height = if (4..8).contains(&x) && y == 7 { 4.0 } else { 8.0 };
        mesh.footing_heights[i] = height * g.tile_height / 8.0;
        mesh.authored_cells[i] = Some("interior/olivine_stage");
        claimed[i] = true;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fixture() -> (Vec<VisualTile>, GridGeometry, u64) {
        let cells: Vec<_> = (0..96).map(|i| VisualTile {
            column: (i % 12) as u32, row: (i / 12) as u32,
            texture: Default::default(), priority: false, animation_frames: None,
            source: VisualTileSource { tileset_id: Arc::from("champions_room"),
                metatile_id: 0x1c, subtile_column: (i % 4) as u8,
                subtile_row: (i / 12 % 4) as u8, tile_index: i as u16 },
        }).collect();
        let hash = identity_hash(cells.iter().map(|c| &c.source)).unwrap();
        (cells, GridGeometry { width: 12, height: 8, tile_width: 12.0,
            tile_height: 6.0, origin_x: -12.0, origin_z: 5.0 }, hash)
    }

    #[test]
    fn olivine_stage_requires_complete_source_and_respects_every_reservation() {
        let (cells, g, hash) = fixture();
        let refs = cells.iter().collect::<Vec<_>>();
        let empty = vec![false; cells.len()];
        let p = resolve_drawing("OlivineGym", &refs, &g, ANCHOR, None, &empty, hash).unwrap();
        for i in 0..96 { for field in 0..5 {
            let mut altered = cells.clone();
            let s = &mut altered[i].source;
            match field {
                0 => s.metatile_id ^= 0x80, 1 => s.tile_index ^= 0x80,
                2 => s.subtile_column ^= 0x80, 3 => s.subtile_row ^= 0x80,
                _ => s.tileset_id = Arc::from("other"),
            }
            assert!(resolve_drawing("OlivineGym", &altered.iter().collect::<Vec<_>>(),
                &g, ANCHOR, None, &empty, hash).is_none());
        }}
        for i in p.indices(g.width) {
            let mut reserved = empty.clone(); reserved[i] = true;
            assert!(resolve_drawing("OlivineGym", &refs, &g, ANCHOR, None,
                &reserved, hash).is_none());
        }
        for map in ["ChampionsRoom", "PewterGym", "Unknown"] {
            assert!(resolve_drawing(map, &refs, &g, ANCHOR, None, &empty, hash).is_none());
        }
        assert!(resolve_drawing("OlivineGym", &refs, &g, [5, 4], None, &empty, hash).is_none());
        assert!(resolve_drawing("OlivineGym", &refs[..95], &g, ANCHOR, None, &empty, hash).is_none());
        let custom: Document = serde_json::from_value(serde_json::json!({"objects":[{
            "name":"Custom stage", "map":"OlivineGym", "tileset":"champions_room",
            "metatile":28, "origin":[0,0], "tiles":[[4]], "ground":1,
            "top_pixels":12,"depth_pixels":8}]})).unwrap();
        assert!(resolve_drawing("OlivineGym", &refs, &g, ANCHOR, Some(&custom), &empty, hash).is_none());
    }

    #[test]
    fn olivine_stage_append_is_atomic_and_stairs_support_the_deck() {
        let (cells, g, hash) = fixture();
        let p = resolve_drawing("OlivineGym", &cells.iter().collect::<Vec<_>>(), &g,
            ANCHOR, None, &vec![false;96], hash).unwrap();
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![72.0;96]; mesh.authored_cells = vec![None;96];
        let mut claims = vec![false;96]; claims[1] = true;
        let before = claims.clone();
        assert!(!append(&mut mesh, &g, &p, &mut claims));
        assert_eq!(claims, before); assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.is_empty());
        assert!(mesh.footing_heights.iter().all(|&h| h == 72.0));
        claims[1] = false; assert!(append(&mut mesh, &g, &p, &mut claims));
        assert_eq!(claims.iter().filter(|&&v|v).count(),94);
        assert_eq!(mesh.footing_heights[0],72.0);
        assert_eq!(mesh.footing_heights[11],72.0);
        for i in p.indices(g.width) {
            let expected = if (4..8).contains(&(i%12)) && i/12 == 7 {3.0} else {6.0};
            assert_eq!(mesh.footing_heights[i],expected);
            assert_eq!(mesh.authored_cells[i],Some("interior/olivine_stage"));
        }
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.iter().all(|&[x,y,z]|
            (-12.0..=132.0).contains(&x) && (0.0..=6.001).contains(&y)
                && (5.0..=53.0).contains(&z)));
        assert!(mesh.textured.cutaway_ranges.is_empty());
        let mut original=SurfaceMeshData::default();
        model(ModelKind::OlivineStage).append_fitted(&mut original,[-12.,132.,5.,53.],0.,6.);
        assert_eq!(mesh.textured.positions,original.positions);
        assert_eq!(mesh.textured.normals,original.normals);
        assert_eq!(mesh.textured.indices,original.indices);
        assert_eq!(mesh.textured.indices.len()/3,2528);
        for [x,y] in [[14,4],[14,3],[15,0],[34,51],[37,49]] {
            let uv=g.uv(x/8,y/8);
            assert!(mesh.textured.uvs.contains(&[
                uv.0+(uv.1-uv.0)*((x%8)as f32+0.5)/8.,
                uv.2+(uv.3-uv.2)*((y%8)as f32+0.5)/8.]));
        }
    }
}
