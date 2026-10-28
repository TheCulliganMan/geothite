//! Selective native snow, encounter grass and sliding ice for outdoor regions.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WinterArt {
    schema_version: u32,
    source_file_sha256: BTreeMap<String, String>,
    source_definition_sha256: BTreeMap<String, String>,
    targets: Vec<Target>,
    recipes: Vec<Recipe>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    source: String,
    target: String,
    expected_blocks: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    name: String,
    source_tileset: String,
    source_block: String,
    collision: [String; 4],
    foregrounds: [[u8; 3]; 3],
    index_map: [u8; 4],
}

/// Adds private outdoor winter tilesets using existing native indexed pixels.
///
/// Original blocks retain their exact glyph addresses, collision and appearance.
/// Only appended snow, tall-grass and striped-ice glyphs receive stable winter
/// foreground colors. The explicit indexed-color map turns source background zero into the
/// authored snow color without changing shared palettes; the two darkest native
/// shades merge in the three-color snow/canopy glyphs. Tall-grass indices
/// remain exact, including transparent zero holes used by native OAM priority. Snow remains FLOOR, icy grass remains TALL_GRASS, and
/// striped ice retains the native ICE forced-movement behavior. The ordinary
/// Cut/Whirlpool replacement metadata is inherited from each source tileset.
/// No map is changed or assigned to these art variants by this operation.
///
/// # Errors
/// Returns an error for source hash drift, an unexpected vocabulary/collision,
/// an occupied target identity, incompatible bank geometry, insufficient glyph
/// capacity or native pack validation failure. The borrowed base is unchanged.
pub fn build_winter_outdoor_art_pack(
    base: &CompiledGamePack,
    specification_json: &str,
) -> Result<CompiledGamePack> {
    let specification: WinterArt =
        serde_json::from_str(specification_json).context("winter art specification")?;
    ensure!(
        specification.schema_version == 1
            && specification.targets.len() == 2
            && specification.recipes.len() == 5
            && specification.source_file_sha256.len() == 8
            && specification.source_definition_sha256.len() == 4,
        "winter art schema/extent changed"
    );
    for (path, expected) in &specification.source_file_sha256 {
        let bytes = base
            .runtime_files()
            .get(path)
            .context("winter art source file")?;
        ensure!(
            format!("{:x}", Sha256::digest(bytes)) == *expected,
            "winter art source preimage changed: {path}"
        );
    }
    for source in ["mn_border_pines", "mn_north_shore", "johto", "ice_path"] {
        let expected = specification
            .source_definition_sha256
            .get(source)
            .context("winter source definition preimage")?;
        let definition = base
            .data
            .tilesets
            .get(source)
            .context("winter source definition")?;
        ensure!(
            format!("{:x}", Sha256::digest(serde_json::to_vec(definition)?)) == *expected,
            "winter source definition changed: {source}"
        );
        for path in [
            format!("gfx/tilesets/{source}.2bpp"),
            format!("data/tilesets/{source}_metatiles.bin"),
        ] {
            ensure!(
                specification.source_file_sha256.contains_key(&path),
                "winter source file preimage omitted: {path}"
            );
        }
    }
    for (recipe, (name, source, block, kind)) in specification.recipes.iter().zip([
        ("WALKABLE_SNOW", "ice_path", "02", "FLOOR"),
        ("ICY_TALL_GRASS", "johto", "03", "TALL_GRASS"),
        ("SLIDING_ICE", "ice_path", "1f", "ICE"),
        ("SNOW_DENSE_CANOPY", "mn_border_pines", "05", "WALL"),
        ("SNOW_SINGLE_TREE", "mn_border_pines", "00", "WALL"),
    ]) {
        ensure!(
            recipe.name == name
                && recipe.source_tileset == source
                && recipe.source_block == block
                && recipe.collision.iter().all(|value| value == kind)
                && recipe.index_map.iter().all(|index| *index <= 3),
            "winter art vocabulary changed: {name}"
        );
    }
    let expected_maps = [
        [1, 2, 3, 3],
        [0, 1, 2, 3],
        [1, 2, 1, 3],
        [1, 2, 3, 3],
        [1, 2, 3, 3],
    ];
    ensure!(
        specification
            .recipes
            .iter()
            .zip(expected_maps)
            .all(|(recipe, map)| recipe.index_map == map),
        "native grass priority holes and ice stripes must survive"
    );
    let mut pack = base.clone();
    for (target, (source, name, count)) in specification.targets.iter().zip([
        ("mn_border_pines", "mn_winter_outdoor", 60),
        ("mn_north_shore", "mn_winter_north_shore", 131),
    ]) {
        ensure!(
            target.source == source && target.target == name && target.expected_blocks == count,
            "winter art target changed"
        );
        pack = append(&pack, target, &specification.recipes)?;
    }
    ensure!(
        base.data.maps == pack.data.maps,
        "winter art changed terrain"
    );
    for (path, bytes) in base.runtime_files() {
        ensure!(
            pack.runtime_files().get(path) == Some(bytes),
            "winter art changed original runtime file: {path}"
        );
    }
    validate_residual(base, &pack)?;
    Ok(pack)
}

