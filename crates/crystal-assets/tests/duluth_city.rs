//! Native asset/core contracts; these are not production controller journeys.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_duluth_city_modpack, build_minnesota_gym_review_pack,
    read_verified_compiled_game_pack,
};
use crystal_core::models::Dv;
use crystal_core::state::GameState;
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::{LedgeJumpOutcome, StepOptions, StepOutcome};
use crystal_core::world::session::OverworldSession;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}
fn source(name: &str) -> String {
    std::fs::read_to_string(root().join(name)).expect("authored local source")
}
fn captured_source(variable: &str, fallback: &str) -> String {
    std::env::var_os(variable)
        .map(|path| std::fs::read_to_string(path).expect("captured Duluth source input"))
        .unwrap_or_else(|| source(fallback))
}
fn inputs() -> &'static (CompiledGamePack, String, String, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let gyms = captured_source("CRYSTAL_DULUTH_GYMS_SOURCE", "planning/minnesota/gyms/interiors.json");
        let base = if let Some(path) = std::env::var_os("CRYSTAL_DULUTH_TEST_PACK") {
            read_verified_compiled_game_pack(path).expect("supplied completed canonical gym pack")
        } else {
            let base = read_verified_compiled_game_pack(root().join("content-packs/text-tui.crystalpack"))
                .expect("completed external TUI source pack");
            build_minnesota_gym_review_pack(&base, &gyms)
                .expect("canonical gym fixture built in memory from the same captured source; no pack publication")
        };
        (
            base,
            captured_source("CRYSTAL_DULUTH_CITY_SOURCE", "planning/minnesota/towns/MnDuluth.json"),
            captured_source("CRYSTAL_DULUTH_LAYOUT_SOURCE", "planning/minnesota/towns/MnDuluth.layout.json"),
            gyms,
        )
    })
}
fn published() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let (base, town, layout, gyms) = inputs();
        build_duluth_city_modpack(base, town, layout, gyms)
            .expect("native Duluth and canonical Gym8 rebinding")
    })
}
fn tile([x, y]: [u16; 2]) -> TilePosition {
    TilePosition::new(
        i16::try_from(x).expect("fixture x"),
        i16::try_from(y).expect("fixture y"),
    )
}
fn point(value: &Value) -> [u16; 2] {
    [
        u16::try_from(value[0].as_u64().expect("source x")).expect("native x"),
        u16::try_from(value[1].as_u64().expect("source y")).expect("native y"),
    ]
}

#[test]
fn duluth_city_preserves_existing_content_and_only_rebinds_canonical_gym_destinations() {
    let base = &inputs().0;
    let added = published();
    assert_eq!(added.data().maps.len(), base.data().maps.len() + 7);
    let mut visitors = BTreeSet::new();
    for name in ["MnDuluth", "MnDuluthPokecenter1F"] {
        let module = &added.data().maps[name];
        let visitor = module
            .objects
            .iter()
            .find(|actor| actor.object_identifier.as_deref() == Some("MN_DULUTH_CLINIC_VISITOR"))
            .expect("both authored clinic visitors retained with original map-local object ID");
        assert!(module.scripts.contains_key(&visitor.script));
        assert!(
            visitors.insert(visitor.script.clone()),
            "split-pack script labels must be globally unique"
        );
    }
    for (name, module) in &base.data().maps {
        if name != "MnDuluthGym" {
            assert_eq!(
                added.data().maps.get(name),
                Some(module),
                "unrelated native module {name}"
            );
        }
    }
    let mut expected = base.data().maps["MnDuluthGym"].clone();
    for warp in &mut expected.events.warps {
        warp.target_map_constant = "MN_DULUTH".into();
        warp.target_map = "MN_DULUTH".into();
        warp.target_warp_id = 4;
    }
    for command in &mut expected.map_event_section_commands {
        if command.command == "warp_event" {
            command.args[2] = "MN_DULUTH".into();
            command.args[3] = "4".into();
        }
    }
    assert_eq!(
        added.data().maps["MnDuluthGym"],
        expected,
        "every canonical puzzle/script/actor/party field must survive"
    );
    assert_eq!(
        added.data().runtime_spawn_points,
        base.data().runtime_spawn_points,
        "HOME and every recovery destination"
    );
    assert_eq!(added.data().tilesets, base.data().tilesets);
    assert_eq!(added.runtime_files(), base.runtime_files());
    assert_eq!(added.compiled_audio(), base.compiled_audio());
    assert_eq!(
        added.audio_manifest().expect("result manifest"),
        base.audio_manifest().expect("base manifest")
    );
    assert_eq!(added.data().audio, base.data().audio);
    assert_eq!(added.data().pokemon, base.data().pokemon);
    assert_eq!(added.data().items, base.data().items);
    assert_eq!(added.data().trainers, base.data().trainers);
    assert_eq!(added.data().wild_encounters, base.data().wild_encounters);
    for (key, value) in &base.data().story_event_script_constants.global {
        assert_eq!(
            added.data().story_event_script_constants.global.get(key),
            Some(value),
            "existing native constant {key}"
        );
    }
    for (key, value) in &base.data().marts.0 {
        assert_eq!(added.data().marts.0.get(key), Some(value));
    }
    assert_eq!(
        added.data().marts.0["MART_MN_DULUTH"],
        [
            "ULTRA_BALL",
            "HYPER_POTION",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL"
        ]
    );
    assert_eq!(
        added.data().maps["MnDuluth"].attributes.tileset_name,
        "johto_modern"
    );
    assert!(
        build_duluth_city_modpack(added, &inputs().1, &inputs().2, &inputs().3).is_err(),
        "duplicate group/map installation"
    );
}

