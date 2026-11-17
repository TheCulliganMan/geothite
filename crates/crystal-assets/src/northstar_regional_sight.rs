//! Sight battles for the fair, city training rooms, riverbanks and town approaches.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::authored_town_geometry::{kind, reachable};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

type Tile = [u16; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Room {
    entries: Vec<Tile>,
    host_text: Option<String>,
    #[serde(default)]
    water_access: bool,
    trainers: BTreeMap<String, Sightline>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sightline {
    direction: String,
    radius: u16,
}

/// Converts existing ordinary trainer events in place, keeping each original
/// party, victory flag, post-battle text and position.
pub(super) fn install(pack: &mut CompiledGamePack, source: &str, story: &Value) -> Result<()> {
    let rooms: BTreeMap<String, Room> = serde_json::from_str(source)?;
    let roster = RegionalRoster::current()?;
    for (id, design) in rooms {
        ensure!(id.starts_with("Mn"), "regional trainer room {id}");
        let mut room = pack.data.maps.get(&id).context("trainer room")?.clone();
        install_room(pack, &mut room, &design, &roster)
            .with_context(|| format!("regional encounters in {id}"))?;
        if let Some(label) = &design.host_text {
            ensure!(
                room.script_text_bodies.contains_key(label),
                "missing training host text"
            );
            let pages: Vec<AuthoredDialoguePage> =
                serde_json::from_value(story["dialogue_overhaul"]["texts"][&id][label].clone())
                    .context("canonical trainer directions")?;
            room.scripts
                .insert(label.clone(), authored_dialogue::compile(&pages)?);
            room.script_text_bodies = crate::parse_script_text_bodies(&id, &room.scripts)?;
        }
        crate::authored_town_publication::replace(pack, room)?;
    }
    for room in pack
        .data
        .maps
        .values()
        .filter(|room| room.id.starts_with("Mn"))
    {
        for actor in room
            .objects
            .iter()
            .filter(|actor| actor.object_type == "OBJECTTYPE_TRAINER")
        {
            ensure!(
                actor.radius > 0 && room.trainer_scripts.contains_key(&actor.script),
                "ordinary trainer {} in {} has no native sight battle",
                actor.script,
                room.id
            );
        }
    }
    Ok(())
}

fn install_room(
    pack: &CompiledGamePack,
    room: &mut MapModule,
    design: &Room,
    roster: &RegionalRoster,
) -> Result<()> {
    let mut footing = BTreeSet::new();
    for &entry in &design.entries {
        ensure!(
            room.events
                .warps
                .iter()
                .any(|warp| [warp.x, warp.y] == entry)
                || room
                    .attributes
                    .connections
                    .iter()
                    .any(|edge| match edge.direction.as_str() {
                        "west" => entry[0] == 0,
                        "east" => entry[0] + 1 == room.attributes.width * 2,
                        "north" => entry[1] == 0,
                        "south" => entry[1] + 1 == room.attributes.height * 2,
                        _ => false,
                    }),
            "sightline proof starts outside a doorway or connected edge at {entry:?}"
        );
        footing.extend(reachable(pack, room, entry, design.water_access)?);
    }
    let mut reserved: BTreeSet<Tile> = room.objects.iter().map(|o| [o.x, o.y]).collect();
    reserved.extend(design.entries.iter().copied());
    reserved.extend(room.events.warps.iter().map(|e| [e.x, e.y]));
    reserved.extend(room.events.bg_events.iter().map(|e| [e.x, e.y]));
    reserved.extend(room.events.coord_events.iter().map(|e| [e.x, e.y]));

    for (id, line) in &design.trainers {
        let index = room
            .objects
            .iter()
            .position(|o| o.object_identifier.as_ref() == Some(id))
            .with_context(|| format!("missing actor {id}"))?;
        let actor = &room.objects[index];
        let battle = room
            .trainer_scripts
            .get(&actor.script)
            .context("native trainer event")?;
        ensure!(
            actor.object_type == "OBJECTTYPE_TRAINER"
                && actor.radius == 0
                && actor.move_range_x == 0
                && actor.move_range_y == 0
                && actor.sprite_has_facings
                && room.scripts[&actor.script][0]["command"] == "trainer"
                && pack
                    .data
                    .story_event_script_constants
                    .global
                    .contains_key(&battle.event_flag),
            "expected a stationary talk-only trainer with a saved victory: {id}"
        );
        let party = &pack
            .data
            .trainers
            .get(&battle.trainer_id)
            .context("existing trainer party")?
            .party;
        ensure!(
            !party.is_empty()
                && party
                    .iter()
                    .all(|member| roster.allowed.contains(&member.species)),
            "ordinary trainer must use the active Minnesota roster: {id}"
        );
        let (dx, dy): (i16, i16) = match line.direction.as_str() {
            "UP" => (0, -1),
            "RIGHT" => (1, 0),
            "DOWN" => (0, 1),
            "LEFT" => (-1, 0),
            _ => anyhow::bail!("invalid facing for {id}"),
        };
        ensure!(
            (1..=3).contains(&line.radius),
            "invalid sight distance for {id}"
        );
        for distance in 1..=i16::try_from(line.radius)? {
            let target = [
                actor
                    .x
                    .checked_add_signed(distance * dx)
                    .context("sightline x")?,
                actor
                    .y
                    .checked_add_signed(distance * dy)
                    .context("sightline y")?,
            ];
            ensure!(
                footing.contains(&target)
                    && reserved.insert(target)
                    && matches!(kind(pack, room, target)?, "FLOOR" | "TALL_GRASS"),
                "sightline for {id} crosses occupied or inaccessible terrain at {target:?}"
            );
        }
        // Touch only the matching native object row. Unrelated warps, scene
        // operands and object rows retain their original source encoding.
        let row = room
            .map_event_section_commands
            .iter_mut()
            .find(|row| row.command == "object_event" && row.args.get(11) == Some(&actor.script))
            .context("published trainer object row")?;
        ensure!(
            row.args.get(3) == Some(&actor.spritemovedata)
                && row.args.get(10).is_some_and(|v| v == "0"),
            "original trainer wire row differs: {id}"
        );
        let actor = &mut room.objects[index];
        actor.radius = line.radius;
        actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", line.direction);
        actor.sightline_direction_override = Some(line.direction.clone());
        row.args[3].clone_from(&actor.spritemovedata);
        row.args[10] = line.radius.to_string();
    }
    Ok(())
}
