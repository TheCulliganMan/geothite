//! Draw the open night-crew book on the Union Hall table where it can be read.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;
use crate::minnesota_pokemon_pixel_art::bitplanes;
use crate::native_tile_png;

const DRAWING: &str =
    include_str!("../../../planning/minnesota/revisions/return-visits-v01/crew-book.json");
const ATLAS: &str = "data/tilesets/house_metatiles.bin";
const GRAPHICS: &str = "gfx/tilesets/house.2bpp";
const TABLE: usize = 0x21;

/// Adds one table variant at the public book's existing interaction square.
/// Returns an error if the table, free graphic addresses or room have changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnDuluthUnionHall2F")
        .context("Union Hall upstairs")?
        .clone();
    let at = 3 * usize::from(room.attributes.width) + 4;
    ensure!(
        room.attributes.tileset_name == "house"
            && room.blocks.get(at) == Some(&(TABLE as u16))
            && room.events.bg_events.iter().any(|e| e.x == 9
                && e.y == 7
                && e.event_type == "BGEVENT_READ"
                && e.script == "MN_INTERIOR_DETAIL_RESTORED_NAMES"),
        "the readable book moved"
    );
    let mut atlas = pack
        .runtime_files
        .get(ATLAS)
        .context("house block arrangements")?
        .clone();
    let bank = pack
        .data
        .tilesets
        .get_mut("house")
        .context("native house tiles")?;
    let graphic = pack
        .runtime_files
        .get_mut(GRAPHICS)
        .context("house graphic")?;
    const FIRST: u8 = 0xa0;
    const FIRST_SLOT: usize = 128;
    ensure!(
        atlas.len() == 64 * 16 && graphic.len() == 192 * 16 && bank.palette_map.len() == 224,
        "unexpected house data"
    );
    for &glyph in &atlas {
        let palette = bank
            .palette_map
            .get(usize::from(glyph))
            .context("house tile palette")?;
        let slot = if palette & 8 != 0 {
            96 + usize::from(glyph & 0x7f)
        } else {
            usize::from(glyph)
        };
        ensure!(
            !(FIRST_SLOT..FIRST_SLOT + 4).contains(&slot) && !(FIRST..FIRST + 4).contains(&glyph),
            "table graphic aliases an original house tile"
        );
    }
    let slots = &mut graphic[FIRST_SLOT * 16..(FIRST_SLOT + 4) * 16];
    ensure!(
        slots.iter().all(|&b| b == 0),
        "house graphic addresses are occupied"
    );
    let rows: [String; 16] = serde_json::from_str(DRAWING)?;
    ensure!(
        rows.iter()
            .all(|r| r.len() == 16 && r.bytes().all(|b| (b'0'..=b'3').contains(&b))),
        "expected a 16-pixel, four-shade crew book"
    );
    let pixels = rows
        .iter()
        .flat_map(|r| r.bytes().map(|b| b - b'0'))
        .collect::<Vec<_>>();
    slots.copy_from_slice(&bitplanes(&pixels, 16, 16));
    let mut block: [u8; 16] = atlas[TABLE * 16..(TABLE + 1) * 16].try_into()?;
    for (i, tile) in [10, 11, 14, 15].into_iter().enumerate() {
        let glyph = FIRST + i as u8;
        let wood = bank.palette_map[usize::from(block[tile])];
        bank.palette_map[usize::from(glyph)] = 8 | (wood & 7);
        block[tile] = glyph;
    }
    let collision = bank
        .collision
        .get("21")
        .context("original table collision")?
        .clone();
    ensure!(
        bank.collision.insert("40".into(), collision).is_none(),
        "occupied book block"
    );
    atlas.extend(block);
    let sheet = native_tile_png::from_2bpp(graphic)?;
    pack.runtime_files
        .insert("gfx/tilesets/house.png".into(), sheet);
    pack.runtime_files.insert(ATLAS.into(), atlas);
    room.blocks[at] = 0x40;
    authored_town_publication::replace(pack, room)
}
