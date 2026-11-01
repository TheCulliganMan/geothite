//! Additive mainland items, dialogue, wildlife and native trainer lowering.
use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::WildEncounterZone;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

use super::{CLAIMED, ITEM, LAKE, TRAINER};

pub(super) fn discoveries(
    pack: &mut CompiledGamePack,
    lake: &mut MapModule,
    hint: &[String],
) -> Result<()> {
    crate::northstar_lair_authoring::flag(pack, CLAIMED)?;
    crate::northstar_lair_authoring::actor(pack, lake, ITEM, "SPRITE_POKE_BALL", [19, 26])?;
    let item = lake.objects.last_mut().context("mainland item ball")?;
    item.event_flag = CLAIMED.into();
    item.object_type = "OBJECTTYPE_ITEMBALL".into();
    lake.scripts
        .insert(ITEM.into(), json!([c("itemball", &["SUPER_REPEL", "1"])]));
    ensure!(
        hint.len() == 3
            && hint
                .iter()
                .all(|s| !s.is_empty() && s.is_ascii() && s.len() <= 18),
        "short native walker directions required"
    );
    let label = "MN_LEECH_SHORE_WALKERText";
    crate::minnesota_field_use::hint(
        lake,
        label,
        &hint.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let mut bodies = crate::parse_script_text_bodies(LAKE, &lake.scripts)?;
    lake.script_text_bodies
        .insert(label.into(), bodies.remove(label).context("walker text")?);
    lake.script_field_pickups =
        crate::parse_script_field_pickups(LAKE, &lake.scripts, &lake.objects)?;
    authoring::refresh_event_section(lake);
    Ok(())
}

pub(super) fn wildlife(pack: &mut CompiledGamePack, zone: WildEncounterZone) -> Result<()> {
    ensure!(
        zone.id == "MN_LEECH_CABIN_PINES"
            && [zone.min_x, zone.min_y, zone.max_x, zone.max_y] == [14, 20, 17, 27]
            && zone.grass_rates.len() == 3
            && ["morning", "day", "night"]
                .iter()
                .all(|t| zone.grass_rates.get(*t) == Some(&10)),
        "reviewed mainland trail habitat required"
    );
    let roster = crate::northstar_regional_roster::RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )?;
    for slots in [&zone.grass.morning, &zone.grass.day, &zone.grass.night] {
        ensure!(slots.len() == 7, "native seven-slot trail habitat required");
        for slot in slots {
            ensure!(
                (30..=35).contains(&slot.level)
                    && roster.allowed.contains(&slot.species)
                    && roster.target(pack, &slot.species, Some(slot.level))? == slot.species,
                "active level-appropriate Minnesota trail wildlife required"
            );
        }
    }
    let lake = pack
        .data
        .wild_encounters
        .get(LAKE)
        .context("inherited island/lake wildlife")?;
    ensure!(
        lake.zones.iter().all(|old| old.id != zone.id
            && (old.max_x < zone.min_x
                || old.min_x > zone.max_x
                || old.max_y < zone.min_y
                || old.min_y > zone.max_y)),
        "mainland habitat must not overlap inherited wildlife zones"
    );
    for table in lake
        .grass
        .iter()
        .chain(&lake.water)
        .chain(lake.zones.iter().map(|z| &z.grass))
        .chain(lake.swarm_overrides.values().map(|s| &s.grass))
    {
        for slot in table.morning.iter().chain(&table.day).chain(&table.night) {
            ensure!(
                roster.allowed.contains(&slot.species)
                    && roster.target(pack, &slot.species, Some(slot.level))? == slot.species,
                "inherited touched lake habitat must remain regional and level appropriate"
            );
        }
    }
    let lake = pack
        .data
        .wild_encounters
        .get_mut(LAKE)
        .context("validated lake habitat")?;
    lake.zones.push(zone);
    Ok(())
}

pub(super) fn preserve_programs(old: &MapModule, authored: &MapModule) -> Result<MapModule> {
    let original = serde_json::to_value(old)?;
    let mut revised = serde_json::to_value(authored)?;
    for (key, before) in original.as_object().context("native map fields")? {
        if !(key.starts_with("script_")
            || key == "gift_pokemon_scripts"
            || key == "trainer_scripts"
            || key == "scripted_trainer_battles")
        {
            continue;
        }
        let Some(rows) = revised[key].as_array() else {
            continue;
        };
        let mut kept = before.as_array().context("native executable rows")?.clone();
        for row in rows {
            if row.as_object().is_some_and(|fields| {
                fields
                    .values()
                    .any(|value| value.as_str().is_some_and(|s| s.starts_with(TRAINER)))
            }) {
                kept.push(row.clone());
            }
        }
        revised[key] = Value::Array(kept);
    }
    Ok(serde_json::from_value(revised)?)
}
