//! Native sightline trainers, hidden-item receipts and canonical cave clues.

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use crystal_core::models::TrainerPartyPokemon;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring;
use crate::northstar_cave_design::{Cave, Trainer};
use crate::northstar_regional_roster::RegionalRoster;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    room: &mut MapModule,
    cave: &Cave,
    story: &str,
) -> Result<()> {
    let policy = RegionalRoster::current()?;
    for trainer in &cave.trainers {
        register(pack, trainer, &policy)?;
        let id = &trainer.id;
        let flag = format!("EVENT_{id}");
        crate::authored_city_expansion_events::flags(pack, std::slice::from_ref(&flag))?;
        ensure!(
            !pack.data.map_scripts.contains_key(id),
            "occupied cave trainer script"
        );
        room.scripts.insert(
            id.clone(),
            json!([authoring::command(
                "trainer",
                &[
                    &trainer.trainer_class,
                    id,
                    &flag,
                    &format!("{id}Seen"),
                    &format!("{id}Won"),
                    &format!("{id}Lost"),
                    &format!("{id}After"),
                ]
            )]),
        );
        room.scripts.insert(
            format!("{id}After"),
            json!([
                authoring::command("endifjustbattled", &[]),
                authoring::command("jumptextfaceplayer", &[&format!("{id}AfterText")]),
            ]),
        );
        authoring::npc(
            pack,
            room,
            id,
            &trainer.sprite,
            i16::try_from(trainer.tile[0])?,
            i16::try_from(trainer.tile[1])?,
        )?;
        let actor = room.objects.last_mut().context("appended cave trainer")?;
        actor.object_identifier = Some(id.clone());
        actor.object_type = "OBJECTTYPE_TRAINER".into();
        actor.radius = trainer.radius;
        actor.spritemovedata = format!(
            "SPRITEMOVEDATA_STANDING_{}",
            trainer.direction.to_ascii_uppercase()
        );
        actor.sprite_has_facings = true;
        actor.sightline_direction_override = Some(trainer.direction.to_ascii_uppercase());
    }
    for item in &cave.hidden_items {
        ensure!(
            pack.data.items.contains_key(&item.item),
            "installed cave reward required"
        );
        ensure!(
            !pack.data.map_scripts.contains_key(&item.id)
                && !room
                    .events
                    .bg_events
                    .iter()
                    .any(|event| [event.x, event.y] == item.tile)
                && !room
                    .events
                    .warps
                    .iter()
                    .any(|warp| [warp.x, warp.y] == item.tile),
            "hidden cave reward overlaps retained content"
        );
        let flag = format!("EVENT_{}", item.id);
        crate::authored_city_expansion_events::flags(pack, std::slice::from_ref(&flag))?;
        room.scripts.insert(
            item.id.clone(),
            json!([authoring::command("hiddenitem", &[&item.item, &flag]),]),
        );
        room.events.bg_events.push(BackgroundEvent {
            x: item.tile[0],
            y: item.tile[1],
            event_type: "BGEVENT_ITEM".into(),
            script: item.id.clone(),
        });
    }
    let story: Value = serde_json::from_str(story)?;
    let texts = story["cave_overhaul"]["dialogue"][&room.id]
        .as_object()
        .context("canonical cave dialogue")?;
    for (label, pages) in texts {
        ensure!(
            room.scripts.contains_key(label)
                || cave
                    .trainers
                    .iter()
                    .any(|trainer| ["Seen", "Won", "Lost", "AfterText"]
                        .iter()
                        .any(|suffix| label == &format!("{}{suffix}", trainer.id))),
            "unowned cave dialogue: {label}"
        );
        let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(pages.clone())?;
        room.scripts
            .insert(label.clone(), authored_dialogue::compile(&pages)?);
    }
    for trainer in &cave.trainers {
        for suffix in ["Seen", "Won", "Lost", "AfterText"] {
            ensure!(
                texts.contains_key(&format!("{}{suffix}", trainer.id)),
                "complete cave battle dialogue"
            );
        }
    }
    room.script_field_pickups =
        crate::parse_script_field_pickups(&room.id, &room.scripts, &room.objects)?;
    for item in &cave.hidden_items {
        ensure!(
            room.script_field_pickups
                .iter()
                .any(|pickup| pickup.source_script == item.id
                    && pickup.command == "hiddenitem"
                    && pickup.item_id.as_deref() == Some(&item.item)
                    && pickup.event_flag.as_deref() == Some(format!("EVENT_{}", item.id).as_str())
                    && pickup.quantity == 1),
            "native hidden-item receipt and Itemfinder metadata"
        );
    }
    Ok(())
}

fn register(pack: &mut CompiledGamePack, source: &Trainer, policy: &RegionalRoster) -> Result<()> {
    ensure!(
        !pack.data.trainers.trainers.contains_key(&source.id)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(&source.id),
        "occupied cave trainer identity"
    );
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|trainer| trainer.trainer_class == source.trainer_class)
        .context("native cave trainer class")?
        .clone();
    let dvs = trainer.party.first().context("native trainer DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|trainer| trainer.trainer_class == source.trainer_class)
        .filter_map(|trainer| {
            pack.data
                .story_event_script_constants
                .global
                .get(&trainer.trainer_id)
        })
        .copied()
        .max()
        .context("native trainer numbering")?
        .checked_add(1)
        .context("cave trainer identifier overflow")?;
    trainer.trainer_id.clone_from(&source.id);
    trainer.name.clone_from(&source.name);
    trainer.items.clear();
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    trainer.party = source
        .party
        .iter()
        .map(|member| {
            ensure!(
                (1..=100).contains(&member.level),
                "valid cave trainer level"
            );
            ensure!(
                policy.target(pack, &member.species, Some(member.level))? == member.species,
                "active level-legal Minnesota cave Pokemon"
            );
            let pokemon = pack
                .data
                .create_pokemon(&member.species, member.level, dvs)?;
            let moves = pokemon
                .moves
                .into_iter()
                .filter(|learned| {
                    !matches!(
                        learned.name.as_str(),
                        "SELFDESTRUCT" | "EXPLOSION" | "SONICBOOM" | "DOUBLE_TEAM"
                    )
                })
                .collect::<Vec<_>>();
            ensure!(!moves.is_empty(), "cave trainer needs usable moves");
            Ok(TrainerPartyPokemon {
                species: member.species.clone(),
                level: member.level,
                item: None,
                moves,
                dvs,
            })
        })
        .collect::<Result<_>>()?;
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(source.id.clone(), next);
    Ok(())
}
