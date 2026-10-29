//! Final, authoritative scene and prose revision on an installed Northstar game.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::{CompiledGamePack, MapModule};

pub(crate) const MANIFEST: &str = "northstar-dialogue-overhaul-v1";

pub(crate) fn ensure_legacy_stage_allowed(base: &CompiledGamePack) -> Result<()> {
    ensure!(
        !base
            .report
            .manifests
            .iter()
            .any(|manifest| manifest == MANIFEST),
        "canonical staged dialogue is installed; rebuild from a reviewed parent with revise_northstar_story"
    );
    Ok(())
}

/// Protects canonical dialogue from later local-flavor overlays.
pub(crate) fn owns_text(map: &str, label: &str) -> bool {
    static LABELS: OnceLock<BTreeMap<String, BTreeSet<String>>> = OnceLock::new();
    LABELS
        .get_or_init(|| {
            let source: serde_json::Value =
                serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
                    .expect("checked-in canonical story JSON must be valid");
            source["dialogue_overhaul"]["texts"]
                .as_object()
                .expect("canonical dialogue map table must exist")
                .iter()
                .map(|(map, texts)| {
                    (
                        map.clone(),
                        texts
                            .as_object()
                            .expect("canonical dialogue labels must be objects")
                            .keys()
                            .cloned()
                            .collect(),
                    )
                })
                .collect()
        })
        .get(map)
        .is_some_and(|labels| labels.contains(label))
}

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
    let mut holding = pack
        .data
        .maps
        .get("MnMinneapolisDispatchOffice")
        .context("installed holding room")?
        .clone();
    crate::northstar_holding_fence::install(&mut pack, &mut holding)?;
    publish(&mut pack, holding)?;
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
    crate::northstar_clinic_revision::install(&mut pack)?;
    crate::northstar_story_connections::install(&mut pack)?;
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
    let depot = body
        .iter()
        .position(|row| *row == c("checkevent", &["EVENT_MN_DEPOT_CLEARED"]))
        .context("harbor custody prerequisite")?;
    ensure!(
        body.get(depot + 1) == Some(&c("iffalse", &["MN_HARBOR_RECEIVERDenied"])),
        "harbor custody branch changed"
    );
    body[depot + 1] = c("iffalse", &["MnOverhaulHarborMissingLog"]);
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
    map.scripts.insert(
        "MnOverhaulHarborMissingLog".into(),
        serde_json::json!([c("jumptextfaceplayer", &["MnOverhaulHarborMissingLogText"])]),
    );
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
    fn dialogue_canonical_pages_cover_the_campaign_and_are_protected() {
        let source: serde_json::Value =
            serde_json::from_str(include_str!("../../../planning/minnesota/story.json"))
                .expect("canonical JSON");
        let revision: super::Revision = serde_json::from_value(source["dialogue_overhaul"].clone())
            .expect("canonical dialogue schema");
        for (map, texts) in revision.texts {
            for (label, pages) in texts {
                assert!(
                    crate::northstar_dialogue::is_reviewed_story_text(&map, &label),
                    "canonical label must survive local overlays: {map}/{label}"
                );
                crate::authored_dialogue::compile(&pages).expect("authored native pages");
            }
        }
        assert!(super::owns_text("MnLeagueHallOfFame", "MnElyRecordText"));
        assert!(super::owns_text(
            "MnDuluthHarborWorkshop",
            "MnHarborSurveyReturnText"
        ));
        assert!(!super::owns_text("MnWinona", "UnrelatedFlavorText"));
    }

    #[test]
    #[ignore = "requires external CRYSTAL_STORY_PRIOR_PARENT and CRYSTAL_STORY_REVISION_PARENT"]
    fn dialogue_parent_keeps_reviewed_story_anchors() {
        let read = |name| {
            crate::read_verified_compiled_game_pack(PathBuf::from(
                std::env::var_os(name).expect("external dialogue parent"),
            ))
            .expect("verified dialogue parent")
        };
        let before = read("CRYSTAL_STORY_PRIOR_PARENT");
        let after = read("CRYSTAL_STORY_REVISION_PARENT");
        let source: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../planning/minnesota/story.json"),
            )
            .expect("canonical story"),
        )
        .expect("canonical JSON");
        assert_eq!(before.data.maps.len(), after.data.maps.len());
        assert_eq!(before.compiled_audio(), after.compiled_audio());
        assert_eq!(before.data.pokemon, after.data.pokemon);
        assert_eq!(before.data.trainers, after.data.trainers);
        assert_eq!(before.data.items, after.data.items);
        assert_eq!(before.data.moves, after.data.moves);
        for (name, value) in &before.data.story_event_script_constants.global {
            assert_eq!(
                after.data.story_event_script_constants.global.get(name),
                Some(value),
                "preserved global identity {name}"
            );
        }
        for (map, constants) in &before.data.story_event_script_constants.maps {
            if map != "MnR03Cave1F" {
                assert!(
                    after.data.story_event_script_constants.maps.get(map) == Some(constants),
                    "preserved local constants {map}"
                );
            }
        }
        for name in source["dialogue_overhaul"]["texts"]
            .as_object()
            .expect("story maps")
            .keys()
        {
            if name == "MnLakeItasca" {
                let mut expected = before.data.maps[name].clone();
                expected.blocks.clone_from(&after.data.maps[name].blocks);
                assert_eq!(
                    expected, after.data.maps[name],
                    "reviewed Itasca grass only"
                );
                continue;
            }
            if name == "MnWinonaFieldStation"
                && after
                    .report
                    .manifests
                    .iter()
                    .any(|m| m == "minnesota-kindlit-research-v1")
            {
                let before = &before.data.maps[name];
                let after = &after.data.maps[name];
                assert_eq!(before.blocks, after.blocks);
                assert_eq!(before.attributes, after.attributes);
                assert_eq!(before.events, after.events);
                assert_eq!(before.objects.len(), after.objects.len());
                for (original, revised) in before.objects.iter().zip(&after.objects) {
                    let mut restored = revised.clone();
                    if original.object_identifier.as_deref() == Some("MN_W_AIDE") {
                        assert_eq!(revised.script, "MnKindlitResearchAide");
                        restored.script.clone_from(&original.script);
                    }
                    assert_eq!(*original, restored);
                }
                for (label, body) in &before.scripts {
                    assert_eq!(
                        after.scripts.get(label),
                        Some(body),
                        "existing aide/scene program {label}"
                    );
                }
                assert!(
                    after
                        .scripts
                        .keys()
                        .filter(|label| !before.scripts.contains_key(*label))
                        .all(|label| label.starts_with("MnKindlit"))
                );
                continue;
            }
            assert!(
                before.data.maps[name] == after.data.maps[name],
                "story anchor map {name}"
            );
        }
        for (name, map) in &before.data.maps {
            if map != &after.data.maps[name] {
                assert!(
                    matches!(
                        name.as_str(),
                        "MnR20C00R00"
                            | "MnR20C00R01"
                            | "MnMaraisCedarApproach"
                            | "MnR03Cave1F"
                            | "MnBemidjiNorthwoods"
                            | "MnDetroitLakesApproach"
                            | "MnElyShorePass"
                            | "MnWinonaFieldStation"
                            | "MnMilleLacs"
                            | "MnGrandMaraisFieldStation"
                            | "MnLakeItasca"
                            | "MnPreservePineEntry"
                    ),
                    "unreviewed parent change {name}"
                );
                eprintln!("reviewed terrain descendant: {name}");
            }
        }
    }

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
        crate::northstar_holding_fence::assert_enclosure(&child);
        if let Some(path) = std::env::var_os("CRYSTAL_STORY_REVISION_PACK") {
            let published = crate::read_verified_compiled_game_pack(PathBuf::from(path))
                .expect("published dialogue candidate");
            assert_eq!(
                published.identity().expect("published identity"),
                child.identity().expect("source identity"),
                "published candidate must match the current canonical builder"
            );
        }
        assert_eq!(base.data.maps.len(), child.data.maps.len());
        assert_eq!(base.compiled_audio(), child.compiled_audio());
        for (name, bytes) in &base.runtime_files {
            assert_eq!(
                child.runtime_files.get(name),
                Some(bytes),
                "original art {name}"
            );
        }
        let added_files = if base
            .data
            .tilesets
            .contains_key(crate::northstar_holding_fence::TILESET)
        {
            0
        } else {
            5
        };
        assert_eq!(
            child.runtime_files.len(),
            base.runtime_files.len() + added_files
        );
        assert!(
            child
                .runtime_files
                .keys()
                .filter(|name| !base.runtime_files.contains_key(*name))
                .all(|name| name.contains("mn_northstar_holding"))
        );
        for (name, tileset) in &base.data.tilesets {
            assert_eq!(
                child.data.tilesets.get(name),
                Some(tileset),
                "original tileset {name}"
            );
        }
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
            "MnR13C00R00",
        ]);
        for (name, before) in &base.data.maps {
            let after = &child.data.maps[name];
            let mut expected_blocks = before.blocks.clone();
            if name == "MnMinneapolisDispatchOffice" {
                for (x, y, id) in crate::northstar_holding_fence::CELLS {
                    expected_blocks[y * 10 + x] = id;
                }
            }
            assert_eq!(expected_blocks, after.blocks, "terrain {name}");
            let mut expected_attributes = before.attributes.clone();
            if name == "MnMinneapolisDispatchOffice" {
                expected_attributes.tileset_name = crate::northstar_holding_fence::TILESET.into();
            }
            for (map, direction, target, offset) in [
                ("MnBoundaryWaters", "south", "MnR13C00R00", 1),
                ("MnR13C00R00", "north", "MnBoundaryWaters", -1),
            ] {
                if name == map {
                    expected_attributes
                        .connections
                        .iter_mut()
                        .find(|link| link.direction == direction && link.target_map == target)
                        .expect("existing League connection")
                        .offset = offset;
                }
            }
            assert_eq!(
                expected_attributes, after.attributes,
                "connections and geometry {name}"
            );
            if name == "MnR13C00R00" {
                let mut expected = before.clone();
                expected.attributes = expected_attributes;
                assert_eq!(
                    &expected, after,
                    "League approach changes only its reciprocal offset"
                );
            }
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
        for result in [
            crate::refresh_northstar_dialogue(&child, ""),
            crate::refresh_northstar_league_dialogue(&child, "", "", ""),
            crate::build_northstar_returns(&child, ""),
            crate::build_northstar_ice_lair(&child, "", "", ""),
            crate::build_northstar_drill_lair(&child, "", "", "", ""),
            crate::build_northstar_ice_headquarters(&child, "", &[]),
            crate::build_northstar_iron_workings(&child, "", &[]),
            crate::build_northstar_rival_arc(&child, "", ""),
        ] {
            match result {
                Err(error) => assert!(error.to_string().contains("canonical staged dialogue")),
                Ok(_) => panic!("legacy overlay must not replace canonical dialogue"),
            }
        }
        if child.data.maps["MnRochester"]
            .scripts
            .contains_key("MnFossilBagFull")
        {
            let mut rejected = child.clone();
            let mut map = rejected.data.maps["MnRochester"].clone();
            map.scripts
                .insert("MnFossilBagFull".into(), serde_json::json!([]));
            let result = crate::authored_town_publication::replace(&mut rejected, map);
            assert!(
                result.is_err_and(|error| error.to_string().contains("MnFossilBagFull")),
                "a shared fossil helper cannot be changed by a story publication"
            );
        }
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
