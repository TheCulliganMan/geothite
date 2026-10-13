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

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let source = std::fs::read_to_string(vendor.join("data/radio/buenas_passwords.asm"))?;
    let mut order = Vec::new();
    let mut categories = BTreeMap::new();
    for line in source.lines().map(code_part) {
        if let Some(label) = line.strip_prefix("dw .") { order.push(label.to_owned()); }
        if let Some((label, operands)) = line.split_once(':') {
            if let Some(operands) = operands.trim().strip_prefix("db ") {
                let args = source_arguments(operands)?;
                ensure!(args.len() == 5, "authored Buena category operands");
                let options = args[2..].iter().map(|arg| {
                    if args[0] == "BUENA_STRING" { arg.trim_matches('"').trim_end_matches('@').replace('#', "POKé") }
                    else { arg.clone() }
                }).collect::<Vec<_>>();
                categories.insert(label.trim_start_matches('.').to_owned(), json!({"categoryType":args[0],"points":args[1].parse::<u8>()?,"options":options}));
            }
        }
    }
    let value = json!({"order":order,"categories":categories});
    let _: crystal_core::systems::special_routines::BuenaPasswordCategories = serde_json::from_value(value.clone())?;
    write(&data, "buena_password_categories", &value)?;
    let read_rows = |path: &str| -> Result<Vec<Vec<String>>> {
        std::fs::read_to_string(vendor.join(path))?.lines().filter_map(|line| code_part(line).strip_prefix("db ")).map(source_arguments).collect()
    };
    let mut prizes = BTreeMap::new();
    for row in read_rows("data/items/buena_prizes.asm")? {
        ensure!(row.len() == 2, "Buena source prize operands");
        prizes.insert(row[0].clone(), row[1].parse::<u8>()?);
    }
    let value = json!(prizes);
    let _: crystal_core::systems::special_routines::BuenaPrizeDefinitionTable = serde_json::from_value(value.clone())?;
    write(&data, "buena_prizes", &value)?;
    let mut apricorns = BTreeMap::new();
    for row in read_rows("data/items/apricorn_balls.asm")? {
        if row == &["-1"] { break; }
        ensure!(row.len() == 2, "Kurt source recipe operands");
        ensure!(apricorns.insert(row[0].clone(), row[1].clone()).is_none(), "duplicate apricorn recipe");
    }
    ensure!(apricorns.len() == 7, "source apricorn recipe count");
    write(&data, "kurt_apricorn_recipes", &json!(apricorns)).context("write authored Kurt recipes")?;
    let fruit_ids = std::fs::read_to_string(vendor.join("constants/script_constants.asm"))?.lines().filter_map(|line| code_part(line).strip_prefix("const FRUITTREE_").map(|name| format!("FRUITTREE_{name}"))).collect::<Vec<_>>();
    let fruit_rows = read_rows("data/items/fruit_trees.asm")?;
    ensure!(fruit_ids.len() == fruit_rows.len(), "source fruit tree pointer count");
    let mut fruits = BTreeMap::new();
    for (id, row) in fruit_ids.into_iter().zip(fruit_rows) {
        ensure!(row.len() == 1, "source fruit tree item");
        fruits.insert(id, row[0].clone());
    }
    let _: crystal_core::systems::field_items::FruitTreeCatalog = serde_json::from_value(json!(fruits))?;
    write(&data, "fruit_trees", &json!(fruits))?;
    let mut buckets = BTreeMap::<String, Vec<String>>::new();
    let mut bucket = None;
    for line in std::fs::read_to_string(vendor.join("data/wild/flee_mons.asm"))?.lines().map(code_part) {
        if let Some(label) = line.strip_suffix("FleeMons:") {
            bucket = match label { "Sometimes" => Some("sometimes"), "Often" => Some("often"), "Always" => Some("always"), "" => None, _ => anyhow::bail!("source flee table label {label}") };
        }
        if let Some(species) = line.strip_prefix("db ") {
            if species == "-1" { continue; }
            buckets.entry(bucket.context("source flee bucket")?.to_owned()).or_default().push(species.to_owned());
        }
    }
    let value = json!({"buckets":buckets});
    let _: crystal_core::systems::flee_mons::FleeMonTables = serde_json::from_value(value.clone())?;
    write(&data, "flee_mons", &value)
}
