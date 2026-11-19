//! Put the fen's three observations on their actual examination squares.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, authored_town_publication, native_tile_png};

const MAP: &str = "MnLayeredFen";
const SHEET: &str = "gfx/tilesets/johto.2bpp";
const ATLAS: &str = "data/tilesets/johto_metatiles.bin";
const BACKGROUNDS: &str = "data/tilesets/johto_native_backgrounds.json";
const ART: &str =
    include_str!("../../../planning/minnesota/revisions/fen-trail-v01/landmarks.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Landmark {
    name: String,
    block: [u16; 2],
    from: u16,
    patches: Vec<Patch>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Patch {
    at: [usize; 2],
    palette: u8,
    background: Option<u8>,
    rows: Vec<String>,
}

// The exporter stores only the first 96 glyphs of each 128-glyph VRAM page.
// Expand both pages in place by logical address, including the existing second-bank art.
fn expand_pages(compact: &[u8], atlas: &[u8], palettes: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        compact.len() == 192 * 16,
        "expected two compact Johto pages"
    );
    let mut full = vec![0; 256 * 16];
    let (low, high) = compact.split_at(96 * 16);
    full[..low.len()].copy_from_slice(low);
    full[128 * 16..224 * 16].copy_from_slice(high);
    for &glyph in atlas {
        let attr = palettes.get(usize::from(glyph)).context("Johto palette")?;
        let i = usize::from(glyph & 127);
        ensure!(
            i < 96 && (glyph < 128) == (attr & 8 == 0),
            "unexpected Johto glyph {glyph:#x}"
        );
        let old = if *attr & 8 == 0 { i } else { i + 96 };
        let new = usize::from(glyph);
        ensure!(
            compact[old * 16..old * 16 + 16] == full[new * 16..new * 16 + 16],
            "Johto glyph changed during page expansion"
        );
    }
    Ok(full)
}

fn draw(
    patch: &Patch,
    block: &mut [u8; 16],
    graphics: &mut [u8],
    palettes: &mut [u8],
    backgrounds: &mut BTreeMap<u8, u8>,
    next: &mut u8,
) -> Result<()> {
    let width = patch.rows.first().context("empty fen drawing")?.len();
    let height = patch.rows.len();
    let [x, y] = patch.at;
    ensure!(
        width > 0
            && width.is_multiple_of(8)
            && height.is_multiple_of(8)
            && x + width / 8 <= 4
            && y + height / 8 <= 4
            && patch.palette < 8
            && patch.background.is_none_or(|p| p < 8)
            && patch
                .rows
                .iter()
                .all(|r| r.len() == width && r.bytes().all(|b| matches!(b, b'.' | b'0'..=b'3'))),
        "invalid four-shade fen patch"
    );
    for gy in 0..height / 8 {
        for gx in 0..width / 8 {
            let index = (y + gy) * 4 + x + gx;
            let old = block[index];
            ensure!(old != 0x14, "fen drawing would mask an animated ripple");
            let glyph = *next;
            ensure!(glyph < 128, "fen drawing exceeds Johto's blank low page");
            let mut ink: [u8; 16] =
                graphics[usize::from(old) * 16..(usize::from(old) + 1) * 16].try_into()?;
            for row in 0..8 {
                let pixels = &patch.rows[gy * 8 + row].as_bytes()[gx * 8..gx * 8 + 8];
                for (column, &p) in pixels.iter().enumerate() {
                    if p == b'.' {
                        continue;
                    }
                    let color = p - b'0';
                    let mask = 1 << (7 - column);
                    for plane in 0..2 {
                        let at = row * 2 + plane;
                        ink[at] &= !mask;
                        if color & (1 << plane) != 0 {
                            ink[at] |= mask;
                        }
                    }
                }
            }
            let off = usize::from(glyph) * 16;
            graphics[off..off + 16].copy_from_slice(&ink);
            palettes[usize::from(glyph)] = patch.palette;
            if let Some(color) = patch.background {
                ensure!(
                    backgrounds.insert(glyph, color).is_none(),
                    "fen glyph already has a background"
                );
            }
            block[index] = glyph;
            *next += 1;
        }
    }
    Ok(())
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(MAP).context(MAP)?.clone();
    ensure!(
        room.attributes.tileset_name == "johto"
            && (room.attributes.width, room.attributes.height) == (20, 13),
        "unexpected fen terrain"
    );
    let bank = pack.data.tilesets.get_mut("johto").context("Johto")?;
    let mut atlas = pack.runtime_files.get(ATLAS).context(ATLAS)?.clone();
    let compact = pack.runtime_files.get(SHEET).context(SHEET)?;
    let mut graphics = expand_pages(compact, &atlas, &bank.palette_map)?;
    let mut backgrounds =
        serde_json::from_slice(pack.runtime_files.get(BACKGROUNDS).context(BACKGROUNDS)?)?;
    let mut next = 0x60;
    let landmarks: [Landmark; 3] = serde_json::from_str(ART)?;
    for landmark in landmarks {
        let [x, y] = landmark.block;
        let pos = usize::from(y) * usize::from(room.attributes.width) + usize::from(x);
        let slot = room.blocks.get_mut(pos).context("fen landmark location")?;
        ensure!(
            *slot == landmark.from
                && room
                    .events
                    .bg_events
                    .iter()
                    .any(|e| e.x == x * 2 && e.y == y * 2),
            "moved {} examination square",
            landmark.name
        );
        let off = usize::from(*slot) * 16;
        let mut block: [u8; 16] = atlas
            .get(off..off + 16)
            .context("fen source block")?
            .try_into()?;
        for patch in &landmark.patches {
            draw(
                patch,
                &mut block,
                &mut graphics,
                &mut bank.palette_map,
                &mut backgrounds,
                &mut next,
            )?;
        }
        let id = u8::try_from(atlas.len() / 16).context("no free Johto metatile")?;
        let permission = bank
            .collision
            .get(&format!("{:02x}", *slot))
            .context("fen landmark permissions")?
            .clone();
        ensure!(
            bank.collision
                .insert(format!("{id:02x}"), permission)
                .is_none(),
            "occupied fen metatile"
        );
        atlas.extend(block);
        *slot = u16::from(id);
    }
    pack.runtime_files.insert(
        "gfx/tilesets/johto.png".into(),
        native_tile_png::from_2bpp(&graphics)?,
    );
    pack.runtime_files.insert(SHEET.into(), graphics);
    pack.runtime_files.insert(ATLAS.into(), atlas);
    pack.runtime_files
        .insert(BACKGROUNDS.into(), serde_json::to_vec(&backgrounds)?);
    authored_town_publication::replace(pack, room)
}
