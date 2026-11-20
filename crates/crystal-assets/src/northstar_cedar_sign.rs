//! Set the Cedar Inlet trail board on the land beside the west bridgehead.

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;
use crate::authored_town_publication;

const MAP: &str = "MnR08C00R01";
const TILESET: &str = "mn_headwaters_thaw";
const SIGN: &str = "MN_R08_SIGN_02";
const ATLAS: &str = "data/tilesets/mn_headwaters_thaw_metatiles.bin";
const BOARD_BLOCK: u16 = 0xb9;

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(MAP).context(MAP)?.clone();
    let width = usize::from(room.attributes.width);
    ensure!(
        room.attributes.tileset_name == TILESET
            && (width, room.attributes.height) == (32, 24)
            && room.blocks.len() == 32 * 24
            && room.blocks[3 * width + 11] == 0xb8
            && room.blocks[2 * width + 16] == 0x86,
        "unexpected Cedar Inlet pine or west bridge approach"
    );
    let atlas = pack.runtime_files.get_mut(ATLAS).context(ATLAS)?;
    ensure!(
        atlas.len() == 16 * usize::from(BOARD_BLOCK),
        "unexpected Headwaters metatile bank"
    );
    // The four upper-left glyphs are the region's ordinary freestanding sign.
    // Every other glyph stays the same as the surrounding thawed trail.
    let mut block = [0; 16];
    block.copy_from_slice(&atlas[16 * 0x86..16 * 0x87]);
    for i in [0, 1, 4, 5] {
        block[i] = atlas[16 * 0x45 + i];
    }
    atlas.extend_from_slice(&block);
    let bank = pack.data.tilesets.get_mut(TILESET).context(TILESET)?;
    let collision = bank.collision.get("45").context("trail board collision")?;
    ensure!(
        collision == &["WALL", "FLOOR", "FLOOR", "FLOOR"],
        "native trail board must block only its own square"
    );
    let collision = collision.clone();
    ensure!(
        bank.collision
            .insert(format!("{BOARD_BLOCK:02x}"), collision)
            .is_none(),
        "Cedar Inlet board slot is occupied"
    );
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&bank.collision)?,
    );
    room.blocks[3 * width + 11] = 0x8a;
    room.blocks[2 * width + 16] = BOARD_BLOCK;

    let event = room
        .events
        .bg_events
        .iter_mut()
        .find(|e| e.script == SIGN)
        .context(SIGN)?;
    ensure!(
        (event.x, event.y) == (22, 6) && event.event_type == "BGEVENT_READ",
        "expected the old Cedar Inlet pine plaque"
    );
    (event.x, event.y) = (32, 4);
    let wire = room
        .map_event_section_commands
        .iter_mut()
        .find(|r| r.command == "bg_event" && r.args == ["22", "6", "BGEVENT_READ", SIGN])
        .context("Cedar Inlet native reading event")?;
    wire.args[0] = event.x.to_string();
    wire.args[1] = event.y.to_string();
    authored_town_publication::replace(pack, room)
}
