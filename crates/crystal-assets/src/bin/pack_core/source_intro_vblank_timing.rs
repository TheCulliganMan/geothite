//! Source-derived VBlank helper paths. Only compact cost models are exported;
//! source bytes and instruction traces remain in the external workspace.
use super::source_intro_oracle::{measure_routine_state_at_phase, symbols};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn export(vendor: &Path) -> Result<Value> {
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let rom = std::fs::read(vendor.join("pokecrystal.gbc"))?;
    let address = |label: &str| -> Result<u16> {
        Ok(symbols
            .get(label)
            .with_context(|| format!("missing VBlank symbol {label}"))?
            .1)
    };
    let mut defaults = vec![("hCGB".to_owned(), 1), ("hOAMUpdate".to_owned(), 1)];
    // The actual source HRAM DMA routine is copied into the isolated fixture.
    let &(bank, start) = symbols
        .get("OAMDMACode")
        .context("source OAM DMA routine")?;
    let length = usize::from(address("OAMDMACode.End")? - start);
    let offset = bank * 0x4000 + usize::from(start) - 0x4000;
    for (index, byte) in rom[offset..offset + length].iter().enumerate() {
        defaults.push((
            format!(
                "${:04x}",
                usize::from(address("hTransferShadowOAM")?) + index
            ),
            *byte,
        ));
    }
    let measure = |label: &str, writes: &[(&str, u8)], phase: u64| -> Result<Value> {
        let mut fixture = defaults.clone();
        fixture.extend(writes.iter().map(|(key, value)| (key.to_string(), *value)));
        measure_routine_state_at_phase(vendor, label, &fixture, &[], phase)
    };
    let cost = |label: &str, writes: &[(&str, u8)]| -> Result<u64> {
        measure(label, writes, 65_664)?["machine_cycles"]
            .as_u64()
            .context("VBlank cost")
    };
    let affine = |label: &str, variable: &str, step: u8, fixture: &[(&str, u8)]| -> Result<Value> {
        let mut a = fixture.to_vec();
        a.push((variable, step));
        let first = cost(label, &a)?;
        a.last_mut().unwrap().1 = step * 2;
        let second = cost(label, &a)?;
        let per_unit = second - first;
        let fixed = first - per_unit;
        for count in [step * 3, step * 4, step * 7, step * 11] {
            a.last_mut().unwrap().1 = count;
            ensure!(
                cost(label, &a)? == fixed + u64::from(count / step) * per_unit,
                "{label} transfer cost disagrees at {count}"
            );
        }
        Ok(
            json!({"fixed_machine_cycles":fixed,"per_unit_machine_cycles":per_unit,"unit_size":step}),
        )
    };
    let mut bgmaps = Vec::new();
    for mode in 0..=4 {
        let mut thirds = Vec::new();
        for third in 0..3 {
            thirds.push(cost(
                "UpdateBGMap",
                &[("hBGMapMode", mode), ("hBGMapThird", third)],
            )?);
        }
        bgmaps.push(thirds);
    }
    let mut gametimer = BTreeMap::new();
    for (key, fixtures) in [
        ("logic_paused", vec![("wGameLogicPaused", 1)]),
        ("timer_paused", vec![]),
        ("capped", vec![("wGameTimerPaused", 1), ("wGameTimeCap", 1)]),
        ("frame", vec![("wGameTimerPaused", 1)]),
        (
            "second",
            vec![("wGameTimerPaused", 1), ("wGameTimeFrames", 59)],
        ),
        (
            "minute",
            vec![
                ("wGameTimerPaused", 1),
                ("wGameTimeFrames", 59),
                ("wGameTimeSeconds", 59),
            ],
        ),
        (
            "hour",
            vec![
                ("wGameTimerPaused", 1),
                ("wGameTimeFrames", 59),
                ("wGameTimeSeconds", 59),
                ("wGameTimeMinutes", 59),
            ],
        ),
        (
            "hour_high",
            vec![
                ("wGameTimerPaused", 1),
                ("wGameTimeFrames", 59),
                ("wGameTimeSeconds", 59),
                ("wGameTimeMinutes", 59),
                ("wGameTimeHours", 3),
            ],
        ),
        (
            "cap",
            vec![
                ("wGameTimerPaused", 1),
                ("wGameTimeFrames", 59),
                ("wGameTimeSeconds", 59),
                ("wGameTimeMinutes", 59),
                ("wGameTimeHours", 3),
                ("wGameTimeHours+1", 231),
            ],
        ),
    ] {
        gametimer.insert(key, cost("GameTimer", &fixtures)?);
    }
    let transfer = |label: &str, size: &str, body: &str| -> Result<Value> {
        let active = measure(label, &[(size, 1)], 65_664)?;
        let instructions = active["instructions"]
            .as_array()
            .context("VBlank helper trace")?;
        let ly_index = instructions
            .iter()
            .position(|i| i["opcode"] == 0xf0)
            .context("source LY load")?;
        let prefix = instructions[..ly_index]
            .iter()
            .map(|i| i["machine_cycles"].as_u64().unwrap())
            .sum::<u64>();
        let mut model = affine(body, size, 1, &[])?;
        model["inactive_machine_cycles"] = json!(cost(label, &[])?);
        model["ly_read_prefix_machine_cycles"] = json!(prefix + 3);
        model["early_return_machine_cycles"] =
            json!(measure(label, &[(size, 1)], 0)?["machine_cycles"]);
        model["late_return_machine_cycles"] =
            json!(measure(label, &[(size, 1)], 146 * 456)?["machine_cycles"]);
        let one = cost(label, &[(size, 1)])?;
        model["admitted_prefix_machine_cycles"] = json!(
            one - model["fixed_machine_cycles"].as_u64().unwrap()
                - model["per_unit_machine_cycles"].as_u64().unwrap()
        );
        Ok(model)
    };
    let mut onebpp = transfer("Serve1bppRequest", "wRequested1bppSize", "Serve1bppRequest")?;
    // Serve1bpp's measured affine cost already includes the admission prefix.
    onebpp["admitted_prefix_machine_cycles"] = json!(0);
    let two = transfer(
        "Serve2bppRequest",
        "wRequested2bppSize",
        "_Serve2bppRequest",
    )?;
    let baseline = measure("VBlank", &[], 65_664)?;
    let tape = baseline["instructions"]
        .as_array()
        .context("VBlank routine trace")?;
    let prefix_to = |name: &str| -> Result<u64> {
        let address = address(name)?;
        let index = tape
            .iter()
            .position(|i| i["address"].as_u64() == Some(u64::from(address)))
            .with_context(|| format!("VBlank does not reach {name}"))?;
        Ok(tape[..index]
            .iter()
            .map(|i| i["machine_cycles"].as_u64().unwrap())
            .sum())
    };
    let total = baseline["machine_cycles"].as_u64().unwrap();
    let prefix = prefix_to("UpdateBGMapBuffer")?;
    let normal_done = prefix_to("UpdateJoypad")?;
    let inactive_graphics = cost("UpdateBGMapBuffer", &[])?
        + 2
        + 6
        + cost("UpdatePalsIfCGB", &[])?
        + 2
        + 6
        + cost("DMATransfer", &[])?
        + 2
        + 6
        + cost("UpdateBGMap", &[])?
        + 6
        + cost("Serve2bppRequest", &[])?
        + 6
        + cost("Serve1bppRequest", &[])?
        + 6
        + cost("AnimateTileset", &[])?;
    let bookkeeping = normal_done - prefix - inactive_graphics;
    let suffix = total
        - normal_done
        - cost("UpdateJoypad", &[])?
        - cost("_UpdateSound", &[])?
        - cost("GameTimer", &[])?;
    let model = json!({"schema_version":1,"unit":"sm83_machine_cycles","entry_prefix_machine_cycles":prefix,"bookkeeping_to_joypad_machine_cycles":bookkeeping,"after_joypad_fixed_machine_cycles":suffix,
        "bg_buffer":{"inactive_machine_cycles":cost("UpdateBGMapBuffer",&[])?,"active":affine("UpdateBGMapBuffer","hBGMapTileCount",2,&[("hBGMapUpdate",1)])?},
        "palettes":{"dmg_machine_cycles":cost("UpdatePalsIfCGB",&[("hCGB",0)])?,"idle_machine_cycles":cost("UpdatePalsIfCGB",&[])?,"active_machine_cycles":cost("UpdatePalsIfCGB",&[("hCGBPalUpdate",1)])?},
        "dma":{"inactive_machine_cycles":cost("DMATransfer",&[])?,"active_machine_cycles":cost("DMATransfer",&[("hDMATransfer",1)])?},
        "bg_map_machine_cycles":bgmaps,"one_bpp":onebpp,"two_bpp":two,
        "animate_tileset":{"inactive_machine_cycles":cost("AnimateTileset",&[])?,"early_return_machine_cycles":measure("AnimateTileset",&[("hMapAnims",1)],0)?["machine_cycles"],"late_return_machine_cycles":measure("AnimateTileset",&[("hMapAnims",1)],151*456)?["machine_cycles"],"ly_read_prefix_machine_cycles":9},
        "oam_machine_cycles":cost("hTransferShadowOAM",&[])?,
        "joypad":{"normal_machine_cycles":cost("UpdateJoypad",&[])?,"disabled_machine_cycles":cost("UpdateJoypad",&[("wJoypadDisable",64)])?,"paused_machine_cycles":cost("UpdateJoypad",&[("wGameLogicPaused",1)])?},"game_timer":gametimer,
        "baseline_machine_cycles":total,"baseline_audio_machine_cycles":cost("_UpdateSound",&[])?});
    let mut fixtures = vec![
        vec![],
        vec![("hOAMUpdate", 0)],
        vec![("hCGB", 0)],
        vec![("hCGBPalUpdate", 1)],
        vec![("hBGMapUpdate", 1), ("hBGMapTileCount", 12)],
        vec![("hDMATransfer", 1)],
        vec![("wOverworldDelay", 3), ("wTextDelayFrames", 5)],
        vec![("wJoypadDisable", 64)],
        vec![("wGameLogicPaused", 1)],
        vec![("wRequested2bppSize", 8)],
        vec![("wRequested1bppSize", 6)],
        vec![("wRequested2bppSize", 8), ("wRequested1bppSize", 6)],
    ];
    fixtures.extend([
        vec![("wGameTimerPaused", 1)],
        vec![("wGameTimerPaused", 1), ("wGameTimeCap", 1)],
        vec![("wGameTimerPaused", 1), ("wGameTimeFrames", 59)],
        vec![
            ("wGameTimerPaused", 1),
            ("wGameTimeFrames", 59),
            ("wGameTimeSeconds", 59),
        ],
        vec![
            ("wGameTimerPaused", 1),
            ("wGameTimeFrames", 59),
            ("wGameTimeSeconds", 59),
            ("wGameTimeMinutes", 59),
        ],
        vec![
            ("wGameTimerPaused", 1),
            ("wGameTimeFrames", 59),
            ("wGameTimeSeconds", 59),
            ("wGameTimeMinutes", 59),
            ("wGameTimeHours", 3),
        ],
        vec![
            ("wGameTimerPaused", 1),
            ("wGameTimeFrames", 59),
            ("wGameTimeSeconds", 59),
            ("wGameTimeMinutes", 59),
            ("wGameTimeHours", 3),
            ("wGameTimeHours+1", 231),
        ],
    ]);
    for mode in 1..=4 {
        for third in 0..3 {
            fixtures.push(vec![("hBGMapMode", mode), ("hBGMapThird", third)]);
            fixtures.push(vec![
                ("hBGMapMode", mode),
                ("hBGMapThird", third),
                ("wRequested2bppSize", 8),
            ]);
        }
    }
    let at = |path: &str| -> u64 { model.pointer(path).unwrap().as_u64().unwrap() };
    let two_prefix = at("/entry_prefix_machine_cycles")
        + at("/bg_buffer/inactive_machine_cycles")
        + 2
        + 6
        + at("/palettes/idle_machine_cycles")
        + 2
        + 6
        + at("/dma/inactive_machine_cycles")
        + 2
        + 6
        + model["bg_map_machine_cycles"][0][0].as_u64().unwrap()
        + 6;
    let one_prefix = two_prefix + at("/two_bpp/inactive_machine_cycles") + 6;
    let mut phases = vec![65_664, 66_100, 66_530, 69_700];
    for (prefix, path) in [
        (two_prefix, "/two_bpp/ly_read_prefix_machine_cycles"),
        (one_prefix, "/one_bpp/ly_read_prefix_machine_cycles"),
    ] {
        for line in [144u64, 146] {
            let boundary = line * 456 - 4 * (prefix + at(path));
            for offset in [-8i64, -4, 0, 4, 8] {
                phases.push((boundary as i64 + offset) as u64);
            }
        }
    }
    phases.sort_unstable();
    phases.dedup();
    for fixture in fixtures {
        for &phase in &phases {
            let mut memory = defaults
                .iter()
                .map(|(key, value)| (key.clone(), u16::from(*value)))
                .collect::<BTreeMap<_, _>>();
            for (key, value) in &fixture {
                memory.insert(key.to_string(), u16::from(*value));
            }
            let expected = crystal_assets::runtime_intro_vblank_machine_cycles(
                &model,
                &mut memory,
                phase,
                cost("_UpdateSound", &[])?,
            )?;
            let observed = measure("VBlank", &fixture, phase)?;
            let actual = observed["machine_cycles"]
                .as_u64()
                .context("full VBlank calibration")?;
            ensure!(
                actual == expected,
                "full VBlank cost disagrees for {fixture:?} at phase {phase}: source {actual}, model {expected}"
            );
            for key in [
                "hVBlankCounter",
                "hBGMapUpdate",
                "hBGMapTileCount",
                "hCGBPalUpdate",
                "hDMATransfer",
                "hBGMapThird",
                "wRequested2bppSize",
                "wRequested1bppSize",
                "wOverworldDelay",
                "wTextDelayFrames",
                "wGameTimeCap",
                "wGameTimeFrames",
                "wGameTimeSeconds",
                "wGameTimeMinutes",
                "wGameTimeHours",
                "wGameTimeHours+1",
            ] {
                let predicted = u64::from(*memory.get(key).unwrap_or(&0));
                let actual = observed["memory"][key]
                    .as_u64()
                    .with_context(|| format!("missing VBlank effect measurement {key}"))?;
                ensure!(
                    predicted == actual,
                    "VBlank effect disagrees for {key}, {fixture:?}, phase {phase}: source {actual}, model {predicted}"
                );
            }
        }
    }
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vblank_helpers_are_measured_from_pinned_source() {
        let Some(root) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let model = export(Path::new(&root)).unwrap();
        eprintln!("VBlank source model: {model}");
        assert!(model["baseline_machine_cycles"].as_u64().unwrap() > 100);
    }
}