fn physical(glyph: u8, palette: u8, count: usize) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + count / 2
    }
}

fn append(
    base: &CompiledGamePack,
    target: &Target,
    recipes: &[Recipe],
) -> Result<CompiledGamePack> {
    let files = base.runtime_files();
    let source = base
        .data
        .tilesets
        .get(&target.source)
        .context("winter source tileset")?;
    let mut definition = source.clone();
    let mut metatiles = files
        .get(&format!("data/tilesets/{}_metatiles.bin", target.source))
        .context("winter source metatiles")?
        .clone();
    let original = files
        .get(&format!("gfx/tilesets/{}.2bpp", target.source))
        .context("winter source pixels")?;
    ensure!(
        metatiles.len() == target.expected_blocks * 16
            && original.len() == 192 * 16
            && definition.palette_map.len() == 224
            && !files.contains_key(&format!("gfx/tilesets/{}.pal", target.source))
            && !files.contains_key(&format!("data/tilesets/{}_foregrounds.json", target.source)),
        "winter source bank geometry/palette changed"
    );
    let used = metatiles.iter().copied().collect::<BTreeSet<_>>();
    for &glyph in &used {
        let attribute = definition.palette_map[usize::from(glyph)];
        ensure!(
            attribute < 16 && (glyph & 127) < 96 && (attribute & 8 != 0 || glyph < 96),
            "winter source glyph convention changed"
        );
    }
    // Expand both banks equally. Logical source addresses remain unchanged;
    // bank-one physical pixels move from96 to128 just as the native renderer
    // resolves a256-glyph sheet. Unused logical addresses hold the new copies.
    let mut graphics = vec![0_u8; 256 * 16];
    graphics[..96 * 16].copy_from_slice(&original[..96 * 16]);
    graphics[128 * 16..224 * 16].copy_from_slice(&original[96 * 16..]);
    // Preserve all224 original palette entries, including unused logical
    // addresses. Only the32 appended logical addresses own winter palettes.
    definition.palette_map.resize(256, 0);
    let mut free = 224_u8..=255;
    let mut shared = BTreeMap::new();
    let mut foregrounds = BTreeMap::new();
    for (offset, recipe) in recipes.iter().enumerate() {
        let donor = base
            .data
            .tilesets
            .get(&recipe.source_tileset)
            .context("winter donor")?;
        let block = usize::from(u8::from_str_radix(&recipe.source_block, 16)?);
        ensure!(
            donor.collision.get(&recipe.source_block).map(Vec::as_slice)
                == Some(recipe.collision.as_slice()),
            "native winter donor collision changed: {}",
            recipe.name
        );
        let pixels = files
            .get(&format!("gfx/tilesets/{}.2bpp", recipe.source_tileset))
            .context("winter donor pixels")?;
        let blocks = files
            .get(&format!(
                "data/tilesets/{}_metatiles.bin",
                recipe.source_tileset
            ))
            .context("winter donor metatiles")?;
        for &glyph in blocks
            .get(block * 16..(block + 1) * 16)
            .context("winter donor block")?
        {
            let slot = physical(
                glyph,
                donor.palette_map[usize::from(glyph)],
                pixels.len() / 16,
            );
            let mut bytes: [u8; 16] = pixels
                .get(slot * 16..(slot + 1) * 16)
                .context("winter donor glyph")?
                .try_into()?;
            remap(&mut bytes, recipe.index_map);
            let key = (bytes, recipe.foregrounds);
            let replacement = if let Some(&existing) = shared.get(&key) {
                existing
            } else {
                let new = free.next().context("insufficient winter glyph space")?;
                graphics[usize::from(new) * 16..(usize::from(new) + 1) * 16]
                    .copy_from_slice(&bytes);
                definition.palette_map[usize::from(new)] = 3 | if new >= 128 { 8 } else { 0 };
                foregrounds.insert(new, recipe.foregrounds);
                shared.insert(key, new);
                new
            };
            metatiles.push(replacement);
        }
        ensure!(
            definition
                .collision
                .insert(
                    format!("{:02x}", target.expected_blocks + offset),
                    recipe.collision.to_vec()
                )
                .is_none(),
            "winter collision address occupied"
        );
    }
    let mut pack = base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: format!("{}-art-v1", target.target.replace('_', "-")),
        tileset_id: target.target.clone(),
        behavior_source_tileset_id: Some(target.source.clone()),
        definition,
        metatiles,
        tile_graphics_png: native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    })?;
    let path = format!("data/tilesets/{}_foregrounds.json", target.target);
    ensure!(
        !pack.runtime_files.contains_key(&path),
        "winter foreground identity occupied"
    );
    pack.runtime_files
        .insert(path, serde_json::to_vec(&foregrounds)?);
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

