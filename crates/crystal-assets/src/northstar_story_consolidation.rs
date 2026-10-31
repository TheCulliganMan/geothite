//! Bounded v27 scenes and writing on the current regional game.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const CURRENT: &str = "544539b3c51802c03cc284ae7ec3d683ea251596af80c71ef6f886ef4888aa16";
const COMMON: &str = "a7f6270ac5c7cdc83fe0f7484fd8485990ceb9749a0fde803dc43cbd22b90bad";
const DONOR: &str = "ccf84221d047e958736a4ca5cbecc917713a4496e03ffe89527e72e2d2593914";
const MARKER: &str = "northstar-current-v27-consolidation-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Writing {
    schema_version: u8,
    texts: BTreeMap<String, BTreeMap<String, Vec<AuthoredDialoguePage>>>,
}

/// Consolidates the exact reviewed v27 story delta into the current fauna/coast.
///
/// Current terrain, services, roster, teams, existing event identities and audio
/// are retained. Only the five reviewed indoor gray-fence cells change terrain.
/// A fresh introduction event is allocated without reusing another event bit.
/// Twenty-four authored conversation replacements retain the recovered writing
/// improvements and resolve newer service
/// and containment context. This returns an unselected pack, never a save migration.
///
/// # Errors
/// Rejects different input identities, repeated installation, changed donor
/// scope, unreviewed conflicts, occupied scene/actor anchors, invalid pages,
/// unrelated changes or failed runtime/geography validation. Inputs are borrowed
/// and remain unchanged on every error.
pub fn consolidate_northstar_story(
    current: &CompiledGamePack,
    common: &CompiledGamePack,
    donor: &CompiledGamePack,
    writing: &str,
) -> Result<CompiledGamePack> {
    for (pack, identity) in [(current, CURRENT), (common, COMMON), (donor, DONOR)] {
        ensure!(
            pack.identity()?.content_hash == identity,
            "exact story consolidation input required: {identity}"
        );
        crate::verify_compiled_game_pack_for_runtime(pack)?;
    }
    ensure!(
        !current.report.manifests.iter().any(|m| m == MARKER),
        "story consolidation already installed"
    );
    crate::verify_northstar_publication_geometry(current)?;
    crate::validate_compact_superior_geography(current)?;
    let writing: Writing =
        serde_json::from_str(writing).context("bounded conversation replacements")?;
    ensure!(
        writing.schema_version == 1
            && writing.texts.values().map(BTreeMap::len).sum::<usize>() == 24,
        "twenty-four reviewed conversations required"
    );
    let mut replacements = BTreeMap::new();
    for (map, labels) in writing.texts {
        ensure!(map.starts_with("Mn"), "Minnesota writing only");
        for (label, pages) in labels {
            ensure!(
                donor
                    .data
                    .maps
                    .get(&map)
                    .is_some_and(|m| m.script_text_bodies.contains_key(&label)),
                "replacement must address a v27 conversation: {map}/{label}"
            );
            replacements.insert((map.clone(), label), authored_dialogue::compile(&pages)?);
        }
    }
    let approved = BTreeSet::from([
        ("MnR01Farmhouse", "MN_R01_FARMERText"),
        ("MnWinonaFieldStation", "MnWTamarackDeliveryReturnText"),
        ("MnStPaulUnionDepot", "MN_GLACIER_LINE_TIMETABLEText"),
        ("MnStPaulUnionDepot", "MN_ST_PAUL_DEPOT_CLERKText"),
        ("MnBoundaryIntake", "MnArcIntakeIvyText"),
        ("MnBoundaryIntake", "MnArcIntakeFinnText"),
        ("MnLeechLake", "MN_LEECH_SHORE_FISHERText"),
        ("MnLeechLake", "MN_LEECH_SHORE_WALKERText"),
        ("MnWinonaFieldStation", "MnOverhaulTamarackIntroText"),
        ("MnWinonaFieldStation", "MnNSEReedArrivalText"),
        ("MnWinonaFieldStation", "MnWReadingsText"),
        ("MnDuluthHarborWorkshop", "MnOverhaulHarborDirectionsText"),
        (
            "MnMinneapolisDispatcherHouse",
            "MN_MP_DISPATCHER_PARTNERAfterText",
        ),
        ("MnMinneapolisDispatcherHouse", "MnNICEEllisAfterText"),
        ("MnMinneapolisDispatchOffice", "MnNICEReedAcceptText"),
        ("MnBemidji", "MnBemidjiComparisonText"),
        ("MnGrandRapidsSurveyDepot", "MnGrandRapidsDepotReadText"),
        ("MnBoundaryIntake", "MnArcFinalBeforeText"),
        ("MnBoundaryIntake", "MnArcFinalWonText"),
        ("MnIronDrillCore", "MnNDrillReedSeenText"),
        ("MnIronDrillCore", "MnNDrillReedAfterText"),
        ("MnStCloud", "MnArcSCStandText"),
        ("MnStCloud", "MnArcSCReleasedText"),
        ("MnStCloud", "MnArcSCAfterText"),
    ]);
    ensure!(
        replacements
            .keys()
            .map(|(map, label)| (map.as_str(), label.as_str()))
            .collect::<BTreeSet<_>>()
            == approved,
        "reviewed conversation scope changed"
    );
    let mut result = current.clone();
    install_additions(&mut result, common, donor)?;
    let mut maps = 0;
    let mut programs = 0;
    for (name, wanted) in &donor.data.maps {
        let before = common.data.maps.get(name).context("v27 cannot add a map")?;
        let delta = wanted
            .scripts
            .iter()
            .filter(|(label, body)| before.scripts.get(*label) != Some(*body))
            .collect::<Vec<_>>();
        if before == wanted {
            continue;
        }
        maps += 1;
        programs += delta.len();
        let original = current.data.maps.get(name).context("current story map")?;
        let mut map = original.clone();
        let mut changed = BTreeSet::new();
        for (label, body) in delta {
            if name == "MnBoundaryIntake" && label == "MnArcIntakeStop" {
                // This now leads to the persisted Verdigrim drill/outlet/bypass
                // sequence. The older single-motor program must never replace it.
                ensure!(
                    original.scripts[label] != before.scripts[label],
                    "new containment control required"
                );
                continue;
            }
            let replacement = replacements.get(&(name.clone(), label.clone()));
            let now = original.scripts.get(label);
            ensure!(
                replacement.is_some() || now == before.scripts.get(label) || now == Some(body),
                "unreviewed story conflict: {name}/{label}"
            );
            map.scripts
                .insert(label.clone(), replacement.unwrap_or(body).clone());
            changed.insert(label.clone());
        }
        // A replacement may refine a v27 conversation whose old parent already
        // happened to contain the same words, so it need not belong to delta.
        for ((replacement_map, label), body) in &replacements {
            if replacement_map == name {
                map.scripts.insert(label.clone(), body.clone());
                changed.insert(label.clone());
            }
        }
        merge_staging(&mut map, before, wanted)?;
        let event_rows = map.map_event_section_commands.clone();
        let script_rows = map.map_script_section_commands.clone();
        let spatial_change = map.objects != original.objects || map.events != original.events;
        crate::ely_league_rooms::finish(&result, &mut map)?;
        map.script_audio_commands = crate::parse_script_audio_commands(name, &map.scripts)?;
        map.map_script_section_commands = script_rows;
        if !spatial_change {
            map.map_event_section_commands = event_rows;
        }
        map = preserve_unmodified_caches(original, map, &changed)?;
        let attributes_changed = map.attributes != original.attributes;
        let blocks_changed = map.blocks != original.blocks;
        crate::authored_town_publication::replace(&mut result, map)?;
        // Publication refreshes split geography even for a text-only change.
        // Current packs retain some assembled aliases there; leave their exact
        // original entries (including absence) alone unless this scene changes
        // the corresponding physical field.
        if attributes_changed {
            ensure!(
                name == "MnMinneapolisDispatchOffice",
                "only the gray holding tileset may change attributes"
            );
            let mut attributes = current
                .data
                .map_attributes
                .get(name)
                .context("current split holding attributes")?
                .clone();
            attributes.tileset_name = "mn_northstar_holding".into();
            result.data.map_attributes.insert(name.clone(), attributes);
        } else {
            restore_split_entry(
                &mut result.data.map_attributes,
                &current.data.map_attributes,
                name,
            );
        }
        if !blocks_changed {
            if let Some(label) = original.attributes.blocks_label.as_ref() {
                restore_split_entry(&mut result.data.map_blocks, &current.data.map_blocks, label);
            }
        }
        if !spatial_change {
            if let Some(npcs) = current.data.npcs.get(name) {
                result.data.npcs.insert(name.clone(), npcs.clone());
            }
        }
    }
    ensure!(
        (maps, programs) == (57, 655),
        "reviewed v27 delta drifted: {maps} maps / {programs} programs"
    );
    for marker in donor
        .report
        .manifests
        .iter()
        .filter(|m| !common.report.manifests.contains(m))
    {
        if !result.report.manifests.contains(marker) {
            result.report.manifests.push(marker.clone());
        }
    }
    result.report.manifests.push(MARKER.into());
    verify_preservation(current, &result)?;
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    crate::verify_northstar_publication_geometry(&result)?;
    crate::validate_compact_superior_geography(&result)?;
    Ok(result)
}

