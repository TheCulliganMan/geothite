//! Give two pre-League gym leaders directions that follow the player's journey.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, authored_dialogue, authored_town_publication};

/// Updates Brannic and Sivren's repeat replies after their badge and TM are earned.
///
/// Returns an error if the source gym has a different handoff or a label is occupied.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let brannic = json!([
        command("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
        command("iftrue", &["MnBadgeBrannicNextGym"]),
        command("checkevent", &["EVENT_MN_DEEP_BORE_MANIFEST_READ"]),
        command("iftrue", &["MnBadgeBrannicNextGym"]),
        command("checkevent", &["EVENT_MN_DISPATCH_NOTE"]),
        command("iftrue", &["MnBadgeBrannicMina"]),
        command("sjump", &["MnBadgeBrannicNextGym"]),
    ]);
    install(
        pack,
        "MnStPaulGym",
        brannic,
        &[
            (
                "MnBadgeBrannicNextGym",
                json!([
                    command("checkflag", &["ENGINE_STORMBADGE"]),
                    command("iftrue", &["MnBadgeBrannicBreeze"]),
                    command("jumptext", &["MnBadgeBrannicWindText"]),
                ]),
            ),
            (
                "MnBadgeBrannicMina",
                json!([command("jumptext", &["MnBadgeBrannicMinaText"])]),
            ),
            (
                "MnBadgeBrannicBreeze",
                json!([command("jumptext", &["MnBadgeBrannicBreezeText"])]),
            ),
        ],
        &[
            "MnBadgeBrannicWindText",
            "MnBadgeBrannicMinaText",
            "MnBadgeBrannicBreezeText",
        ],
    )?;
    let sivren = json!([
        command("checkevent", &["EVENT_MN_SURVEY_RESTORED"]),
        command("iftrue", &["MnBadgeSivrenWake"]),
        command("checkevent", &["EVENT_MN_HEADWATERS_REPORT"]),
        command("iftrue", &["MnBadgeSivrenWake"]),
        command("checkevent", &["EVENT_MN_HEADWATERS_READING"]),
        command("iftrue", &["MnBadgeSivrenOfficer"]),
        command("checkevent", &["EVENT_MN_BRAINERD_REPORT"]),
        command("iftrue", &["MnBadgeSivrenItasca"]),
        command("jumptext", &["MnBrainerdGymLeaderAfterText"]),
    ]);
    install(
        pack,
        "MnBrainerdGym",
        sivren,
        &[
            (
                "MnBadgeSivrenWake",
                json!([command("jumptext", &["MnBadgeSivrenWakeText"])]),
            ),
            (
                "MnBadgeSivrenOfficer",
                json!([command("jumptext", &["MnBadgeSivrenOfficerText"])]),
            ),
            (
                "MnBadgeSivrenItasca",
                json!([command("jumptext", &["MnBadgeSivrenItascaText"])]),
            ),
        ],
        &[
            "MnBadgeSivrenWakeText",
            "MnBadgeSivrenOfficerText",
            "MnBadgeSivrenItascaText",
        ],
    )
}

fn install(
    pack: &mut CompiledGamePack,
    name: &str,
    reply: Value,
    branches: &[(&str, Value)],
    labels: &[&str],
) -> Result<()> {
    let mut room = pack
        .data
        .maps
        .get(name)
        .context("missing badge gym")?
        .clone();
    let repeat = format!("{name}RematchBeforeChampion");
    let original = json!([command("jumptext", &[&format!("{name}LeaderAfterText")])]);
    ensure!(
        room.scripts.get(&repeat) == Some(&original)
            && pack.data.map_scripts.get(&repeat) == Some(&original),
        "unexpected post-badge conversation in {name}"
    );
    for label in branches
        .iter()
        .map(|(k, _)| *k)
        .chain(labels.iter().copied())
    {
        ensure!(
            !room.scripts.contains_key(label) && !pack.data.map_scripts.contains_key(label),
            "occupied gym script {label}"
        );
    }
    room.scripts.insert(repeat, reply);
    for (branch, program) in branches {
        room.scripts.insert((*branch).to_owned(), program.clone());
    }
    for label in labels {
        authored_dialogue::install_canonical(&mut room, label)?;
    }
    room.script_flag_commands = crate::parse_script_flag_commands(name, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(name, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(name, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(name, &room.scripts)?;
    authored_town_publication::replace(pack, room)
}
