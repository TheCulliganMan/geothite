//! Asset/core publication tests. Regional controller journeys remain separate.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_core::models::Dv;
use crystal_core::state::{GameState, OverworldMemory};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;
use crystal_core::world::session::OverworldSession;

use super::*;

fn native_tile([x, y]: [u16; 2]) -> TilePosition {
    TilePosition::new(
        i16::try_from(x).expect("authored fixture x fits native tile coordinates"),
        i16::try_from(y).expect("authored fixture y fits native tile coordinates"),
    )
}

fn inputs() -> &'static (CompiledGamePack, String, String) {
    static INPUTS: OnceLock<(CompiledGamePack, String, String)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let source = std::env::var_os("CRYSTAL_BOUNDARY_BASECAMP_BASE_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-region-review.crystalpack"))
            .canonicalize()
            .expect("completed external regional source pack");
        (
            crate::read_verified_compiled_game_pack(source).expect("verified regional source"),
            std::fs::read_to_string(root.join("planning/minnesota/towns/MnBoundaryWaters.json"))
                .expect("actual basecamp source"),
            std::fs::read_to_string(
                root.join("planning/minnesota/towns/MnBoundaryWaters.layout.json"),
            )
            .expect("actual basecamp layout"),
        )
    })
}
fn published() -> &'static CompiledGamePack {
    static PUBLISHED: OnceLock<CompiledGamePack> = OnceLock::new();
    PUBLISHED.get_or_init(|| {
        let (base, town, layout) = inputs();
        build_boundary_basecamp_modpack(base, town, layout)
            .expect("actual free native basecamp and survey shelter")
    })
}
fn town_value() -> Value {
    serde_json::from_str(&inputs().1).expect("town source")
}
fn layout_value() -> Value {
    serde_json::from_str(&inputs().2).expect("town layout")
}
fn rejection(town: &Value, layout: &Value, expected: &str) {
    let error =
        build_boundary_basecamp_modpack(&inputs().0, &town.to_string(), &layout.to_string())
            .expect_err("invalid actual source must be rejected");
    assert!(
        error.to_string().contains(expected),
        "expected {expected:?}, got {error:#}"
    );
}
fn replace_block(layout: &mut Value, position: [u16; 2], block: &str) {
    layout["layers"].as_array_mut().expect("source layers").push(json!({"kind":"point","name":"negative native geometry fixture","position":position,"block":block}));
}

