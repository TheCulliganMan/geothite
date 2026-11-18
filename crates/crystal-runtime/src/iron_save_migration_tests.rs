//! Release-specific save policy and native collision checks using idle data.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crystal_assets::{AssetRoot, GameDataSet, read_loaded_verified_compiled_game_pack};
use crystal_core::state::{GameState, OverworldMemory, OverworldObjectMapMemory};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, MovementMode, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump, attempt_step,
};
use serde_json::{Value, json};

use super::{
    Compatibility, SAVE_COMPATIBILITY_FILE, cave_tests, direct_iron_parent, gym_tests,
    parse_compatibility, upgrade_source_checkpoint,
};
use crate::{CrystalRuntime, RuntimeGameShell};

const PARENT: &str = "core+northstar-gym-overhaul-v1+northstar-cave-overhaul-v1";
const TARGET: &str =
    "core+northstar-gym-overhaul-v1+northstar-cave-overhaul-v1+northstar-iron-workings-revision-v1";
const HAULAGE: &str = "MnIronHaulage";
const SUMP: &str = "MnIronSump";

fn metadata_value() -> Value {
    let mut value = cave_tests::metadata_value();
    value["schema_version"] = json!(3);
    value["target_manifest"] = json!("northstar-iron-workings-revision-v1");
    value["target_modpack_ids"] = json!([
        TARGET,
        format!("{TARGET}+realtime-clock+player-customization")
    ]);
    for (i, id) in [
        PARENT.to_owned(),
        format!("{PARENT}+realtime-clock+player-customization"),
    ]
    .into_iter()
    .enumerate()
    {
        value["sources"].as_array_mut().expect("sources").push(json!({
            "modpack_id": id, "modpack_hash": format!("{:064x}", 100+i), "content_hash": format!("{:064x}", 102+i)
        }));
    }
    value["layout_reentry"] = json!([
        "MnRochesterGym",
        "MnStPaulGym",
        "MnBrainerdGym",
        "MnVoyageursIce1F",
        "MnVoyageursIceB1F",
        "MnR17Cave1F"
    ]);
    value["iron_relocations"] = json!([
        {"map_id": HAULAGE, "from": [6,8], "landings":[[6,7],[5,8],[6,6]]},
        {"map_id": HAULAGE, "from": [7,8], "landings":[[7,7],[8,8],[7,6]]},
        {"map_id": SUMP, "from": [8,6], "landings":[[7,6],[7,5],[9,5]]},
        {"map_id": SUMP, "from": [9,6], "landings":[[9,5],[10,6],[10,5]]},
    ]);
    value
}

fn parse(value: &Value) -> anyhow::Result<Compatibility> {
    parse_compatibility(&serde_json::to_vec(value)?, TARGET)
}

fn data() -> GameDataSet {
    let mut data = gym_tests::data();
    data.tilesets
        .get_mut("gym-fixture")
        .expect("tileset")
        .collision
        .insert(
            "2".into(),
            ["HOP_DOWN", "HOP_DOWN", "WALL", "WALL"]
                .map(str::to_owned)
                .into(),
        );
    for (map, x, y, block) in [(HAULAGE, 3, 4, 2), (SUMP, 4, 3, 1)] {
        let mut room = gym_tests::map(map);
        room.attributes.width = 12;
        room.attributes.height = 12;
        room.blocks = vec![0; 144];
        room.blocks[y * 12 + x] = block;
        let mut actor = gym_tests::actor();
        actor.x = 4;
        actor.y = 5;
        actor.event_flag = "-1".into();
        room.objects.push(actor);
        data.maps.insert(map.into(), room);
    }
    data
}

fn source_at(map: &str, at: TilePosition, mode: MovementMode) -> GameState {
    let mut state = cave_tests::source_state(map);
    state.overworld = OverworldMemory::Active {
        map_name: map.into(),
        tile: at,
        mode,
        facing: Direction::Left,
    };
    state
}

