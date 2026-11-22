use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use crystal_core::models::{Trainer, TrainerCatalog};
use serde_json::json;

use super::{PREVIEW_PREFIX, Plan, SAVE_POLICY, publication_files, sha256};
use crate::{AssetRoot, TRAINER_PORTRAIT_OVERRIDES_PATH, TrainerPortraitOverrides};

fn catalog() -> TrainerCatalog {
    let mut catalog = TrainerCatalog::default();
    for (id, class) in [
        ("MN_TEST_ONE", "YOUNGSTER"),
        ("MN_TEST_TWO", "YOUNGSTER"),
        ("JOEY", "YOUNGSTER"),
    ] {
        catalog
            .insert(Trainer {
                name: id.into(),
                trainer_id: id.into(),
                trainer_class: class.into(),
                party: Vec::new(),
                win_quote: String::new(),
                lose_quote: String::new(),
                items: Vec::new(),
                base_reward: 7,
                ai_move_flags: 3,
                ai_item_switch_flags: 5,
                encounter_music: "MUSIC_YOUNGSTER_ENCOUNTER".into(),
                ai_layers: vec!["BASIC".into()],
            })
            .expect("unique fixture trainer");
    }
    catalog
}

fn source() -> String {
    let rows: Vec<String> = (0..56)
        .map(|y| (0..56).map(|x| b".123"[(x + y) % 4] as char).collect())
        .collect();
    json!({"schema_version":1,"id":"mn_portrait_test","palette_rgb555":[32767,25370,8683,0],"battle_rows":rows}).to_string()
}

fn fixture() -> Result<(AssetRoot, BTreeMap<String, Vec<u8>>, serde_json::Value)> {
    let root = AssetRoot::new_temporary_in(std::env::temp_dir())?;
    let path = root.repository_root.join("source.json");
    fs::write(&path, source())?;
    let mut files: BTreeMap<_, _> = crate::encode_trainer_battle_native(&source())?
        .into_iter()
        .map(|(path, bytes)| (path.replace("mn_portrait_test", "youngster"), bytes))
        .collect();
    files.insert(
        "gfx/sprites/youngster.png".into(),
        b"unrelated field sprite".to_vec(),
    );
    let plan = json!({"schema_version":1,"portraits":[{"source":path,"trainer_class":"YOUNGSTER",
        "trainer_ids":["MN_TEST_ONE"],"expected_art":{"id":"youngster",
            "png_sha256":sha256(&files["gfx/trainers/youngster.png"]),
            "tiles_sha256":sha256(&files["gfx/trainers/youngster.2bpp"]),
            "palette_sha256":sha256(&files["gfx/trainers/youngster.gbcpal"])}}]});
    Ok((root, files, plan))
}

#[test]
fn overrides_isolate_exact_ids_without_changing_shared_classes_or_art() -> Result<()> {
    let (_root, files, plan) = fixture()?;
    let catalog = catalog();
    let untouched_catalog = catalog.clone();
    let revised = publication_files(&files, &catalog, serde_json::from_value(plan)?)?;
    assert_eq!(revised.len(), files.len() + 4);
    for (key, bytes) in &files {
        assert_eq!(revised.get(key), Some(bytes), "preserve {key}");
    }
    assert_eq!(
        catalog, untouched_catalog,
        "class, AI, music, rewards and parties stay exact"
    );
    let overrides = TrainerPortraitOverrides::from_runtime_files(&revised, &catalog)?;
    assert_eq!(
        overrides.resolve("MN_TEST_ONE", "YOUNGSTER"),
        "mn_portrait_test"
    );
    assert_eq!(overrides.resolve("MN_TEST_TWO", "YOUNGSTER"), "youngster");
    assert_eq!(overrides.resolve("JOEY", "YOUNGSTER"), "youngster");
    assert_eq!(overrides.resolve("MN_TEST_ONE", "BEAUTY"), "beauty");
    assert_eq!(
        TrainerPortraitOverrides::from_runtime_files(&files, &catalog)?,
        TrainerPortraitOverrides::default()
    );
    Ok(())
}

#[test]
fn publication_rejects_wrong_ids_classes_duplicates_collisions_and_stale_art() -> Result<()> {
    let (_root, files, plan) = fixture()?;
    let catalog = catalog();
    for ids in [
        json!(["MN_UNKNOWN"]),
        json!(["JOEY"]),
        json!(["MN_TEST_ONE", "MN_TEST_ONE"]),
        json!([]),
    ] {
        let mut wrong = plan.clone();
        wrong["portraits"][0]["trainer_ids"] = ids;
        assert!(publication_files(&files, &catalog, serde_json::from_value(wrong)?).is_err());
    }
    let mut wrong_class = catalog.clone();
    wrong_class
        .trainers
        .get_mut("MN_TEST_ONE")
        .expect("fixture")
        .trainer_class = "BEAUTY".into();
    assert!(
        publication_files(&files, &wrong_class, serde_json::from_value(plan.clone())?).is_err()
    );
    for extension in ["png", "2bpp", "gbcpal"] {
        let mut stale = files.clone();
        stale
            .get_mut(&format!("gfx/trainers/youngster.{extension}"))
            .expect("fixture bank")
            .push(1);
        assert!(
            publication_files(&stale, &catalog, serde_json::from_value(plan.clone())?).is_err()
        );
    }
    let revised = publication_files(&files, &catalog, serde_json::from_value(plan.clone())?)?;
    assert!(publication_files(&revised, &catalog, serde_json::from_value(plan)?).is_err());
    Ok(())
}

