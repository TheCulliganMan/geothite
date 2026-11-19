//! Two staffed trailheads on the winter mail run between the Falls and Rainy River.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::{command as c, refresh_event_section};

#[derive(Deserialize)]
struct Turnout {
    at: [u16; 2],
    from: u16,
    to: u16,
}

#[derive(Deserialize)]
struct Trailhead {
    map: String,
    tileset: String,
    snow_turnout: Turnout,
    guide: String,
    sprite: String,
    stand: [u16; 2],
    facing: String,
    offer: Vec<AuthoredDialoguePage>,
    arrival: [u16; 2],
}

/// Adds the free guided ski trip at the east street and the west-bank warming hut.
/// Both stops stay in their original regional coordinates. The existing river,
/// eastbound stamp and ranger's reward retain their own progression checks.
pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let [town, river]: [Trailhead; 2] = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/northern-trails-v01/ski-crossing.json"
    ))?;
    install_stop(pack, &town, &river)?;
    install_stop(pack, &river, &town)
}

fn install_stop(pack: &mut CompiledGamePack, here: &Trailhead, there: &Trailhead) -> Result<()> {
    let mut room = crate::northstar_rival_arc::map(pack, &here.map)?;
    let target = pack.data.maps.get(&there.map).context("next ski stop")?;
    let constant = target
        .attributes
        .map_constant
        .as_deref()
        .context("destination constant")?;
    ensure!(
        room.attributes.tileset_name == here.tileset,
        "unexpected trailhead art"
    );
    let [x, y] = here.snow_turnout.at;
    let index = usize::from(y) * usize::from(room.attributes.width) + usize::from(x);
    let block = room.blocks.get_mut(index).context("trailhead turnout")?;
    ensure!(
        *block == here.snow_turnout.from,
        "trailhead tree differs at {x},{y}"
    );
    *block = here.snow_turnout.to;
    for at in [here.stand, here.arrival] {
        ensure!(
            kind(pack, &room, at)? == "FLOOR"
                && !room.objects.iter().any(|o| [o.x, o.y] == at)
                && !room.events.warps.iter().any(|w| [w.x, w.y] == at),
            "ski guide and arrival need free snow at {at:?}"
        );
    }

    let programs = passage(here, there, constant)?;
    ensure!(
        programs
            .keys()
            .all(|k| !pack.data.map_scripts.contains_key(k)),
        "ski passage label is in use"
    );
    let targets = BTreeMap::from([(constant.to_owned(), there.map.clone())]);
    let objects = crate::parse_script_object_commands(&room.id, &programs)?;
    room.script_movements.extend(crate::parse_script_movements(
        &room.id, &programs, &objects,
    )?);
    room.script_object_commands.extend(objects);
    room.script_variable_commands
        .extend(crate::parse_script_variable_commands(&room.id, &programs)?);
    room.script_control_commands
        .extend(crate::parse_script_control_commands(&room.id, &programs)?);
    room.script_runtime_commands
        .extend(crate::parse_script_runtime_commands(&room.id, &programs)?);
    room.script_text_commands
        .extend(crate::parse_script_text_commands(&room.id, &programs)?);
    room.script_text_bodies
        .extend(crate::parse_script_text_bodies(&room.id, &programs)?);
    room.script_map_commands
        .extend(crate::parse_script_map_commands(
            &room.id, &programs, &targets,
        )?);
    room.scripts.extend(programs);
    crate::northstar_rival_arc::actor(pack, &mut room, &here.guide, &here.sprite, here.stand)?;
    let guide = room.objects.last_mut().context("new ski guide")?;
    guide.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", here.facing);
    guide.object_type = "OBJECTTYPE_SCRIPT".into();
    guide.radius = 0;
    refresh_event_section(&mut room);
    // Loaded maps may retain the raw event label in addition to its typed
    // section. Keep both copies current for a later split-payload reload.
    let events = room
        .attributes
        .map_events_label
        .as_ref()
        .context("trailhead events")?;
    if let Some(raw) = room.scripts.get_mut(events) {
        *raw = Value::Array(
            room.map_event_section_commands
                .iter()
                .map(|event| json!({"command":event.command, "args":event.args}))
                .collect(),
        );
    }
    crate::authored_town_publication::replace(pack, room)
}

fn passage(here: &Trailhead, there: &Trailhead, target: &str) -> Result<BTreeMap<String, Value>> {
    let guide = here.guide.as_str();
    let offer = format!("{guide}OfferText");
    let decline = format!("{guide}Decline");
    let go_around = format!("{guide}Pass");
    let travel = format!("{guide}Travel");
    let straight = format!("{guide}Out");
    let aside = format!("{guide}Aside");
    let [x, y] = there.arrival;
    // If the player is already standing in the departure square, the guide takes
    // the parallel snow lane. In both cases the follower steps onto the guide's
    // vacated square. Opposite ends face into the actual public town/hut trails.
    Ok(BTreeMap::from([
        (
            guide.into(),
            json!([
                c("faceplayer", &[]),
                c("opentext", &[]),
                c("writetext", &[&offer]),
                c("yesorno", &[]),
                c("iffalse", &[&decline]),
                c("closetext", &[]),
                c("follow", &[guide, "PLAYER"]),
                c("readvar", &["VAR_FACING"]),
                c("ifequal", &[&here.facing, &go_around]),
                c("applymovement", &[guide, &straight]),
                c("sjump", &[&travel])
            ]),
        ),
        (offer, authored_dialogue::compile(&here.offer)?),
        (decline, json!([c("closetext", &[]), c("end", &[])])),
        (
            go_around,
            json!([c("applymovement", &[guide, &aside]), c("sjump", &[&travel])]),
        ),
        (
            straight,
            json!([c("step", &[&there.facing]), c("step_end", &[])]),
        ),
        (
            aside,
            json!([
                c("step", &["DOWN"]),
                c("step", &[&there.facing]),
                c("step_end", &[])
            ]),
        ),
        (
            travel,
            json!([
                c("stopfollow", &[]),
                c(
                    "warpfacing",
                    &[target, &x.to_string(), &y.to_string(), &there.facing]
                ),
                c("end", &[])
            ]),
        ),
    ]))
}