#[test]
fn iron_policy_requires_the_exact_parent_pair_and_reviewed_rooms() {
    let valid = metadata_value();
    parse(&valid).expect("schema three");
    for (path, replacement) in [
        ("/sources/6/modpack_id", json!("unrelated")),
        (
            "/target_modpack_ids/1",
            json!(format!("{TARGET}+wrong-wrapper")),
        ),
        (
            "/sources/0/modpack_id",
            json!("other+northstar-cave-overhaul-v1"),
        ),
        ("/sources/7/content_hash", json!("missing")),
        ("/iron_relocations/1/from", json!([6, 8])),
        ("/iron_relocations/0/map_id", json!(SUMP)),
        ("/iron_relocations/2/landings", json!([[10, 5]])),
        ("/iron_relocations/2/landings", json!([[7, 6], [7, 6]])),
        ("/layout_reentry/0", json!("MnMankatoGym")),
        ("/cave_reentry/policy", json!("minnesota-five-caves-v1")),
    ] {
        let mut invalid = valid.clone();
        *invalid.pointer_mut(path).expect("fixture path") = replacement;
        assert!(parse(&invalid).is_err(), "{path}");
    }
}

#[test]
fn v04_keeps_unedited_gym_and_cave_rooms_while_older_sources_reenter_all() {
    let metadata = parse(&metadata_value()).expect("schema three");
    let data = data();
    for (map, revised) in [
        ("MnBrainerdGym", true),
        ("MnStPaulGym", true),
        ("MnRochesterGym", true),
        ("MnVoyageursIce1F", true),
        ("MnVoyageursIceB1F", true),
        ("MnR17Cave1F", true),
        ("MnMankatoGym", false),
        ("MnR01Cave1F", false),
    ] {
        let source = cave_tests::source_state(map);
        for release in &metadata.sources {
            let migrated = upgrade_source_checkpoint(source.clone(), &metadata, release, &data)
                .expect("idle save");
            let reentry = revised || !direct_iron_parent(&metadata, &release.modpack_id);
            assert_eq!(
                migrated.map_object_overrides.contains_key(map),
                !reentry,
                "{map}"
            );
            assert_eq!(
                migrated.map_block_overrides.contains_key(map),
                !reentry,
                "{map}"
            );
            assert_eq!(source.overworld == migrated.overworld, !reentry, "{map}");
            assert_eq!(source.flags, migrated.flags);
        }
    }
}

#[test]
fn iron_relocation_consults_saved_objects_and_keeps_every_other_mine_field() {
    let metadata = parse(&metadata_value()).expect("schema three");
    let data = data();
    metadata
        .iron_relocations
        .as_ref()
        .expect("iron policy")
        .validate_landings(&data)
        .expect("known geometry");
    let mut source = source_at(SUMP, TilePosition::new(8, 6), MovementMode::Bike);
    let mut room = data
        .overworld_session(SUMP, TilePosition::new(7, 6), 0)
        .expect("target room");
    room.object_runtime_tiles
        .insert("GYM_ACTOR".into(), TilePosition::new(7, 6));
    let roster = room.object_struct_roster_memory().expect("actor snapshot");
    assert_eq!(roster.structs[0].live_tile, TilePosition::new(7, 6));
    source.map_object_overrides.insert(
        SUMP.into(),
        OverworldObjectMapMemory {
            object_structs: roster,
            ..Default::default()
        },
    );
    source
        .map_block_overrides
        .insert(SUMP.into(), [((10, 3), 0)].into());
    let release = &metadata.sources[6];
    let moved = upgrade_source_checkpoint(source.clone(), &metadata, release, &data)
        .expect("second available landing");
    let mut expected = source.clone();
    if let OverworldMemory::Active { tile, .. } = &mut expected.overworld {
        *tile = TilePosition::new(7, 5);
    }
    for map in metadata.layout_reentry.as_ref().expect("changed layouts") {
        expected.map_object_overrides.remove(map);
        expected.map_block_overrides.remove(map);
    }
    assert_eq!(moved, expected);
    // A retained block can already provide ordinary floor under the source tile.
    source
        .map_block_overrides
        .get_mut(SUMP)
        .expect("overrides")
        .insert((4, 3), 0);
    let stationary =
        upgrade_source_checkpoint(source.clone(), &metadata, release, &data).expect("saved floor");
    assert_eq!(stationary.overworld, source.overworld);
    let blocks = source.map_block_overrides.get_mut(SUMP).expect("overrides");
    blocks.remove(&(4, 3));
    blocks.extend([((3, 2), 1), ((4, 2), 1)]);
    assert!(
        upgrade_source_checkpoint(source, &metadata, release, &data).is_err(),
        "occupied first landing and rock on remaining two"
    );
}

