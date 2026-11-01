//! External-pack preservation and rejection of premature ICE chapter completion.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, build_minneapolis_modpack};
use serde_json::Value;

fn inputs() -> &'static (CompiledGamePack, Vec<String>) {
    static INPUTS: OnceLock<(CompiledGamePack, Vec<String>)> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let predecessor = std::env::var_os("CRYSTAL_MINNEAPOLIS_PREDECESSOR_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-gyms-minneapolis.crystalpack"));
        let base = crystal_assets::read_verified_compiled_game_pack(predecessor)
            .expect("current canonical Gym3 review predecessor");
        let sources = [
            "towns/MnMinneapolis.json",
            "towns/MnMinneapolis.layout.json",
            "story.json",
        ]
        .into_iter()
        .map(|p| std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("source"))
        .collect();
        (base, sources)
    })
}
fn publish(base: &CompiledGamePack, s: &[String]) -> anyhow::Result<CompiledGamePack> {
    build_minneapolis_modpack(base, &s[0], &s[1], &s[2])
}

#[test]
#[ignore = "requires the published canonical Minneapolis gym pack"]
fn minneapolis_adds_only_authored_city_services_and_three_separate_ice_milestones() {
    let (base, source) = inputs();
    let pack = publish(base, source).expect("native Minneapolis");
    let names = [
        "MnMinneapolis",
        "MnMinneapolisPokecenter1F",
        "MnMinneapolisMart",
        "MnMinneapolisDispatchOffice",
        "MnMinneapolisDispatcherHouse",
    ];
    assert_eq!(pack.data().maps.len(), base.data().maps.len() + names.len());
    let old = &base.data().maps["MnMinneapolisGym"];
    let gym = &pack.data().maps["MnMinneapolisGym"];
    let mut normalized = gym.clone();
    normalized.events.warps = old.events.warps.clone();
    normalized.attributes.location = old.attributes.location.clone();
    normalized.map_event_section_commands = old.map_event_section_commands.clone();
    assert!(
        normalized == *old,
        "installed gym changed beyond the paired returns and geographic landmark"
    );
    assert!(
        gym.events
            .warps
            .iter()
            .all(|w| w.target_map_constant == "MN_MINNEAPOLIS" && w.target_warp_id == 4)
    );
    let office = &pack.data().maps["MnMinneapolisDispatchOffice"];
    assert_eq!(office.scripted_trainer_battles.len(), 1);
    assert!(
        office
            .script_map_commands
            .iter()
            .any(|c| c.command == "reloadmapafterbattle"),
        "victory continuation needs its actual native map mutation"
    );
    assert_eq!(office.objects.len(), 2);
    assert!(office.script_item_grants.is_empty() && office.script_economy_commands.is_empty());
    assert_eq!(
        office
            .script_flag_commands
            .iter()
            .filter(|c| c.command == "setevent")
            .map(|c| (c.source_script.as_str(), c.flag_id.as_str()))
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            ("MN_ICE_ENFORCER", "EVENT_MN_ICE_ENFORCER_DEFEATED"),
            ("MnMPCellLatch", "EVENT_MN_ICE_CAPTIVE_RESCUED"),
            ("MnMPDispatchCopy", "EVENT_MN_DISPATCH_NOTE"),
        ])
    );
    assert!(
        !office
            .script_flag_commands
            .iter()
            .any(|c| c.command == "setflag")
    );
    assert!(
        office
            .script_movements
            .iter()
            .any(|m| m.label == "MnMPCaptiveLeaves")
    );
    for name in names.into_iter().chain(["MnMinneapolisGym"]) {
        let mut uncached = pack.data().clone();
        uncached.maps.remove(name);
        let mut assembled = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached production split assembly");
        let expected = &pack.data().maps[name];
        for label in [
            expected.attributes.map_scripts_label.as_ref(),
            expected.attributes.map_events_label.as_ref(),
        ] {
            let label = label.expect("owned transport section");
            if !expected.scripts.contains_key(label) {
                assembled.scripts.remove(label);
            }
        }
        let mut mismatches = Vec::new();
        differences(
            &serde_json::to_value(assembled).expect("assembled JSON"),
            &serde_json::to_value(expected).expect("expected JSON"),
            "",
            &mut mismatches,
        );
        assert!(
            mismatches.is_empty(),
            "uncached split mirror: {name}: {mismatches:#?}"
        );
    }
    let mut restored = pack.data().clone();
    for name in names {
        let map = &pack.data().maps[name];
        let constant = map.attributes.map_constant.as_ref().expect("constant");
        restored.maps.remove(name);
        restored.npcs.remove(name);
        restored.map_attributes.remove(name);
        restored.map_dimensions.remove(constant);
        restored.runtime_map_metadata.remove(constant);
        restored
            .map_blocks
            .remove(map.attributes.blocks_label.as_ref().expect("blocks"));
        for label in map
            .scripts
            .keys()
            .chain(map.script_text_bodies.keys())
            .chain(map.attributes.map_scripts_label.iter())
            .chain(map.attributes.map_events_label.iter())
        {
            restored.map_scripts.remove(label);
        }
        restored.pokegear_landmarks.map_to_landmark.remove(name);
    }
    restored.maps.insert("MnMinneapolisGym".into(), old.clone());
    restored.map_attributes.insert(
        "MnMinneapolisGym".into(),
        base.data().map_attributes["MnMinneapolisGym"].clone(),
    );
    restored.pokegear_landmarks.map_to_landmark.insert(
        "MnMinneapolisGym".into(),
        base.data().pokegear_landmarks.map_to_landmark["MnMinneapolisGym"].clone(),
    );
    restored.map_scripts.insert(
        "MnMinneapolisGym_MapEvents".into(),
        base.data().map_scripts["MnMinneapolisGym_MapEvents"].clone(),
    );
    assert_eq!(
        restored.pokegear_landmarks.landmarks.len(),
        base.data().pokegear_landmarks.landmarks.len() + 1
    );
    restored
        .pokegear_landmarks
        .landmarks
        .retain(|l| l.constant != "LANDMARK_MN_MINNEAPOLIS");
    let added = restored
        .runtime_spawn_points
        .iter()
        .filter(|(k, _)| !base.data().runtime_spawn_points.contains_key(*k))
        .map(|(_, v)| v)
        .collect::<Vec<_>>();
    assert_eq!(added.len(), 2);
    assert!(added.iter().all(|s| s.group_id == 72
        && ["MnMinneapolis", "MnMinneapolisPokecenter1F"].contains(&s.map_name.as_str())));
    restored
        .runtime_spawn_points
        .retain(|k, _| base.data().runtime_spawn_points.contains_key(k));
    restored.marts.0.remove("MART_MN_MINNEAPOLIS");
    let trainer = restored
        .trainers
        .trainers
        .remove("MN_ICE_ENFORCER")
        .expect("new trainer");
    assert_eq!(trainer.trainer_class, "GRUNTM");
    assert_eq!(
        trainer
            .party
            .iter()
            .map(|p| (p.species.as_str(), p.level))
            .collect::<Vec<_>>(),
        [("RATICATE", 19), ("GROWLITHE", 20)]
    );
    for flag in [
        "MAPGROUP_MINNESOTA_MINNEAPOLIS",
        "MN_ICE_ENFORCER",
        "EVENT_MN_ICE_ENFORCER_DEFEATED",
        "EVENT_MN_ICE_CAPTIVE_RESCUED",
        "EVENT_MN_DISPATCH_NOTE",
    ] {
        if !base
            .data()
            .story_event_script_constants
            .global
            .contains_key(flag)
        {
            restored.story_event_script_constants.global.remove(flag);
        }
    }
    assert!(
        restored == *base.data(),
        "unrelated original map/catalog/HOME/recovery/story content changed"
    );
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(
        pack.audio_manifest().expect("audio"),
        base.audio_manifest().expect("old audio")
    );
    assert!(publish(&pack, source).is_err(), "duplicate installation");
}

