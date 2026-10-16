// Synthetic identities exercise matching independently of the external pack.
// tools/check-department-store.py separately verifies every actual map binding.
#[test]
fn joined_service_counter_tops_are_continuous_and_registers_fit_the_bay() {
    for scale in [0.5,1.,1.5] {
        let mut mesh=SurfaceMeshData::default();
        let bays=[(Asset::CounterPlain,[0.,16.,0.,16.]),
            (Asset::CounterRegister,[16.,32.,0.,16.]),
            (Asset::CounterWest,[0.,16.,16.,48.])];
        for (asset,bounds) in bays {
            append_service_counter(&mut mesh,asset,bounds.map(|v|v*scale),scale);
        }
        let mut area=0.;
        for triangle in mesh.indices.chunks_exact(3) {
            let points=[triangle[0],triangle[1],triangle[2]].map(|i|Vec3::from_array(mesh.positions[i as usize]));
            if points.iter().all(|p|(p.y-8.*scale).abs()<0.0001) {
                let normal=(points[1]-points[0]).cross(points[2]-points[0]);
                if normal.y>0. {area+=normal.y*0.5;}
            }
        }
        assert!((area-1024.*scale*scale).abs()<0.001,"tops must cover touching bays without trenches or overlap");
        let mut register=SurfaceMeshData::default();
        append_cash_register(&mut register,[0.,16.*scale,0.,16.*scale],scale);
        assert!(register.positions.iter().all(|p|p[0]>3.9*scale && p[0]<12.*scale
            && p[2]>4.*scale && p[2]<12.*scale && p[1]>=8.*scale && p[1]<11.2*scale));
        assert!(register.positions.iter().any(|p|(p[1]-8.*scale).abs()<0.0001),"drawer must sit on top");
    }
}
fn fixture(
    which: usize,
    origin: [i32; 2],
    size: [usize; 2],
) -> (Vec<VisualTile>, GridGeometry, &'static [Network]) {
    let n = &NETWORKS[which];
    let mut sources = Vec::new();
    for y in 0..n.height {
        for x in 0..n.width {
            sources.push(VisualTileSource {
                tileset_id: Arc::from("mart"),
                metatile_id: 3,
                subtile_column: ((n.anchor[0] as usize + x) % 4) as u8,
                subtile_row: ((n.anchor[1] as usize + y) % 4) as u8,
                tile_index: (32 + x + y * n.width) as u16,
            });
        }
    }
    let nn = Network {
        map: n.map,
        floor: n.floor,
        anchor: n.anchor,
        width: n.width,
        height: n.height,
        fingerprint: identity_hash(sources.iter()),
        rows: n.rows,
        parts: n.parts,
        label: n.label,
    };
    let [width, height] = size;
    let cells = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let px = origin[0] + x as i32 - n.anchor[0];
            let py = origin[1] + y as i32 - n.anchor[1];
            let source = if px >= 0 && py >= 0 && px < n.width as i32 && py < n.height as i32 {
                sources[py as usize * n.width + px as usize].clone()
            } else {
                VisualTileSource {
                    tileset_id: Arc::from("mart"),
                    metatile_id: 4,
                    subtile_column: (x % 4) as u8,
                    subtile_row: (y % 4) as u8,
                    tile_index: 1,
                }
            };
            VisualTile {
                animation_frames: None,
                column: x as u32,
                row: y as u32,
                source,
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
        Box::leak(vec![nn].into_boxed_slice()),
    )
}
#[test]
fn department_store_display_native_palette_preserves_geometry_glazing_and_footing() {
    let (cells, g, networks) = fixture(6, [0, 0], [40, 40]);
    let refs = cells.iter().collect::<Vec<_>>();
    let placements = resolve_fixture(networks, &cells, &g, [0, 0], None,
        &vec![false; cells.len()]);
    assert_eq!(placements.len(), 1);
    let p = &placements[0];
    let mut mesh = TerrainMeshData::default();
    mesh.footing_heights = vec![72.0; cells.len()];
    let mut claimed = vec![false; cells.len()];
    assert!(append(&mut mesh, &refs, &g, p, &mut claimed));
    let part = &p.network.parts[0];
    assert_eq!(part.asset, Asset::ClosedU);
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let mut authored = SurfaceMeshData::default();
    part.asset.model().append_fitted(&mut authored,
        [w, w + 64.0, n, n + 64.0], 0.0, 8.0);
    let first = mesh.textured.positions.windows(authored.positions.len())
        .position(|v| v == authored.positions.as_slice()).unwrap();
    assert_eq!(&mesh.textured.normals[first..], authored.normals.as_slice());
    assert_eq!(mesh.scenery.len(),1);
    let glass=&mesh.scenery[0];
    assert_eq!(glass.texture.alpha_mode(),bevy::prelude::AlphaMode::Blend);
    assert!(glass.mesh.colors.iter().all(|c|c[3]>0. && c[3]<1.));
    assert!(glass.mesh.positions.iter().all(|v|v[2]>=n+63.5));
    assert_eq!(mesh.textured.indices.len()+glass.mesh.indices.len(),56*6+authored.indices.len());
    for pixel in [[1,1],[9,2],[0,0],[3,54],[3,49],[0,48]] {
        let uv = g.uv(p.column + pixel[0]/8, p.row + pixel[1]/8);
        assert!(mesh.textured.uvs[first..].contains(&[
            uv.0+(uv.1-uv.0)*((pixel[0]%8)as f32+0.5)/8.,
            uv.2+(uv.3-uv.2)*((pixel[1]%8)as f32+0.5)/8.]));
    }
    for (&normal,&color) in mesh.textured.normals[first..].iter()
        .zip(&mesh.textured.colors[first..]) {
        assert_eq!(color, crate::interior_models::authored_face_color([1.;4],Vec3::from_array(normal)));
    }
    assert!(mesh.solid.positions.is_empty());
    assert!(mesh.textured.cutaway_ranges.is_empty());
    assert!(mesh.solid.cutaway_ranges.is_empty());
    assert!(mesh.footing_heights.iter().all(|&height| height == 72.0));
    assert_eq!(claimed.iter().filter(|&&v| v).count(), 56);
}
#[test]
fn department_store_windows_keep_authored_material_paint_through_production_append() {
    let mut seen = [false; 5];
    for which in 0..NETWORKS.len() {
        let (cells, g, networks) = fixture(which, [0, 0], [40, 40]);
        let refs = cells.iter().collect::<Vec<_>>();
        let placements = resolve_fixture(networks, &cells, &g, [0, 0], None,
            &vec![false; cells.len()]);
        for p in placements {
            let mut mesh = TerrainMeshData::default();
            mesh.footing_heights = vec![72.0; cells.len()];
            let mut claimed = vec![false; cells.len()];
            assert!(append(&mut mesh, &refs, &g, &p, &mut claimed));
            for part in p.network.parts {
                if !matches!(part.asset, Asset::Window1 | Asset::Window2 |
                    Asset::Window4 | Asset::Window8 | Asset::Window10) { continue; }
                seen[part.asset as usize] = true;
                let (w, _, n, _) = g.bounds(p.column + part.x, p.row + part.y);
                let e = w + part.width as f32 * g.tile_width;
                let s = n + part.depth as f32 * g.tile_height;
                let mut authored = SurfaceMeshData::default();
                part.asset.model().append_fitted(&mut authored,
                    [w, e, s - 3.0, s], 0.0, 16.0);
                let first = mesh.solid.positions.windows(authored.positions.len())
                    .position(|v| v == authored.positions.as_slice())
                    .expect("complete authored window geometry must reach production mesh");
                assert_eq!(&mesh.solid.colors[first..first + authored.colors.len()],
                    authored.colors.as_slice(), "glass and teal shadows must not be recolored by RGB tests");
            }
            assert!(mesh.footing_heights.iter().all(|&v| v == 72.0));
        }
    }
    assert!(seen.iter().all(|&v| v), "every authored bay count must be exercised");
}
#[test]
fn celadon_sixth_floor_callback_wall_is_exact_and_map_scoped() {
    let (mut tiles,g,networks)=fixture(30,[0,0],[34,6]);
    let refs=tiles.iter().collect::<Vec<_>>();
    let wall=resolve_networks(networks,6,"CeladonDeptStore6F",&refs,&g,[0,0],None,&vec![false;tiles.len()]);
    assert_eq!(wall.len(),1);
    assert_eq!(wall[0].indices(g.width).count(),40);
    assert!(resolve_networks(networks,6,"GoldenrodDeptStore6F",&refs,&g,[0,0],None,&vec![false;tiles.len()]).is_empty());
    for y in 0..4 {for x in 0..32 {
        let i=y*g.width+x;
        let old=tiles[i].source.tile_index;
        tiles[i].source.tile_index=0xffff;
        assert!(resolve_networks(networks,6,"CeladonDeptStore6F",&tiles.iter().collect::<Vec<_>>(),&g,[0,0],None,&vec![false;tiles.len()]).is_empty());
        tiles[i].source.tile_index=old;
    }}
}
fn resolve_fixture(
    n: &'static [Network],
    cells: &[VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    doc: Option<&Document>,
    reserved: &[bool],
) -> Vec<Placement> {
    resolve_networks(
        n,
        n[0].floor,
        n[0].map.unwrap_or("GenericStoreFixture"),
        &cells.iter().collect::<Vec<_>>(),
        g,
        origin,
        doc,
        reserved,
    )
}
#[test]
fn both_store_counter_adapters_use_their_owned_source_palette() {
    for which in 0..NETWORKS.len() {
        if !NETWORKS[which].parts.iter().any(|part|counter_part(part.asset)) {continue;}
        let (cells,g,n)=fixture(which,[0,0],[40,40]);
        let refs=cells.iter().collect::<Vec<_>>();
        for prefix in ["Goldenrod","Celadon"] {
            let map=format!("{prefix}DeptStore{}F",n[0].floor);
            let mut claimed=vec![false;cells.len()];
            let placements=resolve_networks(n,n[0].floor,&map,&refs,&g,[0,0],None,&claimed);
            assert_eq!(placements.len(),1);
            let p=&placements[0];assert!(p.native_store);
            let sample=p.indices(g.width).find_map(|i|
                super::modeled_interiors::mart_counter_swatches(&g,i,cells[i].source.tile_index)).unwrap();
            let mut mesh=TerrainMeshData{footing_heights:vec![2.75;cells.len()],..Default::default()};
            assert!(append(&mut mesh,&refs,&g,p,&mut claimed));
            assert_eq!(claimed.iter().filter(|&&v|v).count(),p.indices(g.width).count());
            assert_eq!(mesh.footing_heights,vec![2.75;cells.len()]);
            assert_eq!(mesh.depth_coverage_textured_ranges.len(),p.network.parts.len());
            for uv in sample {
                assert!(mesh.textured.indices.iter().any(|&i|mesh.textured.uvs[i as usize]==uv),
                    "{map}: native roles must use this cabinet's source swatches");
            }
        }
    }
}
#[test]
fn all_exact_drawings_reject_every_changed_source_and_missing_guard() {
    for which in 0..NETWORKS.len() {
        let (cells, g, n) = fixture(which, [0, 0], [33, 17]);
        let x = n[0].anchor[0] as usize;
        let y = n[0].anchor[1] as usize;
        assert!(complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            x,
            y,
            &n[0]
        ));
        for dy in 0..n[0].height {
            for dx in 0..n[0].width {
                for field in 0..5 {
                    let mut changed = cells.clone();
                    let s = &mut changed[(y + dy) * g.width + x + dx].source;
                    match field {
                        0 => s.tileset_id = Arc::from("custom_mart"),
                        1 => s.metatile_id ^= 1,
                        2 => s.subtile_column ^= 1,
                        3 => s.subtile_row ^= 1,
                        _ => s.tile_index ^= 1,
                    }
                    assert!(
                        !complete(&changed.iter().collect::<Vec<_>>(), &g, [0, 0], x, y, &n[0]),
                        "network {which}, cell {dx},{dy}, field {field}"
                    );
                }
            }
        }
        assert!(!complete(
            &cells[..cells.len() - 1].iter().collect::<Vec<_>>(),
            &g,
            [0, 0],
            x,
            y,
            &n[0]
        ));
        assert!(!complete(
            &cells.iter().collect::<Vec<_>>(),
            &g,
            [1, 0],
            x,
            y,
            &n[0]
        ));
    }
    for invalid in [
        "GoldenrodDeptStoreB1F",
        "GoldenrodDeptStoreRoof",
        "GoldenrodDeptStoreElevator",
        "CeladonDeptStore7F",
        "GoldenrodDeptStore1FBeta",
        "Route1",
    ] {
        assert_eq!(floor(invalid), None);
    }
}

