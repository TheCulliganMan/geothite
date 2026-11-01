//! A native remote-feed scene for the separate Northstar receiver interaction.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::CompiledGamePack;
use crate::boundary_basecamp_story::{
    BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT, BOUNDARY_SURVEY_RECEIVER, BOUNDARY_SURVEY_RESTORED_EVENT,
    BOUNDARY_SURVEY_SHELTER,
};
use crate::grand_marais_modpack::authoring::{self, command as c};

const MANIFEST: &str = "minnesota-northstar-relay-scene-v1";
const REFRESH: &str = "MnNorthstarRelayRefresh";
const OFF: &str = "MnNorthstarRelayOff";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    schema_version: u16,
    map: String,
    species: String,
    receiver_core: [u16; 2],
    receiver_metatile: [u16; 2],
    free_access: bool,
    capture_required: bool,
    texts: BTreeMap<String, Vec<String>>,
    contract: String,
}

/// Adds Kettlurk's native picture and a powered-down relay after acknowledgment.
///
/// Requires an installed Q07 shelter and the reviewed KETTLURK species/assets.
/// The existing chief, refusal, choice, alignment and completion remain intact.
/// Only the shelter's private art and receiver continuation change. No saved
/// progress, capture, item, party member or Safari admission is granted.
///
/// # Errors
/// Returns an error for missing species/assets, stale shelter logic or anchors,
/// conflicting scene identities, invalid source text or native pack validation.
/// The supplied pack remains unchanged on failure.
pub fn build_northstar_relay_scene(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let scene: Scene = serde_json::from_str(source)?;
    ensure!(
        scene.schema_version == 1
            && scene.map == BOUNDARY_SURVEY_SHELTER
            && scene.species == "KETTLURK"
            && scene.receiver_core == [12, 3]
            && scene.receiver_metatile == [6, 1]
            && scene.free_access
            && !scene.capture_required
            && !scene.contract.is_empty(),
        "supported free Northstar relay contract required"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "relay scene already installed"
    );
    ensure!(
        base.data.pokemon.contains_key(&scene.species)
            && base
                .runtime_files()
                .contains_key("gfx/pokemon/kettlurk/front.2bpp"),
        "reviewed native KETTLURK species and front pixels required"
    );
    let mut room = base
        .data
        .maps
        .get(BOUNDARY_SURVEY_SHELTER)
        .context("installed Q07 shelter")?
        .clone();
    ensure!(
        room.attributes.width == 8
            && room.attributes.height == 6
            && room.blocks.get(14) == Some(&8),
        "unchanged native shelter/console required"
    );
    ensure!(
        room.events
            .bg_events
            .iter()
            .any(|e| e.script == BOUNDARY_SURVEY_RECEIVER && [e.x, e.y] == scene.receiver_core),
        "native receiver event required"
    );
    let original = room
        .scripts
        .get(BOUNDARY_SURVEY_RECEIVER)
        .and_then(|v| v.as_array())
        .context("native receiver program")?;
    ensure!(
        original.starts_with(&[
            c("checkevent", &[BOUNDARY_SURVEY_RESTORED_EVENT]),
            c("iftrue", &["MnBoundarySurveyReceiverAlready"]),
            c("checkevent", &[BOUNDARY_SURVEY_CHIEF_DEFEATED_EVENT]),
            c("iffalse", &["MnBoundarySurveyReceiverLocked"]),
            c("opentext", &[]),
            c("writetext", &["MnBoundarySurveyReceiverReadyText"]),
            c("yesorno", &[]),
            c("iffalse", &["MnBoundarySurveyReceiverDeclined"]),
        ]),
        "native chief guard and retryable receiver choice required"
    );
    let marker = c("setevent", &[BOUNDARY_SURVEY_RESTORED_EVENT]);
    let position = original
        .iter()
        .position(|v| *v == marker)
        .context("separate restoration continuation")?;
    ensure!(
        original.iter().filter(|v| **v == marker).count() == 1
            && position >= 2
            && original[position - 1] == c("waitbutton", &[])
            && original[position - 2] == c("writetext", &["MnBoundarySurveyReceiverRestoreText"]),
        "acknowledged native alignment must precede restoration"
    );
    let mut commands = original.clone();
    // Old placeholder text is unreachable after the original Q07 binding.
    // Remove it so cached and uncached scene modules have the same closure.
    room.scripts.remove("MN_SURVEY_CHIEFText");
    room.scripts.remove("MN_BOUNDARY_SURVEY_RECEIVERText");
    let mut pack = base.clone();
    let off = crate::northstar_relay_art::install(&mut pack, &mut room)?;
    let off = format!("${off:02x}");
    let feed = vec![
        c("closetext", &[]),
        c("earthquake", &["12"]),
        c("opentext", &[]),
        c("writetext", &["MnNorthstarRelayFeedText"]),
        c("waitbutton", &[]),
        c("closetext", &[]),
        c("pokepic", &["KETTLURK"]),
        c("waitbutton", &[]),
        c("closepokepic", &[]),
        c("opentext", &[]),
        c("writetext", &["MnNorthstarRelayCutText"]),
        c("waitbutton", &[]),
        c("changeblock", &["12", "2", &off]),
        c("refreshmap", &[]),
    ];
    commands.splice(position..position, feed);
    room.scripts
        .insert(BOUNDARY_SURVEY_RECEIVER.into(), json!(commands));
    for (label, lines) in scene.texts {
        ensure!(
            matches!(
                label.as_str(),
                "MnNorthstarRelayFeedText" | "MnNorthstarRelayCutText"
            ) && !room.scripts.contains_key(&label)
                && !lines.is_empty()
                && lines
                    .iter()
                    .all(|l| !l.is_empty() && l.chars().count() <= 18),
            "valid, unique native relay text required: {label}"
        );
        authoring::text(
            &mut room,
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    for label in ["MnNorthstarRelayFeedText", "MnNorthstarRelayCutText"] {
        ensure!(
            room.scripts.contains_key(label),
            "missing relay text {label}"
        );
    }
    ensure!(
        !room.scripts.contains_key(REFRESH) && !room.scripts.contains_key(OFF),
        "unused relay callback labels required"
    );
    crate::ely_league_scripts::callback(
        &mut room,
        "MAPCALLBACK_TILES",
        REFRESH,
        vec![
            c("checkevent", &[BOUNDARY_SURVEY_RESTORED_EVENT]),
            c("iftrue", &[OFF]),
            c("endcallback", &[]),
        ],
    );
    room.scripts.insert(
        OFF.into(),
        json!([c("changeblock", &["12", "2", &off]), c("endcallback", &[])]),
    );
    crate::ely_league_rooms::finish(&pack, &mut room)?;
    crate::authored_town_publication::replace(&mut pack, room)?;
    pack.report.manifests.push(MANIFEST.into());
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
