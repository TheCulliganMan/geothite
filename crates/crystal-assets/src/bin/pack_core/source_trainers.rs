use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let data = root.join("apps/web/assets/data");
    let read = |path: &str| -> Result<String> { Ok(std::fs::read_to_string(vendor.join(path))?) };
    let constants: Value = serde_json::from_slice(&std::fs::read(data.join("story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let numeric = |value: &str| super::source_expressions::evaluate(value, &constants);
    let mut classes = Vec::<(String, Vec<String>)>::new();
    for line in read("constants/trainer_constants.asm")?.lines().map(code_part) {
        if let Some(class) = line.strip_prefix("trainerclass ") { classes.push((class.into(), Vec::new())); }
        else if let Some(id) = line.strip_prefix("const ") { classes.last_mut().context("trainer class before id")?.1.push(id.into()); }
    }
    ensure!(classes.first().is_some_and(|(id, _)| id == "TRAINER_NONE"), "source trainer class sentinel");
    classes.remove(0);
    let pointer_source = read("data/trainers/party_pointers.asm")?;
    let pointers = pointer_source.lines().filter_map(|line| code_part(line).strip_prefix("dw ")).collect::<Vec<_>>();
    ensure!(pointers.len() == classes.len(), "source trainer class pointers");
    let group_ids = pointers.into_iter().zip(0..classes.len()).collect::<BTreeMap<_, _>>();
    let mut attributes = Vec::<Vec<String>>::new();
    for line in read("data/trainers/attributes.asm")?.lines().map(code_part) {
        if let Some(row) = line.strip_prefix("db ").or_else(|| line.strip_prefix("dw ")) { attributes.push(source_arguments(row)?); }
    }
    ensure!(attributes.len() == classes.len() * 4, "source trainer attribute count");
    let dv_rows = read("data/trainers/dvs.asm")?.lines().filter_map(|line| code_part(line).strip_prefix("dn ")).map(source_arguments).collect::<Result<Vec<_>>>()?;
    let music_source = read("data/trainers/encounter_music.asm")?;
    let music = music_source.split("assert_table_length NUM_TRAINER_CLASSES + 1").next().context("source trainer music table")?.lines().filter_map(|line| code_part(line).strip_prefix("db ").map(str::to_owned)).collect::<Vec<_>>();
    let class_names = read("data/trainers/class_names.asm")?.lines().filter_map(|line| code_part(line).strip_prefix("li ").map(|value| value.trim_matches('"').trim_end_matches('@').to_owned())).collect::<Vec<_>>();
    ensure!(dv_rows.len() == classes.len() && music.len() == classes.len() + 1 && class_names.len() == classes.len(), "source trainer class presentation count: classes={}, DVs={}, music={}, names={}", classes.len(), dv_rows.len(), music.len(), class_names.len());
    let moves: BTreeMap<String, crystal_core::models::Move> = serde_json::from_slice(&std::fs::read(data.join("moves.json"))?)?;
    let learnsets: crystal_core::systems::learnsets::SpeciesLearnsets = serde_json::from_slice(&std::fs::read(data.join("learnsets.json"))?)?;
    let mut catalog = crystal_core::models::trainer::TrainerCatalog::default();
    let mut counts = vec![0usize; classes.len()];
    let mut group = None;
    let mut trainer = None::<Value>;
    let mut party = Vec::new();
    let mut mode = 0;
    for line in read("data/trainers/parties.asm")?.lines().map(code_part) {
        if let Some(label) = line.strip_suffix(':').filter(|label| *label != "Trainers") { group = Some(*group_ids.get(label).with_context(|| format!("source trainer group {label}"))?); }
        let Some(row) = line.strip_prefix("db ") else { continue };
        let row = source_arguments(row)?;
        let group = group.context("trainer group before rows")?;
        if row == &["-1"] {
            let mut value = trainer.take().context("trainer header before terminator")?;
            value["party"] = json!(party);
            party.clear();
            let trainer: crystal_core::models::trainer::Trainer = serde_json::from_value(value)?;
            catalog.insert(trainer).map_err(|error| anyhow::anyhow!("source trainer catalog: {error:?}"))?;
        } else if row[0].starts_with('"') {
            ensure!(row.len() == 2 && trainer.is_none(), "source trainer header");
            mode = numeric(&row[1])?;
            let (class, ids) = &classes[group];
            let id = ids.get(counts[group]).with_context(|| format!("source trainer identity {class}"))?;
            counts[group] += 1;
            let attr = &attributes[group * 4..group * 4 + 4];
            ensure!(attr[0].len() == 2 && attr[1].len() == 1 && attr[2].len() == 1 && attr[3].len() == 1, "source trainer class attribute operands");
            let items = attr[0].iter().map(|item| if item == "NO_ITEM" { None } else { Some(item.clone()) }).collect::<Vec<_>>();
            let layers = attr[2][0].split('|').map(str::trim).filter(|id| *id != "NO_AI").collect::<Vec<_>>();
            trainer = Some(json!({"name":row[0].trim_matches('"').trim_end_matches('@'),"trainer_id":id,"trainer_class":class,"party":[],"win_quote":"","lose_quote":"","items":items,"base_reward":numeric(&attr[1][0])?,"ai_move_flags":numeric(&attr[2][0])?,"ai_item_switch_flags":numeric(&attr[3][0])?,"encounter_music":music[group+1],"ai_layers":layers}));
        } else {
            let item_mode = mode & 2 != 0;
            let move_mode = mode & 1 != 0;
            ensure!(row.len() == 2 + usize::from(item_mode) + if move_mode {4} else {0}, "source trainer party operands {row:?}");
            let level = numeric(&row[0])?;
            let species = &row[1];
            let item = if item_mode && row[2] != "NO_ITEM" { Some(&row[2]) } else { None };
            let mut assigned = Vec::new();
            if move_mode { assigned.extend(row[2 + usize::from(item_mode)..].iter().filter(|id| *id != "NO_MOVE").cloned()); }
            else {
                for crystal_core::systems::learnsets::LearnsetEntry(learn_level, name) in learnsets.get(species).context("source trainer species learnset")? {
                    if i64::from(*learn_level) <= level && !assigned.contains(name) { assigned.push(name.clone()); if assigned.len() > 4 { assigned.remove(0); } }
                }
            }
            let learned = assigned.iter().map(|name| -> Result<Value> { Ok(json!({"name":name,"current_pp":moves.get(name).context("source trainer move")?.pp,"pp_ups":0})) }).collect::<Result<Vec<_>>>()?;
            let dv = &dv_rows[group];
            ensure!(dv.len() == 4, "source trainer class DVs");
            let dv = dv.iter().map(|value| numeric(value)).collect::<Result<Vec<_>>>()?;
            let hp = ((dv[0]&1)<<3)|((dv[1]&1)<<2)|((dv[2]&1)<<1)|(dv[3]&1);
            party.push(json!({"species":species,"level":level,"item":item,"moves":learned,"dvs":{"attack":dv[0],"defense":dv[1],"speed":dv[2],"special":dv[3],"hp":hp}}));
        }
    }
    ensure!(trainer.is_none(), "unterminated source trainer");
    for (index, (class, ids)) in classes.iter().enumerate() { ensure!(counts[index] == ids.len(), "source trainer count {class}"); }
    let directory = data.join("content-packs/core-modular/trainers");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), serde_json::to_vec_pretty(&catalog.trainers)?)?;
    std::fs::write(data.join("trainers.json"), serde_json::to_vec_pretty(&catalog)?)?;
    let names = classes.iter().zip(class_names).map(|((id, _), name)| (id.clone(), name)).collect::<BTreeMap<_, _>>();
    let directory = data.join("content-packs/core-modular/trainer_class_names");
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("Source.json"), serde_json::to_vec_pretty(&names)?)?;
    std::fs::write(data.join("trainer_class_names.json"), serde_json::to_vec_pretty(&names)?)?;
    Ok(())
}
