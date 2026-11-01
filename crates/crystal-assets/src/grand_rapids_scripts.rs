//! Native service scripts and source-driven Q06 evidence custody.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::{Value, json};

use super::{Actor, DEPOT, Town};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(super) const DEFEATED: &str = "EVENT_MN_DEPOT_FOREMAN_DEFEATED";
pub(super) const RECOVERED: &str = "EVENT_MN_MINE_INSTRUMENT_RECOVERED";
pub(super) const CLEARED: &str = "EVENT_MN_DEPOT_CLEARED";
const MART: &str = "MART_MN_GRAND_RAPIDS";
const RECIPIENT: &str = "MN_DEPOT_RECIPIENT";
type Texts = BTreeMap<String, Vec<String>>;

pub(super) fn prepare(pack: &mut CompiledGamePack, town: &Town) -> Result<()> {
    pack.data.require_special_routine("HealParty")?;
    pack.data.require_special_routine("PokemonCenterPC")?;
    for (room, id, service) in [
        ("MnGrandRapidsPokecenter1F", "MN_GR_NURSE", "heal_party"),
        ("MnGrandRapidsMart", "MN_GR_MART_CLERK", "pokemart"),
        (DEPOT, RECIPIENT, "evidence_handover"),
    ] {
        let actors = town
            .interiors
            .iter()
            .flat_map(|r| r.residents.iter().map(move |a| (r.id.as_str(), a)))
            .filter(|(_, a)| a.service.as_deref() == Some(service))
            .collect::<Vec<_>>();
        ensure!(
            actors.len() == 1 && actors[0].0 == room && actors[0].1.id == id,
            "Grand Rapids requires its unique native {service}"
        );
    }
    ensure!(
        town.interiors
            .iter()
            .filter(|r| r.pc.is_some())
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>()
            == ["MnGrandRapidsPokecenter1F"],
        "Grand Rapids requires its clinic PC"
    );
    let clerks = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .filter(|a| a.service.as_deref() == Some("pokemart"))
        .collect::<Vec<_>>();
    ensure!(
        clerks.len() == 1
            && clerks[0].stock
                == [
                    "ULTRA_BALL",
                    "HYPER_POTION",
                    "FULL_HEAL",
                    "REVIVE",
                    "MAX_REPEL"
                ],
        "retain Grand Rapids native shop stock"
    );
    ensure!(
        !pack.data.marts.0.contains_key(MART),
        "Grand Rapids mart already installed"
    );
    ensure!(
        clerks[0]
            .stock
            .iter()
            .all(|item| pack.data.items.contains_key(item)),
        "Grand Rapids unknown item"
    );
    pack.data
        .marts
        .0
        .insert(MART.into(), clerks[0].stock.clone());
    for name in [DEFEATED, RECOVERED, CLEARED] {
        if let Some(id) = pack.data.story_event_script_constants.global.get(name) {
            ensure!(
                !pack
                    .data
                    .story_event_script_constants
                    .global
                    .iter()
                    .any(|(k, v)| k != name && k.starts_with("EVENT_") && v == id),
                "Grand Rapids event numeric identity collision"
            );
        } else {
            let next = pack
                .data
                .story_event_script_constants
                .global
                .iter()
                .filter(|(k, _)| k.starts_with("EVENT_"))
                .map(|(_, v)| *v)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .context("Grand Rapids event identity overflow")?;
            pack.data
                .story_event_script_constants
                .global
                .insert(name.into(), next);
        }
    }
    Ok(())
}

