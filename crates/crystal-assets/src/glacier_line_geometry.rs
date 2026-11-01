//! Check actual station collision, public ports, conductor throat and movements.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

use crate::authored_town_geometry::{kind, reachable};
use crate::glacier_line_service::Station;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(pack: &CompiledGamePack, stop: &Station) -> Result<()> {
    let room = pack
        .data
        .maps
        .get(&stop.map)
        .context("approved station hall missing")?;
    ensure!(
        room.attributes.map_constant.as_deref() == Some(&stop.constant)
            && room.attributes.tileset_name == "train_station",
        "native station identity differs"
    );
    let bytes = room
        .blocks
        .iter()
        .copied()
        .map(u8::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        format!("{:x}", Sha256::digest(&bytes)) == stop.blocks_sha256,
        "{}: reviewed station rows changed",
        stop.map
    );
    let tileset = pack
        .data
        .tilesets
        .get("train_station")
        .context("native train tileset")?;
    ensure!(
        room.blocks.iter().all(|block| {
            tileset
                .collision
                .get(&format!("{block:02x}"))
                .is_some_and(|cells| {
                    cells.iter().all(|kind| {
                        !matches!(
                            kind.as_str(),
                            "WATER"
                                | "WATERFALL"
                                | "WHIRLPOOL"
                                | "CURRENT_UP"
                                | "CURRENT_DOWN"
                                | "CURRENT_LEFT"
                                | "CURRENT_RIGHT"
                        )
                    })
                })
        }),
        "dry station required before removing its unused fishing reference"
    );
    let conductors: Vec<_> = room
        .objects
        .iter()
        .filter(|o| o.script == stop.conductor_script)
        .collect();
    ensure!(conductors.len() == 1, "one original conductor required");
    let conductor = conductors[0];
    ensure!(
        [conductor.x, conductor.y] == stop.conductor_core
            && conductor.sprite == "SPRITE_OFFICER"
            && conductor.spritemovedata == "SPRITEMOVEDATA_STANDING_DOWN"
            && conductor.event_flag == "-1"
            && conductor
                .object_identifier
                .as_ref()
                .is_some_and(|id| !id.is_empty()),
        "original named conductor must always close the platform throat"
    );
    ensure!(
        stop.conductor_core[1]
            .checked_add(1)
            .map(|y| [stop.conductor_core[0], y])
            == Some(stop.approach_core),
        "boarding approach must face the conductor from the south"
    );
    let bodies: BTreeSet<_> = room.objects.iter().map(|o| [o.x, o.y]).collect();
    for arrival in [
        stop.public_arrival_core,
        stop.ride_arrival_core,
        stop.approach_core,
    ] {
        ensure!(
            kind(pack, room, arrival)? == "FLOOR" && !bodies.contains(&arrival),
            "unoccupied ordinary Floor required at each public/ride/boarding approach"
        );
    }
    ensure!(
        !room
            .events
            .warps
            .iter()
            .any(|w| [w.x, w.y] == stop.ride_arrival_core)
            && !room
                .events
                .coord_events
                .iter()
                .any(|e| [e.x, e.y] == stop.ride_arrival_core),
        "train arrival cannot overlap an existing trigger or passive warp"
    );
    let arrival = room
        .events
        .warps
        .iter()
        .find(|w| w.index == stop.public_arrival_warp)
        .context("indexed public arrival missing")?;
    ensure!(
        [arrival.x, arrival.y] == stop.public_arrival_core,
        "public arrival index moved"
    );
    let parent = pack
        .data
        .maps
        .get(&stop.parent_map)
        .context("public city/outpost missing")?;
    let entry = parent
        .events
        .warps
        .iter()
        .find(|w| w.index == stop.parent_entry_warp)
        .context("original exterior entry missing")?;
    ensure!(
        entry.target_map_constant == stop.constant
            && entry.target_warp_id == i16::try_from(stop.public_arrival_warp)?,
        "original exterior door must enter the public hall"
    );
    let exits: BTreeSet<_> = stop.exit_warp_indices.iter().copied().collect();
    ensure!(
        !exits.is_empty()
            && exits.len() == stop.exit_warp_indices.len()
            && room.events.warps.len() == exits.len() + 1
            && !exits.contains(&stop.public_arrival_warp),
        "complete original public warp roster required"
    );
    for index in exits {
        let exit = room
            .events
            .warps
            .iter()
            .find(|w| w.index == index)
            .context("public exit missing")?;
        ensure!(
            Some(&exit.target_map_constant) == parent.attributes.map_constant.as_ref()
                && kind(pack, room, [exit.x, exit.y])? == "WARP_CARPET_DOWN",
            "every original exit must remain a carpet to its own city/outpost"
        );
        let target = parent
            .events
            .warps
            .iter()
            .find(|w| i16::try_from(w.index).ok() == Some(exit.target_warp_id))
            .context("safe exterior return missing")?;
        ensure!(
            kind(pack, parent, [target.x, target.y])? == "FLOOR",
            "public exit must return to native safe Floor"
        );
    }
    let public = reachable(pack, room, stop.public_arrival_core, false)?;
    ensure!(
        public.contains(&stop.approach_core) && public.contains(&stop.ride_arrival_core),
        "public entrance, conductor and train arrival must share the concourse"
    );
    for hidden in [
        stop.platform_core,
        stop.train_door_core,
        stop.other_train_door_core,
    ] {
        ensure!(
            !public.contains(&hidden),
            "walking around the conductor bypasses ticket guards"
        );
    }
    let mut opened = room.clone();
    opened.objects.retain(|o| o.script != stop.conductor_script);
    let platform = reachable(pack, &opened, stop.approach_core, false)?;
    ensure!(
        platform.contains(&stop.platform_core) && platform.contains(&stop.train_door_core),
        "removing only the conductor must expose the actual boarding path"
    );
    let others: BTreeSet<_> = opened.objects.iter().map(|o| [o.x, o.y]).collect();
    let mut open_bodies = others.clone();
    open_bodies.insert(stop.approach_core);
    let parked = movement(
        pack,
        room,
        stop.conductor_core,
        &stop.conductor_open_steps,
        &open_bodies,
        None,
    )?;
    let mut boarding_bodies = others.clone();
    boarding_bodies.insert(parked);
    let boarded = movement(
        pack,
        room,
        stop.approach_core,
        &stop.player_board_steps,
        &boarding_bodies,
        Some(stop.train_door_core),
    )?;
    ensure!(
        boarded == stop.train_door_core,
        "boarding movement misses its native train door"
    );
    let mut reset_bodies = others;
    reset_bodies.insert(boarded);
    ensure!(
        movement(
            pack,
            room,
            parked,
            &stop.conductor_restore_steps,
            &reset_bodies,
            None
        )? == stop.conductor_core
            && stop
                .conductor_restore_steps
                .last()
                .is_some_and(|d| d == "DOWN"),
        "departure conductor must return home facing south before map load"
    );
    Ok(())
}