#[test]
fn store_flights_recess_downstairs_and_keep_rotated_bounds_normals_and_support() {
    for which in 7..NETWORKS.len() {
        if NETWORKS[which].label != "mart/department_store_stairs" {
            continue;
        }
        let (cells, mut g, networks) = fixture(which, [0, 0], [33, 17]);
        // Deliberately unequal axes detect fitting after rotation mistakes.
        g.tile_width = 10.;
        g.tile_height = 6.;
        let p = resolve_fixture(
            networks,
            &cells,
            &g,
            [0, 0],
            None,
            &vec![false; cells.len()],
        );
        assert_eq!(p.len(), 1);
        let mut mesh = TerrainMeshData::default();
        mesh.authored_cells = vec![None; cells.len()];
        mesh.footing_heights = vec![0.; cells.len()];
        let mut claimed = vec![false; cells.len()];
        assert!(append(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        let down = networks[0].parts[0].asset == Asset::StairDown;
        let base = if down { -12. } else { 0. };
        assert!(mesh.textured.positions.iter().all(|p| p[1] == base));
        assert!(mesh
            .solid
            .positions
            .iter()
            .all(|v| v[0] >= p[0].column as f32 * 10. - 0.001
                && v[0] <= (p[0].column + 2) as f32 * 10. + 0.001
                && v[2] >= -0.001
                && v[2] <= 12.001));
        assert!(mesh
            .solid
            .normals
            .iter()
            .all(|n| (n.iter().map(|v| v * v).sum::<f32>() - 1.).abs() < 0.001));
        assert_eq!(mesh.footing_heights, vec![0.; cells.len()]);
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 4);
        assert_eq!(
            mesh.authored_cells
                .iter()
                .filter(|v| **v == Some("mart/department_store_stairs"))
                .count(),
            4
        );
    }
}
#[test]
fn lift_fronts_keep_a_clear_aperture_and_original_support() {
    for which in 18..NETWORKS.len() {
        if NETWORKS[which].label != "mart/department_store_lift_front" {
            continue;
        }
        let (cells, mut g, n) = fixture(which, [0, 0], [33, 17]);
        g.tile_width = 10.;
        g.tile_height = 6.;
        let p = resolve_fixture(n, &cells, &g, [0, 0], None, &vec![false; cells.len()]);
        assert_eq!(p.len(), 1);
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; cells.len()],
            authored_cells: vec![None; cells.len()],
            ..Default::default()
        };
        let mut claimed = vec![false; cells.len()];
        assert!(append(
            &mut mesh,
            &cells.iter().collect::<Vec<_>>(),
            &g,
            &p[0],
            &mut claimed
        ));
        assert_eq!(claimed.iter().filter(|&&v| v).count(), 8);
        assert_eq!(mesh.footing_heights, vec![0.; cells.len()]);
        let center = (p[0].column + 1) as f32 * g.tile_width;
        let height = 7.5;
        for triangle in mesh.solid.indices.chunks_exact(3) {
            let points = triangle
                .iter()
                .map(|&i| mesh.solid.positions[i as usize])
                .collect::<Vec<_>>();
            let lo = |axis| points.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min);
            let hi = |axis| {
                points
                    .iter()
                    .map(|p| p[axis])
                    .fold(f32::NEG_INFINITY, f32::max)
            };
            assert!(
                !(lo(0) < center + 1. && hi(0) > center - 1. && lo(1) < height && hi(1) > height),
                "lift frame crosses the entrance aperture"
            );
        }
        // The adjacent call-panel housing and final floor row have other owners.
        assert!(!claimed[p[0].row * g.width + p[0].column + 2]);
        assert!(!claimed[(p[0].row + 3) * g.width + p[0].column]);
        assert!(mesh.solid.positions.iter().any(|p| p[1] > 10.));
    }
}
#[test]
fn native_padding_scrolling_and_crops_preserve_exact_sparse_ownership() {
    for which in 0..NETWORKS.len() {
        let n = &NETWORKS[which];
        let expected: usize = n.rows.iter().map(|r| r.count_ones() as usize).sum();
        for (origin, size) in [
            ([0, 0], [33, 17]),
            ([-32, -32], [84, 82]),
            ([-25, -27], [84, 82]),
        ] {
            let (cells, g, n) = fixture(which, origin, size);
            let reserved = vec![false; cells.len()];
            let p = resolve_fixture(n, &cells, &g, origin, None, &reserved);
            assert_eq!(p.len(), 1);
            let actual = p[0].indices(g.width).collect::<Vec<_>>();
            assert_eq!(actual.len(), expected);
            for i in actual {
                let wx = origin[0] + (i % g.width) as i32;
                let wy = origin[1] + (i / g.width) as i32;
                assert!(
                    n[0].rows[(wy - n[0].anchor[1]) as usize] & (1 << (wx - n[0].anchor[0])) != 0
                );
            }
        }
        // Cropping even an unowned source guard row rejects the entire object.
        for (origin, size) in [
            (n.anchor, [n.width, n.height - 1]),
            ([n.anchor[0] + 1, n.anchor[1]], [n.width, n.height]),
        ] {
            let (cells, g, nn) = fixture(which, origin, size);
            assert!(
                resolve_fixture(nn, &cells, &g, origin, None, &vec![false; cells.len()]).is_empty()
            );
        }
    }
}
#[test]
fn every_wall_avoids_lift_doors_stairs_and_front_walkway() {
    for n in &NETWORKS[..6] {
        for y in 0..4 {
            for x in 4..6 {
                assert_eq!(n.rows[y] & (1 << x), 0);
            }
        }
        assert_eq!(n.rows[2], 0);
        assert_eq!(n.rows[3], 0);
        for x in 30..32 {
            assert_eq!(n.rows[0] & (1 << x), 0);
            assert_eq!(n.rows[1] & (1 << x), 0);
        }
        let stairs = if n.floor == 1 {
            None
        } else if n.floor == 6 {
            Some(26)
        } else {
            Some(24)
        };
        if let Some(x) = stairs {
            for y in 0..2 {
                assert_eq!(n.rows[y] & (3 << x), 0);
            }
        }
    }
    let u = &NETWORKS[6];
    assert_eq!(u.rows.iter().map(|r| r.count_ones()).sum::<u32>(), 56);
    for y in 4..8 {
        assert_eq!(u.rows[y], 255, "the entire lower drawing is closed WALL");
    }
    for y in 2..4 {
        assert_eq!(
            u.rows[y] & 0b00111100,
            0,
            "the upper native floor inset stays open"
        );
    }
}
#[test]
fn reserved_cells_custom_profiles_and_missing_ground_keep_whole_object_fallback() {
    let (cells, g, n) = fixture(4, [0, 0], [33, 17]);
    let mut reserved = vec![false; cells.len()];
    reserved[0] = true;
    assert!(resolve_fixture(n, &cells, &g, [0, 0], None, &reserved).is_empty());
    reserved[0] = false;
    let doc:Document=serde_json::from_value(serde_json::json!({"objects":[{"name":"Custom wall panel","tileset":"mart","metatile":3,"origin":[0,0],"tiles":[[32]],"ground":1,"top_pixels":12,"depth_pixels":8}]})).unwrap();
    assert!(resolve_fixture(n, &cells, &g, [0, 0], Some(&doc), &reserved).is_empty());
    let mut missing = cells.clone();
    for c in &mut missing {
        if c.source.metatile_id == 4 {
            c.source.tile_index = 2;
        }
    }
    assert!(resolve_fixture(n, &missing, &g, [0, 0], None, &reserved).is_empty());
}
#[test]
fn append_is_atomic_preserves_footing_owned_bounds_and_display_reveal() {
    for which in 0..NETWORKS.len() {
        let origin = [-25, -27];
        let (cells, mut g, n) = fixture(which, origin, [84, 82]);
        g.tile_width = 12.;
        g.tile_height = 6.;
        let refs = cells.iter().collect::<Vec<_>>();
        let mut claims = vec![false; cells.len()];
        let p = resolve_fixture(n, &cells, &g, origin, None, &claims);
        let mut mesh = TerrainMeshData::default();
        mesh.footing_heights = vec![2.75; cells.len()];
        mesh.authored_cells = vec![None; cells.len()];
        let first = p[0].indices(g.width).next().unwrap();
        claims[first] = true;
        let before = claims.clone();
        assert!(!append(&mut mesh, &refs, &g, &p[0], &mut claims));
        assert_eq!(claims, before);
        assert!(mesh.solid.positions.is_empty());
        assert!(mesh.textured.positions.is_empty());
        claims[first] = false;
        assert!(append(&mut mesh, &refs, &g, &p[0], &mut claims));
        assert!(mesh.footing_heights.iter().all(|&h| h == 2.75));
        if matches!(
            p[0].network.label,
            "mart/department_store_counter_end"
                | "mart/department_store_service_loop"
        ) {
            let bays=p[0].network.parts.iter().filter(|part|matches!(part.asset,
                Asset::CounterPlain|Asset::CounterRegister|Asset::CounterNorth|Asset::CounterWest)).count();
            assert_eq!(mesh.solid.cutaway_ranges.len(),bays);
            assert_eq!(mesh.depth_coverage_solid_ranges,mesh.solid.cutaway_ranges);
            let mut end=0;
            for range in &mesh.solid.cutaway_ranges {
                assert_eq!(range.start,end,"each independently revealed bay owns its geometry");
                assert!(range.end>range.start);
                end=range.end;
            }
            assert_eq!(end,mesh.solid.positions.len());
            for (part,range) in p[0].network.parts.iter().filter(|part|counter_part(part.asset))
                .zip(&mesh.solid.cutaway_ranges) {
                let (w,_,n,_)=g.bounds(p[0].column+part.x,p[0].row+part.y);
                let e=w+part.width as f32*g.tile_width;
                let s=n+part.depth as f32*g.tile_height;
                for (side,(axis,plane,normal)) in [(0,w,-1.),(0,e,1.),(2,n,-1.),(2,s,1.)].into_iter().enumerate() {
                    let faces=mesh.solid.indices.chunks_exact(3).filter(|tri|tri.iter().all(|&i|
                        range.contains(&(i as usize))
                            &&(mesh.solid.positions[i as usize][axis]-plane).abs()<0.0001
                            &&mesh.solid.normals[i as usize][axis]*normal>0.999)).count();
                    if counter_joins(part,p[0].network.parts)[side] {
                        assert_eq!(faces,0,"buried bay ends must not show during reveal");
                        assert!(mesh.solid.indices.iter().any(|&i| {
                            let i=i as usize;let v=mesh.solid.positions[i];
                            range.contains(&i)&&(v[axis]-plane).abs()<0.0001
                                &&v[1]>0.001&&v[1]<=7.*g.tile_height/8.+0.0001
                                &&mesh.solid.normals[i][if axis==0 {2}else{0}].abs()>0.999
                        }),"body must meet the shared edge below the overhanging cap");
                    }else {assert!(faces>0,"exposed bay ends must remain closed");}
                }
            }
            // Synthetic fixture uses solid counters. Source-textured floor
            // backing must never enter a bay's reveal range.
            assert!(mesh.textured.cutaway_ranges.is_empty());
            assert!(mesh.cutaway_links.is_empty());
        } else if p[0].network.label=="mart/department_store_closed_u" {
            assert!(mesh.solid.cutaway_ranges.is_empty());
            assert!(mesh.textured.cutaway_ranges.is_empty());
            assert_eq!(mesh.scenery.len(),1);
            let glass=&mesh.scenery[0];
            assert_eq!(glass.texture.alpha_mode(),bevy::prelude::AlphaMode::Blend);
            let front=g.origin_z+(p[0].row as f32+63.5/8.)*g.tile_height;
            assert!(glass.mesh.positions.iter().all(|v|v[2]>=front-0.001));
            assert!(mesh.cutaway_links.is_empty());
        } else {
            assert_eq!(
                mesh.solid.cutaway_ranges,
                vec![0..mesh.solid.positions.len()]
            );
        }
        let (w, _, north, _) = g.bounds(p[0].column, p[0].row);
        let stair = matches!(n[0].parts[0].asset, Asset::StairUp | Asset::StairDown);
        let base = if n[0].parts[0].asset == Asset::StairDown {
            -16.
        } else {
            0.
        };
        let top = base + if stair { 22.4 } else { 16. };
        assert!(mesh.solid.positions.iter().all(|&[x, y, z]| x >= w
            && x <= w + n[0].width as f32 * g.tile_width
            && z >= north
            && z <= north + n[0].height as f32 * g.tile_height
            && y >= base * g.tile_height / 8. - 0.001
            && y <= top * g.tile_height / 8. + 0.001));
        assert!(mesh
            .solid
            .normals
            .iter()
            .all(|n| (Vec3::from_array(*n).length_squared() - 1.).abs() < 0.0001));
        assert_eq!(
            claims.iter().filter(|&&v| v).count(),
            n[0].rows
                .iter()
                .map(|r| r.count_ones() as usize)
                .sum::<usize>()
        );
        if which == 28 || which == 29 {
            for &[x, _, z] in &mesh.solid.positions {
                let x = (x - w) / g.tile_width;
                let z = (z - north) / g.tile_height;
                assert!(
                    !(x > 2.0001 && z > 2.0001 && z < if which == 28 { 5.9999 } else { 3.9999 }),
                    "service counter must preserve the clerk floor and right shelves"
                );
            }
        }
        if which == 6 {
            assert!(!mesh.textured.positions.is_empty());
            assert!(mesh.textured.positions.iter().all(|&[x,y,z]|
                x >= w && x <= w+n[0].width as f32*g.tile_width
                && z >= north && z <= north+n[0].height as f32*g.tile_height
                && y >= -0.001 && y <= 8.*g.tile_height/8.+0.001));
            assert!(mesh.textured.normals.iter().all(|normal|
                (Vec3::from_array(*normal).length_squared()-1.).abs()<0.0001));
            for &[x, _, z] in mesh.solid.positions.iter().chain(&mesh.textured.positions) {
                let x = (x - w) / g.tile_width;
                let z = (z - north) / g.tile_height;
                assert!(
                    !(x > 2.0001 && x < 5.9999 && z > 2.0001 && z < 3.9999),
                    "geometry must not enter the genuine floor inset"
                );
            }
        }
    }
}
#[test]
fn live_directory_and_lift_button_use_original_uvs_and_outward_faces() {
    let (cells, g, n) = fixture(0, [0, 0], [33, 17]);
    let mut claimed = vec![false; cells.len()];
    let p = resolve_fixture(n, &cells, &g, [0, 0], None, &claimed);
    let mut mesh = TerrainMeshData::default();
    assert!(append(
        &mut mesh,
        &cells.iter().collect::<Vec<_>>(),
        &g,
        &p[0],
        &mut claimed
    ));
    let base = p[0].indices(g.width).count() * 4;
    assert_eq!(mesh.textured.positions.len(), base + 20);
    let expected = [
        g.uv(6, 0),
        g.uv(28, 0),
        g.uv(29, 0),
        g.uv(28, 1),
        g.uv(29, 1),
    ];
    for (i, (u0, u1, v0, v1)) in expected.into_iter().enumerate() {
        let k = base + i * 4;
        assert_eq!(
            &mesh.textured.uvs[k..k + 4],
            &[[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
        );
        let a = Vec3::from_array(mesh.textured.positions[k]);
        let b = Vec3::from_array(mesh.textured.positions[k + 1]);
        let c = Vec3::from_array(mesh.textured.positions[k + 2]);
        assert!((b - a).cross(c - a).z > 0.);
    }
}
