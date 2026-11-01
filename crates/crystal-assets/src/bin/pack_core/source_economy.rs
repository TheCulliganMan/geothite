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
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let mut currency = BTreeMap::new();
    for (name, value) in constants["global"].as_object().context("source constants")? {
        if name.contains("MONEY") || name.contains("COINS") || name.ends_with("_PRICE") || name.ends_with("_COST") {
            if let Some(value) = value.as_u64() { currency.insert(name.clone(), u32::try_from(value)?); }
        }
    }
    for id in ["START_MONEY", "MAX_MONEY", "MOM_MONEY", "MAX_COINS"] { ensure!(currency.contains_key(id), "source currency {id}"); }
    let _: crystal_core::systems::economy::CurrencyCatalog = serde_json::from_value(json!(currency))?;
    write(&data, "currency_constants", &json!(currency))?;
    let source = std::fs::read_to_string(vendor.join("data/items/marts.asm"))?;
    let mut pointers = Vec::new();
    let mut inventories = BTreeMap::<String, Vec<String>>::new();
    let mut current = None;
    for line in source.lines().map(code_part) {
        if let Some(pointer) = line.strip_prefix("dw ") { pointers.push(pointer.to_owned()); }
        if let Some(label) = line.strip_suffix(':') { inventories.entry(label.into()).or_default(); current = Some(label.to_owned()); }
        if let Some(arguments) = line.strip_prefix("db ") {
            if let Some(label) = &current { inventories.get_mut(label).unwrap().extend(source_arguments(arguments)?); }
        }
    }
    let ids = std::fs::read_to_string(vendor.join("constants/mart_constants.asm"))?.lines().filter_map(|line| code_part(line).strip_prefix("const MART_").filter(|name| !name.starts_with("TYPE_")).map(|name| format!("MART_{name}"))).collect::<Vec<_>>();
    ensure!(ids.len() == pointers.len(), "source mart pointer count");
    let mut marts = BTreeMap::new();
    for (id, pointer) in ids.into_iter().zip(pointers) {
        let inventory = inventories.get(&pointer).with_context(|| format!("source mart {pointer}"))?;
        let count = inventory.first().context("mart item count")?.parse::<usize>()?;
        ensure!(inventory.len() == count + 2 && inventory.last().is_some_and(|value| value == "-1"), "source mart count/terminator {pointer}");
        marts.insert(id, inventory[1..=count].to_vec());
    }
    let _: crystal_core::systems::shop::MartCatalog = serde_json::from_value(json!(marts))?;
    write(&data, "marts", &json!(marts))
}