#[test]
fn ongoing_sump_surf_and_canoe_positions_survive_all_eight_migrations() {
    let metadata = parse(&metadata_value()).expect("schema three");
    for mode in [MovementMode::Surf, MovementMode::SurfPika] {
        let mut source = source_at(SUMP, TilePosition::new(9, 10), mode);
        source
            .script_runtime
            .memory
            .insert("geothiteCanoeActive".into(), "1".into());
        for release in &metadata.sources {
            let moved = upgrade_source_checkpoint(source.clone(), &metadata, release, &data())
                .expect("unchanged water position");
            assert_eq!(source.overworld, moved.overworld);
            assert_eq!(source.script_runtime.memory, moved.script_runtime.memory);
        }
    }
}

fn verify_checkpoint(
    parent: &CrystalRuntime,
    target: &CrystalRuntime,
    root: &AssetRoot,
    dir: &Path,
    road: &str,
    x: i16,
    y: i16,
) -> GameState {
    let parent_slot = dir.join(format!("{road}-parent-{x}-{y}.crystalsave"));
    let target_slot = dir.join(format!("{road}-target-{x}-{y}.crystalsave"));
    let mut game = RuntimeGameShell::new_game_at_runtime_tile(
        root.clone(),
        parent.clone(),
        parent.title_new_game_spawn_identifier().expect("spawn"),
        road,
        x,
        y,
    )
    .expect("idle checkpoint on released map");
    game.save(&parent_slot).expect("save in released format");
    let source = parent
        .load_save(&parent_slot)
        .expect("read released checkpoint");
    let idle = super::normalize_upgrade_checkpoint(source.clone(), parent.data())
        .expect("fixture must be an idle checkpoint in the released game");
    target
        .validate_save_state_for_runtime_pack(&idle)
        .expect("idle source state must remain valid on the revised maps");
    let loaded = super::prepare_runtime_save_slot(target, &target_slot, |candidate| {
        if candidate.modpack_id == parent.modpack().id() {
            parent_slot.clone()
        } else {
            dir.join("absent.crystalsave")
        }
    })
    .expect("migrate previously released checkpoint");
    assert!(loaded, "missing migrated save at {road} ({x},{y})");
    let state = target
        .load_save(&target_slot)
        .expect("read migrated checkpoint");
    assert_eq!(state.overworld, source.overworld);
    assert_eq!(state.flags, source.flags);
    state
}

