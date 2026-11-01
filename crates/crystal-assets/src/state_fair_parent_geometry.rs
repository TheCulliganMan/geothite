//! Reject fair publication from the superseded northern map lineage.
use anyhow::{Result, ensure};

use crate::CompiledGamePack;

const DIMENSIONS: [(&str, [u16; 2]); 6] = [
    ("MnR11C01R00", [4, 24]),
    ("MnR11C01R01", [24, 24]),
    ("MnR12C00R00", [32, 24]),
    ("MnR12C00R01", [32, 8]),
    ("MnGrandMarais", [20, 19]),
    ("MnMaraisCedarApproach", [20, 12]),
];
const RETIRED: [&str; 5] = [
    "MnMaraisInlandNeck",
    "MnR12C01R00",
    "MnR12C01R01",
    "MnR12C00R02",
    "MnR12C01R02",
];

pub(super) fn validate(base: &CompiledGamePack) -> Result<()> {
    validate_dimensions(|name| {
        base.data()
            .maps
            .get(name)
            .map(|map| [map.attributes.width, map.attributes.height])
    })
}

fn validate_dimensions(lookup: impl Fn(&str) -> Option<[u16; 2]>) -> Result<()> {
    for (name, expected) in DIMENSIONS {
        ensure!(
            lookup(name) == Some(expected),
            "State Fair parent has stale northern geometry at {name}; require the compact \
             Superior lineage (minnesota-superior-compact-v03 or its preserving descendants)"
        );
    }
    for name in RETIRED {
        ensure!(
            lookup(name).is_none(),
            "State Fair parent retains retired northern map {name}; require compact Superior lineage"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DIMENSIONS, RETIRED, validate_dimensions};

    fn canonical(name: &str) -> Option<[u16; 2]> {
        DIMENSIONS
            .iter()
            .find_map(|(id, size)| (*id == name).then_some(*size))
    }

    #[test]
    fn compact_parent_geometry_is_accepted() {
        validate_dimensions(canonical).expect("canonical compact parent");
    }

    #[test]
    fn oversized_or_missing_northern_maps_are_rejected() {
        for (changed, _) in DIMENSIONS {
            for replacement in [None, Some([64, 48])] {
                assert!(
                    validate_dimensions(|name| {
                        if name == changed {
                            replacement
                        } else {
                            canonical(name)
                        }
                    })
                    .is_err(),
                    "reject stale {changed}"
                );
            }
        }
    }

    #[test]
    fn retired_northern_maps_cannot_be_reintroduced() {
        for retired in RETIRED {
            assert!(
                validate_dimensions(|name| {
                    if name == retired {
                        Some([32, 24])
                    } else {
                        canonical(name)
                    }
                })
                .is_err(),
                "reject retired {retired}"
            );
        }
    }
}
