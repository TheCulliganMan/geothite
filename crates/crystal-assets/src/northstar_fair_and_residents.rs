//! Align the ring-toss prize with the Pack and restore the Field edition's town replies.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::{authored_dialogue, authored_town_publication, northstar_town_responses};

/// Installs the canonical prize name and the reviewed town conversations where missing.
///
/// Returns an error if the fair prize, its text, or the original resident scripts
/// no longer match the expected installations.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    const FAIR: &str = "MnStateFairgrounds";
    let corn = pack
        .data
        .items
        .get("FAIR_CORN_STICK")
        .context("ring-toss prize")?;
    ensure!(
        corn.name == "SWEET CORN",
        "unexpected Pack name for the ring-toss prize"
    );
    let mut fair = pack
        .data
        .maps
        .get(FAIR)
        .context("State Fair midway")?
        .clone();
    for label in [
        "MnFairGameInviteText",
        "MnFairGameBagText",
        "MnFairGameWinText",
    ] {
        ensure!(
            fair.script_text_bodies.contains_key(label),
            "missing ring-toss page {label}"
        );
        authored_dialogue::install_canonical(&mut fair, label)?;
    }
    fair.script_text_bodies = crate::parse_script_text_bodies(FAIR, &fair.scripts)?;
    if !pack
        .report
        .manifests
        .iter()
        .any(|id| id == "northstar-field-review-v2")
    {
        northstar_town_responses::install(pack).context("restore the four town replies")?;
    }
    authored_town_publication::replace(pack, fair)
}
