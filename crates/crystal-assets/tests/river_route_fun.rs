//! Current-lineage river composition and obsolete-parent rejection.
use crystal_assets::{build_minnesota_fruit_groves, read_verified_compiled_game_pack};
use std::path::PathBuf;

#[test]
fn river_fun_preserves_unrelated_content_and_rejects_stale_or_unsafe_edits() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-r10-rock-gallery-v01.crystalpack"),
    )
    .expect("current compact parent");
    let text = std::fs::read_to_string(
        root.join("planning/minnesota/fruit-groves/groves.compact-v01.json"),
    )
    .expect("current river/fruit source");
    let source: serde_json::Value = serde_json::from_str(&text).expect("revision");
    let mut stale = source.clone();
    stale["base_content_hash"] = "stale".into();
    assert!(build_minnesota_fruit_groves(&base, &stale.to_string()).is_err());
    let mut unsafe_edge = source.clone();
    unsafe_edge["rivers"][0]["blocks"][0] = serde_json::json!(3);
    assert!(build_minnesota_fruit_groves(&base, &unsafe_edge.to_string()).is_err());
    let mut duplicate = source.clone();
    duplicate["rivers"][1]["map"] = duplicate["rivers"][0]["map"].clone();
    assert!(build_minnesota_fruit_groves(&base, &duplicate.to_string()).is_err());
    let result =
        build_minnesota_fruit_groves(&base, &text).expect("complete current river revision");
    assert_eq!(result.runtime_files(), base.runtime_files());
    assert_eq!(
        result.audio_manifest().expect("output audio"),
        base.audio_manifest().expect("parent audio")
    );
    assert_eq!(result.data().maps.len(), base.data().maps.len());
    for (name, old) in &base.data().maps {
        let new = &result.data().maps[name];
        assert_eq!(new.attributes, old.attributes, "geometry/ports {name}");
        assert_eq!(new.events, old.events, "events {name}");
        if !name.starts_with("MnMississippiLowerWoodsC")
            && !name.starts_with("MnMississippiUpstreamWoodsC")
        {
            assert_eq!(new.blocks, old.blocks, "unrelated terrain {name}");
            assert_eq!(
                result.data().wild_encounters.get(name),
                base.data().wild_encounters.get(name)
            );
        }
    }
    let historical = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-shore-gate-v03.crystalpack"),
    )
    .expect("retained read-only stale pack");
    let error = build_minnesota_fruit_groves(&historical, &text)
        .expect_err("retired geometry must fail before source identity checks");
    assert!(
        error.to_string().contains("retired Minnesota exterior")
            || error.to_string().contains("obsolete"),
        "{error:#}"
    );
}
