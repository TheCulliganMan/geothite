use std::path::PathBuf;

use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{COLD, MANIFEST, MAP, WARM, restore_coastal_cedar_warmth, warm_block};
use crate::CompiledGamePack;
use crate::authored_route_layout::Layout;

#[test]
fn cedar_warmth_rejects_unreviewed_ice_and_snow_recipes() {
    for block in [0x85, 0x86, 0x87, 0x89, 0x8a, 0x100] {
        assert!(warm_block(block).is_err(), "unreviewed block {block:02x}");
    }
}

#[test]
fn cedar_warmth_source_alias_is_limited_to_the_compact_coastal_approach() {
    let mut source = json!({
        "schema_version": 2,
        "id": MAP,
        "dimensions_metatiles": [20, 12],
        "compiled_tileset": WARM,
        "shore_fitting": "explicit_source_edges",
        "reject_cliff_cuts": true,
        "layers": []
    });
    Layout::parse(&source.to_string()).expect("compact warm coastal source");
    source["dimensions_metatiles"] = json!([32, 19]);
    assert!(Layout::parse(&source.to_string()).is_err());
    source["id"] = json!("MN_R12");
    source["dimensions_metatiles"] = json!([32, 32]);
    assert!(Layout::parse(&source.to_string()).is_err());
}

fn fixture() -> CompiledGamePack {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_CEDAR_WARMTH_TEST_PACK")
            .expect("explicit current compact cold-Cedar parent"),
    );
    crate::read_verified_compiled_game_pack(path).expect("verified parent")
}

fn hash(value: &impl Serialize) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("serializable native field"))
    )
}

#[test]
#[ignore = "requires the external selected R20 cold-Cedar parent"]
fn cedar_warmth_preserves_every_other_native_field_and_inland_winter() {
    let base = fixture();
    assert!(
        crate::validate_compact_superior_geography(&base).is_err(),
        "the strengthened guard must detect the overlooked coastal snow alias"
    );
    let mut output =
        restore_coastal_cedar_warmth(&base).expect("collision-identical warm correction");
    crate::validate_compact_superior_geography(&output).expect("whole warm coast guard");
    let old = &base.data.maps[MAP];
    let revised = &output.data.maps[MAP];
    assert_eq!(revised.attributes.tileset_name, WARM);
    assert_eq!(revised.attributes.border_block, 0x05);
    assert_eq!(
        old.blocks
            .iter()
            .zip(&revised.blocks)
            .filter(|(a, b)| a != b)
            .count(),
        193
    );
    let cold = &base.data.tilesets[COLD].collision;
    let warm = &output.data.tilesets[WARM].collision;
    for (before, after) in old.blocks.iter().zip(&revised.blocks) {
        assert_eq!(
            cold[&format!("{before:02x}")],
            warm[&format!("{after:02x}")]
        );
    }
    let label = old.attributes.blocks_label.as_ref().expect("native label");
    output.data.maps.insert(MAP.into(), old.clone());
    output
        .data
        .map_attributes
        .insert(MAP.into(), old.attributes.clone());
    output
        .data
        .map_blocks
        .insert(label.clone(), base.data.map_blocks[label].clone());
    assert_eq!(
        hash(&output.data),
        hash(&base.data),
        "entire native data residual"
    );
    assert!(
        output.runtime_files == base.runtime_files,
        "shared/inland art must remain exact"
    );
    assert!(
        output.compiled_audio == base.compiled_audio,
        "audio payloads must remain exact"
    );
    assert_eq!(hash(&output.audio_manifest), hash(&base.audio_manifest));
    assert_eq!(output.audio_compression, base.audio_compression);
    assert_eq!(output.format_version, base.format_version);
    assert_eq!(output.report.manifests.pop().as_deref(), Some(MANIFEST));
    assert_eq!(hash(&output.report), hash(&base.report));
    output.identity.content_hash = base.identity.content_hash.clone();
    output.identity.runtime_modpack_id = base.identity.runtime_modpack_id.clone();
    assert_eq!(output.identity, base.identity);
}

#[test]
#[ignore = "requires the external selected R20 cold-Cedar parent"]
fn cedar_warmth_guard_rejects_a_reintroduced_cold_alias() {
    let base = fixture();
    let mut output = restore_coastal_cedar_warmth(&base).expect("warm correction");
    assert!(
        restore_coastal_cedar_warmth(&output).is_err(),
        "do not apply twice"
    );
    output
        .data
        .maps
        .get_mut(MAP)
        .expect("owned map")
        .attributes
        .tileset_name = COLD.into();
    output
        .data
        .map_attributes
        .get_mut(MAP)
        .expect("owned mirror")
        .tileset_name = COLD.into();
    assert!(crate::validate_compact_superior_geography(&output).is_err());
}
