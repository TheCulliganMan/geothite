//! Full residual equality for the two-cell canopy append.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use super::{CELLS, Foregrounds, GLYPHS, MAP, TILESET, remap};
use crate::CompiledGamePack;

pub(super) fn preserve(base: &CompiledGamePack, output: &CompiledGamePack) -> Result<()> {
    let path = |suffix: &str| format!("data/tilesets/{TILESET}{suffix}");
    let old = base.data.tilesets.get(TILESET).context("old roof art")?;
    let metatiles = base
        .runtime_files
        .get(&path("_metatiles.bin"))
        .context("old roof recipes")?;
    ensure!(metatiles.len() == 137 * 16, "old roof recipe extent");
    let mut expected_definition = old.clone();
    let mut expected_metatiles = metatiles.clone();
    let mut expected_graphics = base
        .runtime_files
        .get(&format!("gfx/tilesets/{TILESET}.2bpp"))
        .context("old roof glyphs")?
        .clone();
    let mut expected_foregrounds: Foregrounds = serde_json::from_slice(
        base.runtime_files
            .get(&path("_foregrounds.json"))
            .context("old roof colors")?,
    )?;
    let colors = *expected_foregrounds
        .get(&metatiles[0x86 * 16])
        .context("reviewed snow canopy")?;
    let donor = base
        .data
        .tilesets
        .get("mn_north_shore")
        .context("native roof donor")?;
    let native = base
        .runtime_files
        .get("gfx/tilesets/mn_north_shore.2bpp")
        .context("native roof source")?;
    let mapping = BTreeMap::from(GLYPHS);
    for (source, target) in GLYPHS {
        let attribute = *donor
            .palette_map
            .get(usize::from(source))
            .context("native roof palette")?;
        let slot = if attribute & 8 == 0 {
            usize::from(source)
        } else {
            usize::from(source & 127) + native.len() / 32
        };
        let pixels = remap(
            native
                .get(slot * 16..(slot + 1) * 16)
                .context("native roof silhouette")?,
        )?;
        expected_graphics[usize::from(target) * 16..usize::from(target + 1) * 16]
            .copy_from_slice(&pixels);
        expected_definition.palette_map[usize::from(target)] = 11;
        expected_foregrounds.insert(target, colors);
    }
    for (_, source, target) in CELLS {
        let glyphs = &metatiles[usize::from(source) * 16..usize::from(source + 1) * 16];
        for (i, glyph) in glyphs.iter().enumerate() {
            expected_metatiles.push(if i < 8 {
                *mapping.get(glyph).context("roof tree reference")?
            } else {
                *glyph
            });
        }
        let collision = old
            .collision
            .get(&format!("{source:02x}"))
            .context("native roof collision")?;
        ensure!(
            expected_definition
                .collision
                .insert(format!("{target:02x}"), collision.clone())
                .is_none(),
            "new roof collision occupied"
        );
    }
    ensure!(
        output.data.tilesets.get(TILESET) == Some(&expected_definition),
        "changed old roof palettes/collisions or unreviewed new recipe"
    );
    let owned_files = [
        (path("_metatiles.bin"), expected_metatiles),
        (
            format!("gfx/tilesets/{TILESET}.2bpp"),
            expected_graphics.clone(),
        ),
        (
            format!("gfx/tilesets/{TILESET}.png"),
            crate::native_tile_png::from_2bpp(&expected_graphics)?,
        ),
        (
            path("_foregrounds.json"),
            serde_json::to_vec(&expected_foregrounds)?,
        ),
        (
            path("_palette_map.json"),
            serde_json::to_vec(&expected_definition.palette_map)?,
        ),
        (
            path(".json"),
            serde_json::to_vec(&expected_definition.collision)?,
        ),
    ];
    let mut normalized = output.clone();
    for (path, expected) in owned_files {
        ensure!(
            output.runtime_files.get(&path) == Some(&expected),
            "roof owned file differs beyond exact append: {path}"
        );
        normalized.runtime_files.insert(
            path.clone(),
            base.runtime_files
                .get(&path)
                .context("old owned roof file")?
                .clone(),
        );
    }
    normalized.data.tilesets.insert(TILESET.into(), old.clone());
    let original = base.data.maps.get(MAP).context("old cold roof map")?;
    let mut expected = original.clone();
    for (index, before, after) in CELLS {
        ensure!(expected.blocks[index] == before, "old roof cell differs");
        expected.blocks[index] = after;
    }
    ensure!(
        output.data.maps.get(MAP) == Some(&expected),
        "roof map changed outside two blocks"
    );
    let label = original
        .attributes
        .blocks_label
        .as_ref()
        .context("roof block label")?;
    let bytes = expected
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        output.data.map_blocks.get(label) == Some(&crate::encode_base64_bytes(&bytes)),
        "roof block mirror differs"
    );
    normalized.data.maps.insert(MAP.into(), original.clone());
    normalized.data.map_blocks.insert(
        label.clone(),
        base.data
            .map_blocks
            .get(label)
            .context("original roof payload")?
            .clone(),
    );
    normalized.identity = base.identity.clone();
    ensure!(
        normalized == *base,
        "roof append changed unrelated maps/programs/audio/report/art"
    );
    let mut before = base.data.clone();
    let mut after = output.data.clone();
    before
        .maps
        .remove(MAP)
        .context("remove original cached roof")?;
    after
        .maps
        .remove(MAP)
        .context("remove candidate cached roof")?;
    let original_wire = before.assemble_map_module_from_compiled_payloads(MAP)?;
    let mut candidate_wire = after.assemble_map_module_from_compiled_payloads(MAP)?;
    ensure!(
        original_wire.blocks == original.blocks && candidate_wire.blocks == expected.blocks,
        "uncached roof geometry differs"
    );
    candidate_wire.blocks = original_wire.blocks.clone();
    ensure!(
        candidate_wire == original_wire,
        "uncached roof gameplay changed beyond geometry"
    );
    Ok(())
}
