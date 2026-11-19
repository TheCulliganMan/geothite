//! Keep the public Safari trail, Granite stair and cabin healer distinct.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const CABIN: &str = "MnBoundaryRootBeerCabin";
const MEDIC: &str = "MN_INTERIOR_DOROTHY";
const NURSE: &str = "SPRITE_NURSE";

fn appoint_medic(room: &mut MapModule) -> Result<()> {
    let actor = room
        .objects
        .iter_mut()
        .find(|actor| actor.script == MEDIC)
        .context("cabin healer")?;
    ensure!(
        [actor.x, actor.y] == [5, 3] && actor.sprite == "SPRITE_GRANNY" && actor.sprite_has_facings,
        "unexpected indoor Dorothy actor"
    );
    actor.sprite = NURSE.into();
    // Preserve the original event operands and its position in the object roster.
    crate::minnesota_trainer_art::rebind_actor_event_source(room, MEDIC, [5, 3], NURSE)?;
    let row = room
        .map_event_section_commands
        .iter_mut()
        .find(|row| row.command == "object_event" && row.args.get(11).is_some_and(|id| id == MEDIC))
        .context("cabin healer event")?;
    ensure!(
        row.args
            .starts_with(&["5".into(), "3".into(), "SPRITE_GRANNY".into()]),
        "unexpected original cabin healer event"
    );
    row.args[2] = NURSE.into();
    Ok(())
}

/// Updates three native text labels and gives the indoor healer the camp medic's sprite.
///
/// Returns an error if an original label, healer or its native appearance has changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, label, anchor) in [
        (
            "MnBoundarySafariC00R02",
            "BWS_REED_RETURN_RANGERText",
            command("para", &["\"Done with SAFARI?\""]),
        ),
        (
            "MnBoundarySafariC01R00",
            "BWS_SIGN_05Text",
            command("line", &["\"DROP TO RETURN\""]),
        ),
        (
            CABIN,
            "MN_INTERIOR_DOROTHYInteriorText",
            command("text", &["\"I'm DOROTHY. Put\""]),
        ),
    ] {
        let mut room = pack
            .data
            .maps
            .get(name)
            .with_context(|| format!("Safari map {name}"))?
            .clone();
        let old = room
            .scripts
            .get(label)
            .with_context(|| format!("Safari text {label}"))?;
        ensure!(
            old.as_array().is_some_and(|lines| lines.contains(&anchor))
                && pack.data.map_scripts.get(label) == Some(old),
            "unexpected original Safari text {label}"
        );
        authored_dialogue::install_canonical(&mut room, label)?;
        let text = BTreeMap::from([(label.to_owned(), room.scripts[label].clone())]);
        room.script_text_bodies
            .extend(crate::parse_script_text_bodies(name, &text)?);
        if name == CABIN {
            appoint_medic(&mut room)?;
        }
        authored_town_publication::replace(pack, room)?;
    }
    Ok(())
}
