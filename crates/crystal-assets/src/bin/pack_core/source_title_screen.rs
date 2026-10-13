//! Lower authored presentation routines into the production presentation IR.
//! Every lowering is tied to an exact source sequence and line span.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};
#[path = "source_title_fragment.rs"]
mod source_title_fragment;

pub(super) fn export(root: &Path) -> Result<()> {
    super::source_title_catalog::export(root, source_title_fragment::assemble(root)?)
}

pub(super) fn export_unvalidated(root: &Path) -> Result<()> {
    let title = super::source_title_catalog::assemble_unvalidated(root, source_title_fragment::assemble(root)?)?;
    super::source_title_catalog::export_timing_graphics(root, &title.program)?;
    let output = root.join("apps/web/assets/data/runtime_title_screen.unvalidated.json");
    std::fs::write(&output, serde_json::to_vec_pretty(&title)?)?;
    println!("wrote authored unvalidated title catalog {}", output.display());
    Ok(())
}

#[path = "source_intro_oracle.rs"]
mod source_intro_oracle;
#[path = "source_intro_audio_timing.rs"]
mod source_intro_audio_timing;
#[path = "source_intro_audio_commands.rs"]
mod source_intro_audio_commands;
#[path = "source_intro_vblank_timing.rs"]
mod source_intro_vblank_timing;
#[path = "source_intro_audio_certification.rs"]
mod source_intro_audio_certification;
#[path = "source_intro_audio_clock_certification.rs"]
mod source_intro_audio_clock_certification;

pub(super) fn certify_intro_audio(vendor: &Path, model: &Value, trace: &Value) -> Result<Value> {
    let mut certificate=source_intro_audio_certification::certify(vendor, model, trace)?;
    certificate["runtime_source_clock"]=source_intro_audio_clock_certification::certify(vendor,model,trace)?;
    Ok(certificate)
}

pub(super) fn certify_intro_control(phase: &Value, trace: &Value) -> Result<usize> {
    source_title_intro_measurements::verify_scene_control(phase, trace)
}
#[path = "source_title_boot.rs"]
mod source_title_boot;
#[path = "source_title_intro.rs"]
mod source_title_intro;
#[path = "source_title_intro_clock.rs"]
mod source_title_intro_clock;
#[path = "source_title_intro_measurements.rs"]
mod source_title_intro_measurements;
#[path = "source_title_intro_scenes.rs"]
mod source_title_intro_scenes;

pub(super) fn intro_clock(vendor: &Path, trace: &Value) -> Result<Value> {
    let mut clock=source_title_intro_clock::lower(vendor,trace)?;
    clock["vblank_normal"]["audio_update"]=source_intro_audio_timing::export(vendor)?;
    clock["vblank_normal"]["cost_models"]=source_intro_vblank_timing::export(vendor)?;
    Ok(clock)
}
pub(super) fn intro_trace(vendor: &Path) -> Result<Value> {
    source_intro_oracle::trace(vendor)
}
pub(super) fn intro_setup_phases(vendor:&Path,constants:&BTreeMap<String,i64>)->Result<Value> {source_title_intro_scenes::setup_phases(vendor,constants)}
pub(super) fn intro_transfer_metadata(vendor:&Path)->Result<Value> {source_title_intro_scenes::transfer_metadata(vendor)}
pub(super) fn intro_bind_measurements(vendor:&Path,phase:&mut Value,metadata:&Value,trace:&Value)->Result<Value> {source_title_intro_measurements::attach(vendor,phase,metadata,trace)}
#[path = "source_title_setup.rs"]
mod source_title_setup;
#[path = "source_title_sprites.rs"]
mod source_title_sprites;

pub(super) fn intro_entry_cleanup(
    vendor: &Path,
    constants: &BTreeMap<String, i64>,
) -> Result<Value> {
    source_title_intro::entry_cleanup(vendor, constants)
}

pub(super) fn intro_sprites(vendor: &Path) -> Result<Value> {
    source_title_sprites::export(vendor)
}

pub(super) fn startup_blocks(
    vendor: &Path,
    constants: &BTreeMap<String, i64>,
) -> Result<BTreeMap<String, Value>> {
    source_title_boot::blocks(vendor, constants)
}

pub(super) fn startup_text(vendor: &Path) -> Result<Vec<Value>> {
    source_title_boot::texts(vendor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_normalization_preserves_authored_string_spacing() {
        assert_eq!(
            normalize_spacing("\ttext  \"Hello!  Sorry to\""),
            "text \"Hello!  Sorry to\""
        );
        assert_eq!(
            normalize_spacing("\tcp  PAD_UP + PAD_B + PAD_SELECT"),
            "cp PAD_UP + PAD_B + PAD_SELECT"
        );
        assert_eq!(
            normalize_spacing(r#"text "say \"  hi\"""#),
            r#"text "say \"  hi\"""#
        );
    }

    #[test]
    fn ordered_lowering_rejects_unhandled_source_instructions() {
        let source = Source {
            file: "authored.asm".into(),
            lines: vec![
                (1, "Entry:".into()),
                (2, "call Unexpected".into()),
                (3, "ret".into()),
            ],
        };
        let mut cursor = Cursor::new(&source, "Entry").unwrap();
        assert!(cursor.take(&["ret"], "return", json!({})).is_err());
        assert!(cursor.finish().is_err());
    }
}

struct Source {
    file: String,
    lines: Vec<(u32, String)>,
}

fn normalize_spacing(code: &str) -> String {
    let mut result = String::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut space = false;
    for ch in code.chars() {
        if !quoted && ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !result.is_empty() {
            result.push(' ');
        }
        space = false;
        result.push(ch);
        if escaped {
            escaped = false;
        } else if quoted && ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            quoted = !quoted;
        }
    }
    result
}

impl Source {
    fn read(vendor: &Path, file: &str) -> Result<Self> {
        let raw = std::fs::read_to_string(vendor.join(file))?;
        let mut lines = Vec::new();
        let mut debug = false;
        for (index, raw) in raw.lines().enumerate() {
            let code = super::source_syntax::code_part(raw);
            if code == "if DEF(_DEBUG)" {
                ensure!(!debug, "nested debug conditional");
                debug = true;
                continue;
            }
            if debug && code == "endc" {
                debug = false;
                continue;
            }
            if debug || code.is_empty() {
                continue;
            }
            lines.push((u32::try_from(index + 1)?, normalize_spacing(code)));
        }
        ensure!(!debug, "unterminated debug source conditional");
        Ok(Self {
            file: file.into(),
            lines,
        })
    }

    fn body(&self, label: &str) -> Result<&[(u32, String)]> {
        let start = self
            .lines
            .iter()
            .position(|(_, line)| line.trim_end_matches(':') == label)
            .with_context(|| format!("{} missing source entry {label}", self.file))?;
        let end = self
            .lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(_, (_, line))| line.ends_with(':') && !line.starts_with('.'))
            .map_or(self.lines.len(), |(index, _)| index);
        Ok(&self.lines[start..end])
    }

    fn span(&self, label: &str) -> Result<Value> {
        let body = self.body(label)?;
        Ok(
            json!({"file":self.file,"start_line":body.first().unwrap().0,"end_line":body.last().unwrap().0}),
        )
    }

    fn operation(&self, label: &str, sequence: &[&str], op: &str, fields: Value) -> Result<Value> {
        ensure!(
            !sequence.is_empty(),
            "empty presentation source certificate"
        );
        let body = self.body(label)?;
        let windows = body
            .windows(sequence.len())
            .filter(|window| {
                window
                    .iter()
                    .zip(sequence)
                    .all(|((_, actual), expected)| actual == expected)
            })
            .collect::<Vec<_>>();
        ensure!(
            windows.len() == 1,
            "{}:{label} requires unique source sequence {sequence:?}; found {}",
            self.file,
            windows.len()
        );
        let window = windows[0];
        let mut fields = fields.as_object().context("presentation fields")?.clone();
        fields.insert("op".into(), json!(op));
        fields.insert("source_span".into(), json!({"file":self.file,"start_line":window.first().unwrap().0,"end_line":window.last().unwrap().0}));
        Ok(Value::Object(fields))
    }
}

