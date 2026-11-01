use std::path::PathBuf;

use serde_json::Value;

use super::{append_winter_mixed_edges_art, hash, validate_residual};
use crate::CompiledGamePack;

const SPECIFICATION: &str =
    include_str!("../../../planning/minnesota/regions/ice/winter-mixed-edges-v01.json");

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_WINTER_MIXED_ART_TEST_PACK")
            .expect("external main ice region pack with corrected winter art"),
    )
    .canonicalize()
    .expect("completed winter pack");
    crate::read_verified_compiled_game_pack(path).expect("verified winter pack")
}

#[test]
#[ignore = "requires external pack with corrected winter art"]
fn mixed_winter_edges_match_source_pixels_and_preserve_all_old_owned_and_unrelated_data() {
    let base = fixture();
    let output = append_winter_mixed_edges_art(&base, SPECIFICATION).expect("mixed winter edges");
    validate_residual(&base, &output).expect("strict prefix and whole-pack equality");
    for (name, metatiles, pixels) in [
        (
            "mn_winter_outdoor",
            "64c816184c270411717d123a89c189188da5f843642b4e0815a59d165164a9ed",
            "93fbec10e71d2d7af81d39588a8ea4221d08fdc2ba14d0d788af65bfa1d5c3fd",
        ),
        (
            "mn_winter_north_shore",
            "cd54acd379e709626bc29b7dd7e5ef45d1937e8f256a0a2c600df76186a34faf",
            "528ed5b914d892849aa5e1755f12bfc4a253f9a07086683ce25f4bf76e0fc277",
        ),
    ] {
        assert_eq!(
            hash(&output.runtime_files[&format!("data/tilesets/{name}_metatiles.bin")]),
            metatiles
        );
        assert_eq!(
            hash(&output.runtime_files[&format!("gfx/tilesets/{name}.2bpp")]),
            pixels
        );
    }
    assert_eq!(base.data.maps, output.data.maps);
    assert_eq!(base.data.field_moves, output.data.field_moves);
    assert!(append_winter_mixed_edges_art(&output, SPECIFICATION).is_err());
}

#[test]
#[ignore = "requires external pack with corrected winter art"]
fn mixed_winter_edges_reject_unsupported_cut_and_stale_or_occupied_source() {
    let base = fixture();
    let mut spec: Value = serde_json::from_str(SPECIFICATION).expect("contract");
    spec["targets"][0]["blocks"][0]["source_block"] = "0f".into();
    assert!(append_winter_mixed_edges_art(&base, &spec.to_string()).is_err());
    let mut spec: Value = serde_json::from_str(SPECIFICATION).expect("contract");
    spec["targets"][0]["blocks"][0]["collision"][0] = "CUT_TREE".into();
    assert!(append_winter_mixed_edges_art(&base, &spec.to_string()).is_err());
    let mut spec: Value = serde_json::from_str(SPECIFICATION).expect("contract");
    spec["targets"][0]["graphics_sha256"] = "00".repeat(32).into();
    assert!(append_winter_mixed_edges_art(&base, &spec.to_string()).is_err());
}

#[test]
#[ignore = "requires external pack with corrected winter art"]
fn mixed_winter_edges_residual_rejects_old_art_priority_collision_palette_and_program_changes() {
    let base = fixture();
    let output = append_winter_mixed_edges_art(&base, SPECIFICATION).expect("mixed winter art");
    let mut damaged = output.clone();
    damaged
        .runtime_files
        .get_mut("data/tilesets/mn_winter_outdoor_metatiles.bin")
        .expect("metatiles")[0] ^= 1;
    assert!(validate_residual(&base, &damaged).is_err());
    let mut damaged = output.clone();
    damaged
        .data
        .tilesets
        .get_mut("mn_winter_outdoor")
        .expect("winter art")
        .palette_map[0] ^= 1;
    assert!(validate_residual(&base, &damaged).is_err());
    let mut damaged = output.clone();
    damaged
        .data
        .tilesets
        .get_mut("mn_winter_outdoor")
        .expect("winter art")
        .collision
        .insert("3d".into(), vec!["FLOOR".into(); 4]);
    assert!(validate_residual(&base, &damaged).is_err());
    let mut damaged = output.clone();
    damaged
        .runtime_files
        .get_mut("gfx/tilesets/mn_winter_outdoor.2bpp")
        .expect("pixels")[224 * 16] ^= 1;
    assert!(validate_residual(&base, &damaged).is_err());
    let mut damaged = output.clone();
    damaged
        .data
        .maps
        .get_mut("MnGrandMarais")
        .expect("preserved town")
        .scripts
        .insert("MnUnexpectedWinterProgram".into(), serde_json::json!([]));
    assert!(validate_residual(&base, &damaged).is_err());
    let mut damaged = output.clone();
    damaged.report.manifests.push("unrelated".into());
    assert!(validate_residual(&base, &damaged).is_err());
}
