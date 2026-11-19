//! Mark the four Inner Basin puzzle controls and restore Gunflint's submerged search pillar.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_publication;
use crate::minnesota_pokemon_pixel_art::bitplanes;
use crate::{CompiledGamePack, MapModule, native_tile_png};

const ART: &[u8] = include_bytes!(
    "../../../planning/minnesota/revisions/northern-trails-v01/outlet-controls.json"
);
const BASIN: &str = "MnInnerBasin";
const GUNFLINT: &str = "MnR12Cave1F";
const RECIPES: &str = "data/tilesets/cave_metatiles.bin";
const GRAPHICS: &str = "gfx/tilesets/cave.2bpp";
const WHEEL: u8 = 0x48;
const LEVER: u8 = 0x4c;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Instruments {
    handwheel: [String; 16],
    outlet_lever: [String; 16],
}

fn replace_at_read(
    map: &mut MapModule,
    at: [u16; 2],
    script: &str,
    before: u16,
    after: u16,
) -> Result<()> {
    let [x, y] = at;
    ensure!(
        map.attributes.tileset_name == "cave"
            && x / 2 < map.attributes.width
            && y / 2 < map.attributes.height
            && map
                .events
                .bg_events
                .iter()
                .any(|event| { event.script == script && event.x == x && event.y == y })
            && !map
                .script_block_changes
                .iter()
                .any(|change| { change.x / 2 == x / 2 && change.y / 2 == y / 2 }),
        "missing fixed cave interaction {script} at {at:?}"
    );
    let index = usize::from(y / 2) * usize::from(map.attributes.width) + usize::from(x / 2);
    ensure!(
        map.blocks.get(index) == Some(&before),
        "unexpected terrain behind {script}"
    );
    map.blocks[index] = after;
    Ok(())
}

fn install_instruments(pack: &mut CompiledGamePack) -> Result<[u16; 3]> {
    let art: Instruments = serde_json::from_slice(ART)?;
    let mut layout = pack
        .runtime_files
        .get(RECIPES)
        .context("native cave blocks")?
        .clone();
    let def = pack
        .data
        .tilesets
        .get_mut("cave")
        .context("native cave definition")?;
    let graphics = pack
        .runtime_files
        .get_mut(GRAPHICS)
        .context("native cave art")?;
    // Both reviewed editions use one 96-tile cave bank. Its last two rows have
    // spare physical addresses. Retain its size and the renderer’s water indices.
    ensure!(
        graphics.len() == 96 * 16
            && def.palette_map.len() >= usize::from(LEVER + 4)
            && layout.len().is_multiple_of(16)
            && layout.iter().all(|&g| usize::from(g) < 96
                && def
                    .palette_map
                    .get(usize::from(g))
                    .is_some_and(|pal| pal & 8 == 0))
            && matches!(def.collision.get("31"), Some(k) if k.len() == 4 && k.iter().all(|s| s == "WALL")),
        "unexpected cave bank or original pillar"
    );
    for (first, rows) in [(WHEEL, &art.handwheel), (LEVER, &art.outlet_lever)] {
        ensure!(
            rows.iter()
                .all(|row| row.len() == 16 && row.bytes().all(|p| (b'0'..=b'3').contains(&p))),
            "invalid 16-pixel outlet artwork"
        );
        let start = usize::from(first) * 16;
        let destination = &mut graphics[start..start + 64];
        ensure!(
            destination.iter().all(|&byte| byte == 0)
                && (first..first + 4).all(|g| !layout.contains(&g)),
            "outlet art would overwrite an occupied cave glyph"
        );
        let pixels: Vec<_> = rows
            .iter()
            .flat_map(|row| row.bytes().map(|c| c - b'0'))
            .collect();
        destination.copy_from_slice(&bitplanes(&pixels, 16, 16));
        def.palette_map[usize::from(first)..usize::from(first) + 4].fill(4);
    }
    let mut ids = [0; 3];
    for (id, (base, first_cell, first_glyph)) in
        ids.iter_mut()
            .zip([(0x1d_usize, 2, WHEEL), (0x1d, 0, WHEEL), (0x0c, 0, LEVER)])
    {
        let mut block: [u8; 16] = layout
            .get(base * 16..base * 16 + 16)
            .context("original cave metatile")?
            .try_into()?;
        for (offset, glyph) in [0, 1, 4, 5].into_iter().zip(first_glyph..first_glyph + 4) {
            block[first_cell + offset] = glyph;
        }
        *id = u16::from(u8::try_from(layout.len() / 16)?);
        let inherited = def
            .collision
            .get(&format!("{base:02x}"))
            .context("original cave collision")?
            .clone();
        ensure!(
            def.collision
                .insert(format!("{id:02x}"), inherited)
                .is_none(),
            "occupied control block"
        );
        layout.extend(block);
    }
    let png = native_tile_png::from_2bpp(graphics)?;
    pack.runtime_files
        .insert("gfx/tilesets/cave.png".into(), png);
    pack.runtime_files.insert(RECIPES.into(), layout);
    pack.runtime_files.insert(
        "data/tilesets/cave.json".into(),
        serde_json::to_vec(&def.collision)?,
    );
    pack.runtime_files.insert(
        "data/tilesets/cave_palette_map.json".into(),
        serde_json::to_vec(&def.palette_map)?,
    );
    Ok(ids)
}

/// Paints the controls at their existing events and returns the Gunflint Revive to solid rock.
/// Fails if the cave graphics, event positions, or underlying map blocks have changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut basin = pack.data.maps.get(BASIN).context("Inner Basin")?.clone();
    let mut grotto = pack
        .data
        .maps
        .get(GUNFLINT)
        .context("Gunflint grotto")?
        .clone();
    let [right_wheel, left_wheel, lever] = install_instruments(pack)?;
    for (at, script, from, to) in [
        ([9, 10], "MnBasinUpperDial", 0x1d, right_wheel),
        ([18, 14], "MnBasinCenterDial", 0x1d, left_wheel),
        ([11, 18], "MnBasinLowerDial", 0x1d, right_wheel),
        ([14, 20], "MnBasinLever", 0x0c, lever),
    ] {
        replace_at_read(&mut basin, at, script, from, to)?;
    }
    replace_at_read(&mut grotto, [28, 16], "MN_R12_GROTTO_REVIVE", 0x02, 0x31)?;
    authored_town_publication::replace(pack, basin)?;
    authored_town_publication::replace(pack, grotto)
}