pub(super) fn main_menu_entry(vendor: &Path) -> Result<Value> {
    let source = Source::read(vendor, "engine/menus/intro_menu.asm")?;
    let operations = vec![
        source.operation(
            "Intro_MainMenu",
            &["ld de, MUSIC_NONE", "call PlayMusic"],
            "stop_audio",
            json!({"audio":"MUSIC_NONE"}),
        )?,
        source.operation(
            "Intro_MainMenu",
            &["call DelayFrame"],
            "wait_frames",
            json!({"frames":1}),
        )?,
        source.operation(
            "Intro_MainMenu",
            &["ld de, MUSIC_MAIN_MENU", "ld a, e", "ld [wMapMusic], a"],
            "write_memory_byte",
            json!({"target":"wMapMusic","value":"MUSIC_MAIN_MENU"}),
        )?,
        source.operation(
            "Intro_MainMenu",
            &["ld [wMapMusic], a", "call PlayMusic"],
            "play_audio",
            json!({"audio":"MUSIC_MAIN_MENU"}),
        )?,
        source.operation(
            "Intro_MainMenu",
            &["farcall MainMenu"],
            "call_subprogram",
            json!({"program":"main_menu"}),
        )?,
        source.operation(
            "Intro_MainMenu",
            &["jp StartTitleScreen"],
            "jump",
            json!({"target":"StartTitleScreen"}),
        )?,
    ];
    Ok(json!({"source_span":source.span("Intro_MainMenu")?,"operations":operations}))
}

/// Certify successive source ranges, including repeated portrait/fade calls.
/// A changed or newly inserted instruction must be lowered explicitly.
struct Cursor<'a> {
    source: &'a Source,
    body: &'a [(u32, String)],
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(source: &'a Source, entry: &str) -> Result<Self> {
        Ok(Self {
            source,
            body: source.body(entry)?,
            index: 1,
        })
    }

    fn take(&mut self, sequence: &[&str], op: &str, fields: Value) -> Result<Value> {
        let end = self.index + sequence.len();
        let range = self
            .body
            .get(self.index..end)
            .context("presentation source sequence runs past routine")?;
        ensure!(
            !range.is_empty()
                && range
                    .iter()
                    .zip(sequence)
                    .all(|((_, actual), expected)| actual == expected),
            "{}:{} unsupported presentation source; expected {sequence:?}",
            self.source.file,
            range.first().map_or(0, |line| line.0)
        );
        let mut fields = fields
            .as_object()
            .context("presentation operation fields")?
            .clone();
        fields.insert("op".into(), json!(op));
        fields.insert("source_span".into(), json!({"file":self.source.file,"start_line":range.first().unwrap().0,"end_line":range.last().unwrap().0}));
        self.index = end;
        Ok(Value::Object(fields))
    }

    fn finish(self) -> Result<()> {
        ensure!(
            self.index == self.body.len(),
            "{} has unlowered presentation instructions at line {}",
            self.source.file,
            self.body[self.index.min(self.body.len() - 1)].0
        );
        Ok(())
    }
}

