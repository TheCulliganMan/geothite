//! Bind authored transfer instructions to measured ROM calls; preserve provenance.
use super::source_intro_oracle;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

fn number(record: &Value, field: &str) -> Result<u64> {
    record[field]
        .as_u64()
        .with_context(|| format!("intro measurement lacks {field}"))
}

/// Source byte loops have a separate counter rollover path. Keep that path in
/// the cost model rather than treating every copied byte as an equal-duration
/// operation. These costs exclude the caller's CALL instruction and interrupts.
pub(super) fn byte_loop_machine_cycles(routine: &str, byte_count: u16) -> Result<u64> {
    let per_byte = match routine {
        "ByteFill" => 6,
        "CopyBytes" => 10,
        _ => anyhow::bail!("unsupported intro byte loop {routine}"),
    };
    Ok(15 + per_byte * u64::from(byte_count) + 3 * u64::from(byte_count / 256))
}

pub(super) fn byte_loop_instruction_machine_cycles(
    routine: &str,
    byte_count: u16,
) -> Result<Vec<u64>> {
    crystal_assets::runtime_intro_byte_loop_instruction_machine_cycles(routine, byte_count)
}

pub(super) fn clear_helper_instruction_machine_cycles(
    routine: &str,
    byte_count: u16,
) -> Result<Vec<u64>> {
    crystal_assets::runtime_intro_clear_loop_instruction_machine_cycles(routine, byte_count)
}
fn certify_byte_loop_source(vendor: &Path, routine: &str) -> Result<()> {
    let source = super::Source::read(vendor, "home/copy.asm")?;
    let instructions = source
        .body(routine)?
        .iter()
        .skip(1)
        .filter(|(_, line)| !line.starts_with('.'))
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>();
    let expected = match routine {
        "CopyBytes" => vec![
            "inc b",
            "inc c",
            "jr .HandleLoop",
            "ld a, [hli]",
            "ld [de], a",
            "inc de",
            "dec c",
            "jr nz, .CopyByte",
            "dec b",
            "jr nz, .CopyByte",
            "ret",
        ],
        "ByteFill" => vec![
            "inc b",
            "inc c",
            "jr .HandleLoop",
            "ld [hli], a",
            "dec c",
            "jr nz, .PutByte",
            "dec b",
            "jr nz, .PutByte",
            "ret",
        ],
        _ => anyhow::bail!("unsupported intro byte-loop source {routine}"),
    };
    ensure!(
        instructions == expected,
        "intro {routine} cycle-model source changed"
    );
    Ok(())
}

fn register_setup_machine_cycles(instruction: &str) -> Option<u64> {
    if instruction == "xor a" {
        return Some(1);
    }
    if instruction.starts_with("lb bc, ")
        || instruction.starts_with("depixel ")
        || instruction.starts_with("hlcoord ")
        || instruction.starts_with("debgcoord ")
    {
        return Some(3);
    }
    let (destination, operand) = instruction.strip_prefix("ld ")?.split_once(", ")?;
    if operand.starts_with('[') {
        return None;
    }
    if ["bc", "de", "hl"].contains(&destination) {
        return Some(3);
    }
    if ["a", "b", "c", "d", "e", "h", "l"].contains(&destination) {
        return Some(if ["a", "b", "c", "d", "e", "h", "l"].contains(&operand) {
            1
        } else {
            2
        });
    }
    None
}

