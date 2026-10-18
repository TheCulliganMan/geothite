//! Assemble source-certified presentation fragments; never publish a partial intro.
use anyhow::{Context, Result, ensure};
use crystal_assets::{
    RuntimePresentationAudio, RuntimePresentationBlock, RuntimePresentationProgram,
    RuntimePresentationResource, RuntimePresentationSourceSpan, RuntimeTitleScreen,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[path = "source_title_splash.rs"]
mod source_title_splash;
#[path = "source_title_credits.rs"]
mod source_title_credits;

pub(super) struct Certificate {
    file: String,
    lines: Vec<(u32, String)>,
}
impl Certificate {
    pub(super) fn read(vendor: &Path, file: &str) -> Result<Self> {
        let source = std::fs::read_to_string(vendor.join(file))?;
        let lines = source
            .lines()
            .enumerate()
            .filter_map(|(i, line)| {
                let code = super::source_syntax::code_part(line);
                (!code.is_empty()).then(|| {
                    (
                        i as u32 + 1,
                        code.split_whitespace().collect::<Vec<_>>().join(" "),
                    )
                })
            })
            .collect();
        Ok(Self {
            file: file.into(),
            lines,
        })
    }
    pub(super) fn sequence(&self, sequence: &[&str]) -> Result<RuntimePresentationSourceSpan> {
        ensure!(!sequence.is_empty(), "empty title source certificate");
        let matches = self
            .lines
            .windows(sequence.len())
            .filter(|window| {
                window
                    .iter()
                    .zip(sequence)
                    .all(|((_, actual), expected)| actual == expected)
            })
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1,
            "{} requires unique authored sequence {sequence:?}, found {}",
            self.file,
            matches.len()
        );
        let window = matches[0];
        Ok(RuntimePresentationSourceSpan {
            file: self.file.clone(),
            start_line: window[0].0,
            end_line: window[window.len() - 1].0,
        })
    }
    pub(super) fn operation(&self, sequence: &[&str], op: &str, fields: Value) -> Result<Value> {
        let mut fields = fields
            .as_object()
            .context("title operation fields")?
            .clone();
        fields.insert("op".into(), json!(op));
        fields.insert(
            "source_span".into(),
            serde_json::to_value(self.sequence(sequence)?)?,
        );
        Ok(Value::Object(fields))
    }
}

/// `intro` is the complete source export supplied by the intro lowering owner.
/// Missing, empty, or unrelated fragments fail before either catalog is written.
pub(super) fn assemble(root: &Path, intro: Value) -> Result<RuntimeTitleScreen> {
    assemble_catalog(root, intro, true)
}

/// Diagnostic assembly retains the authored catalog but does not certify it.
/// Callers must keep the resulting pack unvalidated and require explicit opt-in.
pub(super) fn assemble_unvalidated(root: &Path, intro: Value) -> Result<RuntimeTitleScreen> {
    assemble_catalog(root, intro, false)
}

