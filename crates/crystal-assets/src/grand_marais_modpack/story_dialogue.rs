//! Refresh only the two authored Northstar foreshadowing conversations.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use super::authoring;
use crate::CompiledGamePack;

/// Updates Tamarack's completion and the fisher's current observations in place.
///
/// Requires the installed Grand Marais town and the existing conversation roots.
/// Reads their text from the supplied town JSON; it does not republish the town,
/// change its geometry, grant its demo starter, or alter local/regional flags.
/// All non-text tables, other conversations, catalogs and audio are preserved.
/// Repeating the refresh is idempotent, and the input remains unchanged.
///
/// # Errors
/// Returns an error for missing native maps/texts, changed conversation roots,
/// invalid source dialogue, or native parser/pack validation failures.
pub fn refresh_grand_marais_story_dialogue(
    base: &CompiledGamePack,
    town_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Value = serde_json::from_str(town_json).context("Grand Marais story dialogue")?;
    let mut pack = base.clone();
    for (map, root, label) in [
        (
            "MnGrandMaraisFieldStation",
            "MnGrandMaraisTamarackThanks",
            "MnGrandMaraisTamarackThanksText",
        ),
        (
            "MnGrandMaraisFisherHouse",
            "MnGrandMaraisFisherHouseFisher",
            "MnGrandMaraisFisherHouseFisherText",
        ),
    ] {
        let room = pack
            .data
            .maps
            .get_mut(map)
            .context("installed Grand Marais story room")?;
        ensure!(
            room.scripts.get(root)
                == Some(&json!([authoring::command("jumptextfaceplayer", &[label])])),
            "Grand Marais story refresh requires unchanged conversation {root}"
        );
        ensure!(
            room.scripts.contains_key(label) && room.script_text_bodies.contains_key(label),
            "Grand Marais story refresh requires existing text {label}"
        );
        let lines: Vec<String> = serde_json::from_value(source["dialogue"][map][label].clone())
            .with_context(|| format!("authored story text {label}"))?;
        ensure!(
            !lines.is_empty()
                && lines
                    .iter()
                    .all(|line| !line.is_empty() && line.chars().count() <= 18),
            "Grand Marais story text must fit18columns: {label}"
        );
        authoring::text(
            room,
            label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        room.script_text_commands = crate::parse_script_text_commands(map, &room.scripts)?;
        room.script_text_bodies = crate::parse_script_text_bodies(map, &room.scripts)?;
    }
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
