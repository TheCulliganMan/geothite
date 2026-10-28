//! The one-item mart publication preserves the complete actual story pack.
use std::path::PathBuf;

use crystal_assets::{build_northstar_supplies, read_verified_compiled_game_pack};
use serde_json::Value;

#[test]
#[ignore = "requires the completed ignored Northstar relay and supplies packs"]
fn northstar_supplies_preserves_every_other_dataset_file_and_audio_program() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let source = std::fs::read_to_string(root.join("planning/minnesota/northstar-supplies.json"))
        .expect("supplies contract");
    let base = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-relay-v2.crystalpack"),
    )
    .expect("actual relay parent");
    let result = build_northstar_supplies(&base, &source).expect("one-item native publication");
    let written = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-northstar-supplies.crystalpack"),
    )
    .expect("actual written supplies pack");
    assert_eq!(
        result.identity().expect("rebuilt identity"),
        written.identity().expect("written identity")
    );
    assert!(base.compiled_audio() == result.compiled_audio());
    assert!(base.runtime_files() == result.runtime_files());
    let mut expected = serde_json::to_value(base.data()).expect("complete before dataset");
    expected["marts"]["MART_MN_ST_CLOUD"]
        .as_array_mut()
        .expect("installed mart")
        .push(Value::String("AWAKENING".into()));
    let actual = serde_json::to_value(result.data()).expect("complete after dataset");
    for (field, value) in expected.as_object().expect("dataset fields") {
        assert!(
            actual.get(field) == Some(value),
            "unexpected dataset change: {field}"
        );
    }
    assert!(build_northstar_supplies(&result, &source).is_err());
    let canonical: Value = serde_json::from_str(&source).expect("source JSON");
    for (field, value) in [
        ("base_content_hash", Value::String("stale".into())),
        ("mart", Value::String("MART_MN_BRAINERD".into())),
        ("append", Value::String("HYPER_POTION".into())),
        ("expected_stock", Value::Array(vec![])),
        ("unexpected", Value::Bool(true)),
    ] {
        let mut wrong = canonical.clone();
        wrong[field] = value;
        assert!(
            build_northstar_supplies(&base, &wrong.to_string()).is_err(),
            "reject {field}"
        );
    }
}
