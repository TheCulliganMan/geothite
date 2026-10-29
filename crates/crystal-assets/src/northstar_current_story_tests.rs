//! Exact-parent story replay retains the independently authored room life.
use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::{CompiledGamePack, read_verified_compiled_game_pack};

fn pack(variable: &str) -> CompiledGamePack {
    read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os(variable).expect("story fixture"),
    ))
    .expect("strict story pack")
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_STORY_BASE/PACK/COMPOSITION external fixtures"]
fn current_story_composition_retains_room_life_and_rejects_unreviewed_parents() {
    let base = pack("CRYSTAL_INTERIOR_STORY_BASE");
    let child = pack("CRYSTAL_INTERIOR_STORY_PACK");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota");
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_INTERIOR_STORY_COMPOSITION").expect("explicit reviewed parent"),
    )
    .expect("composition source");
    let rebuilt = super::build_northstar_current_story_from_composition(&base, &root, &source)
        .expect("whole authored story replay on the room branch");
    assert!(
        rebuilt == child,
        "written story equals the complete deterministic Rust replay"
    );
    assert_eq!(child.data.maps.len(), base.data.maps.len() + 13);
    assert_eq!(base.compiled_audio(), child.compiled_audio());
    assert_eq!(base.data.pokemon, child.data.pokemon);
    assert_eq!(base.data.moves, child.data.moves);
    assert_eq!(base.data.wild_encounters, child.data.wild_encounters);
    assert_eq!(base.data.field_encounters, child.data.field_encounters);
    assert_eq!(
        base.data.runtime_title_screen,
        child.data.runtime_title_screen
    );
    for (path, bytes) in &base.runtime_files {
        assert_eq!(
            child.runtime_files.get(path),
            Some(bytes),
            "old native file {path}"
        );
    }
    for (name, id) in &base.data.story_event_script_constants.global {
        assert_eq!(
            child.data.story_event_script_constants.global.get(name),
            Some(id),
            "old numeric identity {name}"
        );
    }
    let story_maps = BTreeSet::from([
        "MnBemidji",
        "MnBoundaryWatersSurveyShelter",
        "MnBrainerd",
        "MnDuluthHarborWorkshop",
        "MnGrandRapidsSurveyDepot",
        "MnLakeItasca",
        "MnMankato",
        "MnMinneapolisDispatchOffice",
        "MnMinneapolisDispatcherHouse",
        "MnR09Cave1F",
        "MnRochesterPokecenter1F",
        "MnStCloud",
        "MnStCloudStation",
        "MnStPaulUnionDepot",
        "MnWinonaFieldStation",
        "MnWinonaHome1F",
        "MnLeagueLobby",
        "MnLeagueRoom1",
        "MnLeagueRoom2",
        "MnLeagueRoom3",
        "MnLeagueRoom4",
        "MnLeagueChampion",
        "MnLeagueHallOfFame",
    ]);
    for (name, original) in &base.data.maps {
        let new = &child.data.maps[name];
        if !story_maps.contains(name.as_str()) {
            assert!(new == original, "all nonstory maps remain exact: {name}");
        }
        if matches!(
            original.attributes.environment.as_deref(),
            Some("TOWN" | "ROUTE")
        ) {
            assert_eq!(
                new.attributes, original.attributes,
                "outdoor geography {name}"
            );
            assert_eq!(new.blocks, original.blocks, "outdoor terrain {name}");
        }
    }
    // Check every authored flavor label against its already-normalized parent,
    // rather than importing old source text or trusting the story-map allowlist.
    for file in [
        "interior-life/stories.json",
        "interior-life/current-root-v01/story-supplement.json",
    ] {
        let plan: Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(file)).expect("room plan"))
                .expect("authored room JSON");
        for room in plan["rooms"].as_array().expect("rooms") {
            let name = room["map"].as_str().expect("room");
            let original = &base.data.maps[name];
            let new = &child.data.maps[name];
            for text in room["texts"].as_array().into_iter().flatten() {
                let label = text["label"].as_str().expect("flavor text");
                assert_eq!(
                    new.scripts.get(label),
                    original.scripts.get(label),
                    "room story {name}/{label}"
                );
            }
            for resident in room["conversations"].as_array().into_iter().flatten() {
                let script = resident["script"].as_str().expect("resident script");
                for label in [
                    script.to_owned(),
                    format!("{script}InteriorText"),
                    format!("{script}InteriorRepeatText"),
                    format!("{script}InteriorNeedText"),
                    format!("{script}InteriorFullText"),
                ] {
                    if let Some(body) = original.scripts.get(&label) {
                        assert_eq!(
                            new.scripts.get(&label),
                            Some(body),
                            "retained resident {name}/{label}"
                        );
                    }
                }
                if !resident["actor"].is_null() {
                    let actor = original
                        .objects
                        .iter()
                        .find(|o| o.script == script)
                        .expect("authored resident");
                    assert!(
                        new.objects.contains(actor),
                        "resident identity and position {name}/{script}"
                    );
                }
            }
        }
    }
    for name in child
        .data
        .maps
        .keys()
        .filter(|n| !base.data.maps.contains_key(*n))
    {
        assert_eq!(
            child
                .data
                .assemble_map_module_from_compiled_payloads(name)
                .expect("native new-room mirrors"),
            child.data.maps[name]
        );
    }
    assert!(!child.report.manifests.iter().any(|m| m == super::COMPOSING));
    let contract: Value = serde_json::from_str(&source).expect("composition JSON");
    for (path, value) in [
        ("schema", json!(2)),
        ("hash", json!("unreviewed")),
        ("maps", json!(0)),
        ("review", json!("")),
    ] {
        let mut bad = contract.clone();
        match path {
            "schema" => bad["schema_version"] = value,
            "hash" => bad["approved_parents"][0]["content_hash"] = value,
            "maps" => bad["approved_parents"][0]["maps"] = value,
            _ => bad["approved_parents"][0]["review"] = value,
        }
        assert!(
            super::build_northstar_current_story_from_composition(&base, &root, &bad.to_string())
                .is_err(),
            "reject stale/unreviewed composition {path}"
        );
    }
    let mut duplicate = contract;
    duplicate["approved_parents"][0]["content_hash"] =
        json!(child.identity().expect("child identity").content_hash);
    duplicate["approved_parents"][0]["maps"] = json!(child.data.maps.len());
    assert!(
        super::build_northstar_current_story_from_composition(
            &child,
            &root,
            &duplicate.to_string()
        )
        .is_err(),
        "an already-installed story cannot be composed again"
    );
}
