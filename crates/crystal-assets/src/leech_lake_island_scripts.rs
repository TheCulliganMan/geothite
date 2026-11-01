//! Native field scripts, island wildlife and additive executable lowering.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use crystal_core::world::encounters::WildEncounterData;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

use super::{CLAIMED, EDDY, HINT, ITEM, LAKE, TRAINER};

pub(super) fn native_current(pack: &mut CompiledGamePack, source: &str) -> Result<()> {
    crate::minnesota_forced_movement_source::install(pack, source)?;
    let standard = pack
        .data
        .story_events
        .iter_mut()
        .find_map(|catalog| catalog.get_mut("StandardScripts"))
        .context("native standard-script catalog")?;
    let roots = standard["GlobalScriptRoots"]
        .as_array_mut()
        .context("native field-script roots")?;
    for label in ["Script_UsedWhirlpool", "Script_ForcedMovement"] {
        if !roots.iter().any(|root| root == label) {
            roots.push(Value::String(label.into()));
        }
    }
    pack.data.materialize_global_scripts()?;
    Ok(())
}

pub(super) fn discoveries(
    pack: &CompiledGamePack,
    lake: &mut MapModule,
    hint: &[String],
) -> Result<()> {
    ensure!(
        pack.data.items.contains_key("MYSTIC_WATER"),
        "native held item required"
    );
    crate::northstar_lair_authoring::actor(pack, lake, ITEM, "SPRITE_POKE_BALL", [53, 33])?;
    let item = lake.objects.last_mut().context("native island item")?;
    item.event_flag = CLAIMED.into();
    item.object_type = "OBJECTTYPE_ITEMBALL".into();
    lake.scripts
        .insert(ITEM.into(), json!([c("itemball", &["MYSTIC_WATER", "1"])]));
    lake.scripts.insert(
        "MN_LEECH_EDDY_CACHE".into(),
        json!([c("hiddenitem", &["PP_UP", EDDY])]),
    );
    lake.events.bg_events.push(BackgroundEvent {
        x: 61,
        y: 20,
        event_type: "BGEVENT_ITEM".into(),
        script: "MN_LEECH_EDDY_CACHE".into(),
    });
    ensure!(
        hint.iter()
            .all(|line| !line.is_empty() && line.is_ascii() && line.len() <= 18),
        "native fisher hint width"
    );
    crate::minnesota_field_use::hint(
        lake,
        HINT,
        &hint.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let mut bodies = crate::parse_script_text_bodies(LAKE, &lake.scripts)?;
    lake.script_text_bodies
        .insert(HINT.into(), bodies.remove(HINT).context("fisher text")?);
    lake.script_field_pickups =
        crate::parse_script_field_pickups(LAKE, &lake.scripts, &lake.objects)?;
    authoring::refresh_event_section(lake);
    Ok(())
}

pub(super) fn wildlife(
    pack: &CompiledGamePack,
    data: WildEncounterData,
    prior: Option<&WildEncounterData>,
) -> Result<WildEncounterData> {
    ensure!(
        data.map_name == LAKE
            && data
                .grass_rates
                .as_ref()
                .is_some_and(|rates| rates.len() == 3
                    && ["morning", "day", "night"]
                        .iter()
                        .all(|time| rates.get(*time) == Some(&0)))
            && data.water_rate == Some(0)
            && data.grass.is_some()
            && data.water.is_some()
            && data.swarm_overrides.is_empty()
            && data.zones.len() == 1,
        "island-only wildlife; ordinary roads and lake stay quiet"
    );
    let zone = &data.zones[0];
    ensure!(
        [zone.min_x, zone.min_y, zone.max_x, zone.max_y] == [50, 32, 51, 33]
            && zone.grass_rates.len() == 3
            && ["morning", "day", "night"]
                .iter()
                .all(|time| zone.grass_rates.get(*time) == Some(&10)),
        "reviewed island grass extent/rates"
    );
    let roster = crate::northstar_regional_roster::RegionalRoster::parse(
        include_str!("../../../planning/minnesota/story-pokemon.json"),
        include_str!("../../../planning/minnesota/pokemon/roster.json"),
        include_str!("../../../planning/minnesota/pokemon/roster-curation.json"),
        include_str!("../../../planning/minnesota/pokemon/species-profiles.json"),
    )?;
    for (table, count) in [
        (&zone.grass, 7),
        (
            data.grass
                .as_ref()
                .context("zero-rate native grass fallback")?,
            7,
        ),
        (
            data.water
                .as_ref()
                .context("zero-rate native water fallback")?,
            3,
        ),
    ] {
        for slots in [&table.morning, &table.day, &table.night] {
            ensure!(slots.len() == count, "exact native encounter slot extent");
            for slot in slots {
                ensure!(
                    (30..=35).contains(&slot.level)
                        && roster.allowed.contains(&slot.species)
                        && roster.target(pack, &slot.species, Some(slot.level))? == slot.species,
                    "active level-appropriate Minnesota island wildlife required"
                );
            }
        }
    }
    let combined = if let Some(prior) = prior {
        ensure!(
            prior.map_name == LAKE
                && prior.zones.iter().all(|old| old.id != zone.id
                    && (old.max_x < zone.min_x
                        || old.min_x > zone.max_x
                        || old.max_y < zone.min_y
                        || old.min_y > zone.max_y)),
            "existing lake habitat overlaps the island zone"
        );
        let mut combined = prior.clone();
        combined.zones.push(zone.clone());
        combined
    } else {
        data
    };
    for table in combined
        .grass
        .iter()
        .chain(&combined.water)
        .chain(combined.zones.iter().map(|z| &z.grass))
        .chain(combined.swarm_overrides.values().map(|s| &s.grass))
    {
        for slot in table.morning.iter().chain(&table.day).chain(&table.night) {
            ensure!(
                roster.allowed.contains(&slot.species)
                    && roster.target(pack, &slot.species, Some(slot.level))? == slot.species,
                "existing touched lake habitat must retain only active level-appropriate Minnesota wildlife"
            );
        }
    }
    Ok(combined)
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