fn attach_byte_loop_costs(
    vendor: &Path,
    operations: &mut [Value],
    symbols: &std::collections::BTreeMap<String, (usize, u16)>,
) -> Result<()> {
    for routine in ["ByteFill", "CopyBytes"] {
        certify_byte_loop_source(vendor, routine)?;
    }
    for (file, routine, expected) in [
        (
            "home/clear_sprites.asm",
            "ClearSprites",
            vec![
                "ld hl, wShadowOAM",
                "ld b, wShadowOAMEnd - wShadowOAM",
                "xor a",
                "ld [hli], a",
                "dec b",
                "jr nz, .loop",
                "ret",
            ],
        ),
        (
            "engine/sprite_anims/core.asm",
            "ClearSpriteAnims",
            vec![
                "ld hl, wSpriteAnimData",
                "ld bc, wSpriteAnimDataEnd - wSpriteAnimData",
                "ld [hl], 0",
                "inc hl",
                "dec bc",
                "ld a, c",
                "or b",
                "jr nz, .loop",
                "ret",
            ],
        ),
    ] {
        let helper = super::Source::read(vendor, file)?;
        let instructions = helper
            .body(routine)?
            .iter()
            .skip(1)
            .filter(|(_, line)| !line.starts_with('.'))
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>();
        ensure!(
            instructions == expected,
            "intro {routine} cycle-model source changed"
        );
    }
    let source = super::Source::read(vendor, "engine/movie/intro.asm")?;
    for operation in operations {
        let span = &operation["source_span"];
        if span["file"] != "engine/movie/intro.asm" {
            continue;
        }
        let Some(end) = span["end_line"].as_u64() else {
            continue;
        };
        let Some((_, instruction)) = source
            .lines
            .iter()
            .find(|(line, _)| u64::from(*line) == end)
        else {
            continue;
        };
        let routine = match instruction.as_str() {
            "call ByteFill" if operation["op"] == "fill_memory" => "ByteFill",
            "call CopyBytes" if operation["op"] == "copy_memory" => "CopyBytes",
            "call ClearSprites" if operation["op"] == "fill_memory" => "ClearSprites",
            "call ClearSpriteAnims"
                if operation["op"] == "fill_memory"
                    || operation["op"] == "deinitialize_all_sprites" =>
            {
                "ClearSpriteAnims"
            }
            _ => continue,
        };
        let bytes = if operation["op"] == "deinitialize_all_sprites" {
            let start = symbols
                .get("wSpriteAnimData")
                .context("intro sprite clear start symbol")?;
            let end = symbols
                .get("wSpriteAnimDataEnd")
                .context("intro sprite clear end symbol")?;
            ensure!(
                start.0 == end.0,
                "intro sprite clear crossed source memory banks"
            );
            end.1
                .checked_sub(start.1)
                .context("intro sprite clear symbols reversed")?
        } else {
            u16::try_from(number(operation, "byte_count")?)
                .context("intro byte loop exceeds source BC register")?
        };
        let clear = routine.starts_with("Clear");
        let machine_cycles = if clear {
            clear_helper_instruction_machine_cycles(routine, bytes)?
                .iter()
                .sum::<u64>()
        } else {
            byte_loop_machine_cycles(routine, bytes)?
        };
        // The source lowering consumes register preparation without emitting a
        // separate visual effect. Preserve its instruction boundaries before
        // this helper; never scan across an earlier call, label or memory effect.
        let call_index = source
            .lines
            .iter()
            .position(|(line, _)| u64::from(*line) == end)
            .context("intro helper caller line disappeared")?;
        let mut caller_start = call_index;
        let mut prefix = Vec::new();
        while caller_start > 0 {
            let Some(cost) = register_setup_machine_cycles(&source.lines[caller_start - 1].1)
            else {
                break;
            };
            caller_start -= 1;
            prefix.push(cost);
        }
        prefix.reverse();
        prefix.push(6); // Actual direct CALL, separate from the helper body's RET.
        operation["source_caller_instruction_machine_cycles"] = json!(prefix);
        operation["source_caller_span"] =
            json!({"file":source.file,"start_line":source.lines[caller_start].0,"end_line":end});
        operation["source_helper_timing"] = json!({
            "routine":routine,
            "byte_count":bytes,
            "machine_cycles":machine_cycles,
            "caller_call_included":false,
            "interrupts_included":false,
            "source_file":match routine {"ClearSprites"=>"home/clear_sprites.asm","ClearSpriteAnims"=>"engine/sprite_anims/core.asm",_=>"home/copy.asm"},
            "model":if clear {"clear_register_counter_loop_v1"} else {"byte_count_and_counter_rollover_v1"}
        });
    }
    Ok(())
}

#[cfg(test)]
mod byte_loop_cost_tests {
    use super::*;

