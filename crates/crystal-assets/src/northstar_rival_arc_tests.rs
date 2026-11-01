//! Story publication preserves the lairs, source regions, native art and audio.
use std::path::PathBuf;

#[test]
#[ignore = "requires the reviewed native-art parent and complete authored rival source"]
fn rival_arc_publication_preserves_unrelated_maps_audio_species_and_original_pixels() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository");
    let base = crate::read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-arc-art-v1.crystalpack"),
    )
    .expect("reviewed art parent");
    let source = std::fs::read_to_string(root.join("planning/minnesota/story-arc-v3.json"))
        .expect("arc source");
    let layout =
        std::fs::read_to_string(root.join("planning/minnesota/lairs/rival-arc/intake.json"))
            .expect("intake layout");
    let result = crate::build_northstar_rival_arc(&base, &source, &layout).expect("native arc");
    assert_eq!(base.data().maps.len() + 1, result.data().maps.len());
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.audio_manifest().expect("audio") == result.audio_manifest().expect("audio"));
    for (key, value) in base.runtime_files() {
        assert_eq!(
            Some(value),
            result.runtime_files().get(key),
            "original art/audio {key}"
        );
    }
    assert_eq!(
        base.data().runtime_spawn_points,
        result.data().runtime_spawn_points
    );
    assert_eq!(base.data().pokemon, result.data().pokemon);
    let mut changed = vec![
        "MnRochesterPokecenter1F",
        "MnWinonaFieldStation",
        "MnMinneapolisDispatchOffice",
        "MnIceRelayFloor",
        "MnStPaulUnionDepot",
        "MnStCloud",
        "MnIronDrillCore",
        "MnBoundaryWatersSurveyShelter",
        "MnDuluthHarborWorkshop",
        "MnLeagueRoom4",
        "MnStCloudStation",
        "MnMinneapolisDispatcherHouse",
        "MnR09Cave1F",
    ];
    let source_data: serde_json::Value = serde_json::from_str(&source).expect("source");
    for guard in source_data["grunts"].as_array().expect("guard coverage") {
        let name = guard["map"].as_str().expect("guard floor");
        if let Some(original) = base.data().maps.get(name) {
            let updated = &result.data().maps[name];
            assert_eq!(
                original.attributes, updated.attributes,
                "floor attributes {name}"
            );
            assert_eq!(original.blocks, updated.blocks, "floor geometry {name}");
            assert_eq!(
                original.events, updated.events,
                "puzzle events and warps {name}"
            );
            for object in &original.objects {
                // Reed and Finn have explicit later-story visibility changes.
                if name != "MnIronDrillCore" && name != "MnIceRelayFloor" {
                    assert!(updated.objects.contains(object), "original actor {name}");
                }
            }
        }
        changed.push(name);
    }
    for (name, map) in &base.data().maps {
        if !changed.contains(&name.as_str()) {
            assert!(
                Some(map) == result.data().maps.get(name),
                "unrelated map {name}"
            );
        }
    }
    for (id, t) in &base.data().trainers.trainers {
        let mut expected = t.clone();
        expected.name = match id.as_str() {
            "MN_LEAGUE_4" => "HALDEN",
            "MN_R08_T07" => "BRAM",
            "MN_R11_T05" => "NILS",
            "MN_R16_T05" => "OTTO",
            "MN_R17_T03" => "ELI",
            _ => &t.name,
        }
        .into();
        assert_eq!(
            Some(&expected),
            result.data().trainers.trainers.get(id),
            "unrelated trainer {id}"
        );
    }
    assert!(crate::build_northstar_rival_arc(&result, &source, &layout).is_err());
    let mut invalid: serde_json::Value = serde_json::from_str(&source).expect("source");
    invalid["id"] = "unsupported".into();
    assert!(crate::build_northstar_rival_arc(&base, &invalid.to_string(), &layout).is_err());
}