#[test]
fn boundary_basecamp_adds_only_real_town_and_shelter_preserving_regional_source() {
    let (base, town, layout) = inputs();
    let pack = published();
    assert_eq!(pack.data().maps.len(), base.data().maps.len() + 2);
    assert_eq!(
        pack.data().runtime_spawn_points,
        base.data().runtime_spawn_points,
        "HOME and recovery table unchanged"
    );
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("audio"),
        base.audio_manifest().expect("source audio")
    );
    assert_eq!(pack.data().items, base.data().items);
    assert_eq!(pack.data().tilesets, base.data().tilesets);
    assert_eq!(
        pack.data().wild_encounters,
        base.data().wild_encounters,
        "free town does not masquerade as Safari encounter mode"
    );
    for (name, module) in &base.data().maps {
        assert_eq!(
            pack.data().maps.get(name),
            Some(module),
            "changed existing map {name}"
        );
    }
    for (key, metadata) in &base.data().runtime_map_metadata {
        assert_eq!(
            pack.data().runtime_map_metadata.get(key),
            Some(metadata),
            "changed metadata {key}"
        );
    }
    for (key, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            pack.data().trainers.trainers.get(key),
            Some(trainer),
            "changed source trainer {key}"
        );
    }
    for (key, bytes) in base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(key),
            Some(bytes),
            "changed source file {key}"
        );
    }
    for (key, value) in &base.data().pokegear_landmarks.map_to_landmark {
        assert_eq!(
            pack.data().pokegear_landmarks.map_to_landmark.get(key),
            Some(value)
        );
    }
    for (key, value) in &base.data().story_event_script_constants.global {
        assert_eq!(
            pack.data().story_event_script_constants.global.get(key),
            Some(value),
            "changed source constant {key}"
        );
    }
    assert!(
        pack.report()
            .manifests
            .iter()
            .any(|id| id == BOUNDARY_BASECAMP_MANIFEST)
    );
    let town: Town = serde_json::from_str(town).expect("typed town");
    let blocks = Layout::parse(layout)
        .expect("native layout")
        .compile()
        .expect("hand-authored strokes");
    let room = &town.interiors[0];
    for (index, name, constant, dimensions, art) in [
        (
            1,
            BOUNDARY_BASECAMP_MAP,
            BOUNDARY_BASECAMP_CONSTANT,
            [32, 24],
            "mn_boundary_safari",
        ),
        (
            2,
            room.id.as_str(),
            crate::boundary_basecamp_story::BOUNDARY_SURVEY_SHELTER_CONSTANT,
            [8, 6],
            "lab",
        ),
    ] {
        let module = &pack.data().maps[name];
        assert_eq!(
            [module.attributes.width, module.attributes.height],
            dimensions
        );
        assert_eq!(module.attributes.tileset_name, art);
        assert!(
            module.attributes.connections.is_empty(),
            "regional integration remains Root-owned"
        );
        let metadata = &pack.data().runtime_map_metadata[constant];
        assert_eq!((metadata.group_id, metadata.map_id), (62, index));
        assert_eq!(metadata.name, name);
        assert_eq!(metadata.group_name, GROUP);
        assert_eq!(
            pack.data()
                .assemble_map_module_from_compiled_payloads(name)
                .expect("actual native file assembly"),
            *module
        );
        assert_eq!(
            pack.data().pokegear_landmarks.map_to_landmark[name],
            LANDMARK
        );
    }
    assert_eq!(pack.data().maps[BOUNDARY_BASECAMP_MAP].blocks, blocks);
    let room_blocks = room
        .rows
        .iter()
        .flat_map(|row| row.split_whitespace())
        .map(|b| u16::from_str_radix(b, 16).expect("room block"))
        .collect::<Vec<_>>();
    assert_eq!(pack.data().maps[&room.id].blocks, room_blocks);
    assert_eq!(
        pack.data().maps[BOUNDARY_BASECAMP_MAP].attributes.music,
        base.data().maps["LakeOfRage"].attributes.music
    );
}

