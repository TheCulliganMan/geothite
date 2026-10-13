//! Assemble the source-lowered intro without substituting synthetic presentation data.
use super::Source;
use anyhow::{Context, Result, ensure};
use crystal_assets::{RuntimePresentationProgram, RuntimePresentationSubprogram};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub(super) fn assemble(root: &Path) -> Result<Value> {
    assemble_with_measurements(root, true)
}

pub(super) fn assemble_unvalidated(root: &Path) -> Result<Value> {
    assemble_with_measurements(root, false)
}

fn attach_dispatcher_timing(scene_dispatch: &mut Value, timing: &Value) -> Result<()> {
    let instructions = timing["source_dispatcher_instruction_machine_cycles"]
        .as_array()
        .context("source intro dispatcher lacks its instruction sequence")?;
    ensure!(
        instructions
            .iter()
            .all(|cost| cost.as_u64().is_some_and(|cost| (1..=6).contains(&cost)))
            && !instructions.is_empty(),
        "source intro dispatcher has invalid instruction costs"
    );
    ensure!(
        timing["source_dispatcher_caller_call_included"].as_bool() == Some(false),
        "source intro dispatcher double counts its outer CALL"
    );
    scene_dispatch["source_dispatcher_instruction_machine_cycles"] =
        timing["source_dispatcher_instruction_machine_cycles"].clone();
    scene_dispatch["source_dispatcher_caller_call_included"] = json!(false);
    Ok(())
}

