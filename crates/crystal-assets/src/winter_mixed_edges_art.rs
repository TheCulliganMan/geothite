//! Snow-ground native signs and Headbutt edges appended to private winter art.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const MANIFEST: &str = "minnesota-winter-mixed-edges-v1";
type Foregrounds = BTreeMap<u8, [[u8; 3]; 3]>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Specification {
    schema_version: u32,
    source_definition_sha256: BTreeMap<String, String>,
    source_file_sha256: BTreeMap<String, String>,
    targets: Vec<Target>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    tileset: String,
    source: String,
    expected_blocks: usize,
    metatiles_sha256: String,
    graphics_sha256: String,
    foregrounds_sha256: String,
    snow_block: String,
    canopy_block: String,
    ground_glyph: String,
    blocks: Vec<Recipe>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    id: String,
    source_block: String,
    collision: [String; 4],
}

/// Appends nine snow-ground native recipes to the two existing winter aliases.
///
/// Source lawn glyphs become already-reviewed snow glyphs at the same positions.
/// Native Headbutt tree shapes use the existing winter canopy profile; signs use
/// native timber/gray ink over snow. Every collision quadrant is copied exactly.
/// No Cut recipe is imported. Existing recipes, glyph addresses, foregrounds,
/// map terrain, programs, field moves and original source art remain unchanged.
/// Only unused private glyph addresses may receive new pixels/palette entries.
///
/// # Errors
/// Returns an error for incompatible source or private-art preimages, changed
/// vocabulary, Cut collision, exhausted glyph space or failed native validation.
/// The borrowed base is unchanged on success and failure.
pub fn append_winter_mixed_edges_art(
    base: &CompiledGamePack,
    specification_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let specification: Specification =
        serde_json::from_str(specification_json).context("winter mixed-edge specification")?;
    ensure!(
        specification.schema_version == 1
            && specification.targets.len() == 2
            && specification.source_definition_sha256.len() == 4
            && specification.source_file_sha256.len() == 5,
        "winter mixed-edge specification extent changed"
    );
    for name in [
        "mn_border_pines",
        "mn_north_shore",
        "mn_winter_outdoor",
        "mn_winter_north_shore",
    ] {
        let definition = base
            .data
            .tilesets
            .get(name)
            .context("winter edge source definition")?;
        let expected = specification
            .source_definition_sha256
            .get(name)
            .context("winter edge definition preimage omitted")?;
        ensure!(
            hash(&serde_json::to_vec(definition)?) == *expected,
            "winter edge definition preimage changed: {name}"
        );
    }
    for (path, expected) in &specification.source_file_sha256 {
        ensure!(
            hash(
                base.runtime_files
                    .get(path)
                    .context("winter edge source file")?
            ) == *expected,
            "winter edge source file preimage changed: {path}"
        );
    }
    for source in ["mn_border_pines", "mn_north_shore"] {
        for path in [
            format!("gfx/tilesets/{source}.2bpp"),
            format!("data/tilesets/{source}_metatiles.bin"),
        ] {
            ensure!(
                specification.source_file_sha256.contains_key(&path),
                "winter edge donor preimage omitted"
            );
        }
    }
    ensure!(
        specification
            .source_file_sha256
            .contains_key("gfx/tilesets/bg_tiles.pal"),
        "winter edge palette preimage omitted"
    );
    ensure!(
        !base.report.manifests.iter().any(|value| value == MANIFEST),
        "winter mixed-edge art already installed"
    );
    let bank = day_bank(base)?;
    let mut output = base.clone();
    for (target, (name, source, count, snow, canopy, ground, originals)) in
        specification.targets.iter().zip([
            (
                "mn_winter_outdoor",
                "mn_border_pines",
                65,
                "3c",
                "3f",
                "05",
                vec!["13", "0e", "10", "11", "12", "14", "15", "16"],
            ),
            (
                "mn_winter_north_shore",
                "mn_north_shore",
                136,
                "83",
                "86",
                "06",
                vec!["45"],
            ),
        ])
    {
        ensure!(
            target.tileset == name
                && target.source == source
                && target.expected_blocks == count
                && target.snow_block == snow
                && target.canopy_block == canopy
                && target.ground_glyph == ground
                && target
                    .blocks
                    .iter()
                    .map(|recipe| recipe.source_block.as_str())
                    .eq(originals),
            "winter mixed-edge vocabulary changed"
        );
        append(base, &mut output, target, &bank)?;
    }
    output.report.manifests.push(MANIFEST.into());
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    validate_residual(base, &output)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hex(value: &str) -> Result<usize> {
    Ok(usize::from(u8::from_str_radix(value, 16)?))
}
fn physical(glyph: u8, palette: u8, count: usize) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + count / 2
    }
}

