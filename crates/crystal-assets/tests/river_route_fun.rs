//! Publication boundaries for the two-bank route revision.
use crystal_assets::{build_river_route_fun, read_verified_compiled_game_pack};
use std::path::PathBuf;

#[test]
fn river_fun_preserves_unrelated_content_and_rejects_stale_or_unsafe_edits() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root");
    let base = read_verified_compiled_game_pack(
        root.join("content-packs/minnesota-park-pond-v05.crystalpack"),
    )
    .expect("completed river parent");
    let text = std::fs::read_to_string(
        root.join("planning/minnesota/connections/mississippi-fun-v01/revision.json"),
    )
    .expect("river source");
    let source: serde_json::Value = serde_json::from_str(&text).expect("revision");
    let mut stale = source.clone();
    stale["base_content_hash"] = "stale".into();
    assert!(build_river_route_fun(&base, &stale.to_string()).is_err());
    let mut unsafe_edge = source.clone();
    unsafe_edge["maps"][0]["cells"][0]["metatile"] = serde_json::json!([0, 0]);
    assert!(build_river_route_fun(&base, &unsafe_edge.to_string()).is_err());
    let mut duplicate = source.clone();
    duplicate["maps"][1]["id"] = duplicate["maps"][0]["id"].clone();
    assert!(build_river_route_fun(&base, &duplicate.to_string()).is_err());
    let result = build_river_route_fun(&base, &text).expect("complete river revision");
    assert_eq!(result.runtime_files(), base.runtime_files());
    assert_eq!(
        result.audio_manifest().expect("output audio"),
        base.audio_manifest().expect("parent audio")
    );
    assert_eq!(result.data().wild_encounters, base.data().wild_encounters);
    for (name, map) in &base.data().maps {
        if !name.starts_with("MnMississippiLowerWoodsC")
            && !name.starts_with("MnMississippiUpstreamWoodsC")
        {
            assert_eq!(&result.data().maps[name], map, "unrelated map {name}");
        }
    }
}
