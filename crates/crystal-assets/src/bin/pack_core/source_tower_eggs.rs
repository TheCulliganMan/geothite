//! Structured authored Battle Tower parties and Odd Egg records.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn records(source: &str) -> Result<Vec<Value>> {
    let mut rows: Vec<(&str, Vec<String>)> = Vec::new();
    let mut records = Vec::new();
    for line in source.lines().map(code_part) {
        let Some((command, raw)) = line.split_once(' ') else {
            continue;
        };
        if !matches!(command, "db" | "dw" | "bigdt" | "bigdw" | "dn" | "dname") {
            continue;
        }
        rows.push((command, source_arguments(raw)?));
        if command != "dname" {
            continue;
        }
        ensure!(
            rows.len() == 24,
            "invalid nicknamed mon record length {}",
            rows.len()
        );
        let expected = [
            "db", "db", "db", "dw", "bigdt", "bigdw", "bigdw", "bigdw", "bigdw", "bigdw", "dn",
            "db", "db", "db", "db", "db", "bigdw", "bigdw", "bigdw", "bigdw", "bigdw", "bigdw",
            "bigdw", "dname",
        ];
        ensure!(
            rows.iter()
                .zip(expected)
                .all(|(row, opcode)| row.0 == opcode),
            "invalid mon record layout"
        );
        let numbers = |index: usize| -> Result<Vec<u32>> {
            rows[index]
                .1
                .iter()
                .map(|n| n.parse().map_err(Into::into))
                .collect()
        };
        let scalar = |index: usize| -> Result<u32> {
            let values = numbers(index)?;
            ensure!(values.len() == 1, "mon scalar field shape");
            Ok(values[0])
        };
        ensure!(
            rows[0].1.len() == 1
                && rows[1].1.len() == 1
                && rows[2].1.len() == 4
                && rows[23].1.len() == 2,
            "invalid mon token fields"
        );
        let stat_exp = (5..10).map(scalar).collect::<Result<Vec<_>>>()?;
        let stats = (16..23).map(scalar).collect::<Result<Vec<_>>>()?;
        records.push(json!({"species":rows[0].1[0],"item":if rows[1].1[0] == "NO_ITEM" {None} else {Some(&rows[1].1[0])},"moves":rows[2].1,"originalTrainerId":scalar(3)?,"experience":scalar(4)?,"statExp":stat_exp,"dvs":numbers(10)?,"pp":numbers(11)?,"happiness":scalar(12)?,"pokerus":numbers(13)?,"level":scalar(14)?,"status":numbers(15)?,"stats":stats,"nickname":rows[23].1[0].trim_matches('"').trim_end_matches('@')}));
        rows.clear();
    }
    ensure!(
        rows.is_empty() && !records.is_empty(),
        "incomplete authored mon records"
    );
    Ok(records)
}

