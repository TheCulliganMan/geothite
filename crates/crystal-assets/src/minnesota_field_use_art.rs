//! Animate the original regional Whirlpool glyphs without replacing source art.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{AuthoredTileAnimation, CompiledGamePack};

pub(crate) const TILESET: &str = "mn_headwaters";
pub(crate) const CLOSED: u16 = 0x07;
pub(crate) const OPEN: u16 = 0x36;

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let path = format!("data/tilesets/{TILESET}_animations.json");
    ensure!(
        !pack.runtime_files.contains_key(&path),
        "reviewed native lake animation metadata changed"
    );
    let layout = pack
        .runtime_files
        .get(&format!("data/tilesets/{TILESET}_metatiles.bin"))
        .context("native lake metatiles")?;
    ensure!(
        layout.len() == 134 * 16,
        "reviewed native lake atlas extent"
    );
    let glyphs = [0x32_u8, 0x33, 0x42, 0x43];
    for map in pack
        .data
        .maps
        .values()
        .filter(|m| m.attributes.tileset_name == TILESET)
    {
        ensure!(
            map.id.starts_with("Mn"),
            "regional animation must not affect another region"
        );
        for &id in &map.blocks {
            let block = layout
                .get(usize::from(id) * 16..(usize::from(id) + 1) * 16)
                .context("original regional block")?;
            ensure!(
                !block.iter().any(|g| glyphs.contains(g)),
                "new Whirlpool animation would affect existing map {}",
                map.id
            );
        }
    }
    let graphics = pack
        .runtime_files
        .get(&format!("gfx/tilesets/{TILESET}.2bpp"))
        .context("native lake glyphs")?;
    let definition = &pack.data.tilesets[TILESET];
    ensure!(
        definition.collision["07"] == ["WHIRLPOOL", "BUOY", "WATER", "BUOY"]
            && definition.collision["36"] == ["WATER", "BUOY", "WATER", "BUOY"],
        "native Whirlpool must preserve its surrounding rock and water quadrants"
    );
    let replacement = pack
        .data
        .field_moves
        .whirlpool
        .replacements
        .get(TILESET)
        .and_then(|rules| rules.get(&CLOSED))
        .context("native regional Whirlpool rule")?;
    ensure!(
        replacement.replacement_block_id == OPEN && replacement.variant == "whirlpool",
        "native regional Whirlpool replacement changed"
    );
    let mut clips = BTreeMap::<u8, AuthoredTileAnimation>::new();
    for (index, glyph) in glyphs.into_iter().enumerate() {
        let frames = pack
            .runtime_files
            .get(&format!("gfx/tilesets/whirlpool/{}.2bpp", index + 1))
            .context("original native Whirlpool strip")?;
        ensure!(
            frames.len() == 64 && definition.palette_map[usize::from(glyph)] & 8 == 0,
            "native Whirlpool strip extent and bank"
        );
        ensure!(
            graphics.get(usize::from(glyph) * 16..(usize::from(glyph) + 1) * 16)
                == Some(&frames[..16]),
            "native Whirlpool first frame must match the original static glyph"
        );
        let clip = AuthoredTileAnimation {
            frame_ticks: 11,
            frames: frames
                .chunks_exact(16)
                .map(|frame| frame.try_into().expect("sixteen-byte native frame"))
                .collect(),
        };
        clip.validate()?;
        clips.insert(glyph, clip);
    }
    // Authored clips extend the production renderer's existing registrations;
    // the original water and flower animations remain owned by that renderer.
    pack.runtime_files.insert(path, serde_json::to_vec(&clips)?);
    pack.report
        .manifests
        .push("minnesota-leech-eddy-native-animation-v1".into());
    Ok(())
}
