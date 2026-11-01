//! Canonical Gym4 and reciprocal city publication preserve the surrounding pack.
use crystal_assets::{CompiledGamePack, build_st_paul_modpack, read_verified_compiled_game_pack};
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
            root.join("content-packs/minnesota-mankato-gym-remake.crystalpack"),
        )
        .expect("Mankato and current gyms");
        let source = [
            "towns/MnStPaul.json",
            "towns/MnStPaul.layout.json",
            "transit/MnStPaulUnionDepot.json",
            "gyms/interiors.json",
        ]
        .map(|p| std::fs::read_to_string(root.join("planning/minnesota").join(p)).expect("source"));
        let result = build_st_paul_modpack(&base, &source[0], &source[1], &source[2], &source[3])
            .expect("St Paul");
        let written = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-st-paul-gym-remake.crystalpack"),
        )
        .expect("published St Paul");
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
#[ignore = "requires external Mankato input and freshly published St Paul pack"]
fn st_paul_preserves_canonical_pads_world_audio_home_and_recovery() {
    let i = input();
    assert_eq!(i.result.data().maps.len(), i.base.data().maps.len() + 5);
    for (name, before) in &i.base.data().maps {
        let after = &i.result.data().maps[name];
        let mut expected = before.clone();
        if name == "MnStPaulGym" {
            expected.attributes.location = Some("LANDMARK_MN_ST_PAUL".into());
            for warp in expected.events.warps.iter_mut().take(3) {
                warp.target_map = "MN_ST_PAUL".into();
                warp.target_map_constant = "MN_ST_PAUL".into();
                warp.target_warp_id = 10;
            }
            let mut count = 0;
            for command in &mut expected.map_event_section_commands {
                if command.command == "warp_event" {
                    if count < 3 {
                        command.args[2] = "MN_ST_PAUL".into();
                        command.args[3] = "10".into();
                    }
                    count += 1;
                }
            }
            assert_eq!(count, 11, "all four internal pad pairs retained");
        } else if name == "MnR04C00R00" {
            expected
                .attributes
                .connections
                .push(crystal_core::map::MapConnection {
                    direction: "east".into(),
                    target_map: "MnStPaul".into(),
                    offset: -3,
                });
            expected.attributes.connection_flags = Some("EAST".into());
        }
        assert_eq!(after, &expected, "unrelated or unintended change to {name}");
    }
    assert_eq!(i.result.data().trainers, i.base.data().trainers);
    assert_eq!(i.result.data().tilesets, i.base.data().tilesets);
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    for (path, bytes) in i.base.runtime_files() {
        assert_eq!(
            i.result.runtime_files().get(path),
            Some(bytes),
            "source file {path}"
        );
    }
    for (id, spawn) in &i.base.data().runtime_spawn_points {
        assert_eq!(
            i.result.data().runtime_spawn_points.get(id),
            Some(spawn),
            "recovery {id}"
        );
    }
    let added = i
        .result
        .data()
        .runtime_spawn_points
        .iter()
        .filter(|(id, _)| !i.base.data().runtime_spawn_points.contains_key(*id))
        .map(|(_, s)| s.map_name.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        added,
        ["MnStPaul", "MnStPaulPokecenter1F"].into_iter().collect()
    );
}
#[test]
#[ignore = "requires external Mankato input and freshly published St Paul pack"]
fn st_paul_rejects_stale_gym_unsafe_doors_train_arrivals_and_duplicate_publication() {
    let i = input();
    for mutation in 0..3 {
        let mut town: serde_json::Value = serde_json::from_str(&i.source[0]).expect("town");
        match mutation {
            0 => {
                town["interiors"][3]["source_reference"]["room_sha256"] = serde_json::json!("stale")
            }
            1 => town["doors"][3]["tile"] = serde_json::json!([55, 15]),
            2 => town["doors"][4]["target_warp"] = serde_json::json!(1),
            _ => unreachable!(),
        }
        assert!(
            build_st_paul_modpack(
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
        build_st_paul_modpack(
            &i.result,
            &i.source[0],
            &i.source[1],
            &i.source[2],
            &i.source[3]
        )
        .is_err()
    );
}
