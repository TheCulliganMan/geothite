//! Explicit external-pack contracts; native controller acceptance is separate.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_r10_gallery_modpack,
};
use serde_json::{Value, json};

const MAP: &str = "MnR10Cave1F";
struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    old: String,
    new: String,
}
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository")
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(||{
        let path=std::env::var_os("CRYSTAL_R10_GALLERY_REVISION_PACK").map(PathBuf::from)
            .unwrap_or_else(||root().join(".local/minnesota-city-revisions/cities-37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6.crystalpack"));
        let base=read_verified_compiled_game_pack(path.canonicalize().expect("actual completed baseline")).expect("verified native original R10 room");
        let old=if let Some(path)=std::env::var_os("CRYSTAL_R10_GALLERY_PREVIOUS_ROOM") {
            std::fs::read_to_string(path).expect("frozen original room JSON")
        }else{
            std::fs::read_to_string(root().join("planning/minnesota/caves/MnR10Cave1F.previous.json"))
                .expect("frozen standalone original gallery source")
        };
        let path=std::env::var_os("CRYSTAL_R10_GALLERY_REVISED_ROOM").map(PathBuf::from).unwrap_or_else(||root().join("planning/minnesota/caves/MnR10Cave1F.candidate.json"));
        let new=std::fs::read_to_string(path).expect("frozen reviewed gallery candidate");
        let result=revise_r10_gallery_modpack(&base,&old,&new).expect("bounded original gallery revision");Inputs{base,result,old,new}
    })
}
fn changed_row(source: &str, at: [usize; 2], block: &str) -> String {
    let mut v: Value = serde_json::from_str(source).expect("room source");
    let row = v["rows"][at[1]].as_str().expect("row");
    let mut cells = row
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    cells[at[0]] = block.into();
    v["rows"][at[1]] = json!(cells.join(" "));
    v.to_string()
}

#[test]
#[ignore = "requires completed external original R10 pack and frozen before/after room sources"]
fn r10_gallery_revision_preserves_every_unrelated_field_and_original_program() {
    let i = inputs();
    let b = i.base.data();
    let a = i.result.data();
    assert_eq!(b.maps.len(), a.maps.len());
    for (name, module) in &b.maps {
        if name != MAP {
            assert_eq!(&a.maps[name], module, "unrelated map changed: {name}");
        }
    }
    let old = &b.maps[MAP];
    let new = &a.maps[MAP];
    assert_eq!(new.attributes.width, 18);
    assert_eq!(new.attributes.height, 14);
    assert_eq!(new.events, old.events);
    assert_eq!(
        new.objects[0], old.objects[0],
        "whole original resident record"
    );
    let mut pickup = new.objects[1].clone();
    assert_eq!([pickup.x, pickup.y], [29, 23]);
    pickup.x = 17;
    pickup.y = 5;
    assert_eq!(pickup, old.objects[1]);
    for (label, program) in &old.scripts {
        if Some(label) != old.attributes.map_events_label.as_ref() {
            assert_eq!(
                new.scripts.get(label),
                Some(program),
                "old program changed: {label}"
            );
        }
    }
    assert_eq!(
        new.script_field_pickups, old.script_field_pickups,
        "original Revive service/item/flag program"
    );
    let mut normalized = a.clone();
    normalized.maps.insert(MAP.into(), old.clone());
    normalized
        .map_attributes
        .insert(MAP.into(), b.map_attributes[MAP].clone());
    normalized
        .runtime_map_metadata
        .insert(MAP.into(), b.runtime_map_metadata[MAP].clone());
    normalized
        .map_dimensions
        .insert(MAP.into(), b.map_dimensions[MAP].clone());
    let blocks = old.attributes.blocks_label.as_ref().expect("blocks");
    normalized
        .map_blocks
        .insert(blocks.clone(), b.map_blocks[blocks].clone());
    let events = old.attributes.map_events_label.as_ref().expect("events");
    normalized
        .map_scripts
        .insert(events.clone(), b.map_scripts[events].clone());
    normalized.npcs.insert(MAP.into(), b.npcs[MAP].clone());
    assert_eq!(
        &normalized, b,
        "only named room terrain/metadata and pickup coordinate mirrors may change"
    );
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.result.audio_manifest().expect("after audio"),
        i.base.audio_manifest().expect("before audio")
    );
    let mut report = i.result.report().clone();
    assert_eq!(
        report.manifests.pop().as_deref(),
        Some("minnesota-r10-seam-gallery-v1")
    );
    assert_eq!(&report, i.base.report());
}

