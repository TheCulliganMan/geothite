//! Stable Staples binding over the shared active-roster approach publisher.
use anyhow::{Result, bail};

use crate::authored_regional_approach::{self, FishingBinding};
use crate::{CompiledGamePack, MapModule};

pub(crate) const FISHING: &str = "FISHGROUP_MN_STAPLES_R15";

pub(crate) fn install(pack: &mut CompiledGamePack, road: &mut MapModule) -> Result<()> {
    let expected_group = match road.attributes.fishing_group.as_deref() {
        Some("FISHGROUP_QWILFISH") => "FISHGROUP_QWILFISH",
        Some("FISHGROUP_MN_QWILFISH") => "FISHGROUP_MN_QWILFISH",
        _ => bail!("Staples requires the original or regional R15 lake fishing group"),
    };
    authored_regional_approach::install(
        pack,
        road,
        FishingBinding {
            expected_group,
            group: FISHING,
            time_prefix: "MN_STAPLES_R15_TIME_",
            swarm_rule: Some("FISHSWARM_MN_STAPLES_R15"),
        },
    )
}
