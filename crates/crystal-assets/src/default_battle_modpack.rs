use anyhow::Result;

use crate::{CompiledGamePack, build_competitive_battle_modpack, build_wild_challenge_modpack};

/// Shared default for native, browser, and text clients. Compose only battle
/// rules: existing encounter placement, story, presentation, and audio survive.
pub fn build_default_battle_modpack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    build_wild_challenge_modpack(&build_competitive_battle_modpack(base)?)
}
