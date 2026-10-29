//! Current-parent mine preservation and canonical northern geometry rejection.
use std::path::PathBuf;

use crystal_assets::{
    CompiledGamePack, build_current_r09_mine_pack, validate_compact_superior_geography,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const OLD: &str =
    include_str!("../../../planning/minnesota/caves/johto-quality-v01/MnR09Cave1F.previous.json");
const NEW: &str =
    include_str!("../../../planning/minnesota/caves/johto-quality-v01/MnR09Cave1F.json");
fn contract_source() -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/caves/current-r09-v01/native.contract.json"),
    )
    .expect("external reviewed current R09 mine contract")
}
const MINE: &str = "MnR09Cave1F";

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn parent() -> CompiledGamePack {
    crystal_assets::read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_CURRENT_R09_PARENT").expect("current R10 pack"),
    ))
    .expect("strict current parent")
}
fn revised_source(
    base: &CompiledGamePack,
    mutate: impl FnOnce(&mut Value),
) -> anyhow::Result<CompiledGamePack> {
    let mut source: Value = serde_json::from_str(NEW)?;
    mutate(&mut source);
    let raw = serde_json::to_string(&source)?;
    let mut contract: Value = serde_json::from_str(&contract_source())?;
    contract["revised_source_sha256"] = hash(raw.as_bytes()).into();
    let contract = serde_json::to_string(&contract)?;
    build_current_r09_mine_pack(base, OLD, &raw, &contract, &hash(contract.as_bytes()))
}

#[test]
#[ignore = "requires CRYSTAL_CURRENT_R09_PARENT and external current-r09-v01/native.contract.json"]
fn current_mine_preserves_the_regional_story_team_and_all_unowned_content() {
    let base = parent();
    let plan = contract_source();
    let out = build_current_r09_mine_pack(&base, OLD, NEW, &plan, &hash(plan.as_bytes()))
        .expect("current mine");
    validate_compact_superior_geography(&out).expect("current compact coast retained");
    assert_eq!(base.runtime_files(), out.runtime_files());
    assert_eq!(base.compiled_audio(), out.compiled_audio());
    assert_eq!(base.report(), out.report());
    assert_eq!(base.data().trainers, out.data().trainers);
    let old = &base.data().maps[MINE];
    let new = &out.data().maps[MINE];
    assert_eq!(
        old.blocks
            .iter()
            .zip(&new.blocks)
            .filter(|(a, b)| a != b)
            .count(),
        83
    );
    assert_eq!(new.attributes.border_block, 0x2e);
    let mut data = serde_json::to_value(out.data()).expect("output data");
    let original = serde_json::to_value(base.data()).expect("parent data");
    for (table, key) in [
        ("maps", MINE),
        ("map_attributes", MINE),
        ("map_blocks", "MnR09Cave1F_Blocks"),
    ] {
        data[table][key] = original[table][key].clone();
    }
    assert_eq!(
        data, original,
        "all other maps, story, flags, text, teams and indexes exact"
    );
    let mut uncached = out.data().clone();
    uncached.maps.remove(MINE);
    let assembled = uncached
        .assemble_map_module_from_compiled_payloads(MINE)
        .expect("uncached mine");
    let mut expected = new.clone();
    for label in ["MnR09Cave1F_MapScripts", "MnR09Cave1F_MapEvents"] {
        expected
            .scripts
            .insert(label.into(), out.data().map_scripts[label].clone());
    }
    assert_eq!(
        assembled, expected,
        "complete uncached story/index reconstruction"
    );
}

#[test]
#[ignore = "requires CRYSTAL_CURRENT_R09_PARENT and external current-r09-v01/native.contract.json"]
fn current_mine_rejects_unsafe_geometry_false_sources_and_native_story_drift() {
    let base = parent();
    assert!(
        build_current_r09_mine_pack(&base, OLD, NEW, &contract_source(), "unreviewed").is_err()
    );
    for mutate in [
        (|v: &mut Value| v["border_block"] = 2.into()) as fn(&mut Value),
        |v| v["story_objects"][0]["recovered_flag"] = "EVENT_MN_DEPOT_CLEARED".into(),
        |v| {
            let mut row = v["rows"][5]
                .as_str()
                .expect("row")
                .split_whitespace()
                .collect::<Vec<_>>();
            row[3] = "07";
            v["rows"][5] = row.join(" ").into();
        },
    ] {
        assert!(
            revised_source(&base, mutate).is_err(),
            "review hash does not permit unsafe content"
        );
    }
    let mut false_old: Value = serde_json::from_str(OLD).expect("source");
    false_old["story_trainers"][0]["party"][0]["species"] = "LUTEBRISK".into();
    let false_old = serde_json::to_string(&false_old).expect("false source");
    let mut plan: Value = serde_json::from_str(&contract_source()).expect("contract");
    plan["previous_source_sha256"] = hash(false_old.as_bytes()).into();
    let plan = serde_json::to_string(&plan).expect("false contract");
    assert!(
        build_current_r09_mine_pack(&base, &false_old, NEW, &plan, &hash(plan.as_bytes())).is_err()
    );
    let mut raw = serde_json::to_value(&base).expect("native parent");
    raw["data"]["trainers"]["trainers"]["MN_LODE_FOREMAN"]["party"][0]["moves"][0]["current_pp"] =
        1.into();
    let changed = serde_json::from_value(raw).expect("structurally valid changed native team");
    assert!(
        build_current_r09_mine_pack(
            &changed,
            OLD,
            NEW,
            &contract_source(),
            &hash(contract_source().as_bytes())
        )
        .is_err()
    );
}

#[test]
#[ignore = "requires CRYSTAL_CURRENT_R09_PARENT and external current-r09-v01/native.contract.json"]
fn compact_guard_rejects_retired_chunks_oversize_seams_ice_and_outdoor_overlap() {
    let base = parent();
    validate_compact_superior_geography(&base).expect("approved compact parent");
    let baseline = serde_json::to_value(&base).expect("parent data");
    for mutate in [
        (|v: &mut Value| v["data"]["maps"]["MnR11C01R00"]["attributes"]["width"] = 24.into())
            as fn(&mut Value),
        |v| v["data"]["maps"]["MnGrandMarais"]["attributes"]["width"] = 32.into(),
        |v| v["data"]["maps"]["MnR12C01R00"] = v["data"]["maps"]["MnR12C00R00"].clone(),
        |v| v["data"]["maps"]["MnGrandMarais"]["attributes"]["connections"][1]["offset"] = 1.into(),
        |v| {
            v["data"]["maps"]["MnR11C00R01"]["attributes"]["tileset_name"] =
                "mn_winter_north_shore".into()
        },
        |v| {
            let b = v["data"]["maps"]["MnR11C00R00"]["blocks"][0]
                .as_u64()
                .expect("block");
            v["data"]["tilesets"]["mn_north_shore"]["collision"][format!("{b:02x}")][0] =
                "ICE".into();
        },
        |v| v["data"]["maps"]["MnStPaul"]["attributes"]["width"] = 200.into(),
    ] {
        let mut value = baseline.clone();
        mutate(&mut value);
        let changed = serde_json::from_value(value).expect("structurally valid geometry mutation");
        assert!(validate_compact_superior_geography(&changed).is_err());
    }
}
