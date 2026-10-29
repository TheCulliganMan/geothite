//! Native permit directions replace terrain-only closure notices in a review.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, SafariZone};

pub(crate) fn install(pack: &mut CompiledGamePack, zone: &SafariZone) -> Result<()> {
    let roles = zone
        .gate_court
        .guard_identifiers
        .iter()
        .chain(&zone.retire_scripts)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut found_roles = BTreeSet::new();
    let mut found_signs = BTreeSet::new();
    for name in zone.maps.keys() {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("Safari guidance map missing")?
            .clone();
        let mut labels = BTreeMap::new();
        for actor in &map.objects {
            if roles.contains(actor.script.as_str()) {
                found_roles.insert(actor.script.clone());
                labels.insert(format!("{}Text", actor.script), false);
            }
        }
        for sign in &map.events.bg_events {
            if matches!(sign.script.as_str(), "BWS_SIGN_01" | "BWS_SIGN_06") {
                found_signs.insert(sign.script.clone());
                labels.insert(format!("{}Text", sign.script), true);
            }
        }
        if labels.is_empty() {
            continue;
        }
        for (label, sign) in labels {
            let old = map
                .script_text_bodies
                .get(&label)
                .with_context(|| format!("Safari closure text missing: {name}/{label}"))?;
            let lines = old
                .commands
                .iter()
                .filter(|command| matches!(command.command.as_str(), "text" | "line" | "para"))
                .filter_map(|command| command.args.first())
                .map(|line| line.trim_matches('"'))
                .collect::<Vec<_>>();
            ensure!(
                lines
                    == [
                        "EXPEDITIONS CLOSED",
                        "Safari permits are",
                        "not available yet.",
                        "No fee is charged."
                    ],
                "Safari closure anchor changed: {name}/{label}"
            );
            ensure!(
                !pack
                    .data
                    .maps
                    .iter()
                    .any(|(other_name, other)| other_name != name
                        && (other.scripts.contains_key(&label)
                            || other.script_text_bodies.contains_key(&label))),
                "Safari guidance label is shared outside its map: {label}"
            );
            let lines: &[&str] = if sign {
                &[
                    "Permit check-in:",
                    "Basecamp ranger.",
                    "Leaving ends trip.",
                    "Keep your catches.",
                ]
            } else {
                &[
                    "Permit required.",
                    "Ask the ranger",
                    "at basecamp to",
                    "start a trip.",
                ]
            };
            authoring::text(&mut map, &label, lines);
            let payload = map
                .scripts
                .get(&label)
                .context("authored permit text")?
                .clone();
            let mut parsed = crate::parse_script_text_bodies(
                name,
                &BTreeMap::from([(label.clone(), payload.clone())]),
            )?;
            map.script_text_bodies.insert(
                label.clone(),
                parsed.remove(&label).context("compiled permit text")?,
            );
            // Text-only changes keep native script command indices, actors,
            // events and terrain intact. Update the existing split mirror too.
            pack.data.map_scripts.insert(label, payload);
        }
        pack.data.maps.insert(name.clone(), map);
    }
    ensure!(
        found_roles.len() == roles.len(),
        "Safari permit/return speaker missing"
    );
    ensure!(found_signs.len() == 2, "Safari checkpoint signs missing");
    Ok(())
}
