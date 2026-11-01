//! Preserve the authored integer ratios; rounded percentages affect damage.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Result, ensure};
use crystal_core::battle::stats::BattleStatMultiplierTables;
use serde_json::{Value, json};
use std::path::Path;
#[path = "source_capture_rules.rs"]
mod source_capture_rules;

pub(super) fn wobble(source: &str) -> Result<Value> {
    let mut rows = Vec::new();
    for line in source.lines().map(code_part) {
        let Some(raw) = line.strip_prefix("db ") else {
            continue;
        };
        let args = source_arguments(raw)?;
        ensure!(args.len() == 2, "invalid wobble probability row");
        rows.push(json!({"catch_rate":args[0].parse::<u8>()?,"chance":args[1].parse::<u8>()?}));
    }
    let value = json!(rows);
    let _: crystal_core::battle::capture::CaptureWobbleProbabilityTable =
        serde_json::from_value(value.clone())?;
    Ok(value)
}

pub(super) fn weather(source: &str, constants: &str) -> Result<Value> {
    let mut ratios = std::collections::BTreeMap::new();
    for line in constants.lines().map(code_part) {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() == 4 && words[0] == "DEF" && words[2] == "EQU" {
            if let Ok(value) = words[3].parse::<u16>() {
                ratios.insert(words[1], value);
            }
        }
    }
    let denominator = ratios
        .get("EFFECTIVE")
        .ok_or_else(|| anyhow::anyhow!("missing EFFECTIVE scale"))?;
    let mut value = json!({"type_modifiers":{},"move_effect_modifiers":{}});
    for (label, key) in [
        ("WeatherTypeModifiers:", "type_modifiers"),
        ("WeatherMoveModifiers:", "move_effect_modifiers"),
    ] {
        let body = source
            .split(label)
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("missing {label}"))?;
        let mut ended = false;
        for line in body.lines().map(code_part) {
            let Some(raw) = line.strip_prefix("db ") else {
                continue;
            };
            if raw == "-1" {
                ended = true;
                break;
            }
            let args = source_arguments(raw)?;
            ensure!(args.len() == 3, "invalid weather modifier row");
            let numerator = ratios
                .get(args[2].as_str())
                .ok_or_else(|| anyhow::anyhow!("unknown weather ratio"))?;
            let subjects = value[key]
                .as_object_mut()
                .unwrap()
                .entry(args[0].clone())
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .unwrap();
            ensure!(
                subjects
                    .insert(
                        if key == "move_effect_modifiers" {
                            args[1]
                                .strip_prefix("EFFECT_")
                                .ok_or_else(|| anyhow::anyhow!("invalid weather effect"))?
                                .to_string()
                        } else {
                            args[1].clone()
                        },
                        json!({"numerator":numerator,"denominator":denominator})
                    )
                    .is_none(),
                "duplicate weather modifier"
            );
        }
        ensure!(ended, "missing weather sentinel");
    }
    let _: crystal_core::battle::damage::WeatherModifiers = serde_json::from_value(value.clone())?;
    Ok(value)
}

