use anyhow::{Context, Result};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

fn source_names(repository: &Path) -> Result<Vec<(String, String)>> {
    let source =
        std::fs::read_to_string(repository.join("vendor/pokecrystal/data/items/names.asm"))?;
    let names = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("li "))
        .map(|line| serde_json::from_str::<String>(line).context("invalid ItemNames string"))
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(
        names.len() == 256,
        "ItemNames must contain 256 source slots"
    );
    let constants = std::fs::read_to_string(
        repository.join("vendor/pokecrystal/constants/item_constants.asm"),
    )?;
    let mut in_macro = false;
    let mut index = 0usize;
    let mut result = Vec::new();
    for raw in constants.lines() {
        let line = raw.split(';').next().unwrap().trim();
        if line.starts_with("MACRO ") {
            in_macro = true;
            continue;
        }
        if line == "ENDM" {
            in_macro = false;
            continue;
        }
        if in_macro {
            continue;
        }
        let Some((command, operand)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let prefix = match command {
            "const" => "",
            "add_tm" => "TM_",
            "add_hm" => "HM_",
            _ => continue,
        };
        let id = format!("{prefix}{}", operand.trim());
        anyhow::ensure!(
            !id.contains(char::is_whitespace),
            "invalid item constant {id}"
        );
        if index == 0 {
            anyhow::ensure!(id == "NO_ITEM", "item constants must start at NO_ITEM");
        } else {
            let name = names
                .get(index - 1)
                .context("item constant exceeds ItemNames")?;
            anyhow::ensure!(
                !result.iter().any(|(previous, _)| previous == &id),
                "duplicate item constant"
            );
            result.push((id, name.clone()));
        }
        index += 1;
    }
    anyhow::ensure!(index == 251, "item constants must end at ITEM_FA");
    Ok(result)
}

fn source_descriptions(repository: &Path) -> Result<Vec<String>> {
    let source =
        std::fs::read_to_string(repository.join("vendor/pokecrystal/data/items/descriptions.asm"))?;
    let (table, bodies) = source
        .split_once("assert_table_length $ff")
        .context("ItemDescriptions has no source table assertion")?;
    let pointers = table
        .lines()
        .filter_map(|line| line.trim().strip_prefix("dw "))
        .map(str::trim)
        .collect::<Vec<_>>();
    anyhow::ensure!(
        pointers.len() == 255,
        "ItemDescriptions must contain 255 source slots"
    );
    let mut descriptions = BTreeMap::new();
    let mut current_label: Option<String> = None;
    let mut lines = Vec::new();
    for raw in bodies.lines() {
        let code = raw.trim();
        if code.is_empty() || code.starts_with(';') {
            continue;
        }
        if let Some(label) = code.strip_suffix(':') {
            anyhow::ensure!(current_label.is_none(), "unterminated item description");
            current_label = Some(label.to_string());
            continue;
        }
        let (command, operand) = code
            .split_once(char::is_whitespace)
            .context("invalid item description command")?;
        anyhow::ensure!(
            (command == "db" && lines.is_empty()) || (command == "next" && !lines.is_empty()),
            "invalid item description line order"
        );
        let text: String =
            serde_json::from_str(operand.trim()).context("invalid item description string")?;
        let finished = text.ends_with('@');
        let text = text.strip_suffix('@').unwrap_or(&text);
        anyhow::ensure!(!text.contains('@'), "embedded item description terminator");
        lines.push(text.to_string());
        if finished {
            let label = current_label
                .take()
                .context("item description has no label")?;
            anyhow::ensure!(
                descriptions.insert(label, lines.join("\n")).is_none(),
                "duplicate item description"
            );
            lines.clear();
        }
    }
    anyhow::ensure!(
        current_label.is_none() && lines.is_empty(),
        "unterminated item description"
    );
    pointers
        .into_iter()
        .map(|label| {
            descriptions
                .get(label)
                .cloned()
                .with_context(|| format!("item description {label} is missing"))
        })
        .collect()
}

pub fn export(repository: &Path) -> Result<()> {
    let names = source_names(repository)?;
    let descriptions = source_descriptions(repository)?;
    let data = repository.join("apps/web/assets/data");
    let path = data.join("items.json");
    let mut aggregate: Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let items = aggregate
        .as_array_mut()
        .context("items catalog must be an array")?;
    let mut outputs = Vec::new();
    for (index, (id, name)) in names.iter().enumerate() {
        let item = items
            .iter_mut()
            .find(|item| item.get("script_name").and_then(Value::as_str) == Some(id.as_str()))
            .with_context(|| format!("source item {id} is absent from the aggregate catalog"))?;
        item["name"] = Value::String(name.clone());
        item["description"] = Value::String(descriptions[index].clone());
        let modular_path = data
            .join("content-packs/core-modular/items")
            .join(format!("{id}.json"));
        let mut modular: Value = serde_json::from_slice(&std::fs::read(&modular_path)?)?;
        let definition = modular.get_mut(id).context("modular item key is absent")?;
        definition["name"] = Value::String(name.clone());
        definition["description"] = Value::String(descriptions[index].clone());
        outputs.push((modular_path, modular));
    }
    outputs.push((path, aggregate));
    for (path, value) in outputs {
        let mut bytes = serde_json::to_vec_pretty(&value)?;
        bytes.push(b'\n');
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn item_names_preserve_source_tokens_and_tm_hm_slots() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let names = super::source_names(&root)
            .unwrap()
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(names.len(), 250);
        assert_eq!(names["POKE_BALL"], "# BALL");
        assert_eq!(names["TM_MUD_SLAP"], "TM31");
        assert_eq!(names["HM_CUT"], "HM01");
        assert_eq!(names["ITEM_FA"], "TERU-SAMA");
    }
    #[test]
    fn item_descriptions_preserve_source_next_lines_and_hyphens() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let descriptions = super::source_descriptions(&root).unwrap();
        assert_eq!(descriptions.len(), 255);
        assert_eq!(descriptions[0x12 - 1], "Restores #MON\nHP by 20.");
        assert_eq!(descriptions[0x14 - 1], "Repels weak #-\nMON for 100 steps.");
        assert_eq!(descriptions[0x05 - 1], "An item for catch-\ning #MON.");
    }
    #[test]
    fn every_item_description_matches_the_pinned_rom_bytes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let descriptions = super::source_descriptions(&root).unwrap();
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../../tools/asm-oracle/fixtures/item-descriptions.json"
        ))
        .unwrap();
        let entries = fixture["entries"].as_array().unwrap();
        assert_eq!(entries.len(), descriptions.len());
        for (index, (description, entry)) in descriptions.iter().zip(entries).enumerate() {
            assert!(crystal_core::models::item::is_exact_item_description(description),
                "source description slot {} was rejected by item validation", index + 1);
            let mut bytes = Vec::new();
            for (line_index, line) in description.split('\n').enumerate() {
                if line_index != 0 {
                    bytes.push(0x4e);
                }
                bytes.extend(crystal_core::systems::radio_text::encode_radio_string(line).unwrap());
            }
            bytes.push(0x50);
            let encoded: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
            assert_eq!(
                encoded,
                entry["bytes"].as_str().unwrap(),
                "item slot {}",
                index + 1
            );
        }
    }
}
