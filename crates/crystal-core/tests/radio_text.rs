use crystal_core::systems::radio_text::{
    RADIO_SCROLL, RadioLinePrinter, RadioScrollState, RadioTextEnvironment, RadioTextError,
    RadioTextWindow,
};
use serde_json::Value;

struct Environment;
impl RadioTextEnvironment for Environment {
    fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
        if address == 0x1234 {
            Ok(vec![0x54, 0x80, 0x50])
        } else {
            Err(RadioTextError::Ram(address))
        }
    }
    fn weekday(&self) -> u8 {
        3
    }
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}
fn window(value: &Value) -> RadioTextWindow {
    let bytes = bytes(value);
    assert_eq!(bytes.len(), 120);
    RadioTextWindow {
        tiles: std::array::from_fn(|row| bytes[row * 20..row * 20 + 20].try_into().unwrap()),
    }
}
fn state(printed: u8) -> RadioScrollState {
    RadioScrollState {
        current_line: 7,
        next_line: 0,
        delay: 0,
        printed,
    }
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../tools/asm-oracle/fixtures/radio-program-rocket.json"
    ))
    .unwrap()
}

#[test]
fn ordinary_radio_text_preserves_the_first_character_and_explicit_coordinates() {
    // PrintTextboxText / PlaceString do not enter PrintRadioLine, whose first
    // two calls overwrite byte 1 and select their own row.
    let text = [0, 0x80, 0x81, 0x50, 0x50];
    for (column, row) in [(1, 14), (9, 14), (1, 16), (12, 16)] {
        let mut window = RadioTextWindow { tiles: [[0x7f; 20]; 6] };
        let mut printer = RadioLinePrinter::text_at(&text, column, row).unwrap();
        assert!(printer.advance_frame(&mut window, &Environment, false).unwrap());
        let mut expected = [[0x7f; 20]; 6];
        expected[usize::from(row - 12)][usize::from(column)] = 0x80;
        expected[usize::from(row - 12)][usize::from(column + 1)] = 0x81;
        assert_eq!(window.tiles, expected);
    }
    for (column, row) in [(20, 14), (1, 11), (1, 18)] {
        assert!(matches!(RadioLinePrinter::text_at(&text, column, row), Err(RadioTextError::Position(_))));
    }
}

#[test]
fn radio_printer_replays_every_rocket_rom_tile_write_and_explicit_pause() {
    let fixture = fixture();
    let prints = fixture["prints"].as_array().unwrap();
    assert_eq!(prints.len(), 17);
    assert!(fixture["unfinished_print"].is_null());
    for record in prints {
        let mut window = window(&record["tiles_before"]);
        let mut state = state(record["printed_before"].as_u64().unwrap() as u8);
        let next = record["next_line"].as_u64().unwrap() as u8;
        let mut printer =
            RadioLinePrinter::new(&bytes(&record["text_bytes"]), &mut state, next).unwrap();
        let mut waited = 0;
        while !printer
            .advance_frame(&mut window, &Environment, false)
            .unwrap()
        {
            waited += 1;
            assert!(waited <= 60, "printer stalled at {record}");
        }
        assert_eq!(
            window,
            self::window(&record["tiles_after"]),
            "source print at {}",
            record["start"]
        );
        assert_eq!(
            state.printed,
            record["printed_after"].as_u64().unwrap() as u8
        );
        assert_eq!(state.next_line, next);
        let start = record["start"].as_u64().unwrap();
        let end = record["end"].as_u64().unwrap();
        let source_wait: u64 = fixture["pauses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|pause| {
                pause["start"].as_u64().unwrap() >= start && pause["end"].as_u64().unwrap() <= end
            })
            .map(|pause| pause["end"].as_u64().unwrap() - pause["start"].as_u64().unwrap())
            .sum();
        assert_eq!(waited, source_wait);
        // Source CPU work can cross a VBlank even without DelayFrames. The
        // printer models explicit waits; it does not invent CPU timing.
        assert!(end - start >= waited);
        state.finish_print();
        assert_eq!((state.current_line, state.delay), (RADIO_SCROLL, 100));
    }
}

#[test]
fn radio_scroll_replays_the_rom_countdown_and_full_tile_rows() {
    let fixture = fixture();
    let loops = fixture["loops"].as_array().unwrap();
    let mut transitions = 0;
    for pair in loops.windows(2) {
        let [before, after] = pair else {
            unreachable!()
        };
        if before["line"] != RADIO_SCROLL {
            continue;
        }
        let mut state = RadioScrollState {
            current_line: RADIO_SCROLL,
            next_line: before["next_line"].as_u64().unwrap() as u8,
            delay: before["delay"].as_u64().unwrap() as u8,
            printed: before["printed"].as_u64().unwrap() as u8,
        };
        let mut tiles = window(&before["textbox_tiles"]);
        if state.delay == 0 {
            transitions += 1;
        }
        state.step(&mut tiles);
        assert_eq!(
            u64::from(state.current_line),
            after["line"].as_u64().unwrap()
        );
        assert_eq!(u64::from(state.delay), after["delay"].as_u64().unwrap());
        assert_eq!(
            tiles,
            window(&after["textbox_tiles"]),
            "scroll at {}",
            before["frame"]
        );
    }
    assert_eq!(transitions, 16);
}

#[test]
fn radio_pause_samples_held_buttons_only_when_entered() {
    let text = [0, 0x4f, 0x80, 0x50, 0x0a, 0, 0x81, 0x57];
    for held_at_entry in [false, true] {
        let mut state = state(2);
        let mut window = RadioTextWindow {
            tiles: [[0x7f; 20]; 6],
        };
        let mut printer = RadioLinePrinter::new(&text, &mut state, 8).unwrap();
        assert_eq!(
            printer
                .advance_frame(&mut window, &Environment, held_at_entry)
                .unwrap(),
            held_at_entry
        );
        assert_eq!(window.tiles[4][1], 0x80);
        if !held_at_entry {
            assert_eq!(
                window.tiles[4][2], 0x7f,
                "pause must not print an ellipsis or later text"
            );
            for remaining in (1..30).rev() {
                assert!(
                    !printer
                        .advance_frame(&mut window, &Environment, true)
                        .unwrap()
                );
                assert_eq!(printer.wait_frames(), remaining);
                assert_eq!(window.tiles[4][2], 0x7f);
            }
            assert!(
                printer
                    .advance_frame(&mut window, &Environment, true)
                    .unwrap()
            );
        }
        assert_eq!(window.tiles[4][2], 0x81);
    }
}

#[test]
fn radio_ram_and_day_commands_keep_place_string_expansion() {
    let mut state = state(2);
    let mut window = RadioTextWindow {
        tiles: [[0x7f; 20]; 6],
    };
    let mut printer = RadioLinePrinter::new(&[1, 0x34, 0x12, 0x15, 0x50], &mut state, 0).unwrap();
    assert!(
        printer
            .advance_frame(&mut window, &Environment, false)
            .unwrap()
    );
    assert_eq!(
        &window.tiles[2][1..15],
        &[
            0x8f, 0x8e, 0x8a, 0xea, 0x80, 0x96, 0x84, 0x83, 0x8d, 0x84, 0x92, 0x83, 0x80, 0x98
        ]
    );
}

