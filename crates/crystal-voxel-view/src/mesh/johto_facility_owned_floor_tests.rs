use super::*;

#[test]
fn johto_facility_floor_replaces_rocket_b2f_owned_gray_lids_with_live_source() {
    let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external pack required");
    let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
    let pack = loaded.pack();
    let data = &pack.data().maps["TeamRocketBaseB2F"];
    let art = &pack.runtime_files()["data/tilesets/facility_metatiles.bin"];
    let (w, h) = (
        data.attributes.width as usize * 4,
        data.attributes.height as usize * 4,
    );
    let tiles = (0..w * h)
        .map(|i| {
            let (x, y) = (i % w, i / w);
            let b = data.blocks[y / 4 * (w / 4) + x / 4];
            VisualTile {
                column: x as u32,
                row: y as u32,
                texture: Default::default(),
                animation_frames: None,
                priority: false,
                source: VisualTileSource {
                    tileset_id: "facility".into(),
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
    let mut resolver = Resolver {
        map: "TeamRocketBaseB2F",
        cells: &refs,
        g: &g,
        claimed: vec![false; w * h],
        out: vec![],
    };
    resolve_into(&mut resolver);
    let mut count = 0;
    for p in &resolver.out {
        let Form::Room(Detail::Surface(Surface::Facility), style) = p.form else {
            continue;
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![2.75; w * h],
            ..Default::default()
        };
        append_with_floor(
            &mut mesh,
            &g,
            p,
            &refs,
            Detail::Surface(Surface::Facility),
            style,
        );
        assert!(p.indices(w).all(|i| {
            crate::mesh::modeled_interiors::native_johto_facility_floor_source(&tiles[i].source)
        }));
        assert!(
            mesh.solid.indices.is_empty(),
            "an opaque gray lid would conceal the checker finish"
        );
        assert_eq!(mesh.textured.indices.len(), p.width * p.height * 6);
        assert_eq!(mesh.footing_heights, vec![2.75; w * h]);
        for (v, uv) in mesh.textured.positions.iter().zip(&mesh.textured.uvs) {
            assert_eq!(v[1], 0.);
            assert!((uv[0] - v[0] / (w as f32 * 8.)).abs() < 0.00001);
            assert!((uv[1] - v[2] / (h as f32 * 8.)).abs() < 0.00001);
        }
        count += p.width * p.height;
    }
    assert_eq!(
        count, 1032,
        "all B2F source checker fragments remain authored and source-colored"
    );
}
