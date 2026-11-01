//! Public travel without repeated officer or investigation permission checks.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::authored_dialogue::{self, AuthoredDialoguePage};
use crate::grand_marais_modpack::authoring::command as c;
use crate::{CompiledGamePack, MapModule};

const MARKER: &str = "northstar-open-public-roads-v1";
const STORY: &str = "northstar-current-v27-consolidation-v1";
const SERVICE: &str = "EVENT_MN_COURIER_DEFEATED";

struct Gate {
    map: &'static str,
    root: &'static str,
    actors: [&'static str; 2],
    badges: &'static [usize],
    events: &'static [&'static str],
    direction: [&'static str; 2],
}

// Exact authored checkpoint identities. The partner safeguard is deliberately
// absent. Native field abilities and real League registration are separate.
const GATES: &[Gate] = &[
    Gate {
        map: "MnBemidji",
        root: "MnBemidjiItascaCheck",
        actors: [
            "MN_BEMIDJI_WEST_WARDEN_NORTH",
            "MN_BEMIDJI_WEST_WARDEN_SOUTH",
        ],
        badges: &[5],
        events: &[],
        direction: ["LAKE ITASCA", "ROUTE 14 west."],
    },
    Gate {
        map: "MnBemidji",
        root: "MnBemidjiIronCheck",
        actors: [
            "MN_BEMIDJI_EAST_WARDEN_NORTH",
            "MN_BEMIDJI_EAST_WARDEN_SOUTH",
        ],
        badges: &[6],
        events: &["EVENT_MN_HEADWATERS_REPORT"],
        direction: ["GRAND RAPIDS", "ROUTE 09 east."],
    },
    Gate {
        map: "MnBemidji",
        root: "MnCohesionBemidjiNorth",
        actors: ["MNCOHESIONBEMIDJINORTH_A", "MNCOHESIONBEMIDJINORTH_B"],
        badges: &[6],
        events: &[],
        direction: ["INTERNATIONAL", "FALLS is north."],
    },
    Gate {
        map: "MnBoundaryWaters",
        root: "MnBoundaryGunflintCheck",
        actors: [
            "MN_BOUNDARY_GUNFLINT_GUARD_NORTH",
            "MN_BOUNDARY_GUNFLINT_GUARD_SOUTH",
        ],
        badges: &[0, 1, 2, 3, 4, 5, 6, 7],
        events: &["EVENT_MN_LIGHTHOUSE_SIGNAL"],
        direction: ["GRAND MARAIS", "Follow GUNFLINT."],
    },
    Gate {
        map: "MnBoundaryWaters",
        root: "MnBoundaryLeagueCheck",
        actors: [
            "MN_BOUNDARY_LEAGUE_GUARD_WEST",
            "MN_BOUNDARY_LEAGUE_GUARD_EAST",
        ],
        badges: &[0, 1, 2, 3, 4, 5, 6, 7],
        events: &["EVENT_MN_LIGHTHOUSE_SIGNAL", "EVENT_MN_SURVEY_RESTORED"],
        direction: ["ELY is south.", "Visit the LEAGUE."],
    },
    Gate {
        map: "MnBoundaryWaters",
        root: "MnBoundaryVoyageursCheck",
        actors: [
            "MN_BOUNDARY_VOYAGEURS_GUARD_NORTH",
            "MN_BOUNDARY_VOYAGEURS_GUARD_SOUTH",
        ],
        badges: &[0, 1, 2, 3, 4, 5, 6, 7],
        events: &["EVENT_MN_LIGHTHOUSE_SIGNAL"],
        direction: ["VOYAGEURS", "Follow ROUTE 20."],
    },
    Gate {
        map: "MnBrainerd",
        root: "MnBrainerdLakeCheck",
        actors: [
            "MN_BRAINERD_NORTH_WARDEN_WEST",
            "MN_BRAINERD_NORTH_WARDEN_EAST",
        ],
        badges: &[5],
        events: &[],
        direction: ["LEECH LAKE", "ROUTE 07 north."],
    },
    Gate {
        map: "MnBrainerd",
        root: "MnCohesionBrainerdWest",
        actors: ["MNCOHESIONBRAINERDWEST_A", "MNCOHESIONBRAINERDWEST_B"],
        badges: &[5],
        events: &[],
        direction: ["DETROIT LAKES", "ROUTE 15 west."],
    },
    Gate {
        map: "MnBrainerd",
        root: "MnCohesionBrainerdEast",
        actors: ["MNCOHESIONBRAINERDEAST_A", "MNCOHESIONBRAINERDEAST_B"],
        badges: &[5],
        events: &[],
        direction: ["MILLE LACS", "ROUTE 18 east."],
    },
    Gate {
        map: "MnDuluth",
        root: "MnDuluthRangeCheck",
        actors: ["MN_DULUTH_RANGE_GUARD_NORTH", "MN_DULUTH_RANGE_GUARD_SOUTH"],
        badges: &[6],
        events: &["EVENT_MN_DEPOT_CLEARED"],
        direction: ["GRAND RAPIDS", "ROUTE 10 west."],
    },
    Gate {
        map: "MnDuluth",
        root: "MnDuluthShoreCheck",
        actors: ["MN_DULUTH_SHORE_GUARD_WEST", "MN_DULUTH_SHORE_GUARD_EAST"],
        badges: &[0, 1, 2, 3, 4, 5, 6, 7],
        events: &["EVENT_MN_LIGHTHOUSE_SIGNAL"],
        direction: ["GRAND MARAIS", "NORTH SHORE path."],
    },
    Gate {
        map: "MnGrandMarais",
        root: "MnCohesionMaraisWest",
        actors: ["MNCOHESIONMARAISWEST_A", "MNCOHESIONMARAISWEST_B"],
        badges: &[0, 1, 2, 3, 4, 5, 6, 7],
        events: &["EVENT_MN_LIGHTHOUSE_SIGNAL"],
        direction: ["DULUTH", "NORTH SHORE path."],
    },
    Gate {
        map: "MnGrandMarais",
        root: "MnCohesionMaraisEast",
        actors: ["MNCOHESIONMARAISEAST_A", "MNCOHESIONMARAISEAST_B"],
        badges: &[0, 1, 2, 3, 4, 5, 6, 7],
        events: &["EVENT_MN_LIGHTHOUSE_SIGNAL"],
        direction: ["BOUNDARY WATERS", "Follow GUNFLINT."],
    },
    Gate {
        map: "MnGrandRapids",
        root: "MnGrandRapidsWestCheck",
        actors: [
            "MN_GRAND_RAPIDS_WEST_GUARD_NORTH",
            "MN_GRAND_RAPIDS_WEST_GUARD_SOUTH",
        ],
        badges: &[6],
        events: &["EVENT_MN_HEADWATERS_REPORT"],
        direction: ["BEMIDJI", "ROUTE 09 west."],
    },
    Gate {
        map: "MnGrandRapids",
        root: "MnGrandRapidsEastCheck",
        actors: [
            "MN_GRAND_RAPIDS_EAST_GUARD_NORTH",
            "MN_GRAND_RAPIDS_EAST_GUARD_SOUTH",
        ],
        badges: &[6],
        events: &["EVENT_MN_DEPOT_CLEARED"],
        direction: ["DULUTH", "ROUTE 10 east."],
    },
    Gate {
        map: "MnLakeItasca",
        root: "MnCohesionItascaWest",
        actors: ["MNCOHESIONITASCAWEST_A", "MNCOHESIONITASCAWEST_B"],
        badges: &[5],
        events: &[],
        direction: ["DETROIT LAKES", "ROUTE 19 west."],
    },
    Gate {
        map: "MnLeechLake",
        root: "MnLeechLakeCheck",
        actors: ["MN_LEECH_NORTH_WARDEN_WEST", "MN_LEECH_NORTH_WARDEN_EAST"],
        badges: &[5],
        events: &[],
        direction: ["BEMIDJI", "ROUTE 08 north."],
    },
    Gate {
        map: "MnMankato",
        root: "MnMKRiverCheck",
        actors: ["MN_MK_RIVER_WARDEN_WEST", "MN_MK_RIVER_WARDEN_EAST"],
        badges: &[1],
        events: &[],
        direction: ["MINNEAPOLIS", "ROUTE 03 north."],
    },
    Gate {
        map: "MnMinneapolis",
        root: "MnMPEastCheck",
        actors: ["MN_MP_EAST_WARDEN_NORTH", "MN_MP_EAST_WARDEN_SOUTH"],
        badges: &[2],
        events: &[],
        direction: ["ST PAUL", "ROUTE 04 east."],
    },
    Gate {
        map: "MnMinneapolis",
        root: "MnMPNorthCheck",
        actors: ["MN_MP_NORTH_WARDEN_WEST", "MN_MP_NORTH_WARDEN_EAST"],
        badges: &[2, 3],
        events: &[],
        direction: ["ST CLOUD", "ROUTE 05 north."],
    },
    Gate {
        map: "MnRochester",
        root: "MnRCPrairieCheck",
        actors: ["MN_RC_PRAIRIE_WARDEN_NORTH", "MN_RC_PRAIRIE_WARDEN_SOUTH"],
        badges: &[0],
        events: &[],
        direction: ["MANKATO", "ROUTE 02 west."],
    },
    Gate {
        map: "MnStCloud",
        root: "MnSCWindCheck",
        actors: ["MN_SC_NORTH_WARDEN_WEST", "MN_SC_NORTH_WARDEN_EAST"],
        badges: &[4],
        events: &[],
        direction: ["BRAINERD", "ROUTE 06 north."],
    },
];

/// Opens the reviewed public road checkpoints after full story consolidation.
///
/// Keeps actor slots and numeric identities, but hides the44 road actors at
/// every entry, reload and Continue. Their coordinate interception is removed;
/// they do not become new trainers, trees, permits or replacement roadblocks.
/// Return trains still require a previously visited destination. The separate
/// partner safeguard, worksite actions, finale controls and League registration
/// remain exact. No terrain, ice puzzle, encounter, reward or event bit changes.
///
/// # Errors
/// Rejects a missing story marker, repeated publication, changed native gate
/// programs/actors, unsupported geography, unrelated mutations or invalid output.
/// The borrowed pack and existing saves remain unchanged on failure.
pub fn open_northstar_public_roads(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        base.report.manifests.iter().any(|m| m == STORY),
        "full consolidated story required before opening public roads"
    );
    ensure!(
        !base.report.manifests.iter().any(|m| m == MARKER),
        "public roads already revised"
    );
    crate::verify_northstar_publication_geometry(base)?;
    crate::validate_compact_superior_geography(base)?;
    ensure!(GATES.len() == 22, "reviewed22-road scope");
    let mut pack = base.clone();
    let mut maps = BTreeMap::<String, (MapModule, BTreeSet<String>, BTreeSet<String>)>::new();
    for gate in GATES {
        let (map, changed, remove) = entry(&mut maps, base, gate.map)?;
        ensure!(
            map.scripts.get(gate.root) == Some(&expected_gate(gate)),
            "changed checkpoint anchor: {}/{}",
            gate.map,
            gate.root
        );
        for id in gate.actors {
            ensure!(
                map.objects
                    .iter()
                    .filter(|o| o.object_identifier.as_deref() == Some(id) && o.script == gate.root)
                    .count()
                    == 1,
                "one exact road actor required: {id}"
            );
        }
        ensure!(
            map.events
                .coord_events
                .iter()
                .any(|e| e.script_name == gate.root),
            "missing checkpoint approach"
        );
        let open = json!([
            c("disappear", &[gate.actors[0]]),
            c("disappear", &[gate.actors[1]]),
            c("end", &[])
        ]);
        for label in [
            gate.root.to_owned(),
            format!("{}Inward", gate.root),
            format!("{}Reconcile", gate.root),
            format!("{}Show", gate.root),
        ] {
            ensure!(
                map.scripts.contains_key(&label),
                "checkpoint helper missing: {label}"
            );
            map.scripts.insert(label.clone(), open.clone());
            changed.insert(label);
        }
        remove.extend([gate.root.into(), format!("{}Inward", gate.root)]);
        // Retire old permit/refusal wording too, including stale menu/actor
        // contexts: these destinations now have ordinary public directions.
        let labels = map
            .scripts
            .keys()
            .filter(|l| l.starts_with(gate.root) && l.ends_with("Text"))
            .cloned()
            .collect::<Vec<_>>();
        for label in labels {
            text(map, changed, &label, &[&gate.direction])?;
        }
    }
    for (name, (map, changed, remove)) in &mut maps {
        let ids = GATES
            .iter()
            .filter(|g| g.map == name.as_str())
            .flat_map(|g| g.actors)
            .collect::<BTreeSet<_>>();
        // No old Show helper or callback may make a retired blocker visible.
        for (label, body) in &mut map.scripts {
            if let Some(rows) = body.as_array_mut() {
                for row in rows {
                    if row["command"] == "appear"
                        && row["args"][0].as_str().is_some_and(|id| ids.contains(id))
                    {
                        row["command"] = "disappear".into();
                        changed.insert(label.clone());
                    }
                }
            }
        }
        let callback = format!("MnPublicRoads{}Open", name.trim_start_matches("Mn"));
        ensure!(
            !map.scripts.contains_key(&callback),
            "occupied public-road callback"
        );
        let mut body = ids
            .into_iter()
            .map(|id| c("disappear", &[id]))
            .collect::<Vec<_>>();
        body.push(c("endcallback", &[]));
        map.scripts.insert(callback.clone(), Value::Array(body));
        changed.insert(callback.clone());
        // TILES runs on Continue, battle reload and submenu return; OBJECTS
        // runs after object setup on walking/warp entry. Both use production
        // callbacks and preserve all existing callbacks in their original order.
        for kind in ["MAPCALLBACK_TILES", "MAPCALLBACK_OBJECTS"] {
            crate::northstar_lair_authoring::callback(map, kind, &callback);
        }
        remove_coordinates(map, remove);
    }
    falls(&mut maps, base)?;
    river(&mut maps, base)?;
    railway(&mut maps, base)?;
    for (name, (mut map, _, _)) in maps {
        let events = map.map_event_section_commands.clone();
        let sections = map.map_script_section_commands.clone();
        crate::ely_league_rooms::finish(&pack, &mut map)?;
        map.script_audio_commands = crate::parse_script_audio_commands(&name, &map.scripts)?;
        map.map_event_section_commands = events;
        map.map_script_section_commands = sections;
        crate::authored_town_publication::replace(&mut pack, map)?;
    }
    // A script-only publisher must never synchronize unrelated legacy split
    // geography or actor aliases merely because it touched a text program.
    pack.data
        .map_attributes
        .clone_from(&base.data.map_attributes);
    pack.data.map_blocks.clone_from(&base.data.map_blocks);
    pack.data
        .map_dimensions
        .clone_from(&base.data.map_dimensions);
    pack.data
        .runtime_map_metadata
        .clone_from(&base.data.runtime_map_metadata);
    pack.data.npcs.clone_from(&base.data.npcs);
    verify_preservation(base, &pack)?;
    pack.report.manifests.push(MARKER.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    Ok(pack)
}

type Changes = BTreeMap<String, (MapModule, BTreeSet<String>, BTreeSet<String>)>;

fn entry<'a>(
    maps: &'a mut Changes,
    base: &CompiledGamePack,
    name: &str,
) -> Result<&'a mut (MapModule, BTreeSet<String>, BTreeSet<String>)> {
    if !maps.contains_key(name) {
        maps.insert(
            name.into(),
            (
                base.data
                    .maps
                    .get(name)
                    .context("reviewed road map")?
                    .clone(),
                BTreeSet::new(),
                BTreeSet::new(),
            ),
        );
    }
    maps.get_mut(name).context("prepared road map")
}

