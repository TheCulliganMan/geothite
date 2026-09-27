use crystal_mapgen::*;

fn region() -> (Vec<GeneratedGrid>, H3BatchConnections) {
    let origin = Coordinate {
        lat: 44.9475196,
        lon: -93.3253477,
    };
    let manifest = plan_h3_batch(origin, 6, 7).unwrap();
    let mut features = Vec::new();
    for cell in &manifest.cells {
        for portal in &cell.plan.portals {
            features.push(Feature {
                details: FeatureDetails::default(),
                kind: FeatureKind::Road,
                name: Some("Connecting road".into()),
                area: false,
                bridge: false,
                points: vec![cell.plan.center, portal.midpoint],
            });
        }
        for dx in [-0.0007, 0.0007] {
            for dy in [-0.0007, 0.0007] {
                features.push(Feature {
                    details: FeatureDetails::default(),
                    kind: FeatureKind::Building,
                    name: None,
                    area: true,
                    bridge: false,
                    points: vec![Coordinate {
                        lat: cell.plan.center.lat + dy,
                        lon: cell.plan.center.lon + dx,
                    }],
                });
            }
        }
    }
    let raw = MapSource {
        schema_version: SOURCE_SCHEMA_VERSION,
        center: origin,
        bounds: BoundingBox {
            south: 44.8,
            west: -93.5,
            north: 45.1,
            east: -93.1,
        },
        attribution: "Synthetic topology fixture".into(),
        features,
        h3: None,
    };
    let mut sources = manifest
        .cells
        .iter()
        .map(|c| prepare_h3_source(raw.clone(), c.plan.clone()).unwrap())
        .collect::<Vec<_>>();
    let contracts = sources
        .iter()
        .map(|s| build_h3_seam_contract(s.h3.as_ref().unwrap(), s, 96, 96).unwrap())
        .collect::<Vec<_>>();
    let regional = plan_h3_region(&manifest, &sources, &contracts).unwrap();
    let connections = build_h3_regional_connections(&manifest, &regional, 96, 96).unwrap();
    let mut grids = Vec::new();
    for source in &mut sources {
        let id = source.h3.as_ref().unwrap().cell.clone();
        attach_h3_regional_plan(source, regional.cell(&id).unwrap().clone(), 96, 96).unwrap();
        grids.push(generate_grid(source.clone(), 96, 96).unwrap());
    }
    finalize_h3_batch_grid_seams(&mut grids).unwrap();
    for grid in &grids {
        let audit = audit_grid(grid);
        assert!(audit.passed, "{:?}", audit.errors);
    }
    let reports = grids
        .iter()
        .map(|g| inspect_h3_regional_grid(g).unwrap())
        .collect::<Vec<_>>();
    let audit = audit_h3_regional_batch(&regional, &reports);
    assert!(audit.passed, "{:?}", audit.errors);
    let profiles = grids
        .iter()
        .map(|g| build_h3_grid_seam_profile(g).unwrap())
        .collect::<Vec<_>>();
    let audit = audit_h3_grid_seams(&profiles);
    assert!(audit.passed, "{:?}", audit.errors);
    (grids, connections)
}

#[test]
fn seven_cells_keep_allocated_services_and_reciprocal_crossings() {
    region();
}

#[test]
#[ignore = "requires CRYSTAL_MAPGEN_TEST_PACK; run with --ignored"]
fn seven_cell_pack_preserves_base_content_and_all_links() {
    let (grids, connections) = region();
    let base = std::env::var_os("CRYSTAL_MAPGEN_TEST_PACK").expect("external pack");
    let output = tempfile::tempdir().unwrap();
    let pack = output.path().join("region.crystalpack");
    let start = grids[0].source.h3.as_ref().unwrap().cell.clone();
    let built = build_region_modpack(
        &grids,
        &connections,
        RegionBuildOptions {
            base_pack: std::path::Path::new(&base),
            output_pack: &pack,
            start_cell: &start,
            registry: WorldMapRegistry::default(),
        },
    )
    .unwrap();
    let loaded = crystal_assets::read_verified_compiled_game_pack(&pack).unwrap();
    assert!(loaded.data().maps.contains_key(&built.start_map));
    for allocation in built.registry.maps.values() {
        let map = &loaded.data().maps[&allocation.map_name];
        for warp in &map.events.warps {
            let target = &loaded.data().runtime_map_metadata[&warp.target_map_constant];
            assert!(
                loaded.data().maps[&target.name]
                    .events
                    .warps
                    .iter()
                    .any(|w| i16::try_from(w.index).unwrap() == warp.target_warp_id)
            );
        }
    }
}