fn exported_radio_bodies()
-> std::collections::BTreeMap<String, crystal_core::systems::script_text::ScriptTextBody> {
    use crystal_core::systems::script_text::{ScriptTextBody, ScriptTextBodyCommand};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let source =
        std::fs::read_to_string(root.join("vendor/pokecrystal/engine/pokegear/radio.asm")).unwrap();
    let catalog: Value = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/story_events/StandardScripts.json"))
            .unwrap(),
    )
    .unwrap();
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("text_far ").map(str::trim))
        .map(|label| {
            let commands = catalog["StandardScripts"][label]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(index, command)| {
                    let mut command = command.clone();
                    command["command_index"] = index.into();
                    serde_json::from_value::<ScriptTextBodyCommand>(command).unwrap()
                })
                .collect();
            (
                label.to_string(),
                ScriptTextBody {
                    label: label.into(),
                    commands,
                },
            )
        })
        .collect()
}

struct CapturedEnvironment<'a>(&'a Value);
impl RadioTextEnvironment for CapturedEnvironment<'_> {
    fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
        self.0["ram_text"]
            .get(address.to_string())
            .map(bytes)
            .ok_or(RadioTextError::Ram(address))
    }
    fn weekday(&self) -> u8 {
        self.0["weekday"].as_u64().unwrap() as u8
    }
}

#[test]
fn radio_exported_commands_compile_to_source_bytes_and_replay_dynamic_lucky_text() {
    use crystal_core::systems::radio_text::encode_radio_text_body;
    let bodies = exported_radio_bodies();
    assert_eq!(bodies.len(), 113);
    let rocket = fixture();
    let ram_symbols = serde_json::from_value(rocket["ram_symbols"].clone()).unwrap();
    for body in bodies.values() {
        let encoded = encode_radio_text_body(body, &ram_symbols).unwrap();
        assert!(
            encoded.len() <= 40,
            "{} exceeds the source radio copy: {}",
            body.label,
            encoded.len()
        );
    }
    let lucky: Value = serde_json::from_str(include_str!(
        "../../../../tools/asm-oracle/fixtures/radio-program-lucky.json"
    ))
    .unwrap();
    assert_eq!(lucky["prints"].as_array().unwrap().len(), 18);
    assert!(lucky["unfinished_print"].is_null());
    for (fixture, is_lucky) in [(&rocket, false), (&lucky, true)] {
        for record in fixture["prints"].as_array().unwrap() {
            let next = record["next_line"].as_u64().unwrap() as u8;
            let label = if is_lucky {
                let number = match next {
                    30..=38 => next - 29,
                    39 => 7,
                    40 => 8,
                    41 => 10,
                    3 => 11,
                    _ => panic!("unexpected source Lucky program transition {next}"),
                };
                format!("_LC_Text{number}")
            } else {
                format!("_RocketRadioText{}", if next == 7 { 10 } else { next - 49 })
            };
            let encoded = encode_radio_text_body(&bodies[&label], &ram_symbols).unwrap();
            let source = bytes(&record["text_bytes"]);
            assert_eq!(encoded, &source[..encoded.len()], "{label} source bytes");
            let mut state = state(record["printed_before"].as_u64().unwrap() as u8);
            let mut window = window(&record["tiles_before"]);
            let mut printer = RadioLinePrinter::new(&encoded, &mut state, next).unwrap();
            let mut frames = 0;
            while !printer
                .advance_frame(&mut window, &CapturedEnvironment(record), false)
                .unwrap()
            {
                frames += 1;
                assert!(frames <= 60);
            }
            assert_eq!(
                window,
                self::window(&record["tiles_after"]),
                "{label} compiled-text tile output"
            );
        }
    }
}

#[test]
fn radio_encoder_preserves_controls_and_rejects_missing_operands() {
    use crystal_core::systems::radio_text::{encode_radio_string, encode_radio_text_body};
    use crystal_core::systems::script_text::{ScriptTextBody, ScriptTextBodyCommand};
    assert_eq!(
        encode_radio_string("#<POKE><PC><TM><TRAINER><ROCKET>I'd@…").unwrap(),
        [0x54, 0x24, 0x5b, 0x5c, 0x5d, 0x5e, 0x88, 0xd0, 0x50, 0x75]
    );
    assert!(encode_radio_string("🙂").is_err());
    let mut body = ScriptTextBody {
        label: "BadRadio".into(),
        commands: vec![
            ScriptTextBodyCommand {
                command: "text_ram".into(),
                args: vec!["missing".into()],
                command_index: 0,
            },
            ScriptTextBodyCommand {
                command: "done".into(),
                args: vec![],
                command_index: 1,
            },
        ],
    };
    assert_eq!(
        encode_radio_text_body(&body, &Default::default()),
        Err(RadioTextError::MissingRamSymbol("missing".into()))
    );
    body.commands[0].args.clear();
    assert!(matches!(
        encode_radio_text_body(&body, &Default::default()),
        Err(RadioTextError::Body { .. })
    ));
    body.commands[0].command = "done".into();
    assert!(matches!(
        encode_radio_text_body(&body, &Default::default()),
        Err(RadioTextError::Body { .. })
    ));
}

