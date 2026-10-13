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
fn text(value: &str) -> Result<String> {
    let value = value.trim();
    ensure!(value.starts_with('"') && value.ends_with('"'), "authored presentation string {value}");
    Ok(value[1..value.len()-1].trim_end_matches('@').replace("<PK><MN>", "POKéMON").replace("<PK>", "POKé").replace("<MN>", "MON").replace('#', "POKé"))
}
pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let pokemon: BTreeMap<String, Value> = serde_json::from_slice(&std::fs::read(data.join("pokemon.json"))?)?;
    let mut ordered = pokemon.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(_, value)| value["int_id"].as_u64());
    let mut icons = BTreeMap::new();
    let icon_source = read("data/pokemon/menu_icons.asm")?;
    let icon_rows = icon_source.lines().filter_map(|line| code_part(line).strip_prefix("db ")).collect::<Vec<_>>();
    ensure!(icon_rows.len() == ordered.len(), "source menu icon species count");
    for ((id, _), icon) in ordered.iter().zip(icon_rows) { icons.insert((*id).clone(), icon.to_owned()); }
    let icon_constants = read("constants/icon_constants.asm")?;
    ensure!(icon_constants.contains("const ICON_EGG"), "authored egg menu icon");
    icons.insert("EGG".into(), "ICON_EGG".into());
    write(&data, "menu_icons", &json!(icons))?;
    let mut pc = BTreeMap::new();
    for line in read("engine/pokemon/bills_pc.asm")?.lines().map(code_part) {
        if let Some((label, body)) = line.split_once(':') {
            if label.starts_with("PCString_") {
                let value = body.trim().strip_prefix("db ").context("source PC string")?;
                pc.insert(label.to_owned(), text(value)?);
            }
        }
    }
    ensure!(!pc.is_empty(), "source PC strings");
    write(&data, "pc_strings", &json!(pc))?;
    let mut names = Vec::new();
    for line in read("data/moves/names.asm")?.lines().map(code_part) {
        if let Some(value) = line.strip_prefix("li ") { names.push(text(value)?); }
    }
    ensure!(names.len() == 251, "source move name count");
    write(&data, "move_names", &json!(names))?;
    let canonical = |value: &str| value.bytes().filter(u8::is_ascii_alphanumeric).map(|byte| char::from(byte.to_ascii_lowercase())).collect::<String>();
    let mut entries = BTreeMap::new();
    for file in std::fs::read_dir(vendor.join("data/pokemon/dex_entries"))? {
        let path = file?.path();
        if !path.extension().is_some_and(|extension| extension == "asm") { continue; }
        let stem = path.file_stem().and_then(|stem| stem.to_str()).context("source dex filename")?;
        let candidates = pokemon.keys().filter(|id| canonical(id) == canonical(stem)).collect::<Vec<_>>();
        ensure!(candidates.len() == 1, "source dex identity {stem}");
        let id = candidates[0];
        let mut classification = None;
        let mut dimensions = None;
        let mut pages = Vec::new();
        let mut page = Vec::new();
        for line in std::fs::read_to_string(&path)?.lines().map(code_part) {
            if let Some(row) = line.strip_prefix("dw ") { dimensions = Some(source_arguments(row)?); continue; }
            if let Some((command, operand)) = line.split_once(' ') {
                if ["db", "next", "page"].contains(&command) {
                    let value = text(operand)?;
                    if dimensions.is_none() { classification = Some(value); continue; }
                    if command == "page" && !page.is_empty() { pages.push(page.join(" @ ")); page.clear(); }
                    page.push(value);
                }
            }
        }
        if !page.is_empty() { pages.push(page.join(" @ ")); }
        let size = dimensions.context("source dex dimensions")?;
        ensure!(size.len() == 2, "source dex dimensions operands");
        let value = json!({"species":id,"classification":classification.context("source dex classification")?,"heightDigits":size[0].parse::<u16>()?,"weightDigits":size[1].parse::<u16>()?,"pages":pages});
        let _: crystal_core::models::pokedex::RuntimePokedexEntry = serde_json::from_value(value.clone())?;
        ensure!(entries.insert(id.clone(), value).is_none(), "duplicate source dex identity");
    }
    ensure!(entries.len() == 251, "source dex entry count");
    write(&data, "pokedex_entries", &json!(entries))
}
