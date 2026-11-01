use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn write(data: &Path, category: &str, value: &Value) -> Result<()> {
    let directory = data.join("content-packs/core-modular").join(category);
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), serde_json::to_vec_pretty(value)?)?;
    std::fs::write(data.join(format!("{category}.json")), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
fn grass(rows: &[Vec<String>], number: &impl Fn(&str) -> Result<i64>) -> Result<(Value, Value)> {
    ensure!(rows.len() == 22 && rows[0].len() == 3, "source grass encounter count/rates");
    let times = ["morning", "day", "night"];
    let mut rates = BTreeMap::new();
    let mut tables = BTreeMap::new();
    for (index, time) in times.into_iter().enumerate() {
        rates.insert(time, number(&rows[0][index])?);
        let mut slots = Vec::new();
        for row in &rows[1 + index * 7..1 + (index + 1) * 7] {
            ensure!(row.len() == 2, "source grass species/level operands");
            slots.push(json!({"level":number(&row[0])?,"species":row[1]}));
        }
        tables.insert(time, slots);
    }
    Ok((json!(rates), json!(tables)))
}
pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let number = |value: &str| super::source_expressions::evaluate(&value.replace("percent", "* $ff / 100"), &constants);
    let attributes: BTreeMap<String, crystal_core::map::MapAttributes> = serde_json::from_slice(&std::fs::read(data.join("map_attributes.json"))?)?;
    let map_names = attributes.iter().filter_map(|(name, attr)| attr.map_constant.as_ref().map(|constant| (constant.clone(), name.clone()))).collect::<BTreeMap<_, _>>();
    let mut encounters = BTreeMap::<String, Value>::new();
    for (filename, surface) in [("johto_grass", "grass"), ("kanto_grass", "grass"), ("johto_water", "water"), ("kanto_water", "water")] {
        let mut current = None;
        let mut rows = Vec::new();
        for line in read(&format!("data/wild/{filename}.asm"))?.lines().map(code_part) {
            if let Some(map) = line.strip_prefix(&format!("def_{surface}_wildmons ")) { ensure!(current.is_none(), "nested source encounter entry"); current = Some(map_names.get(map).context("source wild map identity")?.clone()); }
            else if let Some(args) = line.strip_prefix("db ") { if current.is_some() { rows.push(source_arguments(args)?); } }
            else if line == format!("end_{surface}_wildmons") {
                let name = current.take().context("source encounter header")?;
                let value = encounters.entry(name.clone()).or_insert_with(|| json!({"map_name":name,"grass_rates":null,"water_rate":null,"grass":null,"water":null}));
                if surface == "grass" { let (rates, tables) = grass(&rows, &number)?; value["grass_rates"] = rates; value["grass"] = tables; }
                else {
                    ensure!(rows.len() == 4 && rows[0].len() == 1, "source water encounter count/rate");
                    let slots = rows[1..].iter().map(|row| -> Result<Value> { ensure!(row.len() == 2, "source water species operands"); Ok(json!({"level":number(&row[0])?,"species":row[1]})) }).collect::<Result<Vec<_>>>()?;
                    value["water_rate"] = json!(number(&rows[0][0])?);
                    value["water"] = json!({"morning":slots,"day":slots,"night":slots});
                }
                rows.clear();
            }
        }
        ensure!(current.is_none(), "unterminated source wild entry");
    }
    let mut current = None;
    let mut rows = Vec::new();
    for line in read("data/wild/swarm_grass.asm")?.lines().map(code_part).chain(std::iter::once("map_id END")) {
        if let Some(map) = line.strip_prefix("map_id ") {
            if let Some(name) = current.take() {
                let (rates, tables) = grass(&rows, &number)?;
                let species = ["DUNSPARCE", "YANMA"].into_iter().filter(|species| rows.iter().any(|row| row.get(1).is_some_and(|id| id == species))).collect::<Vec<_>>();
                ensure!(species.len() == 1, "authored swarm species identity");
                let species = species[0];
                ensure!(constants.contains_key(&format!("SWARM_{species}")) && constants.contains_key(&format!("ENGINE_{species}_SWARM")), "source swarm constants");
                let value = encounters.get_mut(&name).context("base encounter data for source swarm")?;
                if value.get("swarm_overrides").is_none() { value["swarm_overrides"] = json!({}); }
                value["swarm_overrides"][format!("SWARM_{species}")] = json!({"engine_flag":format!("ENGINE_{species}_SWARM"),"grass_rates":rates,"grass":tables});
                rows.clear();
            }
            if map != "END" { current = Some(map_names.get(map).context("source swarm map")?.clone()); }
        } else if let Some(args) = line.strip_prefix("db ").filter(|args| *args != "-1") { if current.is_some() { rows.push(source_arguments(args)?); } }
    }
    let _: BTreeMap<String, crystal_core::world::encounters::WildEncounterData> = serde_json::from_value(json!(encounters))?;
    write(&data, "wild_encounters", &json!(encounters))?;
    let mut tables = BTreeMap::<String, Vec<Value>>::new();
    let mut surface = None;
    for line in read("data/wild/probabilities.asm")?.lines().map(code_part) {
        if line == "GrassMonProbTable:" { surface = Some("grass"); }
        if line == "WaterMonProbTable:" { surface = Some("water"); }
        if let Some(args) = line.strip_prefix("mon_prob ") {
            let row = source_arguments(args)?; ensure!(row.len() == 2, "source encounter probability operands");
            tables.entry(surface.context("source encounter probability header")?.into()).or_default().push(json!({"threshold":number(&row[0])?,"slot":number(&row[1])?}));
        }
    }
    let value = json!({"tables":tables});
    let _: crystal_core::world::encounters::EncounterSlotTables = serde_json::from_value(value.clone())?;
    write(&data, "encounter_slot_tables", &value)?;
    let engine = read("engine/overworld/wildmons.asm")?;
    ensure!(engine.contains("cp MUSIC_POKEMON_MARCH") && engine.contains("cp MUSIC_RUINS_OF_ALPH_RADIO") && engine.contains("cp MUSIC_POKEMON_LULLABY") && engine.contains("srl b") && engine.contains("sla b"), "source encounter radio rate rules");
    let value = json!({"modifiers":{"MUSIC_POKEMON_MARCH":{"numerator":2,"denominator":1},"MUSIC_RUINS_OF_ALPH_RADIO":{"numerator":2,"denominator":1},"MUSIC_POKEMON_LULLABY":{"numerator":1,"denominator":2}}});
    let _: crystal_core::world::encounters::EncounterMusicModifiers = serde_json::from_value(value.clone())?;
    write(&data, "encounter_music_modifiers", &value)
}
