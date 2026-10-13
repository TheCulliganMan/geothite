use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let numeric = |name: &str| -> Result<u64> {
        let value = constants["global"][name].as_i64().with_context(|| format!("source constant {name}"))?;
        ensure!((-1..=255).contains(&value), "source roaming byte constant {name}");
        Ok(u64::from(value as u8))
    };
    let location = |name: &str| -> Result<Value> { Ok(json!({"mapGroup":numeric(&format!("GROUP_{name}"))?,"mapNumber":numeric(&format!("MAP_{name}"))?})) };
    let source = std::fs::read_to_string(vendor.join("engine/overworld/wildmons.asm"))?;
    let init = source.split("InitRoamMons:").nth(1).context("source roam initializer")?.split("CheckEncounterRoamMon:").next().unwrap();
    let mut accumulator = String::new();
    let mut writes = BTreeMap::<u8, BTreeMap<String, Value>>::new();
    for line in init.lines().map(code_part) {
        if let Some(value) = line.strip_prefix("ld a, ") { accumulator = value.to_owned(); }
        if line == "xor a" { accumulator = "0".into(); }
        if let Some(target) = line.strip_prefix("ld [wRoamMon").and_then(|value| value.strip_suffix("], a")) {
            let slot: u8 = target[..1].parse()?;
            let field = &target[1..];
            let key = match field { "Species" => "species", "Level" => "level", "MapGroup" => "mapGroup", "MapNumber" => "mapNumber", "HP" => "hp", _ => anyhow::bail!("unknown roaming initializer field {field}") };
            let value = if field == "Species" { json!(accumulator) } else {
                let number = match accumulator.parse::<u64>() { Ok(number) => number, Err(_) => numeric(&accumulator)? };
                json!(number)
            };
            writes.entry(slot).or_default().insert(key.into(), value);
        }
    }
    let mut init_writes = Vec::new();
    for (slot, mut fields) in writes { fields.insert("slot".into(), json!(slot - 1)); init_writes.push(json!(fields)); }
    let mut routes = Vec::new();
    for line in std::fs::read_to_string(vendor.join("data/wild/roammon_maps.asm"))?.lines().map(code_part) {
        if let Some(args) = line.strip_prefix("roam_map ") {
            let args = source_arguments(args)?;
            let mut value = location(&args[0])?;
            value["connections"] = json!(args[1..].iter().map(|name| location(name)).collect::<Result<Vec<_>>>()?);
            routes.push(value);
        }
    }
    ensure!(source.contains("maskbits NUM_ROAMMON_MAPS"), "source roaming jump selection");
    let count = numeric("NUM_ROAMMON_MAPS")?;
    ensure!(routes.len() as u64 == count && count.is_power_of_two(), "source roaming route count");
    let ram = std::fs::read_to_string(vendor.join("ram/wram.asm"))?;
    let slot_count = ram.lines().map(code_part).filter(|line| line.split_once("::").is_some_and(|(label, body)| label.strip_prefix("wRoamMon").is_some_and(|value| value.parse::<u8>().is_ok()) && body.trim().starts_with("roam_struct "))).count();
    let value = json!({"slotCount":slot_count,"inactiveMap":location("N_A")?,"initWrites":init_writes,"routes":routes,"jumpMask":count-1});
    let _: crystal_core::systems::special_routines::RoamingPokemonCatalog = serde_json::from_value(value.clone())?;
    let directory = data.join("content-packs/core-modular/roaming_pokemon");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), serde_json::to_vec_pretty(&value)?)?;
    std::fs::write(data.join("roaming_pokemon.json"), serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}