    #[test]
    fn source_byte_loop_costs_include_empty_and_counter_rollover_paths() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = super::super::Source::read(vendor, "home/copy.asm").unwrap();
        for (routine, copy) in [("ByteFill", false), ("CopyBytes", true)] {
            let instructions = source
                .body(routine)
                .unwrap()
                .iter()
                .skip(1)
                .filter(|(_, line)| !line.starts_with('.'))
                .map(|(_, line)| line.as_str())
                .collect::<Vec<_>>();
            let expected = if copy {
                vec![
                    "inc b",
                    "inc c",
                    "jr .HandleLoop",
                    "ld a, [hli]",
                    "ld [de], a",
                    "inc de",
                    "dec c",
                    "jr nz, .CopyByte",
                    "dec b",
                    "jr nz, .CopyByte",
                    "ret",
                ]
            } else {
                vec![
                    "inc b",
                    "inc c",
                    "jr .HandleLoop",
                    "ld [hli], a",
                    "dec c",
                    "jr nz, .PutByte",
                    "dec b",
                    "jr nz, .PutByte",
                    "ret",
                ]
            };
            assert_eq!(
                instructions, expected,
                "byte-loop source certificate changed"
            );
            for count in [0, 1, 2, 17, 128, 193, 255, 256, 257, 360, 511, 512] {
                let measured = source_intro_oracle::measure_routine_state(
                    vendor,
                    routine,
                    &[],
                    &[("BC", count), ("HL", 0xc800), ("DE", 0xc000), ("A", 1)],
                )
                .unwrap();
                assert_eq!(
                    measured["machine_cycles"].as_u64().unwrap(),
                    byte_loop_machine_cycles(routine, count).unwrap(),
                    "{routine} count {count}"
                );
                let tape = byte_loop_instruction_machine_cycles(routine, count).unwrap();
                let actual = measured["instructions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(
                    tape, actual,
                    "{routine} count {count} instruction boundaries"
                );
            }
        }
        assert!(byte_loop_machine_cycles("Unknown", 1).is_err());
        for (routine, count) in [("ClearSpriteAnims", 193), ("ClearSprites", 160)] {
            let measured = source_intro_oracle::measure_routine(vendor, routine, &[]).unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                clear_helper_instruction_machine_cycles(routine, count).unwrap(),
                actual
            );
        }
    }
}
fn crossings(record: &Value) -> Result<u64> {
    number(record, "end_frame")?
        .checked_sub(number(record, "start_frame")?)
        .context("intro measured frame range reversed")
}
fn identity(trace: &Value, record: &Value) -> Result<Value> {
    let base = number(&trace["entry"], "lcd_t_cycles")? / 70224;
    ensure!(
        number(record, "elapsed_t_cycles")? == number(record, "lcd_elapsed_t_cycles")?,
        "intro transfer disabled its LCD clock"
    );
    Ok(
        json!({"runner":"pack_core Rust intro oracle","schema_version":1,"validated":false,"rom_sha1":trace["rom_sha1"],"instruction_core":"sm83_mcycle_v1","timer_model":"cgb_timer_falling_edge_delayed_reload_v1","start_intro_frame":number(record,"start_frame")?.checked_sub(base).context("hook precedes intro")?,"end_intro_frame":number(record,"end_frame")?.checked_sub(base).context("hook precedes intro")?,"start_frame_phase_t_cycles":record["start_phase_t_cycles"],"elapsed_t_cycles_between_hooks":record["elapsed_t_cycles"]}),
    )
}

