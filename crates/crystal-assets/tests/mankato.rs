//! Mankato binds a canonical gym and preserves every unrelated map/catalog.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{CompiledGamePack, build_mankato_modpack, read_verified_compiled_game_pack};

struct Inputs {
    base: CompiledGamePack,
    source: [String; 3],
    result: CompiledGamePack,
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-gyms-remake.crystalpack"),
        )
        .expect("current eight-gym input");
        let source = [
            "towns/MnMankato.json",
            "towns/MnMankato.layout.json",
            "gyms/interiors.json",
        ]
        .map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p))
                .expect("authored source")
        });
        let result = build_mankato_modpack(&base, &source[0], &source[1], &source[2])
            .expect("Mankato publication");
        let published = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-mankato-gym-remake.crystalpack"),
        )
        .expect("written candidate");
        assert_eq!(
            result.identity().expect("built identity"),
            published.identity().expect("written identity")
        );
        Inputs {
            base,
            source,
            result,
        }
    })
}

#[test]
#[ignore = "requires the external eight-gym input and freshly published Mankato pack"]
fn mankato_preserves_world_and_changes_only_gym_city_wires() {
    let i = input();
    assert_eq!(i.result.data().maps.len(), i.base.data().maps.len() + 3);
    for (id, before) in &i.base.data().maps {
        let after = &i.result.data().maps[id];
        if id == "MnMankatoGym" {
            let mut expected = before.clone();
            expected.attributes.location = Some("LANDMARK_MN_MANKATO".into());
            for w in &mut expected.events.warps {
                w.target_map_constant = "MN_MANKATO".into();
                w.target_map = "MN_MANKATO".into();
                w.target_warp_id = 6;
            }
            for c in &mut expected.map_event_section_commands {
                if c.command == "warp_event" {
                    c.args[2] = "MN_MANKATO".into();
                    c.args[3] = "6".into();
                }
            }
            assert_eq!(after, &expected, "gym content must remain canonical");
        } else {
            assert_eq!(after, before, "unrelated map {id}");
        }
    }
    assert_eq!(i.result.data().trainers, i.base.data().trainers);
    assert_eq!(i.result.data().tilesets, i.base.data().tilesets);
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        serde_json::to_value(i.result.audio_manifest().expect("published audio"))
            .expect("audio JSON"),
        serde_json::to_value(i.base.audio_manifest().expect("base audio")).expect("audio JSON")
    );
    for (path, bytes) in i.base.runtime_files() {
        assert_eq!(
            i.result.runtime_files().get(path),
            Some(bytes),
            "runtime file {path}"
        );
    }
    for (key, spawn) in &i.base.data().runtime_spawn_points {
        assert_eq!(
            i.result.data().runtime_spawn_points.get(key),
            Some(spawn),
            "recovery {key}"
        );
    }
    let additions = i
        .result
        .data()
        .runtime_spawn_points
        .iter()
        .filter(|(key, _)| !i.base.data().runtime_spawn_points.contains_key(*key))
        .map(|(_, s)| s.map_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        additions,
        ["MnMankato", "MnMankatoPokecenter1F"].into_iter().collect()
    );
}

#[test]
#[ignore = "requires the external eight-gym input and freshly published Mankato pack"]
fn mankato_rejects_stale_canonical_rooms_bad_facades_and_missing_ledge() {
    let i = input();
    let mut town: serde_json::Value = serde_json::from_str(&i.source[0]).expect("town");
    town["interiors"][2]["source_reference"]["room_sha256"] = serde_json::json!("stale");
    assert!(build_mankato_modpack(&i.base, &town.to_string(), &i.source[1], &i.source[2]).is_err());
    let mut town: serde_json::Value = serde_json::from_str(&i.source[0]).expect("town");
    town["doors"][2]["tile"] = serde_json::json!([11, 11]);
    assert!(build_mankato_modpack(&i.base, &town.to_string(), &i.source[1], &i.source[2]).is_err());
    let mut layout: serde_json::Value = serde_json::from_str(&i.source[1]).expect("layout");
    layout["layers"]
        .as_array_mut()
        .expect("strokes")
        .retain(|layer| layer["name"] != "One-way terrace drop");
    assert!(
        build_mankato_modpack(&i.base, &i.source[0], &layout.to_string(), &i.source[2]).is_err()
    );
    let mut gyms: serde_json::Value = serde_json::from_str(&i.source[2]).expect("gyms");
    gyms["gyms"][1]["leader"] = serde_json::json!("OTHER");
    assert!(build_mankato_modpack(&i.base, &i.source[0], &i.source[1], &gyms.to_string()).is_err());
    assert!(build_mankato_modpack(&i.result, &i.source[0], &i.source[1], &i.source[2]).is_err());
}