fn assemble_catalog(root: &Path, intro: Value, validate: bool) -> Result<RuntimeTitleScreen> {
    let vendor = root.join("vendor/pokecrystal");
    let constants: Value = serde_json::from_slice(&std::fs::read(
        root.join("apps/web/assets/data/story_event_script_constants.json"),
    )?)?;
    let constants: BTreeMap<String, i64> = serde_json::from_value(constants["global"].clone())?;
    let mut program: RuntimePresentationProgram =
        serde_json::from_value(intro).context("complete authored intro presentation fragment")?;
    ensure!(
        program.schema_version == 1,
        "intro fragment requires schema version 1"
    );
    let crystal = program
        .subprograms
        .iter()
        .find(|s| s.id == "crystal_intro")
        .context("intro fragment is missing crystal_intro")?;
    ensure!(
        crystal.source_entry == "CrystalIntro"
            && !crystal.phases.is_empty()
            && crystal.phases.iter().all(|p| !p.operations.is_empty()),
        "intro fragment is incomplete"
    );
    let menu = Certificate::read(&vendor, "engine/menus/intro_menu.asm")?;
    for (id, value) in super::source_title_screen::startup_blocks(&vendor, &constants)? {
        insert_block(&mut program, &id, serde_json::from_value(value)?)?;
    }
    insert_block(
        &mut program,
        "Intro_MainMenu",
        serde_json::from_value(super::source_title_screen::main_menu_entry(&vendor)?)?,
    )?;
    let intro_ops = vec![
        menu.operation(
            &["callfar SplashScreen"],
            "call_subprogram",
            json!({"program":"splash_screen"}),
        )?,
        menu.operation(
            &["jr c, StartTitleScreen"],
            "branch_result",
            json!({"result":"splash_screen","equals":"cancelled","target":"StartTitleScreen"}),
        )?,
        menu.operation(
            &["farcall CrystalIntro"],
            "call_subprogram",
            json!({"program":"crystal_intro"}),
        )?,
        menu.operation(
            &["farcall CrystalIntro", "StartTitleScreen:"],
            "jump",
            json!({"target":"StartTitleScreen","fallthrough":true}),
        )?,
    ];
    insert_block(
        &mut program,
        "IntroSequence",
        serde_json::from_value(json!({
        "source_span":menu.sequence(&["IntroSequence:","callfar SplashScreen","jr c, StartTitleScreen","farcall CrystalIntro"])? ,
        "operations":intro_ops}))?,
    )?;
    let title = super::source_title_screen::title(&vendor, &constants)?;
    insert_block(
        &mut program,
        "StartTitleScreen",
        serde_json::from_value(json!({
        "source_span":title["source_span"], "operations":[{
            "op":"call_subprogram","program":"start_title_screen","source_span":title["source_span"]}]}))?,
    )?;
    for (label, implementation) in [
        ("DeleteSaveData", "_DeleteSaveData"),
        ("ResetClock", "_ResetClock"),
    ] {
        let call = format!("farcall {implementation}");
        insert_block(
            &mut program,
            label,
            serde_json::from_value(json!({
            "source_span":menu.sequence(&[&format!("{label}:"), &call, "jp Init"])? ,
            "operations":[menu.operation(&[&call],"call_block",json!({"block":implementation}))?,
                menu.operation(&[&format!("{label}:"),&call,"jp Init"],"reset_to_boot",json!({"routine":"Init","target":"GameInit"}))?]}))?,
        )?;
    }
    for value in [
        title,
        super::source_title_screen::main_menu(&vendor, &constants)?,
        super::source_title_screen::gender(&vendor, &constants)?,
        super::source_title_screen::oak_speech(&vendor, &constants)?,
        source_title_splash::lower(&vendor, &constants)?,
        source_title_credits::lower(&vendor, &constants)?,
    ] {
        let subprogram =
            serde_json::from_value::<crystal_assets::RuntimePresentationSubprogram>(value)?;
        ensure!(
            !program.subprograms.iter().any(|s| s.id == subprogram.id),
            "duplicate presentation subprogram {}",
            subprogram.id
        );
        program.subprograms.push(subprogram);
    }
    for value in super::source_title_screen::oak_text(&vendor)?
        .into_iter()
        .chain(super::source_title_screen::startup_text(&vendor)?)
    {
        let text = serde_json::from_value::<crystal_assets::RuntimePresentationText>(value)?;
        ensure!(
            !program.text.iter().any(|t| t.id == text.id),
            "duplicate presentation text {}",
            text.id
        );
        program.text.push(text);
    }
    program.entrypoints = [
        ("boot", "GameInit"),
        ("intro", "IntroSequence"),
        ("title", "StartTitleScreen"),
        ("main_menu", "Intro_MainMenu"),
        ("continue", "Continue"),
        ("new_game", "NewGame"),
        ("delete_save", "DeleteSaveData"),
        ("reset_clock", "ResetClock"),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value.into()))
    .collect();
    collect_catalogs(&vendor, &mut program)?;
    bind_field_animation_audio(&vendor, &mut program)?;
    for path in [
        "gfx/splash/copyright.2bpp",
        "gfx/splash/ditto.2bpp.lz",
        "gfx/splash/ditto.pal",
        "gfx/sgb/predef.pal",
    ] {
        ensure!(
            program
                .resources
                .iter()
                .any(|resource| resource.path == path),
            "required splash resource {path} is missing"
        );
    }
    ensure!(
        program
            .audio
            .iter()
            .any(|audio| audio.id == "SFX_DITTO_BOUNCE" && audio.kind == "sound_effect"),
        "required splash bounce audio is missing"
    );
    let title = RuntimeTitleScreen {
        title_music: Some("MUSIC_TITLE".into()),
        program,
    };
    if validate {
        crystal_assets::validate_runtime_title_screen(&title)?;
        crystal_assets::RuntimeTitlePresentationParameters::from_program(&title.program)
            .context("production title presentation consumer")?;
        crystal_assets::RuntimeTitleMainMenuDefinition::from_program(&title.program)
            .context("production main-menu consumer")?;
        crystal_assets::RuntimeGenderMenuDefinition::from_program(&title.program)
            .context("production gender-menu consumer")?;
        crystal_assets::RuntimeIntroPresentationParameters::from_program(&title.program)
            .context("production intro presentation consumer")?;
    }
    Ok(title)
}

