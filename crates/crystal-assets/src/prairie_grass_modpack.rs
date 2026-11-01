//! Apply authored prairie palette regions without changing their collision graph.
use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_route_layout::Layout;
use crate::{CompiledGamePack, build_prairie_grass_art_pack, revise_authored_route_terrain};

/// Source inputs for one already installed route's regional grass coloration.
pub struct PrairieGrassRevision<'a> {
    /// Exact previously installed layout.
    pub previous_layout: &'a str,
    /// Revised explicit golden-field layout.
    pub layout: &'a str,
    /// Existing native actors, rooms, encounters and gameplay anchors.
    pub dossier: &'a str,
}

/// Installs native golden grass art and applies authored R02/R16 field regions.
///
/// Both prior layouts must match their installed native maps exactly, including
/// actor/room/party contracts. New terrain must have identical native collision
/// in every quadrant: this operation can color grass, not move paths, gates or
/// encounters. It changes only those routes' tileset attributes/block payloads,
/// adds the regional grass art/field replacement aliases and derives identity.
/// HOME, other maps, source assets, scripts, audio and encounters are retained.
///
/// # Errors
/// Returns an error for stale input, wrong route/order/art identities, changed
/// collision, unsafe geometry, art allocation or final runtime verification.
/// The borrowed pack and all input strings remain unchanged on failure.
pub fn build_prairie_grass_modpack(
    base: &CompiledGamePack,
    revisions: [PrairieGrassRevision<'_>; 2],
) -> Result<CompiledGamePack> {
    for revision in &revisions {
        let checked = revise_authored_route_terrain(
            base,
            revision.previous_layout,
            revision.previous_layout,
            revision.dossier,
        )?;
        ensure!(
            checked == *base,
            "previous prairie source must be an exact no-op"
        );
    }
    let mut pack = build_prairie_grass_art_pack(base)?;
    for (revision, (id, source, target, prefix)) in revisions.into_iter().zip([
        ("MN_R02", "kanto", "mn_prairie_kanto", "MnR02C"),
        ("MN_R16", "mn_headwaters", "mn_prairie_headwaters", "MnR16C"),
    ]) {
        let old = Layout::parse(revision.previous_layout)?;
        let new = Layout::parse(revision.layout)?;
        let original_art: Value = serde_json::from_str(revision.previous_layout)?;
        let revised_art: Value = serde_json::from_str(revision.layout)?;
        let original_recipes = original_art["derived_blocks"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let recipes = revised_art["derived_blocks"]
            .as_array()
            .context("golden prairie source recipes")?;
        ensure!(
            recipes.len() == original_recipes.len() + 2
                && recipes[..original_recipes.len()] == original_recipes,
            "retain existing prairie art recipes exactly"
        );
        let (pixel_source, blocks) = if id == "MN_R02" {
            ("kanto", [("80", "01", "FLOOR"), ("81", "0b", "TALL_GRASS")])
        } else {
            ("johto", [("83", "02", "FLOOR"), ("84", "03", "TALL_GRASS")])
        };
        for (recipe, (block, original, collision)) in
            recipes[original_recipes.len()..].iter().zip(blocks)
        {
            ensure!(
                *recipe
                    == serde_json::json!({"id":block,"source_tileset":pixel_source,"source_block":original,"palette_override":7,"collision":collision,"required_typed_extension":"build_prairie_grass_art_pack"}),
                "golden prairie source recipe must match the native palette and glyphs"
            );
        }
        ensure!(
            old.id == id
                && new.id == id
                && old.compiled_tileset == source
                && new.compiled_tileset == target
                && old.dimensions_metatiles == new.dimensions_metatiles,
            "prairie route/art identity mismatch"
        );
        let old_blocks = old.compile()?;
        let new_blocks = new.compile()?;
        ensure!(
            old_blocks.len() == new_blocks.len(),
            "prairie extent changed"
        );
        for (before, after) in old_blocks.iter().zip(&new_blocks) {
            let allowed = if id == "MN_R02" {
                [(0x01, 0x80), (0x0b, 0x81)]
            } else {
                [(0x02, 0x83), (0x03, 0x84)]
            };
            ensure!(
                before == after || allowed.contains(&(*before, *after)),
                "prairie revision may only replace lawn or encounter grass with its golden equivalent"
            );
            ensure!(
                base.data().tilesets[source].collision[&format!("{before:02x}")]
                    == pack.data().tilesets[target].collision[&format!("{after:02x}")],
                "prairie coloration cannot change native collision"
            );
        }
        for (name, module) in &mut pack.data.maps {
            if !name.starts_with(prefix) {
                continue;
            }
            ensure!(
                module.attributes.tileset_name == source,
                "stale prairie map tileset"
            );
            module.attributes.tileset_name = target.into();
            pack.data
                .map_attributes
                .get_mut(name)
                .context("matching prairie attributes")?
                .tileset_name = target.into();
        }
        pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
            pack.format_version,
            &pack.data,
            &pack.audio_manifest,
            &pack.runtime_files,
            &pack.report,
        )?;
        // The old geometry now uses the already-validated regional art alias.
        // Match raw art metadata so the terrain-only helper still rejects drift.
        let mut prior: Value = serde_json::from_str(revision.previous_layout)?;
        let revised: Value = serde_json::from_str(revision.layout)?;
        prior["compiled_tileset"] = target.into();
        for field in ["tileset", "derived_blocks", "source_pixels_modified"] {
            if let Some(value) = revised.get(field) {
                prior[field] = value.clone();
            }
        }
        pack = revise_authored_route_terrain(
            &pack,
            &serde_json::to_string(&prior)?,
            revision.layout,
            revision.dossier,
        )?;
    }
    Ok(pack)
}
