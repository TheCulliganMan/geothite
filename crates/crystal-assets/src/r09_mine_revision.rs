//! Bounded terrain revision of the already installed Q06 mine and its stale mirrors.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde_json::Value;

use crate::authored_cave_revision::{blocks, validate_terrain};
use crate::authored_route_design::Interior;
use crate::authored_route_modpack::{collision, validate_room};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MINE: &str = "MnR09Cave1F";
const FOREMAN: &str = "MN_R09_DEPOT_FOREMAN";
const CACHE: &str = "MN_R09_SURVEY_CACHE";
const TRAINER: &str = "MN_LODE_FOREMAN";

/// Revises the installed 12×11 Q06 mine without changing its story behavior.
///
/// Sources must retain every field except rows, purpose, candidate_status and
/// review_contract. The installed scout, talk battle, canonical story texts,
/// trainer and quest cache are certified using the original story implementation.
/// Only terrain is replaced in the cached module. This operation also repairs
/// one explicitly supported historical publication preimage: the global NPC and
/// event payloads contain only the scout, and all 19 Q06 roots are absent. Those
/// exact cache-owned mirrors are synchronized, not newly authored story content.
/// All other maps, programs, trainer/flag values, art, audio, HOME, recovery,
/// report and storage remain unchanged. No game-state progress is granted.
///
/// # Errors
/// Returns an error for source, party, flag, program, typed-index or split drift;
/// unsupported or partially synchronized mirrors; conflicting script ownership;
/// unsafe geometry; or failed native pack verification. Inputs are unchanged.
/// Previously saved block overrides and earned regional progression are outside
/// this local terrain revision's acceptance contract.
pub fn revise_r09_mine_terrain(
    base: &CompiledGamePack,
    previous_room_json: &str,
    revised_room_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    let old_raw: Value = serde_json::from_str(previous_room_json)?;
    let new_raw: Value = serde_json::from_str(revised_room_json)?;
    let old: Interior = serde_json::from_value(old_raw.clone())?;
    let new: Interior = serde_json::from_value(new_raw.clone())?;
    let retained = |mut raw: Value| -> Result<Value> {
        let fields = raw.as_object_mut().context("R09 room object")?;
        for key in ["rows", "purpose", "candidate_status", "review_contract"] {
            fields.remove(key);
        }
        Ok(raw)
    };
    ensure!(
        retained(old_raw.clone())? == retained(new_raw)?,
        "R09 retained source drift"
    );
    ensure!(
        old.id == MINE
            && old.dimensions_metatiles == [12, 11]
            && old.exits == [[13, 19]]
            && old.clear_arrival == Some([13, 18])
            && old.exit_doors == [[40, 41]]
            && old.exit_collision == "LADDER"
            && old.passive_arrival.is_none()
            && old.keeper_tile.is_none()
            && old.items.is_empty()
            && old.signs.is_empty(),
        "requires the original bounded Q06 mine"
    );
    let previous = blocks(&old)?;
    let replacement = blocks(&new)?;
    ensure!(previous != replacement, "R09 terrain is unchanged");
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let original = base.data.maps.get(MINE).context("installed Q06 mine")?;
    let a = &original.attributes;
    ensure!(
        original.id == MINE
            && original.blocks == previous
            && a.tileset_name == "cave"
            && a.environment.as_deref() == Some("CAVE")
            && [a.width, a.height] == [12, 11]
            && a.connections.is_empty()
            && a.time_of_day.as_deref() == Some("nite")
            && a.palette.as_deref() == Some("PALETTE_NITE")
            && base.data.map_attributes.get(MINE) == Some(a),
        "R09 native identity/geometry/palette preimage differs"
    );
    let blocks_label = a.blocks_label.as_deref().context("R09 block label")?;
    let scripts_label = a
        .map_scripts_label
        .as_deref()
        .context("R09 scripts label")?;
    let events_label = a.map_events_label.as_deref().context("R09 events label")?;
    ensure!(
        blocks_label == "MnR09Cave1F_Blocks"
            && scripts_label == "MnR09Cave1F_MapScripts"
            && events_label == "MnR09Cave1F_MapEvents"
            && base.data.maps.iter().all(|(name, m)| name == MINE
                || m.attributes.blocks_label.as_deref() != Some(blocks_label)),
        "R09 payload identity/ownership differs"
    );
    ensure!(
        crate::decode_base64_bytes(
            base.data
                .map_blocks
                .get(blocks_label)
                .context("R09 blocks")?
        )?
        .into_iter()
        .map(u16::from)
        .eq(previous),
        "R09 encoded preimage differs"
    );
    let ordinary = certify_story(base, original, &old, &old_raw, story_json)?;
    certify_stale_split(base, original, &ordinary, scripts_label, events_label)?;
    let mut revised = original.clone();
    revised.blocks = replacement;
    validate_room(base, &revised, &new)?;
    validate_terrain(base, &revised, &new)?;
    validate_approaches(base, &revised, &old_raw)?;
    let mut result = base.clone();
    let bytes = revised
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    result
        .data
        .map_blocks
        .insert(blocks_label.into(), crate::encode_base64_bytes(&bytes));
    crate::insert_compiled_map_split_payloads(
        &mut result.data.map_scripts,
        &mut result.data.npcs,
        MINE,
        scripts_label,
        events_label,
        &revised,
    )?;
    certify_uncached(&result, &revised, scripts_label, events_label)?;
    result.data.maps.insert(MINE.into(), revised);
    result.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        result.format_version,
        &result.data,
        &result.audio_manifest,
        &result.runtime_files,
        &result.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&result)?;
    Ok(result)
}

