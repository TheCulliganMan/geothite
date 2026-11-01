//! A regional still title and Professor Tamarack's New Game welcome.
use anyhow::{Context, Result, ensure};

use crate::{CompiledGamePack, RuntimePresentationSourceSpan, RuntimePresentationTextCommand};

pub const NORTHSTAR_TITLE_IMAGE: &str = "gfx/title/northstar.png";
const MANIFEST: &str = "minnesota-northstar-opening-v1";

/// Builds the Minnesota title and welcome from the current pack's native art.
///
/// Keeps gameplay, species, source-region art and audio unchanged while binding
/// the existing field-station professor to her authored native sprite.
/// `tamarack_source` is the accepted native NPC pixel source.
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
    let binding =
        include_str!("../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/binding-v1.json");
    let pack = crate::bind_minnesota_trainer_art(base, binding, &art)?;
    let reed_art = crate::minnesota_native_npc::compile(
        include_str!("../../../planning/minnesota/npcs/cast/01-reed/native-pixels-v3.json"),
        Some(include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-field-repairs/01-reed/native-pixels-v2.json"
        )),
    )?;
    let mut pack = crate::bind_minnesota_trainer_art(
        &pack,
        include_str!(
            "../../../planning/minnesota/npcs/regional-trainers-v01/story-bindings/01-reed.json"
        ),
        &reed_art,
    )?;
    let cast_maps = pack.data.maps.clone();
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
        pack.data.maps == cast_maps,
        "preserve cast-bound maps while adding the title"
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

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires the external Northstar title parent pack"]
    fn title_installs_and_binds_tamarack_field_art() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root");
        let base = crate::read_verified_compiled_game_pack(
            root.join("content-packs/minnesota-iron-crosscut-bends-v01.crystalpack"),
        )
        .expect("Northstar title parent");
        let source = include_str!(
            "../../../planning/minnesota/npcs/tamarack-ice/01-tamarack/native-pixels-v4.json"
        );
        let pack = super::build_northstar_title(&base, source).expect("build Northstar title");
        let station = &pack.data.maps["MnWinonaFieldStation"];
        let actor = station
            .objects
            .iter()
            .find(|object| object.object_identifier.as_deref() == Some("MN_W_TAMARACK"))
            .expect("field-station professor");
        assert_eq!(actor.sprite, "SPRITE_MN_PROF_TAMARACK");
        assert!(actor.sprite_has_facings);
        assert_eq!(
            pack.data.trainers.trainers["MN_REED_WINONA"].trainer_class,
            "MN_RIVAL_REED"
        );
        let reed = station
            .objects
            .iter()
            .find(|object| object.script == "MN_NSE_REED")
            .expect("field-station Reed");
        assert_eq!(reed.sprite, "SPRITE_MN_RIVAL_REED");
        for extension in ["png", "2bpp", "gbcpal"] {
            assert!(
                pack.runtime_files
                    .contains_key(&format!("gfx/sprites/mn_prof_tamarack.{extension}"))
            );
        }
        if let Some(commands) = station
            .attributes
            .map_events_label
            .as_ref()
            .and_then(|label| station.scripts.get(label))
        {
            assert!(
                commands
                    .as_array()
                    .expect("event commands")
                    .iter()
                    .any(|command| {
                        command["command"] == "object_event"
                            && command["args"][2] == "SPRITE_MN_PROF_TAMARACK"
                            && command["args"][11] == "MN_W_TAMARACK"
                    })
            );
        }
    }
}