fn assemble_with_measurements(root: &Path, measured: bool) -> Result<Value> {
    let vendor = root.join("vendor/pokecrystal");
    let constants: Value = serde_json::from_slice(&std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let source = Source::read(&vendor, "engine/movie/intro.asm")?;
    let mut dispatch = super::source_title_intro_scenes::scene_dispatch(&vendor, &constants)?;
    let trace = super::intro_trace(&vendor)?;
    let transfers = dispatch["transfer_metadata"].clone();
    let timing = if measured {
        super::intro_bind_measurements(&vendor, &mut dispatch["phase"], &transfers, &trace)?
    } else {
        json!({"rom_frame_crossings":[],"timing_oracle":{"runner":"pack_core Rust intro oracle","schema_version":1,"validated":false,"rom_sha1":trace["rom_sha1"],"instruction_core":"sm83_mcycle_v1","timer_model":"cgb_timer_falling_edge_delayed_reload_v1"}})
    };
    let sprites = super::intro_sprites(&vendor)?;
    let entry_cleanup = super::intro_entry_cleanup(&vendor, &constants)?;
    let clock = super::intro_clock(&vendor, &trace)?;
    let certification = if measured {
        let control_calls = super::certify_intro_control(&dispatch["phase"], &trace)?;
        let audio_proof = super::certify_intro_audio(&vendor, &clock["vblank_normal"]["audio_update"], &trace)?;
        json!({"scene_count":28,"decompression_count":37,"sprite_allocation_count":19,"control_dispatch_calls":control_calls,"continuous_control_state_matches_rom":true,"audio_irq_model_matches_rom":audio_proof["audio_irq_model_matches_rom"],"sound_update_count":audio_proof["sound_update_count"],"audio_helper_count":audio_proof["helper_count"],"independent_audio_state_count":audio_proof["independent_replay_state_count"]})
    } else { Value::Null };
    let labels = transfers["scene_labels"].as_array().context("authored scene labels")?;
    ensure!(labels.len() == 28, "incomplete source intro dispatch");
    let mut entry_offsets = serde_json::Map::new();
    let mut waits = Vec::new();
    let operations = dispatch["phase"]["operations"].as_array().context("authored scene operations")?;
    for (index, label) in labels.iter().enumerate() {
        let label = label.as_str().context("authored scene label")?;
        let offset = dispatch["phase"]["labels"][label].as_u64().context("authored scene offset")?;
        entry_offsets.insert(label.into(), json!(offset));
        let end = labels.get(index + 1).and_then(Value::as_str).and_then(|next|dispatch["phase"]["labels"][next].as_u64()).unwrap_or(operations.len() as u64);
        let frames = operations[offset as usize..end as usize].iter().filter(|operation| operation["op"] == "wait_frames" && operation["source_span"]["file"] == "engine/movie/intro.asm").try_fold(0u8, |sum, operation| -> Result<u8> {
            let frames = u8::try_from(operation["frames"].as_u64().context("authored scene wait")?)?;
            sum.checked_add(frames).context("authored scene wait overflow")
        })?;
        waits.push(frames);
    }
    let frame_wait = source.operation("CrystalIntro", &["call DelayFrame"], "wait_frames", json!({"frames":1}))?;
    let table = source.operation("IntroSceneJumper", &["jumptable IntroScenes, wJumptableIndex"], "dispatch_table", json!({"table":"IntroScenes","source":"wJumptableIndex","index":"wJumptableIndex","entries":labels}))?;
    dispatch["phase"]["operations"].as_array_mut().context("scene operations")?.push(table);
    dispatch["phase"]["id"] = json!("scene_dispatch");
    dispatch["phase"]["source_span"] = source.span("IntroSceneJumper")?;
    dispatch["phase"].as_object_mut().context("scene phase")?.remove("entry_label");
    let mut scheduler = sprites["scheduler"].clone();
    scheduler["rom_frame_crossings"] = timing["rom_frame_crossings"].clone();
    scheduler["timing_oracle"] = timing["timing_oracle"].clone();
    let mut resources = Vec::new();
    let mut subprogram_resources = Vec::new();
    let mut audio = Vec::new();
    let mut audio_seen = std::collections::BTreeSet::new();
    for (line, code) in &source.lines {
        let span = json!({"file":source.file,"start_line":line,"end_line":line});
        if let Some(path) = code.strip_prefix("INCBIN \"").or_else(||code.strip_prefix("INCLUDE \"")).and_then(|path|path.strip_suffix('"')) {
            ensure!(path.starts_with("gfx/"), "unexpected intro resource {path}");
            resources.push(json!({"path":path,"kind":if path.ends_with(".pal"){"palette"}else{"graphics"},"source_span":span}));
            subprogram_resources.push(json!({"path":path,"kind":if path.ends_with(".pal"){"palette"}else{"graphics"},"include_source_span":span,"data_source_span":span}));
        }
        for token in code.split(|ch:char| !(ch.is_ascii_alphanumeric() || ch=='_')) {
            let kind = if token.starts_with("MUSIC_") {"music"} else if token.starts_with("SFX_") {"sound_effect"} else {continue};
            if token != "MUSIC_NONE" && audio_seen.insert(token.to_owned()) {audio.push(json!({"id":token,"kind":kind,"source_span":span}));}
        }
    }
    let input = source.operation("CrystalIntro", &["call JoyTextDelay","ldh a, [hJoyLast]","and PAD_BUTTONS","jr nz, .ShutOffMusic"], "input_poll", json!({"source":"hJoyLast","mask":constants.get("PAD_BUTTONS").context("PAD_BUTTONS")?,"cancel_target":".ShutOffMusic"}))?;
    let mut subprogram = serde_json::to_value(RuntimePresentationSubprogram::default())?;
    subprogram["id"] = json!("crystal_intro");
    subprogram["source_entry"] = json!("CrystalIntro");
    subprogram["accepted_call_forms"] = json!(["farcall"]);
    subprogram["result"] = json!({"completion":"JUMPTABLE_EXIT_F","cancelled_by":"PAD_BUTTONS"});
    subprogram["source_span"] = source.span("CrystalIntro")?;
    subprogram["phases"] = json!([entry_cleanup["entry_init"], dispatch["phase"], entry_cleanup["cleanup"], entry_cleanup["button_cancel"]]);
    let mut scene_dispatch = json!({"table":"IntroScenes","index":"wJumptableIndex","entries":labels,"entry_offsets":entry_offsets,"completion_wait_frames":waits});
    if measured { attach_dispatcher_timing(&mut scene_dispatch, &timing)?; }
    subprogram["loop"] = json!({"input_poll":input,"frame_wait":frame_wait,"scene_dispatch":scene_dispatch,"scheduler":scheduler,"interrupt_timing":clock});
    subprogram["resource_transfers"] = transfers["resource_transfers"].clone();
    subprogram["resources"] = json!(subprogram_resources);
    subprogram["sprite_programs"] = sprites["sprite_programs"].clone();
    subprogram["sprite_operations"] = sprites["activation_operations"].clone();
    subprogram["required_consumer"] = sprites["required_consumer"].clone();
    subprogram["required_consumer"]["certification"] = certification;
    subprogram["audio"] = json!(audio);
    let mut spans = dispatch["implementation_spans"].as_array().context("scene implementation spans")?.clone();
    spans.extend(entry_cleanup["implementation_source_spans"].as_array().context("intro cleanup spans")?.iter().cloned());
    subprogram["implementation_source_spans"] = json!(spans);
    let mut program = serde_json::to_value(RuntimePresentationProgram::default())?;
    program["schema_version"] = json!(1);
    program["subprograms"] = json!([subprogram]);
    program["resources"] = json!(resources);
    program["audio"] = json!(audio);
    let _: RuntimePresentationProgram = serde_json::from_value(program.clone()).context("assembled authored intro schema")?;
    Ok(program)
}

#[cfg(test)]
mod timing_wiring_tests {
    use super::*;

    #[test]
    fn dispatcher_timing_survives_program_schema_roundtrip_without_outer_call() {
        let timing = json!({"source_dispatcher_instruction_machine_cycles":[4,1,2,3,2,2,2,2,1,1],"source_dispatcher_caller_call_included":false});
        let mut dispatch = json!({"table":"IntroScenes"});
        attach_dispatcher_timing(&mut dispatch, &timing).unwrap();
        let mut subprogram = RuntimePresentationSubprogram::default();
        subprogram.loop_ = json!({"scene_dispatch":dispatch});
        let mut program = RuntimePresentationProgram::default();
        program.subprograms.push(subprogram);
        let roundtrip: RuntimePresentationProgram =
            serde_json::from_slice(&serde_json::to_vec(&program).unwrap()).unwrap();
        assert_eq!(
            roundtrip.subprograms[0].loop_["scene_dispatch"]["source_dispatcher_instruction_machine_cycles"],
            timing["source_dispatcher_instruction_machine_cycles"]
        );
        assert_eq!(
            roundtrip.subprograms[0].loop_["scene_dispatch"]["source_dispatcher_caller_call_included"],
            json!(false)
        );
        let mut malformed = timing.clone();
        malformed["source_dispatcher_caller_call_included"] = json!(true);
        assert!(attach_dispatcher_timing(&mut dispatch, &malformed).is_err());
    }
}
