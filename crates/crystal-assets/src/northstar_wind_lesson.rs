//! Give challengers Saven's Bankwing lesson before the first Wind Badge battle.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_dialogue};

const GYM: &str = "MnStCloudGym";
const LESSON: &str = "MnStCloudGymLeaderSeenText";
const PARENTS: [&str; 2] = [
    "e9dea645cb0b4132853ccbfc2f81bb4ee883aaef65917af9cd88fc79825ef25e",
    "0a84a37331f95cc1d56eba9f06c7b0ed9e544d6984a82ae3c52b68c063276549",
];

/// Replaces the middle page of Saven's pre-battle conversation.
///
/// # Errors
/// Returns an error for a different parent, unexpected native pages or an
/// inconsistent map-local or global script, or if runtime verification fails.
pub fn revise_wind_lesson(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        PARENTS.contains(&pack.identity.content_hash.as_str())
            && pack.report.manifests.last().map(String::as_str)
                == Some("northstar-quarry-landings-v1"),
        "wind lesson requires a matching Quarry Landings archive"
    );
    let room = pack.data.maps.get_mut(GYM).context("St. Cloud Gym")?;
    let previous = pack
        .data
        .map_scripts
        .get(LESSON)
        .context("Saven's greeting")?;
    ensure!(
        room.scripts.get(LESSON) == Some(previous),
        "Saven's local greeting differs"
    );
    let old = previous.as_array().context("Saven's original pages")?;
    authored_dialogue::install_canonical(room, LESSON)?;
    let script = room
        .scripts
        .get(LESSON)
        .context("Saven's revised greeting")?;
    let revised = script.as_array().context("Saven's revised pages")?;
    ensure!(
        old.len() == 7
            && revised.len() == 7
            && old[..2] == revised[..2]
            && old[4..] == revised[4..]
            && revised[2]["command"] == "para"
            && revised[3]["command"] == "line"
            && revised[2..4] != old[2..4],
        "only the middle page of Saven's three-page greeting may change"
    );
    let texts = BTreeMap::from([(LESSON.to_owned(), script.clone())]);
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(GYM, &texts)?);
    pack.data.map_scripts.extend(texts);
    pack.report
        .manifests
        .push("northstar-wind-lesson-v1".into());
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
