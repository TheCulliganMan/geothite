fn fixture(origin: [i32; 2], size: [usize; 2]) -> (Vec<VisualTile>, GridGeometry, Network) {
    let mut source = Vec::new();
    for y in 0..9 {
        for x in 0..4 {
            source.push(VisualTileSource {
                tileset_id: Arc::from("pokecenter"),
                metatile_id: if y == 8 {
                    4
                } else if y < 4 {
                    0x31
                } else {
                    0x32
                },
                subtile_column: ((x + 5) % 4) as u8,
                subtile_row: (y % 4) as u8,
                tile_index: if y == 8 { 0x11 } else { 0x16 },
            });
        }
    }
    let network = Network {
        anchor: [5, 0],
        width: 4,
        height: 9,
        fingerprint: identity_hash(source.iter()),
        rows: &[6, 6, 6, 6, 6, 6, 6, 6, 0],
        shells: &[[1, 0, 2, 8]],
        record_sign: false,
    };
    let [width, height] = size;
    let cells = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let sx = x as i32 + origin[0] - 5;
            let sy = y as i32 + origin[1];
            let src = if (0..4).contains(&sx) && (0..9).contains(&sy) {
                source[sy as usize * 4 + sx as usize].clone()
            } else {
                VisualTileSource {
                    tileset_id: Arc::from("pokecenter"),
                    metatile_id: u16::MAX,
                    subtile_column: 0,
                    subtile_row: 0,
                    tile_index: 0,
                }
            };
            VisualTile {
                column: x as u32,
                row: y as u32,
                source: src,
                texture: Default::default(),
                priority: false,
            }
        })
        .collect();
    (
        cells,
        GridGeometry {
            width,
            height,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        },
        network,
    )
}
fn resolve_fixture(
    networks: &'static [Network],
    cells: &[VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    doc: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    resolve_networks(
        networks,
        "Pokecenter2F",
        &cells.iter().collect::<Vec<_>>(),
        g,
        origin,
        doc,
        reserved,
    )
}
#[test]
fn exact_drawing_rejects_every_changed_identity_including_opening_halo() {
    let (cells, g, n) = fixture([5, 0], [4, 9]);
    let refs = cells.iter().collect::<Vec<_>>();
    assert!(complete(&refs, &g, [5, 0], 0, 0, &n));
    for i in 0..cells.len() {
        for field in 0..5 {
            let mut changed = cells.clone();
            let s = &mut changed[i].source;
            match field {
                0 => s.tileset_id = Arc::from("mart"),
                1 => s.metatile_id ^= 1,
                2 => s.subtile_column ^= 1,
                3 => s.subtile_row ^= 1,
                _ => s.tile_index ^= 1,
            }
            assert!(
                !complete(&changed.iter().collect::<Vec<_>>(), &g, [5, 0], 0, 0, &n),
                "cell {i}, field {field}"
            );
        }
    }
    assert!(!complete(&refs, &g, [6, 0], 0, 0, &n));
    assert!(!complete(&refs[..refs.len() - 1], &g, [5, 0], 0, 0, &n));
    assert!(!complete(&refs, &g, [5, 0], 1, 0, &n));
}
#[test]
fn padded_scrolled_and_cropped_canvases_respect_native_anchor() {
    for (origin, size) in [
        ([0, 0], [32, 16]),
        ([-32, -32], [84, 82]),
        ([-25, -27], [84, 82]),
        ([5, 0], [4, 9]),
    ] {
        let (cells, g, n) = fixture(origin, size);
        let nets = Box::leak(vec![n].into_boxed_slice());
        let p = resolve_fixture(nets, &cells, &g, origin, None, &vec![false; cells.len()]);
        assert_eq!(p.len(), 1, "{origin:?}");
        let actual = p[0]
            .indices(g.width)
            .map(|i| {
                (
                    origin[0] + (i % g.width) as i32,
                    origin[1] + (i / g.width) as i32,
                )
            })
            .collect::<Vec<_>>();
        let expected = (0..8)
            .flat_map(|y| (6..8).map(move |x| (x, y)))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
    for (origin, size) in [
        ([6, 0], [3, 9]),
        ([5, 1], [4, 8]),
        ([5, 0], [4, 8]),
        ([5, 0], [3, 9]),
    ] {
        let (cells, g, n) = fixture(origin, size);
        let nets = Box::leak(vec![n].into_boxed_slice());
        assert!(
            resolve_fixture(nets, &cells, &g, origin, None, &vec![false; cells.len()]).is_empty()
        );
    }
}
#[test]
fn sparse_source_masks_keep_booth_warps_actors_and_capsule_opening_clear() {
    assert_eq!(MAPS.len(), 11);
    let mut claims = vec![false; 32 * 16];
    let mut shells = 0;
    for n in NETWORKS {
        assert_eq!(n.rows.len(), n.height);
        assert!(n.width <= 16);
        let p = Placement {
            network: n,
            column: n.anchor[0] as usize,
            row: n.anchor[1] as usize,
            ground: 0,
            grid_origin: [0, 0],
        };
        for i in p.indices(32) {
            assert!(!claims[i]);
            claims[i] = true;
        }
        let mut modeled = vec![false; n.width * n.height];
        for &[x, y, w, h] in n.shells {
            shells += 1;
            for dy in y..y + h {
                for dx in x..x + w {
                    assert!(!modeled[dy * n.width + dx]);
                    modeled[dy * n.width + dx] = true;
                }
            }
        }
        for y in 0..n.height {
            for x in 0..n.width {
                assert_eq!(modeled[y * n.width + x], n.rows[y] & (1 << x) != 0);
            }
        }
    }
    assert_eq!(shells, 6);
    assert_eq!(claims.iter().filter(|&&b| b).count(), 88);
    // Native warp cells (map units are 16px) plus attendant/operator positions.
    for (x, y) in [
        (0, 7),
        (5, 0),
        (9, 0),
        (13, 2),
        (6, 0),
        (10, 0),
        (5, 2),
        (9, 2),
        (13, 3),
        (1, 1),
    ] {
        for dy in 0..2 {
            for dx in 0..2 {
                assert!(!claims[(y * 2 + dy) * 32 + x * 2 + dx]);
            }
        }
    }
    for y in 0..8 {
        for x in 26..28 {
            assert!(!claims[y * 32 + x]);
        }
    }
    for y in 8..16 {
        assert!(claims[y * 32..(y + 1) * 32].iter().all(|&b| !b));
    }
}
#[test]
fn overlap_custom_profiles_and_missing_floor_reject_entire_network() {
    let (cells, g, n) = fixture([5, 0], [4, 9]);
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut reserved = vec![false; cells.len()];
    for y in 0..8 {
        for x in 1..3 {
            let i = y * 4 + x;
            reserved[i] = true;
            assert!(resolve_fixture(nets, &cells, &g, [5, 0], None, &reserved).is_empty());
            reserved[i] = false;
        }
    }
    let doc:Document=serde_json::from_value(serde_json::json!({"objects":[{"name":"Custom Cable Club partition","tileset":"pokecenter","map":"Pokecenter2F","metatile":49,"origin":[2,0],"tiles":[[22]],"ground":17,"top_pixels":1,"depth_pixels":3.0}],"atmosphere":null})).unwrap();
    assert!(resolve_fixture(nets, &cells, &g, [5, 0], Some(&doc), &reserved).is_empty());
    let mut floor_only = doc.clone();
    floor_only.objects[0].metatile = 4;
    floor_only.objects[0].origin = [1, 0];
    floor_only.objects[0].tiles = vec![vec![17]];
    assert_eq!(
        resolve_fixture(nets, &cells, &g, [5, 0], Some(&floor_only), &reserved).len(),
        1
    );
    let mut missing = cells.clone();
    for c in &mut missing {
        if c.source.metatile_id == 4 {
            c.source.metatile_id = 0x31;
        }
    }
    assert!(resolve_fixture(nets, &missing, &g, [5, 0], None, &reserved).is_empty());
    assert!(
        resolve(
            "OtherPokecenter2F",
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [5, 0],
            None,
            &reserved
        )
        .is_empty()
    );
    let canonical: Document = serde_json::from_str(include_str!(
        "../../../../modpacks/voxel-view/profiles.json"
    ))
    .unwrap();
    let original = canonical
        .objects
        .iter()
        .find(|o| o.name == "Pokecenter2F booth partition 31-32")
        .unwrap();
    assert!(canonical_partition(original));
    let mut changed = original.clone();
    changed.depth_pixels += 1.;
    assert!(!canonical_partition(&changed));
}
#[test]
fn append_is_atomic_stays_in_bounds_and_preserves_footing() {
    let origin = [-25, -27];
    let (cells, g, n) = fixture(origin, [84, 82]);
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut claims = vec![false; cells.len()];
    let p = resolve_fixture(nets, &cells, &g, origin, None, &claims);
    let refs = cells.iter().collect::<Vec<_>>();
    let mut mesh = TerrainMeshData::default();
    mesh.footing_heights = vec![2.75; cells.len()];
    mesh.authored_cells = vec![None; cells.len()];
    let first = p[0].indices(g.width).next().unwrap();
    claims[first] = true;
    let before = claims.clone();
    assert!(!append(&mut mesh, &refs, &g, &p[0], &mut claims));
    assert_eq!(before, claims);
    assert!(mesh.solid.positions.is_empty());
    assert!(mesh.textured.positions.is_empty());
    claims[first] = false;
    assert!(append(&mut mesh, &refs, &g, &p[0], &mut claims));
    assert_eq!(claims.iter().filter(|&&b| b).count(), 16);
    assert_eq!(mesh.textured.positions.len(), 16 * 4);
    assert!(mesh.footing_heights.iter().all(|&h| h == 2.75));
    let (w, _, n, _) = g.bounds(p[0].column + 1, p[0].row);
    assert!(mesh.solid.positions.iter().all(|&[x, y, z]| x >= w
        && x <= w + 16.
        && y >= 0.
        && y <= 16.
        && z >= n
        && z <= n + 64.));
    assert!(
        mesh.solid
            .normals
            .iter()
            .all(|normal| (Vec3::from_array(*normal).length_squared() - 1.).abs() < 0.0001)
    );
    let mut missing = cells.clone();
    missing[p[0].ground].source.tileset_id = Arc::from("mart");
    let mut empty = vec![false; cells.len()];
    let mut rejected = TerrainMeshData::default();
    assert!(!append(
        &mut rejected,
        &missing.iter().collect::<Vec<_>>(),
        &g,
        &p[0],
        &mut empty
    ));
    assert!(rejected.solid.positions.is_empty());
    assert!(empty.iter().all(|&b| !b));
}
#[test]
fn live_record_inscription_uses_original_uvs_and_outward_face() {
    let (cells, g, mut n) = fixture([5, 0], [4, 9]);
    n.record_sign = true;
    let nets = Box::leak(vec![n].into_boxed_slice());
    let mut claimed = vec![false; cells.len()];
    let p = resolve_fixture(nets, &cells, &g, [5, 0], None, &claimed);
    let mut mesh = TerrainMeshData::default();
    assert!(append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        &p[0],
        &mut claimed
    ));
    assert_eq!(mesh.textured.positions.len(), 16 * 4 + 8);
    for dx in 0..2 {
        let first = 64 + dx * 4;
        let a = Vec3::from_array(mesh.textured.positions[first]);
        let b = Vec3::from_array(mesh.textured.positions[first + 1]);
        let c = Vec3::from_array(mesh.textured.positions[first + 2]);
        assert!((b - a).cross(c - a).z > 0.);
        let (u0, u1, v0, v1) = g.uv(1 + dx, 6);
        assert_eq!(
            &mesh.textured.uvs[first..first + 4],
            &[[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
        );
        assert!(
            mesh.textured.positions[first..first + 4]
                .iter()
                .all(|p| p[2] > 63.90 && p[2] < 64.)
        );
    }
}
