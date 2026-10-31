//! Self-contained checks of the authored script graph before map publication.
use serde_json::{Value, json};

use super::build_northwoods_discovery_programs;

fn source() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/pokemon/acquisition-v01/rare-discoveries.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("tracked quest source"))
        .expect("authored quest JSON")
}

#[test]
fn northwoods_discovery_graph_binds_targets_and_keeps_encounters_independent() {
    let programs = build_northwoods_discovery_programs(&source().to_string())
        .expect("valid authored native pages");
    for (label, program) in &programs.scripts {
        for command in program.as_array().expect("native command array") {
            let op = command["command"].as_str().expect("native command");
            if [
                "iftrue",
                "iffalse",
                "scall",
                "sjump",
                "writetext",
                "jumptext",
            ]
            .contains(&op)
            {
                let target = command["args"][0].as_str().expect("native target");
                assert!(programs.scripts.contains_key(target), "{label} -> {target}");
            }
        }
    }
    for (species, other) in [("BABEHEMOTH", "BUNJANT"), ("BUNJANT", "BABEHEMOTH")] {
        let encounter = programs.scripts[&format!("MN_TALE_{species}")].to_string();
        assert!(
            encounter.contains("OPTIONAL_LEGENDARY")
                || encounter
                    .contains(crystal_core::battle::capture::OPTIONAL_LEGENDARY_BATTLE_TYPE)
        );
        assert!(!encounter.contains(&format!("{other}_CAUGHT")));
        let caught = format!("EVENT_MN_TALL_TALE_{species}_CAUGHT");
        let setters = programs
            .scripts
            .iter()
            .filter(|(_, p)| {
                p.as_array()
                    .expect("commands")
                    .iter()
                    .any(|c| c["command"] == "setevent" && c["args"][0] == caught)
            })
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(setters, [format!("MnTale{species}Caught")]);
    }
    for (label, restore) in [("MnTaleObjects", false), ("MnTaleNewVisit", true)] {
        for species in ["BABEHEMOTH", "BUNJANT"] {
            let commands = programs.scripts[&format!("{label}{species}")]
                .as_array()
                .expect("callback stage");
            assert_eq!(commands.iter().any(|c| c["command"] == "appear"), restore);
        }
    }
    let observation = programs.scripts["MnTaleCulvert"]
        .as_array()
        .expect("observation");
    let acknowledge = observation
        .iter()
        .position(|c| c["command"] == "promptbutton")
        .expect("acknowledgment");
    let event = observation
        .iter()
        .position(|c| c["command"] == "setevent")
        .expect("observation record");
    assert!(acknowledge < event, "never record an unacknowledged clue");
}

#[test]
fn northwoods_discovery_rejects_invalid_pages_missing_text_and_duplicate_quests() {
    let mut wide = source();
    wide["quests"][0]["dialogue_pages"]["culvert"] = json!([["This line is much too long."]]);
    assert!(build_northwoods_discovery_programs(&wide.to_string()).is_err());
    let mut absent = source();
    absent["quests"][0]["dialogue_pages"]
        .as_object_mut()
        .expect("pages")
        .remove("retry");
    assert!(build_northwoods_discovery_programs(&absent.to_string()).is_err());
    let mut duplicate = source();
    let quest = duplicate["quests"][0].clone();
    duplicate["quests"]
        .as_array_mut()
        .expect("quests")
        .push(quest);
    assert!(build_northwoods_discovery_programs(&duplicate.to_string()).is_err());
}
