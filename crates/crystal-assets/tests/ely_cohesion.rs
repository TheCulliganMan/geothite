//! Additive Ely acceptance on the completed connected-state native parent.
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    CompiledGamePack, build_ely_league_modpack, read_verified_compiled_game_pack,
};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde_json::Value;

struct Fixture {
    base: CompiledGamePack,
    town: String,
    layout: String,
    story: String,
}
fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let path = std::env::var_os("CRYSTAL_ELY_COHESION_BASE")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("content-packs/minnesota-state-cohesion-v01.crystalpack"));
        Fixture {
            base: read_verified_compiled_game_pack(
                path.canonicalize().expect("actual connected-state parent"),
            )
            .expect("strict verified parent"),
            town: std::fs::read_to_string(root.join("planning/minnesota/towns/MnEly.json"))
                .expect("complete frozen Ely source"),
            layout: std::fs::read_to_string(
                root.join("planning/minnesota/towns/MnEly.layout.json"),
            )
            .expect("frozen20x18 Ely layout"),
            story: std::fs::read_to_string(root.join("planning/minnesota/story.json"))
                .expect("five-member native League source"),
        }
    })
}
fn revised() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        let f = fixture();
        build_ely_league_modpack(&f.base, &f.town, &f.layout, &f.story)
            .expect("bounded additive Ely rebase")
    })
}
fn kind(pack: &CompiledGamePack, name: &str, p: [u16; 2]) -> String {
    let m = &pack.data().maps[name];
    let block =
        m.blocks[usize::from(p[1] / 2) * usize::from(m.attributes.width) + usize::from(p[0] / 2)];
    pack.data().tilesets[&m.attributes.tileset_name].collision[&format!("{block:02x}")]
        [usize::from(p[1] % 2 * 2 + p[0] % 2)]
    .clone()
}
fn new_names(pack: &CompiledGamePack) -> Vec<String> {
    let mut rows = pack
        .data()
        .runtime_map_metadata
        .values()
        .filter(|m| m.group_id == 81)
        .map(|m| (m.map_id, m.name.clone()))
        .collect::<Vec<_>>();
    rows.sort();
    assert_eq!(
        rows.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        (1..=10).collect::<Vec<_>>()
    );
    rows.into_iter().map(|(_, name)| name).collect()
}
fn point(value: &Value) -> [u16; 2] {
    [
        u16::try_from(value[0].as_u64().expect("x")).expect("u16 x"),
        u16::try_from(value[1].as_u64().expect("y")).expect("u16 y"),
    ]
}

#[test]
#[ignore = "requires external connected-state parent and frozen complete Ely sources"]
fn ely_cohesion_preserves_all681_old_modules_raw_programs_catalogs_audio_and_recovery() {
    let base = &fixture().base;
    let pack = revised();
    assert_eq!(base.data().maps.len(), 681);
    assert_eq!(pack.data().maps.len(), 691);
    for (name, old) in &base.data().maps {
        let mut expected = old.clone();
        if name == "MnR13C00R02" {
            expected
                .attributes
                .connections
                .push(crystal_core::map::MapConnection {
                    direction: "south".into(),
                    target_map: "MnEly".into(),
                    offset: 5,
                });
            expected.attributes.connection_flags = Some("NORTH | SOUTH".into());
        }
        assert_eq!(
            pack.data().maps[name],
            expected,
            "entire original module retained: {name}"
        );
        assert_eq!(
            pack.data().npcs.get(name),
            base.data().npcs.get(name),
            "original raw NPC mirror: {name}"
        );
    }
    for (label, body) in &base.data().map_scripts {
        assert_eq!(
            pack.data().map_scripts.get(label),
            Some(body),
            "every original global raw body exact: {label}"
        );
    }
    for (name, value) in &base.data().story_event_script_constants.global {
        assert_eq!(
            pack.data().story_event_script_constants.global.get(name),
            Some(value),
            "old numeric identity exact: {name}"
        );
    }
    for (name, row) in &base.data().runtime_spawn_points {
        assert_eq!(
            pack.data().runtime_spawn_points.get(name),
            Some(row),
            "old recovery exact: {name}"
        );
    }
    for (name, row) in &base.data().trainers.trainers {
        assert_eq!(
            pack.data().trainers.trainers.get(name),
            Some(row),
            "old trainer exact: {name}"
        );
    }
    assert_eq!(pack.data().pokemon, base.data().pokemon);
    assert_eq!(pack.data().moves, base.data().moves);
    assert_eq!(pack.data().items, base.data().items);
    assert_eq!(pack.data().wild_encounters, base.data().wild_encounters);
    assert_eq!(pack.data().tilesets, base.data().tilesets);
    assert_eq!(
        pack.data().pokegear_landmarks.landmarks,
        base.data().pokegear_landmarks.landmarks
    );
    assert_eq!(pack.runtime_files(), base.runtime_files());
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    assert_eq!(pack.data().audio, base.data().audio);
}