fn restore_split_entry<T: Clone>(
    target: &mut BTreeMap<String, T>,
    current: &BTreeMap<String, T>,
    key: &str,
) {
    if let Some(value) = current.get(key) {
        target.insert(key.into(), value.clone());
    } else {
        target.remove(key);
    }
}

fn install_additions(
    pack: &mut CompiledGamePack,
    common: &CompiledGamePack,
    donor: &CompiledGamePack,
) -> Result<()> {
    ensure!(
        !pack
            .data
            .story_event_script_constants
            .global
            .contains_key("EVENT_MN_TAMARACK_INTRO"),
        "review existing professor introduction before consolidation"
    );
    crate::northstar_lair_authoring::flag(pack, "EVENT_MN_TAMARACK_INTRO")?;
    for map in ["MnWinonaFieldStation", "MnStPaulUnionDepot"] {
        let wanted = donor
            .data
            .story_event_script_constants
            .maps
            .get(map)
            .context("v27 scene constants")?;
        let previous = common.data.story_event_script_constants.maps.get(map);
        for (name, value) in wanted {
            if previous.and_then(|m| m.get(name)) == Some(value) {
                continue;
            }
            let table = pack
                .data
                .story_event_script_constants
                .maps
                .entry(map.into())
                .or_default();
            ensure!(
                !table.contains_key(name),
                "occupied current scene constant: {map}/{name}"
            );
            table.insert(name.clone(), *value);
        }
    }
    for (path, bytes) in &donor.runtime_files {
        if common.runtime_files.get(path) == Some(bytes) {
            continue;
        }
        ensure!(
            !common.runtime_files.contains_key(path),
            "v27 may only add reviewed art files: {path}"
        );
        ensure!(
            pack.runtime_files
                .get(path)
                .is_none_or(|existing| existing == bytes),
            "current art collision: {path}"
        );
        pack.runtime_files.insert(path.clone(), bytes.clone());
    }
    for (name, tileset) in &donor.data.tilesets {
        if common.data.tilesets.get(name) == Some(tileset) {
            continue;
        }
        ensure!(
            name == "mn_northstar_holding" && !pack.data.tilesets.contains_key(name),
            "only the reviewed gray holding tileset may be added"
        );
        pack.data.tilesets.insert(name.clone(), tileset.clone());
    }
    for (name, palette) in &donor.data.sprite_palette_defaults {
        if common.data.sprite_palette_defaults.get(name) == Some(palette) {
            continue;
        }
        ensure!(
            !common.data.sprite_palette_defaults.contains_key(name)
                && pack
                    .data
                    .sprite_palette_defaults
                    .get(name)
                    .is_none_or(|p| p == palette),
            "private cast palette collision"
        );
        pack.data
            .sprite_palette_defaults
            .insert(name.clone(), *palette);
    }
    for (name, class) in &donor.data.trainer_class_names {
        if common.data.trainer_class_names.get(name) == Some(class) {
            continue;
        }
        ensure!(
            !common.data.trainer_class_names.contains_key(name)
                && pack
                    .data
                    .trainer_class_names
                    .get(name)
                    .is_none_or(|c| c == class),
            "private trainer class collision"
        );
        pack.data
            .trainer_class_names
            .insert(name.clone(), class.clone());
    }
    for id in [
        "MN_REED_WINONA",
        "MN_REED_ROCHESTER",
        "MN_REED_STPAUL",
        "MN_REED_RELAY",
        "MN_REED_MINE",
        "MN_REED_BOUNDARY",
    ] {
        let trainer = pack
            .data
            .trainers
            .trainers
            .get_mut(id)
            .context("current Reed team")?;
        ensure!(
            matches!(trainer.trainer_class.as_str(), "RIVAL1" | "MN_RIVAL_REED"),
            "unexpected Reed class"
        );
        trainer.trainer_class = "MN_RIVAL_REED".into();
    }
    Ok(())
}

