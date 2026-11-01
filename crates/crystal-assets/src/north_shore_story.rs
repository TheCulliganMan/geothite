//! Regional Grand Marais access through the coast gate and Gunflint walking seam.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use crystal_core::systems::script_flags::BADGE_ENGINE_FLAGS;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::authored_checkpoint::{Checkpoint, install as checkpoint};
use crate::authored_town_design::Town;
use crate::authored_town_geometry::{kind, reachable};
use crate::authored_town_publication::{CityIdentity, publish, replace};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const BASE: &str = "4b9dbd5018544719027de20886cafd1a662699d69c9e39e5f3a8562cc8be8bc2";
const MANIFEST: &str = "minnesota-north-shore-story-v1";
const TOWN: &str = "MnGrandMarais";
const GATE: &str = "MnGrandMaraisShoreGate";
const GATE_CONSTANT: &str = "MN_GRAND_MARAIS_SHORE_GATE";

#[derive(Deserialize)]
struct Patch {
    map: String,
    metatile: [u16; 2],
    expected: String,
    block: String,
}
#[derive(Deserialize)]
struct Pair {
    map: String,
    entrances: [[u16; 2]; 2],
    returns: [[u16; 2]; 2],
    first_active: u16,
    first_passive: u16,
    room_arrivals: [[u16; 2]; 2],
    room_passive_indices: [u16; 2],
    room_exits: [[u16; 2]; 2],
    room_exit_indices: [u16; 2],
}
#[derive(Deserialize)]
struct Guard {
    root: String,
    ids: [String; 2],
    bodies: [[u16; 2]; 2],
    inside: [[u16; 2]; 2],
    outside: [[u16; 2]; 2],
}

