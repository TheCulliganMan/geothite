//! Authored Headbutt/Rock Smash tables and time-dependent sleeping species.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{FieldEncounterData, FieldEncounterTable};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub fn tables(
    source: &str,
    asleep: &str,
    sleep_turns: u8,
) -> Result<BTreeMap<String, FieldEncounterTable>> {
    ensure!(
        (1..=7).contains(&sleep_turns),
        "invalid authored tree sleep counter"
    );
    let mut sleeping = BTreeMap::<String, BTreeMap<String, u8>>::new();
    let mut time = None;
    for line in asleep.lines().map(code_part) {
        if let Some(label) = line.strip_suffix(':') {
            time = Some(match label {
                "AsleepTreeMonsMorn" => "morning",
                "AsleepTreeMonsDay" => "day",
                "AsleepTreeMonsNite" => "night",
                _ => anyhow::bail!("unknown sleeping tree table {label}"),
            });
        } else if let Some(species) = line.strip_prefix("db ") {
            if species == "-1" {
                time = None;
                continue;
            }
            let time = time.context("sleep species outside terminated table")?;
            ensure!(
                sleeping
                    .entry(species.to_string())
                    .or_default()
                    .insert(time.to_string(), sleep_turns)
                    .is_none(),
                "duplicate sleeping species/time"
            );
        }
    }
    ensure!(time.is_none(), "unterminated sleeping species table");
    let mut raw = BTreeMap::<String, Vec<Vec<Value>>>::new();
    let mut current = None;
    let mut rows = Vec::new();
    for line in source.lines().map(code_part) {
        if let Some(label) = line
            .strip_suffix(':')
            .filter(|label| label.starts_with("TreeMonSet_"))
        {
            ensure!(rows.is_empty(), "unterminated encounter table");
            current = Some(label.to_string());
            ensure!(
                raw.insert(label.to_string(), Vec::new()).is_none(),
                "duplicate encounter label"
            );
        } else if let Some(args) = line.strip_prefix("db ").filter(|_| current.is_some()) {
            if args == "-1" {
                ensure!(!rows.is_empty(), "empty terminated encounter table");
                raw.get_mut(current.as_ref().unwrap())
                    .unwrap()
                    .push(std::mem::take(&mut rows));
            } else {
                let args = source_arguments(args)?;
                ensure!(
                    args.len() == 3,
                    "encounter row requires weight/species/level"
                );
                let weight: u8 = args[0].parse()?;
                let level: u8 = args[2].parse()?;
                let sleep = if current.as_deref() == Some("TreeMonSet_Rock") {
                    BTreeMap::new()
                } else {
                    sleeping.get(&args[1]).cloned().unwrap_or_default()
                };
                rows.push(json!({"weight": weight, "species": args[1], "level": level, "sleep_turns_by_time": sleep}));
            }
        }
    }
    ensure!(rows.is_empty(), "unterminated final encounter table");
    let none = raw
        .remove("TreeMonSet_None")
        .context("missing no-encounter sentinel")?;
    ensure!(none.is_empty(), "no-encounter sentinel has rows");
    raw.into_iter()
        .map(|(name, mut groups)| {
            ensure!(
                groups.len() == if name == "TreeMonSet_Rock" { 1 } else { 2 },
                "wrong common/rare table count for {name}"
            );
            for group in &groups {
                ensure!(
                    group
                        .iter()
                        .map(|entry| entry["weight"].as_u64().unwrap())
                        .sum::<u64>()
                        == 100,
                    "source encounter weights do not sum to 100"
                );
            }
            let common = groups.remove(0);
            let rare = groups.pop().unwrap_or_default();
            Ok((
                name,
                serde_json::from_value(json!({"common": common, "rare": rare}))?,
            ))
        })
        .collect()
}

pub fn map_catalog(
    source: &str,
    names: &BTreeMap<String, String>,
    sets: &BTreeMap<String, FieldEncounterTable>,
) -> Result<BTreeMap<String, FieldEncounterData>> {
    let mut result = BTreeMap::<String, FieldEncounterData>::new();
    let mut kind = None;
    for line in source.lines().map(code_part) {
        if line == "TreeMonMaps:" {
            kind = Some("headbutt");
        } else if line == "RockMonMaps:" {
            kind = Some("rock_smash");
        } else if line == "db -1" {
            kind = None;
        } else if let Some(args) = line.strip_prefix("treemon_map ") {
            let kind = kind.context("encounter map outside authored table")?;
            let args = source_arguments(args)?;
            ensure!(args.len() == 2, "encounter map requires map/set");
            let name = names
                .get(&args[0])
                .context("unknown source encounter map")?;
            let entry = result
                .entry(name.clone())
                .or_insert_with(|| FieldEncounterData {
                    map_name: name.clone(),
                    tables: BTreeMap::new(),
                });
            if args[1] == "TREEMON_SET_NONE" {
                continue;
            }
            let suffix = args[1]
                .strip_prefix("TREEMON_SET_")
                .context("invalid encounter set")?;
            ensure!(
                !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_uppercase()),
                "invalid encounter set suffix"
            );
            let label = format!(
                "TreeMonSet_{}{}",
                &suffix[..1],
                suffix[1..].to_ascii_lowercase()
            );
            let table = sets.get(&label).context("unknown authored encounter set")?;
            ensure!(
                entry
                    .tables
                    .insert(kind.to_string(), table.clone())
                    .is_none(),
                "duplicate map encounter kind"
            );
        }
    }
    ensure!(
        kind.is_none() && !result.is_empty(),
        "unterminated/empty encounter maps"
    );
    Ok(result)
}

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let read = |path: &str| {
        std::fs::read_to_string(vendor.join(path))
            .with_context(|| format!("field encounter source {path}"))
    };
    let constants = read("constants/battle_constants.asm")?;
    let turns: u8 = constants
        .lines()
        .map(code_part)
        .find_map(|line| line.strip_prefix("DEF TREEMON_SLEEP_TURNS EQU "))
        .context("missing source tree sleep counter")?
        .parse()?;
    let sets = tables(
        &read("data/wild/treemons.asm")?,
        &read("data/wild/treemons_asleep.asm")?,
        turns,
    )?;
    let attributes: BTreeMap<String, crystal_core::map::MapAttributes> =
        serde_json::from_slice(&std::fs::read(data.join("map_attributes.json"))?)?;
    let names = attributes
        .into_iter()
        .filter_map(|(name, attr)| attr.map_constant.map(|constant| (constant, name)))
        .collect();
    let catalog = map_catalog(&read("data/wild/treemon_maps.asm")?, &names, &sets)?;
    let bytes = serde_json::to_vec_pretty(&catalog)?;
    let directory = data.join("content-packs/core-modular/field_encounters");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), &bytes)?;
    std::fs::write(data.join("field_encounters.json"), bytes)?;
    Ok(())
}
