use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn write(data: &Path, category: &str, value: &Value) -> Result<()> {
    let output = data.join("content-packs/core-modular").join(category);
    std::fs::create_dir_all(&output)?;
    std::fs::write(output.join("Source.json"), serde_json::to_vec_pretty(value)?)?;
    std::fs::write(data.join(format!("{category}.json")), serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let pokemon: BTreeMap<String, Value> = serde_json::from_slice(&std::fs::read(data.join("pokemon.json"))?)?;
    let mut ordered = pokemon.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(_, value)| value["int_id"].as_u64());
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let mut cries = BTreeMap::new();
    let mut index = 0;
    for line in std::fs::read_to_string(vendor.join("data/pokemon/cries.asm"))?.lines().map(code_part) {
        if line == "assert_table_length NUM_POKEMON" { break; }
        if let Some(arguments) = line.strip_prefix("mon_cry ") {
            let row = source_arguments(arguments)?;
            ensure!(row.len() == 3 && index < ordered.len(), "source Pokemon cry rows");
            let word = |value: &str| -> Result<i16> {
                let value = super::source_expressions::evaluate(value, &constants)?;
                ensure!((-32768..=65535).contains(&value), "source cry word range");
                Ok(value as u16 as i16)
            };
            let value = json!({"cry":row[0],"pitch":word(&row[1])?,"length":word(&row[2])?});
            let _: crystal_assets::PokemonCryMetadata = serde_json::from_value(value.clone())?;
            cries.insert(ordered[index].0.clone(), value);
            index += 1;
        }
    }
    ensure!(index == 251 && cries.len() == 251, "authored species cry count");
    write(&data, "pokemon_cries", &json!(cries))?;
    let mut specials = BTreeMap::new();
    for line in std::fs::read_to_string(vendor.join("data/events/special_pointers.asm"))?.lines().map(code_part) {
        if let Some(name) = line.strip_prefix("add_special ") {
            ensure!(!name.is_empty() && !name.contains(char::is_whitespace), "authored special pointer identity");
            ensure!(specials.insert(name.to_owned(), json!({})).is_none(), "duplicate authored special pointer {name}");
        }
    }
    ensure!(!specials.is_empty(), "authored special pointer table is empty");
    write(&data, "special_routines", &json!(specials))?;
    let trade_ids = std::fs::read_to_string(vendor.join("constants/npc_trade_constants.asm"))?
        .lines().filter_map(|line| code_part(line).strip_prefix("const NPC_TRADE_").map(|name| format!("NPC_TRADE_{name}"))).collect::<Vec<_>>();
    let trade_rows = std::fs::read_to_string(vendor.join("data/events/npc_trades.asm"))?
        .lines().filter_map(|line| code_part(line).strip_prefix("npctrade ").map(source_arguments)).collect::<Result<Vec<_>>>()?;
    ensure!(trade_ids.len() == trade_rows.len(), "authored NPC trade count");
    let mut trades = BTreeMap::new();
    for (id, row) in trade_ids.into_iter().zip(trade_rows) {
        ensure!(row.len() == 10, "authored NPC trade operands");
        let number = |value: &str| super::source_expressions::evaluate(value, &constants);
        let value = json!({"dialogSet":row[0], "requestedSpecies":row[1], "offeredSpecies":row[2],
            "nickname":row[3].trim_matches('"'), "dvs":[u8::try_from(number(&row[4])?)?,u8::try_from(number(&row[5])?)?],
            "heldItem":row[6], "originalTrainerId":u16::try_from(number(&row[7])?)?,
            "originalTrainerName":row[8].trim_matches('"'), "genderRequirement":row[9]});
        let _: crystal_assets::NpcTradeRule = serde_json::from_value(value.clone())?;
        trades.insert(id, value);
    }
    write(&data, "npc_trades", &json!(trades))?;
    let source = std::fs::read_to_string(vendor.join("engine/events/std_scripts.asm"))?;
    let body = source.split("InitializeEventsScript:").nth(1).context("source initialization script")?;
    let mut event_flags = Vec::new();
    let mut engine_flags = Vec::new();
    let mut variable_sprites = BTreeMap::new();
    for line in body.lines().map(code_part) {
        if line == "end" || line == "endcallback" { break; }
        if line.is_empty() { continue; }
        if let Some(flag) = line.strip_prefix("setevent ") { event_flags.push(flag.to_owned()); }
        else if let Some(flag) = line.strip_prefix("setflag ") { engine_flags.push(flag.to_owned()); }
        else if let Some(arguments) = line.strip_prefix("variablesprite ") {
            let row = source_arguments(arguments)?;
            ensure!(row.len() == 2, "source initial variable sprite");
            variable_sprites.insert(row[0].clone(), row[1].clone());
        } else { anyhow::bail!("unsupported authored initialization command {line}"); }
    }
    let value = json!({"eventFlags":event_flags,"engineFlags":engine_flags,"variableSprites":variable_sprites});
    let _: crystal_core::systems::script_runtime::InitializeEventsConfig = serde_json::from_value(value.clone())?;
    write(&data, "initialize_events", &value)
}
