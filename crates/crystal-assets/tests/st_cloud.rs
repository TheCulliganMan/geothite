//! St Cloud's canonical wind gym and two checked regional walking mouths.
use crystal_assets::{CompiledGamePack, build_st_cloud_modpack, read_verified_compiled_game_pack};
use std::path::PathBuf;
use std::sync::OnceLock;
struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    source: [String; 4],
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-st-paul-gym-remake.crystalpack"),
        )
        .expect("completed St Paul input");
        let source = [
            "towns/MnStCloud.json",
            "towns/MnStCloud.layout.json",
            "transit/MnStCloudStation.json",
            "gyms/interiors.json",
        ]
        .map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p))
                .expect("authored source")
        });
        let result = build_st_cloud_modpack(&base, &source[0], &source[1], &source[2], &source[3])
            .expect("St Cloud publication");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-st-cloud-gym-remake.crystalpack"),
        )
        .expect("written St Cloud");
        assert_eq!(
            result.identity().expect("built identity"),
            written.identity().expect("written identity")
        );
        Inputs {
            base,
            result,
            source,
        }
    })
}
#[test]
#[ignore = "requires completed external St Paul and freshly published St Cloud packs"]
fn st_cloud_preserves_native_wind_world_audio_home_and_recovery() {
    let i = input();
    assert_eq!(i.result.data().maps.len(), i.base.data().maps.len() + 4);
    let marker = i
        .result
        .data()
        .pokegear_landmarks
        .landmarks
        .iter()
        .find(|l| l.constant == "LANDMARK_MN_ST_CLOUD")
        .expect("standalone city landmark");
    assert_eq!(marker.name, "ST CLOUD");
    assert!(
        marker.id <= 0x7f,
        "city steps require a native caught-location area"
    );
    for (name, before) in &i.base.data().maps {
        let mut expected = before.clone();
        if name == "MnStCloudGym" {
            expected.attributes.location = Some("LANDMARK_MN_ST_CLOUD".into());
            for warp in &mut expected.events.warps {
                warp.target_map = "MN_ST_CLOUD".into();
                warp.target_map_constant = "MN_ST_CLOUD".into();
                warp.target_warp_id = 6;
            }
            for command in &mut expected.map_event_section_commands {
                if command.command == "warp_event" {
                    command.args[2] = "MN_ST_CLOUD".into();
                    command.args[3] = "6".into();
                }
            }
        } else if name == "MnR05C00R00" || name == "MnR06C00R01" {
            let (direction, offset) = if name == "MnR05C00R00" {
                ("north", 0)
            } else {
                ("south", -4)
            };
            expected
                .attributes
                .connections
                .push(crystal_core::map::MapConnection {
                    direction: direction.into(),
                    target_map: "MnStCloud".into(),
                    offset,
                });
            expected.attributes.connection_flags = Some(
                expected
                    .attributes
                    .connections
                    .iter()
                    .map(|c| c.direction.to_ascii_uppercase())
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
        }
        assert_eq!(
            i.result.data().maps[name],
            expected,
            "unintended change to {name}"
        );
    }
    assert_eq!(i.result.data().trainers, i.base.data().trainers);
    assert_eq!(i.result.data().tilesets, i.base.data().tilesets);
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    for (path, bytes) in i.base.runtime_files() {
        assert_eq!(
            i.result.runtime_files().get(path),
            Some(bytes),
            "runtime file {path}"
        );
    }
    for (id, spawn) in &i.base.data().runtime_spawn_points {
        assert_eq!(
            i.result.data().runtime_spawn_points.get(id),
            Some(spawn),
            "existing recovery {id}"
        );
    }
    let new = i
        .result
        .data()
        .runtime_spawn_points
        .iter()
        .filter(|(id, _)| !i.base.data().runtime_spawn_points.contains_key(*id))
        .map(|(_, s)| s.map_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        new,
        ["MnStCloud", "MnStCloudPokecenter1F"].into_iter().collect()
    );
}
#[test]
#[ignore = "requires completed external St Paul and freshly published St Cloud packs"]
fn st_cloud_rejects_stale_gym_unsafe_facades_boarding_arrivals_and_duplicates() {
    let i = input();
    for mutation in 0..3 {
        let mut town: serde_json::Value = serde_json::from_str(&i.source[0]).expect("town");
        match mutation {
            0 => {
                town["interiors"][2]["source_reference"]["room_sha256"] = serde_json::json!("stale")
            }
            1 => town["doors"][2]["tile"] = serde_json::json!([9, 27]),
            2 => town["doors"][3]["target_warp"] = serde_json::json!(1),
            _ => unreachable!(),
        }
        assert!(
            build_st_cloud_modpack(
                &i.base,
                &town.to_string(),
                &i.source[1],
                &i.source[2],
                &i.source[3]
            )
            .is_err(),
            "mutation {mutation}"
        );
    }
    assert!(
        build_st_cloud_modpack(
            &i.result,
            &i.source[0],
            &i.source[1],
            &i.source[2],
            &i.source[3]
        )
        .is_err()
    );
}
