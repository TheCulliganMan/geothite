use crystal_mapgen::{
    BoundingBox, Coordinate, Feature, FeatureKind, MapCell, MapSource, audit_grid, generate_grid,
};
const GRID_SIZE: u16 = 64;

#[test]
fn ordinary_buildings_do_not_erase_mapped_commercial_districts() {
    let mut source = realistic_source();
    source
        .features
        .retain(|f| matches!(f.kind, FeatureKind::Building | FeatureKind::Street));
    let mut cover = area(FeatureKind::Park, None, 0.0, 0.0, 1.0, 1.0);
    cover
        .details
        .tags
        .insert("landuse".into(), "commercial".into());
    source.features.push(cover);
    let grid = generate_grid(source, GRID_SIZE, GRID_SIZE).unwrap();
    let scene = grid.scene.as_ref().unwrap();
    assert!(!scene.structures.is_empty());
    for structure in &scene.structures {
        let index = usize::from(structure.origin.1) * usize::from(grid.width)
            + usize::from(structure.origin.0);
        assert_eq!(scene.districts[index], crystal_mapgen::VisualFamily::Urban);
    }
}

#[test]
fn realistic_neighborhood_is_deterministic_and_has_accessible_complete_structures() {
    let first = generate_grid(realistic_source(), GRID_SIZE, GRID_SIZE).unwrap();
    let second = generate_grid(realistic_source(), GRID_SIZE, GRID_SIZE).unwrap();
    assert_eq!(first, second);
    let audit = audit_grid(&first);
    assert!(audit.passed, "{:?}", audit.errors);
    let scene = first.scene.as_ref().unwrap();
    assert!(!scene.structures.is_empty());
    for structure in &scene.structures {
        let (w, h) = structure.recipe.dimensions();
        for y in 0..h {
            for x in 0..w {
                if let Some(block) = structure.recipe.block(x, y) {
                    let index = usize::from(structure.origin.1 + y) * usize::from(first.width)
                        + usize::from(structure.origin.0 + x);
                    assert_eq!(first.crystal_blocks()[index], block);
                }
            }
        }
    }
}

#[test]
fn sparse_source_does_not_invent_a_town_or_require_a_scenery_checklist() {
    let mut source = realistic_source();
    source.features.retain(|f| f.kind == FeatureKind::Street);
    let grid = generate_grid(source, GRID_SIZE, GRID_SIZE).unwrap();
    assert!(grid.scene.as_ref().unwrap().structures.is_empty());
    assert!(!grid.cells.contains(&MapCell::Fountain));
    assert!(audit_grid(&grid).passed, "{:?}", audit_grid(&grid).errors);
}

#[test]
fn mapped_woodland_and_meadow_have_distinct_composition() {
    let mut source = realistic_source();
    source.features.retain(|f| f.kind == FeatureKind::Street);
    let mut cover = area(FeatureKind::Park, Some("Land cover"), 0.0, 0.0, 1.0, 1.0);
    cover.points = vec![
        Coordinate {
            lat: source.bounds.south,
            lon: source.bounds.west,
        },
        Coordinate {
            lat: source.bounds.north,
            lon: source.bounds.west,
        },
        Coordinate {
            lat: source.bounds.north,
            lon: source.bounds.east,
        },
        Coordinate {
            lat: source.bounds.south,
            lon: source.bounds.east,
        },
    ];
    cover.details.tags.insert("natural".into(), "wood".into());
    source.features.push(cover);
    let forest = generate_grid(source.clone(), GRID_SIZE, GRID_SIZE).unwrap();
    source
        .features
        .last_mut()
        .unwrap()
        .details
        .tags
        .insert("natural".into(), "grassland".into());
    let meadow = generate_grid(source, GRID_SIZE, GRID_SIZE).unwrap();
    assert!(
        forest.cells.iter().filter(|c| **c == MapCell::Tree).count()
            > meadow.cells.iter().filter(|c| **c == MapCell::Tree).count()
    );
    assert_ne!(forest.crystal_blocks(), meadow.crystal_blocks());
}

