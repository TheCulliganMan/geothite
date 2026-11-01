//! Source-bound connected winter-to-preserve terrain, with unchanged traversal.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::systems::field_moves::FieldMoveReplacement;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const SOURCE_HASH: &str = "d34e0e85dcbe8a69f92073c3f17b801161f1bb261b6c0a14cbf81e3c143d7831";
const MARKER: &str = "minnesota-tamarack-snowline-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    parent_identity: Value,
    art: Value,
    art_result: Value,
    maps: Vec<Exterior>,
    #[serde(default)]
    cut_rule_sha256: Option<String>,
    #[serde(default)]
    field_move_variants: Vec<CutVariant>,
}

#[derive(Deserialize)]
struct CutVariant {
    tileset: String,
    source_block: String,
    source_replacement: String,
    block: String,
    replacement: String,
    variant: String,
}

fn cut_variant_rule(
    base: &CompiledGamePack,
    source: &str,
    record: &CutVariant,
) -> Result<FieldMoveReplacement> {
    let block = u16::from_str_radix(&record.source_block, 16)?;
    let original = base
        .data
        .field_moves
        .cut
        .replacements
        .get(source)
        .and_then(|rules| rules.get(&block))
        .context("bound native Cut prototype")?;
    ensure!(
        original.replacement_block_id == u16::from_str_radix(&record.source_replacement, 16)?
            && original.variant == record.variant,
        "original Cut variant differs"
    );
    let mut rule = original.clone();
    rule.replacement_block_id = u16::from_str_radix(&record.replacement, 16)?;
    Ok(rule)
}

#[derive(Deserialize)]
struct Exterior {
    id: String,
    dimensions_metatiles: [u16; 2],
    source_tileset: String,
    target_tileset: String,
    native_module_sha256: String,
    before_rows: Vec<String>,
    after_rows: Vec<String>,
}

fn rows(rows: &[String], size: [u16; 2]) -> Result<Vec<u16>> {
    ensure!(
        rows.len() == usize::from(size[1]),
        "complete snowline height required"
    );
    let mut blocks = Vec::new();
    for row in rows {
        let words = row.split_whitespace().collect::<Vec<_>>();
        ensure!(
            words.len() == usize::from(size[0]),
            "complete snowline width required"
        );
        blocks.extend(
            words
                .into_iter()
                .map(|word| u16::from_str_radix(word, 16))
                .collect::<std::result::Result<Vec<_>, _>>()?,
        );
    }
    Ok(blocks)
}

/// Builds the full four-chunk Tamarack snowline, fading into green preserve land.
///
/// Native zero-index masks and all collision quadrants remain exact. This adds
/// no sliding ice and preserves the previous Bemidji/Iron Range repairs, all
/// actors, scripts, encounters, room pairs, Boundary Waters and native audio.
/// Returns an unselected candidate; runtime presentation and controller evidence
/// are separate from this guarded publication and existing saves are not migrated.
///
/// # Errors
/// Rejects any source/parent/module drift, unsafe native glyph storage, changed
/// gameplay collision or unrelated residual changes. Writes no files.
pub fn build_tamarack_snowline(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    build_bound_snowline(
        base,
        source,
        &SnowlineBinding {
            source_hash: SOURCE_HASH,
            marker: MARKER,
            source_tileset: "mn_headwaters",
            full_alias: "mn_winter_tamarack",
            partial_alias: "mn_tamarack_thaw",
            art_manifest: "minnesota-tamarack-thaw-art-v1",
            maps: &[
                ("MnR20C00R00", [32, 24], "mn_winter_tamarack"),
                ("MnR20C00R01", [32, 8], "mn_winter_tamarack"),
                ("MnR20C01R00", [32, 24], "mn_tamarack_thaw"),
                ("MnR20C01R01", [32, 8], "mn_tamarack_thaw"),
            ],
            unchanged_cut_gates: &[],
            extra_cut_variants: 0,
        },
    )
}

