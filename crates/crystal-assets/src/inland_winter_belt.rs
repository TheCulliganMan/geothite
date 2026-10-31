//! Private native snow art for the explicitly bound inland Minnesota exteriors.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, CompiledTilesetExtension};

const MARKER: &str = "minnesota-inland-winter-belt-v1";
type Inks = [[u8; 3]; 3];
type Foregrounds = BTreeMap<u8, Inks>;

#[derive(Deserialize)]
struct Proposal {
    schema_version: u8,
    art: Vec<Alias>,
    maps: Vec<Exterior>,
}

#[derive(Deserialize)]
struct Alias {
    source: String,
    target: String,
    source_blocks: usize,
    source_definition_sha256: String,
    source_runtime_sha256: BTreeMap<String, String>,
    ground_reference_block: String,
    recipes: Vec<Recipe>,
    maps: Vec<String>,
}

#[derive(Deserialize)]
struct Recipe {
    block: String,
    kind: String,
    collision: Vec<String>,
    donor_tileset: Option<String>,
    donor_block: Option<String>,
}

#[derive(Deserialize)]
struct Exterior {
    id: String,
    dimensions_metatiles: [u16; 2],
    source_tileset: String,
    target_tileset: String,
    native_module_sha256: String,
    rows: Vec<String>,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn physical(glyph: u8, palette: u8, count: usize) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + count / 2
    }
}

fn native_day(raw: &[u8]) -> Result<Vec<[u8; 3]>> {
    let mut active = false;
    let mut components = Vec::new();
    for line in std::str::from_utf8(raw)?.lines() {
        let line = line.trim();
        if let Some(bank) = line.strip_prefix(';') {
            let bank = bank.trim();
            if matches!(bank, "morn" | "day" | "nite" | "dark" | "indoor") {
                active = bank == "day";
            }
        }
        if active
            && let Some(values) = line
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .strip_prefix("RGB")
        {
            for component in values.split(',') {
                let value = component
                    .trim()
                    .parse::<u8>()
                    .context("native palette component")?;
                ensure!(
                    value <= 31,
                    "native palette must retain five-bit components"
                );
                components.push(value);
            }
        }
    }
    ensure!(
        components.len() == 96,
        "eight complete native day palettes required"
    );
    Ok(components
        .chunks_exact(3)
        .map(|rgb| [rgb[0], rgb[1], rgb[2]])
        .collect())
}

fn foregrounds(pack: &CompiledGamePack, tileset: &str) -> Result<Foregrounds> {
    pack.runtime_files
        .get(&format!("data/tilesets/{tileset}_foregrounds.json"))
        .map(|bytes| serde_json::from_slice(bytes))
        .transpose()
        .context("native foreground palette map")
        .map(Option::unwrap_or_default)
}

fn donor(
    pack: &CompiledGamePack,
    tileset: &str,
    block: usize,
    day: &[[u8; 3]],
) -> Result<Vec<([u8; 16], Inks)>> {
    let definition = pack
        .data
        .tilesets
        .get(tileset)
        .context("native donor definition")?;
    let metatiles = pack
        .runtime_files
        .get(&format!("data/tilesets/{tileset}_metatiles.bin"))
        .context("native donor metatiles")?;
    let graphics = pack
        .runtime_files
        .get(&format!("gfx/tilesets/{tileset}.2bpp"))
        .context("native donor bitplanes")?;
    let foregrounds = foregrounds(pack, tileset)?;
    let mut parts = Vec::new();
    for &glyph in metatiles
        .get(block * 16..block * 16 + 16)
        .context("bounded native donor block")?
    {
        let palette = *definition
            .palette_map
            .get(usize::from(glyph))
            .context("native donor palette")?;
        let slot = physical(glyph, palette, graphics.len() / 16);
        let raw = graphics
            .get(slot * 16..slot * 16 + 16)
            .context("native donor glyph address")?;
        let colors = if let Some(&colors) = foregrounds.get(&glyph) {
            colors
        } else {
            let start = usize::from(palette & 7) * 4 + 1;
            std::array::from_fn(|index| day[start + index].map(|value| (value << 3) | (value >> 2)))
        };
        parts.push((
            raw.try_into().context("sixteen native glyph bytes")?,
            colors,
        ));
    }
    Ok(parts)
}

