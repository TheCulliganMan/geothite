//! Join the fair's wide native facades without changing walking geometry.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_city_expansion_art;
use crate::authored_route_layout::Layout;

const MAP: &str = "MnStateFairgrounds";
const TILESET: &str = "mn_state_fair_native";
const MANIFEST: &str = "minnesota-state-fair-contiguous-buildings-v1";
const PATCHES: [(usize, usize, u16, u16); 15] = [
    (6, 8, 0x1e, 0x8b),
    (7, 8, 0x1e, 0x8b),
    (8, 8, 0x1e, 0x8b),
    (9, 8, 0x1e, 0x8b),
    (10, 8, 0x1e, 0x8b),
    (38, 25, 0x1e, 0x8b),
    (39, 25, 0x1e, 0x8b),
    (40, 25, 0x1e, 0x8b),
    (41, 25, 0x1e, 0x8b),
    (4, 24, 0x24, 0x8c),
    (5, 24, 0x24, 0x8c),
    (6, 24, 0x24, 0x8c),
    (4, 25, 0x29, 0x8d),
    (5, 25, 0x29, 0x8d),
    (6, 25, 0x29, 0x8d),
];

#[derive(Deserialize)]
struct Art {
    derived_blocks: Vec<Recipe>,
}

#[derive(Deserialize)]
struct Recipe {
    id: String,
    source_tileset: String,
    source_block: String,
    collision: Vec<String>,
    tile_order: Option<[usize; 16]>,
}

/// Repair fifteen solid facade cells in an already installed State Fair.
///
/// Appends three metatiles assembled from existing Johto glyphs. Original pixels,
/// palettes, collision, doors, actors, counters, encounters, other maps and audio
/// remain unchanged. The complete installed exterior must match the layout's
/// preimage. No files are written.
///
/// # Errors
/// Rejects stale terrain, occupied metatile addresses, non-solid glyph recipes,
/// a shared block label, repeated application or invalid resulting content.
pub fn revise_state_fair_buildings(
    base: &CompiledGamePack,
    layout_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == MANIFEST),
        "fair building repair already installed"
    );
    let layout: Layout = serde_json::from_str(layout_json)?;
    let art: Art = serde_json::from_str(layout_json)?;
    let old = base.data.maps.get(MAP).context("installed State Fair")?;
    ensure!(
        layout.schema_version == 2
            && layout.id == MAP
            && layout.compiled_tileset == TILESET
            && layout.dimensions_metatiles == [48, 36]
            && old.attributes.width == 48
            && old.attributes.height == 36
            && old.attributes.tileset_name == TILESET,
        "fair layout identity or extent changed"
    );
    let after = layout.compile()?;
    let mut before = after.clone();
    for (x, y, pre, post) in PATCHES {
        let index = y * 48 + x;
        ensure!(
            after[index] == post,
            "fair building source drift at {x},{y}"
        );
        before[index] = pre;
    }
    ensure!(
        before == old.blocks,
        "complete fair terrain preimage differs"
    );
    let label = old
        .attributes
        .blocks_label
        .as_ref()
        .context("fair block label")?;
    ensure!(
        base.data.maps.iter().all(|(name, map)| {
            name == MAP || map.attributes.blocks_label.as_ref() != Some(label)
        }),
        "fair shares its block label"
    );
    let mut result = base.clone();
    let path = format!("data/tilesets/{TILESET}_metatiles.bin");
    let mut metatiles = base
        .runtime_files
        .get(&path)
        .context("fair native metatiles")?
        .clone();
    let native = base
        .runtime_files
        .get("data/tilesets/johto_metatiles.bin")
        .context("original Johto metatiles")?;
    ensure!(
        metatiles.len() == 0x8b * 16 && metatiles.get(..native.len()) == Some(native.as_slice()),
        "installed fair art no longer inherits the original Johto glyph addresses"
    );
    let definition = result
        .data
        .tilesets
        .get_mut(TILESET)
        .context("fair tileset")?;
    let recipes = &art.derived_blocks;
    ensure!(
        recipes.len() == 14,
        "fair needs eleven imports and three solid compositions"
    );
    for (offset, recipe) in recipes[11..].iter().enumerate() {
        let id = format!("{:02x}", 0x8b + offset);
        ensure!(
            recipe.id == id && recipe.source_tileset == "johto",
            "fair building recipes must append native Johto blocks in order"
        );
        let source = usize::from(u16::from_str_radix(&recipe.source_block, 16)?);
        let glyphs = native
            .get(source * 16..(source + 1) * 16)
            .context("native facade source")?;
        let collision = base
            .data
            .tilesets
            .get("johto")
            .context("Johto collision")?
            .collision
            .get(&recipe.source_block)
            .context("native facade collision")?;
        ensure!(
            recipe.collision == *collision,
            "fair facade collision differs"
        );
        let order = recipe
            .tile_order
            .as_ref()
            .context("explicit facade glyph composition")?;
        metatiles.extend(authored_city_expansion_art::order_solid_glyphs(
            glyphs, collision, order,
        )?);
        ensure!(
            definition.collision.insert(id, collision.clone()).is_none(),
            "occupied fair building collision"
        );
    }
    for (_, _, pre, post) in PATCHES {
        ensure!(
            definition.collision.get(&format!("{pre:02x}"))
                == definition.collision.get(&format!("{post:02x}")),
            "fair building collision must remain exact"
        );
    }
    result.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    result.runtime_files.insert(path, metatiles);
    result
        .data
        .maps
        .get_mut(MAP)
        .context("fair exterior")?
        .blocks = after.clone();
    let bytes = after
        .into_iter()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    result
        .data
        .map_blocks
        .insert(label.clone(), crate::encode_base64_bytes(&bytes));
    result.report.manifests.push(MANIFEST.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}
