//! Give the Detroit ranger a useful page for each outstanding Route 19 observation.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_dialogue;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring::{self, command as c};

const CABIN: &str = "MnDetroitLakesRangerCabin";
const TRAIL: &str = "MnR19C00R00";
const INTRO: &str = "MnDetroitNotesMissing";
const DONE: &str = "MnDetroitNotesEndList";
const STOPS: [(&str, &str, &str); 3] = [
    (
        "MnDetroitCheckPrairie",
        "EVENT_MN_LAKE_OBSERVATION_03",
        "MnDetroitPrairieNoteText",
    ),
    (
        "MnDetroitCheckWetland",
        "EVENT_MN_LAKE_OBSERVATION_02",
        "MnDetroitWetlandNoteText",
    ),
    (
        "MnDetroitCheckPine",
        "EVENT_MN_LAKE_OBSERVATION_01",
        "MnDetroitPineNoteText",
    ),
];

pub(crate) fn apply(pack: &mut CompiledGamePack) -> Result<()> {
    open_public_gate(pack)?;
    mark_observations(pack)
}

fn open_public_gate(pack: &mut CompiledGamePack) -> Result<()> {
    const GATE: &str = "MnR19DetroitGate";
    const TOWN: &str = "MnDetroitLakes";
    let mut gate = pack
        .data
        .maps
        .get(GATE)
        .context("Detroit trail gate")?
        .clone();
    ensure!(
        gate.events.coord_events.len() == 2
            && [4, 5].into_iter().all(|x| gate
                .events
                .coord_events
                .iter()
                .any(|e| e.x == x && e.y == 2 && e.script_name == "MnR19GateCheck")),
        "unexpected gate crossing programs"
    );
    gate.events.coord_events.clear();
    for label in [
        "MnR19GateCheck",
        "MnR19GateQualify",
        "MnR19GateStepSouth",
        "MnR19GateDeniedText",
        "MnR19GateDone",
    ] {
        gate.scripts
            .remove(label)
            .with_context(|| format!("missing superseded gate program {label}"))?;
    }
    authoring::finish(pack, &mut gate)?;
    let event_label = gate
        .attributes
        .map_events_label
        .as_ref()
        .context("Detroit gate events")?;
    if let Some(raw) = gate.scripts.get_mut(event_label) {
        *raw = Value::Array(
            gate.map_event_section_commands
                .iter()
                .map(|r| json!({"command":r.command,"args":r.args}))
                .collect(),
        );
    }
    gate.script_variable_commands = crate::parse_script_variable_commands(GATE, &gate.scripts)?;
    gate.script_object_commands = crate::parse_script_object_commands(GATE, &gate.scripts)?;
    gate.script_movements =
        crate::parse_script_movements(GATE, &gate.scripts, &gate.script_object_commands)?;
    gate.script_runtime_commands = crate::parse_script_runtime_commands(GATE, &gate.scripts)?;
    authored_town_publication::replace(pack, gate)?;
    let mut city = pack.data.maps.get(TOWN).context("Detroit Lakes")?.clone();
    authored_dialogue::install_canonical(&mut city, "MN_DETROIT_NORTH_SIGNText")?;
    city.script_text_bodies = crate::parse_script_text_bodies(TOWN, &city.scripts)?;
    authored_town_publication::replace(pack, city)
}

fn mark_observations(pack: &mut CompiledGamePack) -> Result<()> {
    let mut cabin = pack
        .data
        .maps
        .get(CABIN)
        .context("Detroit ranger cabin")?
        .clone();
    ensure!(
        cabin.scripts.get(INTRO)
            == Some(&json!([
                c("writetext", &["MnDetroitNotesMissingText"]),
                c("waitbutton", &[]),
                c("closetext", &[]),
                c("end", &[])
            ])),
        "unexpected Route 19 incomplete-report conversation"
    );
    for label in ["MnDetroitNotesMissingText", "MnDetroitNotesReviewText"] {
        authored_dialogue::install_canonical(&mut cabin, label)?;
    }
    cabin.scripts.insert(
        INTRO.into(),
        json!([
            c("writetext", &["MnDetroitNotesMissingText"]),
            c("promptbutton", &[]),
            c("sjump", &[STOPS[0].0])
        ]),
    );
    for (index, (branch, flag, clue)) in STOPS.iter().copied().enumerate() {
        ensure!(
            !cabin.scripts.contains_key(branch)
                && !cabin.scripts.contains_key(clue)
                && pack
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(flag),
            "missing observation or occupied script {branch}"
        );
        authored_dialogue::install_canonical(&mut cabin, clue)?;
        let next = STOPS.get(index + 1).map_or(DONE, |stop| stop.0);
        cabin.scripts.insert(
            branch.into(),
            json!([
                c("checkevent", &[flag]),
                c("iftrue", &[next]),
                c("writetext", &[clue]),
                c("promptbutton", &[]),
                c("sjump", &[next])
            ]),
        );
    }
    ensure!(
        cabin
            .scripts
            .insert(
                DONE.into(),
                Value::Array(vec![c("closetext", &[]), c("end", &[])])
            )
            .is_none(),
        "occupied Route 19 list end"
    );
    authoring::finish(pack, &mut cabin)?;
    cabin.script_runtime_commands = crate::parse_script_runtime_commands(CABIN, &cabin.scripts)?;
    authored_town_publication::replace(pack, cabin)?;

    let mut trail = pack
        .data
        .maps
        .get(TRAIL)
        .context("prairie observation")?
        .clone();
    let attrs = &trail.attributes;
    ensure!(
        attrs.width == 32 && attrs.height == 24 && attrs.tileset_name == "mn_outdoor_johto_stairs",
        "unexpected prairie observation terrain"
    );
    let collision = &pack.data.tilesets[&attrs.tileset_name].collision;
    ensure!(
        collision.get("02") == collision.get("04") && collision.contains_key("04"),
        "native meadow flowers must retain all four ordinary grass permissions"
    );
    // Extend the existing flowers around the grassy sides of the bend, keeping its pale walking lane intact.
    for (x, y) in [(8, 22), (9, 22), (11, 23)] {
        let block = &mut trail.blocks[y * 32 + x];
        ensure!(
            *block == 0x02,
            "unexpected ground by the flower reading at ({x},{y})"
        );
        *block = 0x04;
    }
    authored_town_publication::replace(pack, trail)
}
