//! Measure pinned external-ROM intro hooks with the Rust instruction core.
//! Raw measurements remain uncertified until the complete presentation and
//! interrupt model are checked against them; this module does not publish packs.
#[path = "source_intro_cpu.rs"]
mod source_intro_cpu;
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use source_intro_cpu::Cpu;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::SystemTime,
};

const FRAME: u64 = 70_224;
const ROM_SHA1: &str = "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133";

#[derive(Clone, PartialEq, Eq)]
struct SourceFingerprint {
    rom_size: u64,
    rom_modified: SystemTime,
    symbols_size: u64,
    symbols_modified: SystemTime,
}

struct TimingSource {
    rom: Arc<Vec<u8>>,
    symbols: BTreeMap<String, (usize, u16)>,
}

fn source_fingerprint(vendor: &Path) -> Result<SourceFingerprint> {
    let rom = std::fs::metadata(vendor.join("pokecrystal.gbc"))?;
    let symbols = std::fs::metadata(vendor.join("pokecrystal.sym"))?;
    Ok(SourceFingerprint {
        rom_size: rom.len(),
        rom_modified: rom.modified()?,
        symbols_size: symbols.len(),
        symbols_modified: symbols.modified()?,
    })
}

fn timing_source(vendor: &Path) -> Result<Arc<TimingSource>> {
    type Sources = BTreeMap<PathBuf, (SourceFingerprint, Arc<TimingSource>)>;
    static SOURCES: OnceLock<Mutex<Sources>> = OnceLock::new();
    let vendor = vendor.canonicalize()?;
    let fingerprint = source_fingerprint(&vendor)?;
    let mut sources = SOURCES
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .map_err(|_| anyhow::anyhow!("intro timing source cache was poisoned"))?;
    if let Some((cached, source)) = sources.get(&vendor) {
        if *cached == fingerprint {
            return Ok(Arc::clone(source));
        }
    }
    let rom = std::fs::read(vendor.join("pokecrystal.gbc"))?;
    ensure!(
        format!("{:x}", Sha1::digest(&rom)) == ROM_SHA1,
        "routine timing ROM differs from pinned source"
    );
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    ensure!(
        source_fingerprint(&vendor)? == fingerprint,
        "timing inputs changed while being loaded"
    );
    let source = Arc::new(TimingSource {
        rom: Arc::new(rom),
        symbols,
    });
    if sources.len() >= 4 {
        sources.clear();
    }
    sources.insert(vendor, (fingerprint, Arc::clone(&source)));
    Ok(source)
}

#[cfg(test)]
mod source_cache_tests {
    use super::*;

    #[test]
    fn cached_sources_keep_fresh_ram_and_reject_changed_roms() {
        let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
            return;
        };
        let vendor = Path::new(&vendor);
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "geothite-intro-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let scratch = Scratch(directory);
        for file in ["pokecrystal.gbc", "pokecrystal.sym"] {
            std::fs::copy(vendor.join(file), scratch.0.join(file)).unwrap();
        }
        let first = timing_source(&scratch.0).unwrap();
        assert!(Arc::ptr_eq(&first, &timing_source(&scratch.0).unwrap()));
        for (callback, expected) in [(0, 18), (0x43, 40), (0, 18)] {
            let measured =
                measure_routine(&scratch.0, "LCD", &[("hLCDCPointer".into(), callback)]).unwrap();
            assert_eq!(measured["machine_cycles"], expected);
        }
        std::fs::write(scratch.0.join("pokecrystal.gbc"), [0]).unwrap();
        assert!(
            timing_source(&scratch.0)
                .err()
                .unwrap()
                .to_string()
                .contains("differs from pinned source")
        );
    }
}

fn fixture_address(symbols: &BTreeMap<String, (usize, u16)>, expression: &str) -> Result<u16> {
    if let Some(hex) = expression.strip_prefix('$') {
        return Ok(u16::from_str_radix(hex, 16)?);
    }
    if let Some(address) = symbols.get(expression) {
        return Ok(address.1);
    }
    let (name, offset) = expression
        .rsplit_once('+')
        .with_context(|| format!("missing timing fixture variable {expression}"))?;
    let offset = offset.trim();
    let offset = if let Some(hex) = offset.strip_prefix('$') {
        u16::from_str_radix(hex, 16)?
    } else {
        offset.parse::<u16>()?
    };
    symbols
        .get(name.trim())
        .with_context(|| format!("missing timing fixture variable {name}"))?
        .1
        .checked_add(offset)
        .context("timing fixture address overflow")
}

