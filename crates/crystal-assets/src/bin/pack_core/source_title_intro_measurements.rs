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

fn certify_farcall_source(vendor: &Path) -> Result<()> {
    for (file, label, expected) in [
        ("home/header.asm", "FarCall", vec!["jp FarCall_hl"]),
        (
            "home/header.asm",
            "Bankswitch",
            vec!["ldh [hROMBank], a", "ld [rROMB], a", "ret"],
        ),
        (
            "home/farcall.asm",
            "FarCall_hl",
            vec![
                "ldh [hTempBank], a",
                "ldh a, [hROMBank]",
                "push af",
                "ldh a, [hTempBank]",
                "rst Bankswitch",
                "call FarCall_JumpToHL",
            ],
        ),
        ("home/farcall.asm", "FarCall_JumpToHL", vec!["jp hl"]),
        (
            "home/farcall.asm",
            "ReturnFarCall",
            vec![
                "ld a, b",
                "ld [wFarCallBC], a",
                "ld a, c",
                "ld [wFarCallBC + 1], a",
                "pop bc",
                "ld a, b",
                "rst Bankswitch",
                "ld a, [wFarCallBC]",
                "ld b, a",
                "ld a, [wFarCallBC + 1]",
                "ld c, a",
                "ret",
            ],
        ),
    ] {
        let source = super::Source::read(vendor, file)?;
        let actual = source
            .body(label)?
            .iter()
            .skip(1)
            .take_while(|(_, line)| !line.starts_with("SECTION "))
            .filter(|(_, line)| !line.starts_with('.'))
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>();
        ensure!(
            actual == expected,
            "intro farcall cycle-model source changed: {label}"
        );
    }
    let macros = super::Source::read(vendor, "macros/farcall.asm")?;
    let start = macros
        .lines
        .iter()
        .position(|(_, line)| line == "MACRO farcall")
        .context("intro farcall macro missing")?;
    let actual = macros.lines[start + 1..]
        .iter()
        .take_while(|(_, line)| line != "ENDM")
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>();
    ensure!(
        actual == ["ld a, BANK(\\1)", "ld hl, \\1", "rst FarCall"],
        "intro farcall macro differs from its instruction model"
    );
    Ok(())
}

/// Plain SM83 source instructions retain their boundaries. Conditional control
/// transfers have distinct taken and fallthrough paths; no ROM frame readings
/// participate in this model.
pub(super) fn plain_instruction_cycles(instruction: &str, taken: bool) -> Option<Vec<u64>> {
    let registers = ["a", "b", "c", "d", "e", "h", "l"];
    if instruction.starts_with("assert ")
        || instruction.starts_with('.')
        || instruction.ends_with(':')
    {
        return Some(vec![]);
    }
    let cost = if instruction == "ret" {
        4
    } else if instruction.starts_with("ret ") {
        if taken { 5 } else { 2 }
    } else if instruction.starts_with("jr ") {
        if instruction.contains(", ") && !taken {
            2
        } else {
            3
        }
    } else if instruction == "jp hl" {
        1
    } else if instruction.starts_with("jp ") {
        if instruction.contains(", ") && !taken {
            3
        } else {
            4
        }
    } else if instruction.starts_with("push ") {
        4
    } else if instruction.starts_with("pop ") {
        3
    } else if instruction.starts_with("ldh ") {
        3
    } else if instruction.starts_with("lb ")
        || instruction.starts_with("hlcoord ")
        || instruction.starts_with("debgcoord ")
        || instruction.starts_with("decoord ")
        || instruction.starts_with("depixel ")
    {
        3
    } else if let Some(raw) = instruction.strip_prefix("ld ") {
        let (destination, operand) = raw.split_once(", ")?;
        if ["bc", "de", "hl", "sp"].contains(&destination) {
            3
        } else if destination.starts_with('[') || operand.starts_with('[') {
            if ["[hl]", "[hli]", "[hld]", "[de]", "[bc]"].contains(&destination)
                || ["[hl]", "[hli]", "[hld]", "[de]", "[bc]"].contains(&operand)
            {
                if destination == "[hl]" && !registers.contains(&operand) {
                    3
                } else {
                    2
                }
            } else {
                4
            }
        } else if registers.contains(&destination) {
            if registers.contains(&operand) { 1 } else { 2 }
        } else {
            return None;
        }
    } else if let Some(operand) = instruction
        .strip_prefix("inc ")
        .or_else(|| instruction.strip_prefix("dec "))
    {
        if operand == "[hl]" {
            3
        } else if ["bc", "de", "hl", "sp"].contains(&operand) {
            2
        } else {
            1
        }
    } else if instruction.starts_with("bit ")
        || instruction.starts_with("set ")
        || instruction.starts_with("res ")
    {
        if instruction.ends_with(", [hl]") {
            if instruction.starts_with("bit ") {
                3
            } else {
                4
            }
        } else {
            2
        }
    } else if ["sla ", "srl ", "sra ", "swap ", "rl ", "rr "]
        .iter()
        .any(|prefix| instruction.starts_with(prefix))
    {
        2
    } else if instruction.starts_with("add hl, ") {
        2
    } else if let Some(operand) = ["and ", "or ", "xor ", "cp ", "sub ", "add ", "adc "]
        .iter()
        .find_map(|prefix| instruction.strip_prefix(prefix))
    {
        let operand = operand.strip_prefix("a, ").unwrap_or(operand);
        if registers.contains(&operand) { 1 } else { 2 }
    } else if ["nop", "di", "ei", "rra", "rlca", "rrca", "rla"].contains(&instruction) {
        1
    } else if instruction == "call NextIntroScene" {
        return Some(vec![6, 3, 3, 4]);
    } else if instruction == "call Intro_SetCGBPalUpdate" {
        return Some(vec![6, 2, 3, 4]);
    } else {
        return None;
    };
    Some(vec![cost])
}

fn straight_helper_tape(source: &super::Source, routine: &str) -> Result<Vec<u64>> {
    let body = source.body(routine)?;
    let mut tape = Vec::new();
    let mut repeat = None;
    let mut repeated = Vec::new();
    for (_, line) in body.iter().skip(1) {
        if line.starts_with('.') {
            break;
        }
        if let Some(count) = line.strip_prefix("rept ") {
            ensure!(
                repeat.is_none(),
                "intro timing has nested straight-helper repetition"
            );
            repeat = Some(count.parse::<usize>()?);
            continue;
        }
        if line == "endr" {
            for _ in 0..repeat
                .take()
                .context("intro timing repetition end without start")?
            {
                tape.extend_from_slice(&repeated);
            }
            repeated.clear();
            continue;
        }
        let cost = plain_instruction_cycles(line, false)
            .with_context(|| format!("unmodeled straight helper {routine}: {line}"))?;
        ensure!(
            cost == plain_instruction_cycles(line, true).context("straight helper taken cost")?,
            "straight helper has a conditional instruction: {routine}"
        );
        if repeat.is_some() {
            repeated.extend(cost);
        } else {
            tape.extend(cost);
        }
    }
    ensure!(
        repeat.is_none() && tape.last() == Some(&4),
        "straight helper has no final RET: {routine}"
    );
    Ok(tape)
}

