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
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let number = |value: &str| -> Result<i64> { super::source_expressions::evaluate(value, &constants) };
    let mut lengths = Vec::new();
    for line in read("data/events/magikarp_lengths.asm")?.lines().map(code_part) {
        if let Some(arguments) = line.strip_prefix("dwb ") {
            let row = source_arguments(arguments)?;
            ensure!(row.len() == 2, "source Magikarp length operands");
            lengths.push(json!({"threshold":number(&row[0])?,"divisor":number(&row[1])?}));
        }
    }
    let _: crystal_core::systems::special_routines::MagikarpLengthTable = serde_json::from_value(json!(lengths))?;
    write(&data, "magikarp_lengths", &json!(lengths))?;
    let mut dratini = BTreeMap::<u8, Vec<String>>::new();
    let mut mode = None;
    for line in read("engine/events/dratini.asm")?.lines().map(code_part) {
        if let Some(id) = line.strip_prefix(".Moveset").and_then(|id| id.strip_suffix(':')).filter(|id| id.bytes().all(|byte| byte.is_ascii_digit())) { mode = Some(id.parse::<u8>()?); }
        if let Some(value) = line.strip_prefix("db ") {
            let mode = mode.context("source Dratini moveset mode")?;
            if value == "0" { continue; }
            dratini.entry(mode).or_default().extend(source_arguments(value)?);
        }
    }
    let _: crystal_core::systems::special_routines::DratiniMoveSetTable = serde_json::from_value(json!(dratini))?;
    write(&data, "dratini_move_sets", &json!(dratini))?;
    let mut changes = BTreeMap::new();
    let codes = constants.iter().filter(|(id, _)| id.starts_with("HAPPINESS_")).map(|(id, index)| (*index, id.clone())).collect::<BTreeMap<_, _>>();
    for (index, row) in read("data/events/happiness_changes.asm")?.lines().filter_map(|line| code_part(line).strip_prefix("db ")).map(source_arguments).collect::<Result<Vec<_>>>()?.into_iter().enumerate() {
        ensure!(row.len() == 3, "source happiness change operands");
        let index = index as i64 + 1;
        changes.insert(index.to_string(), json!({"code":codes.get(&index).context("source happiness code")?,"low":number(&row[0])?,"mid":number(&row[1])?,"high":number(&row[2])?}));
    }
    let mut services = BTreeMap::<String, Vec<Value>>::new();
    let mut routine = None;
    let engine = read("engine/events/haircut.asm")?;
    for line in read("data/events/happiness_probabilities.asm")?.lines().map(code_part) {
        if let Some(name) = line.strip_prefix("HappinessData_").and_then(|name| name.strip_suffix(':')) {
            ensure!(engine.contains(&format!("{name}:")) && engine.contains(&format!("ld hl, HappinessData_{name}")), "source happiness service {name}");
            routine = Some(name.to_owned());
        }
        if let Some(arguments) = line.strip_prefix("db ") {
            let row = source_arguments(arguments)?;
            ensure!(row.len() == 3, "source happiness outcome operands");
            let weight = number(&row[0].replace("percent", "* $ff / 100"))?;
            ensure!((-1..=255).contains(&weight), "source happiness probability byte");
            services.entry(routine.clone().context("source happiness service label")?).or_default().push(json!({"rollWeight":weight as u8,"scriptValue":number(&row[1])?,"changeCode":number(&row[2])?}));
        }
    }
    let value = json!({"changes":changes,"services":services});
    let _: crystal_core::systems::special_routines::HappinessData = serde_json::from_value(value.clone())?;
    write(&data, "happiness_data", &value)?;
    let mut ratings = Vec::new();
    for line in read("data/events/pokedex_ratings.asm")?.lines().map(code_part) {
        if let Some(arguments) = line.strip_prefix("rating ") {
            let row = source_arguments(arguments)?;
            ensure!(row.len() == 3, "source Oak rating operands");
            ratings.push(json!({"caughtCountLimit":number(&row[0])?,"fanfare":row[1],"textLabel":row[2]}));
        }
    }
    let _: crystal_core::systems::special_routines::OakRatingTable = serde_json::from_value(json!(ratings))?;
    write(&data, "oak_ratings", &json!(ratings))?;
    let shuckle = read("engine/events/shuckle.asm")?;
    let lines = shuckle.lines().map(code_part).collect::<Vec<_>>();
    let register_binding = |target: &str| -> Result<String> {
        let index = lines.iter().position(|line| *line == format!("ld [{target}], a")).context("source gift register target")?;
        Ok(lines.get(index.wrapping_sub(1)).and_then(|line| line.strip_prefix("ld a, ")).context("source gift value binding")?.to_owned())
    };
    let string = |label: &str| -> Result<String> {
        let body = shuckle.split(&format!("{label}:" )).nth(1).context("source gift text label")?;
        let value = body.lines().map(code_part).find_map(|line| line.strip_prefix("db ")).context("source gift text")?;
        ensure!(value.starts_with('"') && value.ends_with('"'), "source gift string");
        Ok(value.trim_matches('"').trim_end_matches('@').to_owned())
    };
    let ot_id = lines.iter().find_map(|line| line.strip_prefix("DEF MANIA_OT_ID EQU ")).context("source gift original trainer id")?;
    let held_item = lines.iter().find_map(|line| line.strip_prefix("ld [hl], ").filter(|value| !value.starts_with("LOW("))).context("source gift held item")?;
    ensure!(shuckle.contains("set DAILYFLAGS1_GOT_SHUCKIE_TODAY_F, [hl]") && constants.contains_key("ENGINE_GOT_SHUCKIE_TODAY"), "source gift daily flag");
    let value = json!({"species":register_binding("wCurPartySpecies")?,"level":number(&register_binding("wCurPartyLevel")?)?,"heldItem":held_item,"nickname":string("SpecialShuckleNickname")?,"originalTrainerName":string("SpecialShuckleOT")?,"originalTrainerId":number(ot_id)?,"gotTodayEngineFlag":"ENGINE_GOT_SHUCKIE_TODAY"});
    let _: crystal_core::systems::special_routines::ShuckieGiftDefinition = serde_json::from_value(value.clone())?;
    write(&data, "shuckie_gift", &value)?;
    let flag_source = read("data/events/bug_contest_flags.asm")?;
    let flags = flag_source.lines().filter_map(|line| code_part(line).strip_prefix("dw ").map(str::to_owned)).collect::<Vec<_>>();
    ensure!(flags.len() as i64 == number("NUM_BUG_CONTESTANTS")?, "source bug contest contestant flags");
    let selection = read("engine/events/bug_contest/contest_2.asm")?;
    let selected = selection.lines().map(code_part).filter_map(|line| line.strip_prefix("ld c, ")).find(|value| value.bytes().all(|byte| byte.is_ascii_digit())).context("source contest participant selection")?;
    let mut encounters = Vec::new();
    for line in read("data/wild/bug_contest_mons.asm")?.lines().map(code_part) {
        if let Some(arguments) = line.strip_prefix("db ") {
            let row = source_arguments(arguments)?;
            ensure!(row.len() == 4, "source contest encounter operands");
            let weight = number(&row[0])?;
            ensure!((-1..=255).contains(&weight), "source contest weight byte");
            encounters.push(json!({"weight":weight as u8,"species":row[1],"minLevel":number(&row[2])?,"maxLevel":number(&row[3])?}));
        }
    }
    let value = json!({"parkBalls":number("BUG_CONTEST_BALLS")?,"timerMinutes":number("BUG_CONTEST_MINUTES")?,"timerSeconds":number("BUG_CONTEST_SECONDS")?,"selectedContestantCount":number(selected)?,"contestantFlags":flags,"encounters":encounters});
    let _: crystal_core::systems::special_routines::BugContestConfig = serde_json::from_value(value.clone())?;
    write(&data, "bug_contest_config", &value)
}
