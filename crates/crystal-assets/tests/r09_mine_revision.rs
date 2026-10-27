//! Actual Q06 mine: terrain-only cache delta and precisely certified split repair.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{CompiledGamePack, read_verified_compiled_game_pack, revise_r09_mine_terrain};
use serde_json::{Value, json};

const MAP: &str = "MnR09Cave1F";
const EVENTS: &str = "MnR09Cave1F_MapEvents";
struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    old: String,
    new: String,
    story: String,
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo")
}
fn source(env: &str, default: &str) -> String {
    std::fs::read_to_string(
        std::env::var_os(env)
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join(default)),
    )
    .expect("explicit frozen source")
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let path=std::env::var_os("CRYSTAL_R09_MINE_REVISION_PACK").map(PathBuf::from).unwrap_or_else(||root().join("content-packs/minnesota-northstar-glacial-cleft-review.crystalpack"));
        let base=read_verified_compiled_game_pack(path).expect("actual completed Q06 preimage");
        let old=source("CRYSTAL_R09_MINE_PREVIOUS_ROOM","planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.previous.json");
        let new=source("CRYSTAL_R09_MINE_REVISED_ROOM","planning/minnesota/renders/caves/r09-candidate/243f04da49a01f59/MnR09Cave1F.candidate.json");
        let story=source("CRYSTAL_R09_MINE_STORY","planning/minnesota/story.json");
        let result=revise_r09_mine_terrain(&base,&old,&new,&story).expect("bounded original Q06 mine revision");
        Inputs{base,result,old,new,story}
    })
}
fn change_row(text: &str, at: [usize; 2], block: &str) -> String {
    let mut v: Value = serde_json::from_str(text).expect("room");
    let mut row = v["rows"][at[1]]
        .as_str()
        .expect("row")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    row[at[0]] = block.into();
    v["rows"][at[1]] = json!(row.join(" "));
    v.to_string()
}

#[test]
#[ignore = "requires completed external Q06 native pack and frozen R09/story inputs"]
fn r09_mine_revision_preserves_all_other_data_and_exact_native_story() {
    let i = inputs();
    let b = i.base.data();
    let a = i.result.data();
    let mut expected = b.maps[MAP].clone();
    expected.blocks = a.maps[MAP].blocks.clone();
    assert_eq!(
        a.maps[MAP], expected,
        "no cached actor/program/index/party rewrite"
    );
    let added = a
        .map_scripts
        .keys()
        .filter(|k| !b.map_scripts.contains_key(*k))
        .collect::<Vec<_>>();
    assert_eq!(added.len(), 19);
    for label in &added {
        assert_eq!(a.map_scripts.get(*label), b.maps[MAP].scripts.get(*label));
    }
    for (label, body) in &b.map_scripts {
        if label != EVENTS {
            assert_eq!(a.map_scripts.get(label), Some(body));
        }
    }
    assert_eq!(
        a.map_scripts[EVENTS].as_array().expect("new events").len(),
        9
    );
    assert_eq!(
        b.map_scripts[EVENTS]
            .as_array()
            .expect("stale events")
            .len(),
        7
    );
    let mut normalized = a.clone();
    normalized.maps.insert(MAP.into(), b.maps[MAP].clone());
    let label = b.maps[MAP]
        .attributes
        .blocks_label
        .as_ref()
        .expect("owned blocks");
    normalized
        .map_blocks
        .insert(label.clone(), b.map_blocks[label].clone());
    normalized.npcs.insert(MAP.into(), b.npcs[MAP].clone());
    normalized
        .map_scripts
        .insert(EVENTS.into(), b.map_scripts[EVENTS].clone());
    for label in added {
        normalized.map_scripts.remove(label);
    }
    assert_eq!(
        &normalized, b,
        "all other data exact, including whole trainer/flags/HOME/recovery"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.report(), i.base.report());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.result.audio_manifest().expect("new audio"),
        i.base.audio_manifest().expect("old audio")
    );
}