fn remap(raw: [u8; 16], map: [u8; 4]) -> [u8; 16] {
    let mut result = [0; 16];
    for row in 0..8 {
        for bit in 0..8 {
            let old = ((raw[row * 2] >> bit) & 1) | (((raw[row * 2 + 1] >> bit) & 1) << 1);
            let value = map[usize::from(old)];
            result[row * 2] |= (value & 1) << bit;
            result[row * 2 + 1] |= (value >> 1) << bit;
        }
    }
    result
}

fn install(pack: &mut CompiledGamePack, alias: &Alias, day: &[[u8; 3]]) -> Result<()> {
    ensure!(
        alias.target.starts_with("mn_winter_") && alias.source != alias.target,
        "private winter identity required"
    );
    let original = pack
        .data
        .tilesets
        .get(&alias.source)
        .context("installed alias source")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
            == alias.source_definition_sha256,
        "complete native source definition changed"
    );
    for (path, expected) in &alias.source_runtime_sha256 {
        ensure!(
            hash(
                pack.runtime_files
                    .get(path)
                    .context("bound native source file")?
            ) == *expected,
            "native source file changed: {path}"
        );
    }
    let old_metatiles = pack
        .runtime_files
        .get(&format!("data/tilesets/{}_metatiles.bin", alias.source))
        .context("source native metatiles")?;
    let old_graphics = pack
        .runtime_files
        .get(&format!("gfx/tilesets/{}.2bpp", alias.source))
        .context("source native graphics")?;
    let count = old_graphics.len() / 16;
    ensure!(
        matches!(count, 192 | 256) && old_metatiles.len() == alias.source_blocks * 16,
        "source bank/block extent changed"
    );
    let mut definition = original.clone();
    definition.palette_map.resize(256, 0);
    let mut graphics = vec![0; 256 * 16];
    if count == 192 {
        graphics[..96 * 16].copy_from_slice(&old_graphics[..96 * 16]);
        graphics[128 * 16..224 * 16].copy_from_slice(&old_graphics[96 * 16..]);
    } else {
        graphics.copy_from_slice(old_graphics);
    }
    let used = old_metatiles.iter().copied().collect::<BTreeSet<_>>();
    let protected = used
        .iter()
        .map(|&glyph| physical(glyph, definition.palette_map[usize::from(glyph)], 256))
        .collect::<BTreeSet<_>>();
    let mut colors = foregrounds(pack, &alias.source)?;
    let preferred = (224..256).chain(96..128).chain(0..96).chain(128..224);
    let mut free = preferred
        .filter(|&slot| {
            !used.contains(&(slot as u8))
                && !protected.contains(&slot)
                && !colors.contains_key(&(slot as u8))
                && graphics[slot * 16..slot * 16 + 16]
                    .iter()
                    .all(|&byte| byte == 0)
        })
        .collect::<Vec<_>>()
        .into_iter();
    let mut shared = BTreeMap::new();
    for (&glyph, &inks) in &colors {
        let slot = physical(glyph, definition.palette_map[usize::from(glyph)], 256);
        shared.insert((graphics[slot * 16..slot * 16 + 16].to_vec(), inks), glyph);
    }
    let canopy = donor(pack, "mn_winter_north_shore", 0x8b, day)?[0].1;
    let frost = donor(pack, "mn_winter_outdoor", 0x3d, day)?[0].1;
    let ground = usize::from(u8::from_str_radix(&alias.ground_reference_block, 16)?);
    let ground_glyphs = old_metatiles
        .get(ground * 16..ground * 16 + 16)
        .context("native ground reference")?
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut metatiles = old_metatiles.clone();
    let mut changed = BTreeSet::new();
    for recipe in &alias.recipes {
        let block = usize::from(u8::from_str_radix(&recipe.block, 16)?);
        ensure!(
            changed.insert(block)
                && block < alias.source_blocks
                && original.collision.get(&recipe.block) == Some(&recipe.collision),
            "native recipe collision/preimage changed"
        );
        let parts = match recipe.kind.as_str() {
            "native_copy" => {
                let tileset = recipe
                    .donor_tileset
                    .as_deref()
                    .context("native copy donor")?;
                let key = recipe.donor_block.as_deref().context("native copy block")?;
                ensure!(
                    pack.data
                        .tilesets
                        .get(tileset)
                        .and_then(|d| d.collision.get(key))
                        == Some(&recipe.collision),
                    "native donor changes traversal"
                );
                donor(
                    pack,
                    tileset,
                    usize::from(u8::from_str_radix(key, 16)?),
                    day,
                )?
            }
            "frost_flowers" => donor(pack, &alias.source, block, day)?
                .into_iter()
                .map(|(raw, _)| (raw, frost))
                .collect(),
            "mixed_foliage" => donor(pack, &alias.source, block, day)?
                .into_iter()
                .enumerate()
                .map(|(index, (raw, inks))| {
                    let glyph = old_metatiles[block * 16 + index];
                    if original.palette_map[usize::from(glyph)] & 7 != 2 {
                        (raw, inks)
                    } else if ground_glyphs.contains(&glyph) {
                        (raw, frost)
                    } else {
                        (remap(raw, [1, 2, 3, 3]), canopy)
                    }
                })
                .collect(),
            _ => anyhow::bail!("unsupported explicit native winter recipe"),
        };
        for (index, (raw, inks)) in parts.into_iter().enumerate() {
            let glyph = old_metatiles[block * 16 + index];
            if recipe.kind == "mixed_foliage" && original.palette_map[usize::from(glyph)] & 7 != 2 {
                continue;
            }
            let key = (raw.to_vec(), inks);
            let destination = if let Some(&glyph) = shared.get(&key) {
                glyph
            } else {
                let slot = free
                    .next()
                    .context("private alias exhausted blank native glyph capacity")?;
                graphics[slot * 16..slot * 16 + 16].copy_from_slice(&raw);
                let glyph = u8::try_from(slot)?;
                definition.palette_map[slot] = 3 | if slot >= 128 { 8 } else { 0 };
                colors.insert(glyph, inks);
                shared.insert(key, glyph);
                glyph
            };
            metatiles[block * 16 + index] = destination;
        }
    }
    ensure!(
        definition.collision == original.collision,
        "snow may not change collision"
    );
    let extension = CompiledTilesetExtension {
        manifest_id: format!("{}-art-v1", alias.target.replace('_', "-")),
        tileset_id: alias.target.clone(),
        behavior_source_tileset_id: Some(alias.source.clone()),
        definition,
        metatiles,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    let mut snow_day = day.to_vec();
    for index in (0..32).step_by(4) {
        snow_day[index] = [29, 31, 31];
    }
    let mut palette = String::from("; Native day inks with snow background0.\n");
    for bank in snow_day.chunks_exact(4) {
        palette.push_str("RGB ");
        palette.push_str(
            &bank
                .iter()
                .flatten()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        );
        palette.push('\n');
    }
    pack.runtime_files.insert(
        format!("gfx/tilesets/{}.pal", alias.target),
        palette.into_bytes(),
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{}_foregrounds.json", alias.target),
        serde_json::to_vec(&colors)?,
    );
    Ok(())
}