pub(super) fn measure_routine(
    vendor: &Path,
    label: &str,
    writes: &[(String, u8)],
) -> Result<Value> {
    measure_routine_state(vendor, label, writes, &[])
}

pub(super) fn measure_routine_state(
    vendor: &Path,
    label: &str,
    writes: &[(String, u8)],
    registers: &[(&str, u16)],
) -> Result<Value> {
    measure_routine_state_at_phase(vendor, label, writes, registers, 0)
}

pub(super) fn measure_routine_state_at_phase(
    vendor: &Path,
    label: &str,
    writes: &[(String, u8)],
    registers: &[(&str, u16)],
    lcd_phase: u64,
) -> Result<Value> {
    measure_routine_fixture(vendor, label, writes, registers, lcd_phase, false)
}

/// Calibrate caller instructions around actual DelayFrame executions. The
/// fixture wakes HALT and supplies completion flags; the recorded frame bodies
/// are excluded from caller tapes and do not certify interrupt/frame timing.
pub(super) fn measure_routine_with_frame_wait_fixture(
    vendor: &Path,
    label: &str,
    writes: &[(String, u8)],
    registers: &[(&str, u16)],
) -> Result<Value> {
    measure_routine_fixture(vendor, label, writes, registers, 0, true)
}

fn measure_routine_fixture(
    vendor: &Path,
    label: &str,
    writes: &[(String, u8)],
    registers: &[(&str, u16)],
    lcd_phase: u64,
    wake_frame_waits: bool,
) -> Result<Value> {
    let source = timing_source(vendor)?;
    let symbols = &source.symbols;
    let &(bank, address) = symbols
        .get(label)
        .with_context(|| format!("missing timing routine {label}"))?;
    let writes = writes
        .iter()
        .map(|(name, value)| Ok((fixture_address(&symbols, name)?, *value)))
        .collect::<Result<Vec<_>>>()?;
    let mut cpu = Cpu::new(Arc::clone(&source.rom));
    cpu.prepare_call(bank, address, &writes);
    cpu.fixture_lcd_phase(lcd_phase);
    cpu.fixture_registers(registers)
        .map_err(anyhow::Error::msg)?;
    let mut instructions = Vec::new();
    let delay_frame = *symbols.get("DelayFrame").context("source DelayFrame")?;
    let mut frame_wait = None;
    let mut frame_waits = Vec::new();
    for _ in 0..1_000_000 {
        if let Some((return_location, return_sp, start)) = frame_wait {
            if cpu.location() == return_location && cpu.registers().3 == return_sp {
                frame_waits.push(json!({"start_instruction":start,"end_instruction":instructions.len()}));
                frame_wait = None;
            }
        }
        if cpu.location() == (0, 0) {
            return Ok(
                json!({"routine":label,"machine_cycles":cpu.clock().0/4,"instructions":instructions,"frame_waits":frame_waits,"rom_sha1":ROM_SHA1,"interrupts_enabled":false,"memory":scene_memory(&cpu,&symbols)}),
            );
        }
        let location = cpu.location();
        if wake_frame_waits && location == delay_frame {
            ensure!(frame_wait.is_none(), "nested DelayFrame fixture");
            let sp = cpu.registers().3;
            let pc = u16::from_le_bytes([cpu.read(sp),cpu.read(sp.wrapping_add(1))]);
            frame_wait = Some(((if pc<0x4000 {0}else{cpu.rom_bank()},pc),sp.wrapping_add(2),instructions.len()));
        }
        let opcode = cpu.read(location.1);
        let start = cpu.clock().0;
        cpu.step().map_err(anyhow::Error::msg)?;
        instructions.push(json!({"bank":location.0,"address":location.1,"opcode":opcode,"machine_cycles":(cpu.clock().0-start)/4}));
        if wake_frame_waits && frame_wait.is_some() && opcode == 0x76 {
            for name in ["wVBlankOccurred", "wRequested2bppSize", "wRequested1bppSize"] {
                cpu.fixture_write(fixture_address(symbols,name)?,0);
            }
            cpu.fixture_write(0xffff,1);
            cpu.fixture_write(0xff0f,1);
        }
    }
    anyhow::bail!("timing routine {label} exceeded bounded instruction limit")
}

