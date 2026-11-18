//! Publish the route signs and conversations that describe the open fishing piers.
use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::CompiledGamePack;
use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::parse_script_text_bodies;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacement {
    map: String,
    label: String,
    source: String,
    previous: Vec<AuthoredDialoguePage>,
}

#[derive(Deserialize)]
struct Route {
    signs: Vec<Sign>,
}

#[derive(Deserialize)]
struct Sign {
    id: String,
    lines: Vec<String>,
}

fn pages(entry: &Replacement, story: &Value, region: &Path) -> Result<Vec<AuthoredDialoguePage>> {
    let table = match entry.source.as_str() {
        "dialogue_overhaul" => &story["dialogue_overhaul"]["texts"],
        "story_flow" => &story["story_flow_revision"]["dialogue"],
        route => {
            let route_id = entry.map.get(2..5).context("missing route number")?;
            ensure!(
                route == format!("MN_{route_id}.json"),
                "unexpected sign source {route}"
            );
            let design: Route = super::source(&region.join("routes"), route)?;
            let sign = design
                .signs
                .into_iter()
                .find(|sign| entry.label.strip_suffix("Text") == Some(&sign.id))
                .with_context(|| format!("missing sign {} in {route}", entry.label))?;
            return Ok(sign
                .lines
                .chunks(2)
                .map(|lines| AuthoredDialoguePage {
                    lines: lines.to_vec(),
                })
                .collect());
        }
    };
    serde_json::from_value(table[&entry.map][&entry.label].clone())
        .with_context(|| format!("authored pages {}/{}", entry.map, entry.label))
}

pub(super) fn install(pack: &mut CompiledGamePack, dir: &Path) -> Result<()> {
    let region = dir
        .parent()
        .and_then(Path::parent)
        .context("missing regional source directory")?;
    let story: Value = super::source(region, "story.json")?;
    for entry in super::source::<Vec<Replacement>>(dir, "dock-dialogue.json")? {
        let map = pack
            .data
            .maps
            .get_mut(&entry.map)
            .with_context(|| format!("dock map {}", entry.map))?;
        let old = authored_dialogue::compile(&entry.previous)?;
        ensure!(
            map.scripts.get(&entry.label) == Some(&old)
                && pack.data.map_scripts.get(&entry.label) == Some(&old),
            "unexpected dock text or global mirror {}/{}",
            entry.map,
            entry.label
        );
        let mut text = BTreeMap::from([(entry.label.clone(), old)]);
        let prior = parse_script_text_bodies(&entry.map, &text)?;
        ensure!(
            map.script_text_bodies.get(&entry.label) == prior.get(&entry.label),
            "unexpected cached dock text {}",
            entry.label
        );
        let revised = authored_dialogue::compile(&pages(&entry, &story, region)?)?;
        text.insert(entry.label.clone(), revised.clone());
        map.script_text_bodies
            .extend(parse_script_text_bodies(&entry.map, &text)?);
        map.scripts.insert(entry.label.clone(), revised.clone());
        pack.data.map_scripts.insert(entry.label, revised);
    }
    Ok(())
}
