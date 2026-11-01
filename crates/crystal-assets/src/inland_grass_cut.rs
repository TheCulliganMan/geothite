//! Source-bound native grass Cut for snow and melt habitats, retaining cliffs.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const SOURCE_HASH: &str = "7948e87690a561afb548089d19b5bd3e22c1f3521696a0b8cfa33901ef1988ef";
const MARKER: &str = "minnesota-inland-grass-cut-v1";

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    parent_identity: Value,
    cut_rule_sha256: String,
    tilesets: BTreeMap<String, ArtBinding>,
    rules: Vec<Rule>,
    composites: Vec<Composite>,
}

#[derive(Deserialize)]
struct ArtBinding {
    definition_sha256: String,
    runtime_sha256: BTreeMap<String, String>,
    source_blocks: usize,
}

#[derive(Deserialize)]
struct Rule {
    tileset: String,
    grass_block: String,
    replacement_block: String,
}

#[derive(Deserialize)]
struct Composite {
    tileset: String,
    grass_block: String,
    ground_block: String,
    replacement_block: String,
    collision: Vec<String>,
    native_glyph_rows: Vec<String>,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn block(value: &str) -> Result<usize> {
    usize::from_str_radix(value, 16).context("native grass/floor block")
}

/// Restores native Cut on all placed grass in the new inland winter stages.
///
/// Adds twenty-one grass replacement rules. Twelve appended native metatiles
/// clear only the grass quadrants of existing cliff/grass pieces; all other
/// quadrants and every original layout/graphic byte remain intact. Maps, tree
/// gates, badge requirements, actors, scripts, audio and other regions are exact.
/// Returns an unselected candidate and does not migrate saved terrain overrides.
///
/// # Errors
/// Rejects source/parent/art/catalog drift, occupied rule slots, unsafe glyph
/// recipes, changed non-grass collision or unrelated residual pack changes.
pub fn repair_inland_grass_cut(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE_HASH,
        "reviewed inland grass-Cut source differs"
    );
    let plan: Proposal = serde_json::from_str(source)?;
    let aliases = [
        "mn_iron_descent_thaw",
        "mn_tamarack_thaw",
        "mn_winter_border_pines",
        "mn_winter_border_pines_south",
        "mn_winter_iron_descent",
    ];
    ensure!(
        plan.schema_version == 1
            && plan.rules.len() == 21
            && plan.composites.len() == 12
            && plan.tilesets.keys().map(String::as_str).eq(aliases)
            && serde_json::to_value(&base.identity)? == plan.parent_identity
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "complete source-bound grass-Cut stage required"
    );
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(
            &base.data.field_moves.cut
        )?)?)
            == plan.cut_rule_sha256,
        "original native Cut rule differs"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    for (alias, binding) in &plan.tilesets {
        let definition = base
            .data
            .tilesets
            .get(alias)
            .context("native grass alias")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(definition)?)?)
                == binding.definition_sha256,
            "native grass definition differs"
        );
        for (path, expected) in &binding.runtime_sha256 {
            ensure!(
                hash(
                    base.runtime_files
                        .get(path)
                        .context("bound native grass asset")?
                ) == *expected,
                "native grass asset differs: {path}"
            );
        }
        ensure!(
            base.runtime_files[&format!("data/tilesets/{alias}_metatiles.bin")].len()
                == binding.source_blocks * 16,
            "complete native layout required"
        );
    }
    let prototype = base
        .data
        .field_moves
        .cut
        .replacements
        .get("johto")
        .and_then(|rules| rules.get(&3))
        .context("native Johto grass Cut")?;
    ensure!(
        prototype.replacement_block_id == 2 && prototype.variant == "grass",
        "unchanged native grass behavior required"
    );
    let mut child = base.clone();
    let mut art_changed = BTreeSet::new();
    for composite in &plan.composites {
        ensure!(
            matches!(
                composite.tileset.as_str(),
                "mn_iron_descent_thaw" | "mn_winter_iron_descent"
            ),
            "only descent half-grass art may append blocks"
        );
        let alias = &composite.tileset;
        let grass = block(&composite.grass_block)?;
        let ground = block(&composite.ground_block)?;
        let target = block(&composite.replacement_block)?;
        let path = format!("data/tilesets/{alias}_metatiles.bin");
        let mut layout = child
            .runtime_files
            .get(&path)
            .context("native half-grass layout")?
            .clone();
        ensure!(
            target == layout.len() / 16 && target < 256,
            "append only within native block capacity"
        );
        let definition = child
            .data
            .tilesets
            .get_mut(alias)
            .context("native half-grass definition")?;
        let original = definition
            .collision
            .get(&composite.grass_block)
            .context("original half grass")?;
        ensure!(
            original.len() == 4
                && original.iter().any(|kind| kind == "TALL_GRASS")
                && original.iter().any(|kind| kind == "WALL")
                && definition
                    .collision
                    .get(&composite.ground_block)
                    .is_some_and(|kinds| kinds.iter().map(String::as_str).eq(["FLOOR"; 4])),
            "native half-grass and matching floor required"
        );
        let expected_collision = original
            .iter()
            .map(|kind| {
                if kind == "TALL_GRASS" {
                    "FLOOR".to_owned()
                } else {
                    kind.clone()
                }
            })
            .collect::<Vec<_>>();
        ensure!(
            composite.collision == expected_collision && composite.native_glyph_rows.len() == 4,
            "Cut must retain every non-grass quadrant"
        );
        let mut parts = Vec::with_capacity(16);
        for row in &composite.native_glyph_rows {
            let values = row
                .split_whitespace()
                .map(|word| u8::from_str_radix(word, 16))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            ensure!(values.len() == 4, "four original glyph columns required");
            parts.extend(values);
        }
        for (index, &glyph) in parts.iter().enumerate() {
            let from = if original[(index / 8) * 2 + (index % 4) / 2] == "TALL_GRASS" {
                ground
            } else {
                grass
            };
            ensure!(
                layout.get(from * 16 + index) == Some(&glyph),
                "native wall/ground glyph phase differs"
            );
        }
        ensure!(
            definition
                .collision
                .insert(composite.replacement_block.clone(), expected_collision)
                .is_none(),
            "new floor collision slot is occupied"
        );
        layout.extend(parts);
        child.runtime_files.insert(path, layout);
        child.runtime_files.insert(
            format!("data/tilesets/{alias}.json"),
            serde_json::to_vec(definition)?,
        );
        art_changed.insert(alias.clone());
    }
    let mut seen = BTreeSet::new();
    for rule in &plan.rules {
        ensure!(
            plan.tilesets.contains_key(&rule.tileset)
                && seen.insert((&rule.tileset, &rule.grass_block)),
            "unique bound grass rule required"
        );
        let grass = u16::try_from(block(&rule.grass_block)?)?;
        let target = u16::try_from(block(&rule.replacement_block)?)?;
        let definition = &child.data.tilesets[&rule.tileset];
        let before = definition
            .collision
            .get(&rule.grass_block)
            .context("Cut grass collision")?;
        let after = definition
            .collision
            .get(&rule.replacement_block)
            .context("Cut floor collision")?;
        ensure!(
            before.len() == 4
                && after.len() == 4
                && before.iter().any(|kind| kind == "TALL_GRASS")
                && before
                    .iter()
                    .zip(after)
                    .all(|(before, after)| if before == "TALL_GRASS" {
                        after == "FLOOR"
                    } else {
                        before == after
                    }),
            "grass Cut changes a cliff, tree, water or non-grass quadrant"
        );
        let mut replacement = prototype.clone();
        replacement.replacement_block_id = target;
        ensure!(
            child
                .data
                .field_moves
                .cut
                .replacements
                .entry(rule.tileset.clone())
                .or_default()
                .insert(grass, replacement)
                .is_none(),
            "existing native Cut rule must remain exact"
        );
    }
    let mut restored_cut = child.data.field_moves.cut.clone();
    for rule in &plan.rules {
        let rules = restored_cut
            .replacements
            .get_mut(&rule.tileset)
            .context("new grass rules")?;
        rules
            .remove(&u16::try_from(block(&rule.grass_block)?)?)
            .context("new grass rule")?;
        if rules.is_empty()
            && !base
                .data
                .field_moves
                .cut
                .replacements
                .contains_key(&rule.tileset)
        {
            restored_cut.replacements.remove(&rule.tileset);
        }
    }
    ensure!(
        restored_cut == base.data.field_moves.cut,
        "existing Cut gates or permissions changed"
    );
    child.report.manifests.push(MARKER.into());
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
    residual.data.field_moves.cut = restored_cut;
    for alias in art_changed {
        residual
            .data
            .tilesets
            .insert(alias.clone(), base.data.tilesets[&alias].clone());
        for path in [
            format!("data/tilesets/{alias}.json"),
            format!("data/tilesets/{alias}_metatiles.bin"),
        ] {
            residual
                .runtime_files
                .insert(path.clone(), base.runtime_files[&path].clone());
        }
    }
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual == *base,
        "unrelated content changed during native grass-Cut repair"
    );
    Ok(child)
}
