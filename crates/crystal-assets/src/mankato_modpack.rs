//! Mankato's terrace, ordinary services and canonical second-gym entrance.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, PlayerMovementState, StepOptions, StepOutcome, attempt_ledge_jump,
    attempt_step,
};
use serde_json::{Value, json};

use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_scripts::{finish, ordinary};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const CLINIC: &str = "MN_MANKATO_POKECENTER_1F";
const MART: &str = "MART_MN_MANKATO";

/// Adds Mankato and binds its already-installed canonical second gym.
///
/// Care, PC and shopping use native services. The gym retains its complete
/// puzzle, original characters, parties and rewards; only its three exterior
/// warp destinations and landmark change. HOME and existing recovery entries
/// remain intact. Regional route seams are separate; installation grants no
/// player progress.
///
/// # Errors
/// Returns an error for occupied identities, stale canonical gym content,
/// malformed layouts/scripts, unsafe doors/services/ledge geometry or a pack
/// verification failure. The borrowed input remains unchanged on failure.
pub fn build_mankato_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    gyms_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Mankato town")?;
    let layout: Layout = serde_json::from_str(layout_json).context("Mankato strokes")?;
    let raw: Value = serde_json::from_str(town_json)?;
    ensure!(
        town.id == "MnMankato"
            && town.native_constant == "MN_MANKATO"
            && town.compiled_tileset == "johto"
            && town.dimensions_metatiles == [24, 18]
            && layout.id == town.id
            && layout.compiled_tileset == town.compiled_tileset
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && town.interiors.len() == 3
            && town.doors.len() == 3
            && !base.data.maps.contains_key(&town.id),
        "Mankato identity/inventory or duplicate installation"
    );
    for (room, (id, size)) in town.interiors.iter().zip([
        ("MnMankatoPokecenter1F", [10, 8]),
        ("MnMankatoMart", [6, 4]),
        ("MnMankatoGym", [8, 10]),
    ]) {
        ensure!(
            room.id == id && room.dimensions_metatiles == size,
            "Mankato room inventory"
        );
    }
    let mut pack = base.clone();
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    let mut gym = crate::mankato_gym::bind(
        &pack,
        &town.interiors[2],
        &raw["interiors"][2]["source_reference"],
        gyms_json,
    )?;
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
                    "POKE_BALL",
                    "POTION",
                    "SUPER_POTION",
                    "ANTIDOTE",
                    "PARLYZ_HEAL",
                    "ESCAPE_ROPE"
                ]
            && !pack.data.marts.0.contains_key(MART),
        "Mankato native mart stock"
    );
    pack.data
        .marts
        .0
        .insert(MART.into(), clerks[0].stock.clone());
    let mut exterior = authoring::blank(&pack, "VioletCity", &town.id, &town.native_constant)?;
    exterior.attributes.width = town.dimensions_metatiles[0];
    exterior.attributes.height = town.dimensions_metatiles[1];
    exterior.attributes.tileset_name = town.compiled_tileset.clone();
    exterior.attributes.border_block = 5;
    exterior.blocks = layout.compile()?;
    ordinary(
        &pack,
        &mut exterior,
        &town.residents,
        &town.signs,
        CLINIC,
        MART,
    )?;
    for (i, door) in town.doors.iter().enumerate() {
        let room = &town.interiors[i];
        ensure!(
            usize::from(door.warp_index) == i * 2 + 1
                && door.return_warp_index == Some(door.warp_index + 1)
                && door.target == room.id
                && door.target_warp == 3
                && door.arrival_tile == room.passive_arrival.tile,
            "Mankato reciprocal passive door pairing"
        );
        authoring::warp(
            &mut exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            3,
        );
        authoring::warp(
            &mut exterior,
            (door.return_tile[0], door.return_tile[1]),
            &town.native_constant,
            i16::try_from(door.return_warp_index.context("passive return")?)?,
        );
    }
    validate_terrace(&pack, &exterior, &raw)?;
    let mut modules = vec![(exterior, [40, 18])];
    for room in &town.interiors {
        if room.id == "MnMankatoGym" {
            finish(&pack, &mut gym, &town)?;
            modules.push((gym.clone(), room.passive_arrival.tile));
        } else {
            modules.push((
                crate::authored_town_rooms::build(&pack, &town, room, CLINIC, MART)?,
                room.passive_arrival.tile,
            ));
        }
    }
    for (map, _) in &mut modules {
        finish(&pack, map, &town)?;
    }
    crate::authored_town_geometry::validate(&pack, &town, &modules)?;
    crate::mankato_publication::publish(&mut pack, modules)?;
    pack.report
        .manifests
        .push("minnesota-mankato-city-v1".into());
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

fn validate_terrace(pack: &CompiledGamePack, map: &MapModule, raw: &Value) -> Result<()> {
    let contract = &raw["ledge_contract"];
    let approach: [u16; 2] = serde_json::from_value(contract["approach_core"].clone())?;
    let landing: [u16; 2] = serde_json::from_value(contract["landing_core"].clone())?;
    ensure!(
        approach == [10, 17]
            && landing == [10, 20]
            && contract["hop_core"] == json!([10, 18])
            && contract["blocked_reverse_core"] == json!([10, 19])
            && contract["minimum_ordinary_return_steps"] == 24,
        "Mankato terrace contract differs"
    );
    let kind = crate::authored_town_geometry::kind;
    ensure!(
        kind(pack, map, approach)? == "FLOOR"
            && kind(pack, map, [10, 18])? == "HOP_DOWN"
            && kind(pack, map, [10, 19])? == "WALL"
            && kind(pack, map, landing)? == "FLOOR",
        "native terrace hop and reverse collision differ"
    );
    let occupied = map
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        !occupied.contains(&[10, 18]) && !occupied.contains(&landing),
        "Mankato terrace actor blocks the hop"
    );
    let native = OverworldMapData::from_attributes(&map.id, &map.attributes, map.blocks.clone());
    let collision = pack.data.tileset_collision(&map.attributes.tileset_name)?;
    let mut player = PlayerMovementState::new(TilePosition::new(10, 18));
    ensure!(
        matches!(
            attempt_ledge_jump(&mut player, Direction::Down, &native, &collision, StepOptions::default()),
            LedgeJumpOutcome::Jumped { to, .. } if to == TilePosition::new(10, 20)
        ),
        "Mankato terrace must execute a native hop from the HOP_DOWN quadrant"
    );
    ensure!(
        matches!(
            attempt_step(
                &mut player,
                Direction::Up,
                &native,
                &collision,
                StepOptions {
                    force_step_after_turn: true,
                    ..StepOptions::default()
                }
            ),
            StepOutcome::Blocked { .. }
        ),
        "Mankato terrace must block reverse movement"
    );
    let distances =
        crate::authored_route_surface::walking_map_distances(pack, map, landing, &occupied)?;
    ensure!(
        distances.get(&approach).is_some_and(|&steps| steps >= 24),
        "terrace return bypass"
    );
    for block in &map.blocks {
        ensure!(
            !pack.data.tilesets["johto"].collision[&format!("{block:02x}")]
                .iter()
                .any(|k| k == "CUT_TREE"),
            "incidental Cut tree in Mankato"
        );
    }
    Ok(())
}
