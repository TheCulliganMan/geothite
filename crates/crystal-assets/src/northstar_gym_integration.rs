//! Integrate the reviewed gym branch without importing its older regional world.

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::CompiledGamePack;
use crate::current_minnesota_release_composition::merge_value;

const CURRENT: &str = "38ef157b041a1809cc0aa89cdf4fa3080f46994b56c0dcc1e15365055e6eb693";
const COMMON: &str = "834d4be07c45079fdc81f1777856a37058d064b4188f548e31616684436da5e3";
const REVIEWED: &str = "ba3b067a92192dad36d9b8884296dc036653d2acf3e3eb22325cef8b3ee0fd73";
const MARKER: &str = "northstar-current-eight-gyms-v1";
const GYMS: [&str; 8] = [
    "MnRochesterGym",
    "MnMankatoGym",
    "MnMinneapolisGym",
    "MnStPaulGym",
    "MnStCloudGym",
    "MnBrainerdGym",
    "MnBemidjiGym",
    "MnDuluthGym",
];

/// Integrates eight reviewed rooms and gym teams into the selected Glacier world.
///
/// Only the gym maps, their private tilesets, Cut replacements, gym trainers and
/// newly required gym events are imported. Newer canoe art, exterior geometry,
/// regional encounters, services and all audio remain owned by `current`.
/// Independent script changes merge by program; conflicting changes are errors.
/// The returned candidate has a new identity. This does not migrate saves.
///
/// # Errors
/// Rejects changed input identities, conflicting edits, shared gym art, invalid
/// events or invalid resulting content. All inputs remain unchanged on failure.
pub fn integrate_northstar_gyms(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    reviewed: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    ensure!(
        current.identity()?.content_hash == CURRENT,
        "exact Glacier world required"
    );
    install_reviewed_gyms(current, common, reviewed)
}

