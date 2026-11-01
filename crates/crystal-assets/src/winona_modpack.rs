//! The opening town and ordinary services, compiled from explicit native rows.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::{Value, json};

use crate::authored_route_layout::Layout;
use crate::authored_town_design::Town;
use crate::authored_town_scripts::{finish, ordinary};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, winona_opening, winona_publication};

const TOWN: &str = "MnWinona";
const CONSTANT: &str = "MN_WINONA";
const MART: &str = "MART_MN_WINONA";

/// Publish Winona's guardian scene, three real starter choices and survey errand.
///
/// Adds six authored maps, native healing/PC/paid shopping and real room/stair
/// warps. A declined choice grants nothing; a confirmed level5 gift is unique.
/// The survey bundle is represented by the canonical parcel event after its
/// pages are acknowledged. Five balls have a separate once-only, full-bag retry.
/// New Game explicitly enters the upstairs floor through the title program's
/// spawn write; the original house recovery destination is retained under an addon identity.
/// Other original recovery records and unrelated content remain unchanged.
/// Regional R01 binding is a separate step. No badge or later quest is granted.
///
/// # Errors
/// Returns an error for malformed/stale physical or story contracts, occupied
/// map/spawn/group identities, unsupported services, unsafe geometry, missing
/// native assets, invalid scripts or failed runtime/title verification. Inputs
/// are unchanged on failure. Existing saves are not relocated to this opening.
pub fn build_winona_opening_modpack(
    base: &CompiledGamePack,
    town_json: &str,
    layout_json: &str,
    story_json: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let town: Town = serde_json::from_str(town_json).context("Winona dossier")?;
    let raw: Value = serde_json::from_str(town_json)?;
    let layout: Layout = serde_json::from_str(layout_json).context("Winona layout")?;
    let opening: winona_opening::Opening = serde_json::from_value(raw["opening"].clone())?;
    let story: Value = serde_json::from_str(story_json)?;
    ensure!(
        town.id == TOWN
            && town.native_constant == CONSTANT
            && town.compiled_tileset == "johto"
            && town.dimensions_metatiles == [16, 14]
            && layout.id == TOWN
            && layout.dimensions_metatiles == town.dimensions_metatiles
            && town.interiors.len() == 5
            && town.doors.len() == 4,
        "explicit Winona map/room inventory required"
    );
    ensure!(
        !base.data.maps.contains_key(TOWN),
        "Winona already installed"
    );
    ensure!(
        raw["internal_stairs"]
            == json!([{
                "from_room":"MnWinonaHome1F","from_tile":[9,0],"from_warp":3,
                "to_room":"MnWinonaHome2F","to_tile":[7,0],"to_warp":1
            }]),
        "reciprocal native home staircase contract required"
    );
    let mut pack = base.clone();
    winona_opening::prepare(&mut pack, &opening, &story)?;
    for routine in ["HealParty", "PokemonCenterPC"] {
        pack.data.require_special_routine(routine)?;
    }
    ensure!(
        !pack.data.marts.0.contains_key(MART),
        "Winona mart occupied"
    );
    let clerk = town
        .interiors
        .iter()
        .flat_map(|r| &r.residents)
        .find(|a| a.service.as_deref() == Some("pokemart"))
        .context("Winona clerk")?;
    ensure!(
        clerk.stock == ["POKE_BALL", "POTION", "ANTIDOTE", "PARLYZ_HEAL"]
            && clerk
                .stock
                .iter()
                .all(|id| pack.data.items.contains_key(id)),
        "opening shop stock"
    );
    pack.data.marts.0.insert(MART.into(), clerk.stock.clone());
    let mut exterior = authoring::blank(&pack, "NewBarkTown", TOWN, CONSTANT)?;
    exterior.attributes.width = 16;
    exterior.attributes.height = 14;
    exterior.attributes.tileset_name = "johto".into();
    exterior.attributes.border_block = 5;
    exterior.blocks = layout.compile()?;
    ordinary(
        &pack,
        &mut exterior,
        &town.residents,
        &town.signs,
        "MN_WINONA_POKECENTER_1F",
        MART,
    )?;
    for door in &town.doors {
        ensure!(
            usize::from(door.warp_index) == exterior.events.warps.len() + 1
                && door.return_warp_index == Some(door.warp_index + 1),
            "Winona facade order"
        );
        let room = town
            .interiors
            .iter()
            .find(|r| r.id == door.target)
            .context("paired Winona room")?;
        ensure!(
            door.target_warp == i16::try_from(room.passive_arrival.index)?
                && door.arrival_tile == room.passive_arrival.tile,
            "passive indoor arrival pairing"
        );
        authoring::warp(
            &mut exterior,
            (door.tile[0], door.tile[1]),
            &room.native_constant,
            door.target_warp,
        );
        authoring::warp(
            &mut exterior,
            (door.return_tile[0], door.return_tile[1]),
            CONSTANT,
            i16::try_from(door.return_warp_index.context("outdoor return")?)?,
        );
    }
    let mut modules = vec![(exterior, [14, 18])];
    for room in &town.interiors {
        ensure!(
            !pack.data.maps.contains_key(&room.id),
            "opening room occupied"
        );
        let mut map = authoring::blank(&pack, &room.template, &room.id, &room.native_constant)?;
        authoring::blocks(
            &mut map,
            room.dimensions_metatiles[0],
            room.dimensions_metatiles[1],
            &room.rows.join(" "),
        )?;
        map.attributes.tileset_name = room.tileset.clone();
        map.attributes.border_block = room.border_block;
        ordinary(
            &pack,
            &mut map,
            &room.residents,
            &room.signs,
            "MN_WINONA_POKECENTER_1F",
            MART,
        )?;
        for exit in &room.exits {
            let door = town
                .doors
                .iter()
                .find(|d| d.target == room.id)
                .context("room facade")?;
            ensure!(
                exit.index == u16::try_from(map.events.warps.len() + 1)?
                    && exit.target == TOWN
                    && Some(u16::try_from(exit.target_warp)?) == door.return_warp_index,
                "room returns to its own facade"
            );
            authoring::warp(
                &mut map,
                (exit.tile[0], exit.tile[1]),
                CONSTANT,
                exit.target_warp,
            );
        }
        match room.id.as_str() {
            "MnWinonaHome1F" => {
                authoring::warp(&mut map, (9, 0), "MN_WINONA_HOME_2F", 1);
                winona_opening::install_home(&mut map, &story)?;
            }
            "MnWinonaHome2F" => authoring::warp(&mut map, (7, 0), "MN_WINONA_HOME_1F", 3),
            "MnWinonaPokecenter1F" | "MnWinonaMart" | "MnWinonaFieldStation" => {}
            _ => anyhow::bail!("unexpected opening room"),
        }
        ensure!(
            room.passive_arrival.index == u16::try_from(map.events.warps.len() + 1)?,
            "opening passive warp order"
        );
        let (target, index) = if room.id == "MnWinonaHome2F" {
            ("MN_WINONA_HOME_2F", 2)
        } else {
            let door = town
                .doors
                .iter()
                .find(|d| d.target == room.id)
                .context("arrival facade")?;
            (
                CONSTANT,
                i16::try_from(door.return_warp_index.context("safe outdoor return")?)?,
            )
        };
        authoring::warp(
            &mut map,
            (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
            target,
            index,
        );
        if let Some(pc) = &room.pc {
            let label = "MnWClinicPC";
            map.scripts.insert(
                label.into(),
                json!([
                    authoring::command("opentext", &[]),
                    authoring::command("special", &["PokemonCenterPC"]),
                    authoring::command("closetext", &[]),
                    authoring::command("end", &[])
                ]),
            );
            map.events.bg_events.push(BackgroundEvent {
                x: pc.tile[0],
                y: pc.tile[1],
                event_type: "BGEVENT_READ".into(),
                script: label.into(),
            });
        }
        modules.push((map, room.passive_arrival.tile));
    }
    for (map, _) in &mut modules {
        finish(&pack, map, &town)?;
    }
    crate::authored_town_geometry::validate(&pack, &town, &modules)?;
    for (map, start) in &modules {
        let seen = crate::authored_town_geometry::reachable(&pack, map, *start, false)?;
        for (room, tile, approach) in [
            ("MnWinonaHome1F", [9, 0], [9, 1]),
            ("MnWinonaHome2F", [7, 0], [7, 1]),
        ] {
            if map.id == room {
                ensure!(
                    crate::authored_town_geometry::kind(&pack, map, tile)? == "STAIRCASE"
                        && seen.contains(&approach),
                    "native home stairs need a clear ordinary approach"
                );
            }
        }
        if map.id == "MnWinonaHome2F" {
            ensure!(
                *start == opening.bedroom_start,
                "bedroom floor must match the New Game spawn"
            );
        }
        if map.id == "MnWinonaFieldStation" {
            for ball in &opening.starters {
                ensure!(
                    crate::authored_town_geometry::kind(&pack, map, ball.tile)? == "WALL"
                        && seen.contains(&ball.approach),
                    "starter ball needs its native tabletop and reachable approach"
                );
            }
        }
    }
    for (map, _) in &mut modules {
        if map.id == "MnWinonaFieldStation" {
            winona_opening::install_station(&pack, map, &opening, &story)?;
        }
        finish(&pack, map, &town)?;
    }
    winona_publication::publish(pack, modules)
}