fn day_bank(base: &CompiledGamePack) -> Result<Vec<[[u8; 3]; 4]>> {
    let text = std::str::from_utf8(
        base.runtime_files
            .get("gfx/tilesets/bg_tiles.pal")
            .context("native background palette")?,
    )?;
    let mut day = false;
    let mut colors = Vec::new();
    for line in text.lines().map(str::trim) {
        if let Some(section) = line.strip_prefix(';') {
            day = section.trim() == "day";
        }
        if !day {
            continue;
        }
        if let Some(values) = line.strip_prefix("RGB") {
            let values = values.split(';').next().context("palette color line")?;
            let values = values
                .split(',')
                .map(|v| v.trim().parse::<u8>())
                .collect::<std::result::Result<Vec<_>, _>>()?;
            ensure!(
                values.len().is_multiple_of(3) && values.iter().all(|value| *value <= 31),
                "native palette range"
            );
            for rgb in values.as_chunks::<3>().0 {
                colors.push(rgb.map(|v| (v << 3) | (v >> 2)));
            }
        }
    }
    ensure!(
        colors.len() == 32,
        "native day bank must contain eight palettes"
    );
    Ok(colors.as_chunks::<4>().0.to_vec())
}

fn remap(bytes: &[u8]) -> Result<[u8; 16]> {
    ensure!(bytes.len() == 16, "winter edge native glyph extent");
    let mut result = [0_u8; 16];
    for y in 0..8 {
        for x in 0..8 {
            let bit = 7 - x;
            let index = ((bytes[y * 2] >> bit) & 1) | (((bytes[y * 2 + 1] >> bit) & 1) << 1);
            let value = [1, 2, 3, 3][usize::from(index)];
            result[y * 2] |= (value & 1) << bit;
            result[y * 2 + 1] |= (value >> 1) << bit;
        }
    }
    Ok(result)
}

