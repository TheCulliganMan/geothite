//! Translate pinned ball routines, retaining the cartridge's known quirks.
use super::super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn heavy_modifier(weight: u16, thresholds: &[(u8, i16)], light: i16) -> Result<i16> {
    // HeavyBallMultiplier divides pounds by two, then subtracts 1/16 and
    // 1/32 of that intermediate value before comparing only its high byte.
    let mut converted = weight >> 1;
    let fraction = converted >> 4;
    converted = converted.wrapping_sub(fraction).wrapping_sub(fraction >> 1);
    let high = (converted >> 8) as u8;
    if high < 4 {
        return Ok(-light);
    }
    thresholds
        .iter()
        .find(|(bound, _)| high < *bound)
        .map(|(_, boost)| *boost)
        .context("heavy weight table does not cover source weight")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heavy_ball_uses_truncated_cartridge_conversion_and_high_byte() {
        let thresholds = [(8, 0), (12, 20), (16, 30), (255, 40)];
        assert_eq!(heavy_modifier(2000, &thresholds, 20).unwrap(), -20);
        assert_eq!(heavy_modifier(2257, &thresholds, 20).unwrap(), -20);
        assert_eq!(heavy_modifier(2258, &thresholds, 20).unwrap(), 0);
        assert_eq!(heavy_modifier(8820, &thresholds, 20).unwrap(), 30);
        assert_eq!(heavy_modifier(10140, &thresholds, 20).unwrap(), 40);
        for (species, weight) in [("KADABRA", 1250), ("TAUROS", 1950), ("SUNFLORA", 190)] {
            assert_eq!(
                heavy_modifier(weight, &thresholds, 20).unwrap(),
                -20,
                "{species} must use its own authored weight"
            );
        }
        assert!(heavy_modifier(10140, &[(8, 0)], 20).is_err());
    }
}

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let source = std::fs::read_to_string(vendor.join("engine/items/item_effects.asm"))?;
    let code: Vec<_> = source
        .lines()
        .map(code_part)
        .filter(|s| !s.is_empty())
        .collect();
    let body = |start: &str, end: &str| -> Result<Vec<&str>> {
        let start = code
            .iter()
            .position(|s| *s == start)
            .with_context(|| format!("missing {start}"))?;
        let finish = code
            .iter()
            .skip(start + 1)
            .position(|s| *s == end)
            .with_context(|| format!("missing {end}"))?
            + start
            + 1;
        Ok(code[start + 1..finish].to_vec())
    };
    let require = |body: &[&str], instruction: &str| -> Result<()> {
        ensure!(
            body.contains(&instruction),
            "unsupported capture routine: missing {instruction}"
        );
        Ok(())
    };
    let ultra = body("UltraBallMultiplier:", "SafariBallMultiplier:")?;
    require(&ultra, "sla b")?;
    let great = body("ParkBallMultiplier:", "HeavyBall_GetDexEntryBank:")?;
    for instruction in ["ld a, b", "srl a", "add b"] {
        require(&great, instruction)?;
    }
    let lure = body("LureBallMultiplier:", "MoonBallMultiplier:")?;
    for instruction in ["cp BATTLETYPE_FISH", "ret nz", "add a", "add b"] {
        require(&lure, instruction)?;
    }
    let moon = body("MoonBallMultiplier:", "LoveBallMultiplier:")?;
    require(&moon, "cp MOON_STONE_RED")?; // Source bug compares the Gen I item ID.
    let love = body("LoveBallMultiplier:", "FastBallMultiplier:")?;
    for instruction in ["ld a, [wTempEnemyMonSpecies]", "cp c", "cp d", "ret nz"] {
        require(&love, instruction)?;
    }
    let fast = body("FastBallMultiplier:", "LevelBallMultiplier:")?;
    require(&fast, "ld d, 3")?;
    let level = body("LevelBallMultiplier:", "BallDodgedText:")?;
    ensure!(
        level.iter().filter(|s| **s == "sla b").count() == 3
            && level.iter().filter(|s| **s == "srl c").count() == 2,
        "unsupported level multiplier"
    );
    let rule = |numerator, denominator| json!({"multiplier_numerator":numerator,"multiplier_denominator":denominator,"battle_type":"","skip_hp_calc":false,"use_heavy_ball_weight_modifier":false,"use_level_ball_multiplier":false,"require_same_species":false,"require_same_gender":false,"require_fast_species":false});
    let mut balls = BTreeMap::new();
    let table = body("BallMultiplierFunctionTable:", "UltraBallMultiplier:")?;
    for line in table {
        let Some(raw) = line.strip_prefix("dbw ") else {
            continue;
        };
        let args = source_arguments(raw)?;
        ensure!(args.len() == 2, "invalid ball routine row");
        let mut value = match args[1].as_str() {
            "UltraBallMultiplier" => rule(2, 1),
            "GreatBallMultiplier" | "SafariBallMultiplier" | "ParkBallMultiplier" => rule(3, 2),
            "HeavyBallMultiplier" | "LevelBallMultiplier" | "MoonBallMultiplier" => rule(1, 1),
            "LureBallMultiplier" => rule(3, 1),
            "LoveBallMultiplier" => {
                ensure!(
                    love.iter().filter(|s| **s == "sla b").count() == 3,
                    "unsupported love multiplier"
                );
                rule(8, 1)
            }
            "FastBallMultiplier" => {
                ensure!(
                    fast.iter().filter(|s| **s == "sla b").count() == 2,
                    "unsupported fast multiplier"
                );
                rule(4, 1)
            }
            other => anyhow::bail!("unsupported source ball routine {other}"),
        };
        match args[1].as_str() {
            "HeavyBallMultiplier" => value["use_heavy_ball_weight_modifier"] = json!(true),
            "LevelBallMultiplier" => {
                value["use_level_ball_multiplier"] = json!(true);
                value["skip_hp_calc"] = json!(true);
            }
            "LureBallMultiplier" => value["battle_type"] = json!("BATTLETYPE_FISH"),
            "LoveBallMultiplier" => {
                value["require_same_species"] = json!(true);
                value["require_same_gender"] = json!(true);
            }
            "FastBallMultiplier" => value["require_fast_species"] = json!(true),
            _ => {}
        }
        ensure!(
            balls.insert(args[0].clone(), value).is_none(),
            "duplicate ball rule"
        );
    }
    let data = root.join("apps/web/assets/data");
    let items: Vec<Value> = serde_json::from_slice(&std::fs::read(data.join("items.json"))?)?;
    for item in items {
        if item["battle_capture_ball"] == true {
            ensure!(
                item["effect"] == "PokeBallEffect",
                "unsupported capture item effect"
            );
            let id = item["script_name"].as_str().context("ball item identity")?;
            balls.entry(id.to_string()).or_insert_with(|| rule(1, 1));
        }
    }
    ensure!(
        ["POKE_BALL", "FRIEND_BALL", "MASTER_BALL"]
            .iter()
            .all(|id| balls.contains_key(*id)),
        "missing source neutral ball items"
    );
    let species: BTreeMap<String, Value> =
        serde_json::from_slice(&std::fs::read(data.join("pokemon.json"))?)?;
    let heavy_body = body("HeavyBallMultiplier:", "LureBallMultiplier:")?;
    for instruction in ["cp HIGH(1024)", "srl h", "rept 4", "srl b", "sub 20"] {
        require(&heavy_body, instruction)?;
    }
    let mut thresholds = Vec::new();
    let weights = heavy_body
        .iter()
        .position(|line| *line == ".WeightsTable:")
        .context("missing heavy thresholds")?;
    for line in &heavy_body[weights + 1..] {
        let Some(raw) = line.strip_prefix("db ") else {
            continue;
        };
        let args = source_arguments(raw)?;
        ensure!(args.len() == 2, "invalid heavy threshold row");
        let limit: u16 = args[0]
            .strip_prefix("HIGH(")
            .and_then(|s| s.strip_suffix(')'))
            .context("source weight high byte")?
            .parse()?;
        thresholds.push(((limit >> 8) as u8, args[1].parse::<i16>()?));
    }
    ensure!(
        thresholds.len() == 4 && thresholds.windows(2).all(|w| w[0].0 < w[1].0),
        "invalid heavy threshold ordering"
    );
    let mut heavy = BTreeMap::new();
    for (id, entry) in species {
        // Use this species' authored dex entry. This deliberately fixes the
        // cartridge's wrong-bank lookup for Kadabra, Tauros and Sunflora.
        let dex = std::fs::read_to_string(vendor.join(format!(
            "data/pokemon/dex_entries/{}.asm",
            id.to_lowercase()
        )))?;
        let rows: Vec<_> = dex
            .lines()
            .map(code_part)
            .filter_map(|s| s.strip_prefix("dw "))
            .collect();
        ensure!(rows.len() == 1, "missing authored dex weight for {id}");
        let args = source_arguments(rows[0])?;
        ensure!(args.len() == 2, "invalid authored dex measures for {id}");
        let weight: u16 = args[1].parse()?;
        ensure!(
            entry["weight"].as_u64() == Some(u64::from(weight)),
            "species/dex weight mismatch for {id}"
        );
        heavy.insert(id, heavy_modifier(weight, &thresholds, 20)?);
    }
    let flee = std::fs::read_to_string(vendor.join("data/wild/flee_mons.asm"))?;
    let fast_species: Vec<_> = flee
        .lines()
        .map(code_part)
        .filter_map(|s| s.strip_prefix("db "))
        .take(3)
        .collect();
    ensure!(
        fast_species.len() == 3 && fast_species.iter().all(|id| *id != "-1"),
        "incomplete fast-ball source"
    );
    ensure!(
        source.contains("and 1 << FRZ | SLP_MASK")
            && source.contains("ld c, 10")
            && source.contains("cp MASTER_BALL"),
        "unsupported capture status/guarantee"
    );
    let value = json!({"fast_ball_species":fast_species,"heavy_ball_modifiers":heavy,"ball_rules":balls,"guaranteed_capture_balls":["MASTER_BALL"],"status_bonus":{"SLEEP":10,"FREEZE":10}});
    let _: crystal_core::battle::capture::CaptureRules = serde_json::from_value(value.clone())?;
    let directory = data.join("content-packs/core-modular/capture_rules");
    std::fs::create_dir_all(&directory)?;
    let bytes = serde_json::to_vec_pretty(&value)?;
    std::fs::write(data.join("capture_rules.json"), &bytes)?;
    std::fs::write(directory.join("rules.json"), bytes)?;
    Ok(())
}