/// Connects the installed Grand Marais to the existing R11 and R12 roads.
///
/// Outbound checks require eight badges and the harbor signal; inbound care is
/// free. The authentic Ilex–Azalea room has two lanes with passive arrivals on
/// both sides. Native reef quadrants close the formerly unbound town sea.
/// The demo starter/ball/potion bundle becomes an optional local lamp-part offer.
/// Adds visit-selected recovery records for the two existing regional clinics.
/// No badge, regional story result or HOME change is granted.
///
/// # Errors
/// Returns an error for stale input, duplicate publication, unsafe geometry,
/// occupied identities, incompatible prerequisites, invalid native art/scripts
/// or failed pack verification. The borrowed input remains unchanged.
pub fn build_north_shore_story_modpack(
    base: &CompiledGamePack,
    source_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Value = serde_json::from_str(source_json)?;
    ensure!(
        base.identity.content_hash == BASE
            && source["base_content_hash"] == BASE
            && source["schema_version"] == 1
            && source["manifest"] == MANIFEST
            && source["reference_gate"] == "IlexForestAzaleaGate"
            && source["water_encounter_reference"] == "MnR11C00R00"
            && source["recovery_clinics"]
                == json!([
                    {"map":"MnDuluthPokecenter1F","nurse":"MN_DULUTH_NURSE","floor":[10,13]},
                    {"map":"MnGrandMaraisPokecenter","nurse":"MnGrandMaraisNurse","floor":[3,5]}
                ])
            && source["gate_name"] == GATE
            && source["gate_constant"] == GATE_CONSTANT
            && source["local_lamp_required"] == false
            && source["r12_link"]
                == json!({"town_edge":[[0,8],[0,9]],"route":"MnR12C01R01","route_edge":[[63,8],[63,9]],"town_offset":0})
            && source["required_flags"]
                == json!(
                    (1..=8)
                        .map(|i| format!("BADGE_{i}"))
                        .chain(["EVENT_MN_LIGHTHOUSE_SIGNAL".into()])
                        .collect::<Vec<_>>()
                )
            && !base.data.maps.contains_key(GATE),
        "exact Itasca input and independent local lamp required"
    );
    let mut pack = crate::marais_shore_art::install(base, &source)?;
    let water = pack
        .data
        .wild_encounters
        .get("MnR11C00R00")
        .context("North Shore water encounters")?
        .clone();
    let town_encounters = pack
        .data
        .wild_encounters
        .get_mut(TOWN)
        .context("installed town encounters")?;
    ensure!(
        town_encounters.water.is_none() && town_encounters.water_rate.is_none(),
        "previously absent harbor water table"
    );
    ensure!(
        water.water.is_some() && water.water_rate.is_some(),
        "native adjoining water table"
    );
    town_encounters.water = water.water;
    town_encounters.water_rate = water.water_rate;
    crate::marais_regional_services::allocate(&mut pack)?;
    let town_design: Town = serde_json::from_value(json!({
        "id":TOWN,"native_constant":"MN_GRAND_MARAIS","compiled_tileset":"mn_marais_shore",
        "dimensions_metatiles":[20,19],"ports":[],"residents":[],"signs":[],"doors":[],"interiors":[]
    }))?;
    let mut maps = [
        TOWN,
        "MnR11C00R00",
        "MnR12C01R01",
        "MnGrandMaraisFieldStation",
        "MnDuluthPokecenter1F",
        "MnGrandMaraisPokecenter",
    ]
    .into_iter()
    .map(|n| {
        Ok((
            n.to_owned(),
            pack.data
                .maps
                .get(n)
                .context("installed regional map")?
                .clone(),
        ))
    })
    .collect::<Result<BTreeMap<String, MapModule>>>()?;
    crate::marais_regional_services::recoveries(&mut pack, &mut maps)?;
    for p in serde_json::from_value::<Vec<Patch>>(source["patches"].clone())? {
        let m = maps.get_mut(&p.map).context("bounded patch map")?;
        let [x, y] = p.metatile;
        ensure!(
            x < m.attributes.width && y < m.attributes.height,
            "patch outside native map"
        );
        let i = usize::from(y) * usize::from(m.attributes.width) + usize::from(x);
        ensure!(
            m.blocks[i] == u16::from_str_radix(&p.expected, 16)?,
            "regional patch drift"
        );
        m.blocks[i] = u16::from_str_radix(&p.block, 16)?;
    }
    maps.get_mut(TOWN).context("town")?.attributes.tileset_name = "mn_marais_shore".into();
    crate::marais_regional_services::station(
        maps.get_mut("MnGrandMaraisFieldStation")
            .context("field station")?,
    )?;
    let mut gate = authoring::blank(&pack, "IlexForestAzaleaGate", GATE, GATE_CONSTANT)?;
    authoring::speech(
        &pack,
        &mut gate,
        "Attendant",
        "SPRITE_TEACHER",
        (5, 2),
        &[
            "GRAND MARAIS east.",
            "DULUTH: south.",
            "GUNFLINT leaves",
            "from town's north.",
        ],
    )?;
    let pairs: Vec<Pair> = serde_json::from_value(source["gate_pairs"].clone())?;
    ensure!(pairs.len() == 2, "both gate exteriors required");
    for (side, p) in pairs.iter().enumerate() {
        let m = maps.get_mut(&p.map).context("gate exterior")?;
        ensure!(
            m.events.warps.len() + 1 == usize::from(p.first_active)
                && p.first_passive == p.first_active + 2,
            "append active then passive exterior records"
        );
        let constant = m
            .attributes
            .map_constant
            .clone()
            .context("exterior constant")?;
        for lane in 0..2 {
            ensure!(
                kind(&pack, m, p.entrances[lane])?
                    == if side == 0 {
                        "WARP_CARPET_RIGHT"
                    } else {
                        "WARP_CARPET_LEFT"
                    }
                    && kind(&pack, m, p.returns[lane])? == "FLOOR"
                    && p.room_exit_indices[lane] == (side * 2 + lane + 1) as u16
                    && p.room_exits[lane] == [if side == 0 { 0 } else { 9 }, 4 + lane as u16]
                    && p.room_passive_indices[lane] == (side * 2 + lane + 5) as u16
                    && p.room_arrivals[lane] == [if side == 0 { 1 } else { 8 }, 4 + lane as u16],
                "native gate pairing differs"
            );
            let [x, y] = p.entrances[lane];
            authoring::warp(
                m,
                (x, y),
                GATE_CONSTANT,
                p.room_passive_indices[lane] as i16,
            );
            let [x, y] = p.room_exits[lane];
            authoring::warp(
                &mut gate,
                (x, y),
                &constant,
                (p.first_passive + lane as u16) as i16,
            );
        }
        for lane in 0..2 {
            let [x, y] = p.returns[lane];
            authoring::warp(m, (x, y), &constant, (p.first_passive + lane as u16) as i16);
        }
    }
    for p in &pairs {
        let constant = maps[&p.map]
            .attributes
            .map_constant
            .as_deref()
            .context("gate exterior constant")?;
        for lane in 0..2 {
            let [x, y] = p.room_arrivals[lane];
            ensure!(
                kind(&pack, &gate, [x, y])? == "FLOOR",
                "passive room arrival must be floor"
            );
            authoring::warp(
                &mut gate,
                (x, y),
                constant,
                (p.first_passive + lane as u16) as i16,
            );
        }
    }
    ensure!(
        gate.events.warps.len() == 8
            && gate.blocks == pack.data.maps["IlexForestAzaleaGate"].blocks,
        "original gate plan and eight warp records"
    );
    let city = maps.get_mut(TOWN).context("city checkpoints")?;
    let guards: Vec<Guard> = serde_json::from_value(source["checkpoints"].clone())?;
    ensure!(guards.len() == 2, "both public trails need checkpoints");
    for g in &guards {
        checkpoint(
            &pack,
            city,
            &Checkpoint {
                identifiers: &g.ids,
                bodies: g.bodies,
                inside: g.inside,
                outside: g.outside,
                root: &g.root,
                incoming_facing: "RIGHT",
                outward: "west",
                required_event: "EVENT_MN_LIGHTHOUSE_SIGNAL",
                badge_flags: &BADGE_ENGINE_FLAGS[..8],
                badge_lines: &["Bring eight BADGES".into(), "The trail is long.".into()],
                evidence_lines: &["Visit the DULUTH".into(), "harbor workshop.".into()],
            },
        )?;
    }
    for e in &mut city.events.coord_events {
        if guards.iter().any(|g| e.script_name.starts_with(&g.root)) {
            e.scene_id = "-1".into();
        }
    }
    let outside = [[0, 8], [0, 9], [4, 18], [4, 19], [5, 18], [5, 19]];
    let closed = reachable(&pack, city, [19, 12], true)?;
    ensure!(
        outside.iter().all(|p| !closed.contains(p)),
        "walk/Surf bypass around regional guards"
    );
    let bodies = city
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let native_closed =
        crate::authored_route_surface::reachable_map_with_surf(&pack, city, [19, 12], &bodies)?;
    ensure!(
        outside.iter().all(|p| !native_closed.contains(p)),
        "native ledge/Surf bypass around regional guards"
    );
    let mut open = city.clone();
    open.objects.retain(|o| {
        !o.object_identifier
            .as_ref()
            .is_some_and(|id| guards.iter().any(|g| g.ids.contains(id)))
    });
    let seen = reachable(&pack, &open, [19, 12], true)?;
    ensure!(
        outside.iter().all(|p| seen.contains(p)),
        "regional gate disconnected"
    );
    ensure!(
        reachable(&pack, &open, [19, 12], false)?.contains(&[3, 22]),
        "western quay must retain a dry public approach"
    );
    for [x, y] in seen {
        ensure!(
            (x > 0 && x < 39 && y > 0 && y < 37) || [[0, 8], [0, 9]].contains(&[x, y]),
            "unbound Marais walking/Surf boundary"
        );
    }
    for p in [[0, 8], [0, 9]] {
        ensure!(
            kind(&pack, city, p)? == "FLOOR",
            "Gunflint town mouth must be ordinary floor"
        );
    }
    crate::lake_country_geometry::connect(city, "west", "MnR12C01R01", 0)?;
    for p in [[63, 8], [63, 9]] {
        ensure!(
            kind(&pack, &maps["MnR12C01R01"], p)? == "FLOOR",
            "Gunflint route mouth must be ordinary floor"
        );
    }
    crate::lake_country_geometry::connect(
        maps.get_mut("MnR12C01R01").context("Gunflint")?,
        "east",
        TOWN,
        0,
    )?;
    crate::authored_town_scripts::finish(&pack, &mut gate, &town_design)?;
    publish(
        &mut pack,
        vec![(gate, [1, 4])],
        CityIdentity {
            group: "MINNESOTA_MARAIS_GATE",
            group_id: 80,
            landmark: "LANDMARK_MN_MARAIS_GATE",
            fallback_landmark: Some("LANDMARK_MN_GRAND_MARAIS"),
            constant: GATE_CONSTANT,
            name: "SHORE GATE",
            position: [140, 16],
            manifest: MANIFEST,
            town: "",
            clinic: "",
            gym: "",
        },
    )?;
    for (_, mut m) in maps {
        crate::authored_town_scripts::finish(&pack, &mut m, &town_design)?;
        replace(&mut pack, m)?;
    }
    for lane in 0..2 {
        for (name, target, outside, expected) in [
            (TOWN, "MnR12C01R01", [-1, 8 + lane], [63, 8 + lane]),
            ("MnR12C01R01", TOWN, [64, 8 + lane], [0, 8 + lane]),
        ] {
            let map = &pack.data.maps[name];
            let native =
                OverworldMapData::from_attributes(name, &map.attributes, map.blocks.clone());
            let trigger =
                connection_for_tile_checked(&native, TilePosition::new(outside[0], outside[1]))?
                    .context("native Gunflint connection")?;
            let actual = pack
                .data
                .resolve_connection_transition(&trigger)?
                .destination;
            ensure!(
                actual.map_name == target
                    && actual.tile == TilePosition::new(expected[0], expected[1]),
                "native Gunflint lane projection differs"
            );
        }
    }
    crate::authored_region_geometry::route_perimeter(
        &pack,
        "MnR11",
        [112, 192],
        &[[46, 191], [47, 191], [42, 4], [42, 5]],
    )?;
    crate::authored_region_geometry::route_perimeter(
        &pack,
        "MnR12",
        [128, 112],
        &[[0, 56], [0, 57], [127, 56], [127, 57]],
    )?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
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
