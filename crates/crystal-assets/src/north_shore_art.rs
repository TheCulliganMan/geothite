//! A bounded native waterfall addition; route geography is authored separately.

use anyhow::{Context, Result, ensure};

use crate::native_tile_png;
use crate::{CompiledGamePack, CompiledTilesetExtension};

const TILESET: &str = "mn_north_shore";
const WATERFALL_BLOCK: usize = 0x80;
const FREE_TILE: u8 = 0x20;

/// Adds the North Shore's derived Johto tileset without adding or changing maps.
///
/// Original files and audio remain unchanged. Block 0x80 uses the supplied
/// cave waterfall glyph and collision. All original Johto blocks retain their
/// addressing; the new glyph occupies a previously unreferenced tile slot.
///
/// # Errors
/// Returns an error for missing source art, an incompatible source layout,
/// an occupied destination slot, an existing derived tileset, or pack validation.
pub fn build_north_shore_art_pack(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    let files = base.runtime_files();
    let mut definition = base
        .data()
        .tilesets
        .get("johto")
        .context("Johto source tileset")?
        .clone();
    let cave = base
        .data()
        .tilesets
        .get("cave")
        .context("cave source tileset")?;
    let mut metatiles = files
        .get("data/tilesets/johto_metatiles.bin")
        .context("Johto metatiles")?
        .clone();
    let source_blocks = files
        .get("data/tilesets/cave_metatiles.bin")
        .context("cave metatiles")?;
    let source_block = source_blocks
        .get(0x2c * 16..0x2d * 16)
        .context("native cave waterfall block")?;
    let source_tile = source_block[0];
    ensure!(
        source_block.iter().all(|tile| *tile == source_tile),
        "waterfall source no longer uses one repeated glyph"
    );
    ensure!(
        metatiles.len() == WATERFALL_BLOCK * 16 && !metatiles.contains(&FREE_TILE),
        "Johto block extent or free tile slot changed"
    );
    ensure!(
        !files.contains_key("gfx/tilesets/johto.pal")
            && !files.contains_key("gfx/tilesets/cave.pal"),
        "derived art expects the shared source background palette"
    );
    let mut graphics = files
        .get("gfx/tilesets/johto.2bpp")
        .context("Johto tile graphics")?
        .clone();
    let source_graphics = files
        .get("gfx/tilesets/cave.2bpp")
        .context("cave tile graphics")?;
    let source_palette = *cave
        .palette_map
        .get(usize::from(source_tile))
        .context("waterfall glyph palette")?;
    ensure!(
        source_palette & 8 == 0,
        "waterfall source moved to another graphics bank"
    );
    let tile_bytes = source_graphics
        .get(usize::from(source_tile) * 16..(usize::from(source_tile) + 1) * 16)
        .context("waterfall glyph bytes")?;
    graphics
        .get_mut(usize::from(FREE_TILE) * 16..(usize::from(FREE_TILE) + 1) * 16)
        .context("derived glyph slot")?
        .copy_from_slice(tile_bytes);
    *definition
        .palette_map
        .get_mut(usize::from(FREE_TILE))
        .context("derived glyph palette slot")? = source_palette;
    metatiles.extend([FREE_TILE; 16]);
    definition.collision.insert(
        "80".into(),
        cave.collision
            .get("2c")
            .context("waterfall collision")?
            .clone(),
    );
    let image = native_tile_png::from_2bpp(&graphics)?;
    base.with_tileset_extension(CompiledTilesetExtension {
        manifest_id: "minnesota-north-shore-art".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some("johto".into()),
        definition,
        metatiles,
        tile_graphics_2bpp: graphics,
        tile_graphics_png: image,
    })
}