#[test]
fn duluth_city_all_seven_doors_and_fourteen_carpet_exits_resolve_to_passive_floor() {
    let data = published().data();
    let source: Value = serde_json::from_str(&inputs().1).expect("town source");
    let session = |name: &str, at: [u16; 2]| {
        let module = data.map_module(name).expect("native module");
        OverworldSession::with_events(
            data.overworld_map(name).expect("native map"),
            module.events.clone(),
            data.tileset_collision(&module.attributes.tileset_name)
                .expect("native collision"),
            tile(at),
        )
    };
    for door in source["doors"].as_array().expect("seven doors") {
        let name = door["target"].as_str().expect("room name");
        let room = source["interiors"]
            .as_array()
            .expect("rooms")
            .iter()
            .find(|r| r["id"] == name)
            .expect("paired room");
        let arrival = point(&room["passive_arrival"]["tile"]);
        let outside = point(&door["return_tile"]);
        let trigger = session("MnDuluth", point(&door["tile"]))
            .check_warp_checked()
            .expect("native door check")
            .expect("actual DOOR");
        let result = data
            .resolve_warp_transition(&trigger)
            .expect("native room entry");
        assert_eq!(result.destination.map_name, name);
        assert_eq!(result.destination.tile, tile(arrival));
        assert_eq!(result.destination.warp.index, 3);
        assert!(
            session(name, arrival)
                .check_warp_checked()
                .expect("passive entry")
                .is_none()
        );
        for exit in room["exits"].as_array().expect("two carpets") {
            let trigger = session(name, point(&exit["tile"]))
                .check_warp_checked()
                .expect("native carpet check")
                .expect("actual carpet");
            let result = data
                .resolve_warp_transition(&trigger)
                .expect("safe exterior return");
            assert_eq!(result.destination.map_name, "MnDuluth");
            assert_eq!(result.destination.tile, tile(outside));
            assert_eq!(
                u64::from(result.destination.warp.index),
                door["return_warp_index"]
                    .as_u64()
                    .expect("passive exterior index")
            );
            assert!(
                session("MnDuluth", outside)
                    .check_warp_checked()
                    .expect("passive exterior")
                    .is_none()
            );
        }
    }
}

