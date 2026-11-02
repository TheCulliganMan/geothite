//! Exact-parent dialogue corrections for the installed Northstar story.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};

const PARENT_CONTENT_HASH: &str =
    "0af22959200a9f4a8f39f9be0fb67e9e5fc03f68553c5b39b40e6a1804afa25f";
const MANIFEST: &str = "northstar-coherent-dialogue-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    schema_version: u8,
    replacements: Vec<Replacement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacement {
    map: String,
    interaction: Option<String>,
    label: String,
    expected_lines: Vec<String>,
    pages: Vec<AuthoredDialoguePage>,
}

fn original_lines(program: &Value) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for command in program.as_array().context("text program is not an array")? {
        let opcode = command["command"].as_str().context("text opcode missing")?;
        match opcode {
            "text" | "line" | "para" | "cont" => {
                let quoted = command["args"][0]
                    .as_str()
                    .context("text argument missing")?;
                let value = quoted
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                    .context("expected literal text")?;
                lines.push(value.to_owned());
            }
            "done" => {}
            _ => anyhow::bail!("unsupported text control {opcode}"),
        }
    }
    Ok(lines)
}

/// Replace reviewed text while preserving game mechanics, maps and assets.
///
/// # Errors
/// Rejects a different parent, repeated revision, stale or shared labels,
/// unreachable references, unsupported text controls, or invalid native pages.
pub fn revise_northstar_coherence(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.identity()?.content_hash == PARENT_CONTENT_HASH,
        "unexpected Northstar parent"
    );
    ensure!(
        !base.report.manifests.iter().any(|item| item == MANIFEST),
        "revision already installed"
    );
    let revision: Revision = serde_json::from_str(source).context("parse dialogue revision")?;
    ensure!(
        revision.schema_version == 1 && !revision.replacements.is_empty(),
        "invalid revision schema"
    );
    let mut pack = base.clone();
    let mut seen = BTreeSet::new();
    for change in revision.replacements {
        ensure!(
            seen.insert((change.map.clone(), change.label.clone())),
            "duplicate dialogue target"
        );
        let owners: Vec<_> = pack
            .data
            .maps
            .values()
            .filter(|map| map.scripts.contains_key(&change.label))
            .map(|map| map.id.as_str())
            .collect();
        ensure!(
            owners == [change.map.as_str()],
            "shared or missing dialogue label {}",
            change.label
        );
        let room = pack
            .data
            .maps
            .get_mut(&change.map)
            .with_context(|| format!("missing map {}", change.map))?;
        if let Some(interaction) = &change.interaction {
            let program = room
                .scripts
                .get(interaction)
                .with_context(|| format!("missing interaction {interaction}"))?;
            ensure!(
                program
                    .as_array()
                    .is_some_and(|commands| commands.iter().any(|command| {
                        matches!(
                            command["command"].as_str(),
                            Some("jumptext" | "jumptextfaceplayer" | "writetext")
                        ) && command["args"][0] == change.label
                    })),
                "interaction {interaction} no longer references {}",
                change.label
            );
        } else {
            ensure!(
                room.scripts
                    .values()
                    .any(|program| program.as_array().is_some_and(|commands| {
                        commands.iter().any(|command| {
                            command["args"]
                                .as_array()
                                .is_some_and(|args| args.iter().any(|arg| arg == &change.label))
                        })
                    })),
                "dialogue {} has no installed script reference",
                change.label
            );
        }
        let old = room
            .scripts
            .get(&change.label)
            .context("missing source text")?;
        ensure!(
            original_lines(old)? == change.expected_lines,
            "stale dialogue {}/{}",
            change.map,
            change.label
        );
        ensure!(
            pack.data.map_scripts.get(&change.label) == Some(old),
            "split text differs for {}",
            change.label
        );
        let revised = authored_dialogue::compile(&change.pages)
            .with_context(|| format!("invalid pages {}/{}", change.map, change.label))?;
        room.scripts.insert(change.label.clone(), revised.clone());
        room.script_text_bodies = crate::parse_script_text_bodies(&room.id, &room.scripts)?;
        pack.data.map_scripts.insert(change.label, revised);
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
    Ok(pack)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revised_pages_fit_native_dialogue() {
        let source = include_str!(
            "../../../planning/minnesota/revisions/northstar-coherence-v01/dialogue.json"
        );
        let revision: Revision = serde_json::from_str(source).expect("reviewed revision");
        let canonical: Value =
            serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
                .expect("canonical story");
        assert!(!revision.replacements.is_empty());
        for change in revision.replacements {
            authored_dialogue::compile(&change.pages).expect("native page width and shape");
            let authored = &canonical["dialogue_overhaul"]["texts"][&change.map][&change.label];
            if !authored.is_null() {
                let expected: Vec<_> = change.pages.iter().map(|page| &page.lines).collect();
                assert_eq!(
                    *authored,
                    serde_json::json!(expected),
                    "canonical {}/{}",
                    change.map,
                    change.label
                );
            }
        }
    }

    #[test]
    #[ignore = "requires the deployed Northstar base pack"]
    fn published_coherence_preserves_unrelated_content() {
        let path = std::env::var_os("CRYSTAL_NORTHSTAR_COHERENCE_PARENT").expect("base pack path");
        let base = crate::read_verified_compiled_game_pack(path).expect("verified base");
        let source = include_str!(
            "../../../planning/minnesota/revisions/northstar-coherence-v01/dialogue.json"
        );
        let candidate = revise_northstar_coherence(&base, source).expect("coherent dialogue");
        assert_eq!(candidate.audio_manifest, base.audio_manifest);
        assert_eq!(candidate.compiled_audio, base.compiled_audio);
        assert_eq!(candidate.runtime_files, base.runtime_files);
        assert_eq!(candidate.data.map_blocks, base.data.map_blocks);
        assert_eq!(candidate.data.map_attributes, base.data.map_attributes);
        let changes = serde_json::from_str::<Revision>(source)
            .expect("revision")
            .replacements;
        let changed: BTreeSet<_> = changes.iter().map(|change| change.map.as_str()).collect();
        for (name, map) in &base.data.maps {
            if !changed.contains(name.as_str()) {
                assert_eq!(
                    candidate.data.maps.get(name),
                    Some(map),
                    "unrelated map {name}"
                );
            }
        }
        let mut restored_data = candidate.data.clone();
        restored_data.maps = base.data.maps.clone();
        restored_data.map_scripts = base.data.map_scripts.clone();
        assert_eq!(
            restored_data, base.data,
            "only map texts and their split scripts may change"
        );
        for name in changed {
            let original = &base.data.maps[name];
            let mut restored = candidate.data.maps[name].clone();
            for change in changes.iter().filter(|change| change.map == name) {
                restored.scripts.insert(
                    change.label.clone(),
                    original.scripts[&change.label].clone(),
                );
            }
            restored.script_text_bodies = original.script_text_bodies.clone();
            assert_eq!(
                restored, *original,
                "only reviewed text programs may change in {name}"
            );
        }
        assert!(revise_northstar_coherence(&candidate, source).is_err());
    }
}
