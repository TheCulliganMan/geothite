//! Complete the native narrow pine silhouette using two unused winter glyphs.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, TilesetDefinition};

const CONTRACT_SHA: &str = "7b6a19623f04ad706259211a9cb6411ae95e50202c87eff67f7430fbb058b36e";
const TILESET: &str = "mn_winter_north_shore";
const RECIPE: [u8; 16] = [
    250, 251, 250, 251, 254, 255, 254, 255, 254, 255, 254, 255, 252, 253, 252, 253,
];
type Foregrounds = BTreeMap<u8, [[u8; 3]; 3]>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    historical_parent_file_sha256: String,
    target_tileset: String,
    source_tileset: String,
    source_definition_sha256: BTreeMap<String, String>,
    source_file_sha256: BTreeMap<String, String>,
    expected_original_blocks: usize,
    glyphs: Vec<Glyph>,
    source_block: u16,
    target_block: u16,
    glyph_recipe: Vec<u8>,
    collision: Vec<String>,
    foregrounds: [[u8; 3]; 3],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Glyph {
    source: u8,
    target: u8,
    remapped_pixels_sha256: String,
}

/// Appends snowy native Johto pine `8b` without assigning it to any map.
///
/// The existing snowy roof recipes already supply four pine tip/base glyphs.
/// Only the two missing trunk glyphs are copied from Johto, shade-remapped using
/// the installed snow-canopy profile, and placed in unused private slots fe/ff.
/// All old glyphs, recipes, palettes, collision, maps, scripts, audio and report
/// remain exact. An approved later parent can compose while all guarded art
/// preimages remain unchanged. The caller binds the actual file hash and owns
/// fresh publication and strict readback. This does not certify terrain use.
///
/// # Errors
/// Rejects stale contract/art/identity, occupied slots, altered native pine
/// geometry, inconsistent mirrors, or unrelated mutations. The base is borrowed
/// and remains unchanged on success and failure.
pub fn build_inland_snow_pines_pack(
    base: &CompiledGamePack,
    contract_json: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == CONTRACT_SHA,
        "frozen pine art contract required"
    );
    let plan: Contract = serde_json::from_str(contract_json).context("inland pine contract")?;
    ensure!(
        plan.schema_version == 1
            && plan.historical_parent_file_sha256
                == "ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8"
            && plan.target_tileset == TILESET
            && plan.source_tileset == "johto"
            && plan.source_definition_sha256.len() == 2
            && plan.source_file_sha256.len() == 9
            && plan.expected_original_blocks == 139
            && plan.source_block == 5
            && plan.target_block == 139
            && plan.glyph_recipe == RECIPE
            && plan.collision.iter().map(String::as_str).eq(["WALL"; 4])
            && plan.glyphs.len() == 2,
        "bounded native pine profile required"
    );
    ensure!(
        base.identity == identity(base)?,
        "pine parent identity is stale"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    for (name, expected) in &plan.source_definition_sha256 {
        let value = base
            .data
            .tilesets
            .get(name)
            .context("pine source definition")?;
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(value)?)?) == *expected,
            "pine definition preimage differs: {name}"
        );
    }
    for (path, expected) in &plan.source_file_sha256 {
        ensure!(
            hash(base.runtime_files.get(path).context("pine source file")?) == *expected,
            "pine art preimage differs: {path}"
        );
    }
    let (definition, files) = art(base, &plan)?;
    let mut output = base.clone();
    output
        .data
        .tilesets
        .insert(TILESET.into(), definition.clone());
    for (path, bytes) in &files {
        output.runtime_files.insert(path.clone(), bytes.clone());
    }
    output.identity = identity(&output)?;
    preserve(base, &output, &definition, &files)?;
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn art(
    base: &CompiledGamePack,
    plan: &Contract,
) -> Result<(TilesetDefinition, BTreeMap<String, Vec<u8>>)> {
    let path = |suffix: &str| format!("data/tilesets/{TILESET}{suffix}");
    let old = base
        .data
        .tilesets
        .get(TILESET)
        .context("winter pine definition")?;
    let mut definition = old.clone();
    let mut metatiles = base
        .runtime_files
        .get(&path("_metatiles.bin"))
        .context("winter recipes")?
        .clone();
    let mut graphics = base
        .runtime_files
        .get(&format!("gfx/tilesets/{TILESET}.2bpp"))
        .context("winter glyphs")?
        .clone();
    let mut foregrounds: Foregrounds = serde_json::from_slice(
        base.runtime_files
            .get(&path("_foregrounds.json"))
            .context("winter colors")?,
    )?;
    ensure!(
        metatiles.len() == 139 * 16
            && graphics.len() == 256 * 16
            && definition.palette_map.len() == 256,
        "pine art extent changed"
    );
    ensure!(
        base.runtime_files
            .get(&format!("gfx/tilesets/{TILESET}.png"))
            == Some(&crate::native_tile_png::from_2bpp(&graphics)?),
        "old winter PNG differs from bitplanes"
    );
    ensure!(
        foregrounds.get(&metatiles[0x86 * 16]) == Some(&plan.foregrounds),
        "existing snow canopy palette differs"
    );
    let donor = base
        .data
        .tilesets
        .get("johto")
        .context("native pine donor")?;
    let native = base
        .runtime_files
        .get("gfx/tilesets/johto.2bpp")
        .context("native pine pixels")?;
    let source_blocks = base
        .runtime_files
        .get("data/tilesets/johto_metatiles.bin")
        .context("native pine recipe")?;
    let native_recipe = [
        30, 31, 30, 31, 46, 47, 46, 47, 46, 47, 46, 47, 62, 63, 62, 63,
    ];
    ensure!(
        source_blocks.get(5 * 16..6 * 16) == Some(native_recipe.as_slice()),
        "native narrow pine shape changed"
    );
    ensure!(
        donor.collision["05"]
            .iter()
            .map(String::as_str)
            .eq(["WALL"; 4]),
        "native pine collision changed"
    );
    for (source, target) in [(0x1e, 0xfa), (0x1f, 0xfb), (0x3e, 0xfc), (0x3f, 0xfd)] {
        let pixels = remap(donor_glyph(native, &donor.palette_map, source)?)?;
        ensure!(
            graphics.get(usize::from(target) * 16..(usize::from(target) + 1) * 16)
                == Some(pixels.as_slice())
                && foregrounds.get(&target) == Some(&plan.foregrounds)
                && definition.palette_map[usize::from(target)] == 11,
            "existing snowy pine tip/base differs"
        );
    }
    for (glyph, (source, target)) in plan.glyphs.iter().zip([(0x2e, 0xfe), (0x2f, 0xff)]) {
        ensure!(
            glyph.source == source
                && glyph.target == target
                && !metatiles.contains(&target)
                && definition.palette_map[usize::from(target)] == 0
                && !foregrounds.contains_key(&target)
                && graphics[usize::from(target) * 16..(usize::from(target) + 1) * 16]
                    .iter()
                    .all(|b| *b == 0),
            "pine private glyph occupied or changed"
        );
        let pixels = remap(donor_glyph(native, &donor.palette_map, source)?)?;
        ensure!(
            hash(&pixels) == glyph.remapped_pixels_sha256,
            "pine remapped bitplanes differ"
        );
        graphics[usize::from(target) * 16..(usize::from(target) + 1) * 16].copy_from_slice(&pixels);
        definition.palette_map[usize::from(target)] = 11;
        foregrounds.insert(target, plan.foregrounds);
    }
    metatiles.extend_from_slice(&RECIPE);
    ensure!(
        definition
            .collision
            .insert("8b".into(), vec!["WALL".into(); 4])
            .is_none(),
        "pine block already occupied"
    );
    let files = BTreeMap::from([
        (path("_metatiles.bin"), metatiles),
        (format!("gfx/tilesets/{TILESET}.2bpp"), graphics.clone()),
        (
            format!("gfx/tilesets/{TILESET}.png"),
            crate::native_tile_png::from_2bpp(&graphics)?,
        ),
        (path("_foregrounds.json"), serde_json::to_vec(&foregrounds)?),
        (
            path("_palette_map.json"),
            serde_json::to_vec(&definition.palette_map)?,
        ),
        (path(".json"), serde_json::to_vec(&definition.collision)?),
    ]);
    Ok((definition, files))
}

