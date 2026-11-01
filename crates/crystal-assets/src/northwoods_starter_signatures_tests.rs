//! Focused binary-unit preservation checks against the external Northwoods pack.

use std::path::PathBuf;

use crystal_assets::CompiledGamePack;
use crystal_core::battle::damage::MoveCategory;
use crystal_core::models::Stat;
use serde_json::Value;

const CATALOG: &str =
    include_str!("../../../planning/minnesota/pokemon/starter-signature-moves.json");

fn base() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_STARTER_SIGNATURE_BASE_PACK")
            .expect("set CRYSTAL_STARTER_SIGNATURE_BASE_PACK to the completed224-bank pack"),
    );
    crystal_assets::read_verified_compiled_game_pack(path).expect("verified actual Northwoods pack")
}

#[test]
#[ignore = "requires the completed external Northwoods pack"]
fn starter_supplement_preserves_every_existing_asset_and_final_only_learnsets() {
    let base = base();
    let data = base.data();
    assert_eq!(data.moves.len(), 270);
    assert_eq!(
        data.moves.values().map(|mov| mov.source_index).max(),
        Some(270)
    );
    let pack = crystal_assets::build_northwoods_starter_signatures_modpack(&base)
        .expect("focused starter supplement");
    crystal_assets::verify_northwoods_starter_signatures_preservation(&base, &pack)
        .expect("whole-dataset, bundle, report and audio preservation");
    assert_eq!(pack.data().moves.len(), 272);
    assert_eq!(pack.data().move_names.len(), 272);
    assert_eq!(pack.data().battle_animation_table.len(), 273);
    assert_eq!(pack.data().pokemon, data.pokemon);
    assert_eq!(pack.data().maps, data.maps);
    assert_eq!(pack.data().audio, data.audio);
    assert_eq!(pack.compiled_audio(), base.compiled_audio());
    for (path, bytes) in base.runtime_files() {
        assert_eq!(
            pack.runtime_files().get(path),
            Some(bytes),
            "existing asset {path}"
        );
    }
    let overworld_banks = base
        .runtime_files()
        .keys()
        .filter(|path| path.starts_with("gfx/pokemon/") && path.ends_with("/overworld.2bpp"))
        .count();
    assert_eq!(
        overworld_banks, 224,
        "the actual complete reviewed sprite roster"
    );
    assert_eq!(pack.runtime_files().len(), base.runtime_files().len() + 8);
    assert_eq!(&pack.data().move_names[..270], &data.move_names);
    assert_eq!(
        &pack.data().battle_animation_table[..271],
        &data.battle_animation_table
    );
    for (id, mov) in &data.moves {
        assert_eq!(pack.data().moves.get(id), Some(mov), "existing move {id}");
    }
    for (species, before) in &data.learnsets {
        let after = &pack.data().learnsets[species];
        let expected = match species.as_str() {
            "BOREALYNX" => Some("RIMEBRAND"),
            "VOYAGEAR" => Some("PADDLE_SWEEP"),
            _ => None,
        };
        if let Some(id) = expected {
            assert_eq!(after.len(), before.len() + 1);
            let additions: Vec<_> = after.iter().filter(|row| row.1 == id).collect();
            assert_eq!(additions.len(), 1);
            assert_eq!(additions[0].0, 36);
            assert_eq!(
                after.iter().filter(|row| row.1 != id).collect::<Vec<_>>(),
                before.iter().collect::<Vec<_>>()
            );
        } else {
            assert_eq!(after, before, "unrelated learner {species}");
        }
    }
    let fire = &pack.data().moves["RIMEBRAND"];
    assert_eq!(
        (
            fire.source_index,
            fire.power,
            fire.accuracy,
            fire.pp,
            fire.effect_chance
        ),
        (271, 85, 100, 10, 20)
    );
    assert_eq!(fire.move_type, "FIRE");
    assert_eq!(fire.effect, "BURN_HIT");
    assert_eq!(fire.stat, None);
    let water = &pack.data().moves["PADDLE_SWEEP"];
    assert_eq!(
        (
            water.source_index,
            water.power,
            water.accuracy,
            water.pp,
            water.effect_chance
        ),
        (272, 80, 100, 15, 30)
    );
    assert_eq!(water.move_type, "WATER");
    assert_eq!(water.effect, "ATTACK_DOWN_HIT");
    assert_eq!(water.stat, Some(Stat::Attack));
    assert_eq!(water.amount, Some(-1));
    for id in ["RIMEBRAND", "PADDLE_SWEEP"] {
        assert_eq!(
            pack.data().type_categories.moves[id],
            MoveCategory::Physical
        );
        let index = usize::from(pack.data().moves[id].source_index);
        assert_eq!(
            pack.data().battle_animation_table[index],
            crystal_assets::starter_signature_animations::animation_label(id)
        );
    }
    assert_eq!(
        pack.data().moves["SLIPPER_WALTZ"],
        data.moves["SLIPPER_WALTZ"]
    );
    println!(
        "actual base max270; extended272; all224overworld banks and {} prior runtime assets/audio unchanged",
        base.runtime_files().len()
    );
}

