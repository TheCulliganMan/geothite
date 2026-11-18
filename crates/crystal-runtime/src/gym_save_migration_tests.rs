//! Pure save-data and geometry checks; no controller or gameplay is advanced.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crystal_assets::{GameDataSet, MapModule, TilesetDefinition};
use crystal_core::map::{MapAttributes, MapEvents, MapSceneTable, ObjectEvent};
use crystal_core::models::{BaseStats, Dv, Pokemon, PokemonSpecies};
use crystal_core::save::{self, SaveModpackIdentity};
use crystal_core::state::{
    GameState, OverworldMemory, OverworldObjectMapMemory, OverworldObjectMemory,
};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use serde_json::{Value, json};

use super::{Compatibility, parse_compatibility, read_pair, upgrade_checkpoint};
use crate::gym_save_migration::GYMS;

const TARGET: &str = "core+northstar-gym-overhaul-v1";

pub(super) fn metadata_value() -> Value {
    json!({
        "schema_version": 2,
        "target_manifest": "northstar-gym-overhaul-v1",
        "target_modpack_ids": [TARGET, format!("{TARGET}+hosted")],
        "sources": (0..6).map(|index| json!({
            "modpack_id": format!("source-release-{index}"),
            "modpack_hash": format!("{:064x}", index + 1),
            "content_hash": format!("{:064x}", index + 7),
        })).collect::<Vec<_>>(),
        "gym_reentry": {
            "policy": "minnesota-eight-gyms-v1",
            "maps": GYMS.iter().enumerate().map(|(index, map)| json!({
                "map_id": map, "arrival": {"x": 1 + index % 3, "y": 5}, "facing": "down",
            })).collect::<Vec<_>>()
        }
    })
}

fn metadata() -> Compatibility {
    parse_compatibility(
        &serde_json::to_vec(&metadata_value()).expect("metadata"),
        TARGET,
    )
    .expect("approved metadata")
}

pub(super) fn source_state(map: &str) -> GameState {
    let mut state = GameState {
        overworld: OverworldMemory::Active {
            map_name: map.into(),
            tile: TilePosition::new(2, 2),
            facing: Direction::Left,
            mode: MovementMode::Normal,
        },
        money: 4321,
        coins: 27,
        ..GameState::default()
    };
    state.badges.johto[2] = true;
    state.bag.items.insert("POTION".into(), 7);
    let species = PokemonSpecies::new_for_tests("TESTMON", BaseStats::new(50, 50, 50, 50, 50, 50));
    state.storage.party.pokemon[0] = Some(Pokemon::new_for_tests(species, 17, Dv::default()));
    state.sync_party_from_storage();
    for flag in [
        "EVENT_MN_GYM_3_TRAINER_1_DEFEATED",
        "EVENT_GYM_TM_RECEIVED",
        "EVENT_REMATCH_REWARD_RECEIVED",
    ] {
        state
            .flags
            .set_event_flag(flag, true)
            .expect("receipt flag");
    }
    state
        .script_runtime
        .memory
        .insert("museum_receipt".into(), "earned".into());
    for map in GYMS.into_iter().chain(["MnWinona"]) {
        state
            .map_block_overrides
            .insert(map.into(), [((1, 1), 4)].into());
        state.map_object_overrides.insert(
            map.into(),
            OverworldObjectMapMemory {
                objects: [("OLD_ACTOR".into(), OverworldObjectMemory { x: 3, y: 3 })].into(),
                hidden_object_identifiers: ["HIDDEN_ACTOR".into()].into(),
                shown_object_identifiers: ["SHOWN_ACTOR".into()].into(),
                ..OverworldObjectMapMemory::default()
            },
        );
    }
    state
}

#[test]
fn gym_upgrade_relocates_all_eight_rooms_and_preserves_every_other_field() {
    let metadata = metadata();
    metadata
        .gym_reentry
        .as_ref()
        .expect("policy")
        .validate_arrivals(&data())
        .expect("base arrivals");
    for (index, map) in GYMS.iter().enumerate() {
        let source = source_state(map);
        let original = source.clone();
        let upgraded =
            upgrade_checkpoint(source.clone(), &metadata, None, &data()).expect("idle gym upgrade");
        let mut expected = source.clone();
        for gym in GYMS {
            expected.map_block_overrides.remove(gym);
            expected.map_object_overrides.remove(gym);
        }
        expected.overworld = OverworldMemory::Active {
            map_name: (*map).into(),
            tile: TilePosition::new(1 + (index % 3) as i16, 5),
            facing: Direction::Down,
            mode: MovementMode::Normal,
        };
        assert_eq!(upgraded, expected, "{map}");
        assert_eq!(source, original);
    }
}

