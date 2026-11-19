//! Shape Route 18's low public beach and offshore landing in its own Headwaters bank.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{
    AuthoredTileAnimation, CompiledGamePack, CompiledTilesetExtension, authored_town_publication,
    native_tile_png,
};

const LAKE: &str = "mn_r18_shore";
const SOURCE: &str =
    include_str!("../../../planning/minnesota/revisions/shore-watch-v01/route-terrain.json");
const FIRST_GLYPH: u8 = 0x60;
const HEADWATERS: &str = "mn_headwaters";

#[derive(Deserialize)]
struct Terrain {
    masks: Vec<WaterMask>,
    metatiles: Vec<Block>,
    rooms: BTreeMap<String, Vec<[usize; 4]>>,
}

#[derive(Deserialize)]
struct WaterMask {
    wet: [u8; 8],
    shallow: [u8; 8],
}

#[derive(Deserialize)]
struct Block {
    tiles: [u8; 16],
    collision: [String; 4],
}

impl WaterMask {
    // Mask all four frames of the actual lake ripple. In the shallows replace
    // deep blue (ink 2) with light blue (ink 1), preserving the moving crest.
    fn frame(&self, water: &[u8; 16]) -> [u8; 16] {
        let mut result = [0; 16];
        for y in 0..8 {
            let lo = water[2 * y] & self.wet[y];
            let hi = water[2 * y + 1] & self.wet[y];
            let lighten = hi & !lo & self.shallow[y];
            result[2 * y] = lo | lighten;
            result[2 * y + 1] = hi & !lighten;
        }
        result
    }
}

fn install_beach_bank(pack: &mut CompiledGamePack, data: &Terrain) -> Result<usize> {
    let mut bank = pack
        .data
        .tilesets
        .get(HEADWATERS)
        .context(HEADWATERS)?
        .clone();
    let file = |name: &str| {
        pack.runtime_files
            .get(name)
            .with_context(|| name.to_owned())
    };
    let mut atlas = file("data/tilesets/mn_headwaters_metatiles.bin")?.clone();
    let mut sheet = file("gfx/tilesets/mn_headwaters.2bpp")?.clone();
    let water = file("gfx/tilesets/water/water.2bpp")?;
    let (frames, leftover) = water.as_chunks::<16>();
    let mut clips: BTreeMap<u8, AuthoredTileAnimation> =
        serde_json::from_slice(file("data/tilesets/mn_headwaters_animations.json")?)?;
    let mut backgrounds: BTreeMap<u8, u8> =
        serde_json::from_slice(file("data/tilesets/mn_headwaters_native_backgrounds.json")?)?;
    ensure!(
        frames.len() == 4 && leftover.is_empty() && sheet.len() == 256 * 16,
        "expected the native four-frame lake and Headwaters sheet"
    );
    for (i, mask) in data.masks.iter().enumerate() {
        let slot = FIRST_GLYPH
            .checked_add(u8::try_from(i)?)
            .context("shore glyph index")?;
        let offset = usize::from(slot) * 16;
        ensure!(
            slot < 0x80
                && !atlas.contains(&slot)
                && !clips.contains_key(&slot)
                && !backgrounds.contains_key(&slot)
                && sheet[offset..offset + 16].iter().all(|b| *b == 0)
                && (0..8).all(|y| mask.shallow[y] & !mask.wet[y] == 0),
            "shore glyph {slot:#04x} overlaps other terrain or dry ground"
        );
        let clip = AuthoredTileAnimation {
            frame_ticks: 22,
            frames: frames.iter().map(|water| mask.frame(water)).collect(),
        };
        clip.validate()?;
        sheet[offset..offset + 16].copy_from_slice(&clip.frames[0]);
        *bank
            .palette_map
            .get_mut(usize::from(slot))
            .context("beach palette")? = 3;
        backgrounds.insert(slot, 0);
        clips.insert(slot, clip);
    }
    let first = atlas.len() / 16;
    ensure!(
        atlas.len().is_multiple_of(16) && bank.collision.len() == first,
        "unaligned Headwaters art and collision tables"
    );
    for block in &data.metatiles {
        let id = u8::try_from(atlas.len() / 16)?;
        ensure!(
            block
                .collision
                .iter()
                .all(|c| matches!(c.as_str(), "FLOOR" | "WATER" | "WALL")),
            "unexpected island terrain"
        );
        ensure!(
            bank.collision
                .insert(format!("{id:02x}"), block.collision.to_vec())
                .is_none(),
            "shore metatile collision already assigned"
        );
        atlas.extend(block.tiles);
    }
    let extension = CompiledTilesetExtension {
        manifest_id: "northstar-r18-sand-shore-v1".into(),
        tileset_id: LAKE.into(),
        behavior_source_tileset_id: Some(HEADWATERS.into()),
        definition: bank,
        metatiles: atlas,
        tile_graphics_png: native_tile_png::from_2bpp(&sheet)?,
        tile_graphics_2bpp: sheet,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    crate::insert_compiled_tileset_extension(pack, extension)?;
    pack.runtime_files.insert(
        format!("data/tilesets/{LAKE}_animations.json"),
        serde_json::to_vec(&clips)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{LAKE}_native_backgrounds.json"),
        serde_json::to_vec(&backgrounds)?,
    );
    Ok(first)
}

/// Install both halves of the public beach, with the islet's dry approach and water remaining connected.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let data: Terrain = serde_json::from_str(SOURCE)?;
    let first = install_beach_bank(pack, &data)?;
    for (map, cells) in data.rooms {
        let mut room = pack
            .data
            .maps
            .get(&map)
            .with_context(|| map.clone())?
            .clone();
        let width = usize::from(room.attributes.width);
        ensure!(
            room.attributes.tileset_name == HEADWATERS
                && usize::from(room.attributes.height) == 24
                && matches!(
                    (map.as_str(), width),
                    ("MnR18C00R00", 32) | ("MnR18C01R00", 8)
                ),
            "unexpected Headwaters approach at {map}"
        );
        for [x, y, old, new] in cells {
            ensure!(x < width && y < 24, "shore placement outside {map}");
            let cell = room
                .blocks
                .get_mut(y * width + x)
                .context("shore placement")?;
            ensure!(
                usize::from(*cell) == old,
                "unexpected {map} shore at {x},{y}"
            );
            let id = if new >= 256 {
                ensure!(new - 256 < data.metatiles.len(), "unknown beach block");
                first + new - 256
            } else {
                ensure!(new < first, "unknown Headwaters block");
                new
            };
            *cell = u16::try_from(id)?;
        }
        room.attributes.tileset_name = LAKE.into();
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
