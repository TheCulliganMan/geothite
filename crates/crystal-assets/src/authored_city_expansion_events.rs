//! Native activities for new city content; existing story tables are not reparsed.

use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerPartyPokemon;
use serde_json::json;

use crate::authored_route_design::{Pickup, RouteTrainer, Sign};
use crate::authored_town_design::Actor;
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(super) fn flags(pack: &mut CompiledGamePack, flags: &[String]) -> Result<()> {
    let constants = &mut pack.data.story_event_script_constants.global;
    let mut next = constants
        .iter()
        .filter(|(key, _)| key.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .context("native event constants")?;
    for flag in flags {
        ensure!(flag.starts_with("EVENT_MN_"), "city flag namespace: {flag}");
        ensure!(!constants.contains_key(flag), "occupied city flag: {flag}");
        next = next.checked_add(1).context("city flag overflow")?;
        ensure!(
            next <= i64::from(u16::MAX),
            "city event flag exceeds native identifier range"
        );
        constants.insert(flag.clone(), next);
    }
    Ok(())
}

pub(super) fn ordinary(
    pack: &mut CompiledGamePack,
    map: &mut MapModule,
    actors: &[Actor],
    signs: &[Sign],
    clinic: &str,
    default_mart: &str,
) -> Result<()> {
    for actor in actors {
        let mart = if actor.service.as_deref() == Some("pokemart") {
            ensure!(!actor.stock.is_empty(), "{} has no stock", actor.id);
            for item in &actor.stock {
                ensure!(
                    pack.data.items.contains_key(item),
                    "unknown shop item {item}"
                );
            }
            let id = format!("MART_{}", actor.id);
            ensure!(
                pack.data
                    .marts
                    .0
                    .insert(id.clone(), actor.stock.clone())
                    .is_none(),
                "occupied city mart {id}"
            );
            id
        } else {
            ensure!(
                actor.service.is_none() || actor.service.as_deref() == Some("heal_party"),
                "unsupported city service {}",
                actor.id
            );
            default_mart.into()
        };
        crate::authored_town_scripts::ordinary(
            pack,
            map,
            std::slice::from_ref(actor),
            &[],
            clinic,
            &mart,
        )?;
    }
    crate::authored_town_scripts::ordinary(pack, map, &[], signs, clinic, default_mart)
}

pub(super) fn activities(
    pack: &mut CompiledGamePack,
    map: &mut MapModule,
    trainers: &[RouteTrainer],
    items: &[Pickup],
) -> Result<()> {
    let c = authoring::command;
    for trainer in trainers {
        ensure!(
            !pack.data.trainers.trainers.contains_key(&trainer.id),
            "occupied trainer {}",
            trainer.id
        );
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(&trainer.defeated_flag),
            "undefined trainer flag"
        );
        let mut opponent = pack
            .data
            .trainers
            .trainers
            .values()
            .find(|t| t.trainer_class == trainer.trainer_class)
            .with_context(|| format!("unknown trainer class {}", trainer.trainer_class))?
            .clone();
        let dvs = opponent.party.first().context("source trainer DVs")?.dvs;
        opponent.trainer_id = trainer.id.clone();
        opponent.name = trainer.display_name.to_ascii_uppercase();
        ensure!(
            !trainer.party.is_empty() && trainer.party.len() <= 6,
            "city party size"
        );
        opponent.party = trainer
            .party
            .iter()
            .map(|member| {
                ensure!((1..=100).contains(&member.level), "city trainer level");
                let pokemon = pack
                    .data
                    .create_pokemon(&member.species, member.level, dvs)?;
                Ok(TrainerPartyPokemon {
                    species: member.species.clone(),
                    level: member.level,
                    item: None,
                    moves: pokemon.moves,
                    dvs,
                })
            })
            .collect::<Result<_>>()?;
        opponent.win_quote.clear();
        opponent.lose_quote.clear();
        pack.data.trainers.insert(opponent)?;
        let [seen, won, lost, after] =
            ["Seen", "Won", "Lost", "After"].map(|suffix| format!("{}{suffix}", trainer.id));
        for (label, lines) in [
            (&seen, &trainer.before_lines),
            (&won, &trainer.after_lines),
            (&lost, &trainer.loss_lines),
        ] {
            authoring::text(
                map,
                label,
                &lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
        map.scripts.insert(
            trainer.id.clone(),
            json!([c(
                "trainer",
                &[
                    &trainer.trainer_class,
                    &trainer.id,
                    &trainer.defeated_flag,
                    &seen,
                    &won,
                    &lost,
                    &after
                ]
            )]),
        );
        map.scripts.insert(
            after,
            json!([c("endifjustbattled", &[]), c("jumptextfaceplayer", &[&won])]),
        );
        authoring::npc(
            pack,
            map,
            &trainer.id,
            &trainer.sprite,
            i16::try_from(trainer.tile[0])?,
            i16::try_from(trainer.tile[1])?,
        )?;
        let object = map.objects.last_mut().context("new city trainer")?;
        object.object_identifier = Some(trainer.id.clone());
        object.spritemovedata = trainer.facing.movement().into();
        object.object_type = "OBJECTTYPE_TRAINER".into();
        object.radius = trainer.sight_range.into();
    }
    for item in items {
        ensure!(
            pack.data.items.contains_key(&item.item) && item.quantity > 0,
            "invalid city pickup {}",
            item.id
        );
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(&item.flag),
            "undefined pickup flag"
        );
        map.scripts.insert(
            item.id.clone(),
            json!([c("itemball", &[&item.item, &item.quantity.to_string()])]),
        );
        authoring::npc(
            pack,
            map,
            &item.id,
            "SPRITE_POKE_BALL",
            i16::try_from(item.tile[0])?,
            i16::try_from(item.tile[1])?,
        )?;
        let object = map.objects.last_mut().context("new city pickup")?;
        object.object_identifier = Some(item.id.clone());
        object.object_type = "OBJECTTYPE_ITEMBALL".into();
        object.event_flag = item.flag.clone();
    }
    Ok(())
}

pub(super) fn finish(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    names: &BTreeMap<String, String>,
) -> Result<()> {
    for (label, commands) in &map.scripts {
        for command in commands
            .as_array()
            .context("new city program command array")?
        {
            if matches!(
                command["command"].as_str(),
                Some("text" | "line" | "cont" | "para")
            ) {
                let text = command["args"][0]
                    .as_str()
                    .context("new city printed line")?
                    .trim_matches('"');
                ensure!(
                    text.chars().count() <= 18,
                    "new city text {label} exceeds the native 18-column box: {text}"
                );
            }
        }
    }
    authoring::finish(pack, map)?;
    let name = &map.id;
    let scripts = &map.scripts;
    map.script_runtime_commands = crate::parse_script_runtime_commands(name, scripts)?;
    map.script_object_commands = crate::parse_script_object_commands(name, scripts)?;
    map.script_variable_commands = crate::parse_script_variable_commands(name, scripts)?;
    map.script_movements =
        crate::parse_script_movements(name, scripts, &map.script_object_commands)?;
    map.script_menu_definitions = crate::parse_script_menu_definitions(name, scripts)?;
    map.script_vertical_menus =
        crate::parse_script_vertical_menus(name, scripts, &map.script_menu_definitions)?;
    map.script_map_commands = crate::parse_script_map_commands(name, scripts, names)?;
    map.trainer_scripts = crate::parse_trainer_scripts(name, scripts)?;
    map.script_field_pickups = crate::parse_script_field_pickups(name, scripts, &map.objects)?;
    Ok(())
}