#[test]
fn radio_program_transitions_match_four_source_broadcasts() {
    use crystal_core::systems::{radio_program::*, radio_text::encode_radio_text_body};
    let bodies = exported_radio_bodies();
    let fixtures = [
        fixture(),
        serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-lucky.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-ben-sunday.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-fern-monday.json"
        ))
        .unwrap(),
    ];
    let mut checked = 0;
    for fixture in &fixtures {
        let symbols = serde_json::from_value(fixture["ram_symbols"].clone()).unwrap();
        for print in fixture["prints"].as_array().unwrap() {
            let before = fixture["loops"]
                .as_array()
                .unwrap()
                .iter()
                .rev()
                .find(|entry| entry["frame"].as_u64().unwrap() <= print["start"].as_u64().unwrap())
                .unwrap();
            let line = before["line"].as_u64().unwrap() as u8;
            let context = RadioProgramContext {
                buena: None,
                oak: None,
                printed: print["printed_before"].as_u64().unwrap() as u8,
                weekday: print["weekday"].as_u64().unwrap() as u8,
                people_places: None,
                caught_pokemon: None,
            };
            // Captured Lucky shows take the ordinary (nonzero) branch. Its
            // entire byte-domain branch condition is checked separately below.
            let step = radio_program_step(line, context, &mut |_| {
                Ok({
                    assert_eq!(line, 41);
                    1
                })
            })
            .unwrap();
            let text = step.text.unwrap();
            assert_eq!(
                u64::from(text.next_line),
                print["next_line"].as_u64().unwrap()
            );
            let encoded = encode_radio_text_body(
                &bodies[match text.source {
                    RadioProgramTextSource::Label(label) => label,
                    _ => panic!("expected named source text"),
                }],
                &symbols,
            )
            .unwrap();
            let source = bytes(&print["text_bytes"]);
            assert_eq!(
                encoded,
                &source[..encoded.len()],
                "source program line {line}"
            );
            if line == 36 {
                assert_eq!(step.effects, [RadioProgramEffect::FormatLuckyNumber]);
            }
            if line == 39 {
                assert!(
                    step.effects.is_empty(),
                    "repeat must reuse the formatted number"
                );
            }
            if line == 3 {
                assert_eq!(
                    step.effects.last(),
                    Some(&RadioProgramEffect::CheckAndResetLuckyNumberShow)
                );
            }
            if matches!(line, 2 | 6) {
                let expected_id = if context.weekday & 1 == 0 { 81 } else { 80 };
                assert_eq!(fixture["music"][0]["music_id"], expected_id);
                assert_eq!(
                    step.effects,
                    [
                        RadioProgramEffect::ClearTextbox,
                        RadioProgramEffect::RestartMusic(
                            radio_music_channel_song(context.weekday).unwrap()
                        )
                    ]
                );
                assert_eq!(
                    fixture["loops"].as_array().unwrap().last().unwrap()["line"],
                    28
                );
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 46);
}

#[test]
fn places_people_starts_the_source_program_instead_of_a_transcript_alias() {
    use crystal_core::systems::radio_program::*;
    let step = radio_program_step(
        5,
        RadioProgramContext {
            buena: None,
            oak: None,
            printed: 0,
            weekday: 0,
            people_places: None,
            caught_pokemon: None,
        },
        &mut |_| panic!("intro must not draw random bytes"),
    )
    .unwrap();
    assert_eq!(
        step.text,
        Some(RadioProgramText {
            source: RadioProgramTextSource::Label("_PnP_Text1"),
            next_line: 44
        })
    );
    assert_eq!(
        step.effects,
        [
            RadioProgramEffect::ClearTextbox,
            RadioProgramEffect::RestartMusic("MUSIC_VIRIDIAN_CITY")
        ]
    );
}

#[test]
fn radio_program_preserves_rare_lucky_branch_music_end_and_start_conditions() {
    use crystal_core::systems::radio_program::*;
    let mut context = RadioProgramContext {
        buena: None,
        oak: None,
        printed: 2,
        weekday: 0,
        people_places: None,
        caught_pokemon: None,
    };
    for random_byte in 0..=255u8 {
        let mut calls = 0;
        let step = radio_program_step(41, context, &mut |_| {
            Ok({
                calls += 1;
                random_byte
            })
        })
        .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(
            step.text.unwrap().next_line,
            if random_byte == 0 { 42 } else { 3 }
        );
    }
    let mut no_random = |_| panic!("source line makes no Random call");
    assert_eq!(
        radio_program_step(42, context, &mut no_random)
            .unwrap()
            .text
            .unwrap()
            .source,
        RadioProgramTextSource::Label("_LC_DragText1")
    );
    assert_eq!(
        radio_program_step(43, context, &mut no_random)
            .unwrap()
            .text
            .unwrap()
            .next_line,
        3
    );
    let end = radio_program_step(28, context, &mut no_random).unwrap();
    assert!(end.effects.is_empty() && end.text.is_none());
    assert!(
        radio_program_step(7, context, &mut no_random)
            .unwrap()
            .effects
            .is_empty()
    );
    for weekday in 0..7 {
        context.weekday = weekday;
        for line in [2, 6] {
            let step = radio_program_step(line, context, &mut no_random).unwrap();
            assert_eq!(
                step.effects,
                [
                    RadioProgramEffect::ClearTextbox,
                    RadioProgramEffect::RestartMusic(if weekday & 1 == 0 {
                        "MUSIC_POKEMON_MARCH"
                    } else {
                        "MUSIC_POKEMON_LULLABY"
                    })
                ]
            );
        }
    }
    for line in 8..=10 {
        context.printed = 0;
        let first = radio_program_step(line, context, &mut no_random).unwrap();
        assert_eq!(first.effects.len(), 3);
        assert_eq!(
            first.effects.last(),
            Some(&RadioProgramEffect::SetPrintedLines(1))
        );
        assert!(first.text.is_none());
        context.printed = 1;
        assert_eq!(
            radio_program_step(line, context, &mut no_random)
                .unwrap()
                .effects,
            [RadioProgramEffect::SetPrintedLines(1)]
        );
    }
    assert!(matches!(
        radio_program_step(88, context, &mut no_random),
        Err(RadioProgramError::UnimplementedLine(88))
    ));
}

#[test]
fn places_people_replays_all_source_random_draws_text_and_tiles() {
    use crystal_core::systems::{radio_program::*, radio_text::encode_radio_text_body};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let classes =
        std::fs::read_to_string(root.join("vendor/pokecrystal/constants/trainer_constants.asm"))
            .unwrap();
    let classes = classes
        .lines()
        .filter_map(|line| line.trim().strip_prefix("trainerclass "))
        .map(|line| line.split_whitespace().next().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(classes.len(), 68);
    assert_eq!(classes[0], "TRAINER_NONE");
    assert_eq!(classes[67], "MYSTICALMAN");
    let catalog: Value = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/story_events/StandardScripts.json"))
            .unwrap(),
    )
    .unwrap();
    let hidden = |label: &str| {
        catalog["StandardScripts"][label]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["args"][0].as_str().unwrap())
            .take_while(|name| *name != "-1")
            .map(|name| {
                classes
                    .iter()
                    .position(|candidate| *candidate == name)
                    .unwrap() as u8
            })
            .collect::<Vec<_>>()
    };
    let before = hidden("PnP_HiddenPeople");
    let e4 = hidden("PnP_HiddenPeople_BeatE4");
    let kanto = hidden("PnP_HiddenPeople_BeatKanto");
    let data = RadioPeoplePlacesContext {
        hall_of_fame: false,
        kanto_badges: 0,
        trainer_class_count: (classes.len() - 1) as u8,
        place_count: catalog["StandardScripts"]["PnP_Places"]
            .as_array()
            .unwrap()
            .len() as u8,
        hidden_people: &before,
        hidden_people_beat_e4: &e4,
        hidden_people_beat_kanto: &kanto,
    };
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tools/asm-oracle/fixtures/radio-program-places-people.json"
    ))
    .unwrap();
    let draws = fixture["random"].as_array().unwrap();
    let symbols = serde_json::from_value(fixture["ram_symbols"].clone()).unwrap();
    let bodies = exported_radio_bodies();
    let mut consumed = 0;
    assert_eq!(fixture["prints"].as_array().unwrap().len(), 18);
    for print in fixture["prints"].as_array().unwrap() {
        let entry = fixture["loops"]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .find(|entry| entry["frame"].as_u64().unwrap() <= print["start"].as_u64().unwrap())
            .unwrap();
        let line = entry["line"].as_u64().unwrap() as u8;
        let mut state = state(print["printed_before"].as_u64().unwrap() as u8);
        let step = radio_program_step(
            line,
            RadioProgramContext {
                buena: None,
                oak: None,
                printed: state.printed,
                weekday: 0,
                people_places: Some(&data),
                caught_pokemon: None,
            },
            &mut |carry| {
                Ok({
                    let draw = &draws[consumed];
                    assert_eq!(draw["carry_in"], carry);
                    consumed += 1;
                    assert_eq!(draw["line"].as_u64().unwrap(), u64::from(line));
                    assert!(draw["frame"].as_u64().unwrap() <= print["start"].as_u64().unwrap());
                    draw["value"].as_u64().unwrap() as u8
                })
            },
        )
        .unwrap();
        if line == 46 {
            assert_eq!(
                step.effects,
                [RadioProgramEffect::FormatPeoplePlacesTrainer {
                    class_id: ((draws[consumed - 1]["value"].as_u64().unwrap() as u8) & 127) + 1,
                    trainer_id: 1
                }]
            );
        } else if line == 48 {
            assert_eq!(
                step.effects,
                [RadioProgramEffect::FormatPeoplePlacesLandmark {
                    index: draws[consumed - 1]["value"].as_u64().unwrap() as u8
                }]
            );
        }
        let text = step.text.unwrap();
        assert_eq!(
            u64::from(text.next_line),
            print["next_line"].as_u64().unwrap()
        );
        let encoded = encode_radio_text_body(
            &bodies[match text.source {
                RadioProgramTextSource::Label(label) => label,
                _ => panic!("expected named source text"),
            }],
            &symbols,
        )
        .unwrap();
        assert_eq!(
            encoded,
            bytes(&print["text_bytes"])[..encoded.len()],
            "line {line}"
        );
        let mut window = window(&print["tiles_before"]);
        let mut printer = RadioLinePrinter::new(&encoded, &mut state, text.next_line).unwrap();
        assert!(
            printer
                .advance_frame(&mut window, &CapturedEnvironment(print), false)
                .unwrap()
        );
        assert_eq!(window, self::window(&print["tiles_after"]), "line {line}");
    }
    assert_eq!(consumed, 86);
    assert_eq!(consumed, draws.len());
}