#[test]
fn gym_upgrade_preserves_outside_position_mode_and_unrelated_overrides() {
    let mut source = source_state("MnWinona");
    if let OverworldMemory::Active { mode, .. } = &mut source.overworld {
        *mode = MovementMode::Surf;
    }
    let upgraded =
        upgrade_checkpoint(source.clone(), &metadata(), None, &data()).expect("outside upgrade");
    let mut expected = source;
    for gym in GYMS {
        expected.map_block_overrides.remove(gym);
        expected.map_object_overrides.remove(gym);
    }
    assert_eq!(upgraded, expected);
}

#[test]
fn gym_upgrade_rejects_unsafe_checkpoints_without_changing_source() {
    let metadata = metadata();
    let mut cases = Vec::new();
    let mut surfing = source_state(GYMS[0]);
    if let OverworldMemory::Active { mode, .. } = &mut surfing.overworld {
        *mode = MovementMode::Surf;
    }
    cases.push(surfing);
    let mut scene = source_state(GYMS[0]);
    scene.script_runtime.all_input_locked = true;
    cases.push(scene);
    let mut hidden = source_state(GYMS[0]);
    hidden
        .map_object_overrides
        .get_mut(GYMS[0])
        .expect("gym")
        .player_hidden = true;
    cases.push(hidden);
    let mut facing = source_state(GYMS[0]);
    facing
        .map_object_overrides
        .get_mut(GYMS[0])
        .expect("gym")
        .object_structs
        .player_fixed_facing = true;
    cases.push(facing);
    let mut moving = source_state(GYMS[0]);
    moving
        .map_object_overrides
        .get_mut(GYMS[0])
        .expect("gym")
        .object_structs
        .player_last_tile_occupied_remaining_frames = 1;
    cases.push(moving);
    for source in cases {
        let original = source.clone();
        assert!(upgrade_checkpoint(source.clone(), &metadata, None, &data()).is_err());
        assert_eq!(source, original);
    }
}

#[test]
fn gym_metadata_rejects_wrong_schema_target_sources_and_map_allowlist() {
    let valid = metadata_value();
    let encoded = serde_json::to_vec(&valid).expect("valid metadata");
    assert!(parse_compatibility(&encoded, &format!("{TARGET}+next-release")).is_err());
    let mut invalid_cases = Vec::new();
    for (pointer, replacement) in [
        ("/schema_version", json!(3)),
        ("/target_manifest", json!("northstar-refinement-v1")),
        ("/target_modpack_ids", json!([TARGET])),
        ("/target_modpack_ids/1", json!(TARGET)),
        ("/target_modpack_ids/1", json!("unrelated-release")),
        ("/sources/1", valid["sources"][0].clone()),
        ("/sources/0/modpack_hash", json!("invalid")),
        ("/gym_reentry", Value::Null),
        ("/gym_reentry/policy", json!("all-maps")),
        ("/gym_reentry/maps/1/map_id", json!(GYMS[0])),
        ("/gym_reentry/maps/0/map_id", json!("MnWinona")),
        ("/gym_reentry/maps/0/arrival/x", json!(-1)),
        ("/gym_reentry/maps/0/facing", json!("north")),
    ] {
        let mut invalid = valid.clone();
        *invalid.pointer_mut(pointer).expect("metadata path") = replacement;
        invalid_cases.push(invalid);
    }
    let mut short_sources = valid.clone();
    short_sources["sources"]
        .as_array_mut()
        .expect("sources")
        .pop();
    invalid_cases.push(short_sources);
    let mut inconsistent_source = valid.clone();
    inconsistent_source["sources"][1] = valid["sources"][0].clone();
    inconsistent_source["sources"][1]["content_hash"] = json!("e".repeat(64));
    invalid_cases.push(inconsistent_source);
    let mut short_maps = valid.clone();
    short_maps["gym_reentry"]["maps"]
        .as_array_mut()
        .expect("maps")
        .pop();
    invalid_cases.push(short_maps);
    for invalid in invalid_cases {
        assert!(
            parse_compatibility(
                &serde_json::to_vec(&invalid).expect("invalid metadata"),
                TARGET
            )
            .is_err(),
            "accepted {invalid}"
        );
    }
}

