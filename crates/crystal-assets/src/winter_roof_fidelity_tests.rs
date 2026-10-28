use std::path::PathBuf;

use super::{CELLS, MAP, TILESET, build_winter_roof_fidelity_pack, hash, validation};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/regions/ice/roof-fidelity-v01/roof-fidelity.contract.json"
);

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_WINTER_ROOF_TEST_PACK")
            .expect("published aa1aa251 winter parent"),
    )
    .canonicalize()
    .expect("completed winter pack path");
    crate::read_verified_compiled_game_pack(path).expect("verified winter parent")
}

#[test]
#[ignore = "requires exact external northern-winter-v01 parent"]
fn winter_roof_append_changes_only_two_canopies_and_preserves_native_wire_and_gameplay() {
    let base = fixture();
    let output = build_winter_roof_fidelity_pack(&base, CONTRACT).expect("reviewed roof append");
    validation::preserve(&base, &output).expect("whole pack residual and uncached proof");
    assert_eq!(output.data.maps.len(), 683);
    assert_eq!(
        hash(&output.runtime_files["data/tilesets/mn_winter_north_shore_metatiles.bin"]),
        "d0dec8542592547471d107ccbc7f0e5b110e18d55277d7a141184e2e67a936d3"
    );
    assert_eq!(
        hash(&output.runtime_files["gfx/tilesets/mn_winter_north_shore.2bpp"]),
        "f1445a0c889bb3e34f98de2796859cfc91ec378dac4fd8620efcd4360d673c56"
    );
    for (index, before, after) in CELLS {
        assert_eq!(base.data.maps[MAP].blocks[index], before);
        assert_eq!(output.data.maps[MAP].blocks[index], after);
        assert_eq!(
            base.data.tilesets[TILESET].collision[&format!("{before:02x}")],
            output.data.tilesets[TILESET].collision[&format!("{after:02x}")]
        );
    }
    assert!(build_winter_roof_fidelity_pack(&output, CONTRACT).is_err());
}

#[test]
#[ignore = "requires exact external northern-winter-v01 parent"]
fn winter_roof_append_rejects_stale_native_source_occupied_glyph_and_unreviewed_contract() {
    let base = fixture();
    let mut changed = base.clone();
    changed.data.maps.get_mut(MAP).expect("map").blocks[299] = 5;
    assert!(build_winter_roof_fidelity_pack(&changed, CONTRACT).is_err());
    let mut changed = base.clone();
    changed
        .runtime_files
        .get_mut("gfx/tilesets/mn_winter_north_shore.2bpp")
        .expect("pixels")[250 * 16] = 1;
    assert!(build_winter_roof_fidelity_pack(&changed, CONTRACT).is_err());
    assert!(
        build_winter_roof_fidelity_pack(
            &base,
            &CONTRACT.replace("\"after\": 137", "\"after\": 138")
        )
        .is_err()
    );
}

#[test]
#[ignore = "requires exact external northern-winter-v01 parent"]
fn winter_roof_residual_rejects_other_art_geometry_programs_and_metadata_mutations() {
    let base = fixture();
    let output = build_winter_roof_fidelity_pack(&base, CONTRACT).expect("roof append");
    let mut changed = output.clone();
    changed.data.maps.get_mut(MAP).expect("map").blocks[0] ^= 1;
    assert!(validation::preserve(&base, &changed).is_err());
    let mut changed = output.clone();
    changed
        .data
        .maps
        .get_mut(MAP)
        .expect("map")
        .scripts
        .insert("UnrelatedRoofScript".into(), serde_json::json!([]));
    assert!(validation::preserve(&base, &changed).is_err());
    let mut changed = output.clone();
    changed
        .data
        .tilesets
        .get_mut(TILESET)
        .expect("tileset")
        .palette_map[0] ^= 1;
    assert!(validation::preserve(&base, &changed).is_err());
    let mut changed = output.clone();
    changed
        .runtime_files
        .get_mut("gfx/tilesets/mn_winter_north_shore.2bpp")
        .expect("pixels")[0] ^= 1;
    assert!(validation::preserve(&base, &changed).is_err());
    let mut changed = output.clone();
    changed
        .report
        .manifests
        .push("unrelated-roof-manifest".into());
    assert!(validation::preserve(&base, &changed).is_err());
}