fn check_ledge_save_landings(target: &CrystalRuntime, wayfinding: bool, crossings: bool) {
    let parent_path = PathBuf::from(
        std::env::var_os("CRYSTAL_REGIONAL_SAVE_PARENT").expect("released parent pack"),
    );
    let root = AssetRoot::new(parent_path.parent().expect("parent pack directory"));
    let parent = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&parent_path).expect("verified parent"),
    )
    .expect("parent runtime");
    let dir = std::env::temp_dir().join(format!(
        "geothite-carrier-saves-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir(&dir).expect("unique save directory");
    let options = StepOptions {
        force_step_after_turn: true,
        ..StepOptions::default()
    };
    let mut cliffs = vec![("MnR10C00R00", "mn_iron_descent_thaw", 20, 24..=27)];
    if wayfinding {
        cliffs.push(("MnR01C00R00", "mn_outdoor_johto_stairs", 16, 24..=25));
    }
    for (road, atlas, y, columns) in cliffs {
        let map = target.data().overworld_map(road).expect("cliff road");
        let tileset = target
            .data()
            .tileset_collision(atlas)
            .expect("cliff collision");
        for x in columns {
            let lip = TilePosition::new(x, y);
            let state = verify_checkpoint(&parent, target, &root, &dir, road, x, y);
            let OverworldMemory::Active {
                map_name,
                tile,
                facing,
                mode,
            } = state.overworld
            else {
                panic!("expected idle overworld save");
            };
            assert_eq!(map_name, road);
            assert_eq!(tile, lip);
            let player = PlayerMovementState { tile, facing, mode };
            let mut upper = player;
            assert!(matches!(
                attempt_step(&mut upper, Direction::Up, &map, &tileset, options),
                StepOutcome::Moved { to, .. } if to == TilePosition::new(x, y - 1)
            ));
            let mut lower = player;
            assert!(matches!(
                attempt_ledge_jump(&mut lower, Direction::Down, &map, &tileset, options),
                LedgeJumpOutcome::Jumped { from, over, to, .. }
                    if from == lip && over == TilePosition::new(x, y + 1) && to == TilePosition::new(x, y + 2)
            ));
            assert!(matches!(
                attempt_step(&mut lower, Direction::Up, &map, &tileset, options),
                StepOutcome::Blocked { .. }
            ));
            assert!(matches!(
                attempt_step(&mut lower, Direction::Down, &map, &tileset, options),
                StepOutcome::Moved { to, .. } if to == TilePosition::new(x, y + 3)
            ));
        }
    }
    if crossings {
        for (map, x, y) in [
            ("MnLeagueHallOfFame", 6, 5),
            ("MnLeagueHallOfFame", 4, 11),
            ("MnR15C00R00", 40, 28),
            ("MnR18C00R00", 4, 20),
            ("MnFormerShores", 8, 7),
            ("MnR12C00R00", 35, 14),
        ] {
            verify_checkpoint(&parent, target, &root, &dir, map, x, y);
        }
    }
    std::fs::remove_dir_all(dir).expect("remove test saves");
}

#[test]
#[ignore = "requires CRYSTAL_IRON_TARGET revision pack"]
fn iron_workings_target_accepts_both_v04_parents_and_reviewed_landings() {
    let path = PathBuf::from(std::env::var_os("CRYSTAL_IRON_TARGET").expect("target pack"));
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified target"),
    )
    .expect("runtime");
    let meta = parse_compatibility(
        runtime
            .runtime_file(SAVE_COMPATIBILITY_FILE)
            .expect("compatibility bytes"),
        runtime.modpack().id(),
    )
    .expect("valid floorplan save policy");
    if runtime
        .modpack()
        .id()
        .split('+')
        .any(|part| part == super::STORY_FLOW_MANIFEST)
    {
        assert_eq!(meta.schema_version, 4);
        let current: BTreeSet<_> = meta
            .sources
            .iter()
            .filter(|source| super::direct_story_parent(&meta, &source.modpack_id))
            .map(|source| source.modpack_hash.as_str())
            .collect();
        assert_eq!(
            current,
            BTreeSet::from([
                "2e2554784f82f6b9e53c8b1d7a24e5993f730eefb460239d3edc4bc7d558a3df",
                "de7379234cf30325688436bd2c846b7f00e23830f0125d762ea962ad304fddb4",
            ])
        );
    } else {
        assert_eq!(meta.schema_version, 3);
    }
    if runtime
        .modpack()
        .id()
        .split('+')
        .any(|id| id == super::REGIONAL_DESIGN_MANIFEST)
    {
        let wayfinding = runtime
            .modpack()
            .id()
            .split('+')
            .any(|id| id == super::WAYFINDING_MANIFEST);
        let crossings = runtime
            .modpack()
            .id()
            .split('+')
            .any(|id| id == super::CROSSINGS_MANIFEST);
        let direct_count = runtime
            .modpack()
            .id()
            .split('+')
            .filter(|id| {
                matches!(
                    *id,
                    super::CROSSINGS_MANIFEST
                        | super::WAYFINDING_MANIFEST
                        | super::LAKESIDE_MANIFEST
                )
            })
            .count()
            * 2;
        assert_eq!(meta.sources.len(), 12 + direct_count);
        assert_eq!(meta.unchanged_checkpoint_parents.len(), direct_count);
        for direct in &meta.sources[..direct_count] {
            assert!(
                meta.unchanged_checkpoint_parents
                    .contains(&direct.modpack_id)
            );
        }
        assert_eq!(meta.regional_parents.len(), 2);
        check_ledge_save_landings(&runtime, wayfinding, crossings);
        for source in &meta.sources[direct_count..direct_count + 2] {
            assert!(meta.regional_parents.contains(&source.modpack_id));
            let state = source_at(
                "MnR17C00R01",
                TilePosition::new(24, 2),
                MovementMode::Normal,
            );
            let upgraded = upgrade_source_checkpoint(state.clone(), &meta, source, runtime.data())
                .expect("regional design preserves the exact road checkpoint");
            assert_eq!(upgraded.overworld, state.overworld);
            assert_eq!(upgraded.flags, state.flags);
        }
        let bytes = runtime
            .runtime_file(SAVE_COMPATIBILITY_FILE)
            .expect("release metadata");
        let mut bad: Value = serde_json::from_slice(bytes).expect("JSON");
        bad["sources"][0]["content_hash"] = json!("0".repeat(64));
        assert!(
            parse_compatibility(
                &serde_json::to_vec(&bad).expect("JSON"),
                runtime.modpack().id()
            )
            .is_err()
        );
    }
    meta.iron_relocations
        .as_ref()
        .expect("iron policy")
        .validate_landings(runtime.data())
        .expect("native landings");
    meta.gym_reentry
        .as_ref()
        .expect("gym policy")
        .validate_arrivals(runtime.data())
        .expect("gym arrivals");
    meta.cave_reentry
        .as_ref()
        .expect("cave policy")
        .validate_arrivals(runtime.data())
        .expect("cave arrivals");
    let parents: BTreeSet<_> = meta
        .sources
        .iter()
        .filter(|s| direct_iron_parent(&meta, &s.modpack_id))
        .map(|s| (s.modpack_hash.as_str(), s.content_hash.as_str()))
        .collect();
    assert_eq!(
        parents,
        BTreeSet::from([
            (
                "11b81fcc2e06bab49de4b95f3a4211bb270872fcafbbeed5fbfad9021f143834",
                "1cd664f35594c693a87a68e25c2430a0ad24a814cd1959bf198678dd340c8149"
            ),
            (
                "c0a19e3336c77920a119c7996601aca7f3962cc9b56eca84087ca1d29b1e335a",
                "fd71e9f9231a031d5931caf559faeee0fa2287a8cb6e6686ccd3d156f8850e41"
            )
        ])
    );
    for parent in meta
        .sources
        .iter()
        .filter(|s| direct_iron_parent(&meta, &s.modpack_id))
    {
        for (map, from, to) in [
            (HAULAGE, [6, 8], [6, 7]),
            (HAULAGE, [7, 8], [7, 7]),
            (SUMP, [8, 6], [7, 6]),
            (SUMP, [9, 6], [9, 5]),
        ] {
            let state = source_at(
                map,
                TilePosition::new(from[0], from[1]),
                MovementMode::Normal,
            );
            let moved = upgrade_source_checkpoint(state, &meta, parent, runtime.data())
                .expect("relocate on the native map");
            assert_eq!(
                moved.overworld.snapshot_identity().expect("overworld").1,
                TilePosition::new(to[0], to[1])
            );
        }
    }
}