#[test]
fn refinement_schema_retains_its_original_policy_without_gym_resets() {
    let mut value = metadata_value();
    value["schema_version"] = json!(1);
    value["target_manifest"] = json!("northstar-refinement-v1");
    value["target_modpack_ids"] = json!(["core+northstar-refinement-v1"]);
    value
        .as_object_mut()
        .expect("metadata object")
        .remove("gym_reentry");
    let metadata = parse_compatibility(
        &serde_json::to_vec(&value).expect("refinement metadata"),
        "core+northstar-refinement-v1",
    )
    .expect("legacy schema");
    let source = source_state(GYMS[0]);
    assert_eq!(
        upgrade_checkpoint(source.clone(), &metadata, None, &data()).expect("legacy upgrade"),
        source
    );
    value["gym_reentry"] = metadata_value()["gym_reentry"].clone();
    assert!(
        parse_compatibility(
            &serde_json::to_vec(&value).expect("mixed metadata"),
            "core+northstar-refinement-v1"
        )
        .is_err()
    );
}

#[test]
fn gym_arrivals_require_unoccupied_base_floor_with_no_callback_repair() {
    let metadata = metadata();
    let policy = metadata.gym_reentry.as_ref().expect("policy");
    policy
        .validate_arrivals(&data())
        .expect("all arrivals valid");
    let mut wall = data();
    wall.maps.get_mut(GYMS[0]).expect("gym").blocks.fill(1);
    assert!(policy.validate_arrivals(&wall).is_err());
    let mut small = data();
    let room = small.maps.get_mut(GYMS[0]).expect("gym");
    room.attributes.height = 1;
    room.blocks = vec![0; 4];
    assert!(policy.validate_arrivals(&small).is_err());
    let mut occupied = data();
    occupied
        .maps
        .get_mut(GYMS[0])
        .expect("gym")
        .objects
        .push(actor());
    assert!(policy.validate_arrivals(&occupied).is_err());
    let mut missing = data();
    missing.maps.remove(GYMS[0]);
    assert!(policy.validate_arrivals(&missing).is_err());
}