pub(super) fn oak_speech(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let source = Source::read(vendor, "engine/menus/intro_menu.asm")?;
    let fade = Source::read(vendor, "home/fade.asm")?;
    let eval = |expression: &str| super::source_expressions::evaluate(expression, constants);
    let (clock, _) = source_title_setup::clock(vendor)?;
    let naming = source_title_setup::naming(vendor, constants)?;
    let fade_words = source
        .body("IntroFadePalettes")?
        .iter()
        .filter(|(_, line)| line.starts_with("dc "))
        .count();
    ensure!(fade_words > 0, "empty authored intro fade palette table");
    source.operation(
        "Intro_RotatePalettesLeftFrontpic",
        &[
            "ld hl, IntroFadePalettes",
            "ld b, IntroFadePalettes.End - IntroFadePalettes",
        ],
        "source_fade_table",
        json!({}),
    )?;
    source.operation(
        "Intro_RotatePalettesLeftFrontpic",
        &[
            "ld c, 10",
            "call DelayFrames",
            "dec b",
            "jr nz, .loop",
            "ret",
        ],
        "source_fade_delay",
        json!({}),
    )?;
    fade.operation(
        "RotateThreePalettesRight",
        &[
            "ld hl, IncGradGBPalTable_05",
            "ld b, 3",
            "jr RotatePalettesRight",
        ],
        "source_fade_words",
        json!({}),
    )?;
    fade.operation(
        "RotatePalettesRight",
        &[
            "ld c, 8",
            "call DelayFrames",
            "pop de",
            "dec b",
            "jr nz, RotatePalettesRight",
            "ret",
        ],
        "source_fade_delay",
        json!({}),
    )?;
    source.operation(
        "Intro_WipeInFrontpic",
        &[
            "ld a, $77",
            "ldh [hWX], a",
            "call DelayFrame",
            "ld a, %11100100",
            "call DmgToCgbBGPals",
        ],
        "source_wipe_start",
        json!({}),
    )?;
    source.operation(
        "Intro_WipeInFrontpic",
        &[
            ".loop",
            "call DelayFrame",
            "ldh a, [hWX]",
            "sub $8",
            "cp -1",
            "ret z",
            "ldh [hWX], a",
            "jr .loop",
        ],
        "source_wipe_step",
        json!({}),
    )?;
    source.operation(
        "OakText2",
        &[
            "text_far _OakText2",
            "text_asm",
            "ld a, WOOPER",
            "call PlayMonCry",
            "call WaitSFX",
            "ld hl, OakText3",
            "ret",
        ],
        "source_text_continuation",
        json!({}),
    )?;
    source.operation(
        "Intro_PrepTrainerPic",
        &[
            "xor a",
            "ldh [hGraphicStartTile], a",
            "hlcoord 6, 4",
            "lb bc, 7, 7",
            "predef PlaceGraphic",
            "ret",
        ],
        "source_portrait_geometry",
        json!({}),
    )?;
    let portrait_fade = json!({"words":fade_words,"frames_per_word":eval("10")?});
    let dismissal_fade = json!({"words":eval("3")?,"frames_per_word":eval("8")?});
    let mut cursor = Cursor::new(&source, "OakSpeech")?;
    let mut operations = Vec::new();
    macro_rules! take {
        ($sequence:expr,$op:expr,$fields:expr) => {
            operations.push(cursor.take($sequence, $op, $fields)?);
        };
    }
    take!(&["farcall InitClock"], "initialize_clock", clock);
    take!(
        &["call RotateFourPalettesLeft", "call ClearTilemap"],
        "prepare_intro_display",
        json!({"routine":"RotateFourPalettesLeft","clear_tilemap":true})
    );
    take!(
        &["ld de, MUSIC_ROUTE_30", "call PlayMusic"],
        "play_audio",
        json!({"audio":"MUSIC_ROUTE_30"})
    );
    take!(
        &[
            "call RotateFourPalettesRight",
            "call RotateThreePalettesRight"
        ],
        "prepare_intro_palettes",
        json!({"routines":["RotateFourPalettesRight","RotateThreePalettesRight"]})
    );
    let trainer_picture = [
        "xor a",
        "ld [wCurPartySpecies], a",
        "ld a, POKEMON_PROF",
        "ld [wTrainerClass], a",
        "call Intro_PrepTrainerPic",
        "ld b, SCGB_TRAINER_OR_MON_FRONTPIC_PALS",
        "call GetSGBLayout",
        "call Intro_RotatePalettesLeftFrontpic",
    ];
    take!(
        &trainer_picture,
        "present_intro_portrait",
        json!({"kind":"trainer","trainer_class":"POKEMON_PROF","coordinates":{"x":6,"y":4},"fade":portrait_fade})
    );
    take!(
        &["ld hl, OakText1", "call PrintText"],
        "present_text_sequence",
        json!({"entries":["OakText1"]})
    );
    take!(
        &["call RotateThreePalettesRight", "call ClearTilemap"],
        "dismiss_intro_portrait",
        json!({"fade":dismissal_fade,"clear_tilemap":true})
    );
    take!(
        &[
            "ld a, WOOPER",
            "ld [wCurSpecies], a",
            "ld [wCurPartySpecies], a",
            "call GetBaseData",
            "hlcoord 6, 4",
            "call PrepMonFrontpic",
            "xor a",
            "ld [wTempMonDVs], a",
            "ld [wTempMonDVs + 1], a",
            "ld b, SCGB_TRAINER_OR_MON_FRONTPIC_PALS",
            "call GetSGBLayout",
            "call Intro_WipeInFrontpic"
        ],
        "present_intro_portrait",
        json!({"kind":"pokemon","species":"WOOPER","coordinates":{"x":6,"y":4},"dvs":[0,0],"wipe":{"window_x_start":eval("$77")?,"delta":-eval("$8")?,"stop_before":eval("-1")?,"initial_wait_frames":1}})
    );
    take!(
        &[
            "ld hl, OakText2",
            "call PrintText",
            "ld hl, OakText4",
            "call PrintText"
        ],
        "present_text_sequence",
        json!({"entries":["OakText2","OakText3","OakText4"],"embedded_audio":[{"after":"OakText2","cry":"WOOPER","wait_for_sfx":true}]})
    );
    take!(
        &["call RotateThreePalettesRight", "call ClearTilemap"],
        "dismiss_intro_portrait",
        json!({"fade":dismissal_fade,"clear_tilemap":true})
    );
    take!(
        &trainer_picture,
        "present_intro_portrait",
        json!({"kind":"trainer","trainer_class":"POKEMON_PROF","coordinates":{"x":6,"y":4},"fade":portrait_fade})
    );
    take!(
        &["ld hl, OakText5", "call PrintText"],
        "present_text_sequence",
        json!({"entries":["OakText5"]})
    );
    take!(
        &["call RotateThreePalettesRight", "call ClearTilemap"],
        "dismiss_intro_portrait",
        json!({"fade":dismissal_fade,"clear_tilemap":true})
    );
    take!(
        &[
            "xor a",
            "ld [wCurPartySpecies], a",
            "farcall DrawIntroPlayerPic",
            "ld b, SCGB_TRAINER_OR_MON_FRONTPIC_PALS",
            "call GetSGBLayout",
            "call Intro_RotatePalettesLeftFrontpic"
        ],
        "present_intro_portrait",
        json!({"kind":"player","fade":portrait_fade})
    );
    take!(
        &["ld hl, OakText6", "call PrintText"],
        "present_text_sequence",
        json!({"entries":["OakText6"]})
    );
    take!(&["call NamePlayer"], "name_player", naming);
    take!(
        &["ld hl, OakText7", "call PrintText"],
        "present_text_sequence",
        json!({"entries":["OakText7"]})
    );
    take!(&["ret"], "return", json!({}));
    cursor.finish()?;
    Ok(
        json!({"id":"oak_speech","source_entry":"OakSpeech","accepted_call_forms":["call"],"result":{},"phases":[{"id":"speech","source_span":source.span("OakSpeech")?,"operations":operations}],"loop":{},"resource_transfers":[],"tilemap_writes":[],"resources":[],"audio":[],"sprite_operations":[],"sprite_programs":[],"required_consumer":{},"source_span":source.span("OakSpeech")?,"implementation_source_spans":[source.span("Intro_RotatePalettesLeftFrontpic")?,source.span("Intro_WipeInFrontpic")?,fade.span("RotateThreePalettesRight")?,fade.span("RotatePalettesRight")?]}),
    )
}

