//! Certify the accepted shore bends against the installed Ryan-house epoch.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::{
    CompiledGamePack, read_verified_compiled_game_pack, revise_authored_route_terrain,
};
use serde_json::Value;

const CANDIDATE: &str = "planning/minnesota/renders/north-shore/house-merge/897d491e05181a05";

struct Inputs {
    base: CompiledGamePack,
    result: CompiledGamePack,
    previous: String,
    revised: String,
    dossier: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("real repository root")
}

fn source(name: &str) -> String {
    std::fs::read_to_string(root().join(CANDIDATE).join(name))
        .expect("immutable current-house source")
}

fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let path = std::env::var_os("CRYSTAL_NORTH_SHORE_BENDS_PACK")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                root().join("content-packs/minnesota-northstar-ryans-house.crystalpack")
            });
        let base = read_verified_compiled_game_pack(path)
            .expect("completed native house pack, never the pre-house 505 generation");
        let previous = source("preimage/MN_R11.layout.json");
        let revised = source("MN_R11.layout.json");
        // The live planning name Sivren has not been published. This separate
        // certificate names installed MARA and leaves all party data unchanged.
        let dossier = source("native-certification/MN_R11.c0c15e-certification.json");
        let result = revise_authored_route_terrain(&base, &previous, &revised, &dossier)
            .expect("bounded sixteen-cell revision retaining Ryan's passive arrival3");
        Inputs {
            base,
            result,
            previous,
            revised,
            dossier,
        }
    })
}

#[test]
#[ignore = "requires the completed external current-house pack and frozen candidate"]
fn north_shore_bends_change_exactly_sixteen_cells_and_no_other_content() {
    let i = inputs();
    let expected: Vec<(usize, usize, u16, u16)> = vec![
        (20, 75, 0x02, 0x05),
        (21, 75, 0x02, 0x05),
        (22, 75, 0x02, 0x05),
        (23, 75, 0x02, 0x05),
        (22, 76, 0x02, 0x05),
        (23, 76, 0x02, 0x05),
        (23, 83, 0x02, 0x05),
        (22, 84, 0x01, 0x62),
        (23, 84, 0x02, 0x05),
        (22, 85, 0x01, 0x62),
        (23, 85, 0x02, 0x05),
        (21, 86, 0x01, 0x65),
        (22, 86, 0x01, 0x65),
        (24, 86, 0x02, 0x05),
        (21, 87, 0x02, 0x05),
        (22, 87, 0x02, 0x05),
    ];
    let before = i.base.data();
    let after = i.result.data();
    assert_eq!(before.maps.len(), 510);
    let mut actual = Vec::new();
    for y in 0..96 {
        for x in 0..56 {
            let name = format!("MnR11C{:02}R{:02}", x / 32, y / 24);
            let old = &before.maps[&name];
            let new = &after.maps[&name];
            let index = (y % 24) * usize::from(old.attributes.width) + x % 32;
            if old.blocks[index] != new.blocks[index] {
                actual.push((x, y, old.blocks[index], new.blocks[index]));
            }
        }
    }
    assert_eq!(actual, expected, "only the reviewed fork/Lester cells");
    let name = "MnR11C00R03";
    let mut restored = after.clone();
    restored
        .maps
        .get_mut(name)
        .expect("existing south chunk")
        .blocks = before.maps[name].blocks.clone();
    let label = before.maps[name]
        .attributes
        .blocks_label
        .as_ref()
        .expect("owned split terrain payload");
    restored
        .map_blocks
        .insert(label.clone(), before.map_blocks[label].clone());
    assert_eq!(
        restored, *before,
        "all maps, connections, rooms, scripts, parties, flags, rewards, encounters, HOME and recovery stay exact"
    );
    assert_eq!(i.base.format_version(), i.result.format_version());
    assert_eq!(i.base.runtime_files(), i.result.runtime_files());
    assert_eq!(i.base.compiled_audio(), i.result.compiled_audio());
    assert_eq!(
        i.base.audio_manifest().expect("original manifest"),
        i.result.audio_manifest().expect("retained manifest")
    );
    assert_eq!(i.base.report(), i.result.report());
    assert_ne!(
        i.base.identity().expect("input identity"),
        i.result.identity().expect("revised identity")
    );
}