#[test]
fn gym_candidate_backup_migrates_a_copy_and_rejects_wrong_identity() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let directory = std::env::temp_dir().join(format!(
        "geothite-gym-copy-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).expect("owned test directory");
    let path = directory.join("old.crystalsave");
    let metadata = metadata();
    let source = &metadata.sources[0];
    let identity = SaveModpackIdentity::new(&source.modpack_id, &source.modpack_hash)
        .expect("source identity");
    let state = GameState {
        overworld: source_state(GYMS[0]).overworld,
        money: 4321,
        ..GameState::default()
    };
    save::write_new_compatible_save(&path, state.clone(), &identity, &source.content_hash)
        .expect("source pair");
    let backup = save::save_backup_path(&path);
    let original_backup = std::fs::read(&backup).expect("original backup");
    std::fs::write(&path, b"damaged source primary").expect("damaged primary");
    let selected = read_pair(&path, |bytes| {
        let saved = save::read_save_game_bytes_for_modpack(
            bytes,
            "gym candidate",
            &identity,
            &source.content_hash,
        )?;
        upgrade_checkpoint(saved.into_state(), &metadata, None, &data())
    })
    .expect("valid backup candidate");
    assert!(selected.backup);
    assert_eq!(selected.state.money, state.money);
    assert_eq!(
        selected
            .state
            .overworld
            .snapshot_identity()
            .expect("overworld")
            .1,
        TilePosition::new(1, 5)
    );
    assert_eq!(
        std::fs::read(&backup).expect("backup untouched"),
        original_backup
    );
    assert_eq!(
        std::fs::read(&path).expect("primary untouched"),
        b"damaged source primary"
    );
    let wrong = SaveModpackIdentity::new("wrong-release", "f".repeat(64)).expect("wrong identity");
    assert!(
        read_pair(&path, |bytes| Ok(save::read_save_game_bytes_for_modpack(
            bytes,
            "wrong source",
            &wrong,
            &source.content_hash
        )?
        .into_state()))
        .is_none()
    );
    std::fs::remove_dir_all(&directory).expect("remove owned fixture");
}

#[test]
fn shore_roster_restoration_uses_target_sprite_and_preserves_saved_poses() {
    for (map_id, actor_id) in [
        ("MnWinona", "MN_RESTORE_WINONA_GILLPUP"),
        ("MnLeechLake", "MN_RESTORE_LEECH_GILLPUP"),
    ] {
        let mut data = data();
        let mut room = map(map_id);
        let mut partner = actor();
        partner.object_identifier = Some(actor_id.into());
        partner.sprite = "SPRITE_GILLPUP".into();
        room.objects.push(partner);
        data.maps.insert(map_id.into(), room);
        let mut old = data
            .overworld_session(map_id, TilePosition::new(2, 2), 0)
            .expect("source geometry");
        old.object_runtime_tiles
            .insert(actor_id.into(), TilePosition::new(3, 5));
        old.object_facings.insert(actor_id.into(), Direction::Left);
        let saved = old.object_struct_roster_memory().expect("saved roster");
        assert_eq!(saved.structs.len(), 1, "partner must be allocated");
        let original = saved.clone();

        data.maps.get_mut(map_id).expect("shore room").objects[0].sprite = "SPRITE_CHORIP".into();
        let mut target = data
            .overworld_session(map_id, TilePosition::new(2, 2), 0)
            .expect("target geometry");
        target
            .restore_object_struct_roster_memory(&saved)
            .expect("restore source pose into target actor");
        assert_eq!(target.objects[0].sprite, "SPRITE_CHORIP");
        assert_eq!(
            target.objects[0].object_identifier.as_deref(),
            Some(actor_id)
        );
        assert_eq!(
            target.object_struct_roster_memory().expect("target roster"),
            original
        );
        assert_eq!(saved, original);
    }
}

pub(super) fn data() -> GameDataSet {
    GameDataSet {
        maps: GYMS
            .into_iter()
            .map(|name| (name.into(), map(name)))
            .collect(),
        tilesets: [(
            "gym-fixture".into(),
            TilesetDefinition {
                collision: [
                    ("0".into(), vec!["FLOOR".into(); 4]),
                    ("1".into(), vec!["WALL".into(); 4]),
                ]
                .into(),
                palette_map: Vec::new(),
            },
        )]
        .into(),
        ..GameDataSet::default()
    }
}

pub(super) fn map(name: &str) -> MapModule {
    MapModule {
        id: name.into(),
        attributes: MapAttributes {
            tileset_name: "gym-fixture".into(),
            border_block: 0,
            width: 4,
            height: 4,
            connections: Vec::new(),
            time_of_day: None,
            phone_service: 0,
            phone_flag: false,
            environment: Some("indoor".into()),
            location: None,
            music: None,
            palette: None,
            fishing_group: None,
            map_constant: None,
            map_group_constant: None,
            blocks_label: None,
            map_scripts_label: None,
            map_events_label: None,
            connection_flags: None,
        },
        scripts: BTreeMap::new(),
        trainer_scripts: BTreeMap::new(),
        scripted_trainer_battles: Vec::new(),
        scripted_wild_battles: Vec::new(),
        script_item_grants: Vec::new(),
        script_item_checks: Vec::new(),
        script_item_takes: Vec::new(),
        script_economy_commands: Vec::new(),
        gift_pokemon_scripts: Vec::new(),
        script_flag_commands: Vec::new(),
        script_scene_commands: Vec::new(),
        script_audio_commands: Vec::new(),
        script_block_changes: Vec::new(),
        script_object_commands: Vec::new(),
        script_movements: Vec::new(),
        script_map_commands: Vec::new(),
        script_text_commands: Vec::new(),
        script_text_bodies: BTreeMap::new(),
        script_menu_definitions: BTreeMap::new(),
        script_vertical_menus: BTreeMap::new(),
        script_elevators: BTreeMap::new(),
        script_variable_commands: Vec::new(),
        script_control_commands: Vec::new(),
        script_field_pickups: Vec::new(),
        script_shop_commands: Vec::new(),
        script_phone_commands: Vec::new(),
        script_runtime_commands: Vec::new(),
        script_swarm_commands: Vec::new(),
        map_script_section_commands: Vec::new(),
        map_event_section_commands: Vec::new(),
        scenes: MapSceneTable::default(),
        events: MapEvents::default(),
        objects: Vec::new(),
        blocks: vec![0; 16],
    }
}

pub(super) fn actor() -> ObjectEvent {
    ObjectEvent {
        sprite: "SPRITE_MON".into(),
        sprite_has_facings: true,
        x: 1,
        y: 5,
        spritemovedata: "SPRITEMOVEDATA_STANDING_DOWN".into(),
        move_range_x: 0,
        move_range_y: 0,
        hram_x: -1,
        hram_y: -1,
        pal: 0,
        object_type: "OBJECTTYPE_SCRIPT".into(),
        radius: 0,
        script: "GymActor".into(),
        label: None,
        event_flag: "EVENT_HIDDEN".into(),
        object_identifier: Some("GYM_ACTOR".into()),
        sightline_direction_override: None,
    }
}