#[test]
fn duluth_city_native_ledge_hops_down_denies_reverse_and_has_real_uphill_walk() {
    let data = published().data();
    let module = &data.maps["MnDuluth"];
    let mut session = OverworldSession::with_events_and_objects(
        data.overworld_map("MnDuluth").expect("native city map"),
        module.events.clone(),
        module.objects.clone(),
        data.tileset_collision("johto_modern")
            .expect("native collision"),
        tile([14, 19]),
    );
    let options = StepOptions {
        force_step_after_turn: true,
        ..StepOptions::default()
    };
    // TryJump reads the player's current HOP quadrant. First enter the upper
    // lip normally, then jump two native tiles over its reverse-blocking wall.
    assert!(
        matches!(session.step_checked(Direction::Down, options).expect("enter ledge lip"),
        StepOutcome::Moved { to, .. } if to == tile([14, 20]))
    );
    assert!(
        matches!(session.ledge_jump_checked(Direction::Down, options).expect("native hop"),
        LedgeJumpOutcome::Jumped { from, over, to, .. }
        if from == tile([14,20]) && over == tile([14,21]) && to == tile([14,22]))
    );
    assert!(matches!(
        session
            .step_checked(Direction::Up, options)
            .expect("reverse wall"),
        StepOutcome::Blocked { .. }
    ));
    assert_eq!(session.player.tile, tile([14, 22]));
    let occupied = module
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    let floor = |[x, y]: [u16; 2]| {
        let block = module.blocks[usize::from(y / 2) * 32 + usize::from(x / 2)];
        data.tilesets["johto_modern"].collision[&format!("{block:02x}")]
            [usize::from(y % 2 * 2 + x % 2)]
            == "FLOOR"
            && !occupied.contains(&[x, y])
    };
    let mut parents = BTreeMap::from([([14_u16, 22_u16], None)]);
    let mut queue = VecDeque::from([[14_u16, 22_u16]]);
    while let Some([x, y]) = queue.pop_front() {
        for next in [
            x.checked_sub(1).map(|x| [x, y]),
            y.checked_sub(1).map(|y| [x, y]),
            x.checked_add(1).map(|x| [x, y]),
            y.checked_add(1).map(|y| [x, y]),
        ]
        .into_iter()
        .flatten()
        {
            if next[0] < 64 && next[1] < 48 && floor(next) && !parents.contains_key(&next) {
                parents.insert(next, Some([x, y]));
                queue.push_back(next);
            }
        }
    }
    let mut route = vec![[14, 19]];
    while let Some(previous) = parents
        .get(route.last().expect("return endpoint"))
        .expect("ordinary return is connected")
    {
        route.push(*previous);
    }
    route.reverse();
    assert!(route.len() > 40, "ledge must save a meaningful uphill walk");
    for segment in route.windows(2) {
        let [from, to] = [segment[0], segment[1]];
        let direction = match (
            i32::from(to[0]) - i32::from(from[0]),
            i32::from(to[1]) - i32::from(from[1]),
        ) {
            (-1, 0) => Direction::Left,
            (1, 0) => Direction::Right,
            (0, -1) => Direction::Up,
            (0, 1) => Direction::Down,
            _ => unreachable!("adjacent path points"),
        };
        assert!(
            matches!(session.step_checked(direction,options).expect("native uphill step"),
            StepOutcome::Moved { to: actual, .. } if actual == tile(to))
        );
    }
    assert_eq!(session.player.tile, tile([14, 19]));
}