fn certify_story(
    base: &CompiledGamePack,
    module: &MapModule,
    room: &Interior,
    source: &Value,
    story_json: &str,
) -> Result<MapModule> {
    let story: Value = serde_json::from_str(story_json)?;
    ensure!(
        module.objects.len() == 3
            && module.events.warps.len() == 1
            && module.events.warps[0].index == 1
            && [module.events.warps[0].x, module.events.warps[0].y] == [13, 19]
            && module.events.warps[0].target_map == "MnR09C00R00"
            && module.events.warps[0].target_warp_id == 2
            && module.events.bg_events.is_empty()
            && module.events.coord_events.is_empty(),
        "R09 original object/warp inventory differs"
    );
    let mut ordinary = module.clone();
    ordinary
        .objects
        .retain(|o| ![FOREMAN, CACHE].contains(&o.script.as_str()));
    ensure!(
        ordinary.objects.len() == 1,
        "R09 story actor identities differ"
    );
    let (foreman, cache, texts) =
        crate::iron_range_story::validate_source(base, &ordinary, source, &story)?;
    let mut expected = module.clone();
    expected.objects.clear();
    expected.scripts.clear();
    expected.script_text_bodies.clear();
    crate::authored_route_events::room_events(base, &mut expected, room)?;
    ordinary = expected.clone();
    for (id, tile, sprite) in [
        (FOREMAN, foreman, "SPRITE_ROCKET"),
        (CACHE, cache, "SPRITE_POKE_BALL"),
    ] {
        authoring::npc(
            base,
            &mut expected,
            id,
            sprite,
            i16::try_from(tile[0])?,
            i16::try_from(tile[1])?,
        )?;
        let object = expected.objects.last_mut().context("Q06 expected actor")?;
        object.object_identifier = Some(id.into());
        object.object_type = "OBJECTTYPE_SCRIPT".into();
        object.radius = 0;
        object.sprite_has_facings = sprite != "SPRITE_POKE_BALL";
    }
    expected.scripts.extend(crate::iron_range_story::scripts());
    for (label, lines) in texts {
        authoring::text(
            &mut expected,
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    ensure!(
        expected.objects == module.objects && expected.scripts == module.scripts,
        "R09 native actor/program source certification failed"
    );
    let mut trainer_control = base.clone();
    trainer_control
        .data
        .trainers
        .trainers
        .remove(TRAINER)
        .context("installed foreman party")?;
    trainer_control
        .data
        .story_event_script_constants
        .global
        .remove(TRAINER)
        .context("installed foreman numeric identity")?;
    crate::iron_range_story::add_trainer(&mut trainer_control)?;
    ensure!(
        trainer_control.data.trainers == base.data.trainers
            && trainer_control.data.story_event_script_constants
                == base.data.story_event_script_constants,
        "R09 original native trainer template/party/identity drifted"
    );
    for (name, value) in [
        ("ENGINE_GLACIERBADGE", 33),
        ("EVENT_MN_HEADWATERS_REPORT", 2218),
        ("EVENT_MN_DEPOT_FOREMAN_DEFEATED", 2219),
        ("EVENT_MN_MINE_INSTRUMENT_RECOVERED", 2220),
        ("EVENT_MN_DEPOT_CLEARED", 2277),
        (TRAINER, 32),
    ] {
        ensure!(
            base.data.story_event_script_constants.global.get(name) == Some(&value),
            "R09 native constant differs: {name}"
        );
    }
    Ok(ordinary)
}

fn section<T: serde::Serialize>(commands: &[T]) -> Result<Value> {
    let encoded = serde_json::to_value(commands)?;
    let values = encoded.as_array().context("typed map section")?;
    Ok(Value::Array(
        values
            .iter()
            .map(|v| {
                serde_json::json!({
                    "command":v["command"], "args":v["args"],
                })
            })
            .collect(),
    ))
}

fn certify_stale_split(
    base: &CompiledGamePack,
    module: &MapModule,
    ordinary: &MapModule,
    scripts_label: &str,
    events_label: &str,
) -> Result<()> {
    ensure!(
        base.data.map_scripts.get(scripts_label)
            == Some(&section(&ordinary.map_script_section_commands)?)
            && base.data.map_scripts.get(events_label)
                == Some(&section(&ordinary.map_event_section_commands)?)
            && base.data.npcs.get(MINE) == Some(&serde_json::to_value(&ordinary.objects)?),
        "R09 historical scout-only split preimage differs"
    );
    let mut missing = BTreeSet::new();
    for (label, body) in &module.scripts {
        if let Some(ordinary_body) = ordinary.scripts.get(label) {
            ensure!(
                body == ordinary_body && base.data.map_scripts.get(label) == Some(body),
                "R09 existing ordinary script mirror differs: {label}"
            );
        } else {
            ensure!(
                !base.data.map_scripts.contains_key(label),
                "R09 expected absent story root already exists: {label}"
            );
            ensure!(
                base.data.maps.iter().all(|(name, m)| name == MINE
                    || (!m.scripts.contains_key(label)
                        && !m.script_text_bodies.contains_key(label))),
                "R09 story root has another owner: {label}"
            );
            missing.insert(label);
        }
    }
    ensure!(
        missing.len() == 19 && ordinary.scripts.len() == 2,
        "R09 historical script inventory differs"
    );
    // Reassemble the complete *old* cached module using a repaired validation
    // clone, so every typed index must already match before terrain is touched.
    let mut control = base.clone();
    crate::insert_compiled_map_split_payloads(
        &mut control.data.map_scripts,
        &mut control.data.npcs,
        MINE,
        scripts_label,
        events_label,
        module,
    )?;
    certify_uncached(&control, module, scripts_label, events_label)
}

fn certify_uncached(
    pack: &CompiledGamePack,
    module: &MapModule,
    scripts_label: &str,
    events_label: &str,
) -> Result<()> {
    let mut data = pack.data.clone();
    data.maps
        .remove(MINE)
        .context("remove cached mine for actual split certification")?;
    let assembled = data.assemble_map_module_from_compiled_payloads(MINE)?;
    let mut expected = module.clone();
    for label in [scripts_label, events_label] {
        ensure!(
            !expected.scripts.contains_key(label),
            "R09 cache unexpectedly owns transport root"
        );
        expected
            .scripts
            .insert(label.into(), pack.data.map_scripts[label].clone());
    }
    ensure!(
        assembled == expected,
        "R09 actual uncached typed/program preimage differs"
    );
    Ok(())
}

fn validate_approaches(base: &CompiledGamePack, module: &MapModule, source: &Value) -> Result<()> {
    for field in ["story_trainers", "story_objects"] {
        for actor in source[field].as_array().context("Q06 story actors")? {
            let body: [u16; 2] = serde_json::from_value(actor["tile"].clone())?;
            let approach: [u16; 2] = serde_json::from_value(actor["approach"].clone())?;
            ensure!(
                body[0].abs_diff(approach[0]) + body[1].abs_diff(approach[1]) == 1
                    && collision(base, &module.blocks, module.attributes.width, body, "cave")?
                        == "FLOOR"
                    && collision(
                        base,
                        &module.blocks,
                        module.attributes.width,
                        approach,
                        "cave"
                    )? == "FLOOR"
                    && !module.objects.iter().any(|o| [o.x, o.y] == approach),
                "R09 required story interaction approach is blocked"
            );
        }
    }
    Ok(())
}