fn branching_helper_tape(
    source: &super::Source,
    routine: &str,
    decisions: &[(&str, bool)],
) -> Result<Vec<u64>> {
    let body = source.body(routine)?;
    let mut tape = Vec::new();
    let mut index = 1usize;
    let mut byte_count = 0u16;
    for _ in 0..1000 {
        let (_, line) = body
            .get(index)
            .context("intro branching helper leaves source body")?;
        index += 1;
        if line.starts_with('.') {
            continue;
        }
        if let Some(value) = line.strip_prefix("ld bc, ") {
            byte_count = if let Some(hex) = value.strip_prefix('$') {
                u16::from_str_radix(hex, 16)?
            } else if let Some(palettes) = value.strip_suffix(" palettes") {
                palettes.parse::<u16>()? * 8
            } else {
                value.parse()?
            };
        }
        if line == "call ByteFill" || line == "call CopyBytes" {
            tape.push(6);
            tape.extend(byte_loop_instruction_machine_cycles(
                line.strip_prefix("call ").unwrap(),
                byte_count,
            )?);
            continue;
        }
        let conditional =
            (line.starts_with("jr ") && line.contains(", ")) || line.starts_with("ret ");
        let taken = if conditional {
            decisions
                .iter()
                .find(|(instruction, _)| instruction == &line.as_str())
                .with_context(|| format!("intro helper branch has no source decision: {line}"))?
                .1
        } else {
            true
        };
        tape.extend(
            plain_instruction_cycles(line, taken).with_context(|| {
                format!("intro branching helper has unknown instruction: {line}")
            })?,
        );
        if line == "ret" || (line.starts_with("ret ") && taken) {
            return Ok(tape);
        }
        if taken && line.starts_with("jr ") {
            let target = line
                .split_once(", ")
                .map(|(_, target)| target)
                .unwrap_or(&line[3..]);
            index = body
                .iter()
                .position(|(_, label)| label.trim_end_matches(':') == target)
                .with_context(|| format!("intro helper branch target missing: {target}"))?;
        }
    }
    anyhow::bail!("intro branching helper exceeded bounded source instructions")
}

fn counted_helper_tape(source: &super::Source, routine: &str) -> Result<Vec<u64>> {
    let body = source.body(routine)?;
    let mut tape = Vec::new();
    let mut index = 1usize;
    let mut b = 0u8;
    let mut c = 0u8;
    for _ in 0..5000 {
        let (_, line) = body
            .get(index)
            .context("intro counted helper leaves source body")?;
        index += 1;
        if line.starts_with('.') {
            continue;
        }
        if let Some(value) = line.strip_prefix("ld b, ") {
            b = if value == "SCREEN_HEIGHT" {
                18
            } else {
                value.parse()?
            };
        }
        if let Some(value) = line.strip_prefix("ld c, ") {
            c = if value == "SCREEN_WIDTH" {
                20
            } else if value == "1 palettes" {
                8
            } else {
                value.parse()?
            };
        }
        if line == "dec b" {
            b = b.wrapping_sub(1);
        }
        if line == "dec c" {
            c = c.wrapping_sub(1);
        }
        let taken = if line == "jr nz, .col" || line == "jr nz, .loop2" {
            c != 0
        } else if line == "jr nz, .row" || line == "jr nz, .loop1" {
            b != 0
        } else if line.starts_with("jr ") {
            anyhow::bail!("unknown counted helper branch: {line}")
        } else {
            false
        };
        tape.extend(
            plain_instruction_cycles(line, taken)
                .with_context(|| format!("unknown counted helper instruction: {line}"))?,
        );
        if line == "ret" {
            return Ok(tape);
        }
        if taken {
            let target = line
                .split_once(", ")
                .context("counted helper branch target")?
                .1;
            index = body
                .iter()
                .position(|(_, label)| label.trim_end_matches(':') == target)
                .context("counted helper label")?;
        }
    }
    anyhow::bail!("counted intro helper exceeded instruction bound")
}

