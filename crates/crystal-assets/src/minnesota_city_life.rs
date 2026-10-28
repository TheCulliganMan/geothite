//! Additive, explicitly placed residents and native once-only city trades.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::json;

use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule, NpcTradeRule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Population {
    schema_version: u8,
    cities: Vec<City>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct City {
    map: String,
    trade: Trade,
    actors: Vec<Actor>,
    placements: Vec<Placement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Trade {
    requested: String,
    offered: String,
    nickname: String,
    trainer: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    id: String,
    sprite: String,
    lines: Vec<String>,
    movement: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Placement {
    dimensions: [u16; 2],
    tiles: Vec<[u16; 2]>,
}

/// Adds local residents and one native Pokémon trader to every installed city
/// in the supplied population contract. The first actor in each city trades;
/// others speak and optionally walk a bounded three-tile horizontal/vertical
/// lane. Explicit placements support different published city compositions.
/// Cities absent from this pack are left for their own publication lineage.
/// Existing scripts, objects, terrain, services, story, species and audio survive.
///
/// # Errors
/// Returns an error for malformed/duplicate authoring, an unsupported installed
/// extent, invalid trade species or art, blocked/overlapping positions, a lane
/// over a warp/event, lost walking access, exhausted byte-sized object IDs, an
/// existing population overlay, or failed native pack verification. Input is
/// borrowed and is unchanged even on failure.
pub fn build_minnesota_city_life_modpack(
    base: &CompiledGamePack,
    population_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let population: Population = serde_json::from_str(population_json)?;
    ensure!(
        population.schema_version == 1,
        "unsupported city population schema"
    );
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|id| id == "minnesota-city-life-v1"),
        "city population is already installed"
    );
    let mut pack = base.clone();
    let mut cities = BTreeSet::new();
    let mut labels = base
        .data
        .map_scripts
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let expected = [
        "MnWinona",
        "MnRochester",
        "MnMankato",
        "MnMinneapolis",
        "MnStPaul",
        "MnStCloud",
        "MnBrainerd",
        "MnLeechLake",
        "MnBemidji",
        "MnGrandRapids",
        "MnDuluth",
        "MnGrandMarais",
        "MnBoundaryWaters",
        "MnEly",
        "MnLakeItasca",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    ensure!(
        population
            .cities
            .iter()
            .map(|city| city.map.as_str())
            .collect::<BTreeSet<_>>()
            == expected,
        "population must cover every implemented Minnesota town, including Ely"
    );
    let mut installed = 0;
    for (index, city) in population.cities.iter().enumerate() {
        ensure!(
            cities.insert(&city.map),
            "duplicate population city {}",
            city.map
        );
        let Some(original) = base.data.maps.get(&city.map) else {
            continue;
        };
        ensure!(
            city.actors.len() >= 8,
            "{} needs an inhabited cast",
            city.map
        );
        let placement = city
            .placements
            .iter()
            .filter(|p| p.dimensions == [original.attributes.width, original.attributes.height])
            .collect::<Vec<_>>();
        ensure!(
            placement.len() == 1,
            "{} needs one explicit placement for its installed extent",
            city.map
        );
        let placement = placement[0];
        ensure!(
            placement.tiles.len() == city.actors.len(),
            "{} incomplete actor positions",
            city.map
        );
        ensure!(
            original.objects.len() + city.actors.len()
                <= crystal_core::world::object_capacity::MAX_MAP_OBJECTS,
            "{} exceeds byte-sized object identities",
            city.map
        );
        ensure!(
            base.data.pokemon.contains_key(&city.trade.requested)
                && base.data.pokemon.contains_key(&city.trade.offered)
                && city.trade.requested != city.trade.offered,
            "{} requires two installed trade species",
            city.map
        );
        ensure!(
            !city.trade.nickname.is_empty()
                && city.trade.nickname.len() <= 10
                && !city.trade.trainer.is_empty()
                && city.trade.trainer.len() <= 7,
            "{} trade name exceeds native storage",
            city.map
        );
        let trade_id = format!("NPC_TRADE_{}_LOCAL", city.map.to_ascii_uppercase());
        ensure!(
            !pack.data.npc_trades.contains_key(&trade_id),
            "duplicate city trade {trade_id}"
        );
        pack.data.npc_trades.insert(
            trade_id.clone(),
            NpcTradeRule {
                dialog_set: "TRADE_DIALOGSET_HAPPY".into(),
                requested_species: city.trade.requested.clone(),
                offered_species: city.trade.offered.clone(),
                nickname: city.trade.nickname.clone(),
                dvs: vec![0x98, 0x88],
                held_item: "BERRY".into(),
                original_trainer_id: 42000 + u16::try_from(index)?,
                original_trainer_name: city.trade.trainer.clone(),
                gender_requirement: "TRADE_GENDER_EITHER".into(),
            },
        );
        let mut map = original.clone();
        let existing_scripts = std::mem::take(&mut map.scripts);
        for (actor_index, (actor, &tile)) in city.actors.iter().zip(&placement.tiles).enumerate() {
            ensure!(
                labels.insert(actor.id.clone()),
                "duplicate resident label {}",
                actor.id
            );
            ensure!(
                !actor.lines.is_empty(),
                "{} has no local dialogue",
                actor.id
            );
            let sprite = actor
                .sprite
                .strip_prefix("SPRITE_")
                .context("native sprite token")?
                .to_ascii_lowercase();
            ensure!(
                base.runtime_files
                    .contains_key(&format!("gfx/sprites/{sprite}.png")),
                "{} needs installed sprite {}",
                actor.id,
                actor.sprite
            );
            authoring::npc(
                base,
                &mut map,
                &actor.id,
                &actor.sprite,
                i16::try_from(tile[0])?,
                i16::try_from(tile[1])?,
            )?;
            let object = map.objects.last_mut().context("added city resident")?;
            object.object_identifier = Some(actor.id.clone());
            match actor.movement.as_str() {
                "stand" => {}
                "walk_x" => {
                    object.spritemovedata = "SPRITEMOVEDATA_WALK_LEFT_RIGHT".into();
                    // Native movement rejects the radius boundary itself. A value
                    // of two gives the authored center plus one tile each side.
                    object.move_range_x = 2;
                }
                "walk_y" => {
                    object.spritemovedata = "SPRITEMOVEDATA_WALK_UP_DOWN".into();
                    object.move_range_y = 2;
                }
                other => anyhow::bail!("unsupported resident movement {other}"),
            }
            let text = format!("{}Text", actor.id);
            ensure!(
                labels.insert(text.clone()),
                "duplicate resident text {text}"
            );
            // Lower only the new programs; existing authored scenes retain their
            // exact command indices and independently compiled metadata.
            authoring::text(
                &mut map,
                &text,
                &actor.lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
            let c = authoring::command;
            let body = if actor_index == 0 {
                json!([
                    c("faceplayer", &[]),
                    c("opentext", &[]),
                    c("writetext", &[&text]),
                    c("waitbutton", &[]),
                    c("trade", &[&trade_id]),
                    c("closetext", &[]),
                    c("end", &[])
                ])
            } else {
                json!([c("jumptextfaceplayer", &[&text])])
            };
            map.scripts.insert(actor.id.clone(), body);
        }
        let scripts = std::mem::replace(&mut map.scripts, existing_scripts);
        validate_placements(base, original, &map, city, placement)?;
        append_programs(&mut map, scripts)?;
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(&mut pack, map)?;
        installed += 1;
    }
    ensure!(
        installed > 0,
        "no installed cities match the population contract"
    );
    pack.report.manifests.push("minnesota-city-life-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

fn append_programs(
    map: &mut MapModule,
    scripts: BTreeMap<String, serde_json::Value>,
) -> Result<()> {
    map.script_text_bodies
        .extend(crate::parse_script_text_bodies(&map.id, &scripts)?);
    map.script_text_commands
        .extend(crate::parse_script_text_commands(&map.id, &scripts)?);
    map.script_control_commands
        .extend(crate::parse_script_control_commands(&map.id, &scripts)?);
    map.script_runtime_commands
        .extend(crate::parse_script_runtime_commands(&map.id, &scripts)?);
    map.script_object_commands
        .extend(crate::parse_script_object_commands(&map.id, &scripts)?);
    map.scripts.extend(scripts);
    Ok(())
}

fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

fn validate_placements(
    base: &CompiledGamePack,
    original: &MapModule,
    map: &MapModule,
    city: &City,
    placement: &Placement,
) -> Result<()> {
    let mut occupied = original
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    let reserved = original
        .events
        .warps
        .iter()
        .map(|w| [w.x, w.y])
        .chain(original.events.coord_events.iter().map(|e| [e.x, e.y]))
        .chain(
            base.data
                .runtime_spawn_points
                .values()
                .filter(|s| s.map_name == map.id)
                .filter_map(|s| {
                    Some([u16::try_from(s.tile_x).ok()?, u16::try_from(s.tile_y).ok()?])
                }),
        )
        .collect::<BTreeSet<_>>();
    for (actor, &[x, y]) in city.actors.iter().zip(&placement.tiles) {
        let lane = match actor.movement.as_str() {
            "walk_x" => vec![
                [x.checked_sub(1).context("walk lane at edge")?, y],
                [x, y],
                [x + 1, y],
            ],
            "walk_y" => vec![
                [x, y.checked_sub(1).context("walk lane at edge")?],
                [x, y],
                [x, y + 1],
            ],
            _ => vec![[x, y]],
        };
        for tile in lane {
            ensure!(
                kind(base, map, tile)? == "FLOOR",
                "{} at {tile:?} is not native floor",
                actor.id
            );
            ensure!(
                occupied.insert(tile),
                "{} overlaps another resident/lane at {tile:?}",
                actor.id
            );
            ensure!(
                !reserved
                    .iter()
                    .any(|p| p[0].abs_diff(tile[0]) + p[1].abs_diff(tile[1]) <= 1),
                "{} at {tile:?} obstructs a warp, scene or arrival",
                actor.id
            );
        }
    }
    // Edge checkpoints can deliberately isolate a tiny arrival component until
    // a story scene opens them. Use a native arrival in the inhabited component,
    // rather than treating the first sorted edge coordinate as the whole city.
    let mut component = None;
    for start in reserved
        .iter()
        .copied()
        .flat_map(|p| std::iter::once(p).chain(neighbors(p)))
        .filter(|p| !occupied.contains(p))
    {
        if kind(base, original, start).ok() != Some("FLOOR") {
            continue;
        }
        let before = reachable(base, original, start, false)?;
        if placement
            .tiles
            .iter()
            .all(|&tile| neighbors(tile).any(|p| before.contains(&p)))
        {
            component = Some((start, before));
            break;
        }
    }
    let (start, before) = component.context(format!(
        "{} residents need a connected native city arrival",
        map.id
    ))?;
    // Treat every walker lane as occupied at once: even its worst possible
    // position must leave the original connected walking area accessible.
    let mut blocked = map.clone();
    for &tile in occupied
        .iter()
        .filter(|p| !map.objects.iter().any(|a| [a.x, a.y] == **p))
    {
        let mut obstacle = map.objects[0].clone();
        obstacle.x = tile[0];
        obstacle.y = tile[1];
        blocked.objects.push(obstacle);
    }
    let after = reachable(base, &blocked, start, false)?;
    for tile in before.difference(&after) {
        ensure!(
            occupied.contains(tile),
            "{} population blocks original walking access to {tile:?}",
            map.id
        );
    }
    for (actor, &tile) in city.actors.iter().zip(&placement.tiles) {
        ensure!(
            neighbors(tile).any(|p| after.contains(&p)),
            "{} is unreachable from the city arrival",
            actor.id
        );
    }
    Ok(())
}