pub(super) fn symbols(source: &str) -> Result<BTreeMap<String, (usize, u16)>> {
    let mut symbols = BTreeMap::new();
    for line in source.lines().filter(|line| !line.starts_with(';')) {
        let Some((address, label)) = line.split_once(' ') else {
            continue;
        };
        let Some((bank, address)) = address.split_once(':') else {
            continue;
        };
        let address = (
            usize::from_str_radix(bank, 16)?,
            u16::from_str_radix(address, 16)?,
        );
        ensure!(
            symbols.insert(label.trim().to_owned(), address).is_none(),
            "duplicate ROM symbol {label}"
        );
    }
    Ok(symbols)
}

struct Pending {
    name: String,
    return_address: (usize, u16),
    return_sp: u16,
    cpu: u64,
    lcd: u64,
    interrupt_cycles: u64,
    idle_cycles: u64,
    record: Value,
}

fn scene_memory(cpu: &Cpu, symbols: &BTreeMap<String, (usize, u16)>) -> Value {
    let mut memory = serde_json::Map::new();
    for name in [
        "wJumptableIndex",
        "wIntroSceneFrameCounter",
        "wIntroSceneTimer",
        "wGlobalAnimXOffset",
        "wGlobalAnimYOffset",
        "hSCX",
        "hSCY",
        "hWX",
        "hWY",
        "hLCDCPointer",
        "hBGMapMode",
        "hBGMapAddress",
        "hCGBPalUpdate",
        "hBGP",
        "hOBP",
        "hVBlank",
        "hVBlankCounter",
        "hInMenu",
        "hCGB",
        "hOAMUpdate",
        "hBGMapUpdate",
        "hBGMapTileCount",
        "hBGMapThird",
        "hDMATransfer",
        "hMapAnims",
        "wRequested2bppSize",
        "wRequested2bppSource",
        "wRequested2bppDest",
        "wRequested1bppSize",
        "wRequested1bppSource",
        "wRequested1bppDest",
        "wOverworldDelay",
        "wTextDelayFrames",
        "wJoypadDisable",
        "wGameLogicPaused",
        "wGameTimerPaused",
        "wGameTimeCap",
        "wGameTimeFrames",
        "wGameTimeSeconds",
        "wGameTimeMinutes",
        "wGameTimeHours",
        "wMusicPlaying",
        "wChannel1Flags1",
        "wChannel2Flags1",
        "wChannel3Flags1",
        "wChannel4Flags1",
        "wChannel5Flags1",
        "wChannel6Flags1",
        "wChannel7Flags1",
        "wChannel8Flags1",
    ] {
        if let Some((bank, address)) = symbols.get(name) {
            memory.insert(name.into(), json!(cpu.read_symbol(*bank, *address)));
        }
    }
    if let Some((bank,address)) = symbols.get("wGameTimeHours") {
        memory.insert("wGameTimeHours+1".into(),json!(cpu.read_symbol(*bank,address.wrapping_add(1))));
    }
    if let Some((_, address)) = symbols.get("wLYOverrides") {
        memory.insert("wLYOverrides".into(), json!(cpu.read(*address)));
        memory.insert(
            "wLYOverrides+95".into(),
            json!(cpu.read(address.wrapping_add(95))),
        );
    }
    Value::Object(memory)
}

