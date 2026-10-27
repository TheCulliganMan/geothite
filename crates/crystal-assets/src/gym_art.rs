//! Compose native source pixels into small gym tilesets with explicit behavior.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::gym_design::Gym;
use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

/// Installs native glyphs, retaining source collision except declared wind slats.
pub(crate) fn install(pack: &mut CompiledGamePack, gym: &Gym) -> Result<BTreeMap<char, u16>> {
    let tileset_id = gym.tileset();
    ensure!(
        !pack.data.tilesets.contains_key(&tileset_id),
        "gym art already installed"
    );
    let mut definition = pack
        .data
        .tilesets
        .get("elite_four_room")
        .context("native gym art")?
        .clone();
    definition.collision.clear();
    definition.palette_map.fill(0);
    let mut graphics = vec![0_u8; 192 * 16];
    let mut metatiles = Vec::new();
    let mut relocated = BTreeMap::new();
    let mut symbols = BTreeMap::new();
    let mut next = 0_usize;
    let needed: BTreeSet<_> = gym
        .rows
        .iter()
        .flat_map(|row| row.chars())
        .chain(gym.gates.iter().map(|gate| gate.open_symbol))
        .chain(gym.cut_gates.iter().map(|gate| gate.replacement_symbol))
        .chain(['#'])
        .collect();
    for (symbol, [source, id]) in gym.art.iter().filter(|(symbol, _)| needed.contains(symbol)) {
        let source_block = usize::from(u16::from_str_radix(id, 16).context("source metatile id")?);
        let files = pack.runtime_files();
        ensure!(
            !files.contains_key(&format!("gfx/tilesets/{source}.pal")),
            "gym recipes require the shared native palette"
        );
        let source_definition = pack
            .data
            .tilesets
            .get(source)
            .context("source tileset definition")?;
        let source_blocks = files
            .get(&format!("data/tilesets/{source}_metatiles.bin"))
            .context("source metatiles")?;
        let source_graphics = files
            .get(&format!("gfx/tilesets/{source}.2bpp"))
            .context("source glyphs")?;
        let target_id = u16::try_from(symbols.len())?;
        for glyph in source_blocks
            .get(source_block * 16..(source_block + 1) * 16)
            .context("source block bounds")?
        {
            let key = (source.as_str(), *glyph);
            let target = if let Some(target) = relocated.get(&key) {
                *target
            } else {
                ensure!(next < 192, "selected gym art exceeds native glyph capacity");
                let palette = source_definition.palette_map[usize::from(*glyph)];
                let physical = if palette & 8 == 0 {
                    usize::from(*glyph)
                } else {
                    usize::from(*glyph & 0x7f) + source_graphics.len() / 32
                };
                let pixels = source_graphics
                    .get(physical * 16..(physical + 1) * 16)
                    .context("source glyph addressing")?;
                let (logical, bank) = if next < 96 {
                    (next, 0)
                } else {
                    ((next - 96) | 0x80, 8)
                };
                graphics[next * 16..(next + 1) * 16].copy_from_slice(pixels);
                definition.palette_map[logical] = (palette & 7) | bank;
                let logical = u8::try_from(logical)?;
                relocated.insert(key, logical);
                next += 1;
                logical
            };
            metatiles.push(target);
        }
        let mut collision = source_definition
            .collision
            .get(id)
            .context("native source collision")?
            .clone();
        if let Some(wind) = gym.winds.get(symbol) {
            ensure!(
                source == "tower"
                    && id == "0b"
                    && matches!(wind.as_str(), "WALK_RIGHT" | "WALK_LEFT"),
                "wind requires native slat art and horizontal walk collision"
            );
            collision.fill(wind.clone());
        }
        definition
            .collision
            .insert(format!("{target_id:02x}"), collision);
        symbols.insert(*symbol, target_id);
    }
    let png = native_tile_png::from_2bpp(&graphics)?;
    let extension = CompiledTilesetExtension {
        manifest_id: format!("minnesota-gym-{}-art-v1", gym.slot),
        tileset_id,
        behavior_source_tileset_id: Some("elite_four_room".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: png,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    crate::gym_cut::install(pack, gym, &symbols)?;
    Ok(symbols)
}