#[test]
fn runtime_manifest_rejects_unknown_ids_wrong_classes_and_missing_banks() -> Result<()> {
    let (_root, files, plan) = fixture()?;
    let catalog = catalog();
    let revised = publication_files(&files, &catalog, serde_json::from_value(plan)?)?;
    for (id, class, portrait) in [
        ("MN_UNKNOWN", "YOUNGSTER", "mn_portrait_test"),
        ("MN_TEST_ONE", "BEAUTY", "mn_portrait_test"),
        ("MN_TEST_ONE", "YOUNGSTER", "../youngster"),
    ] {
        let mut invalid = revised.clone();
        invalid.insert(
            TRAINER_PORTRAIT_OVERRIDES_PATH.into(),
            serde_json::to_vec(&json!({
            "schema_version":1,"trainers":{id:{"trainer_class":class,"portrait":portrait}}}))?,
        );
        assert!(TrainerPortraitOverrides::from_runtime_files(&invalid, &catalog).is_err());
    }
    for extension in ["png", "2bpp", "gbcpal"] {
        let mut invalid = revised.clone();
        invalid.remove(&format!("gfx/trainers/mn_portrait_test.{extension}"));
        assert!(TrainerPortraitOverrides::from_runtime_files(&invalid, &catalog).is_err());
    }
    Ok(())
}

#[test]
fn native_battle_only_encoder_roundtrips_tiles_and_rejects_field_payloads() -> Result<()> {
    let encoded = crate::encode_trainer_battle_native(&source())?;
    assert_eq!(encoded.len(), 3);
    assert!(encoded.keys().all(|path| path.starts_with("gfx/trainers/")));
    let tiles = &encoded["gfx/trainers/mn_portrait_test.2bpp"];
    for y in 0..56 {
        for x in 0..56 {
            let offset = (y / 8 * 7 + x / 8) * 16 + (y % 8) * 2;
            let bit = 7 - x % 8;
            assert_eq!(
                ((tiles[offset] >> bit) & 1) | (((tiles[offset + 1] >> bit) & 1) << 1),
                ((x + y) % 4) as u8
            );
        }
    }
    let mut field: serde_json::Value = serde_json::from_str(&source())?;
    field["overworld_frames"] = json!([]);
    assert!(crate::encode_trainer_battle_native(&field.to_string()).is_err());
    Ok(())
}

#[test]
#[ignore = "requires CRYSTAL_PORTRAIT_OVERRIDE_PARENT and CRYSTAL_PORTRAIT_OVERRIDE_PLAN"]
fn override_publication_preserves_complete_pack_except_new_art_and_mapping() -> Result<()> {
    let base = crate::read_verified_compiled_game_pack(
        std::env::var_os("CRYSTAL_PORTRAIT_OVERRIDE_PARENT").expect("explicit parent"),
    )?;
    let source = fs::read_to_string(
        std::env::var_os("CRYSTAL_PORTRAIT_OVERRIDE_PLAN").expect("explicit plan"),
    )?;
    let mut recipe: serde_json::Value = serde_json::from_str(&source)?;
    // Cargo runs this test from the crate. The published plan names paths
    // relative to the repository, as used by the pack command.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for portrait in recipe["portraits"].as_array_mut().context("portraits")? {
        portrait["source"] = json!(repo.join(portrait["source"].as_str().context("source path")?));
    }
    let source = recipe.to_string();
    let plan: Plan = serde_json::from_str(&source)?;
    let mut stale = recipe;
    stale["expected_parent_content_hash"] = json!("unrelated-parent");
    let error = super::add_minnesota_trainer_portrait_overrides(&base, &stale.to_string())
        .expect_err("refuse a portrait plan for a different parent");
    assert!(error.to_string().contains("different parent"));
    let revised = super::add_minnesota_trainer_portrait_overrides(&base, &source)?;
    let overrides = TrainerPortraitOverrides::from_runtime_files(
        &revised.runtime_files,
        &revised.data.trainers,
    )?;
    for portrait in plan.portraits {
        for id in portrait.trainer_ids {
            assert!(
                overrides
                    .resolve(&id, &portrait.trainer_class)
                    .starts_with("mn_portrait_")
            );
        }
    }
    for (key, bytes) in &base.runtime_files {
        if key != TRAINER_PORTRAIT_OVERRIDES_PATH && key != SAVE_POLICY {
            assert_eq!(revised.runtime_files.get(key), Some(bytes));
        }
    }
    assert!(!revised.runtime_files.contains_key(SAVE_POLICY));
    assert_ne!(
        revised.identity.runtime_modpack_id,
        base.identity.runtime_modpack_id
    );
    let mut normalized = revised;
    let marker = normalized
        .report
        .manifests
        .pop()
        .expect("portrait save manifest");
    if let Some(policy) = base.runtime_files.get(SAVE_POLICY) {
        normalized
            .runtime_files
            .insert(SAVE_POLICY.into(), policy.clone());
    }
    let artwork = super::portrait_identity(&normalized)?;
    assert_eq!(marker, format!("{PREVIEW_PREFIX}{}", artwork.content_hash));
    normalized.runtime_files.clone_from(&base.runtime_files);
    normalized.identity.clone_from(&base.identity);
    assert_eq!(
        normalized, base,
        "all gameplay, audio, field art and report metadata remain exact"
    );
    Ok(())
}
