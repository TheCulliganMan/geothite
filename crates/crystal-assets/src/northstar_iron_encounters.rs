//! Fixed native mine trainers and one-time searches of authored rock landmarks.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::northstar_cave_encounters::add_hidden_item;
use crate::northstar_cave_geometry::resting_tiles;
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

type Tile = [u16; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Discovery {
    map_id: String,
    id: String,
    item: String,
    tile: Tile,
    landmark: String,
    #[serde(default)]
    requires_surf: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sightline {
    map_id: String,
    id: String,
    direction: String,
    radius: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscoverySource {
    schema_version: u8,
    discoveries: Vec<Discovery>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SightlineSource {
    schema_version: u8,
    trainers: Vec<Sightline>,
}

pub(super) struct Encounters {
    discoveries: Vec<Discovery>,
    trainers: Vec<Sightline>,
    roster: RegionalRoster,
}

impl Encounters {
    pub(super) fn parse(discoveries: &str, sightlines: &str) -> Result<Self> {
        let discoveries: DiscoverySource = serde_json::from_str(discoveries)?;
        let sightlines: SightlineSource = serde_json::from_str(sightlines)?;
        ensure!(
            discoveries.schema_version == 1
                && sightlines.schema_version == 1
                && (2..=4).contains(&discoveries.discoveries.len())
                && sightlines.trainers.len() == 12,
            "complete Iron Range discovery and trainer sources required"
        );
        let mut ids = BTreeSet::new();
        let maps: BTreeSet<_> = sightlines
            .trainers
            .iter()
            .map(|trainer| trainer.map_id.as_str())
            .collect();
        ensure!(maps.len() == 6, "expected the six Iron Range mine floors");
        for trainer in &sightlines.trainers {
            ensure!(
                trainer.id.starts_with("MN_LODE_ARC_")
                    && ids.insert(&trainer.id)
                    && matches!(trainer.direction.as_str(), "UP" | "DOWN" | "LEFT" | "RIGHT")
                    && (1..=3).contains(&trainer.radius),
                "invalid mine trainer {}",
                trainer.id
            );
        }
        for discovery in &discoveries.discoveries {
            ensure!(
                maps.contains(discovery.map_id.as_str())
                    && discovery.id.starts_with("MN_IRON_CACHE_")
                    && ids.insert(&discovery.id)
                    && !discovery.landmark.trim().is_empty(),
                "invalid mine discovery {}",
                discovery.id
            );
        }
        Ok(Self {
            discoveries: discoveries.discoveries,
            trainers: sightlines.trainers,
            roster: RegionalRoster::current()?,
        })
    }

    pub(super) fn install(&self, pack: &mut CompiledGamePack, room: &mut MapModule) -> Result<()> {
        let mut reserved = reserved_tiles(room)?;
        let dry = arrival_tiles(pack, room, false)?;
        let trainers: Vec<_> = self
            .trainers
            .iter()
            .filter(|trainer| trainer.map_id == room.id)
            .collect();
        ensure!(
            trainers.len() == 2
                && trainers.len()
                    == room
                        .objects
                        .iter()
                        .filter(|actor| actor.object_type == "OBJECTTYPE_TRAINER")
                        .count(),
            "all ordinary trainers on {} require a sightline",
            room.id
        );
        for trainer in trainers {
            let index = room
                .objects
                .iter()
                .position(|actor| {
                    actor.object_identifier.as_ref() == Some(&trainer.id)
                        && actor.object_type == "OBJECTTYPE_TRAINER"
                        && actor.script == trainer.id
                })
                .with_context(|| format!("missing native mine trainer {}", trainer.id))?;
            let native = room
                .trainer_scripts
                .get(&trainer.id)
                .context("compiled trainer event")?;
            ensure!(
                native.trainer_id == trainer.id
                    && native.event_flag == format!("EVENT_{}_DEFEATED", trainer.id)
                    && pack
                        .data
                        .story_event_script_constants
                        .global
                        .contains_key(&native.event_flag),
                "expected original mine trainer victory receipt for {}",
                trainer.id
            );
            let opponent = pack
                .data
                .trainers
                .get(&native.trainer_id)
                .context("mine trainer party")?;
            ensure!(!opponent.party.is_empty(), "empty mine trainer party");
            let mut family = BTreeSet::new();
            for member in &opponent.party {
                ensure!(
                    self.roster
                        .target(pack, &member.species, Some(member.level))?
                        == member.species,
                    "illegal mine trainer species {}",
                    member.species
                );
                family.insert(member.species.clone());
            }
            // Scaling can traverse more than one generation at higher levels.
            let mut pending = family.iter().cloned().collect::<Vec<_>>();
            while let Some(species) = pending.pop() {
                ensure!(
                    self.roster.allowed.contains(&species)
                        && pack.data.pokemon.contains_key(&species),
                    "mine trainer evolves outside the active roster: {species}"
                );
                for evo in pack.data.evolutions.0.get(&species).into_iter().flatten() {
                    if family.insert(evo.species.clone()) {
                        pending.push(evo.species.clone());
                    }
                }
            }
            let actor = &room.objects[index];
            ensure!(
                actor.move_range_x == 0 && actor.move_range_y == 0 && actor.sprite_has_facings,
                "mine sentry needs native stationary facings"
            );
            let mut at = [actor.x, actor.y];
            for _ in 0..trainer.radius {
                at = step(at, &trainer.direction)?;
                ensure!(
                    kind(pack, room, at)? == "FLOOR" && dry.contains(&at) && reserved.insert(at),
                    "{} cannot see over occupied terrain at {at:?}",
                    trainer.id
                );
            }
            let actor = &mut room.objects[index];
            actor.radius = trainer.radius;
            actor.sightline_direction_override = Some(trainer.direction.clone());
            actor.spritemovedata = format!("SPRITEMOVEDATA_STANDING_{}", trainer.direction);
        }
        let discoveries: Vec<_> = self
            .discoveries
            .iter()
            .filter(|item| item.map_id == room.id)
            .collect();
        for item in &discoveries {
            ensure!(
                kind(pack, room, item.tile)? == "WALL"
                    && reserved.insert(item.tile)
                    && !room.scripts.contains_key(&item.id),
                "mine cache {} must own its rock",
                item.id
            );
            let is_near = |tiles: &BTreeSet<Tile>| {
                tiles
                    .iter()
                    .any(|at| at[0].abs_diff(item.tile[0]) + at[1].abs_diff(item.tile[1]) == 1)
            };
            ensure!(
                is_near(&dry) != item.requires_surf
                    && (!item.requires_surf || is_near(&arrival_tiles(pack, room, true)?)),
                "mine cache {} lost its required approach",
                item.id
            );
            add_hidden_item(pack, room, &item.id, &item.item, item.tile)?;
        }
        if !discoveries.is_empty() {
            room.script_field_pickups =
                crate::parse_script_field_pickups(&room.id, &room.scripts, &room.objects)?;
            for item in discoveries {
                let flag = format!("EVENT_{}", item.id);
                ensure!(
                    room.script_field_pickups
                        .iter()
                        .any(|pickup| pickup.source_script == item.id
                            && pickup.command == "hiddenitem"
                            && pickup.item_id.as_deref() == Some(&item.item)
                            && pickup.quantity == 1
                            && pickup.event_flag.as_deref() == Some(&flag)),
                    "missing native mine Itemfinder receipt for {}",
                    item.id
                );
            }
        }
        Ok(())
    }
}

fn step([x, y]: Tile, direction: &str) -> Result<Tile> {
    match direction {
        "UP" => Ok([x, y.checked_sub(1).context("north mine edge")?]),
        "DOWN" => Ok([x, y.checked_add(1).context("south mine edge")?]),
        "LEFT" => Ok([x.checked_sub(1).context("west mine edge")?, y]),
        "RIGHT" => Ok([x.checked_add(1).context("east mine edge")?, y]),
        _ => anyhow::bail!("invalid mine direction {direction}"),
    }
}

fn arrival_tiles(pack: &CompiledGamePack, room: &MapModule, surf: bool) -> Result<BTreeSet<Tile>> {
    let mut seen = BTreeSet::new();
    for port in room.events.warps.iter().filter(|port| port.index % 2 == 0) {
        // Closed shutters can split a floor into several independently entered chambers.
        seen.extend(resting_tiles(pack, room, [port.x, port.y], surf)?);
    }
    Ok(seen)
}

fn reserved_tiles(room: &MapModule) -> Result<BTreeSet<Tile>> {
    let mut tiles: BTreeSet<_> = room
        .objects
        .iter()
        .map(|actor| [actor.x, actor.y])
        .chain(room.events.warps.iter().map(|port| [port.x, port.y]))
        .chain(room.events.bg_events.iter().map(|event| [event.x, event.y]))
        .chain(
            room.events
                .coord_events
                .iter()
                .map(|event| [event.x, event.y]),
        )
        .collect();
    for change in &room.script_block_changes {
        for [dx, dy] in [[0, 0], [0, 1], [1, 0], [1, 1]] {
            tiles.insert([change.x + dx, change.y + dy]);
        }
    }
    for command in &room.script_object_commands {
        if let (Some(x), Some(y)) = (command.x, command.y) {
            tiles.insert([x, y]);
        }
    }
    for movement in &room.script_movements {
        let mut at = match movement.label.as_str() {
            "MnNDrillPlayerToCrew" => [16, 3],
            "MnNDrillCrewOut" => [15, 9],
            _ => {
                let event = room
                    .events
                    .coord_events
                    .iter()
                    .find(|event| movement.source_script.as_ref() == Some(&event.script_name))
                    .context("unknown mine scene movement origin")?;
                [event.x, event.y]
            }
        };
        tiles.insert(at);
        for op in &movement.steps {
            match op.command.as_str() {
                "step" => {
                    at = step(
                        at,
                        op.direction
                            .as_deref()
                            .context("mine scene step direction")?,
                    )?;
                    tiles.insert(at);
                }
                "step_end" => {}
                _ => anyhow::bail!("unsupported mine scene movement: {}", op.command),
            }
        }
    }
    Ok(tiles)
}
