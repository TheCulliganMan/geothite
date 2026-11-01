//! Native permit directions replace terrain-only closure notices in a review.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::systems::safari::{ADMISSION_FEE, BALL_BUDGET, STEP_BUDGET};

use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, SafariZone};

#[derive(Clone, Copy)]
enum TextRole {
    Permit,
    Rules,
    PublicApproach,
    PublicBoard,
    Admission,
}

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
    let mut found_public = BTreeSet::new();
    let names = zone
        .maps
        .keys()
        .chain([&zone.admission_map, &zone.gate_court.approach_map])
        .collect::<BTreeSet<_>>();
    for name in names {
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
                labels.insert(format!("{}Text", actor.script), TextRole::Permit);
            } else if name == &zone.admission_map {
                let role = if actor.script == zone.admission_script {
                    Some(TextRole::Admission)
                } else if matches!(
                    actor.script.as_str(),
                    "MN_BOUNDARY_PENDING_GATE_WEST" | "MN_BOUNDARY_PENDING_GATE_EAST"
                ) {
                    Some(TextRole::Permit)
                } else {
                    None
                };
                if let Some(role) = role {
                    found_public.insert(actor.script.clone());
                    labels.insert(format!("{}Text", actor.script), role);
                }
            }
        }
        for sign in &map.events.bg_events {
            if matches!(sign.script.as_str(), "BWS_SIGN_01" | "BWS_SIGN_06") {
                found_signs.insert(sign.script.clone());
                labels.insert(format!("{}Text", sign.script), TextRole::Rules);
            } else if name == &zone.gate_court.approach_map
                && sign.script == "MN_COHESION_PRESERVE_NOTICE"
            {
                found_public.insert(sign.script.clone());
                labels.insert(format!("{}Text", sign.script), TextRole::PublicApproach);
            } else if name == &zone.admission_map && sign.script == "MN_BOUNDARY_GATE_NOTICE" {
                found_public.insert(sign.script.clone());
                labels.insert(format!("{}Text", sign.script), TextRole::PublicBoard);
            }
        }
        if labels.is_empty() {
            continue;
        }
        for (label, role) in labels {
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
            let anchor: &[&str] = match role {
                TextRole::PublicApproach => &[
                    "PRESERVE ENTRY",
                    "Expeditions",
                    "closed.",
                    "No fee is charged.",
                ],
                TextRole::PublicBoard => &[
                    "EXPEDITIONS CLOSED",
                    "No fee is charged.",
                    "The survey shelter",
                    "is open to all.",
                ],
                TextRole::Permit | TextRole::Rules | TextRole::Admission => &[
                    "EXPEDITIONS CLOSED",
                    "Safari permits are",
                    "not available yet.",
                    "No fee is charged.",
                ],
            };
            ensure!(
                lines == anchor,
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
            let fee = format!("{ADMISSION_FEE} entry fee.");
            let balls = format!("{BALL_BUDGET} SAFARI BALLS.");
            let steps = format!("{STEP_BUDGET} travel steps.");
            let lines: &[&str] = match role {
                TextRole::Rules => &[
                    "Permit check-in:",
                    "Basecamp ranger.",
                    "Leaving ends trip.",
                    "Keep your catches.",
                ],
                TextRole::Permit => &[
                    "Permit required.",
                    "Ask the ranger",
                    "at basecamp to",
                    "start a trip.",
                ],
                TextRole::PublicApproach => &[
                    "PRESERVE ENTRY",
                    "Free trail ahead.",
                    "Safari check-in:",
                    "Basecamp ranger.",
                ],
                TextRole::PublicBoard => &[
                    "SAFARI CHECK-IN",
                    "Ranger: just east.",
                    "Public trails and",
                    "survey stay free.",
                ],
                TextRole::Admission => &[&fee, &balls, &steps, "Ask to check in."],
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
    ensure!(
        found_public.len() == 5,
        "Safari public check-in guidance missing"
    );
    Ok(())
}