fn expected_gate(gate: &Gate) -> Value {
    let mut rows = Vec::new();
    for (i, index) in gate.badges.iter().enumerate() {
        let branch = if gate.root == "MnMPNorthCheck" && i == 0 {
            "MnMPNorthNeedCurrent".into()
        } else {
            format!("{}Badges", gate.root)
        };
        rows.extend([
            c(
                "checkflag",
                &[crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[*index]],
            ),
            c("iffalse", &[&branch]),
        ]);
    }
    for event in gate.events {
        let suffix = if gate.root.starts_with("MnBoundary") {
            if *event == "EVENT_MN_SURVEY_RESTORED" {
                "Restored"
            } else {
                "Signal"
            }
        } else {
            "Evidence"
        };
        rows.extend([
            c("checkevent", &[event]),
            c("iffalse", &[&format!("{}{suffix}", gate.root)]),
        ]);
    }
    rows.extend([
        c("disappear", &[gate.actors[0]]),
        c("disappear", &[gate.actors[1]]),
        c("end", &[]),
    ]);
    Value::Array(rows)
}

fn remove_coordinates(map: &mut MapModule, roots: &BTreeSet<String>) {
    map.events
        .coord_events
        .retain(|e| !roots.contains(&e.script_name));
    map.map_event_section_commands.retain(|row| {
        !(row.command == "coord_event" && row.args.get(3).is_some_and(|s| roots.contains(s)))
    });
    for (i, row) in map.map_event_section_commands.iter_mut().enumerate() {
        row.command_index = i;
    }
}

