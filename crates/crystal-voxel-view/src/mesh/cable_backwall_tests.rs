#[test]
fn rear_fixture_masks_keep_mobile_warps_negative_space_and_actors_clear() {
    let mut used = vec![false; 32 * 16];
    let mut counts = [0; 3];
    for n in REAR_FIXTURES {
        let p = Placement {
            network: n,
            column: n.anchor[0] as usize,
            row: n.anchor[1] as usize,
            ground: 0,
            grid_origin: [0, 0],
        };
        for i in p.indices(32) {
            assert!(!used[i]);
            used[i] = true;
            let (x, y) = (i % 32, i / 32);
            assert!(
                !((12..14).contains(&x) || (20..22).contains(&x)) || y >= 2,
                "mobile warp at {x},{y}"
            );
            assert!(![12, 20].contains(&x) || y != 2, "source negative space");
            for (ax, ay) in [(5, 2), (9, 2), (13, 3), (1, 1)] {
                assert!(!(ax * 2..ax * 2 + 2).contains(&x) || !(ay * 2..ay * 2 + 2).contains(&y));
            }
            counts[match n.finish {
                Finish::RearWall => 0,
                Finish::LinkConsole => 1,
                Finish::Doorway => 2,
                _ => panic!("wrong fixture"),
            }] += 1;
        }
    }
    assert_eq!(counts, [4, 14, 12]);
    // The actual source doorways get open portal geometry rather than a full
    // PictureFrame backing; mobile warp art remains wholly outside ownership.
    for x in [10, 11, 18, 19] {
        for y in 0..2 {
            assert!(used[y * 32 + x]);
        }
    }
    for x in [26, 27] {
        for y in 4..6 {
            assert!(used[y * 32 + x]);
        }
    }
}
#[test]
fn rear_fixtures_share_atomic_identity_custom_and_reservation_fallback() {
    for finish in [Finish::RearWall, Finish::LinkConsole, Finish::Doorway] {
        let (cells, g, mut n) = fixture([5, 0], [4, 9]);
        n.finish = finish;
        let nets = Box::leak(vec![n].into_boxed_slice());
        let reserved = vec![false; cells.len()];
        assert_eq!(
            resolve_fixture(nets, &cells, &g, [5, 0], None, &reserved).len(),
            1
        );
        for i in 0..cells.len() {
            for field in 0..5 {
                let mut altered = cells.clone();
                let s = &mut altered[i].source;
                match field {
                    0 => s.tileset_id = Arc::from("mart"),
                    1 => s.metatile_id ^= 1,
                    2 => s.subtile_column ^= 1,
                    3 => s.subtile_row ^= 1,
                    _ => s.tile_index ^= 1,
                }
                assert!(resolve_fixture(nets, &altered, &g, [5, 0], None, &reserved).is_empty());
            }
        }
        let custom:Document=serde_json::from_value(serde_json::json!({"objects":[{"name":"Custom Cable Club architecture","tileset":"pokecenter","map":"Pokecenter2F","metatile":49,"origin":[2,0],"tiles":[[22]],"ground":17,"top_pixels":1,"depth_pixels":3.0}],"atmosphere":null})).unwrap();
        assert!(resolve_fixture(nets, &cells, &g, [5, 0], Some(&custom), &reserved).is_empty());
        let mut blocked = reserved.clone();
        blocked[1] = true;
        assert!(resolve_fixture(nets, &cells, &g, [5, 0], None, &blocked).is_empty());
        assert!(resolve_fixture(nets, &cells, &g, [6, 0], None, &reserved).is_empty());
    }
}
#[test]
fn source_door_is_live_at_zero_and_open_frame_does_not_change_footing() {
    let (cells, g, mut n) = fixture([5, 0], [4, 9]);
    n.finish = Finish::Doorway;
    n.rows = &[6, 6, 0, 0, 0, 0, 0, 0, 0];
    n.shells = &[[1, 0, 2, 2]];
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut claimed = vec![false; cells.len()];
    let p = resolve_fixture(nets, &cells, &g, [5, 0], None, &claimed);
    let refs = cells.iter().collect::<Vec<_>>();
    let mut mesh = TerrainMeshData::default();
    mesh.footing_heights = vec![2.5; cells.len()];
    mesh.authored_cells = vec![None; cells.len()];
    claimed[1] = true;
    assert!(!append(&mut mesh, &refs, &g, &p[0], &mut claimed));
    assert!(mesh.solid.positions.is_empty());
    claimed[1] = false;
    assert!(append(&mut mesh, &refs, &g, &p[0], &mut claimed));
    assert_eq!(claimed.iter().filter(|&&x| x).count(), 4);
    assert_eq!(mesh.textured.positions.len(), 16);
    assert!(mesh.textured.positions.iter().all(|v| v[1] == 0.0));
    let expected = (0..2)
        .flat_map(|y| (1..3).map(move |x| g.uv(x, y)))
        .collect::<Vec<_>>();
    for (quad, (u0, u1, v0, v1)) in mesh.textured.uvs.chunks(4).zip(expected) {
        assert_eq!(quad, &[[u0, v0], [u0, v1], [u1, v1], [u1, v0]]);
    }
    assert!(mesh.footing_heights.iter().all(|&h| h == 2.5));
    // The original open-sliding-door asset has neither back plane nor glass
    // across its middle. Only a subpixel threshold crosses the low center.
    for &[x, y, _] in &mesh.solid.positions {
        assert!(!((12.0..20.0).contains(&x) && (1.0..15.0).contains(&y)));
    }
}
#[test]
fn link_console_preserves_mobile_warp_and_native_negative_corner() {
    let (cells, g, mut n) = fixture([5, 0], [4, 9]);
    n.finish = Finish::LinkConsole;
    n.rows = &[0, 0, 4, 6, 6, 6, 0, 0, 0];
    n.shells = &[[1, 2, 2, 4]];
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut claimed = vec![false; cells.len()];
    let p = resolve_fixture(nets, &cells, &g, [5, 0], None, &claimed);
    let mut mesh = TerrainMeshData::default();
    mesh.footing_heights = vec![1.25; cells.len()];
    mesh.authored_cells = vec![None; cells.len()];
    assert!(append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        &p[0],
        &mut claimed
    ));
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 7);
    assert!(!claimed[2 * g.width + 1]);
    assert!(
        mesh.solid
            .positions
            .iter()
            .all(|&[x, y, z]| (8.0..=24.0).contains(&x)
                && (0.0..=20.0).contains(&y)
                && (16.0..=48.0).contains(&z))
    );
    assert!(
        mesh.solid
            .positions
            .iter()
            .all(|&[x, _, z]| x >= 16.0 || z >= 24.0)
    );
    assert!(mesh.footing_heights.iter().all(|&h| h == 1.25));
}