fn merge_staging(map: &mut MapModule, before: &MapModule, wanted: &MapModule) -> Result<()> {
    ensure!(
        before.objects.len() == wanted.objects.len(),
        "v27 cannot replace the actor population"
    );
    for (old, new) in before.objects.iter().zip(&wanted.objects) {
        if old == new {
            continue;
        }
        let mut permitted = old.clone();
        permitted.sprite.clone_from(&new.sprite);
        permitted.sprite_has_facings = new.sprite_has_facings;
        let visibility_change = new.script == "MN_ICE_HQ_TESS"
            || (map.id == "MnRochesterPokecenter1F"
                && matches!(
                    new.script.as_str(),
                    "MN_ROCHESTER_RECIPIENT" | "MN_RC_PATIENT"
                ));
        if visibility_change {
            permitted.event_flag.clone_from(&new.event_flag);
        }
        ensure!(
            permitted == *new,
            "unreviewed v27 actor mutation: {}/{}",
            map.id,
            new.script
        );
        let object = map
            .objects
            .iter_mut()
            .find(|o| o.object_identifier == old.object_identifier && o.script == old.script)
            .context("current staged actor")?;
        ensure!(
            (object.x, object.y) == (old.x, old.y),
            "current actor moved: {}/{}",
            map.id,
            old.script
        );
        ensure!(
            object.sprite == old.sprite || object.sprite == new.sprite,
            "current cast sprite conflict"
        );
        object.sprite.clone_from(&new.sprite);
        object.sprite_has_facings = new.sprite_has_facings;
        if visibility_change {
            ensure!(
                object.event_flag == old.event_flag || object.event_flag == new.event_flag,
                "current captive state conflict"
            );
            object.event_flag.clone_from(&new.event_flag);
        }
    }
    ensure!(
        before.events.warps == wanted.events.warps
            && before.events.bg_events == wanted.events.bg_events,
        "v27 scene delta may not move entrances or signs"
    );
    for event in &wanted.events.coord_events {
        if before.events.coord_events.contains(event) || map.events.coord_events.contains(event) {
            continue;
        }
        ensure!(
            matches!(
                map.id.as_str(),
                "MnWinonaHome1F" | "MnWinonaFieldStation" | "MnStPaulUnionDepot"
            ),
            "unreviewed coordinate scene"
        );
        ensure!(
            !map.events
                .coord_events
                .iter()
                .any(|e| (e.x, e.y) == (event.x, event.y)),
            "occupied scene approach"
        );
        map.events.coord_events.push(event.clone());
    }
    if before.scenes != wanted.scenes {
        ensure!(
            map.scenes == before.scenes || map.scenes == wanted.scenes,
            "current scene table conflict: {}",
            map.id
        );
        map.scenes.clone_from(&wanted.scenes);
        for row in &wanted.map_script_section_commands {
            if row.command != "scene_script"
                || before
                    .map_script_section_commands
                    .iter()
                    .any(|r| r.command == row.command && r.args == row.args)
            {
                continue;
            }
            if !map
                .map_script_section_commands
                .iter()
                .any(|r| r.command == row.command && r.args == row.args)
            {
                let at = map
                    .map_script_section_commands
                    .iter()
                    .position(|r| r.command == "def_callbacks")
                    .context("native callback section")?;
                map.map_script_section_commands.insert(at, row.clone());
            }
        }
        for (index, row) in map.map_script_section_commands.iter_mut().enumerate() {
            row.command_index = index;
        }
    }
    if before.blocks != wanted.blocks {
        if map.id == "MnMinneapolisDispatchOffice" {
            ensure!(
                map.blocks == before.blocks
                    && map.attributes.tileset_name == before.attributes.tileset_name,
                "gray holding room preimage changed"
            );
            map.blocks.clone_from(&wanted.blocks);
            map.attributes
                .tileset_name
                .clone_from(&wanted.attributes.tileset_name);
        } else {
            // The later coast already contains the corrected League turnout.
            ensure!(
                map.id == "MnR13C00R00" && map.blocks[16] == wanted.blocks[16],
                "current League floor alignment required"
            );
        }
    }
    Ok(())
}

