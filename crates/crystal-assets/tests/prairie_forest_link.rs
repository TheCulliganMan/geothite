//! Native publication/components; production input and Detroit hand-in remain separate.
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use crystal_assets::{CompiledGamePack, LAKE_OBSERVATION_EVENTS, RuntimeScriptCommandRef};
use crystal_core::random::ReplayDivider;
use crystal_core::save::{
    SaveModpackIdentity, read_save_game_for_modpack, write_save_game_for_modpack,
};
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::encounters::{FieldEncounterKind, select_headbutt_encounter};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use serde_json::{Value, json};

const MAP: &str = "MnR19C00R00";
const SIGNS: [&str; 3] = [
    "MN_R19_OBSERVE_PINE",
    "MN_R19_OBSERVE_WETLAND",
    "MN_R19_OBSERVE_PRAIRIE",
];
const REWARDED: &str = "EVENT_MN_LAKE_OBSERVATIONS_REWARDED";

fn inputs() -> &'static (CompiledGamePack, CompiledGamePack, String, String) {
    static PACKS: OnceLock<(CompiledGamePack, CompiledGamePack, String, String)> = OnceLock::new();
    PACKS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let source = crystal_assets::read_verified_compiled_game_pack(
            root.join("content-packs/text-tui.crystalpack"),
        )
        .expect("supplied source pack");
        let read = |name: &str| {
            fs::read_to_string(root.join("planning/minnesota/routes").join(name))
                .expect("authored source")
        };
        let base = crystal_assets::build_river_valley_modpack(
            &source,
            &read("MN_R03.layout.json"),
            &read("MN_R03.json"),
        )
        .expect("stable R03 baseline");
        let layout = read("MN_R19.layout.json");
        let dossier = read("MN_R19.json");
        let pack = crystal_assets::build_prairie_forest_link_modpack(&base, &layout, &dossier)
            .expect("native prairie/forest route");
        (base, pack, layout, dossier)
    })
}

fn collision(pack: &CompiledGamePack, x: usize, y: usize) -> &str {
    let module = &pack.data().maps[&format!("MnR19C00R{:02}", y / 48)];
    let block = module.blocks[(y % 48) / 2 * 32 + x / 2];
    &pack.data().tilesets["johto"].collision[&format!("{block:02x}")][y % 2 * 2 + x % 2]
}

fn expect_design_rejection(result: Result<CompiledGamePack>, case: &str) {
    let error = result.expect_err(case);
    let message = format!("{error:#}");
    assert!(
        !["missing field", "invalid type", "unknown variant"]
            .iter()
            .any(|s| message.contains(s)),
        "{case} rejected JSON schema instead of the native contract: {message}"
    );
}