#[test]
fn story_release_keeps_floorplan_positions_and_inherits_earlier_repairs() {
    let marker = super::STORY_FLOW_MANIFEST;
    let target = format!("{TARGET}+{marker}");
    let hosted = format!("{target}+realtime-clock+player-customization");
    let mut value = metadata_value();
    value["schema_version"] = json!(4);
    value["target_manifest"] = json!(marker);
    value["target_modpack_ids"] = json!([target, hosted]);
    for (index, id) in [
        TARGET.to_owned(),
        format!("{TARGET}+realtime-clock+player-customization"),
    ]
    .into_iter()
    .enumerate()
    {
        value["sources"].as_array_mut().expect("sources").push(json!({
            "modpack_id": id, "modpack_hash": format!("{:064x}", 300+index), "content_hash": format!("{:064x}", 310+index)
        }));
    }
    let metadata = parse_compatibility(&serde_json::to_vec(&value).expect("metadata"), &target)
        .expect("raw story child");
    parse_compatibility(&serde_json::to_vec(&value).expect("metadata"), &hosted)
        .expect("hosted story child");
    for source in &metadata.sources[8..] {
        let state = source_at(
            "MnBrainerdGym",
            TilePosition::new(8, 10),
            MovementMode::Normal,
        );
        let copy =
            upgrade_source_checkpoint(state.clone(), &metadata, source, &GameDataSet::default())
                .expect("floorplan save does not need room relocation");
        assert_eq!(state.overworld, copy.overworld);
    }
    assert!(direct_iron_parent(&metadata, PARENT));
    value["sources"][8]["modpack_id"] = json!("unknown");
    assert!(parse_compatibility(&serde_json::to_vec(&value).expect("metadata"), &target).is_err());
}
