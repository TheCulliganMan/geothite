//! Exact gym-generation refresh and additive cities on the full story lineage.
use crystal_assets::{
    CompiledGamePack, RegionalGymSources, build_regional_gyms_modpack,
    read_verified_compiled_game_pack, refresh_installed_minnesota_gyms,
};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;

struct Inputs {
    base: CompiledGamePack,
    donor: CompiledGamePack,
    refreshed: CompiledGamePack,
    regional: CompiledGamePack,
    source: Vec<String>,
}
fn input() -> &'static Inputs {
    static INPUT: OnceLock<Inputs> = OnceLock::new();
    INPUT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository");
        let base = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-northstar-rochester.crystalpack"),
        )
        .expect("exact Rochester input");
        let donor = read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-gyms-remake.crystalpack"),
        )
        .expect("reviewed current gyms");
        let source = [
            "gyms/interiors.json",
            "towns/MnMankato.json",
            "towns/MnMankato.layout.json",
            "towns/MnStPaul.json",
            "towns/MnStPaul.layout.json",
            "transit/MnStPaulUnionDepot.json",
            "towns/MnStCloud.json",
            "towns/MnStCloud.layout.json",
            "transit/MnStCloudStation.json",
        ]
        .iter()
        .map(|p| {
            std::fs::read_to_string(root.join("planning/minnesota").join(p))
                .expect("current source")
        })
        .collect::<Vec<_>>();
        let refreshed =
            refresh_installed_minnesota_gyms(&base, &donor, &source[0]).expect("exact refresh");
        let regional = build_regional_gyms_modpack(
            &base,
            &donor,
            &RegionalGymSources {
                gyms: &source[0],
                mankato: [&source[1], &source[2]],
                st_paul: [&source[3], &source[4], &source[5]],
                st_cloud: [&source[6], &source[7], &source[8]],
            },
        )
        .expect("regional candidate");
        for (pack, file) in [
            (&refreshed, "minnesota-northstar-gyms-refreshed.crystalpack"),
            (&regional, "minnesota-northstar-eight-gyms.crystalpack"),
        ] {
            let written = read_verified_compiled_game_pack(root.join("content-packs").join(file))
                .expect("written candidate");
            assert_eq!(
                pack.identity().expect("built identity"),
                written.identity().expect("written identity")
            );
        }
        Inputs {
            base,
            donor,
            refreshed,
            regional,
            source,
        }
    })
}
fn replaced(id: &str) -> bool {
    [2, 4, 5, 6, 7, 8]
        .iter()
        .any(|s| id == format!("MN_GYM_{s}") || id.starts_with(&format!("MN_GYM_{s}_TRAINER_")))
}
#[test]
#[ignore = "requires completed external Rochester, reviewed gyms and regional candidates"]
fn regional_refresh_preserves_story_audio_existing_numbers_and_retained_gyms() {
    let i = input();
    let a = i.base.data();
    let b = i.refreshed.data();
    assert_eq!(a.maps.len(), b.maps.len());
    let changed_gyms = [
        "MnMankatoGym",
        "MnStPaulGym",
        "MnStCloudGym",
        "MnBrainerdGym",
        "MnBemidjiGym",
        "MnDuluthGym",
    ];
    let renamed = [
        ("MnBemidji", vec!["MN_BEMIDJI_GYMText"]),
        (
            "MnBemidjiPokecenter1F",
            vec!["MN_BEMIDJI_CLINIC_WALKERText"],
        ),
        (
            "MnBrainerd",
            vec![
                "MN_BRAINERD_GYM_SIGNText",
                "MN_BRAINERD_GYM_SPECTATORText",
                "MnBrainerdRangerBeforeText",
            ],
        ),
        (
            "MnBrainerdPokecenter1F",
            vec!["MN_BRAINERD_CLINIC_READERText"],
        ),
        ("MnDuluth", vec!["MnDuluth_GYM_SIGNText"]),
    ];
    for (name, before) in &a.maps {
        if changed_gyms.contains(&name.as_str()) {
            continue;
        }
        let mut after = b.maps[name].clone();
        if name == "MnGymReviewHall" {
            assert_eq!(after, i.donor.data().maps[name]);
            continue;
        }
        if let Some((_, labels)) = renamed.iter().find(|(map, _)| *map == name) {
            for &label in labels {
                assert_ne!(
                    after.scripts[label], before.scripts[label],
                    "name reference did not change"
                );
                after
                    .scripts
                    .insert(label.into(), before.scripts[label].clone());
                after
                    .script_text_bodies
                    .insert(label.into(), before.script_text_bodies[label].clone());
            }
        }
        assert_eq!(&after, before, "unintended story/map change: {name}");
    }
    for (id, trainer) in &a.trainers.trainers {
        if !replaced(id) {
            assert_eq!(
                b.trainers.trainers.get(id),
                Some(trainer),
                "unrelated trainer {id}"
            );
        }
    }
    assert!(
        !b.trainers.trainers.contains_key("MN_GYM_2_TRAINER_3"),
        "obsolete third practice trainer retained"
    );
    for (name, number) in &a.story_event_script_constants.global {
        if !replaced(name) {
            assert_eq!(
                b.story_event_script_constants.global.get(name),
                Some(number),
                "existing numeric identity {name}"
            );
        }
    }
    for (path, bytes) in i.base.runtime_files() {
        let owned = [2, 4, 5, 6, 7, 8].iter().any(|slot| {
            path == &format!("data/tilesets/mn_gym_{slot}.json")
                || path == &format!("data/tilesets/mn_gym_{slot}_metatiles.bin")
                || path == &format!("data/tilesets/mn_gym_{slot}_palette_map.json")
                || path == &format!("gfx/tilesets/mn_gym_{slot}.2bpp")
                || path == &format!("gfx/tilesets/mn_gym_{slot}.png")
        });
        if !owned {
            assert_eq!(
                i.refreshed.runtime_files().get(path),
                Some(bytes),
                "unrelated file {path}"
            );
        }
    }
    assert_eq!(i.base.compiled_audio(), i.refreshed.compiled_audio());
    assert_eq!(a.runtime_spawn_points, b.runtime_spawn_points);
    for (ts, rules) in &a.field_moves.cut.replacements {
        assert_eq!(b.field_moves.cut.replacements.get(ts), Some(rules));
    }
    assert_eq!(
        b.field_moves.cut.replacements["mn_gym_7"],
        i.donor.data().field_moves.cut.replacements["mn_gym_7"]
    );
    assert_eq!(a.field_moves.whirlpool, b.field_moves.whirlpool);
}
#[test]
#[ignore = "requires completed external Rochester, reviewed gyms and regional candidates"]
fn regional_candidate_has_every_current_room_roster_and_real_city_return() {
    let i = input();
    assert_eq!(i.regional.data().maps.len(), i.base.data().maps.len() + 12);
    for (slot, city, return_id) in [
        (1, "MN_ROCHESTER", 6),
        (2, "MN_MANKATO", 6),
        (3, "MN_MINNEAPOLIS", 4),
        (4, "MN_ST_PAUL", 10),
        (5, "MN_ST_CLOUD", 6),
        (6, "MN_BRAINERD", 6),
        (7, "MN_BEMIDJI", 2),
        (8, "MN_DULUTH", 4),
    ] {
        let id = format!("MN_GYM_{slot}");
        let name = i.donor.data().trainers.trainers[&id].name.clone();
        assert_eq!(i.regional.data().trainers.trainers[&id].name, name);
        let map = [
            "MnRochesterGym",
            "MnMankatoGym",
            "MnMinneapolisGym",
            "MnStPaulGym",
            "MnStCloudGym",
            "MnBrainerdGym",
            "MnBemidjiGym",
            "MnDuluthGym",
        ][slot - 1];
        let native = &i.regional.data().maps[map];
        let reviewed = &i.donor.data().maps[map];
        assert_eq!(native.blocks, reviewed.blocks);
        assert_eq!(
            [native.attributes.width, native.attributes.height],
            [reviewed.attributes.width, reviewed.attributes.height]
        );
        for (actual, source) in native
            .events
            .warps
            .iter()
            .zip(&reviewed.events.warps)
            .take(3)
        {
            assert_eq!([actual.x, actual.y], [source.x, source.y]);
            assert_eq!(actual.target_map_constant, city);
            assert_eq!(actual.target_warp_id, return_id);
        }
    }
    let before = i
        .base
        .data()
        .runtime_spawn_points
        .keys()
        .collect::<BTreeSet<_>>();
    assert!(
        before
            .iter()
            .all(|key| i.regional.data().runtime_spawn_points.get(*key)
                == i.base.data().runtime_spawn_points.get(*key))
    );
    assert_eq!(
        i.regional.data().runtime_spawn_points.len(),
        i.base.data().runtime_spawn_points.len() + 6
    );
    for map in i.regional.data().maps.values() {
        if let Some(location) = &map.attributes.location {
            assert!(
                i.regional
                    .data()
                    .pokegear_landmarks
                    .landmarks
                    .iter()
                    .any(|l| &l.constant == location && l.id <= 0x7f),
                "{} has an invalid native caught-location area",
                map.id
            );
        }
    }
    assert_eq!(
        i.regional.data().maps["MnStPaul"]
            .attributes
            .location
            .as_deref(),
        Some("LANDMARK_MN_TWIN_CITIES")
    );
    assert_eq!(
        i.regional.data().maps["MnStCloud"]
            .attributes
            .location
            .as_deref(),
        Some("LANDMARK_MN_RIVER_BLUFFS")
    );
}
#[test]
#[ignore = "requires completed external Rochester, reviewed gyms and regional candidates"]
fn regional_refresh_rejects_wrong_generation_stale_source_and_repeated_transition() {
    let i = input();
    assert!(refresh_installed_minnesota_gyms(&i.refreshed, &i.donor, &i.source[0]).is_err());
    assert!(refresh_installed_minnesota_gyms(&i.base, &i.base, &i.source[0]).is_err());
    let mut source: serde_json::Value = serde_json::from_str(&i.source[0]).expect("source");
    source["gyms"][5]["leader"] = serde_json::json!("OTHER");
    assert!(refresh_installed_minnesota_gyms(&i.base, &i.donor, &source.to_string()).is_err());
    let mut incomplete: serde_json::Value = serde_json::from_str(&i.source[0]).expect("source");
    incomplete["gyms"].as_array_mut().expect("gyms").truncate(1);
    assert!(refresh_installed_minnesota_gyms(&i.base, &i.donor, &incomplete.to_string()).is_err());
}