fn bind_field_animation_audio(
    vendor: &Path,
    program: &mut RuntimePresentationProgram,
) -> Result<()> {
    let source = Certificate::read(vendor, "engine/events/heal_machine_anim.asm")?;
    let start = source
        .lines
        .iter()
        .position(|(_, code)| code == ".HOF_PlaySFX:")
        .context("authored Hall of Fame healing sound routine")?;
    let operand = source
        .lines
        .get(start + 1)
        .context("healing sound operand")?
        .1
        .strip_prefix("ld de, ")
        .context("healing sound register load")?;
    ensure!(
        operand.starts_with("SFX_"),
        "healing cue must reference an authored sound effect"
    );
    let span = source.sequence(&[
        ".HOF_PlaySFX:",
        &format!("ld de, {operand}"),
        "call PlaySFX",
        "call .FlashPalettes8Times",
        "call WaitSFX",
    ])?;
    ensure!(
        !program
            .host_effects
            .iter()
            .any(|effect| effect["id"] == "field_animation_audio"),
        "duplicate field animation audio binding"
    );
    program
        .host_effects
        .push(json!({"id":"field_animation_audio",
        "required_consumer":{"heal_machine_kind_2":operand},"source_span":span}));
    if !program
        .audio
        .iter()
        .any(|audio| audio.id == operand && audio.kind == "sound_effect")
    {
        program.audio.push(RuntimePresentationAudio {
            id: operand.into(),
            kind: "sound_effect".into(),
            source_span: span,
        });
    }
    Ok(())
}

fn insert_block(
    program: &mut RuntimePresentationProgram,
    id: &str,
    block: RuntimePresentationBlock,
) -> Result<()> {
    ensure!(
        !program.blocks.contains_key(id),
        "duplicate presentation block {id}"
    );
    program.blocks.insert(id.into(), block);
    Ok(())
}