fn append(
    base: &CompiledGamePack,
    output: &mut CompiledGamePack,
    target: &Target,
    bank: &[[[u8; 3]; 4]],
) -> Result<()> {
    let name = &target.tileset;
    let metatile_path = format!("data/tilesets/{name}_metatiles.bin");
    let graphics_path = format!("gfx/tilesets/{name}.2bpp");
    let foreground_path = format!("data/tilesets/{name}_foregrounds.json");
    let old_blocks = base
        .runtime_files
        .get(&metatile_path)
        .context("private winter metatiles")?;
    let old_graphics = base
        .runtime_files
        .get(&graphics_path)
        .context("private winter glyphs")?;
    let mut foregrounds: Foregrounds = serde_json::from_slice(
        base.runtime_files
            .get(&foreground_path)
            .context("private winter foregrounds")?,
    )?;
    ensure!(
        old_blocks.len() == target.expected_blocks * 16
            && old_graphics.len() == 256 * 16
            && hash(old_blocks) == target.metatiles_sha256
            && hash(old_graphics) == target.graphics_sha256
            && hash(&serde_json::to_vec(&foregrounds)?) == target.foregrounds_sha256,
        "private winter art preimage changed"
    );
    ensure!(
        base.runtime_files.get(&format!("gfx/tilesets/{name}.png"))
            == Some(&crate::native_tile_png::from_2bpp(old_graphics)?),
        "private winter PNG preimage differs"
    );
    let mut definition = base
        .data
        .tilesets
        .get(name)
        .context("private winter definition")?
        .clone();
    ensure!(
        definition.palette_map.len() == 256,
        "private winter palette extent changed"
    );
    let mut blocks = old_blocks.clone();
    let mut graphics = old_graphics.clone();
    let used = old_blocks.iter().copied().collect::<BTreeSet<_>>();
    let mut free = (224_u8..=255).filter(|glyph| !used.contains(glyph));
    let mut shared = BTreeMap::new();
    for (&glyph, colors) in &foregrounds {
        let slot = physical(glyph, definition.palette_map[usize::from(glyph)], 256);
        let bytes: [u8; 16] = old_graphics[slot * 16..(slot + 1) * 16].try_into()?;
        shared.insert((bytes, *colors), glyph);
    }
    let snow = &old_blocks[hex(&target.snow_block)? * 16..(hex(&target.snow_block)? + 1) * 16];
    let canopy =
        &old_blocks[hex(&target.canopy_block)? * 16..(hex(&target.canopy_block)? + 1) * 16];
    let white = foregrounds.get(&snow[0]).context("reviewed snow color")?[0];
    let tree_colors = *foregrounds
        .get(&canopy[0])
        .context("reviewed winter canopy colors")?;
    let native = base
        .data
        .tilesets
        .get(&target.source)
        .context("native edge definition")?;
    let native_blocks = base
        .runtime_files
        .get(&format!("data/tilesets/{}_metatiles.bin", target.source))
        .context("native edge metatiles")?;
    let native_graphics = base
        .runtime_files
        .get(&format!("gfx/tilesets/{}.2bpp", target.source))
        .context("native edge pixels")?;
    for recipe in &target.blocks {
        let block = hex(&recipe.source_block)?;
        ensure!(
            hex(&recipe.id)? == blocks.len() / 16
                && native
                    .collision
                    .get(&recipe.source_block)
                    .map(Vec::as_slice)
                    == Some(recipe.collision.as_slice())
                && !recipe.collision.iter().any(|kind| kind == "CUT_TREE"),
            "winter edge collision/address changed"
        );
        for (position, &glyph) in native_blocks
            .get(block * 16..(block + 1) * 16)
            .context("native edge recipe")?
            .iter()
            .enumerate()
        {
            if usize::from(glyph) == hex(&target.ground_glyph)? {
                blocks.push(snow[position]);
                continue;
            }
            let attribute = *native
                .palette_map
                .get(usize::from(glyph))
                .context("native edge palette")?;
            let palette = usize::from(attribute & 7);
            let slot = physical(glyph, attribute, native_graphics.len() / 16);
            let pixels = remap(
                native_graphics
                    .get(slot * 16..(slot + 1) * 16)
                    .context("native edge glyph")?,
            )?;
            let colors = if palette == 2 {
                tree_colors
            } else {
                [white, bank[palette][1], bank[palette][3]]
            };
            let replacement = if let Some(&existing) = shared.get(&(pixels, colors)) {
                existing
            } else {
                let new = free
                    .next()
                    .context("winter edge append exceeds free private glyph space")?;
                ensure!(
                    definition.palette_map[usize::from(new)] == 0
                        && !foregrounds.contains_key(&new)
                        && graphics[usize::from(new) * 16..(usize::from(new) + 1) * 16]
                            .iter()
                            .all(|b| *b == 0),
                    "winter edge glyph destination is occupied"
                );
                graphics[usize::from(new) * 16..(usize::from(new) + 1) * 16]
                    .copy_from_slice(&pixels);
                definition.palette_map[usize::from(new)] = 11;
                foregrounds.insert(new, colors);
                shared.insert((pixels, colors), new);
                new
            };
            blocks.push(replacement);
        }
        ensure!(
            definition
                .collision
                .insert(recipe.id.clone(), recipe.collision.to_vec())
                .is_none(),
            "winter edge collision target occupied"
        );
    }
    ensure!(
        &blocks[..old_blocks.len()] == old_blocks.as_slice()
            && definition.palette_map[..224] == base.data.tilesets[name].palette_map[..224],
        "winter edge original prefix changed"
    );
    for &glyph in &used {
        ensure!(
            definition.palette_map[usize::from(glyph)]
                == base.data.tilesets[name].palette_map[usize::from(glyph)],
            "winter edge altered an existing glyph palette"
        );
        let slot = physical(glyph, definition.palette_map[usize::from(glyph)], 256);
        ensure!(
            graphics[slot * 16..(slot + 1) * 16] == old_graphics[slot * 16..(slot + 1) * 16],
            "winter edge altered existing pixels"
        );
    }
    output.runtime_files.insert(metatile_path, blocks);
    output.runtime_files.insert(graphics_path, graphics.clone());
    output.runtime_files.insert(
        format!("gfx/tilesets/{name}.png"),
        crate::native_tile_png::from_2bpp(&graphics)?,
    );
    output
        .runtime_files
        .insert(foreground_path, serde_json::to_vec(&foregrounds)?);
    output.runtime_files.insert(
        format!("data/tilesets/{name}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    output.runtime_files.insert(
        format!("data/tilesets/{name}_palette_map.json"),
        serde_json::to_vec(&definition.palette_map)?,
    );
    output.data.tilesets.insert(name.clone(), definition);
    Ok(())
}

fn validate_residual(base: &CompiledGamePack, output: &CompiledGamePack) -> Result<()> {
    let mut normalized = output.clone();
    for name in ["mn_winter_outdoor", "mn_winter_north_shore"] {
        validate_owned_prefix(base, output, name)?;
        normalized
            .data
            .tilesets
            .insert(name.into(), base.data.tilesets[name].clone());
        for path in [
            format!("data/tilesets/{name}_metatiles.bin"),
            format!("data/tilesets/{name}.json"),
            format!("data/tilesets/{name}_palette_map.json"),
            format!("data/tilesets/{name}_foregrounds.json"),
            format!("gfx/tilesets/{name}.2bpp"),
            format!("gfx/tilesets/{name}.png"),
        ] {
            normalized
                .runtime_files
                .insert(path.clone(), base.runtime_files[&path].clone());
        }
    }
    ensure!(
        normalized.report.manifests.pop().as_deref() == Some(MANIFEST),
        "winter edge manifest missing"
    );
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "winter mixed-edge art changed unrelated pack content"
    );
    Ok(())
}

