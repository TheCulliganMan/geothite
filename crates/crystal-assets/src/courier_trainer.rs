//! Additive courier trainer identity and two native story event constants.
use anyhow::{Context, Result, ensure};
use crystal_core::models::TrainerPartyPokemon;
use serde_json::{Value, json};

use crate::CompiledGamePack;
use crate::courier_story::{READ, TRAINER, WIN};

pub(crate) fn install(pack: &mut CompiledGamePack, story: &Value) -> Result<()> {
    let bosses = story["bosses"].as_array().context("story bosses")?;
    let bosses = bosses
        .iter()
        .filter(|b| b["id"] == TRAINER)
        .collect::<Vec<_>>();
    ensure!(
        bosses.len() == 1
            && bosses[0]["win_flag"] == WIN
            && bosses[0]["party"]
                == json!([{"species":"MAGNEMITE","level":23},{"species":"DROWZEE","level":22}]),
        "canonical Magnemite23/Drowzee22 courier required"
    );
    ensure!(
        !pack.data.trainers.trainers.contains_key(TRAINER)
            && !pack
                .data
                .story_event_script_constants
                .global
                .contains_key(TRAINER),
        "courier trainer identity already occupied"
    );
    for name in [WIN, READ] {
        let globals = &mut pack.data.story_event_script_constants.global;
        ensure!(
            !globals.contains_key(name),
            "courier event already occupied"
        );
        let next = globals
            .iter()
            .filter(|(k, _)| k.starts_with("EVENT_"))
            .map(|(_, n)| *n)
            .max()
            .context("native event catalog")?
            .checked_add(1)
            .context("event overflow")?;
        u16::try_from(next).context("event outside native range")?;
        globals.insert(name.into(), next);
    }
    let mut trainer = pack
        .data
        .trainers
        .trainers
        .values()
        .find(|t| t.trainer_class == "GRUNTM")
        .context("native grunt class")?
        .clone();
    let dvs = trainer.party.first().context("native grunt DVs")?.dvs;
    let next = pack
        .data
        .trainers
        .trainers
        .values()
        .filter(|t| t.trainer_class == "GRUNTM")
        .map(|t| {
            pack.data
                .story_event_script_constants
                .global
                .get(&t.trainer_id)
                .copied()
                .context("native grunt number")
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .context("grunt catalog")?
        .checked_add(1)
        .context("trainer number overflow")?;
    u8::try_from(next).context("trainer number outside native range")?;
    trainer.trainer_id = TRAINER.into();
    trainer.name = "LODE COURIER".into();
    trainer.party = [("MAGNEMITE", 23), ("DROWZEE", 22)]
        .into_iter()
        .map(|(species, level)| {
            let p = pack.data.create_pokemon(species, level, dvs)?;
            Ok(TrainerPartyPokemon {
                species: species.into(),
                level,
                item: None,
                moves: p.moves,
                dvs,
            })
        })
        .collect::<Result<_>>()?;
    trainer.win_quote.clear();
    trainer.lose_quote.clear();
    pack.data.trainers.insert(trainer)?;
    pack.data
        .story_event_script_constants
        .global
        .insert(TRAINER.into(), next);
    Ok(())
}