pub(super) fn verify_scene_control(phase: &Value, trace: &Value) -> Result<usize> {
    use crystal_assets::{
        RuntimePresentationPhase, RuntimePresentationPhaseMachine, RuntimePresentationProgram,
        RuntimePresentationSubprogram,
    };
    let mut phase = RuntimePresentationPhase {
        id: "scene_dispatch".into(),
        labels: serde_json::from_value(phase["labels"].clone())?,
        operations: serde_json::from_value(phase["operations"].clone())?,
        ..Default::default()
    };
    // Setup transfers are executed by the production timed cursor, not the
    // control machine. Their exact ROM bindings are checked by attach().
    // Keep their positions so authored label offsets remain unchanged.
    for operation in &mut phase.operations {
        if matches!(
            operation.op.as_str(),
            "decompress_lz3_resource"
                | "request_2bpp_transfer"
                | "copy_memory"
                | "copy_strided_memory"
                | "tilemap_xor"
                | "save_memory_byte"
                | "restore_memory_byte"
                | "write_memory_bytes"
        ) {
            operation.op = "wait_frames".into();
            operation.fields = serde_json::from_value(json!({"frames":0}))?;
        }
    }
    let program = RuntimePresentationProgram {
        subprograms: vec![RuntimePresentationSubprogram {
            id: "crystal_intro".into(),
            phases: vec![phase],
            ..Default::default()
        }],
        ..Default::default()
    };
    let records = trace["records"]
        .as_array()
        .context("intro control trace records")?;
    let mut checked = 0;
    let mut scenes = std::collections::BTreeSet::new();
    let mut calls = records
        .iter()
        .filter(|record| record["hook"] == "IntroSceneJumper")
        .collect::<Vec<_>>();
    calls.sort_by_key(|record| record["start_cpu_t_cycles"].as_u64());
    let mut machine =
        RuntimePresentationPhaseMachine::new(&program, "crystal_intro", "scene_dispatch")?;
    machine.memory = serde_json::from_value(
        calls.first().context("intro control calls empty")?["start_memory"].clone(),
    )?;
    for record in calls {
        let scene = number(record, "dispatcher_entry")?;
        scenes.insert(scene);
        machine
            .run_from_label_with_effects(
                &program,
                &format!("IntroScene{}", scene + 1),
                0,
                &mut |effect, memory| {
                    if effect.op == "fill_memory"
                        && effect.fields.get("target").and_then(Value::as_str)
                            == Some("wLYOverrides")
                    {
                        let value = effect.fields["value"]
                            .as_u64()
                            .context("LY override fill value")?
                            as u16;
                        memory.insert("wLYOverrides".into(), value);
                        memory.insert("wLYOverrides+95".into(), value);
                    }
                    if effect.op == "fill_memory"
                        && effect.fields.get("target").and_then(Value::as_str)
                            == Some("wSpriteAnimData")
                    {
                        ensure!(
                            effect.fields["value"] == 0 && effect.fields["byte_count"] == 193,
                            "unsupported sprite animation reset domain"
                        );
                        memory.insert("wGlobalAnimXOffset".into(), 0);
                        memory.insert("wGlobalAnimYOffset".into(), 0);
                    }
                    if effect.op == "perspective_scroll" {
                        let effect_frame = memory["wIntroSceneFrameCounter"];
                        let frame = &effect.fields["frame"];
                        ensure!(
                            frame["source"] == "wIntroSceneFrameCounter",
                            "unknown perspective clock"
                        );
                        let mask = frame["parity_mask"]
                            .as_u64()
                            .context("perspective parity mask")?
                            as u16;
                        for band in effect.fields["bands"]
                            .as_array()
                            .context("perspective bands")?
                        {
                            let offset = band["offset"].as_u64().context("perspective offset")?;
                            let key = if offset == 0 {
                                "wLYOverrides"
                            } else {
                                ensure!(offset == 95, "unknown perspective band");
                                "wLYOverrides+95"
                            };
                            let active = match band["cadence"].as_str() {
                                Some("every_frame") => true,
                                Some("odd_frames") => effect_frame & mask != 0,
                                _ => anyhow::bail!("unknown perspective cadence"),
                            };
                            if active {
                                let delta =
                                    band["delta"].as_u64().context("perspective delta")? as u8;
                                memory.insert(
                                    key.into(),
                                    u16::from((memory[key] as u8).wrapping_add(delta)),
                                );
                            }
                        }
                        ensure!(
                            effect.fields["horizontal_scroll"]["source_offset"] == 0,
                            "unknown perspective scroll source"
                        );
                        memory.insert("hSCX".into(), memory["wLYOverrides"]);
                    }
                    Ok(())
                },
            )
            .with_context(|| format!("ROM scene {} tick {}", scene + 1, record["dispatch_tick"]))?;
        for name in [
            "wJumptableIndex",
            "wIntroSceneFrameCounter",
            "wIntroSceneTimer",
            "wGlobalAnimXOffset",
            "wGlobalAnimYOffset",
            "hSCX",
            "hSCY",
        ] {
            let expected = record["end_memory"][name]
                .as_u64()
                .with_context(|| format!("intro trace omits {name}"))?;
            let actual = machine
                .memory
                .get(name)
                .copied()
                .context("intro machine omitted source memory")?;
            ensure!(
                u64::from(actual) == expected,
                "intro source/ROM control mismatch scene {} tick {} {name}: runtime {actual}, ROM {expected}",
                scene + 1,
                record["dispatch_tick"]
            );
        }
        checked += 1;
    }
    ensure!(
        scenes.len() == 28 && checked > 28,
        "intro control proof lacks complete scene coverage"
    );
    Ok(checked)
}