#[cfg(test)]
#[path = "winter_mixed_edges_art_tests.rs"]
mod tests;

fn validate_owned_prefix(
    base: &CompiledGamePack,
    output: &CompiledGamePack,
    name: &str,
) -> Result<()> {
    let (first, originals, count) = if name == "mn_winter_outdoor" {
        (65, vec!["13", "0e", "10", "11", "12", "14", "15", "16"], 8)
    } else {
        (136, vec!["45"], 4)
    };
    let source = if name == "mn_winter_outdoor" {
        "mn_border_pines"
    } else {
        "mn_north_shore"
    };
    let metatiles = format!("data/tilesets/{name}_metatiles.bin");
    let graphics = format!("gfx/tilesets/{name}.2bpp");
    let foreground = format!("data/tilesets/{name}_foregrounds.json");
    let old_blocks = &base.runtime_files[&metatiles];
    let new_blocks = &output.runtime_files[&metatiles];
    ensure!(
        new_blocks.len() == (first + originals.len()) * 16
            && &new_blocks[..old_blocks.len()] == old_blocks.as_slice(),
        "winter edge metatile prefix changed"
    );
    let old_definition = &base.data.tilesets[name];
    let mut definition = output.data.tilesets[name].clone();
    for (i, block) in originals.iter().enumerate() {
        ensure!(
            definition
                .collision
                .remove(&format!("{:02x}", first + i))
                .as_ref()
                == base.data.tilesets[source].collision.get(*block),
            "winter edge collision does not match native source"
        );
    }
    let old_foregrounds: Foregrounds = serde_json::from_slice(&base.runtime_files[&foreground])?;
    let mut new_foregrounds: Foregrounds =
        serde_json::from_slice(&output.runtime_files[&foreground])?;
    let added = new_foregrounds
        .keys()
        .filter(|glyph| !old_foregrounds.contains_key(glyph))
        .copied()
        .collect::<Vec<_>>();
    ensure!(
        added.len() == count,
        "winter edge glyph append extent changed"
    );
    let old_graphics = &base.runtime_files[&graphics];
    let mut new_graphics = output.runtime_files[&graphics].clone();
    ensure!(
        new_graphics.len() == old_graphics.len() && definition.palette_map.len() == 256,
        "winter edge native graphics geometry changed"
    );
    ensure!(
        output
            .runtime_files
            .get(&format!("gfx/tilesets/{name}.png"))
            == Some(&crate::native_tile_png::from_2bpp(&new_graphics)?),
        "winter edge PNG differs from native bitplanes"
    );
    for glyph in added {
        let index = usize::from(glyph);
        ensure!(
            glyph >= 224
                && !old_blocks.contains(&glyph)
                && old_definition.palette_map[index] == 0
                && definition.palette_map[index] == 11
                && old_graphics[index * 16..(index + 1) * 16]
                    .iter()
                    .all(|value| *value == 0),
            "winter edge overwrote an occupied glyph"
        );
        new_graphics[index * 16..(index + 1) * 16]
            .copy_from_slice(&old_graphics[index * 16..(index + 1) * 16]);
        definition.palette_map[index] = old_definition.palette_map[index];
        new_foregrounds.remove(&glyph);
    }
    ensure!(
        new_foregrounds == old_foregrounds
            && definition == *old_definition
            && new_graphics == *old_graphics,
        "winter edge altered existing glyph/foreground/collision data"
    );
    ensure!(
        output.runtime_files[&format!("data/tilesets/{name}.json")]
            == serde_json::to_vec(&output.data.tilesets[name].collision)?
            && output.runtime_files[&format!("data/tilesets/{name}_palette_map.json")]
                == serde_json::to_vec(&output.data.tilesets[name].palette_map)?,
        "winter edge raw collision/palette mirrors differ"
    );
    Ok(())
}