pub(super) fn trace(vendor: &Path) -> Result<Value> {
    let rom = std::fs::read(vendor.join("pokecrystal.gbc"))?;
    let hash = format!("{:x}", Sha1::digest(&rom));
    ensure!(
        hash == ROM_SHA1,
        "intro oracle ROM SHA-1 differs from pinned source"
    );
    let symbols = symbols(&std::fs::read_to_string(vendor.join("pokecrystal.sym"))?)?;
    let symbol = |label: &str| {
        symbols
            .get(label)
            .copied()
            .with_context(|| format!("missing intro oracle symbol {label}"))
    };
    let entry = symbol("CrystalIntro")?;
    let dispatcher = symbol("IntroSceneJumper")?;
    let cleanup_entry = symbol("CrystalIntro.done")?;
    let scene_address = symbol("wJumptableIndex")?.1;
    let input = symbol("JoyTextDelay")?;
    let mut hooks = BTreeMap::new();
    for label in [
        "Decompress",
        "Request2bpp",
        "Intro_DecompressRequest2bpp_64Tiles",
        "Intro_DecompressRequest2bpp_128Tiles",
        "Intro_DecompressRequest2bpp_255Tiles",
        "DoNextFrameForAllSprites",
        "PlaySpriteAnimations",
        "InitSpriteAnimStruct",
        "JoyTextDelay",
        "IntroSceneJumper",
        "DelayFrame",
        "PlaySFX",
        "_UpdateSound",
        "ApplyPitchSlide",
        "HandleTrackVibrato",
        "HandleNoise",
        "UpdateChannels",
        "ParseMusic",
        "PlayDanger",
        "FadeMusic",
        "ReadNoiseSample",
    ] {
        hooks.insert(symbol(label)?, label);
    }
    let mut cpu = Cpu::new(rom);
    let mut entered = false;
    let mut entry_clock = None;
    let mut first_input = None;
    let mut current_scene = u8::MAX;
    let mut dispatch_tick = 0u16;
    let mut scenes = Vec::<Value>::new();
    let mut records = Vec::new();
    let mut pending = Vec::<Pending>::new();
    let mut interrupt_cycles = 0u64;
    let mut idle_cycles = 0u64;
    let mut interrupt = None::<(u32, (usize, u16), u16, u64)>;
    let mut interrupt_timings = BTreeMap::<u32, std::collections::BTreeSet<u64>>::new();
    for _ in 0..150_000_000 {
        if let Some((kind, location, sp, start)) = interrupt {
            if cpu.location() == location && cpu.registers().3 == sp {
                let elapsed = cpu.clock().0 - start;
                interrupt_cycles += elapsed;
                interrupt_timings.entry(kind).or_default().insert(elapsed);
                interrupt = None;
            }
        }
        if entered && interrupt.is_none() {
            if let Some(kind) = cpu.next_interrupt() {
                interrupt = Some((kind, cpu.location(), cpu.registers().3, cpu.clock().0));
            }
        }
        if cpu.instruction_ready() {
            let location = cpu.location();
            let (cycles, lcd) = cpu.clock();
            let (bc, de, hl, sp) = cpu.registers();
            let mut index = 0;
            while index < pending.len() {
                if location == pending[index].return_address && sp == pending[index].return_sp {
                    let finished = pending.remove(index);
                    let mut record = finished.record;
                    if finished.name == "IntroSceneJumper" {
                        record["end_memory"] = scene_memory(&cpu, &symbols);
                    }
                    record["hook"] = json!(finished.name);
                    record["start_frame"] = json!(finished.lcd / FRAME);
                    record["start_cpu_t_cycles"] = json!(finished.cpu);
                    record["end_cpu_t_cycles"] = json!(cycles);
                    record["end_frame"] = json!(lcd / FRAME);
                    record["start_phase_t_cycles"] = json!(finished.lcd % FRAME);
                    record["elapsed_t_cycles"] = json!(cycles - finished.cpu);
                    record["lcd_elapsed_t_cycles"] = json!(lcd - finished.lcd);
                    let instruction_cycles = cycles
                        - finished.cpu
                        - (interrupt_cycles - finished.interrupt_cycles)
                        - (idle_cycles - finished.idle_cycles);
                    ensure!(
                        instruction_cycles % 4 == 0,
                        "intro hook instruction cost is not a machine-cycle multiple"
                    );
                    record["instruction_machine_cycles"] = json!(instruction_cycles / 4);
                    records.push(record);
                } else {
                    index += 1
                }
            }
            if location == entry {
                ensure!(!entered, "intro oracle unexpectedly reentered CrystalIntro");
                entered = true;
                let (timer_period, first_timer_request) = cpu
                    .timer_calendar()
                    .context("intro entry timer is disabled")?;
                entry_clock = Some(
                    json!({"cpu_t_cycles":cycles,"lcd_t_cycles":lcd,"phase_t_cycles":lcd%FRAME,"timer_request_period_t_cycles":timer_period,"first_timer_request_after_intro_entry_t_cycles":first_timer_request,"memory":scene_memory(&cpu,&symbols)}),
                );
            }
            if entered && first_input.is_none() && location == input {
                first_input = Some(
                    json!({"cpu_t_cycles":cycles,"lcd_t_cycles":lcd,"interrupt_t_cycles_since_entry":interrupt_cycles,"idle_t_cycles_since_entry":idle_cycles}),
                );
            }
            if entered && location == dispatcher {
                let scene = cpu.read(scene_address);
                if scene != current_scene {
                    if let Some(last) = scenes.last_mut() {
                        last["dispatch_ticks"] = json!(dispatch_tick);
                    }
                    current_scene = scene;
                    dispatch_tick = 0;
                    scenes.push(json!({"dispatcher_entry":scene,"start_frame":lcd/FRAME,"cpu_t_cycles":cycles,"phase_t_cycles":lcd%FRAME}));
                }
                dispatch_tick = dispatch_tick
                    .checked_add(1)
                    .context("intro dispatch tick overflow")?;
            }
            if entered {
                if let Some(name) = hooks.get(&location) {
                    let return_pc =
                        u16::from_le_bytes([cpu.read(sp), cpu.read(sp.wrapping_add(1))]);
                    let return_bank = if return_pc < 0x4000 {
                        0
                    } else {
                        cpu.rom_bank()
                    };
                    pending.push(Pending {
                        name: (*name).into(), return_address: (return_bank,return_pc), return_sp:sp.wrapping_add(2), cpu:cycles,lcd,interrupt_cycles,idle_cycles,
                        record: json!({"dispatcher_entry":current_scene,"dispatch_tick":dispatch_tick,"bc":bc,"de":de,"hl":hl,"a":cpu.accumulator(),"bank":cpu.rom_bank(),"hook_bank":location.0,"return_pc":return_pc,"vram_bank":cpu.read(0xff4f)&1,"wram_bank":cpu.read(0xff70)&7}),
                    });
                    if *name == "IntroSceneJumper" {
                        pending.last_mut().unwrap().record["start_memory"] =
                            scene_memory(&cpu, &symbols);
                    }
                    if matches!(
                        *name,
                        "_UpdateSound"
                            | "ApplyPitchSlide"
                            | "HandleTrackVibrato"
                            | "HandleNoise"
                            | "UpdateChannels"
                            | "ParseMusic"
                            | "PlayDanger"
                            | "FadeMusic"
                            | "ReadNoiseSample"
                    ) {
                        let begin = symbol("wAudio")?.1;
                        let end = symbol("wAudioEnd")?.1;
                        ensure!(
                            end > begin && end - begin < 4096,
                            "invalid authored audio RAM domain"
                        );
                        let audio = (begin..end)
                            .map(|address| cpu.read(address))
                            .collect::<Vec<_>>();
                        let sample = symbol("wNoiseSampleAddress")?.1;
                        let sample = u16::from_le_bytes([
                            cpu.read(sample),
                            cpu.read(sample.wrapping_add(1)),
                        ]);
                        let record = &mut pending.last_mut().unwrap().record;
                        record["audio_ram_start_address"] = json!(begin);
                        record["audio_ram"] = json!(audio);
                        record["noise_sample_first_byte"] = json!(cpu.read(sample));
                        for name in ["wMusicPlaying", "wOptions", "wPlayerState", "hROMBank"] {
                            record[name] = json!(cpu.read(symbol(name)?.1));
                        }
                    }
                }
                if location == cleanup_entry {
                    ensure!(
                        cpu.read(scene_address) & 0x80 != 0,
                        "intro exited without authored completion flag"
                    );
                    if let Some(last) = scenes.last_mut() {
                        last["dispatch_ticks"] = json!(dispatch_tick);
                    }
                    ensure!(
                        scenes.len() == 28,
                        "intro oracle did not traverse the authored 28 scenes"
                    );
                    ensure!(
                        pending.is_empty(),
                        "intro oracle completed inside measured hooks: {:?}",
                        pending
                            .iter()
                            .map(|hook| (&hook.name, hook.return_address, hook.return_sp))
                            .collect::<Vec<_>>()
                    );
                    return Ok(
                        json!({"schema_version":1,"validated":false,"runner":"pack_core Rust intro oracle","rom_sha1":hash,"startup":"CGB post-boot registers, LCD enabled, zeroed external RAM","frame_t_cycles":FRAME,"entry":entry_clock,"first_input":first_input,"scenes":scenes,"records":records,"interrupt_timing_samples_t_cycles":interrupt_timings,"completion":{"cpu_t_cycles":cycles,"lcd_t_cycles":lcd}}),
                    );
                }
            }
        }
        let was_idle = entered && cpu.idle_wait();
        let before = cpu.clock().0;
        cpu.step().map_err(anyhow::Error::msg)?;
        if was_idle {
            idle_cycles += cpu.clock().0 - before;
        }
    }
    anyhow::bail!("intro oracle exceeded its bounded instruction limit")
}