#[test]
fn places_people_rare_branches_and_progression_exclusions_match_source() {
    use crystal_core::systems::radio_program::*;
    let mut data = RadioPeoplePlacesContext {
        hall_of_fame: false,
        kanto_badges: 0,
        trainer_class_count: 67,
        place_count: 9,
        hidden_people: &[1, 2, 3],
        hidden_people_beat_e4: &[2, 3],
        hidden_people_beat_kanto: &[3],
    };
    for (hof, badges, expected) in [(false, 0xff, 4), (true, 0xfe, 1), (true, 0xff, 1)] {
        data.hall_of_fame = hof;
        data.kanto_badges = badges;
        let mut bytes = [255, 66, 0, 1, 2, 3].into_iter();
        let step = radio_program_step(
            46,
            RadioProgramContext {
                buena: None,
                oak: None,
                printed: 2,
                weekday: 0,
                people_places: Some(&data),
                caught_pokemon: None,
            },
            &mut |_| Ok(bytes.next().unwrap()),
        )
        .unwrap();
        assert_eq!(
            step.effects,
            [RadioProgramEffect::FormatPeoplePlacesTrainer {
                class_id: expected,
                trainer_id: 1
            }]
        );
    }
    // Class 2 becomes eligible only after all Kanto badges and the Hall of Fame.
    for (hof, badges, expected) in [(false, 0xff, 4), (true, 0xfe, 4), (true, 0xff, 2)] {
        data.hall_of_fame = hof;
        data.kanto_badges = badges;
        let mut bytes = [1, 2, 3].into_iter();
        let step = radio_program_step(
            46,
            RadioProgramContext {
                buena: None,
                oak: None,
                printed: 2,
                weekday: 0,
                people_places: Some(&data),
                caught_pokemon: None,
            },
            &mut |_| Ok(bytes.next().unwrap()),
        )
        .unwrap();
        assert_eq!(
            step.effects,
            [RadioProgramEffect::FormatPeoplePlacesTrainer {
                class_id: expected,
                trainer_id: 1
            }]
        );
    }
    let context = RadioProgramContext {
        buena: None,
        oak: None,
        printed: 2,
        weekday: 0,
        people_places: Some(&data),
        caught_pokemon: None,
    };
    for byte in 0..=255u8 {
        let branch = radio_program_step(45, context, &mut |_| Ok(byte)).unwrap();
        assert_eq!(
            branch.text.unwrap().next_line,
            if byte < 123 { 46 } else { 48 }
        );
        for line in [47, 49] {
            let mut bytes = [255, byte, 123].into_iter();
            let step =
                radio_program_step(line, context, &mut |_| Ok(bytes.next().unwrap())).unwrap();
            let text = step.text.unwrap();
            assert_eq!(text.source, RadioProgramTextSource::Label("_PnP_OddText"));
            assert_eq!(text.next_line, if byte < 10 { 5 } else { 48 });
            assert_eq!(bytes.len(), if byte < 10 { 1 } else { 0 });
        }
    }
    let repeat = radio_program_step(5, context, &mut |_| panic!("repeat intro RNG")).unwrap();
    assert!(repeat.effects.is_empty());
}

#[test]
fn pokedex_show_selects_only_caught_species_with_source_rejection_sampling() {
    use crystal_core::systems::radio_program::*;
    let mut caught = [false; 251];
    caught[154] = true; // Cyndaquil: CheckCaughtMon uses a zero-based bit.
    let mut draws = [255, 251, 0, 153, 154].into_iter();
    let step = radio_program_step(
        1,
        RadioProgramContext {
            buena: None,
            oak: None,
            printed: 0,
            weekday: 0,
            people_places: None,
            caught_pokemon: Some(&caught),
        },
        &mut |_| Ok(draws.next().expect("unexpected extra source Random call")),
    )
    .unwrap();
    assert_eq!(draws.next(), None);
    assert_eq!(
        step.text,
        Some(RadioProgramText {
            source: RadioProgramTextSource::Label("_PokedexShowText"),
            next_line: 19
        })
    );
}

#[test]
fn pokedex_show_replays_source_selection_category_and_six_entry_lines() {
    use crystal_core::systems::{radio_program::*, radio_text::encode_radio_text_body};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tools/asm-oracle/fixtures/radio-program-pokedex.json"
    ))
    .unwrap();
    let mut caught = [false; 251];
    caught[fixture["caught_species"].as_u64().unwrap() as usize - 1] = true;
    let bodies = exported_radio_bodies();
    let symbols = serde_json::from_value(fixture["ram_symbols"].clone()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let entries: std::collections::BTreeMap<
        String,
        crystal_core::models::pokedex::RuntimePokedexEntry,
    > = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/pokedex_entries.json")).unwrap(),
    )
    .unwrap();
    let entry =
        crystal_core::systems::radio_text::encode_radio_pokedex_entry(&entries["CYNDAQUIL"])
            .unwrap();
    assert_eq!(
        entry,
        bytes(&fixture["pokedex_entry"]["bytes"])[..entry.len()]
    );
    let mut cursor = RadioPokedexEntryCursor::default();
    let draws = fixture["random"].as_array().unwrap();
    let mut consumed = 0;
    let prints = fixture["prints"].as_array().unwrap();
    for print in prints {
        let before = fixture["loops"]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .find(|record| record["frame"].as_u64().unwrap() <= print["start"].as_u64().unwrap())
            .unwrap();
        let line = before["line"].as_u64().unwrap() as u8;
        let printed = print["printed_before"].as_u64().unwrap() as u8;
        let step = radio_program_step(
            line,
            RadioProgramContext {
                buena: None,
                oak: None,
                printed,
                weekday: 0,
                people_places: None,
                caught_pokemon: Some(&caught),
            },
            &mut |carry| {
                Ok({
                    assert_eq!(draws[consumed]["carry_in"], carry);
                    assert_eq!(draws[consumed]["line"], line);
                    let value = draws[consumed]["value"].as_u64().unwrap() as u8;
                    consumed += 1;
                    value
                })
            },
        )
        .unwrap();
        if line == 1 {
            assert_eq!(
                step.effects.last(),
                Some(&RadioProgramEffect::FormatPokedexSpecies { species: 155 })
            );
            assert_eq!(print["species"], 155);
            assert_eq!(step.effects.len(), if printed == 0 { 3 } else { 1 });
            cursor = RadioPokedexEntryCursor::default();
        } else {
            assert!(step.effects.is_empty());
        }
        let text = step.text.unwrap();
        assert_eq!(print["next_line"], text.next_line);
        let encoded = match text.source {
            RadioProgramTextSource::Label(label) => {
                encode_radio_text_body(&bodies[label], &symbols).unwrap()
            }
            RadioProgramTextSource::RetainedBuffer => {
                panic!("Pokédex Show does not reuse a radio buffer")
            }
            RadioProgramTextSource::PokedexCategory => cursor.copy_text(&entry, true).unwrap(),
            RadioProgramTextSource::PokedexEntryLine => cursor.copy_text(&entry, false).unwrap(),
        };
        assert_eq!(
            encoded,
            bytes(&print["text_bytes"])[..encoded.len()],
            "line {line}"
        );
        let mut state = state(printed);
        let mut window = window(&print["tiles_before"]);
        let mut printer = RadioLinePrinter::new(&encoded, &mut state, text.next_line).unwrap();
        assert!(
            printer
                .advance_frame(&mut window, &CapturedEnvironment(print), false)
                .unwrap()
        );
        assert_eq!(window, self::window(&print["tiles_after"]), "line {line}");
    }
    assert_eq!(prints.len(), 18);
    assert_eq!(consumed, draws.len());
}

