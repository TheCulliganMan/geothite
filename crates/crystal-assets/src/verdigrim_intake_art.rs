//! Private native intake art; original atlas bytes and other maps stay intact.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::{AuthoredTileAnimation, CompiledGamePack, CompiledTilesetExtension};

pub(crate) const TILESET: &str = "mn_verdigrim_intake";
const GREEN: [[u8; 3]; 3] = [[148, 206, 66], [49, 132, 57], [16, 66, 41]];
const COPPER: [[u8; 3]; 3] = [[222, 156, 82], [148, 82, 49], [74, 49, 33]];
const STOPPED: [[u8; 3]; 3] = [[165, 165, 165], [99, 99, 99], [49, 49, 49]];
const CLEAN: [[u8; 3]; 3] = [[123, 206, 255], [49, 123, 206], [24, 57, 123]];

pub(crate) struct IntakeArt {
    pub green_water: u16,
    pub copper_cover: u16,
    pub stopped_control: u16,
    pub open_control: u16,
    pub broken_line: u16,
}

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<IntakeArt> {
    let rock =
        crate::northstar_rival_art::import_block(pack, "mn_arc_intake", TILESET, "cave", 0x34)?;
    let mut definition = pack.data.tilesets[TILESET].clone();
    let mut blocks = pack.runtime_files[&format!("data/tilesets/{TILESET}_metatiles.bin")].clone();
    let mut graphics = pack.runtime_files[&format!("gfx/tilesets/{TILESET}.2bpp")].clone();
    ensure!(
        graphics.len() == 192 * 16,
        "certified two-bank intake atlas required"
    );
    // The facility's checker floor used water/red inks across the whole bay.
    // Keep its native pixels and VRAM bank, but use native gray palette zero
    // on this private atlas so controls and contaminated water remain legible.
    for &glyph in &blocks[7 * 16..8 * 16] {
        definition.palette_map[usize::from(glyph)] &= 8;
    }
    let mut colors = BTreeMap::new();
    let copper_cover = duplicate_block(
        &mut blocks,
        &mut graphics,
        &mut definition,
        &mut colors,
        rock,
        COPPER,
    )?;
    let green_water = duplicate_block(
        &mut blocks,
        &mut graphics,
        &mut definition,
        &mut colors,
        0x40,
        GREEN,
    )?;
    let stopped_control = duplicate_block(
        &mut blocks,
        &mut graphics,
        &mut definition,
        &mut colors,
        0x16,
        STOPPED,
    )?;
    let open_control = duplicate_block(
        &mut blocks,
        &mut graphics,
        &mut definition,
        &mut colors,
        0x16,
        CLEAN,
    )?;
    let broken_line = u16::try_from(blocks.len() / 16)?;
    let mut rubble =
        blocks[usize::from(copper_cover) * 16..(usize::from(copper_cover) + 1) * 16].to_vec();
    // A one-tile pipe platform touches the outflow. Its other three quadrants
    // remain solid, so Surf cannot bypass the still-sealed creature chamber.
    for index in [10, 11, 14, 15] {
        rubble[index] = blocks[7 * 16 + index];
    }
    blocks.extend(rubble);
    let mut platform = definition.collision["02"].clone();
    ensure!(
        platform.iter().all(|kind| kind == "WALL"),
        "solid native pipe boundary required"
    );
    platform[3] = "FLOOR".into();
    definition
        .collision
        .insert(format!("{broken_line:02x}"), platform);
    let clean = blocks[0x40 * 16];
    let green = blocks[usize::from(green_water) * 16];
    ensure!(
        blocks[0x40 * 16..0x41 * 16].iter().all(|v| *v == clean),
        "one native water glyph required"
    );
    ensure!(
        blocks[usize::from(green_water) * 16..(usize::from(green_water) + 1) * 16]
            .iter()
            .all(|v| *v == green),
        "one recolored water glyph required"
    );
    let water = pack
        .runtime_files
        .get("gfx/tilesets/water/water.2bpp")
        .context("supplied native four-frame water")?;
    ensure!(water.len() == 64, "four native water frames required");
    let (frames, _) = water.as_chunks::<16>();
    let clip = AuthoredTileAnimation {
        frame_ticks: 22,
        frames: frames.to_vec(),
    };
    clip.validate()?;
    let animations = BTreeMap::from([(clean, clip.clone()), (green, clip)]);
    let extension = CompiledTilesetExtension {
        manifest_id: "verdigrim-intake-native-art-v1".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some("mn_arc_intake".into()),
        definition,
        metatiles: blocks,
        tile_graphics_png: crate::native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::validate_compiled_tileset_extension(&extension)?;
    // The import above created this private atlas. Replace only that new atlas
    // with its validated final append; all pre-existing runtime files remain.
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_metatiles.bin"),
        extension.metatiles,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&extension.definition.collision)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_palette_map.json"),
        serde_json::to_vec(&extension.definition.palette_map)?,
    );
    pack.runtime_files.insert(
        format!("gfx/tilesets/{TILESET}.2bpp"),
        extension.tile_graphics_2bpp,
    );
    pack.runtime_files.insert(
        format!("gfx/tilesets/{TILESET}.png"),
        extension.tile_graphics_png,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_foregrounds.json"),
        serde_json::to_vec(&colors)?,
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_animations.json"),
        serde_json::to_vec(&animations)?,
    );
    pack.data
        .tilesets
        .insert(TILESET.into(), extension.definition);
    Ok(IntakeArt {
        green_water,
        copper_cover,
        stopped_control,
        open_control,
        broken_line,
    })
}

fn duplicate_block(
    blocks: &mut Vec<u8>,
    graphics: &mut [u8],
    definition: &mut crate::TilesetDefinition,
    colors: &mut BTreeMap<u8, [[u8; 3]; 3]>,
    block: u16,
    ink: [[u8; 3]; 3],
) -> Result<u16> {
    let bytes = blocks
        .get(usize::from(block) * 16..(usize::from(block) + 1) * 16)
        .context("source metatile")?
        .to_vec();
    let mut used: BTreeSet<_> = blocks.iter().copied().collect();
    let mut copies = BTreeMap::new();
    let mut output = Vec::new();
    for original in bytes {
        let copied = if let Some(&copied) = copies.get(&original) {
            copied
        } else {
            let next = (0..96_u8)
                .chain(128..224_u8)
                .find(|g| !used.contains(g))
                .context("private native glyph capacity")?;
            let attribute = definition.palette_map[usize::from(original)];
            let slot = if attribute & 8 == 0 {
                usize::from(original)
            } else {
                usize::from(original & 127) + 96
            };
            let target = if next < 96 {
                usize::from(next)
            } else {
                usize::from(next - 128) + 96
            };
            let source = graphics
                .get(slot * 16..(slot + 1) * 16)
                .context("native source glyph")?
                .to_vec();
            graphics[target * 16..(target + 1) * 16].copy_from_slice(&source);
            definition.palette_map[usize::from(next)] =
                (attribute & 7) | if next >= 128 { 8 } else { 0 };
            colors.insert(next, ink);
            used.insert(next);
            copies.insert(original, next);
            next
        };
        output.push(copied);
    }
    let id = u16::try_from(blocks.len() / 16)?;
    definition.collision.insert(
        format!("{id:02x}"),
        definition
            .collision
            .get(&format!("{block:02x}"))
            .context("source collision")?
            .clone(),
    );
    blocks.extend(output);
    Ok(id)
}
