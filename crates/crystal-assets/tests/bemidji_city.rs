//! Native city publication contracts; production controller play remains separate.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, ContentPackCategory, GameDataSet, RuntimeMutationCommand,
    RuntimeMutationResult, build_bemidji_city_modpack,
};
use crystal_core::models::{Dv, LearnedMove};
use crystal_core::state::GameState;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::session::OverworldSession;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

struct Inputs {
    base: CompiledGamePack,
    town: String,
    layout: String,
    gyms: String,
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonical repository root");
        let read =
            |path: &str| std::fs::read_to_string(root.join(path)).expect("authored source file");
        let gyms = std::env::var_os("CRYSTAL_BEMIDJI_CITY_GYMS_SOURCE")
            .map(|path| std::fs::read_to_string(path).expect("captured canonical gym JSON"))
            .unwrap_or_else(|| read("planning/minnesota/gyms/interiors.json"));
        let base = if let Some(path) = std::env::var_os("CRYSTAL_BEMIDJI_CITY_BASE_PACK") {
            crystal_assets::read_verified_compiled_game_pack(
                PathBuf::from(path)
                    .canonicalize()
                    .expect("canonical supplied gym pack path"),
            )
            .expect("supplied completed canonical gym pack")
        } else {
            let pack = crystal_assets::read_verified_compiled_game_pack(
                root.join("content-packs/text-tui.crystalpack"),
            )
            .expect("completed external TUI source pack");
            crystal_assets::build_minnesota_gym_review_pack(&pack, &gyms)
                .expect("install actual canonical gyms as fixture setup, without publishing a file")
        };
        Inputs {
            base,
            town: std::env::var_os("CRYSTAL_BEMIDJI_CITY_TOWN_SOURCE")
                .map(|path| std::fs::read_to_string(path).expect("captured city dossier JSON"))
                .unwrap_or_else(|| read("planning/minnesota/towns/MnBemidji.json")),
            layout: read("planning/minnesota/towns/MnBemidji.layout.json"),
            gyms,
        }
    })
}
fn published() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let i = inputs();
        build_bemidji_city_modpack(&i.base, &i.town, &i.layout, &i.gyms)
            .expect("authored native Bemidji")
    })
}
fn reject(town: &Value, layout: &Value, needle: &str) {
    let i = inputs();
    let error =
        build_bemidji_city_modpack(&i.base, &town.to_string(), &layout.to_string(), &i.gyms)
            .expect_err("invalid city contract must fail");
    assert!(
        format!("{error:#}").contains(needle),
        "wanted {needle:?}, got {error:#}"
    );
}

