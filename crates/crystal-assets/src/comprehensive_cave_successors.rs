//! Preserve current campaign content while adopting the accepted cave successors.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::comprehensive_current_world::adopt_entry;

const MARKER: &str = "comprehensive-cave-and-stair-successors-v1";
const FARMER: &str = "MN_R01_FARMERText";

/// Integrates the accepted R10/R20 caves and R01/R03 stair directions.
///
/// The three inputs are pinned to the reviewed comprehensive, common world,
/// and accepted cave revisions. Only 55 conflict-free entries and the farmer's
/// three mirrored text commands change. The farmer retains the newer story's
/// IVY referral while describing the actual east steps. Assets, audio, other
/// maps and existing saves are unchanged. The output receives a new identity.
///
/// # Errors
/// Rejects unreviewed inputs, stale entry or text preimages, repeated integration,
/// changed donor assets/audio, and invalid runtime data or regional geography.
pub fn integrate_comprehensive_cave_successors(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    latest: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    for (pack, hash) in [
        (
            current,
            "1442b9b9f2ff69b954c03bb3762605c9119c17e99f317cdaac6557186101a19c",
        ),
        (
            common,
            "acbcb88bb951f3cc343ac550a4a83e423904a7b42fa4dfccfc88497d49092866",
        ),
        (
            latest,
            "e964dc760a49b64510f93d1469169a5d88aa73104d12f026e8dcf4084f73bc8f",
        ),
    ] {
        ensure!(
            pack.identity()?.content_hash == hash,
            "exact cave integration inputs required"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        common.runtime_files == latest.runtime_files,
        "donor assets changed"
    );
    ensure!(
        common.audio_manifest == latest.audio_manifest,
        "donor audio changed"
    );
    ensure!(
        !current.report.manifests.iter().any(|m| m == MARKER),
        "fresh integration required"
    );
    let mut pack = current.clone();
    for map in ["MnR01C00R00", "MnR03C00R00", "MnR10Cave1F", "MnR20Cave1F"] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            map,
        )?;
    }
    for map in ["MnVoyageursIce1F", "MnVoyageursIceB1F"] {
        adopt_entry(
            &mut pack.data.maps,
            &common.data.maps,
            &latest.data.maps,
            map,
        )?;
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
            &format!("{map}_Blocks"),
        )?;
    }
    for map in ["MnR10Cave1F", "MnR20Cave1F"] {
        adopt_entry(
            &mut pack.data.map_attributes,
            &common.data.map_attributes,
            &latest.data.map_attributes,
            map,
        )?;
        adopt_entry(
            &mut pack.data.map_dimensions,
            &common.data.map_dimensions,
            &latest.data.map_dimensions,
            map,
        )?;
        adopt_entry(
            &mut pack.data.npcs,
            &common.data.npcs,
            &latest.data.npcs,
            map,
        )?;
        adopt_entry(
            &mut pack.data.runtime_map_metadata,
            &common.data.runtime_map_metadata,
            &latest.data.runtime_map_metadata,
            map,
        )?;
        adopt_entry(
            &mut pack.data.map_blocks,
            &common.data.map_blocks,
            &latest.data.map_blocks,
            &format!("{map}_Blocks"),
        )?;
        adopt_entry(
            &mut pack.data.map_scripts,
            &common.data.map_scripts,
            &latest.data.map_scripts,
            &format!("{map}_MapEvents"),
        )?;
    }
    for trainer in [
        "MN_EXTRA_R10_SPRING",
        "MN_EXTRA_R10_SURVEY",
        "MN_EXTRA_R20_WAYFINDER",
    ] {
        adopt_entry(
            &mut pack.data.trainers.trainers,
            &common.data.trainers.trainers,
            &latest.data.trainers.trainers,
            trainer,
        )?;
        for suffix in ["", "After", "Declined", "Lost", "Seen", "Won"] {
            adopt_entry(
                &mut pack.data.map_scripts,
                &common.data.map_scripts,
                &latest.data.map_scripts,
                &format!("{trainer}{suffix}"),
            )?;
        }
        for flag in [trainer.to_owned(), format!("EVENT_{trainer}")] {
            adopt_entry(
                &mut pack.data.story_event_script_constants.global,
                &common.data.story_event_script_constants.global,
                &latest.data.story_event_script_constants.global,
                &flag,
            )?;
        }
    }
    for label in [
        "MN_R01_BLUFF_WALKERText",
        "MN_R01_SIGN_05Text",
        "MN_R01_T03Lost",
        "MN_R01_T03Won",
        "MN_R03_T04Won",
        "MN_R03_TERRACE_SIGNText",
        "MN_R10_ESCAPE_ROPE",
    ] {
        adopt_entry(
            &mut pack.data.map_scripts,
            &common.data.map_scripts,
            &latest.data.map_scripts,
            label,
        )?;
    }
    adopt_entry(
        &mut pack.data.story_event_script_constants.global,
        &common.data.story_event_script_constants.global,
        &latest.data.story_event_script_constants.global,
        "EVENT_MN_R10_ESCAPE_ROPE",
    )?;
    let raw = pack
        .data
        .map_scripts
        .get_mut(FARMER)
        .context("raw farmer text")?;
    let room = pack
        .data
        .maps
        .get_mut("MnR01Farmhouse")
        .context("farmer room")?;
    let local = room.scripts.get_mut(FARMER).context("local farmer text")?;
    ensure!(raw == local, "farmer raw/local mirrors disagree");
    let body = room
        .script_text_bodies
        .get_mut(FARMER)
        .context("cached farmer text")?;
    ensure!(body.commands.len() == 11, "reviewed farmer cache required");
    for commands in [raw, local] {
        let commands = commands.as_array_mut().context("farmer text commands")?;
        ensure!(commands.len() == 11, "reviewed farmer text required");
        let args = commands[4]
            .get_mut("args")
            .and_then(serde_json::Value::as_array_mut)
            .context("farmer direction operands")?;
        ensure!(
            args.len() == 1 && args[0].as_str() == Some("\"The east ramp\""),
            "stale farmer directions"
        );
        args[0] = serde_json::Value::String("\"The east steps\"".into());
    }
    let args = &mut body.commands[4].args;
    ensure!(
        args.as_slice() == ["\"The east ramp\""],
        "stale farmer directions"
    );
    args[0] = "\"The east steps\"".into();
    for manifest in &latest.report.manifests {
        if !common.report.manifests.contains(manifest) && !pack.report.manifests.contains(manifest)
        {
            pack.report.manifests.push(manifest.clone());
        }
    }
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::verify_northstar_publication_geometry(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}
