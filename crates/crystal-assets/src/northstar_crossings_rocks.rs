//! Keep the outlines of the Gunflint fracture and the Mille Lacs erratic legible.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, authored_town_publication, northstar_trail_caches};

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    // Return the cliff face, snow and talus to their original local tiles. The
    // placed Rock Smash sprite is the visual cue for the opening in the ridge.
    replace_equivalent_blocks(
        pack,
        "MnR12C00R00",
        &[
            (16, 6, 81, 45),
            (17, 6, 82, 74),
            (17, 5, 83, 50),
            (18, 5, 84, 50),
            (17, 7, 85, 60),
            (24, 3, 86, 77),
        ],
    )?;

    const BANK: &str = "mn_headwaters";
    const BLOCK: usize = 0x87;
    northstar_trail_caches::install_landmark_art_at(
        pack,
        include_bytes!("../../../planning/minnesota/revisions/crossings-v01/mille-lacs-stone.json"),
        0xd3,
    )?;
    let block = &pack.runtime_files["data/tilesets/mn_headwaters_metatiles.bin"]
        [BLOCK * 16..(BLOCK + 1) * 16];
    let atlas = pack
        .data
        .tilesets
        .get_mut(BANK)
        .context("Mille Lacs atlas")?;
    // Four central tiles are grey, the rim and emerging grass use the lawn
    // palette. Each 8x8 tile selects one native four-shade palette for all pixels.
    for index in [5, 6, 9, 10] {
        let glyph = block[index];
        ensure!((0xd3..=0xdf).contains(&glyph), "expected a new stone face");
        atlas.palette_map[usize::from(glyph)] = 8;
    }
    pack.runtime_files.insert(
        "data/tilesets/mn_headwaters_palette_map.json".into(),
        serde_json::to_vec(&atlas.palette_map)?,
    );
    replace_equivalent_blocks(pack, "MnR18C00R00", &[(1, 10, 0x86, 0x87)])
}

fn replace_equivalent_blocks(
    pack: &mut CompiledGamePack,
    name: &str,
    edits: &[(usize, usize, u16, u16)],
) -> Result<()> {
    let mut map = pack.data.maps.get(name).context("northern route")?.clone();
    let collision = &pack.data.tilesets[&map.attributes.tileset_name].collision;
    for &(x, y, from, to) in edits {
        let square = map
            .blocks
            .get_mut(y * usize::from(map.attributes.width) + x)
            .context("rock art placement")?;
        ensure!(
            *square == from
                && collision
                    .get(&format!("{from:02x}"))
                    .context("old stone collision")?
                    == collision
                        .get(&format!("{to:02x}"))
                        .context("new stone collision")?,
            "rock at {name} ({x},{y}) must retain its footing"
        );
        *square = to;
    }
    authored_town_publication::replace(pack, map)
}