fn donor_glyph<'a>(pixels: &'a [u8], palettes: &[u8], source: u8) -> Result<&'a [u8]> {
    let attribute = *palettes
        .get(usize::from(source))
        .context("native pine palette address")?;
    ensure!(attribute & 7 == 2, "native pine donor color changed");
    let slot = if attribute & 8 == 0 {
        usize::from(source)
    } else {
        usize::from(source & 127) + pixels.len() / 32
    };
    pixels
        .get(slot * 16..(slot + 1) * 16)
        .context("native pine glyph extent")
}

fn remap(raw: &[u8]) -> Result<[u8; 16]> {
    ensure!(raw.len() == 16, "native pine bitplane extent");
    let mut output = [0; 16];
    for y in 0..8 {
        for x in 0..8 {
            let bit = 7 - x;
            let index = ((raw[y * 2] >> bit) & 1) | (((raw[y * 2 + 1] >> bit) & 1) << 1);
            let level = [1, 2, 3, 3][usize::from(index)];
            output[y * 2] |= (level & 1) << bit;
            output[y * 2 + 1] |= (level >> 1) << bit;
        }
    }
    Ok(output)
}

fn preserve(
    base: &CompiledGamePack,
    output: &CompiledGamePack,
    definition: &TilesetDefinition,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    ensure!(
        output.data.tilesets.get(TILESET) == Some(definition),
        "pine definition exceeds reviewed append"
    );
    let mut normalized = output.clone();
    for (path, bytes) in files {
        ensure!(
            output.runtime_files.get(path) == Some(bytes),
            "pine owned file differs: {path}"
        );
        normalized.runtime_files.insert(
            path.clone(),
            base.runtime_files
                .get(path)
                .context("old pine owned file")?
                .clone(),
        );
    }
    normalized.data.tilesets.insert(
        TILESET.into(),
        base.data
            .tilesets
            .get(TILESET)
            .context("original pine definition")?
            .clone(),
    );
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "pine art changed unrelated native content"
    );
    Ok(())
}

fn identity(pack: &CompiledGamePack) -> Result<crate::CompiledGamePackIdentity> {
    crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "inland_snow_pines_tests.rs"]
mod tests;
