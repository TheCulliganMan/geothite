//! Exact additive art, inherited native behavior, isolation and composition.
use std::path::PathBuf;
use std::sync::OnceLock;

use super::{Contract, build_winter_gunflint_alias_pack};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/inland-winter-r12-r13/gunflint-alias-v01/native.contract.json"
);

fn fixture() -> &'static CompiledGamePack {
    static BASE: OnceLock<CompiledGamePack> = OnceLock::new();
    BASE.get_or_init(|| {
        crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_WINTER_GUNFLINT_ALIAS_TEST_PACK")
                .expect("completed parent retaining frozen cold art and Gunflint behavior"),
        ))
        .expect("verified actual alias parent")
    })
}

#[test]
fn winter_gunflint_alias_frozen_contract_is_exact_and_additive() {
    assert_eq!(super::hash(CONTRACT.as_bytes()), super::CONTRACT_SHA);
    let c: Contract = serde_json::from_str(CONTRACT).expect("frozen contract");
    assert_eq!(c.source_files_sha256.len(), 6);
    assert_eq!((c.expected_blocks, c.expected_glyphs), (73, 256));
    assert_eq!(c.behavior_source, "mn_gunflint");
    assert!(
        c.source_files_sha256
            .keys()
            .all(|p| p.contains(super::SOURCE))
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_GUNFLINT_ALIAS_TEST_PACK"]
fn winter_gunflint_alias_preserves_all_art_prefixes_maps_and_native_cut() {
    let base = fixture();
    let output = build_winter_gunflint_alias_pack(base, CONTRACT).expect("exact alias");
    let c: Contract = serde_json::from_str(CONTRACT).expect("contract");
    super::preserve(base, &output, &c).expect("strict full inverse residual");
    assert_eq!(output.data.maps, base.data.maps);
    assert_eq!(output.data.map_blocks, base.data.map_blocks);
    assert_eq!(output.data.map_attributes, base.data.map_attributes);
    assert_eq!(
        output.data.tilesets[super::TARGET],
        base.data.tilesets[super::SOURCE]
    );
    let native = &base.data.field_moves.cut.replacements[super::BEHAVIOR];
    assert_eq!(
        output.data.field_moves.cut.replacements[super::TARGET],
        *native
    );
    assert_eq!(native[&15].replacement_block_id, 23);
    for (path, expected) in &c.source_files_sha256 {
        assert_eq!(
            super::hash(&output.runtime_files[&super::target_path(path)]),
            *expected
        );
    }
    assert_ne!(base.identity, output.identity);
    assert!(build_winter_gunflint_alias_pack(&output, CONTRACT).is_err());
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_GUNFLINT_ALIAS_TEST_PACK"]
fn winter_gunflint_alias_rejects_stale_sources_identity_contract_and_occupied_files() {
    let base = fixture();
    assert!(build_winter_gunflint_alias_pack(base, &format!("{CONTRACT}\n")).is_err());
    for case in 0..5 {
        let mut bad = base.clone();
        match case {
            0 => bad.identity.content_hash = "stale".into(),
            1 => {
                bad.runtime_files
                    .get_mut("gfx/tilesets/mn_winter_outdoor.2bpp")
                    .expect("source art")[0] ^= 1;
            }
            2 => {
                bad.data
                    .field_moves
                    .cut
                    .replacements
                    .get_mut(super::BEHAVIOR)
                    .expect("Cut")
                    .get_mut(&15)
                    .expect("tree")
                    .replacement_block_id = 1;
            }
            3 => {
                bad.runtime_files.insert(
                    format!("data/tilesets/{}_foregrounds.json", super::TARGET),
                    vec![],
                );
            }
            _ => {
                bad.data.tilesets.insert(
                    super::TARGET.into(),
                    bad.data.tilesets[super::SOURCE].clone(),
                );
            }
        }
        bad.identity = super::identity(&bad).expect("counterexample identity");
        if case == 0 {
            bad.identity.content_hash = "stale".into();
        }
        assert!(
            build_winter_gunflint_alias_pack(&bad, CONTRACT).is_err(),
            "case {case}"
        );
    }
    assert_eq!(
        fixture(),
        base,
        "failed builds leave borrowed parent intact"
    );
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_GUNFLINT_ALIAS_TEST_PACK"]
fn winter_gunflint_alias_inverse_rejects_extra_maps_programs_art_behavior_report_and_identity() {
    let base = fixture();
    let output = build_winter_gunflint_alias_pack(base, CONTRACT).expect("alias");
    let c: Contract = serde_json::from_str(CONTRACT).expect("contract");
    for case in 0..7 {
        let mut bad = output.clone();
        match case {
            0 => {
                bad.data
                    .maps
                    .get_mut("MnR12C00R00")
                    .expect("native map")
                    .blocks[0] ^= 1
            }
            1 => {
                bad.data
                    .maps
                    .get_mut("MnR12C00R00")
                    .expect("native map")
                    .scripts
                    .insert("Unapproved".into(), serde_json::json!([]));
            }
            2 => {
                bad.runtime_files
                    .get_mut("gfx/tilesets/mn_winter_gunflint.2bpp")
                    .expect("new art")[0] ^= 1
            }
            3 => {
                bad.data
                    .field_moves
                    .cut
                    .replacements
                    .get_mut(super::TARGET)
                    .expect("new Cut")
                    .get_mut(&15)
                    .expect("tree")
                    .replacement_block_id = 1;
            }
            4 => bad.report.manifests.push("unapproved-report".into()),
            5 => {
                bad.runtime_files
                    .get_mut("gfx/tilesets/mn_winter_outdoor.2bpp")
                    .expect("old art")[0] ^= 1
            }
            _ => bad.identity.content_hash = "stale".into(),
        }
        if case != 6 {
            bad.identity = super::identity(&bad).expect("counterexample identity");
        }
        assert!(
            super::preserve(base, &bad, &c).is_err(),
            "residual case {case}"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINTER_GUNFLINT_ALIAS_TEST_PACK"]
fn winter_gunflint_alias_composes_without_resetting_valid_unrelated_parent_content() {
    let mut base = fixture().clone();
    let map = base
        .data
        .maps
        .get_mut("MnMinneapolis")
        .expect("unrelated city");
    let collision = &base.data.tilesets[&map.attributes.tileset_name].collision;
    let old = map.blocks[0];
    let permissions = &collision[&format!("{old:02x}")];
    map.blocks[0] = collision
        .iter()
        .find_map(|(key, value)| {
            let block = u16::from_str_radix(key, 16).ok()?;
            (block != old && block <= 255 && value == permissions).then_some(block)
        })
        .expect("another valid recipe with identical collision");
    let label = map
        .attributes
        .blocks_label
        .clone()
        .expect("city block payload");
    let blocks = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<Result<Vec<_>, _>>()
        .expect("native byte blocks");
    base.data
        .map_blocks
        .insert(label, crate::encode_base64_bytes(&blocks));
    base.report
        .manifests
        .push("unrelated-approved-parent-revision".into());
    base.runtime_files.insert(
        "data/unrelated-approved-parent.txt".into(),
        b"retain exact parent bytes".to_vec(),
    );
    base.identity = super::identity(&base).expect("composed parent identity");
    crate::verify_compiled_game_pack_for_runtime(&base).expect("valid unrelated parent");
    let output = build_winter_gunflint_alias_pack(&base, CONTRACT).expect("composable alias");
    let c: Contract = serde_json::from_str(CONTRACT).expect("contract");
    super::preserve(&base, &output, &c).expect("entire unrelated actual parent retained");
    assert_eq!(
        output.runtime_files["data/unrelated-approved-parent.txt"],
        b"retain exact parent bytes"
    );
}