#[test]
#[ignore = "requires actual connected-state parent; native resolver and facade contracts"]
fn ely_cohesion_native_two_lane_seam_and_all_original_room_floorplans_and_returns() {
    let pack = revised();
    for (name, direction, target, from, to) in [
        (
            "MnR13C00R02",
            "south",
            "MnEly",
            [[34, 31], [35, 31]],
            [[24, 0], [25, 0]],
        ),
        (
            "MnEly",
            "north",
            "MnR13C00R02",
            [[24, 0], [25, 0]],
            [[34, 31], [35, 31]],
        ),
    ] {
        let m = &pack.data().maps[name];
        let native = OverworldMapData::from_attributes(name, &m.attributes, m.blocks.clone());
        for (a, b) in from.into_iter().zip(to) {
            assert_eq!(kind(pack, name, a), "FLOOR");
            assert_eq!(kind(pack, target, b), "FLOOR");
            let next = TilePosition::new(
                i16::try_from(a[0]).expect("x"),
                i16::try_from(a[1]).expect("y") + if direction == "north" { -1 } else { 1 },
            );
            let transition = connection_for_tile_checked(&native, next)
                .expect("native projection")
                .expect("actual native connection");
            let actual = pack
                .data()
                .resolve_connection_transition(&transition)
                .expect("native destination")
                .destination;
            assert_eq!(actual.map_name, target);
            assert_eq!(
                actual.tile,
                TilePosition::new(
                    i16::try_from(b[0]).expect("x"),
                    i16::try_from(b[1]).expect("y")
                )
            );
        }
    }
    assert_eq!(
        [
            pack.data().maps["MnEly"].attributes.width,
            pack.data().maps["MnEly"].attributes.height
        ],
        [20, 18]
    );
    let town: Value = serde_json::from_str(&fixture().town).expect("frozen original doors");
    for door in town["doors"].as_array().expect("four facade lanes") {
        let target = door["target"].as_str().expect("room");
        let exterior = &pack.data().maps["MnEly"];
        let active = &exterior.events.warps[usize::try_from(
            door["warp_index"].as_u64().expect("active index") - 1,
        )
        .expect("index")];
        assert_eq!([active.x, active.y], point(&door["tile"]));
        assert_eq!(kind(pack, "MnEly", [active.x, active.y]), "DOOR");
        let passive = &exterior.events.warps[usize::try_from(
            door["return_warp_index"].as_u64().expect("return index") - 1,
        )
        .expect("index")];
        assert_eq!([passive.x, passive.y], point(&door["return_tile"]));
        assert_eq!(kind(pack, "MnEly", [passive.x, passive.y]), "FLOOR");
        let room = &pack.data().maps[target];
        let arrival = &room.events.warps[usize::try_from(
            door["target_warp"].as_u64().expect("room arrival index") - 1,
        )
        .expect("index")];
        assert_eq!([arrival.x, arrival.y], point(&door["arrival_tile"]));
        assert_eq!(kind(pack, target, [arrival.x, arrival.y]), "FLOOR");
    }
    for (name, template) in [
        ("MnElyPokecenter1F", "GoldenrodPokecenter1F"),
        ("MnElyMart", "CherrygroveMart"),
        ("MnLeagueLobby", "IlexForestAzaleaGate"),
        ("MnLeagueRoom1", "KogasRoom"),
        ("MnLeagueRoom2", "BrunosRoom"),
        ("MnLeagueRoom3", "WillsRoom"),
        ("MnLeagueRoom4", "KarensRoom"),
        ("MnLeagueChampion", "LancesRoom"),
        ("MnLeagueHallOfFame", "HallOfFame"),
    ] {
        let mut blocks = fixture().base.data().maps[template].blocks.clone();
        if name == "MnElyPokecenter1F" {
            blocks[15] = 4;
        }
        assert_eq!(
            pack.data().maps[name].blocks,
            blocks,
            "full native room geometry: {name}"
        );
    }
}

