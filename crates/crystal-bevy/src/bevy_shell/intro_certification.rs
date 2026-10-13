//! Certify fresh authored data by continuously executing the production intro.
use super::*;

/// No hook state is copied into the live game. Only the initial authored entry
/// is seeded; every later boundary must result from production execution.
pub fn certify_source_intro_timeline(
    pack_path: &Path,
    trace_path: &Path,
) -> Result<crystal_assets::RuntimeTitleScreen> {
    let trace: serde_json::Value = serde_json::from_slice(&std::fs::read(trace_path)?)?;
    anyhow::ensure!(
        trace["rom_sha1"] == "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"
            && trace["runner"] == "pack_core Rust intro oracle",
        "intro certification requires the independent pinned ROM trace"
    );
    let scenes = trace["scenes"]
        .as_array()
        .context("independent scene boundaries missing")?;
    anyhow::ensure!(scenes.len() == 28, "independent intro trace is incomplete");
    let frame_cycles = trace["frame_t_cycles"]
        .as_u64()
        .context("ROM frame duration missing")?;
    anyhow::ensure!(
        frame_cycles == 70224,
        "independent intro trace has an incompatible frame duration"
    );
    let initial_frame = trace["first_input"]["lcd_t_cycles"]
        .as_u64()
        .context("ROM first input clock missing")?
        / frame_cycles;
    let mut boundaries = BTreeMap::new();
    let mut preceding = None;
    for record in trace["records"]
        .as_array()
        .context("independent hook records missing")?
    {
        if record["hook"] == "JoyTextDelay" {
            preceding = Some((
                record["start_frame"]
                    .as_u64()
                    .context("ROM input frame missing")?,
                record["start_phase_t_cycles"]
                    .as_u64()
                    .context("ROM input phase missing")?,
            ));
        } else if record["hook"] == "IntroSceneJumper" && record["dispatch_tick"] == 1 {
            boundaries.insert(
                record["dispatcher_entry"]
                    .as_u64()
                    .context("ROM dispatcher missing")?,
                preceding.context("ROM dispatcher precedes its input hook")?,
            );
        }
    }
    anyhow::ensure!(
        boundaries.len() == 28,
        "independent intro input boundaries are incomplete"
    );
    let asset_root = AssetRoot::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let loaded =
        crystal_assets::read_unverified_compiled_game_pack_for_intro_timing_diagnostic(pack_path)?;
    let runtime = CrystalRuntime::from_unverified_compiled_pack_for_intro_timing_diagnostic(
        &asset_root,
        loaded,
    )?;
    let title = runtime.data().runtime_title_screen.clone();
    let parameters =
        RuntimeIntroPresentationParameters::from_unverified_program_for_timing_diagnostic(
            runtime.title_presentation_program(),
        )?;
    let mut intro =
        VisibleIntroScreen::from_program(parameters, runtime.title_presentation_program())?;
    apply_visible_intro_background_binding(&mut intro, runtime.title_presentation_program())?;
    let initial_phase = u64::from(intro.frame_clock.phase_t_cycles);
    let initial_elapsed = intro.frame_clock.elapsed_t_cycles;
    let sprite_bundle = load_intro_sprite_anim_bundle(runtime.data().sprite_anim_bundle.as_str())?;
    let spawn_identifier = runtime.title_new_game_spawn_identifier()?;
    let mut shell = initialize_bevy_runtime_shell(
        asset_root,
        runtime,
        BevyShellStart::NewGame { spawn_identifier },
        BevyShellConfig::default(),
    )?;
    shell.title_menu = Some(build_visible_title_menu(
        shell.runtime.title_presentation_program(),
        spawn_identifier,
        None,
    )?);
    shell.pending_name_choice = None;
    shell.pending_name_input = None;
    shell.intro_sprite_bundle = Some(sprite_bundle);
    shell.intro_display_screen = Some(intro.clone());
    shell.intro_screen = Some(intro);
    let mut ended = false;
    for _ in 0..10000 {
        if shell.intro_screen.is_none() {
            ended = true;
            break;
        }
        tick_visible_intro_screen(&mut shell).context("execute continuous production intro")?;
    }
    anyhow::ensure!(
        ended,
        "production intro did not finish within its bounded replay"
    );
    anyhow::ensure!(
        shell.title_menu.is_some(),
        "production intro did not hand off to the title menu"
    );
    let completion = shell
        .intro_completion_clock
        .context("production intro did not retain its return clock")?;
    anyhow::ensure!(trace["completion"]["boundary"] == "CrystalIntro_return",
        "independent intro trace stops before the actual source return");
    let return_absolute = trace["completion"]["lcd_t_cycles"].as_u64()
        .context("ROM intro return clock missing")?;
    let actual_elapsed = completion
        .elapsed_t_cycles
        .checked_sub(initial_elapsed)
        .context("production return clock moved backwards")?;
    anyhow::ensure!(
        (
            (initial_phase + actual_elapsed) / frame_cycles,
            u64::from(completion.phase_t_cycles)
        ) == (
            return_absolute / frame_cycles - initial_frame,
            return_absolute % frame_cycles
        ),
        "production intro return differs from ROM: actual=({}, {}), expected=({}, {})",
        (initial_phase + actual_elapsed) / frame_cycles,
        completion.phase_t_cycles,
        return_absolute / frame_cycles - initial_frame,
        return_absolute % frame_cycles
    );
    let expected_joy: Vec<_> = trace["records"].as_array()
        .context("independent hook records missing")?.iter()
        .filter(|record| record["hook"] == "JoyTextDelay").collect();
    anyhow::ensure!(shell.intro_completed_input_boundaries.len() == expected_joy.len(),
        "production input count including exit {} differs from ROM {}",
        shell.intro_completed_input_boundaries.len(), expected_joy.len());
    for (ordinal, (actual, expected)) in shell.intro_completed_input_boundaries.iter()
        .zip(expected_joy).enumerate() {
        let elapsed = actual.clock.elapsed_t_cycles.checked_sub(initial_elapsed)
            .context("production input clock moved backwards")?;
        let frame = (initial_phase + elapsed) / frame_cycles;
        let expected_frame = expected["start_frame"].as_u64().context("ROM input frame missing")?
            .checked_sub(initial_frame).context("ROM input precedes intro")?;
        let expected_phase = expected["start_phase_t_cycles"].as_u64().context("ROM input phase missing")?;
        anyhow::ensure!((frame, u64::from(actual.clock.phase_t_cycles)) == (expected_frame,expected_phase),
            "production input {ordinal} clock differs from ROM: actual=({frame},{}), expected=({expected_frame},{expected_phase})",
            actual.clock.phase_t_cycles);
        for (name, actual_byte) in [("wIntroSceneFrameCounter",actual.scene_frame_counter),
            ("wIntroSceneTimer",actual.scene_timer),("wTextDelayFrames",actual.text_delay_frames)] {
            let expected_byte = expected["start_memory"][name].as_u64()
                .with_context(|| format!("ROM input {ordinal} scalar {name} missing"))?;
            anyhow::ensure!(u64::from(actual_byte) == expected_byte,
                "production input {ordinal} scalar {name} differs from ROM: actual={actual_byte}, expected={expected_byte}");
        }
    }
    let actual_inputs: Vec<_> = shell
        .intro_completed_input_boundaries
        .iter()
        .filter(|boundary| boundary.dispatcher_entry < 28)
        .collect();
    let mut expected_inputs = Vec::new();
    let mut preceding = None;
    for record in trace["records"]
        .as_array()
        .context("independent hook records missing")?
    {
        if record["hook"] == "JoyTextDelay" {
            preceding = Some(record);
        } else if record["hook"] == "IntroSceneJumper" {
            expected_inputs.push((
                record,
                preceding
                    .take()
                    .context("ROM dispatcher lacks its input hook")?,
            ));
        }
    }
    anyhow::ensure!(
        actual_inputs.len() == expected_inputs.len(),
        "production input boundary count {} differs from ROM {}",
        actual_inputs.len(),
        expected_inputs.len()
    );
    let mut observed = Vec::<(usize, u64, u32, u16)>::new();
    for (actual, (dispatch, input)) in actual_inputs.iter().zip(&expected_inputs) {
        let elapsed = actual
            .clock
            .elapsed_t_cycles
            .checked_sub(initial_elapsed)
            .context("production intro clock moved backwards")?;
        let frame = (initial_phase + elapsed) / frame_cycles;
        let index = dispatch["dispatcher_entry"]
            .as_u64()
            .context("ROM dispatcher index missing")?;
        let tick = dispatch["dispatch_tick"]
            .as_u64()
            .context("ROM dispatch tick missing")?;
        let expected_frame = input["start_frame"]
            .as_u64()
            .context("ROM input frame missing")?
            .checked_sub(initial_frame)
            .context("ROM input precedes intro")?;
        let phase = input["start_phase_t_cycles"]
            .as_u64()
            .context("ROM input phase missing")?;
        anyhow::ensure!(
            (
                actual.dispatcher_entry as u64,
                actual.dispatch_tick as u64,
                frame,
                u64::from(actual.clock.phase_t_cycles)
            ) == (index, tick, expected_frame, phase),
            "production input scene {index} tick {tick} differs from ROM: actual=({}, {}, {frame}, {}), expected=({index}, {tick}, {expected_frame}, {phase})",
            actual.dispatcher_entry,
            actual.dispatch_tick,
            actual.clock.phase_t_cycles
        );
        if observed
            .last()
            .is_none_or(|last| last.0 != actual.dispatcher_entry)
        {
            observed.push((
                actual.dispatcher_entry,
                frame,
                actual.clock.phase_t_cycles,
                actual.dispatch_tick,
            ));
        } else if let Some(last) = observed.last_mut() {
            last.3 = actual.dispatch_tick;
        }
    }
    anyhow::ensure!(
        observed.len() == 28,
        "production intro did not traverse all 28 scenes"
    );
    for (actual, expected) in observed.iter().zip(scenes) {
        let index = expected["dispatcher_entry"]
            .as_u64()
            .context("ROM scene index missing")?;
        let (frame, phase) = boundaries[&index];
        let frame = frame
            .checked_sub(initial_frame)
            .context("ROM scene precedes intro entry")?;
        let ticks = expected["dispatch_ticks"]
            .as_u64()
            .context("ROM dispatch count missing")?;
        anyhow::ensure!(
            (actual.0 as u64, actual.1, actual.2 as u64, actual.3 as u64)
                == (index, frame, phase, ticks),
            "production intro scene {index} differs from ROM: actual={actual:?}, expected=({index}, {frame}, {phase}, {ticks})"
        );
    }
    let mut value = serde_json::to_value(title)?;
    fn certify_oracles(value: &mut serde_json::Value) -> Result<usize> {
        let mut count = 0;
        match value {
            serde_json::Value::Object(fields) => {
                if fields
                    .get("runner")
                    .is_some_and(|value| value == "pack_core Rust intro oracle")
                    && fields.contains_key("validated")
                {
                    anyhow::ensure!(
                        fields.get("rom_sha1").is_some_and(
                            |value| value == "f4cd194bdee0d04ca4eac29e09b8e4e9d818c133"
                        ),
                        "intro oracle identity changed"
                    );
                    fields.insert("validated".into(), serde_json::Value::Bool(true));
                    count += 1;
                }
                for child in fields.values_mut() {
                    count += certify_oracles(child)?;
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    count += certify_oracles(child)?;
                }
            }
            _ => {}
        }
        Ok(count)
    }
    let intro = value["program"]["subprograms"]
        .as_array_mut()
        .context("source title subprograms missing")?
        .iter_mut()
        .find(|subprogram| subprogram["id"] == "crystal_intro")
        .context("source CrystalIntro subprogram missing")?;
    anyhow::ensure!(
        certify_oracles(intro)? > 0,
        "source intro lacks timing oracles to certify"
    );
    let title: crystal_assets::RuntimeTitleScreen = serde_json::from_value(value)?;
    crystal_assets::validate_runtime_title_screen(&title)?;
    RuntimeIntroPresentationParameters::from_program(&title.program)?;
    Ok(title)
}
