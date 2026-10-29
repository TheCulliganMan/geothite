//! Verified optional expedition configuration over an existing native pack.

use anyhow::{Result, ensure};

use crate::{CompiledGamePack, SafariZone};

impl CompiledGamePack {
    /// Bind capture expeditions to validated native habitats and ranger actors.
    ///
    /// Derives a fresh identity while preserving existing terrain, scripts,
    /// source art, audio and recovery. This is a publication primitive; callers
    /// must separately verify production input and presentation before selecting
    /// the result as the playable regional pack.
    ///
    /// # Errors
    /// Rejects an invalid pack, already configured expedition, missing native
    /// references, illegal wildlife tables or failed resulting pack validation.
    pub fn with_safari_zone(&self, zone: SafariZone) -> Result<Self> {
        crate::verify_compiled_game_pack_for_runtime(self)?;
        ensure!(
            self.data.safari_zone.is_none(),
            "Safari is already configured"
        );
        zone.validate(&self.data)?;
        let mut pack = self.clone();
        pack.data.safari_zone = Some(zone);
        pack.report
            .manifests
            .push("native-safari-expedition-v1".into());
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