#[test]
#[ignore = "requires connected-state parent; fresh identities and actual native source parties"]
fn ely_cohesion_allocates_fresh_flags_recovery_landmarks_and_all_five_native_parties() {
    let base = fixture().base.data();
    let pack = revised();
    let d = pack.data();
    assert_eq!(
        d.story_event_script_constants.global["EVENT_MN_LEAGUE_RUN_ACTIVE"],
        2382
    );
    assert_eq!(
        d.story_event_script_constants.global["EVENT_MN_LEAGUE_CHAMPION"],
        2383
    );
    assert_eq!(
        d.story_event_script_constants.global["EVENT_MN_MP_CLUB_CEDAR_BEATEN"],
        2310
    );
    assert_eq!(
        d.story_event_script_constants.global["EVENT_MN_MP_CLUB_COACH_INEZ_BEATEN"],
        2311
    );
    assert_eq!(d.runtime_spawn_points["278"].map_name, "MnEly");
    assert_eq!(d.runtime_spawn_points["279"].map_name, "MnElyPokecenter1F");
    assert_eq!(
        d.story_event_script_constants.maps["MnLeagueHallOfFame"]["GEOTHITE_POST_CREDITS_SPAWN"],
        279
    );
    assert_eq!(
        d.runtime_spawn_points.len(),
        base.runtime_spawn_points.len() + 2
    );
    for name in new_names(pack) {
        let landmark = d
            .pokegear_landmark_for_map(&name)
            .expect("actual registered geography");
        assert_eq!(
            (landmark.constant.as_str(), landmark.id),
            ("LANDMARK_MN_LEAGUE_APPROACH", 100)
        );
        assert_eq!(
            d.maps[&name].attributes.location.as_deref(),
            Some("LANDMARK_MN_LEAGUE_APPROACH")
        );
    }
    let story: Value = serde_json::from_str(&fixture().story).expect("source League");
    for member in story["league"].as_array().expect("five members") {
        let trainer = &d.trainers.trainers[member["id"].as_str().expect("source trainer")];
        assert_eq!(trainer.name, member["name"].as_str().expect("source name"));
        let party = member["party"].as_array().expect("source party");
        assert_eq!(trainer.party.len(), party.len());
        for (actual, expected) in trainer.party.iter().zip(party) {
            assert_eq!(
                actual.species,
                expected["species"].as_str().expect("native species")
            );
            assert_eq!(
                u64::from(actual.level),
                expected["level"].as_u64().expect("source level")
            );
            assert!((1..=251).contains(&base.pokemon[&actual.species].int_id));
            let native = base
                .create_pokemon(&actual.species, actual.level, actual.dvs)
                .expect("source-native legal moves");
            assert_eq!(actual.moves, native.moves);
        }
    }
    assert_eq!(d.trainers.trainers.len(), base.trainers.trainers.len() + 5);
    assert_eq!(
        d.marts.0["MART_MN_ELY"],
        [
            "ULTRA_BALL",
            "HYPER_POTION",
            "MAX_POTION",
            "FULL_HEAL",
            "REVIVE",
            "MAX_REPEL"
        ]
        .map(str::to_owned)
    );
}

#[test]
#[ignore = "requires external connected-state parent; removes each new cache and writes an owned temporary roundtrip"]
fn ely_cohesion_all10_uncached_native_modules_and_written_roundtrip_are_exact() {
    let pack = revised();
    let mut data = pack.data().clone();
    for name in new_names(pack) {
        let expected = data.maps.remove(&name).expect("new cached module");
        let mut actual = data
            .assemble_map_module_from_compiled_payloads(&name)
            .expect("real native uncached parser");
        for label in [
            &expected.attributes.map_scripts_label,
            &expected.attributes.map_events_label,
        ] {
            actual
                .scripts
                .remove(label.as_ref().expect("transport label"));
        }
        assert_eq!(actual, expected, "all typed/raw native fields: {name}");
        data.maps.insert(name, expected);
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "ely-cohesion-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    assert!(!path.exists());
    pack.write_preserving_storage(&path)
        .expect("owned temporary strict write");
    let restored =
        read_verified_compiled_game_pack(path.canonicalize().expect("written actual path"))
            .expect("strict native readback");
    assert_eq!(
        restored.identity().expect("written identity"),
        pack.identity().expect("expected identity")
    );
    assert_eq!(
        serde_json::to_value(restored.data()).expect("written full data"),
        serde_json::to_value(pack.data()).expect("expected full data")
    );
    std::fs::remove_file(path).expect("remove only successful own test pack");
}

#[test]
#[ignore = "requires actual native parent; reviewed source cannot silently shrink or change League requirements"]
fn ely_cohesion_rejects_reinstallation_changed_extent_missing_member_and_invented_species() {
    let f = fixture();
    assert!(build_ely_league_modpack(revised(), &f.town, &f.layout, &f.story).is_err());
    let mut bad: Value = serde_json::from_str(&f.town).expect("town");
    bad["dimensions_metatiles"] = serde_json::json!([18, 18]);
    assert!(build_ely_league_modpack(&f.base, &bad.to_string(), &f.layout, &f.story).is_err());
    let mut bad: Value = serde_json::from_str(&f.layout).expect("layout");
    bad["dimensions_metatiles"] = serde_json::json!([20, 16]);
    assert!(build_ely_league_modpack(&f.base, &f.town, &bad.to_string(), &f.story).is_err());
    let mut bad: Value = serde_json::from_str(&f.story).expect("story");
    bad["league"].as_array_mut().expect("members").pop();
    assert!(build_ely_league_modpack(&f.base, &f.town, &f.layout, &bad.to_string()).is_err());
    let mut bad: Value = serde_json::from_str(&f.story).expect("story");
    bad["league"][0]["party"][0]["species"] = "MN_INVENTED".into();
    assert!(build_ely_league_modpack(&f.base, &f.town, &f.layout, &bad.to_string()).is_err());
    assert_eq!(
        new_names(revised())
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        10
    );
}