fn attach_plain_instruction_costs(vendor: &Path, operations: &mut [Value]) -> Result<()> {
    let source = super::Source::read(vendor, "engine/movie/intro.asm")?;
    // These two expanded direct calls are certified against their exact bodies.
    for (routine, expected) in [
        (
            "NextIntroScene",
            vec!["ld hl, wJumptableIndex", "inc [hl]", "ret"],
        ),
        (
            "Intro_SetCGBPalUpdate",
            vec!["ld a, TRUE", "ldh [hCGBPalUpdate], a", "ret"],
        ),
    ] {
        let body = source.body(routine)?;
        ensure!(
            body.iter()
                .skip(1)
                .map(|(_, line)| line.as_str())
                .collect::<Vec<_>>()
                == expected,
            "intro plain helper source changed: {routine}"
        );
    }
    let audio=super::Source::read(vendor,"home/audio.asm")?;
    ensure!(audio.body("SFXChannelsOff")?.iter().skip(1).map(|(_,line)|line.as_str()).collect::<Vec<_>>()
        ==["xor a","ld [wChannel5Flags1], a","ld [wChannel6Flags1], a","ld [wChannel7Flags1], a","ld [wChannel8Flags1], a","ld [wPitchSweep], a","ret"],
        "intro scheduled SFXChannelsOff instruction-model source changed");
    let mut previous_scene_end = 0u64;
    for operation in operations {
        let span = operation["source_span"].clone();
        let Some(file) = span["file"].as_str() else {
            continue;
        };
        if !["engine/movie/intro.asm", "home/tilemap.asm"].contains(&file) {
            continue;
        }
        let other;
        let current = if file == source.file {
            &source
        } else {
            other = super::Source::read(vendor, file)?;
            &other
        };
        let (Some(start), Some(end)) = (span["start_line"].as_u64(), span["end_line"].as_u64())
        else {
            continue;
        };
        let is_scene = file == source.file
            && source
                .lines
                .iter()
                .filter(|(line, _)| u64::from(*line) <= start)
                .rev()
                .find(|(_, line)| line.ends_with(':') && !line.starts_with('.'))
                .is_some_and(|(_, label)| {
                    label.starts_with("IntroScene") && label != "IntroScenes:"
                });
        if is_scene && end == previous_scene_end {
            // A lowering can emit a second semantic alias for the same source
            // result. That alias executes no second copy of the instructions.
            operation["source_instruction_machine_cycles"] = json!([]);
            continue;
        }
        let scene_start = source
            .lines
            .iter()
            .filter(|(line, _)| u64::from(*line) < start)
            .rev()
            .find(|(_, label)| label.starts_with("IntroScene") && label.ends_with(':'))
            .map(|(line, _)| u64::from(*line))
            .unwrap_or(start.saturating_sub(1));
        let from = if !is_scene {
            start
        } else if previous_scene_end >= scene_start && previous_scene_end < start {
            previous_scene_end + 1
        } else {
            scene_start + 1
        };
        if is_scene {
            previous_scene_end = end;
        }
        if operation.get("source_helper_timing").is_some() {
            continue;
        }
        let lines = current
            .lines
            .iter()
            .filter(|(line, _)| (from..=end).contains(&u64::from(*line)));
        let mut normal = Vec::new();
        let mut taken = Vec::new();
        let mut complete = true;
        for (_, line) in lines {
            if line == "call PlayMusic" || line == "call PlaySFX" {
                let mut caller = normal.clone();
                caller.push(6);
                operation["source_audio_caller_instruction_machine_cycles"] = json!(caller);
                complete = false;
                break;
            }
            if line == "call .PlayUnownSound" {
                let mut caller = normal.clone();
                caller.push(6);
                operation["source_scheduled_audio_caller_instruction_machine_cycles"] =
                    json!(caller);
                operation["source_scheduled_audio_timing"] = json!({"routine":"IntroScene12.PlayUnownSound",
                    "source_file":"engine/movie/intro.asm","model":"source_table_search_v1","entry_count":8,
                    "caller_call_included":false,"interrupts_included":false});
                complete = false;
                break;
            }
            if line == "call Intro_ColoredSuicuneFrameSwap"
                || line == "call Intro_Scene16_AnimateSuicune"
            {
                let mut caller = normal.clone();
                caller.push(6);
                operation["source_tilemap_timing"] = json!({"routine":line.strip_prefix("call ").unwrap(),
                    "source_file":"engine/movie/intro.asm","model":"tile_byte_branch_counter_v1",
                    "target":"wTilemap","byte_count":360,"caller_call_included":false,"interrupts_included":false});
                operation["source_tilemap_caller_instruction_machine_cycles"] = json!(caller);
                complete = false;
                break;
            }
            if line == "call Intro_RustleGrass" {
                let mut caller = normal.clone();
                caller.push(6);
                let variants = [false,true].into_iter().map(|finished| {
                    let mut tape = caller.clone();
                    tape.extend(branching_helper_tape(&source,"Intro_RustleGrass",&[("ret nc",finished)])?);
                    Ok(json!({"predicate":if finished {"unsigned_greater_or_equal"} else {"unsigned_less_than"},"operand":36,"instruction_machine_cycles":tape}))
                }).collect::<Result<Vec<_>>>()?;
                operation["source_instruction_timing_variants"] = json!({"selector":{"source":"wIntroSceneFrameCounter","mask":255},"variants":variants});
                complete = false;
                break;
            }
            if line == "call Intro_Scene20_AppearUnown" {
                let mut caller = normal.clone();
                caller.push(6);
                let variants = [false,true].into_iter().map(|nonzero| {
                    let mut tape = caller.clone();
                    tape.extend(branching_helper_tape(&source,"Intro_Scene20_AppearUnown",&[("jr nz, .load_pal_2",nonzero)])?);
                    Ok(json!({"predicate":if nonzero {"not_equal"} else {"equal"},"operand":0,"instruction_machine_cycles":tape}))
                }).collect::<Result<Vec<_>>>()?;
                operation["source_instruction_timing_variants"] = json!({"selector":{"source":"accumulator","scope":"values","mask":255},"variants":variants});
                complete = false;
                break;
            }
            if line == "call Intro_PerspectiveScrollBG" || line == "call CrystalIntro_UnownFade" {
                let mut caller = normal.clone();
                caller.push(6);
                let variants = if line == "call Intro_PerspectiveScrollBG" {
                    let entries = (0..=1)
                        .map(|parity| {
                            let mut tape = caller.clone();
                            tape.extend(branching_helper_tape(
                                &source,
                                "Intro_PerspectiveScrollBG",
                                &[("jr z, .skip", parity == 0)],
                            )?);
                            Ok(json!({"equals":parity,"instruction_machine_cycles":tape}))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    json!({"selector":{"source":"wIntroSceneFrameCounter","mask":1},"variants":entries})
                } else {
                    let entries = [("unsigned_less_than",31,false,true),("equals",31,true,false),("unsigned_greater_than",31,false,false)].into_iter().map(|(predicate,operand,equal,below)| {
                        let mut tape = caller.clone();
                        tape.extend(branching_helper_tape(&source,"CrystalIntro_UnownFade",&[("jr z, .okay",equal),("jr c, .okay",below)])?);
                        Ok(json!({"predicate":predicate,"operand":operand,"instruction_machine_cycles":tape}))
                    }).collect::<Result<Vec<_>>>()?;
                    json!({"selector":{"source":"wIntroSceneTimer","mask":63},"variants":entries})
                };
                operation["source_instruction_timing_variants"] = variants;
                complete = false;
                break;
            }
            if line == "call Intro_LoadTilemap" || line == "call Intro_Scene24_ApplyPaletteFade" {
                let helper = counted_helper_tape(&source, line.strip_prefix("call ").unwrap())?;
                normal.push(6);
                taken.push(6);
                normal.extend_from_slice(&helper);
                taken.extend(helper);
                continue;
            }
            if line == "call Intro_FadeUnownWordPals" {
                let helper = straight_helper_tape(&source, "Intro_FadeUnownWordPals")?;
                normal.push(6);
                taken.push(6);
                normal.extend_from_slice(&helper);
                taken.extend(helper);
                continue;
            }
            let (Some(a), Some(b)) = (
                plain_instruction_cycles(line, false),
                plain_instruction_cycles(line, true),
            ) else {
                complete = false;
                break;
            };
            normal.extend(a);
            taken.extend(b);
        }
        if complete {
            if file == "home/tilemap.asm" && operation["op"] == "palette_transfer_request" {
                // This effect returns from the CGB ClearPalettes branch before
                // falling through ClearBGPalettes into WaitBGMap.
                let last = current
                    .lines
                    .iter()
                    .find(|(line, _)| u64::from(*line) == end);
                if !last.is_some_and(|(_, line)| line == "ret") {
                    let following = current
                        .lines
                        .iter()
                        .find(|(line, _)| u64::from(*line) > end);
                    ensure!(
                        following.is_some_and(|(_, line)| line == "ret"),
                        "intro palette clear return source changed at {file}:{start}..{end}; last {last:?}, following {following:?}"
                    );
                    normal.push(4);
                    taken.push(4);
                }
            }
            operation["source_instruction_machine_cycles"] = json!(normal);
            if normal != taken {
                operation["source_instruction_machine_cycles_taken"] = json!(taken);
            }
            operation["source_instruction_span"] =
                json!({"file":current.file,"start_line":from,"end_line":end});
        }
    }
    Ok(())
}

fn attach_byte_loop_costs(
    vendor: &Path,
    operations: &mut [Value],
    symbols: &std::collections::BTreeMap<String, (usize, u16)>,
) -> Result<()> {
    certify_farcall_source(vendor)?;
    for (file, routine, expected) in [
        (
            "engine/movie/intro.asm",
            "Intro_ClearBGPals",
            vec![
                "ldh a, [rWBK]",
                "push af",
                "ld a, BANK(wBGPals2)",
                "ldh [rWBK], a",
                "ld hl, wBGPals2",
                "ld bc, 16 palettes",
                "xor a",
                "call ByteFill",
                "pop af",
                "ldh [rWBK], a",
                "ld a, TRUE",
                "ldh [hCGBPalUpdate], a",
                "call DelayFrame",
                "call DelayFrame",
                "ret",
            ],
        ),
        (
            "engine/movie/intro.asm",
            "Intro_ResetLYOverrides",
            vec![
                "ldh a, [rWBK]",
                "push af",
                "ld a, BANK(wLYOverrides)",
                "ldh [rWBK], a",
                "ld hl, wLYOverrides",
                "ld bc, wLYOverridesEnd - wLYOverrides",
                "xor a",
                "call ByteFill",
                "pop af",
                "ldh [rWBK], a",
                "ld a, LOW(rSCX)",
                "ldh [hLCDCPointer], a",
                "ret",
            ],
        ),
        (
            "home/text.asm",
            "ClearTilemap",
            vec![
                "hlcoord 0, 0",
                "ld a, ' '",
                "ld bc, wTilemapEnd - wTilemap",
                "call ByteFill",
                "ldh a, [rLCDC]",
                "bit B_LCDC_ENABLE, a",
                "ret z",
                "jp WaitBGMap",
            ],
        ),
        (
            "home/tilemap.asm",
            "ClearPalettes",
            vec![
                "ldh a, [hCGB]",
                "and a",
                "jr nz, .cgb",
                "xor a",
                "ldh [rBGP], a",
                "ldh [rOBP0], a",
                "ldh [rOBP1], a",
                "ret",
                "ldh a, [rWBK]",
                "push af",
                "ld a, BANK(wBGPals2)",
                "ldh [rWBK], a",
                "ld hl, wBGPals2",
                "ld bc, 16 palettes",
                "ld a, $ff",
                "call ByteFill",
                "pop af",
                "ldh [rWBK], a",
                "ld a, TRUE",
                "ldh [hCGBPalUpdate], a",
                "ret",
            ],
        ),
    ] {
        let source = super::Source::read(vendor, file)?;
        let body = source.body(routine)?;
        let actual = body
            .iter()
            .skip(1)
            .filter(|(_, line)| !line.starts_with('.'))
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>();
        ensure!(
            actual == expected,
            "intro split helper source changed: {routine}"
        );
    }
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
            "call Intro_ClearBGPals"
            | "call Intro_ResetLYOverrides"
            | "call ClearBGPalettes"
            | "call ClearTilemap"
                if operation["op"] == "fill_memory" =>
            {
                instruction.strip_prefix("call ").unwrap()
            }
            "call ClearSpriteAnims" | "farcall ClearSpriteAnims"
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
        let split = [
            "Intro_ClearBGPals",
            "Intro_ResetLYOverrides",
            "ClearBGPalettes",
            "ClearTilemap",
        ]
        .contains(&routine);
        let clear = routine.starts_with("Clear") && !split;
        let machine_cycles = if split {
            let surrounding = match routine {
                "Intro_ClearBGPals" | "Intro_ResetLYOverrides" => 31,
                "ClearBGPalettes" => 45,
                "ClearTilemap" => 25,
                _ => unreachable!(),
            };
            byte_loop_machine_cycles("ByteFill", bytes)? + surrounding
        } else if clear {
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
        let farcall = instruction.starts_with("farcall ");
        while !farcall && caller_start > 0 {
            let Some(cost) = register_setup_machine_cycles(&source.lines[caller_start - 1].1)
            else {
                break;
            };
            caller_start -= 1;
            prefix.push(cost);
        }
        if farcall {
            let (before, after) =
                crystal_assets::runtime_intro_farcall_instruction_machine_cycles();
            prefix = before;
            operation["source_caller_model"] = json!("farcall_hl_v1");
            operation["source_return_instruction_machine_cycles"] = json!(after);
        } else {
            prefix.reverse();
            prefix.push(6); // Actual direct CALL, separate from the helper body's RET.
        }
        operation["source_caller_instruction_machine_cycles"] = json!(prefix);
        operation["source_caller_span"] =
            json!({"file":source.file,"start_line":source.lines[caller_start].0,"end_line":end});
        operation["source_helper_timing"] = json!({
            "routine":routine,
            "byte_count":bytes,
            "machine_cycles":machine_cycles,
            "caller_call_included":false,
            "interrupts_included":false,
            "source_file":match routine {"ClearSprites"=>"home/clear_sprites.asm","ClearSpriteAnims"=>"engine/sprite_anims/core.asm","Intro_ClearBGPals"|"Intro_ResetLYOverrides"=>"engine/movie/intro.asm","ClearBGPalettes"=>"home/tilemap.asm","ClearTilemap"=>"home/text.asm",_=>"home/copy.asm"},
            "model":if split {"split_memory_effect_v1"} else if clear {"clear_register_counter_loop_v1"} else {"byte_count_and_counter_rollover_v1"}
        });
    }
    Ok(())
}

#[cfg(test)]
mod byte_loop_cost_tests {
    use super::*;

    #[test]
    fn plain_scene_taken_and_fallthrough_tapes_match_independent_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for (routine, frame, lines, taken) in [
            (
                "IntroScene27",
                128,
                vec![
                    "ld hl, wIntroSceneTimer",
                    "inc [hl]",
                    "ld hl, wIntroSceneFrameCounter",
                    "ld a, [hl]",
                    "inc [hl]",
                    "cp $80",
                    "jr nc, .done",
                    "call NextIntroScene",
                    "ld a, $80",
                    "ld [wIntroSceneFrameCounter], a",
                    "ret",
                ],
                true,
            ),
            (
                "IntroScene28",
                12,
                vec![
                    "ld hl, wIntroSceneFrameCounter",
                    "ld a, [hl]",
                    "and a",
                    "jr z, .done",
                    "dec [hl]",
                    "cp $18",
                    "jr z, .clear",
                    "cp $8",
                    "ret nz",
                ],
                false,
            ),
        ] {
            let mut expected = Vec::new();
            for line in lines {
                expected.extend(plain_instruction_cycles(line, taken || line == "ret nz").unwrap());
            }
            let measured = source_intro_oracle::measure_routine(
                vendor,
                routine,
                &[("wIntroSceneFrameCounter".into(), frame)],
            )
            .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(expected, actual, "{routine}");
        }
        assert!(plain_instruction_cycles("call UnmodeledHelper", false).is_none());
        assert_eq!(
            plain_instruction_cycles("jr nc, .done", false),
            Some(vec![2])
        );
        assert_eq!(
            plain_instruction_cycles("jr nc, .done", true),
            Some(vec![3])
        );
    }

    #[test]
    fn split_memory_helper_tapes_match_independent_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for (routine, count, surrounding, writes, suffix) in [
            ("Intro_ResetLYOverrides", 144, 31, vec![], vec![2, 3, 4]),
            (
                "ClearBGPalettes",
                128,
                45,
                vec![("hCGB".into(), 1)],
                vec![2, 3, 4],
            ),
            ("ClearTilemap", 360, 25, vec![("$ff40".into(), 0)], vec![]),
        ] {
            let descriptor = json!({"routine":routine,"byte_count":count,
                "machine_cycles":byte_loop_machine_cycles("ByteFill",count).unwrap()+surrounding,
                "source_file":match routine {"ClearBGPalettes"=>"home/tilemap.asm","ClearTilemap"=>"home/text.asm",_=>"engine/movie/intro.asm"},
                "model":"split_memory_effect_v1","caller_call_included":false,"interrupts_included":false});
            let mut tape =
                crystal_assets::runtime_intro_helper_instruction_machine_cycles(&descriptor)
                    .unwrap();
            let measured_routine = if routine == "ClearBGPalettes" {
                tape.remove(0);
                "ClearPalettes"
            } else {
                routine
            };
            if routine == "ClearTilemap" {
                // The fixture's LCD is off: RET Z is taken, no WaitBGMap JP.
                tape.truncate(tape.len() - 2);
                tape.push(5);
            }
            tape.extend(suffix);
            let measured =
                source_intro_oracle::measure_routine(vendor, measured_routine, &writes).unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(tape, actual, "{routine}");
        }
    }

    #[test]
    fn straight_word_palette_helper_matches_sm83_instruction_boundaries() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = super::super::Source::read(vendor, "engine/movie/intro.asm").unwrap();
        let tape = straight_helper_tape(&source, "Intro_FadeUnownWordPals").unwrap();
        for (palette, timer) in [(0, 0), (4, 8), (7, 15)] {
            let measured = source_intro_oracle::measure_routine_state(
                vendor,
                "Intro_FadeUnownWordPals",
                &[("wIntroSceneTimer".into(), timer)],
                &[("A", palette)],
            )
            .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(tape, actual, "palette {palette} timer {timer}");
        }
    }

    #[test]
    fn lz3_source_counter_and_command_paths_match_independent_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let symbols = source_intro_oracle::symbols(
            &std::fs::read_to_string(vendor.join("pokecrystal.sym")).unwrap(),
        )
        .unwrap();
        for command in 0u8..=6 {
            for count in [1usize, 2, 31, 32, 255, 256, 257, 512, 768] {
                for negative in [false, true] {
                    let mut compressed = vec![0, 0x53]; // Seed output for rewrite commands.
                    let encoded = count - 1;
                    if count <= 32 {
                        compressed.push((command << 5) | encoded as u8);
                    } else {
                        compressed.extend([
                            0xe0 | (command << 2) | ((encoded >> 8) as u8),
                            encoded as u8,
                        ]);
                    }
                    match command {
                        0 => compressed.extend(std::iter::repeat_n(0x53, count)),
                        1 => compressed.push(0x53),
                        2 => compressed.extend([0x53, 0xaa]),
                        3 => (),
                        _ if negative => compressed.push(0x80),
                        _ => compressed.extend([0, 0]),
                    }
                    compressed.push(255);
                    let predicted =
                        crystal_assets::runtime_intro_decompress_instruction_machine_cycles(
                            &compressed,
                        )
                        .unwrap();
                    let writes = compressed
                        .iter()
                        .enumerate()
                        .map(|(offset, byte)| (format!("wShadowOAM + {offset}"), *byte))
                        .collect::<Vec<_>>();
                    let measured = source_intro_oracle::measure_routine_state(
                        vendor,
                        "Decompress",
                        &writes,
                        &[("HL", symbols["wShadowOAM"].1), ("DE", 0xd000)],
                    )
                    .unwrap();
                    let actual = measured["instructions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(
                        predicted, actual,
                        "command {command} count {count} negative {negative}"
                    );
                }
            }
        }
    }

    #[test]
    fn request2bpp_blocking_segments_match_source_cpu_control() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for count in [0u8, 1, 7, 8, 16, 31, 64, 128, 255] {
            let segments =
                crystal_assets::runtime_intro_request2bpp_instruction_segments(count, 8).unwrap();
            let expected = segments
                .iter()
                .flat_map(|(_, before, after)| before.iter().chain(after))
                .copied()
                .collect::<Vec<_>>();
            let measured = source_intro_oracle::measure_routine_with_frame_wait_fixture(
                vendor,
                "Request2bpp",
                &[("hROMBank".into(), 1), ("wLinkMode".into(), 0)],
                &[
                    ("BC", 0x100 | u16::from(count)),
                    ("HL", 0x8000),
                    ("DE", 0xc000),
                ],
            )
            .unwrap();
            let waits = measured["frame_waits"].as_array().unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .filter(|(index, _)| {
                    !waits.iter().any(|wait| {
                        let start = wait["start_instruction"].as_u64().unwrap() as usize;
                        let end = wait["end_instruction"].as_u64().unwrap() as usize;
                        (*index >= start) && (*index < end)
                    })
                })
                .map(|(_, instruction)| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(expected, actual, "tiles {count}");
            assert_eq!(
                segments.len(),
                waits.len(),
                "tiles {count} source wait count"
            );
        }
        // Certify every split source body before its timing descriptor is used.
        attach_byte_loop_costs(vendor, &mut [], &std::collections::BTreeMap::new()).unwrap();
    }

    #[test]
    fn branching_palette_and_scroll_helpers_match_independent_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = super::super::Source::read(vendor, "engine/movie/intro.asm").unwrap();
        for frame in [0, 1, 128, 255] {
            let predicted = branching_helper_tape(
                &source,
                "Intro_PerspectiveScrollBG",
                &[("jr z, .skip", frame & 1 == 0)],
            )
            .unwrap();
            let measured = source_intro_oracle::measure_routine(
                vendor,
                "Intro_PerspectiveScrollBG",
                &[("wIntroSceneFrameCounter".into(), frame)],
            )
            .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(predicted, actual, "scroll frame {frame}");
        }
        for timer in [0, 30, 31, 32, 63, 64, 128, 255] {
            let value = timer & 63;
            let predicted = branching_helper_tape(
                &source,
                "CrystalIntro_UnownFade",
                &[("jr z, .okay", value == 31), ("jr c, .okay", value < 31)],
            )
            .unwrap();
            let measured = source_intro_oracle::measure_routine(
                vendor,
                "CrystalIntro_UnownFade",
                &[("wIntroSceneTimer".into(), timer)],
            )
            .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(predicted, actual, "fade timer {timer}");
        }
    }

    #[test]
    fn counted_tilemap_and_palette_helpers_match_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = super::super::Source::read(vendor, "engine/movie/intro.asm").unwrap();
        for routine in ["Intro_LoadTilemap", "Intro_Scene24_ApplyPaletteFade"] {
            let predicted = counted_helper_tape(&source, routine).unwrap();
            let measured =
                source_intro_oracle::measure_routine_state(vendor, routine, &[], &[("A", 0)])
                    .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(predicted, actual, "{routine}");
        }
    }

    #[test]
    fn indexed_palette_copy_paths_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = super::super::Source::read(vendor, "engine/movie/intro.asm").unwrap();
        for variant in [0u16, 1, 255] {
            let predicted = branching_helper_tape(
                &source,
                "Intro_Scene20_AppearUnown",
                &[("jr nz, .load_pal_2", variant != 0)],
            )
            .unwrap();
            let measured = source_intro_oracle::measure_routine_state(
                vendor,
                "Intro_Scene20_AppearUnown",
                &[("wIntroSceneTimer".into(), 7)],
                &[("A", variant)],
            )
            .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(predicted, actual, "palette variant {variant}");
        }
    }

    #[test]
    fn grass_request_and_early_return_paths_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let source = super::super::Source::read(vendor, "engine/movie/intro.asm").unwrap();
        for frame in [0, 1, 35, 36, 255] {
            let predicted =
                branching_helper_tape(&source, "Intro_RustleGrass", &[("ret nc", frame >= 36)])
                    .unwrap();
            let measured = source_intro_oracle::measure_routine(
                vendor,
                "Intro_RustleGrass",
                &[("wIntroSceneFrameCounter".into(), frame)],
            )
            .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(predicted, actual, "grass frame {frame}");
        }
    }

    #[test]
    fn tilemap_xor_byte_and_frame_branches_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for fill in [
            vec![0u8; 360],
            vec![1; 360],
            vec![128; 360],
            (0..360).map(|i| [0, 1, 127, 128, 255][i % 5]).collect(),
        ] {
            for frame in [None, Some(0u8), Some(1), Some(2), Some(3), Some(255)] {
                let predicted =
                    crystal_assets::runtime_intro_tilemap_xor_instruction_machine_cycles(
                        &fill, frame,
                    )
                    .unwrap();
                let routine = if frame.is_some() {
                    "Intro_Scene16_AnimateSuicune"
                } else {
                    "Intro_ColoredSuicuneFrameSwap"
                };
                let mut writes = fill
                    .iter()
                    .enumerate()
                    .map(|(i, &value)| (format!("wTilemap + {i}"), value))
                    .collect::<Vec<_>>();
                if let Some(frame) = frame {
                    writes.push(("wIntroSceneFrameCounter".into(), frame));
                }
                let measured =
                    source_intro_oracle::measure_routine(vendor, routine, &writes).unwrap();
                let actual = measured["instructions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(predicted, actual, "{routine} frame {frame:?}");
            }
        }
    }

    #[test]
    fn sprite_sine_macro_sign_and_amplitude_paths_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for (routine, cosine, wrapper) in [
            ("Sprites_Sine", false, false),
            ("Sprites_Cosine", true, false),
            ("AnimSeqs_Sine", false, true),
            ("AnimSeqs_Cosine", true, true),
        ] {
            for angle in [0u8, 15, 16, 31, 32, 47, 48, 63, 255] {
                for amplitude in [0u8, 1, 2, 32, 127, 128, 255] {
                    let predicted =
                        crystal_assets::runtime_intro_sprite_sine_instruction_machine_cycles(
                            cosine, angle, amplitude, wrapper,
                        );
                    let measured = source_intro_oracle::measure_routine_state(
                        vendor,
                        routine,
                        &[],
                        &[("A", u16::from(angle)), ("DE", u16::from(amplitude) << 8)],
                    )
                    .unwrap();
                    let actual = measured["instructions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(
                        predicted, actual,
                        "{routine} angle {angle} amplitude {amplitude}"
                    );
                }
            }
        }
    }

    #[test]
    fn sprite_allocator_capacity_and_counter_paths_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for occupied in 0u8..=10 {
            for count in [0u8, 1, 254, 255] {
                let predicted =
                    crystal_assets::runtime_intro_sprite_init_instruction_machine_cycles(
                        occupied, count,
                    )
                    .unwrap();
                let mut writes = (0..occupied)
                    .map(|slot| {
                        (
                            format!("wSpriteAnimationStructs + {}", u16::from(slot) * 16),
                            1,
                        )
                    })
                    .collect::<Vec<_>>();
                writes.extend([
                    ("wSpriteAnimCount".into(), count),
                    ("hROMBank".into(), 0x25),
                ]);
                let measured = source_intro_oracle::measure_routine_state(
                    vendor,
                    "InitSpriteAnimStruct",
                    &writes,
                    &[("A", 0x29), ("DE", 0x5858)],
                )
                .unwrap();
                let actual = measured["instructions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(predicted, actual, "occupied {occupied} count {count}");
            }
        }
    }

    #[test]
    fn unown_group_and_home_reinit_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let predicted = crystal_assets::runtime_intro_sprite_reinit_instruction_machine_cycles();
        let measured = source_intro_oracle::measure_routine_state(
            vendor,
            "ReinitSpriteAnimFrame",
            &[("hROMBank".into(), 0x39)],
            &[("BC", 0xc500), ("A", 0x49)],
        )
        .unwrap();
        let actual = measured["instructions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(predicted, actual, "home reinit");
        for occupied in 0u8..=6 {
            for count in [0u8, 254, 255] {
                let predicted =
                    crystal_assets::runtime_intro_sprite_group_instruction_machine_cycles(
                        [occupied, occupied + 1, occupied + 2, occupied + 3],
                        count,
                    )
                    .unwrap();
                let mut writes = (0..occupied)
                    .map(|slot| {
                        (
                            format!("wSpriteAnimationStructs + {}", u16::from(slot) * 16),
                            1,
                        )
                    })
                    .collect::<Vec<_>>();
                writes.extend([
                    ("wSpriteAnimCount".into(), count),
                    ("hROMBank".into(), 0x39),
                ]);
                let measured = source_intro_oracle::measure_routine_state(
                    vendor,
                    "CrystalIntro_InitUnownAnim",
                    &writes,
                    &[("DE", 0x5858)],
                )
                .unwrap();
                let actual = measured["instructions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(
                    predicted, actual,
                    "Unown group occupied {occupied} count {count}"
                );
            }
        }
    }

    #[test]
    fn sprite_frame_control_flip_and_oam_capacity_paths_match_source_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let root = vendor.parent().unwrap().parent().unwrap();
        let bundle: Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/sprite_anim_bundle.json")).unwrap(),
        )
        .unwrap();
        let constants =
            std::fs::read_to_string(vendor.join("constants/sprite_anim_constants.asm")).unwrap();
        let names = constants
            .lines()
            .filter_map(|line| {
                line.trim()
                    .split(';')
                    .next()
                    .unwrap()
                    .trim()
                    .strip_prefix("const ")
                    .filter(|name| name.starts_with("SPRITE_ANIM_FRAMESET_"))
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        for (id, name) in names
            .iter()
            .enumerate()
            .filter(|(_, name)| name.starts_with("SPRITE_ANIM_FRAMESET_INTRO_"))
        {
            let frames = bundle["framesets"][name]["steps"].as_array().unwrap();
            for before in std::iter::once(255u8).chain(0..frames.len() as u8) {
                for duration in [0u8, 1] {
                    if duration == 0 && before != 255 && usize::from(before) + 1 >= frames.len() {
                        continue;
                    }
                    if duration != 0
                        && (before == 255
                            || ["restart", "end"]
                                .contains(&frames[before as usize]["command"].as_str().unwrap()))
                    {
                        continue;
                    }
                    let mut frame = before;
                    let mut commands = Vec::new();
                    let terminal;
                    loop {
                        if duration == 0 {
                            frame = frame.wrapping_add(1);
                        }
                        let step = &frames[frame as usize];
                        let command = step["command"].as_str().unwrap();
                        commands.push(match command {
                            "restart" => 254,
                            "end" => 255,
                            "wait" => 253,
                            "delete" => 252,
                            _ => 0,
                        });
                        match command {
                            "restart" => frame = 255,
                            "end" => frame = frame.wrapping_sub(2),
                            _ => {
                                terminal = step;
                                break;
                            }
                        }
                        assert!(commands.len() < 64);
                    }
                    let flags = (terminal["attr_flags"].as_u64().unwrap() as u8) >> 1;
                    let count = terminal["oam_set"]
                        .as_str()
                        .map(|name| {
                            bundle["oam_sets"][name]["pieces"].as_array().unwrap().len() as u8
                        })
                        .unwrap_or(0);
                    for oam_addr in [0u8, 156, 160] {
                        let predicted =
                            crystal_assets::runtime_intro_sprite_update_instruction_machine_cycles(
                                duration, &commands, flags, count, oam_addr,
                            )
                            .unwrap();
                        let writes = vec![
                            ("wSpriteAnimationStructs + 1".into(), id as u8),
                            ("wSpriteAnimationStructs + 8".into(), duration),
                            ("wSpriteAnimationStructs + 10".into(), before),
                            ("wCurSpriteOAMAddr".into(), oam_addr),
                        ];
                        let measured = source_intro_oracle::measure_routine_state(
                            vendor,
                            "UpdateAnimFrame",
                            &writes,
                            &[("BC", 0xc314)],
                        )
                        .unwrap();
                        let actual = measured["instructions"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                            .collect::<Vec<_>>();
                        let mismatch = predicted.iter().zip(&actual).position(|(a, b)| a != b);
                        assert!(
                            predicted == actual,
                            "{name} before{before} duration{duration} oam{oam_addr} mismatch{mismatch:?} predicted{:?} actual{:?} lengths{}:{}",
                            mismatch.map(|i| predicted[i]),
                            mismatch.map(|i| actual[i]),
                            predicted.len(),
                            actual.len()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn sprite_scheduler_source_slot_order_and_oam_clear_match_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        for mask in [0u16, 1, 2, 0x155, 0x2aa, 0x3ff] {
            let callback = vec![4, 2, 5]; // IntroUnownF with source scene counter0.
            let update = crystal_assets::runtime_intro_sprite_update_instruction_machine_cycles(
                0,
                &[253],
                0,
                0,
                0,
            )
            .unwrap();
            let slots = (0..10)
                .map(|slot| {
                    if mask & (1 << slot) != 0 {
                        Some((callback.clone(), update.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let predicted =
                crystal_assets::runtime_intro_sprite_scheduler_instruction_machine_cycles(
                    &slots, None, 0,
                )
                .unwrap();
            let mut writes = vec![("wSlotsDelay".into(), 0)];
            for slot in 0..10 {
                if mask & (1 << slot) == 0 {
                    continue;
                }
                for (offset, value) in [(0, 1), (1, 0x3f), (2, 0x21), (10, 255)] {
                    writes.push((
                        format!("wSpriteAnimationStructs + {}", slot * 16 + offset),
                        value,
                    ));
                }
            }
            let measured =
                source_intro_oracle::measure_routine(vendor, "PlaySpriteAnimations", &writes)
                    .unwrap();
            let actual = measured["instructions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            let mismatch = predicted.iter().zip(&actual).position(|(a, b)| a != b);
            assert!(
                predicted == actual,
                "scheduler mask{mask:x} mismatch{mismatch:?} predicted{:?} actual{:?} lengths{}:{}",
                mismatch.map(|i| predicted[i]),
                mismatch.map(|i| actual[i]),
                predicted.len(),
                actual.len()
            );
        }
    }

    #[test]
    fn scheduled_unown_audio_search_and_nested_call_boundaries_match_sm83() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        let symbols = source_intro_oracle::symbols(
            &std::fs::read_to_string(vendor.join("pokecrystal.sym")).unwrap(),
        )
        .unwrap();
        for frame in [0u8, 1, 32, 64, 96, 128, 144, 160, 176, 177, 255] {
            let matched = [0, 32, 64, 96, 128, 144, 160, 176]
                .iter()
                .position(|&entry| entry == frame);
            let (prefix, suffix) =
                crystal_assets::runtime_intro_scheduled_audio_instruction_machine_cycles(
                    matched, 8,
                )
                .unwrap();
            let measured = source_intro_oracle::measure_routine(
                vendor,
                "IntroScene12.PlayUnownSound",
                &[
                    ("wIntroSceneFrameCounter".into(), frame),
                    ("hROMBank".into(), 0x39),
                ],
            )
            .unwrap();
            let instructions = measured["instructions"].as_array().unwrap();
            let actual = instructions
                .iter()
                .map(|instruction| instruction["machine_cycles"].as_u64().unwrap())
                .collect::<Vec<_>>();
            if matched.is_none() {
                assert_eq!(prefix, actual, "no sound frame{frame}");
            } else {
                let play = instructions
                    .iter()
                    .position(|instruction| {
                        instruction["bank"].as_u64() == Some(0)
                            && instruction["address"].as_u64()
                                == Some(u64::from(symbols["PlaySFX"].1))
                    })
                    .unwrap();
                assert_eq!(
                    prefix,
                    actual[..play],
                    "scheduled sound frame{frame} prefix"
                );
                assert_eq!(
                    suffix,
                    actual[actual.len() - suffix.len()..],
                    "scheduled sound frame{frame} suffix"
                );
            }
        }
    }

    #[test]
    fn source_farcall_clear_cost_matches_the_independent_rom_instructions() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        certify_farcall_source(vendor).unwrap();
        let symbols = source_intro_oracle::symbols(
            &std::fs::read_to_string(vendor.join("pokecrystal.sym")).unwrap(),
        )
        .unwrap();
        let (bank, address) = symbols["ClearSpriteAnims"];
        let measured = source_intro_oracle::measure_routine_state(
            vendor,
            "FarCall",
            &[("hROMBank".into(), 1)],
            &[("A", bank as u16), ("HL", address)],
        )
        .unwrap();
        let (prefix, suffix) = crystal_assets::runtime_intro_farcall_instruction_machine_cycles();
        let body = clear_helper_instruction_machine_cycles("ClearSpriteAnims", 193).unwrap();
        // Entering at rst8 excludes the caller's LD/LD/RST macro (2+3+4).
        let predicted: u64 = prefix.iter().chain(&body).chain(&suffix).sum::<u64>() - 9;
        assert_eq!(predicted, 2407);
        assert_eq!(measured["machine_cycles"].as_u64(), Some(predicted));
    }

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
    let source = super::Source::read(vendor, "engine/movie/intro.asm")?;
    ensure!(
        source
            .body("IntroSceneJumper")?
            .iter()
            .skip(1)
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            == ["jumptable IntroScenes, wJumptableIndex"],
        "intro scene dispatcher source changed"
    );
    let macros = super::Source::read(vendor, "macros/code.asm")?;
    let start = macros
        .lines
        .iter()
        .position(|(_, line)| line == "MACRO jumptable")
        .context("intro dispatcher macro missing")?;
    let macro_body = macros.lines[start + 1..]
        .iter()
        .take_while(|(_, line)| line != "ENDM")
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>();
    ensure!(
        macro_body
            == [
                "ld a, [\\2]",
                "ld e, a",
                "ld d, 0",
                "ld hl, \\1",
                "add hl, de",
                "add hl, de",
                "ld a, [hli]",
                "ld h, [hl]",
                "ld l, a",
                "jp hl"
            ],
        "intro dispatcher instruction macro changed"
    );
    {
        use sha2::Digest;
        let digest = format!(
            "{:x}",
            sha2::Sha256::digest(std::fs::read(vendor.join("home/decompress.asm"))?)
        );
        ensure!(
            digest == "fb08952b65c7c834dfbe09b8a3d9c53832043f0635e8165ebcd42543ed1a9e9f",
            "intro LZ3 instruction-model source changed"
        );
        let gfx_digest = format!(
            "{:x}",
            sha2::Sha256::digest(std::fs::read(vendor.join("home/gfx.asm"))?)
        );
        ensure!(
            gfx_digest == "bf9d276ec0a3d8c800ddcbe26f10555abe79718c6c40a7dd2afd62e89ed52af2",
            "intro Request2bpp instruction-model source changed"
        );
    }
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
    attach_plain_instruction_costs(vendor, operations)?;
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
        let resource = transfer["resource"]
            .as_str()
            .context("intro compressed resource path")?;
        let compressed = std::fs::read(vendor.join(resource))?;
        let decompress_tape =
            crystal_assets::runtime_intro_decompress_instruction_machine_cycles(&compressed)?;
        let modeled_cycles = decompress_tape.iter().sum::<u64>();
        ensure!(
            record["instruction_machine_cycles"].as_u64() == Some(modeled_cycles),
            "intro source LZ3 instruction model disagrees with ROM: {resource}"
        );
        operations[index]["source_decompress_timing"] = json!({
            "routine":"Decompress","source_file":"home/decompress.asm",
            "model":"lz3_source_command_counter_v1","compressed_byte_count":compressed.len(),
            "machine_cycles":modeled_cycles,"caller_call_included":false,"interrupts_included":false});
        let span = &operations[index]["source_span"];
        let start = span["start_line"]
            .as_u64()
            .context("intro decompressor caller start")?;
        let end = span["end_line"]
            .as_u64()
            .context("intro decompressor caller end")?;
        let mut caller = Vec::new();
        for (_, line) in source
            .lines
            .iter()
            .filter(|(line, _)| (start..end).contains(&u64::from(*line)))
        {
            caller.extend(
                plain_instruction_cycles(line, false)
                    .context("intro decompressor register setup instruction")?,
            );
        }
        caller.extend([6, 3, 4, 2, 3, 4, 3, 6]);
        operations[index]["source_decompress_caller_instruction_machine_cycles"] = json!(caller);
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
        next["source_request_timing"] = json!({"routine":"Request2bpp","source_file":"home/gfx.asm",
            "model":"normal_link_blocking_segments_v1","tile_count":transfer["tile_count"],
            "tiles_per_cycle":metadata["chunking"]["default_tiles_per_vblank"],
            "link_mode":"not_mobile","delay_frame_body_included":false,"interrupts_included":false});
        next["source_request_caller_instruction_machine_cycles"] = json!([3, 3, 3, 6]);
        next["source_request_return_instruction_machine_cycles"] = json!([3, 3, 4]);
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
        json!({"source_dispatcher_instruction_machine_cycles":[4,1,2,3,2,2,2,2,1,1],"source_dispatcher_caller_call_included":false,"rom_frame_crossings":crossings,"timing_oracle":{"runner":"pack_core Rust intro oracle","schema_version":1,"validated":false,"rom_sha1":trace["rom_sha1"],"instruction_core":"sm83_mcycle_v1","timer_model":"cgb_timer_falling_edge_delayed_reload_v1"}}),
    )
}
