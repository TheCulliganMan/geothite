//! Scope preservation, native welcome controls, and malformed-input rejection.
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use super::{
    IntroRevision, MANIFEST, OPENING_CONFIG, OPENING_MARKER, WELCOME_LABELS,
    polish_northstar_intro, replace_welcome,
};
use crate::authored_dialogue::AuthoredDialoguePage;
use crate::{
    RuntimePresentationSourceSpan, RuntimePresentationText, RuntimePresentationTextCommand,
};

fn welcome() -> RuntimePresentationText {
    let span = RuntimePresentationSourceSpan::default();
    RuntimePresentationText {
        id: "OakText3".into(),
        source_span: span.clone(),
        commands: [
            ("text_promptbutton", None),
            ("text", Some("Old welcome.")),
            ("prompt", None),
        ]
        .into_iter()
        .map(|(command, text)| RuntimePresentationTextCommand {
            command: command.into(),
            args: text
                .map(|line| vec![format!("\"{line}\"")])
                .unwrap_or_default(),
            source_span: span.clone(),
        })
        .collect(),
    }
}

#[test]
fn intro_welcome_preserves_cry_acknowledgement_and_expands_native_widths() -> Result<()> {
    let mut text = welcome();
    let original = text.clone();
    replace_welcome(
        &mut text,
        &[
            AuthoredDialoguePage {
                lines: vec!["<PLAYER>, hello.".into(), "Meet a #MON.".into()],
            },
            AuthoredDialoguePage {
                lines: vec!["Take your time.".into()],
            },
        ],
    )?;
    ensure!(
        text.commands.first() == original.commands.first(),
        "cry acknowledgement changed"
    );
    ensure!(
        text.commands.last() == original.commands.last(),
        "final acknowledgement changed"
    );
    ensure!(
        text.commands[1].args == ["\"<PLAYER>, hello.\""],
        "player token was expanded in storage"
    );
    ensure!(
        text.commands[2].args == ["\"Meet a #MON.\""],
        "Pokemon token was expanded in storage"
    );
    ensure!(text.commands[3].command == "para", "page boundary was lost");
    for line in [
        "<PLAYER>, far too much!",
        "Hello, <OTHER>.",
        "A forbidden \".",
        "",
        "non-ASCII é",
    ] {
        let pages = [AuthoredDialoguePage {
            lines: vec![line.into()],
        }];
        ensure!(
            replace_welcome(&mut welcome(), &pages).is_err(),
            "accepted invalid welcome line: {line}"
        );
    }
    let mut without_cry_boundary = welcome();
    without_cry_boundary.commands.remove(0);
    ensure!(
        replace_welcome(
            &mut without_cry_boundary,
            &[AuthoredDialoguePage {
                lines: vec!["Hello.".into()]
            }]
        )
        .is_err(),
        "accepted a missing cry acknowledgement"
    );
    Ok(())
}