pub(super) fn type_tables(types: &str, matchups: &str, constants: &str) -> Result<(Value, Value)> {
    let mut physical = Vec::new();
    let mut special = Vec::new();
    let mut section = "";
    for line in types.lines().map(code_part) {
        if line == "DEF PHYSICAL EQU const_value" {
            section = "physical";
        } else if line == "DEF SPECIAL EQU const_value" {
            section = "special";
        } else if line.starts_with("DEF UNUSED_TYPES ") || line.starts_with("DEF TYPES_END ") {
            section = "";
        } else if let Some(id) = line.strip_prefix("const ") {
            if section == "physical" {
                physical.push(id);
            } else if section == "special" {
                special.push(id);
            }
        }
    }
    let categories = json!({"physical":physical,"special":special});
    let _: crystal_core::battle::damage::TypeCategories =
        serde_json::from_value(categories.clone())?;
    let mut ratios = std::collections::BTreeMap::new();
    for line in constants.lines().map(code_part) {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() == 4 && words[0] == "DEF" && words[2] == "EQU" {
            if let Ok(value) = words[3].parse::<u16>() {
                ratios.insert(words[1], value);
            }
        }
    }
    let denominator = *ratios
        .get("EFFECTIVE")
        .ok_or_else(|| anyhow::anyhow!("missing EFFECTIVE scale"))?;
    let mut normal = Vec::new();
    let mut foresight = Vec::new();
    let mut seen_foresight = false;
    let mut ended = false;
    for line in matchups.lines().map(code_part) {
        let Some(raw) = line.strip_prefix("db ") else {
            continue;
        };
        ensure!(!ended, "matchups after end sentinel");
        if raw == "-2" {
            ensure!(!seen_foresight, "duplicate Foresight sentinel");
            seen_foresight = true;
            continue;
        }
        if raw == "-1" {
            ended = true;
            continue;
        }
        let args = source_arguments(raw)?;
        ensure!(args.len() == 3, "invalid matchup row");
        let numerator = ratios
            .get(args[2].as_str())
            .ok_or_else(|| anyhow::anyhow!("unknown effectiveness {}", args[2]))?;
        let row = json!({"attacker":args[0],"defender":args[1],"multiplier":{"numerator":numerator,"denominator":denominator}});
        if seen_foresight {
            foresight.push(row);
        } else {
            normal.push(row);
        }
    }
    ensure!(ended && seen_foresight, "missing type table sentinel");
    let table = json!({"matchups":normal,"foresight_matchups":foresight});
    let _: crystal_core::battle::damage::TypeEffectivenessTable =
        serde_json::from_value(table.clone())?;
    Ok((categories, table))
}

pub(super) fn priorities(
    effects: &str,
    rows: &str,
    constants: &str,
    engine: &str,
) -> Result<Value> {
    let base: i8 = constants
        .lines()
        .map(code_part)
        .find_map(|line| line.strip_prefix("DEF BASE_PRIORITY EQU "))
        .ok_or_else(|| anyhow::anyhow!("missing authored BASE_PRIORITY"))?
        .parse()?;
    let mut values = serde_json::Map::new();
    for line in effects.lines().map(code_part) {
        if let Some(effect) = line.strip_prefix("const EFFECT_") {
            ensure!(
                values
                    .insert(
                        effect
                            .replace("SP_ATK", "SPECIAL_ATTACK")
                            .replace("SP_DEF", "SPECIAL_DEFENSE"),
                        json!(base)
                    )
                    .is_none(),
                "duplicate effect"
            );
        }
    }
    let mut ended = false;
    for line in rows.lines().map(code_part) {
        let Some(raw) = line.strip_prefix("db ") else {
            continue;
        };
        ensure!(!ended, "priority rows after sentinel");
        if raw == "-1" {
            ended = true;
            continue;
        }
        let args = source_arguments(raw)?;
        ensure!(args.len() == 2, "invalid priority row");
        let effect = args[0]
            .strip_prefix("EFFECT_")
            .ok_or_else(|| anyhow::anyhow!("invalid priority effect"))?;
        let effect = effect
            .replace("SP_ATK", "SPECIAL_ATTACK")
            .replace("SP_DEF", "SPECIAL_DEFENSE");
        ensure!(values.contains_key(&effect), "unknown priority effect");
        values.insert(effect, json!(args[1].parse::<i8>()?));
    }
    ensure!(ended, "missing priority sentinel");
    let body = engine
        .split("GetMovePriority:")
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("missing priority routine"))?
        .split("call GetMoveEffect")
        .next()
        .unwrap();
    let code: Vec<_> = body
        .lines()
        .map(code_part)
        .filter(|s| !s.is_empty())
        .collect();
    let override_row = code
        .windows(3)
        .find(|w| w[0].starts_with("cp ") && w[1].starts_with("ld a, ") && w[2] == "ret z")
        .ok_or_else(|| anyhow::anyhow!("missing authored move priority override"))?;
    let value = json!({"base_priority":base,"effect_priorities":values,"move_priorities":[{"move":override_row[0].trim_start_matches("cp "),"priority":override_row[1].trim_start_matches("ld a, ").parse::<i8>()?}]});
    let _: crystal_core::battle::turn::MovePriorityTable = serde_json::from_value(value.clone())?;
    Ok(value)
}