#[test]
#[ignore = "requires completed external Q06 native pack and frozen R09/story inputs"]
fn r09_mine_revision_true_uncached_and_actual_written_roundtrip_are_complete() {
    let i = inputs();
    let mut d = i.result.data().clone();
    d.maps.remove(MAP).expect("remove real cache");
    let actual = d
        .assemble_map_module_from_compiled_payloads(MAP)
        .expect("complete actual Q06 assembly");
    let mut expected = i.result.data().maps[MAP].clone();
    for label in ["MnR09Cave1F_MapScripts", EVENTS] {
        assert!(
            expected
                .scripts
                .insert(label.into(), d.map_scripts[label].clone())
                .is_none()
        );
    }
    assert_eq!(
        actual, expected,
        "only exactly two raw section labels augment uncached native module"
    );
    assert_eq!(actual.scripted_trainer_battles.len(), 1);
    assert_eq!(actual.objects.len(), 3);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r09-mine-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    i.result
        .write_preserving_storage(&path)
        .expect("actual Rust write");
    assert_eq!(
        read_verified_compiled_game_pack(&path).expect("strict written read"),
        i.result
    );
    std::fs::remove_file(path).expect("scratch cleanup");
    assert!(
        revise_r09_mine_terrain(&i.result, &i.old, &i.new, &i.story).is_err(),
        "old terrain/stale split cannot silently reapply"
    );
}

#[test]
#[ignore = "requires completed external Q06 native pack and frozen R09/story inputs"]
fn r09_mine_revision_rejects_changed_and_matching_false_story_sources() {
    let i = inputs();
    for n in 0..10 {
        let mutate = |v: &mut Value| match n {
            0 => v["story_trainers"][0]["party"][0]["level"] = json!(34),
            1 => v["story_trainers"][0]["defeated_flag"] = json!("EVENT_MN_DEPOT_CLEARED"),
            2 => v["story_trainers"][0]["before_lines"][0] = json!("Changed report"),
            3 => v["story_trainers"][0]["tile"] = json!([8, 5]),
            4 => v["story_objects"][0]["recovered_flag"] = json!("EVENT_MN_DEPOT_CLEARED"),
            5 => v["story_objects"][0]["payload_storage"] = json!("bag_item"),
            6 => v["story_objects"][0]["requires"] = json!([]),
            7 => v["residents"][0]["lines"][0] = json!("An invented scout"),
            8 => v["exits"][0] = json!([12, 19]),
            _ => v["story_objects"][0]["success_lines"][0] = json!("An invented meter"),
        };
        let mut new: Value = serde_json::from_str(&i.new).expect("new");
        mutate(&mut new);
        assert!(
            revise_r09_mine_terrain(&i.base, &i.old, &new.to_string(), &i.story).is_err(),
            "new source drift{n}"
        );
        let mut old: Value = serde_json::from_str(&i.old).expect("old");
        mutate(&mut old);
        assert!(
            revise_r09_mine_terrain(&i.base, &old.to_string(), &new.to_string(), &i.story).is_err(),
            "matching false sources{n}"
        );
    }
    let mut story: Value = serde_json::from_str(&i.story).expect("story");
    let milestone = story["milestones"]
        .as_array_mut()
        .expect("milestones")
        .iter_mut()
        .find(|m| m["id"] == "MS08")
        .expect("foreman milestone");
    milestone["requires"] = json!(["BADGE_6", "EVENT_MN_HEADWATERS_REPORT"]);
    assert!(revise_r09_mine_terrain(&i.base, &i.old, &i.new, &story.to_string()).is_err());
}

