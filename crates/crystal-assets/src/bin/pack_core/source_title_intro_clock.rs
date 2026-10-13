//! Source-certified outer-loop and interrupt costs measured from the pinned ROM.
use super::{Source, source_intro_oracle};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

fn cycles(value: &Value) -> Result<u64> {
    value["machine_cycles"]
        .as_u64()
        .context("routine calibration has no machine-cycle cost")
}
fn tape(value: &Value) -> Result<Vec<u64>> {
    value["instructions"]
        .as_array()
        .context("routine instruction tape")?
        .iter()
        .map(|instruction| {
            instruction["machine_cycles"]
                .as_u64()
                .context("instruction cost")
        })
        .collect()
}
fn gap(trace: &Value, from: &str, to: &str) -> Result<u64> {
    let records = trace["records"].as_array().context("intro trace records")?;
    let mut minimum = None;
    for source in records.iter().filter(|record| record["hook"] == from) {
        let end = source["end_cpu_t_cycles"]
            .as_u64()
            .context("hook end clock")?;
        let next = records
            .iter()
            .filter(|record| record["hook"] == to)
            .filter_map(|record| record["start_cpu_t_cycles"].as_u64())
            .filter(|start| *start >= end)
            .min();
        if let Some(next) = next {
            let elapsed = next - end;
            if elapsed < 4000 && elapsed % 4 == 0 {
                minimum = Some(minimum.map_or(elapsed / 4, |old: u64| old.min(elapsed / 4)));
            }
        }
    }
    minimum.with_context(|| format!("unmeasured intro outer-loop gap {from}->{to}"))
}

