//! Add locations from an installed pack to a preserved Pokédex release.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use crystal_assets::CompiledGamePack;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CSS: &str = include_str!("../../../../../web-client/pokedex/locations.css");

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct Location {
    map: String,
    place: String,
    method: String,
    area: String,
    times: BTreeSet<String>,
    levels: BTreeSet<u8>,
    note: String,
    #[serde(default)]
    minimum_badges: Option<u8>,
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn words(token: &str) -> String {
    token
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().to_string() + &chars.as_str().to_lowercase())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn place(pack: &CompiledGamePack, map: &str) -> String {
    match map {
        "MnMoorhead" => return "Moorhead · Red River".into(),
        "MnStCloud" => return "St Cloud · Riverfront".into(),
        "MnStPaul" => return "St Paul · Mississippi River".into(),
        "MnDetroitLakes" => return "Detroit Lakes".into(),
        "MnDriftlessLampGallery" => return "Driftless Fossil Gallery".into(),
        "MnRochesterPatientHouse" => return "Rochester · Resting House".into(),
        "MnIronDrillApproach" => return "Iron Range Drill Approach".into(),
        "MnIronSump" => return "Iron Range Pump Sump".into(),
        "MnBoundaryIntake" => return "Boundary Intake Channel".into(),
        "MnDevilsKettle" => return "Devil's Kettle".into(),
        "MnTallTaleTrail" => return "Bemidji · Tall Tale Trail".into(),
        "MnFieldMargin" => return "Prairie Field Margin".into(),
        "MnIcemythServicePassage" => return "State Fair Exhibition · Service Passage".into(),
        "MnInnerBasin" => return "Former Shores · Inner Basin".into(),
        "MnLayeredFen" => return "Bemidji · Layered Fen".into(),
        "MnVoyageursIce1F" => return "Voyageurs Ice Cavern · 1F".into(),
        "MnVoyageursIceB1F" => return "Voyageurs Ice Cavern · B1F".into(),
        _ => {}
    }
    if map == "MnRochesterFossilLab" {
        return "Rochester Fossil Lab".into();
    }
    if map == "MnStateFairgrounds" {
        return "Minnesota State Fair".into();
    }
    if map == "MnWinonaFieldStation" {
        return "Winona Field Station".into();
    }
    if map == "MnGrandMaraisFieldStation" {
        return "Grand Marais Field Station".into();
    }
    let location = pack.data().maps[map]
        .attributes
        .location
        .as_deref()
        .unwrap_or(map);
    let name = words(
        location
            .trim_start_matches("LANDMARK_MN_")
            .trim_start_matches("LANDMARK_"),
    );
    if let Some(route) = map
        .strip_prefix("MnR")
        .and_then(|rest| rest.get(..2))
        .filter(|route| route.bytes().all(|b| b.is_ascii_digit()))
    {
        let cave = if pack.data().maps[map].attributes.environment.as_deref() == Some("CAVE") {
            " · Cave"
        } else {
            ""
        };
        format!("Route {} · {name}{cave}", route.parse::<u8>().unwrap_or(0))
    } else {
        name
    }
}

fn wild_note(map: &str, method: &str) -> &'static str {
    match (map, method) {
        ("MnMoorhead", "Surf") => "Use the signed brick opening in the Red River levee.",
        ("MnR03Cave1F", "Surf") => "The seep lies beyond the existing Strength passage.",
        ("MnDriftlessLampGallery", "Cave") => {
            "The lower gallery is reached from Driftless Hollow. Bring a partner that can use Flash."
        }
        ("MnR12Cave1F" | "MnR13Cave1F", "Surf") => {
            "Surf in the refuge spring. The dry floor remains peaceful."
        }
        _ => "",
    }
}

fn add(
    rows: &mut BTreeMap<String, Vec<Location>>,
    species: &str,
    mut row: Location,
    level: u8,
    time: &str,
) {
    let entries = rows.entry(species.into()).or_default();
    if let Some(existing) = entries.iter_mut().find(|e| {
        e.map == row.map && e.method == row.method && e.area == row.area && e.note == row.note
    }) {
        existing.levels.insert(level);
        existing.times.insert(time.into());
    } else {
        row.levels.insert(level);
        row.times.insert(time.into());
        entries.push(row);
    }
}

