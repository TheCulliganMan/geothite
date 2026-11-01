//! Certify the evolving source sound clock against all independent intro IRQs.
//! Playback and channel shutdowns come from authored calls; ROM RAM snapshots
//! and observed cycle totals never initialize or adjust the predictor.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn certify(vendor: &Path, model: &Value, trace: &Value) -> Result<Value> {
    ensure!(
        trace["rom_sha1"] == "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"
            && trace["scenes"].as_array().is_some_and(|s| s.len() == 28),
        "source-clock audio certificate requires the complete pinned intro trace"
    );
    let entry = &trace["entry"]["memory"];
    ensure!(
        entry["wMusicPlaying"] == 1 && (1..=8).all(|n| entry[format!("wChannel{n}Flags1")] == 0),
        "source sound clock requires the authored empty-channel bootstrap"
    );
    let root = vendor
        .parent()
        .and_then(Path::parent)
        .context("source export workspace")?;
    let catalog: Vec<crystal_assets::ModpackAudioAsset> = serde_json::from_slice(&std::fs::read(
        root.join("apps/web/assets/data/audio.json"),
    )?)?;
    let ids = |file: &str| -> Result<Vec<String>> {
        Ok(std::fs::read_to_string(vendor.join(file))?
            .lines()
            .filter_map(|line| {
                line.trim()
                    .strip_prefix("const ")
                    .and_then(|line| line.split_whitespace().next())
                    .map(str::to_owned)
            })
            .collect())
    };
    let music = ids("constants/music_constants.asm")?;
    let sfx = ids("constants/sfx_constants.asm")?;
    let records = trace["records"]
        .as_array()
        .context("source intro records")?;
    let time = |record: &Value, key: &str| {
        record[key]
            .as_u64()
            .with_context(|| format!("source sound trace lacks {key}"))
    };
    let mut events = records
        .iter()
        .filter(|r| {
            matches!(
                r["hook"].as_str(),
                Some("PlayMusic" | "PlaySFX" | "SFXChannelsOff")
            )
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|r| r["end_cpu_t_cycles"].as_u64().unwrap_or(u64::MAX));
    ensure!(
        events.iter().any(|r| r["hook"] == "PlayMusic")
            && events.iter().any(|r| r["hook"] == "SFXChannelsOff"),
        "source-clock certificate requires PlayMusic and SFXChannelsOff hooks"
    );
    let mut updates = records
        .iter()
        .filter(|r| r["hook"] == "_UpdateSound")
        .collect::<Vec<_>>();
    updates.sort_by_key(|r| r["start_cpu_t_cycles"].as_u64().unwrap_or(u64::MAX));
    let mut clock = crystal_audio::synth::SourceAudioClock::new()?;
    let mut event = 0;
    let mut compared = 0;
    let mut playbacks = 0;
    let mut shutdowns = 0;
    let mut current_sfx_priority = 0u8;
    for record in updates {
        let begin = time(record, "start_cpu_t_cycles")?;
        while event < events.len() && time(events[event], "end_cpu_t_cycles")? <= begin {
            let playback = events[event];
            if playback["hook"] == "SFXChannelsOff" {
                ensure!(
                    time(playback, "instruction_machine_cycles")? == 25,
                    "source SFXChannelsOff body differs from its authored instructions"
                );
                clock.stop_sfx_channels();
                shutdowns += 1;
                event += 1;
                continue;
            }
            let id = usize::try_from(time(playback, "de")?)?;
            let name = if playback["hook"] == "PlayMusic" {
                music.get(id)
            } else {
                sfx.get(id)
            }
            .context("authored source audio identity outside catalog")?;
            let channels = clock.channels();
            let caller = crystal_assets::runtime_intro_source_audio_playback_timing(
                model,
                name,
                &channels,
                Some(current_sfx_priority),
            )?;
            let predicted = caller.instruction_machine_cycles.iter().sum::<u64>();
            let actual = time(playback, "instruction_machine_cycles")?;
            ensure!(
                predicted == actual,
                "actual {name} caller disagrees with evolving source state: predicted {predicted}, actual {actual}"
            );
            if caller.accepted {
                if name == "MUSIC_NONE" {
                    clock.stop_all();
                    current_sfx_priority = 0;
                } else {
                    let asset = catalog
                        .iter()
                        .find(|asset| &asset.id == name)
                        .with_context(|| format!("missing source playback {name}"))?;
                    let program = crystal_audio::synth::decode_midi(
                        &asset
                            .midi_program
                            .as_ref()
                            .context("source playback has no audio program")?
                            .midi_base64,
                    )?;
                    clock.play(&program)?;
                    if playback["hook"] == "PlaySFX" {
                        current_sfx_priority = u8::try_from(id)?;
                    }
                }
            }
            playbacks += 1;
            event += 1;
        }
        // The pinned continuous trace has no audio IRQ inside its playback
        // callers. A changed caller cadence must supply MusicOn/Off hooks,
        // rather than borrowing observed RAM to guess the intermediate state.
        ensure!(
            !events.iter().any(
                |e| time(e, "start_cpu_t_cycles").is_ok_and(|start| start <= begin)
                    && time(e, "end_cpu_t_cycles").is_ok_and(|end| begin < end)
            ),
            "source audio IRQ interrupted a playback transition; instruction-level playback effects are required"
        );
        let before = clock.channels();
        let steps = clock.update_vblank()?;
        let predicted = crystal_assets::runtime_intro_source_audio_update_machine_cycles(
            model,
            &steps,
            &before,
            clock.stereo,
        )?;
        let actual = time(record, "instruction_machine_cycles")?;
        ensure!(
            predicted == actual,
            "actual source sound IRQ {compared}, scene {}, dispatch {} disagrees: predicted {predicted}, actual {actual}",
            record["dispatcher_entry"],
            record["dispatch_tick"]
        );
        compared += 1;
    }
    ensure!(
        compared > 2000 && event == events.len(),
        "source-clock audio certificate lacks full IRQ/playback coverage"
    );
    Ok(
        json!({"schema_version":1,"validated":true,"model":"evolving_source_audio_clock_v1","rom_sha1":trace["rom_sha1"],"sound_irqs_compared":compared,"playback_callers_compared":playbacks,"authored_sfx_shutdowns":shutdowns,"state_reseeding":false}),
    )
}