#[test]
fn duluth_city_native_care_heals_hp_status_and_pp_and_shop_charges_actual_prices() {
    let data = published().data();
    for (name, approach, facing, script) in [
        (
            "MnDuluthPokecenter1F",
            [7, 5],
            Direction::Up,
            "MN_DULUTH_NURSE",
        ),
        ("MnDuluthPokecenter1F", [13, 4], Direction::Up, "PCScript"),
        (
            "MnDuluthMart",
            [3, 3],
            Direction::Left,
            "MN_DULUTH_MART_CLERK",
        ),
    ] {
        let module = &data.maps[name];
        let mut session = OverworldSession::with_events_and_objects(
            data.overworld_map(name).expect("service map"),
            module.events.clone(),
            module.objects.clone(),
            data.tileset_collision(&module.attributes.tileset_name)
                .expect("service collision"),
            tile(approach),
        );
        session.set_player_facing(facing);
        assert_eq!(
            session
                .check_interaction_checked(1)
                .expect("native counter/PC lookup")
                .expect("reachable production interaction")
                .script,
            script
        );
    }
    let nurse = data.maps["MnDuluthPokecenter1F"]
        .script_runtime_commands
        .iter()
        .find(|c| c.source_script == "MN_DULUTH_NURSE" && c.command == "special")
        .expect("parsed real nurse special");
    assert_eq!(nurse.args, ["HealParty"]);
    let mut state = GameState::reset_wram_for_new_game();
    let mut mon = data
        .create_pokemon("MEW", 45, Dv::default())
        .expect("explicit legal partner fixture");
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
    let clerk = data.maps["MnDuluthMart"]
        .script_shop_commands
        .iter()
        .find(|c| c.source_script == "MN_DULUTH_MART_CLERK")
        .expect("actual parsed native mart");
    data.open_script_shop(
        &mut state,
        "MnDuluthMart",
        "MnDuluthMart",
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
}

#[test]
fn duluth_city_q07_receiver_has_all_eight_badge_checks_depot_and_only_one_result() {
    let room = &published().data().maps["MnDuluthHarborWorkshop"];
    let actor = room
        .objects
        .iter()
        .find(|a| a.script == "MN_HARBOR_RECEIVER")
        .expect("actual receiver actor");
    assert_eq!([actor.x, actor.y], [11, 5]);
    let flags = room
        .script_flag_commands
        .iter()
        .filter(|c| c.source_script == actor.script)
        .collect::<Vec<_>>();
    let badges = flags
        .iter()
        .filter(|c| c.command == "checkflag")
        .map(|c| c.flag_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(badges, BADGE_ENGINE_FLAGS[..8].iter().copied().collect());
    assert!(
        flags
            .iter()
            .any(|c| c.command == "checkevent" && c.flag_id == "EVENT_MN_DEPOT_CLEARED")
    );
    assert!(
        flags
            .iter()
            .any(|c| c.command == "checkevent" && c.flag_id == "EVENT_MN_LIGHTHOUSE_SIGNAL")
    );
    assert_eq!(
        flags
            .iter()
            .filter(|c| c.command == "setevent")
            .map(|c| c.flag_id.as_str())
            .collect::<Vec<_>>(),
        ["EVENT_MN_LIGHTHOUSE_SIGNAL"]
    );
    assert!(!flags.iter().any(|c| c.command == "setflag"));
    assert!(room.script_item_grants.is_empty());
    assert!(room.script_shop_commands.is_empty());
}

#[test]
fn duluth_city_rejects_false_doors_water_escape_services_and_stale_canonical_reference() {
    let (base, town, layout, gyms) = inputs();
    for case in [
        "active arrival",
        "unsafe return",
        "receiver missing badge",
        "stale gym",
        "alternative gym rows",
        "canonical passive changed",
        "duplicate room facade",
        "blocked nurse counter",
    ] {
        let mut changed: Value = serde_json::from_str(town).expect("source");
        match case {
            "active arrival" => changed["doors"][0]["target_warp"] = json!(1),
            "unsafe return" => changed["doors"][0]["return_tile"] = json!([9, 10]),
            "receiver missing badge" => {
                changed["interiors"][3]["residents"][0]["requires"] =
                    json!(["BADGE_8", "EVENT_MN_DEPOT_CLEARED"])
            }
            "stale gym" => {
                changed["interiors"][2]["source_reference"]["room_sha256"] = json!("0".repeat(64))
            }
            "alternative gym rows" => changed["interiors"][2]["rows"] = json!(["############"]),
            "canonical passive changed" => {
                changed["interiors"][2]["passive_arrival"]["tile"] = json!([12, 20])
            }
            "duplicate room facade" => {
                changed["doors"][1]["target"] = changed["doors"][0]["target"].clone()
            }
            "blocked nurse counter" => {
                changed["interiors"][0]["residents"][0]["counter_tile"] = json!([7, 3])
            }
            _ => unreachable!("fixed authored mutation"),
        };
        assert!(
            build_duluth_city_modpack(base, &changed.to_string(), layout, gyms).is_err(),
            "accepted {case}"
        );
    }
    let mut escape: Value = serde_json::from_str(layout).expect("layout");
    escape["layers"]
        .as_array_mut()
        .expect("explicit strokes")
        .push(
            json!({"name":"Invalid harbor escape","kind":"point","position":[31,18],"block":"35"}),
        );
    assert!(build_duluth_city_modpack(base, town, &escape.to_string(), gyms).is_err());
    let mut other: Value = serde_json::from_str(gyms).expect("canonical collection");
    other["gyms"][0]["leader"] = json!("Unrelated metadata");
    assert!(
        build_duluth_city_modpack(base, town, layout, &other.to_string()).is_ok(),
        "unrelated canonical gyms must not invalidate Gym8"
    );
}

#[test]
fn duluth_city_rejects_stale_native_gym_program_even_with_updated_source_digest() {
    fn ordered(value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(key, value)| (key, ordered(value)))
                    .collect::<BTreeMap<_, _>>()
                    .into_iter()
                    .collect(),
            ),
            Value::Array(values) => Value::Array(values.into_iter().map(ordered).collect()),
            scalar => scalar,
        }
    }
    let (base, town, layout, gyms) = inputs();
    let mut changed: Value = serde_json::from_str(gyms).expect("canonical collection");
    let gym = changed["gyms"]
        .as_array_mut()
        .expect("gyms")
        .iter_mut()
        .find(|g| g["id"] == "MnDuluthGym")
        .expect("Gym8");
    gym["guide"] = json!(["This guide changed."]);
    let digest = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&ordered(gym.clone())).expect("canonical digest"))
    );
    let mut source: Value = serde_json::from_str(town).expect("city source");
    source["interiors"][2]["source_reference"]["room_sha256"] = json!(digest);
    let error = build_duluth_city_modpack(base, &source.to_string(), layout, &changed.to_string())
        .expect_err("old native program cannot masquerade as new canonical source");
    assert!(
        format!("{error:#}").contains("complete module/program differs"),
        "wrong rejection: {error:#}"
    );
}