fn gift_note(pack: &CompiledGamePack, species: &str, map: &str, script: &str) -> String {
    let early = pack
        .report()
        .manifests
        .iter()
        .any(|id| id == "minnesota-early-research-access-v1");
    if early {
        if map == "MnWinonaFieldStation" && script.starts_with("MnEarlyPartner") {
            return "Free level-5 starter partner from the Winona research aide after delivering Tamarack's first parcel to the Rochester clinic. Return for each of the two starters you did not choose. Leave one party slot; declined or full-party offers wait for you.".into();
        }
        if map == "MnRochesterFossilLab" && matches!(species, "TRILUME" | "SIPHLET") {
            let fossil = if species == "TRILUME" {
                "LENS FOSSIL"
            } else {
                "SPIRE FOSSIL"
            };
            return format!(
                "Choose a free specimen from the Route 1 bluff researcher. Bring the {fossil} to the Fossil Lab beside Rochester's clinic with the Care Badge and one free party slot. After restoring either fossil, the lab assistant gives you the other specimen for free. Both fossil lines are available in the same journey."
            );
        }
    }
    match (species, map, script) {
        ("PIPULSE", "MnRochesterPatientHouse", "MN_SMALL_INTERIOR_ROCHESTER_PATIENT_FRIEND") => "Free level-8 companion from the friend at Rochester's resting house after delivering the first parcel to the clinic. Leave one party slot. Refusal or a full party preserves the offer; no badge or payment is required.".into(),
        ("TRILUME", "MnRochesterFossilLab", _) => "Bring the LENS FOSSIL from the Route 1 bluff researcher to the Fossil Lab beside Rochester's clinic. Requires the Care Badge and a free party slot. The other fossil is offered after becoming Champion.".into(),
        ("SIPHLET", "MnRochesterFossilLab", _) => "Bring the SPIRE FOSSIL from the Route 1 bluff researcher to the Fossil Lab beside Rochester's clinic. Requires the Care Badge and a free party slot. The other fossil is offered after becoming Champion.".into(),
        ("TRILUME", "MnRochester", _) => "Bring the LENS FOSSIL from the Route 1 bluff researcher to the scientist in Rochester's clinic courtyard. Requires the Care Badge and a free party slot. The other fossil is offered after becoming Champion.".into(),
        ("SIPHLET", "MnRochester", _) => "Bring the SPIRE FOSSIL from the Route 1 bluff researcher to the scientist in Rochester's clinic courtyard. Requires the Care Badge and a free party slot. The other fossil is offered after becoming Champion.".into(),
        (_, "MnWinonaFieldStation", "MN_W_CHIKORITA" | "MN_W_CYNDAQUIL" | "MN_W_TOTODILE") => "Choose one of the three starter partners at the field station.".into(),
        (_, _, "MnKindlitResearchAide") => "Research partner offered after becoming the regional Champion. Delivery requires space in your party or PC.".into(),
        ("BIRCHIT", "MnGrandMaraisFieldStation", _) => "A partner offered by the Grand Marais Field Station. Leave space in your party or the selected PC box.".into(),
        ("GILLPUP", "MnWinonaFieldStation", _) => "Receive the research egg at the Winona Field Station. Carry it in your party and walk to hatch it.".into(),
        _ => "Requires the local story or research conditions. Speak to the residents for the handoff.".into(),
    }
}

fn terrain<'a>(pack: &'a CompiledGamePack, map: &str, x: usize, y: usize) -> Option<&'a str> {
    let room = &pack.data().maps[map];
    let block = room.blocks[y / 2 * usize::from(room.attributes.width) + x / 2];
    pack.data().tilesets[&room.attributes.tileset_name]
        .collision
        .get(&format!("{block:02x}"))
        .map(|c| c[y % 2 * 2 + x % 2].as_str())
}

