#[path = "../src/bin/pack_core/source_map_geometry.rs"]
mod source_map_geometry;

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "geothite-map-geometry-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        for name in ["constants", "data/maps", "maps"] {
            std::fs::create_dir_all(root.join(name)).unwrap();
        }
        let fixture = Self(root);
        fixture.write(
            "constants/map_constants.asm",
            "newgroup TEST\nmap_const ALPHA, 2, 1\nmap_const BETA, 2, 1\n",
        );
        fixture.write("data/maps/maps.asm","map Alpha, TILESET_FOREST, CAVE, LANDMARK_TEST, MUSIC_TEST, FALSE, PALETTE_AUTO, FISHGROUP_TEST\nmap Beta, TILESET_FOREST, CAVE, LANDMARK_TEST, MUSIC_TEST, TRUE, PALETTE_DAY, FISHGROUP_TEST\n");
        fixture.write("data/maps/attributes.asm","map_attributes Alpha, ALPHA, $05\nconnection east, Beta, BETA, -2\nmap_attributes Beta, BETA, $06\nconnection west, Alpha, ALPHA, 2\n");
        fixture.write(
            "data/maps/blocks.asm",
            "Alpha_Blocks:\nBeta_Blocks:\nINCBIN \"maps/shared.blk\"\n",
        );
        std::fs::write(fixture.0.join("maps/shared.blk"), [1, 7]).unwrap();
        fixture
    }
    fn write(&self, name: &str, text: &str) {
        std::fs::write(self.0.join(name), text).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let resolved = self.0.canonicalize().unwrap();
        assert_eq!(
            resolved.parent(),
            Some(std::env::temp_dir().canonicalize().unwrap().as_path())
        );
        assert!(
            resolved
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("geothite-map-geometry-")
        );
        std::fs::remove_dir_all(resolved).unwrap();
    }
}

#[test]
fn shared_layout_aliases_and_source_metadata_are_preserved() {
    let fixture = Fixture::new();
    let maps = source_map_geometry::read(&fixture.0).unwrap();
    assert_eq!(maps["Alpha"].blocks, vec![1, 7]);
    assert_eq!(maps["Beta"].blocks, vec![1, 7]);
    assert_eq!(maps["Alpha"].attributes.connections[0].offset, -2);
    assert_eq!(
        maps["Alpha"].attributes.connection_flags.as_deref(),
        Some("EAST")
    );
    assert!(maps["Beta"].attributes.phone_flag);
    assert_eq!(maps["Alpha"].attributes.time_of_day, None);
    assert_eq!(maps["Alpha"].attributes.phone_service, 0);
    assert_eq!(maps["Beta"].attributes.time_of_day.as_deref(), Some("day"));
    assert_eq!(maps["Beta"].attributes.phone_service, 0x11);
    assert_eq!(maps["Alpha"].attributes.tileset_name, "forest");
}

#[test]
fn fixed_map_palettes_preserve_source_nibbles_and_clock_override() {
    for (palette, expected, nibble) in [
        ("PALETTE_AUTO", None, 0),
        ("PALETTE_DAY", Some("day"), 1),
        ("PALETTE_NITE", Some("nite"), 2),
        ("PALETTE_MORN", Some("morn"), 3),
        ("PALETTE_DARK", Some("dark"), 4),
    ] {
        let fixture = Fixture::new();
        fixture.write("data/maps/maps.asm", &format!("map Alpha, TILESET_FOREST, CAVE, LANDMARK_TEST, MUSIC_TEST, FALSE, {palette}, FISHGROUP_TEST\nmap Beta, TILESET_FOREST, CAVE, LANDMARK_TEST, MUSIC_TEST, TRUE, {palette}, FISHGROUP_TEST\n"));
        let maps = source_map_geometry::read(&fixture.0).unwrap();
        assert_eq!(maps["Alpha"].attributes.time_of_day.as_deref(), expected);
        assert_eq!(maps["Beta"].attributes.time_of_day.as_deref(), expected);
        assert_eq!(maps["Alpha"].attributes.phone_service, nibble);
        assert_eq!(maps["Beta"].attributes.phone_service, 0x10 | nibble);
    }
    let fixture = Fixture::new();
    fixture.write("data/maps/maps.asm", "map Alpha, TILESET_FOREST, CAVE, LANDMARK_TEST, MUSIC_TEST, FALSE, PALETTE_UNKNOWN, FISHGROUP_TEST\nmap Beta, TILESET_FOREST, CAVE, LANDMARK_TEST, MUSIC_TEST, TRUE, PALETTE_DAY, FISHGROUP_TEST\n");
    assert!(
        source_map_geometry::read(&fixture.0)
            .unwrap_err()
            .to_string()
            .contains("unknown palette")
    );
}

