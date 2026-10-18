//! Source paint for the three southern-office computers.
use super::*;
pub(super) fn computer(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    p.kind == ModelKind::Computer
        && p.width == 4
        && p.height == 3
        && p.depth_pixels == 12.
        && p.height_pixels == 22.
        && p.base_pixels == 0.
        && p.ground < cells.len()
        && p.column + 4 <= g.width
        && p.row + 3 <= g.height
        && cells.len() == g.width * g.height
        && p.indices(g.width).enumerate().all(|(j, i)| {
            crate::pokecom::station_local(map, &cells[i].source)
                == Some(((j % 4) as u8, (j / 4) as u8))
        })
}
pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    if map != crate::pokecom::ADMIN_OFFICE {
        return;
    }
    let groups = grouped_flat_card_placements(
        cells,
        g,
        crate::pokecom::WORKSTATION_FLOOR_TILE,
        true,
        |s| crate::pokecom::station_local(map, s).map(|(x, y)| (x, y, 4, 3)),
    );
    for group in groups {
        let Some(ground) = ground_sample(
            cells,
            map,
            "pokecom_center",
            crate::pokecom::WORKSTATION_FLOOR_TILE,
        ) else {
            continue;
        };
        let p = Placement {
            kind: ModelKind::Computer,
            column: group.column,
            row: group.row,
            width: 4,
            height: 3,
            ground,
            depth_pixels: 12.,
            height_pixels: 22.,
            base_pixels: 0.,
            footing_pixels: None,
        };
        if !computer(map, &p, cells, g) || p.indices(g.width).any(|i| claimed[i]) {
            continue;
        }
        for i in p.indices(g.width) {
            claimed[i] = true;
        }
        out.push(p);
    }
}
// The upper source tile also contains the red counter lip. Electronics use
// the gray monitor/keyboard tiles below it, including their own palette bank.
pub(super) const PAINT: [[usize; 2]; 5] = [[4, 8], [4, 12], [2, 14], [2, 17], [4, 8]];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_pokecom_computers_claim_complete_side_keyboards_and_keep_source_support() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap();
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let map = crate::pokecom::ADMIN_OFFICE;
        let module = &pack.data().maps[map];
        let art = &pack.runtime_files()["data/tilesets/pokecom_center_metatiles.bin"];
        let (w, h) = (
            module.attributes.width as usize * 4,
            module.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let b = module.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "pokecom_center".into(),
                        metatile_id: b,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect::<Vec<_>>();
        let refs = tiles.iter().collect::<Vec<_>>();
        let g = GridGeometry {
            width: w,
            height: h,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let ps = resolve(map, &refs, &g, None);
        let pcs = ps
            .iter()
            .filter(|p| computer(map, p, &refs, &g))
            .collect::<Vec<_>>();
        assert_eq!(pcs.len(), 3);
        let png = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/pokecom_center.png"])
            .unwrap()
            .to_luma8();
        for p in pcs {
            let pigments = PAINT.map(|[x, y]| {
                let s = &tiles[(p.row + y / 8) * w + p.column + x / 8].source;
                png.get_pixel(
                    (s.tile_index as u32 % 16) * 8 + (x % 8) as u32,
                    (s.tile_index as u32 / 16) * 8 + (y % 8) as u32,
                )[0]
            });
            assert_eq!(pigments, [85, 170, 255, 170, 85]);
            for [x, y] in PAINT {
                let s = &tiles[(p.row + y / 8) * w + p.column + x / 8].source;
                assert!(
                    matches!(s.tile_index, 0x38..=0x3b),
                    "do not sample the red countertop above the monitor"
                );
                assert_eq!(
                    pack.data().tilesets["pokecom_center"].palette_map[s.tile_index as usize] & 7,
                    0,
                    "electronics retain their gray palette bank"
                );
            }
            let model = crate::interior_models::selected_model(p.kind).unwrap();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            let mut claims = vec![false; w * h];
            assert!(append(&mut mesh, map, &refs, &g, p, &mut claims, [0, 0]));
            assert_eq!(claims.iter().filter(|&&c| c).count(), 12);
            assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
            assert!(mesh.solid.indices.is_empty());
            let mut native = SurfaceMeshData::default();
            let b = [0., 32., 0., 12.];
            assert!(model.append_native_pokecom_station(
                &mut native,
                b,
                p.base_pixels,
                p.height_pixels,
                native_source_swatches(p, &g, PAINT)
            ));
            assert!(
                native
                    .positions
                    .iter()
                    .all(|v| v[0] >= 0. && v[0] <= 32. && v[2] >= 0. && v[2] <= 12.)
            );
            let paints = native_source_swatches(p, &g, PAINT);
            assert!(
                mesh.textured
                    .uvs
                    .iter()
                    .skip(48)
                    .take(native.positions.len())
                    .all(|uv| paints.contains(uv)),
                "electronics have only their own five paints"
            );
            let mut backing = SurfaceMeshData::default();
            native_pokecom_office::append_station_backing(&mut backing, map, p, &refs, &g);
            assert_eq!(
                &mesh.textured.positions[48 + native.positions.len()..],
                backing.positions
            );
            if p.row == 56 {
                assert!(
                    !backing.indices.is_empty(),
                    "lower station's red source band is an eight-pixel counter backing, never floor"
                );
                assert_eq!(
                    backing.positions.iter().map(|v| v[1]).fold(0., f32::max),
                    8.
                );
                assert!(
                    backing
                        .positions
                        .iter()
                        .all(|v| v[2] >= (p.row as f32) * 8. && v[2] <= ((p.row + 1) as f32) * 8.)
                );
            } else {
                assert!(
                    backing.indices.is_empty(),
                    "upper stations retain their existing three counter headers"
                );
            }
            for i in p.indices(w) {
                let mut damaged = tiles.clone();
                damaged[i].source.tile_index ^= 0x80;
                let r = damaged.iter().collect::<Vec<_>>();
                assert!(!computer(map, p, &r, &g));
                let mut stale = TerrainMeshData::default();
                let mut stale_claims = vec![false; w * h];
                assert!(!append(
                    &mut stale,
                    map,
                    &r,
                    &g,
                    p,
                    &mut stale_claims,
                    [0, 0]
                ));
                assert!(stale.textured.positions.is_empty() && stale.solid.positions.is_empty());
                assert!(stale_claims.iter().all(|v| !*v));
                assert!(
                    !resolve(map, &r, &g, None)
                        .iter()
                        .any(|q| q.kind == ModelKind::Computer
                            && q.column == p.column
                            && q.row == p.row)
                );
            }
            assert!(!computer("PlayersHouse2F", p, &refs, &g));
        }
    }
}
