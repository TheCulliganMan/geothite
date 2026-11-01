//! A regional still title and Professor Tamarack's New Game welcome.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, RuntimePresentationSourceSpan, RuntimePresentationTextCommand};

pub const NORTHSTAR_TITLE_IMAGE: &str = "gfx/title/northstar.png";
const MANIFEST: &str = "minnesota-northstar-opening-v1";

/// Builds the Minnesota title and welcome from the current pack's native art.
///
/// Keeps all maps, gameplay, species, source-region art and audio unchanged.
/// `tamarack_source` is the accepted native NPC pixel source. Only its portrait
/// is installed; this does not rebind any existing field actor.
///
/// # Errors
/// Rejects a non-Minnesota or already revised pack, missing native art, invalid
/// Tamarack pixels, incompatible welcome operations or runtime validation errors.
pub fn build_northstar_title(
    base: &CompiledGamePack,
    tamarack_source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let roster = crate::authored_regional_approach::policy()?;
    ensure!(
        ["JIGLET", "KETTLURK"]
            .iter()
            .all(|name| roster.allowed.contains(*name)),
        "opening species must belong to the active curated roster"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "Northstar opening already installed"
    );
    ensure!(
        base.data
            .story_event_script_constants
            .global
            .contains_key("MN_DEX_MEMBER_KETTLURK")
            && base.data.maps.contains_key("MnWinonaHome2F"),
        "current Minnesota pack required"
    );
    let art = crate::minnesota_native_npc::compile(tamarack_source, None)?;
    ensure!(
        art.contains_key("gfx/trainers/mn_prof_tamarack.png"),
        "accepted Tamarack identity required"
    );
    let title = crate::northstar_title_art::compose(base.runtime_files())?;
    let mut pack = base.clone();
    for (path, bytes) in art
        .into_iter()
        .filter(|(path, _)| path.starts_with("gfx/trainers/"))
    {
        if let Some(old) = pack.runtime_files.get(&path) {
            ensure!(*old == bytes, "preserve existing Tamarack portrait");
        }
        pack.runtime_files.insert(path, bytes);
    }
    pack.runtime_files
        .insert(NORTHSTAR_TITLE_IMAGE.into(), title);
    pack.data.runtime_title_screen.static_title_image = Some(NORTHSTAR_TITLE_IMAGE.into());
    let program = &mut pack.data.runtime_title_screen.program;
    let speech = program
        .subprograms
        .iter_mut()
        .find(|s| s.id == "oak_speech")
        .and_then(|s| s.phases.iter_mut().find(|p| p.id == "speech"))
        .context("New Game welcome program")?;
    let mut portraits = 0;
    for operation in &mut speech.operations {
        if operation.op == "present_intro_portrait"
            && operation
                .fields
                .get("kind")
                .and_then(serde_json::Value::as_str)
                == Some("trainer")
        {
            operation
                .fields
                .insert("trainer_class".into(), "MN_PROF_TAMARACK".into());
            portraits += 1;
        }
    }
    ensure!(portraits == 2, "two professor portraits required");
    ensure!(
        base.data
            .story_event_script_constants
            .global
            .contains_key("MN_DEX_MEMBER_JIGLET"),
        "active Jiglet required"
    );
    for operation in &mut speech.operations {
        if operation.op == "present_intro_portrait"
            && operation
                .fields
                .get("kind")
                .and_then(serde_json::Value::as_str)
                == Some("pokemon")
        {
            operation.fields.insert("species".into(), "JIGLET".into());
        }
        if operation.op == "present_text_sequence"
            && let Some(audio) = operation
                .fields
                .get_mut("embedded_audio")
                .and_then(serde_json::Value::as_array_mut)
        {
            ensure!(audio.len() == 1, "one showcase cry required");
            audio[0]["cry"] = "JIGLET".into();
        }
    }

    // Keep the cry-bearing showcase entry and its native page boundary.
    for (id, pages, ending) in [
        (
            "OakText1",
            vec![
                ["Welcome to", "MINNESOTA!"],
                ["I'm Professor", "TAMARACK."],
                ["I study #MON", "who call it home."],
            ],
            "prompt",
        ),
        ("OakText2", vec![["Meet JIGLET,", "a local #MON!"]], "done"),
        (
            "OakText3",
            vec![
                ["From river bluffs", "to northern pines,"],
                ["#MON live all", "around us."],
            ],
            "prompt",
        ),
        (
            "OakText4",
            vec![
                ["We work and play", "alongside #MON."],
                ["Travel together.", "Learn their ways."],
            ],
            "prompt",
        ),
        (
            "OakText5",
            vec![
                ["I'm starting a", "new field survey."],
                ["Come to my station", "here in WINONA."],
                ["Your first partner", "is waiting there!"],
            ],
            "prompt",
        ),
        (
            "OakText6",
            vec![["Before we begin,", "what is your name?"]],
            "prompt",
        ),
        (
            "OakText7",
            vec![
                ["<PLAYER>, follow", "your curiosity."],
                ["Find a partner.", "Take the long way."],
                ["I'll meet you at", "the field station!"],
            ],
            "done",
        ),
        (
            "OakTimeWokeUpText",
            vec![
                ["TAMARACK: Before", "we head outside,"],
                ["let's set your", "field clock."],
            ],
            "prompt",
        ),
        (
            "OakTimeWhatTimeIsItText",
            vec![["What time is it?", ""]],
            "done",
        ),
        (
            "OakTimeWhatHoursText",
            vec![["The hour is", ""]],
            "text_end",
        ),
        (
            "OakTimeHowManyMinutesText",
            vec![["And the minutes?", ""]],
            "done",
        ),
        (
            "OakTimeWhoaMinutesText",
            vec![["Minutes past:", ""]],
            "text_end",
        ),
        (
            "OakTimeOversleptText",
            vec![["!", "A fresh morning!"]],
            "done",
        ),
        ("OakTimeYikesText", vec![["!", "Time to explore!"]], "done"),
        ("OakTimeSoDarkText", vec![["!", "A night survey!"]], "done"),
    ] {
        let text = program
            .text
            .iter_mut()
            .find(|t| t.id == id)
            .with_context(|| format!("welcome text {id}"))?;
        text.source_span = span();
        text.commands.clear();
        for (index, lines) in pages.into_iter().enumerate() {
            ensure!(
                lines.iter().all(|line| line
                    .replace("#MON", "POKEMON")
                    .replace("<PLAYER>", "1234567")
                    .chars()
                    .count()
                    <= 18),
                "welcome line exceeds the native text box: {id}"
            );
            text.commands.push(command(
                if index == 0 { "text" } else { "para" },
                Some(lines[0]),
            ));
            if !lines[1].is_empty() {
                text.commands.push(command("line", Some(lines[1])));
            }
        }
        if id == "OakText3" {
            text.commands.insert(0, command("text_promptbutton", None));
        }
        text.commands.push(command(ending, None));
    }
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    ensure!(
        pack.compiled_audio == base.compiled_audio && pack.audio_manifest == base.audio_manifest,
        "preserve complete audio programs and PCM contracts"
    );
    ensure!(
        pack.data.maps == base.data.maps,
        "preserve every complete map"
    );
    for (path, bytes) in &base.runtime_files {
        ensure!(
            pack.runtime_files.get(path) == Some(bytes),
            "preserve original resource {path}"
        );
    }
    Ok(pack)
}

fn span() -> RuntimePresentationSourceSpan {
    RuntimePresentationSourceSpan {
        file: "crates/crystal-assets/src/northstar_title.rs".into(),
        start_line: 1,
        end_line: 1,
    }
}
fn command(name: &str, text: Option<&str>) -> RuntimePresentationTextCommand {
    RuntimePresentationTextCommand {
        command: name.into(),
        args: text.map(|s| vec![format!("\"{s}\"")]).unwrap_or_default(),
        source_span: span(),
    }
}
