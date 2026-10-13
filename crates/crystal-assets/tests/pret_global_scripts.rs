#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;
mod radio_text_export {
    pub(crate) use super::source_syntax::{code_part, source_arguments};
}
#[path = "../src/bin/pack_core/source_global_scripts.rs"]
mod source_global_scripts;
#[path = "../src/bin/pack_core/source_playability.rs"]
mod source_playability;
#[path = "../src/bin/pack_core/source_map_geometry.rs"]
mod source_map_geometry;

#[test]
fn playability_uses_authored_home_and_keeps_strict_map_checks() {
    let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
        eprintln!("set CRYSTAL_EXPORT_WORKSPACE to verify authored startup policy");
        return;
    };
    let policy = source_playability::read(std::path::Path::new(&root)).unwrap();
    assert_eq!(policy["start_maps"], serde_json::json!(["PlayersHouse2F"]));
    assert_eq!(
        policy["start_tiles"][0]["tile"],
        serde_json::json!({"x":3,"y":3})
    );
    assert_eq!(policy["require_all_maps_reachable"], true);
    assert_eq!(policy["require_walkable_maps"], true);
    assert!(
        policy["initial_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|flag| flag == "EVENT_RADIO_TOWER_ROCKET_TAKEOVER")
    );
    assert!(policy["initial_items"].as_array().unwrap().is_empty());
    assert!(policy["progression_rules"].as_array().unwrap().is_empty());
}
#[path = "../src/bin/pack_core/source_scripts.rs"]
mod source_scripts;

#[test]
fn authored_player_entries_and_phone_branches_preserve_source_contracts() {
    let Some(root) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
        eprintln!("set CRYSTAL_PRET_SOURCE_ROOT to verify external authored scripts");
        return;
    };
    let (phone, events) = source_global_scripts::read(std::path::Path::new(&root)).unwrap();
    let events = &events["OverworldEvents"];
    // Item-ball entry must not absorb its failure branch or CPU implementation.
    assert_eq!(events["FindItemInBallScript"].as_array().unwrap().len(), 10);
    assert_eq!(
        events[".no_room@FindItemInBallScript"][0]["command"],
        "opentext"
    );
    assert_eq!(
        events[".TryReceiveItem@FindItemInBallScript"][0]["command"],
        "xor"
    );
    assert_eq!(
        events["OverworldWhiteoutScript"][2],
        serde_json::json!({"command":"sjump","args":["Script_Whiteout"]})
    );
    assert_eq!(
        events["SeenByTrainerScript"][3]["args"][0],
        "TrainerWalkToPlayer"
    );
    assert_eq!(events["RockMonEncounter"].as_array().unwrap().len(), 15);
    assert_eq!(
        events[".no_battle@RockMonEncounter"],
        serde_json::json!([
            {"command":"xor","args":["a"]},
            {"command":"ret","args":[]},
            {"command":"db","args":["5"]}
        ])
    );
    assert_eq!(events["RockSmashScript"].as_array().unwrap().len(), 15);
    assert_eq!(
        phone["JackPhoneCalleeScript"][3]["args"][0],
        "PhoneScript_AnswerPhone_Male"
    );
    assert_eq!(
        phone[".WantsBattle@JackPhoneCalleeScript"][1]["args"][0],
        "JackWantsBattleScript"
    );
    assert!(phone.get("PhoneScript_AnswerPhone_Male").is_some());
    assert_eq!(
        phone["LoadPhoneScriptBank"],
        serde_json::json!([
            {"command":"memcall","args":["wPhoneScriptBank"]},
            {"command":"endcallback","args":[]}
        ])
    );
    assert_eq!(
        phone["LoadOutOfAreaScript"],
        serde_json::json!([
            {"command":"scall","args":["PhoneOutOfAreaScript"]},
            {"command":"endcallback","args":[]}
        ])
    );
    assert_eq!(
        phone["PhoneOutOfAreaScript"][0]["args"][0],
        "PhoneOutOfAreaText"
    );
    assert_eq!(
        phone["PhoneScript_JustTalkToThem"][0]["args"][0],
        "PhoneJustTalkToThemText"
    );
    assert_eq!(
        phone["PhoneOutOfAreaText"][0]["args"][0],
        "_PhoneOutOfAreaText"
    );
    assert_eq!(
        phone["PhoneJustTalkToThemText"][0]["args"][0],
        "_PhoneJustTalkToThemText"
    );
    assert!(phone.as_object().unwrap().len() > 500);
}

#[test]
fn production_global_loader_follows_standard_field_scripts_across_catalogs() {
    let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
        return;
    };
    let root = std::path::Path::new(&root);
    let data_root = root.join("apps/web/assets/data");
    let read = |name: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(data_root.join(name)).unwrap()).unwrap()
    };
    let (phone, events) = source_global_scripts::read(&root.join("vendor/pokecrystal")).unwrap();
    let mut data = crystal_assets::modpack::GameDataSet::default();
    data.story_events
        .extend([read("story_events/StandardScripts.json"), events]);
    data.phone_scripts.push(phone);
    data.story_event_script_constants =
        serde_json::from_value(read("story_event_script_constants.json")).unwrap();
    data.map_attributes = serde_json::from_value(read("map_attributes.json")).unwrap();
    data.map_blocks = serde_json::from_value(read("map_blocks.json")).unwrap();
    data.map_dimensions = serde_json::from_value(read("map_dimensions.json")).unwrap();
    data.map_scripts = serde_json::from_value(read("map_scripts.json")).unwrap();
    data.runtime_map_metadata = serde_json::from_value(read("runtime_map_metadata.json")).unwrap();
    data.npcs = serde_json::from_value(read("npcs.json")).unwrap();
    data.tilesets = serde_json::from_value(read("tilesets.json")).unwrap();
    for name in data.map_attributes.keys() {
        data.maps.insert(
            name.clone(),
            data.assemble_map_module_from_compiled_payloads(name)
                .unwrap(),
        );
    }
    data.materialize_global_scripts().unwrap();
    let scripts = &data.global_scripts.as_ref().unwrap().scripts;
    for root in [
        "StrengthBoulderScript",
        "AskStrengthScript",
        "SmashRockScript",
        "AskRockSmashScript",
        "LoadPhoneScriptBank",
        "LoadOutOfAreaScript",
        "PhoneOutOfAreaScript",
        "PhoneScript_JustTalkToThem",
    ] {
        assert!(
            scripts.contains_key(root),
            "authored global closure omitted {root}"
        );
    }
    let assets = crystal_assets::AssetRoot::new(root.join("apps/web/assets"));
    let report = crystal_assets::modpack::ModpackCompiler::new(&assets).audit(&data);
    assert!(
        !report
            .diagnostics
            .iter()
            .any(
                |diagnostic| diagnostic.subject == "GlobalScripts:RockSmashScript:8"
                    && matches!(
                        diagnostic.code.as_str(),
                        "non_executable_callasm_target" | "unknown_script_runtime_target"
                    )
            ),
        "authored RockMonEncounter must satisfy its production CPU certificate"
    );
}
