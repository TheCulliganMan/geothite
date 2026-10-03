#[test]
fn room_finish_masks_preserve_source_doorway_and_partition_ownership() {
    let mut occupied = vec![false; 32 * 16];
    for n in NETWORKS {
        let p = Placement {
            network: n,
            column: n.anchor[0] as usize,
            row: n.anchor[1] as usize,
            ground: 0,
            grid_origin: [0, 0],
        };
        for i in p.indices(32) {
            occupied[i] = true;
        }
    }
    let mut capsule = 0;
    let mut floors = 0;
    for n in ROOM_FINISH {
        assert_eq!(n.rows.len(), n.height);
        let p = Placement {
            network: n,
            column: n.anchor[0] as usize,
            row: n.anchor[1] as usize,
            ground: 0,
            grid_origin: [0, 0],
        };
        for i in p.indices(32) {
            assert!(!occupied[i], "finish overlaps a model at {i}");
            occupied[i] = true;
            let (x, y) = (i % 32, i / 32);
            match n.finish {
                Finish::TimeCapsule => {
                    capsule += 1;
                    assert!((26..28).contains(&x) && y < 4);
                }
                Finish::CeramicFloor => {
                    floors += 1;
                    assert!(y == 2 || y == 6);
                }
                _ => panic!("room finish must not duplicate another fixture"),
            }
            // The live source doorway remains entirely outside the new kit.
            assert!(!(26..28).contains(&x) || !(4..6).contains(&y));
        }
    }
    assert_eq!((capsule, floors), (8, 34));
    // These two source floor-looking strips belong to WALL quadrants. Leave
    // them to the original renderer instead of inventing floor permission.
    assert!(!occupied[2 * 32 + 12]);
    assert!(!occupied[2 * 32 + 20]);
}

#[test]
fn capsule_and_floor_use_the_same_atomic_source_and_custom_profile_guards() {
    for finish in [Finish::TimeCapsule, Finish::CeramicFloor] {
        let (cells, g, mut n) = fixture([5, 0], [4, 9]);
        n.finish = finish;
        let nets = Box::leak(vec![n].into_boxed_slice());
        let reserved = vec![false; cells.len()];
        assert_eq!(
            resolve_fixture(nets, &cells, &g, [5, 0], None, &reserved).len(),
            1
        );
        for index in 0..cells.len() {
            let mut changed = cells.clone();
            changed[index].source.tile_index ^= 1;
            assert!(resolve_fixture(nets, &changed, &g, [5, 0], None, &reserved).is_empty());
        }
        let doc: Document = serde_json::from_value(serde_json::json!({"objects":[{"name":"Custom room finish","tileset":"pokecenter","map":"Pokecenter2F","metatile":49,"origin":[2,0],"tiles":[[22]],"ground":17,"top_pixels":1,"depth_pixels":3.0}],"atmosphere":null})).unwrap();
        assert!(resolve_fixture(nets, &cells, &g, [5, 0], Some(&doc), &reserved).is_empty());
        let mut blocked = reserved.clone();
        blocked[1] = true;
        assert!(resolve_fixture(nets, &cells, &g, [5, 0], None, &blocked).is_empty());
        assert!(resolve_fixture(nets, &cells, &g, [6, 0], None, &reserved).is_empty());
    }
}

#[test]
fn ceramic_finish_is_zero_height_preserves_footing_and_is_not_object_coverage() {
    let (cells, g, mut n) = fixture([-25, -27], [84, 82]);
    n.finish = Finish::CeramicFloor;
    n.shells = &[];
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut claims = vec![false; cells.len()];
    let p = resolve_fixture(nets, &cells, &g, [-25, -27], None, &claims);
    let refs = cells.iter().collect::<Vec<_>>();
    let mut mesh = TerrainMeshData::default();
    mesh.footing_heights = vec![2.75; cells.len()];
    mesh.authored_cells = vec![None; cells.len()];
    let first = p[0].indices(g.width).next().unwrap();
    claims[first] = true;
    assert!(!append(&mut mesh, &refs, &g, &p[0], &mut claims));
    assert!(mesh.solid.positions.is_empty());
    claims[first] = false;
    assert!(append(&mut mesh, &refs, &g, &p[0], &mut claims));
    assert!(!mesh.solid.positions.is_empty());
    assert!(mesh.solid.positions.iter().all(|v| v[1] == 0.0));
    assert!(mesh.solid.normals.iter().all(|n| *n == [0.0, 1.0, 0.0]));
    assert!(mesh.textured.positions.is_empty());
    assert!(mesh.authored_cells.iter().all(Option::is_none));
    assert!(mesh.footing_heights.iter().all(|&h| h == 2.75));
    assert_eq!(claims.iter().filter(|&&b| b).count(), 16);
    let (west, _, north, _) = g.bounds(p[0].column + 1, p[0].row);
    assert!(
        mesh.solid
            .positions
            .iter()
            .all(|&[x, _, z]| x >= west && x <= west + 16.0 && z >= north && z <= north + 64.0)
    );
}

#[test]
fn capsule_append_stays_in_its_eight_cells_without_changing_footing() {
    let (cells, g, mut n) = fixture([5, 0], [4, 9]);
    n.finish = Finish::TimeCapsule;
    n.rows = &[6, 6, 6, 6, 0, 0, 0, 0, 0];
    n.shells = &[[1, 0, 2, 4]];
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut claims = vec![false; cells.len()];
    let p = resolve_fixture(nets, &cells, &g, [5, 0], None, &claims);
    let mut mesh = TerrainMeshData::default();
    mesh.footing_heights = vec![0.0; cells.len()];
    mesh.authored_cells = vec![None; cells.len()];
    assert!(append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        &p[0],
        &mut claims
    ));
    assert_eq!(claims.iter().filter(|&&b| b).count(), 8);
    assert_eq!(mesh.textured.positions.len(), 8 * 4);
    assert!(
        mesh.solid
            .positions
            .iter()
            .all(|&[x, y, z]| (8.0..=24.0).contains(&x)
                && (0.0..=22.0).contains(&y)
                && (0.0..=32.0).contains(&z))
    );
    assert!(mesh.footing_heights.iter().all(|&v| v == 0.0));
    assert_eq!(
        mesh.authored_cells
            .iter()
            .filter(|&&v| v == Some("pokecenter/time_capsule"))
            .count(),
        8
    );
    assert!(claims[4 * g.width..].iter().all(|&v| !v));
}
