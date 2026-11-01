//! Frozen source, parent guards and exact inverse-residual regressions.
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::json;

use super::{CEDAR, CONTRACT_SHA, Contract, FLAG, NECK};
use crate::CompiledGamePack;

const SOURCE: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/geography/native-west-v02/contract.json"
);

fn fixture() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_SUPERIOR_GEOGRAPHY_PARENT")
                .expect("actual completed Grand Marais town parent"),
        );
        crate::read_verified_compiled_game_pack(path).expect("verified geography parent")
    })
}

#[test]
fn superior_geography_frozen_west_source_preserves_exact_whole_map_dimensions() {
    assert_eq!(super::hash(SOURCE.as_bytes()), CONTRACT_SHA);
    let plan: Contract = serde_json::from_str(SOURCE).expect("frozen geography source");
    assert_eq!(plan.preimages.len(), 4);
    let rows = plan.inland["neck"]["rows"]
        .as_array()
        .expect("whole65-wide rows");
    assert_eq!(rows.len(), 19);
    assert!(
        rows.iter()
            .all(|r| r.as_str().expect("explicit row").split_whitespace().count() == 65)
    );
    assert_eq!(plan.inland["neck"]["dimensions_metatiles"], json!([65, 19]));
    assert_eq!(plan.inland["neck"]["group_id"], 40);
    assert_eq!(plan.inland["neck"]["map_id"], 8);
    assert_eq!(
        plan.inland["cedar"]["cells"]
            .as_array()
            .expect("Cedar changes")
            .len(),
        440
    );
    let headland = plan.headland["changes"]
        .as_array()
        .expect("two terminal chunks");
    assert_eq!(headland.len(), 2);
    assert_eq!(
        headland
            .iter()
            .map(|c| c["cells"].as_array().expect("cells").len())
            .sum::<usize>(),
        306
    );
    for row in plan.inland["cedar"]["text_changes"]
        .as_array()
        .expect("two sign corrections")
    {
        assert!(
            row["after"]
                .as_str()
                .expect("sign text")
                .trim_matches('"')
                .chars()
                .count()
                <= 18
        );
    }
}

#[test]
#[ignore = "requires completed external town parent in CRYSTAL_SUPERIOR_GEOGRAPHY_PARENT"]
fn superior_geography_native_map_registration_scope_and_uncached_behavior_are_exact() {
    let base = fixture();
    let output = super::build_superior_inland_geography_modpack(base, SOURCE)
        .expect("reviewed native west-entry geography");
    let plan: Contract = serde_json::from_str(SOURCE).expect("contract");
    super::validation::validate(base, &output, &plan).expect("whole residual/native graph proof");
    assert_eq!(output.data.maps.len(), 694);
    assert_eq!(output.data.maps[NECK].attributes.width, 65);
    assert_eq!(output.data.maps[NECK].attributes.height, 19);
    assert_eq!(
        output.data.maps[CEDAR].objects,
        base.data.maps[CEDAR].objects
    );
    assert_eq!(
        output.data.wild_encounters[CEDAR],
        base.data.wild_encounters[CEDAR]
    );
    assert_eq!(
        output.data.runtime_spawn_points,
        base.data.runtime_spawn_points
    );
    assert_eq!(output.runtime_files, base.runtime_files);
    assert_eq!(output.compiled_audio, base.compiled_audio);
    assert_eq!(output.data.story_event_script_constants.global[FLAG], 2387);
    let pickup = output.data.maps[NECK]
        .script_field_pickups
        .first()
        .expect("native camp pickup");
    assert_eq!(pickup.item_id.as_deref(), Some("BERRY"));
    assert_eq!(pickup.quantity, 3);
    assert_eq!(pickup.event_flag.as_deref(), Some(FLAG));
}

#[test]
#[ignore = "requires completed external town parent in CRYSTAL_SUPERIOR_GEOGRAPHY_PARENT"]
fn superior_geography_rejects_unowned_wire_art_encounter_recovery_and_pickup_drift() {
    let base = fixture();
    let output =
        super::build_superior_inland_geography_modpack(base, SOURCE).expect("geography fixture");
    let plan: Contract = serde_json::from_str(SOURCE).expect("contract");
    for corruption in 0..7 {
        let mut bad = output.clone();
        match corruption {
            0 => {
                bad.data
                    .npcs
                    .insert(CEDAR.into(), json!({"objects":base.data.npcs[CEDAR]}));
            }
            1 => {
                bad.runtime_files
                    .insert("geography-unowned-art.bin".into(), vec![0]);
            }
            2 => {
                bad.data
                    .wild_encounters
                    .get_mut(CEDAR)
                    .expect("old encounters")
                    .map_name = "wrong".into();
            }
            3 => {
                bad.data.runtime_spawn_points.insert(
                    "280".into(),
                    base.data
                        .runtime_spawn_points
                        .values()
                        .next()
                        .expect("old recovery")
                        .clone(),
                );
            }
            4 => {
                bad.data.story_event_script_constants.global.remove(FLAG);
            }
            5 => {
                bad.data
                    .maps
                    .get_mut(NECK)
                    .expect("new map")
                    .script_field_pickups[0]
                    .quantity = 4;
            }
            6 => {
                bad.data
                    .maps
                    .get_mut("MnMinneapolis")
                    .expect("unrelated city")
                    .blocks[0] ^= 1;
            }
            _ => unreachable!("fixed corruption range"),
        }
        bad.identity = super::identity(&bad).expect("corrupt fixture identity");
        assert!(
            super::validation::validate(base, &bad, &plan).is_err(),
            "accepted corruption{corruption}"
        );
    }
    let mut wrong_source = SOURCE.to_string();
    wrong_source.push(' ');
    assert!(super::build_superior_inland_geography_modpack(base, &wrong_source).is_err());
}