#[test]
fn bemidji_city_preserves_original_maps_art_audio_flags_and_home_except_declared_gym_returns() {
    let i = inputs();
    let pack = published();
    assert_eq!(pack.data().maps.len(), i.base.data().maps.len() + 6);
    assert_eq!(pack.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("published audio metadata"),
        i.base.audio_manifest().expect("source audio metadata")
    );
    assert_eq!(pack.data().trainers, i.base.data().trainers);
    assert_eq!(pack.data().items, i.base.data().items);
    assert_eq!(pack.data().wild_encounters, i.base.data().wild_encounters);
    for (path, bytes) in i.base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(path),
            Some(bytes),
            "changed original runtime asset {path}"
        );
    }
    for (key, spawn) in &i.base.data().runtime_spawn_points {
        assert_eq!(
            pack.data().runtime_spawn_points.get(key),
            Some(spawn),
            "original HOME/recovery changed"
        );
    }
    assert_eq!(
        pack.data().runtime_spawn_points.len(),
        i.base.data().runtime_spawn_points.len() + 2
    );
    for (name, module) in &i.base.data().maps {
        if name == "MnBemidjiGym" {
            continue;
        }
        assert_eq!(
            pack.data().maps.get(name),
            Some(module),
            "unrelated map changed: {name}"
        );
    }
    for (key, value) in &i.base.data().story_event_script_constants.global {
        assert_eq!(
            pack.data().story_event_script_constants.global.get(key),
            Some(value),
            "original flag/constant changed: {key}"
        );
    }
    let before = &i.base.data().maps["MnBemidjiGym"];
    let after = &pack.data().maps["MnBemidjiGym"];
    let mut expected = serde_json::to_value(before).expect("gym source");
    let actual = serde_json::to_value(after).expect("gym result");
    expected["events"]["warps"] = actual["events"]["warps"].clone();
    expected["map_event_section_commands"] = actual["map_event_section_commands"].clone();
    assert_eq!(
        expected, actual,
        "only native warp records/wire data may change"
    );
    assert_eq!(after.events.warps.len(), 3);
    for warp in &after.events.warps {
        assert_eq!(warp.target_map_constant, "MN_BEMIDJI");
        assert_eq!(warp.target_warp_id, 2);
    }
    let events_label = after
        .attributes
        .map_events_label
        .as_deref()
        .expect("Gym7 events label");
    let mut expected_mirror = i.base.data().map_scripts[events_label].clone();
    let mut changed_warps = 0;
    for command in expected_mirror
        .as_array_mut()
        .expect("source event section")
    {
        if command["command"] == "warp_event" {
            let args = command["args"]
                .as_array_mut()
                .expect("source warp arguments");
            assert_eq!(args.len(), 4);
            args[2] = json!("MN_BEMIDJI");
            args[3] = json!("2");
            changed_warps += 1;
        }
    }
    assert_eq!(changed_warps, 3, "exactly three canonical return records");
    assert_eq!(
        pack.data().map_scripts[events_label],
        expected_mirror,
        "global event mirror may change only the three warp target arguments"
    );
    let actual_wire = Value::Array(
        after
            .map_event_section_commands
            .iter()
            .map(|command| json!({"command": command.command, "args": command.args}))
            .collect(),
    );
    assert_eq!(
        pack.data().map_scripts[events_label],
        actual_wire,
        "written split catalog must match actual module event wire"
    );
    for (label, payload) in &i.base.data().map_scripts {
        if label != events_label {
            assert_eq!(
                pack.data().map_scripts.get(label),
                Some(payload),
                "unrelated original split script changed: {label}"
            );
        }
    }
    for (name, payload) in &i.base.data().npcs {
        assert_eq!(
            pack.data().npcs.get(name),
            Some(payload),
            "original NPC split payload changed: {name}"
        );
    }
    // Force the split-source path instead of the cached-module fast path.
    let mut split_data = pack.data().clone();
    split_data.maps.remove("MnBemidjiGym");
    let reassembled = split_data
        .assemble_map_module_from_compiled_payloads("MnBemidjiGym")
        .expect("reassemble actual published split gym payloads");
    assert_eq!(
        reassembled.events.warps, after.events.warps,
        "split-payload reload must preserve all three city-bound returns"
    );
    assert_eq!(
        reassembled.map_event_section_commands,
        after.map_event_section_commands
    );
    assert_eq!(pack.data().maps["MnBemidji"].events.warps.len(), 12);
}

