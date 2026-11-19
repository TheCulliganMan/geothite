//! Fit Mitwist's hollow between mature pines and their sheltering young growth.
use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const FEN: &str = "MnLayeredFen";
const RESTORE: &str = "MnFenRestoreTerrain";
const OPEN_TEXT: &str = "MnFenText_hollow_open";
const RETURN_TEXT: &str = "MnFenText_hollow_return";

// West and east stands taper at different points on the walk up to Mitwist.
// These are stock Johto pines and saplings. None removes through Cut.
const TREES: [(usize, usize, u16, u16); 13] = [
    (15, 1, 0x02, 0x05),
    (16, 1, 0x02, 0x05),
    (14, 2, 0x02, 0x05),
    (15, 2, 0x02, 0x5c),
    (16, 2, 0x04, 0x3c),
    (17, 2, 0x02, 0x62),
    (14, 3, 0x02, 0x5d),
    (15, 3, 0x04, 0x02),
    (16, 3, 0x02, 0x3f),
    (17, 3, 0x02, 0x05),
    (15, 4, 0x02, 0x3e),
    (16, 4, 0x02, 0x05),
    (17, 4, 0x02, 0x05),
];

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(FEN).context(FEN)?.clone();
    let width = usize::from(room.attributes.width);
    ensure!(
        width == 20 && room.attributes.tileset_name == "johto" && room.blocks[4 * width + 13] == 5,
        "expected the trail's closed northern pine gate"
    );
    for (x, y, before, after) in TREES {
        let block = room.blocks.get_mut(y * width + x).context("Fen hollow")?;
        ensure!(*block == before, "unexpected hollow terrain at ({x},{y})");
        *block = after;
    }
    let rows = room
        .scripts
        .get_mut(RESTORE)
        .and_then(|v| v.as_array_mut())
        .context(RESTORE)?;
    let opening = rows.get_mut(2).context("Fen trail reveal")?;
    ensure!(
        *opening == command("changeblock", &["26", "8", "1"]),
        "expected the original gravel opening"
    );
    // The surviving sapling stands on the south shoulder, so three ordinary
    // ground squares lead into the recess. Reentry restores this same terrain.
    *opening = command("changeblock", &["26", "8", "62"]);
    for (script, index, verb, before, after) in [
        (
            "MnFenTrailOpened",
            0,
            "jumptext",
            "MnFenText_folded_path",
            OPEN_TEXT,
        ),
        (
            "MnFenObserverAfter",
            3,
            "writetext",
            "MnFenText_observer_after",
            RETURN_TEXT,
        ),
        (
            "MnFenSouthBallFound",
            0,
            "jumptext",
            "MnFenText_observer_after",
            RETURN_TEXT,
        ),
    ] {
        let line = room
            .scripts
            .get_mut(script)
            .and_then(|body| body.get_mut(index))
            .context(script)?;
        ensure!(
            *line == command(verb, &[before]),
            "unexpected {script} direction"
        );
        *line = command(verb, &[after]);
    }
    for label in [OPEN_TEXT, RETURN_TEXT] {
        ensure!(
            !pack.data.map_scripts.contains_key(label),
            "occupied Fen page {label}"
        );
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    room.script_control_commands = crate::parse_script_control_commands(FEN, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(FEN, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(FEN, &room.scripts)?;
    room.script_block_changes = crate::parse_script_block_changes(FEN, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(FEN, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