#[test]
#[ignore = "requires the completed external current-house pack and frozen candidate"]
fn north_shore_bends_preserve_house_passive_three_and_reject_active_one() {
    let i = inputs();
    for pack in [&i.base, &i.result] {
        let outside = &pack.data().maps["MnR11C00R00"];
        let room = &pack.data().maps["MnRyansHouse"];
        let outside_constant = outside
            .attributes
            .map_constant
            .as_deref()
            .expect("bank constant");
        let room_constant = room
            .attributes
            .map_constant
            .as_deref()
            .expect("house constant");
        assert_eq!(outside.events.warps.len(), 2);
        for (warp, (index, x, y)) in outside.events.warps.iter().zip([(1, 23, 21), (2, 23, 22)]) {
            assert_eq!((warp.index, warp.x, warp.y), (index, x, y));
            assert_eq!(warp.target_map, room_constant);
            assert_eq!(warp.target_map_constant, room_constant);
            assert_eq!(warp.target_warp_id, 3);
        }
        assert_eq!(room.events.warps.len(), 3);
        for (warp, (index, x, y)) in room
            .events
            .warps
            .iter()
            .zip([(1, 5, 7), (2, 6, 7), (3, 5, 6)])
        {
            assert_eq!((warp.index, warp.x, warp.y), (index, x, y));
            assert_eq!(warp.target_map, outside_constant);
            assert_eq!(warp.target_map_constant, outside_constant);
            assert_eq!(warp.target_warp_id, 2);
        }
        assert_eq!(room.objects.len(), 1);
        assert_eq!(
            room.objects[0].object_identifier.as_deref(),
            Some("MN_RYAN")
        );
        assert_eq!((room.objects[0].x, room.objects[0].y), (6, 3));
    }
    let mut invalid: Value = serde_json::from_str(&i.dossier).expect("certification source");
    let door = invalid["doors"]
        .as_array_mut()
        .expect("authored doors")
        .iter_mut()
        .find(|door| door["target"] == "MnRyansHouse")
        .expect("current actual house door");
    door["target_warp"] = 1.into();
    let invalid = serde_json::to_string(&invalid).expect("negative active-arrival source");
    let error = revise_authored_route_terrain(&i.base, &i.previous, &i.revised, &invalid)
        .expect_err("an active carpet cannot replace certified passive3");
    assert!(
        error
            .to_string()
            .contains("source/native door or passive return"),
        "exact native warp comparison must reject target1: {error:#}"
    );
}

#[test]
#[ignore = "requires the completed external current-house pack and frozen candidate"]
fn north_shore_bends_reject_reusing_the_old_house_terrain_preimage() {
    let i = inputs();
    let error = revise_authored_route_terrain(&i.result, &i.previous, &i.revised, &i.dossier)
        .expect_err("an already changed sixteen-cell preimage must not be silently accepted");
    assert!(
        error
            .to_string()
            .contains("installed blocks do not match previous layout"),
        "must reject the actual stale source generation: {error:#}"
    );
}

#[test]
#[ignore = "requires the completed external current-house pack and frozen candidate"]
fn north_shore_bends_certification_preserves_party_and_rejects_unpublished_name() {
    let i = inputs();
    let authoring: Value =
        serde_json::from_str(&source("MN_R11.json")).expect("live-era authoring snapshot");
    let certified: Value = serde_json::from_str(&i.dossier).expect("native certification");
    assert_eq!(authoring["trainers"][2]["display_name"], "Sivren");
    assert_eq!(certified["trainers"][2]["display_name"], "Mara");
    assert_eq!(
        authoring["trainers"][2]["party"],
        certified["trainers"][2]["party"]
    );
    let mut invalid = certified;
    invalid["trainers"][2]["display_name"] = "Sivren".into();
    let error = revise_authored_route_terrain(
        &i.base,
        &i.previous,
        &i.revised,
        &serde_json::to_string(&invalid).expect("unpublished trainer name"),
    )
    .expect_err("certification cannot silently rename native MARA");
    assert!(
        error
            .to_string()
            .contains("MN_R11_T03: source/native trainer identity, flag or party differs"),
        "native trainer identity remains strict: {error:#}"
    );
}
