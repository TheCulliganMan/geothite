//! Native waterfront frontage variety without moving the playable city.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{CompiledGamePack, TilesetDefinition};

const MAP: &str = "MnMinneapolis";
const TILESET: &str = "mn_minneapolis_city";
const PARENT: &str = "1d3d63099937c03ad2c390ef2910ddfe23de232321cfdcb226aa26e250df6347";
const PARENT_FILE: &str = "1f5604a6c743904e5b0c8fbfe36b32ef056f50dab309a42795aa639457ac0bf3";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Contract {
    schema_version: u16,
    id: String,
    base_pack_sha256: String,
    base_identity: String,
    base_module_sha256: String,
    base_definition_sha256: String,
    map: String,
    tileset: String,
    expected_blocks: usize,
    cells: Vec<Cell>,
    glyph_copies: Vec<GlyphCopy>,
    blocks: Vec<Block>,
    source_files_sha256: BTreeMap<String, String>,
    donor_definition_sha256: BTreeMap<String, String>,
    source_art_contract_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    metatile: [u16; 2],
    before: u16,
    after: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GlyphCopy {
    target: u8,
    source_tileset: String,
    source_glyph: u8,
    source_palette: u8,
    pixels_sha256: String,
    four_bank_rgb_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    target: u16,
    glyphs: [u8; 16],
    collision: [String; 4],
}

/// Publishes four reviewed Minneapolis waterfront fronts using native glyphs.
///
/// Reuses the city's slate roof and imports only exact original Johto/Kanto
/// pixels. Repacking the two 96-glyph banks keeps every referenced old logical
/// glyph and all original palette entries exact. Changes only declared facade
/// block IDs and their encoded mirror, plus five private tileset runtime files.
/// All collision quadrants, doors, scripts, rooms, species, recovery, reports
/// and audio remain exact. This does not establish earned progression or visual
/// acceptance; review the written pack with the native renderer separately.
///
/// # Errors
/// Rejects an unreviewed hash, stale parent/preimages, occupied glyphs, changed
/// native art, edits outside the four fronts, changed collision or unrelated
/// pack mutations. The borrowed parent is unchanged on success and failure.
pub fn build_minneapolis_waterfront_pack(
    base: &CompiledGamePack,
    contract_json: &str,
    reviewed_sha256: &str,
) -> Result<CompiledGamePack> {
    ensure!(
        hash(contract_json.as_bytes()) == reviewed_sha256,
        "reviewed waterfront contract hash differs"
    );
    let plan: Contract = serde_json::from_str(contract_json)?;
    ensure!(
        plan.schema_version == 1
            && plan.id == "MINNEAPOLIS_WATERFRONT_V1"
            && plan.map == MAP
            && plan.tileset == TILESET
            && plan.expected_blocks == 169
            && plan.base_identity == PARENT
            && base.identity.content_hash == PARENT
            && plan.base_pack_sha256 == PARENT_FILE
            && base.data.maps.len() == 691,
        "exact pine-knoll parent and four-front contract required"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let original = base.data.maps.get(MAP).context("city module")?;
    let old_definition = base.data.tilesets.get(TILESET).context("city tileset")?;
    ensure!(
        original.attributes.width == 40
            && original.attributes.height == 32
            && hash(&serde_json::to_vec(&serde_json::to_value(original)?)?)
                == plan.base_module_sha256
            && hash(&serde_json::to_vec(&serde_json::to_value(old_definition)?)?)
                == plan.base_definition_sha256
            && base
                .data
                .maps
                .iter()
                .filter(|(_, m)| m.attributes.tileset_name == TILESET)
                .count()
                == 1,
        "whole city/art preimage or private ownership differs"
    );
    ensure!(
        plan.source_art_contract_sha256.len() == 64 && !plan.source_files_sha256.is_empty(),
        "frozen source provenance required"
    );
    for (path, expected) in &plan.source_files_sha256 {
        ensure!(
            hash(base.runtime_files.get(path).context("native source file")?) == *expected,
            "native source changed: {path}"
        );
    }
    for (name, expected) in &plan.donor_definition_sha256 {
        ensure!(
            matches!(name.as_str(), "kanto" | "johto"),
            "unreviewed native donor"
        );
        ensure!(
            hash(&serde_json::to_vec(&serde_json::to_value(
                base.data.tilesets.get(name).context("donor definition")?
            )?)?)
                == *expected,
            "donor palette/collision changed"
        );
    }
    let (definition, metatiles, graphics) = append_art(base, &plan)?;
    let expected_cells = (2..6)
        .map(|x| [x, 12])
        .chain((8..10).map(|x| [x, 13]))
        .chain((19..21).flat_map(|y| (11..13).map(move |x| [x, y])))
        .chain((21..23).flat_map(|y| (2..5).map(move |x| [x, y])))
        .collect::<BTreeSet<[u16; 2]>>();
    ensure!(
        plan.cells.len() == 16
            && plan
                .cells
                .iter()
                .map(|c| c.metatile)
                .collect::<BTreeSet<_>>()
                == expected_cells,
        "exact waterfront facade cells required"
    );
    let mut map = original.clone();
    for cell in &plan.cells {
        let [x, y] = cell.metatile;
        let index = usize::from(y) * 40 + usize::from(x);
        ensure!(
            map.blocks[index] == cell.before,
            "facade cell preimage differs"
        );
        ensure!(
            old_definition
                .collision
                .get(&format!("{:02x}", cell.before))
                == definition.collision.get(&format!("{:02x}", cell.after)),
            "facade collision changed"
        );
        map.blocks[index] = cell.after;
    }
    let label = original
        .attributes
        .blocks_label
        .as_deref()
        .context("city block mirror")?;
    let bytes = map
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut output = base.clone();
    output.data.maps.insert(MAP.into(), map);
    output
        .data
        .map_blocks
        .insert(label.into(), crate::encode_base64_bytes(&bytes));
    let owned = [
        (
            format!("gfx/tilesets/{TILESET}.png"),
            crate::native_tile_png::from_2bpp(&graphics)?,
        ),
        (format!("gfx/tilesets/{TILESET}.2bpp"), graphics),
        (format!("data/tilesets/{TILESET}_metatiles.bin"), metatiles),
        (
            format!("data/tilesets/{TILESET}_palette_map.json"),
            serde_json::to_vec(&definition.palette_map)?,
        ),
        (
            format!("data/tilesets/{TILESET}.json"),
            serde_json::to_vec(&definition.collision)?,
        ),
    ];
    for (path, bytes) in &owned {
        ensure!(
            base.runtime_files.contains_key(path),
            "private art file must already exist"
        );
        output.runtime_files.insert(path.clone(), bytes.clone());
    }
    output.data.tilesets.insert(TILESET.into(), definition);
    output.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        output.format_version,
        &output.data,
        &output.audio_manifest,
        &output.runtime_files,
        &output.report,
    )?;
    let mut normalized = output.clone();
    for (path, _) in owned {
        normalized
            .runtime_files
            .insert(path.clone(), base.runtime_files[&path].clone());
    }
    normalized
        .data
        .tilesets
        .insert(TILESET.into(), old_definition.clone());
    normalized.data.maps.insert(MAP.into(), original.clone());
    normalized
        .data
        .map_blocks
        .insert(label.into(), base.data.map_blocks[label].clone());
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "waterfront changed unrelated gameplay, art, species or audio"
    );
    crate::verify_compiled_game_pack_for_runtime(&output)?;
    Ok(output)
}