#[test]
fn pokedex_show_rejects_empty_caught_data_and_accepts_last_species() {
    use crystal_core::systems::radio_program::*;
    let mut caught = [false; 251];
    assert!(matches!(
        radio_program_step(
            1,
            RadioProgramContext {
                buena: None,
                oak: None,
                printed: 0,
                weekday: 0,
                people_places: None,
                caught_pokemon: Some(&caught)
            },
            &mut |_| panic!("no valid selection")
        ),
        Err(RadioProgramError::SelectionData(_))
    ));
    caught[250] = true;
    let step = radio_program_step(
        1,
        RadioProgramContext {
            buena: None,
            oak: None,
            printed: 2,
            weekday: 0,
            people_places: None,
            caught_pokemon: Some(&caught),
        },
        &mut |_| Ok(250),
    )
    .unwrap();
    assert_eq!(
        step.effects,
        [RadioProgramEffect::FormatPokedexSpecies { species: 251 }]
    );
}

#[test]
fn oak_radio_starts_its_source_program() {
    use crystal_core::systems::radio_program::*;
    let step = radio_program_step(
        0,
        RadioProgramContext {
            buena: None,
            oak: None,
            printed: 0,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        },
        &mut |_| panic!("Oak intro does not sample Random"),
    )
    .unwrap();
    assert_eq!(
        step.text,
        Some(RadioProgramText {
            source: RadioProgramTextSource::Label("_OPT_IntroText1"),
            next_line: 11,
        })
    );
}

#[test]
fn oak_radio_replays_source_cycle_and_bumper_delays() {
    use crystal_core::systems::{radio_program::*, radio_text::encode_radio_text_body};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tools/asm-oracle/fixtures/radio-program-oak-cycle.json"
    ))
    .unwrap();
    let bodies = exported_radio_bodies();
    let symbols = serde_json::from_value(fixture["ram_symbols"].clone()).unwrap();
    let loops = fixture["loops"].as_array().unwrap();
    let draws = fixture["random"].as_array().unwrap();
    let mut consumed = 0;
    for print in fixture["prints"].as_array().unwrap() {
        let before = loops
            .iter()
            .rev()
            .find(|record| record["frame"].as_u64().unwrap() <= print["start"].as_u64().unwrap())
            .unwrap();
        let line = before["line"].as_u64().unwrap() as u8;
        let printed = print["printed_before"].as_u64().unwrap() as u8;
        let oak = RadioOakContext {
            segment_counter: before["oak_segments"].as_u64().unwrap() as u8,
            delay: before["delay"].as_u64().unwrap() as u8,
            grass_routes: &[true; 15],
        };
        let step = radio_program_step(
            line,
            RadioProgramContext {
                buena: None,
                oak: Some(&oak),
                printed,
                weekday: 0,
                caught_pokemon: None,
                people_places: None,
            },
            &mut |carry| {
                Ok({
                    assert_eq!(draws[consumed]["carry_in"], carry);
                    assert_eq!(draws[consumed]["line"], line);
                    let value = draws[consumed]["value"].as_u64().unwrap() as u8;
                    consumed += 1;
                    value
                })
            },
        )
        .unwrap();
        let text = step.text.unwrap();
        assert_eq!(print["next_line"], text.next_line);
        let RadioProgramTextSource::Label(label) = text.source else {
            panic!("source fixture has every route")
        };
        let encoded = encode_radio_text_body(&bodies[label], &symbols).unwrap();
        assert_eq!(
            encoded,
            bytes(&print["text_bytes"])[..encoded.len()],
            "Oak line {line}"
        );
        let mut state = state(printed);
        let mut window = window(&print["tiles_before"]);
        let mut printer = RadioLinePrinter::new(&encoded, &mut state, text.next_line).unwrap();
        assert!(
            printer
                .advance_frame(&mut window, &CapturedEnvironment(print), false)
                .unwrap()
        );
        assert_eq!(
            window,
            self::window(&print["tiles_after"]),
            "Oak line {line}"
        );
    }
    assert_eq!(consumed, draws.len());
    assert_eq!(fixture["prints"].as_array().unwrap().len(), 37);
    let mut checked = 0;
    for records in loops.windows(2) {
        let line = records[0]["line"].as_u64().unwrap() as u8;
        if !(60..=63).contains(&line) {
            continue;
        }
        let oak = RadioOakContext {
            segment_counter: 5,
            delay: records[0]["delay"].as_u64().unwrap() as u8,
            grass_routes: &[true; 15],
        };
        let step = radio_program_step(
            line,
            RadioProgramContext {
                buena: None,
                oak: Some(&oak),
                printed: 2,
                weekday: 0,
                caught_pokemon: None,
                people_places: None,
            },
            &mut |_| panic!("bumper does not sample Random"),
        )
        .unwrap();
        assert!(step.text.is_none());
        let (mut next_line, mut delay) = (line, oak.delay);
        for effect in step.effects {
            match effect {
                RadioProgramEffect::SetCurrentLine(value) => next_line = value,
                RadioProgramEffect::SetRadioDelay(value) => delay = value,
                _ => {}
            }
        }
        assert_eq!(records[1]["line"], next_line);
        assert_eq!(records[1]["delay"], delay);
        checked += 1;
    }
    assert_eq!(checked, 400);
}

