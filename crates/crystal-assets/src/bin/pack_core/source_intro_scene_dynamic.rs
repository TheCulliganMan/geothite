//! Strict lowering of the authored animated intro scenes.
use super::{Cursor, Source, emit, helpers};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
fn evaluate(expression: &str, constants: &BTreeMap<String, i64>) -> Result<i64> {
    super::evaluate(expression, constants, &BTreeMap::new())
}

pub(super) fn phases(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let source = Source::read(vendor, "engine/movie/intro.asm")?;
    let sprites = super::super::source_title_sprites::export(vendor)?;
    let mut phases = serde_json::Map::new();
    for number in [2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 25, 27, 28] {
        let label = format!("IntroScene{number}");
        let mut cursor = Cursor::new(&source, &label)?;
        let mut operations = Vec::new();
        let mut labels = serde_json::Map::new();
        let mut hl = String::new();
        let mut de = String::new();
        let mut compare = 0i64;
        let mut accumulator_source = "accumulator";
        while cursor.index < cursor.body.len() {
            let line = cursor.body[cursor.index].1.clone();
            if line.starts_with('.') {
                let local = line.trim_end_matches(':');
                if number == 12 && local == ".PlayUnownSound" {
                    break;
                }
                if number == 20 && local == ".AppearUnownPal2" {
                    while cursor.index < cursor.body.len()
                        && cursor.body[cursor.index].1.trim_end_matches(':') != ".finished"
                    {
                        cursor.index += 1;
                    }
                    continue;
                }
                labels.insert(format!("{local}@{label}"), json!(operations.len()));
                cursor.index += 1;
                continue;
            }
            let mut count = 1;
            if number == 14 && (line == "ldh a, [hSCX]" || line == "ld a, [wGlobalAnimXOffset]") {
                let target = if line.starts_with("ldh") {
                    "hSCX"
                } else {
                    "wGlobalAnimXOffset"
                };
                let next = &cursor.body[cursor.index + 1].1;
                if let Some(delta) = next.strip_prefix("sub ") {
                    ensure!(
                        cursor.body[cursor.index + 2].1.ends_with("], a"),
                        "intro run offset store changed"
                    );
                    emit(
                        &mut cursor,
                        3,
                        "subtract_memory_byte",
                        json!({"target":target,"delta":evaluate(delta,constants)?,"wrap":"u8"}),
                        &mut operations,
                    )?;
                    continue;
                }
                if next == "cp $88" {
                    ensure!(
                        cursor.body[cursor.index + 2].1 == "jr c, .disappear",
                        "intro disappearance changed"
                    );
                    emit(
                        &mut cursor,
                        3,
                        "branch_memory_compare",
                        json!({"source":target,"operand":136,"predicate":"unsigned_less_than","target":format!(".disappear@{label}")}),
                        &mut operations,
                    )?;
                    continue;
                }
            }
            if number == 14
                && line == "ld a, $1"
                && cursor.body[cursor.index + 1].1 == "ld [wIntroSceneTimer], a"
            {
                emit(
                    &mut cursor,
                    2,
                    "write_memory_byte",
                    json!({"target":"wIntroSceneTimer","value":1}),
                    &mut operations,
                )?;
                continue;
            }
            if number == 14
                && line == "sub $8"
                && cursor.body[cursor.index + 1].1 == "ld [wGlobalAnimXOffset], a"
            {
                emit(
                    &mut cursor,
                    2,
                    "subtract_memory_byte",
                    json!({"target":"wGlobalAnimXOffset","delta":8,"wrap":"u8"}),
                    &mut operations,
                )?;
                continue;
            }
            if number == 20 && line == "ldh a, [hSCY]" {
                ensure!(
                    cursor.body[cursor.index + 1].1 == "inc a"
                        && cursor.body[cursor.index + 2].1 == "ldh [hSCY], a",
                    "intro reveal scroll changed"
                );
                emit(
                    &mut cursor,
                    3,
                    "increment_memory_byte",
                    json!({"target":"hSCY","delta":1,"wrap":"u8"}),
                    &mut operations,
                )?;
                continue;
            }
            if number == 20 && line == "sub $18" {
                emit(
                    &mut cursor,
                    2,
                    "set_local_from_result",
                    json!({"name":"reveal_phase","source":"intro_scene_frame","subtract":24,"wrap":"u8"}),
                    &mut operations,
                )?;
                ensure!(
                    cursor.body[cursor.index].1 == "and $3"
                        && cursor.body[cursor.index + 1].1 == "cp $3"
                        && cursor.body[cursor.index + 2].1 == "ret nz",
                    "intro reveal cadence changed"
                );
                emit(
                    &mut cursor,
                    3,
                    "return_unless_mask_equal",
                    json!({"source":"reveal_phase","mask":3,"operand":3}),
                    &mut operations,
                )?;
                ensure!(
                    cursor.body[cursor.index].1 == "ld a, c"
                        && cursor.body[cursor.index + 1].1 == "and $1c"
                        && cursor.body[cursor.index + 2].1 == "srl a"
                        && cursor.body[cursor.index + 3].1 == "srl a"
                        && cursor.body[cursor.index + 4].1 == "ld [wIntroSceneTimer], a",
                    "intro reveal timer changed"
                );
                emit(
                    &mut cursor,
                    5,
                    "write_memory_byte_from_masked_result",
                    json!({"target":"wIntroSceneTimer","result":"reveal_phase","mask":28,"shift_right":2}),
                    &mut operations,
                )?;
                continue;
            }
            let (op, fields) = if line.starts_with("ld hl, ") {
                hl = line[7..].to_string();
                cursor.index += 1;
                continue;
            } else if line.starts_with("ld de, ") {
                de = line[7..].to_string();
                cursor.index += 1;
                continue;
            } else if line == "ld a, [hl]"
                && cursor
                    .body
                    .get(cursor.index + 1)
                    .is_some_and(|x| x.1 == "inc [hl]")
            {
                count = 2;
                accumulator_source = "intro_scene_frame";
                emit(
                    &mut cursor,
                    count,
                    "postincrement_memory_byte",
                    json!({"target":hl,"result":"intro_scene_frame","delta":1}),
                    &mut operations,
                )?;
                if number == 20 || number == 14 {
                    continue;
                }
                let mut copy = operations.last().unwrap().clone();
                copy["op"] = json!("set_local_from_result");
                copy["name"] = json!("accumulator");
                copy["source"] = json!("intro_scene_frame");
                copy["wrap"] = json!("u8");
                copy["subtract"] = json!(0);
                operations.push(copy);
                continue;
            } else if line == "ld a, [hl]" {
                (
                    "read_memory_byte",
                    json!({"target":hl,"result":"accumulator"}),
                )
            } else if line == "inc [hl]" {
                ("increment_memory_byte", json!({"target":hl,"delta":1}))
            } else if line == "dec [hl]" {
                accumulator_source = "predecrement_value";
                (
                    "decrement_memory_byte",
                    json!({"target":hl,"delta":1,"wrap":"u8","comparison_value":"predecrement_value"}),
                )
            } else if let Some(bit) = line
                .strip_prefix("set ")
                .and_then(|x| x.strip_suffix(", [hl]"))
            {
                (
                    "set_memory_bit",
                    json!({"target":hl,"bit":evaluate(bit,constants)?}),
                )
            } else if line.starts_with("ld a, [") || line.starts_with("ldh a, [") {
                accumulator_source = "accumulator";
                let target = line.split_once('[').unwrap().1.trim_end_matches(']');
                (
                    "read_memory_byte",
                    json!({"target":target,"result":"accumulator"}),
                )
            } else if (line.starts_with("ld [") || line.starts_with("ldh ["))
                && line.ends_with(", a")
            {
                let target = line.split_once('[').unwrap().1.split_once(']').unwrap().0;
                (
                    "write_memory_byte_from_result",
                    json!({"target":target,"result":"accumulator"}),
                )
            } else if number == 24 && line == "ld c, a" {
                ensure!(
                    cursor.body[cursor.index + 1].1 == "and $3"
                        && cursor.body[cursor.index + 2].1 == "ret nz",
                    "intro fade cadence changed"
                );
                count = 3;
                (
                    "return_unless_mask_equal",
                    json!({"source":"intro_scene_frame","mask":3,"operand":0}),
                )
            } else if number == 24 && line == "ld a, c" {
                ensure!(
                    cursor.body[cursor.index + 1].1 == "and $1c"
                        && cursor.body[cursor.index + 2].1 == "sla a",
                    "intro fade selector changed"
                );
                count = 3;
                (
                    "set_local_from_masked_result",
                    json!({"name":"accumulator","source":"intro_scene_frame","mask":28,"shift_left":1,"wrap":"u8","valid_values":[0,8,16,24,32,40,48,56]}),
                )
            } else if line == "ld c, a"
                || line == "push af"
                || line == "ld a, c"
                || line == "pop af"
            {
                let (name, input) = match line.as_str() {
                    "ld c, a" => ("saved_c", "accumulator"),
                    "push af" => ("saved_af", "accumulator"),
                    "ld a, c" => ("accumulator", "saved_c"),
                    _ => ("accumulator", "saved_af"),
                };
                (
                    "set_local_from_result",
                    json!({"name":name,"source":input,"wrap":"u8","subtract":0}),
                )
            } else if line == "xor a" || line.starts_with("ld a, ") {
                accumulator_source = "accumulator";
                let value = if line == "xor a" {
                    0
                } else {
                    evaluate(&line[6..], constants)?
                };
                ("set_local", json!({"name":"accumulator","value":value}))
            } else if line.starts_with("cp ") || line == "and a" {
                compare = if line == "and a" {
                    0
                } else {
                    evaluate(&line[3..], constants)?
                };
                cursor.index += 1;
                continue;
            } else if line.starts_with("and ")
                || line.starts_with("or ")
                || ["sla a", "srl a", "swap a"].contains(&line.as_str())
            {
                let (step, value) = if let Some(x) = line.strip_prefix("and ") {
                    ("mask", evaluate(x, constants)?)
                } else if let Some(x) = line.strip_prefix("or ") {
                    ("or", evaluate(x, constants)?)
                } else {
                    match line.as_str() {
                        "sla a" => ("shift_left", 1),
                        "srl a" => ("shift_right", 1),
                        _ => ("swap_nibbles", 0),
                    }
                };
                (
                    "compute_byte",
                    json!({"input":"accumulator","result":"accumulator","steps":[{"op":step,"value":value}]}),
                )
            } else if line.starts_with("sub ")
                || line.starts_with("add ")
                || line == "inc a"
                || line == "dec a"
            {
                let subtract = if let Some(x) = line.strip_prefix("sub ") {
                    evaluate(x, constants)? as u8
                } else if let Some(x) = line.strip_prefix("add ") {
                    (evaluate(x, constants)? as u8).wrapping_neg()
                } else if line == "inc a" {
                    255
                } else {
                    1
                };
                compare = 0;
                (
                    "set_local_from_result",
                    json!({"name":"accumulator","source":"accumulator","subtract":subtract,"wrap":"u8"}),
                )
            } else if let Some(branch) = line.strip_prefix("jr ") {
                if let Some((condition, target)) = branch.split_once(", ") {
                    let target = format!("{target}@{label}");
                    let predicate = match condition {
                        "z" => "equal",
                        "nz" => "not_equal",
                        "nc" => "unsigned_greater_or_equal",
                        "c" => "unsigned_less_than",
                        _ => anyhow::bail!("intro branch condition {condition}"),
                    };
                    if condition == "c" {
                        let fallthrough = format!(".lowered_c_{}@{label}", cursor.index);
                        emit(
                            &mut cursor,
                            1,
                            "branch_compare",
                            json!({"value":"accumulator","predicate":"unsigned_greater_or_equal","operand":compare,"target":fallthrough}),
                            &mut operations,
                        )?;
                        let span = operations.last().unwrap()["source_span"].clone();
                        operations.push(json!({"op":"jump","target":target,"source_span":span}));
                        labels.insert(fallthrough, json!(operations.len()));
                        continue;
                    }
                    (
                        "branch_compare",
                        json!({"value":accumulator_source,"predicate":predicate,"operand":compare,"target":target}),
                    )
                } else {
                    ("jump", json!({"target":format!("{branch}@{label}")}))
                }
            } else if line == "ret" {
                ("return", json!({}))
            } else if let Some(condition) = line.strip_prefix("ret ") {
                let predicate = match condition {
                    "z" => "equal",
                    "nz" => "not_equal",
                    "nc" => "unsigned_greater_or_equal",
                    "c" => "unsigned_less_than",
                    _ => anyhow::bail!("intro return condition {condition}"),
                };
                (
                    "return_if_compare",
                    json!({"value":accumulator_source,"predicate":predicate,"operand":compare}),
                )
            } else if line.starts_with("depixel ") {
                let call = cursor
                    .body
                    .get(cursor.index + 1)
                    .context("intro sprite activation")?
                    .1
                    .clone();
                count = if call == "call CrystalIntro_InitUnownAnim" {
                    2
                } else {
                    3
                };
                let call_line = cursor.body[cursor.index + count - 1].0;
                let matches = sprites["activation_operations"]
                    .as_array()
                    .context("intro sprite activations")?
                    .iter()
                    .filter(|x| {
                        x["dispatcher_entry"] == number - 1
                            && x["source_span"]["end_line"] == call_line
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                ensure!(
                    !matches.is_empty(),
                    "no certified intro allocation for {label}:{call_line}"
                );
                emit(
                    &mut cursor,
                    count,
                    "source_sprite_activation_group",
                    json!({}),
                    &mut operations,
                )?;
                operations.pop();
                operations.extend(matches);
                continue;
            } else if let Some(call) = line
                .strip_prefix("call ")
                .or_else(|| line.strip_prefix("farcall "))
            {
                match call {
                    "NextIntroScene" => (
                        "increment_memory_byte",
                        json!({"target":"wJumptableIndex","delta":1,"implementation_source_span":source.span(call)?}),
                    ),
                    "DeinitializeAllSprites" => (
                        "deinitialize_all_sprites",
                        json!({"implementation_source_span":Source::read(vendor,"engine/sprite_anims/core.asm")?.span(call)?}),
                    ),
                    "PlaySFX" => (
                        "play_audio",
                        json!({"kind":"sfx","source_id":de,"audio":de,"dispatcher_entry":number-1}),
                    ),
                    "ClearBGPalettes" => {
                        let home = Source::read(vendor, "home/tilemap.asm")?;
                        emit(
                            &mut cursor,
                            1,
                            "fill_memory",
                            json!({"target":"wBGPals2","value":255,"byte_count":128,"implementation_source_span":home.span("ClearPalettes")?}),
                            &mut operations,
                        )?;
                        operations.push(home.operation(
                            "ClearPalettes",
                            &["ld a, TRUE", "ldh [hCGBPalUpdate], a", "ret"],
                            "palette_transfer_request",
                            json!({"target":"hCGBPalUpdate","value":1}),
                        )?);
                        operations.push(home.operation(
                            "WaitBGMap",
                            &["ld a, 1", "ldh [hBGMapMode], a"],
                            "write_memory_byte",
                            json!({"target":"hBGMapMode","value":1}),
                        )?);
                        operations.push(home.operation(
                            "WaitBGMap",
                            &["ld c, 4", "call DelayFrames", "ret"],
                            "wait_frames",
                            json!({"frames":4}),
                        )?);
                        continue;
                    }
                    ".PlayUnownSound" => {
                        let entries=cursor.body.iter().filter_map(|(_,x)|x.strip_prefix("dbw ")).map(|x|{let (frame,sound)=x.split_once(", ").context("intro sound table")?;Ok(json!({"frame":evaluate(frame,constants)?,"kind":"sfx","source_id":sound,"audio":sound}))}).collect::<Result<Vec<_>>>()?;
                        (
                            "scheduled_audio",
                            json!({"clock":"wIntroSceneFrameCounter","entries":entries,"sentinel":255,"on_match":{"play_entry":true,"stop_sfx_channels":[5,6,7,8]}}),
                        )
                    }
                    _ => helpers::fields(vendor, &source, call)?,
                }
            } else {
                anyhow::bail!("unlowered dynamic intro {label}: {line}")
            };
            emit(&mut cursor, count, op, fields, &mut operations)?;
        }
        if number != 12 {
            cursor.finish()?;
        }
        phases.insert(
            label.clone(),
            json!({"source_span":source.span(&label)?,"labels":labels,"operations":operations}),
        );
    }
    Ok(json!(phases))
}