pub(super) fn oak_text(vendor: &Path) -> Result<Vec<Value>> {
    let menu = Source::read(vendor, "engine/menus/intro_menu.asm")?;
    let common2 = Source::read(vendor, "data/text/common_2.asm")?;
    let common3 = Source::read(vendor, "data/text/common_3.asm")?;
    let mut texts = Vec::new();
    for index in 1..=7 {
        let entry = format!("OakText{index}");
        let target = format!("_{entry}");
        menu.operation(
            &entry,
            &[&format!("text_far {target}")],
            "source_text_alias",
            json!({}),
        )?;
        let source = if index <= 5 { &common2 } else { &common3 };
        let body = source.body(&target)?;
        let mut commands = Vec::new();
        let mut end_line = None;
        for (line, code) in &body[1..] {
            let (command, operands) = code.split_once(' ').unwrap_or((code, ""));
            ensure!(
                matches!(
                    command,
                    "text"
                        | "line"
                        | "cont"
                        | "para"
                        | "prompt"
                        | "done"
                        | "text_end"
                        | "text_promptbutton"
                ),
                "unsupported Oak text command {code}"
            );
            let args = super::source_syntax::source_arguments(operands)?;
            let expected = usize::from(matches!(command, "text" | "line" | "cont" | "para"));
            ensure!(args.len() == expected, "malformed Oak text command {code}");
            commands.push(json!({"command":command,"args":args,"source_span":{"file":source.file,"start_line":line,"end_line":line}}));
            if matches!(command, "prompt" | "done" | "text_end") {
                end_line = Some(*line);
                break;
            }
        }
        let end_line = end_line.context("Oak text has no authored terminator")?;
        texts.push(json!({"id":entry,"source_span":{"file":source.file,"start_line":body[0].0,"end_line":end_line},"commands":commands}));
    }
    texts.extend(source_title_setup::clock(vendor)?.1);
    Ok(texts)
}

