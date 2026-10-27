//! Same-extent R12 terrain revision against explicit frozen native inputs.
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_authored_cave_terrain,
};
use serde_json::{Value, json};

const MAP: &str = "MnR12Cave1F";
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
fn source(variable: &str, fallback: &str) -> String {
    std::fs::read_to_string(
        std::env::var_os(variable)
            .map(PathBuf::from)
            .unwrap_or_else(|| root().join(fallback)),
    )
    .expect("frozen R12 room source")
}
fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let path=std::env::var_os("CRYSTAL_R12_SHELTER_REVISION_PACK").map(PathBuf::from).unwrap_or_else(||root().join(".local/minnesota-city-revisions/gallery-e0cf8cf00b18c1b25c7cab70e37ebbcc93f1ff25d67fb7aa7f4c4371d567085e.crystalpack"));
        let base=read_verified_compiled_game_pack(path.canonicalize().expect("completed native baseline")).expect("verified original shelter");
        let old=source("CRYSTAL_R12_SHELTER_PREVIOUS_ROOM","planning/minnesota/renders/caves/r12-candidate/v04/MnR12Cave1F.previous.json");
        let new=source("CRYSTAL_R12_SHELTER_REVISED_ROOM","planning/minnesota/renders/caves/r12-candidate/v04/MnR12Cave1F.candidate.json");
        let result=revise_authored_cave_terrain(&base,&old,&new).expect("bounded same-extent cave revision");
        Inputs{base,result,old,new}
    })
}
fn row_change(source: &str, at: [usize; 2], block: &str) -> String {
    let mut v: Value = serde_json::from_str(source).expect("room source");
    let mut row = v["rows"][at[1]]
        .as_str()
        .expect("native row")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    row[at[0]] = block.into();
    v["rows"][at[1]] = json!(row.join(" "));
    v.to_string()
}

#[test]
#[ignore = "requires completed external original R12 pack and frozen before/after cave sources"]
fn r12_shelter_revision_changes_only_room_blocks_and_owned_payload() {
    let i = inputs();
    let b = i.base.data();
    let a = i.result.data();
    assert_ne!(a.maps[MAP].blocks, b.maps[MAP].blocks);
    let mut normalized = a.clone();
    normalized.maps.get_mut(MAP).expect("same room").blocks = b.maps[MAP].blocks.clone();
    let label = b.maps[MAP]
        .attributes
        .blocks_label
        .as_ref()
        .expect("owned block label");
    normalized
        .map_blocks
        .insert(label.clone(), b.map_blocks[label].clone());
    assert_eq!(
        &normalized, b,
        "all other compiled data exact, including typed and raw script indexes"
    );
    assert_eq!(i.result.report(), i.base.report());
    assert_eq!(i.result.runtime_files(), i.base.runtime_files());
    assert_eq!(i.result.compiled_audio(), i.base.compiled_audio());
    assert_eq!(
        i.result.audio_manifest().expect("after audio"),
        i.base.audio_manifest().expect("before audio")
    );
    let room = &a.maps[MAP];
    assert_eq!([room.attributes.width, room.attributes.height], [16, 12]);
    assert_eq!(room.attributes.time_of_day.as_deref(), Some("nite"));
    assert_eq!(room.attributes.palette.as_deref(), Some("PALETTE_NITE"));
    assert_eq!(room.objects, b.maps[MAP].objects);
    assert_eq!(room.events, b.maps[MAP].events);
    assert_eq!(room.script_field_pickups, b.maps[MAP].script_field_pickups);
}

