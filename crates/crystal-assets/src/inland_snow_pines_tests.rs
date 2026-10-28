use std::path::PathBuf;
use std::sync::OnceLock;

use super::{Contract, RECIPE, TILESET, art, build_inland_snow_pines_pack, preserve};
use crate::{CompiledGamePack, read_verified_compiled_game_pack};

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/pine-art-v01/native.contract.json"
);

fn base() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        let path = PathBuf::from(
            std::env::var_os("CRYSTAL_INLAND_PINES_TEST_PACK").expect("completed pine art parent"),
        );
        read_verified_compiled_game_pack(path).expect("verified native pine parent")
    })
}

#[test]
#[ignore = "requires actual native parent in CRYSTAL_INLAND_PINES_TEST_PACK"]
fn inland_pines_native_append_preserves_all_maps_and_old_art() {
    let output = build_inland_snow_pines_pack(base(), CONTRACT).expect("native pine append");
    assert_eq!(output.data.maps, base().data.maps);
    assert_eq!(output.data.map_scripts, base().data.map_scripts);
    assert_eq!(output.audio_manifest, base().audio_manifest);
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let before = &base().runtime_files[&path];
    let after = &output.runtime_files[&path];
    assert_eq!(&after[..before.len()], before);
    assert_eq!(&after[before.len()..], RECIPE);
    let path = format!("gfx/tilesets/{TILESET}.2bpp");
    assert_eq!(
        &output.runtime_files[&path][..254 * 16],
        &base().runtime_files[&path][..254 * 16]
    );
    assert_eq!(output.data.tilesets[TILESET].collision["8b"], ["WALL"; 4]);
    assert_ne!(output.identity, base().identity);
}

#[test]
#[ignore = "requires actual native parent in CRYSTAL_INLAND_PINES_TEST_PACK"]
fn inland_pines_reject_stale_private_slots_and_contract() {
    assert!(build_inland_snow_pines_pack(base(), &format!("{CONTRACT} ")).is_err());
    for path in [
        format!("gfx/tilesets/{TILESET}.2bpp"),
        "gfx/tilesets/johto.2bpp".into(),
    ] {
        let mut bad = base().clone();
        bad.runtime_files.get_mut(&path).expect("source glyphs")[0] ^= 1;
        bad.identity = super::identity(&bad).expect("fresh damaged identity");
        assert!(build_inland_snow_pines_pack(&bad, CONTRACT).is_err());
    }
}

#[test]
#[ignore = "requires actual native parent in CRYSTAL_INLAND_PINES_TEST_PACK"]
fn inland_pines_residual_rejects_old_glyph_recipe_palette_map_and_program_changes() {
    let output = build_inland_snow_pines_pack(base(), CONTRACT).expect("reviewed append");
    let plan: Contract = serde_json::from_str(CONTRACT).expect("frozen pine plan");
    let (definition, files) = art(base(), &plan).expect("bounded art");
    for path in [
        format!("gfx/tilesets/{TILESET}.2bpp"),
        format!("data/tilesets/{TILESET}_metatiles.bin"),
    ] {
        let mut bad = output.clone();
        bad.runtime_files.get_mut(&path).expect("owned art")[0] ^= 1;
        assert!(preserve(base(), &bad, &definition, &files).is_err());
    }
    let mut bad = output.clone();
    bad.data
        .tilesets
        .get_mut(TILESET)
        .expect("owned definition")
        .palette_map[0] ^= 1;
    assert!(preserve(base(), &bad, &definition, &files).is_err());
    let mut bad = output.clone();
    bad.data
        .maps
        .get_mut("MnGrandMarais")
        .expect("unrelated town")
        .blocks[0] ^= 1;
    assert!(preserve(base(), &bad, &definition, &files).is_err());
    let mut bad = output.clone();
    let script = bad
        .data
        .map_scripts
        .values_mut()
        .next()
        .expect("native programs");
    *script = serde_json::json!([]);
    assert!(preserve(base(), &bad, &definition, &files).is_err());
}
