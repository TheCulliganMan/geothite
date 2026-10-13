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
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let number = |name: &str| -> Result<i64> { constants["global"][name].as_i64().with_context(|| format!("source constant {name}")) };
    let mut sprite_ids = Vec::new();
    for line in read("constants/sprite_constants.asm")?.lines().map(code_part) {
        if line.starts_with("DEF NUM_OVERWORLD_SPRITES") { break; }
        if let Some(name) = line.strip_prefix("const SPRITE_").filter(|name| *name != "NONE") { sprite_ids.push(format!("SPRITE_{name}")); }
    }
    let sprite_source = read("data/sprites/sprites.asm")?;
    let rows = sprite_source.lines().filter_map(|line| code_part(line).strip_prefix("overworld_sprite ")).map(source_arguments).collect::<Result<Vec<_>>>()?;
    ensure!(sprite_ids.len() == rows.len(), "authored sprite palette count");
    let mut palettes = BTreeMap::new();
    for (id, row) in sprite_ids.into_iter().zip(rows) { ensure!(row.len() == 4, "source sprite palette operands"); palettes.insert(id, number(&row[3])?); }
    write(&data, "sprite_palette_defaults", &json!(palettes))?;
    let mut palette_map = BTreeMap::<String, Vec<String>>::new();
    let mut key = None;
    let palette_source = read("gfx/pokegear/town_map_palette_map.asm")?;
    let palette_tokens = palette_source.lines().filter_map(|line| code_part(line).strip_prefix("const PAL_TOWNMAP_")).collect::<Vec<_>>();
    ensure!(!palette_tokens.is_empty(), "authored town map palette constants");
    for raw in palette_source.lines() {
        if let Some(name) = raw.trim().strip_prefix("; gfx/pokegear/").and_then(|name| name.strip_suffix(".png")) { key = Some(name.to_owned()); }
        if let Some(arguments) = code_part(raw).strip_prefix("townmappals ") {
            let row = source_arguments(arguments)?;
            for palette in &row { ensure!(palette_tokens.contains(&palette.as_str()), "source town map palette {palette}"); }
            palette_map.entry(key.clone().context("source town map image header")?).or_default().extend(row);
        }
    }
    ensure!(palette_map.contains_key("town_map") && palette_map.contains_key("pokegear"), "source town map palette images");
    write(&data, "pokegear_town_map_palette_map", &json!(palette_map))?;
    let source = read("data/maps/landmarks.asm")?;
    let mut names = BTreeMap::new();
    let mut rows = Vec::new();
    for line in source.lines().map(code_part) {
        if let Some(row) = line.strip_prefix("landmark ") { rows.push(source_arguments(row)?); }
        if let Some((label, body)) = line.split_once(':') {
            if let Some(text) = body.trim().strip_prefix("db ").filter(|text| text.starts_with('"')) { names.insert(label, text.trim_matches('"').trim_end_matches('@').replace("<BSP>", " ").replace('#', "POKé")); }
        }
    }
    let landmark_constants = read("constants/landmark_constants.asm")?;
    let ids = landmark_constants.lines().map(code_part).take_while(|line| !line.starts_with("DEF NUM_LANDMARKS ")).filter_map(|line| line.strip_prefix("const LANDMARK_").map(|id| format!("LANDMARK_{id}"))).collect::<Vec<_>>();
    ensure!(ids.len() == rows.len(), "source landmark count");
    let mut landmarks = Vec::new();
    for (constant, row) in ids.into_iter().zip(rows) {
        ensure!(row.len() == 3, "source landmark operands");
        let id = number(&constant)?;
        landmarks.push(json!({"id":id,"constant":constant,"label":constant.strip_prefix("LANDMARK_").unwrap(),"name":names.get(row[2].as_str()).context("source landmark name")?,"x":row[0].parse::<i16>()?,"y":row[1].parse::<i16>()?,"region":if id >= number("KANTO_LANDMARK")? && id < number("OTHER_LANDMARK")? {"KANTO"} else {"JOHTO"}}));
    }
    let attributes: BTreeMap<String, crystal_core::map::MapAttributes> = serde_json::from_slice(&std::fs::read(data.join("map_attributes.json"))?)?;
    let map_to_landmark = attributes.into_iter().map(|(id, value)| (id, value.location)).collect::<BTreeMap<_, _>>();
    let value = json!({"landmarks":landmarks,"map_to_landmark":map_to_landmark});
    let _: crystal_core::models::display_metadata::PokegearLandmarksPayload = serde_json::from_value(value.clone())?;
    write(&data, "pokegear_landmarks", &value)
}
