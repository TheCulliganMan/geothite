//! Prevent regional publication from reviving retired Superior geometry.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

// These are the canonical compact exterior dimensions, in metatiles. Checking
// the data rather than a filename/manifest also catches stale downstream packs.
pub(crate) fn require_compact_superior(pack: &CompiledGamePack) -> Result<()> {
    for (name, width, height) in [
        ("MnR11C01R00", 4, 24),
        ("MnR12C00R00", 32, 24),
        ("MnR12C00R01", 32, 8),
        ("MnMaraisCedarApproach", 20, 12),
        ("MnGrandMarais", 20, 19),
    ] {
        let map = pack
            .data
            .maps
            .get(name)
            .with_context(|| format!("compact Superior parent required: missing {name}"))?;
        ensure!(
            map.attributes.width == width && map.attributes.height == height,
            "compact Superior parent required: stale geometry in {name}; use a verified descendant of minnesota-superior-compact-v03.crystalpack"
        );
    }
    for retired in [
        "MnMaraisInlandNeck",
        "MnR12C01R00",
        "MnR12C01R01",
        "MnR12C00R02",
        "MnR12C01R02",
    ] {
        ensure!(
            !pack.data.maps.contains_key(retired),
            "compact Superior parent required: retired exterior {retired} remains"
        );
    }
    Ok(())
}