pub(super) fn story(source: &str) -> Result<Texts> {
    let story: Value = serde_json::from_str(source).context("Grand Rapids canonical story")?;
    let milestones = story["milestones"]
        .as_array()
        .context("Grand Rapids milestones")?
        .iter()
        .filter(|m| m["id"] == "MS09")
        .collect::<Vec<_>>();
    ensure!(milestones.len() == 1, "Grand Rapids needs one MS09");
    let m = milestones[0];
    let requires: Vec<String> = serde_json::from_value(m["requires"].clone())?;
    let sets: Vec<String> = serde_json::from_value(m["sets"].clone())?;
    ensure!(
        m["location"] == DEPOT
            && requires.len() == 2
            && requires.iter().map(String::as_str).collect::<BTreeSet<_>>()
                == BTreeSet::from([DEFEATED, RECOVERED])
            && sets == [CLEARED],
        "Grand Rapids requires actual defeat AND separate recovery; it sets only depot clearance"
    );
    let flags: Vec<String> = serde_json::from_value(story["flags"].clone())?;
    ensure!(
        [DEFEATED, RECOVERED, CLEARED]
            .iter()
            .all(|f| flags.iter().any(|s| s == f)),
        "Grand Rapids story events must be declared"
    );
    let texts: Texts = serde_json::from_value(m["native_texts"].clone())
        .context("Grand Rapids native dialogue")?;
    let expected = ["Missing", "Offer", "Declined", "Read", "Done"]
        .map(|s| format!("MnGrandRapidsDepot{s}Text"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    ensure!(
        texts.keys().cloned().collect::<BTreeSet<_>>() == expected,
        "Grand Rapids exact five native labels required"
    );
    for (label, lines) in &texts {
        validate_text(label, lines)?;
    }
    Ok(texts)
}

fn validate_text(label: &str, lines: &[String]) -> Result<()> {
    ensure!(
        !lines.is_empty()
            && lines.iter().all(|s| !s.is_empty()
                && s.chars().count() <= 18
                && s.chars()
                    .all(|c| c.is_ascii() && !c.is_control() && c != '"')),
        "Grand Rapids text must fit native 18-column pages: {label}"
    );
    Ok(())
}

pub(super) fn install(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    actors: &[Actor],
    signs: &[crate::authored_route_design::Sign],
    texts: &Texts,
) -> Result<()> {
    let c = authoring::command;
    for actor in actors {
        ensure!(
            !map.scripts.contains_key(&actor.id),
            "Grand Rapids script collision"
        );
        let label = format!("{}Text", actor.id);
        if actor.service.as_deref() != Some("evidence_handover") {
            validate_text(&label, &actor.lines)?;
            authoring::text(
                map,
                &label,
                &actor.lines.iter().map(String::as_str).collect::<Vec<_>>(),
            );
        }
        let commands = match actor.service.as_deref() {
            None => json!([c("jumptextfaceplayer", &[&label])]),
            Some("heal_party") => {
                ensure!(
                    map.id == "MnGrandRapidsPokecenter1F" && actor.id == "MN_GR_NURSE",
                    "Grand Rapids nurse identity"
                );
                json!([
                    c("faceplayer", &[]),
                    c("opentext", &[]),
                    c("writetext", &[&label]),
                    c("waitbutton", &[]),
                    c("special", &["HealParty"]),
                    c("blackoutmod", &["MN_GRAND_RAPIDS_POKECENTER_1F"]),
                    c("closetext", &[]),
                    c("end", &[])
                ])
            }
            Some("pokemart") => {
                ensure!(
                    map.id == "MnGrandRapidsMart" && actor.id == "MN_GR_MART_CLERK",
                    "Grand Rapids clerk identity"
                );
                json!([
                    c("faceplayer", &[]),
                    c("opentext", &[]),
                    c("pokemart", &["MARTTYPE_STANDARD", MART]),
                    c("closetext", &[]),
                    c("end", &[])
                ])
            }
            Some("evidence_handover") => {
                ensure!(
                    map.id == DEPOT
                        && actor.id == RECIPIENT
                        && actor.tile == [11, 9]
                        && actor.approach == Some([11, 10])
                        && actor.sprite == "SPRITE_SCIENTIST",
                    "Grand Rapids reserved technician identity"
                );
                for (label, lines) in texts {
                    authoring::text(
                        map,
                        label,
                        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
                    );
                }
                for suffix in ["Missing", "Done"] {
                    let target = format!("MnGrandRapidsDepot{suffix}");
                    map.scripts.insert(
                        target.clone(),
                        json!([c("jumptextfaceplayer", &[&format!("{target}Text")])]),
                    );
                }
                map.scripts.insert(
                    "MnGrandRapidsDepotDeclined".into(),
                    json!([
                        c("writetext", &["MnGrandRapidsDepotDeclinedText"]),
                        c("waitbutton", &[]),
                        c("closetext", &[]),
                        c("end", &[])
                    ]),
                );
                json!([
                    c("faceplayer", &[]),
                    c("checkevent", &[CLEARED]),
                    c("iftrue", &["MnGrandRapidsDepotDone"]),
                    c("checkevent", &[DEFEATED]),
                    c("iffalse", &["MnGrandRapidsDepotMissing"]),
                    c("checkevent", &[RECOVERED]),
                    c("iffalse", &["MnGrandRapidsDepotMissing"]),
                    c("opentext", &[]),
                    c("writetext", &["MnGrandRapidsDepotOfferText"]),
                    c("yesorno", &[]),
                    c("iffalse", &["MnGrandRapidsDepotDeclined"]),
                    c("writetext", &["MnGrandRapidsDepotReadText"]),
                    c("waitbutton", &[]),
                    c("setevent", &[CLEARED]),
                    c("closetext", &[]),
                    c("end", &[])
                ])
            }
            Some(service) => anyhow::bail!("unsupported Grand Rapids service {service}"),
        };
        map.scripts.insert(actor.id.clone(), commands);
        ensure!(
            pack.runtime_files().contains_key(&format!(
                "gfx/sprites/{}.png",
                actor
                    .sprite
                    .trim_start_matches("SPRITE_")
                    .to_ascii_lowercase()
            )),
            "Grand Rapids original sprite missing"
        );
        authoring::npc(
            pack,
            map,
            &actor.id,
            &actor.sprite,
            i16::try_from(actor.tile[0])?,
            i16::try_from(actor.tile[1])?,
        )?;
        let object = map
            .objects
            .last_mut()
            .context("Grand Rapids inserted actor")?;
        object.object_identifier = Some(actor.id.clone());
        object.object_type = "OBJECTTYPE_SCRIPT".into();
        object.radius = 0;
        object.spritemovedata = actor.facing.movement().into();
    }
    for sign in signs {
        let label = format!("{}Text", sign.id);
        validate_text(&label, &sign.lines)?;
        ensure!(
            !map.scripts.contains_key(&sign.id),
            "Grand Rapids sign collision"
        );
        authoring::text(
            map,
            &label,
            &sign.lines.iter().map(String::as_str).collect::<Vec<_>>(),
        );
        map.scripts
            .insert(sign.id.clone(), json!([c("jumptext", &[&label])]));
        map.events.bg_events.push(BackgroundEvent {
            x: sign.tile[0],
            y: sign.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: sign.id.clone(),
        });
    }
    Ok(())
}

pub(super) fn finish(pack: &CompiledGamePack, map: &mut MapModule) -> Result<()> {
    authoring::finish(pack, map)?;
    map.script_runtime_commands = crate::parse_script_runtime_commands(&map.id, &map.scripts)?;
    map.script_object_commands = crate::parse_script_object_commands(&map.id, &map.scripts)?;
    map.script_variable_commands = crate::parse_script_variable_commands(&map.id, &map.scripts)?;
    Ok(())
}