#[test]
#[ignore = "requires the selected parent in CRYSTAL_INTRO_BASE_PACK"]
fn intro_polish_preserves_everything_outside_selected_text_and_rejects_bad_sources() -> Result<()> {
    let path = std::env::var_os("CRYSTAL_INTRO_BASE_PACK").context("selected parent pack")?;
    let base = crate::read_verified_compiled_game_pack(path)?;
    let source_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/story.json");
    let source = std::fs::read_to_string(source_path)?;
    let source_json: Value = serde_json::from_str(&source)?;
    let revision: IntroRevision = serde_json::from_value(source_json["intro_revision"].clone())?;
    let revised = polish_northstar_intro(&base, &source)?;
    ensure!(
        base.identity != revised.identity,
        "revision needs a new content identity"
    );
    ensure!(
        revised
            .report
            .manifests
            .last()
            .is_some_and(|id| id == MANIFEST),
        "missing intro manifest"
    );
    ensure!(
        polish_northstar_intro(&revised, &source).is_err(),
        "duplicate intro revision accepted"
    );

    // Revert only the explicitly permitted fields. Equality of the remainder
    // covers every scene/command table, alias, Pokemon, asset, audio and report.
    let mut residual = revised.clone();
    for (map_name, labels) in &revision.dialogue_labels {
        let original = &base.data.maps[map_name];
        let room = residual
            .data
            .maps
            .get_mut(map_name)
            .context("revised room")?;
        for label in labels {
            let ending = |pack: &crate::CompiledGamePack| {
                pack.data.maps[map_name].scripts[label]
                    .as_array()
                    .and_then(|rows| rows.last())
                    .map(|row| row["command"].clone())
            };
            ensure!(
                ending(&base) == ending(&revised),
                "field acknowledgement changed: {label}"
            );
            room.scripts
                .insert(label.clone(), original.scripts[label].clone());
            match original.script_text_bodies.get(label) {
                Some(body) => {
                    room.script_text_bodies.insert(label.clone(), body.clone());
                }
                None => {
                    room.script_text_bodies.remove(label);
                }
            }
            match base.data.map_scripts.get(label) {
                Some(body) => {
                    residual
                        .data
                        .map_scripts
                        .insert(label.clone(), body.clone());
                }
                None => {
                    residual.data.map_scripts.remove(label);
                }
            }
        }
    }
    for text in &mut residual.data.runtime_title_screen.program.text {
        if WELCOME_LABELS.contains(&text.id.as_str()) {
            let original = base
                .data
                .runtime_title_screen
                .program
                .text
                .iter()
                .find(|candidate| candidate.id == text.id)
                .context("original welcome")?;
            ensure!(
                text.commands.last() == original.commands.last(),
                "welcome ending changed: {}",
                text.id
            );
            if text.id == "OakText3" {
                ensure!(
                    text.commands.first() == original.commands.first(),
                    "cry acknowledgement changed"
                );
            }
            *text = original.clone();
        }
    }
    ensure!(
        residual.runtime_files.remove(OPENING_MARKER).as_deref() == Some(OPENING_CONFIG),
        "animated opening opt-in changed"
    );
    residual.report.manifests.pop();
    residual.identity = base.identity.clone();
    ensure!(
        residual == base,
        "intro revision changed content outside its selected text scope"
    );

    let mut bad = source_json.clone();
    bad["intro_revision"]["dialogue_labels"]["NewBarkTown"] = serde_json::json!(["UnusedText"]);
    ensure!(
        polish_northstar_intro(&base, &bad.to_string()).is_err(),
        "out-of-region map accepted"
    );
    let mut bad = source_json.clone();
    bad["intro_revision"]["dialogue_labels"]["MnWinonaHome1F"] = serde_json::json!(["MissingText"]);
    ensure!(
        polish_northstar_intro(&base, &bad.to_string()).is_err(),
        "missing label accepted"
    );
    let mut bad = source_json.clone();
    bad["intro_revision"]["dialogue_labels"]["MnWinonaHome1F"] =
        serde_json::json!(["MnOverhaulMomScene"]);
    ensure!(
        polish_northstar_intro(&base, &bad.to_string()).is_err(),
        "scene program accepted as prose"
    );
    let mut bad = source_json.clone();
    bad["intro_revision"]["welcome"]["OakText7"] = serde_json::json!([["<PLAYER>, far too much!"]]);
    ensure!(
        polish_northstar_intro(&base, &bad.to_string()).is_err(),
        "overflowing player name accepted"
    );
    let mut bad = source_json;
    bad["intro_revision"]["welcome"]
        .as_object_mut()
        .context("welcome source")?
        .remove("OakText2");
    ensure!(
        polish_northstar_intro(&base, &bad.to_string()).is_err(),
        "incomplete welcome accepted"
    );
    ensure!(
        base.identity == residual.identity,
        "rejected revisions mutated their parent"
    );
    Ok(())
}
