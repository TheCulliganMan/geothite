//! Authored animation metadata and sprite presentation tables.
use super::source_syntax::{code_part, source_arguments};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn frame_attr_flags(indices: &[u8]) -> Result<u8> {
    indices.iter().try_fold(0, |flags, index| {
        ensure!((4..=6).contains(index), "unsupported sprite frame flag bit");
        Ok(flags | (1 << (index + 1)))
    })
}

pub(super) fn battle(source: &str, move_count: usize) -> Result<(Value, Value)> {
    let mut table = Vec::new();
    let mut table_done = false;
    let mut labels = Vec::new();
    let mut bodies: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut commands = Vec::new();
    for line in source.lines().map(code_part).filter(|s| !s.is_empty()) {
        if !table_done {
            if let Some(label) = line.strip_prefix("dw ") {
                table.push(label.to_string());
            }
            if line == "assert_table_length NUM_ATTACKS + 1" {
                ensure!(
                    table.len() == move_count + 1,
                    "authored move animation count mismatch"
                );
                table_done = true;
            }
        }
        if line.ends_with(':') && !line.starts_with('.') && !line.starts_with("BattleAnimations:") {
            if !commands.is_empty() {
                bodies.push((std::mem::take(&mut labels), std::mem::take(&mut commands)));
            }
            labels.push(line.trim_end_matches(':').to_string());
        } else if !labels.is_empty() {
            ensure!(
                line.starts_with("anim_")
                    || (line.starts_with('.') && !line.contains(char::is_whitespace)),
                "unsupported battle animation directive {line}"
            );
            commands.push(if line.starts_with('.') && !line.ends_with(':') {
                format!("{line}:")
            } else {
                line.to_string()
            });
        }
    }
    if !commands.is_empty() {
        bodies.push((labels, commands));
    }
    ensure!(
        table_done && !bodies.is_empty(),
        "missing battle animation source"
    );
    let mut animations = BTreeMap::new();
    for (aliases, commands) in bodies {
        for label in aliases {
            ensure!(
                animations.insert(label, commands.clone()).is_none(),
                "duplicate animation label"
            );
        }
    }
    let _: crystal_core::models::BattleAnimationCommandTable =
        serde_json::from_value(json!(animations))?;
    let issues =
        crystal_core::models::battle_animation_catalog_issues(&animations, &table, move_count);
    ensure!(
        issues.is_empty(),
        "invalid authored animation catalog: {issues:?}"
    );
    Ok((json!(animations), json!(table)))
}