#[test]
fn oak_radio_samples_source_route_time_and_middle_slots() {
    use crystal_core::systems::radio_program::*;
    let oak = RadioOakContext {
        segment_counter: 5,
        delay: 0,
        grass_routes: &[true; 15],
    };
    let mut draws = [31, 15, 14, 3, 7, 2, 0, 1, 7, 4].into_iter();
    let step = radio_program_step(
        13,
        RadioProgramContext {
            buena: None,
            oak: Some(&oak),
            printed: 2,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        },
        &mut |_| Ok(draws.next().unwrap()),
    )
    .unwrap();
    assert_eq!(
        step.effects,
        [RadioProgramEffect::FormatOakEncounter {
            route_index: 14,
            time_of_day: 2,
            slot: 4
        }]
    );
    assert_eq!(draws.next(), None);
    let missing = RadioOakContext {
        grass_routes: &[false],
        ..oak
    };
    let mut calls = 0;
    let step = radio_program_step(
        13,
        RadioProgramContext {
            buena: None,
            oak: Some(&missing),
            printed: 2,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        },
        &mut |_| {
            Ok({
                calls += 1;
                0
            })
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert!(step.effects.is_empty());
    assert_eq!(
        step.text,
        Some(RadioProgramText {
            source: RadioProgramTextSource::RetainedBuffer,
            next_line: 0
        })
    );
}

#[test]
fn oak_radio_counter_and_bumper_delay_use_source_byte_wrapping() {
    use crystal_core::systems::radio_program::*;
    for value in 0..=255u8 {
        let oak = RadioOakContext {
            segment_counter: value,
            delay: value,
            grass_routes: &[true; 15],
        };
        let context = RadioProgramContext {
            buena: None,
            oak: Some(&oak),
            printed: 2,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        };
        let step = radio_program_step(18, context, &mut |_| Ok(0)).unwrap();
        assert_eq!(
            step.text.unwrap().next_line,
            if value == 1 { 59 } else { 13 }
        );
        assert_eq!(
            step.effects,
            [RadioProgramEffect::SetOakSegmentCounter(if value == 1 {
                5
            } else {
                value.wrapping_sub(1)
            })]
        );
        for line in 60..=63 {
            let step =
                radio_program_step(line, context, &mut |_| panic!("no random calls")).unwrap();
            assert_eq!(
                step.effects[0],
                RadioProgramEffect::SetRadioDelay(value.wrapping_sub(1))
            );
            assert_eq!(step.effects.len() == 1, value != 1);
        }
    }
}

#[test]
fn buena_radio_has_a_native_program_entry() {
    use crystal_core::systems::radio_program::*;
    let buena = RadioBuenaContext {
        hour: 18,
        password: 0,
        password_generated: false,
    };
    let result = radio_program_step(
        4,
        RadioProgramContext {
            buena: Some(&buena),
            oak: None,
            printed: 0,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        },
        &mut |_| panic!("entry must not generate a password"),
    )
    .unwrap();
    assert_eq!(
        result.text,
        Some(RadioProgramText {
            source: RadioProgramTextSource::Label("_BuenaRadioText1"),
            next_line: 64
        })
    );
}

#[test]
fn buena_radio_uses_source_hours_and_midnight_branches() {
    use crystal_core::systems::radio_program::*;
    for hour in 0..24 {
        let buena = RadioBuenaContext {
            hour,
            password: 0,
            password_generated: true,
        };
        for (line, printed, expected) in [
            (4, 0, if hour >= 18 { 64 } else { 83 }),
            (4, 2, if hour >= 18 { 64 } else { 71 }),
            (65, 2, if hour >= 18 { 66 } else { 70 }),
            (66, 2, if hour >= 18 { 67 } else { 71 }),
            (69, 2, if hour >= 18 { 4 } else { 70 }),
            (83, 2, if hour >= 18 { 64 } else { 83 }),
        ] {
            let step = radio_program_step(
                line,
                RadioProgramContext {
                    buena: Some(&buena),
                    oak: None,
                    printed,
                    weekday: 0,
                    caught_pokemon: None,
                    people_places: None,
                },
                &mut |_| panic!("saved password must be reused"),
            )
            .unwrap();
            assert_eq!(
                step.text.unwrap().next_line,
                expected,
                "hour {hour}, line {line}"
            );
            if hour < 18 && line != 83 {
                assert!(
                    step.effects
                        .contains(&RadioProgramEffect::SetBuenaPasswordGenerated(false))
                );
            }
            if line == 83 {
                assert_eq!(
                    &step.effects[..2],
                    &[
                        RadioProgramEffect::SetCurrentLine(4),
                        RadioProgramEffect::SetPrintedLines(0)
                    ]
                );
            }
        }
    }
}

#[test]
fn buena_radio_samples_once_and_reuses_all_source_password_indices() {
    use crystal_core::systems::radio_program::*;
    let buena = RadioBuenaContext {
        hour: 23,
        password: 255,
        password_generated: false,
    };
    let mut draws = [15, 11, 10, 3, 7, 2].into_iter();
    let step = radio_program_step(
        66,
        RadioProgramContext {
            buena: Some(&buena),
            oak: None,
            printed: 2,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        },
        &mut |_| Ok(draws.next().unwrap()),
    )
    .unwrap();
    assert_eq!(draws.next(), None);
    assert_eq!(
        step.effects,
        [
            RadioProgramEffect::StoreBuenaPassword(0xa2),
            RadioProgramEffect::SetBuenaPasswordGenerated(true),
            RadioProgramEffect::FormatBuenaPassword(0xa2)
        ]
    );
    for group in 0..11 {
        for word in 0..3 {
            let password = (group << 4) | word;
            let buena = RadioBuenaContext {
                password,
                password_generated: true,
                ..buena
            };
            let step = radio_program_step(
                66,
                RadioProgramContext {
                    buena: Some(&buena),
                    oak: None,
                    printed: 2,
                    weekday: 0,
                    caught_pokemon: None,
                    people_places: None,
                },
                &mut |_| panic!("daily password was already generated"),
            )
            .unwrap();
            assert_eq!(
                step.effects,
                [RadioProgramEffect::FormatBuenaPassword(password)]
            );
        }
    }
}

#[test]
fn buena_radio_shutdown_preserves_source_text_order_and_resets() {
    use crystal_core::systems::radio_program::*;
    let buena = RadioBuenaContext {
        hour: 0,
        password: 0,
        password_generated: true,
    };
    let bodies = exported_radio_bodies();
    for line in 70..=83 {
        let step = radio_program_step(
            line,
            RadioProgramContext {
                buena: Some(&buena),
                oak: None,
                printed: 2,
                weekday: 0,
                caught_pokemon: None,
                people_places: None,
            },
            &mut |_| panic!("shutdown never generates a password"),
        )
        .unwrap();
        let text = step.text.unwrap();
        let expected = match line {
            70 | 80 | 81 => "_BuenaRadioMidnightText10".to_string(),
            71..=79 => format!("_BuenaRadioMidnightText{}", line - 70),
            _ => "_BuenaOffTheAirText".to_string(),
        };
        let RadioProgramTextSource::Label(label) = text.source else {
            panic!("source shutdown text")
        };
        assert_eq!(label, expected);
        assert!(bodies.contains_key(label));
        assert_eq!(text.next_line, if line == 83 { 83 } else { line + 1 });
        if line == 82 {
            assert_eq!(
                step.effects,
                [
                    RadioProgramEffect::NoRadioMusic,
                    RadioProgramEffect::NoRadioName,
                    RadioProgramEffect::SetBuenaPasswordGenerated(false),
                    RadioProgramEffect::SetCurrentLine(4),
                    RadioProgramEffect::SetPrintedLines(0)
                ]
            );
        }
    }
}

#[test]
fn buena_radio_replays_source_night_and_off_air_prints() {
    use crystal_core::systems::{radio_program::*, radio_text::encode_radio_text_body};
    let bodies = exported_radio_bodies();
    let mut checked = 0;
    for source in [
        include_str!("../../../../tools/asm-oracle/fixtures/radio-program-buena-night.json"),
        include_str!("../../../../tools/asm-oracle/fixtures/radio-program-buena-day.json"),
    ] {
        let fixture: Value = serde_json::from_str(source).unwrap();
        let symbols = serde_json::from_value(fixture["ram_symbols"].clone()).unwrap();
        let loops = fixture["loops"].as_array().unwrap();
        let draws = fixture["random"].as_array().unwrap();
        let mut consumed = 0;
        for print in fixture["prints"].as_array().unwrap() {
            let before = loops
                .iter()
                .rev()
                .find(|record| {
                    record["frame"].as_u64().unwrap() <= print["start"].as_u64().unwrap()
                })
                .unwrap();
            let line = before["line"].as_u64().unwrap() as u8;
            let buena = RadioBuenaContext {
                hour: print["hour"].as_u64().unwrap() as u8,
                password: before["buena_password"].as_u64().unwrap() as u8,
                password_generated: before["buena_generated"].as_bool().unwrap(),
            };
            let step = radio_program_step(
                line,
                RadioProgramContext {
                    buena: Some(&buena),
                    oak: None,
                    printed: before["printed"].as_u64().unwrap() as u8,
                    weekday: 0,
                    caught_pokemon: None,
                    people_places: None,
                },
                &mut |carry| {
                    Ok({
                        assert_eq!(draws[consumed]["carry_in"], carry);
                        assert_eq!(draws[consumed]["line"], line);
                        let value = draws[consumed]["value"].as_u64().unwrap() as u8;
                        consumed += 1;
                        value
                    })
                },
            )
            .unwrap();
            let text = step.text.unwrap();
            assert_eq!(print["next_line"], text.next_line);
            let RadioProgramTextSource::Label(label) = text.source else {
                panic!("Buena uses named source text")
            };
            let encoded = encode_radio_text_body(&bodies[label], &symbols).unwrap();
            assert_eq!(
                encoded,
                bytes(&print["text_bytes"])[..encoded.len()],
                "Buena line {line}"
            );
            let mut state = state(before["printed"].as_u64().unwrap() as u8);
            for effect in step.effects {
                if let RadioProgramEffect::SetPrintedLines(value) = effect {
                    state.printed = value;
                }
            }
            assert_eq!(print["printed_before"], state.printed);
            let mut window = window(&print["tiles_before"]);
            let mut printer = RadioLinePrinter::new(&encoded, &mut state, text.next_line).unwrap();
            assert!(
                printer
                    .advance_frame(&mut window, &CapturedEnvironment(print), false)
                    .unwrap()
            );
            assert_eq!(
                window,
                self::window(&print["tiles_after"]),
                "Buena line {line}"
            );
            checked += 1;
        }
        assert_eq!(consumed, draws.len());
    }
    assert_eq!(checked, 23);
}

#[test]
fn radio_random_retry_preserves_source_carry_input() {
    use crystal_core::systems::radio_program::*;
    let oak = RadioOakContext {
        segment_counter: 5,
        delay: 0,
        grass_routes: &[true; 15],
    };
    let mut draws = [0, 0, 0, 1, 2].into_iter();
    let mut carries = Vec::new();
    radio_program_step(
        13,
        RadioProgramContext {
            buena: None,
            oak: Some(&oak),
            printed: 2,
            weekday: 0,
            caught_pokemon: None,
            people_places: None,
        },
        &mut |carry| {
            Ok({
                carries.push(carry);
                draws.next().unwrap()
            })
        },
    )
    .unwrap();
    assert_eq!(
        carries,
        [false, false, false, true, true],
        "cp 2 leaves carry set on rejected slots 0 and 1"
    );
    let people = RadioPeoplePlacesContext {
        hall_of_fame: false,
        kanto_badges: 0,
        trainer_class_count: 67,
        place_count: 9,
        hidden_people: &[1],
        hidden_people_beat_e4: &[],
        hidden_people_beat_kanto: &[],
    };
    let mut draws = [0, 1].into_iter();
    carries.clear();
    radio_program_step(
        46,
        RadioProgramContext {
            buena: None,
            oak: None,
            printed: 2,
            weekday: 0,
            caught_pokemon: None,
            people_places: Some(&people),
        },
        &mut |carry| {
            Ok({
                carries.push(carry);
                draws.next().unwrap()
            })
        },
    )
    .unwrap();
    assert_eq!(
        carries,
        [false, true],
        "IsInArray returns carry on an excluded trainer"
    );
}

#[test]
fn radio_playback_runs_rocket_program_printer_and_scroll_as_one_owner() {
    use crystal_core::systems::{radio_playback::*, radio_program::*};
    struct Host {
        texts: std::collections::VecDeque<Vec<u8>>,
        bodies:
            std::collections::BTreeMap<String, crystal_core::systems::script_text::ScriptTextBody>,
        symbols: std::collections::BTreeMap<String, u16>,
        effects: Vec<RadioProgramEffect>,
    }
    impl RadioTextEnvironment for Host {
        fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
            Err(RadioTextError::Ram(address))
        }
        fn weekday(&self) -> u8 {
            0
        }
    }
    impl RadioPlaybackHost for Host {
        fn effect(
            &mut self,
            effect: &RadioProgramEffect,
            window: &mut RadioTextWindow,
        ) -> Result<bool, RadioPlaybackError> {
            self.effects.push(effect.clone());
            match effect {
                RadioProgramEffect::ClearTextbox => {
                    for row in &mut window.tiles[1..5] {
                        row[1..19].fill(0x7f);
                    }
                }
                RadioProgramEffect::RestartMusic("MUSIC_ROCKET_OVERTURE") => {}
                _ => panic!("unexpected Rocket effect {effect:?}"),
            }
            Ok(true)
        }
        fn text(&mut self, source: &RadioProgramTextSource) -> Result<Vec<u8>, RadioPlaybackError> {
            let RadioProgramTextSource::Label(label) = source else {
                panic!("Rocket uses named text")
            };
            let encoded = crystal_core::systems::radio_text::encode_radio_text_body(
                &self.bodies[*label],
                &self.symbols,
            )?;
            let captured = self.texts.pop_front().unwrap();
            assert_eq!(encoded, captured[..encoded.len()]);
            Ok(encoded)
        }
    }
    let fixture = fixture();
    let prints = fixture["prints"].as_array().unwrap();
    let mut host = Host {
        texts: prints.iter().map(|p| bytes(&p["text_bytes"])).collect(),
        effects: Vec::new(),
        bodies: exported_radio_bodies(),
        symbols: serde_json::from_value(fixture["ram_symbols"].clone()).unwrap(),
    };
    let mut playback = RadioPlayback::new(0, window(&prints[0]["tiles_before"]));
    let context = RadioProgramContext {
        buena: None,
        oak: None,
        printed: 0,
        weekday: 0,
        caught_pokemon: None,
        people_places: None,
    };
    for (index, print) in prints.iter().enumerate() {
        playback
            .advance_frame(
                context,
                true,
                true,
                false,
                &mut |_| panic!("Rocket has no RNG"),
                &mut host,
            )
            .unwrap();
        let mut resumed = 0;
        while playback.printing() {
            playback
                .advance_frame(
                    context,
                    true,
                    true,
                    false,
                    &mut |_| unreachable!(),
                    &mut host,
                )
                .unwrap();
            resumed += 1;
            assert!(resumed <= 60);
        }
        assert_eq!(
            playback.window,
            window(&print["tiles_after"]),
            "print {index}"
        );
        assert_eq!(
            playback.state.printed,
            print["printed_after"].as_u64().unwrap() as u8
        );
        assert_eq!(
            playback.state.next_line,
            print["next_line"].as_u64().unwrap() as u8
        );
        assert_eq!(
            (playback.state.current_line, playback.state.delay),
            (84, 100)
        );
        for _ in 0..100 {
            playback
                .advance_frame(
                    context,
                    true,
                    true,
                    false,
                    &mut |_| unreachable!(),
                    &mut host,
                )
                .unwrap();
            assert_eq!(playback.state.current_line, 84);
        }
        playback
            .advance_frame(
                context,
                true,
                true,
                false,
                &mut |_| unreachable!(),
                &mut host,
            )
            .unwrap();
        assert_eq!(playback.state.current_line, playback.state.next_line);
    }
    assert!(host.texts.is_empty());
    assert_eq!(
        host.effects.len(),
        2,
        "station restart must not replay the opening music/clear"
    );
}

#[test]
fn radio_playback_suspends_oak_bumper_effects_until_bg_map_completes() {
    use crystal_core::systems::{radio_playback::*, radio_program::*};
    struct Host {
        ready: bool,
        calls: Vec<RadioProgramEffect>,
    }
    impl RadioTextEnvironment for Host {
        fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
            Err(RadioTextError::Ram(address))
        }
        fn weekday(&self) -> u8 {
            0
        }
    }
    impl RadioPlaybackHost for Host {
        fn effect(
            &mut self,
            effect: &RadioProgramEffect,
            _: &mut RadioTextWindow,
        ) -> Result<bool, RadioPlaybackError> {
            if *effect == RadioProgramEffect::WaitBgMap && !self.ready {
                return Ok(false);
            }
            self.calls.push(effect.clone());
            Ok(true)
        }
        fn text(&mut self, _: &RadioProgramTextSource) -> Result<Vec<u8>, RadioPlaybackError> {
            panic!("bumper uses PrintTextboxText rather than PrintRadioLine")
        }
    }
    let oak = RadioOakContext {
        segment_counter: 5,
        delay: 77,
        grass_routes: &[true; 15],
    };
    let context = RadioProgramContext {
        buena: None,
        oak: Some(&oak),
        printed: 0,
        weekday: 0,
        caught_pokemon: None,
        people_places: None,
    };
    let mut playback = RadioPlayback::new(
        59,
        RadioTextWindow {
            tiles: [[0x7f; 20]; 6],
        },
    );
    let mut host = Host {
        ready: false,
        calls: Vec::new(),
    };
    for _ in 0..3 {
        playback
            .advance_frame(
                context,
                true,
                true,
                false,
                &mut |_| unreachable!(),
                &mut host,
            )
            .unwrap();
        assert_eq!(
            host.calls,
            [
                RadioProgramEffect::RestartPokemonChannelMusic,
                RadioProgramEffect::ClearTextbox
            ]
        );
        assert_eq!(playback.state.current_line, 59);
        assert_eq!(playback.state.delay, 0);
    }
    host.ready = true;
    playback
        .advance_frame(
            context,
            true,
            true,
            false,
            &mut |_| unreachable!(),
            &mut host,
        )
        .unwrap();
    assert_eq!(
        host.calls,
        [
            RadioProgramEffect::RestartPokemonChannelMusic,
            RadioProgramEffect::ClearTextbox,
            RadioProgramEffect::WaitBgMap,
            RadioProgramEffect::PrintTextboxLabel("_OPT_PokemonChannelText")
        ]
    );
    assert_eq!(
        (playback.state.current_line, playback.state.delay),
        (60, 100)
    );
    playback
        .advance_frame(
            context,
            true,
            true,
            false,
            &mut |_| unreachable!(),
            &mut host,
        )
        .unwrap();
    assert_eq!(
        (playback.state.current_line, playback.state.delay),
        (60, 99),
        "use owned delay, not stale external context"
    );
}

