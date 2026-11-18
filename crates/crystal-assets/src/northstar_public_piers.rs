//! Shared native timber for the public piers and the open fishing dock in Brainerd.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::native_tile_png;
use crate::{CompiledGamePack, MapModule, authored_town_publication};

#[path = "northstar_city_piers.rs"]
mod northstar_city_piers;

const SOURCE: &str = "kanto";
const BRAINERD_BANK: &str = "mn_outdoor_johto_stairs";
const BANKS: [&str; 8] = [
    BRAINERD_BANK,
    "mn_outdoor_headwaters_stairs",
    "mn_headwaters_thaw",
    "mn_prairie_headwaters",
    "mn_headwaters",
    "mn_winter_tamarack",
    "mn_winter_bemidji",
    "mn_detroit_lakes_glacier_rail",
];
const TIMBER: [u8; 10] = [0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x6e, 0x6f];
const WATER: u16 = 0x35;
const PIER: &str =
    include_str!("../../../planning/minnesota/revisions/outdoor-contours-v01/brainerd-pier.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pier {
    metatiles: Vec<Deck>,
    placements: Vec<Placement>,
    open_water: Vec<OldBlock>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Deck {
    name: String,
    tiles: [u8; 16],
    collision: [String; 4],
}

#[derive(Deserialize)]
struct OldBlock {
    at: [u16; 2],
    from: u16,
}

#[derive(Deserialize)]
struct Placement {
    #[serde(flatten)]
    old: OldBlock,
    deck: String,
}

struct TimberGlyph {
    id: u8,
    palette: u8,
    pixels: [u8; 16],
}

fn address(glyph: u8, palette: u8, tiles: usize) -> usize {
    if palette & 8 == 0 {
        usize::from(glyph)
    } else {
        tiles / 2 + usize::from(glyph & 0x7f)
    }
}

fn imported_id(bank: &str, source: u8) -> Option<u8> {
    let (planks, ends) = match bank {
        BRAINERD_BANK | "mn_outdoor_headwaters_stairs" => (0xc0, 0xce),
        "mn_winter_bemidji" => (0xb0, 0xb8),
        "mn_detroit_lakes_glacier_rail" => (0xd0, 0xd8),
        "mn_headwaters_thaw" => (0xd0, 0xde),
        "mn_prairie_headwaters" => (0xd3, 0xdb),
        "mn_winter_tamarack" => (0xd8, 0xf0),
        // The shared 192-tile Headwaters sheet has only four spare addresses.
        "mn_headwaters" => {
            return match source {
                0x60 => Some(0x9d),
                0x61 => Some(0x9e),
                0x64 => Some(0x9f),
                0x6e => Some(0xc3),
                _ => None,
            };
        }
        _ => return None,
    };
    match source {
        0x60..=0x67 => Some(planks + (source - 0x60)),
        0x6e..=0x6f => Some(ends + (source - 0x6e)),
        _ => None,
    }
}

pub(super) fn is_timber_glyph(bank: &str, glyph: u8) -> bool {
    TIMBER
        .iter()
        .any(|&source| imported_id(bank, source) == Some(glyph))
}

/// Import existing Kanto dock pixels into unused addresses in each outdoor bank.
pub(super) fn install_timber(pack: &mut CompiledGamePack) -> Result<()> {
    let gfx = pack
        .runtime_files
        .get("gfx/tilesets/kanto.2bpp")
        .context("Kanto source tiles")?;
    let colors = &pack
        .data
        .tilesets
        .get(SOURCE)
        .context("Kanto source palette")?
        .palette_map;
    let mut imports = Vec::with_capacity(TIMBER.len());
    for id in TIMBER {
        let palette = *colors
            .get(usize::from(id))
            .context("Kanto timber palette")?;
        let start = 16 * address(id, palette, gfx.len() / 16);
        let pixels = gfx
            .get(start..start + 16)
            .context("Kanto timber pixels")?
            .try_into()?;
        imports.push(TimberGlyph {
            id,
            palette,
            pixels,
        });
    }

    for bank in BANKS {
        let art = format!("gfx/tilesets/{bank}.2bpp");
        let layout = pack
            .runtime_files
            .get(&format!("data/tilesets/{bank}_metatiles.bin"))
            .context("dock metatile inventory")?;
        let def = pack
            .data
            .tilesets
            .get_mut(bank)
            .context("target dock tileset")?;
        let tile_count = pack
            .runtime_files
            .get(&art)
            .context("target dock graphics")?
            .len()
            / 16;
        let mut used = BTreeSet::new();
        for &glyph in layout {
            let palette = *def
                .palette_map
                .get(usize::from(glyph))
                .context("target glyph palette")?;
            used.insert(address(glyph, palette, tile_count));
        }
        let local: Vec<_> = imports
            .iter()
            .filter_map(|tile| imported_id(bank, tile.id).map(|id| (tile, id)))
            .collect();
        for &(tile, id) in &local {
            let palette = 8 | (tile.palette & 7);
            let physical = address(id, palette, tile_count);
            ensure!(
                physical < tile_count && !layout.contains(&id) && used.insert(physical),
                "timber would overwrite active {bank} glyph {id:02x}"
            );
        }
        let gfx = pack
            .runtime_files
            .get_mut(&art)
            .context("target dock graphics")?;
        for (tile, id) in local {
            let palette = 8 | (tile.palette & 7);
            let start = 16 * address(id, palette, tile_count);
            gfx[start..start + 16].copy_from_slice(&tile.pixels);
            *def.palette_map
                .get_mut(usize::from(id))
                .context("allocated timber palette")? = palette;
        }
        let png = native_tile_png::from_2bpp(gfx)?;
        pack.runtime_files
            .insert(format!("gfx/tilesets/{bank}.png"), png);
        pack.runtime_files.insert(
            format!("data/tilesets/{bank}_palette_map.json"),
            serde_json::to_vec(&def.palette_map)?,
        );
    }
    Ok(())
}

fn place(map: &mut MapModule, old: OldBlock, to: u16) -> Result<()> {
    let [x, y] = old.at;
    ensure!(
        x < map.attributes.width && y < map.attributes.height,
        "pier outside town"
    );
    let index = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
    let current = map
        .blocks
        .get_mut(index)
        .context("missing city pier terrain")?;
    ensure!(*current == old.from, "{} shore changed at {x},{y}", map.id);
    *current = to;
    Ok(())
}

fn install_deck(
    pack: &mut CompiledGamePack,
    map: &mut MapModule,
    expected_bank: &str,
    decks: Vec<Deck>,
    placements: Vec<Placement>,
) -> Result<()> {
    let bank = &map.attributes.tileset_name;
    ensure!(bank == expected_bank, "city pier tileset changed");
    let layout = pack
        .runtime_files
        .get_mut(&format!("data/tilesets/{bank}_metatiles.bin"))
        .context("city pier metatiles")?;
    let def = pack
        .data
        .tilesets
        .get_mut(bank)
        .context("city pier collision")?;
    let mut deck_ids = BTreeMap::new();
    for deck in decks {
        ensure!(
            layout.len().is_multiple_of(16),
            "incomplete city pier metatile"
        );
        let id = u8::try_from(layout.len() / 16).context("full city pier metatile bank")?;
        ensure!(
            deck.tiles.iter().all(|&g| g == 0x14
                || (g == 0x3d && bank == "mn_winter_bemidji")
                || is_timber_glyph(bank, g)),
            "city pier deck requires native timber, ripples or the snowy shoreline"
        );
        ensure!(
            def.collision
                .insert(format!("{id:02x}"), Vec::from(deck.collision))
                .is_none()
                && deck_ids.insert(deck.name, u16::from(id)).is_none(),
            "duplicate city pier deck"
        );
        layout.extend(deck.tiles);
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{bank}.json"),
        serde_json::to_vec(&def.collision)?,
    );
    for cell in placements {
        place(
            map,
            cell.old,
            *deck_ids.get(&cell.deck).context("unknown city pier deck")?,
        )?;
    }
    Ok(())
}

/// Fit the existing public pier with a continuous deck and fishable water on three sides.
pub(super) fn install_brainerd(pack: &mut CompiledGamePack) -> Result<()> {
    let design: Pier = serde_json::from_str(PIER)?;
    let mut map = pack
        .data
        .maps
        .get("MnBrainerd")
        .context("Brainerd town")?
        .clone();
    install_deck(
        pack,
        &mut map,
        BRAINERD_BANK,
        design.metatiles,
        design.placements,
    )?;
    for water in design.open_water {
        place(&mut map, water, WATER)?;
    }

    for y in 13..=19 {
        for (x, surface) in [(46, "WATER"), (47, "FLOOR"), (48, "FLOOR"), (49, "WATER")] {
            ensure!(
                kind(pack, &map, [x, y])? == surface,
                "Brainerd open dock edge {x},{y}"
            );
        }
    }
    for x in [47, 48] {
        ensure!(
            kind(pack, &map, [x, 12])? == "WATER" && kind(pack, &map, [x, 20])? == "FLOOR",
            "Brainerd pier must connect to the street and the lake"
        );
    }
    ensure!(
        map.objects
            .iter()
            .any(|o| o.script == "MN_BRAINERD_FISHER" && [o.x, o.y] == [48, 16]),
        "Brainerd fisher must stand on the dock"
    );
    authored_town_publication::replace(pack, map)?;
    northstar_city_piers::install(pack)
}
