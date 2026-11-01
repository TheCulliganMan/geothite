//! Authentic east–west Crystal gate, with separate active and passive warps.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_town_design::Town;
use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
struct Side {
    entrances: [[u16; 2]; 2],
    returns: [[u16; 2]; 2],
    active_indices: [u16; 2],
    passive_indices: [u16; 2],
}
#[derive(Deserialize)]
struct Arrival {
    index: u16,
    tile: [u16; 2],
    target: String,
    target_warp: i16,
}
#[derive(Deserialize)]
struct Contract {
    reference: String,
    park: Side,
    route: Side,
    passive_arrivals: Vec<Arrival>,
}

fn outside(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    side: &Side,
    arrival_start: i16,
    carpet: &str,
) -> Result<()> {
    ensure!(map.events.warps.is_empty(), "unoccupied gatehouse exterior");
    for lane in 0..2 {
        ensure!(
            side.active_indices[lane] == lane as u16 + 1
                && side.passive_indices[lane] == lane as u16 + 3
                && kind(pack, map, side.entrances[lane])? == carpet
                && kind(pack, map, side.returns[lane])? == "FLOOR",
            "native side carpet and safe exterior return required"
        );
        let [x, y] = side.entrances[lane];
        authoring::warp(
            map,
            (x, y),
            "MN_ITASCA_GATEHOUSE",
            arrival_start + lane as i16,
        );
    }
    let constant = map
        .attributes
        .map_constant
        .clone()
        .context("exterior constant")?;
    for lane in 0..2 {
        let [x, y] = side.returns[lane];
        authoring::warp(map, (x, y), &constant, side.passive_indices[lane] as i16);
    }
    Ok(())
}

pub(crate) fn build(
    pack: &mut CompiledGamePack,
    town: &Town,
    source: &Value,
    park: &mut MapModule,
) -> Result<MapModule> {
    let c: Contract = serde_json::from_value(source["gatehouse_contract"].clone())?;
    let room = town.interiors.first().context("native gatehouse source")?;
    let template = pack
        .data
        .maps
        .get(&c.reference)
        .context("native gate reference")?;
    ensure!(
        c.reference == "IlexForestAzaleaGate"
            && room.template == c.reference
            && room.id == "MnItascaGatehouse"
            && room.native_constant == "MN_ITASCA_GATEHOUSE"
            && room.tileset == "gate"
            && room.dimensions_metatiles == [5, 4]
            && room.exits.len() == 4
            && c.passive_arrivals.len() == 4
            && room.pc.is_none(),
        "authentic Ilex–Azalea floorplan and both paired mouths required"
    );
    let mut gate = authoring::blank(pack, &room.template, &room.id, &room.native_constant)?;
    authoring::blocks(&mut gate, 5, 4, &room.rows.join(" "))?;
    ensure!(
        gate.blocks == template.blocks,
        "original gatehouse floorplan must remain exact"
    );
    crate::authored_town_scripts::ordinary(pack, &mut gate, &room.residents, &[], "", "")?;
    outside(pack, park, &c.park, 5, "WARP_CARPET_RIGHT")?;
    let mut route = pack
        .data
        .maps
        .get("MnR14C00R00")
        .context("R14 west approach")?
        .clone();
    for (x, y, previous, block) in [
        (0, 20, 5, 0x08),
        (1, 20, 5, 0x09),
        (0, 21, 1, 0x10),
        (1, 21, 1, 0x11),
        (2, 21, 1, 0x0b),
    ] {
        let i = y * 32 + x;
        ensure!(
            route.blocks[i] == previous,
            "reserved R14 gatehouse approach changed"
        );
        route.blocks[i] = block;
    }
    outside(pack, &mut route, &c.route, 7, "WARP_CARPET_LEFT")?;
    let route_constant = route
        .attributes
        .map_constant
        .as_deref()
        .context("R14 constant")?;
    for (i, exit) in room.exits.iter().enumerate() {
        let west = i < 2;
        let expected_tile = [if west { 0 } else { 9 }, 4 + (i % 2) as u16];
        ensure!(
            exit.index == i as u16 + 1
                && exit.tile == expected_tile
                && exit.target
                    == if west {
                        town.id.as_str()
                    } else {
                        route.id.as_str()
                    }
                && exit.target_warp == 3 + (i % 2) as i16
                && kind(pack, &gate, exit.tile)?
                    == if west {
                        "WARP_CARPET_LEFT"
                    } else {
                        "WARP_CARPET_RIGHT"
                    },
            "native gatehouse active exit pairing differs"
        );
        authoring::warp(
            &mut gate,
            (exit.tile[0], exit.tile[1]),
            if west {
                &town.native_constant
            } else {
                route_constant
            },
            exit.target_warp,
        );
    }
    for (i, a) in c.passive_arrivals.iter().enumerate() {
        let west = i < 2;
        ensure!(
            a.index == i as u16 + 5
                && a.tile == [if west { 1 } else { 8 }, 4 + (i % 2) as u16]
                && a.target
                    == if west {
                        town.native_constant.as_str()
                    } else {
                        route_constant
                    }
                && a.target_warp == 3 + (i % 2) as i16
                && kind(pack, &gate, a.tile)? == "FLOOR",
            "separate passive gatehouse arrivals required"
        );
        authoring::warp(&mut gate, (a.tile[0], a.tile[1]), &a.target, a.target_warp);
    }
    let seen = reachable(pack, &gate, [1, 4], false)?;
    for w in &gate.events.warps {
        ensure!(seen.contains(&[w.x, w.y]), "gatehouse lane disconnected");
    }
    for a in &room.residents {
        ensure!(
            kind(pack, &gate, a.tile)? == "FLOOR",
            "gatehouse resident blocked"
        );
        let approach = a.approach.unwrap_or([a.tile[0], a.tile[1] + 1]);
        ensure!(seen.contains(&approach), "gatehouse resident inaccessible");
        if let Some(counter) = a.counter_tile {
            ensure!(
                kind(pack, &gate, counter)? == "COUNTER",
                "native counter required"
            );
        }
    }
    let active = room.exits.iter().map(|e| e.tile).collect::<BTreeSet<_>>();
    for y in 0..8 {
        for x in 0..10 {
            ensure!(
                matches!(
                    kind(pack, &gate, [x, y])?,
                    "WARP_CARPET_LEFT" | "WARP_CARPET_RIGHT"
                ) == active.contains(&[x, y]),
                "every native side carpet must have exactly one exit"
            );
        }
    }
    // This original indoor plan has FLOOR at the south edge. With no map
    // connections, the production controller rejects stepping out of bounds.
    ensure!(
        gate.attributes.connections.is_empty(),
        "indoor gate cannot have an outdoor seam"
    );
    crate::authored_town_scripts::finish(pack, &mut route, town)?;
    crate::authored_town_publication::replace(pack, route)?;
    Ok(gate)
}