pub(super) fn odd_eggs(source: &str, trainer_name: &str) -> Result<Value> {
    let (probabilities, mons) = source.split_once("OddEggs:").context("missing OddEggs")?;
    ensure!(
        probabilities.contains("dw odd_egg_prob_total * $ffff / 100"),
        "unsupported odd egg probability scale"
    );
    let probabilities = probabilities
        .lines()
        .map(code_part)
        .filter_map(|s| s.strip_prefix("odd_egg_prob "))
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mons = records(mons)?;
    ensure!(
        probabilities.len() == mons.len() && probabilities.iter().sum::<u32>() == 100,
        "odd egg probability count/total mismatch"
    );
    let mut definitions = Vec::new();
    for (mut mon, probability) in mons.into_iter().zip(probabilities) {
        let value = mon.as_object_mut().unwrap();
        for key in ["item", "statExp", "pp", "pokerus", "status", "stats"] {
            value.remove(key);
        }
        let cycles = value.remove("happiness").unwrap();
        value.insert("hatchCycles".into(), cycles);
        // The Rust sampler consumes weights totaling 100; the ASM table
        // encodes the same authored percentages as cumulative 16-bit cutoffs.
        value.insert("probability".into(), json!(probability));
        value.insert("originalTrainerName".into(), json!(trainer_name));
        value
            .get_mut("moves")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .retain(|id| id != "0");
        definitions.push(mon);
    }
    let value = json!(definitions);
    let _: crystal_core::systems::special_routines::OddEggDefinitionTable =
        serde_json::from_value(value.clone())?;
    Ok(value)
}

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let read = |path: &str| {
        std::fs::read_to_string(vendor.join(path))
            .with_context(|| format!("tower/egg source {path}"))
    };
    let data = root.join("apps/web/assets/data");
    let constants: Value = serde_json::from_slice(&std::fs::read(
        data.join("story_event_script_constants.json"),
    )?)?;
    let numeric = |id: &str| -> Result<u64> {
        constants["global"][id]
            .as_u64()
            .with_context(|| format!("tower constant {id}"))
    };
    let engine_egg = read("engine/events/odd_egg.asm")?;
    let ot = engine_egg
        .split(".Odd:")
        .nth(1)
        .context("odd egg OT label")?
        .lines()
        .map(code_part)
        .find_map(|s| s.strip_prefix("dname "))
        .context("odd egg OT name")?;
    let ot = source_arguments(ot)?;
    let odd = odd_eggs(&read("data/events/odd_eggs.asm")?, ot[0].trim_matches('"'))?;
    let mons = records(&read("data/battle_tower/parties.asm")?)?;
    let group_size = usize::try_from(numeric("BATTLETOWER_NUM_UNIQUE_MON")?)?;
    ensure!(mons.len() % group_size == 0, "incomplete tower groups");
    let groups = mons
        .chunks(group_size)
        .map(|mons| mons.to_vec())
        .collect::<Vec<_>>();
    for (index, group) in groups.iter().enumerate() {
        ensure!(
            group.iter().all(|mon| mon["level"] == (index + 1) * 10),
            "tower authored level group mismatch"
        );
    }
    let classes_source = read("constants/trainer_constants.asm")?;
    let mut classes: Vec<_> = classes_source
        .lines()
        .map(code_part)
        .filter_map(|s| s.strip_prefix("trainerclass "))
        .filter(|s| *s != "TRAINER_NONE")
        .collect();
    let sprites_source = read("data/trainers/sprites.asm")?;
    let sprites: Vec<_> = sprites_source
        .lines()
        .map(code_part)
        .filter_map(|s| s.strip_prefix("db "))
        .collect();
    let genders_source = read("data/trainers/genders.asm")?;
    let genders: Vec<_> = genders_source
        .lines()
        .map(code_part)
        .filter_map(|s| s.strip_prefix("db "))
        .collect();
    ensure!(sprites_source.contains("assert_table_length NUM_TRAINER_CLASSES - 1") && genders_source.contains("assert_table_length NUM_TRAINER_CLASSES - 1") && classes.last() == Some(&"MYSTICALMAN"), "source tower presentation exclusion");
    classes.pop();
    ensure!(
        classes.len() == sprites.len() && classes.len() == genders.len(),
        "tower trainer presentation count"
    );
    let class_presentation: BTreeMap<_, _> = classes
        .into_iter()
        .zip(sprites.into_iter().zip(genders))
        .collect();
    let mut trainers = Vec::new();
    for line in read("data/battle_tower/classes.asm")?
        .lines()
        .map(code_part)
    {
        let Some(raw) = line.strip_prefix("bt_trainer ") else {
            continue;
        };
        let args = source_arguments(raw)?;
        ensure!(args.len() == 2, "invalid tower trainer row");
        let (sprite, gender) = class_presentation
            .get(args[0].as_str())
            .context("tower trainer class")?;
        ensure!(matches!(*gender, "MALE" | "FEMALE"), "tower trainer gender");
        trainers.push(json!({"index":trainers.len(),"trainerClass":args[0],"name":args[1].trim_matches('"'),"spriteConstant":sprite,"female":*gender == "FEMALE"}));
    }
    ensure!(
        trainers.len() == numeric("BATTLETOWER_NUM_UNIQUE_TRAINERS")? as usize,
        "tower trainer roster count"
    );
    let mobile = read("mobile/mobile_46.asm")?;
    let ubers = mobile
        .split("BattleTower_UbersCheck:")
        .nth(1)
        .context("tower species restriction routine")?
        .split(".uber_under_70")
        .next()
        .unwrap();
    ensure!(
        ubers.contains("cp MEWTWO")
            && ubers.contains("cp MEW")
            && ubers.contains("cp LUGIA")
            && ubers.contains("cp NUM_POKEMON + 1")
            && ubers.contains("cp 70 / 10"),
        "unsupported tower species restrictions"
    );
    let species: BTreeMap<String, Value> =
        serde_json::from_slice(&std::fs::read(data.join("pokemon.json"))?)?;
    let mut banned = BTreeMap::new();
    for (id, mon) in species {
        let index = mon["int_id"].as_u64().context("tower species index")?;
        if id == "MEWTWO"
            || id == "MEW"
            || (numeric("LUGIA")?..=numeric("NUM_POKEMON")?).contains(&index)
        {
            banned.insert(id, json!({}));
        }
    }
    let engine = read("engine/events/battle_tower/battle_tower.asm")?;
    let reward = engine
        .split("BattleTower_RandomlyChooseReward:")
        .nth(1)
        .context("tower reward routine")?
        .split("BattleTowerAction_CheckExplanationRead:")
        .next()
        .unwrap();
    ensure!(
        reward.contains("cp LUCKY_PUNCH") && reward.contains("jr z, .loop"),
        "unsupported tower reward exclusion"
    );
    let items: Vec<Value> = serde_json::from_slice(&std::fs::read(data.join("items.json"))?)?;
    let mut reward_values = BTreeMap::new();
    let mut candidates = Vec::new();
    for item in items {
        let id = item["script_name"].as_str().context("source item ID")?;
        let value = numeric(id)?;
        if (numeric("BATTLETOWER_MIN_REWARD")?..=numeric("BATTLETOWER_MAX_REWARD")?)
            .contains(&value)
        {
            candidates.push((value, id.to_string()));
            reward_values.insert(id.to_string(), value);
        }
    }
    candidates.sort();
    reward_values.insert("POTION".into(), numeric("POTION")?);
    let rules_source = read("engine/events/battle_tower/rules.asm")?;
    let texts: Vec<_> = rules_source
        .split("_CheckForBattleTowerRules:")
        .nth(1)
        .context("tower party rules")?
        .split(".TextPointers:")
        .nth(1)
        .context("tower party failure texts")?
        .lines()
        .map(code_part)
        .filter_map(|s| s.strip_prefix("dw "))
        .take(5)
        .collect();
    ensure!(
        texts.len() == 5
            && engine.contains("ld a, POTION")
            && mobile.contains("ld c, 10")
            && mobile.contains("Strings_L10ToL100"),
        "unsupported tower rules"
    );
    let rules = json!({"bannedSpecies":banned,"requiredPartyCount":numeric("BATTLETOWER_PARTY_LENGTH")?,"challengeStreakLength":numeric("BATTLETOWER_STREAK_LENGTH")?,"rewardCandidates":candidates.into_iter().map(|(_,id)|id).collect::<Vec<_>>(),"excludedRewardItems":["LUCKY_PUNCH"],"rewardQuantity":numeric("BATTLETOWER_REWARD_QUANTITY")?,"rewardFailureSentinel":"POTION","rewardItemValues":reward_values,"minimumLevelGroup":1,"maximumLevelGroup":groups.len(),"levelGroupSize":10,"partyCountFailureText":texts[1],"duplicateSpeciesFailureText":texts[2],"duplicateHeldItemFailureText":texts[3],"eggFailureText":texts[4],"trainers":trainers,"monGroups":groups});
    let _: crystal_core::systems::special_routines::BattleTowerRules =
        serde_json::from_value(rules.clone())?;
    for (name, value) in [("odd_egg_definitions", odd), ("battle_tower_rules", rules)] {
        let directory = data.join("content-packs/core-modular").join(name);
        std::fs::create_dir_all(&directory)?;
        let bytes = serde_json::to_vec_pretty(&value)?;
        std::fs::write(data.join(format!("{name}.json")), &bytes)?;
        std::fs::write(directory.join("rules.json"), bytes)?;
    }
    Ok(())
}