#[test]
#[ignore = "requires completed external original R12 pack and frozen before/after cave sources"]
fn r12_shelter_revision_uncached_split_and_actual_written_roundtrip_are_exact() {
    let i = inputs();
    let mut old = i.base.data().clone();
    old.maps.remove(MAP).expect("old cached room");
    let mut expected = old
        .assemble_map_module_from_compiled_payloads(MAP)
        .expect("old full production split assembly");
    expected.blocks = i.result.data().maps[MAP].blocks.clone();
    let mut revised = i.result.data().clone();
    revised.maps.remove(MAP).expect("new cached room");
    assert_eq!(
        revised
            .assemble_map_module_from_compiled_payloads(MAP)
            .expect("new full production split assembly"),
        expected,
        "no script/transport exceptions"
    );
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "geothite-r12-shelter-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    i.result
        .write_preserving_storage(&path)
        .expect("actual external pack write");
    assert_eq!(
        read_verified_compiled_game_pack(&path).expect("strict actual readback"),
        i.result
    );
    std::fs::remove_file(path).expect("remove scratch native pack");
    assert!(
        revise_authored_cave_terrain(&i.result, &i.old, &i.new).is_err(),
        "reject stale repeated preimage"
    );
}

#[test]
#[ignore = "requires completed external original R12 pack and frozen before/after cave sources"]
fn r12_shelter_revision_rejects_actor_program_reward_warp_palette_and_extent_source_drift() {
    let i = inputs();
    for mutation in 0..10 {
        let mut v: Value = serde_json::from_str(&i.new).expect("frozen candidate");
        match mutation {
            0 => v["residents"][0]["lines"][0] = json!("A different story"),
            1 => v["residents"][0]["tile"] = json!([6, 5]),
            2 => v["items"][0]["item"] = json!("MASTER_BALL"),
            3 => v["items"][0]["quantity"] = json!(99),
            4 => v["items"][0]["flag"] = json!("EVENT_MN_GYM_6_DEFEATED"),
            5 => v["items"][0]["tile"] = json!([26, 5]),
            6 => v["exits"][0] = json!([16, 23]),
            7 => v["native_arrival"] = json!([17, 22]),
            8 => v["tileset"] = json!("house"),
            _ => v["dimensions_metatiles"] = json!([18, 14]),
        }
        assert!(
            revise_authored_cave_terrain(&i.base, &i.old, &v.to_string()).is_err(),
            "accepted unrelated source mutation {mutation}"
        );
    }
    assert!(
        revise_authored_cave_terrain(&i.base, &row_change(&i.old, [0, 0], "02"), &i.new).is_err(),
        "old rows must match installed native preimage"
    );
}

#[test]
#[ignore = "requires completed external original R12 pack and frozen before/after cave sources"]
fn r12_shelter_revision_rejects_unpaired_ladder_open_boundary_and_unsafe_hop_landing() {
    let i = inputs();
    for (at, block, reason) in [
        ([2, 2], "1b", "unpaired native ladder"),
        ([0, 0], "02", "open outer Floor"),
        ([0, 0], "0f", "open outer Water"),
        ([7, 8], "01", "blocked real hop landing"),
        ([8, 11], "01", "missing original exit"),
        ([4, 8], "01", "severed only ordinary lower stair"),
        ([5, 5], "ff", "unknown native cave block"),
    ] {
        assert!(
            revise_authored_cave_terrain(&i.base, &i.old, &row_change(&i.new, at, block)).is_err(),
            "accepted {reason}"
        );
    }
}

#[test]
#[ignore = "requires completed external original R12 pack and frozen before/after cave sources"]
fn r12_shelter_revision_retains_exact_active_mouth_return_and_original_actor_anchors() {
    let i = inputs();
    let old = &i.base.data().maps[MAP];
    let room = &i.result.data().maps[MAP];
    assert_eq!(room.attributes, old.attributes);
    assert_eq!(room.events, old.events);
    assert_eq!(room.objects.len(), 2);
    assert_eq!([room.objects[0].x, room.objects[0].y], [5, 5]);
    assert_eq!([room.objects[1].x, room.objects[1].y], [27, 5]);
    let exit = &room.events.warps[0];
    assert_eq!([exit.x, exit.y], [17, 23]);
    assert_eq!(exit.target_map, "MN_R12_C01R00");
    assert_eq!(
        exit.target_warp_id, 1,
        "original active exterior warp, not invented passive record"
    );
    assert_eq!(
        i.result.data().maps["MnR12C01R00"],
        i.base.data().maps["MnR12C01R00"]
    );
}