/// Frozen native aliases and complete map scope for a reviewed climate stage.
pub(crate) struct SnowlineBinding<'a> {
    pub source_hash: &'a str,
    pub marker: &'a str,
    pub source_tileset: &'a str,
    pub full_alias: &'a str,
    pub partial_alias: &'a str,
    pub art_manifest: &'a str,
    pub maps: &'a [(&'a str, [u16; 2], &'a str)],
    pub unchanged_cut_gates: &'a [(&'a str, u16)],
    pub extra_cut_variants: usize,
}

/// Applies complete reviewed rows and verifies all unrelated content is exact.
pub(crate) fn build_bound_snowline(
    base: &CompiledGamePack,
    source: &str,
    binding: &SnowlineBinding<'_>,
) -> Result<CompiledGamePack> {
    ensure!(
        format!("{:x}", Sha256::digest(source.as_bytes())) == binding.source_hash,
        "reviewed snowline source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    ensure!(
        plan.schema_version == 1
            && plan.maps.len() == binding.maps.len()
            && plan.field_move_variants.len() == binding.extra_cut_variants
            && serde_json::to_value(&base.identity)? == plan.parent_identity
            && !base.report.manifests.iter().any(|m| m == binding.marker),
        "new source-bound snowline parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    if !plan.field_move_variants.is_empty() {
        let rules = base
            .data
            .field_moves
            .cut
            .replacements
            .get(binding.source_tileset)
            .context("native source Cut rules")?;
        let expected = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&serde_json::to_value(rules)?)?)
        );
        ensure!(
            plan.cut_rule_sha256.as_deref() == Some(expected.as_str()),
            "original Cut catalog differs"
        );
    }
    let mut child = base.clone();
    crate::tamarack_snowline_art::install_with_aliases(
        &mut child,
        &plan.art,
        &plan.art_result,
        binding.source_tileset,
        binding.full_alias,
        binding.partial_alias,
        binding.art_manifest,
    )?;
    let mut seen = BTreeSet::new();
    for exterior in &plan.maps {
        let (_, size, alias) = binding
            .maps
            .iter()
            .find(|(id, _, _)| *id == exterior.id)
            .context("only the bound real snowline chunks may change")?;
        let size = *size;
        let alias = *alias;
        ensure!(
            seen.insert(&exterior.id)
                && exterior.dimensions_metatiles == size
                && exterior.source_tileset == binding.source_tileset
                && exterior.target_tileset == alias,
            "native snowline identity differs"
        );
        let original = base
            .data
            .maps
            .get(&exterior.id)
            .context("original snowline module")?;
        ensure!(
            [original.attributes.width, original.attributes.height] == size
                && original.attributes.tileset_name == exterior.source_tileset
                && original.blocks == rows(&exterior.before_rows, size)?
                && format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&serde_json::to_value(original)?)?)
                ) == exterior.native_module_sha256,
            "complete snowline module preimage differs"
        );
        let blocks = rows(&exterior.after_rows, size)?;
        let old = &base.data.tilesets[&exterior.source_tileset].collision;
        let new = &child.data.tilesets[alias].collision;
        for (&before, &after) in original.blocks.iter().zip(&blocks) {
            let previous = old
                .get(&format!("{before:02x}"))
                .context("original collision")?;
            let current = new
                .get(&format!("{after:02x}"))
                .context("new snow collision")?;
            ensure!(
                previous == current,
                "snow must preserve every native walking quadrant"
            );
            if previous
                .iter()
                .any(|kind| matches!(kind.as_str(), "CUT_TREE" | "WHIRLPOOL"))
            {
                ensure!(
                    before == after
                        && !previous.iter().any(|kind| kind == "WHIRLPOOL")
                        && binding
                            .unchanged_cut_gates
                            .contains(&(exterior.id.as_str(), before))
                        && child
                            .data
                            .field_moves
                            .cut
                            .replacements
                            .get(alias)
                            .and_then(|r| r.get(&before))
                            == base
                                .data
                                .field_moves
                                .cut
                                .replacements
                                .get(binding.source_tileset)
                                .and_then(|r| r.get(&before)),
                    "new field-move block variants require explicit replacement metadata"
                );
            }
        }
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("original snowline blocks label")?;
        let bytes = blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut map = original.clone();
        map.blocks = blocks;
        map.attributes.tileset_name = alias.into();
        child
            .data
            .map_attributes
            .insert(exterior.id.clone(), map.attributes.clone());
        child.data.maps.insert(exterior.id.clone(), map);
        child
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    }
    for record in &plan.field_move_variants {
        ensure!(
            record.tileset == binding.partial_alias,
            "only the bound thaw Cut table may grow"
        );
        let alias = &child.data.tilesets[&record.tileset].collision;
        let original = &base.data.tilesets[binding.source_tileset].collision;
        ensure!(
            alias.get(&record.block) == original.get(&record.source_block)
                && alias.get(&record.replacement) == original.get(&record.source_replacement),
            "Cut must preserve the native source/replacement quadrants"
        );
        let rule = cut_variant_rule(base, binding.source_tileset, record)?;
        ensure!(
            child
                .data
                .field_moves
                .cut
                .replacements
                .get_mut(&record.tileset)
                .context("inherited native Cut table")?
                .insert(u16::from_str_radix(&record.block, 16)?, rule)
                .is_none(),
            "new thaw Cut slot is occupied"
        );
    }
    child.report.manifests.push(binding.marker.into());
    child.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        child.format_version,
        &child.data,
        &child.audio_manifest,
        &child.runtime_files,
        &child.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&child)?;
    crate::validate_compact_superior_geography(&child)?;
    let mut residual = child.clone();
    for exterior in &plan.maps {
        let original = base
            .data
            .maps
            .get(&exterior.id)
            .context("original snowline module")?;
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("original block label")?;
        residual
            .data
            .maps
            .insert(exterior.id.clone(), original.clone());
        residual
            .data
            .map_attributes
            .insert(exterior.id.clone(), original.attributes.clone());
        residual.data.map_blocks.insert(
            label.clone(),
            base.data
                .map_blocks
                .get(label)
                .context("original blocks")?
                .clone(),
        );
    }
    for alias in [binding.full_alias, binding.partial_alias] {
        for record in plan
            .field_move_variants
            .iter()
            .filter(|r| r.tileset == alias)
        {
            let rule = cut_variant_rule(base, binding.source_tileset, record)?;
            ensure!(
                residual
                    .data
                    .field_moves
                    .cut
                    .replacements
                    .get_mut(alias)
                    .context("new Cut table")?
                    .remove(&u16::from_str_radix(&record.block, 16)?)
                    == Some(rule),
                "new Cut variant changed during publication"
            );
        }
        residual
            .data
            .tilesets
            .remove(alias)
            .context("new native alias")?;
        for path in [
            format!("data/tilesets/{alias}.json"),
            format!("data/tilesets/{alias}_metatiles.bin"),
            format!("data/tilesets/{alias}_palette_map.json"),
            format!("data/tilesets/{alias}_foregrounds.json"),
            format!("data/tilesets/{alias}_native_backgrounds.json"),
            format!("gfx/tilesets/{alias}.2bpp"),
            format!("gfx/tilesets/{alias}.png"),
            format!("gfx/tilesets/{alias}.pal"),
        ] {
            residual
                .runtime_files
                .remove(&path)
                .context("new snowline runtime file")?;
        }
        ensure!(
            residual
                .data
                .field_moves
                .cut
                .replacements
                .remove(alias)
                .as_ref()
                == base
                    .data
                    .field_moves
                    .cut
                    .replacements
                    .get(binding.source_tileset),
            "Cut inheritance differs"
        );
        ensure!(
            residual
                .data
                .field_moves
                .whirlpool
                .replacements
                .remove(alias)
                .as_ref()
                == base
                    .data
                    .field_moves
                    .whirlpool
                    .replacements
                    .get(binding.source_tileset),
            "Whirlpool inheritance differs"
        );
    }
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during snowline composition"
    );
    Ok(child)
}
