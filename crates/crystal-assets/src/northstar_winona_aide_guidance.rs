//! Give the Winona aide the right directions at each stage of the first errand.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

const ROOM: &str = "MnWinonaFieldStation";
const AIDE: &str = "MnKindlitResearchAide";
const MEET: &str = "MnWinonaAideMeet";
const MEET_TEXT: &str = "MnWinonaAideMeetText";
const READINGS: &str = "MnWinonaAideReadingsText";

pub(super) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(ROOM).context(ROOM)?.clone();
    ensure!(
        room.objects
            .iter()
            .any(|a| a.object_identifier.as_deref() == Some("MN_W_AIDE") && a.script == AIDE)
            && room.scripts.get(AIDE)
                == Some(&json!([
                    command("scall", &["MN_W_AIDE"]),
                    command("sjump", &["MnEarlyPartners"]),
                ])),
        "unexpected Winona research aide"
    );
    for label in [MEET, MEET_TEXT, READINGS] {
        ensure!(
            !pack.data.map_scripts.contains_key(label) && !room.scripts.contains_key(label),
            "occupied Winona aide label {label}"
        );
    }
    room.scripts.insert(
        AIDE.into(),
        json!([
            command("checkevent", &["EVENT_MN_STARTER_RECEIVED"]),
            command("iffalse", &[MEET]),
            command("scall", &["MN_W_AIDE"]),
            command("checkevent", &["EVENT_MN_DELIVERED_ROCHESTER_PARCEL"]),
            command("iftrue", &["MnEarlyPartners"]),
            command("checkevent", &["EVENT_MN_R01_PARCEL_RECEIVED"]),
            command("iftrue", &["MnEarlyPartners"]),
            command("jumptextfaceplayer", &[READINGS]),
        ]),
    );
    room.scripts.insert(
        MEET.into(),
        json!([command("jumptextfaceplayer", &[MEET_TEXT])]),
    );
    for label in [MEET_TEXT, READINGS] {
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    room.script_control_commands = crate::parse_script_control_commands(ROOM, &room.scripts)?;
    room.script_flag_commands = crate::parse_script_flag_commands(ROOM, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(ROOM, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(ROOM, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
