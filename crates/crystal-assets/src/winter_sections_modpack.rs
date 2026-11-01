//! Apply only reviewed winter terrain and private tileset aliases to old routes.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::winter_sections_design::{self as design, Plan};
use crate::{CompiledGamePack, MapModule};

/// Applies the frozen continuous cold sections without rebuilding old gameplay.
///
/// Requires the exact original R17/R11 modules and already installed native
/// winter art plus its mixed-ground append. Sources are keyed by their frozen
/// contract paths. Only four existing chunks' blocks/tileset names and matching
/// attributes/block mirrors change. No actor, program, encounter, Cut rule,
/// room, port, recovery, catalog, audio or runtime file is replaced. Existing
/// cached/split script differences are preserved independently, not repaired.
///
/// # Errors
/// Rejects stale inputs/art/native preimages, unreviewed geometry, unsafe ICE
/// courses, a weakened original gate, unrelated mutation or native validation.
/// The borrowed input remains unchanged; callers own publication and readback.
pub fn build_winter_sections_modpack(
    base: &CompiledGamePack,
    contract_json: &str,
    sources: &BTreeMap<String, String>,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let plan = design::parse(contract_json)?;
    preflight_art(base, &plan)?;
    let mut expected = BTreeMap::new();
    for route in &plan.routes {
        let before = design::compile(sources, route, true)?;
        let after = design::compile(sources, route, false)?;
        for name in [&route.previous_dossier, &route.candidate_dossier] {
            design::source(sources, route, name)?;
        }
        let previous: Value =
            serde_json::from_str(design::source(sources, route, &route.previous_dossier)?)?;
        let mut candidate: Value =
            serde_json::from_str(design::source(sources, route, &route.candidate_dossier)?)?;
        candidate
            .as_object_mut()
            .context("winter dossier object")?
            .remove("winter_section_review");
        let fields = candidate.as_object_mut().context("winter dossier object")?;
        if let Some(original) = previous.get("compiled_tileset") {
            fields.insert("compiled_tileset".into(), original.clone());
        } else {
            fields.remove("compiled_tileset");
        }
        ensure!(candidate == previous, "old route activities/story changed");
        let [width, height] = route.dimensions_metatiles;
        let mut assembled = vec![None; usize::from(width) * usize::from(height)];
        for source in &route.native_modules {
            let old = base
                .data
                .maps
                .get(&source.map)
                .context("original winter native chunk")?;
            ensure!(
                crate::authored_river_connections::module_hash(old)? == source.module_sha256,
                "whole original native winter module changed: {}",
                source.map
            );
            ensure!(
                old.attributes.tileset_name == route.old_tileset,
                "original winter tileset changed"
            );
            let [x0, y0] = [source.column * 32, source.row * 24];
            let mut map = old.clone();
            for y in 0..old.attributes.height {
                for x in 0..old.attributes.width {
                    ensure!(
                        x0 + x < width && y0 + y < height,
                        "winter chunk exceeds whole source"
                    );
                    let local = usize::from(y) * usize::from(old.attributes.width) + usize::from(x);
                    let global = usize::from(y0 + y) * usize::from(width) + usize::from(x0 + x);
                    ensure!(
                        assembled[global].replace(old.blocks[local]).is_none(),
                        "overlapping winter chunks"
                    );
                    map.blocks[local] = after[global];
                }
            }
            let changed = map.blocks != old.blocks;
            ensure!(
                changed == source.retarget,
                "unreviewed winter affected chunk set"
            );
            if changed {
                map.attributes.tileset_name = route.new_tileset.clone();
                ensure!(
                    expected.insert(source.map.clone(), map).is_none(),
                    "duplicate winter target"
                );
            }
        }
        ensure!(
            assembled.into_iter().collect::<Option<Vec<_>>>().as_ref() == Some(&before),
            "whole current native source differs from frozen layout/cap preimage"
        );
        crate::winter_sections_validation::geometry(base, route, &before, &after)?;
    }
    ensure!(
        expected.len() == 4,
        "exact four existing winter chunks required"
    );
    let mut pack = base.clone();
    for (name, map) in &expected {
        let label = map
            .attributes
            .blocks_label
            .as_ref()
            .context("winter native block payload")?;
        ensure!(
            !base
                .data
                .maps
                .values()
                .any(|other| other.id != *name
                    && other.attributes.blocks_label.as_ref() == Some(label)),
            "winter block label shared with unrelated map"
        );
        let bytes = map
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        *pack
            .data
            .map_blocks
            .get_mut(label)
            .context("old winter encoded blocks")? = crate::encode_base64_bytes(&bytes);
        pack.data
            .map_attributes
            .insert(name.clone(), map.attributes.clone());
        pack.data.maps.insert(name.clone(), map.clone());
    }
    preserve(base, &pack, &expected)?;
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

fn preflight_art(base: &CompiledGamePack, plan: &Plan) -> Result<()> {
    let mut aliases = BTreeSet::new();
    for art in &plan.art {
        ensure!(aliases.insert(&art.alias), "duplicate winter art alias");
        let definition = base
            .data
            .tilesets
            .get(&art.alias)
            .context("install winter mixed art before sections")?;
        ensure!(
            design::hash(&serde_json::to_vec(&serde_json::to_value(definition)?)?)
                == art.definition_sha256,
            "installed winter collision/palette definition differs: {}",
            art.alias
        );
        for (path, expected) in &art.files {
            ensure!(
                design::hash(
                    base.runtime_files
                        .get(path)
                        .context("installed winter art file")?
                ) == *expected,
                "installed winter native pixels differ: {path}"
            );
        }
    }
    for route in &plan.routes {
        let old = base
            .data
            .tilesets
            .get(&route.old_tileset)
            .context("native source tileset")?;
        let new = base
            .data
            .tilesets
            .get(&route.new_tileset)
            .context("winter alias")?;
        for (block, collision) in &old.collision {
            ensure!(
                new.collision.get(block) == Some(collision),
                "winter prefix collision changed"
            );
        }
        let original = base
            .data
            .field_moves
            .cut
            .replacements
            .get(&route.old_tileset);
        ensure!(
            base.data
                .field_moves
                .cut
                .replacements
                .get(&route.new_tileset)
                == original,
            "winter alias must retain original native Cut replacements"
        );
    }
    Ok(())
}

fn preserve(
    base: &CompiledGamePack,
    actual: &CompiledGamePack,
    expected: &BTreeMap<String, MapModule>,
) -> Result<()> {
    let mut normalized = actual.data.clone();
    let mut before_wire = base.data.clone();
    let mut after_wire = actual.data.clone();
    for (name, map) in expected {
        let old = base
            .data
            .maps
            .get(name)
            .context("original winter terrain module")?;
        let mut cached = old.clone();
        cached.blocks = map.blocks.clone();
        cached.attributes.tileset_name = map.attributes.tileset_name.clone();
        ensure!(
            cached == *map && actual.data.maps.get(name) == Some(map),
            "old cached winter gameplay changed"
        );
        before_wire
            .maps
            .remove(name)
            .context("remove old cache for native assembly")?;
        after_wire
            .maps
            .remove(name)
            .context("remove new cache for native assembly")?;
        let mut old_assembled = before_wire.assemble_map_module_from_compiled_payloads(name)?;
        let new_assembled = after_wire.assemble_map_module_from_compiled_payloads(name)?;
        old_assembled.blocks = map.blocks.clone();
        old_assembled.attributes.tileset_name = map.attributes.tileset_name.clone();
        ensure!(
            new_assembled == old_assembled,
            "uncached winter wire changed beyond exact terrain/alias: {name}"
        );
        ensure!(
            actual.data.map_attributes.get(name) == Some(&map.attributes),
            "winter attributes mirror differs from prepared module"
        );
        let bytes = map
            .blocks
            .iter()
            .copied()
            .map(u8::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let block_label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("original block label")?;
        ensure!(
            actual.data.map_blocks.get(block_label) == Some(&crate::encode_base64_bytes(&bytes)),
            "winter encoded blocks mirror differs from prepared module"
        );
        before_wire.maps.insert(name.clone(), old.clone());
        after_wire.maps.insert(name.clone(), map.clone());
        normalized.maps.insert(name.clone(), old.clone());
        normalized.map_attributes.insert(
            name.clone(),
            base.data
                .map_attributes
                .get(name)
                .context("original attributes mirror")?
                .clone(),
        );
        let label = old
            .attributes
            .blocks_label
            .as_ref()
            .context("original block mirror")?;
        normalized.map_blocks.insert(
            label.clone(),
            base.data
                .map_blocks
                .get(label)
                .context("old encoded blocks")?
                .clone(),
        );
    }
    ensure!(
        normalized == base.data,
        "unrelated winter maps/catalogs/programs/encounters/flags/recovery changed"
    );
    ensure!(
        actual.runtime_files == base.runtime_files
            && actual.audio_manifest == base.audio_manifest
            && actual.compiled_audio == base.compiled_audio
            && actual.audio_compression == base.audio_compression
            && actual.report == base.report
            && actual.format_version == base.format_version,
        "unrelated winter runtime files/audio/report/format changed"
    );
    Ok(())
}

#[cfg(test)]
#[path = "winter_sections_tests.rs"]
mod tests;
