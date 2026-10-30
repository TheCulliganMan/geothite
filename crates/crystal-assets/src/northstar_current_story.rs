//! Compose the complete authored story onto a certified current regional parent.
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::minnesota_overworld_sprites::OverworldManifest;

const COMPOSING: &str = "northstar-current-story-composing-v1";
const MANIFEST: &str = "northstar-current-story-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Composition {
    schema_version: u8,
    id: String,
    approved_parents: Vec<Parent>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parent {
    content_hash: String,
    maps: usize,
    review: String,
}

/// Only the exact-parent composition entry point installs this transient marker.
pub(crate) fn is_composing(pack: &CompiledGamePack) -> bool {
    pack.report.manifests.iter().any(|m| m == COMPOSING)
        && pack
            .report
            .manifests
            .iter()
            .any(|m| m == "northstar-minnesota-only-v1")
}

fn read(directory: &Path, name: &str) -> Result<String> {
    std::fs::read_to_string(directory.join(name)).with_context(|| format!("story source {name}"))
}

fn layouts(directory: &Path, prefix: &str, names: &[&str]) -> Result<Vec<String>> {
    names
        .iter()
        .map(|name| read(directory, &format!("{prefix}/{name}.json")))
        .collect()
}

/// Replays the acted opening, both complex lairs, six rival encounters and finale.
///
/// The certified parent is the compact regional R20/R04 successor, already
/// bound to Minnesota species. Its binder marker is retained throughout. Only
/// newly authored/changed story data is bound, preserving established ordinary
/// teams, grass, caves, optional Vergloom and the other source regions. The
/// guarded League entrance receives its single native floor turnout. Installs
/// reviewed native Kettlurk/Gillpup banks and the written League ending.
///
/// # Errors
/// Rejects a different parent, missing/changed scene preimages, missing authored
/// sources, invalid layouts/species, or failed native publication verification.
/// No partial pack is returned and the borrowed parent remains unchanged.
pub fn build_northstar_current_story(
    base: &CompiledGamePack,
    directory: &Path,
) -> Result<CompiledGamePack> {
    build_northstar_current_story_from_composition(
        base,
        directory,
        &read(directory, "revisions/current-story-v01/composition.json")?,
    )
}