fn text(
    map: &mut MapModule,
    changed: &mut BTreeSet<String>,
    label: &str,
    pages: &[&[&str]],
) -> Result<()> {
    ensure!(
        map.scripts.contains_key(label),
        "reviewed road text missing: {}/{label}",
        map.id
    );
    let pages = pages
        .iter()
        .map(|lines| AuthoredDialoguePage {
            lines: lines.iter().map(|line| (*line).into()).collect(),
        })
        .collect::<Vec<_>>();
    map.scripts
        .insert(label.into(), authored_dialogue::compile(&pages)?);
    changed.insert(label.into());
    Ok(())
}

fn falls(maps: &mut Changes, base: &CompiledGamePack) -> Result<()> {
    let (map, changed, remove) = entry(maps, base, "MnRainyRiverIcefields")?;
    let body = map
        .scripts
        .get("MnFallsTrailCheck")
        .and_then(Value::as_array)
        .context("Falls public trail check")?;
    ensure!(
        body.first() == Some(&c("readvar", &["VAR_FACING"]))
            && body.get(1) == Some(&c("ifnotequal", &["RIGHT", "MnFallsTrailEnd"])),
        "Falls direction anchor changed"
    );
    let checks = body
        .iter()
        .filter(|r| r["command"] == "checkflag" || r["command"] == "checkevent")
        .collect::<Vec<_>>();
    ensure!(
        checks.len() == 10
            && checks[..8].iter().enumerate().all(|(i, r)| **r
                == c(
                    "checkflag",
                    &[crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS[i]]
                ))
            && *checks[8] == c("checkevent", &["EVENT_MN_BORDER_REPORT"])
            && *checks[9] == c("checkevent", &["EVENT_MN_LIGHTHOUSE_SIGNAL"]),
        "exact Falls trail permission checks required"
    );
    map.scripts
        .insert("MnFallsTrailCheck".into(), json!([c("end", &[])]));
    changed.insert("MnFallsTrailCheck".into());
    remove.insert("MnFallsTrailCheck".into());
    remove_coordinates(map, remove);
    for label in ["MnFallsdenied_linesText", "MnFallswarden_linesText"] {
        text(
            map,
            changed,
            label,
            &[
                &["The public trail", "joins VOYAGEURS."],
                &["The FALLS ranger", "welcomes reports."],
            ],
        )?;
    }
    Ok(())
}

