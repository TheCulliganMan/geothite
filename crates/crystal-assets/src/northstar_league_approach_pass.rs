//! Fit the last Route 13 clearing to Ely's two-lane trail and open its Headbutt grove.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_town_publication, native_tile_png};

const ROOM: &str = "MnR13C00R02";
const BANK: &str = "mn_winter_north_shore";
const ATLAS: &str = "data/tilesets/mn_winter_north_shore_metatiles.bin";
const COLLISION: &str = "data/tilesets/mn_winter_north_shore.json";
const WIDTH: usize = 32;
const SNOW: u16 = 0x83;
const FOREST: u16 = 0x8b;

// The packed path melts first away from the west treeline. Each value is the
// depth of the last snow in one native pixel column of the 32px town entrance.
const SNOW_DEPTH: [usize; 32] = [
    23, 23, 22, 22, 21, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 8, 7, 7, 7, 7, 8, 9,
    10, 11, 12, 13, 13, 14,
];

type Inks = [[u8; 3]; 3];

fn town_snowline(pack: &mut CompiledGamePack) -> Result<()> {
    const ELY: &str = "mn_ely_snow";
    const TOWN_ATLAS: &str = "data/tilesets/mn_ely_snow_metatiles.bin";
    const TOWN_SHEET: &str = "gfx/tilesets/mn_ely_snow.2bpp";
    const TOWN_INKS: &str = "data/tilesets/mn_ely_snow_foregrounds.json";
    const SNOW_INKS: &str = "data/tilesets/mn_winter_north_shore_foregrounds.json";

    let mut town = pack.data.maps.get("MnEly").context("Ely")?.clone();
    ensure!(
        town.attributes.tileset_name == ELY
            && town.attributes.width == 20
            && town.blocks.get(12) == Some(&0x31),
        "unexpected Ely trailhead"
    );
    let mut atlas = pack
        .runtime_files
        .get(TOWN_ATLAS)
        .context(TOWN_ATLAS)?
        .clone();
    ensure!(
        atlas.len().is_multiple_of(16) && atlas.get(0x31 * 16..0x32 * 16) == Some(&[0x39; 16]),
        "original Ely gravel"
    );
    let mut gfx = pack
        .runtime_files
        .get(TOWN_SHEET)
        .context(TOWN_SHEET)?
        .clone();
    ensure!(gfx.len() == 4096, "unexpected Ely tile pages");
    let road: [u8; 16] = gfx[0x39 * 16..0x3a * 16].try_into()?;
    let snow = pack
        .runtime_files
        .get("gfx/tilesets/mn_winter_north_shore.2bpp")
        .context("Route 13 snow")?;
    let snow_tiles = pack.runtime_files[ATLAS]
        .get(0x83 * 16..0x84 * 16)
        .context("native snow patterns")?;
    let snow_inks: BTreeMap<u8, Inks> = serde_json::from_slice(&pack.runtime_files[SNOW_INKS])?;
    let mut inks: BTreeMap<u8, Inks> = serde_json::from_slice(&pack.runtime_files[TOWN_INKS])?;
    let bank = pack.data.tilesets.get_mut(ELY).context(ELY)?;
    ensure!(
        bank.palette_map.get(0x39) == Some(&0),
        "native gravel palette"
    );
    let mut recipe = [0x39; 16];
    for (index, tile) in recipe.iter_mut().enumerate() {
        let x = (index % 4) * 8;
        let y = (index / 4) * 8;
        let depths = &SNOW_DEPTH[x..x + 8];
        if depths.iter().all(|depth| *depth <= y) {
            continue;
        }
        let src = snow_tiles[index];
        let source = usize::from(src) * 16;
        let mut ink: [u8; 16] = snow
            .get(source..source + 16)
            .context("snow pixels")?
            .try_into()?;
        let mut colors = *snow_inks.get(&src).context("snow color table")?;
        if depths.iter().any(|depth| *depth < y + 8) {
            // On mixed 4-color tiles gray belongs to the gravel; the two
            // deepest snow flecks share the existing pale blue.
            colors[2] = [173; 3];
            for row in 0..8 {
                let covered = depths
                    .iter()
                    .fold(0u8, |mask, depth| (mask << 1) | u8::from(y + row < *depth));
                let blue = ink[2 * row + 1] & covered;
                let gravel = road[2 * row] & !covered;
                ink[2 * row] = (covered & !blue) | gravel;
                ink[2 * row + 1] = blue | gravel;
            }
        }
        let slot = 0x66 + u8::try_from(index)?;
        let offset = usize::from(slot) * 16;
        ensure!(
            !atlas.contains(&slot)
                && gfx[offset..offset + 16].iter().all(|&v| v == 0)
                && inks.insert(slot, colors).is_none(),
            "occupied Ely snowline slot {slot:02x}"
        );
        bank.palette_map[usize::from(slot)] = 0;
        gfx[offset..offset + 16].copy_from_slice(&ink);
        *tile = slot;
    }
    let block = u8::try_from(atlas.len() / 16)?;
    let floor = bank
        .collision
        .get("31")
        .context("Ely walking surface")?
        .clone();
    ensure!(
        floor == ["FLOOR"; 4]
            && bank
                .collision
                .insert(format!("{block:02x}"), floor)
                .is_none(),
        "Ely trailhead permission"
    );
    atlas.extend(recipe);
    pack.runtime_files.insert(TOWN_ATLAS.into(), atlas);
    pack.runtime_files.insert(
        "gfx/tilesets/mn_ely_snow.png".into(),
        native_tile_png::from_2bpp(&gfx)?,
    );
    pack.runtime_files.insert(TOWN_SHEET.into(), gfx);
    pack.runtime_files
        .insert(TOWN_INKS.into(), serde_json::to_vec(&inks)?);
    pack.runtime_files.insert(
        "data/tilesets/mn_ely_snow.json".into(),
        serde_json::to_vec(&bank.collision)?,
    );
    pack.runtime_files.insert(
        "data/tilesets/mn_ely_snow_palette_map.json".into(),
        serde_json::to_vec(&bank.palette_map)?,
    );
    town.blocks[12] = u16::from(block);
    authored_town_publication::replace(pack, town)
}