pub(super) fn verify_setup_timeline(phase: &Value, trace: &Value) -> Result<usize> {
    use crystal_assets::{
        RuntimePresentation2bppTransferMode, RuntimePresentationPhase, RuntimePresentationProgram,
        RuntimePresentationSubprogram, RuntimePresentationTimedPhaseCursor,
    };
    let phase = RuntimePresentationPhase {
        id: "scene_dispatch".into(),
        labels: serde_json::from_value(phase["labels"].clone())?,
        operations: serde_json::from_value(phase["operations"].clone())?,
        ..Default::default()
    };
    let program = RuntimePresentationProgram {
        subprograms: vec![RuntimePresentationSubprogram {
            id: "crystal_intro".into(),
            phases: vec![phase],
            ..Default::default()
        }],
        ..Default::default()
    };
    let phase = &program.subprograms[0].phases[0];
    let mut checked = 0;
    for scene in 0..28 {
        let start = phase.labels[&format!("IntroScene{}", scene + 1)];
        let next = phase
            .labels
            .get(&format!("IntroScene{}", scene + 2))
            .copied()
            .unwrap_or(phase.operations.len());
        let body = &phase.operations[start..next];
        if !body
            .iter()
            .any(|operation| operation.op == "decompress_lz3_resource")
        {
            continue;
        }
        let end = start
            + body
                .iter()
                .position(|operation| operation.op == "return")
                .context("setup return")?;
        let mut cursor = RuntimePresentationTimedPhaseCursor::new(
            &program,
            "crystal_intro",
            "scene_dispatch",
            start,
            end,
        )?
        .with_2bpp_transfer_mode(RuntimePresentation2bppTransferMode::Default)
        .with_frame_t_cycles(70224)?;
        let mut ticks = 0;
        loop {
            ticks += 1;
            ensure!(ticks < 1000, "setup timeline failed to complete");
            if cursor.tick(&program)?.complete {
                break;
            }
        }
        let calls = trace["records"]
            .as_array()
            .context("setup trace")?
            .iter()
            .filter(|record| {
                record["hook"] == "IntroSceneJumper" && record["dispatcher_entry"] == scene
            })
            .collect::<Vec<_>>();
        ensure!(
            calls.len() == 1,
            "setup scene has multiple ROM dispatch calls"
        );
        let expected = crossings(calls[0])?;
        ensure!(
            ticks - 1 == expected,
            "intro setup scene {} runtime cursor waits {} frames, ROM crosses {expected}",
            scene + 1,
            ticks - 1
        );
        checked += 1;
    }
    ensure!(checked == 13, "incomplete setup timeline coverage");
    Ok(checked)
}

#[cfg(test)]
mod control_tests {
    #[test]
    fn all_intro_dispatch_calls_match_rom_control_state() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let vendor = root.join("vendor/pokecrystal");
        let constants: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
                .unwrap(),
        )
        .unwrap();
        let constants = serde_json::from_value(constants["global"].clone()).unwrap();
        let dispatch =
            super::super::source_title_intro_scenes::scene_dispatch(&vendor, &constants).unwrap();
        let trace = super::source_intro_oracle::trace(&vendor).unwrap();
        let count = super::verify_scene_control(&dispatch["phase"], &trace).unwrap();
        println!("{count} ROM dispatch calls match production control state");
        assert_eq!(count, 1753);
        let mut measured = dispatch["phase"].clone();
        super::attach(
            &vendor,
            &mut measured,
            &dispatch["transfer_metadata"],
            &trace,
        )
        .unwrap();
        // The current timed cursor has not earned whole-phase certification:
        // transfer waits alone omit source work outside the measured wrappers.
        assert!(
            super::verify_setup_timeline(&measured, &trace)
                .unwrap_err()
                .to_string()
                .contains("scene 1 runtime cursor waits 64 frames, ROM crosses 65")
        );
        let pichu = measured["operations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|operation| {
                operation["op"] == "play_audio" && operation["audio"] == "SFX_INTRO_PICHU"
            })
            .map(|operation| operation["dispatch_tick"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(pichu, [33, 65].into());
        let mut changed = dispatch["phase"].clone();
        let transition = changed["operations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|operation| {
                operation["op"] == "increment_memory_byte"
                    && operation["target"] == "wJumptableIndex"
            })
            .unwrap();
        transition["delta"] = serde_json::json!(2);
        assert!(
            super::verify_scene_control(&changed, &trace)
                .unwrap_err()
                .to_string()
                .contains("control mismatch")
        );
    }
}

