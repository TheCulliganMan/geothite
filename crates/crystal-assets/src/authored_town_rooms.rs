//! Native ordinary rooms with reciprocal carpets, services and a passive arrival.
use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde_json::json;

use crate::authored_town_design::{Room, Town};
use crate::authored_town_scripts::{finish, ordinary};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn build(
    pack: &CompiledGamePack,
    town: &Town,
    room: &Room,
    clinic: &str,
    mart: &str,
) -> Result<MapModule> {
    let mut map = authoring::blank(pack, &room.template, &room.id, &room.native_constant)?;
    authoring::blocks(
        &mut map,
        room.dimensions_metatiles[0],
        room.dimensions_metatiles[1],
        &room.rows.join(" "),
    )?;
    map.attributes.tileset_name = room.tileset.clone();
    map.attributes.border_block = room.border_block;
    ordinary(pack, &mut map, &room.residents, &room.signs, clinic, mart)?;
    let door = town
        .doors
        .iter()
        .find(|d| d.target == room.id)
        .context("paired facade")?;
    ensure!(
        usize::from(room.passive_arrival.index) == room.exits.len() + 1,
        "active carpets followed by one passive arrival"
    );
    for (i, exit) in room.exits.iter().enumerate() {
        ensure!(
            usize::from(exit.index) == i + 1
                && exit.target == town.id
                && Some(u16::try_from(exit.target_warp)?) == door.return_warp_index,
            "authored room carpet pairing"
        );
        authoring::warp(
            &mut map,
            (exit.tile[0], exit.tile[1]),
            &town.native_constant,
            exit.target_warp,
        );
    }
    authoring::warp(
        &mut map,
        (room.passive_arrival.tile[0], room.passive_arrival.tile[1]),
        &town.native_constant,
        i16::try_from(door.return_warp_index.context("passive return")?)?,
    );
    if let Some(pc) = &room.pc {
        let label = format!("{}PC", room.id);
        let c = authoring::command;
        map.scripts.insert(
            label.clone(),
            json!([
                c("opentext", &[]),
                c("special", &["PokemonCenterPC"]),
                c("closetext", &[]),
                c("end", &[])
            ]),
        );
        map.events.bg_events.push(BackgroundEvent {
            x: pc.tile[0],
            y: pc.tile[1],
            event_type: "BGEVENT_READ".into(),
            script: label,
        });
    }
    finish(pack, &mut map, town)?;
    Ok(map)
}