#[test]
fn bemidji_city_lowers_real_healing_shop_pc_and_keeps_q05_unbound() {
    let pack = published();
    let data = pack.data();
    let clinic = &data.maps["MnBemidjiPokecenter1F"];
    for (name, approach, facing, script) in [
        (
            "MnBemidjiPokecenter1F",
            [7, 5],
            Direction::Up,
            "MN_BEMIDJI_NURSE",
        ),
        (
            "MnBemidjiPokecenter1F",
            [13, 4],
            Direction::Up,
            "MnBemidjiClinicPc",
        ),
        ("MnBemidjiMart", [3, 3], Direction::Left, "MN_BEMIDJI_CLERK"),
    ] {
        let module = &data.maps[name];
        let mut session = OverworldSession::with_events_and_objects(
            data.overworld_map(name).expect("service map"),
            module.events.clone(),
            module.objects.clone(),
            data.tileset_collision(&module.attributes.tileset_name)
                .expect("service collision"),
            TilePosition::new(approach[0], approach[1]),
        );
        session.set_player_facing(facing);
        assert_eq!(
            session
                .check_interaction_checked(1)
                .expect("native counter/PC lookup")
                .expect("reachable native interaction")
                .script,
            script
        );
    }
    let nurse = clinic
        .script_runtime_commands
        .iter()
        .find(|c| c.source_script == "MN_BEMIDJI_NURSE" && c.command == "special")
        .expect("parsed nurse special");
    assert_eq!(nurse.args, ["HealParty"]);
    let mut state = GameState::reset_wram_for_new_game();
    let mut mon = data
        .create_pokemon("MEW", 45, Dv::default())
        .expect("legal partner fixture");
    mon.hp = 1;
    mon.status = Some("POISON".into());
    for movement in &mut mon.moves {
        movement.current_pp = 0;
    }
    assert!(state.storage.party.add_pokemon(mon));
    state.sync_party_from_storage();
    state.money = 10000;
    let resources = (state.money, state.bag.clone(), state.flags.clone());
    data.apply_special_routine(&mut state, &nurse.args[0], &BTreeSet::new())
        .expect("production HealParty mutation");
    let healed = state.storage.party.pokemon[0]
        .as_ref()
        .expect("rested partner");
    assert_eq!(healed.hp, healed.max_hp);
    assert!(healed.status.is_none());
    assert!(healed.moves.iter().all(|m| m.current_pp > 0));
    assert_eq!(
        (state.money, state.bag.clone(), state.flags.clone()),
        resources
    );
    let clerk = data.maps["MnBemidjiMart"]
        .script_shop_commands
        .iter()
        .find(|c| c.source_script == "MN_BEMIDJI_CLERK")
        .expect("actual native mart");
    data.open_script_shop(
        &mut state,
        "MnBemidjiMart",
        "MnBemidjiMart",
        &clerk.source_script,
        clerk.command_index,
    )
    .expect("native mart open");
    let price = data.items["HYPER_POTION"].price;
    assert!(
        data.buy_shop_item(&mut state, "HYPER_POTION", 1)
            .expect("native buy")
            .success
    );
    assert_eq!(state.money, 10000 - u32::from(price));
    assert_eq!(state.bag.quantity(&data.items["HYPER_POTION"]), 1);
    state.money = 0;
    let before = (state.money, state.bag.clone());
    assert!(
        !data
            .buy_shop_item(&mut state, "HYPER_POTION", 1)
            .expect("insufficient money")
            .success
    );
    assert_eq!((state.money, state.bag.clone()), before);
    assert!(
        !clinic.script_object_commands.is_empty(),
        "facing/wait behavior must be lowered"
    );
    assert!(
        clinic
            .events
            .bg_events
            .iter()
            .any(|b| b.script == "MnBemidjiClinicPc" && [b.x, b.y] == [13, 3])
    );
    assert_eq!(
        data.marts.0["MART_MN_BEMIDJI"],
        [
            "ULTRA_BALL",
            "HYPER_POTION",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL"
        ]
    );
    assert!(!data.maps["MnBemidjiMart"].script_shop_commands.is_empty());
    let officer = data.maps["MnBemidji"]
        .scripts
        .get("MN_BEMIDJI_OFFICER")
        .expect("source officer ordinary speech");
    assert_eq!(officer.as_array().expect("ordinary script").len(), 1);
    assert_eq!(
        officer[0]["command"], "jumptextfaceplayer",
        "no replacement story scene or report flag grant"
    );
    for map in [
        "MnBemidji",
        "MnBemidjiPokecenter1F",
        "MnBemidjiMart",
        "MnBemidjiShelfHouse",
        "MnBemidjiNorthHouse",
        "MnBemidjiHeritageHouse",
    ] {
        assert!(
            data.maps[map].attributes.connections.is_empty(),
            "regional seams remain explicitly unbound"
        );
    }
}

#[test]
fn bemidji_city_rejects_false_doors_unsafe_returns_open_lake_and_art_drift() {
    let i = inputs();
    let d: Value = serde_json::from_str(&i.town).expect("city source");
    let l: Value = serde_json::from_str(&i.layout).expect("layout source");
    let mut bad = d.clone();
    bad["doors"][0]["tile"] = json!([11, 9]);
    reject(&bad, &l, "unbound or false");
    let mut bad = d.clone();
    bad["doors"][0]["return_tile"] = json!([10, 9]);
    reject(&bad, &l, "unsafe actual door");
    let mut bad = l.clone();
    bad["layers"].as_array_mut().expect("strokes").push(json!({"name":"negative lake boundary hole","kind":"point","position":[23,20],"block":"35"}));
    reject(&d, &bad, "Surf outer boundary");
    let mut bad = l.clone();
    bad["derived_blocks"][0]["source_block"] = json!("24");
    reject(&d, &bad, "exact native Bemidji timber");
    let mut bad = d.clone();
    bad["interiors"][2]["canonical_gym_sha256"] = json!("stale");
    reject(&bad, &l, "stale city canonical");
}