#[test]
#[ignore = "requires completed external Q06 native pack and frozen R09/story inputs"]
fn r09_mine_revision_rejects_unsafe_actor_stair_hop_mouth_and_boundary_geometry() {
    let i = inputs();
    for (at, block, reason) in [
        ([4, 3], "01", "foreman approach/stair"),
        ([9, 3], "01", "scout standing floor"),
        ([5, 7], "01", "mouth stair"),
        ([5, 5], "01", "both hop landings"),
        ([6, 9], "01", "original mouth"),
        ([0, 0], "02", "open outer floor"),
        ([0, 0], "0f", "open outer water"),
        ([2, 2], "1b", "unpaired ladder"),
    ] {
        assert!(
            revise_r09_mine_terrain(&i.base, &i.old, &change_row(&i.new, at, block), &i.story)
                .is_err(),
            "accepted unsafe{reason}"
        );
    }
}

#[cfg(feature = "test-fixtures")]
fn native_fixture(data: crystal_assets::GameDataSet) -> CompiledGamePack {
    let i = inputs();
    let mut raw = serde_json::to_value(&i.base).expect("whole audio/storage fixture");
    raw["data"] = serde_json::to_value(data).expect("typed data");
    let pack: CompiledGamePack = serde_json::from_value(raw).expect("native storage retained");
    pack.with_runtime_files_for_tests(i.base.runtime_files().clone())
}
#[cfg(feature = "test-fixtures")]
#[test]
#[ignore = "requires external Q06 pack, frozen sources and test-fixtures valid identity controls"]
fn r09_mine_revision_rejects_typed_party_flag_and_noncanonical_stale_split_drift() {
    let i = inputs();
    let control = native_fixture(i.base.data().clone());
    assert_eq!(control, i.base);
    crystal_assets::verify_compiled_game_pack_for_runtime(&control)
        .expect("unchanged native/audio control valid");
    revise_r09_mine_terrain(&control, &i.old, &i.new, &i.story).expect("valid control accepted");
    for n in 0..10 {
        let mut d = i.base.data().clone();
        match n {
            0 => {
                d.maps.get_mut(MAP).expect("map").scripted_trainer_battles[0]
                    .startbattle_command_index = 12
            }
            1 => {
                d.maps.get_mut(MAP).expect("map").script_flag_commands[3].flag_id =
                    "EVENT_MN_DEPOT_CLEARED".into()
            }
            2 => {
                d.trainers
                    .trainers
                    .get_mut("MN_LODE_FOREMAN")
                    .expect("trainer")
                    .party[0]
                    .level = 34
            }
            3 => {
                d.trainers
                    .trainers
                    .get_mut("MN_LODE_FOREMAN")
                    .expect("trainer")
                    .base_reward += 1
            }
            4 => {
                d.story_event_script_constants
                    .global
                    .insert("EVENT_MN_MINE_INSTRUMENT_RECOVERED".into(), 2500);
            }
            5 => d.npcs.get_mut(MAP).expect("NPC mirror")[0]["x"] = json!(18),
            6 => d.map_scripts.get_mut(EVENTS).expect("events")[6]["args"][0] = json!("18"),
            7 => {
                d.map_scripts.insert(
                    "MN_R09_SURVEY_CACHE".into(),
                    d.maps[MAP].scripts["MN_R09_SURVEY_CACHE"].clone(),
                );
            }
            8 => {
                d.maps
                    .get_mut(MAP)
                    .expect("map")
                    .scripts
                    .get_mut("MN_R09_SURVEY_CACHE")
                    .expect("cache")[11]["args"][0] = json!("EVENT_MN_DEPOT_CLEARED")
            }
            _ => {
                d.map_scripts
                    .get_mut("MN_R09_MINE_SCOUTText")
                    .expect("scout")[0]["args"][0] = json!("\"Invented scout\"")
            }
        }
        let corrupt = native_fixture(d);
        corrupt
            .identity()
            .expect("valid identity, not identity-only rejection");
        assert!(
            revise_r09_mine_terrain(&corrupt, &i.old, &i.new, &i.story).is_err(),
            "accepted native drift{n}"
        );
    }
}
