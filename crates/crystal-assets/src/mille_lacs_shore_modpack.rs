//! Native Mille Lacs shore publication and the three-report fishing-rod quest.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

/// Saved report events, in reed, meadow and beach order.
pub const SHORE_WATCH_REPORT_EVENTS: [&str; 3] = [
    "EVENT_MN_SHORE_WATCH_01",
    "EVENT_MN_SHORE_WATCH_02",
    "EVENT_MN_SHORE_WATCH_03",
];
/// Saved only after actual Good Rod delivery or adequate-rod acknowledgment.
pub const SHORE_WATCH_REWARDED_EVENT: &str = "EVENT_MN_SHORE_WATCH_REWARDED";

const WATCHERS: [&str; 3] = [
    "MN_R18_WATCH_REED",
    "MN_R18_WATCH_MEADOW",
    "MN_R18_WATCH_BEACH",
];
const RANGER: &str = "MN_R18_DOCK_RANGER";
const TEXT_LABELS: [&str; 7] = [
    "MN_R18_WatchBadgeDeniedText",
    "MN_R18_WatchAlreadyRewardedText",
    "MN_R18_WatchNeedsNewsText",
    "MN_R18_WatchRewardIntroText",
    "MN_R18_WatchRewardGivenText",
    "MN_R18_WatchHasRodText",
    "MN_R18_WatchBagFullText",
];

/// Compiles the two shore maps and binds SQ04 to the four existing volunteers.
///
/// The native script engine owns report events, badge checks, item delivery,
/// bag-full retry and saved completion. The optional Surf-islet Revive is
/// independent of the walking quest. Existing maps, art, items, audio, HOME and
/// recovery destinations remain unchanged.
///
/// # Errors
/// Returns an error for invalid geometry, missing source art, conflicting map,
/// actor, event or script identities, altered quest control flow, unavailable
/// native rods, or failed native compilation/validation.
pub fn build_mille_lacs_shore_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> Result<CompiledGamePack> {
    crate::authored_route_modpack::build_authored_route(base, layout_json, dossier_json, "MN_R18")
}

#[derive(Deserialize)]
struct QuestInput {
    quest_contract: WatchQuest,
}

