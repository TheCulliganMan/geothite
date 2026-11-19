//! Join Rochester's court and Mankato's quarry to the stairways seen from below.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule, authored_town_publication};

type Block = ([u8; 16], [&'static str; 4]);

const FLOOR: &str = "FLOOR";
const WALL: &str = "WALL";
const HOP: &str = "HOP_DOWN";

// Court tile, steel side post, checkerboard landing, and a two-tile-wide stair.
#[rustfmt::skip]
const COURT_CORNERS: [Block; 2] = [
    ([
        0x35, 0x36, 0x2c, 0x2c,
        0x37, 0x38, 0x2c, 0x2c,
        0x01, 0x02, 0x3a, 0x3b,
        0x02, 0x01, 0x3a, 0x3b,
    ], [WALL, FLOOR, FLOOR, FLOOR]),
    ([
        0x2c, 0x2c, 0x35, 0x36,
        0x2c, 0x2c, 0x37, 0x38,
        0x3a, 0x3b, 0x01, 0x02,
        0x3a, 0x3b, 0x02, 0x01,
    ], [FLOOR, WALL, FLOOR, FLOOR]),
];

// Existing quarry stone closes the outside flank of the upper west staircase.
// Both high drops carry the upper sandstone all the way to the existing lip.
#[rustfmt::skip]
const QUARRY_EDGES: [Block; 2] = [
    ([
        0x07, 0x08, 0x24, 0x24,
        0x09, 0x0a, 0x24, 0x24,
        0x07, 0x08, 0x00, 0x01,
        0x09, 0x0a, 0x02, 0x03,
    ], [WALL, FLOOR, WALL, WALL]),
    ([
        0x24, 0x24, 0x24, 0x24,
        0x24, 0x24, 0x24, 0x24,
        0x24, 0x24, 0x24, 0x24,
        0x1a, 0x1a, 0x1a, 0x1a,
    ], [HOP, HOP, WALL, WALL]),
];

fn extend(pack: &mut CompiledGamePack, name: &str, start: u8, blocks: &[Block]) -> Result<()> {
    let bank = pack.data.tilesets.get_mut(name).context("gym tileset")?;
    let path = format!("data/tilesets/{name}_metatiles.bin");
    let atlas = pack
        .runtime_files
        .get_mut(&path)
        .context("gym terrain artwork")?;
    ensure!(
        atlas.len() == usize::from(start) * 16 && bank.collision.len() == usize::from(start),
        "unexpected {name} metatile count"
    );
    for (offset, (glyphs, collision)) in blocks.iter().enumerate() {
        ensure!(
            glyphs
                .iter()
                .all(|glyph| atlas[..usize::from(start) * 16].contains(glyph)),
            "missing {name} source glyph"
        );
        let index = usize::from(start) + offset;
        let old = bank.collision.insert(
            format!("{index:02x}"),
            collision.iter().map(|&role| role.to_owned()).collect(),
        );
        ensure!(old.is_none(), "occupied {name} metatile {index:02x}");
        atlas.extend_from_slice(glyphs);
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{name}.json"),
        serde_json::to_vec(&bank.collision)?,
    );
    Ok(())
}

fn place(room: &mut MapModule, changes: &[(usize, usize, u16, u16)]) -> Result<()> {
    let w = usize::from(room.attributes.width);
    for &(x, y, old, new) in changes {
        let block = room.blocks.get_mut(y * w + x).context("gym layout")?;
        ensure!(
            x < w && *block == old,
            "unexpected {} block at {x},{y}",
            room.id
        );
        *block = new;
    }
    Ok(())
}

/// Connects the existing gym stairways to their respective upper courts.
/// Returns an error for a changed parent layout, tileset or east-gate callback.
pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut court = pack
        .data
        .maps
        .get("MnRochesterGym")
        .context("Rochester gym")?
        .clone();
    ensure!(
        court.attributes.tileset_name == "mn_gym_1",
        "unexpected court tileset"
    );
    extend(pack, "mn_gym_1", 0x19, &COURT_CORNERS)?;
    place(
        &mut court,
        &[
            (2, 2, 0x17, 0x16),
            (7, 2, 0x17, 0x18),
            (2, 3, 0x07, 0x19),
            (7, 3, 0x01, 0x1a),
            (2, 4, 0x06, 0x07),
        ],
    )?;
    authored_town_publication::replace(pack, court)?;

    let mut quarry = pack
        .data
        .maps
        .get("MnMankatoGym")
        .context("Mankato gym")?
        .clone();
    ensure!(
        quarry.attributes.tileset_name == "mn_gym_2",
        "unexpected quarry tileset"
    );
    extend(pack, "mn_gym_2", 0x13, &QUARRY_EDGES)?;
    place(
        &mut quarry,
        &[
            (0, 3, 0x0b, 0x13),
            (3, 3, 0x0a, 0x14),
            (5, 3, 0x0d, 0x14),
            (6, 2, 0x10, 0x11),
        ],
    )?;
    let label = "MnMankatoGymV2Gate1Open";
    let gate = quarry.scripts.get_mut(label).context("east quarry gate")?;
    ensure!(
        *gate
            == json!([
                command("changeblock", &["12", "6", "17"]),
                command("end", &[])
            ]),
        "unexpected open quarry gate"
    );
    *gate = json!([
        command("changeblock", &["12", "6", "16"]),
        command("end", &[])
    ]);
    quarry.script_block_changes = crate::parse_script_block_changes(&quarry.id, &quarry.scripts)?;
    authored_town_publication::replace(pack, quarry)
}
