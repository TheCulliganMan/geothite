//! Let the harbor and shelter crews remember calls already answered on the intake stairs.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const LINK: &str = "EVENT_MN_ARC_LINK_SENT";
const LANDING: &str = "EVENT_MN_VERDIGRIM_COUNTER_CURRENT";

#[derive(Deserialize)]
struct Story {
    regional_guidance_revision: BTreeMap<String, Guide>,
    harbor_return_dialogue: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

#[derive(Deserialize)]
struct Guide {
    replies: Vec<Reply>,
}

#[derive(Deserialize)]
struct Reply {
    id: String,
    pages: Vec<AuthoredDialoguePage>,
}

fn add_reply(
    room: &mut MapModule,
    entry: &str,
    flag: &str,
    command_name: &str,
    original_text: &str,
    branch: &str,
    text: &str,
) -> Result<()> {
    let reply = room
        .scripts
        .get_mut(entry)
        .and_then(Value::as_array_mut)
        .with_context(|| format!("missing native return script {entry}"))?;
    let original = command(command_name, &[original_text]);
    ensure!(
        reply.last() == Some(&original),
        "unexpected original reply in {entry}"
    );
    let at = reply.len() - 1;
    reply.splice(
        at..at,
        [command("checkevent", &[flag]), command("iftrue", &[branch])],
    );
    ensure!(
        room.scripts
            .insert(
                branch.into(),
                Value::Array(vec![command(command_name, &[text])])
            )
            .is_none(),
        "occupied return branch {branch}"
    );
    Ok(())
}

/// Add return conversations for players who leave the shelter radio or the intake landing.
///
/// Returns an error if a native dialogue branch or its reviewed preimage has changed.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let story: Story =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    for (name, labels) in story.harbor_return_dialogue {
        let mut room = pack
            .data
            .maps
            .get(&name)
            .with_context(|| format!("missing {name}"))?
            .clone();
        match name.as_str() {
            "MnDuluthHarborWorkshop" => {
                add_reply(
                    &mut room,
                    "MN_HARBOR_RECEIVERDone",
                    LINK,
                    "jumptextfaceplayer",
                    "MN_HARBOR_RECEIVERDoneText",
                    "MnHarborIntakeReturn",
                    "MnHarborIntakeReturnText",
                )?;
            }
            "MnBoundaryWatersSurveyShelter" => {
                add_reply(
                    &mut room,
                    "MnWayStopDrive",
                    LANDING,
                    "jumptextfaceplayer",
                    "MnWayStopDriveText",
                    "MnShelterCrewBelow",
                    "MnShelterCrewBelowText",
                )?;
                add_reply(
                    &mut room,
                    "MnArcLinkAfter",
                    LANDING,
                    "jumptext",
                    "MnArcLinkAfterText",
                    "MnShelterRadioBelow",
                    "MnShelterRadioBelowText",
                )?;
            }
            _ => anyhow::bail!("unexpected harbor return room {name}"),
        }
        for (label, pages) in labels {
            ensure!(
                room.scripts
                    .insert(label.clone(), authored_dialogue::compile(&pages)?)
                    .is_none(),
                "occupied native text {label}"
            );
        }
        room.script_flag_commands = crate::parse_script_flag_commands(&name, &room.scripts)?;
        room.script_control_commands = crate::parse_script_control_commands(&name, &room.scripts)?;
        room.script_text_commands = crate::parse_script_text_commands(&name, &room.scripts)?;
        room.script_text_bodies = crate::parse_script_text_bodies(&name, &room.scripts)?;
        crate::authored_town_publication::replace(pack, room)?;
    }
    let mut town = pack
        .data
        .maps
        .get("MnBoundaryWaters")
        .context("Boundary public trails")?
        .clone();
    authored_dialogue::install_canonical(&mut town, "MN_BOUNDARY_SOUTH_TRAILText")?;
    let ranger = story
        .regional_guidance_revision
        .get(&town.id)
        .and_then(|guide| guide.replies.iter().find(|r| r.id == "MnWayLeagueTrail"))
        .context("restored shore ranger reply")?;
    ensure!(
        town.scripts.contains_key("MnWayLeagueTrailText"),
        "missing ranger's existing trail dialogue"
    );
    town.scripts.insert(
        "MnWayLeagueTrailText".into(),
        authored_dialogue::compile(&ranger.pages)?,
    );
    town.script_text_bodies = crate::parse_script_text_bodies(&town.id, &town.scripts)?;
    crate::authored_town_publication::replace(pack, town)
}