/// Preserve the native pixel positions while applying the reviewed index map.
fn remap(bytes: &mut [u8; 16], map: [u8; 4]) {
    for row in 0..8 {
        let [low, high] = [bytes[row * 2], bytes[row * 2 + 1]];
        bytes[row * 2] = 0;
        bytes[row * 2 + 1] = 0;
        for column in 0..8 {
            let bit = 7 - column;
            let index = ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
            let mapped = map[usize::from(index)];
            bytes[row * 2] |= (mapped & 1) << bit;
            bytes[row * 2 + 1] |= ((mapped >> 1) & 1) << bit;
        }
    }
}

#[cfg(test)]
#[path = "winter_outdoor_art_tests.rs"]
mod tests;

fn remove_inherited<T: PartialEq>(
    original: &BTreeMap<String, T>,
    actual: &mut BTreeMap<String, T>,
    source: &str,
    target: &str,
) -> Result<()> {
    ensure!(
        !original.contains_key(target),
        "winter inherited behavior target occupied"
    );
    let removed = actual.remove(target);
    ensure!(
        removed.as_ref() == original.get(source),
        "winter inherited behavior differs: {source} to {target}"
    );
    Ok(())
}

fn validate_residual(base: &CompiledGamePack, output: &CompiledGamePack) -> Result<()> {
    let mut normalized = output.clone();
    let mut expected_report = base.report.clone();
    for (source, target) in [
        ("mn_border_pines", "mn_winter_outdoor"),
        ("mn_north_shore", "mn_winter_north_shore"),
    ] {
        ensure!(
            normalized.data.tilesets.remove(target).is_some(),
            "winter tileset missing"
        );
        for path in [
            format!("data/tilesets/{target}_metatiles.bin"),
            format!("data/tilesets/{target}.json"),
            format!("data/tilesets/{target}_palette_map.json"),
            format!("gfx/tilesets/{target}.2bpp"),
            format!("gfx/tilesets/{target}.png"),
            format!("data/tilesets/{target}_foregrounds.json"),
        ] {
            ensure!(
                normalized.runtime_files.remove(&path).is_some(),
                "winter art file missing: {path}"
            );
        }
        remove_inherited(
            &base.data.field_moves.cut.replacements,
            &mut normalized.data.field_moves.cut.replacements,
            source,
            target,
        )?;
        remove_inherited(
            &base.data.field_moves.whirlpool.replacements,
            &mut normalized.data.field_moves.whirlpool.replacements,
            source,
            target,
        )?;
        expected_report
            .manifests
            .push(format!("{}-art-v1", target.replace('_', "-")));
    }
    ensure!(
        output.report == expected_report,
        "winter changed unrelated pack report data"
    );
    normalized
        .report
        .manifests
        .truncate(base.report.manifests.len());
    normalized.identity = base.identity.clone();
    ensure!(normalized == *base, "winter changed unrelated pack content");
    Ok(())
}
