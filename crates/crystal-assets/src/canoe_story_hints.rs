//! Replace obsolete regional HM-only shore directions with canoe instructions.
use anyhow::{Context, Result};

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

/// Refresh regional travel dialogue on a pack that already contains the canoe.
///
/// # Errors
/// Returns an error if the pack is invalid, lacks the canoe, or lacks a required
/// map or conversation.
pub fn build_canoe_dialogue_refresh(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    anyhow::ensure!(
        base.data
            .items
            .contains_key(crystal_core::systems::field_moves::CANOE_ITEM),
        "canoe-equipped pack required"
    );
    let mut pack = base.clone();
    install(&mut pack)?;
    pack.report
        .manifests
        .push("minnesota-canoe-dialogue-refresh-v1".into());
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
                "I launch my CANOE",
                "at the north end.",
                "Keep off the",
                "rails when waves",
                "wash over them.",
            ][..],
        ),
        (
            "MnBrainerdOutfitter",
            "MN_BRAINERD_CAMP_HELPERText",
            &[
                "I put my boots in",
                "the water to see",
                "how deep it was.",
                "Took all afternoon",
                "to dry them. A",
                "CANOE keeps your",
                "feet a bit drier.",
            ][..],
        ),
        (
            "MnBrainerdOutfitter",
            "MN_BRAINERD_GEAR_KEEPERText",
            &[
                "Check the gear",
                "before you reach",
                "the landing. A",
                "loose strap is",
                "easier to fix here",
                "than in a wet",
                "CANOE. Pack well",
                "before you paddle.",
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
                "I was near the",
                "falls when the",
                "current stopped.",
                "Then it came back",
                "in a rush. Mind",
                "your CANOE out",
                "there.",
            ][..],
        ),
        (
            "MnMississippiLowerWoodsC01R01",
            "MnPepinText_needs_surf",
            &[
                "The reeds hide",
                "the water here.",
                "A quiet approach",
                "by CANOE may help.",
            ][..],
        ),
        (
            "MnMoorhead",
            "MnMoorheadRiverAccessText",
            &[
                "RED RIVER ACCESS",
                "Cast from the",
                "bank. Launch a",
                "CANOE from shore.",
            ][..],
        ),
        (
            "MnMinneapolis",
            "MnMPNorthCheckBadgesText",
            &[
                "The north crossing",
                "can flood.",
                "ST PAUL has a",
                "CANOE SHOP.",
                "They fit a boat",
                "after you earn the",
                "CAPITOL BADGE.",
            ][..],
        ),
        (
            "MnR07C00R00",
            "MN_R07_LAKE_WALKERText",
            &[
                "A narrow meadow",
                "links the dry",
                "pine peninsula.",
                "No paddling is",
                "needed here.",
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
    // Preserve each installed gym conversation; replace only the common,
    // redundant travel-direction suffix when that exact suffix is present.
    let mut gym = pack
        .data
        .maps
        .get("MnStPaulGym")
        .context("St Paul gym")?
        .clone();
    for label in [
        "MnStPaulGymGuideText",
        "MnStPaulGymLeaderWonText",
        "MnStPaulGymLeaderGiftText",
        "MnStPaulGymLeaderAfterText",
    ] {
        let commands = gym
            .scripts
            .get(label)
            .and_then(|value| value.as_array())
            .context("gym leader text")?;
        let lines: Vec<String> = commands
            .iter()
            .filter_map(|command| {
                let opcode = command.get("command")?.as_str()?;
                if !matches!(opcode, "text" | "line" | "para" | "cont") {
                    return None;
                }
                let quoted = command.get("args")?.as_array()?.first()?.as_str()?;
                serde_json::from_str(quoted).ok()
            })
            .collect();
        if let Some(start) = lines.iter().position(|line| line == "For HM SURF, visit")
            && lines[start..]
                == [
                    "For HM SURF, visit",
                    "the ANGLER HOUSE",
                    "here in ST PAUL.",
                    "The CAPITOL BADGE",
                    "allows field use.",
                ]
        {
            let mut revised: Vec<&str> = lines[..start].iter().map(String::as_str).collect();
            revised.extend([
                "With the BADGE,",
                "the CANOE SHOP",
                "will fit you a",
                "free canoe.",
            ]);
            authoring::text(&mut gym, label, &revised);
        }
    }
    authoring::finish(pack, &mut gym)?;
    crate::authored_town_publication::replace(pack, gym)?;
    Ok(())
}
