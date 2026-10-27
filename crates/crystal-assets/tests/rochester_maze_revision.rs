use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_rochester_maze};
use serde_json::Value;

struct Inputs {
    base: CompiledGamePack,
    revised: CompiledGamePack,
    source: [String; 2],
}

fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-leech-shore.crystalpack"),
        )
        .expect("completed regional input");
        let source = ["gyms/interiors.json", "gyms/rochester-candidate.json"].map(|name| {
            std::fs::read_to_string(root.join("planning/minnesota").join(name))
                .expect("authored source")
        });
        let revised = revise_rochester_maze(&base, &source[0], &source[1]).expect("maze revision");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-rochester-readable.crystalpack"),
        )
        .expect("written maze candidate");
        assert_eq!(
            revised.identity().expect("identity"),
            written.identity().expect("written identity")
        );
        Inputs {
            base,
            revised,
            source,
        }
    })
}

#[test]
#[ignore = "requires the regional base and published readable Rochester pack"]
fn rochester_maze_revision_preserves_other_rooms_battles_gates_city_returns_art_and_audio() {
    let i = input();
    let a = i.base.data();
    let b = i.revised.data();
    assert_eq!(a.maps.len(), b.maps.len());
    for (name, original) in &a.maps {
        if name != "MnRochesterGym" {
            assert_eq!(original, &b.maps[name], "unrelated {name}");
        }
    }
    let original = &a.maps["MnRochesterGym"];
    let room = &b.maps["MnRochesterGym"];
    assert_ne!(room.blocks, original.blocks);
    let mut allowed = original.clone();
    allowed.blocks.clone_from(&room.blocks);
    for label in ["MnRochesterGymGuideText", "MN_GYM_1_TRAINER_1AfterText"] {
        allowed
            .scripts
            .insert(label.into(), room.scripts[label].clone());
        allowed
            .script_text_bodies
            .insert(label.into(), room.script_text_bodies[label].clone());
    }
    assert_eq!(allowed, *room, "only terrain and two hint bodies change");
    assert_eq!(a.trainers, b.trainers);
    assert_eq!(
        a.story_event_script_constants,
        b.story_event_script_constants
    );
    assert_eq!(a.tilesets, b.tilesets);
    assert_eq!(i.base.runtime_files(), i.revised.runtime_files());
    assert_eq!(i.base.compiled_audio(), i.revised.compiled_audio());
    let mut uncached = b.clone();
    uncached.maps.remove("MnRochesterGym");
    let mut rebuilt = uncached
        .assemble_map_module_from_compiled_payloads("MnRochesterGym")
        .expect("rebuild from native split projections");
    for label in [
        room.attributes.map_scripts_label.as_ref(),
        room.attributes.map_events_label.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if !room.scripts.contains_key(label) {
            rebuilt.scripts.remove(label);
        }
    }
    assert_eq!(rebuilt, *room);
}

#[test]
#[ignore = "requires the regional base and published readable Rochester pack"]
fn rochester_maze_revision_rejects_changed_battles_gates_original_and_repeat_installation() {
    let i = input();
    for field in ["party", "requires", "leader_at"] {
        let mut source: Value = serde_json::from_str(&i.source[1]).expect("candidate");
        source["gyms"][0][field] = serde_json::json!([]);
        assert!(
            revise_rochester_maze(&i.base, &i.source[0], &source.to_string()).is_err(),
            "{field}"
        );
    }
    let mut old: Value = serde_json::from_str(&i.source[0]).expect("original");
    old["gyms"][0]["leader"] = "CHANGED".into();
    assert!(revise_rochester_maze(&i.base, &old.to_string(), &i.source[1]).is_err());
    assert!(revise_rochester_maze(&i.revised, &i.source[0], &i.source[1]).is_err());
}
