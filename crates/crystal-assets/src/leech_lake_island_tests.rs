//! External current-parent fixture; no pack/source bytes embedded at compilation.
use std::path::PathBuf;

use serde_json::Value;

use super::{LAKE, build_leech_lake_island};

#[test]
#[ignore = "requires CRYSTAL_LEECH_ISLAND_PARENT and CRYSTAL_NATIVE_FORCED_MOVEMENT_SOURCE"]
fn leech_island_preserves_current_parent_and_rejects_bypassed_grass_ledges_and_current() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/towns/leech-lake-island-v01");
    let base = crate::read_verified_compiled_game_pack(PathBuf::from(
        std::env::var_os("CRYSTAL_LEECH_ISLAND_PARENT").expect("completed parent pack"),
    ))
    .expect("verified current parent");
    let source = std::fs::read_to_string(root.join("after.json")).expect("reviewed island source");
    let contract =
        std::fs::read_to_string(root.join("contract-v02.json")).expect("current town preimage");
    let forced = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_NATIVE_FORCED_MOVEMENT_SOURCE").expect("pinned source path"),
    )
    .expect("local original forced movement");
    let revised = build_leech_lake_island(&base, &source, &contract, &forced)
        .expect("complete native island");
    assert_eq!(revised.data.maps.len(), base.data.maps.len());
    assert_eq!(
        revised.data.maps["MnDriftlessLampGallery"],
        base.data.maps["MnDriftlessLampGallery"]
    );
    assert_eq!(
        revised.data.maps[LAKE].events.warps,
        base.data.maps[LAKE].events.warps
    );
    for (id, data) in &base.data.wild_encounters {
        assert_eq!(revised.data.wild_encounters.get(id), Some(data));
    }
    for (id, trainer) in &base.data.trainers.trainers {
        assert_eq!(revised.data.trainers.trainers.get(id), Some(trainer));
    }
    let mut inhabited = base.clone();
    let mut native_lake = revised.data.wild_encounters[LAKE].clone();
    native_lake.zones.clear();
    native_lake.water_rate = Some(5);
    inhabited
        .data
        .wild_encounters
        .insert(LAKE.into(), native_lake.clone());
    inhabited.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        inhabited.format_version,
        &inhabited.data,
        &inhabited.audio_manifest,
        &inhabited.runtime_files,
        &inhabited.report,
    )
    .expect("explicit inhabited-parent fixture identity");
    let child = build_leech_lake_island(&inhabited, &source, &contract, &forced)
        .expect("preserve existing native lake wildlife");
    let mut inherited = child.data.wild_encounters[LAKE].clone();
    assert_eq!(inherited.zones.len(), 1);
    inherited.zones.clear();
    assert_eq!(
        inherited, native_lake,
        "new island must not replace previously authored lake habitats"
    );
    for (field, from, to) in [
        ("grass", "03", "02"),
        ("drop", "57", "4f"),
        ("cap", "36", "59"),
    ] {
        let mut bad: Value = serde_json::from_str(&source).expect("source");
        let row = if field == "grass" {
            16
        } else if field == "drop" {
            17
        } else {
            9
        };
        let column = if field == "cap" { 30 } else { 25 };
        let mut cells = bad["rows"][row]
            .as_str()
            .expect("native row")
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(cells[column], from);
        cells[column] = to.into();
        bad["rows"][row] = cells.join(" ").into();
        assert!(
            build_leech_lake_island(&base, &bad.to_string(), &contract, &forced).is_err(),
            "unsafe {field} accepted"
        );
    }
    let mut bad: Value = serde_json::from_str(&source).expect("source");
    bad["encounters"]["zones"][0]["grass"]["day"][0]["species"] = "MAGIKARP".into();
    assert!(build_leech_lake_island(&base, &bad.to_string(), &contract, &forced).is_err());
    let mut stale: Value = serde_json::from_str(&contract).expect("contract");
    stale["native_module_sha256"] = "stale".into();
    assert!(build_leech_lake_island(&base, &source, &stale.to_string(), &forced).is_err());
    assert!(build_leech_lake_island(&base, &source, &contract, "changed original source").is_err());
}
