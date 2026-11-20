//! Ada shares the first slice with her partner at the State Fair.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(crate) fn give_ada_pie(pack: &mut CompiledGamePack) -> Result<()> {
    const PIE: &str = "MN_BLUEBERRY_SLICE";
    let item = pack.data.items.get(PIE).context("blueberry pie slice")?;
    ensure!(
        item.held_effect == "HELD_BERRY" && item.parameter == 50,
        "Ada needs a working 50 HP held pie"
    );
    let lead = pack
        .data
        .trainers
        .trainers
        .get_mut("MN_FAIR_BAKER_TRAINER")
        .and_then(|trainer| trainer.party.first_mut())
        .context("the State Fair baker's lead")?;
    ensure!(
        lead.species == "DOUGHLIN" && lead.item.is_none(),
        "expected Ada's empty-handed Doughlin"
    );
    lead.item = Some(PIE.into());
    Ok(())
}