fn river(maps: &mut Changes, base: &CompiledGamePack) -> Result<()> {
    let (map, changed, remove) = entry(maps, base, "MnMinneapolis")?;
    for (root, denial) in [
        ("MnMPUpperPoolBuoyCheck", "MnMPUpperPoolBuoyDenied"),
        ("MnMPUpperPoolBuoyRefresh", "MnMPUpperPoolBuoyDone"),
    ] {
        let expected = json!([
            c("checkflag", &["ENGINE_PLAINBADGE"]),
            c("iffalse", &[denial]),
            c("checkflag", &["ENGINE_FOGBADGE"]),
            c("iffalse", &[denial]),
            c("sjump", &["MnMPUpperPoolBuoyOpen"])
        ]);
        ensure!(
            map.scripts.get(root) == Some(&expected),
            "exact upper-pool buoy anchor required"
        );
        map.scripts
            .insert(root.into(), json!([c("sjump", &["MnMPUpperPoolBuoyOpen"])]));
        changed.insert(root.into());
    }
    ensure!(
        map.scripts.get("MnMPUpperPoolBuoyOpen")
            == Some(&json!([
                c("changeblock", &["32", "2", "88"]),
                c("changeblock", &["34", "2", "53"]),
                c("changeblock", &["36", "2", "89"]),
                c("end", &[])
            ])),
        "reviewed connected north river aperture required"
    );
    remove.insert("MnMPUpperPoolBuoyCheck".into());
    remove_coordinates(map, remove);
    text(
        map,
        changed,
        "MnMPUpperPoolBuoyDeniedText",
        &[
            &["The river runs", "north to ST CLOUD."],
            &["Use SURF at the", "open landing."],
        ],
    )?;
    text(
        map,
        changed,
        "MnMPNorthCurrentText",
        &[&["ST PAUL is east.", "ST CLOUD is north."]],
    )?;
    Ok(())
}

