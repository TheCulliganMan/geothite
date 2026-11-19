//! Replace Moorhead's two historical gatehouse-roof halves with the porch-swap plant table.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::authored_route_surface::walking_map_distances;
use crate::{authored_town_publication, native_tile_png};

const BANK: &str = "johto_modern";
const LAYOUT: &str = "data/tilesets/johto_modern_metatiles.bin";
const GRAPHICS: &str = "gfx/tilesets/johto_modern.2bpp";
const GARDEN: &str =
    include_str!("../../../planning/minnesota/revisions/floodwatch-v01/porch-garden.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Garden {
    tiles: Vec<Tile>,
    blocks: [[u8; 16]; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tile {
    name: String,
    glyph: u8,
    palette: u8,
    rows: [String; 8],
}

fn pixels(tile: &Tile) -> Result<[u8; 16]> {
    let mut bytes = [0; 16];
    for (y, row) in tile.rows.iter().enumerate() {
        ensure!(
            row.len() == 8 && row.bytes().all(|v| matches!(v, b'0'..=b'3')),
            "invalid garden pixels in {}",
            tile.name
        );
        for (x, value) in row.bytes().enumerate() {
            let shade = value - b'0';
            bytes[2 * y] |= (shade & 1) << (7 - x);
            bytes[2 * y + 1] |= (shade >> 1) << (7 - x);
        }
    }
    Ok(bytes)
}

fn install_art(pack: &mut CompiledGamePack) -> Result<()> {
    let art: Garden = serde_json::from_str(GARDEN)?;
    let layout = pack
        .runtime_files
        .get(LAYOUT)
        .context("Johto modern blocks")?;
    let bank = pack
        .data
        .tilesets
        .get_mut(BANK)
        .context("Johto modern palette")?;
    ensure!(
        layout.len() == 0x80 * 16,
        "unexpected number of native Johto blocks"
    );
    let original: BTreeSet<_> = layout.iter().copied().collect();
    // Use the same four sign quadrants as the adjacent native welcome placard.
    let sign = &layout[0x45 * 16..0x46 * 16];
    ensure!(
        [0, 1, 4, 5]
            .into_iter()
            .all(|i| art.blocks[0][i] == sign[i])
            && art
                .blocks
                .iter()
                .flatten()
                .all(
                    |glyph| [0, sign[0], sign[1], sign[4], sign[5]].contains(glyph)
                        || art.tiles.iter().any(|tile| tile.glyph == *glyph)
                ),
        "porch table must use the native sign and its own garden pixels"
    );
    let graphics = pack
        .runtime_files
        .get_mut(GRAPHICS)
        .context("Johto modern tile sheet")?;
    ensure!(
        graphics.len() == 192 * 16 && bank.palette_map.len() == 224,
        "unexpected size of Johto modern tiles"
    );
    let mut occupied = BTreeSet::new();
    for &glyph in &original {
        let palette = bank
            .palette_map
            .get(usize::from(glyph))
            .context("native glyph palette")?;
        occupied.insert(if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            96 + usize::from(glyph & 0x7f)
        });
    }
    for tile in &art.tiles {
        let at = 96 + usize::from(tile.glyph & 0x7f);
        ensure!(
            (0xa0..=0xb2).contains(&tile.glyph)
                && matches!(tile.palette, 2 | 5)
                && !original.contains(&tile.glyph)
                && occupied.insert(at)
                && art.blocks.iter().flatten().any(|&g| g == tile.glyph),
            "occupied or unused Moorhead garden tile: {}",
            tile.name
        );
        graphics[at * 16..(at + 1) * 16].copy_from_slice(&pixels(tile)?);
        bank.palette_map[usize::from(tile.glyph)] = 8 | tile.palette;
    }
    let wall = bank
        .collision
        .get("08")
        .context("former roof collision")?
        .clone();
    ensure!(
        wall.iter().map(String::as_str).eq(["WALL"; 4]),
        "expected solid Moorhead table footprint"
    );
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert("gfx/tilesets/johto_modern.png".into(), png);
    let layout = pack
        .runtime_files
        .get_mut(LAYOUT)
        .context("Johto modern blocks")?;
    for (index, block) in art.blocks.into_iter().enumerate() {
        let id = format!("{:02x}", 0x80 + index);
        ensure!(
            bank.collision.insert(id, wall.clone()).is_none(),
            "occupied garden block"
        );
        layout.extend(block);
    }
    Ok(())
}

/// Installs a signed, shallow planting table at the original two solid garden squares.
///
/// Returns an error if the parent art, town, sign, or public walking approach has changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut town = pack
        .data
        .maps
        .get("MnMoorhead")
        .context("Moorhead exterior")?
        .clone();
    ensure!(
        town.attributes.tileset_name == BANK
            && town.attributes.width == 16
            && town.attributes.height == 12
            && town.blocks.len() == 192
            && town
                .events
                .bg_events
                .iter()
                .any(|e| e.script == "MN_MOORHEAD_COMMUNITY_STARTS" && e.x == 22 && e.y == 16)
            && town
                .objects
                .iter()
                .any(|o| o.script == "MN_MOORHEAD_GARDENER" && o.x == 21 && o.y == 18),
        "expected the original Moorhead porch-swap court"
    );
    let start = 8 * usize::from(town.attributes.width) + 11;
    ensure!(
        town.blocks[start..start + 2] == [0x08, 0x08],
        "Moorhead garden preimage"
    );
    install_art(pack)?;
    town.blocks[start..start + 2].copy_from_slice(&[0x80, 0x81]);
    let people = town
        .objects
        .iter()
        .map(|person| [person.x, person.y])
        .collect();
    let street = walking_map_distances(pack, &town, [31, 12], &people)?;
    ensure!(
        street.contains_key(&[21, 16]) && street.contains_key(&[20, 18]),
        "unreachable garden sign or gardener"
    );
    authored_town_publication::replace(pack, town)
}