#[test]
fn boundary_basecamp_keeps_source_actors_free_services_and_closed_admission_honest() {
    let pack = published();
    let town: Town = serde_json::from_str(&inputs().1).expect("source town");
    let value = town_value();
    let exterior = &pack.data().maps[BOUNDARY_BASECAMP_MAP];
    let shelter = &pack.data().maps[&town.interiors[0].id];
    geometry::validate(pack, &town, &value, exterior, shelter)
        .expect("actual native topology and free approaches");
    for (module, residents, signs) in [
        (exterior, &town.residents, &town.signs),
        (
            shelter,
            &town.interiors[0].residents,
            &town.interiors[0].signs,
        ),
    ] {
        assert_eq!(module.objects.len(), residents.len());
        assert_eq!(module.events.bg_events.len(), signs.len());
        for source in residents {
            let object = module
                .objects
                .iter()
                .find(|o| o.object_identifier.as_deref() == Some(source.id.as_str()))
                .expect("original actual native actor");
            assert_eq!([object.x, object.y], source.tile);
            assert_eq!(object.sprite, source.sprite);
            assert_eq!(object.script, source.id);
            assert_eq!(object.object_type, "OBJECTTYPE_SCRIPT");
            assert_eq!(object.spritemovedata, source.facing.movement());
            assert_eq!(object.event_flag, "-1");
        }
        for source in signs {
            let bg = module
                .events
                .bg_events
                .iter()
                .find(|bg| bg.script == source.id)
                .expect("source native sign");
            assert_eq!([bg.x, bg.y], source.tile);
            assert_eq!(bg.event_type, "BGEVENT_READ");
        }
    }
    assert!(
        exterior.script_runtime_commands.is_empty(),
        "closed check-in and outdoor navigation have no paid/session/rest routine"
    );
    assert!(
        exterior.script_economy_commands.is_empty()
            && exterior.script_item_grants.is_empty()
            && exterior.script_shop_commands.is_empty()
    );
    assert!(
        exterior.events.coord_events.is_empty(),
        "no fabricated regional arrival/SQ03 scene"
    );
    assert!(
        exterior.script_flag_commands.is_empty(),
        "terrain does not grant story or paid admission"
    );
    let medical = shelter
        .script_runtime_commands
        .iter()
        .filter(|command| command.source_script == "MN_BOUNDARY_SURVEY_MEDIC")
        .collect::<Vec<_>>();
    assert_eq!(medical.len(), 1);
    assert_eq!(medical[0].command, "special");
    assert_eq!(medical[0].args, ["HealParty"]);
    assert!(
        shelter
            .script_object_commands
            .iter()
            .any(
                |command| command.source_script == "MN_BOUNDARY_SURVEY_MEDIC"
                    && command.command == "faceplayer"
            )
    );
    let trainer = &pack.data().trainers.trainers
        [crate::boundary_basecamp_story::BOUNDARY_SURVEY_CHIEF_TRAINER];
    assert_eq!(
        trainer
            .party
            .iter()
            .map(|p| (p.species.as_str(), p.level))
            .collect::<Vec<_>>(),
        [("MAGNETON", 44), ("WEEZING", 44), ("CROBAT", 46)]
    );
}

#[test]
fn boundary_basecamp_native_door_and_both_carpets_resolve_to_passive_floor_without_bounce() {
    let data = published().data();
    let room = crate::boundary_basecamp_story::BOUNDARY_SURVEY_SHELTER;
    let source = town_value();
    let door = point(&source["doors"][0]["tile"]).expect("actual door");
    let outside = point(&source["doors"][0]["return_tile"]).expect("outside passive floor");
    let session = |name: &str, tile: [u16; 2]| {
        let module = data.map_module(name).expect("actual published map");
        OverworldSession::with_events(
            data.overworld_map(name).expect("native map"),
            module.events.clone(),
            data.tileset_collision(&module.attributes.tileset_name)
                .expect("native collision"),
            native_tile(tile),
        )
    };
    // Explicit core-boundary placement tests real trigger/resolution, not an
    // earned regional player journey. Controller inputs are a separate proof.
    for (name, tile, index, destination, arrival, destination_index) in [
        (BOUNDARY_BASECAMP_MAP, door, 1, room, [8, 9], 3),
        (room, [8, 11], 1, BOUNDARY_BASECAMP_MAP, outside, 2),
        (room, [9, 11], 2, BOUNDARY_BASECAMP_MAP, outside, 2),
    ] {
        let trigger = session(name, tile)
            .check_warp_checked()
            .expect("native trigger")
            .expect("actual DOOR/carpet");
        assert_eq!(trigger.warp.index, index);
        assert_eq!(trigger.warp.target_map, trigger.warp.target_map_constant);
        let transition = data
            .resolve_warp_transition(&trigger)
            .expect("native constant destination resolution");
        assert_eq!(transition.destination.map_name, destination);
        assert_eq!(transition.destination.tile, native_tile(arrival));
        assert_eq!(transition.destination.warp.index, destination_index);
        assert!(
            session(destination, arrival)
                .check_warp_checked()
                .expect("passive floor collision")
                .is_none()
        );
        let mut wrong = trigger;
        wrong.warp.target_map = destination.into();
        assert!(
            data.resolve_warp_transition(&wrong)
                .expect_err("map names must not replace constants")
                .to_string()
                .contains("does not match target_map_constant")
        );
    }
}

