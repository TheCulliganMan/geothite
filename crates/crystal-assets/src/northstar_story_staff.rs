//! Captives can leave once the acted headquarters rescue makes the route safe.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get("MnIceTransfer")
        .context("installed headquarters transfer room")?
        .clone();
    let tess = map
        .objects
        .iter_mut()
        .find(|actor| actor.object_identifier.as_deref() == Some("MN_ICE_HQ_TESS"))
        .context("existing captive Tess")?;
    ensure!(
        tess.script == "MN_ICE_HQ_TESS"
            && (tess.x, tess.y) == (25, 5)
            && matches!(
                tess.event_flag.as_str(),
                "-1" | "EVENT_MN_ICE_CAPTIVE_RESCUED"
            ),
        "review the existing captive before changing her departure"
    );
    // The primary release sets this existing event only after the player has
    // cleared the guards, stopped the holding circuit and let Ellis walk out.
    // Tess remains present before that acknowledgment and can leave offscreen
    // afterwards. No new flag, reward or save migration is introduced.
    tess.event_flag = "EVENT_MN_ICE_CAPTIVE_RESCUED".into();
    crate::northstar_story_revision::publish(pack, map)
}
