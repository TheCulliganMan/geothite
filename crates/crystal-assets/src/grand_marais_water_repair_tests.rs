//! External written-pack and corruption coverage for the isolated Water repair.
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{MAP, repair_grand_marais_water_encounters, validate_preservation, validate_water};
use crate::CompiledGamePack;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("existing repository root")
}
fn source() -> String {
    std::fs::read_to_string(root().join("planning/minnesota/routes/MN_R11.json"))
        .expect("existing authoritative North Shore source")
}
fn base() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        crate::read_verified_compiled_game_pack(
            root().join("content-packs/minnesota-cohesion-r01-upper-loop-v01.crystalpack"),
        )
        .expect("completed exact58b9 terrain parent")
    })
}
fn repaired() -> &'static CompiledGamePack {
    static PACK: OnceLock<CompiledGamePack> = OnceLock::new();
    PACK.get_or_init(|| {
        repair_grand_marais_water_encounters(base(), &source())
            .expect("bounded native Water repair")
    })
}

#[test]
#[ignore = "requires the completed external58b9 terrain pack"]
fn grand_marais_water_repair_changes_only_two_fields_and_survives_strict_wire() {
    let parent = base();
    let result = repaired();
    assert_ne!(parent.identity, result.identity);
    validate_preservation(parent, result).expect("complete residual content equality");
    assert_eq!(
        parent.data.maps, result.data.maps,
        "every native module, including buoy terrain, exact"
    );
    let target = &result.data.wild_encounters[MAP];
    let donor = &parent.data.wild_encounters[super::TEMPLATE];
    assert_eq!(target.water, donor.water);
    assert_eq!(target.water_rate, donor.water_rate);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "grand-marais-water-{}-{nonce}.crystalpack",
        std::process::id()
    ));
    result
        .write_preserving_storage(&path)
        .expect("write actual Water repair");
    let written =
        crate::read_verified_compiled_game_pack(&path).expect("strict actual written readback");
    assert_eq!(&written, result, "entire actual native wire retained");
    validate_preservation(parent, &written)
        .expect("written preservation and all native slot rolls");
    std::fs::remove_file(path).expect("remove only this verification pack");
}

#[test]
#[ignore = "requires the completed external58b9 terrain pack"]
fn grand_marais_water_repair_rejects_stale_source_parent_and_repeat() {
    assert!(repair_grand_marais_water_encounters(base(), &(source() + " ")).is_err());
    assert!(repair_grand_marais_water_encounters(repaired(), &source()).is_err());
    let mut wrong = base().clone();
    wrong.identity.content_hash = "not-the-accepted-parent".into();
    assert!(repair_grand_marais_water_encounters(&wrong, &source()).is_err());
}

#[test]
#[ignore = "requires the completed external58b9 terrain pack"]
fn grand_marais_water_repair_rejects_missing_empty_illegal_and_zero_rate_water() {
    for variant in 0..5 {
        let mut target = repaired().data.wild_encounters[MAP].clone();
        match variant {
            0 => target.water = None,
            1 => target.water_rate = Some(0),
            2 => target.water.as_mut().expect("Water table").night.clear(),
            3 => target.water.as_mut().expect("Water table").day[1].species = "NOT_A_MON".into(),
            4 => {
                target.water.as_mut().expect("Water table").morning.pop();
            }
            _ => unreachable!("five native invalid-table cases"),
        }
        assert!(
            validate_water(repaired(), &target).is_err(),
            "invalid actual Water case{variant}"
        );
    }
}

#[test]
#[ignore = "requires the completed external58b9 terrain pack"]
fn grand_marais_water_repair_rejects_global_weights_and_unowned_content_changes() {
    let parent = base();
    for variant in 0..5 {
        let mut wrong = repaired().clone();
        match variant {
            0 => {
                wrong
                    .data
                    .encounter_slot_tables
                    .tables
                    .get_mut("water")
                    .expect("native Water weights")[0]
                    .threshold += 1
            }
            1 => wrong.data.maps.get_mut(MAP).expect("town").blocks[360] = 0x35,
            2 => {
                wrong.data.maps.get_mut(MAP).expect("town").scripts.clear();
            }
            3 => {
                wrong.runtime_files.remove("gfx/tilesets/johto.png");
            }
            4 => {
                wrong
                    .compiled_audio
                    .insert("unexpected-audio".into(), vec![1]);
            }
            _ => unreachable!("five explicit unrelated changes"),
        }
        assert!(
            validate_preservation(parent, &wrong).is_err(),
            "unowned change{variant} rejected"
        );
    }
    let mut grass = repaired().clone();
    grass
        .data
        .wild_encounters
        .get_mut(MAP)
        .expect("town encounters")
        .grass_rates
        .as_mut()
        .expect("retained grass rates")
        .insert("day".into(), 0);
    assert!(
        validate_preservation(parent, &grass).is_err(),
        "grass encounters cannot be changed"
    );
}