pub(super) fn attach(
    vendor: &Path,
    phase: &mut Value,
    metadata: &Value,
    trace: &Value,
) -> Result<Value> {
    let symbols =
        source_intro_oracle::symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let records = trace["records"].as_array().context("intro trace records")?;
    let mut decompressions = records
        .iter()
        .filter(|record| record["hook"] == "Decompress")
        .collect::<Vec<_>>();
    decompressions.sort_by_key(|record| record["start_cpu_t_cycles"].as_u64());
    let mut wrappers = records
        .iter()
        .filter(|record| {
            record["hook"]
                .as_str()
                .is_some_and(|name| name.starts_with("Intro_DecompressRequest2bpp_"))
        })
        .collect::<Vec<_>>();
    wrappers.sort_by_key(|record| record["start_cpu_t_cycles"].as_u64());
    let transfers = metadata["resource_transfers"]
        .as_array()
        .context("authored intro transfers")?;
    ensure!(
        decompressions.len() == 37 && wrappers.len() == 37 && transfers.len() == 37,
        "incomplete measured/source intro transfer coverage"
    );
    let operations = phase["operations"]
        .as_array_mut()
        .context("intro phase operations")?;
    attach_byte_loop_costs(vendor, operations, &symbols)?;
    let mut count = 0;
    for index in 0..operations.len() {
        if operations[index]["op"] != "decompress_lz3_resource" {
            continue;
        }
        let record = *decompressions
            .get(count)
            .context("extra intro decompression")?;
        let wrapper = wrappers[count];
        let transfer = &transfers[count];
        let source_symbol = transfer["source_symbol"]
            .as_str()
            .context("intro source symbol")?;
        let &(bank, address) = symbols
            .get(source_symbol)
            .with_context(|| format!("missing intro data symbol {source_symbol}"))?;
        ensure!(
            number(record, "bank")? == bank as u64 && number(record, "hl")? == address as u64,
            "intro compressed source disagrees with measured ROM pointer"
        );
        ensure!(
            operations[index]["resource"] == transfer["resource"]
                && operations[index]["dispatcher_entry"] == record["dispatcher_entry"]
                && operations[index]["dispatch_tick"] == record["dispatch_tick"],
            "intro decompression source order differs from ROM dispatch"
        );
        ensure!(
            wrapper["hook"] == transfer["wrapper"],
            "intro wrapper identity differs from source"
        );
        let begin = number(wrapper, "start_cpu_t_cycles")?;
        let end = number(wrapper, "end_cpu_t_cycles")?;
        let requests = records
            .iter()
            .filter(|candidate| {
                candidate["hook"] == "Request2bpp"
                    && candidate["start_cpu_t_cycles"]
                        .as_u64()
                        .is_some_and(|clock| clock >= begin && clock < end)
            })
            .collect::<Vec<_>>();
        ensure!(
            requests.len() == 1,
            "intro wrapper has ambiguous transfer request"
        );
        let request = requests[0];
        ensure!(
            number(request, "vram_bank")? == number(transfer, "target_vram_bank")?,
            "intro VRAM bank differs from source"
        );
        ensure!(
            number(request, "bc")? & 255 == number(transfer, "tile_count")?,
            "intro transfer size differs from source"
        );
        operations[index]["decompress_machine_cycles"] =
            record["instruction_machine_cycles"].clone();
        operations[index]["decompress_frame_boundaries_crossed"] = json!(crossings(record)?);
        operations[index]["timing_oracle"] = identity(trace, record)?;
        let next = operations
            .get_mut(index + 1)
            .context("intro decompression missing request")?;
        ensure!(
            next["op"] == "request_2bpp_transfer",
            "intro decompression request is not adjacent"
        );
        next["request_2bpp_machine_cycles"] = request["instruction_machine_cycles"].clone();
        next["request_2bpp_frame_boundaries_crossed"] = json!(crossings(request)?);
        next["request_2bpp_timing_oracle"] = identity(trace, request)?;
        next["decompress_request_frame_boundaries_crossed"] = json!(crossings(wrapper)?);
        next["decompress_request_timing_oracle"] = identity(trace, wrapper)?;
        ensure!(
            crossings(record)? + crossings(request)? == crossings(wrapper)?,
            "intro wrapper crossing count differs from its measured children"
        );
        count += 1;
    }
    ensure!(count == 37, "intro phase omits measured decompressions");
    let root = vendor
        .parent()
        .and_then(Path::parent)
        .context("intro source workspace")?;
    let constants: Value = serde_json::from_slice(&std::fs::read(
        root.join("apps/web/assets/data/story_event_script_constants.json"),
    )?)?;
    let source = super::Source::read(vendor, "engine/movie/intro.asm")?;
    let mut global = String::new();
    let mut source_labels = Vec::new();
    for (line, code) in &source.lines {
        let name = code.trim_end_matches(':');
        if !name.contains(' ') && (code.ends_with(':') || name.starts_with('.')) {
            let name = if name.starts_with('.') {
                format!("{global}{name}")
            } else {
                global = name.into();
                global.clone()
            };
            if let Some(&(bank, address)) = symbols.get(&name) {
                source_labels.push((*line, bank, address));
            }
        }
    }
    for operation in operations
        .iter_mut()
        .filter(|operation| operation["op"] == "play_audio" && operation["dispatch_tick"].is_null())
    {
        let id = operation["source_id"]
            .as_str()
            .context("intro sound source identity")?;
        let expected = constants["global"][id]
            .as_u64()
            .with_context(|| format!("intro sound constant {id}"))?;
        let line = operation["source_span"]["start_line"]
            .as_u64()
            .context("intro sound call line")?;
        let position = source_labels
            .iter()
            .rposition(|(start, _, _)| u64::from(*start) <= line)
            .context("intro sound call has no source label")?;
        let (_, bank, begin) = source_labels[position];
        let end = source_labels
            .get(position + 1)
            .filter(|(_, next_bank, _)| *next_bank == bank)
            .map(|(_, _, address)| *address)
            .unwrap_or(0x8000);
        let matching = records
            .iter()
            .filter(|record| {
                record["hook"] == "PlaySFX"
                    && record["dispatcher_entry"] == operation["dispatcher_entry"]
                    && record["de"].as_u64() == Some(expected)
                    && record["return_pc"].as_u64().is_some_and(|address| {
                        address > u64::from(begin) && address <= u64::from(end)
                    })
            })
            .collect::<Vec<_>>();
        ensure!(
            matching.len() == 1,
            "intro sound {id} has {} matching ROM calls",
            matching.len()
        );
        operation["dispatch_tick"] = matching[0]["dispatch_tick"].clone();
    }
    let crossings=records.iter().filter(|record|record["hook"]=="PlaySpriteAnimations").filter_map(|record|match crossings(record) {Ok(value)if value>0=>Some(Ok(json!({"dispatcher_entry":record["dispatcher_entry"],"dispatch_tick":record["dispatch_tick"],"elapsed_t_cycles_between_hooks":record["elapsed_t_cycles"]}))),Ok(_)=>None,Err(error)=>Some(Err(error))}).collect::<Result<Vec<_>>>()?;
    ensure!(
        !crossings.is_empty(),
        "intro scheduler timing coverage empty"
    );
    Ok(
        json!({"rom_frame_crossings":crossings,"timing_oracle":{"runner":"pack_core Rust intro oracle","schema_version":1,"validated":false,"rom_sha1":trace["rom_sha1"],"instruction_core":"sm83_mcycle_v1","timer_model":"cgb_timer_falling_edge_delayed_reload_v1"}}),
    )
}
