//! Strict preservation outside the drill rooms and spoken depot bridge.
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::grand_marais_modpack::authoring::command as c;

#[test]
#[ignore = "requires the external ICE chapter; checks every data field and audio program"]
fn drill_chapter_preserves_unrelated_content_and_requires_live_crew_evidence() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let base = crate::read_verified_compiled_game_pack(
        repo.join("content-packs/minnesota-northstar-ice-lair-v4.crystalpack")
            .canonicalize()
            .expect("parent"),
    )
    .expect("verified ICE chapter");
    let source = |name: &str| {
        std::fs::read_to_string(repo.join("planning/minnesota/lairs").join(name))
            .expect("authored source")
    };
    let result = crate::build_northstar_drill_lair(
        &base,
        &source("iron-events.json"),
        &source("iron-approach.json"),
        &source("iron-gallery.json"),
        &source("iron-core.json"),
    )
    .expect("native drilling chapter");
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.audio_manifest().expect("audio") == result.audio_manifest().expect("audio"));
    assert!(base.runtime_files() == result.runtime_files());
    let before = serde_json::to_value(base.data()).expect("parent data");
    let after = serde_json::to_value(result.data()).expect("chapter data");
    let changed = BTreeSet::from([
        "maps",
        "map_scripts",
        "npcs",
        "map_attributes",
        "map_dimensions",
        "map_blocks",
        "runtime_map_metadata",
        "story_event_script_constants",
        "trainers",
        "pokegear_landmarks",
    ]);
    for (field, original) in before.as_object().expect("data fields") {
        if !changed.contains(field.as_str()) {
            assert!(original == &after[field], "unrelated dataset {field}");
        }
    }
    let rooms = BTreeSet::from([
        "MnR09Cave1F",
        "MnStPaulUnionDepot",
        "MnIronDrillApproach",
        "MnIronPumpGallery",
        "MnIronDrillCore",
    ]);
    let mut labels = BTreeSet::new();
    for name in &rooms {
        let map = &result.data().maps[*name];
        labels.extend(map.scripts.keys().map(String::as_str));
        labels.insert(
            map.attributes
                .map_scripts_label
                .as_deref()
                .expect("script section"),
        );
        labels.insert(
            map.attributes
                .map_events_label
                .as_deref()
                .expect("event section"),
        );
    }
    for (name, original) in before["maps"].as_object().expect("maps") {
        if !rooms.contains(name.as_str()) {
            assert!(original == &after["maps"][name], "unrelated map {name}");
        }
    }
    for (name, original) in &base.data().map_scripts {
        if !labels.contains(name.as_str()) {
            assert!(
                result.data().map_scripts.get(name) == Some(original),
                "unrelated script {name}"
            );
        }
    }
    for (id, trainer) in &base.data().trainers.trainers {
        assert_eq!(
            &result.data().trainers.trainers[id],
            trainer,
            "original trainer {id}"
        );
    }
    assert_eq!(result.data().maps.len(), base.data().maps.len() + 3);
    assert_eq!(
        base.data().runtime_spawn_points,
        result.data().runtime_spawn_points,
        "dungeons are not recovery/Fly points"
    );
    let core = &result.data().maps["MnIronDrillCore"];
    assert_eq!(
        core.script_audio_commands
            .iter()
            .filter(|a| a.source_script == "MnNDrillWitness"
                && a.audio_id.as_deref() == Some("SFX_STRENGTH"))
            .count(),
        2,
        "both native piston cycles need actual sound mutation indices"
    );
    for (name, result_flag) in [
        ("MnR09Cave1F", "EVENT_MN_MINE_INSTRUMENT_RECOVERED"),
        ("MnStPaulUnionDepot", "EVENT_MN_DEEP_BORE_MANIFEST_READ"),
    ] {
        let expected = if name == "MnStPaulUnionDepot" {
            "MN_NSTP_MINA"
        } else {
            "MN_NDRILL_FINN"
        };
        for (label, program) in &result.data().maps[name].scripts {
            if program
                .as_array()
                .is_some_and(|rows| rows.contains(&c("setevent", &[result_flag])))
            {
                assert_eq!(label, expected, "found text cannot grant the human quest");
            }
        }
    }
    let crew = core.scripts["MN_NDRILL_FINN"]
        .as_array()
        .expect("crew quest");
    let granted = crew
        .iter()
        .position(|row| *row == c("setevent", &["EVENT_MN_MINE_INSTRUMENT_RECOVERED"]))
        .expect("crew grant");
    assert_eq!(crew[granted - 1], c("waitbutton", &[]));
    assert!(
        crate::build_northstar_drill_lair(
            &result,
            &source("iron-events.json"),
            &source("iron-approach.json"),
            &source("iron-gallery.json"),
            &source("iron-core.json")
        )
        .is_err()
    );
}
