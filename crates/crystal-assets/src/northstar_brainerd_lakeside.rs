//! The shallow swimming beach around Brainerd's three training sandbars.
use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::permissions;
use serde::Deserialize;

use crate::northstar_gym_surface::Surface;
use crate::{
    CompiledGamePack, authored_town_publication, minnesota_pokemon_pixel_art, native_tile_png,
};

const GYM: &str = "MnBrainerdGym";
const ATLAS: &str = "mn_gym_6";
const FIRST_GLYPH: usize = 27;
const FIRST_BLOCK: usize = 16;
const LAYOUT: &str =
    include_str!("../../../planning/minnesota/gyms/brainerd-lakeside-v01/shallows.json");
const PALETTE: &[u8] =
    include_bytes!("../../../planning/minnesota/gyms/brainerd-lakeside-v01/lake.pal");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Design {
    glyphs: Vec<Glyph>,
    blocks: Vec<Block>,
    prior: Vec<u16>,
    room: Vec<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Glyph {
    palette: u8,
    rows: [String; 8],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Block {
    from: usize,
    glyphs: [u8; 16],
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let design: Design = serde_json::from_str(LAYOUT)?;
    let mut gym = pack.data.maps.get(GYM).context("Brainerd Gym")?.clone();
    ensure!(
        gym.attributes.tileset_name == ATLAS
            && gym.attributes.width == 9
            && gym.attributes.height == 7
            && gym.blocks == design.prior
            && design.room.len() == gym.blocks.len()
            && pack
                .data
                .maps
                .values()
                .filter(|room| room.attributes.tileset_name == ATLAS)
                .count()
                == 1,
        "expected Brainerd's private 9 by 7 beach"
    );
    install_art(pack, &design)?;
    let collisions = &pack.data.tilesets[ATLAS].collision;
    for (before, after) in gym.blocks.iter().zip(&design.room) {
        let original = collisions
            .get(&format!("{before:02x}"))
            .context("original beach footing")?;
        let revised = collisions
            .get(&format!("{after:02x}"))
            .context("revised beach footing")?;
        ensure!(
            original == revised,
            "Brainerd beach must retain every original walking square"
        );
    }
    gym.blocks = design.room;
    let shore = Surface::new(pack, &gym)?;
    for y in 4..=12 {
        ensure!(
            shore.permission([8, y])? == permissions::FLOOR && !shore.occupied([8, y])?,
            "the centre of the beach must remain open from the door to Sivren"
        );
    }
    authored_town_publication::replace(pack, gym)
}

fn install_art(pack: &mut CompiledGamePack, design: &Design) -> Result<()> {
    let tileset = pack
        .data
        .tilesets
        .get_mut(ATLAS)
        .context("Brainerd atlas")?;
    let mut layout = pack
        .runtime_files
        .get("data/tilesets/mn_gym_6_metatiles.bin")
        .context("original beach metatiles")?
        .clone();
    let gfx = pack
        .runtime_files
        .get_mut("gfx/tilesets/mn_gym_6.2bpp")
        .context("Brainerd two-bit tiles")?;
    let end = FIRST_GLYPH + design.glyphs.len();
    ensure!(
        layout.len() == FIRST_BLOCK * 16
            && layout.iter().all(|&tile| usize::from(tile) < FIRST_GLYPH)
            && layout
                .iter()
                .all(|&tile| tileset.palette_map[usize::from(tile)] < 8)
            && end <= gfx.len() / 16
            && end <= tileset.palette_map.len()
            && end <= 256,
        "new beach tiles must occupy the unused part of the native sheet"
    );
    for (index, glyph) in design.glyphs.iter().enumerate() {
        ensure!(
            glyph.palette < 8
                && glyph
                    .rows
                    .iter()
                    .all(|row| row.len() == 8
                        && row.bytes().all(|value| matches!(value, b'0'..=b'3'))),
            "invalid native beach tile"
        );
        let pixels: Vec<_> = glyph
            .rows
            .iter()
            .flat_map(|row| row.bytes().map(|value| value - b'0'))
            .collect();
        let tile = FIRST_GLYPH + index;
        gfx[tile * 16..(tile + 1) * 16]
            .copy_from_slice(&minnesota_pokemon_pixel_art::bitplanes(&pixels, 8, 8));
        tileset.palette_map[tile] = glyph.palette;
    }
    let png = native_tile_png::from_2bpp(gfx)?;
    ensure!(
        FIRST_BLOCK + design.blocks.len() <= 256,
        "beach metatile capacity"
    );
    for (index, block) in design.blocks.iter().enumerate() {
        ensure!(
            block.from < FIRST_BLOCK && block.glyphs.iter().all(|&g| usize::from(g) < end),
            "invalid beach metatile reference"
        );
        let footing = tileset
            .collision
            .get(&format!("{:02x}", block.from))
            .context("beach quadrant footing")?
            .clone();
        ensure!(
            tileset
                .collision
                .insert(format!("{:02x}", FIRST_BLOCK + index), footing)
                .is_none(),
            "native beach metatile is already in use"
        );
        layout.extend(block.glyphs);
    }
    let files = &mut pack.runtime_files;
    files.insert("gfx/tilesets/mn_gym_6.png".into(), png);
    ensure!(
        files
            .insert("gfx/tilesets/mn_gym_6.pal".into(), PALETTE.to_vec())
            .is_none(),
        "unexpected existing private beach palette"
    );
    files.insert("data/tilesets/mn_gym_6_metatiles.bin".into(), layout);
    files.insert(
        "data/tilesets/mn_gym_6_palette_map.json".into(),
        serde_json::to_vec(&tileset.palette_map)?,
    );
    files.insert(
        "data/tilesets/mn_gym_6.json".into(),
        serde_json::to_vec(&tileset.collision)?,
    );
    Ok(())
}
