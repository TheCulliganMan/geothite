//! Retain the authenticated blue paving, arena and threshold drawings.
use super::*;

fn map(style: FloorStyle) -> Option<&'static str> {
    match style {
        FloorStyle::TowerArena => Some("BattleTowerBattleRoom"),
        FloorStyle::TowerLift => Some("BattleTowerElevator"),
        _ => None,
    }
}

pub(super) fn placement(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry,
    detail: Detail) -> bool {
    if !matches!(map, "BattleTowerBattleRoom" | "BattleTowerElevator")
        || cells.len() != g.width * g.height || p.column + p.width > g.width
        || p.row + p.height > g.height { return false; }
    match detail {
        Detail::Surface(Surface::Ceramic) | Detail::Threshold => {
            p.width == 1 && p.height == 1
                && match (detail, cell_detail(map, &cells[p.row * g.width + p.column].source)) {
                    (Detail::Surface(Surface::Ceramic), Some(Detail::Surface(Surface::Ceramic)))
                    | (Detail::Threshold, Some(Detail::Threshold)) => true,
                    _ => false,
                }
        }
        Detail::Arena | Detail::Asset(Asset::TowerRoundFixture) => OBJECTS.iter().any(|o|
            matches!((detail,o.detail), (Detail::Arena,Detail::Arena)
                | (Detail::Asset(Asset::TowerRoundFixture),Detail::Asset(Asset::TowerRoundFixture)))
            && o.width == p.width && o.height == p.height
            && matches_object(map, cells, g, p.column, p.row, o)),
        _ => false,
    }
}

pub(super) fn append(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement,
    cells: &[&VisualTile], detail: Detail, style: FloorStyle) -> bool {
    let Some(map) = map(style) else { return false; };
    if !placement(map, p, cells, g, detail) { return false; }
    // One coplanar source drawing: do not add an unrelated gray arena above it.
    for i in p.indices(g.width) {
        append_top(&mut mesh.textured, g.bounds(i % g.width, i / g.width).into(), 0.,
            g.uv(i % g.width, i / g.width));
    }
    true
}

pub(super) fn underlay(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement,
    cells: &[&VisualTile]) -> bool {
    let Form::Room(_, style) = p.form else { return false; };
    let Some(map) = map(style) else { return false; };
    let Some(ground) = cells.get(p.ground) else { return false; };
    if !matches!(cell_detail(map, &ground.source), Some(Detail::Surface(Surface::Ceramic))) {
        return false;
    }
    for i in p.indices(g.width) {
        append_top(&mut mesh.textured, g.bounds(i % g.width, i / g.width).into(), 0.,
            g.uv(p.ground % g.width, p.ground / g.width));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_battle_tower_special_floors_keep_source_arena_thresholds_and_support() {
        let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else { return; };
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let layout = &pack.pack().runtime_files()["data/tilesets/battle_tower_inside_metatiles.bin"];
        for name in ["BattleTowerBattleRoom", "BattleTowerElevator"] {
            let room = &pack.pack().data().maps[name];
            let (w,h) = (room.attributes.width as usize * 4, room.attributes.height as usize * 4);
            let g = GridGeometry { width: w, height: h, tile_width: 8., tile_height: 8., origin_x: 0., origin_z: 0. };
            let tiles = (0..w*h).map(|i| {
                let (x,y) = (i % w, i / w); let block = room.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile { column: x as u32, row: y as u32, texture: Default::default(),
                    animation_frames: None, priority: false, source: VisualTileSource {
                        tileset_id: "battle_tower_inside".into(), metatile_id: block,
                        subtile_column: (x % 4) as u8, subtile_row: (y % 4) as u8,
                        tile_index: layout[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    } }
            }).collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let mut r = Resolver { map: name, cells: &refs, g: &g, claimed: vec![false; w*h], out: vec![] };
            resolve_into(&mut r); assert!(r.claimed.iter().all(|&v| v));
            let mut counts = [0;4];
            for p in &r.out {
                let Form::Room(detail, style) = p.form else { panic!("room binding"); };
                assert_eq!(map(style), Some(name));
                if !placement(name, p, &refs, &g, detail) { continue; }
                let index = match detail { Detail::Arena => 0, Detail::Threshold => 1,
                    Detail::Asset(Asset::TowerRoundFixture) => 3, _ => 2 };
                counts[index] += p.width * p.height;
                let mut mesh = TerrainMeshData { footing_heights: vec![2.75;w*h], ..Default::default() };
                append_with_floor(&mut mesh, &g, p, &refs, detail, style);
                assert!(mesh.solid.indices.is_empty());
                assert_eq!(mesh.textured.indices.len(), p.width * p.height * 6);
                assert_eq!(mesh.footing_heights, vec![2.75;w*h]);
                for (v,uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
                    assert_eq!(v[1], 0.);
                    assert!((uv[0] - v[0] / (w as f32 * 8.)).abs() < 0.0001);
                    assert!((uv[1] - v[2] / (h as f32 * 8.)).abs() < 0.0001);
                }
                for bad in p.indices(w) {
                    let mut changed = tiles.clone(); changed[bad].source.tile_index ^= 0x80;
                    assert!(!placement(name, p, &changed.iter().collect::<Vec<_>>(), &g, detail));
                }
                assert!(!placement("UnknownTowerRoom", p, &refs, &g, detail));
            }
            assert_eq!(counts, if name == "BattleTowerBattleRoom" { [64,16,116,8] } else { [0,8,36,0] });
            let mut custom = Resolver { map: name, cells: &refs, g: &g, claimed: vec![true;w*h], out: vec![] };
            resolve_into(&mut custom); assert!(custom.out.is_empty(), "existing source ownership wins");
        }
    }
}