#[test]
fn radio_random_exhaustion_stops_rejection_sampling_without_a_fabricated_byte() {
    use crystal_core::{
        random::{CrystalRandom, CrystalRandomState, ReplayDivider},
        systems::radio_program::*,
    };
    let mut caught = [false; 251];
    caught[154] = true;
    let context = RadioProgramContext {
        buena: None,
        oak: None,
        printed: 0,
        weekday: 0,
        caught_pokemon: Some(&caught),
        people_places: None,
    };
    let mut rng = CrystalRandom::new(CrystalRandomState::default(), ReplayDivider::new([0, 0]));
    let result = radio_program_step(1, context, &mut |carry| {
        rng.random(carry)
            .map(|output| output.value)
            .map_err(|error| RadioProgramError::Random(error.to_string()))
    });
    assert_eq!(
        result,
        Err(RadioProgramError::Random(
            "divider replay exhausted after 2 samples".into()
        ))
    );
    assert_eq!(rng.random_calls(), 1);
}

#[test]
fn radio_pokedex_catalog_encoding_matches_all_source_entry_bytes() {
    use crystal_core::{
        models::pokedex::RuntimePokedexEntry,
        systems::radio_text::{encode_radio_pokedex_entry, encode_radio_string},
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let entries: std::collections::BTreeMap<String, RuntimePokedexEntry> = serde_json::from_slice(
        &std::fs::read(root.join("apps/web/assets/data/pokedex_entries.json")).unwrap(),
    )
    .unwrap();
    let mut checked = 0;
    for (species, entry) in entries {
        let file = species.to_lowercase();
        let source = std::fs::read_to_string(root.join(format!(
            "vendor/pokecrystal/data/pokemon/dex_entries/{file}.asm"
        )))
        .unwrap();
        let mut expected = Vec::new();
        for line in source
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with(';'))
        {
            let (command, operands) = line.split_once(char::is_whitespace).unwrap();
            if command == "dw" {
                for word in operands.split(';').next().unwrap().split(',') {
                    expected.extend(word.trim().parse::<u16>().unwrap().to_le_bytes());
                }
            } else {
                match command {
                    "db" => {}
                    "next" => expected.push(0x4e),
                    "page" => expected.push(0x50),
                    _ => panic!("unexpected source macro {command}"),
                }
                let text = operands.split('"').nth(1).unwrap();
                expected.extend(encode_radio_string(text).unwrap());
            }
        }
        assert_eq!(
            encode_radio_pokedex_entry(&entry).unwrap(),
            expected,
            "{species}"
        );
        checked += 1;
    }
    assert_eq!(checked, 251);
}