pub(super) fn main_menu(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let source = Source::read(vendor, "engine/menus/main_menu.asm")?;
    let mut constants = constants.clone();
    let mut next = 0;
    for (_, line) in &source.lines {
        if line == "const_def" {
            next = 0;
        } else if let Some(name) = line.strip_prefix("const ") {
            constants.insert(name.into(), next);
            next += 1;
        }
    }
    let body = source.body("MainMenu")?;
    let table = |label: &str| -> Result<Vec<String>> {
        let start = body
            .iter()
            .position(|(_, line)| line.trim_end_matches(':') == label)
            .context("source menu table")?;
        Ok(body[start + 1..]
            .iter()
            .take_while(|(_, line)| !line.starts_with('.'))
            .map(|(_, line)| line.clone())
            .collect())
    };
    let strings = table(".Strings")?
        .iter()
        .map(|line| {
            let value = line
                .strip_prefix("db \"")
                .and_then(|value| value.strip_suffix("@\""))
                .context("authored menu string")?;
            Ok(value.to_owned())
        })
        .collect::<Result<Vec<_>>>()?;
    let entries = table(".Jumptable")?
        .iter()
        .map(|line| {
            line.strip_prefix("dw ")
                .map(str::to_owned)
                .context("authored menu dispatch")
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        strings.len() == entries.len(),
        "source menu strings and dispatch disagree"
    );
    let rows = source
        .body("MainMenuItems")?
        .iter()
        .filter_map(|(_, line)| line.strip_prefix("db "))
        .map(|value| {
            super::source_expressions::evaluate(&value.replace("DEF(_DEBUG)", "0"), &constants)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut item_sets = Vec::new();
    let mut cursor = 0;
    while cursor < rows.len() {
        let count = usize::try_from(rows[cursor])?;
        cursor += 1;
        ensure!(
            cursor + count < rows.len(),
            "incomplete authored main menu item set"
        );
        let items = &rows[cursor..cursor + count];
        ensure!(
            items
                .iter()
                .all(|value| *value >= 0 && (*value as usize) < strings.len()),
            "authored main menu item index outside strings"
        );
        item_sets.push(items.to_vec());
        cursor += count;
        ensure!(rows[cursor] == -1, "main menu item set lacks sentinel");
        cursor += 1;
    }
    let header = table(".MenuHeader")?;
    let coordinates = header
        .iter()
        .find_map(|line| line.strip_prefix("menu_coords "))
        .context("source menu coordinates")?;
    let coords = super::source_syntax::source_arguments(coordinates)?
        .iter()
        .map(|value| super::source_expressions::evaluate(value, &constants))
        .collect::<Result<Vec<_>>>()?;
    ensure!(coords.len() == 4, "source menu coordinate arity");
    let default_option = header
        .last()
        .and_then(|line| line.strip_prefix("db "))
        .context("source menu default option")?;
    let default_option = super::source_expressions::evaluate(default_option, &constants)?;
    source.operation(
        "ClearTilemapEtc",
        &[
            "xor a",
            "ldh [hMapAnims], a",
            "call ClearTilemap",
            "call LoadFontsExtra",
            "call LoadStandardFont",
            "call ClearWindowData",
            "ret",
        ],
        "source_prepare",
        json!({}),
    )?;
    source.operation(
        "MainMenuJoypadLoop",
        &[
            "ld a, [w2DMenuFlags1]",
            "set _2DMENU_WRAP_UP_DOWN_F, a",
            "ld [w2DMenuFlags1], a",
        ],
        "source_wrap",
        json!({}),
    )?;
    source.operation(
        "MainMenuJoypadLoop",
        &[".loop", "call MainMenu_PrintCurrentTimeAndDay"],
        "source_poll_refresh",
        json!({}),
    )?;
    let mut operations = vec![
        source.operation("MainMenu", &["xor a", "ld [wDisableTextAcceleration], a"], "write_memory_byte", json!({"target":"wDisableTextAcceleration","value":0}))?,
        source.operation("MainMenu", &["call ClearTilemapEtc"], "prepare_main_menu_display", json!({"map_animations":0,"clear_tilemap":true,"clear_window_data":true}))?,
        source.operation("MainMenu", &["ld b, SCGB_DIPLOMA", "call GetSGBLayout"], "apply_palette_layout", json!({"layout":{"symbol":"SCGB_DIPLOMA","value":constants.get("SCGB_DIPLOMA").context("source diploma palette constant")?}}))?,
        source.operation("MainMenu", &["call SetDefaultBGPAndOBP"], "set_default_palettes", json!({"routine":"SetDefaultBGPAndOBP"}))?,
        source.operation("MainMenu", &["ld hl, wGameTimerPaused","res GAME_TIMER_COUNTING_F, [hl]"], "clear_memory_bit", json!({"target":"wGameTimerPaused","bit":"GAME_TIMER_COUNTING_F"}))?,
        source.operation("MainMenu", &["call MainMenu_GetWhichMenu","ld [wWhichIndexSet], a"], "select_main_menu_variant", json!({"result":"wWhichIndexSet","variants":[{"id":"new_game","value":constants.get("MAINMENU_NEW_GAME").context("new game variant")?},{"id":"continue","value":constants.get("MAINMENU_CONTINUE").context("continue variant")?},{"id":"mystery","value":constants.get("MAINMENU_MYSTERY").context("mystery variant")?}]}))?,
    ];
    source.operation(
        "MainMenu_PrintCurrentTimeAndDay",
        &[
            "MainMenu_PrintCurrentTimeAndDay:",
            "ld a, [wSaveFileExists]",
            "and a",
            "ret z",
        ],
        "source_condition",
        json!({}),
    )?;
    operations.extend([
        source.operation("MainMenu", &["call MainMenu_PrintCurrentTimeAndDay"], "render_main_menu_time_and_day", json!({"only_when":{"source":"wSaveFileExists","predicate":"nonzero"}}))?,
        source.operation("MainMenu", &["ld hl, .MenuHeader","call LoadMenuHeader"], "load_menu", json!({"strings":strings,"item_sets":item_sets,"coordinates":{"left":coords[0],"top":coords[1],"right":coords[2],"bottom":coords[3]},"default_option":default_option}))?,
        source.operation("MainMenuJoypadLoop", &["call GetScrollingMenuJoypad","ld a, [wMenuJoypad]","cp PAD_B","jr z, .b_button","cp PAD_A","jr z, .a_button"], "menu_input_loop", json!({"sampler":"GetScrollingMenuJoypad","result":"wMenuJoypad","accept":"PAD_A","cancel":"PAD_B","wrap_vertical":true,"refresh_before_each_poll":"MainMenu_PrintCurrentTimeAndDay"}))?,
        source.operation("MainMenu", &["call CloseWindow"], "close_window", json!({"routine":"CloseWindow"}))?,
        source.operation("MainMenu", &["jr c, .quit"], "branch_result", json!({"result":"main_menu_input","equals":"cancelled","target":".quit@MainMenu"}))?,
        source.operation("MainMenu", &["call ClearTilemap"], "clear_tilemap", json!({}))?,
        source.operation("MainMenu", &["ld a, [wMenuSelection]","ld hl, .Jumptable","rst JumpTable"], "dispatch_table", json!({"dispatcher":"MainMenu selection","index":"wMenuSelection","entries":entries}))?,
        source.operation("MainMenu", &["jr .loop"], "jump", json!({"target":".loop@MainMenu"}))?,
        source.operation("MainMenu", &[".quit","ret"], "return", json!({}))?,
    ]);
    Ok(
        json!({"id":"main_menu","source_entry":"MainMenu","accepted_call_forms":["farcall"],"result":{},"phases":[{"id":"main_menu","source_span":source.span("MainMenu")?,"labels":{".loop@MainMenu":0,".quit@MainMenu":operations.len()-1},"operations":operations}],"loop":{},"resource_transfers":[],"tilemap_writes":[],"resources":[],"audio":[],"sprite_operations":[],"sprite_programs":[],"required_consumer":{},"source_span":source.span("MainMenu")?,"implementation_source_spans":[]}),
    )
}

pub(super) fn gender(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let source = Source::read(vendor, "engine/menus/init_gender.asm")?;
    let body = source.body("InitGender")?;
    let items = body
        .iter()
        .filter_map(|(_, line)| {
            line.strip_prefix("db \"")
                .and_then(|value| value.strip_suffix("@\""))
        })
        .collect::<Vec<_>>();
    ensure!(items.len() == 2, "authored gender menu item count");
    let coordinates = body
        .iter()
        .find_map(|(_, line)| line.strip_prefix("menu_coords "))
        .context("gender menu coordinates")?;
    let coords = super::source_syntax::source_arguments(coordinates)?
        .iter()
        .map(|value| super::source_expressions::evaluate(value, constants))
        .collect::<Result<Vec<_>>>()?;
    ensure!(coords.len() == 4, "gender menu coordinate arity");
    let header = body
        .iter()
        .position(|(_, line)| line == ".MenuHeader:")
        .context("gender menu header")?;
    let default_option = body[header..]
        .iter()
        .take_while(|(_, line)| line != ".MenuData:")
        .filter_map(|(_, line)| line.strip_prefix("db "))
        .last()
        .context("gender menu default")?;
    let default_option = super::source_expressions::evaluate(default_option, constants)?;
    let delay = body
        .iter()
        .find_map(|(_, line)| line.strip_prefix("ld c, "))
        .context("gender confirmation delay")?;
    let delay = super::source_expressions::evaluate(delay, constants)?;
    let domain = items
        .iter()
        .enumerate()
        .map(|(index, label)| json!({"cursor":index+1,"label":label,"value":index}))
        .collect::<Vec<_>>();
    let operations = vec![
        source.operation("InitGender", &["ld hl, .MenuHeader","call LoadMenuHeader"],"load_menu",json!({"items":items,"coordinates":{"left":coords[0],"top":coords[1],"right":coords[2],"bottom":coords[3]},"default_option":default_option}))?,
        source.operation("InitGender", &["ld a, [wMenuCursorY]","dec a","ld [wPlayerGender], a"],"select_player_gender",json!({"target":"wPlayerGender","domain":domain}))?,
        source.operation("InitGender", &[&format!("ld c, {delay}"),"call DelayFrames"],"wait_frames",json!({"frames":delay}))?,
    ];
    Ok(
        json!({"id":"player_profile_setup","source_entry":"PlayerProfileSetup","accepted_call_forms":["call"],"result":{},"phases":[{"id":"gender_selection","source_span":source.span("InitGender")?,"operations":operations}],"loop":{},"resource_transfers":[],"tilemap_writes":[],"resources":[],"audio":[],"sprite_operations":[],"sprite_programs":[],"required_consumer":{},"source_span":source.span("InitGender")?,"implementation_source_spans":[]}),
    )
}

pub(super) fn title(vendor: &Path, constants: &BTreeMap<String, i64>) -> Result<Value> {
    let menu = Source::read(vendor, "engine/menus/intro_menu.asm")?;
    let movie = Source::read(vendor, "engine/movie/title.asm")?;
    let mut constants = constants.clone();
    let mut next = None;
    for (_, line) in &menu.lines {
        if line == "const_def" {
            next = Some(0_i64);
        } else if let Some(symbol) = line.strip_prefix("const ") {
            let value = next.context("title option without const_def")?;
            ensure!(
                symbol.starts_with("TITLESCREENOPTION_"),
                "unexpected title option constant {symbol}"
            );
            constants.insert(symbol.into(), value);
            next = Some(value + 1);
        } else if line == "DEF NUM_TITLESCREENOPTIONS EQU const_value" {
            constants.insert(
                "NUM_TITLESCREENOPTIONS".into(),
                next.context("title option count without const_def")?,
            );
            next = None;
        }
    }
    let eval = |expression: &str| super::source_expressions::evaluate(expression, &constants);
    let options = menu
        .body("StartTitleScreen")?
        .iter()
        .filter_map(|(_, line)| line.strip_prefix("dw "))
        .collect::<Vec<_>>();
    ensure!(
        i64::try_from(options.len())? == eval("NUM_TITLESCREENOPTIONS")?,
        "title option dispatch count disagrees with authored enum"
    );
    for (symbol, target) in [
        ("TITLESCREENOPTION_MAIN_MENU", "Intro_MainMenu"),
        ("TITLESCREENOPTION_DELETE_SAVE_DATA", "DeleteSaveData"),
        ("TITLESCREENOPTION_RESTART", "IntroSequence"),
        ("TITLESCREENOPTION_UNUSED", "IntroSequence"),
        ("TITLESCREENOPTION_RESET_CLOCK", "ResetClock"),
    ] {
        let index = usize::try_from(eval(symbol)?)?;
        ensure!(
            options.get(index) == Some(&target),
            "title option {symbol} does not dispatch to {target}"
        );
    }
    let mut operations = Vec::new();
    let mut labels = BTreeMap::<String, usize>::new();
    macro_rules! label {
        ($name:expr) => {
            labels.insert($name.to_owned(), operations.len());
        };
    }
    macro_rules! m {
        ($root:expr,$sequence:expr,$op:expr,$fields:expr) => {
            operations.push(menu.operation($root, $sequence, $op, $fields)?);
        };
    }
    macro_rules! v {
        ($root:expr,$sequence:expr,$op:expr,$fields:expr) => {
            operations.push(movie.operation($root, $sequence, $op, $fields)?);
        };
    }
    v!(
        "_TitleScreen",
        &["ld a, +112", "ldh [hSCX], a"],
        "write_memory_byte",
        json!({"target":"hSCX","value":eval("+112")?})
    );
    v!(
        "InitializeBackground",
        &[
            "ld hl, wShadowOAMSprite00",
            "ld d, -$22",
            "ld e, $0",
            "ld c, 5"
        ],
        "initialize_title_crystal_oam",
        json!({"target":"wShadowOAMSprite00","initial_y":eval("-$22")?,"columns":5})
    );
    label!("TitleScreenScene");
    let scenes = menu
        .body("TitleScreenScene")?
        .iter()
        .filter_map(|(_, line)| line.strip_prefix("dw "))
        .collect::<Vec<_>>();
    ensure!(scenes.len() == 4, "authored title scene count");
    m!(
        "TitleScreenScene",
        &["ld hl, .scenes", "add hl, de", "add hl, de"],
        "dispatch_table",
        json!({"dispatcher":"TitleScreenScene","entries":scenes,"index":"wJumptableIndex"})
    );
    label!("TitleScreenEntrance");
    m!(
        "TitleScreenEntrance",
        &["ldh a, [hSCX]", "and a", "jr z, .done"],
        "branch_memory_compare",
        json!({"source":"hSCX","predicate":"equal","operand":0,"target":".done@TitleScreenEntrance"})
    );
    m!(
        "TitleScreenEntrance",
        &["sub 4", "ldh [hSCX], a"],
        "subtract_memory_byte",
        json!({"target":"hSCX","delta":4})
    );
    m!(
        "TitleScreenEntrance",
        &["ld hl, wLYOverrides", "ld bc, 8 * 10", "call ByteFill"],
        "fill_memory",
        json!({"target":"wLYOverrides","length":eval("8 * 10")?,"value_from":"hSCX"})
    );
    v!(
        "AnimateTitleCrystal",
        &[
            "ld hl, wShadowOAMSprite00YCoord",
            "ld a, [hl]",
            "cp 6 + 2 * TILE_WIDTH",
            "ret z"
        ],
        "animate_title_crystal",
        json!({"target":"wShadowOAMSprite00YCoord","stop_at":eval("6 + 2 * TILE_WIDTH")?,"y_delta":2,"parts":30})
    );
    movie.operation(
        "AnimateTitleCrystal",
        &["ld c, 30", ".loop", "ld a, [hl]", "add 2"],
        "source_animation_delta",
        json!({}),
    )?;
    m!(
        "TitleScreenEntrance",
        &["farcall AnimateTitleCrystal", "ret"],
        "return",
        json!({})
    );
    label!(".done@TitleScreenEntrance");
    m!(
        "TitleScreenEntrance",
        &[".done", "ld hl, wJumptableIndex", "inc [hl]"],
        "increment_memory_byte",
        json!({"target":"wJumptableIndex","delta":1})
    );
    m!(
        "TitleScreenEntrance",
        &["xor a", "ldh [hLCDCPointer], a"],
        "write_memory_byte",
        json!({"target":"hLCDCPointer","value":0})
    );
    m!(
        "TitleScreenEntrance",
        &["ld de, MUSIC_TITLE", "call PlayMusic"],
        "play_audio",
        json!({"audio":"MUSIC_TITLE"})
    );
    m!(
        "TitleScreenEntrance",
        &["ld a, $88", "ldh [hWY], a"],
        "write_memory_byte",
        json!({"target":"hWY","value":eval("$88")?})
    );
    m!(
        "TitleScreenEntrance",
        &["ldh [hWY], a", "ret"],
        "return",
        json!({})
    );
    label!("TitleScreenTimer");
    m!(
        "TitleScreenTimer",
        &["ld hl, wJumptableIndex", "inc [hl]"],
        "increment_memory_byte",
        json!({"target":"wJumptableIndex","delta":1})
    );
    m!(
        "TitleScreenTimer",
        &[
            "ld hl, wTitleScreenTimer",
            "ld de, 73 * 60 + 36",
            "ld [hl], e",
            "inc hl",
            "ld [hl], d"
        ],
        "write_memory_word",
        json!({"target":"wTitleScreenTimer","value":eval("73 * 60 + 36")?})
    );
    m!(
        "TitleScreenTimer",
        &["ld [hl], d", "ret"],
        "return",
        json!({})
    );
    label!("TitleScreenMain");
    m!(
        "TitleScreenMain",
        &[
            "ld a, e",
            "or d",
            "jr z, .end",
            "dec de",
            "ld [hl], d",
            "dec hl",
            "ld [hl], e"
        ],
        "decrement_memory_word_unless_zero",
        json!({"target":"wTitleScreenTimer","zero_target":".end@TitleScreenMain"})
    );
    m!(
        "TitleScreenMain",
        &["call GetJoypad", "ld hl, hJoyDown"],
        "sample_input",
        json!({"result":"hJoyDown"})
    );
    for (sequence, expression, predicate, target) in [
        (
            vec![
                "and PAD_UP + PAD_B + PAD_SELECT",
                "cp PAD_UP + PAD_B + PAD_SELECT",
                "jr z, .delete_save_data",
            ],
            "PAD_UP + PAD_B + PAD_SELECT",
            "masked_equals",
            ".delete_save_data@TitleScreenMain",
        ),
        (
            vec![
                "and PAD_DOWN + PAD_B + PAD_SELECT",
                "cp PAD_DOWN + PAD_B + PAD_SELECT",
                "jr nz, .check_start",
            ],
            "PAD_DOWN + PAD_B + PAD_SELECT",
            "masked_not_equal",
            ".check_start@TitleScreenMain",
        ),
    ] {
        if target == ".check_start@TitleScreenMain" {
            m!(
                "TitleScreenMain",
                &[
                    "ldh a, [hClockResetTrigger]",
                    "cp $34",
                    "jr z, .check_clock_reset"
                ],
                "branch_memory_compare",
                json!({"source":"hClockResetTrigger","predicate":"equal","operand":eval("$34")?,"target":".check_clock_reset@TitleScreenMain"})
            );
        }
        m!(
            "TitleScreenMain",
            &sequence,
            "input_chord_branch",
            json!({"sample":"hJoyDown","mask":eval(expression)?,"operand":eval(expression)?,"predicate":predicate,"target":target})
        );
    }
    m!(
        "TitleScreenMain",
        &["ld a, $34", "ldh [hClockResetTrigger], a"],
        "write_memory_byte",
        json!({"target":"hClockResetTrigger","value":eval("$34")?})
    );
    m!(
        "TitleScreenMain",
        &["ldh [hClockResetTrigger], a", "jr .check_start"],
        "jump",
        json!({"target":".check_start@TitleScreenMain"})
    );
    label!(".check_clock_reset@TitleScreenMain");
    m!(
        "TitleScreenMain",
        &["bit B_PAD_SELECT, [hl]", "jr nz, .check_start"],
        "input_bit_branch",
        json!({"sample":"hJoyDown","bit":eval("B_PAD_SELECT")?,"predicate":"set","target":".check_start@TitleScreenMain"})
    );
    m!(
        "TitleScreenMain",
        &["xor a", "ldh [hClockResetTrigger], a"],
        "write_memory_byte",
        json!({"target":"hClockResetTrigger","value":0})
    );
    m!(
        "TitleScreenMain",
        &[
            "and PAD_LEFT + PAD_UP",
            "cp PAD_LEFT + PAD_UP",
            "jr z, .reset_clock"
        ],
        "input_chord_branch",
        json!({"sample":"hJoyDown","mask":eval("PAD_LEFT + PAD_UP")?,"operand":eval("PAD_LEFT + PAD_UP")?,"predicate":"masked_equals","target":".reset_clock@TitleScreenMain"})
    );
    label!(".check_start@TitleScreenMain");
    m!(
        "TitleScreenMain",
        &["and PAD_START | PAD_A", "jr nz, .incave"],
        "input_chord_branch",
        json!({"sample":"hJoyDown","mask":eval("PAD_START | PAD_A")?,"predicate":"masked_nonzero","target":".incave@TitleScreenMain"})
    );
    m!(
        "TitleScreenMain",
        &["jr nz, .incave", "ret"],
        "return",
        json!({})
    );
    for (local, symbol) in [
        (".incave", "TITLESCREENOPTION_MAIN_MENU"),
        (".delete_save_data", "TITLESCREENOPTION_DELETE_SAVE_DATA"),
        (".reset_clock", "TITLESCREENOPTION_RESET_CLOCK"),
    ] {
        label!(&format!("{local}@TitleScreenMain"));
        m!(
            "TitleScreenMain",
            &[local, &format!("ld a, {symbol}")],
            "write_memory_byte",
            json!({"target":"wTitleScreenSelectedOption","value":eval(symbol)?})
        );
        if local != ".reset_clock" {
            if local == ".incave" {
                m!(
                    "TitleScreenMain",
                    &["ld a, TITLESCREENOPTION_MAIN_MENU", "jr .done"],
                    "jump",
                    json!({"target":".done@TitleScreenMain"})
                );
            } else {
                label!(".done@TitleScreenMain");
                m!(
                    "TitleScreenMain",
                    &[
                        ".done",
                        "ld [wTitleScreenSelectedOption], a",
                        "ld hl, wJumptableIndex",
                        "set JUMPTABLE_EXIT_F, [hl]",
                        "ret"
                    ],
                    "set_memory_bit",
                    json!({"target":"wJumptableIndex","bit":eval("JUMPTABLE_EXIT_F")?})
                );
                m!(
                    "TitleScreenMain",
                    &[
                        ".done",
                        "ld [wTitleScreenSelectedOption], a",
                        "ld hl, wJumptableIndex",
                        "set JUMPTABLE_EXIT_F, [hl]",
                        "ret"
                    ],
                    "return",
                    json!({})
                );
            }
        } else {
            let sequence = [
                ".reset_clock",
                "ld a, TITLESCREENOPTION_RESET_CLOCK",
                "ld [wTitleScreenSelectedOption], a",
                "ld hl, wJumptableIndex",
                "set JUMPTABLE_EXIT_F, [hl]",
                "ret",
            ];
            m!(
                "TitleScreenMain",
                &sequence,
                "set_memory_bit",
                json!({"target":"wJumptableIndex","bit":eval("JUMPTABLE_EXIT_F")?})
            );
            m!("TitleScreenMain", &sequence, "return", json!({}));
        }
    }
    label!(".end@TitleScreenMain");
    m!(
        "TitleScreenMain",
        &[".end", "ld hl, wJumptableIndex", "inc [hl]"],
        "increment_memory_byte",
        json!({"target":"wJumptableIndex","delta":1})
    );
    m!(
        "TitleScreenMain",
        &[
            "xor a",
            "ld [wMusicFadeID], a",
            "ld [wMusicFadeID + 1], a",
            "ld hl, wMusicFade",
            "ld [hl], 8"
        ],
        "fade_audio",
        json!({"audio":"MUSIC_NONE","frames":64,"fade_register":{"target":"wMusicFade","value":8}})
    );
    m!(
        "TitleScreenMain",
        &["ld hl, wTitleScreenTimer", "inc [hl]", "ret"],
        "increment_memory_byte",
        json!({"target":"wTitleScreenTimer","delta":1})
    );
    m!(
        "TitleScreenMain",
        &["ld hl, wTitleScreenTimer", "inc [hl]", "ret"],
        "return",
        json!({})
    );
    label!("TitleScreenEnd");
    m!(
        "TitleScreenEnd",
        &["ld hl, wTitleScreenTimer", "inc [hl]"],
        "increment_memory_byte",
        json!({"target":"wTitleScreenTimer","delta":1})
    );
    m!(
        "TitleScreenEnd",
        &["ld a, [wMusicFade]", "and a", "ret nz"],
        "return_if_memory_nonzero",
        json!({"source":"wMusicFade"})
    );
    m!(
        "TitleScreenEnd",
        &[
            "ld a, TITLESCREENOPTION_RESTART",
            "ld [wTitleScreenSelectedOption], a"
        ],
        "write_memory_byte",
        json!({"target":"wTitleScreenSelectedOption","value":eval("TITLESCREENOPTION_RESTART")?})
    );
    m!(
        "TitleScreenEnd",
        &[
            "ld hl, wJumptableIndex",
            "set JUMPTABLE_EXIT_F, [hl]",
            "ret"
        ],
        "set_memory_bit",
        json!({"target":"wJumptableIndex","bit":eval("JUMPTABLE_EXIT_F")?})
    );
    m!(
        "TitleScreenEnd",
        &["set JUMPTABLE_EXIT_F, [hl]", "ret"],
        "return",
        json!({})
    );
    label!("SuicuneFrameIterator");
    v!(
        "SuicuneFrameIterator",
        &["ld hl, wSuicuneFrame", "ld a, [hl]", "ld c, a", "inc [hl]"],
        "postincrement_memory_byte",
        json!({"target":"wSuicuneFrame","result":"title_suicune_frame","delta":1})
    );
    v!(
        "SuicuneFrameIterator",
        &["and %111", "ret nz"],
        "return_unless_masked_zero",
        json!({"value":"title_suicune_frame","mask":eval("%111")?})
    );
    let frames = movie
        .body("SuicuneFrameIterator")?
        .iter()
        .filter_map(|(_, line)| line.strip_prefix("db "))
        .map(eval)
        .collect::<Result<Vec<_>>>()?;
    v!(
        "SuicuneFrameIterator",
        &["ld a, c", "and %11000", "sla a", "swap a"],
        "draw_indexed_title_suicune_frame",
        json!({"frames":frames,"selector":{"mask":eval("%11000")?,"shift_left":1,"swap_nibbles":true},"result":"title_suicune_frame"})
    );
    v!(
        "SuicuneFrameIterator",
        &["ldh [hBGMapThird], a", "ret"],
        "return",
        json!({})
    );
    label!("TitleTeardown");
    let tilemap = Source::read(vendor, "home/tilemap.asm")?;
    tilemap.operation(
        "WaitBGMap",
        &["ld c, 4", "call DelayFrames", "ret"],
        "source_frame_wait",
        json!({}),
    )?;
    tilemap.operation(
        "WaitBGMap2",
        &[
            "ldh a, [hCGB]",
            "and a",
            "jr z, .bg0",
            "ld a, 2",
            "ldh [hBGMapMode], a",
            "ld c, 4",
            "call DelayFrames",
        ],
        "source_cgb_frame_wait",
        json!({}),
    )?;
    tilemap.operation(
        "WaitBGMap2",
        &[
            ".bg0",
            "ld a, 1",
            "ldh [hBGMapMode], a",
            "ld c, 4",
            "call DelayFrames",
            "ret",
        ],
        "source_frame_wait",
        json!({}),
    )?;
    m!(
        "StartTitleScreen",
        &["call ClearSprites", "call ClearBGPalettes"],
        "fill_memory",
        json!({"target":"wShadowOAM","value":0,"length":eval("NUM_SPRITE_OAM_STRUCTS * OBJ_SIZE")?})
    );
    m!(
        "StartTitleScreen",
        &["call ClearBGPalettes"],
        "wait_frames",
        json!({"frames":4})
    );
    m!(
        "StartTitleScreen",
        &["pop af", "ldh [rWBK], a"],
        "restore_memory_byte",
        json!({"target":"rWBK"})
    );
    m!(
        "StartTitleScreen",
        &["ld hl, rLCDC", "res B_LCDC_OBJ_SIZE, [hl]"],
        "clear_memory_bit",
        json!({"target":"rLCDC","bit":"B_LCDC_OBJ_SIZE"})
    );
    let text = Source::read(vendor, "home/text.asm")?;
    text.operation(
        "ClearScreen",
        &["jr ClearTilemap"],
        "source_clear",
        json!({}),
    )?;
    text.operation(
        "ClearTilemap",
        &[
            "ldh a, [rLCDC]",
            "bit B_LCDC_ENABLE, a",
            "ret z",
            "jp WaitBGMap",
        ],
        "source_visible_wait",
        json!({}),
    )?;
    m!(
        "StartTitleScreen",
        &["call ClearScreen"],
        "wait_frames",
        json!({"frames":4})
    );
    m!(
        "StartTitleScreen",
        &["call WaitBGMap2"],
        "wait_frames",
        json!({"frames":8})
    );
    m!(
        "StartTitleScreen",
        &[
            "xor a",
            "ldh [hLCDCPointer], a",
            "ldh [hSCX], a",
            "ldh [hSCY], a"
        ],
        "write_memory_byte",
        json!({"target":"hLCDCPointer","value":0})
    );
    for target in ["hSCX", "hSCY"] {
        m!(
            "StartTitleScreen",
            &[&format!("ldh [{target}], a")],
            "write_memory_byte",
            json!({"target":target,"value":0})
        );
    }
    for (target, expression) in [("hWX", "$7"), ("hWY", "$90")] {
        m!(
            "StartTitleScreen",
            &[
                &format!("ld a, {expression}"),
                &format!("ldh [{target}], a")
            ],
            "write_memory_byte",
            json!({"target":target,"value":eval(expression)?})
        );
    }
    m!(
        "StartTitleScreen",
        &["ld b, SCGB_DIPLOMA", "call GetSGBLayout"],
        "apply_palette_layout",
        json!({"layout":{"symbol":"SCGB_DIPLOMA","value":eval("SCGB_DIPLOMA")?}})
    );
    m!(
        "StartTitleScreen",
        &["call UpdateTimePals"],
        "update_time_palettes",
        json!({"routine":"UpdateTimePals"})
    );
    m!(
        "StartTitleScreen",
        &["ld hl, .dw", "add hl, de", "add hl, de"],
        "dispatch_table",
        json!({"dispatcher":"StartTitleScreen option tail","index":"wTitleScreenSelectedOption","entries":options})
    );
    Ok(
        json!({"id":"start_title_screen","source_entry":"StartTitleScreen","accepted_call_forms":["call","jp"],"result":{},"phases":[{"id":"title_screen","source_span":menu.span("StartTitleScreen")?,"labels":labels,"operations":operations}],"loop":{},"resource_transfers":[],"tilemap_writes":[],"resources":[],"audio":[],"sprite_operations":[],"sprite_programs":[],"required_consumer":{},"source_span":menu.span("StartTitleScreen")?,"implementation_source_spans":[movie.span("_TitleScreen")?]}),
    )
}
