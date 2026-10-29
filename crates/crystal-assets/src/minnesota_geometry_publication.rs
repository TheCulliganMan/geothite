//! Prevent historical rectangular coast geometry from becoming a new pack.
use std::collections::BTreeMap;

use anyhow::{Result, ensure};

use crate::CompiledGamePack;

const CURRENT: [(&str, [u16; 2]); 5] = [
    ("MnR11C01R00", [4, 24]),
    ("MnR12C00R00", [32, 24]),
    ("MnR12C00R01", [32, 8]),
    ("MnMaraisCedarApproach", [20, 12]),
    ("MnGrandMarais", [20, 19]),
];
const RETIRED: [&str; 5] = [
    "MnMaraisInlandNeck",
    "MnR12C01R00",
    "MnR12C01R01",
    "MnR12C00R02",
    "MnR12C01R02",
];

/// Historical packs remain readable, but cannot be republished as current work.
pub(crate) fn validate(pack: &CompiledGamePack) -> Result<()> {
    let dimensions = pack
        .data
        .maps
        .iter()
        .map(|(name, map)| (name.as_str(), [map.attributes.width, map.attributes.height]))
        .collect();
    validate_dimensions(&dimensions)
}

fn validate_dimensions(maps: &BTreeMap<&str, [u16; 2]>) -> Result<()> {
    for name in RETIRED {
        ensure!(
            !maps.contains_key(name),
            "retired Minnesota exterior {name} cannot be published; retain historical packs read-only and build from the current compact lineage"
        );
    }
    if !maps
        .keys()
        .any(|name| name.starts_with("MnR11C") || name.starts_with("MnR12C"))
    {
        return Ok(());
    }
    for (name, dimensions) in CURRENT {
        ensure!(
            maps.get(name) == Some(&dimensions),
            "obsolete or incomplete Minnesota coastal geometry: {name} must be {dimensions:?}; use build_compact_superior with its frozen inputs, or a verified descendant of world.json active_local_review_revision; rectangular standalone North Shore/Gunflint publication is retired"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CURRENT, RETIRED, validate_dimensions};
    use std::collections::BTreeMap;

    #[test]
    fn publication_keeps_current_descendants_and_non_minnesota_packs() {
        let mut current = BTreeMap::from(CURRENT);
        current.insert("MnR10Cave1F", [16, 12]);
        assert!(validate_dimensions(&current).is_ok());
        assert!(validate_dimensions(&BTreeMap::from([("NewBarkTown", [10, 9])])).is_ok());
    }

    #[test]
    fn publication_rejects_each_obsolete_footprint_and_retired_exterior() {
        for (name, _) in CURRENT {
            let mut stale = BTreeMap::from(CURRENT);
            stale.insert(name, [64, 56]);
            assert!(validate_dimensions(&stale).is_err(), "stale {name}");
            stale.remove(name);
            assert!(validate_dimensions(&stale).is_err(), "missing {name}");
        }
        for name in RETIRED {
            assert!(validate_dimensions(&BTreeMap::from([(name, [32, 24])])).is_err());
            let mut stale = BTreeMap::from(CURRENT);
            stale.insert(name, [32, 24]);
            assert!(validate_dimensions(&stale).is_err(), "retired {name}");
        }
    }

    #[test]
    #[ignore = "requires the actual compact descendant and historical winter parent"]
    fn publication_checks_actual_current_and_stale_packs_without_writing() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content-packs")
            .canonicalize()
            .expect("local ignored content-packs directory");
        let current = crate::read_verified_compiled_game_pack(
            root.join("minnesota-r10-rock-gallery-v01.crystalpack"),
        )
        .expect("verified current R10 child");
        super::validate(&current).expect("legitimate current work retained");
        let stale = crate::read_verified_compiled_game_pack(
            root.join("minnesota-superior-inland-winter-v01.crystalpack"),
        )
        .expect("read-only historical parent");
        assert!(super::validate(&stale).is_err());
    }
}