#[test]
#[ignore = "requires the published canonical Minneapolis gym pack"]
fn minneapolis_rejects_skipped_battle_rescue_and_stale_gym_or_gate() {
    let (base, source) = inputs();
    for change in [
        "skip_battle",
        "skip_rescue",
        "extra_reward",
        "changed_gym",
        "open_cell",
    ] {
        let mut changed = source.clone();
        if matches!(change, "skip_battle" | "skip_rescue" | "extra_reward") {
            let mut story: Value = serde_json::from_str(&changed[2]).expect("story");
            let id = if change == "skip_battle" {
                "MS03_RESCUE"
            } else {
                "MS03"
            };
            let ms = story["milestones"]
                .as_array_mut()
                .expect("milestones")
                .iter_mut()
                .find(|m| m["id"] == id)
                .expect("chapter");
            if change == "extra_reward" {
                ms["sets"]
                    .as_array_mut()
                    .expect("results")
                    .push(serde_json::json!("BADGE_3"));
            } else {
                ms["requires"].as_array_mut().expect("requirements").pop();
            }
            changed[2] = story.to_string();
        } else {
            let mut town: Value = serde_json::from_str(&changed[0]).expect("town");
            if change == "changed_gym" {
                town["interiors"][1]["source_reference"]["native_content_sha256"] =
                    serde_json::json!("stale");
            } else {
                town["ice_scene"]["release"]["closed_block"] = serde_json::json!("05");
            }
            changed[0] = town.to_string();
        }
        assert!(publish(base, &changed).is_err(), "accepted {change}");
    }
}

#[test]
#[ignore = "requires the external native source and canonical gym packs"]
fn minneapolis_accepts_the_same_gym_from_the_full_eight_room_review() {
    let (_, source) = inputs();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = crystal_assets::read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-boundary-basecamp-connected.crystalpack"),
    )
    .expect("complete external predecessor");
    let design = std::fs::read_to_string(root.join("planning/minnesota/gyms/interiors.json"))
        .expect("canonical gym source");
    let all = crystal_assets::build_minnesota_gym_review_pack(&base, &design)
        .expect("full eight-room fixture, not visual acceptance");
    let result =
        publish(&all, source).expect("city binding ignores only the old warp destinations");
    assert_eq!(result.data().maps["MnMinneapolisGym"].attributes.width, 5);
    assert!(
        result.data().maps["MnMinneapolisGym"]
            .events
            .warps
            .iter()
            .all(|warp| warp.target_map_constant == "MN_MINNEAPOLIS" && warp.target_warp_id == 4)
    );
}

fn differences(actual: &Value, expected: &Value, path: &str, output: &mut Vec<String>) {
    if actual == expected {
        return;
    }
    match (actual, expected) {
        (Value::Object(a), Value::Object(b)) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                differences(
                    a.get(key).unwrap_or(&Value::Null),
                    b.get(key).unwrap_or(&Value::Null),
                    &format!("{path}/{key}"),
                    output,
                );
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                differences(a, b, &format!("{path}/{index}"), output);
            }
        }
        _ => output.push(format!("{path}: assembled={actual} cached={expected}")),
    }
}