/// Replays the authored story using an explicit exact-parent composition contract.
///
/// The contract has the same schema and parent checks as the canonical entry
/// point. This lets an independently reviewed interior branch retain its own
/// binding without editing the shared regional allowlist. Chapter preimages,
/// native geography, existing art/audio and ordinary teams remain guarded.
/// No partial composition marker is returned.
///
/// # Errors
/// Rejects a malformed/stale contract, a different parent, changed chapter
/// anchors, missing sources, invalid regional content or failed publication
/// verification. Inputs are borrowed and remain unchanged on failure.
pub fn build_northstar_current_story_from_composition(
    base: &CompiledGamePack,
    directory: &Path,
    composition_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let source: Composition = serde_json::from_str(composition_json)?;
    ensure!(
        source.schema_version == 1 && source.id == MANIFEST,
        "supported current story composition required"
    );
    let hash = base.identity()?.content_hash;
    ensure!(
        source
            .approved_parents
            .iter()
            .any(|parent| parent.content_hash == hash
                && parent.maps == base.data.maps.len()
                && !parent.review.is_empty()),
        "reviewed exact current story parent required: {hash}"
    );
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|m| m == COMPOSING || m == MANIFEST),
        "current story already composing/installed"
    );
    let mut pack = base.clone();
    pack.report.manifests.push(COMPOSING.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    ensure!(
        is_composing(&pack),
        "current parent must already be region-bound"
    );
    for name in [
        "pokemon/overworld/20261027-v1/sources/mn_f045/manifest-kettlurk-root-v3-safe-phase.json",
        "pokemon/overworld/20261027-v1/sources/mn_f140/manifest-gillpup-v8-actual-pupils.json",
    ] {
        let path = directory.join(name).canonicalize()?;
        let manifest = OverworldManifest::from_json(&std::fs::read(&path)?)?;
        pack = manifest
            .compile(path.parent().context("native bank directory")?)?
            .build_pack(&pack)?;
    }
    let story = read(directory, "story.json")?;
    pack = crate::build_northstar_relay_scene(&pack, &read(directory, "northstar-relay.json")?)
        .context("current relay scene")?;
    pack = crate::build_northstar_supplies(&pack, &read(directory, "northstar-supplies.json")?)
        .context("current sleep recovery")?;
    pack = crate::refresh_northstar_dialogue(&pack, &story).context("current chapter dialogue")?;
    pack = crate::build_northstar_story_events(
        &pack,
        &story,
        &read(directory, "story-events-v2.json")?,
    )
    .context("current acted opening/returns")?;
    pack = crate::build_northstar_ice_lair(
        &pack,
        &read(directory, "lairs/minneapolis-events.json")?,
        &read(directory, "lairs/minneapolis-cargo.json")?,
        &read(directory, "lairs/minneapolis-relay.json")?,
    )
    .context("current ICE rescue")?;
    pack = crate::build_northstar_drill_lair(
        &pack,
        &read(directory, "lairs/iron-events.json")?,
        &read(directory, "lairs/iron-approach.json")?,
        &read(directory, "lairs/iron-gallery.json")?,
        &read(directory, "lairs/iron-core.json")?,
    )
    .context("current Lode drill")?;
    pack = crate::build_northstar_ice_headquarters(
        &pack,
        &read(directory, "lairs/ice-headquarters/network.json")?,
        &layouts(
            directory,
            "lairs/ice-headquarters",
            &["operations", "records", "maintenance", "transfer"],
        )?,
    )
    .context("current ICE headquarters")?;
    pack = crate::build_northstar_iron_workings(
        &pack,
        &read(directory, "lairs/iron-workings/network.json")?,
        &layouts(
            directory,
            "lairs/iron-workings",
            &["haulage", "sump", "vent"],
        )?,
    )
    .context("current interleaved mine")?;
    pack = crate::build_northstar_rival_arc(
        &pack,
        &read(directory, "story-arc-v3.json")?,
        &read(directory, "lairs/rival-arc/intake.json")?,
    )
    .context("current recurring rival/intake")?;
    let roster = read(directory, "pokemon/roster.json")?;
    let curation = read(directory, "pokemon/roster-curation.json")?;
    pack = crate::northstar_regional_cast::bind_story_delta(
        &pack,
        base,
        &read(directory, "story-pokemon.json")?,
        &roster,
        &curation,
        &read(directory, "pokemon/species-profiles.json")?,
    )
    .context("current story-only regional binding")?;
    pack = crate::refresh_northstar_league_dialogue(
        &pack,
        &read(directory, "revisions/league-finale-v01/dialogue.json")?,
        &roster,
        &curation,
    )
    .context("current written League ending")?;
    pack = crate::northstar_story_revision::finish_composition(&pack, &story)
        .context("current staged dialogue overhaul")?;
    pack.report.manifests.retain(|m| m != COMPOSING);
    pack.report.manifests.push(MANIFEST.into());
    ensure!(
        base.compiled_audio() == pack.compiled_audio(),
        "story composition changed audio"
    );
    ensure!(
        base.data.pokemon == pack.data.pokemon && base.data.moves == pack.data.moves,
        "story composition changed species/moves"
    );
    for (name, bytes) in &base.runtime_files {
        ensure!(
            pack.runtime_files.get(name) == Some(bytes),
            "story composition changed existing native art/runtime file: {name}"
        );
    }
    let policy: serde_json::Value = serde_json::from_str(&read(directory, "story-pokemon.json")?)?;
    for (id, original) in &base.data.trainers.trainers {
        if policy["faction_teams"].get(id).is_none() {
            let mut expected = original.clone();
            for (renamed, name) in [
                ("MN_LEAGUE_4", "HALDEN"),
                ("MN_R08_T07", "BRAM"),
                ("MN_R11_T05", "NILS"),
                ("MN_R16_T05", "OTTO"),
                ("MN_R17_T03", "ELI"),
            ] {
                if id == renamed {
                    expected.name = name.into();
                }
            }
            ensure!(
                pack.data.trainers.trainers.get(id) == Some(&expected),
                "story composition changed unrelated trainer: {id}"
            );
        }
    }
    ensure!(
        base.data.wild_encounters == pack.data.wild_encounters
            && base.data.field_encounters == pack.data.field_encounters,
        "story composition changed ordinary encounters"
    );
    for (name, before) in &base.data.maps {
        if matches!(
            before.attributes.environment.as_deref(),
            Some("ROUTE" | "TOWN")
        ) {
            let after = &pack.data.maps[name];
            let turnout_blocks = (name == "MnR13C00R00").then(|| {
                // The final shared scene builder validates the native wall,
                // adjacent floor and absence of actors/events before opening
                // this one metatile at the canonical connection offset.
                let mut blocks = before.blocks.clone();
                blocks[16] = blocks[15];
                blocks
            });
            let expected_blocks = turnout_blocks.as_ref().unwrap_or(&before.blocks);
            ensure!(
                *expected_blocks == after.blocks && before.attributes == after.attributes,
                "story composition changed outdoor geography: {name}"
            );
        }
    }
    crate::verify_northstar_publication_geometry(&pack)?;
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

#[cfg(test)]
#[path = "northstar_current_story_tests.rs"]
mod tests;