fn movement(
    pack: &CompiledGamePack,
    room: &MapModule,
    start: [u16; 2],
    steps: &[String],
    bodies: &BTreeSet<[u16; 2]>,
    final_door: Option<[u16; 2]>,
) -> Result<[u16; 2]> {
    ensure!(
        !steps.is_empty() && steps.len() <= 32,
        "bounded explicit boarding movement required"
    );
    let mut at = start;
    for (i, direction) in steps.iter().enumerate() {
        at = match direction.as_str() {
            "UP" => [
                at[0],
                at[1].checked_sub(1).context("movement north of map")?,
            ],
            "DOWN" => [at[0], at[1].checked_add(1).context("movement y overflow")?],
            "LEFT" => [at[0].checked_sub(1).context("movement west of map")?, at[1]],
            "RIGHT" => [at[0].checked_add(1).context("movement x overflow")?, at[1]],
            _ => anyhow::bail!("boarding movement must use an ordinary native step"),
        };
        let collision = kind(pack, room, at)?;
        ensure!(
            !bodies.contains(&at)
                && (collision == "FLOOR"
                    || (collision == "DOOR" && i + 1 == steps.len() && final_door == Some(at))),
            "{} boarding movement hits {collision} or another actor at {at:?}",
            room.id
        );
        ensure!(
            !room.events.warps.iter().any(|w| [w.x, w.y] == at)
                && !room.events.coord_events.iter().any(|e| [e.x, e.y] == at),
            "boarding movement overlaps an existing native event"
        );
    }
    Ok(at)
}