#[test]
fn invalid_layout_lengths_and_escaping_paths_are_rejected() {
    let fixture = Fixture::new();
    std::fs::write(fixture.0.join("maps/shared.blk"), [1]).unwrap();
    assert!(
        source_map_geometry::read(&fixture.0)
            .unwrap_err()
            .to_string()
            .contains("block count")
    );
    fixture.write(
        "data/maps/blocks.asm",
        "Alpha_Blocks:\nINCBIN \"../outside.blk\"\n",
    );
    assert!(
        source_map_geometry::read(&fixture.0)
            .unwrap_err()
            .to_string()
            .contains("escapes source root")
    );
}

#[test]
fn connection_identity_mismatches_are_rejected() {
    let fixture = Fixture::new();
    fixture.write("data/maps/attributes.asm","map_attributes Alpha, ALPHA, 5\nconnection east, Beta, ALPHA, 0\nmap_attributes Beta, BETA, 6\n");
    assert!(
        source_map_geometry::read(&fixture.0)
            .unwrap_err()
            .to_string()
            .contains("name/constant mismatch")
    );
}

#[test]
fn external_source_geometry_preserves_all_map_layouts_and_connections() {
    let Some(root) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
        return;
    };
    let maps = source_map_geometry::read(std::path::Path::new(&root)).unwrap();
    assert_eq!(maps.len(), 388);
    let families: std::collections::BTreeSet<_> =
        maps.values().map(|m| &m.attributes.tileset_name).collect();
    assert_eq!(families.len(), 35);
    let house = &maps["PlayersHouse2F"];
    assert_eq!((house.attributes.width, house.attributes.height), (4, 3));
    for map in maps.values() {
        assert_eq!(
            map.blocks.len(),
            usize::from(map.attributes.width) * usize::from(map.attributes.height)
        );
        let encoded = serde_json::to_value(&map.attributes).unwrap();
        let restored: crystal_core::map::MapAttributes = serde_json::from_value(encoded).unwrap();
        assert_eq!(restored, map.attributes);
        for connection in &map.attributes.connections {
            assert!(maps.contains_key(&connection.target_map));
        }
    }
    assert!(
        maps["NewBarkTown"]
            .attributes
            .connections
            .iter()
            .any(|c| c.target_map == "Route29" && c.direction == "west")
    );
}

/// Core object allocation regression for the source-authored GS Ball staging
/// position. This checks allocation, not script dispatch or rendered visibility.
#[test]
fn goldenrod_off_map_receptionist_can_be_staged_then_allocated_in_map() {
    use crystal_core::{
        map::{MapEvents, ObjectEvent},
        world::{
            collision::{MetatileCollision, TilesetCollision},
            map::{OverworldMapData, TilePosition},
            session::OverworldSession,
        },
    };
    let id = "GOLDENRODPOKECENTER1F_PCC_TRADE_CORNER_RECEPTIONIST";
    let receptionist = ObjectEvent {
        sprite: "SPRITE_LINK_RECEPTIONIST".into(),
        sprite_has_facings: true,
        x: 16,
        y: 8,
        spritemovedata: "SPRITEMOVEDATA_STANDING_DOWN".into(),
        move_range_x: 0,
        move_range_y: 0,
        hram_x: -1,
        hram_y: -1,
        pal: 2,
        object_type: "OBJECTTYPE_SCRIPT".into(),
        radius: 0,
        script: "ObjectEvent".into(),
        label: None,
        event_flag: "-1".into(),
        object_identifier: Some(id.into()),
        sightline_direction_override: None,
    };
    let mut session = OverworldSession::with_events_and_objects(
        OverworldMapData {
            name: "GoldenrodPokecenter1F".into(),
            width: 5,
            height: 4,
            border_block: 0,
            connections: vec![],
            metatile_ids: vec![0; 20],
        },
        MapEvents::default(),
        vec![receptionist],
        TilesetCollision {
            metatiles: vec![MetatileCollision { collision: [0; 4] }],
        },
        TilePosition::new(3, 7),
    );
    assert_eq!(
        session.object_runtime_tile_by_id(id).unwrap(),
        TilePosition::new(16, 8)
    );
    assert!(
        !session.object_has_loaded_struct(0),
        "staged actor must not consume a visible object struct"
    );
    // The script writes map-object memory before disappear/appear copies it.
    session.objects[0].x = 0;
    session.objects[0].y = 7;
    session.delete_loaded_object_struct_for_disappear(id);
    assert!(session.copy_object_struct_for_appear(id).unwrap());
    assert!(session.object_struct_is_visible(0));
    assert_eq!(
        session.object_runtime_tile_by_id(id).unwrap(),
        TilePosition::new(0, 7)
    );
    session.delete_loaded_object_struct_for_disappear(id);
    assert!(!session.object_struct_is_visible(0));
    assert!(!session.object_has_loaded_struct(0));
}
