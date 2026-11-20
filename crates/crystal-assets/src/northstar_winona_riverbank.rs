//! Fit the native pebble bank beside Winona's Itemfinder lesson to its dry land.
use std::collections::BTreeMap;

use anyhow::{Context, Result, bail, ensure};

use crate::{CompiledGamePack, authored_town_publication, native_tile_png};

const BANK: &str = "johto_glacier_rail";
const ATLAS: &str = "data/tilesets/johto_glacier_rail_metatiles.bin";
const SHEET: &str = "gfx/tilesets/johto_glacier_rail.2bpp";
const SHORE: &str =
    include_str!("../../../planning/minnesota/revisions/finder-lesson-v01/winona-riverbank.txt");

fn rotate(source: &[u8], clockwise: bool) -> [u8; 16] {
    let mut result = [0; 16];
    for y in 0..8 {
        for x in 0..8 {
            let (sx, sy) = if clockwise { (y, 7 - x) } else { (7 - y, x) };
            for plane in 0..2 {
                let bit = (source[2 * sy + plane] >> (7 - sx)) & 1;
                result[2 * y + plane] |= bit << (7 - x);
            }
        }
    }
    result
}

fn install_glyphs(pack: &mut CompiledGamePack, atlas: &[u8]) -> Result<()> {
    let johto = pack
        .runtime_files
        .get("gfx/tilesets/johto.2bpp")
        .context("Johto art")?;
    let native_pines = johto
        .get(0xe0 * 16..0xe3 * 16)
        .context("Fen's narrow fir")?
        .to_vec();
    let palette = &mut pack.data.tilesets.get_mut(BANK).context(BANK)?.palette_map;
    let sheet = pack.runtime_files.get_mut(SHEET).context(SHEET)?;
    ensure!(
        sheet.len() == 192 * 16 && palette.len() == 224,
        "Winona rail graphics banks changed"
    );
    // On this 192-glyph sheet, bank 1 begins at physical tile 96.
    // The next five logical addresses follow the four existing rail glyphs.
    for old in atlas.iter().copied() {
        let i = usize::from(old);
        let second = (i & 0x7f) + 96;
        let physical = if palette.get(i).is_some_and(|p| p & 8 != 0) && second < 192 {
            second
        } else if i >= 192 {
            i - 128
        } else {
            i
        };
        ensure!(
            !(0xa4..=0xa8).contains(&physical) && !(0xc4..=0xc8).contains(&old),
            "new Winona glyph overlaps existing terrain"
        );
    }
    ensure!(
        sheet[0xa4 * 16..0xa9 * 16].iter().all(|&b| b == 0),
        "Winona glyph slots already occupied"
    );
    let stones = rotate(&sheet[0x4c * 16..0x4d * 16], false);
    let gravel = rotate(&sheet[0x2c * 16..0x2d * 16], true);
    sheet[0xa4 * 16..0xa7 * 16].copy_from_slice(&native_pines);
    sheet[0xa7 * 16..0xa8 * 16].copy_from_slice(&stones);
    sheet[0xa8 * 16..0xa9 * 16].copy_from_slice(&gravel);
    palette[0xc4..=0xc8].copy_from_slice(&[10, 10, 10, 13, 13]);
    let png = native_tile_png::from_2bpp(sheet)?;
    pack.runtime_files
        .insert("gfx/tilesets/johto_glacier_rail.png".into(), png);
    pack.runtime_files.insert(
        "data/tilesets/johto_glacier_rail_palette_map.json".into(),
        serde_json::to_vec(palette)?,
    );
    Ok(())
}

fn glyph(mark: u8, original: u8) -> Result<u8> {
    Ok(match mark {
        b'.' => original,
        b'~' => 0x14, // Ordinary, animated Johto water.
        b'a' => 0xc4,
        b't' => 0xc5,
        b'b' => 0xc6, // Complete narrow fir.
        b'r' => 0xc7,
        b'v' => 0xc8, // Higher woodland edge and low gravel put-in.
        b's' => 0x2c,
        b'7' => 0x5c,
        b'9' => 0x5d,
        b'A' => 0x4b,
        b'p' => 0x1e,
        b'q' => 0x1f, // Crown of the retained full-width pine.
        _ => bail!("unknown Winona riverbank glyph {mark}"),
    })
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut map = pack.data.maps.get("MnWinona").context("Winona")?.clone();
    let width = usize::from(map.attributes.width);
    ensure!(
        (
            width,
            map.attributes.height,
            map.attributes.tileset_name.as_str()
        ) == (16, 14, BANK)
            && [
                map.blocks[3 * width + 12],
                map.blocks[4 * width + 12],
                map.blocks[4 * width + 13],
                map.blocks[6 * width + 13],
                map.blocks[6 * width + 14]
            ] == [5, 2, 0x58, 5, 0x58],
        "unexpected Winona landing"
    );
    let rows: Vec<_> = SHORE.lines().map(str::as_bytes).collect();
    ensure!(
        rows.len() == 28 && rows.iter().all(|r| r.len() == 7),
        "Winona bank must be 7 by 28 native tiles"
    );
    let mut atlas = pack.runtime_files.get(ATLAS).context(ATLAS)?.clone();
    ensure!(
        atlas.len() == 130 * 16,
        "unexpected native Winona metatile count"
    );
    install_glyphs(pack, &atlas)?;
    let bank = pack.data.tilesets.get_mut(BANK).context(BANK)?;
    let mut known = BTreeMap::new();
    for (i, tiles) in atlas.as_chunks::<16>().0.iter().enumerate() {
        let roles = bank
            .collision
            .get(&format!("{i:02x}"))
            .context("Winona collision")?;
        known.insert((*tiles, roles.clone()), u16::try_from(i)?);
    }
    // Each legend mark occupies a single 8-pixel field tile. The plan begins
    // at (50,8) inside the fir stand; the walking/launch contact is (25,8).
    for y in 2..9 {
        for x in 12..15 {
            let cell = &mut map.blocks[y * width + x];
            let begin = usize::from(*cell) * 16;
            let old: [u8; 16] = atlas[begin..begin + 16].try_into()?;
            let roles = bank
                .collision
                .get(&format!("{cell:02x}"))
                .context("old shore collision")?
                .clone();
            let mut tiles = old;
            for (i, tile) in tiles.iter_mut().enumerate() {
                let native_x = x * 4 + i % 4;
                if !(50..57).contains(&native_x) {
                    continue;
                }
                let mark = rows[y * 4 + i / 4 - 8][native_x - 50];
                if mark == b'.' {
                    continue;
                }
                ensure!(
                    (mark == b'~') == (roles[(i / 8) * 2 + (i % 4) / 2] == "WATER"),
                    "Winona stones and water disagree with footing"
                );
                *tile = glyph(mark, *tile)?;
            }
            if tiles == old {
                continue;
            }
            let key = (tiles, roles);
            *cell = if let Some(&shared) = known.get(&key) {
                shared
            } else {
                let next =
                    u8::try_from(atlas.len() / 16).context("Winona exceeds native metatiles")?;
                atlas.extend_from_slice(&key.0);
                bank.collision.insert(format!("{next:02x}"), key.1.clone());
                known.insert(key, u16::from(next));
                u16::from(next)
            };
        }
    }
    pack.runtime_files.insert(ATLAS.into(), atlas);
    pack.runtime_files.insert(
        "data/tilesets/johto_glacier_rail.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    authored_town_publication::replace(pack, map)
}