pub(super) fn export(root: &Path) -> Result<()> {
    let vendor = root.join("vendor/pokecrystal");
    let read = |path: &str| std::fs::read_to_string(vendor.join(path));
    let data = root.join("apps/web/assets/data");
    let moves: Value = serde_json::from_slice(&std::fs::read(data.join("moves.json"))?)?;
    let move_count = moves.as_object().context("authored move map")?.len();
    let (animations, table) = battle(&read("data/moves/animations.asm")?, move_count)?;
    let constants: Value = serde_json::from_slice(&std::fs::read(
        data.join("story_event_script_constants.json"),
    )?)?;
    let number = |token: &str| -> Result<i16> {
        if let Some(value) = token.strip_prefix('$') {
            return Ok(i16::from_str_radix(value, 16)?);
        }
        token
            .parse()
            .ok()
            .or_else(|| {
                constants["global"][token]
                    .as_i64()
                    .and_then(|n| i16::try_from(n).ok())
            })
            .with_context(|| format!("unknown sprite constant {token}"))
    };
    let flags = |source: &str| -> Result<i16> {
        source
            .split('|')
            .try_fold(0, |flags, token| Ok(flags | number(token.trim())?))
    };
    let constants_source = read("constants/sprite_anim_constants.asm")?;
    let names = |prefix: &str| -> Vec<String> {
        constants_source
            .lines()
            .map(code_part)
            .filter_map(|line| {
                line.strip_prefix("const ")
                    .filter(|name| name.starts_with(prefix))
                    .map(str::to_string)
            })
            .collect()
    };
    let mut object_rows = Vec::new();
    for line in read("data/sprite_anims/objects.asm")?
        .lines()
        .map(code_part)
    {
        if let Some(raw) = line.strip_prefix("db ") {
            let args = source_arguments(raw)?;
            ensure!(args.len() == 3, "invalid sprite object");
            object_rows.push(args);
        }
    }
    let object_names = names("SPRITE_ANIM_OBJ_");
    ensure!(
        object_names.len() == object_rows.len(),
        "sprite object count mismatch"
    );
    let objects: BTreeMap<_, _> = object_names
        .into_iter()
        .zip(object_rows)
        .map(|(name, args)| {
            (
                name.clone(),
                json!({"name":name,"frameset":args[0],"function":args[1],"dictionary":args[2]}),
            )
        })
        .collect();
    let oam_source = read("data/sprite_anims/oam.asm")?;
    let mut pointers = Vec::new();
    let mut blocks: BTreeMap<String, (usize, Vec<Value>)> = BTreeMap::new();
    let mut label = None;
    for raw in oam_source.lines() {
        let line = code_part(raw);
        if let Some(args) = line.strip_prefix("spriteanimoam ") {
            let args = source_arguments(args)?;
            ensure!(args.len() == 2, "invalid sprite OAM pointer");
            pointers.push((number(&args[0])?, args[1].clone()));
        } else if line.starts_with('.') && line.ends_with(':') {
            let name = line.trim_end_matches(':').to_string();
            ensure!(
                blocks.insert(name.clone(), (0, Vec::new())).is_none(),
                "duplicate OAM block"
            );
            label = Some(name);
        } else if let Some(name) = &label {
            if let Some(count) = line.strip_prefix("db ") {
                blocks.get_mut(name).unwrap().0 = count.parse()?;
            } else if let Some(args) = line.strip_prefix("dbsprite ") {
                let args = source_arguments(args)?;
                ensure!(args.len() == 6, "invalid sprite OAM piece");
                let x = (number(&args[0])? * 8 + number(&args[2])?) as u8 as i8;
                let y = (number(&args[1])? * 8 + number(&args[3])?) as u8 as i8;
                blocks.get_mut(name).unwrap().1.push(
                    json!({"x":x,"y":y,"tile":number(&args[4])?,"attributes":flags(&args[5])?}),
                );
            }
        }
    }
    let oam_names = names("SPRITE_ANIM_OAMSET_");
    ensure!(
        oam_names.len() == pointers.len(),
        "sprite OAM pointer count mismatch"
    );
    let mut oam_sets = BTreeMap::new();
    for (name, (offset, target)) in oam_names.into_iter().zip(pointers) {
        let (count, pieces) = blocks
            .get(&target)
            .with_context(|| format!("missing OAM target {target}"))?;
        ensure!(
            *count == pieces.len() && *count > 0,
            "OAM piece count mismatch {target}"
        );
        oam_sets.insert(
            name.clone(),
            json!({"name":name,"tile_offset":offset,"pieces":pieces}),
        );
    }
    let mut pointers = Vec::new();
    let mut blocks: Vec<(String, Vec<Value>)> = Vec::new();
    for line in read("data/sprite_anims/framesets.asm")?
        .lines()
        .map(code_part)
    {
        if let Some(target) = line.strip_prefix("dw ") {
            pointers.push(target.to_string());
        } else if line.starts_with('.') && line.ends_with(':') {
            blocks.push((line.trim_end_matches(':').to_string(), Vec::new()));
        } else if line.starts_with("oam") {
            let (command, raw) = line.split_once(' ').unwrap_or((line, ""));
            let step = match command {
                "oamframe" => {
                    let args = source_arguments(raw)?;
                    ensure!(
                        (2..=4).contains(&args.len()) && oam_sets.contains_key(&args[0]),
                        "invalid sprite frame"
                    );
                    let bits = args
                        .iter()
                        .skip(2)
                        .map(|token| Ok(u8::try_from(number(token)?)?))
                        .collect::<Result<Vec<_>>>()?;
                    json!({"command":"frame","oam_set":args[0],"duration":number(&args[1])?,"attr_flags":frame_attr_flags(&bits)?})
                }
                "oamwait" => {
                    json!({"command":"wait","oam_set":null,"duration":number(raw)?,"attr_flags":0})
                }
                "oamend" | "oamrestart" | "oamdelete" => {
                    json!({"command":command.trim_start_matches("oam"),"oam_set":null,"duration":0,"attr_flags":0})
                }
                _ => anyhow::bail!("unknown sprite frame command {command}"),
            };
            blocks
                .last_mut()
                .context("sprite frame without label")?
                .1
                .push(step);
        }
    }
    let frameset_names = names("SPRITE_ANIM_FRAMESET_");
    ensure!(
        frameset_names.len() == pointers.len(),
        "sprite frameset count mismatch"
    );
    let mut framesets = BTreeMap::new();
    for (name, target) in frameset_names.into_iter().zip(pointers) {
        let start = blocks
            .iter()
            .position(|(name, _)| name == &target)
            .context("missing sprite frameset target")?;
        let mut steps = Vec::new();
        let mut ended = false;
        for (_, block) in &blocks[start..] {
            for step in block {
                steps.push(step.clone());
                if matches!(step["command"].as_str(), Some("end" | "restart" | "delete")) {
                    ended = true;
                    break;
                }
            }
            if ended {
                break;
            }
        }
        ensure!(ended, "unterminated sprite frameset {name}");
        framesets.insert(name.clone(), json!({"name":name,"steps":steps}));
    }
    let bundle = json!({"objects":objects,"oam_sets":oam_sets,"framesets":framesets});
    for (name, value, file) in [
        ("battle_animations", animations, "animations.json"),
        ("battle_animation_table", table, "table.json"),
        ("sprite_anim_bundle", bundle, "bundle.json"),
    ] {
        let directory = data.join("content-packs/core-modular").join(name);
        std::fs::create_dir_all(&directory)?;
        let bytes = serde_json::to_vec_pretty(&value)?;
        std::fs::write(data.join(format!("{name}.json")), &bytes)?;
        std::fs::write(directory.join(file), bytes)?;
    }
    Ok(())
}