fn realistic_source() -> MapSource {
    let mut features = vec![
        area(
            FeatureKind::Water,
            Some("North Lake"),
            0.64,
            0.78,
            0.97,
            0.98,
        ),
        area(
            FeatureKind::Park,
            Some("West Preserve"),
            0.04,
            0.04,
            0.27,
            0.27,
        ),
        area(
            FeatureKind::Park,
            Some("Central Commons"),
            0.37,
            0.04,
            0.61,
            0.27,
        ),
        area(
            FeatureKind::Park,
            Some("East Meadow"),
            0.72,
            0.04,
            0.96,
            0.27,
        ),
        area(
            FeatureKind::Pitch,
            Some("Neighborhood Field"),
            0.08,
            0.68,
            0.18,
            0.75,
        ),
        line(
            FeatureKind::MajorRoad,
            Some("Main Street"),
            &[(0.01, 0.50), (0.35, 0.51), (0.67, 0.49), (0.99, 0.50)],
        ),
        line(
            FeatureKind::Street,
            Some("Lake Avenue"),
            &[(0.08, 0.47), (0.48, 0.48), (0.92, 0.47)],
        ),
        line(
            FeatureKind::Street,
            Some("Park Avenue"),
            &[(0.12, 0.54), (0.52, 0.53), (0.89, 0.54)],
        ),
    ];

    // A dense real-world block is intentionally compressed by the generator
    // into a small number of readable Crystal houses along the main frontage.
    for (index, lon) in [0.05, 0.15, 0.25, 0.35, 0.45, 0.55, 0.65, 0.75, 0.85, 0.95]
        .into_iter()
        .enumerate()
    {
        features.push(area(
            FeatureKind::Building,
            Some(&format!("Parcel {index}")),
            lon - 0.018,
            0.55,
            lon + 0.018,
            0.58,
        ));
    }

    MapSource {
        schema_version: 2,
        center: Coordinate { lat: 0.5, lon: 0.5 },
        bounds: BoundingBox {
            south: 0.0,
            west: 0.0,
            north: 1.0,
            east: 1.0,
        },
        attribution: "synthetic OpenStreetMap-style fixture".to_string(),
        features,
        h3: None,
    }
}

fn area(
    kind: FeatureKind,
    name: Option<&str>,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
) -> Feature {
    Feature {
        details: Default::default(),
        kind,
        name: name.map(str::to_string),
        area: true,
        bridge: false,
        points: vec![
            Coordinate {
                lat: south,
                lon: west,
            },
            Coordinate {
                lat: north,
                lon: west,
            },
            Coordinate {
                lat: north,
                lon: east,
            },
            Coordinate {
                lat: south,
                lon: east,
            },
            Coordinate {
                lat: south,
                lon: west,
            },
        ],
    }
}

fn line(kind: FeatureKind, name: Option<&str>, points: &[(f64, f64)]) -> Feature {
    Feature {
        details: Default::default(),
        kind,
        name: name.map(str::to_string),
        area: false,
        bridge: false,
        points: points
            .iter()
            .map(|&(lon, lat)| Coordinate { lat, lon })
            .collect(),
    }
}

#[test]
fn empty_source_has_safe_start_and_services_without_fabricated_osm_buildings() {
    let mut source = realistic_source();
    source.features.clear();
    let grid = generate_grid(source, 64, 64).unwrap();
    assert!(grid.scene.as_ref().unwrap().structures.is_empty());
    assert!(audit_grid(&grid).passed, "{:?}", audit_grid(&grid).errors);
}

#[test]
fn lake_island_survives_composition_and_optional_decoration() {
    let mut source = realistic_source();
    source.features.retain(|f| f.kind == FeatureKind::Street);
    let mut water = area(
        FeatureKind::Water,
        Some("Lake with island"),
        0.62,
        0.60,
        0.95,
        0.95,
    );
    water
        .details
        .inner_rings
        .push(area(FeatureKind::Park, None, 0.71, 0.70, 0.85, 0.84).points);
    source.features.push(water);
    let grid = generate_grid(source, 64, 64).unwrap();
    let scene = grid.scene.as_ref().unwrap();
    assert!(scene.islands.iter().any(|v| *v));
    for (i, island) in scene.islands.iter().enumerate() {
        if *island {
            assert!(!matches!(
                grid.cells[i],
                MapCell::Water
                    | MapCell::WaterAccessEast
                    | MapCell::WaterAccessWest
                    | MapCell::WaterAccessSouth
            ));
        }
    }
}

#[test]
fn source_order_does_not_change_scene_or_tiles() {
    let mut source = realistic_source();
    let first = generate_grid(source.clone(), 64, 64).unwrap();
    source.features.reverse();
    let reordered = generate_grid(source, 64, 64).unwrap();
    assert_eq!(first, reordered);
}

#[test]
fn building_courtyard_stays_open_during_structure_siting() {
    let mut source = realistic_source();
    source.features.retain(|f| f.kind == FeatureKind::Street);
    let mut building = area(FeatureKind::Building, Some("Courtyard"), 0.1, 0.2, 0.4, 0.7);
    building
        .details
        .inner_rings
        .push(area(FeatureKind::Park, None, 0.18, 0.3, 0.32, 0.6).points);
    source.features.push(building);
    let grid = generate_grid(source, 64, 64).unwrap();
    let scene = grid.scene.as_ref().unwrap();
    assert!(scene.courtyards.iter().any(|v| *v));
    for (i, hole) in scene.courtyards.iter().enumerate() {
        if *hole {
            assert!(!matches!(
                grid.cells[i],
                MapCell::Building | MapCell::Tree | MapCell::Boulder
            ));
        }
    }
}