/// Builds private snow aliases for complete explicitly bound inland exteriors.
///
/// Original block IDs, collision, scripts, encounters, indexed anchors, Cut and
/// Whirlpool replacements stay intact. All unrelated maps and native art/audio
/// remain exact. Snow is ordinary floor; this overlay adds no sliding ice.
/// Returns a verified candidate only; renderer seasonal registration, visual and
/// production gameplay acceptance, selection and save migration are separate.
///
/// # Errors
/// Rejects stale complete source/map hashes, a warm excluded coast/preserve map,
/// changed rows/collision, duplicate aliases, insufficient glyph space or any
/// unrelated change after normalizing the declared aliases and map bindings.
pub fn build_inland_winter_belt(
    base: &CompiledGamePack,
    proposal_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let plan: Proposal = serde_json::from_str(proposal_json)?;
    ensure!(
        plan.schema_version == 1
            && !plan.art.is_empty()
            && !plan.maps.is_empty()
            && !base.report.manifests.iter().any(|marker| marker == MARKER),
        "new explicit winter proposal required"
    );
    let names = plan
        .maps
        .iter()
        .map(|map| map.id.as_str())
        .collect::<BTreeSet<_>>();
    let aliases = plan
        .art
        .iter()
        .map(|alias| alias.target.as_str())
        .collect::<BTreeSet<_>>();
    ensure!(
        names.len() == plan.maps.len() && aliases.len() == plan.art.len(),
        "duplicate winter identity"
    );
    ensure!(
        names.iter().all(|name| matches!(
            *name,
            "MnBemidji" | "MnBemidjiNorthwoods" | "MnGrandRapids"
        ) || name.starts_with("MnR09C")
            || name.starts_with("MnR17C")),
        "proposal includes an unreviewed winter map"
    );
    let day = native_day(
        base.runtime_files
            .get("gfx/tilesets/bg_tiles.pal")
            .context("native day palette")?,
    )?;
    let mut result = base.clone();
    for alias in &plan.art {
        install(&mut result, alias, &day)?;
    }
    for exterior in &plan.maps {
        let original = base
            .data
            .maps
            .get(&exterior.id)
            .context("bound inland map")?;
        ensure!(
            original.attributes.tileset_name == exterior.source_tileset
                && [original.attributes.width, original.attributes.height]
                    == exterior.dimensions_metatiles
                && hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                    == exterior.native_module_sha256,
            "complete inland map preimage changed: {}",
            exterior.id
        );
        let alias = plan
            .art
            .iter()
            .find(|alias| alias.target == exterior.target_tileset)
            .context("declared map alias")?;
        ensure!(
            alias.source == exterior.source_tileset
                && alias.maps.iter().any(|name| name == &exterior.id),
            "source/map alias binding differs"
        );
        let rows = exterior
            .rows
            .iter()
            .map(|row| {
                row.split_whitespace()
                    .map(|word| u16::from_str_radix(word, 16))
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        ensure!(
            rows.len() == usize::from(exterior.dimensions_metatiles[1])
                && rows
                    .iter()
                    .all(|row| row.len() == usize::from(exterior.dimensions_metatiles[0]))
                && rows.concat() == original.blocks,
            "winter may not rewrite terrain or the repaired roof"
        );
        let mut map = original.clone();
        map.attributes.tileset_name = exterior.target_tileset.clone();
        crate::authored_town_publication::replace(&mut result, map)?;
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
    let mut residual = result.clone();
    for exterior in &plan.maps {
        let original = base
            .data
            .maps
            .get(&exterior.id)
            .context("original inland module")?;
        residual
            .data
            .maps
            .insert(exterior.id.clone(), original.clone());
        residual
            .data
            .map_attributes
            .insert(exterior.id.clone(), original.attributes.clone());
    }
    for alias in &plan.art {
        residual
            .data
            .tilesets
            .remove(&alias.target)
            .context("new winter definition")?;
        for path in [
            format!("data/tilesets/{}_metatiles.bin", alias.target),
            format!("data/tilesets/{}.json", alias.target),
            format!("data/tilesets/{}_palette_map.json", alias.target),
            format!("gfx/tilesets/{}.2bpp", alias.target),
            format!("gfx/tilesets/{}.png", alias.target),
            format!("gfx/tilesets/{}.pal", alias.target),
            format!("data/tilesets/{}_foregrounds.json", alias.target),
        ] {
            residual
                .runtime_files
                .remove(&path)
                .context("new winter runtime file")?;
        }
        ensure!(
            residual
                .data
                .field_moves
                .cut
                .replacements
                .remove(&alias.target)
                .as_ref()
                == base.data.field_moves.cut.replacements.get(&alias.source),
            "Cut behavior inheritance differs"
        );
        ensure!(
            residual
                .data
                .field_moves
                .whirlpool
                .replacements
                .remove(&alias.target)
                .as_ref()
                == base
                    .data
                    .field_moves
                    .whirlpool
                    .replacements
                    .get(&alias.source),
            "Whirlpool behavior inheritance differs"
        );
    }
    residual.identity = base.identity.clone();
    residual.report = base.report.clone();
    ensure!(
        residual == *base,
        "unrelated content changed in inland winter overlay"
    );
    Ok(result)
}
