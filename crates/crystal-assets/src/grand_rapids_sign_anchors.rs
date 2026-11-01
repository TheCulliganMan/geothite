//! Re-anchor three existing civic signs without changing their art or behavior.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::CompiledGamePack;

/// Repairs only Grand Rapids' three misplaced background-event coordinates.
///
/// # Errors
/// Rejects changed source, event preimages, board art/collisions or unrelated data.
pub fn revise_grand_rapids_sign_anchors(
    base: &CompiledGamePack,
    before_json: &str,
    after_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    const MANIFEST: &str = "minnesota-grand-rapids-sign-anchors-v1";
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "already revised"
    );
    let before: Value = serde_json::from_str(before_json)?;
    let after: Value = serde_json::from_str(after_json)?;
    ensure!(
        before["id"] == "MnGrandRapids" && after["id"] == before["id"],
        "wrong town source"
    );
    let mut restored_source = after.clone();
    let old_signs = before["signs"].as_array().context("before signs")?;
    let new_signs = after["signs"].as_array().context("after signs")?;
    ensure!(
        old_signs.len() == 3 && new_signs.len() == 3,
        "exact three signs required"
    );
    let mut map = base
        .data
        .maps
        .get("MnGrandRapids")
        .context("current town")?
        .clone();
    ensure!(
        (map.attributes.width, map.attributes.height) == (24, 24)
            && map.events.bg_events.len() == 3,
        "changed town/event inventory"
    );
    for (index, (id, old, new)) in [
        ("MN_GR_WEST_SIGN", [7, 23], [14, 22]),
        ("MN_GR_EAST_SIGN", [39, 23], [38, 22]),
        ("MN_GR_DEPOT_SIGN", [23, 13], [22, 12]),
    ]
    .into_iter()
    .enumerate()
    {
        ensure!(
            old_signs[index]["id"] == id
                && new_signs[index]["id"] == id
                && old_signs[index]["tile"] == serde_json::json!(old)
                && new_signs[index]["tile"] == serde_json::json!(new),
            "sign source drift: {id}"
        );
        restored_source["signs"][index]["tile"] = old_signs[index]["tile"].clone();
        let event = map
            .events
            .bg_events
            .iter_mut()
            .find(|e| e.script == id)
            .context("native sign event")?;
        ensure!(
            [event.x, event.y] == old && event.event_type == "BGEVENT_READ",
            "event preimage drift: {id}"
        );
        event.x = new[0];
        event.y = new[1];
        let commands = map
            .map_event_section_commands
            .iter_mut()
            .filter(|command| {
                command.command == "bg_event"
                    && command.args.get(3).is_some_and(|script| script == id)
            })
            .collect::<Vec<_>>();
        ensure!(commands.len() == 1, "exact raw sign command required: {id}");
        let command = commands.into_iter().next().context("raw sign command")?;
        ensure!(
            command.args
                == [
                    old[0].to_string(),
                    old[1].to_string(),
                    "BGEVENT_READ".into(),
                    id.into()
                ],
            "raw sign event preimage drift: {id}"
        );
        command.args[0] = new[0].to_string();
        command.args[1] = new[1].to_string();
        ensure!(
            map.blocks[usize::from(new[1] / 2) * 24 + usize::from(new[0] / 2)] == 0x45,
            "sign must use its existing native board art: {id}"
        );
        ensure!(
            crate::authored_town_geometry::kind(base, &map, new)? == "WALL"
                && crate::authored_town_geometry::kind(base, &map, [new[0], new[1] + 1])?
                    == "FLOOR",
            "board and south approach drift: {id}"
        );
    }
    ensure!(
        restored_source == before,
        "source changed beyond three sign coordinates"
    );
    let mut result = base.clone();
    crate::authored_town_publication::replace(&mut result, map)?;
    let mut restored = result.clone();
    crate::authored_town_publication::replace(
        &mut restored,
        base.data.maps["MnGrandRapids"].clone(),
    )?;
    ensure!(
        serde_json::to_value(&restored.data)? == serde_json::to_value(&base.data)?,
        "unrelated native data changed"
    );
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}
