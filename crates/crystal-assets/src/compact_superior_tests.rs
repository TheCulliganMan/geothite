//! Frozen compact source and exact preservation regression coverage.
use std::path::PathBuf;

use super::{Contract, build_compact_superior_pack, hash};
use crate::CompiledGamePack;

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/superior-gunflint-quality-v01/native.compact.contract.json"
);
const SHA: &str = "fec56ec1141f91bb3d3ae7d371990ba8bf0b4ea22cc6505e5b0e58c3a636e135";
const COAST: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/coast-quality-v01/source-v04/compact-bend.contract.json"
);
const R12_LAYOUT: &str = include_str!(
    "../../../planning/minnesota/revisions/gunflint-quality-v01/source-v10/MN_R12.candidate.layout.json"
);
const R12_DOSSIER: &str = include_str!(
    "../../../planning/minnesota/revisions/gunflint-quality-v01/source-v10/MN_R12.candidate.json"
);
const CEDAR: &str = include_str!(
    "../../../planning/minnesota/revisions/superior-gunflint-quality-v01/cedar-source-v02/contract.json"
);

fn build(base: &CompiledGamePack) -> anyhow::Result<CompiledGamePack> {
    build_compact_superior_pack(base, CONTRACT, SHA, COAST, R12_LAYOUT, R12_DOSSIER, CEDAR)
}
#[test]
fn compact_superior_reviewed_source_hashes_and_scope_are_exact() {
    assert_eq!(hash(CONTRACT.as_bytes()), SHA);
    let plan: Contract = serde_json::from_str(CONTRACT).expect("frozen compact contract");
    assert_eq!(plan.module_preimages.len(), 14);
    for (key, text) in [
        ("coast", COAST),
        ("r12_layout", R12_LAYOUT),
        ("r12_dossier", R12_DOSSIER),
        ("cedar", CEDAR),
    ] {
        assert_eq!(plan.source_sha256[key], hash(text.as_bytes()));
    }
}
#[test]
#[ignore = "requires external completed parent in CRYSTAL_COMPACT_SUPERIOR_PARENT_PACK"]
fn compact_superior_publishes689_maps_and_rejects_stale_owned_inputs() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_COMPACT_SUPERIOR_PARENT_PACK")
            .expect("explicit completed parent"),
    );
    let base = crate::read_verified_compiled_game_pack(path).expect("verified actual parent");
    let revised = build(&base).expect("bounded compact revision");
    assert_eq!(revised.data.maps.len(), 689);
    assert_eq!(base.data.maps.len(), 694);
    let mut changed = base.clone();
    changed
        .data
        .maps
        .get_mut(super::UPPER)
        .expect("upper chunk")
        .blocks[0] ^= 1;
    assert!(build(&changed).is_err());
    assert!(
        build_compact_superior_pack(
            &base,
            CONTRACT,
            SHA,
            COAST,
            &format!("{R12_LAYOUT} "),
            R12_DOSSIER,
            CEDAR
        )
        .is_err()
    );
    let mut corrupted = revised.clone();
    corrupted
        .data
        .maps
        .get_mut("NewBarkTown")
        .expect("unrelated map")
        .blocks[0] ^= 1;
    assert!(
        super::validation::validate(
            &base,
            &corrupted,
            &revised
                .data
                .maps
                .iter()
                .filter(|(n, _)| super::OWNED.contains(&n.as_str()))
                .map(|(n, m)| (n.clone(), m.clone()))
                .collect()
        )
        .is_err()
    );
}
