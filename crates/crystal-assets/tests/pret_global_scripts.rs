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

#[test]
#[ignore = "requires the pinned external source; set CRYSTAL_PRET_SOURCE_ROOT"]
fn pinned_field_move_import_retains_headbutt_visual_closure_and_source_timing() {
    let source_root = std::path::PathBuf::from(
        std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT")
            .expect("CRYSTAL_PRET_SOURCE_ROOT must name the pinned external pokecrystal checkout"),
    )
    .canonicalize()
    .expect("resolve external pinned source");
    let lock: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pret-source.lock.json"),
        )
        .expect("repository pret source lock"),
    )
    .expect("typed source lock");
    let revision = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&source_root)
        .output()
        .expect("read actual external source revision");
    assert!(
        revision.status.success(),
        "external source revision unavailable"
    );
    assert_eq!(
        std::str::from_utf8(&revision.stdout)
            .expect("source revision UTF-8")
            .trim(),
        lock["commit"].as_str().expect("pinned source commit"),
        "the exporter regression requires the actual pinned source"
    );
    let source = std::fs::read_to_string(source_root.join("engine/events/field_moves.asm"))
        .expect("actual pinned field-move implementation");
    let authored = source_scripts::parse(&source).expect("real Rust field-move source parser");
    let (_, imported) =
        source_global_scripts::read(&source_root).expect("production global source importer");
    let imported = imported["OverworldEvents"]
        .as_object()
        .expect("imported overworld definitions");

    // Symbol identities only: bodies are read from the external source and
    // compared through the actual Rust parser, never copied into a catalog.
    // This is the complete current ShakeHeadbuttTree visual certificate closure.
    let labels: [&str; 30] = [
        "ShakeHeadbuttTree",
        ".loop@ShakeHeadbuttTree",
        ".done@ShakeHeadbuttTree",
        "HideHeadbuttTree",
        "TreeRelativeLocationTable",
        "OWCutAnimation",
        ".loop@OWCutAnimation",
        ".finish@OWCutAnimation",
        ".LoadCutGFX@OWCutAnimation",
        "OWCutJumptable",
        ".dw@OWCutJumptable",
        "Cut_SpawnAnimateTree",
        "Cut_SpawnAnimateLeaves",
        "Cut_StartWaiting",
        "Cut_WaitAnimSFX",
        ".finished@Cut_WaitAnimSFX",
        "Cut_SpawnLeaf",
        "Cut_GetLeafSpawnCoords",
        ".left_side@Cut_GetLeafSpawnCoords",
        ".top_side@Cut_GetLeafSpawnCoords",
        ".Coords@Cut_GetLeafSpawnCoords",
        "Cut_Headbutt_GetPixelFacing",
        ".Coords@Cut_Headbutt_GetPixelFacing",
        "FlyFromAnim",
        ".loop@FlyFromAnim",
        ".exit@FlyFromAnim",
        "FlyFunction_InitGFX",
        "FlyFunction_FrameTimer",
        ".exit@FlyFunction_FrameTimer",
        ".SpawnLeaf@FlyFunction_FrameTimer",
    ];
    assert_eq!(
        labels
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        30,
        "certificate symbols must be unique"
    );
    for label in labels {
        let expected = authored
            .get(label)
            .unwrap_or_else(|| panic!("pinned source omitted {label}"));
        assert!(
            expected.as_array().is_some_and(|body| !body.is_empty()),
            "actual source body for {label} is empty"
        );
        assert_eq!(
            imported.get(label),
            Some(expected),
            "production importer omitted or changed native visual closure {label}"
        );
    }
    let commands = |label: &str| {
        imported[label]
            .as_array()
            .unwrap_or_else(|| panic!("native command array for {label}"))
    };
    let is = |entry: &serde_json::Value, command: &str, args: &[&str]| {
        entry["command"] == command
            && entry["args"].as_array().is_some_and(|actual| {
                actual.len() == args.len()
                    && actual
                        .iter()
                        .zip(args)
                        .all(|(actual, expected)| actual.as_str() == Some(*expected))
            })
    };
    let shake = commands("ShakeHeadbuttTree");
    let timer = shake
        .windows(2)
        .filter(|pair| {
            is(&pair[0], "ld", &["a", "32"]) && is(&pair[1], "ld", &["[wFrameCounter]", "a"])
        })
        .count();
    assert_eq!(
        timer, 1,
        "Headbutt must initialize its actual 32-frame source counter once"
    );
    assert!(
        shake.windows(3).any(|triple| {
            is(&triple[0], "call", &["WaitSFX"])
                && is(&triple[1], "ld", &["de", "SFX_SANDSTORM"])
                && is(&triple[2], "call", &["PlaySFX"])
        }),
        "Headbutt must retain the authored sandstorm sound and wait ordering"
    );
    let loop_body = commands(".loop@ShakeHeadbuttTree");
    for (command, args) in [
        ("ld", vec!["hl", "wFrameCounter"]),
        ("dec", vec!["[hl]"]),
        ("farcall", vec!["DoNextFrameForAllSprites"]),
        ("call", vec!["DelayFrame"]),
        ("jr", vec![".loop"]),
    ] {
        assert!(
            loop_body.iter().any(|entry| is(entry, command, &args)),
            "actual native Headbutt timing loop lost {command} {args:?}"
        );
    }
    let done = commands(".done@ShakeHeadbuttTree");
    assert!(
        done.iter()
            .any(|entry| is(entry, "call", &["LoadOverworldTilemapAndAttrmapPals"]))
    );
    assert!(
        done.iter()
            .any(|entry| is(entry, "farcall", &["ClearSpriteAnims"]))
    );
    assert!(
        done.iter()
            .any(|entry| is(entry, "call", &["UpdatePlayerSprite"]))
    );
    assert!(commands("HeadbuttScript").iter().any(|entry| is(
        entry,
        "callasm",
        &["ShakeHeadbuttTree"]
    )));
}
