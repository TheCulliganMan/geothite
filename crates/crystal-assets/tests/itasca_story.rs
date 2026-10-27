//! Bounded publication, original gate art and independent story conditions.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_itasca_story_modpack, read_verified_compiled_game_pack,
};
use serde_json::{Value, json};

struct Input {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 5],
}
fn input() -> &'static Input {
    static INPUT: OnceLock<Input> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-leech-shore.crystalpack"),
        )
        .expect("shore base");
        let source = [
            "towns/MnLakeItasca.json",
            "towns/MnLakeItasca.layout.json",
            "story.json",
            "routes/MN_R14.json",
            "routes/MN_R09.json",
        ]
        .map(|p| std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("source"));
        let result = build(&base, &source).expect("Itasca publication");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-itasca-story.crystalpack"),
        )
        .expect("written Itasca");
        assert_eq!(
            result.identity().expect("identity"),
            written.identity().expect("written identity")
        );
        Input {
            base,
            result,
            source,
        }
    })
}
fn build(base: &CompiledGamePack, s: &[String; 5]) -> anyhow::Result<CompiledGamePack> {
    build_itasca_story_modpack(base, &s[0], &s[1], &s[2], &s[3], &s[4])
}

#[test]
#[ignore = "requires completed external shore and Itasca packs"]
fn itasca_story_preserves_unrelated_maps_audio_recovery_and_uncached_lowering() {
    let i = input();
    let old = i.base.data();
    let new = i.result.data();
    let changed = ["MnBemidji", "MnR14C00R00", "MnR14C01R00", "MnR09C00R01"];
    let added = ["MnLakeItasca", "MnItascaGatehouse"];
    assert_eq!(new.maps.len(), old.maps.len() + 2);
    for (name, m) in &old.maps {
        let actual = &new.maps[name];
        if !changed.contains(&name.as_str()) {
            assert_eq!(actual, m, "unrelated map {name}");
            continue;
        }
        assert_eq!(&actual.objects[..m.objects.len()], &m.objects);
        for (label, body) in &m.scripts {
            if label != "MN_BEMIDJI_OFFICER" {
                assert_eq!(
                    actual.scripts.get(label),
                    Some(body),
                    "existing script {label}"
                );
            }
        }
        if name != "MnR14C00R00" {
            assert_eq!(actual.events.warps, m.events.warps);
        }
        let expected = match name.as_str() {
            "MnBemidji" => vec![12 * 24, 12 * 24 + 23],
            "MnR14C00R00" => vec![20 * 32, 20 * 32 + 1, 21 * 32, 21 * 32 + 1, 21 * 32 + 2],
            _ => vec![],
        };
        assert_eq!(
            m.blocks
                .iter()
                .zip(&actual.blocks)
                .enumerate()
                .filter_map(|(n, (a, b))| (a != b).then_some(n))
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert_eq!(
        new.maps["MnItascaGatehouse"].blocks,
        old.maps["IlexForestAzaleaGate"].blocks
    );
    assert_eq!(new.maps["MnItascaGatehouse"].events.warps.len(), 8);
    for name in changed.into_iter().chain(added) {
        let expected = &new.maps[name];
        let mut uncached = new.clone();
        uncached.maps.remove(name);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached map");
        for label in expected
            .attributes
            .map_scripts_label
            .iter()
            .chain(expected.attributes.map_events_label.iter())
        {
            if !expected.scripts.contains_key(label) {
                rebuilt.scripts.remove(label);
            }
        }
        assert_eq!(&rebuilt, expected, "uncached {name}");
    }
    assert_eq!(
        new.runtime_spawn_points, old.runtime_spawn_points,
        "no new HOME/blackout destination"
    );
    for (k, v) in &old.runtime_map_metadata {
        assert_eq!(new.runtime_map_metadata.get(k), Some(v));
    }
    for (k, v) in &old.story_event_script_constants.global {
        assert_eq!(new.story_event_script_constants.global.get(k), Some(v));
    }
    assert_eq!(
        new.story_event_script_constants.global.len(),
        old.story_event_script_constants.global.len() + 2
    );
    assert_eq!(
        new.story_event_script_constants.global["MAPGROUP_MINNESOTA_ITASCA"],
        79
    );
    assert_eq!(
        new.story_event_script_constants.global["EVENT_MN_HEADWATERS_READING"],
        2308
    );
    let mut restored = new.clone();
    restored.maps = old.maps.clone();
    restored.map_scripts = old.map_scripts.clone();
    restored.map_blocks = old.map_blocks.clone();
    restored.map_dimensions = old.map_dimensions.clone();
    restored.map_attributes = old.map_attributes.clone();
    restored.npcs = old.npcs.clone();
    restored.runtime_map_metadata = old.runtime_map_metadata.clone();
    restored.story_event_script_constants = old.story_event_script_constants.clone();
    restored.pokegear_landmarks = old.pokegear_landmarks.clone();
    assert_eq!(restored, *old, "all other game data exact");
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
}

#[test]
#[ignore = "requires completed external shore and Itasca packs"]
fn itasca_story_rejects_unsafe_gate_arrivals_floorplan_changes_bypasses_and_extra_gates() {
    let i = input();
    for mutation in 0..6 {
        let mut s = i.source.clone();
        let mut town: Value = serde_json::from_str(&s[0]).expect("town");
        let mut layout: Value = serde_json::from_str(&s[1]).expect("layout");
        let mut route: Value = serde_json::from_str(&s[3]).expect("road");
        match mutation {
            0 => town["gatehouse_contract"]["passive_arrivals"][0]["tile"] = json!([0, 4]),
            1 => town["interiors"][0]["exits"][2]["target_warp"] = json!(1),
            2 => town["interiors"][0]["rows"][0] = json!("01 01 01 01 01"),
            3 => layout["layers"]
                .as_array_mut()
                .expect("layers")
                .push(json!({"name":"rail gap","kind":"point","position":[11,12],"block":"01"})),
            4 => layout["layers"]
                .as_array_mut()
                .expect("layers")
                .push(json!({"name":"park bypass","kind":"point","position":[23,13],"block":"01"})),
            5 => route["required_flags"] = json!(["BADGE_6", "EVENT_MN_BRAINERD_REPORT"]),
            _ => unreachable!(),
        }
        s[0] = town.to_string();
        s[1] = layout.to_string();
        s[3] = route.to_string();
        assert!(build(&i.base, &s).is_err(), "mutation {mutation}");
    }
    assert!(build(&i.result, &i.source).is_err());
}