#[test]
fn bemidji_city_rejects_runtime_gym_drift_and_occupied_group_without_mutating_base() {
    let i = inputs();
    let before = serde_json::to_value(&i.base).expect("base snapshot");
    let mut gyms: Value = serde_json::from_str(&i.gyms).expect("canonical gyms");
    let source = gyms["gyms"]
        .as_array_mut()
        .expect("all gyms")
        .iter_mut()
        .find(|g| g["id"] == "MnBemidjiGym")
        .expect("Gym7");
    source["guide"] = json!(["Changed gym guide."]);
    let hash = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(source).expect("canonical object"))
    );
    let mut town: Value = serde_json::from_str(&i.town).expect("city source");
    town["interiors"][2]["canonical_gym_sha256"] = json!(hash);
    let error =
        build_bemidji_city_modpack(&i.base, &town.to_string(), &i.layout, &gyms.to_string())
            .expect_err("installed module must match canonical source");
    assert!(
        format!("{error:#}").contains("installed Gym7 module"),
        "wrong rejection: {error:#}"
    );
    let error = build_bemidji_city_modpack(published(), &i.town, &i.layout, &i.gyms)
        .expect_err("occupied city group must reject repeat installation");
    assert!(format!("{error:#}").contains("group70 occupied"));
    assert_eq!(
        before,
        serde_json::to_value(&i.base).expect("unchanged borrowed base")
    );
}