#[test]
fn prairie_forest_link_publishes_two_clipped_maps_and_preserves_source() {
    let (base, result, _, dossier) = inputs();
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 2);
    for (name, module) in &base.data().maps {
        assert_eq!(
            result.data().maps.get(name),
            Some(module),
            "changed map {name}"
        );
    }
    for (name, bytes) in base.runtime_files() {
        assert_eq!(
            result.runtime_files().get(name),
            Some(bytes),
            "changed source {name}"
        );
    }
    for (name, tileset) in &base.data().tilesets {
        assert_eq!(
            result.data().tilesets.get(name),
            Some(tileset),
            "changed art {name}"
        );
    }
    for (name, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            result.data().trainers.trainers.get(name),
            Some(trainer),
            "changed opponent {name}"
        );
    }
    for (name, encounter) in &base.data().wild_encounters {
        assert_eq!(
            result.data().wild_encounters.get(name),
            Some(encounter),
            "changed encounters {name}"
        );
    }
    for (name, metadata) in &base.data().runtime_map_metadata {
        assert_eq!(
            result.data().runtime_map_metadata.get(name),
            Some(metadata),
            "changed metadata {name}"
        );
    }
    assert_eq!(result.data().pokemon, base.data().pokemon);
    assert_eq!(result.data().items, base.data().items);
    assert_eq!(result.data().marts, base.data().marts);
    assert_eq!(
        result.data().runtime_spawn_points,
        base.data().runtime_spawn_points
    );
    assert_eq!(
        result.data().story_event_script_constants.global["SPAWN_HOME"],
        base.data().story_event_script_constants.global["SPAWN_HOME"]
    );
    assert_eq!(result.compiled_audio(), base.compiled_audio());
    assert_eq!(
        result.audio_manifest().expect("result audio manifest"),
        base.audio_manifest().expect("source audio manifest")
    );

    assert_eq!(
        result.data().tilesets,
        base.data().tilesets,
        "no art extension is required"
    );
    assert_eq!(result.data().moves, base.data().moves);
    let source_globals = base
        .data()
        .global_scripts
        .as_ref()
        .expect("source definitions");
    let target_globals = result
        .data()
        .global_scripts
        .as_ref()
        .expect("retained native globals");
    assert_eq!(
        source_globals.definitions, target_globals.definitions,
        "native source bodies must remain unchanged"
    );
    for root in ["HeadbuttFromMenuScript", "HeadbuttScript"] {
        assert_eq!(
            result.data().compiled_script_body(root),
            source_globals.definitions.get(root)
        );
    }
    let source_catalog = base
        .data()
        .story_events
        .iter()
        .find_map(|p| p.get("StandardScripts"))
        .expect("native source catalog");
    let target_catalog = result
        .data()
        .story_events
        .iter()
        .find_map(|p| p.get("StandardScripts"))
        .expect("retained source catalog");
    for (label, body) in source_catalog.as_object().expect("source catalog object") {
        if label == "GlobalScriptRoots" {
            let prior = body.as_array().expect("original retained roots");
            assert!(
                target_catalog[label]
                    .as_array()
                    .expect("retained roots")
                    .starts_with(prior)
            );
        } else {
            assert_eq!(target_catalog.get(label), Some(body), "source body {label}");
        }
    }

    assert_eq!(result.data().field_moves, base.data().field_moves);
    for (name, table) in &base.data().field_encounters {
        assert_eq!(
            result.data().field_encounters.get(name),
            Some(table),
            "source field table {name}"
        );
    }
    assert_eq!(
        result.data().field_encounters.len(),
        base.data().field_encounters.len() + 1
    );
    assert!(!result.data().field_encounters.contains_key("MnR19C00R01"));
    let mut totals = [0, 0, 0, 0];
    for row in 0..2 {
        let name = format!("MnR19C00R{row:02}");
        let module = &result.data().maps[&name];
        assert_eq!(
            (module.attributes.width, module.attributes.height),
            (32, if row == 0 { 24 } else { 6 })
        );
        assert_eq!(module.attributes.tileset_name, "johto");
        assert_eq!(
            result.data().runtime_map_metadata[module
                .attributes
                .map_constant
                .as_deref()
                .expect("native map constant")]
            .group_id,
            59
        );
        assert_eq!(
            module.attributes.music,
            result.data().maps["Route29"].attributes.music
        );
        assert_eq!(
            result
                .data()
                .assemble_map_module_from_compiled_payloads(&name)
                .expect("native split payload"),
            *module
        );
        assert!(result.data().wild_encounters.contains_key(&name));
        assert_eq!(module.attributes.connections.len(), 1);
        let back = &result.data().maps[&module.attributes.connections[0].target_map]
            .attributes
            .connections[0];
        assert_eq!(back.target_map, name);
        assert_eq!(back.offset, 0);
        assert!(module.events.warps.is_empty());
        totals[0] += module.trainer_scripts.len();
        totals[1] += module.script_field_pickups.len();
        totals[2] += module.events.bg_events.len();
        totals[3] += module.objects.len();
    }
    assert_eq!(totals, [6, 4, 6, 12]);
    let design: Value = serde_json::from_str(dossier).expect("dossier");
    let mut opponents = 0;
    for key in ["trainers", "residents", "items"] {
        for authored in design[key].as_array().expect("actors") {
            let id = authored["id"].as_str().expect("ID");
            let x = authored["tile"][0].as_u64().expect("x");
            let y = authored["tile"][1].as_u64().expect("y");
            let module = &result.data().maps[&format!("MnR19C00R{:02}", y / 48)];
            let actor = module
                .objects
                .iter()
                .find(|a| a.script == id)
                .expect("authored actor");
            assert_eq!((u64::from(actor.x), u64::from(actor.y)), (x, y % 48));
            if key == "items" {
                assert_eq!(
                    actor.event_flag,
                    authored["flag"].as_str().expect("pickup flag")
                );
            }
            if key == "trainers" {
                let expected: Vec<_> = authored["party"]
                    .as_array()
                    .expect("party")
                    .iter()
                    .map(|p| {
                        (
                            p["species"].as_str().expect("species"),
                            p["level"].as_u64().expect("level"),
                        )
                    })
                    .collect();
                let actual: Vec<_> = result.data().trainers.trainers[id]
                    .party
                    .iter()
                    .map(|p| (p.species.as_str(), u64::from(p.level)))
                    .collect();
                assert_eq!(actual, expected);
                opponents += actual.len();
                assert_eq!(
                    module.trainer_scripts[id].event_flag,
                    authored["defeated_flag"].as_str().expect("defeat flag")
                );
            }
        }
    }
    assert_eq!(opponents, 18);
    for x in 22..28 {
        for y in 48..52 {
            assert_eq!(collision(result, x, y), "TALL_GRASS");
        }
    }
    for (x, y) in [
        (11, 35),
        (11, 36),
        (11, 37),
        (11, 38),
        (19, 35),
        (19, 36),
        (19, 37),
    ] {
        assert_eq!(collision(result, x, y), "FLOOR");
    }
    assert_eq!(collision(result, 19, 38), "TALL_GRASS");
    for y in 0..60 {
        for x in 0..64 {
            let kind = collision(result, x, y);
            assert_ne!(kind, "CUT_TREE", "R19 has no declared Cut mechanic");
            if x == 0 || x == 63 || y == 0 || y == 59 {
                assert!(!kind.starts_with("WATER"), "outer water at {x},{y}");
            }
        }
    }
    let globals = &result.data().story_event_script_constants.global;
    for (name, value) in &base.data().story_event_script_constants.global {
        assert_eq!(globals.get(name), Some(value), "changed global {name}");
    }
    let events: BTreeSet<_> = LAKE_OBSERVATION_EVENTS
        .map(|name| globals[name])
        .into_iter()
        .collect();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|id| {
        !base
            .data()
            .story_event_script_constants
            .global
            .iter()
            .any(|(name, value)| name.starts_with("EVENT_") && value == id)
    }));
    assert_eq!(
        globals.get(REWARDED),
        base.data()
            .story_event_script_constants
            .global
            .get(REWARDED),
        "route must not allocate completion for the unauthored hand-in"
    );
    let module = &result.data().maps[MAP];
    for (sign, event) in SIGNS.into_iter().zip(LAKE_OBSERVATION_EVENTS) {
        assert_eq!(
            module.scripts[sign],
            design["quest_contract"]["script_labels"][sign]
        );
        assert_eq!(module.scripts[sign][2]["command"], "waitbutton");
        assert_eq!(module.scripts[sign][3]["command"], "closetext");
        assert!(
            module
                .script_flag_commands
                .iter()
                .any(|c| c.source_script == sign
                    && c.command_index == 4
                    && c.command == "setevent"
                    && c.flag_id == event)
        );
        assert!(
            module
                .script_text_bodies
                .contains_key(&format!("{sign}Text"))
        );
    }
    for module in result
        .data()
        .maps
        .values()
        .filter(|m| m.id.starts_with("MnR19"))
    {
        assert!(
            !module
                .scripts
                .contains_key("MN_DETROIT_LAKE_OBSERVATION_RANGER")
        );
        assert!(
            !module
                .script_flag_commands
                .iter()
                .any(|c| c.flag_id == REWARDED)
        );
        assert!(
            module.script_item_grants.is_empty(),
            "observations cannot hand out the town reward"
        );
    }
    assert!(
        crystal_assets::build_prairie_forest_link_modpack(result, &inputs().2, dossier).is_err()
    );
}