/// Appends the declared exact native cave stone-stair blocks to an installed
/// Northwoods tileset. Existing metatile addresses and referenced glyph pixels,
/// palette entries and collision quadrants remain unchanged. Only unused physical
/// glyph slots with unused logical addresses are allocated, checking both banks.
/// Source stair collisions are ordinary Floor/Wall.
/// This does not create a warp, install terrain, or write a pack.
///
/// Errors on an unexpected destination extent, occupied manifest, missing or
/// inconsistent runtime art, graphics-bank drift, insufficient unused glyphs,
/// changed source stair collisions, or invalid resulting native content.
pub(crate) fn append_native_cave_stairs(
    base: &CompiledGamePack,
    tileset: &str,
    first_block: u8,
    stair_blocks: &[u8],
    manifest: &str,
) -> Result<CompiledGamePack> {
    use std::collections::{BTreeMap, BTreeSet};

    ensure!(
        matches!(
            (tileset, first_block),
            ("mn_north_shore", 0x81) | ("mn_boundary_safari", 0x84) | ("mn_r10_rockstairs", 0x80)
        ),
        "unsupported bounded native stair destination"
    );
    ensure!(
        match tileset {
            "mn_north_shore" => stair_blocks == [0x12, 0x30] || stair_blocks == [0x12, 0x30, 0x36],
            "mn_boundary_safari" => stair_blocks == [0x12],
            "mn_r10_rockstairs" => stair_blocks == [0x12],
            _ => false,
        },
        "unsupported bounded stone stair source recipe"
    );
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|id| id == manifest),
        "native stair manifest already installed"
    );
    let definition = base
        .data
        .tilesets
        .get(tileset)
        .context("installed stair tileset")?;
    let source = base
        .data
        .tilesets
        .get("cave")
        .context("native cave stairs")?;
    let metatile_path = format!("data/tilesets/{tileset}_metatiles.bin");
    let graphics_path = format!("gfx/tilesets/{tileset}.2bpp");
    let mut metatiles = base
        .runtime_files
        .get(&metatile_path)
        .context("installed metatiles")?
        .clone();
    let mut graphics = base
        .runtime_files
        .get(&graphics_path)
        .context("installed graphics")?
        .clone();
    ensure!(
        metatiles.len() == usize::from(first_block) * 16,
        "native stair block extent drift"
    );
    ensure!(
        !base
            .runtime_files
            .contains_key(&format!("gfx/tilesets/{tileset}.pal"))
            && !base.runtime_files.contains_key("gfx/tilesets/cave.pal"),
        "native stair import requires the shared background palette"
    );
    let source_blocks = base
        .runtime_files
        .get("data/tilesets/cave_metatiles.bin")
        .context("cave metatiles")?;
    let source_graphics = base
        .runtime_files
        .get("gfx/tilesets/cave.2bpp")
        .context("cave graphics")?;
    let mut updated = definition.clone();
    let mut allocation = BTreeMap::new();
    ensure!(
        graphics.len() == 192 * 16,
        "native stair destination graphics banks changed"
    );
    let referenced: BTreeSet<_> = metatiles.iter().copied().collect();
    let physical = |glyph: u8, palette: u8| {
        if palette & 8 == 0 {
            usize::from(glyph)
        } else {
            usize::from(glyph & 0x7f) + 96
        }
    };
    let protected = referenced
        .iter()
        .map(|&glyph| {
            let palette = *definition
                .palette_map
                .get(usize::from(glyph))
                .context("installed native glyph palette address")?;
            Ok(physical(glyph, palette))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(
        protected.iter().all(|slot| *slot < 192),
        "installed native glyph addresses missing bank"
    );
    let mut free = (0usize..192)
        .filter(|slot| !protected.contains(slot))
        .filter_map(|slot| {
            let (glyph, bank) = if slot < 96 {
                (slot as u8, 0)
            } else {
                (((slot - 96) as u8) | 0x80, 8)
            };
            (!referenced.contains(&glyph) && usize::from(glyph) < definition.palette_map.len())
                .then_some((glyph, slot, bank))
        });
    for (offset, &source_block) in stair_blocks.iter().enumerate() {
        let block = usize::from(source_block);
        let expected = match source_block {
            0x12 => ["FLOOR", "FLOOR", "WALL", "FLOOR"],
            0x30 => ["FLOOR", "FLOOR", "FLOOR", "WALL"],
            0x36 => ["FLOOR", "WALL", "FLOOR", "WALL"],
            _ => anyhow::bail!("unsupported stone stair block"),
        };
        let key = format!("{block:02x}");
        let collision = source
            .collision
            .get(&key)
            .context("native stone stair collision")?;
        ensure!(
            collision.iter().map(String::as_str).eq(expected),
            "native stone stair collision drift"
        );
        let glyphs = source_blocks
            .get(block * 16..(block + 1) * 16)
            .context("native stair block pixels")?;
        for source_tile in glyphs {
            let destination = if let Some(tile) = allocation.get(source_tile) {
                *tile
            } else {
                let (tile, slot, bank) = free
                    .next()
                    .context("insufficient unused native glyph slots for stone stairs")?;
                let palette = *source
                    .palette_map
                    .get(usize::from(*source_tile))
                    .context("native stair palette")?;
                ensure!(
                    palette < 8,
                    "native stone stairs moved outside graphics bank zero"
                );
                let bytes = source_graphics
                    .get(usize::from(*source_tile) * 16..(usize::from(*source_tile) + 1) * 16)
                    .context("native stair glyph bytes")?;
                graphics[slot * 16..(slot + 1) * 16].copy_from_slice(bytes);
                updated.palette_map[usize::from(tile)] = palette | bank;
                allocation.insert(*source_tile, tile);
                tile
            };
            metatiles.push(destination);
        }
        let destination = first_block
            .checked_add(u8::try_from(offset)?)
            .context("native stair block overflow")?;
        ensure!(
            !updated
                .collision
                .contains_key(&format!("{destination:02x}")),
            "native stair collision slot already occupied"
        );
        updated
            .collision
            .insert(format!("{destination:02x}"), collision.clone());
    }
    let mut result = base.clone();
    result.runtime_files.insert(metatile_path, metatiles);
    result.runtime_files.insert(
        format!("gfx/tilesets/{tileset}.png"),
        native_tile_png::from_2bpp(&graphics)?,
    );
    result.runtime_files.insert(graphics_path, graphics);
    result.runtime_files.insert(
        format!("data/tilesets/{tileset}.json"),
        serde_json::to_vec(&updated.collision)?,
    );
    result.runtime_files.insert(
        format!("data/tilesets/{tileset}_palette_map.json"),
        serde_json::to_vec(&updated.palette_map)?,
    );
    result.data.tilesets.insert(tileset.into(), updated);
    result.report.manifests.push(manifest.into());
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

/// Certifies the installed original North Shore art against its exact native
/// Johto plus waterfall recipe before an in-place append operation.
pub(crate) fn validate_installed_north_shore_art(base: &CompiledGamePack) -> Result<()> {
    let files = &base.runtime_files;
    let mut definition = base
        .data
        .tilesets
        .get("johto")
        .context("native Johto art")?
        .clone();
    let cave = base.data.tilesets.get("cave").context("native cave art")?;
    let mut blocks = files
        .get("data/tilesets/johto_metatiles.bin")
        .context("Johto metatiles")?
        .clone();
    let mut graphics = files
        .get("gfx/tilesets/johto.2bpp")
        .context("Johto graphics")?
        .clone();
    let source_blocks = files
        .get("data/tilesets/cave_metatiles.bin")
        .context("cave metatiles")?;
    let waterfall = source_blocks
        .get(0x2c * 16..0x2d * 16)
        .context("native waterfall")?;
    ensure!(
        blocks.len() == 0x80 * 16
            && !blocks.contains(&FREE_TILE)
            && waterfall.iter().all(|glyph| *glyph == waterfall[0]),
        "native waterfall recipe drift"
    );
    let source = usize::from(waterfall[0]);
    let palette = *cave
        .palette_map
        .get(source)
        .context("waterfall source palette")?;
    ensure!(palette < 8, "waterfall graphics bank changed");
    let source_graphics = files
        .get("gfx/tilesets/cave.2bpp")
        .context("native cave glyphs")?;
    graphics
        .get_mut(usize::from(FREE_TILE) * 16..(usize::from(FREE_TILE) + 1) * 16)
        .context("waterfall destination slot")?
        .copy_from_slice(
            source_graphics
                .get(source * 16..(source + 1) * 16)
                .context("waterfall source glyph")?,
        );
    *definition
        .palette_map
        .get_mut(usize::from(FREE_TILE))
        .context("waterfall palette slot")? = palette;
    blocks.extend([FREE_TILE; 16]);
    definition.collision.insert(
        "80".into(),
        cave.collision
            .get("2c")
            .context("waterfall source collision")?
            .clone(),
    );
    ensure!(
        base.data.tilesets.get(TILESET) == Some(&definition)
            && files.get("data/tilesets/mn_north_shore_metatiles.bin") == Some(&blocks)
            && files.get("gfx/tilesets/mn_north_shore.2bpp") == Some(&graphics)
            && files.get("gfx/tilesets/mn_north_shore.png")
                == Some(&native_tile_png::from_2bpp(&graphics)?),
        "installed North Shore art differs from native Johto/waterfall recipe"
    );
    let collision: std::collections::BTreeMap<String, Vec<String>> = serde_json::from_slice(
        files
            .get("data/tilesets/mn_north_shore.json")
            .context("installed collision runtime file")?,
    )?;
    let palettes: Vec<u8> = serde_json::from_slice(
        files
            .get("data/tilesets/mn_north_shore_palette_map.json")
            .context("installed palette runtime file")?,
    )?;
    ensure!(
        collision == definition.collision && palettes == definition.palette_map,
        "installed North Shore runtime collision/palette mirror drift"
    );
    Ok(())
}
