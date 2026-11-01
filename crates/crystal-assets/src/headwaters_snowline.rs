//! Frozen native Bemidji-to-Leech winter terraces and thawing lake portages.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const SOURCE_HASH: &str = "efac4a83335a645163ceb89c364a31c4efdd8461b5011731369bb30bec3739c5";
const MARKER: &str = "minnesota-headwaters-snowline-v1";
const ALIASES: [(&str, &str); 3] = [
    ("mn_winter_headwaters", "mn_headwaters"),
    ("mn_headwaters_thaw", "mn_headwaters"),
    (
        "mn_winter_headwaters_stairs",
        "mn_outdoor_headwaters_stairs",
    ),
];

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .with_context(|| format!("snowline {key}"))
}

fn rows(value: &Value, width: u16, height: u16) -> Result<Vec<u16>> {
    let lines = value.as_array().context("complete snowline rows")?;
    ensure!(
        lines.len() == usize::from(height),
        "snowline height differs"
    );
    let mut blocks = Vec::new();
    for row in lines {
        let words = row
            .as_str()
            .context("native row")?
            .split_whitespace()
            .collect::<Vec<_>>();
        ensure!(words.len() == usize::from(width), "snowline width differs");
        for word in words {
            blocks.push(u16::from_str_radix(word, 16)?);
        }
    }
    Ok(blocks)
}

