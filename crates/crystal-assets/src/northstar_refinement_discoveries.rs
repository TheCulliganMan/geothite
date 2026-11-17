//! Local clues and lasting rewards for the existing hidden discoveries.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "northstar-discovery-clues-v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Refinement {
    schema_version: u8,
    id: String,
    parent_content_hash: String,
    rewards: Vec<Reward>,
    hints: Vec<Hint>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reward {
    cache: String,
    item: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Hint {
    cache: String,
    actor: String,
    script: String,
}

#[derive(Deserialize)]
struct OriginalDiscoveries {
    caches: Vec<OriginalCache>,
}

#[derive(Deserialize)]
struct OriginalCache {
    id: String,
    map: String,
    tile: [u16; 2],
    item: String,
}

pub(crate) fn install(
    pack: &mut CompiledGamePack,
    story_source: &str,
    discovery_source: &str,
) -> Result<()> {
    let refinement: Refinement = serde_json::from_str(discovery_source)?;
    ensure!(
        refinement.schema_version == 1
            && refinement.id == MANIFEST
            && refinement.parent_content_hash
                == "d4ce7998673f393e69771a36c3575878060f121102f6db840ddf197c583979dd",
        "unsupported discovery refinement"
    );
    ensure!(
        !pack.report.manifests.iter().any(|id| id == MANIFEST),
        "discovery clues already installed"
    );
    let original: OriginalDiscoveries = serde_json::from_str(include_str!(
        "../../../planning/minnesota/discoveries/hidden-items-v01.json"
    ))?;
    let caches: BTreeMap<_, _> = original
        .caches
        .iter()
        .map(|cache| (cache.id.as_str(), cache))
        .collect();
    ensure!(caches.len() == 40, "original discovery identities changed");
    ensure!(
        refinement.rewards.len() == 6 && refinement.hints.len() == 6,
        "six selected rewards and local clues required"
    );
    let story: Value = serde_json::from_str(story_source)?;
    let dialogue: BTreeMap<String, Vec<AuthoredDialoguePage>> =
        serde_json::from_value(story["refinement_discoveries"]["dialogue"].clone())?;
    ensure!(
        dialogue.keys().map(String::as_str).collect::<BTreeSet<_>>()
            == refinement
                .hints
                .iter()
                .map(|hint| hint.cache.as_str())
                .collect(),
        "discovery clues must match their canonical pages exactly"
    );
    let mut changed = BTreeSet::new();
    for reward in &refinement.rewards {
        ensure!(changed.insert(&reward.cache), "duplicate cache reward");
        ensure!(
            matches!(
                reward.item.as_str(),
                "MYSTIC_WATER"
                    | "MIRACLE_SEED"
                    | "NEVERMELTICE"
                    | "BLACKGLASSES"
                    | "HARD_STONE"
                    | "SPELL_TAG"
            ) && pack.data.items.contains_key(&reward.item),
            "discovery reward must be an installed, optional held boost"
        );
        let cache = caches.get(reward.cache.as_str()).context("known cache")?;
        let mut map = pack.data.maps.get(&cache.map).context("cache map")?.clone();
        let label = format!("MN_HIDDEN_{}", cache.id);
        let flag = format!("EVENT_{label}");
        check_pickup(pack, &map, cache)?;
        ensure!(
            map.scripts.get(&label) == Some(&json!([command("hiddenitem", &[&cache.item, &flag])])),
            "original hidden reward changed: {}",
            cache.id
        );
        // Keep the existing receipt: a previously collected cache stays empty.
        map.scripts.insert(
            label,
            json!([command("hiddenitem", &[&reward.item, &flag])]),
        );
        map.script_field_pickups =
            crate::parse_script_field_pickups(&map.id, &map.scripts, &map.objects)?;
        crate::northstar_story_revision::publish(pack, map)?;
    }
    let mut hinted = BTreeSet::new();
    for hint in &refinement.hints {
        ensure!(hinted.insert(&hint.cache), "duplicate discovery clue");
        let cache = caches.get(hint.cache.as_str()).context("hint cache")?;
        let mut map = pack.data.maps.get(&cache.map).context("hint map")?.clone();
        check_pickup(pack, &map, cache)?;
        ensure!(
            map.objects
                .iter()
                .filter(|actor| {
                    actor.object_identifier.as_deref() == Some(hint.actor.as_str())
                        && actor.script == hint.script
                })
                .count()
                == 1,
            "hint actor binding changed: {}",
            hint.actor
        );
        let original = map
            .scripts
            .get(&hint.script)
            .context("ambient program")?
            .clone();
        let body = original.as_array().context("ambient program array")?;
        ensure!(
            body.len() == 1 && body[0]["command"] == "jumptextfaceplayer",
            "discovery clue cannot replace a service, trainer or story branch"
        );
        let after = format!("MnDiscovery{}After", cache.id);
        let text = format!("MnDiscovery{}ClueText", cache.id);
        ensure!(
            !map.scripts.contains_key(&after) && !map.scripts.contains_key(&text),
            "discovery clue labels already exist"
        );
        map.scripts.insert(after.clone(), original);
        map.scripts.insert(
            text.clone(),
            authored_dialogue::compile(&dialogue[&hint.cache])?,
        );
        map.scripts.insert(
            hint.script.clone(),
            json!([
                command("checkevent", &[&format!("EVENT_MN_HIDDEN_{}", cache.id)]),
                command("iftrue", &[&after]),
                command("jumptextfaceplayer", &[&text])
            ]),
        );
        crate::northstar_story_revision::publish(pack, map)?;
    }
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn check_pickup(pack: &CompiledGamePack, map: &MapModule, cache: &OriginalCache) -> Result<()> {
    let label = format!("MN_HIDDEN_{}", cache.id);
    ensure!(
        map.events
            .bg_events
            .iter()
            .filter(|event| {
                event.event_type == "BGEVENT_ITEM"
                    && event.script == label
                    && [event.x, event.y] == cache.tile
            })
            .count()
            == 1,
        "hidden discovery moved: {}",
        cache.id
    );
    ensure!(
        pack.data
            .story_event_script_constants
            .global
            .contains_key(&format!("EVENT_{label}")),
        "original hidden receipt missing: {}",
        cache.id
    );
    Ok(())
}
