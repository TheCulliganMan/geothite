//! Explicit habitat corrections, including local zones inherited from old tables.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Review {
    reason: String,
    relocations: Vec<Relocation>,
    restrictions: Vec<Restriction>,
    field_relocations: Vec<FieldRelocation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldRelocation {
    map: String,
    before: String,
    after: String,
    slots: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relocation {
    map: String,
    zone: Option<String>,
    before: String,
    after: String,
    slots: usize,
    reason: String,
    sample: Option<[u16; 2]>,
    #[serde(rename = "loop")]
    next: Option<[u16; 2]>,
    time: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Restriction {
    species: BTreeSet<String>,
    maps: BTreeSet<String>,
    reason: String,
}

pub(super) fn install(pack: &mut CompiledGamePack, review: &Review) -> Result<()> {
    ensure!(
        !review.reason.is_empty(),
        "habitat review rationale required"
    );
    let policy = crate::authored_regional_approach::policy()?;
    let mut owned = BTreeSet::new();
    for edit in &review.relocations {
        ensure!(
            !edit.reason.is_empty()
                && edit.slots > 0
                && edit.before != edit.after
                && owned.insert((&edit.map, &edit.zone, &edit.before)),
            "unique explicit habitat relocation required"
        );
        let mut data = pack
            .data
            .wild_encounters
            .get(&edit.map)
            .context("habitat relocation map")?
            .clone();
        let map = pack
            .data
            .maps
            .get(&edit.map)
            .context("habitat map geometry")?;
        let surface = |at: [u16; 2]| -> Result<bool> {
            let terrain = crate::authored_town_geometry::kind(pack, map, at)?;
            Ok((terrain == "TALL_GRASS"
                || (terrain == "FLOOR"
                    && matches!(
                        map.attributes.environment.as_deref(),
                        Some("CAVE" | "DUNGEON")
                    )))
                && data
                    .zone_at(at[0].try_into()?, at[1].try_into()?)
                    .map(|z| &z.id)
                    == edit.zone.as_ref())
        };
        if let (Some(sample), Some(next)) = (edit.sample, edit.next) {
            ensure!(
                sample[0].abs_diff(next[0]) + sample[1].abs_diff(next[1]) == 1,
                "adjacent native habitat replay steps required"
            );
            for at in [sample, next] {
                ensure!(
                    surface(at)? && !map.objects.iter().any(|o| [o.x, o.y] == at),
                    "habitat replay must be in its actual native encounter surface"
                );
            }
        } else {
            ensure!(
                edit.sample.is_none() && edit.next.is_none(),
                "complete replay pair required"
            );
            for y in 0..map.attributes.height * 2 {
                for x in 0..map.attributes.width * 2 {
                    ensure!(
                        !surface([x, y])?,
                        "active habitat needs a native capture replay"
                    );
                }
            }
        }
        let table = if let Some(id) = &edit.zone {
            &mut data
                .zones
                .iter_mut()
                .find(|z| &z.id == id)
                .context("habitat relocation zone")?
                .grass
        } else {
            data.grass.as_mut().context("habitat relocation grass")?
        };
        let witnessed = match edit.time.as_str() {
            "morning" => &table.morning,
            "day" => &table.day,
            "night" => &table.night,
            _ => anyhow::bail!("native habitat replay time required"),
        };
        ensure!(
            witnessed.iter().any(|s| s.species == edit.before),
            "habitat replay preimage"
        );
        let mut count = 0;
        for slot in table
            .morning
            .iter_mut()
            .chain(&mut table.day)
            .chain(&mut table.night)
        {
            if slot.species == edit.before {
                crate::minnesota_biome_ecology::validate_species(
                    pack,
                    &policy,
                    &edit.after,
                    slot.level,
                )?;
                slot.species.clone_from(&edit.after);
                count += 1;
            }
        }
        ensure!(
            count == edit.slots,
            "stale habitat preimage: {} {:?}",
            edit.map,
            edit.zone
        );
        pack.data.wild_encounters.insert(edit.map.clone(), data);
    }
    for edit in &review.field_relocations {
        let mut field = pack
            .data
            .field_encounters
            .get(&edit.map)
            .context("field habitat")?
            .clone();
        let table = field
            .tables
            .get_mut("headbutt")
            .context("native tree habitat")?;
        let mut count = 0;
        for slot in table.common.iter_mut().chain(&mut table.rare) {
            if slot.species == edit.before {
                crate::minnesota_biome_ecology::validate_species(
                    pack,
                    &policy,
                    &edit.after,
                    slot.level,
                )?;
                slot.species.clone_from(&edit.after);
                count += 1;
            }
        }
        ensure!(count == edit.slots && count > 0, "field habitat preimage");
        pack.data.field_encounters.insert(edit.map.clone(), field);
    }
    validate(pack, &review.restrictions)
}

fn validate(pack: &CompiledGamePack, restrictions: &[Restriction]) -> Result<()> {
    for name in [
        "CURDLET",
        "SQUEAKURD",
        "DOUGHLIN",
        "CRUMCONE",
        "PAILBARON",
        "TILTAWISP",
    ] {
        ensure!(
            restrictions.iter().any(|r| r.species.contains(name)
                && r.maps == BTreeSet::from(["MnStateFairgrounds".into()])),
            "State Fair exclusivity is required for {name}"
        );
    }
    let mut species = BTreeSet::new();
    for rule in restrictions {
        ensure!(
            !rule.reason.is_empty() && !rule.maps.is_empty() && !rule.species.is_empty(),
            "complete habitat restriction required"
        );
        for name in &rule.species {
            ensure!(species.insert(name), "duplicate habitat restriction");
            ensure!(
                pack.data.pokemon.contains_key(name),
                "unknown habitat species"
            );
        }
        ensure!(
            rule.maps
                .iter()
                .all(|m| m.starts_with("Mn") && pack.data.maps.contains_key(m)),
            "installed regional habitat maps required"
        );
    }
    for (name, map) in &pack.data.maps {
        if !name.starts_with("Mn") {
            continue;
        }
        let mut sources = vec![
            serde_json::to_value(pack.data.wild_encounters.get(name))?,
            serde_json::to_value(pack.data.field_encounters.get(name))?,
            serde_json::to_value(&map.gift_pokemon_scripts)?,
            serde_json::to_value(&map.scripted_wild_battles)?,
        ];
        for program in map.scripts.values().filter_map(Value::as_array) {
            for row in program {
                if matches!(
                    row["command"].as_str(),
                    Some("givepoke" | "giveegg" | "loadwildmon")
                ) {
                    sources.push(row["args"].clone());
                }
            }
        }
        let mut groups = map
            .attributes
            .fishing_group
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        loop {
            let more = pack
                .data
                .fishing
                .swarm_rules
                .values()
                .filter(|r| groups.contains(&r.base_group))
                .map(|r| r.swarm_group.clone())
                .collect::<Vec<_>>();
            let count = groups.len();
            groups.extend(more);
            if count == groups.len() {
                break;
            }
        }
        for id in groups {
            if let Some(group) = pack.data.fishing.groups.get(&id) {
                sources.push(serde_json::to_value(group)?);
                for slot in group.rod_tables.values().flat_map(|t| &t.slots) {
                    if let Some(time) = slot
                        .time_group
                        .as_ref()
                        .and_then(|t| pack.data.fishing.time_groups.get(t))
                    {
                        sources.push(serde_json::to_value(time)?);
                    }
                }
            }
        }
        for source in sources {
            check_source(name, &source, restrictions)?;
        }
    }
    Ok(())
}

fn check_source(map: &str, value: &Value, restrictions: &[Restriction]) -> Result<()> {
    match value {
        Value::String(s) => {
            for rule in restrictions {
                ensure!(
                    !rule.species.contains(s) || rule.maps.contains(map),
                    "misplaced habitat species {s} in {map}: {}",
                    rule.reason
                );
            }
        }
        Value::Array(rows) => {
            for row in rows {
                check_source(map, row, restrictions)?;
            }
        }
        Value::Object(rows) => {
            for row in rows.values() {
                check_source(map, row, restrictions)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nested_zone_swarm_and_time_sources_obey_fair_exclusivity() {
        let rules = vec![Restriction {
            species: BTreeSet::from(["CURDLET".into(), "PAILBARON".into()]),
            maps: BTreeSet::from(["MnStateFairgrounds".into()]),
            reason: "fair food lives at the State Fair".into(),
        }];
        for source in [
            json!({"zones":[{"grass":{"night":[{"species":"CURDLET"}]}}]}),
            json!({"swarm":{"species":"PAILBARON"}}),
            json!({"time_group":{"night_species":"CURDLET"}}),
            json!(["PAILBARON", "25"]),
        ] {
            assert!(check_source("MnR02C00R00", &source, &rules).is_err());
            check_source("MnStateFairgrounds", &source, &rules).expect("fair habitat");
        }
    }
}