// Reparse changed source programs, while retaining exact native rows owned by
// newer railway/research/containment programs and legacy assembled aliases.
fn preserve_unmodified_caches(
    original: &MapModule,
    revised: MapModule,
    changed: &BTreeSet<String>,
) -> Result<MapModule> {
    let old = serde_json::to_value(original)?;
    let mut new = serde_json::to_value(revised)?;
    for (field, old_rows) in old.as_object().context("typed map")? {
        let is_cache = field.starts_with("script_")
            || matches!(
                field.as_str(),
                "gift_pokemon_scripts" | "scripted_trainer_battles" | "scripted_wild_battles"
            );
        if !is_cache || !old_rows.is_array() {
            continue;
        }
        let rows = merge_cache_rows(field, old_rows, &new[field], changed)?;
        new[field] = serde_json::to_value(rows)?;
    }
    Ok(serde_json::from_value(new)?)
}

fn merge_cache_rows(
    field: &str,
    old: &Value,
    new: &Value,
    changed: &BTreeSet<String>,
) -> Result<Vec<Value>> {
    let owns = |row: &Value| {
        row["source_script"]
            .as_str()
            .is_some_and(|label| changed.contains(label))
            || (field == "script_movements"
                && row["label"]
                    .as_str()
                    .is_some_and(|label| changed.contains(label)))
    };
    Ok(old
        .as_array()
        .context("old typed rows")?
        .iter()
        .filter(|row| !owns(row))
        .chain(
            new.as_array()
                .context("reparsed typed rows")?
                .iter()
                .filter(|row| owns(row)),
        )
        .cloned()
        .collect())
}