#[test]
fn prairie_observation_native_flag_mutations_are_idempotent_and_survive_save() {
    let pack = &inputs().1;
    let data = pack.data();
    let mut state = GameState::reset_wram_for_new_game();
    state.overworld = OverworldMemory::Active {
        map_name: MAP.into(),
        tile: TilePosition::new(55, 14),
        facing: Direction::Left,
        mode: MovementMode::Normal,
    };
    let mut session = data
        .overworld_session(MAP, TilePosition::new(55, 14), 0)
        .expect("actual sign approach component");
    let bag = state.bag.clone();
    for (sign, event) in SIGNS.into_iter().zip(LAKE_OBSERVATION_EVENTS) {
        // This is native mutation-component evidence, not a dialogue driver.
        // The production controller must separately prove page acknowledgment.
        let before = state.flags.clone();
        assert!(
            data.apply_script_flag_mutation(&mut state, &mut session, MAP, MAP, sign, 2)
                .is_err(),
            "waitbutton cannot masquerade as the flag command"
        );
        assert_eq!(state.flags, before);
        data.apply_script_flag_mutation(&mut state, &mut session, MAP, MAP, sign, 4)
            .expect("native setevent after authored acknowledgment edge");
        assert_eq!(state.flags.is_event_flag_set(event), Ok(true));
        data.apply_script_flag_mutation(&mut state, &mut session, MAP, MAP, sign, 4)
            .expect("repeat observation is idempotent");
    }
    assert_eq!(
        state.bag, bag,
        "signs never deliver the pending Detroit reward"
    );
    let hash = pack
        .identity()
        .expect("actual native identity")
        .content_hash
        .clone();
    let identity = SaveModpackIdentity::new("prairie-observation-component", hash.clone())
        .expect("component save identity");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("save clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "geothite-prairie-observations-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir(&dir).expect("owned save directory");
    let path = dir.join("notes.crystalsave");
    write_save_game_for_modpack(&path, state.clone(), &identity, &hash)
        .expect("native save writer");
    let loaded = read_save_game_for_modpack(&path, &identity, &hash).expect("native save reader");
    for event in LAKE_OBSERVATION_EVENTS {
        assert_eq!(loaded.state().flags.is_event_flag_set(event), Ok(true));
    }
    assert_eq!(loaded.state().bag, state.bag);
    assert_eq!(loaded.state().overworld, state.overworld);
    fs::remove_dir_all(dir).expect("remove owned component save");
}

#[test]
fn prairie_headbutt_uses_source_buckets_rng_and_native_tree_battle_origin() {
    let (base, pack, _, _) = inputs();
    // Reload uses this same production materializer. The additive retained-root
    // list must rebuild executable Headbutt tables without in-memory injection.
    let mut reloaded_data = pack.data().clone();
    reloaded_data
        .materialize_global_scripts()
        .expect("rebuild native global tables from retained source roots");
    let data = &reloaded_data;
    let source = &base.data().field_encounters["Route29"];
    let target = &data.field_encounters[MAP];
    let mut expected = source
        .table(FieldEncounterKind::Headbutt)
        .expect("source Headbutt")
        .clone();
    for entry in expected.common.iter_mut().chain(&mut expected.rare) {
        entry.level = 28;
    }
    assert_eq!(target.map_name, MAP);
    assert_eq!(target.tables.len(), 1);
    assert_eq!(target.table(FieldEncounterKind::Headbutt), Some(&expected));
    for player in 0..10 {
        for roll in 0..100 {
            let a = select_headbutt_encounter(source, 31, 20, player, 0, roll)
                .expect("native source selection");
            let b = select_headbutt_encounter(target, 31, 20, player, 0, roll)
                .expect("native route selection");
            let a = a.resolved.expect("chance0 hits every tree score");
            let b = b.resolved.expect("route hit");
            assert_eq!(a.encounter.species, b.encounter.species);
            assert_eq!(a.slot, b.slot);
            assert_eq!(b.encounter.level, 28);
            assert_eq!(b.level, 28);
        }
    }
    assert!(collision(pack, 31, 20).starts_with("HEADBUTT_TREE"));
    assert_eq!(collision(pack, 32, 20), "FLOOR");
    let mut state = GameState {
        player_id: 4,
        ..GameState::default()
    };
    state
        .script_runtime
        .memory
        .insert("wBattleScriptFlags".into(), "0".into());
    let mut session = data
        .overworld_session(MAP, TilePosition::new(32, 20), 0)
        .expect("actual oak approach component");
    session.player.facing = Direction::Left;
    let command = RuntimeScriptCommandRef::new(MAP, "HeadbuttScript", 4);
    let mut divider = ReplayDivider::new([255, 0, 53, 0]);
    let outcome = data
        .resolve_tree_mon_encounter(&mut state, &session, &command, &mut divider)
        .expect("native exact TreeMonEncounter source edge");
    assert_eq!(
        (outcome.roll.target_tile_x, outcome.roll.target_tile_y),
        (31, 20)
    );
    assert_eq!(outcome.roll.kind, FieldEncounterKind::Headbutt);
    assert_eq!(outcome.roll.score, Some(0));
    assert_eq!(outcome.roll.entry_roll, Some(54));
    assert_eq!(divider.consumed(), 4);
    let found = outcome.roll.resolved.expect("source rare Pineco roll");
    assert_eq!(found.encounter.species, "PINECO");
    assert_eq!(found.encounter.level, 28);
    let mut battle_divider = ReplayDivider::new([0; 32]);
    let start = data
        .start_scripted_wild_battle(
            &mut state,
            MAP,
            MAP,
            "HeadbuttScript",
            8,
            &mut battle_divider,
        )
        .expect("native dynamic tree battle start");
    assert_eq!(start.battle_type, "BATTLETYPE_TREE");
    assert_eq!(start.enemy_pokemon.species.id, "PINECO");
    assert_eq!(start.enemy_pokemon.level, 28);
    assert!(data.saved_static_wild_battle_origin_exists(
        MAP,
        "HeadbuttScript",
        8,
        9,
        "BATTLETYPE_TREE",
        "PINECO",
        28
    ));
    assert!(
        !data.saved_static_wild_battle_origin_exists(
            MAP,
            "HeadbuttScript",
            8,
            9,
            "BATTLETYPE_TREE",
            "PINECO",
            10
        ),
        "source Route29 level must not validate as a route encounter"
    );
}

#[test]
fn prairie_forest_link_rejects_changed_observations_headbutt_and_terrain() {
    let (base, _, layout, dossier) = inputs();
    for (name, pointer, value) in [
        (
            "wrong observation flag",
            "/quest_contract/observations/0/flag",
            json!("EVENT_MN_LAKE_OBSERVATION_02"),
        ),
        (
            "completion set at sign",
            "/quest_contract/script_labels/MN_R19_OBSERVE_PINE/4/args/0",
            json!(REWARDED),
        ),
        (
            "event before acknowledgment",
            "/quest_contract/script_labels/MN_R19_OBSERVE_PINE/2/command",
            json!("setevent"),
        ),
        (
            "unplanned reward map",
            "/quest_contract/reward_map",
            json!(MAP),
        ),
        (
            "altered source table",
            "/headbutt_contract/source_table_map",
            json!("Route30"),
        ),
        (
            "unplanned Headbutt map",
            "/headbutt_contract/map_ids/0",
            json!("MnR19C00R01"),
        ),
        (
            "unplanned level",
            "/headbutt_contract/level_override",
            json!(27),
        ),
        (
            "changed native weights policy",
            "/headbutt_contract/preserve_native_weights_rare_tables_and_time_sleep_rules",
            json!(false),
        ),
        ("wrong sixth badge", "/required_flags", json!(["BADGE_5"])),
    ] {
        let mut changed: Value = serde_json::from_str(dossier).expect("dossier");
        *changed.pointer_mut(pointer).expect("field") = value;
        expect_design_rejection(
            crystal_assets::build_prairie_forest_link_modpack(base, layout, &changed.to_string()),
            name,
        );
    }
    for (name, layer) in [
        (
            "main habitat paved",
            json!({"kind":"rect","material":"floor","bounds":[11,24,14,26]}),
        ),
        (
            "ordinary ramp sealed",
            json!({"kind":"point","position":[5,18],"block":"0a"}),
        ),
        (
            "incidental Cut canopy",
            json!({"kind":"point","position":[1,1],"block":"63"}),
        ),
    ] {
        let mut changed: Value = serde_json::from_str(layout).expect("layout");
        let mut layer = layer;
        layer["name"] = json!(name);
        changed["layers"]
            .as_array_mut()
            .expect("layers")
            .push(layer);
        expect_design_rejection(
            crystal_assets::build_prairie_forest_link_modpack(base, &changed.to_string(), dossier),
            name,
        );
    }
}

#[test]
fn prairie_headbutt_accepts_exact_source_export_forms_and_rejects_tampered_edges() {
    let pack = &inputs().1;
    let native = pack
        .data()
        .compiled_script_body("HeadbuttScript")
        .expect("retained source body");
    let commands = native.as_array().expect("native script commands");
    assert_eq!(
        commands.len(),
        15,
        "published supplied source must remain untrimmed"
    );
    let local = &pack
        .data()
        .global_scripts
        .as_ref()
        .expect("native globals")
        .definitions[".no_battle@HeadbuttScript"];
    assert_eq!(
        &commands[11..],
        local.as_array().expect("qualified native tail")
    );
    let command = RuntimeScriptCommandRef::new(MAP, "HeadbuttScript", 4);
    for length in [11, 15] {
        // Explicit source-shape component fixture only. The published pack and
        // original catalog remain unchanged; both forms use the same parser.
        let mut data = pack.data().clone();
        let catalog = data
            .story_events
            .iter_mut()
            .find_map(|p| p.get_mut("OverworldEvents"))
            .expect("native OverworldEvents definitions");
        catalog["HeadbuttScript"]
            .as_array_mut()
            .expect("source body")
            .truncate(length);
        data.materialize_global_scripts()
            .expect("parse exact split/flattened native source form");
        assert!(
            data.is_exact_tree_mon_encounter_command(&command)
                .expect("exact native certificate"),
            "source form length {length}"
        );
    }
    for (case, label, index, replacement) in [
        (
            "flattened local tail",
            "HeadbuttScript",
            11,
            "UseHeadbuttText",
        ),
        (
            "qualified local tail",
            ".no_battle@HeadbuttScript",
            0,
            "UseHeadbuttText",
        ),
        (
            "success encounter edge",
            "HeadbuttScript",
            4,
            "RockMonEncounter",
        ),
    ] {
        let mut data = pack.data().clone();
        let catalog = data
            .story_events
            .iter_mut()
            .find_map(|p| p.get_mut("OverworldEvents"))
            .expect("native OverworldEvents definitions");
        catalog[label][index]["args"][0] = json!(replacement);
        data.materialize_global_scripts()
            .expect("valid native command schema in tampered fixture");
        let result = data.is_exact_tree_mon_encounter_command(&command);
        assert!(
            !matches!(result, Ok(true)),
            "tampered {case} must fail its exact native certificate"
        );
    }
    assert_eq!(
        pack.data().compiled_script_body("HeadbuttScript"),
        Some(native)
    );
}
