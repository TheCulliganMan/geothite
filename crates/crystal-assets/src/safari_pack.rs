//! Verified optional expedition configuration over an existing native pack.

use anyhow::{Result, ensure};

use crate::{CompiledGamePack, SafariZone};

impl CompiledGamePack {
    /// Bind capture expeditions to validated native habitats and ranger actors.
    ///
    /// Derives a fresh identity while preserving terrain, executable scripts,
    /// source art, audio and recovery. Replaces only the preserve's obsolete
    /// closure text with native permit/check-in directions. This is a
    /// publication primitive; callers must separately verify production input
    /// and presentation before selecting the result as the playable regional
    /// pack.
    ///
    /// # Errors
    /// Rejects an invalid pack, already configured expedition, missing native
    /// references, illegal wildlife tables, changed/shared closure-text anchors,
    /// or failed resulting pack validation.
    pub fn with_safari_zone(&self, zone: SafariZone) -> Result<Self> {
        crate::verify_compiled_game_pack_for_runtime(self)?;
        ensure!(
            self.data.safari_zone.is_none(),
            "Safari is already configured"
        );
        zone.validate(&self.data)?;
        let mut pack = self.clone();
        crate::safari_guidance::install(&mut pack, &zone)?;
        pack.data.safari_zone = Some(zone);
        pack.report
            .manifests
            .push("native-safari-expedition-v2".into());
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        crate::verify_compiled_game_pack_for_runtime(&pack)?;
        Ok(pack)
    }
}
