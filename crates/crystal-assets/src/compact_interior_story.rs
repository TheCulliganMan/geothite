//! Replay preserved room stories on the reviewed compact regional branch.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, build_minnesota_interior_life_modpack};

const STORY_SHA256: &str = "0d1d6fc72c44d944a9dc1125a572704d8aec018660b7626bc01a034b07237e46";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Supplement {
    schema_version: u8,
    base_content_hash: String,
    story_source_sha256: String,
    manifest: String,
    rooms: Vec<Value>,
}

/// Replays the preserved 187-room authoring source and its explicit supplement.
///
/// The verified compact parent keeps every existing numeric event identity,
/// exterior, cave revision, service and art/audio catalog. The ordinary interior
/// publisher allocates fresh story flags above the parent's existing events;
/// no numeric flags or old exterior modules are imported from the stale pack.
/// The combined contract must cover every installed non-gym regional interior.
/// This returns a pack without writing it and does not establish gameplay or
/// visual acceptance, compatibility with old saves, or completion of missing
/// buildings elsewhere in the region.
///
/// # Errors
/// Rejects stale parent/source hashes, unsupported or duplicated supplements,
/// incomplete coverage, service/quest replacement, unsafe actors and failed
/// native publication or compact-geography checks. Inputs stay unchanged.
pub fn build_compact_interior_story_pack(
    base: &CompiledGamePack,
    story_source: &str,
    supplement_source: &str,
) -> Result<CompiledGamePack> {
    crate::validate_compact_superior_geography(base)?;
    let supplement: Supplement = serde_json::from_str(supplement_source)?;
    ensure!(
        supplement.schema_version == 1
            && supplement.base_content_hash == base.identity()?.content_hash
            && supplement.story_source_sha256 == STORY_SHA256
            && format!("{:x}", Sha256::digest(story_source.as_bytes())) == STORY_SHA256
            && supplement.manifest == "minnesota-interior-life-compact-stories-v1",
        "reviewed compact parent, preserved story source and supplement required"
    );
    let mut stories: Value = serde_json::from_str(story_source)?;
    ensure!(
        stories["schema_version"] == 1 && stories["all_installed_interiors"] == true,
        "preserve the complete room coverage contract"
    );
    let original_rooms = stories["rooms"]
        .as_array()
        .context("preserved room array")?;
    ensure!(
        original_rooms.len() == 187,
        "preserved story coverage differs"
    );
    let original_names = original_rooms
        .iter()
        .map(room_name)
        .collect::<Result<BTreeSet<_>>>()?;
    let installed = base
        .data
        .maps
        .values()
        .filter(|m| {
            m.id.starts_with("Mn")
                && !m.id.contains("Gym")
                && matches!(
                    m.attributes.environment.as_deref(),
                    Some("INDOOR" | "GATE" | "CAVE" | "DUNGEON")
                )
        })
        .map(|m| m.id.as_str())
        .collect::<BTreeSet<_>>();
    let additional = supplement
        .rooms
        .iter()
        .map(room_name)
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(
        additional.len() == supplement.rooms.len()
            && additional == installed.difference(&original_names).copied().collect(),
        "supplement must cover exactly the additional installed interiors"
    );
    stories["manifest"] = Value::String(supplement.manifest);
    stories["rooms"]
        .as_array_mut()
        .context("mutable room array")?
        .extend(supplement.rooms);
    let pack = build_minnesota_interior_life_modpack(base, &serde_json::to_string(&stories)?)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

fn room_name(room: &Value) -> Result<&str> {
    room["map"].as_str().context("supplement room name")
}

#[cfg(test)]
#[path = "compact_interior_story_tests.rs"]
mod tests;