fn collect(pack: &CompiledGamePack) -> Result<BTreeMap<String, Vec<Location>>> {
    let mut rows = BTreeMap::new();
    for (map, room) in &pack.data().maps {
        if !map.starts_with("Mn") {
            continue;
        }
        let base = Location {
            map: map.clone(),
            place: place(pack, map),
            ..Default::default()
        };
        if let Some(data) = pack.data().wild_encounters.get(map) {
            let cave = room.attributes.environment.as_deref() == Some("CAVE");
            for (method, table) in [
                (if cave { "Cave" } else { "Grass" }, data.grass.as_ref()),
                ("Surf", data.water.as_ref()),
            ] {
                let Some(table) = table else {
                    continue;
                };
                let has_surface = (0..usize::from(room.attributes.height) * 2).any(|y| {
                    (0..usize::from(room.attributes.width) * 2).any(|x| {
                        let kind = terrain(pack, map, x, y).unwrap_or("");
                        if method == "Surf" {
                            kind == "WATER" || kind.starts_with("WATER_")
                        } else {
                            (kind == "TALL_GRASS" || cave && kind == "FLOOR")
                                && data.zone_at(x as i16, y as i16).is_none()
                        }
                    })
                });
                if !has_surface {
                    continue;
                }
                for (time, slots) in [
                    ("morning", &table.morning),
                    ("day", &table.day),
                    ("night", &table.night),
                ] {
                    let active = if method == "Surf" {
                        data.water_rate.unwrap_or(0)
                    } else {
                        data.grass_rates
                            .as_ref()
                            .and_then(|r| r.get(time))
                            .copied()
                            .unwrap_or(0)
                    };
                    if active == 0 {
                        continue;
                    }
                    for slot in slots {
                        add(
                            &mut rows,
                            &slot.species,
                            Location {
                                method: method.into(),
                                note: wild_note(map, method).into(),
                                area: if method == "Surf" {
                                    "Open water"
                                } else if cave {
                                    "Cave floor"
                                } else {
                                    "Route grass"
                                }
                                .into(),
                                ..base.clone()
                            },
                            slot.level,
                            time,
                        );
                    }
                }
            }
            for zone in &data.zones {
                let label = if zone.id == "MN_LEECH_PINE_ISLAND" {
                    "Pine Island".into()
                } else {
                    words(zone.id.trim_start_matches("MN_HABITAT_"))
                };
                for (time, slots) in [
                    ("morning", &zone.grass.morning),
                    ("day", &zone.grass.day),
                    ("night", &zone.grass.night),
                ] {
                    if zone.grass_rates.get(time).copied().unwrap_or(0) == 0 {
                        continue;
                    }
                    let surface_exists = (zone.min_y..=zone.max_y).any(|y| {
                        (zone.min_x..=zone.max_x).any(|x| {
                            x >= 0
                                && y >= 0
                                && x < room.attributes.width as i16 * 2
                                && y < room.attributes.height as i16 * 2
                                && terrain(pack, map, x as usize, y as usize) == Some("TALL_GRASS")
                        })
                    });
                    if !surface_exists {
                        continue;
                    }
                    for slot in slots {
                        add(
                            &mut rows,
                            &slot.species,
                            Location {
                                method: "Grass".into(),
                                area: label.clone(),
                                ..base.clone()
                            },
                            slot.level,
                            time,
                        );
                    }
                }
            }
        }
        let has_water = (0..usize::from(room.attributes.height) * 2).any(|y| {
            (0..usize::from(room.attributes.width) * 2).any(|x| {
                terrain(pack, map, x, y).is_some_and(|k| k == "WATER" || k.starts_with("WATER_"))
            })
        });
        if has_water
            && let Some(group) = room
                .attributes
                .fishing_group
                .as_ref()
                .and_then(|g| pack.data().fishing.groups.get(g))
        {
            for (rod, table) in &group.rod_tables {
                let mut previous = 0;
                for slot in &table.slots {
                    let enabled = slot.threshold > previous && group.bite_threshold > 0;
                    previous = slot.threshold;
                    if !enabled {
                        continue;
                    }
                    let row = Location {
                        method: words(rod),
                        ..base.clone()
                    };
                    if let Some(species) = &slot.species {
                        add(&mut rows, species, row, slot.level, "all day");
                    } else if let Some(entry) = slot
                        .time_group
                        .as_ref()
                        .and_then(|g| pack.data().fishing.time_groups.get(g))
                    {
                        add(
                            &mut rows,
                            &entry.day_species,
                            row.clone(),
                            entry.day_level,
                            "morning / day",
                        );
                        add(
                            &mut rows,
                            &entry.night_species,
                            row,
                            entry.night_level,
                            "night",
                        );
                    }
                }
            }
        }
        if let Some(field) = pack.data().field_encounters.get(map) {
            let has_tree = (0..usize::from(room.attributes.height) * 2).any(|y| {
                (0..usize::from(room.attributes.width) * 2).any(|x| {
                    terrain(pack, map, x, y).is_some_and(|kind| kind.starts_with("HEADBUTT_TREE"))
                })
            });
            if has_tree && let Some(table) = field.tables.get("headbutt") {
                for slot in table.common.iter().chain(&table.rare) {
                    add(&mut rows, &slot.species, Location {
                            method: "Headbutt".into(), area: "Headbutt trees".into(),
                            note: "Use Headbutt on a tree. Different trees can attract different Pokémon.".into(),
                            ..base.clone()
                        }, slot.level, "all day");
                }
            }
        }
        for gift in &room.gift_pokemon_scripts {
            add(
                &mut rows,
                &gift.species_id,
                Location {
                    method: if gift.egg { "Egg" } else { "Gift / research" }.into(),
                    note: gift_note(pack, &gift.species_id, map, &gift.source_script),
                    ..base.clone()
                },
                gift.level,
                "story conditions",
            );
        }
        // Scripted battles are conditional encounters, not ordinary grass rolls.
        let battles = serde_json::to_value(&room.scripted_wild_battles)?;
        for battle in battles.as_array().context("scripted battles")? {
            if let (Some(species), Some(level)) = (
                battle["request"]["species"].as_str(),
                battle["request"]["level"].as_u64(),
            ) {
                add(
                    &mut rows,
                    species,
                    Location {
                        method: "Discovery".into(),
                        note: match species {
                            "CASTOROCK" => "Use Strength to enter the incisor gallery. Examine the old waterline grooves, worn pillar and fresh toothmarks. If it retreats after RUN or defeat, leave and reenter the cave to try again.",
                            "VERGLOOM" => "Inspect the petrified timber, peat footprint and moss scrape in the ancient mire. Acknowledge all three observations to reveal Vergloom. Leave and return for another attempt after RUN or defeat.",
                            "BABEHEMOTH" | "BUNJANT" => "Enter Tall Tale Trail through the Bemidji northwoods gate. Read the culvert, crescent hoofprints and windthrow stack, then compare them with the trail worker. Inspect the camp path and marked log, and report back to reveal both creatures. Each encounter is optional and independent; leave and reenter for another uncaught attempt.",
                            "PEPIVERN" => "Investigate the lower Mississippi dock sketch and shoreline eddy, then observe the reed opening while using Surf. Compare all three observations with the dock operator and approach the sheltered inlet. Leave and reenter if the uncaught creature retreats.",
                            "ICEMYTH" => "At the State Fair Exhibition Hall, speak to the attendant, delivery worker and visitor, and inspect the latch and meltwater tray. Return to the attendant, then inspect the empty case in the service passage. Leave and reenter for another uncaught attempt.",
                            "KENSIGARD" => "Follow the prairie field margin and compare the marker soil, buried edge and shifting grooves. Locate the displaced marker and return to the caretaker. Revisit the margin for the optional encounter; leave and reenter to retry if uncaught.",
                            "FLARKINDLE" => "Enter the layered fen from the Bemidji northwoods. Read the peat rib, pool and buried root observations, then compare them at the shelter. Follow the inner markers in order: old root, pool edge, living rib. The optional encounter remains retryable after leaving and reentering.",
                            "STRANDRAK" => "Explore the three former-shore terraces off Route 16 and compare their marks with the surveyor. In the inner basin, operate all three dials: upper high, center middle, lower low. Use the outlet lever, then approach the creature. Leaving and reentering permits another uncaught attempt.",
                            "KETTLURK" => "Follow the Devil's Kettle stairs from the Marais cedar approach after the intake release and acknowledged Boundary Waters restoration. The optional encounter remains available after refusal; leave and reenter after an uncaught retreat.",
                            "VERDIGRIM" => "Investigate the Boundary Intake excavation and complete the separate drill, outlet and clean-bypass controls. Acknowledge the crew's clearance before approaching Verdigrim. Capture is optional and never required to restore the water; an uncaught encounter can be retried.",
                            _ => "Requires the local discovery conditions.",
                        }.into(),
                        ..base.clone()
                    },
                    u8::try_from(level)?,
                    "story conditions",
                );
            }
        }
    }
    if pack
        .report()
        .manifests
        .iter()
        .any(|id| id == "minnesota-encounter-progression-v1")
    {
        let plan: Value = serde_json::from_str(include_str!(
            "../../../../../planning/minnesota/pokemon/acquisition-v01/encounter-progression-v01/progression.json"
        ))?;
        let mut access = BTreeMap::new();
        let mut badges = BTreeMap::new();
        for group in plan["groups"]
            .as_array()
            .context("authored journey groups")?
        {
            let note = group["access"].as_str().context("journey access")?;
            for map in group["maps"].as_array().context("journey maps")? {
                access.insert(map.as_str().context("journey map")?, note);
                badges.insert(
                    map.as_str().context("journey map")?,
                    u8::try_from(
                        group["minimum_badges"]
                            .as_u64()
                            .context("journey badge count")?,
                    )?,
                );
            }
        }
        for source in rows.values_mut().flatten() {
            source.minimum_badges =
                badges
                    .get(source.map.as_str())
                    .copied()
                    .or(match source.map.as_str() {
                        "MnWinonaFieldStation" | "MnRochesterPatientHouse" => Some(0),
                        "MnRochesterFossilLab" => Some(1),
                        "MnGrandMaraisFieldStation" => Some(8),
                        _ => None,
                    });
            if source.method == "Surf" {
                source.minimum_badges = source.minimum_badges.map(|n| n.max(4));
            }
            if source.method == "Good Rod" {
                source.minimum_badges = source.minimum_badges.map(|n| n.max(2));
            }
            if source.method == "Super Rod" {
                source.minimum_badges = source.minimum_badges.map(|n| n.max(5));
            }
            if matches!(
                source.method.as_str(),
                "Gift / research" | "Egg" | "Discovery"
            ) {
                continue;
            }
            if let Some(note) = access.get(source.map.as_str()) {
                if !source.note.is_empty() {
                    source.note.push(' ');
                }
                source.note.push_str(note);
            }
        }
    }
    for locations in rows.values_mut() {
        locations.sort();
        locations.dedup();
    }
    Ok(rows)
}

