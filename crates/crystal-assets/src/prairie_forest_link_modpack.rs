//! Native prairie/oak route, acknowledged SQ01 observations and Headbutt tables.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::world::encounters::{FieldEncounterData, FieldEncounterKind};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Saved only after the corresponding pine, wetland or prairie sign is read.
pub const LAKE_OBSERVATION_EVENTS: [&str; 3] = [
    "EVENT_MN_LAKE_OBSERVATION_01",
    "EVENT_MN_LAKE_OBSERVATION_02",
    "EVENT_MN_LAKE_OBSERVATION_03",
];

const SIGNS: [&str; 3] = [
    "MN_R19_OBSERVE_PINE",
    "MN_R19_OBSERVE_WETLAND",
    "MN_R19_OBSERVE_PRAIRIE",
];
const HEADBUTT_MAP: &str = "MnR19C00R00";
const REWARDED_EVENT: &str = "EVENT_MN_LAKE_OBSERVATIONS_REWARDED";

/// Compiles the two clipped Prairie–Forest Link maps and local observations.
///
/// Uses unchanged Johto art and the existing native script/field-move engine.
/// The three observation signs retain ordinary text and set separate saved
/// events after acknowledgment. Only the route's northern map receives a copy
/// of Route29's native Headbutt table, with levels set to 28 and unchanged species,
/// weights, rare/common buckets and time-based sleep rules. Source tables,
/// unrelated maps/catalogs, audio, HOME and recovery remain unchanged.
///
/// Detroit Lakes and its ranger hand-in are not authored here. Reading all
/// signs does not deliver a reward or set SQ01 completion.
///
/// # Errors
/// Returns an error for invalid geometry/events, changed observation flow,
/// missing native Headbutt support, event/table collisions or invalid native
/// lowering. Neither input authoring file nor the base pack is modified.
pub fn build_prairie_forest_link_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R19")
}

#[derive(Deserialize)]
struct Input {
    quest_contract: Quest,
    headbutt_contract: Headbutt,
    ramp_loop: RampLoop,
}

