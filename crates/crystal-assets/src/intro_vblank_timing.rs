//! Ordered, source-measured VBlank work for the continuous intro clock.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::collections::BTreeMap;

/// Execute the source VBlank_Normal timing and its authoritative byte effects.
/// `phase_t_cycles` is the LCD phase at entry to VBlank, after hardware entry
/// and the interrupt-vector JP. Audio cost is the current sound engine body.
pub fn runtime_intro_vblank_machine_cycles(
    model: &Value,
    memory: &mut BTreeMap<String, u16>,
    phase_t_cycles: u64,
    audio_machine_cycles: u64,
) -> Result<u64> {
    fn cost(model: &Value, key: &str) -> Result<u64> {
        model[key]
            .as_u64()
            .with_context(|| format!("missing source VBlank cost {key}"))
    }
    fn byte(memory: &BTreeMap<String, u16>, key: &str) -> Result<u8> {
        u8::try_from(*memory.get(key).unwrap_or(&0))
            .with_context(|| format!("VBlank byte {key} exceeds byte range"))
    }
    fn affine(model: &Value, count: u8) -> Result<u64> {
        Ok(cost(model, "fixed_machine_cycles")?
            + cost(model, "per_unit_machine_cycles")? * (u64::from(count) / cost(model,"unit_size")?))
    }
    fn serve(
        model: &Value,
        memory: &mut BTreeMap<String, u16>,
        size: &str,
        prefix: u64,
        phase: u64,
    ) -> Result<u64> {
        let count = byte(memory, size)?;
        if count == 0 {
            return cost(model, "inactive_machine_cycles");
        }
        let ly =
            ((phase + 4 * (prefix + cost(model, "ly_read_prefix_machine_cycles")?)) / 456) % 154;
        if ly < 144 {
            return cost(model, "early_return_machine_cycles");
        }
        if ly >= 146 {
            return cost(model, "late_return_machine_cycles");
        }
        memory.insert(size.into(), 0);
        let source = size.replace("Size", "Source");
        let destination = size.replace("Size", "Dest");
        for (key, bytes) in [
            (
                source,
                u16::from(count) * if size.contains("1bpp") { 8 } else { 16 },
            ),
            (destination, u16::from(count) * 16),
        ] {
            let current = *memory.get(&key).unwrap_or(&0);
            memory.insert(key, current.wrapping_add(bytes));
        }
        Ok(affine(model, count)? + cost(model, "admitted_prefix_machine_cycles")?)
    }
    ensure!(
        model["schema_version"] == 1,
        "unsupported source VBlank cost model"
    );
    ensure!(
        byte(memory, "hVBlank")? == 0,
        "intro VBlank selector is not VBlank_Normal"
    );
    let mut total = cost(model, "entry_prefix_machine_cycles")?;
    let bg_buffer = byte(memory, "hBGMapUpdate")? != 0;
    if bg_buffer {
        let count = byte(memory, "hBGMapTileCount")?;
        ensure!(
            count != 0 && count % 2 == 0,
            "VBlank BG buffer needs a nonzero even source tile count"
        );
        total += affine(&model["bg_buffer"]["active"], count)? + 3;
        memory.insert("hBGMapUpdate".into(), 0);
        memory.insert("hBGMapTileCount".into(), 0);
    } else {
        total += cost(&model["bg_buffer"], "inactive_machine_cycles")? + 2 + 6;
        let cgb = byte(memory, "hCGB")? != 0;
        let palettes = cgb && byte(memory, "hCGBPalUpdate")? != 0;
        total += cost(
            &model["palettes"],
            if !cgb {
                "dmg_machine_cycles"
            } else if palettes {
                "active_machine_cycles"
            } else {
                "idle_machine_cycles"
            },
        )?;
        if palettes {
            total += 3;
            memory.insert("hCGBPalUpdate".into(), 0);
        } else {
            total += 2 + 6;
            let dma = byte(memory, "hDMATransfer")? != 0;
            total += cost(
                &model["dma"],
                if dma {
                    "active_machine_cycles"
                } else {
                    "inactive_machine_cycles"
                },
            )?;
            if dma {
                total += 3;
                memory.insert("hDMATransfer".into(), 0);
            } else {
                total += 2 + 6;
                let mode = usize::from(byte(memory, "hBGMapMode")?);
                let third = usize::from(byte(memory, "hBGMapThird")?);
                ensure!(
                    mode <= 4 && third <= 2,
                    "unsupported source BG map mode {mode} / third {third}"
                );
                total += model["bg_map_machine_cycles"][mode][third]
                    .as_u64()
                    .context("missing source BG map path")?;
                if mode != 0 {
                    memory.insert("hBGMapThird".into(), ((third + 1) % 3) as u16);
                }
                total += 6;
                total += serve(
                    &model["two_bpp"],
                    memory,
                    "wRequested2bppSize",
                    total,
                    phase_t_cycles,
                )?;
                total += 6;
                total += serve(
                    &model["one_bpp"],
                    memory,
                    "wRequested1bppSize",
                    total,
                    phase_t_cycles,
                )?;
                total += 6;
                let anim = &model["animate_tileset"];
                if byte(memory, "hMapAnims")? == 0 {
                    total += cost(anim, "inactive_machine_cycles")?;
                } else {
                    let ly = ((phase_t_cycles
                        + 4 * (total + cost(anim, "ly_read_prefix_machine_cycles")?))
                        / 456)
                        % 154;
                    if ly < 144 {
                        total += cost(anim, "early_return_machine_cycles")?;
                    } else if ly >= 151 {
                        total += cost(anim, "late_return_machine_cycles")?;
                    } else {
                        anyhow::bail!(
                            "intro requested an active tileset animation without its authored timing model"
                        );
                    }
                }
            }
        }
    }
    total += cost(model, "bookkeeping_to_joypad_machine_cycles")?;
    // The baseline skips OAM; the enabled path's JR costs one fewer cycle,
    // then calls the exact copied HRAM DMA loop.
    if byte(memory, "hOAMUpdate")? == 0 {
        total += 5 + cost(model, "oam_machine_cycles")?;
    }
    for key in ["wOverworldDelay", "wTextDelayFrames"] {
        let value = byte(memory, key)?;
        if value != 0 {
            total += 4;
            memory.insert(key.into(), u16::from(value - 1));
        }
    }
    memory.insert("wVBlankOccurred".into(), 0);
    let joypad = &model["joypad"];
    let paused = byte(memory, "wGameLogicPaused")? != 0;
    let disabled = byte(memory, "wJoypadDisable")? & 0xd0 != 0;
    total += cost(
        joypad,
        if disabled {
            "disabled_machine_cycles"
        } else if paused {
            "paused_machine_cycles"
        } else {
            "normal_machine_cycles"
        },
    )?;
    total += cost(model, "after_joypad_fixed_machine_cycles")? + audio_machine_cycles;
    let timer = &model["game_timer"];
    let key = if paused {
        "logic_paused"
    } else if byte(memory, "wGameTimerPaused")? & 1 == 0 {
        "timer_paused"
    } else if byte(memory, "wGameTimeCap")? & 1 != 0 {
        "capped"
    } else {
        let frame = byte(memory, "wGameTimeFrames")?.wrapping_add(1);
        if frame < 60 {
            memory.insert("wGameTimeFrames".into(), u16::from(frame));
            "frame"
        } else {
            memory.insert("wGameTimeFrames".into(), 0);
            let second = byte(memory, "wGameTimeSeconds")?.wrapping_add(1);
            if second < 60 {
                memory.insert("wGameTimeSeconds".into(), u16::from(second));
                "second"
            } else {
                memory.insert("wGameTimeSeconds".into(), 0);
                let minute = byte(memory, "wGameTimeMinutes")?.wrapping_add(1);
                if minute < 60 {
                    memory.insert("wGameTimeMinutes".into(), u16::from(minute));
                    "minute"
                } else {
                    memory.insert("wGameTimeMinutes".into(), 0);
                    // Authored GameTimer loads the low-named byte into H.
                    let high = byte(memory, "wGameTimeHours")?;
                    let low = byte(memory, "wGameTimeHours+1")?;
                    let hour = u16::from_be_bytes([high, low]).wrapping_add(1);
                    if hour >= 1000 {
                        memory.insert("wGameTimeCap".into(), 1);
                        memory.insert("wGameTimeSeconds".into(), 59);
                        memory.insert("wGameTimeMinutes".into(), 59);
                        "cap"
                    } else {
                        memory.insert("wGameTimeHours".into(), u16::from(hour >> 8));
                        memory.insert("wGameTimeHours+1".into(), hour & 255);
                        if hour >> 8 < 3 { "hour" } else { "hour_high" }
                    }
                }
            }
        }
    };
    total += cost(timer, key)?;
    memory.insert(
        "hVBlankCounter".into(),
        u16::from(byte(memory, "hVBlankCounter")?.wrapping_add(1)),
    );
    Ok(total)
}
