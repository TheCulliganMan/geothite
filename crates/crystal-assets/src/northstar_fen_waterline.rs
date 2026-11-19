//! Fit the Layered Fen's low stone banks to dry ground at native tile scale.
use std::collections::BTreeMap;

use anyhow::{Context, Result, bail, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;
use crate::native_tile_png;

const ROOM: &str = "MnLayeredFen";
const ATLAS: &str = "data/tilesets/johto_metatiles.bin";
const SHEET: &str = "gfx/tilesets/johto.2bpp";
const SHORE: &str =
    include_str!("../../../planning/minnesota/revisions/fen-waterline-v01/fen-shore.txt");

// Each mark is one native 8×8 tile. The cardinal letters name the bank of
// the lake, the number-pad corners bend outwards and A–D turn back inwards.
fn glyph(mark: u8, original: u8) -> Result<u8> {
    Ok(match mark {
        b'.' => original,
        b'~' => 0x14,
        b'n' => 0x4c,
        b'e' => 0x3b,
        b's' => 0x2c,
        b'w' => 0x3d,
        b'7' => 0x5c,
        b'9' => 0x5d,
        b'1' => 0x74,
        b'3' => 0x75,
        b'A' => 0x4d,
        b'B' => 0x4b,
        b'C' => 0x76,
        b'D' => 0x77,
        b':' => 0x05, // Short grass in the space beside a tree.
        b'q' => 0x1e, // Left and right crown.
        b'p' => 0x1f,
        b'b' => 0x3e, // Left and right roots.
        b'd' => 0x3f,
        b'^' => 0xe0, // An eight-pixel pine's crown, trunk and roots.
        b'|' => 0xe1,
        b'v' => 0xe2,
        _ => bail!("unknown Fen shoreline mark {:?}", char::from(mark)),
    })
}

// The narrowest shelves hold one tile of pine behind the stones. Keep the
// native tree's source colors, paired central branches and full root row.
fn fit_narrow_pines(pack: &mut CompiledGamePack, atlas: &[u8]) -> Result<()> {
    let sheet = pack.runtime_files.get_mut(SHEET).context(SHEET)?;
    let bank = pack
        .data
        .tilesets
        .get_mut("johto")
        .context("Johto colors")?;
    ensure!(
        sheet.len() == 256 * 16 && bank.palette_map.len() == 0xe0,
        "unexpected Johto tile or palette page"
    );
    for (tier, left) in [0x1e_usize, 0x2e, 0x3e].into_iter().enumerate() {
        let slot = 0xe0 + tier;
        ensure!(
            !atlas.contains(&u8::try_from(slot)?)
                && sheet[slot * 16..(slot + 1) * 16].iter().all(|&b| b == 0),
            "narrow Fen pine overlaps existing art"
        );
        let columns = if tier == 1 {
            [2, 4, 5, 7, 8, 10, 12, 14]
        } else {
            [1, 3, 5, 7, 8, 10, 12, 14]
        };
        for line in 0..16 {
            let mut bits = 0;
            for (column, source) in columns.iter().copied().enumerate() {
                let plane = sheet[(left + source / 8) * 16 + line];
                bits |= ((plane >> (7 - source % 8)) & 1) << (7 - column);
            }
            sheet[slot * 16 + line] = bits;
        }
        bank.palette_map.push(bank.palette_map[left]);
    }
    let png = native_tile_png::from_2bpp(sheet)?;
    pack.runtime_files
        .insert("gfx/tilesets/johto.png".into(), png);
    pack.runtime_files.insert(
        "data/tilesets/johto_palette_map.json".into(),
        serde_json::to_vec(&bank.palette_map)?,
    );
    Ok(())
}

fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    let width = usize::from(room.attributes.width);
    let height = usize::from(room.attributes.height);
    let rows: Vec<_> = SHORE.lines().map(str::as_bytes).collect();
    ensure!(
        room.attributes.tileset_name == "johto"
            && (width, height) == (20, 13)
            && room.blocks.len() == width * height
            && rows.len() == height * 4
            && rows.iter().all(|row| row.len() == width * 4),
        "unexpected Fen shore dimensions or art bank"
    );
    let mut atlas = pack.runtime_files.get(ATLAS).context(ATLAS)?.clone();
    fit_narrow_pines(pack, &atlas)?;
    let bank = pack
        .data
        .tilesets
        .get_mut("johto")
        .context("Johto terrain")?;
    let mut known = BTreeMap::new();
    let (blocks, remainder) = atlas.as_chunks::<16>();
    ensure!(remainder.is_empty(), "incomplete Johto metatile");
    for (index, &tiles) in blocks.iter().enumerate() {
        let roles = bank
            .collision
            .get(&format!("{index:02x}"))
            .context("Johto collision")?;
        ensure!(
            roles.len() == 4,
            "incomplete Johto collision at {index:02x}"
        );
        known.insert((tiles, roles.clone()), u16::try_from(index)?);
    }
    for (index, placed) in room.blocks.iter_mut().enumerate() {
        let start = usize::from(*placed) * 16;
        let original: [u8; 16] = atlas
            .get(start..start + 16)
            .context("Fen block")?
            .try_into()?;
        let roles = bank
            .collision
            .get(&format!("{placed:02x}"))
            .context("Fen collision")?;
        let mut tiles = original;
        for (offset, tile) in tiles.iter_mut().enumerate() {
            let (x, y) = (offset % 4, offset / 4);
            let mark = rows[index / width * 4 + y][index % width * 4 + x];
            ensure!(
                (mark == b'~') == (roles[y / 2 * 2 + x / 2] == "WATER"),
                "Fen shoreline disagrees with water permission at block {index}, tile {offset}"
            );
            *tile = glyph(mark, *tile)?;
        }
        if tiles == original {
            continue;
        }
        let layout = (tiles, roles.clone());
        if let Some(&id) = known.get(&layout) {
            *placed = id;
        } else {
            let id = u8::try_from(atlas.len() / 16).context("Fen shore exceeded native blocks")?;
            atlas.extend_from_slice(&layout.0);
            bank.collision.insert(format!("{id:02x}"), layout.1.clone());
            known.insert(layout, u16::from(id));
            *placed = u16::from(id);
        }
    }
    pack.runtime_files.insert(ATLAS.into(), atlas);
    pack.runtime_files.insert(
        "data/tilesets/johto.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    authored_town_publication::replace(pack, room)
}

/// Aligns the Layered Fen's native rock banks with its walking and canoe squares.
///
/// # Errors
/// Returns an error if the parent is not a Cave Returns edition, the expected
/// terrain differs, or the resulting content fails runtime validation.
pub fn revise_fen_waterline(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-cave-returns-v1"),
        "Fen Waterline requires a Cave Returns edition"
    );
    apply(&mut pack)?;
    pack.report
        .manifests
        .push("northstar-fen-waterline-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