#[test]
fn boundary_basecamp_native_rest_services_heal_without_fee_or_safari_renewal() {
    let data = published().data();
    let source: Town = serde_json::from_str(&inputs().1).expect("actual town source");
    // Explicit asset-mutation fixtures. Each parsed real rest interaction uses
    // the authoritative native adapter; these are not controller journeys.
    for (map, residents) in [
        (BOUNDARY_BASECAMP_MAP, &source.residents),
        (
            source.interiors[0].id.as_str(),
            &source.interiors[0].residents,
        ),
    ] {
        for resident in residents
            .iter()
            .filter(|resident| resident.service.is_some())
        {
            let routine = data.maps[map]
                .script_runtime_commands
                .iter()
                .find(|command| command.source_script == resident.id)
                .expect("actual bound native rest command");
            assert_eq!(routine.command, "special");
            assert_eq!(routine.args, ["HealParty"]);
            let mut state = GameState::reset_wram_for_new_game();
            state.overworld = OverworldMemory::Active {
                map_name: map.into(),
                tile: native_tile([
                    resident.tile[0].checked_add(1).expect("adjacent fixture x"),
                    resident.tile[1],
                ]),
                facing: Direction::Left,
                mode: MovementMode::Normal,
            };
            let mut partner = data
                .create_pokemon("MEW", 45, Dv::default())
                .expect("legal source-created partner");
            partner.hp = 1;
            partner.status = Some("POISON".into());
            for movement in &mut partner.moves {
                movement.current_pp = 0;
            }
            assert!(state.storage.party.add_pokemon(partner));
            state.sync_party_from_storage();
            state.money = 1234;
            state
                .flags
                .set_engine_flag("ENGINE_SAFARI_ZONE", true)
                .expect("explicit unrelated Safari flag fixture");
            state
                .script_runtime
                .memory
                .insert("wSafariBalls".into(), "7".into());
            state
                .script_runtime
                .memory
                .insert("wSafariSteps".into(), "123".into());
            let resources = (
                state.money,
                state.bag.clone(),
                state.flags.clone(),
                state.script_runtime.memory.clone(),
            );
            data.apply_special_routine(&mut state, &routine.args[0], &BTreeSet::new())
                .expect("actual native HealParty routine");
            let partner = state.storage.party.pokemon[0]
                .as_ref()
                .expect("rested partner");
            assert_eq!(partner.hp, partner.max_hp);
            assert_eq!(partner.status, None);
            assert!(partner.moves.iter().all(|movement| movement.current_pp > 0));
            assert_eq!(
                (
                    state.money,
                    state.bag.clone(),
                    state.flags.clone(),
                    state.script_runtime.memory.clone()
                ),
                resources
            );
        }
    }
}

#[test]
fn boundary_basecamp_rejects_false_doors_active_arrivals_or_furniture_overlap() {
    let layout = layout_value();
    let mut town = town_value();
    town["doors"][0]["target_warp"] = json!(1);
    rejection(&town, &layout, "passive FLOOR arrival3");
    let mut town = town_value();
    town["interiors"][0]["passive_arrival"]["tile"] = json!([8, 11]);
    rejection(&town, &layout, "passive FLOOR arrival3");
    let mut town = town_value();
    town["interiors"][0]["rows"][5] = json!("00 00 00 00 10 00 00 00");
    rejection(&town, &layout, "room exit collision");
    let mut town = town_value();
    town["interiors"][0]["residents"][2]["tile"] = json!([12, 3]);
    rejection(&town, &layout, "room actor stands on furniture");
    let mut bad = layout.clone();
    let door = point(&town_value()["doors"][0]["tile"]).expect("door");
    replace_block(&mut bad, door.map(|n| n / 2), "02");
    rejection(&town_value(), &bad, "facade DOOR");
}

