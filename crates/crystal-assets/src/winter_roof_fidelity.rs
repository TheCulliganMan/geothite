//! Snow only the native canopy embedded above two existing cold-region roofs.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;

const MAP: &str = "MnR11C00R00";
const TILESET: &str = "mn_winter_north_shore";
const CONTRACT_SHA: &str = "db9a077f82b7a307c5df41a4c2751bf64d5f88064d405ee40dd7306d214d96eb";
const BASE_IDENTITY: &str = "ab1c60993ffc1bb0d4c3ef4c02f2455208cadc939faee3ffa5ce310ebbb61805";
const BASE_FILE: &str = "aa1aa251f7383640412b82b9b3ed9d6e0bb1304569230a79624c75274db60e5a";
const GLYPHS: [(u8, u8); 4] = [(0x1e, 0xfa), (0x1f, 0xfb), (0x3e, 0xfc), (0x3f, 0xfd)];
const CELLS: [(usize, u16, u16); 2] = [(299, 0x2c, 0x89), (300, 0x2d, 0x8a)];
type Foregrounds = BTreeMap<u8, [[u8; 3]; 3]>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    base_pack_sha256: String,
    base_identity: String,
    base_map_count: usize,
    map: String,
    tileset: String,
    base_module_sha256: String,
    base_definition_sha256: String,
    source_files_sha256: BTreeMap<String, String>,
    expected_blocks: usize,
    glyphs: Vec<Glyph>,
    blocks: Vec<Block>,
    cells: Vec<Cell>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Glyph {
    source: u8,
    target: u8,
    remapped_pixels_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    source: u16,
    target: u16,
    glyphs: Vec<u8>,
    collision: [String; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    metatile: [u16; 2],
    before: u16,
    after: u16,
}

/// Appends two canopy-only roof recipes and applies their exact two cold cells.
///
/// Requires the frozen 683-map northern-winter parent and reviewed contract.
/// Four unused private glyph slots retain native roof-tree silhouettes using
/// the already installed winter-canopy shade mapping and foreground profile.
/// Original metatile recipes, old glyph pixels/palettes, roof timber, doors,
/// collision semantics, map attributes, actors, events, programs, report and
/// audio remain exact. Only the owned art append, two map blocks and encoded
/// block mirror plus derived identity change. The caller owns publication and
/// strict written readback. This does not establish controller/GPU acceptance.
///
/// # Errors
/// Rejects stale contract/source preimages, occupied private glyphs, changed
/// collision, unrelated mutations or inconsistent cached/uncached geometry.
/// The borrowed parent is unchanged on success or failure.
pub fn build_winter_roof_fidelity_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "exact reviewed roof contract required"
    );
    let plan: Contract = serde_json::from_str(contract_json).context("roof fidelity contract")?;
    ensure!(
        plan.schema_version == 1
            && plan.base_pack_sha256 == BASE_FILE
            && plan.base_identity == BASE_IDENTITY
            && base.identity.content_hash == BASE_IDENTITY
            && plan.base_map_count == 683
            && base.data.maps.len() == 683
            && plan.map == MAP
            && plan.tileset == TILESET
            && plan.expected_blocks == 137
            && plan.source_files_sha256.len() == 8
            && plan.glyphs.len() == 4
            && plan.blocks.len() == 2
            && plan.cells.len() == 2,
        "exact northern-winter roof parent/profile required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let original = base.data.maps.get(MAP).context("cold roof module")?;
    let old_definition = base
        .data
        .tilesets
        .get(TILESET)
        .context("winter shore art")?;
    ensure!(
        hash(&serde_json::to_vec(&serde_json::to_value(original)?)?) == plan.base_module_sha256
            && hash(&serde_json::to_vec(&serde_json::to_value(old_definition)?)?)
                == plan.base_definition_sha256
            && original.attributes.tileset_name == TILESET
            && [original.attributes.width, original.attributes.height] == [32, 24],
        "whole roof module/art preimage differs"
    );
    for (path, expected) in &plan.source_files_sha256 {
        ensure!(
            hash(base.runtime_files.get(path).context("roof source file")?) == *expected,
            "roof source preimage changed: {path}"
        );
    }
    let path = |suffix: &str| format!("data/tilesets/{TILESET}{suffix}");
    let old_metatiles = base
        .runtime_files
        .get(&path("_metatiles.bin"))
        .context("winter metatiles")?;
    let old_graphics = base
        .runtime_files
        .get(&format!("gfx/tilesets/{TILESET}.2bpp"))
        .context("winter graphics")?;
    ensure!(
        old_metatiles.len() == 137 * 16
            && old_graphics.len() == 256 * 16
            && old_definition.palette_map.len() == 256,
        "winter roof art extent changed"
    );
    ensure!(
        base.runtime_files
            .get(&format!("gfx/tilesets/{TILESET}.png"))
            == Some(&crate::native_tile_png::from_2bpp(old_graphics)?),
        "winter PNG differs from native bitplanes"
    );
    let mut foregrounds: Foregrounds = serde_json::from_slice(
        base.runtime_files
            .get(&path("_foregrounds.json"))
            .context("winter foregrounds")?,
    )?;
    let tree_colors = *foregrounds
        .get(&old_metatiles[0x86 * 16])
        .context("reviewed snow-canopy profile")?;
    let donor = base
        .data
        .tilesets
        .get("mn_north_shore")
        .context("native shore donor")?;
    let donor_pixels = base
        .runtime_files
        .get("gfx/tilesets/mn_north_shore.2bpp")
        .context("native roof-tree pixels")?;
    let mut definition = old_definition.clone();
    let mut graphics = old_graphics.clone();
    let mut mapping = BTreeMap::new();
    for (glyph, (source, target)) in plan.glyphs.iter().zip(GLYPHS) {
        ensure!(
            glyph.source == source
                && glyph.target == target
                && !old_metatiles.contains(&target)
                && definition.palette_map[usize::from(target)] == 0
                && !foregrounds.contains_key(&target)
                && graphics[usize::from(target) * 16..usize::from(target + 1) * 16]
                    .iter()
                    .all(|v| *v == 0),
            "roof glyph slot is occupied or unreviewed"
        );
        let attribute = *donor
            .palette_map
            .get(usize::from(source))
            .context("native tree palette")?;
        ensure!(attribute & 7 == 2, "roof donor must be native canopy");
        let slot = if attribute & 8 == 0 {
            usize::from(source)
        } else {
            usize::from(source & 127) + donor_pixels.len() / 32
        };
        let pixels = remap(
            donor_pixels
                .get(slot * 16..(slot + 1) * 16)
                .context("native roof glyph")?,
        )?;
        ensure!(
            hash(&pixels) == glyph.remapped_pixels_sha256,
            "native roof remap differs"
        );
        graphics[usize::from(target) * 16..usize::from(target + 1) * 16].copy_from_slice(&pixels);
        definition.palette_map[usize::from(target)] = 11;
        foregrounds.insert(target, tree_colors);
        mapping.insert(source, target);
    }
    let mut metatiles = old_metatiles.clone();
    for (recipe, (source, target)) in plan.blocks.iter().zip([(0x2c, 0x89), (0x2d, 0x8a)]) {
        let key = format!("{source:02x}");
        ensure!(
            recipe.source == source
                && recipe.target == target
                && metatiles.len() == usize::from(target) * 16
                && recipe.collision.iter().map(String::as_str).eq([
                    "HEADBUTT_TREE",
                    "HEADBUTT_TREE",
                    "WALL",
                    "WALL"
                ])
                && old_definition.collision.get(&key).map(Vec::as_slice)
                    == Some(recipe.collision.as_slice()),
            "roof recipe/collision changed"
        );
        let original = &old_metatiles[usize::from(source) * 16..usize::from(source + 1) * 16];
        let glyphs = original
            .iter()
            .enumerate()
            .map(|(i, g)| {
                if i < 8 {
                    mapping
                        .get(g)
                        .copied()
                        .context("unreviewed top canopy glyph")
                } else {
                    Ok(*g)
                }
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            glyphs == recipe.glyphs && glyphs[8..] == original[8..],
            "roof timber pixels changed"
        );
        metatiles.extend_from_slice(&glyphs);
        ensure!(
            definition
                .collision
                .insert(format!("{target:02x}"), recipe.collision.to_vec())
                .is_none(),
            "roof target block occupied"
        );
    }
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("roof map block label")?;
    ensure!(
        base.data
            .maps
            .iter()
            .all(|(name, m)| name == MAP || m.attributes.blocks_label.as_deref() != Some(label)),
        "roof map payload shared"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(label)
                .context("roof map payload")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(original.blocks.iter().copied()),
        "old roof block mirror differs"
    );
    let mut module = original.clone();
    for ((index, before, after), cell) in CELLS.into_iter().zip(&plan.cells) {
        ensure!(
            cell.metatile == [u16::try_from(index % 32)?, 9]
                && cell.before == before
                && cell.after == after
                && module.blocks[index] == before,
            "unreviewed roof cell"
        );
        module.blocks[index] = after;
    }
    let bytes = module
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut output = base.clone();
    output.data.maps.insert(MAP.into(), module);
    output
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
    output
        .runtime_files
        .insert(path("_metatiles.bin"), metatiles);
    output
        .runtime_files
        .insert(format!("gfx/tilesets/{TILESET}.2bpp"), graphics.clone());
    output.runtime_files.insert(
        format!("gfx/tilesets/{TILESET}.png"),
        crate::native_tile_png::from_2bpp(&graphics)?,
    );
    output
        .runtime_files
        .insert(path("_foregrounds.json"), serde_json::to_vec(&foregrounds)?);
    output.runtime_files.insert(
        path("_palette_map.json"),
        serde_json::to_vec(&definition.palette_map)?,
    );
    output
        .runtime_files
        .insert(path(".json"), serde_json::to_vec(&definition.collision)?);
    output.data.tilesets.insert(TILESET.into(), definition);
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    validation::preserve(base, &output)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn remap(raw: &[u8]) -> Result<[u8; 16]> {
    ensure!(raw.len() == 16, "native roof glyph extent");
    let mut result = [0; 16];
    for y in 0..8 {
        for x in 0..8 {
            let bit = 7 - x;
            let index = (raw[y * 2] >> bit & 1) | ((raw[y * 2 + 1] >> bit & 1) << 1);
            let level = [1, 2, 3, 3][usize::from(index)];
            result[y * 2] |= (level & 1) << bit;
            result[y * 2 + 1] |= (level >> 1) << bit;
        }
    }
    Ok(result)
}

#[path = "winter_roof_fidelity_validation.rs"]
mod validation;

#[cfg(test)]
#[path = "winter_roof_fidelity_tests.rs"]
mod tests;
