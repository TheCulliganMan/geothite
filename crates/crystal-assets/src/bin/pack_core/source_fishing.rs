use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let number = |value: &str| -> Result<i64> { super::source_expressions::evaluate(value, &constants) };
    let macros = std::fs::read_to_string(vendor.join("macros/data.asm"))?;
    ensure!(macros.lines().map(code_part).any(|line| line == "DEF percent EQUS \"* $ff / 100\""), "source percent macro");
    let threshold = |value: &str| -> Result<u8> { Ok(u8::try_from(number(&value.replace("percent", "* $ff / 100"))?)?) };
    let source = std::fs::read_to_string(vendor.join("data/wild/fish.asm"))?;
    let mut group_rows = Vec::new();
    let mut tables = BTreeMap::<String, Vec<Vec<String>>>::new();
    let mut current = None;
    let mut labels = Vec::new();
    for line in source.lines().map(code_part) {
        if let Some(args) = line.strip_prefix("fishgroup ") { group_rows.push(source_arguments(args)?); }
        if let Some(label) = line.strip_suffix(':') { tables.entry(label.into()).or_default(); current = Some(label.to_owned()); labels.push(label.to_owned()); }
        if let Some(args) = line.strip_prefix("db ") {
            if let Some(current) = &current { tables.get_mut(current).unwrap().push(source_arguments(args)?); }
        }
    }
    for index in (0..labels.len().saturating_sub(1)).rev() {
        if tables[&labels[index]].is_empty() {
            let rows = tables[&labels[index + 1]].clone();
            tables.insert(labels[index].clone(), rows);
        }
    }
    let ids = std::fs::read_to_string(vendor.join("constants/map_data_constants.asm"))?.lines().filter_map(|line| code_part(line).strip_prefix("const FISHGROUP_").filter(|id| *id != "NONE").map(|id| format!("FISHGROUP_{id}"))).collect::<Vec<_>>();
    ensure!(ids.len() == group_rows.len(), "authored fishing group count");
    let rods = ["OLD_ROD", "GOOD_ROD", "SUPER_ROD"];
    let mut groups = BTreeMap::new();
    for (id, row) in ids.into_iter().zip(group_rows) {
        ensure!(row.len() == 4, "source fishing group operands");
        let mut rod_tables = BTreeMap::new();
        for (rod, label) in rods.iter().zip(&row[1..]) {
            let mut slots = Vec::new();
            for row in tables.get(label).context("source fishing rod table")? {
                let value = if row.len() == 2 {
                    let index = row[1].strip_prefix("time_group ").context("source timed fishing operand")?;
                    json!({"threshold":threshold(&row[0])?,"species":null,"level":0,"time_group":index})
                } else {
                    ensure!(row.len() == 3, "source fishing slot operands");
                    json!({"threshold":threshold(&row[0])?,"species":row[1],"level":number(&row[2])?,"time_group":null})
                };
                slots.push(value);
            }
            rod_tables.insert(*rod, json!({"slots":slots}));
        }
        groups.insert(id.clone(), json!({"source_index":number(&id)?,"bite_threshold":threshold(&row[0])?,"rod_tables":rod_tables}));
    }
    let mut time_groups = BTreeMap::new();
    for (index, row) in tables.get("TimeFishGroups").context("source fishing time table")?.iter().enumerate() {
        ensure!(row.len() == 4, "source fishing time operands");
        time_groups.insert(index.to_string(), json!({"day_species":row[0],"day_level":number(&row[1])?,"night_species":row[2],"night_level":number(&row[3])?}));
    }
    let engine = std::fs::read_to_string(vendor.join("engine/events/fish.asm"))?;
    ensure!(engine.contains("bit DAILYFLAGS1_FISH_SWARM_F, [hl]"), "source fishing swarm gate");
    let mut swarm_rules = BTreeMap::new();
    for species in ["QWILFISH", "REMORAID"] {
        ensure!(engine.contains(&format!("cp FISHSWARM_{species}")) && engine.contains(&format!("ld d, FISHGROUP_{species}_SWARM")), "authored fishing swarm routing {species}");
        swarm_rules.insert(format!("FISHSWARM_{species}"), json!({"daily_flag_bit":number("DAILYFLAGS1_FISH_SWARM_F")?,"swarm":number(&format!("FISHSWARM_{species}"))?,"base_group":format!("FISHGROUP_{species}"),"swarm_group":format!("FISHGROUP_{species}_SWARM")}));
    }
    let rod_items = rods.iter().map(|rod| (*rod, *rod)).collect::<BTreeMap<_, _>>();
    let value = json!({"groups":groups,"time_groups":time_groups,"swarm_rules":swarm_rules,"rod_items":rod_items});
    let _: crystal_core::world::fishing::FishingCatalog = serde_json::from_value(value.clone())?;
    let directory = data.join("content-packs/core-modular/fishing");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), serde_json::to_vec_pretty(&value)?)?;
    std::fs::write(data.join("fishing.json"), serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}