fn railway(maps: &mut Changes, base: &CompiledGamePack) -> Result<()> {
    for (map_name, id) in [
        ("MnDetroitLakesStation", "DETROIT_LAKES"),
        ("MnStaplesStation", "STAPLES"),
        ("MnStCloudStation", "ST_CLOUD"),
        ("MnStPaulUnionDepot", "ST_PAUL"),
        ("MnWinonaStation", "WINONA"),
    ] {
        let (map, changed, _) = entry(maps, base, map_name)?;
        let prefix = format!("MnGlacier{id}");
        let denied = format!("{prefix}Denied");
        let conductor = map
            .objects
            .iter()
            .filter(|o| {
                map.scripts
                    .get(&o.script)
                    .and_then(Value::as_array)
                    .is_some_and(|rows| {
                        rows.get(2) == Some(&c("checkevent", &[SERVICE]))
                            && rows.get(3) == Some(&c("iffalse", &[&denied]))
                    })
            })
            .map(|o| o.script.clone())
            .collect::<Vec<_>>();
        ensure!(
            conductor.len() == 1,
            "one exact Glacier conductor required: {map_name}"
        );
        let rows = map
            .scripts
            .get_mut(&conductor[0])
            .and_then(Value::as_array_mut)
            .context("conductor program")?;
        rows.drain(2..4);
        changed.insert(conductor[0].clone());
        let rides = map
            .scripts
            .keys()
            .filter(|l| l.starts_with(&format!("{prefix}Ride")))
            .cloned()
            .collect::<Vec<_>>();
        ensure!(rides.len() == 4, "four exact train destinations required");
        for label in rides {
            let rows = map
                .scripts
                .get_mut(&label)
                .and_then(Value::as_array_mut)
                .context("native ride")?;
            ensure!(
                rows.first() == Some(&c("checkevent", &[SERVICE]))
                    && rows.get(1) == Some(&c("iffalse", &[&denied]))
                    && rows.get(2).is_some_and(|r| r["command"] == "checkevent"
                        && r["args"][0]
                            .as_str()
                            .is_some_and(|s| s.starts_with("EVENT_MN_STATION_")))
                    && rows.get(3) == Some(&c("iffalse", &[&format!("{prefix}Unknown")])),
                "native destination discovery guard must remain"
            );
            rows.drain(0..2);
            changed.insert(label);
        }
        text(
            map,
            changed,
            &format!("{prefix}DeniedText"),
            &[
                &["Return trips link", "visited stations."],
                &["Ask the conductor", "to choose a stop."],
            ],
        )?;
    }
    let (map, changed, _) = entry(maps, base, "MnStPaulUnionDepot")?;
    text(
        map,
        changed,
        "MN_ST_PAUL_DEPOT_CLERKText",
        &[
            &["CLERK: Welcome to", "UNION DEPOT."],
            &["The conductor can", "take you back to"],
            &["stations you have", "visited on foot."],
        ],
    )?;
    let (map, changed, _) = entry(maps, base, "MnDuluth")?;
    text(
        map,
        changed,
        "MnDuluth_NORTH_GATE_SIGNText",
        &[&["NORTH SHORE PATH", "GRAND MARAIS"]],
    )?;
    let (map, changed, _) = entry(maps, base, "MnLeagueLobby")?;
    text(
        map,
        changed,
        "MnElySurveyDeniedText",
        &[
            &["REGISTRAR: Finish", "the rescue first."],
            &["Help the crews in", "BOUNDARY WATERS."],
            &["Then return here", "to register."],
        ],
    )?;
    Ok(())
}

