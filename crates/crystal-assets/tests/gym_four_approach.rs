//! The next gym approach changes only one forest mouth and two native modules.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, build_gym_four_approach, read_verified_compiled_game_pack};
use serde_json::{Value, json};

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 3],
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-river-story.crystalpack"),
        )
        .expect("River Valley base");
        let source = [
            "playable/gym-four-approach.json",
            "routes/MN_R04.json",
            "story.json",
        ]
        .map(|path| {
            std::fs::read_to_string(root.join("planning/minnesota").join(path))
                .expect("authored contract")
        });
        let result = build_gym_four_approach(&base, &source[0], &source[1], &source[2])
            .expect("fourth-gym approach");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-gym-four-approach.crystalpack"),
        )
        .expect("written approach");
        assert_eq!(
            result.identity().expect("identity"),
            written.identity().expect("identity")
        );
        Inputs {
            base,
            result,
            source,
        }
    })
}

#[test]
#[ignore = "requires external River Valley base and written fourth-gym approach"]
fn gym_four_approach_preserves_all_gyms_route_encounters_story_and_uncached_modules() {
    let i = input();
    let before = i.base.data();
    let after = i.result.data();
    let changed = ["MnMinneapolis", "MnR04C00R00"];
    assert_eq!(after.maps.len(), before.maps.len());
    let mut labels = BTreeSet::new();
    let mut blocks = BTreeSet::new();
    for (id, old) in &before.maps {
        let new = &after.maps[id];
        if !changed.contains(&id.as_str()) {
            assert_eq!(new, old, "unrelated map {id}");
            continue;
        }
        for map in [old, new] {
            labels.extend(
                map.scripts
                    .keys()
                    .chain(map.script_text_bodies.keys())
                    .cloned(),
            );
            labels.extend(
                map.attributes
                    .map_scripts_label
                    .iter()
                    .chain(map.attributes.map_events_label.iter())
                    .cloned(),
            );
            blocks.extend(map.attributes.blocks_label.iter().cloned());
        }
        assert_eq!(
            new.events.warps, old.events.warps,
            "existing doors and pad returns"
        );
        assert_eq!(&new.objects[..old.objects.len()], old.objects.as_slice());
        assert_eq!(
            &new.attributes.connections[..old.attributes.connections.len()],
            old.attributes.connections.as_slice()
        );
        for (label, body) in &old.scripts {
            assert_eq!(new.scripts.get(label), Some(body), "existing story {label}");
        }
        let diffs = old
            .blocks
            .iter()
            .zip(&new.blocks)
            .enumerate()
            .filter_map(|(n, (a, b))| (a != b).then_some(n))
            .collect::<Vec<_>>();
        assert_eq!(
            diffs,
            if id == "MnMinneapolis" {
                vec![12 * 32 + 31]
            } else {
                vec![]
            }
        );
        let mut uncached = after.clone();
        uncached.maps.remove(id);
        let mut rebuilt = uncached
            .assemble_map_module_from_compiled_payloads(id)
            .expect("uncached native module");
        for label in [
            new.attributes.map_scripts_label.as_ref(),
            new.attributes.map_events_label.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if !new.scripts.contains_key(label) {
                rebuilt.scripts.remove(label);
            }
        }
        assert_eq!(rebuilt, *new, "split payloads agree for {id}");
    }
    for (label, body) in &before.map_scripts {
        if !labels.contains(label) {
            assert_eq!(after.map_scripts.get(label), Some(body));
        }
    }
    assert!(
        after
            .map_scripts
            .keys()
            .all(|label| before.map_scripts.contains_key(label) || labels.contains(label))
    );
    for (label, data) in &before.map_blocks {
        if !blocks.contains(label) {
            assert_eq!(after.map_blocks.get(label), Some(data));
        }
    }
    for id in before
        .map_attributes
        .keys()
        .filter(|id| !changed.contains(&id.as_str()))
    {
        assert_eq!(after.map_attributes.get(id), before.map_attributes.get(id));
        assert_eq!(after.npcs.get(id), before.npcs.get(id));
    }
    let gate = serde_json::to_string(&after.maps["MnMinneapolis"].scripts["MnMPEastGate"])
        .expect("checkpoint");
    assert!(gate.contains("ENGINE_PLAINBADGE") && !gate.contains("EVENT_MN_DISPATCH_NOTE"));
    assert!(
        !after.maps["MnMinneapolis"]
            .scripts
            .contains_key("MnMPEastGateEvidence")
    );
    let mut restored = after.clone();
    restored.maps = before.maps.clone();
    restored.map_scripts = before.map_scripts.clone();
    restored.map_blocks = before.map_blocks.clone();
    restored.map_attributes = before.map_attributes.clone();
    restored.npcs = before.npcs.clone();
    assert_eq!(
        restored, *before,
        "catalogs, events, encounters, HOME and recovery remain exact"
    );
    assert_eq!(i.base.runtime_files(), i.result.runtime_files());
    assert_eq!(i.base.compiled_audio(), i.result.compiled_audio());
    assert_eq!(
        serde_json::to_value(i.base.audio_manifest().expect("audio")).expect("JSON"),
        serde_json::to_value(i.result.audio_manifest().expect("audio")).expect("JSON")
    );
}

#[test]
#[ignore = "requires external River Valley base and written fourth-gym approach"]
fn gym_four_approach_rejects_duplicate_changed_seams_and_missing_story_prerequisites() {
    let i = input();
    assert!(build_gym_four_approach(&i.result, &i.source[0], &i.source[1], &i.source[2]).is_err());
    for (field, value) in [
        ("offset", json!(2)),
        ("requires", json!(["BADGE_3", "EVENT_MN_DISPATCH_NOTE"])),
        ("checkpoint_bodies", json!([[60, 24], [60, 26]])),
    ] {
        let mut source: Value = serde_json::from_str(&i.source[0]).expect("contract");
        source[field] = value;
        assert!(
            build_gym_four_approach(&i.base, &source.to_string(), &i.source[1], &i.source[2])
                .is_err(),
            "changed {field}"
        );
    }
    let mut story: Value = serde_json::from_str(&i.source[2]).expect("story");
    let milestone = story["quests"]
        .as_array_mut()
        .expect("quests")
        .iter_mut()
        .find(|m| m["id"] == "Q03")
        .expect("Twin Cities road");
    milestone["requires"] = json!(["BADGE_2"]);
    assert!(
        build_gym_four_approach(&i.base, &i.source[0], &i.source[1], &story.to_string()).is_err()
    );
}
