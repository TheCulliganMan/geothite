#[path = "../src/bin/pack_core/source_syntax.rs"]
mod source_syntax;
mod radio_text_export {
    pub(crate) use super::source_syntax::code_part;
}
#[path = "../src/bin/pack_core/source_expressions.rs"]
mod source_expressions;
#[path = "../src/bin/pack_core/source_title_screen.rs"]
mod source_title_screen;
#[path = "../src/bin/pack_core/source_title_catalog.rs"]
mod source_title_catalog;

#[test]
fn source_transfers_bind_to_exact_rom_resources_and_vram_banks() {
    let Some(root)=std::env::var_os("CRYSTAL_EXPORT_WORKSPACE") else {return};
    let root=std::path::Path::new(&root);let vendor=root.join("vendor/pokecrystal");
    let constants:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json")).unwrap()).unwrap();
    let constants=serde_json::from_value(constants["global"].clone()).unwrap();
    let trace=source_title_screen::intro_trace(&vendor).unwrap();
    let phases=source_title_screen::intro_setup_phases(&vendor,&constants).unwrap();
    let operations=(1..=28).filter_map(|number|phases.get(format!("IntroScene{number}"))).flat_map(|phase|phase["operations"].as_array().unwrap().iter().cloned()).collect::<Vec<_>>();
    let mut phase=serde_json::json!({"operations":operations});
    let metadata=source_title_screen::intro_transfer_metadata(&vendor).unwrap();
    let scheduler=source_title_screen::intro_bind_measurements(&vendor,&mut phase,&metadata,&trace).unwrap();
    assert_eq!(scheduler["rom_frame_crossings"].as_array().unwrap().len(),23);
    for operation in phase["operations"].as_array().unwrap().iter().filter(|operation|operation["op"]=="decompress_lz3_resource") {
        assert!(operation["decompress_machine_cycles"].as_u64().unwrap()>0);
        assert_eq!(operation["timing_oracle"]["validated"],false);
    }
}

#[test]
fn source_intro_clock_uses_measured_interrupt_and_joypad_paths() {
    let Some(vendor) = std::env::var_os("CRYSTAL_PRET_SOURCE_ROOT") else {
        return;
    };
    let vendor = std::path::Path::new(&vendor);
    let trace = source_title_screen::intro_trace(vendor).unwrap();
    let clock = source_title_screen::intro_clock(vendor, &trace).unwrap();
    assert_eq!(clock["entry_to_first_input_machine_cycles"], 59);
    assert_eq!(
        clock["outer_loop_body"],
        serde_json::json!({"after_input_before_scene_dispatch":21,"scene_dispatch_to_sprite_scheduler":48,"sprite_scheduler_to_frame_wait":49})
    );
    for (path, value) in [
        ("/joy_text_delay/pressed_repeat_reset_machine_cycles", 101),
        ("/joy_text_delay/repeat_suppressed_machine_cycles", 107),
        ("/joy_text_delay/repeat_restart_machine_cycles", 110),
        ("/lcd_stat/callback_zero_machine_cycles", 27),
        ("/lcd_stat/callback_nonzero_machine_cycles", 49),
        ("/timer/inactive_machine_cycles", 48),
    ] {
        assert_eq!(clock.pointer(path).unwrap(), value);
    }
    assert_eq!(clock["frame_clock"]["timing_oracle"]["validated"], false);
}