fn append_art(
    base: &CompiledGamePack,
    plan: &Contract,
) -> Result<(TilesetDefinition, Vec<u8>, Vec<u8>)> {
    let mut definition = base.data.tilesets[TILESET].clone();
    let mut metatiles =
        base.runtime_files[&format!("data/tilesets/{TILESET}_metatiles.bin")].clone();
    let original = &base.runtime_files[&format!("gfx/tilesets/{TILESET}.2bpp")];
    ensure!(
        metatiles.len() == 169 * 16
            && original.len() == 192 * 16
            && definition.palette_map.len() == 224
            && definition.collision.len() == 169,
        "native city art extent differs"
    );
    ensure!(
        !plan.blocks.is_empty() && plan.blocks.len() <= 24 && plan.glyph_copies.len() <= 32,
        "bounded native art append required"
    );
    let mut graphics = vec![0; 256 * 16];
    // Preserve the valid original bank0 aliases96..127 as well as the actual
    // bank1. The old two96 banks become two128 banks; merely padding the PNG
    // would silently shift every referenced high-bank source glyph.
    graphics[..128 * 16].copy_from_slice(&original[..128 * 16]);
    graphics[128 * 16..224 * 16].copy_from_slice(&original[96 * 16..192 * 16]);
    for &glyph in metatiles.iter().collect::<BTreeSet<_>>() {
        let palette = definition.palette_map[usize::from(glyph)];
        let old = slot(glyph, palette, 192)?;
        let new = slot(glyph, palette, 256)?;
        ensure!(
            original[old * 16..old * 16 + 16] == graphics[new * 16..new * 16 + 16],
            "old logical city glyph changed"
        );
    }
    definition.palette_map.resize(256, 0);
    for (index, copy) in plan.glyph_copies.iter().enumerate() {
        ensure!(
            usize::from(copy.target) == 224 + index
                && !metatiles.contains(&copy.target)
                && copy.four_bank_rgb_sha256.len() == 64,
            "new logical glyph address occupied"
        );
        ensure!(
            plan.donor_definition_sha256
                .contains_key(&copy.source_tileset),
            "native glyph donor not frozen"
        );
        let donor = base
            .data
            .tilesets
            .get(&copy.source_tileset)
            .context("native glyph donor")?;
        ensure!(
            donor.palette_map.get(usize::from(copy.source_glyph)) == Some(&copy.source_palette),
            "native donor palette differs"
        );
        let pixels = base
            .runtime_files
            .get(&format!("gfx/tilesets/{}.2bpp", copy.source_tileset))
            .context("donor pixels")?;
        let from = slot(copy.source_glyph, copy.source_palette, pixels.len() / 16)? * 16;
        let pixels = pixels
            .get(from..from + 16)
            .context("complete native glyph")?;
        ensure!(
            hash(pixels) == copy.pixels_sha256,
            "native donor pixels differ"
        );
        let to = usize::from(copy.target) * 16;
        graphics[to..to + 16].copy_from_slice(pixels);
        definition.palette_map[usize::from(copy.target)] = (copy.source_palette & 7) | 8;
    }
    let known = metatiles
        .iter()
        .copied()
        .chain(plan.glyph_copies.iter().map(|g| g.target))
        .collect::<BTreeSet<_>>();
    for (index, recipe) in plan.blocks.iter().enumerate() {
        ensure!(
            usize::from(recipe.target) == 169 + index
                && recipe.glyphs.iter().all(|g| known.contains(g)),
            "metatile must append exact existing/copied native glyphs"
        );
        metatiles.extend(recipe.glyphs);
        ensure!(
            definition
                .collision
                .insert(format!("{:02x}", recipe.target), recipe.collision.to_vec())
                .is_none(),
            "new recipe address occupied"
        );
    }
    Ok((definition, metatiles, graphics))
}

fn slot(glyph: u8, palette: u8, count: usize) -> Result<usize> {
    let slot = if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        usize::from(glyph & 127) + count / 2
    };
    ensure!(slot < count, "native glyph bank points outside graphics");
    Ok(slot)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
