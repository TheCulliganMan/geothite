//! Ely's League sign and the registrar's last advice before the five matches.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::{CompiledGamePack, authored_town_publication};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PageEdit {
    before: Vec<AuthoredDialoguePage>,
    after: Vec<AuthoredDialoguePage>,
}

/// Updates the directions outside the Ely League and before registration.
///
/// # Errors
/// Returns an error if the edition, earlier pages or canonical story differ
/// from the authored revision, or if the resulting archive is invalid.
pub fn revise_ely_challenge(mut pack: CompiledGamePack) -> Result<CompiledGamePack> {
    ensure!(
        pack.report.manifests.last().map(String::as_str) == Some("northstar-badge-battles-v1"),
        "Ely Challenge requires the Badge Battles edition"
    );
    let edits: BTreeMap<String, BTreeMap<String, PageEdit>> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/ely-challenge-v01/registration.json"
    ))?;
    let story: Value =
        serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))?;
    let signs = story["regional_guidance_revision"]["MnEly"]["replies"]
        .as_array()
        .context("Ely sign replies")?;
    for (name, labels) in edits {
        let mut room = pack
            .data
            .maps
            .get(&name)
            .with_context(|| name.clone())?
            .clone();
        let mut texts = BTreeMap::new();
        for (label, edit) in labels {
            let previous = authored_dialogue::compile(&edit.before)?;
            ensure!(
                room.scripts.get(&label) == Some(&previous)
                    && pack.data.map_scripts.get(&label) == Some(&previous),
                "unexpected prior text for {name}/{label}"
            );
            let canonical = if name == "MnEly" {
                let id = label
                    .strip_suffix("Text")
                    .context("expected an Ely sign text label")?;
                &signs
                    .iter()
                    .find(|reply| reply["id"] == id)
                    .context("expected a sign reply")?["pages"]
            } else {
                &story["dialogue_overhaul"]["texts"][&name][&label]
            };
            let pages: Vec<AuthoredDialoguePage> = serde_json::from_value(canonical.clone())?;
            let updated = authored_dialogue::compile(&pages)?;
            ensure!(
                updated == authored_dialogue::compile(&edit.after)?,
                "canonical pages differ at {label}"
            );
            texts.insert(label.clone(), updated.clone());
            room.scripts.insert(label, updated);
        }
        room.script_text_bodies
            .extend(crate::parse_script_text_bodies(&name, &texts)?);
        authored_town_publication::replace(&mut pack, room)?;
    }
    pack.report
        .manifests
        .push("northstar-ely-challenge-v1".into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}
