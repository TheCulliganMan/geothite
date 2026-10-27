//! Replace only owned gym rosters; preserve all unrelated trainer/event numbers.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

pub(super) fn owned(id: &str, slot: u8) -> bool {
    id == format!("MN_GYM_{slot}") || id.starts_with(&format!("MN_GYM_{slot}_TRAINER_"))
}

pub(super) fn import(
    pack: &mut CompiledGamePack,
    original: &CompiledGamePack,
    donor: &CompiledGamePack,
    slots: &[u8],
) -> Result<()> {
    let removed = pack
        .data
        .trainers
        .trainers
        .keys()
        .filter(|id| slots.iter().any(|&s| owned(id, s)))
        .cloned()
        .collect::<BTreeSet<_>>();
    for id in &removed {
        pack.data.trainers.trainers.remove(id);
        pack.data.story_event_script_constants.global.remove(id);
    }
    for (id, trainer) in &donor.data.trainers.trainers {
        if !slots.iter().any(|&s| owned(id, s)) {
            continue;
        }
        ensure!(
            removed.contains(id),
            "reviewed trainer has no owned predecessor: {id}"
        );
        let old = &original.data.trainers.trainers[id];
        let class = &trainer.trainer_class;
        ensure!(
            pack.data.story_event_script_constants.global.get(class)
                == donor.data.story_event_script_constants.global.get(class),
            "native trainer class identity differs: {id}"
        );
        let number = if old.trainer_class == *class {
            *original
                .data
                .story_event_script_constants
                .global
                .get(id)
                .context("owned trainer number")?
        } else {
            let current = pack
                .data
                .trainers
                .trainers
                .values()
                .filter(|t| t.trainer_class == *class)
                .map(|t| {
                    pack.data
                        .story_event_script_constants
                        .global
                        .get(&t.trainer_id)
                        .copied()
                        .context("native trainer number")
                })
                .collect::<Result<Vec<_>>>()?;
            // Reserve original class numbers too, including owned trainers
            // that will be reinserted later in deterministic key order.
            let reserved = original
                .data
                .trainers
                .trainers
                .values()
                .filter(|t| t.trainer_class == *class)
                .filter_map(|t| {
                    original
                        .data
                        .story_event_script_constants
                        .global
                        .get(&t.trainer_id)
                })
                .copied();
            current
                .into_iter()
                .chain(reserved)
                .max()
                .context("native class roster")?
                .checked_add(1)
                .context("reviewed trainer number overflow")?
        };
        u16::try_from(number).context("reviewed trainer number exceeds native range")?;
        ensure!(
            !pack
                .data
                .trainers
                .trainers
                .values()
                .any(|t| t.trainer_class == *class
                    && pack
                        .data
                        .story_event_script_constants
                        .global
                        .get(&t.trainer_id)
                        == Some(&number)),
            "reviewed trainer number is occupied: {id}"
        );
        pack.data
            .story_event_script_constants
            .global
            .insert(id.clone(), number);
        pack.data
            .trainers
            .trainers
            .insert(id.clone(), trainer.clone());
    }
    for name in donor
        .data
        .story_event_script_constants
        .global
        .keys()
        .filter(|name| {
            slots
                .iter()
                .any(|s| name.starts_with(&format!("EVENT_MN_GYM_{s}_")))
        })
    {
        if pack
            .data
            .story_event_script_constants
            .global
            .contains_key(name)
        {
            continue;
        }
        let next = pack
            .data
            .story_event_script_constants
            .global
            .iter()
            .filter(|(key, _)| key.starts_with("EVENT_"))
            .map(|(_, v)| *v)
            .max()
            .context("native event table")?
            .checked_add(1)
            .context("reviewed gym event overflow")?;
        u16::try_from(next).context("reviewed event exceeds native range")?;
        pack.data
            .story_event_script_constants
            .global
            .insert(name.clone(), next);
    }
    Ok(())
}
