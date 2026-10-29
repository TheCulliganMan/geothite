//! Final, authoritative scene and prose revision on an installed Northstar game.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::{CompiledGamePack, MapModule};

pub(crate) const MANIFEST: &str = "northstar-dialogue-overhaul-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    schema_version: u8,
    id: String,
    approved_parents: Vec<String>,
    texts: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

/// Revises the installed Minnesota story without rebuilding unrelated content.
///
/// The canonical `dialogue_overhaul` section owns every revised text label and
/// runs after chapter overlays. Existing event identities and save identities
/// are not migrated. The returned pack has a new content identity.
///
/// # Errors
/// Rejects unreviewed parents, duplicate publication, missing scene anchors,
/// invalid native pages, retired geography, or invalid resulting runtime data.
/// The borrowed parent remains unchanged on error.
pub fn revise_northstar_story(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::verify_northstar_publication_geometry(base)?;
    let root: serde_json::Value = serde_json::from_str(source)?;
    let revision: Revision = serde_json::from_value(root["dialogue_overhaul"].clone())
        .context("canonical dialogue overhaul")?;
    ensure!(
        revision.schema_version == 1 && revision.id == MANIFEST,
        "unsupported dialogue revision"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "dialogue overhaul already installed"
    );
    ensure!(
        revision
            .approved_parents
            .contains(&base.identity()?.content_hash)
            || crate::northstar_current_story::is_composing(base),
        "reviewed current regional dialogue parent required"
    );
    let mut pack = base.clone();
    crate::northstar_opening_revision::install(&mut pack)?;
    crate::northstar_courier_revision::install(&mut pack)?;
    for (name, texts) in revision.texts {
        ensure!(
            name.starts_with("Mn"),
            "dialogue revision is Minnesota-only"
        );
        let labels = texts.keys().cloned().collect::<BTreeSet<_>>();
        let mut map = pack
            .data
            .maps
            .get(&name)
            .context("revised story map")?
            .clone();
        for (label, pages) in texts {
            ensure!(
                map.script_text_bodies.contains_key(&label) || label.starts_with("MnOverhaul"),
                "missing revised dialogue anchor: {name}/{label}"
            );
            map.scripts.insert(
                label.clone(),
                authored_dialogue::compile(&pages)
                    .with_context(|| format!("dialogue {name}/{label}"))?,
            );
        }
        // These story waits are authored acknowledgements: expose the native
        // cursor and confirmation cue on the final page as well as paragraphs.
        for body in map
            .scripts
            .values_mut()
            .filter_map(serde_json::Value::as_array_mut)
        {
            for index in 1..body.len() {
                if body[index]["command"] == "waitbutton"
                    && body[index - 1]["command"] == "writetext"
                    && body[index - 1]["args"][0]
                        .as_str()
                        .is_some_and(|label| labels.contains(label))
                {
                    body[index]["command"] = "promptbutton".into();
                }
            }
        }
        publish(&mut pack, map)?;
    }
    stage_harbor(&mut pack)?;
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

fn stage_harbor(pack: &mut CompiledGamePack) -> Result<()> {
    use crate::grand_marais_modpack::authoring::command as c;

    let mut map = pack
        .data
        .maps
        .get("MnDuluthHarborWorkshop")
        .context("harbor workshop")?
        .clone();
    let body = map
        .scripts
        .get_mut("MN_HARBOR_RECEIVER")
        .and_then(serde_json::Value::as_array_mut)
        .context("harbor conversation")?;
    let at = body
        .iter()
        .position(|row| *row == c("writetext", &["MN_HARBOR_RECEIVERText"]))
        .context("harbor evidence page")?;
    ensure!(
        body.get(at + 1) == Some(&c("promptbutton", &[])),
        "harbor acknowledgement changed"
    );
    let mut sequence = vec![
        c("writetext", &["MN_HARBOR_RECEIVERText"]),
        c("promptbutton", &[]),
    ];
    for label in [
        "MnOverhaulHarborEvidenceText",
        "MnOverhaulHarborReactionText",
        "MnOverhaulHarborDirectionsText",
    ] {
        sequence.extend([
            c("closetext", &[]),
            c("pause", &["10"]),
            c("opentext", &[]),
            c("writetext", &[label]),
            c("promptbutton", &[]),
        ]);
    }
    body.splice(at..at + 2, sequence);
    publish(pack, map)
}

pub(crate) fn publish(pack: &mut CompiledGamePack, mut map: MapModule) -> Result<()> {
    crate::ely_league_rooms::finish(pack, &mut map)?;
    map.script_audio_commands = crate::parse_script_audio_commands(&map.id, &map.scripts)?;
    crate::authored_town_publication::replace(pack, map)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_STORY_REVISION_PARENT external current-region pack"]
    fn dialogue_revision_preserves_geography_catalogs_and_unrelated_rooms() {
        let base = crate::read_verified_compiled_game_pack(PathBuf::from(
            std::env::var_os("CRYSTAL_STORY_REVISION_PARENT").expect("reviewed current pack"),
        ))
        .expect("verified parent");
        let source = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/story.json"),
        )
        .expect("canonical story");
        let child = super::revise_northstar_story(&base, &source).expect("staged revision");
        assert_eq!(base.data.maps.len(), child.data.maps.len());
        assert_eq!(base.compiled_audio(), child.compiled_audio());
        assert_eq!(base.runtime_files, child.runtime_files);
        assert_eq!(base.data.pokemon, child.data.pokemon);
        assert_eq!(base.data.moves, child.data.moves);
        assert_eq!(base.data.trainers, child.data.trainers);
        assert_eq!(base.data.items, child.data.items);
        assert_eq!(base.data.wild_encounters, child.data.wild_encounters);
        assert_eq!(base.data.field_encounters, child.data.field_encounters);
        assert_eq!(
            base.data.runtime_title_screen,
            child.data.runtime_title_screen
        );
        assert_eq!(
            base.data.runtime_map_metadata,
            child.data.runtime_map_metadata
        );
        let source: serde_json::Value = serde_json::from_str(&source).expect("story JSON");
        let mut changed = source["dialogue_overhaul"]["texts"]
            .as_object()
            .expect("dialogue maps")
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        changed.extend([
            "MnWinonaHome1F",
            "MnWinonaFieldStation",
            "MnStPaulUnionDepot",
        ]);
        for (name, before) in &base.data.maps {
            let after = &child.data.maps[name];
            assert_eq!(before.blocks, after.blocks, "terrain {name}");
            assert_eq!(
                before.attributes, after.attributes,
                "connections and geometry {name}"
            );
            assert_eq!(before.events.warps, after.events.warps, "warps {name}");
            if !changed.contains(name.as_str()) {
                assert!(before == after, "unrelated map {name}");
            }
            assert_eq!(
                child
                    .data
                    .assemble_map_module_from_compiled_payloads(name)
                    .expect("uncached native map"),
                *after,
                "native mirrors {name}"
            );
        }
        for (name, value) in &base.data.story_event_script_constants.global {
            assert_eq!(
                child.data.story_event_script_constants.global.get(name),
                Some(value),
                "event identity {name}"
            );
        }
        assert!(
            super::revise_northstar_story(&child, &serde_json::to_string(&source).expect("source"))
                .is_err()
        );
        let mut invalid = source;
        invalid["dialogue_overhaul"]["texts"]["MnWinonaHome1F"]["MnWHomeIntroText"][0][0] =
            "A line that cannot fit the native text box".into();
        assert!(
            super::revise_northstar_story(
                &base,
                &serde_json::to_string(&invalid).expect("invalid source")
            )
            .is_err()
        );
    }
}
