//! Keep interior publications on the shared compact-region validation path.
use anyhow::{Context, Result};

use crate::CompiledGamePack;

pub(crate) fn require_compact_coast(pack: &CompiledGamePack) -> Result<()> {
    crate::validate_compact_superior_geography(pack).context(
        "interior publication refuses obsolete North Shore/Gunflint geometry; rebase onto the verified compact coast first",
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{build_minnesota_interior_layout_modpack, read_verified_compiled_game_pack};

    #[test]
    #[ignore = "requires CRYSTAL_COMPACT_SUPERIOR_TEST_PACK"]
    fn interior_lineage_accepts_compact_and_rejects_obsolete_dimensions_and_chunks() {
        let compact = read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_COMPACT_SUPERIOR_TEST_PACK").expect("verified compact pack"),
        ))
        .expect("strict compact input");
        super::require_compact_coast(&compact).expect("canonical geometry");
        let mut stale = compact.clone();
        stale
            .data
            .maps
            .get_mut("MnR11C01R00")
            .expect("coast")
            .attributes
            .width = 24;
        assert!(super::require_compact_coast(&stale).is_err());
        let source =
            include_str!("../../../planning/minnesota/interior-life/readable-details.json");
        assert!(
            build_minnesota_interior_layout_modpack(&stale, source).is_err(),
            "the publication entry point rejects the old coast before touching rooms"
        );
        let mut stale = compact.clone();
        let old = stale.data.maps["MnR12C00R00"].clone();
        stale.data.maps.insert("MnR12C01R00".into(), old);
        assert!(super::require_compact_coast(&stale).is_err());
    }
}
