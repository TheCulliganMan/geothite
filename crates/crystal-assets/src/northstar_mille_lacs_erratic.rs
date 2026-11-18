//! The rounded, glacially transported stone at the west end of the Mille Lacs grove.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_town_publication, northstar_trail_caches};

const ART: &[u8] =
    include_bytes!("../../../planning/minnesota/revisions/wayfinding-v01/mille-lacs-erratic.json");
const TILESET: &str = "mn_headwaters";
const BLOCK: u8 = 0x86;
const GRASS: u8 = 2;

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get("MnR18C00R00")
        .context("Mille Lacs grove")?
        .clone();
    let position = usize::from(room.attributes.width) * 10 + 1;
    ensure!(
        room.attributes.tileset_name == TILESET
            && room.blocks.get(position) == Some(&10)
            && room
                .events
                .bg_events
                .iter()
                .any(|bg| bg.x == 3 && bg.y == 20 && bg.script == "MN_R18_ERRATIC_STONE"),
        "expected the existing Hard Stone site"
    );
    northstar_trail_caches::install_landmark_art(pack, ART)?;
    let layout = &pack.runtime_files["data/tilesets/mn_headwaters_metatiles.bin"];
    let (old, face) = layout.split_at(usize::from(BLOCK) * 16);
    let atlas = pack
        .data
        .tilesets
        .get_mut(TILESET)
        .context("Mille Lacs tile bank")?;
    ensure!(
        face.len() == 16 && atlas.collision.get("86") == atlas.collision.get("0a"),
        "erratic must occupy the original four blocking squares"
    );
    // The fringe is mostly lawn with a few dark edge pixels. Giving these
    // subtiles the grass palette preserves the actual surrounding turf texture.
    for i in [1, 3, 4, 12, 15] {
        let glyph = face[i];
        ensure!(
            !old.contains(&glyph),
            "erratic edge shares a published glyph"
        );
        atlas.palette_map[usize::from(glyph)] = 8 | GRASS;
    }
    // The mixed stone/grass subtiles use grey for the rock planes. Their
    // transparent-color shade comes from the native lawn at each time of day.
    let mut backgrounds = BTreeMap::new();
    for i in [2, 5, 7, 8, 11, 13, 14] {
        let glyph = face[i];
        ensure!(
            !old.contains(&glyph),
            "erratic verge shares a published glyph"
        );
        backgrounds.insert(glyph, GRASS);
    }
    let palette_map = serde_json::to_vec(&atlas.palette_map)?;
    pack.runtime_files.insert(
        "data/tilesets/mn_headwaters_palette_map.json".into(),
        palette_map,
    );
    ensure!(
        pack.runtime_files
            .insert(
                "data/tilesets/mn_headwaters_native_backgrounds.json".into(),
                serde_json::to_vec(&backgrounds)?
            )
            .is_none(),
        "Mille Lacs background overrides already exist"
    );
    room.blocks[position] = BLOCK.into();
    authored_town_publication::replace(pack, room)
}