fn collect_catalogs(vendor: &Path, program: &mut RuntimePresentationProgram) -> Result<()> {
    let mut audio = program
        .audio
        .iter()
        .map(|a| (a.id.clone(), a.kind.clone()))
        .collect::<BTreeSet<_>>();
    let mut resources = program
        .resources
        .iter()
        .map(|r| r.path.clone())
        .collect::<BTreeSet<_>>();
    for file in [
        "engine/movie/splash.asm",
        "engine/movie/title.asm",
        "engine/menus/intro_menu.asm",
    ] {
        let source = Certificate::read(vendor, file)?;
        for (line, code) in &source.lines {
            let span = RuntimePresentationSourceSpan {
                file: file.into(),
                start_line: *line,
                end_line: *line,
            };
            if let Some(path) = code
                .strip_prefix("INCBIN ")
                .or_else(|| code.strip_prefix("INCLUDE "))
            {
                let path = path.trim_matches('"');
                ensure!(
                    path.starts_with("gfx/") || path == "data/copyright.asm",
                    "unexpected presentation resource {path}"
                );
                if resources.insert(path.into()) {
                    program.resources.push(RuntimePresentationResource {
                        path: path.into(),
                        kind: if path == "data/copyright.asm" {
                            "source_text"
                        } else if path.ends_with(".pal") {
                            "palette"
                        } else {
                            "graphics"
                        }
                        .into(),
                        source_span: span.clone(),
                    });
                }
            }
            for token in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                let kind = if token.starts_with("MUSIC_") {
                    "music"
                } else if token.starts_with("SFX_") {
                    "sound_effect"
                } else {
                    continue;
                };
                if token == "MUSIC_NONE" {
                    continue;
                }
                if audio.insert((token.into(), kind.into())) {
                    program.audio.push(RuntimePresentationAudio {
                        id: token.into(),
                        kind: kind.into(),
                        source_span: span.clone(),
                    });
                }
            }
        }
    }
    let source = Certificate::read(vendor, "gfx/misc.asm")?;
    for (label, path) in [
        ("CopyrightGFX::", "gfx/splash/copyright.2bpp"),
        ("", "gfx/splash/ditto.2bpp.lz"),
    ] {
        let directive = format!("INCBIN \"{path}\"");
        let derived_label = source
            .lines
            .windows(2)
            .find(|pair| pair[1].1 == directive)
            .map(|pair| pair[0].1.as_str())
            .context("graphics include label")?;
        anyhow::ensure!(
            label.is_empty() || label == derived_label,
            "copyright graphics label changed"
        );
        let source_span = source.sequence(&[derived_label, &directive])?;
        if resources.insert(path.into()) {
            program.resources.push(RuntimePresentationResource {
                path: path.into(),
                kind: "graphics".into(),
                source_span,
            });
        }
    }
    for (file, directive, path) in [
        (
            "engine/gfx/cgb_layouts.asm",
            "INCLUDE \"gfx/splash/ditto.pal\"",
            "gfx/splash/ditto.pal",
        ),
        (
            "engine/gfx/color.asm",
            "INCLUDE \"gfx/sgb/predef.pal\"",
            "gfx/sgb/predef.pal",
        ),
    ] {
        let source = Certificate::read(vendor, file)?;
        let source_span = source.sequence(&[directive])?;
        if resources.insert(path.into()) {
            program.resources.push(RuntimePresentationResource {
                path: path.into(),
                kind: "palette".into(),
                source_span,
            });
        }
    }
    Ok(())
}

pub(super) fn export(root: &Path, intro: Value) -> Result<()> {
    let title = assemble(root, intro)?;
    export_timing_graphics(root, &title.program)?;
    let bytes = serde_json::to_vec_pretty(&title)?;
    let data = root.join("apps/web/assets/data");
    let catalog = data.join("content-packs/core-modular/runtime_title_screen");
    std::fs::create_dir_all(&catalog)?;
    std::fs::write(catalog.join("Source.json"), &bytes)?;
    std::fs::write(data.join("runtime_title_screen.json"), bytes)?;
    Ok(())
}