/// Builds the four-chunk winter-to-green Leech Lake approach from frozen rows.
///
/// Snow remains ordinary floor. Every original collision quadrant, fitted stair,
/// Cut tree, lake, actor, script, reward and connection stays intact. Cold/melt
/// grass receives native Cut replacements immediately. Returns an unselected
/// candidate; native presentation and controller verification remain separate.
/// Existing saved block overrides are not migrated.
///
/// # Errors
/// Rejects source/parent/map/art drift, changed collision or field-move gates,
/// occupied aliases, unsafe native glyph storage and unrelated residual changes.
pub fn build_headwaters_snowline(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(source.as_bytes()) == SOURCE_HASH,
        "reviewed Headwaters source differs"
    );
    let plan: Value = serde_json::from_str(source)?;
    ensure!(
        plan["schema_version"] == 1
            && serde_json::to_value(&base.identity)? == plan["parent_identity"]
            && !base.report.manifests.iter().any(|m| m == MARKER),
        "new bound Headwaters parent required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let mut child = base.clone();
    crate::tamarack_snowline_art::install_with_aliases(
        &mut child,
        &plan["art"],
        &plan["art_result"],
        "mn_headwaters",
        "mn_winter_headwaters",
        "mn_headwaters_thaw",
        "minnesota-headwaters-thaw-art-v1",
    )?;
    crate::inland_winter_belt::install_source_bound_alias(&mut child, &plan["stair_art"])?;

    let animation_path = text(&plan["inherited_animations"], "source")?;
    let animation_bytes = base
        .runtime_files
        .get(animation_path)
        .context("native inherited animations")?;
    ensure!(
        hash(animation_bytes) == text(&plan["inherited_animations"], "sha256")?,
        "native Headwaters clips differ"
    );
    let animations: BTreeMap<u8, Value> = serde_json::from_slice(animation_bytes)?;
    let mut animated = animations
        .keys()
        .copied()
        .chain([3, 20])
        .collect::<Vec<_>>();
    animated.sort_unstable();
    animated.dedup();
    ensure!(
        serde_json::to_value(&animated)? == plan["inherited_animations"]["glyphs"],
        "all inherited clips required"
    );
    let backgrounds = animated
        .into_iter()
        .map(|glyph| {
            (
                glyph,
                base.data.tilesets["mn_outdoor_headwaters_stairs"].palette_map[usize::from(glyph)]
                    & 7,
            )
        })
        .collect::<BTreeMap<_, _>>();
    ensure!(
        serde_json::to_value(&backgrounds)?
            == plan["inherited_animations"]["full_stair_backgrounds"],
        "native animated backgrounds differ"
    );
    child.runtime_files.insert(
        "data/tilesets/mn_winter_headwaters_stairs_native_backgrounds.json".into(),
        serde_json::to_vec(&backgrounds)?,
    );

    let expected = [
        (
            "MnR08C00R00",
            [32, 24],
            "mn_outdoor_headwaters_stairs",
            "mn_winter_headwaters_stairs",
        ),
        (
            "MnR08C00R01",
            [32, 24],
            "mn_headwaters",
            "mn_headwaters_thaw",
        ),
        (
            "MnR08C01R00",
            [8, 24],
            "mn_headwaters",
            "mn_winter_headwaters",
        ),
        (
            "MnR08C01R01",
            [8, 24],
            "mn_headwaters",
            "mn_headwaters_thaw",
        ),
    ];
    let maps = plan["maps"].as_array().context("four complete chunks")?;
    ensure!(
        maps.len() == expected.len(),
        "complete Headwaters scope required"
    );
    let mut field_gates = 0;
    for (record, &(name, size, original_alias, alias)) in maps.iter().zip(&expected) {
        let original = base
            .data
            .maps
            .get(name)
            .context("original Headwaters module")?;
        ensure!(
            record["id"] == name
                && record["source_tileset"] == original_alias
                && record["target_tileset"] == alias
                && record["dimensions_metatiles"] == serde_json::to_value(size)?
                && [original.attributes.width, original.attributes.height] == size
                && original.attributes.tileset_name == original_alias
                && hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                    == text(record, "native_module_sha256")?
                && original.blocks == rows(&record["before_rows"], size[0], size[1])?,
            "complete Headwaters preimage differs"
        );
        let blocks = rows(&record["after_rows"], size[0], size[1])?;
        for (&before, &after) in original.blocks.iter().zip(&blocks) {
            let old = &base.data.tilesets[original_alias].collision[&format!("{before:02x}")];
            let new = &child.data.tilesets[alias].collision[&format!("{after:02x}")];
            ensure!(
                old == new,
                "Headwaters snow changed a native walking quadrant"
            );
            if old
                .iter()
                .any(|kind| matches!(kind.as_str(), "CUT_TREE" | "WHIRLPOOL"))
            {
                ensure!(
                    name == "MnR08C00R00"
                        && before == 0x63
                        && after == before
                        && child.data.field_moves.cut.replacements[alias].get(&before)
                            == base.data.field_moves.cut.replacements[original_alias].get(&before),
                    "only the unchanged original cold lake-pier gate is permitted"
                );
                field_gates += 1;
            }
        }
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("native block label")?;
        let raw = blocks
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
            .insert(name.into(), map.attributes.clone());
        child.data.maps.insert(name.into(), map);
        child
            .data
            .map_blocks
            .insert(label.clone(), crate::encode_base64_bytes(&raw));
    }
    ensure!(field_gates == 1, "original lake-pier Cut gate required");
    let prototype = base.data.field_moves.cut.replacements["mn_headwaters"][&3].clone();
    ensure!(
        prototype.replacement_block_id == 2 && prototype.variant == "grass",
        "native grass Cut required"
    );
    for (grass, floor) in [(0x88, 0x87), (0x9f, 0x9e)] {
        let mut rule = prototype.clone();
        rule.replacement_block_id = floor;
        ensure!(
            child
                .data
                .field_moves
                .cut
                .replacements
                .get_mut("mn_headwaters_thaw")
                .context("native Cut inheritance")?
                .insert(grass, rule)
                .is_none(),
            "new grass rule occupied"
        );
    }
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
    for &(name, _, _, _) in &expected {
        let original = &base.data.maps[name];
        let label = original
            .attributes
            .blocks_label
            .as_ref()
            .context("original blocks")?;
        residual.data.maps.insert(name.into(), original.clone());
        residual
            .data
            .map_attributes
            // An older stair overlay updated the module but retained the
            // original table alias. Restore each preimage independently.
            .insert(name.into(), base.data.map_attributes[name].clone());
        residual
            .data
            .map_blocks
            .insert(label.clone(), base.data.map_blocks[label].clone());
    }
    for (alias, original) in ALIASES {
        residual
            .data
            .tilesets
            .remove(alias)
            .context("new native winter alias")?;
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
                .context("new native winter file")?;
        }
        let mut cut = residual
            .data
            .field_moves
            .cut
            .replacements
            .remove(alias)
            .context("native grass/tree rules")?;
        if alias == "mn_headwaters_thaw" {
            for (grass, floor) in [(0x88, 0x87), (0x9f, 0x9e)] {
                let mut rule = prototype.clone();
                rule.replacement_block_id = floor;
                ensure!(
                    cut.remove(&grass) == Some(rule),
                    "new cold/melt Cut differs"
                );
            }
        }
        ensure!(
            Some(&cut) == base.data.field_moves.cut.replacements.get(original),
            "original tree/badge/Cut rules differ"
        );
        ensure!(
            residual
                .data
                .field_moves
                .whirlpool
                .replacements
                .remove(alias)
                .as_ref()
                == base.data.field_moves.whirlpool.replacements.get(original),
            "native Whirlpool inheritance differs"
        );
    }
    residual.report = base.report.clone();
    residual.identity = base.identity.clone();
    ensure!(
        residual.runtime_files == base.runtime_files,
        "unrelated native runtime files changed: {:?}",
        residual
            .runtime_files
            .keys()
            .chain(base.runtime_files.keys())
            .filter(|path| residual.runtime_files.get(*path) != base.runtime_files.get(*path))
            .collect::<Vec<_>>()
    );
    ensure!(
        residual.data.maps == base.data.maps,
        "unrelated complete modules changed"
    );
    ensure!(
        residual.data.map_attributes == base.data.map_attributes,
        "unrelated map attribute table changed"
    );
    ensure!(
        residual.data.map_blocks == base.data.map_blocks,
        "unrelated block table changed"
    );
    ensure!(
        residual == *base,
        "unrelated content changed during Headwaters composition"
    );
    Ok(child)
}
