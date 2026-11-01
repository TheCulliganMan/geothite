//! Badge-paced field-machine gifts and visible directions through Minnesota.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

const MANIFEST: &str = "minnesota-hm-progression-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Handoff {
    item: String,
    rule: String,
    badge_index: usize,
    badge: String,
    flag: String,
    map: String,
    actor: String,
    gym: String,
    directions: Vec<String>,
}

/// Places all seven reusable HMs at named regional workers, distributed at
/// their native field-use badges. Gym guides and leaders explain where to go.
/// Gifts are free, once per owned machine, and retryable; the production field
/// permission and compatibility tables remain unchanged. Existing Brainerd
/// testimony still requires its own badge and acknowledged report.
///
/// # Errors
/// Rejects a duplicate stage, missing actors, mismatched native badge rules,
/// missing/non-reusable machines, malformed scripts or an invalid pack.
pub fn build_minnesota_hm_progression(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|m| m == MANIFEST),
        "HM progression already installed"
    );
    let handoffs: Vec<Handoff> = serde_json::from_str(include_str!(
        "../../../planning/minnesota/hm-progression.json"
    ))?;
    ensure!(handoffs.len() == 7, "seven regional HMs required");
    let fields = serde_json::to_value(&base.data.field_moves)?;
    let mut pack = base.clone();
    for h in handoffs {
        ensure!(
            fields[&h.rule]["badge"]["region"] == "johto"
                && fields[&h.rule]["badge"]["index"].as_u64()
                    == Some(u64::try_from(h.badge_index)?),
            "HM source disagrees with native badge permission: {}",
            h.item
        );
        let item = pack.data.items.get(&h.item).context("native HM")?;
        ensure!(
            !item.consumable && item.tmhm_index.is_some_and(|i| (51..=57).contains(&i)),
            "native reusable HM required"
        );
        ensure!(
            pack.data
                .story_event_script_constants
                .global
                .contains_key(&h.flag),
            "native field badge flag"
        );
        let mut map = pack.data.maps.get(&h.map).context("HM giver map")?.clone();
        let actor = map
            .objects
            .iter_mut()
            .find(|o| o.script == h.actor)
            .context("HM giver")?;
        let root = format!("{}FieldHM", h.actor);
        actor.script = root.clone();
        let denied = format!("{root}Denied");
        let full = format!("{root}Full");
        let done = format!("{root}Done");
        let given_text = format!("{root}GivenText");
        let denied_text = format!("{root}DeniedText");
        let full_text = format!("{root}FullText");
        let c = authoring::command;
        let mut programs = BTreeMap::from([
            (
                root.clone(),
                serde_json::json!([
                    c("faceplayer", &[]),
                    c("checkitem", &[&h.item]),
                    c("iftrue", &[&done]),
                    c("checkflag", &[&h.flag]),
                    c("iffalse", &[&denied]),
                    c("opentext", &[]),
                    c("verbosegiveitem", &[&h.item, "1"]),
                    c("iffalse", &[&full]),
                    c("writetext", &[&given_text]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("sjump", &[&done]),
                ]),
            ),
            (done.clone(), serde_json::json!([c("sjump", &[&h.actor])])),
            (
                denied.clone(),
                serde_json::json!([
                    c("opentext", &[]),
                    c("writetext", &[&denied_text]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("sjump", &[&done]),
                ]),
            ),
            (
                full,
                serde_json::json!([
                    c("writetext", &[&full_text]),
                    c("waitbutton", &[]),
                    c("closetext", &[]),
                    c("sjump", &[&done]),
                ]),
            ),
        ]);
        for label in programs.keys() {
            ensure!(!map.scripts.contains_key(label), "occupied HM script");
        }
        let move_name = h.item.strip_prefix("HM_").context("HM name")?;
        authoring::text(
            &mut map,
            &given_text,
            &[
                &format!("Take HM {move_name}!"),
                "Open PACK and",
                "choose TM/HM to",
                "teach a partner.",
                "HMs can be reused.",
                &format!("Your {} BADGE", h.badge),
                &format!("allows {move_name}"),
                "outside battle.",
            ],
        );
        authoring::text(
            &mut map,
            &denied_text,
            &[
                &format!("For HM {move_name},"),
                &format!("Earn the {}", h.badge),
                "BADGE, then return",
                "and it is yours.",
            ],
        );
        authoring::text(
            &mut map,
            &full_text,
            &[
                "Your TM/HM pocket",
                "cannot hold it.",
                "Make room and",
                "ask me again.",
            ],
        );
        for label in [&given_text, &denied_text, &full_text] {
            let body = map.scripts.remove(label).context("new HM text")?;
            for command in body.as_array().context("HM text commands")? {
                if let Some(line) = command["args"][0].as_str() {
                    ensure!(
                        line.trim_matches('"').chars().count() <= 18,
                        "HM text exceeds native row: {label}"
                    );
                }
            }
            programs.insert(label.clone(), body);
        }
        append(&mut map, programs)?;
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(&mut pack, map)?;
        gym_directions(&mut pack, &h)?;
    }
    retain_brainerd_report(&mut pack)?;
    crate::minnesota_fly_destinations::install(&mut pack)?;
    // A bulk shop must not make later field machines available at badge three.
    for (id, stock) in &mut pack.data.marts.0 {
        if id.starts_with("MART_MN_") {
            stock.retain(|item| !item.starts_with("HM_"));
        }
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

fn append(map: &mut crate::MapModule, programs: BTreeMap<String, Value>) -> Result<()> {
    let mut linked = map.scripts.clone();
    linked.extend(programs.clone());
    map.script_control_commands.extend(
        crate::parse_script_control_commands(&map.id, &linked)?
            .into_iter()
            .filter(|command| programs.contains_key(&command.source_script)),
    );
    map.script_flag_commands
        .extend(crate::parse_script_flag_commands(&map.id, &programs)?);
    map.script_text_commands
        .extend(crate::parse_script_text_commands(&map.id, &programs)?);
    map.script_text_bodies
        .extend(crate::parse_script_text_bodies(&map.id, &programs)?);
    map.script_object_commands
        .extend(crate::parse_script_object_commands(&map.id, &programs)?);
    map.script_item_grants
        .extend(crate::parse_script_item_grants(&map.id, &programs)?);
    let (checks, takes) = crate::parse_script_item_accesses(&map.id, &programs)?;
    map.script_item_checks.extend(checks);
    map.script_item_takes.extend(takes);
    map.scripts.extend(programs);
    Ok(())
}

fn gym_directions(pack: &mut CompiledGamePack, h: &Handoff) -> Result<()> {
    let mut gym = pack
        .data
        .maps
        .get(&h.gym)
        .context("HM direction gym")?
        .clone();
    let labels = gym
        .scripts
        .keys()
        .filter(|label| {
            label.ends_with("GuideText")
                || label.ends_with("LeaderWonText")
                || label.ends_with("LeaderGiftText")
                || label.ends_with("LeaderAfterText")
        })
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        labels.len() >= 3,
        "gym guide and victory directions required"
    );
    let mut lines = h.directions.iter().map(String::as_str).collect::<Vec<_>>();
    let badge = format!("The {} BADGE", h.badge);
    lines.extend([badge.as_str(), "allows field use."]);
    let temp = format!("{}HMDirections", h.gym);
    authoring::text(&mut gym, &temp, &lines);
    let mut tail = gym
        .scripts
        .remove(&temp)
        .context("wrapped HM directions")?
        .as_array()
        .context("direction rows")?
        .clone();
    tail.pop();
    tail.first_mut().context("first direction")?["command"] = Value::String("para".into());
    for label in labels {
        let body = gym
            .scripts
            .get_mut(&label)
            .and_then(Value::as_array_mut)
            .context("gym text")?;
        let end = body.pop().context("gym text end")?;
        ensure!(end["command"] == "done", "native gym text terminator");
        body.extend(tail.clone());
        body.push(end);
    }
    gym.script_text_commands = crate::parse_script_text_commands(&gym.id, &gym.scripts)?;
    gym.script_text_bodies = crate::parse_script_text_bodies(&gym.id, &gym.scripts)?;
    crate::authored_town_publication::replace(pack, gym)
}

fn retain_brainerd_report(pack: &mut CompiledGamePack) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get("MnBrainerd")
        .context("Brainerd report")?
        .clone();
    let script = map
        .scripts
        .get_mut("MN_BRAINERD_RANGER")
        .and_then(Value::as_array_mut)
        .context("report program")?;
    let gift = script
        .iter()
        .position(|command| command["command"] == "giveitem" && command["args"][0] == "HM_SURF")
        .context("existing Surf report gift")?;
    let c = authoring::command;
    let owned = "MnBrainerdRangerSurfAlreadyOwned";
    script.splice(
        gift..gift,
        [c("checkitem", &["HM_SURF"]), c("iftrue", &[owned])],
    );
    map.scripts.insert(
        owned.into(),
        serde_json::json!([
            c("setevent", &["EVENT_MN_BRAINERD_REPORT"]),
            c("setevent", &["EVENT_MN_SURF_GIFT_RECEIVED"]),
            c("sjump", &["MnBrainerdRangerRepeat"]),
        ]),
    );
    // The report remains behind its existing Lake Badge and acknowledged text.
    // Earlier Surf ownership skips only the duplicate item, never the report.
    authoring::finish(pack, &mut map)?;
    map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
    map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
    crate::authored_town_publication::replace(pack, map)
}
