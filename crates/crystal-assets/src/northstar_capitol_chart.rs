//! Place a readable river chart on the Capitol Library's back desk.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::{command, refresh_event_section};
use crate::northstar_rival_arc::put;
use crate::northstar_rival_art;

const ROOM: &str = "MnStPaulCapitolLibrary";
const TILESET: &str = "mn_capitol_library";
const CHART: &str = "MnCapitolRiverChart";
const CHART_TEXT: &str = "MnCapitolRiverChartText";
const HOST_TEXT: &str = "MN_SPX_CAPITOLLIBRARY_HOSTText";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context("Capitol Library")?.clone();
    ensure!(
        room.attributes.tileset_name == "house"
            && (room.attributes.width, room.attributes.height) == (8, 6)
            && room.blocks.get(21) == Some(&0x0f)
            && room.events.bg_events.len() == 1
            && room.events.bg_events[0].script == "MN_INTERIOR_DETAIL_CAPITOL_LENDING_SHELF"
            && !room.scripts.contains_key(CHART),
        "expected the original Capitol Library shelves and open east aisle"
    );

    // The radio station's desk has a loose sheet on its left half. Replace the
    // stool on its right with the library floor so readers can walk around it.
    let desk = northstar_rival_art::import_block(pack, "house", TILESET, "radio_tower", 0x25)?;
    let tiles = pack
        .runtime_files
        .get_mut(&format!("data/tilesets/{TILESET}_metatiles.bin"))
        .context("private library metatiles")?;
    for row in [0, 4, 8, 12] {
        let aisle = 0x0f * 16 + row + 2;
        tiles.copy_within(aisle..aisle + 2, usize::from(desk) * 16 + row + 2);
    }
    let definition = pack
        .data
        .tilesets
        .get_mut(TILESET)
        .context("library desk")?;
    let collision = definition
        .collision
        .get_mut(&format!("{desk:02x}"))
        .context("desk collision")?;
    ensure!(
        collision.as_slice() == ["COUNTER", "FLOOR", "COUNTER", "FLOOR"],
        "expected a narrow writing desk"
    );
    for quadrant in [0, 2] {
        collision[quadrant] = "WALL".into();
    }
    pack.runtime_files.insert(
        format!("data/tilesets/{TILESET}.json"),
        serde_json::to_vec(&definition.collision)?,
    );
    room.blocks[21] = desk;
    room.attributes.tileset_name = TILESET.into();
    for y in [4, 5] {
        room.events.bg_events.push(BackgroundEvent {
            x: 10,
            y,
            event_type: "BGEVENT_READ".into(),
            script: CHART.into(),
        });
    }
    refresh_event_section(&mut room);
    put(&mut room, CHART, vec![command("jumptext", &[CHART_TEXT])]);
    for label in [CHART_TEXT, HOST_TEXT] {
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
