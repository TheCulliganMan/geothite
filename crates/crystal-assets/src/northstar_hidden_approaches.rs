//! Let the angler on Duluth's outer float point out the submerged snag beside it.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;

const MAP: &str = "MnDuluth";
const FISHER: &str = "MnDuluthFloatingFisher2";
const ORIGINAL: &str = "MnDuluthFloatingFisher2Text";
const CLUE: &str = "MnDuluthFloatSnagText";
const AFTER: &str = "MnDuluthFloatAfterSnag";
const FLAG: &str = "EVENT_MN_HIDDEN_DULUTH";

/// Adds the angler's pre-collection line and keeps his harbor reminiscence after pickup.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let mut room = pack.data.maps.get(MAP).context("Duluth harbor")?.clone();
    ensure!(
        room.objects
            .iter()
            .any(|actor| actor.script == FISHER && (actor.x, actor.y) == (71, 81))
            && room
                .events
                .bg_events
                .iter()
                .any(|event| event.event_type == "BGEVENT_ITEM"
                    && event.script == "MN_HIDDEN_DULUTH"
                    && (event.x, event.y) == (68, 81))
            && room.scripts.get("MN_HIDDEN_DULUTH")
                == Some(&json!([command("hiddenitem", &["NUGGET", FLAG])]))
            && room.scripts.get(FISHER)
                == Some(&json!([command("jumptextfaceplayer", &[ORIGINAL])])),
        "expected the angler and submerged Nugget at the outer float"
    );
    let pages: Vec<AuthoredDialoguePage> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/lake-observations-v01/duluth-float-clue.json"
    ))?;
    ensure!(
        !room.scripts.contains_key(CLUE) && !room.scripts.contains_key(AFTER),
        "occupied Duluth clue"
    );
    room.scripts
        .insert(CLUE.into(), authored_dialogue::compile(&pages)?);
    room.scripts.insert(
        AFTER.into(),
        json!([command("jumptextfaceplayer", &[ORIGINAL])]),
    );
    room.scripts.insert(
        FISHER.into(),
        json!([
            command("checkevent", &[FLAG]),
            command("iftrue", &[AFTER]),
            command("jumptextfaceplayer", &[CLUE])
        ]),
    );
    room.script_flag_commands = crate::parse_script_flag_commands(MAP, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(MAP, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(MAP, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(MAP, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