pub(super) fn export_timing_graphics(root: &Path, program: &RuntimePresentationProgram) -> Result<()> {
    for resource in &program.resources {
        if resource.kind == "graphics" && resource.path.ends_with(".lz") {
            crystal_assets::validate_compiled_runtime_file_key(&resource.path)?;
            let target = root.join("apps/web/assets").join(&resource.path);
            std::fs::create_dir_all(target.parent().context("source timing graphic parent")?)?;
            std::fs::copy(root.join("vendor/pokecrystal").join(&resource.path), &target)
                .with_context(|| format!("export source timing graphic {}", resource.path))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_splash_preserves_cancellation_timing_and_scene_delays() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE").map(std::path::PathBuf::from)
        else {
            return;
        };
        let constants: Value = serde_json::from_slice(
            &std::fs::read(root.join("apps/web/assets/data/story_event_script_constants.json"))
                .unwrap(),
        )
        .unwrap();
        let constants = serde_json::from_value(constants["global"].clone()).unwrap();
        let splash =
            source_title_splash::lower(&root.join("vendor/pokecrystal"), &constants).unwrap();
        if let Some(path) = std::env::var_os("CRYSTAL_SPLASH_REVIEW_JSON") {
            std::fs::write(path, serde_json::to_vec_pretty(&splash).unwrap()).unwrap();
        }
        let subprogram: crystal_assets::RuntimePresentationSubprogram =
            serde_json::from_value(splash).unwrap();
        let phase = &subprogram.phases[0];
        let waits = phase
            .operations
            .iter()
            .filter(|op| op.op == "wait_frames")
            .map(|op| op.fields["frames"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(waits, vec![10, 100, 1]);
        let cancelled = phase.labels[".pressed_button"];
        let completed = phase.labels[".finish"];
        assert_eq!(phase.operations[cancelled + 1].fields["carry"], true);
        assert_eq!(phase.operations[completed + 1].fields["carry"], false);
        assert_eq!(
            subprogram.required_consumer["scene_delays"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v["frames"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![32, 64, 128]
        );
        let mut program = RuntimePresentationProgram {
            schema_version: 1,
            ..Default::default()
        };
        program.subprograms.push(subprogram);
        let mut interpreter = crystal_assets::RuntimePresentationSubprogramInterpreter::new(
            &program,
            "splash_screen",
            "splash",
        )
        .unwrap();
        collect_catalogs(&root.join("vendor/pokecrystal"), &mut program).unwrap();
        assert!(
            program
                .resources
                .iter()
                .any(|resource| resource.path == "gfx/splash/ditto.pal")
        );
        assert!(
            program
                .audio
                .iter()
                .any(|audio| audio.id == "SFX_DITTO_BOUNCE" && audio.kind == "sound_effect")
        );
        interpreter
            .jump_to_label(&program, ".pressed_button")
            .unwrap();
        assert_eq!(
            interpreter.step(&program).unwrap().unwrap().op,
            "finish_studio_splash"
        );
        let mut machine = crystal_assets::RuntimePresentationPhaseMachine::new(
            &program,
            "splash_screen",
            "splash",
        )
        .unwrap();
        machine.memory.insert("wJumptableIndex".into(), 0);
        // The shared phase machine must reject unimplemented splash host effects.
        // Source lowering alone does not certify graphical splash execution.
        let cancelled = machine
            .run_from_label(&program, ".joy_loop", 1)
            .unwrap_err();
        assert!(
            cancelled
                .to_string()
                .contains("cannot execute source operation finish_studio_splash")
        );
        machine
            .memory
            .insert("wJumptableIndex".into(), 1 << constants["JUMPTABLE_EXIT_F"]);
        let completed = machine
            .run_from_label(&program, ".joy_loop", 0)
            .unwrap_err();
        assert!(
            completed
                .to_string()
                .contains("cannot execute source operation finish_studio_splash")
        );
        for (label, carry) in [(".pressed_button", true), (".finish", false)] {
            interpreter.jump_to_label(&program, label).unwrap();
            assert_eq!(
                interpreter.step(&program).unwrap().unwrap().op,
                "finish_studio_splash"
            );
            let returned = interpreter.step(&program).unwrap().unwrap();
            assert_eq!(returned.op, "return");
            assert_eq!(returned.fields["carry"], carry);
        }
    }
    #[test]
    fn incomplete_intro_fragment_cannot_publish_a_title_catalog() {
        let Some(root) = std::env::var_os("CRYSTAL_EXPORT_WORKSPACE").map(std::path::PathBuf::from)
        else {
            return;
        };
        let fragment = serde_json::to_value(RuntimePresentationProgram {
            schema_version: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(
            assemble_unvalidated(&root, fragment.clone())
                .unwrap_err()
                .to_string()
                .contains("missing crystal_intro")
        );
        assert!(
            assemble(&root, fragment)
                .unwrap_err()
                .to_string()
                .contains("missing crystal_intro")
        );
    }
}
