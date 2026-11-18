//! Animate Shiner Lake's original ripple drawings around its timber dock.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{
    AuthoredTileAnimation, CompiledGamePack, CompiledTilesetExtension, authored_town_publication,
    insert_compiled_tileset_extension,
};

const MAP: &str = "MnR15C00R00";
const SOURCE: &str = "kanto";
const TILESET: &str = "mn_shiner_narrows";

// The first drawing is the installed still. Dark water has three glints and
// a quiet blue frame; its shallows have their own three marks and flat color.
// Stagger their crests so the entire shoreline never flashes at once.
const RIPPLES: [(u8, [u8; 4]); 6] = [
    (0x91, [0x91, 0x93, 0x90, 0x92]),
    (0x92, [0x92, 0x91, 0x93, 0x90]),
    (0x93, [0x93, 0x90, 0x92, 0x91]),
    (0x95, [0x95, 0x94, 0x96, 0x97]),
    (0x96, [0x96, 0x97, 0x95, 0x94]),
    (0x97, [0x97, 0x95, 0x94, 0x96]),
];

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(MAP).context("Shiner Narrows")?.clone();
    ensure!(room.attributes.tileset_name == SOURCE, "Shiner art parent");
    let source = pack.data.tilesets.get(SOURCE).context("Kanto tileset")?;
    let file = |name: &str| {
        pack.runtime_files
            .get(name)
            .with_context(|| name.to_owned())
    };
    let pixels = file("gfx/tilesets/kanto.2bpp")?;
    let blocks = file("data/tilesets/kanto_metatiles.bin")?;
    let original_clips = "data/tilesets/kanto_animations.json";
    let mut clips: BTreeMap<u8, AuthoredTileAnimation> = pack
        .runtime_files
        .get(original_clips)
        .map(|data| serde_json::from_slice(data))
        .transpose()?
        .unwrap_or_default();
    for (glyph, order) in RIPPLES {
        ensure!(
            room.blocks.iter().any(|&id| {
                let i = usize::from(id) * 16;
                blocks.get(i..i + 16).is_some_and(|g| g.contains(&glyph))
            }),
            "Shiner ripple {glyph:#x} missing from the lake"
        );
        let mut frames = Vec::with_capacity(order.len());
        for source_index in order {
            let offset = usize::from(source_index) * 16;
            let frame: [u8; 16] = pixels
                .get(offset..offset + 16)
                .context("Shiner water source tile")?
                .try_into()?;
            ensure!(!frames.contains(&frame), "repeated Shiner ripple drawing");
            frames.push(frame);
        }
        let clip = AuthoredTileAnimation {
            frame_ticks: 22,
            frames,
        };
        clip.validate()?;
        ensure!(clips.insert(glyph, clip).is_none(), "occupied lake ripple");
    }
    let extension = CompiledTilesetExtension {
        manifest_id: "minnesota-shiner-dock-water-v1".into(),
        tileset_id: TILESET.into(),
        behavior_source_tileset_id: Some(SOURCE.into()),
        definition: source.clone(),
        metatiles: blocks.clone(),
        tile_graphics_2bpp: pixels.clone(),
        tile_graphics_png: file("gfx/tilesets/kanto.png")?.clone(),
    };
    // This is an exact alias of a verified native bank. Native Kanto uses
    // direct tile indices throughout its 160-tile sheet, including 0x58 and
    // the lake at 0x90. The validator for newly authored two-bank extensions
    // assigns 80 tiles to each bank and would reject that native addressing.
    insert_compiled_tileset_extension(pack, extension)?;
    // Preserve any authored clock colors of this exact parent, alongside its
    // compiled collision, palettes and Cut / Whirlpool replacement tables.
    for (prefix, suffix) in [
        ("gfx/tilesets/", ".pal"),
        ("data/tilesets/", "_foregrounds.json"),
        ("data/tilesets/", "_native_backgrounds.json"),
    ] {
        if let Some(bytes) = pack.runtime_files.get(&format!("{prefix}{SOURCE}{suffix}")) {
            pack.runtime_files
                .insert(format!("{prefix}{TILESET}{suffix}"), bytes.clone());
        }
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}_animations.json"),
        serde_json::to_vec(&clips)?,
    );
    room.attributes.tileset_name = TILESET.into();
    authored_town_publication::replace(pack, room)
}