fn verify_preservation(base: &CompiledGamePack, pack: &CompiledGamePack) -> Result<()> {
    for (name, old) in &base.data.maps {
        let new = pack.data.maps.get(name).context("retained map")?;
        ensure!(
            new.attributes == old.attributes
                && new.blocks == old.blocks
                && new.objects == old.objects
                && new.scenes == old.scenes
                && new.events.warps == old.events.warps
                && new.events.bg_events == old.events.bg_events,
            "public travel changed physical geography or actor identities: {name}"
        );
    }
    for name in [
        "MnWinona",
        "MnBoundaryWatersSurveyShelter",
        "MnBoundaryIntake",
        "MnR09Cave1F",
        "MnGrandRapidsSurveyDepot",
        "MnWinonaFieldStation",
    ] {
        ensure!(
            pack.data.maps[name] == base.data.maps[name],
            "essential quest, starter or League program changed: {name}"
        );
    }
    for (label, body) in &base.data.maps["MnLeagueLobby"].scripts {
        if label != "MnElySurveyDeniedText" {
            ensure!(
                pack.data.maps["MnLeagueLobby"].scripts.get(label) == Some(body),
                "real League registration changed: {label}"
            );
        }
    }
    let mut data = pack.data.clone();
    data.maps.clone_from(&base.data.maps);
    data.map_scripts.clone_from(&base.data.map_scripts);
    ensure!(
        data == base.data,
        "public road overlay changed unrelated catalogs, caches or progression identities"
    );
    ensure!(
        pack.runtime_files == base.runtime_files
            && pack.audio_manifest == base.audio_manifest
            && pack.compiled_audio == base.compiled_audio
            && pack.audio_compression == base.audio_compression,
        "public travel must preserve all art and audio"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn public_scope_excludes_partner_and_league_registration() {
        assert_eq!(super::GATES.len(), 22);
        assert!(
            super::GATES
                .iter()
                .all(|g| g.root != "MnWTrailCheck" && g.map != "MnLeagueLobby")
        );
        for gate in super::GATES {
            assert!(
                super::expected_gate(gate)
                    .as_array()
                    .expect("native commands")
                    .iter()
                    .all(|r| r["command"] != "setevent")
            );
        }
    }

    #[test]
    #[ignore = "requires exact consolidated story in CRYSTAL_PUBLIC_ROADS_BASE; reads external pack at runtime"]
    fn public_roads_open_every_reviewed_checkpoint_without_granting_progression() {
        let path =
            std::env::var_os("CRYSTAL_PUBLIC_ROADS_BASE").expect("explicit consolidated parent");
        let base = crate::read_verified_compiled_game_pack(std::path::PathBuf::from(path))
            .expect("verified parent");
        let result =
            super::open_northstar_public_roads(&base).expect("bounded public-road overlay");
        for gate in super::GATES {
            let map = &result.data.maps[gate.map];
            let root = map.scripts[gate.root].as_array().expect("open road");
            assert_eq!(root.len(), 3);
            assert!(root.iter().all(|r| !matches!(
                r["command"].as_str(),
                Some("checkflag" | "checkevent" | "setevent" | "applymovement")
            )));
            assert!(
                !map.events
                    .coord_events
                    .iter()
                    .any(|e| e.script_name == gate.root
                        || e.script_name == format!("{}Inward", gate.root))
            );
            assert_eq!(map.objects, base.data.maps[gate.map].objects);
            let callback = format!("MnPublicRoads{}Open", gate.map.trim_start_matches("Mn"));
            for kind in ["MAPCALLBACK_TILES", "MAPCALLBACK_OBJECTS"] {
                assert!(
                    map.map_script_section_commands
                        .iter()
                        .any(|r| r.command == "callback" && r.args == [kind, &callback])
                );
            }
        }
        assert_eq!(
            result.data.story_event_script_constants,
            base.data.story_event_script_constants
        );
        assert_eq!(
            result.data.maps["MnLeagueLobby"].scripts["MnElyRegister"],
            base.data.maps["MnLeagueLobby"].scripts["MnElyRegister"]
        );
        assert!(
            super::open_northstar_public_roads(&result).is_err(),
            "duplicate publication must fail"
        );
    }
}