#[derive(Deserialize)]
struct Quest {
    id: String,
    completion_flag: String,
    reward_map: String,
    reward_actor_id: String,
    reward_item: String,
    quantity: u16,
    observations: Vec<Observation>,
    script_labels: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct Observation {
    sign_id: String,
    flag: String,
}

#[derive(Deserialize)]
struct RampLoop {
    id: String,
    ramps_metatiles: Vec<[u16; 2]>,
    clear_upper_core: Vec<[u16; 2]>,
    walking_lower_core: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
struct Headbutt {
    map_ids: Vec<String>,
    source_table_map: String,
    source_table_kind: String,
    level_override: u8,
    preserve_native_weights_rare_tables_and_time_sleep_rules: bool,
    party_move: String,
}

/// Binds only the three existing background roots before native publication.
///
/// Called after ordinary route events/constants exist. Allocates three fresh
/// event IDs from the existing EVENT namespace, installs parsed native commands
/// and adds one typed Headbutt table. It never publishes the unauthored Detroit
/// ranger or installs its draft reward scripts/completion flag.
///
/// # Errors
/// Returns an error for altered quest/table contracts, missing/ambiguous native
/// sign roots, namespace collisions, missing source data or parser failure.
pub(crate) fn bind_prairie_forest_link(
    pack: &mut CompiledGamePack,
    modules: &mut [MapModule],
    dossier_json: &str,
) -> Result<()> {
    let input: Input = serde_json::from_str(dossier_json)?;
    let ramps = input.ramp_loop;
    ensure!(
        ramps.id == "WESTERN_TWO_RAMP_LOOP"
            && ramps.ramps_metatiles == [[5, 18], [9, 18]]
            && ramps.clear_upper_core == [[11, 35], [19, 35]]
            && ramps.walking_lower_core == [[11, 38], [19, 38]],
        "retain both ordinary western ramp approaches"
    );
    let northern = modules
        .iter()
        .find(|module| module.id == HEADBUTT_MAP)
        .context("R19 northern ramp map")?;
    for [x, y] in ramps.ramps_metatiles {
        let index = usize::from(y) * usize::from(northern.attributes.width) + usize::from(x);
        ensure!(
            northern.blocks.get(index) == Some(&0x74),
            "R19 western spur requires both native ordinary walking ramps"
        );
    }
    let quest = input.quest_contract;
    ensure!(
        quest.id == "SQ01"
            && quest.completion_flag == REWARDED_EVENT
            && quest.reward_map == "MnDetroitLakes"
            && quest.reward_actor_id == "MN_DETROIT_LAKE_OBSERVATION_RANGER"
            && quest.reward_item == "MAX_REPEL"
            && quest.quantity == 1
            && quest.observations.len() == SIGNS.len(),
        "retain SQ01 observations and the pending Detroit Lakes hand-in"
    );
    let expected = observation_scripts();
    let mut sign_modules = Vec::new();
    for ((observation, sign), event) in quest
        .observations
        .iter()
        .zip(SIGNS)
        .zip(LAKE_OBSERVATION_EVENTS)
    {
        ensure!(
            observation.sign_id == sign && observation.flag == event,
            "SQ01 observation sign/event pairing changed"
        );
        ensure!(
            quest.script_labels.get(sign) == expected.get(sign),
            "SQ01 observation {sign} must set its event only after ordinary text acknowledgment"
        );
        let found: Vec<_> = modules
            .iter()
            .enumerate()
            .filter(|(_, module)| {
                module
                    .events
                    .bg_events
                    .iter()
                    .any(|event| event.script == sign)
            })
            .map(|(index, _)| index)
            .collect();
        ensure!(found.len() == 1, "SQ01 sign {sign} is absent or ambiguous");
        let index = found[0];
        let module = &modules[index];
        ensure!(
            module.id.starts_with("MnR19C")
                && module
                    .events
                    .bg_events
                    .iter()
                    .filter(|event| event.script == sign)
                    .count()
                    == 1
                && module.scripts.get(sign)
                    == Some(&json!([authoring::command(
                        "jumptext",
                        &[&format!("{sign}Text")]
                    )]))
                && module.scripts.contains_key(&format!("{sign}Text")),
            "SQ01 may replace only the existing ordinary background root {sign}"
        );
        sign_modules.push(index);
    }
    let binding = input.headbutt_contract;
    ensure!(
        binding.map_ids == [HEADBUTT_MAP]
            && binding.source_table_map == "Route29"
            && binding.source_table_kind == "headbutt"
            && binding.level_override == 28
            && binding.preserve_native_weights_rare_tables_and_time_sleep_rules
            && binding.party_move == "HEADBUTT",
        "retain the source Route29 Headbutt table with only levels changed to 28"
    );
    ensure!(
        modules.iter().any(|module| module.id == HEADBUTT_MAP)
            && !pack.data.field_encounters.contains_key(HEADBUTT_MAP)
            && pack.data.field_moves.headbutt.move_id == "HEADBUTT"
            && pack.data.moves.contains_key("HEADBUTT"),
        "SQ01 Headbutt requires its native map/move and an unused per-map table"
    );
    let mut table = pack
        .data
        .require_field_encounters_for_map("Route29")?
        .table(FieldEncounterKind::Headbutt)
        .context("native Route29 Headbutt table")?
        .clone();
    ensure!(
        !table.common.is_empty() && !table.rare.is_empty(),
        "native common/rare Headbutt buckets required"
    );
    for entry in table.common.iter_mut().chain(&mut table.rare) {
        ensure!(
            pack.data.pokemon.contains_key(&entry.species),
            "unavailable source Headbutt species {}",
            entry.species
        );
        entry.level = binding.level_override;
    }
    retain_headbutt_roots(pack)?;
    let globals = &mut pack.data.story_event_script_constants.global;
    for event in LAKE_OBSERVATION_EVENTS {
        ensure!(
            !globals.contains_key(event),
            "SQ01 observation event collision {event}"
        );
    }
    let next = globals
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("SQ01 event identifier overflow")?;
    for (index, event) in LAKE_OBSERVATION_EVENTS.into_iter().enumerate() {
        let value = next
            .checked_add(i64::try_from(index)?)
            .context("SQ01 event identifier overflow")?;
        ensure!(
            !globals
                .iter()
                .any(|(name, old)| name.starts_with("EVENT_") && *old == value),
            "SQ01 numeric event collision"
        );
        globals.insert(event.into(), value);
    }
    pack.data.field_encounters.insert(
        HEADBUTT_MAP.into(),
        FieldEncounterData::for_crystal(HEADBUTT_MAP, Some(table), None),
    );
    for ((sign, index), commands) in SIGNS
        .into_iter()
        .zip(sign_modules)
        .zip(SIGNS.map(|id| expected[id].clone()))
    {
        modules[index].scripts.insert(sign.into(), commands);
    }
    for module in modules {
        authoring::finish(pack, module)?;
        module.script_runtime_commands =
            crate::parse_script_runtime_commands(&module.id, &module.scripts)?;
        module.script_object_commands =
            crate::parse_script_object_commands(&module.id, &module.scripts)?;
    }
    Ok(())
}

fn observation_scripts() -> BTreeMap<String, Value> {
    SIGNS
        .into_iter()
        .zip(LAKE_OBSERVATION_EVENTS)
        .map(|(sign, event)| {
            (
                sign.into(),
                json!([
                    authoring::command("opentext", &[]),
                    authoring::command("writetext", &[&format!("{sign}Text")]),
                    authoring::command("waitbutton", &[]),
                    authoring::command("closetext", &[]),
                    authoring::command("setevent", &[event]),
                    authoring::command("end", &[])
                ]),
            )
        })
        .collect()
}

/// Retain native direct-entry field scripts through every pack reload.
fn retain_headbutt_roots(pack: &mut CompiledGamePack) -> Result<()> {
    const ROOTS: [&str; 2] = ["HeadbuttFromMenuScript", "HeadbuttScript"];
    let globals = pack
        .data
        .global_scripts
        .as_ref()
        .context("native global definitions")?;
    ensure!(
        ROOTS
            .iter()
            .all(|root| globals.definitions.contains_key(*root)),
        "native Headbutt roots missing from source definitions"
    );
    let catalogs: Vec<_> = pack
        .data
        .story_events
        .iter()
        .enumerate()
        .filter(|(_, payload)| payload.get("StandardScripts").is_some())
        .map(|(index, _)| index)
        .collect();
    ensure!(
        catalogs.len() == 1,
        "native StandardScripts catalog must be unique"
    );
    let roots = pack.data.story_events[catalogs[0]]["StandardScripts"]["GlobalScriptRoots"]
        .as_array_mut()
        .context("native GlobalScriptRoots list")?;
    for root in ROOTS {
        if !roots.iter().any(|entry| entry.as_str() == Some(root)) {
            roots.push(json!(root));
        }
    }
    // Extend the existing retained-root declaration, then use the production
    // subset walker/parsers. Do not clone source bodies into a new catalog or
    // inject command tables that would disappear on the next pack reload.
    pack.data.materialize_global_scripts()?;
    ensure!(
        pack.data
            .is_exact_tree_mon_encounter_command(&crate::RuntimeScriptCommandRef::new(
                HEADBUTT_MAP,
                "HeadbuttScript",
                4
            ))?,
        "native Headbutt command/certificate was not retained"
    );
    Ok(())
}
