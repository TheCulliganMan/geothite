//! Authored species progression tables from the pinned external disassembly.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn blocks(source: &str) -> Result<BTreeMap<String, Vec<Vec<String>>>> {
    let mut result = BTreeMap::new();
    let mut label = None;
    for line in source.lines() {
        let line = code_part(line);
        if line.ends_with(':') {
            let name = line.trim_end_matches(':').to_owned();
            result.entry(name.clone()).or_insert_with(Vec::new);
            label = Some(name);
        } else if let Some(arguments) = line.strip_prefix("db ") {
            if let Some(label) = &label {
                result.get_mut(label).unwrap().push(source_arguments(arguments)?);
            }
        }
    }
    Ok(result)
}

fn write(data: &Path, category: &str, values: &BTreeMap<String, Value>) -> Result<()> {
    let directory = data.join("content-packs/core-modular").join(category);
    std::fs::create_dir_all(&directory)?;
    for (id, value) in values {
        let record = modular_record(category, id, value)?;
        std::fs::write(directory.join(format!("{id}.json")), serde_json::to_vec_pretty(&json!({id:record}))?)?;
    }
    std::fs::write(data.join(format!("{category}.json")), serde_json::to_vec_pretty(values)?)?;
    Ok(())
}

fn modular_record(category: &str, species: &str, value: &Value) -> Result<Value> {
    Ok(match category {
        "learnsets" => json!({"species":species,"learnset":value}),
        "evolutions" => json!({"species":species,"evolutions":value}),
        "egg_moves" => json!({"species":species,"moves":value}),
        "level_up_moves" => {
            let learned: Vec<crystal_core::systems::learnsets::LearnsetEntry> = serde_json::from_value(value.clone())?;
            let moves = learned.into_iter().map(|crystal_core::systems::learnsets::LearnsetEntry(level,move_id)|
                json!({"level":level,"move":move_id})).collect::<Vec<_>>();
            json!({"species":species,"moves":moves})
        }
        _ => value.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_assets::{ContentPackCategory, GameDataSet};

    #[test]
    fn progression_catalog_records_pass_production_parsers() {
        let cases = [("learnsets",ContentPackCategory::Learnsets,json!([[1,"TELEPORT"]])),
            ("level_up_moves",ContentPackCategory::LevelUpMoves,json!([[1,"TELEPORT"]])),
            ("egg_moves",ContentPackCategory::EggMoves,json!(["LIGHT_SCREEN"])),
            ("evolutions",ContentPackCategory::Evolutions,json!([]))];
        for (name,category,value) in cases {
            let record = modular_record(name,"ABRA",&value).unwrap();
            GameDataSet::validate_authored_content_payloads(&[(category,json!({"ABRA":record}))]).unwrap();
            assert!(GameDataSet::validate_authored_content_payloads(&[(category,json!({"ABRA":value}))]).is_err(),
                "aggregate schema must not be accepted as a modular {name} record");
            let empty = modular_record(name,"ABRA",&json!([])).unwrap();
            GameDataSet::validate_authored_content_payloads(&[(category,json!({"ABRA":empty}))]).unwrap();
        }
    }

    #[test]
    fn exported_catalog_shapes_match_production_parsers() {
        let Some(workspace) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else { return; };
        let directory = Path::new(&workspace).join("apps/web/assets/data/content-packs/core-modular");
        let mut failures = Vec::new();
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if !entry.path().is_dir() { continue; }
            let name = entry.file_name().into_string().unwrap();
            let Ok(category) = serde_json::from_value::<ContentPackCategory>(json!(name)) else { continue; };
            // Fresh intro certification belongs to the continuous production
            // replay; this read-only schema audit never changes its flags.
            if category == ContentPackCategory::RuntimeTitleScreen { continue; }
            let mut payloads = Vec::new();
            for file in std::fs::read_dir(entry.path()).unwrap() {
                let path = file.unwrap().path();
                if !path.extension().is_some_and(|extension| extension == "json") { continue; }
                let mut payload: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                // Validate the corrected source serialization without exporting
                // or mutating the shared workspace currently owned by root.
                if ["learnsets","level_up_moves","egg_moves","evolutions"].contains(&name.as_str()) {
                    let object = payload.as_object_mut().unwrap();
                    for (id,value) in object { if value.is_array() { *value = modular_record(&name,id,value).unwrap(); } }
                } else if category == ContentPackCategory::Trainers && payload.get("trainers").is_some() {
                    payload = payload["trainers"].clone();
                } else if category == ContentPackCategory::Audio && payload.is_array() {
                    payload = Value::Object(payload.as_array().unwrap().iter().map(|asset|
                        (asset["id"].as_str().unwrap().to_string(),asset.clone())).collect());
                } else if category == ContentPackCategory::Playability {
                    let mut seen = std::collections::BTreeSet::new();
                    payload["initial_events"].as_array_mut().unwrap().retain(|flag|
                        seen.insert(flag.as_str().unwrap().to_string()));
                }
                payloads.push((category,payload));
            }
            if let Err(error) = GameDataSet::validate_authored_content_payloads(&payloads) {
                failures.push(format!("{name}: {error:#}"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let species: BTreeMap<String, Value> = serde_json::from_slice(&std::fs::read(data.join("pokemon.json"))?)?;
    let mut ordered = species.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(_, value)| value["int_id"].as_u64());
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let pointers = |source: &str| -> Vec<String> {
        source.lines().filter_map(|line| code_part(line).strip_prefix("dw ").map(str::to_owned)).collect()
    };
    let evolution_pointers = pointers(&read("data/pokemon/evos_attacks_pointers.asm")?);
    let egg_pointers = pointers(&read("data/pokemon/egg_move_pointers.asm")?);
    ensure!(ordered.len() == 251 && evolution_pointers.len() == 251 && egg_pointers.len() == 251, "source progression pointer count");
    let evolution_blocks = blocks(&read("data/pokemon/evos_attacks.asm")?)?;
    let egg_blocks = blocks(&read("data/pokemon/egg_moves.asm")?)?;
    let mut learnsets = BTreeMap::new();
    let mut evolutions = BTreeMap::new();
    let mut eggs = BTreeMap::new();
    for (index, (id, _)) in ordered.into_iter().enumerate() {
        let rows = evolution_blocks.get(&evolution_pointers[index]).context("missing evolution body")?;
        let mut rules = Vec::new();
        let mut learned = Vec::new();
        let mut attacks = false;
        for row in rows {
            if row == &["0"] { if attacks { break; } attacks = true; continue; }
            if attacks {
                ensure!(row.len() == 2, "invalid learned move {id}: {row:?}");
                learned.push(json!([row[0].parse::<u8>()?, row[1]]));
            } else {
                let method = row[0].strip_prefix("EVOLVE_").context("source evolution method")?;
                let mut rule = json!({"method":method,"species":row.last().context("evolution target")?,"level":null,"item":null,"held_item":null,"happiness":null,"stat_ratio":null});
                match method {
                    "LEVEL" => rule["level"] = json!(row[1].parse::<u8>()?),
                    "ITEM" => rule["item"] = json!(row[1]),
                    "TRADE" => rule["held_item"] = json!(row[1]),
                    "HAPPINESS" => rule["happiness"] = json!(row[1]),
                    "STAT" => { rule["level"] = json!(row[1].parse::<u8>()?); rule["stat_ratio"] = json!(row[2]); }
                    _ => anyhow::bail!("unknown source evolution method {method}"),
                }
                let _: crystal_core::systems::evolution::EvolutionEntry = serde_json::from_value(rule.clone())?;
                rules.push(rule);
            }
        }
        learnsets.insert(id.clone(), json!(learned));
        evolutions.insert(id.clone(), json!(rules));
        let egg_rows = egg_blocks.get(&egg_pointers[index]).with_context(|| format!("missing egg moves {}", egg_pointers[index]))?;
        let mut moves = Vec::new();
        for row in egg_rows { if row == &["-1"] { break; } moves.extend(row.iter().cloned()); }
        eggs.insert(id.clone(), json!(moves));
    }
    let _: crystal_core::systems::learnsets::SpeciesLearnsets = serde_json::from_value(json!(learnsets))?;
    let _: crystal_core::systems::evolution::EvolutionTable = serde_json::from_value(json!(evolutions))?;
    write(&data, "learnsets", &learnsets)?;
    write(&data, "level_up_moves", &learnsets)?;
    write(&data, "evolutions", &evolutions)?;
    write(&data, "egg_moves", &eggs)?;
    let growth_ids = read("constants/pokemon_data_constants.asm")?.lines().filter_map(|line| code_part(line).strip_prefix("const GROWTH_").map(|id| format!("GROWTH_{id}"))).collect::<Vec<_>>();
    let growth_source = read("data/growth_rates.asm")?;
    let growth_rows = growth_source.lines().filter_map(|line| code_part(line).strip_prefix("growth_rate ")).map(source_arguments).collect::<Result<Vec<_>>>()?;
    ensure!(growth_ids.len() == growth_rows.len(), "source growth rate count");
    let mut growth = BTreeMap::new();
    for (id, row) in growth_ids.into_iter().zip(growth_rows) {
        ensure!(row.len() == 5, "source growth coefficients");
        let values = row.iter().map(|value| value.parse::<i32>()).collect::<std::result::Result<Vec<_>, _>>()?;
        let value = json!({"id":id,"numerator":values[0],"denominator":values[1],"quadratic":values[2],"linear":values[3],"constant":values[4]});
        let _: crystal_core::systems::experience::GrowthRateCurve = serde_json::from_value(value.clone())?;
        growth.insert(id, value);
    }
    write(&data, "growth_rates", &growth)
}