#[test]
#[ignore = "requires completed external original R10 pack and frozen before/after room sources"]
fn r10_gallery_revision_true_uncached_assembly_and_actual_written_pack_roundtrip() {
    let i = inputs();
    let mut data = i.result.data().clone();
    let m = data.maps.remove(MAP).expect("new gallery");
    assert_eq!(
        data.assemble_map_module_from_compiled_payloads(MAP)
            .expect("full production uncached assembly"),
        m
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r10-gallery-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    i.result
        .write_preserving_storage(&path)
        .expect("actual native pack write");
    let reloaded =
        read_verified_compiled_game_pack(&path).expect("strict written pack verification");
    assert_eq!(reloaded, i.result);
    std::fs::remove_file(path).expect("remove scratch native pack, never render history");
    assert!(
        revise_r10_gallery_modpack(&i.result, &i.old, &i.new).is_err(),
        "deny repeated application"
    );
}

#[test]
#[ignore = "requires completed external original R10 pack and frozen before/after room sources"]
fn r10_gallery_revision_rejects_stale_preimage_and_original_actor_reward_exit_drift() {
    let i = inputs();
    let wrong = changed_row(&i.old, [0, 0], "02");
    assert!(
        revise_r10_gallery_modpack(&i.base, &wrong, &i.new).is_err(),
        "reject false installed old terrain"
    );
    for mutation in 0..6 {
        let mut v: Value = serde_json::from_str(&i.new).expect("candidate");
        match mutation {
            0 => v["residents"][0]["lines"][0] = json!("A different story"),
            1 => v["residents"][0]["tile"] = json!([10, 5]),
            2 => v["items"][0]["item"] = json!("MASTER_BALL"),
            3 => v["items"][0]["flag"] = json!("EVENT_MN_GYM_6_DEFEATED"),
            4 => v["exits"][0] = json!([4, 13]),
            _ => v["items"][0]["quantity"] = json!(99),
        }
        assert!(
            revise_r10_gallery_modpack(&i.base, &i.old, &v.to_string()).is_err(),
            "accepted actor/story/reward/warp mutation {mutation}"
        );
    }
}

#[test]
#[ignore = "requires completed external original R10 pack and frozen before/after room sources"]
fn r10_gallery_revision_rejects_unpaired_ladders_disconnected_stairs_and_wrong_hop_landing() {
    let i = inputs();
    for (at, block, reason) in [
        ([2, 5], "1b", "unpaired ladder"),
        ([3, 4], "02", "painted floor stair"),
        ([6, 4], "01", "blocked hop landing"),
        ([0, 0], "0f", "open outer water"),
    ] {
        let changed = changed_row(&i.new, at, block);
        assert!(
            revise_r10_gallery_modpack(&i.base, &i.old, &changed).is_err(),
            "accepted {reason}"
        );
    }
    let changed = changed_row(&i.new, [3, 3], "20");
    assert!(
        revise_r10_gallery_modpack(&i.base, &i.old, &changed).is_err(),
        "reject previously observed disconnected first-stair upper lane"
    );
}

#[test]
#[ignore = "requires completed external original R10 pack and frozen before/after room sources"]
fn r10_gallery_revision_preserves_both_original_exterior_door_records_and_numeric_identity() {
    let i = inputs();
    let b = i.base.data();
    let a = i.result.data();
    assert_eq!(a.maps["MnR10C00R00"], b.maps["MnR10C00R00"]);
    let room = &a.maps[MAP];
    assert_eq!(room.events.warps.len(), 1);
    let exit = &room.events.warps[0];
    assert_eq!([exit.x, exit.y], [3, 13]);
    assert_eq!(exit.target_map, "MnR10C00R00");
    assert_eq!(exit.target_warp_id, 2);
    let meta = &a.runtime_map_metadata[MAP];
    assert_eq!(
        [meta.group_id, meta.map_id, meta.width, meta.height],
        [55, 3, 18, 14]
    );
    assert_eq!(a.runtime_spawn_points, b.runtime_spawn_points);
    assert_eq!(
        a.story_event_script_constants,
        b.story_event_script_constants
    );
    let def = &a.tilesets["cave"];
    let kind = |x: usize, y: usize| {
        let block = room.blocks[y / 2 * 18 + x / 2];
        def.collision[&format!("{block:02x}")][y % 2 * 2 + x % 2].as_str()
    };
    assert_eq!(kind(3, 12), "FLOOR");
    assert_eq!(kind(3, 13), "LADDER");
    assert_eq!(kind(12, 6), "HOP_DOWN");
    assert_eq!(kind(12, 7), "WALL");
    assert_eq!(kind(12, 8), "FLOOR");
}