/// Reuses the two native 16px pine crowns and replaces their summer ground with local snow.
fn snow_pines(pack: &mut CompiledGamePack) -> Result<u16> {
    let atlas = pack.runtime_files.get_mut(ATLAS).context(ATLAS)?;
    ensure!(
        atlas.len().is_multiple_of(16),
        "incomplete Route 13 metatile"
    );
    let id = u8::try_from(atlas.len() / 16).context("Route 13 metatile bank full")?;
    let crowns = atlas.get(0x89 * 16..0x89 * 16 + 8).context("pine crowns")?;
    let ground = atlas
        .get(0x83 * 16 + 8..0x84 * 16)
        .context("winter floor")?;
    let mut block = [0; 16];
    block[..8].copy_from_slice(crowns);
    block[8..].copy_from_slice(ground);
    let bank = pack.data.tilesets.get_mut(BANK).context(BANK)?;
    let permissions = bank
        .collision
        .get("5d")
        .context("Route 13 Headbutt grove")?
        .clone();
    ensure!(
        permissions == ["HEADBUTT_TREE", "HEADBUTT_TREE", "FLOOR", "FLOOR"]
            && !bank.collision.contains_key(&format!("{id:02x}")),
        "unexpected pine permissions"
    );
    bank.collision.insert(format!("{id:02x}"), permissions);
    atlas.extend(block);
    pack.runtime_files
        .insert(COLLISION.into(), serde_json::to_vec(&bank.collision)?);
    Ok(u16::from(id))
}

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    ensure!(
        usize::from(room.attributes.width) == WIDTH
            && room.attributes.height == 16
            && room.attributes.tileset_name == BANK
            && room.blocks.len() == WIDTH * 16
            && room
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == "south" && c.target_map == "MnEly" && c.offset == 5),
        "unexpected Route 13 / Ely boundary"
    );
    let pines = snow_pines(pack)?;
    for (x, y, from, to) in [
        (14, 8, 0x5d, pines),
        (14, 9, FOREST, SNOW),
        // The snow strip east of Tomas was entirely enclosed.
        (21, 9, SNOW, FOREST),
        (21, 10, SNOW, FOREST),
        (21, 11, SNOW, FOREST),
        (21, 12, SNOW, FOREST),
        (20, 12, 0x0a, FOREST),
        // Taper to the two actual town-bound walking lanes at x34 and x35.
        (15, 14, SNOW, FOREST),
        (15, 15, SNOW, FOREST),
        (16, 15, SNOW, FOREST),
    ] {
        let tile = &mut room.blocks[y * WIDTH + x];
        ensure!(*tile == from, "Route 13 ({x},{y}) preimage changed");
        *tile = to;
    }
    authored_town_publication::replace(pack, room)?;
    town_snowline(pack)
}