fn copy_release(source: &Path, output: &Path, old: &str, new: &str) -> Result<()> {
    fs::create_dir(output)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        ensure!(
            !entry.file_type()?.is_symlink(),
            "release may not contain symlinks"
        );
        let path = entry.path();
        let target = output.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_release(&path, &target, old, new)?;
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("html" | "json" | "js" | "css" | "svg")
        ) {
            fs::write(target, fs::read_to_string(path)?.replace(old, new))?;
        } else {
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

fn panel(sources: &[Location], evolution: &[Value], available: bool) -> String {
    // Keep exact map/zone records in the download, but present one readable
    // entry for contiguous beds or multiple chunks of the same named route.
    let mut grouped = BTreeMap::new();
    for source in sources {
        let mut row = source.clone();
        if row.area.starts_with("Smokehouse ") {
            row.area = "Behind and beside the North Shore Smokehouse".into();
        } else if row.area.starts_with("Harvest Oats ") {
            row.area = "Eastern harvest fields".into();
        }
        let key = (
            row.place.clone(),
            row.method.clone(),
            row.area.clone(),
            row.note.clone(),
        );
        let combined: &mut Location = grouped.entry(key).or_insert_with(|| row.clone());
        combined.times.extend(row.times);
        combined.levels.extend(row.levels);
    }
    let mut sources: Vec<_> = grouped.into_values().collect();
    sources.sort_by(|a, b| {
        (
            a.minimum_badges.unwrap_or(9),
            a.levels.first().copied().unwrap_or(100),
            &a.place,
            &a.area,
        )
            .cmp(&(
                b.minimum_badges.unwrap_or(9),
                b.levels.first().copied().unwrap_or(100),
                &b.place,
                &b.area,
            ))
    });
    let mut html = String::from(
        "<section class=\"locations-panel\" aria-labelledby=\"locations-title\"><span class=\"eyebrow\">FIELD GUIDE</span><h2 id=\"locations-title\">Where to find</h2>",
    );
    if !available {
        html.push_str("<p class=\"location-pending\">This Pokémon's acquisition route has not yet been added to the current map.</p>");
    }
    if !sources.is_empty() {
        html.push_str("<ul class=\"location-list\">");
        for (index, source) in sources.iter().enumerate() {
            if index == 4 {
                html.push_str("</ul><details class=\"more-locations\"><summary>More locations</summary><ul class=\"location-list\">");
            }
            let level = match (source.levels.first(), source.levels.last()) {
                (Some(a), Some(b)) if a == b => format!("Lv. {a}"),
                (Some(a), Some(b)) => format!("Lv. {a}–{b}"),
                _ => String::new(),
            };
            let times = if source.times.contains("all day")
                || source.times == BTreeSet::from(["morning".into(), "day".into(), "night".into()])
                || (source.times.contains("morning / day") && source.times.contains("night"))
            {
                "All day".into()
            } else {
                source
                    .times
                    .iter()
                    .map(|s| words(s))
                    .collect::<Vec<_>>()
                    .join(" · ")
            };
            html.push_str(&format!("<li><strong>{}</strong><span class=\"location-method\">{} · {level} · {}</span><span>{}</span><p>{}</p></li>",
                escape(&source.place), escape(&source.method), escape(&times), escape(&source.area), escape(&source.note)));
        }
        html.push_str("</ul>");
        if sources.len() > 4 {
            html.push_str("</details>");
        }
    }
    for edge in evolution {
        let from = edge["from"].as_str().unwrap_or("");
        html.push_str(&format!("<p class=\"location-evolution\">Evolve <a href=\"/pokedex/species/{}.html\">{}</a> at level {}.</p>",
            escape(&from.to_ascii_lowercase()), escape(&words(from)), edge["level"]));
    }
    html.push_str("<p class=\"location-caption\">Locations follow the current local Minnesota map. Levels shown are base levels; wild Pokémon may scale with your team. Story and field-move requirements still apply. This guide is updated as more habitats are added.</p></section>");
    html
}

fn evolution_locations(
    pack: &CompiledGamePack,
    sources: &BTreeMap<String, Vec<Location>>,
    species: &str,
) -> Vec<Location> {
    let mut pending = vec![(species.to_owned(), Vec::<(String, u8)>::new())];
    let mut visited = BTreeSet::new();
    let mut locations = Vec::new();
    while let Some((current, steps)) = pending.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        for (parent, edges) in &pack.data().evolutions.0 {
            for edge in edges
                .iter()
                .filter(|e| e.species == current && e.method == "LEVEL")
            {
                let Some(level) = edge.level else {
                    continue;
                };
                let mut path = vec![(current.clone(), level)];
                path.extend(steps.clone());
                if let Some(rows) = sources.get(parent) {
                    let instructions = path
                        .iter()
                        .enumerate()
                        .map(|(i, (target, level))| {
                            format!(
                                "{} {} at level {level}",
                                if i == 0 { "Evolve to" } else { "then to" },
                                words(target)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    for row in rows {
                        let mut row = row.clone();
                        row.method = format!("Raise {}", words(parent));
                        row.note =
                            format!("Catch {} here. {instructions}. {}", words(parent), row.note);
                        locations.push(row);
                    }
                } else {
                    pending.push((parent.clone(), path));
                }
            }
        }
    }
    locations.sort();
    locations.dedup();
    locations
}

/// Publishes an incremental location guide while preserving the reviewed media.
///
/// # Errors
/// Rejects occupied outputs, malformed releases, catalog/roster mismatch and
/// invalid installed game content. This never modifies the supplied game pack.
pub(crate) fn publish(pack: &CompiledGamePack, baseline: &Path, output: &Path) -> Result<()> {
    ensure!(!output.exists(), "location release must be fresh");
    let baseline = baseline.canonicalize()?;
    let output = output
        .parent()
        .context("output parent")?
        .canonicalize()?
        .join(output.file_name().context("release name")?);
    ensure!(
        !output.starts_with(&baseline),
        "output cannot be inside baseline"
    );
    let asset_base = |path: &Path| -> Result<String> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("release name")?;
        ensure!(
            name.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')),
            "unsafe release id"
        );
        Ok(format!("/pokedex/releases/{name}"))
    };
    let old = asset_base(&baseline)?;
    let new = asset_base(&output)?;
    let mut catalog: Value = serde_json::from_str(
        &fs::read_to_string(baseline.join("pokedex.json"))?.replace(&old, &new),
    )?;
    let sources = collect(pack)?;
    let active: BTreeSet<_> = catalog["species"]
        .as_array()
        .context("active species")?
        .iter()
        .map(|s| {
            s["identity"]["name"]
                .as_str()
                .unwrap_or("")
                .to_ascii_uppercase()
        })
        .collect();
    ensure!(
        active.len() == catalog["species_count"].as_u64().context("species count")? as usize,
        "unique active catalog required"
    );
    let roster: Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/pokemon/roster.json"
    ))?;
    let curation: Value = serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/pokemon/roster-curation.json"
    ))?;
    let retired: BTreeSet<_> = curation["retired_families"]
        .as_array()
        .context("retired families")?
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    let roster_names: BTreeSet<_> = roster["families"]
        .as_array()
        .context("roster families")?
        .iter()
        .filter(|f| !retired.contains(f["id"].as_str().unwrap_or("")))
        .flat_map(|f| f["members"].as_array().into_iter().flatten())
        .filter_map(|m| m["name"].as_str())
        .map(str::to_ascii_uppercase)
        .collect();
    ensure!(
        active == roster_names,
        "baseline differs from the active roster; export the new catalog before adding locations"
    );
    let mut available: BTreeSet<_> = sources
        .keys()
        .filter(|s| active.contains(*s))
        .cloned()
        .collect();
    loop {
        let more: BTreeSet<_> = available
            .iter()
            .flat_map(|s| pack.data().evolutions.0.get(s).into_iter().flatten())
            .filter(|e| active.contains(&e.species))
            .map(|e| e.species.clone())
            .collect();
        let size = available.len();
        available.extend(more);
        if size == available.len() {
            break;
        }
    }
    copy_release(&baseline, &output, &old, &new)?;
    let mut index = fs::read_to_string(output.join("index.html"))?;
    for entry in catalog["species"].as_array_mut().context("species array")? {
        let species = entry["identity"]["name"]
            .as_str()
            .context("species name")?
            .to_ascii_uppercase();
        let locations = sources.get(&species).cloned().unwrap_or_default();
        let evolution_sources = if locations.is_empty() {
            evolution_locations(pack, &sources, &species)
        } else {
            Vec::new()
        };
        let evolution: Vec<_> = pack
            .data()
            .evolutions
            .0
            .iter()
            .filter(|(from, _)| active.contains(*from))
            .flat_map(|(from, edges)| {
                edges
                    .iter()
                    .filter(|e| e.species == species && e.method == "LEVEL")
                    .map(move |e| json!({"from":from,"level":e.level}))
            })
            .collect();
        entry["acquisition"] = json!({"status":if available.contains(&species) {"installed"} else {"pending"},
            "direct_sources":locations,"evolution_sources":evolution_sources,"evolutions":evolution});
        let path = output.join(format!("species/{}.html", species.to_ascii_lowercase()));
        let mut html = fs::read_to_string(&path)?;
        if pack.data().maps.contains_key("MnRochesterFossilLab")
            && matches!(
                species.as_str(),
                "TRILUME" | "PANOPLITE" | "SIPHLET" | "OBELURK"
            )
        {
            let start = html
                .find("<div class=\"habitat-note\">")
                .context("fossil habitat section")?;
            let paragraph = start
                + html[start..]
                    .find("<p>")
                    .context("fossil habitat paragraph")?
                + 3;
            let end = paragraph
                + html[paragraph..]
                    .find("</p>")
                    .context("fossil habitat end")?;
            html.replace_range(paragraph..end, "Fossil-bearing bluff layers along the Driftless Trail. Revived specimens are studied at Rochester Fossil Lab.");
        }
        html = html.replace(
            "<span class=\"eyebrow\">HABITAT NOTES</span>",
            "<span class=\"eyebrow\">NATURAL HABITAT</span>",
        );
        html = html.replace("<small>Encounter locations are design concepts for the Minnesota region; this guide does not imply they are currently catchable in the live game.</small>",
            "<small>See Where to find for locations in the current map.</small>");
        // Do not display old proposed route labels beside installed locations.
        if let Some(start) = html.find("<p class=\"planned-location\"") {
            let end = start
                + html[start..]
                    .find("</p>")
                    .context("planned paragraph end")?
                + 4;
            html.replace_range(start..end, "");
        }
        if let Some(start) = html.find("<section class=\"locations-panel\"") {
            let end = start
                + html[start..]
                    .find("</section>")
                    .context("location panel end")?
                + "</section>".len();
            html.replace_range(start..end, "");
        }
        let start = html
            .find("<section class=\"move-animation-entry\"")
            .or_else(|| html.find("<section class=\"move-panel\""))
            .context("entry insertion point")?;
        html.insert_str(
            start,
            &panel(
                if locations.is_empty() {
                    &evolution_sources
                } else {
                    &locations
                },
                &evolution,
                available.contains(&species),
            ),
        );
        if !html.contains("/locations.css") {
            html = html.replace(
                "</head>",
                &format!("<link rel=\"stylesheet\" href=\"{new}/locations.css\"></head>"),
            );
        }
        fs::write(&path, html)?;
        let card = format!(
            "<article class=\"pokemon-card\" data-id=\"{}\"",
            entry["identity"]["id"].as_str().context("species id")?
        );
        let card_start = index.find(&card).context("index card")?;
        let extra = locations
            .iter()
            .chain(&evolution_sources)
            .map(|s| format!(" {} {} {}", s.place, s.area, s.method))
            .collect::<String>();
        let tag_end = card_start + index[card_start..].find('>').context("card tag end")?;
        let attribute = "data-location-search=\"";
        if let Some(offset) = index[card_start..tag_end].find(attribute) {
            let start = card_start + offset + attribute.len();
            let end = start + index[start..].find('"').context("location search end")?;
            index.replace_range(start..end, &escape(&extra));
        } else {
            index.insert_str(tag_end, &format!(" {attribute}{}\"", escape(&extra)));
        }
    }
    catalog["acquisition_source"] = json!({"pack_content_hash":pack.identity()?.content_hash,
        "description":"Installed local Minnesota habitats and conditional gifts; earned access is not inferred from tables.",
        "available_species":available.len(),"pending_species":active.difference(&available).collect::<Vec<_>>()});
    fs::write(
        output.join("pokedex.json"),
        serde_json::to_vec_pretty(&catalog)?,
    )?;
    fs::write(output.join("locations.css"), CSS)?;
    fs::write(output.join("index.html"), index)?;
    let script_path = output.join("app.js");
    let script = fs::read_to_string(&script_path)?.replace(
        "normalize(data.search).includes(term)",
        "normalize(`${data.search} ${data.locationSearch || ''}`).includes(term)",
    );
    ensure!(
        script.contains("data.locationSearch"),
        "baseline search integration differs"
    );
    fs::write(script_path, script)?;
    println!(
        "Published locations for {} species; {} pending",
        active.len(),
        active.len() - available.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Location, add, panel};
    use std::collections::BTreeMap;

    #[test]
    fn same_habitat_merges_times_and_levels_but_keeps_methods() {
        let mut rows = BTreeMap::new();
        let grass = Location {
            method: "Grass".into(),
            map: "MnTest".into(),
            ..Default::default()
        };
        add(&mut rows, "OATLET", grass.clone(), 7, "day");
        add(&mut rows, "OATLET", grass, 8, "night");
        add(
            &mut rows,
            "OATLET",
            Location {
                method: "Surf".into(),
                map: "MnTest".into(),
                ..Default::default()
            },
            9,
            "day",
        );
        assert_eq!(rows["OATLET"].len(), 2);
        assert_eq!(rows["OATLET"][0].levels.len(), 2);
        let html = panel(&rows["OATLET"], &[], true);
        assert!(html.contains("Lv. 7–8"));
        assert!(!html.contains("location-pending"));
    }

    #[test]
    fn unplaced_species_and_untrusted_labels_are_visible_and_safe() {
        assert!(panel(&[], &[], false).contains("not yet been added"));
        let row = Location {
            place: "<script>bad</script>".into(),
            ..Default::default()
        };
        assert!(!panel(&[row], &[], true).contains("<script>"));
    }
}