#[test]
fn bemidji_addon_recovery_runs_native_teleport_blackout_and_preserves_source_fly_range() {
    let pack = published();
    let data = pack.data();
    let audio = pack.audio_manifest().expect("native audio catalogs");
    let music = audio.music.keys().cloned().collect::<BTreeSet<_>>();
    let effects = audio.sound_effects.keys().cloned().collect::<BTreeSet<_>>();
    let cries = audio.cries.keys().cloned().collect::<BTreeSet<_>>();
    let town = &data.maps["MnBemidji"];
    let make_session = || {
        OverworldSession::with_events_and_objects(
            data.overworld_map("MnBemidji").expect("native city"),
            town.events.clone(),
            town.objects.clone(),
            data.tileset_collision(&town.attributes.tileset_name)
                .expect("native city collision"),
            TilePosition::new(21, 22),
        )
    };
    let mut initial = GameState::reset_wram_for_new_game();
    let mut abra = data
        .create_pokemon("ABRA", 10, Dv::default())
        .expect("legal Teleport actor");
    abra.moves = vec![LearnedMove {
        name: "TELEPORT".into(),
        current_pp: 20,
        pp_ups: 0,
    }];
    assert!(initial.storage.party.add_pokemon(abra));
    initial.sync_party_from_storage();
    initial.money = 10000;
    let original_home = inputs()
        .base
        .data()
        .runtime_spawn_point(0)
        .expect("original HOME");
    assert_eq!(
        data.runtime_spawn_point(0).expect("retained HOME"),
        original_home
    );
    assert_eq!(data.fly_destinations, inputs().base.data().fly_destinations);
    for invalid in [28, 255] {
        let error = data
            .runtime_spawn_point(invalid)
            .expect_err("source byte gap stays invalid");
        assert!(error.to_string().contains("reserved gap"));
    }
    for (constant, name, expected_tile) in [
        ("MN_BEMIDJI", "MnBemidji", TilePosition::new(31, 10)),
        (
            "MN_BEMIDJI_POKECENTER_1F",
            "MnBemidjiPokecenter1F",
            TilePosition::new(10, 13),
        ),
    ] {
        let spawn = data
            .runtime_spawn_point_for_map_constant(constant)
            .expect("addon recovery row");
        assert!(spawn.identifier >= 256);
        assert_eq!(
            data.runtime_spawn_point(spawn.identifier)
                .expect("native lookup"),
            spawn
        );
        data.validate_saved_spawn_reference("city.recovery", spawn.identifier)
            .expect("saved exact addon catalog identity");
        let mut state = initial.clone();
        state.last_spawn_map_constant = Some(constant.into());
        let mut session = make_session();
        let before = session.snapshot();
        let prepared = data
            .use_teleport_field_move_in_session(&mut state, &mut session, 0, &music)
            .expect("prepare actual native Teleport");
        assert_eq!(prepared.destination_spawn_identifier, spawn.identifier);
        assert_eq!(prepared.destination_map, name);
        assert_eq!(prepared.destination_tile, expected_tile);
        assert_eq!(
            session.snapshot(),
            before,
            "travel waits for native commit boundary"
        );
        data.commit_pending_field_travel(&mut state, &mut session, &music)
            .expect("commit actual native Teleport");
        assert_eq!(session.map.name, name);
        assert_eq!(session.player.tile, expected_tile);
        assert!(state.script_runtime.pending_field_travel.is_none());
        assert_eq!(state.last_spawn_map_constant.as_deref(), Some(constant));
        assert_eq!(state.money, initial.money);
        assert_eq!(state.bag, initial.bag);

        let mut state = initial.clone();
        state.last_spawn_map_constant = Some(constant.into());
        state.storage.party.pokemon[0]
            .as_mut()
            .expect("fainted partner")
            .hp = 0;
        state.sync_party_from_storage();
        crystal_core::battle::start::deactivate_battle_after_loss(&mut state);
        state
            .flags
            .set_engine_flag("ENGINE_SAFARI_ZONE", true)
            .expect("Safari cleanup fixture");
        let mut session = make_session();
        let outcome = data
            .apply_runtime_mutation_command(
                &mut state,
                &mut session,
                RuntimeMutationCommand::ResolveBlackoutToLastSpawn,
                &music,
                &effects,
                &cries,
            )
            .expect("actual native blackout resolution");
        let RuntimeMutationResult::BlackoutResolved(recovery) = outcome.result else {
            panic!("expected native blackout result");
        };
        assert_eq!(recovery.spawn_identifier, Some(spawn.identifier));
        assert_eq!(recovery.map_name, name);
        assert_eq!(recovery.tile, expected_tile);
        assert_eq!(session.map.name, name);
        assert_eq!(session.player.tile, expected_tile);
        assert_eq!(state.last_spawn_map_constant.as_deref(), Some(constant));
        assert_eq!(state.money, 5000);
        assert_eq!(state.bag, initial.bag);
        let healed = state.storage.party.pokemon[0]
            .as_ref()
            .expect("recovered partner");
        assert_eq!(healed.hp, healed.max_hp);
        assert!(
            !state
                .flags
                .is_engine_flag_set("ENGINE_SAFARI_ZONE")
                .expect("cleanup flag")
        );

        // Some completed source packs intentionally contain no authored Fly
        // destination rows. Exercise the real parser with an isolated schema
        // fixture instead of inventing or installing a live destination.
        let fly_payload = |identifier| {
            json!({
                "ENGINE_FLYPOINT_NEW_BARK": {
                    "flypoint_flag": "ENGINE_FLYPOINT_NEW_BARK",
                    "destination_spawn_identifier": identifier,
                    "label": "LANDMARK_NEW_BARK_TOWN",
                }
            })
        };
        for original in [0_u16, 27] {
            GameDataSet::validate_authored_content_payloads(&[(
                ContentPackCategory::FlyDestinations,
                fly_payload(original),
            )])
            .expect("original source-range Fly parser positive control");
        }
        for forbidden in [28, 255, spawn.identifier, u16::MAX] {
            let error = GameDataSet::validate_authored_content_payloads(&[(
                ContentPackCategory::FlyDestinations,
                fly_payload(forbidden),
            )])
            .expect_err("Fly must retain source range even when addon recovery is valid");
            assert!(format!("{error:#}").contains("outside Crystal's SpawnPoints table"));
        }
    }
    let mut state = initial;
    state.last_spawn_map_constant = Some("NO_SPAWN_MAP".into());
    state.storage.party.pokemon[0]
        .as_mut()
        .expect("fainted partner")
        .hp = 0;
    state.sync_party_from_storage();
    crystal_core::battle::start::deactivate_battle_after_loss(&mut state);
    let mut session = make_session();
    let outcome = data
        .apply_runtime_mutation_command(
            &mut state,
            &mut session,
            RuntimeMutationCommand::ResolveBlackoutToLastSpawn,
            &music,
            &effects,
            &cries,
        )
        .expect("unknown recovery retains native HOME fallback");
    let RuntimeMutationResult::BlackoutResolved(recovery) = outcome.result else {
        panic!("expected native HOME recovery");
    };
    assert_eq!(recovery.spawn_identifier, Some(0));
    assert_eq!(recovery.map_name, original_home.map_name);
    assert_eq!(
        session.player.tile,
        TilePosition::new(original_home.tile_x, original_home.tile_y)
    );
}