fn verify_preservation(current: &CompiledGamePack, result: &CompiledGamePack) -> Result<()> {
    ensure!(
        result.data.maps.len() == current.data.maps.len(),
        "current map population must remain exact"
    );
    let mut split_attributes = current.data.map_attributes.clone();
    split_attributes
        .get_mut("MnMinneapolisDispatchOffice")
        .context("split holding attributes")?
        .tileset_name = "mn_northstar_holding".into();
    ensure!(
        result.data.map_attributes == split_attributes,
        "unrelated split map attributes changed"
    );
    let mut split_blocks = current.data.map_blocks.clone();
    let holding = &result.data.maps["MnMinneapolisDispatchOffice"];
    let label = holding
        .attributes
        .blocks_label
        .as_ref()
        .context("holding block label")?;
    let bytes = holding
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    split_blocks.insert(label.clone(), crate::encode_base64_bytes(&bytes));
    ensure!(
        result.data.map_blocks == split_blocks,
        "unrelated split map blocks changed"
    );
    for (name, old) in &current.data.maps {
        let new = result.data.maps.get(name).context("retained current map")?;
        let mut attributes = old.attributes.clone();
        let mut blocks = old.blocks.clone();
        if name == "MnMinneapolisDispatchOffice" {
            attributes.tileset_name = "mn_northstar_holding".into();
            for &(x, y, id) in &crate::northstar_holding_fence::CELLS {
                blocks[y * 10 + x] = id;
            }
        }
        ensure!(
            new.attributes == attributes
                && new.blocks == blocks
                && new.events.warps == old.events.warps
                && new.events.bg_events == old.events.bg_events,
            "current geography changed: {name}"
        );
        for (label, body) in &old.scripts {
            let protected = label.starts_with("MnGlacier")
                || label.starts_with("MnVerdigrim")
                || label.starts_with("MnEarlyPartner")
                || label == "MnKindlitResearchAide"
                || (name == "MnWinonaFieldStation"
                    && (label.starts_with("MnReward")
                        || matches!(label.as_str(), "MN_W_AIDE" | "MN_W_TAMARACK")))
                || (name == "MnBoundaryIntake" && label == "MnArcIntakeStop");
            if protected {
                ensure!(
                    new.scripts.get(label) == Some(body),
                    "current service or containment program changed: {name}/{label}"
                );
            }
        }
        for row in old
            .map_script_section_commands
            .iter()
            .filter(|row| row.command == "callback")
        {
            ensure!(
                new.map_script_section_commands
                    .iter()
                    .any(|r| r.command == row.command && r.args == row.args),
                "current map callback removed: {name}"
            );
        }
    }
    for (name, value) in &current.data.story_event_script_constants.global {
        ensure!(
            result.data.story_event_script_constants.global.get(name) == Some(value),
            "existing global identity changed: {name}"
        );
    }
    for (map, constants) in &current.data.story_event_script_constants.maps {
        for (name, value) in constants {
            ensure!(
                result
                    .data
                    .story_event_script_constants
                    .maps
                    .get(map)
                    .and_then(|m| m.get(name))
                    == Some(value),
                "existing map identity changed: {map}/{name}"
            );
        }
    }
    let mut teams = result.data.trainers.clone();
    for (id, trainer) in &mut teams.trainers {
        trainer
            .trainer_class
            .clone_from(&current.data.trainers.trainers[id].trainer_class);
    }
    ensure!(
        teams == current.data.trainers,
        "current trainer teams, AI or rewards changed"
    );
    let mut data = result.data.clone();
    data.maps.clone_from(&current.data.maps);
    data.map_scripts.clone_from(&current.data.map_scripts);
    data.map_attributes.clone_from(&current.data.map_attributes);
    data.map_blocks.clone_from(&current.data.map_blocks);
    data.npcs.clone_from(&current.data.npcs);
    data.story_event_script_constants
        .clone_from(&current.data.story_event_script_constants);
    data.sprite_palette_defaults
        .clone_from(&current.data.sprite_palette_defaults);
    data.trainers.clone_from(&current.data.trainers);
    data.trainer_class_names
        .clone_from(&current.data.trainer_class_names);
    data.tilesets.clone_from(&current.data.tilesets);
    ensure!(
        data == current.data,
        "story consolidation changed unrelated game catalogs or services"
    );
    ensure!(
        result.audio_manifest == current.audio_manifest
            && result.compiled_audio == current.compiled_audio
            && result.audio_compression == current.audio_compression,
        "current audio must remain exact"
    );
    ensure!(
        current
            .runtime_files
            .iter()
            .all(|(path, bytes)| result.runtime_files.get(path) == Some(bytes)),
        "current runtime asset replaced"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::json;

    #[test]
    fn text_publication_retains_stale_split_aliases_and_missing_entries() {
        let current =
            std::collections::BTreeMap::from([("MnR08C00R00".into(), "mn_headwaters".to_owned())]);
        let mut refreshed = std::collections::BTreeMap::from([
            (
                "MnR08C00R00".into(),
                "mn_outdoor_headwaters_stairs".to_owned(),
            ),
            ("Unexpected".into(), "new".to_owned()),
        ]);
        super::restore_split_entry(&mut refreshed, &current, "MnR08C00R00");
        super::restore_split_entry(&mut refreshed, &current, "Unexpected");
        assert_eq!(refreshed, current);
    }

    #[test]
    fn changed_intro_does_not_rewrite_current_research_or_train_cache_rows() {
        let old = json!([
            {"source_script":"Intro","command_index":0,"text_label":"Old"},
            {"source_script":"MnGlacierWINONAChoose0_0","command_index":7,"value":9},
            {"source_script":"MnEarlyPartners","command_index":2,"value":225}
        ]);
        let reparsed = json!([
            {"source_script":"Intro","command_index":4,"text_label":"New"},
            {"source_script":"MnGlacierWINONAChoose0_0","command_index":1,"value":9},
            {"source_script":"MnEarlyPartners","command_index":0,"value":225}
        ]);
        let rows = super::merge_cache_rows(
            "script_text_commands",
            &old,
            &reparsed,
            &BTreeSet::from(["Intro".into()]),
        )
        .expect("program-scoped rows");
        assert_eq!(
            rows,
            vec![old[1].clone(), old[2].clone(), reparsed[0].clone()]
        );
    }

    #[test]
    fn changed_movement_label_replaces_its_standalone_native_definition() {
        let old = json!([{"source_script":null,"label":"Approach","steps":["DOWN"]},{"source_script":null,"label":"TrainExit","steps":["LEFT"]}]);
        let reparsed = json!([{"source_script":null,"label":"Approach","steps":["UP"]},{"source_script":null,"label":"TrainExit","steps":["RIGHT"]}]);
        let rows = super::merge_cache_rows(
            "script_movements",
            &old,
            &reparsed,
            &BTreeSet::from(["Approach".into()]),
        )
        .expect("movement-scoped rows");
        assert_eq!(rows, vec![old[1].clone(), reparsed[0].clone()]);
    }

    #[test]
    #[ignore = "requires exact current coast/fauna, Minneapolis cycling parent and v27 packs in ignored content-packs"]
    fn consolidation_keeps_current_services_and_all_three_authored_opening_scenes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let load = |name: &str| {
            crate::read_verified_compiled_game_pack(root.join("content-packs").join(name))
                .expect("explicit exact external story fixture")
        };
        let current = load("minnesota-north-shore-passing-width-fauna-v02.crystalpack");
        let common = load("minnesota-minneapolis-citywide-cycling-v02.crystalpack");
        let donor = load("minnesota-dialogue-overhaul-v27.crystalpack");
        let writing = std::fs::read_to_string(
            root.join("planning/minnesota/revisions/story-consolidation-v01/writing.json"),
        )
        .expect("checked-in authored conversation source");
        let result = super::consolidate_northstar_story(&current, &common, &donor, &writing)
            .expect("bounded story consolidation");
        for (map, label) in [
            ("MnWinonaHome1F", "MnOverhaulMomScene"),
            ("MnWinonaFieldStation", "MnOverhaulLabEntry"),
            ("MnStPaulUnionDepot", "MnOverhaulDepotEntry"),
        ] {
            assert_eq!(
                result.data.maps[map].scripts[label],
                donor.data.maps[map].scripts[label]
            );
        }
        for (map, label) in [
            ("MnWinonaFieldStation", "MnKindlitResearchAide"),
            ("MnStPaulUnionDepot", "MN_ST_PAUL_GLACIER_CONDUCTOR"),
            ("MnBoundaryIntake", "MnArcIntakeStop"),
        ] {
            assert_eq!(
                result.data.maps[map].scripts[label],
                current.data.maps[map].scripts[label]
            );
        }
        assert_ne!(
            result.identity().expect("new identity"),
            current.identity().expect("retained parent identity")
        );
    }
}
