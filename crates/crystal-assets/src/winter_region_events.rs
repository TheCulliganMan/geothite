//! Lower only newly authored winter trainers, pickups and shelter services.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::authored_town_design::Actor;
use crate::winter_region_design::{HUT, HUT_CONSTANT, OUTDOOR, OUTDOOR_CONSTANT, Region};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    outdoor: &mut MapModule,
    hut: &mut MapModule,
    region: &Region,
) -> Result<Vec<String>> {
    let flags = region
        .trainers
        .iter()
        .map(|t| t.defeated_flag.clone())
        .chain(region.items.iter().map(|p| p.flag.clone()))
        .collect::<Vec<_>>();
    crate::authored_city_expansion_events::flags(pack, &flags)?;
    let mut constants = flags;
    for trainer in &region.trainers {
        let global = &pack.data.story_event_script_constants.global;
        let next = pack
            .data
            .trainers
            .trainers
            .values()
            .filter(|t| t.trainer_class == trainer.trainer_class)
            .filter_map(|t| global.get(&t.trainer_id))
            .copied()
            .max()
            .context("existing native winter trainer class namespace")?
            .checked_add(1)
            .context("winter trainer constant overflow")?;
        ensure!(
            next <= 255 && !global.contains_key(&trainer.id),
            "occupied or overflowing winter trainer identity"
        );
        pack.data
            .story_event_script_constants
            .global
            .insert(trainer.id.clone(), next);
        constants.push(trainer.id.clone());
    }
    crate::authored_city_expansion_events::activities(
        pack,
        outdoor,
        &region.trainers,
        &region.items,
    )?;
    let actors = region
        .residents
        .iter()
        .map(|r| {
            ensure!(
                r.service.is_none(),
                "outdoor winter resident is dialogue-only"
            );
            Ok(Actor {
                id: r.id.clone(),
                sprite: r.sprite.clone(),
                tile: r.tile,
                facing: r.facing,
                lines: r.lines.clone(),
                service: None,
                repeat_lines: Vec::new(),
                stock: Vec::new(),
                approach: None,
                counter_tile: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    crate::authored_town_scripts::ordinary(pack, outdoor, &actors, &[], "", "")?;
    // The existing route helper implements seven-command native HealParty rest,
    // without adding blackout recovery, SQ03 flags or a fictional border desk.
    crate::authored_route_events::room_events(pack, hut, &region.interiors[0])?;
    let mut names = pack
        .data
        .map_attributes
        .iter()
        .filter_map(|(name, a)| {
            a.map_constant
                .as_ref()
                .map(|constant| (constant.clone(), name.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    names.insert(OUTDOOR_CONSTANT.into(), OUTDOOR.into());
    names.insert(HUT_CONSTANT.into(), HUT.into());
    for module in [outdoor, hut] {
        crate::authored_city_expansion_events::finish(pack, module, &names)?;
    }
    Ok(constants)
}
