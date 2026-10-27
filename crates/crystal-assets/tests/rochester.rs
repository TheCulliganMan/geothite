//! Bounded Rochester/Gym1/seam publication and unrelated-world preservation.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, build_rochester_modpack, connect_winona_rochester,
    read_verified_compiled_game_pack,
};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: Vec<String>,
}
fn differences(a: &serde_json::Value, b: &serde_json::Value, path: &str, out: &mut Vec<String>) {
    if a == b || out.len() >= 16 {
        return;
    }
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                differences(
                    a.get(key).unwrap_or(&serde_json::Value::Null),
                    b.get(key).unwrap_or(&serde_json::Value::Null),
                    &format!("{path}.{key}"),
                    out,
                );
            }
        }
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
            if a.len() != b.len() {
                out.push(format!("{path}.length {} != {}", a.len(), b.len()));
            }
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                differences(a, b, &format!("{path}[{i}]"), out);
            }
        }
        _ => out.push(path.to_owned()),
    }
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-winona.crystalpack"),
        )
        .expect("completed Winona input");
        let source = [
            "towns/MnRochester.json",
            "towns/MnRochester.layout.json",
            "story.json",
            "revisions/rochester-gym1.json",
            "playable/rochester-links.json",
        ]
        .map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p))
                .expect("authored source")
        })
        .to_vec();
        let city = build_rochester_modpack(&base, &source[0], &source[1], &source[2], &source[3])
            .expect("city");
        let result = connect_winona_rochester(&city, &source[4]).expect("native seams");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-rochester.crystalpack"),
        )
        .expect("written Rochester candidate");
        assert_eq!(
            result.identity().expect("built identity"),
            written.identity().expect("written identity"),
            "written pack differs from current authored build"
        );
        Inputs {
            base,
            result,
            source,
        }
    })
}
#[test]
#[ignore = "requires the external Winona pack"]
fn rochester_preserves_unrelated_world_and_reconstructs_all_published_modules_uncached() {
    let i = input();
    let before = i.base.data();
    let after = i.result.data();
    let mut restored = after.clone();
    let new = [
        "MnRochester",
        "MnRochesterPokecenter1F",
        "MnRochesterMart",
        "MnRochesterClerkHouse",
        "MnRochesterPatientHouse",
    ];
    let changed = ["MnWinona", "MnR01C00R00", "MnRochesterGym"];
    assert_eq!(after.maps.len(), before.maps.len() + 5);
    for (name, m) in &before.maps {
        if !changed.contains(&name.as_str()) {
            assert_eq!(after.maps.get(name), Some(m), "unrelated map: {name}");
        }
    }
    let mut labels = BTreeSet::new();
    for name in new.into_iter().chain(changed) {
        let m = &after.maps[name];
        let mut uncached = after.clone();
        uncached.maps.remove(name);
        let mut assembled = uncached
            .assemble_map_module_from_compiled_payloads(name)
            .expect("uncached native map");
        for label in [
            m.attributes.map_scripts_label.as_ref(),
            m.attributes.map_events_label.as_ref(),
        ] {
            let label = label.expect("transport");
            if !m.scripts.contains_key(label) {
                assembled.scripts.remove(label);
            }
        }
        let mut paths = Vec::new();
        differences(
            &serde_json::to_value(&assembled).expect("assembled"),
            &serde_json::to_value(m).expect("cached"),
            name,
            &mut paths,
        );
        assert!(assembled == *m, "uncached split payload differs: {paths:?}");
        let constant = m.attributes.map_constant.as_ref().expect("constant");
        labels.extend(m.scripts.keys().chain(m.script_text_bodies.keys()).cloned());
        labels.extend(
            m.attributes
                .map_scripts_label
                .iter()
                .chain(m.attributes.map_events_label.iter())
                .cloned(),
        );
        if let Some(old) = before.maps.get(name) {
            labels.extend(
                old.scripts
                    .keys()
                    .chain(old.script_text_bodies.keys())
                    .cloned(),
            );
            restored.maps.insert(name.into(), old.clone());
            restored
                .map_attributes
                .insert(name.into(), before.map_attributes[name].clone());
            restored
                .map_dimensions
                .insert(constant.clone(), before.map_dimensions[constant].clone());
            restored.runtime_map_metadata.insert(
                constant.clone(),
                before.runtime_map_metadata[constant].clone(),
            );
            restored.npcs.insert(name.into(), before.npcs[name].clone());
            let block = old.attributes.blocks_label.as_ref().expect("blocks");
            restored
                .map_blocks
                .insert(block.clone(), before.map_blocks[block].clone());
        } else {
            restored.maps.remove(name);
            restored.map_attributes.remove(name);
            restored.map_dimensions.remove(constant);
            restored.runtime_map_metadata.remove(constant);
            restored.npcs.remove(name);
            restored
                .map_blocks
                .remove(m.attributes.blocks_label.as_ref().expect("blocks"));
        }
    }
    for key in before.map_scripts.keys().chain(after.map_scripts.keys()) {
        if !labels.contains(key) {
            assert_eq!(
                before.map_scripts.get(key),
                after.map_scripts.get(key),
                "unrelated script: {key}"
            );
        }
    }
    restored.map_scripts = before.map_scripts.clone();
    let r01 = &after.maps["MnR01C00R00"];
    let old_r01 = &before.maps["MnR01C00R00"];
    assert_eq!(r01.blocks, old_r01.blocks);
    assert_eq!(r01.objects, old_r01.objects);
    assert_eq!(after.wild_encounters, before.wild_encounters);
    for (key, point) in &before.runtime_spawn_points {
        assert_eq!(after.runtime_spawn_points.get(key), Some(point));
    }
    assert_eq!(
        after.runtime_spawn_points.len(),
        before.runtime_spawn_points.len() + 2
    );
    restored.runtime_spawn_points = before.runtime_spawn_points.clone();
    let trainers = ["MN_GYM_1", "MN_GYM_1_TRAINER_1", "MN_GYM_1_TRAINER_2"];
    for (name, t) in &before.trainers.trainers {
        if !trainers.contains(&name.as_str()) {
            assert_eq!(after.trainers.trainers.get(name), Some(t));
        }
    }
    restored.trainers = before.trainers.clone();
    for (name, value) in &before.story_event_script_constants.global {
        if !trainers.contains(&name.as_str()) {
            assert_eq!(
                after.story_event_script_constants.global.get(name),
                Some(value),
                "original native number: {name}"
            );
        }
    }
    for name in after
        .story_event_script_constants
        .global
        .keys()
        .filter(|n| !before.story_event_script_constants.global.contains_key(*n))
    {
        assert!(
            name == "MAPGROUP_MINNESOTA_ROCHESTER" || name.starts_with("EVENT_MN_GYM_1_"),
            "unexpected symbol: {name}"
        );
    }
    restored.story_event_script_constants.global =
        before.story_event_script_constants.global.clone();
    restored
        .tilesets
        .insert("mn_gym_1".into(), before.tilesets["mn_gym_1"].clone());
    restored.marts.0.remove("MART_MN_ROCHESTER");
    assert_eq!(
        after.pokegear_landmarks.landmarks.len(),
        before.pokegear_landmarks.landmarks.len() + 1
    );
    for (name, l) in &before.pokegear_landmarks.map_to_landmark {
        if name != "MnRochesterGym" {
            assert_eq!(after.pokegear_landmarks.map_to_landmark.get(name), Some(l));
        }
    }
    restored.pokegear_landmarks = before.pokegear_landmarks.clone();
    assert!(restored == *before, "unrelated dataset changed");
    let mut files = i.result.runtime_files().clone();
    for path in [
        "data/tilesets/mn_gym_1.json",
        "data/tilesets/mn_gym_1_metatiles.bin",
        "data/tilesets/mn_gym_1_palette_map.json",
        "gfx/tilesets/mn_gym_1.2bpp",
        "gfx/tilesets/mn_gym_1.png",
    ] {
        files.insert(path.into(), i.base.runtime_files()[path].clone());
    }
    assert_eq!(&files, i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.result.audio_manifest().expect("audio"),
        i.base.audio_manifest().expect("old audio")
    );
}