fn program() -> Option<crystal_assets::RuntimePresentationProgram> {
    let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_EXPORT_WORKSPACE")?);
    let constants: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
            .unwrap(),
    )
    .unwrap();
    let constants = serde_json::from_value(constants["global"].clone()).unwrap();
    let vendor = root.join("vendor/pokecrystal");
    let entry_cleanup = source_title_screen::intro_entry_cleanup(&vendor, &constants).unwrap();
    assert_eq!(entry_cleanup["button_cancel"]["operations"].as_array().unwrap().len(), 1);
    assert_eq!(entry_cleanup["button_cancel"]["operations"][0]["op"], "stop_audio");
    assert_eq!(entry_cleanup["button_cancel"]["operations"][0]["audio"], "MUSIC_NONE");
    assert_eq!(
        entry_cleanup["entry_init"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    assert_eq!(
        entry_cleanup["cleanup"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        19
    );
    let mut program = crystal_assets::RuntimePresentationProgram::default();
    for (name, block) in source_title_screen::startup_blocks(&vendor, &constants).unwrap() {
        program
            .blocks
            .insert(name, serde_json::from_value(block).unwrap());
    }
    program
        .entrypoints
        .insert("main_menu".into(), "Intro_MainMenu".into());
    program.blocks.insert(
        "Intro_MainMenu".into(),
        serde_json::from_value(source_title_screen::main_menu_entry(&vendor).unwrap()).unwrap(),
    );
    for value in [
        source_title_screen::main_menu(&vendor, &constants).unwrap(),
        source_title_screen::gender(&vendor, &constants).unwrap(),
        source_title_screen::title(&vendor, &constants).unwrap(),
        source_title_screen::oak_speech(&vendor, &constants).unwrap(),
    ] {
        program
            .subprograms
            .push(serde_json::from_value(value).unwrap());
    }
    program.text = source_title_screen::oak_text(&vendor)
        .unwrap()
        .into_iter()
        .map(|value| serde_json::from_value(value).unwrap())
        .collect();
    program.text.extend(
        source_title_screen::startup_text(&vendor)
            .unwrap()
            .into_iter()
            .map(|value| serde_json::from_value(value).unwrap()),
    );
    Some(program)
}

#[test]
fn source_main_menu_entry_preserves_music_stop_wait_restart_and_call_order() {
    let Some(program) = program() else {
        return;
    };
    let block = &program.blocks["Intro_MainMenu"];
    assert_eq!(
        block
            .operations
            .iter()
            .map(|operation| operation.op.as_str())
            .collect::<Vec<_>>(),
        [
            "stop_audio",
            "wait_frames",
            "write_memory_byte",
            "play_audio",
            "call_subprogram",
            "jump"
        ]
    );
    let mut interpreter =
        crystal_assets::RuntimePresentationInterpreter::new(&program, "main_menu").unwrap();
    for operation in &block.operations[..5] {
        let crystal_assets::RuntimePresentationStep::Operation(actual) =
            interpreter.step(&program).unwrap()
        else {
            panic!("main menu advanced before its authored call");
        };
        assert_eq!(&actual, operation);
    }
    assert_eq!(block.operations[0].fields["audio"], "MUSIC_NONE");
    assert_eq!(block.operations[1].fields["frames"], 1);
    assert_eq!(block.operations[3].fields["audio"], "MUSIC_MAIN_MENU");
    assert_eq!(block.operations[4].fields["program"], "main_menu");
}

#[test]
fn source_oak_sequence_preserves_four_portraits_and_embedded_cry_continuation() {
    let Some(program) = program() else {
        return;
    };
    let oak = program
        .subprograms
        .iter()
        .find(|subprogram| subprogram.id == "oak_speech")
        .unwrap();
    let speech = &oak.phases[0];
    let clock = speech
        .operations
        .iter()
        .find(|operation| operation.op == "initialize_clock")
        .unwrap();
    assert_eq!(
        clock.fields["startup_sequence"].as_array().unwrap().len(),
        15
    );
    assert_eq!(clock.fields["text_entries"].as_array().unwrap().len(), 10);
    for (key, value) in [
        ("press_counter", 15),
        ("restart_counter", 5),
        ("first_repeat_frames", 10),
        ("later_repeat_frames", 5),
    ] {
        assert_eq!(clock.fields["held_direction_repeat"][key], value);
    }
    assert_eq!(
        clock.fields["response_text_by_period"]["morn"],
        "OakTimeOversleptText"
    );
    let naming = speech
        .operations
        .iter()
        .find(|operation| operation.op == "name_player")
        .unwrap();
    for gender in ["male", "female"] {
        assert_eq!(
            naming.fields["preset_choices"][gender]
                .as_array()
                .unwrap()
                .len(),
            5
        );
    }
    assert_eq!(naming.fields["picture_motion"]["steps"], 8);
    assert_eq!(naming.fields["picture_motion"]["bg_map_wait_frames"], 4);
    assert_eq!(naming.fields["picture_motion"]["delay_frames"], 1);
    assert_eq!(
        naming.fields["custom_return"]["fade_out"],
        serde_json::json!({"steps":3,"frames_per_step":8})
    );
    let portraits = speech
        .operations
        .iter()
        .filter(|operation| operation.op == "present_intro_portrait")
        .collect::<Vec<_>>();
    assert_eq!(portraits.len(), 4);
    assert_eq!(portraits[0].fields["trainer_class"], "POKEMON_PROF");
    assert_eq!(portraits[1].fields["species"], "WOOPER");
    assert_eq!(portraits[2].fields["trainer_class"], "POKEMON_PROF");
    assert_eq!(portraits[3].fields["kind"], "player");
    assert_eq!(
        portraits[0].fields["fade"],
        serde_json::json!({"words":6,"frames_per_word":10})
    );
    assert_eq!(portraits[1].fields["wipe"]["delta"], -8);
    let sequences = speech
        .operations
        .iter()
        .filter(|operation| operation.op == "present_text_sequence")
        .collect::<Vec<_>>();
    assert_eq!(sequences.len(), 5);
    assert_eq!(
        sequences[1].fields["entries"],
        serde_json::json!(["OakText2", "OakText3", "OakText4"])
    );
    assert_eq!(
        sequences[1].fields["embedded_audio"],
        serde_json::json!([{"after":"OakText2","cry":"WOOPER","wait_for_sfx":true}])
    );
    let continuation = program
        .text
        .iter()
        .find(|text| text.id == "OakText3")
        .unwrap();
    assert_eq!(
        continuation
            .commands
            .iter()
            .map(|command| command.command.as_str())
            .collect::<Vec<_>>(),
        ["text_promptbutton", "text_end"]
    );
    assert!(
        program
            .text
            .iter()
            .find(|text| text.id == "OakText7")
            .unwrap()
            .commands
            .iter()
            .any(|command| command.args.iter().any(|arg| arg.contains("<PLAYER>")))
    );
    for pair in speech.operations.windows(2) {
        assert!(
            pair[0].source_span.end_line < pair[1].source_span.start_line,
            "Oak instructions must be consumed exactly once in source order"
        );
    }
}

#[test]
fn source_menu_geometry_and_title_timing_match_production_consumers() {
    let Some(program) = program() else {
        eprintln!("set CRYSTAL_EXPORT_WORKSPACE for authored title verification");
        return;
    };
    let menu = crystal_assets::RuntimeTitleMainMenuDefinition::from_program(&program).unwrap();
    assert_eq!(menu.variants.len(), 9);
    assert_eq!(
        (menu.left, menu.top, menu.right, menu.bottom),
        (0, 0, 16, 7)
    );
    let gender = crystal_assets::RuntimeGenderMenuDefinition::from_program(&program).unwrap();
    assert_eq!(gender.items, ["Boy", "Girl"]);
    assert_eq!(gender.confirm_delay_frames, 10);
    let title = crystal_assets::RuntimeTitlePresentationParameters::from_program(&program).unwrap();
    assert_eq!(title.timeout_frames, 4416);
    assert_eq!(title.timeout_fade_frames, 64);
    assert_eq!(title.suicune_frames, [128, 136, 0, 8]);
}

#[test]
fn source_title_input_runs_authoritative_start_delete_and_clock_phases() {
    let Some(program) = program() else {
        return;
    };
    let parameters =
        crystal_assets::RuntimeTitlePresentationParameters::from_program(&program).unwrap();
    let make = || {
        let mut machine = crystal_assets::RuntimePresentationPhaseMachine::new(
            &program,
            "start_title_screen",
            "title_screen",
        )
        .unwrap();
        for (name, value) in [
            ("wJumptableIndex", 2),
            ("hClockResetTrigger", 0),
            ("wTitleScreenTimer", 4416),
            ("wTitleScreenSelectedOption", 0),
        ] {
            machine.memory.insert(name.into(), value);
        }
        machine
    };
    for (input, option) in [(parameters.start_mask, 0), (parameters.delete_save_mask, 1)] {
        let mut machine = make();
        machine
            .run_from_label(&program, "TitleScreenMain", input)
            .unwrap();
        assert_eq!(machine.memory["wTitleScreenSelectedOption"], option);
        assert_eq!(machine.memory["wJumptableIndex"] & 128, 128);
        assert_eq!(machine.memory["wTitleScreenTimer"], 4415);
    }
    let mut machine = make();
    machine
        .run_from_label(&program, "TitleScreenMain", parameters.clock_reset_arm_mask)
        .unwrap();
    assert_eq!(machine.memory["wJumptableIndex"], 2);
    assert_eq!(machine.memory["hClockResetTrigger"], 0x34);
    machine
        .run_from_label(
            &program,
            "TitleScreenMain",
            parameters.clock_reset_finish_mask,
        )
        .unwrap();
    assert_eq!(machine.memory["wTitleScreenSelectedOption"], 4);
    assert_eq!(machine.memory["hClockResetTrigger"], 0);
    assert_eq!(machine.memory["wJumptableIndex"] & 128, 128);
}

#[test]
fn source_clock_and_player_naming_keep_authored_data_and_frame_cadence() {
    let Some(program) = program() else { return };
    let oak = program
        .subprograms
        .iter()
        .find(|subprogram| subprogram.id == "oak_speech")
        .unwrap();
    let operations = &oak.phases[0].operations;
    let clock = operations
        .iter()
        .find(|operation| operation.op == "initialize_clock")
        .unwrap();
    assert_eq!(
        clock.fields["startup_sequence"].as_array().unwrap().len(),
        15
    );
    assert_eq!(clock.fields["text_entries"].as_array().unwrap().len(), 10);
    assert_eq!(clock.fields["default_hour"], 10);
    assert_eq!(clock.fields["default_minute"], 0);
    assert_eq!(
        clock.fields["held_direction_repeat"]["first_repeat_frames"],
        10
    );
    assert_eq!(
        clock.fields["held_direction_repeat"]["later_repeat_frames"],
        5
    );
    for id in clock.fields["text_entries"].as_array().unwrap() {
        assert!(
            program
                .text
                .iter()
                .any(|text| Some(text.id.as_str()) == id.as_str())
        );
    }
    assert_eq!(
        clock.fields["response_text_by_period"]["nite"],
        "OakTimeSoDarkText"
    );
    let naming = operations
        .iter()
        .find(|operation| operation.op == "name_player")
        .unwrap();
    assert_eq!(
        naming.fields["preset_choices"]["male"],
        serde_json::json!(["NEW NAME", "CHRIS", "MAT", "ALLAN", "JON"])
    );
    assert_eq!(
        naming.fields["preset_choices"]["female"],
        serde_json::json!(["NEW NAME", "KRIS", "AMANDA", "JUANA", "JODI"])
    );
    assert_eq!(naming.fields["picture_motion"]["steps"], 8);
    assert_eq!(naming.fields["picture_motion"]["bg_map_wait_frames"], 4);
    assert_eq!(naming.fields["picture_motion"]["delay_frames"], 1);
    assert_eq!(
        naming.fields["custom_return"]["fade_out"],
        serde_json::json!({"steps":3,"frames_per_step":8})
    );
    assert_eq!(naming.fields["preset_menu"]["title"], " NAME ");
}