pub(super) fn multipliers(stat: &str, accuracy: &str) -> Result<BattleStatMultiplierTables> {
    let parse = |source: &str| -> Result<Vec<Value>> {
        let mut rows = Vec::new();
        for line in source.lines().map(code_part) {
            let Some(raw) = line.strip_prefix("db ") else {
                continue;
            };
            let args = source_arguments(raw)?;
            ensure!(
                args.len() == 2,
                "stat multiplier requires exactly two authored bytes"
            );
            let byte = |value: &str| -> Result<u8> {
                Ok(if let Some(value) = value.strip_prefix('$') {
                    u8::from_str_radix(value, 16)?
                } else {
                    value.parse()?
                })
            };
            rows.push(json!({"numerator":byte(&args[0])?,"denominator":byte(&args[1])?}));
        }
        Ok(rows)
    };
    Ok(serde_json::from_value(
        json!({"stat":parse(stat)?,"accuracy":parse(accuracy)?}),
    )?)
}

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal/data/battle");
    let tables = multipliers(
        &std::fs::read_to_string(vendor.join("stat_multipliers.asm"))?,
        &std::fs::read_to_string(vendor.join("accuracy_multipliers.asm"))?,
    )?;
    let data = root.join("apps/web/assets/data");
    let directory = data.join("content-packs/core-modular/battle_stat_multipliers");
    std::fs::create_dir_all(&directory)?;
    let bytes = serde_json::to_vec_pretty(&tables)?;
    std::fs::write(data.join("battle_stat_multipliers.json"), &bytes)?;
    std::fs::write(directory.join("rules.json"), bytes)?;
    let source = root.join("vendor/pokecrystal");
    let read = |path: &str| std::fs::read_to_string(source.join(path));
    let priority = priorities(
        &read("constants/move_effect_constants.asm")?,
        &read("data/moves/effects_priorities.asm")?,
        &read("constants/battle_constants.asm")?,
        &read("engine/battle/core.asm")?,
    )?;
    let directory = data.join("content-packs/core-modular/move_priorities");
    std::fs::create_dir_all(&directory)?;
    let bytes = serde_json::to_vec_pretty(&priority)?;
    std::fs::write(data.join("move_priorities.json"), &bytes)?;
    std::fs::write(directory.join("rules.json"), bytes)?;
    let (categories, effectiveness) = type_tables(
        &read("constants/type_constants.asm")?,
        &read("data/types/type_matchups.asm")?,
        &read("constants/battle_constants.asm")?,
    )?;
    let weather = weather(
        &read("data/battle/weather_modifiers.asm")?,
        &read("constants/battle_constants.asm")?,
    )?;
    let wobble = wobble(&read("data/battle/wobble_probabilities.asm")?)?;
    for (name, value) in [
        ("type_categories", categories),
        ("type_effectiveness", effectiveness),
        ("weather_modifiers", weather),
        ("capture_wobble_probabilities", wobble),
    ] {
        let directory = data.join("content-packs/core-modular").join(name);
        std::fs::create_dir_all(&directory)?;
        let bytes = serde_json::to_vec_pretty(&value)?;
        std::fs::write(data.join(format!("{name}.json")), &bytes)?;
        std::fs::write(directory.join("rules.json"), bytes)?;
    }
    source_capture_rules::export(root)?;
    Ok(())
}
