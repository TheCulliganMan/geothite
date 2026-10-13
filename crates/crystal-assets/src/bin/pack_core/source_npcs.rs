use super::radio_text_export::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn integer(value: &str, constants: &BTreeMap<String, i64>) -> Result<i64> {
    if value.contains('|') {
        return value.split('|').try_fold(0, |result, part| {
            Ok(result | integer(part.trim(), constants)?)
        });
    }
    if let Some((left, right)) = value.split_once("<<") {
        return Ok(integer(left.trim(), constants)? << integer(right.trim(), constants)?);
    }
    if let Some(value) = constants.get(value) {
        return Ok(*value);
    }
    if let Some(value) = value.strip_prefix('$') {
        return Ok(i64::from_str_radix(value, 16)?);
    }
    if let Some(value) = value.strip_prefix('%') {
        return Ok(i64::from_str_radix(value, 2)?);
    }
    value
        .parse()
        .with_context(|| format!("unsupported object expression {value}"))
}
pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let mut constants = BTreeMap::from([
        ("MORN".to_owned(), 1),
        ("DAY".to_owned(), 2),
        ("NITE".to_owned(), 4),
        ("DARKNESS".to_owned(), 8),
    ]);
    let mut index = 0;
    for line in std::fs::read_to_string(vendor.join("constants/sprite_data_constants.asm"))?
        .lines()
        .map(code_part)
    {
        if let Some(value) = line.strip_prefix("const_def") {
            index = if value.trim().is_empty() {
                0
            } else {
                integer(value.trim(), &constants)?
            };
        }
        if let Some(name) = line.strip_prefix("const ") {
            constants.insert(name.to_owned(), index);
            index += 1;
        }
    }
    let mut sprite_ids = Vec::new();
    for line in std::fs::read_to_string(vendor.join("constants/sprite_constants.asm"))?
        .lines()
        .map(code_part)
    {
        if line.starts_with("DEF NUM_OVERWORLD_SPRITES ") {
            break;
        }
        if let Some(name) = line.strip_prefix("const ") {
            if name != "SPRITE_NONE" {
                sprite_ids.push(name.to_owned());
            }
        }
    }
    let sprite_source = std::fs::read_to_string(vendor.join("data/sprites/sprites.asm"))?;
    let sprites = sprite_source
        .lines()
        .filter_map(|line| code_part(line).strip_prefix("overworld_sprite "))
        .map(source_arguments)
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        sprites.len() == sprite_ids.len(),
        "sprite identity and definition table lengths differ"
    );
    let mut facings = BTreeMap::new();
    for (id, row) in sprite_ids.into_iter().zip(sprites) {
        ensure!(row.len() == 4, "source sprite row");
        facings.insert(id, row[2] != "STILL_SPRITE");
    }
    // Variable actors are assigned authored sprite identities by scripts.
    for line in std::fs::read_to_string(vendor.join("engine/events/std_scripts.asm"))?
        .lines()
        .map(code_part)
    {
        if let Some(args) = line.strip_prefix("variablesprite ") {
            let args = source_arguments(args)?;
            if args.len() == 2
                && let Some(value) = facings.get(&args[1]).copied()
            {
                facings.insert(args[0].clone(), value);
            }
        }
    }
    let data = root.join("apps/web/assets/data");
    let output = data.join("content-packs/core-modular/npcs");
    std::fs::create_dir_all(&output)?;
    let mut catalog = BTreeMap::new();
    let mut paths = std::fs::read_dir(vendor.join("maps"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    for path in paths
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "asm"))
    {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .context("source map filename")?;
        let source = std::fs::read_to_string(&path)?;
        let mut identifiers = Vec::new();
        let mut reading = false;
        for line in source
            .lines()
            .map(code_part)
            .filter(|line| !line.is_empty())
        {
            if line == "object_const_def" {
                reading = true;
                continue;
            }
            if reading {
                if let Some(id) = line.strip_prefix("const ") {
                    identifiers.push(id.to_owned());
                } else {
                    reading = false;
                }
            }
        }
        let mut objects = Vec::new();
        for args in source
            .lines()
            .filter_map(|line| code_part(line).strip_prefix("object_event "))
        {
            let row = source_arguments(args)?;
            ensure!(row.len() == 13, "invalid object_event in {name}");
            let value = json!({"x":integer(&row[0],&constants)?,"y":integer(&row[1],&constants)?,"sprite":row[2],
                "sprite_has_facings":facings.get(&row[2]).copied().unwrap_or(false),"spritemovedata":row[3],
                "move_range_x":integer(&row[4],&constants)?,"move_range_y":integer(&row[5],&constants)?,
                "hram_x":integer(&row[6],&constants)?,"hram_y":integer(&row[7],&constants)?,"pal":integer(&row[8],&constants)?,
                "object_type":row[9],"radius":integer(&row[10],&constants)?,"script":row[11],"label":null,
                "event_flag":row[12],"object_identifier":identifiers.get(objects.len()),"sightline_direction_override":null});
            let _: crystal_core::map::ObjectEvent = serde_json::from_value(value.clone())
                .with_context(|| format!("validate source NPC {name}"))?;
            objects.push(value);
        }
        ensure!(
            identifiers.is_empty() || identifiers.len() == objects.len(),
            "object identity count differs for {name}"
        );
        std::fs::write(
            output.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&json!({name:objects}))?,
        )?;
        catalog.insert(name.to_owned(), Value::Array(objects));
    }
    std::fs::write(data.join("npcs.json"), serde_json::to_vec_pretty(&catalog)?)?;
    Ok(())
}