#[test]
fn boundary_basecamp_rejects_broken_ports_open_water_dock_and_closed_gate_bypass() {
    let source = town_value();
    let layout = layout_value();
    let mut town = source.clone();
    town["ports"][0]["landing_metatile"] = json!([27, 12]);
    rejection(&town, &layout, "reserved port changed");
    let mut bad = layout.clone();
    replace_block(&mut bad, [31, 12], "05");
    rejection(&source, &bad, "free town port disconnected");
    let mut bad = layout.clone();
    replace_block(&mut bad, [0, 0], "80");
    rejection(&source, &bad, "unconnected outer water");
    let mut bad = layout.clone();
    replace_block(&mut bad, [7, 7], "80");
    rejection(&source, &bad, "native dock rail collision");
    let mut town = source.clone();
    let residents = town["residents"].as_array_mut().expect("actors");
    let guard = residents
        .iter_mut()
        .find(|actor| actor["id"] == "MN_BOUNDARY_PENDING_GATE_WEST")
        .expect("guard");
    guard["tile"] = json!([32, 8]);
    rejection(
        &town,
        &layout,
        "closed expedition admission has a walking/Surf bypass",
    );
    let mut bad = layout;
    replace_block(&mut bad, [20, 12], "5b");
    rejection(&source, &bad, "undeclared decorative Cut tree");
}

#[test]
fn boundary_basecamp_rejects_paid_story_access_or_unbound_medic_and_duplicate_publication() {
    let layout = layout_value();
    let mut town = town_value();
    town["story_contract"]["Q07"]["paid_state_required"] = json!(true);
    rejection(&town, &layout, "story access is free");
    let mut town = town_value();
    town["admission_contract"]["pending_preserve_warp"]["enabled"] = json!(true);
    rejection(&town, &layout, "admission must remain closed");
    let mut town = town_value();
    town["interiors"][0]["residents"][2]
        .as_object_mut()
        .expect("medic")
        .remove("service");
    rejection(&town, &layout, "medic must bind native HealParty");
    assert!(
        build_boundary_basecamp_modpack(published(), &inputs().1, &inputs().2)
            .expect_err("duplicate group/map publication")
            .to_string()
            .contains("already occupied")
    );
}

#[test]
fn boundary_basecamp_rejects_source_chief_party_battle_prerequisite_and_victory_drift() {
    let layout = layout_value();
    let mut town = town_value();
    town["interiors"][0]["residents"][0]["party"][0]["level"] = json!(45);
    rejection(&town, &layout, "source chief party differs");
    let mut town = town_value();
    town["interiors"][0]["residents"][0]["battle"] = json!("MN_ALTERNATE_CHIEF");
    rejection(&town, &layout, "source chief battle differs");
    let mut town = town_value();
    town["interiors"][0]["residents"][0]["requires"]
        .as_array_mut()
        .expect("source chief prerequisites")
        .pop();
    rejection(&town, &layout, "source chief prerequisites differ");
    let mut town = town_value();
    town["interiors"][0]["residents"][0]["win_flag"] = json!("EVENT_MN_ALTERNATE_CHIEF_DEFEATED");
    rejection(&town, &layout, "source chief win_flag differs");
}

#[test]
fn boundary_basecamp_rejects_source_receiver_result_prerequisite_and_story_anchor_drift() {
    let layout = layout_value();
    let mut town = town_value();
    town["interiors"][0]["receiver_contract"]["sets"] =
        json!(["EVENT_MN_ALTERNATE_SURVEY_RESTORED"]);
    rejection(&town, &layout, "source receiver sets differ");
    let mut town = town_value();
    town["interiors"][0]["receiver_contract"]["requires"] = json!(["EVENT_MN_LIGHTHOUSE_SIGNAL"]);
    rejection(&town, &layout, "source receiver prerequisites differ");
    let mut town = town_value();
    town["story_contract"]["Q07"]["chief_actor"] = json!("MN_ALTERNATE_CHIEF");
    rejection(&town, &layout, "source story anchors differ");
}
