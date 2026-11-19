//! Return Mina and Sera to the current stage of the lake investigation.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const TOWN: &str = "MnStCloud";
const SERA: &str = "MN_ARC_SC_SERA";
const REED: &str = "MN_ARC_SC_REED";
const RELEASED: &str = "EVENT_MN_REED_WORKER_RELEASED";
const DRILL: &str = "EVENT_MN_DRILL_STOPPED";
const RESTORED: &str = "EVENT_MN_SURVEY_RESTORED";
const STATION: &str = "MnArcSCWorkerHome";
const END: &str = "MnArcSCEnd";

fn script<'a>(room: &'a mut MapModule, label: &str) -> Result<&'a mut Vec<Value>> {
    room.scripts
        .get_mut(label)
        .and_then(Value::as_array_mut)
        .with_context(|| format!("missing central journey scene {label}"))
}

type Dialogue = BTreeMap<String, Vec<AuthoredDialoguePage>>;

#[derive(Deserialize)]
struct Story {
    central_mina_dialogue: Dialogue,
}

fn compare_lake_log(pack: &mut CompiledGamePack, text: &Dialogue) -> Result<()> {
    const DEPOT: &str = "MnStPaulUnionDepot";
    const REPORT: &str = "EVENT_MN_BRAINERD_REPORT";
    let mut room = pack.data.maps.get(DEPOT).context(DEPOT)?.clone();
    let offer = script(&mut room, "MnFlowMinaOfferDate")?;
    ensure!(
        offer.get(1) == Some(&command("writetext", &["MnNSTPMinaReadyText"]))
            && offer.get(4) == Some(&command("writetext", &["MnNSTPMinaQuestText"]))
            && offer.get(6) == Some(&command("setevent", &["EVENT_MN_DEEP_BORE_MANIFEST_READ"])),
        "unexpected Mina date offer"
    );
    // Keep the same yes/no, refusal, gift receipt and text-window ownership
    // when a player brings the lake log before accepting Mina's paper.
    let mut compare = offer.clone();
    compare[1] = command("writetext", &["MnCentralMinaCompareOfferText"]);
    compare[4] = command("writetext", &["MnCentralMinaCompareGivenText"]);
    offer.splice(
        0..0,
        [
            command("checkevent", &[REPORT]),
            command("iftrue", &["MnCentralMinaCompare"]),
        ],
    );
    let after = script(&mut room, "MnNSTPMinaAfter")?;
    ensure!(
        after.as_slice() == [command("jumptext", &["MnNSTPMinaAfterText"])],
        "unexpected Mina reminder"
    );
    after.splice(
        0..0,
        [
            command("checkevent", &[REPORT]),
            command("iftrue", &["MnCentralMinaLakeLog"]),
        ],
    );
    for (label, body) in [
        ("MnCentralMinaCompare", compare),
        (
            "MnCentralMinaLakeLog",
            vec![command("jumptext", &["MnCentralMinaLakeLogText"])],
        ),
    ] {
        ensure!(
            room.scripts
                .insert(label.into(), Value::Array(body))
                .is_none(),
            "occupied Mina branch {label}"
        );
    }
    for label in [
        "MnCentralMinaCompareOfferText",
        "MnCentralMinaCompareGivenText",
        "MnCentralMinaLakeLogText",
    ] {
        let pages = text
            .get(label)
            .with_context(|| format!("missing Mina pages {label}"))?;
        ensure!(
            room.scripts
                .insert(label.into(), authored_dialogue::compile(pages)?)
                .is_none(),
            "occupied Mina text {label}"
        );
    }
    room.script_flag_commands = crate::parse_script_flag_commands(DEPOT, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(DEPOT, &room.scripts)?;
    room.script_text_commands = crate::parse_script_text_commands(DEPOT, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(DEPOT, &room.scripts)?;
    crate::authored_town_publication::replace(pack, room)
}

/// Places Sera beside the depot and lets Mina read a previously collected lake log.
///
/// Returns an error when the expected original confrontation or return is missing.
pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    let story: Story =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    compare_lake_log(pack, &story.central_mina_dialogue)?;
    let mut room = pack.data.maps.get(TOWN).context(TOWN)?.clone();
    let callback = script(&mut room, STATION)?;
    let old_mark = command("moveobject", &[SERA, "24", "9"]);
    ensure!(
        callback.get(2) == Some(&old_mark) && callback.get(3) == Some(&command("appear", &[SERA])),
        "unexpected Sera return position"
    );
    callback[2] = command("moveobject", &[SERA, "38", "6"]);
    callback.insert(4, command("turnobject", &[SERA, "DOWN"]));

    let scene = script(&mut room, "MnArcSCScene")?;
    let depart = command("applymovement", &[SERA, "MnArcSeraLeaves"]);
    let begin = scene
        .iter()
        .position(|s| s == &depart)
        .context("Sera's departure")?;
    let movements = [
        depart,
        command("applymovement", &[REED, "MnArcSCReedLeaves"]),
        command("disappear", &[REED]),
    ];
    let thanks = [
        command("opentext", &[]),
        command("writetext", &["MnArcSCReleasedText"]),
        command("promptbutton", &[]),
        command("closetext", &[]),
    ];
    ensure!(
        scene[begin..].starts_with(&movements)
            && scene[begin + movements.len()..].starts_with(&thanks),
        "unexpected Sera farewell"
    );
    // Let her thank the player face to face before walking around the woods.
    scene.splice(
        begin..begin + movements.len() + thanks.len(),
        [command("faceobject", &[SERA, "PLAYER"])]
            .into_iter()
            .chain(thanks)
            .chain(movements)
            .chain([command("turnobject", &[SERA, "DOWN"])]),
    );

    let walk = script(&mut room, "MnArcSeraLeaves")?;
    ensure!(
        walk.len() == 7
            && walk.first() == Some(&command("step", &["RIGHT"]))
            && walk.last() == Some(&command("step_end", &[])),
        "unexpected north-road walk"
    );
    walk.clear();
    // The two lanes at x24-25 sit in a stand of trees. The nearest cross
    // street is y12; the depot's unobstructed pavement starts at x38.
    for (direction, count) in [("RIGHT", 1), ("DOWN", 7), ("RIGHT", 13), ("UP", 6)] {
        walk.extend(std::iter::repeat_with(|| command("step", &[direction])).take(count));
    }
    walk.push(command("step_end", &[]));

    for (label, target) in [
        ("MnArcSCActors", STATION),
        (SERA, "MnArcSCAfter"),
        ("MnArcSCEntry24", END),
        ("MnArcSCEntry25", END),
        ("MnArcSCScene", END),
    ] {
        let body = script(&mut room, label)?;
        ensure!(
            body.first() == Some(&command("checkevent", &[RELEASED])),
            "unexpected Sera encounter guard in {label}"
        );
        body.splice(
            0..0,
            [
                command("checkevent", &[RESTORED]),
                command("iftrue", &[target]),
                command("checkevent", &[DRILL]),
                command("iftrue", &[target]),
            ],
        );
    }
    ensure!(
        room.script_text_bodies.contains_key("MnArcSCDeclinedText"),
        "missing original refusal"
    );
    authored_dialogue::install_canonical(&mut room, "MnArcSCDeclinedText")?;
    room.script_flag_commands = crate::parse_script_flag_commands(TOWN, &room.scripts)?;
    room.script_control_commands = crate::parse_script_control_commands(TOWN, &room.scripts)?;
    room.script_object_commands = crate::parse_script_object_commands(TOWN, &room.scripts)?;
    room.script_movements =
        crate::parse_script_movements(TOWN, &room.scripts, &room.script_object_commands)?;
    room.script_text_commands = crate::parse_script_text_commands(TOWN, &room.scripts)?;
    room.script_text_bodies = crate::parse_script_text_bodies(TOWN, &room.scripts)?;
    room.script_runtime_commands = crate::parse_script_runtime_commands(TOWN, &room.scripts)?;
    crate::authored_town_publication::replace(pack, room)
}
