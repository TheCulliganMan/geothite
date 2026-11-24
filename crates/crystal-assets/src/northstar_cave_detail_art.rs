//! Native metatile recipes and small original glyphs for authored cave features.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TilesetArt {
    tileset: String,
    metatiles: Vec<Metatile>,
    glyphs: Vec<Glyph>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metatile {
    id: u8,
    glyphs: [u8; 16],
    collision: [String; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Glyph {
    id: u8,
    palette: u8,
    rows: [String; 8],
}

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    sources: &[&str],
    allowed: &BTreeSet<&str>,
) -> Result<()> {
    for source in sources {
        for art in serde_json::from_str::<Vec<TilesetArt>>(source)? {
            ensure!(
                allowed.contains(art.tileset.as_str()),
                "unexpected cave tileset {}",
                art.tileset
            );
            install_bank(pack, art)?;
        }
    }
    Ok(())
}

fn install_bank(pack: &mut CompiledGamePack, art: TilesetArt) -> Result<()> {
    let TilesetArt {
        tileset,
        metatiles,
        glyphs,
    } = art;
    let def = pack
        .data
        .tilesets
        .get_mut(&tileset)
        .context("cave tileset")?;
    let layout_path = format!("data/tilesets/{tileset}_metatiles.bin");
    let mut layout = pack
        .runtime_files
        .get(&layout_path)
        .context("cave layouts")?
        .clone();
    let original_glyphs = layout.iter().copied().collect::<BTreeSet<_>>();
    let graphics_path = format!("gfx/tilesets/{tileset}.2bpp");
    let graphics = pack
        .runtime_files
        .get_mut(&graphics_path)
        .context("cave graphics")?;
    ensure!(
        layout.len().is_multiple_of(16),
        "incomplete metatiles in {tileset}"
    );
    let glyph_count = graphics.len() / 16;
    let physical_address = |id: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(id)
        } else {
            usize::from(id & 127) + glyph_count / 2
        }
    };
    let occupied = original_glyphs
        .iter()
        .filter_map(|&id| {
            def.palette_map
                .get(usize::from(id))
                .map(|&palette| physical_address(id, palette))
        })
        .collect::<BTreeSet<_>>();
    let mut fresh = BTreeSet::new();
    for glyph in &glyphs {
        let slot = usize::from(glyph.id);
        ensure!(
            glyph.id < 128
                && glyph.palette < 8
                && !original_glyphs.contains(&glyph.id)
                && !occupied.contains(&slot)
                && fresh.insert(glyph.id),
            "occupied or invalid cave glyph {tileset}:{:02x}",
            glyph.id
        );
        let palette = def
            .palette_map
            .get_mut(slot)
            .context("cave glyph palette slot")?;
        let ink = graphics
            .get_mut(slot * 16..(slot + 1) * 16)
            .context("cave glyph slot")?;
        ink.copy_from_slice(&bitplanes(&glyph.rows)?);
        *palette = glyph.palette;
    }
    let mut referenced = BTreeSet::new();
    for block in metatiles {
        ensure!(
            layout.len() == usize::from(block.id) * 16,
            "cave metatiles must append in order in {tileset}"
        );
        for glyph in block.glyphs {
            ensure!(
                original_glyphs.contains(&glyph) || fresh.contains(&glyph),
                "unknown cave glyph {tileset}:{glyph:02x}"
            );
            referenced.insert(glyph);
        }
        for token in &block.collision {
            crate::resolve_collision_token(token)?;
        }
        ensure!(
            def.collision
                .insert(format!("{:02x}", block.id), block.collision.into())
                .is_none(),
            "occupied cave collision in {tileset}"
        );
        layout.extend_from_slice(&block.glyphs);
    }
    ensure!(fresh.is_subset(&referenced), "unused glyph in {tileset}");
    if !glyphs.is_empty() {
        let png = crate::native_tile_png::from_2bpp(graphics)?;
        pack.runtime_files
            .insert(format!("gfx/tilesets/{tileset}.png"), png);
        pack.runtime_files.insert(
            format!("data/tilesets/{tileset}_palette_map.json"),
            serde_json::to_vec(&def.palette_map)?,
        );
    }
    pack.runtime_files.insert(layout_path, layout);
    pack.runtime_files.insert(
        format!("data/tilesets/{tileset}.json"),
        serde_json::to_vec(&def.collision)?,
    );
    Ok(())
}

fn bitplanes(rows: &[String; 8]) -> Result<[u8; 16]> {
    let mut ink = [0; 16];
    for (y, row) in rows.iter().enumerate() {
        ensure!(row.len() == 8, "native glyph row width");
        for (x, pixel) in row.bytes().enumerate() {
            ensure!((b'0'..=b'3').contains(&pixel), "native glyph palette index");
            let shade = pixel - b'0';
            ink[y * 2] |= (shade & 1) << (7 - x);
            ink[y * 2 + 1] |= (shade >> 1) << (7 - x);
        }
    }
    Ok(ink)
}
