//! Original, palette-limited lodge art assembled into private native tile banks.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::{CompiledGamePack, CompiledTilesetExtension, native_tile_png};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Atlas {
    tileset: String,
    source: String,
    retained: Vec<u8>,
    stamps: Vec<Stamp>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Stamp {
    name: String,
    rows: Vec<String>,
    palettes: [String; 4],
    collision: [String; 4],
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let atlases: Vec<Atlas> = serde_json::from_slice(include_bytes!(
        "../../../planning/minnesota/landmarks/mothlight-lodge/art.json"
    ))?;
    for atlas in atlases {
        install_atlas(pack, atlas)?;
    }
    Ok(())
}

fn install_atlas(pack: &mut CompiledGamePack, atlas: Atlas) -> Result<()> {
    let files = &pack.runtime_files;
    let old = files
        .get(&format!("data/tilesets/{}_metatiles.bin", atlas.source))
        .context("lodge source metatiles")?;
    let gfx = files
        .get(&format!("gfx/tilesets/{}.2bpp", atlas.source))
        .context("lodge source glyphs")?;
    let mut definition = pack
        .data
        .tilesets
        .get(&atlas.source)
        .context("lodge source collision")?
        .clone();
    let old_palette = definition.palette_map.clone();
    let mut glyphs = BTreeMap::<(Vec<u8>, u8), u8>::new();
    let mut graphics = Vec::new();
    let mut palettes = vec![0; 256];
    let mut intern = |bytes: Vec<u8>, palette: u8| -> Result<u8> {
        let key = (bytes, palette);
        if let Some(&id) = glyphs.get(&key) {
            return Ok(id);
        }
        let id = u8::try_from(glyphs.len()).context("lodge exceeds 256 native glyphs")?;
        graphics.extend(&key.0);
        palettes[usize::from(id)] = palette | if id >= 128 { 8 } else { 0 };
        glyphs.insert(key, id);
        Ok(id)
    };
    // Unused blocks are solid shadow, including indoor camera borders.
    let blank = intern(vec![255; 16], 0)?;
    let mut metatiles = vec![blank; old.len()];
    for block in atlas.retained {
        for i in 0..16 {
            let id = usize::from(old[usize::from(block) * 16 + i]);
            let p = old_palette[id];
            let slot = if p & 8 != 0 {
                (id & 127) + gfx.len() / 32
            } else {
                id
            };
            metatiles[usize::from(block) * 16 + i] =
                intern(gfx[slot * 16..slot * 16 + 16].to_vec(), p & 7)?;
        }
    }
    for stamp in atlas.stamps {
        ensure!(
            stamp.rows.len() == 32
                && stamp.rows.iter().all(|r| r.len() == 32)
                && stamp.palettes.iter().all(|r| r.len() == 4),
            "invalid stamp {}",
            stamp.name
        );
        let block = metatiles.len() / 16;
        ensure!(block < 256, "lodge exceeds native metatiles");
        for cell in 0..16 {
            let mut bytes = vec![0_u8; 16];
            for y in 0..8 {
                for x in 0..8 {
                    let shade = stamp.rows[cell / 4 * 8 + y].as_bytes()[cell % 4 * 8 + x];
                    ensure!((b'0'..=b'3').contains(&shade), "invalid lodge pigment");
                    bytes[2 * y] |= ((shade - b'0') & 1) << (7 - x);
                    bytes[2 * y + 1] |= ((shade - b'0') >> 1) << (7 - x);
                }
            }
            let pigment = stamp.palettes[cell / 4].as_bytes()[cell % 4];
            ensure!((b'0'..=b'7').contains(&pigment), "invalid lodge palette");
            let p = pigment - b'0';
            metatiles.push(intern(bytes, p)?);
        }
        definition
            .collision
            .insert(format!("{block:02x}"), stamp.collision.to_vec());
    }
    graphics.resize(4096, 0);
    definition.palette_map = palettes;
    let extension = CompiledTilesetExtension {
        manifest_id: format!("mothlight-{}-v2", atlas.tileset),
        tileset_id: atlas.tileset.clone(),
        behavior_source_tileset_id: None,
        definition,
        metatiles,
        tile_graphics_png: native_tile_png::from_2bpp(&graphics)?,
        tile_graphics_2bpp: graphics,
    };
    crate::insert_compiled_tileset_extension(pack, extension)?;
    if atlas.tileset != "mn_mothlight_exterior" {
        pack.runtime_files.insert(
            format!("gfx/tilesets/{}.pal", atlas.tileset),
            include_bytes!("../../../planning/minnesota/landmarks/mothlight-lodge/hall.pal")
                .to_vec(),
        );
    }
    Ok(())
}