#[test]
#[ignore = "requires the external Winona pack"]
fn rochester_rejects_duplicate_bad_passive_arrival_premature_delivery_and_changed_gym() {
    let i = input();
    let s = &i.source;
    assert!(connect_winona_rochester(&i.result, &s[4]).is_err());
    assert!(build_rochester_modpack(&i.result, &s[0], &s[1], &s[2], &s[3]).is_err());
    let mut town: serde_json::Value = serde_json::from_str(&s[0]).expect("town");
    town["doors"][0]["arrival_tile"] = serde_json::json!([10, 15]);
    assert!(build_rochester_modpack(&i.base, &town.to_string(), &s[1], &s[2], &s[3]).is_err());
    let mut story: serde_json::Value = serde_json::from_str(&s[2]).expect("story");
    story["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS01")
        .expect("delivery")["requires"] = serde_json::json!([]);
    assert!(build_rochester_modpack(&i.base, &s[0], &s[1], &story.to_string(), &s[3]).is_err());
    let mut gym: serde_json::Value = serde_json::from_str(&s[3]).expect("gym");
    gym["gyms"][0]["party"][0]["level"] = 50.into();
    assert!(build_rochester_modpack(&i.base, &s[0], &s[1], &s[2], &gym.to_string()).is_err());
}
