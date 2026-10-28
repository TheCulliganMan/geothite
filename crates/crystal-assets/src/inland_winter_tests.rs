//! Exact terrain, bounded collision changes and complete parent preservation.
use std::path::PathBuf;

use super::{Contract, build_inland_winter_pack};

const CONTRACT: &str = include_str!(
    "../../../planning/minnesota/revisions/north-shore-ely-geography-v01/inland-winter-r12-r13/source-v02/native.review.contract.json"
);

#[test]
fn inland_winter_frozen_4586_cells_and40_ice_quadrants_are_exact() {
    assert_eq!(super::hash(CONTRACT.as_bytes()), super::CONTRACT_SHA);
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen inland review");
    assert_eq!(contract.modules.len(), 9);
    assert_eq!(
        contract
            .modules
            .iter()
            .map(|m| m.cells.len())
            .sum::<usize>(),
        4586
    );
    let mut ice = 0;
    for module in &contract.modules {
        let mut seen = std::collections::BTreeSet::new();
        for cell in &module.cells {
            assert!(seen.insert(cell.metatile));
            for (before, after) in cell.before_quadrants.iter().zip(&cell.after_quadrants) {
                if before != after {
                    assert_eq!((before.as_str(), after.as_str()), ("FLOOR", "ICE"));
                    ice += 1;
                }
                if before == "WATER" {
                    assert_eq!(after, "WATER", "native lakes must stay water");
                }
            }
        }
    }
    assert_eq!(ice, 40);
}

#[test]
#[ignore = "requires final geography parent in CRYSTAL_INLAND_WINTER_TEST_PACK"]
fn inland_winter_publishes_exact_cells_mirrors_programs_and_preserves_parent() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_INLAND_WINTER_TEST_PACK")
            .expect("completed final geography parent"),
    );
    let base = crate::read_verified_compiled_game_pack(path).expect("verified geography parent");
    let output = build_inland_winter_pack(&base, CONTRACT).expect("guarded winter overlay");
    let contract: Contract = serde_json::from_str(CONTRACT).expect("frozen winter");
    super::preserve(&base, &output, &contract).expect("complete parent and native mirrors");
    assert_eq!(
        base.data
            .maps
            .iter()
            .flat_map(|(name, m)| m.blocks.iter().zip(&output.data.maps[name].blocks))
            .filter(|(a, b)| a != b)
            .count(),
        4586
    );
    let mut stale = base.clone();
    stale
        .data
        .maps
        .get_mut("MnR12C01R01")
        .expect("R12 seam")
        .attributes
        .connections
        .iter_mut()
        .find(|c| c.direction == "east")
        .expect("east")
        .offset = 1;
    stale.identity = super::identity(&stale).expect("identity for negative fixture");
    assert!(
        build_inland_winter_pack(&stale, CONTRACT).is_err(),
        "offset preimage must remain exact"
    );
    assert!(
        build_inland_winter_pack(&output, CONTRACT).is_err(),
        "republishing must fail preimages"
    );
}