pub(super) fn lower(vendor: &Path, trace: &Value) -> Result<Value> {
    ensure!(
        trace["rom_sha1"] == "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133",
        "intro clock trace uses different ROM"
    );
    let source = Source::read(vendor, "engine/movie/intro.asm")?;
    source.operation(
        "CrystalIntro",
        &[
            ".loop",
            "call JoyTextDelay",
            "ldh a, [hJoyLast]",
            "and PAD_BUTTONS",
            "jr nz, .ShutOffMusic",
            "ld a, [wJumptableIndex]",
            "bit JUMPTABLE_EXIT_F, a",
            "jr nz, .done",
            "call IntroSceneJumper",
            "farcall PlaySpriteAnimations",
            "call DelayFrame",
            "jp .loop",
        ],
        "source_intro_outer_loop",
        json!({}),
    )?;
    let joy = Source::read(vendor, "home/joypad.asm")?;
    joy.operation(
        "JoyTextDelay",
        &[
            "ldh a, [hJoyPressed]",
            "and a",
            "jr z, .checkframedelay",
            "ld a, 15",
            "ld [wTextDelayFrames], a",
            "ret",
        ],
        "source_pressed_repeat_reset",
        json!({}),
    )?;
    joy.operation(
        "JoyTextDelay",
        &[
            ".checkframedelay",
            "ld a, [wTextDelayFrames]",
            "and a",
            "jr z, .restartframedelay",
            "xor a",
            "ldh [hJoyLast], a",
            "ret",
        ],
        "source_repeat_suppression",
        json!({}),
    )?;
    joy.operation(
        "JoyTextDelay",
        &[
            ".restartframedelay",
            "ld a, 5",
            "ld [wTextDelayFrames], a",
            "ret",
        ],
        "source_repeat_restart",
        json!({}),
    )?;
    let lcd = Source::read(vendor, "home/lcd.asm")?;
    lcd.operation(
        "LCD",
        &["push af", "ldh a, [hLCDCPointer]", "and a", "jr z, .done"],
        "source_lcd_callback_guard",
        json!({}),
    )?;
    let mobile = Source::read(vendor, "home/mobile.asm")?;
    mobile.operation(
        "MobileTimer",
        &["ldh a, [hMobile]", "and a", "jr z, .pop_ret"],
        "source_inactive_mobile_guard",
        json!({}),
    )?;
    let timer = Source::read(vendor, "home/game_time.asm")?;
    let vblank = Source::read(vendor, "home/vblank.asm")?;
    vblank.operation(
        "VBlank",
        &[
            "call GameTimer",
            "pop hl",
            "pop de",
            "pop bc",
            "pop af",
            "reti",
        ],
        "source_vblank_epilogue",
        json!({}),
    )?;
    timer.operation(
        "GameTimer",
        &[
            "ld hl, wGameTimerPaused",
            "bit GAME_TIMER_COUNTING_F, [hl]",
            "ret z",
        ],
        "source_inactive_game_timer",
        json!({}),
    )?;
    let fixtures = [
        vec![("hInMenu".into(), 1), ("hJoypadDown".into(), 1)],
        vec![("hInMenu".into(), 1), ("wTextDelayFrames".into(), 1)],
        vec![("hInMenu".into(), 1), ("wTextDelayFrames".into(), 0)],
    ];
    let measured = fixtures
        .iter()
        .map(|writes| source_intro_oracle::measure_routine(vendor, "JoyTextDelay", writes))
        .collect::<Result<Vec<_>>>()?;
    let tapes = measured.iter().map(tape).collect::<Result<Vec<_>>>()?;
    let shared = (0..tapes.iter().map(Vec::len).min().unwrap())
        .take_while(|index| {
            tapes.iter().all(|value| value[*index] == tapes[0][*index])
                && measured.iter().all(|value| {
                    value["instructions"][*index]["address"]
                        == measured[0]["instructions"][*index]["address"]
                })
        })
        .count();
    ensure!(
        shared > 0,
        "JoyTextDelay paths have no common instruction prefix"
    );
    let zero = source_intro_oracle::measure_routine(vendor, "LCD", &[("hLCDCPointer".into(), 0)])?;
    let nonzero =
        source_intro_oracle::measure_routine(vendor, "LCD", &[("hLCDCPointer".into(), 0x43)])?;
    let inactive_timer =
        source_intro_oracle::measure_routine(vendor, "MobileTimer", &[("hMobile".into(), 0)])?;
    let inactive_game = source_intro_oracle::measure_routine(
        vendor,
        "GameTimer",
        &[("wGameTimerPaused".into(), 0)],
    )?;
    let entry = &trace["entry"];
    let input = &trace["first_input"];
    let entry_cost = (input["cpu_t_cycles"]
        .as_u64()
        .context("first input clock")?
        - entry["cpu_t_cycles"].as_u64().context("entry clock")?
        - input["interrupt_t_cycles_since_entry"]
            .as_u64()
            .context("entry IRQ cost")?)
        / 4;
    let oracle = json!({"runner":"pack_core Rust intro oracle","schema_version":1,"validated":false,"rom_sha1":trace["rom_sha1"],"instruction_core":"sm83_mcycle_v1","timer_model":"cgb_timer_falling_edge_delayed_reload_v1"});
    Ok(
        json!({"unit":"sm83_machine_cycles","entry_to_first_input_machine_cycles":entry_cost,"hardware_entry":5,"vectors":{"instruction":"jp","machine_cycles":4},"frame_clock":{"frame_t_cycles":trace["frame_t_cycles"],"intro_entry_phase_t_cycles":entry["phase_t_cycles"],"timing_oracle":oracle},"outer_loop_body":{"after_input_before_scene_dispatch":gap(trace,"JoyTextDelay","IntroSceneJumper")?,"scene_dispatch_to_sprite_scheduler":gap(trace,"IntroSceneJumper","PlaySpriteAnimations")?,"sprite_scheduler_to_frame_wait":gap(trace,"PlaySpriteAnimations","DelayFrame")?},"joy_text_delay":{"pressed_repeat_reset_machine_cycles":cycles(&measured[0])?,"repeat_suppressed_machine_cycles":cycles(&measured[1])?,"repeat_restart_machine_cycles":cycles(&measured[2])?,"common_instruction_machine_cycles":tapes[0][..shared],"pressed_repeat_reset_tail_machine_cycles":tapes[0][shared..],"repeat_suppressed_tail_machine_cycles":tapes[1][shared..],"repeat_restart_tail_machine_cycles":tapes[2][shared..]},"lcd_stat":{"handler":"LCD","trigger_register":"rSTAT","trigger_mask":"STAT_MODE_0","trigger":"hblank","callback_pointer":"hLCDCPointer","interrupts_per_visible_frame":144,"scanline_t_cycles":456,"hblank_request_t_cycles":250,"vblank_request_t_cycles":65664,"callback_zero_machine_cycles":cycles(&zero)?+9,"callback_nonzero_machine_cycles":cycles(&nonzero)?+9},"timer":{"handler":"MobileTimer","enable":"IE_TIMER","inactive_guard":{"source":"hMobile","predicate":"zero"},"request_period_t_cycles":entry["timer_request_period_t_cycles"],"first_request_after_intro_entry_t_cycles":entry["first_timer_request_after_intro_entry_t_cycles"],"inactive_machine_cycles":cycles(&inactive_timer)?+9},"vblank_normal":{"handler":"VBlank_Normal","selector":{"source":"hVBlank","value":0},"inactive_game_timer_machine_cycles":cycles(&inactive_game)?,"wrapper_epilogue_machine_cycles":16},"implementation_source_spans":[source.span("CrystalIntro")?,joy.span("JoyTextDelay")?,lcd.span("LCD")?,mobile.span("MobileTimer")?,timer.span("GameTimer")?]}),
    )
}
