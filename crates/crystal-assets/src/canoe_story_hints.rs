//! Replace obsolete regional HM-only shore directions with canoe instructions.
use anyhow::{Context, Result};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for (map_name, label, lines) in [
        (
            "MnBemidji",
            "MN_BEMIDJI_DOCKText",
            &["LOW LAUNCH", "Paddle from here."][..],
        ),
        (
            "MnBoundarySafariC00R01",
            "BWS_SIGN_02Text",
            &["REED PORTAGE", "LAND / PORTAGE", "THEN PADDLE ON."][..],
        ),
        (
            "MnBoundaryWaters",
            "MN_BOUNDARY_DOCKHANDText",
            &[
                "PUBLIC LANDING",
                "Launch your CANOE",
                "at the north end.",
                "Keep off rails.",
            ][..],
        ),
        (
            "MnBrainerdOutfitter",
            "MN_BRAINERD_CAMP_HELPERText",
            &[
                "Carry a CANOE for",
                "the lakes. ST PAUL",
                "CANOE SHOP can",
                "fit one for you.",
            ][..],
        ),
        (
            "MnDuluth",
            "MnDuluth_LAUNCH_SIGNText",
            &[
                "LOW HARBOR LAUNCH",
                "Paddle from shore.",
                "Stay inside rocks.",
            ][..],
        ),
        (
            "MnGrandMaraisFisherHouse",
            "MnGrandMaraisFisherHouseFisherText",
            &[
                "Near the falls, a",
                "current vanished.",
                "Then it returned.",
                "I felt a pulse.",
                "Carry a CANOE when",
                "taking to water.",
            ][..],
        ),
        (
            "MnR10C01R00",
            "MN_R10_SURF_SIGNText",
            &[
                "PUBLIC LANDING",
                "Walk to the water",
                "to launch a CANOE.",
                "Keep off rocks.",
            ][..],
        ),
        (
            "MnR18C00R00",
            "MN_R18_SURF_SIGNText",
            &[
                "PUBLIC LAUNCH",
                "Walk to the water",
                "to launch a CANOE.",
                "Rocks mark limits.",
            ][..],
        ),
    ] {
        let mut map = pack
            .data
            .maps
            .get(map_name)
            .with_context(|| format!("canoe direction map {map_name}"))?
            .clone();
        anyhow::ensure!(
            map.scripts.contains_key(label),
            "canoe direction text {label} missing"
        );
        authoring::text(&mut map, label, lines);
        authoring::finish(pack, &mut map)?;
        crate::authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}