#[test]
#[ignore = "requires the completed external Northwoods pack"]
fn starter_supplement_rejects_reinstallation_and_bad_indices_or_learners() {
    let base = base();
    let original_identity = base.identity().expect("base identity");
    let pack =
        crystal_assets::build_northwoods_starter_signatures_modpack(&base).expect("supplement");
    let duplicate = crystal_assets::build_northwoods_starter_signatures_modpack(&pack)
        .expect_err("duplicate supplement must fail");
    assert!(duplicate.to_string().contains("already installed"));
    for (field, replacement) in [
        ("index", serde_json::json!(270)),
        ("learner", serde_json::json!("EMBERKIT")),
        ("level", serde_json::json!(1)),
        ("category", serde_json::json!("status")),
    ] {
        let mut json: Value = serde_json::from_str(CATALOG).expect("authored catalog");
        let value = match field {
            "index" => &mut json["moves"][0]["battle"]["source_index"],
            "learner" => &mut json["moves"][0]["learners"][0]["runtime_name"],
            "level" => &mut json["moves"][0]["learners"][0]["level"],
            _ => &mut json["moves"][0]["category"],
        };
        *value = replacement;
        assert!(
            crystal_assets::build_northwoods_starter_signatures_modpack_from_json(
                &base,
                &json.to_string()
            )
            .is_err(),
            "reject malformed {field}"
        );
    }
    let mut balanced: Value = serde_json::from_str(CATALOG).expect("authored catalog");
    balanced["moves"][0]["battle"]["pp"] = serde_json::json!(11);
    balanced["moves"][0]["battle"]["power"] = serde_json::json!(86);
    let adjusted = crystal_assets::build_northwoods_starter_signatures_modpack_from_json(
        &base,
        &balanced.to_string(),
    )
    .expect("JSON remains authoritative for supported balancing changes");
    assert_eq!(adjusted.data().moves["RIMEBRAND"].pp, 11);
    assert_eq!(adjusted.data().moves["RIMEBRAND"].power, 86);
    assert_eq!(
        base.identity().expect("unchanged base identity"),
        original_identity
    );
    assert!(!base.data().moves.contains_key("RIMEBRAND"));
}

#[test]
#[ignore = "requires the completed external Northwoods pack and test-fixtures feature"]
#[cfg(feature = "test-fixtures")]
fn starter_preservation_rejects_an_altered_existing_overworld_bank() {
    let base = base();
    let pack =
        crystal_assets::build_northwoods_starter_signatures_modpack(&base).expect("supplement");
    let mut files = pack.runtime_files().clone();
    let path = files
        .keys()
        .find(|path| path.starts_with("gfx/pokemon/") && path.ends_with("/overworld.2bpp"))
        .expect("actual custom overworld bank")
        .clone();
    files.get_mut(&path).expect("existing bank")[0] ^= 1;
    let altered = pack.with_runtime_files_for_tests(files);
    let error = crystal_assets::verify_northwoods_starter_signatures_preservation(&base, &altered)
        .expect_err("even one changed existing native byte must fail");
    assert!(
        error.to_string().contains("existing runtime file changed"),
        "{error}"
    );
}
