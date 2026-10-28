//! Ely's complete native rest camp and the five-battle League continuation.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;
use serde_json::Value;

use crate::authored_route_design::{Door, Sign};
use crate::authored_route_layout::Layout;
use crate::authored_town_design::Actor;
use crate::authored_town_geometry::{kind, reachable};
use crate::authored_town_publication::{CityIdentity, publish};
use crate::ely_league_rooms::{self as rooms, MAPS};
use crate::ely_league_scripts::{self as scripts, Member};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const BASE: &str = "ab793e8df7a4f16f4b811aad5e2fc5e34c59fa394c33907f718e5a9fd4ea3c8f";
const MANIFEST: &str = "minnesota-ely-league-v1";

#[path = "ely_league_rebase.rs"]
mod rebase;

#[derive(Deserialize)]
struct Town {
    id: String,
    native_constant: String,
    dimensions_metatiles: [u16; 2],
    compiled_tileset: String,
    residents: Vec<Actor>,
    signs: Vec<Sign>,
    doors: Vec<Door>,
}

/// Publishes Ely, its free public services and the sequential native League.
///
/// Registration independently checks eight badges and the restored survey link.
/// Battle victories advance persistent run memory. Leaving or blacking out ends
/// that run; Hall of Fame history and completed story flags survive. All new
/// rooms use unchanged source floorplans, sprites, collisions and battle rules.
///
/// # Errors
/// Returns an error for stale input, conflicting identities, inconsistent source
/// geometry/parties, unsafe connections, or failed native script/pack validation.
/// The borrowed input remains unchanged.
pub fn build_ely_league_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    rebase::parent(base)?;
    let town: Town = serde_json::from_str(town_json)?;
    let layout: Layout = serde_json::from_str(layout_json)?;
    ensure!(
        town.id == "MnEly"
            && town.native_constant == "MN_ELY"
            && town.dimensions_metatiles == [20, 18]
            && town.compiled_tileset == "kanto"
            && layout.id == town.id
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && layout.compiled_tileset == town.compiled_tileset
            && layout.schema_version == 2,
        "complete explicit Ely source required"
    );
    let story: Value = serde_json::from_str(story_json)?;
    rebase::sources(base, town_json, layout_json, &story)?;
    let members: Vec<Member> = serde_json::from_value(story["league"].clone())?;
    ensure!(
        members.len() == 5
            && members
                .iter()
                .zip(&MAPS[3..8])
                .all(|(m, (name, _, _))| m.map == *name),
        "canonical five-member League roster required"
    );
    let mut pack = base.clone();
    scripts::allocate(&mut pack, &members)?;
    ensure!(
        pack.data
            .marts
            .0
            .insert(
                "MART_MN_ELY".into(),
                [
                    "ULTRA_BALL",
                    "HYPER_POTION",
                    "MAX_POTION",
                    "FULL_HEAL",
                    "REVIVE",
                    "MAX_REPEL",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            )
            .is_none(),
        "Ely mart must be a fresh identity"
    );
    let mut map = authoring::blank(&pack, "Route23", &town.id, &town.native_constant)?;
    map.attributes.width = 20;
    map.attributes.height = 18;
    map.attributes.border_block = 0x0f;
    map.blocks = layout.compile()?;
    crate::authored_town_scripts::ordinary(
        &pack,
        &mut map,
        &town.residents,
        &town.signs,
        "MN_ELY_POKECENTER_1F",
        "MART_MN_ELY",
    )?;
    ensure!(town.doors.len() == 4, "all four real facade lanes required");
    for (i, d) in town.doors.iter().enumerate() {
        ensure!(
            usize::from(d.warp_index) == i + 1 && d.return_warp_index == Some(d.warp_index + 4),
            "active facade order followed by passive floor returns"
        );
        let target = MAPS
            .iter()
            .find(|(n, _, _)| *n == d.target)
            .context("registered exterior destination")?
            .1;
        authoring::warp(&mut map, (d.tile[0], d.tile[1]), target, d.target_warp);
    }
    for d in &town.doors {
        let target = MAPS
            .iter()
            .find(|(n, _, _)| *n == d.target)
            .context("registered exterior destination")?
            .1;
        authoring::warp(
            &mut map,
            (d.return_tile[0], d.return_tile[1]),
            target,
            d.target_warp,
        );
    }
    validate_exterior(&pack, &map, &town)?;
    rooms::finish(&pack, &mut map)?;
    let mut modules = vec![(map, [24, 4])];
    modules.extend(rooms::build(&pack, &members)?);
    for d in &town.doors {
        let room = modules
            .iter()
            .find(|(m, _)| m.id == d.target)
            .context("paired indoor map")?;
        let arrival = room
            .0
            .events
            .warps
            .iter()
            .find(|w| i16::try_from(w.index).ok() == Some(d.target_warp))
            .context("paired passive indoor arrival")?;
        ensure!(
            [arrival.x, arrival.y] == d.arrival_tile
                && kind(&pack, &room.0, d.arrival_tile)? == "FLOOR",
            "authored facade arrival differs from native room"
        );
    }
    for (map, _) in &modules {
        for actor in &map.objects {
            let sprite = actor
                .sprite
                .strip_prefix("SPRITE_")
                .context("native overworld sprite token")?
                .to_ascii_lowercase();
            ensure!(
                pack.runtime_files
                    .contains_key(&format!("gfx/sprites/{sprite}.png")),
                "{} requires original sprite art {sprite}",
                map.id
            );
        }
    }
    for (room, start) in &modules[1..] {
        ensure!(
            kind(&pack, room, *start)? == "FLOOR",
            "{} passive arrival must be native floor",
            room.id
        );
        let seen = reachable(&pack, room, *start, false)?;
        let mut opened = room.clone();
        if room.id.starts_with("MnLeagueRoom") {
            opened.blocks[7] = 0x16;
        } else if room.id == "MnLeagueChampion" {
            opened.blocks[2] = 0x0b;
        }
        for w in &room.events.warps {
            let k = kind(&pack, &opened, [w.x, w.y])?;
            ensure!(
                k != "WALL",
                "{} warp{} sits in native wall",
                room.id,
                w.index
            );
        }
        ensure!(
            room.objects
                .iter()
                .all(|o| kind(&pack, room, [o.x, o.y]).is_ok_and(|k| k == "FLOOR")),
            "{} actors require native floor",
            room.id
        );
        ensure!(
            room.objects.iter().all(|o| [
                [o.x.saturating_sub(1), o.y],
                [o.x + 1, o.y],
                [o.x, o.y + 1]
            ]
            .iter()
            .any(|p| seen.contains(p)))
                || room.id == "MnLeagueLobby"
                || room.id == "MnElyPokecenter1F"
                || room.id == "MnElyMart",
            "{} actors need reachable approaches",
            room.id
        );
    }
    publish(
        &mut pack,
        modules,
        CityIdentity {
            group: "MINNESOTA_ELY",
            group_id: 81,
            landmark: "LANDMARK_MN_ELY",
            fallback_landmark: Some("LANDMARK_MN_LEAGUE_APPROACH"),
            constant: "MN_ELY",
            name: "ELY",
            position: [68, 22],
            manifest: MANIFEST,
            town: "MnEly",
            clinic: "MnElyPokecenter1F",
            gym: "",
        },
    )?;
    let spawn = pack
        .data
        .runtime_spawn_points
        .values()
        .find(|s| s.map_name == "MnElyPokecenter1F")
        .context("Ely recovery")?
        .identifier;
    pack.data
        .story_event_script_constants
        .maps
        .entry("MnLeagueHallOfFame".into())
        .or_default()
        .insert("GEOTHITE_POST_CREDITS_SPAWN".into(), i64::from(spawn));
    connect(&mut pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.report.maps = pack.data.maps.len();
    rebase::validate(base, &pack, &members)?;
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

fn validate_exterior(pack: &CompiledGamePack, map: &MapModule, town: &Town) -> Result<()> {
    let seen = reachable(pack, map, [24, 0], true)?;
    let edges = BTreeSet::from([[24, 0], [25, 0]]);
    for p in &seen {
        ensure!(
            edges.contains(p) || (p[0] > 0 && p[0] < 39 && p[1] > 0 && p[1] < 35),
            "Ely unbound walking or Surf edge {p:?}"
        );
        ensure!(
            kind(pack, map, *p)? != "CUT_TREE",
            "Ely has no incidental Cut gate"
        );
        if kind(pack, map, *p)? == "DOOR" {
            ensure!(
                town.doors.iter().any(|d| d.tile == *p),
                "Ely unpaired facade"
            );
        }
    }
    for d in &town.doors {
        ensure!(
            seen.contains(&d.tile)
                && kind(pack, map, d.tile)? == "DOOR"
                && seen.contains(&d.return_tile)
                && kind(pack, map, d.return_tile)? == "FLOOR",
            "Ely disconnected facade or return"
        );
    }
    Ok(())
}

fn connect(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, side, target, offset, edge, to) in [
        (
            "MnEly",
            "north",
            "MnR13C00R02",
            -5,
            [[24, 0], [25, 0]],
            [[34, 31], [35, 31]],
        ),
        (
            "MnR13C00R02",
            "south",
            "MnEly",
            5,
            [[34, 31], [35, 31]],
            [[24, 0], [25, 0]],
        ),
    ] {
        let mut map = pack
            .data
            .maps
            .get(name)
            .context("League road endpoint")?
            .clone();
        for p in edge {
            ensure!(kind(pack, &map, p)? == "FLOOR", "League road edge floor");
        }
        crate::lake_country_geometry::connect(&mut map, side, target, offset)?;
        // Connections change only attributes and the cached module. Original
        // R13 script/event/NPC mirrors are never rewritten during this rebase.
        pack.data
            .map_attributes
            .insert(name.into(), map.attributes.clone());
        pack.data.maps.insert(name.into(), map);
        let m = &pack.data.maps[name];
        let native = OverworldMapData::from_attributes(name, &m.attributes, m.blocks.clone());
        for (p, destination) in edge.into_iter().zip(to) {
            let transition = connection_for_tile_checked(
                &native,
                TilePosition::new(
                    p[0] as i16,
                    p[1] as i16 + if side == "north" { -1 } else { 1 },
                ),
            )?
            .context("League seam")?;
            let actual = pack
                .data
                .resolve_connection_transition(&transition)?
                .destination;
            ensure!(
                actual.map_name == target
                    && actual.tile
                        == TilePosition::new(destination[0] as i16, destination[1] as i16),
                "League connection projection: {name} {p:?} -> {} {:?}, wanted {target} {destination:?}",
                actual.map_name,
                actual.tile
            );
        }
    }
    Ok(())
}