/// Internal stage after the exact-parent story consolidation. Its caller owns
/// the current-world identity check; donor identities remain pinned here.
pub(crate) fn install_reviewed_gyms(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    reviewed: &CompiledGamePack,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(current)?;
    for (pack, expected) in [(common, COMMON), (reviewed, REVIEWED)] {
        crate::verify_compiled_game_pack_for_runtime(pack)?;
        ensure!(
            pack.identity()?.content_hash == expected,
            "gym integration input drift"
        );
    }
    let mut result = current.clone();
    for (index, name) in GYMS.iter().enumerate() {
        let tileset = format!("mn_gym_{}", index + 1);
        ensure!(
            current
                .data
                .maps
                .values()
                .all(|room| { room.id == *name || room.attributes.tileset_name != tileset }),
            "gym art is shared outside {name}"
        );
        let mut current_room = current.data.maps.get(*name).context("current gym")?.clone();
        let mut common_room = common.data.maps.get(*name).context("common gym")?.clone();
        let mut reviewed_room = reviewed
            .data
            .maps
            .get(*name)
            .context("reviewed gym")?
            .clone();
        if *name == "MnStPaulGym" {
            // Both branches changed these pages: the reviewed room changed the
            // pad course and boat hints, while the current world promises a
            // free canoe. Canonical pages retain both the new course and the
            // current reward, without the removed public-road badge gate.
            // No other conflict is waived.
            for room in [&mut current_room, &mut common_room, &mut reviewed_room] {
                for label in [
                    "MnStPaulGymGuideText",
                    "MnStPaulGymLeaderAfterText",
                    "MnStPaulGymLeaderGiftText",
                    "MnStPaulGymLeaderWonText",
                ] {
                    crate::authored_dialogue::install_canonical(room, label)?;
                }
                room.script_text_commands = crate::parse_script_text_commands(name, &room.scripts)?;
                room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
            }
        }
        let room = merge(
            &current_room,
            &common_room,
            &reviewed_room,
            &format!("pack.data.maps.{name}"),
        )?;
        result.data.tilesets.insert(
            tileset.clone(),
            merge(
                current
                    .data
                    .tilesets
                    .get(&tileset)
                    .context("current gym art")?,
                common
                    .data
                    .tilesets
                    .get(&tileset)
                    .context("common gym art")?,
                reviewed
                    .data
                    .tilesets
                    .get(&tileset)
                    .context("reviewed gym art")?,
                &format!("tilesets.{tileset}"),
            )?,
        );
        for path in [
            format!("data/tilesets/{tileset}.json"),
            format!("data/tilesets/{tileset}_metatiles.bin"),
            format!("data/tilesets/{tileset}_palette_map.json"),
            format!("gfx/tilesets/{tileset}.png"),
            format!("gfx/tilesets/{tileset}.2bpp"),
        ] {
            let old = common
                .runtime_files
                .get(&path)
                .context("common private gym file")?;
            let new = reviewed
                .runtime_files
                .get(&path)
                .context("reviewed private gym file")?;
            let installed = current
                .runtime_files
                .get(&path)
                .context("current private gym file")?;
            ensure!(
                installed == old || installed == new,
                "conflicting gym file: {path}"
            );
            result.runtime_files.insert(path, new.clone());
        }
        if let Some(replacement) = reviewed.data.field_moves.cut.replacements.get(&tileset) {
            ensure!(
                current.data.field_moves.cut.replacements.get(&tileset)
                    == common.data.field_moves.cut.replacements.get(&tileset),
                "current Cut rules changed: {tileset}"
            );
            result
                .data
                .field_moves
                .cut
                .replacements
                .insert(tileset, replacement.clone());
        }
        crate::authored_town_publication::replace(&mut result, room)?;
    }
    for (id, trainer) in &reviewed.data.trainers.trainers {
        if id.starts_with("MN_GYM_") {
            result.data.trainers.trainers.insert(
                id.clone(),
                merge(
                    current
                        .data
                        .trainers
                        .trainers
                        .get(id)
                        .context("current gym trainer")?,
                    common
                        .data
                        .trainers
                        .trainers
                        .get(id)
                        .context("common gym trainer")?,
                    trainer,
                    &format!("trainers.{id}"),
                )?,
            );
        }
    }
    for name in reviewed.data.story_event_script_constants.global.keys() {
        if name.starts_with("EVENT_MN_GYM_")
            && !result
                .data
                .story_event_script_constants
                .global
                .contains_key(name)
        {
            // Allocate in the current catalog; the older branch's numeric bit
            // may already belong to a newer regional event.
            crate::northstar_lair_authoring::flag(&mut result, name)?;
        }
    }
    // Only carry provenance for the gym rooms, art and teams copied above.
    // Unrelated canoe/world markers belong to the current parent.
    for marker in &reviewed.report.manifests {
        if (marker.starts_with("minnesota-gym-")
            || matches!(
                marker.as_str(),
                "minnesota-rochester-training-hall-review-v1"
                    | "minnesota-rochester-gym-progress-repair-v1"
                    | "minnesota-st-cloud-airworks-review-v1"
                    | "minnesota-brainerd-wooden-docks-review-v1"
                    | "minnesota-bemidji-northwoods-gym-review-v1"
            ))
            && !result.report.manifests.contains(marker)
        {
            result.report.manifests.push(marker.clone());
        }
    }
    result.report.manifests.push(MARKER.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::verify_northstar_publication_geometry(&result)?;
    for (name, original) in &current.data.maps {
        if !GYMS.contains(&name.as_str()) {
            ensure!(
                result.data.maps.get(name) == Some(original),
                "unrelated map changed: {name}"
            );
        }
    }
    Ok(result)
}

fn merge<T: Serialize + DeserializeOwned>(
    current: &T,
    before: &T,
    after: &T,
    path: &str,
) -> Result<T> {
    let current = serde_json::to_value(current)?;
    let before = serde_json::to_value(before)?;
    let after = serde_json::to_value(after)?;
    let merged = merge_value(Some(&current), Some(&before), Some(&after), path)?
        .with_context(|| format!("integration removed {path}"))?;
    serde_json::from_value(merged).with_context(|| format!("typed integration of {path}"))
}