#[derive(Deserialize)]
struct WatchQuest {
    id: String,
    required_flags: Vec<String>,
    volunteers: Vec<WatchVolunteer>,
    ranger_id: String,
    reward_item: String,
    quantity: u16,
    reward_flag: String,
    script_labels: BTreeMap<String, Value>,
    text_labels: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct WatchVolunteer {
    actor_id: String,
    flag: String,
}

/// Called by the shared route compiler after ordinary event allocation and
/// before native module lowering. No published source module is modified.
pub(crate) fn bind_shore_watch_quest(
    pack: &mut CompiledGamePack,
    modules: &mut [MapModule],
    dossier_json: &str,
) -> Result<()> {
    let input: QuestInput = serde_json::from_str(dossier_json)?;
    let quest = input.quest_contract;
    ensure!(
        quest.id == "SQ04"
            && quest.required_flags == ["BADGE_6"]
            && quest.ranger_id == RANGER
            && quest.reward_item == "GOOD_ROD"
            && quest.quantity == 1
            && quest.reward_flag == SHORE_WATCH_REWARDED_EVENT
            && quest.volunteers.len() == WATCHERS.len(),
        "retain SQ04's three-report, sixth-badge and Good Rod contract"
    );
    for ((volunteer, actor), event) in quest
        .volunteers
        .iter()
        .zip(WATCHERS)
        .zip(SHORE_WATCH_REPORT_EVENTS)
    {
        ensure!(
            volunteer.actor_id == actor && volunteer.flag == event,
            "SQ04 report actor/event pairing changed"
        );
    }
    ensure!(
        quest.script_labels == expected_scripts(),
        "SQ04 native control flow differs from the reviewed retry/idempotence contract"
    );
    ensure!(
        quest.text_labels.len() == TEXT_LABELS.len()
            && TEXT_LABELS
                .iter()
                .all(|label| quest.text_labels.contains_key(*label))
            && quest.text_labels.values().all(|lines| {
                !lines.is_empty()
                    && lines
                        .iter()
                        .all(|line| !line.is_empty() && line.chars().count() <= 18)
            }),
        "SQ04 text labels must exist and fit the native text box"
    );
    let globals = &pack.data.story_event_script_constants.global;
    ensure!(
        globals.get("ENGINE_MINERALBADGE") == Some(&31),
        "SQ04 requires the supplied native sixth Johto badge flag"
    );
    for rod in ["OLD_ROD", "GOOD_ROD", "SUPER_ROD"] {
        let item = pack
            .data
            .items
            .get(rod)
            .context("native fishing rod missing")?;
        ensure!(
            item.script_name == rod
                && item.pocket == "KEY_ITEM"
                && item.field_usable
                && item.field_menu == "ITEMMENU_CLOSE"
                && !item.consumable,
            "SQ04 requires unchanged native key-pocket fishing rods"
        );
    }
    let mut actor_modules = BTreeMap::new();
    for id in WATCHERS.into_iter().chain([RANGER]) {
        let matches = modules
            .iter()
            .enumerate()
            .filter(|(_, module)| module.objects.iter().any(|actor| actor.script == id))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        ensure!(matches.len() == 1, "SQ04 actor {id} is absent or ambiguous");
        let index = matches[0];
        let module = &modules[index];
        ensure!(
            module.id.starts_with("MnR18C")
                && module
                    .objects
                    .iter()
                    .filter(|actor| actor.script == id)
                    .count()
                    == 1
                && module.scripts.get(id)
                    == Some(&json!([authoring::command(
                        "jumptextfaceplayer",
                        &[&format!("{id}Text")]
                    )]))
                && module.scripts.contains_key(&format!("{id}Text")),
            "SQ04 may replace only its existing ordinary resident root {id}"
        );
        actor_modules.insert(id, index);
    }
    let ranger_index = actor_modules[RANGER];
    for label in quest.script_labels.keys().chain(quest.text_labels.keys()) {
        if WATCHERS.contains(&label.as_str()) || label == RANGER {
            continue;
        }
        ensure!(
            !modules[ranger_index].scripts.contains_key(label),
            "SQ04 script namespace collision {label}"
        );
    }
    let globals = &mut pack.data.story_event_script_constants.global;
    let events = SHORE_WATCH_REPORT_EVENTS
        .into_iter()
        .chain([SHORE_WATCH_REWARDED_EVENT]);
    for event in events.clone() {
        ensure!(!globals.contains_key(event), "SQ04 event collision {event}");
    }
    let next = globals
        .iter()
        .filter(|(name, _)| name.starts_with("EVENT_"))
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .context("SQ04 event identifier overflow")?;
    for (index, event) in events.enumerate() {
        let value = next
            .checked_add(i64::try_from(index)?)
            .context("SQ04 event identifier overflow")?;
        ensure!(
            !globals
                .iter()
                .any(|(name, old)| name.starts_with("EVENT_") && *old == value),
            "SQ04 numeric event collision"
        );
        globals.insert(event.into(), value);
    }
    for (label, commands) in quest.script_labels {
        let index = actor_modules
            .get(label.as_str())
            .copied()
            .unwrap_or(ranger_index);
        modules[index].scripts.insert(label, commands);
    }
    for (label, lines) in quest.text_labels {
        authoring::text(
            &mut modules[ranger_index],
            &label,
            &lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
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

fn expected_scripts() -> BTreeMap<String, Value> {
    let mut scripts = BTreeMap::new();
    for (actor, event) in WATCHERS.into_iter().zip(SHORE_WATCH_REPORT_EVENTS) {
        scripts.insert(
            actor.into(),
            json!([
                authoring::command("faceplayer", &[]),
                authoring::command("opentext", &[]),
                authoring::command("writetext", &[&format!("{actor}Text")]),
                authoring::command("waitbutton", &[]),
                authoring::command("closetext", &[]),
                authoring::command("setevent", &[event]),
                authoring::command("end", &[])
            ]),
        );
    }
    scripts.insert(
        RANGER.into(),
        json!([
            authoring::command("faceplayer", &[]),
            authoring::command("checkflag", &["ENGINE_MINERALBADGE"]),
            authoring::command("iffalse", &["MN_R18_WatchBadgeDenied"]),
            authoring::command("checkevent", &[SHORE_WATCH_REWARDED_EVENT]),
            authoring::command("iftrue", &["MN_R18_WatchAlreadyRewarded"]),
            authoring::command("checkevent", &[SHORE_WATCH_REPORT_EVENTS[0]]),
            authoring::command("iffalse", &["MN_R18_WatchNeedsNews"]),
            authoring::command("checkevent", &[SHORE_WATCH_REPORT_EVENTS[1]]),
            authoring::command("iffalse", &["MN_R18_WatchNeedsNews"]),
            authoring::command("checkevent", &[SHORE_WATCH_REPORT_EVENTS[2]]),
            authoring::command("iffalse", &["MN_R18_WatchNeedsNews"]),
            authoring::command("checkitem", &["SUPER_ROD"]),
            authoring::command("iftrue", &["MN_R18_WatchHasRod"]),
            authoring::command("checkitem", &["GOOD_ROD"]),
            authoring::command("iftrue", &["MN_R18_WatchHasRod"]),
            authoring::command("opentext", &[]),
            authoring::command("writetext", &["MN_R18_WatchRewardIntroText"]),
            authoring::command("promptbutton", &[]),
            authoring::command("giveitem", &["GOOD_ROD", "1"]),
            authoring::command("iffalse", &["MN_R18_WatchBagFull"]),
            authoring::command("setevent", &[SHORE_WATCH_REWARDED_EVENT]),
            authoring::command("writetext", &["MN_R18_WatchRewardGivenText"]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    for (label, text, event) in [
        ("MN_R18_WatchBadgeDenied", TEXT_LABELS[0], None),
        ("MN_R18_WatchAlreadyRewarded", TEXT_LABELS[1], None),
        ("MN_R18_WatchNeedsNews", TEXT_LABELS[2], None),
        (
            "MN_R18_WatchHasRod",
            TEXT_LABELS[5],
            Some(SHORE_WATCH_REWARDED_EVENT),
        ),
    ] {
        let mut commands = vec![
            authoring::command("opentext", &[]),
            authoring::command("writetext", &[text]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
        ];
        if let Some(event) = event {
            commands.push(authoring::command("setevent", &[event]));
        }
        commands.push(authoring::command("end", &[]));
        scripts.insert(label.into(), Value::Array(commands));
    }
    scripts.insert(
        "MN_R18_WatchBagFull".into(),
        json!([
            authoring::command("writetext", &[TEXT_LABELS[6]]),
            authoring::command("waitbutton", &[]),
            authoring::command("closetext", &[]),
            authoring::command("end", &[])
        ]),
    );
    scripts
}
