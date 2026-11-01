//! Validate the actual source geometry, not just its declared destinations.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use super::{Town, point};
use crate::authored_route_design::ResidentService;
use crate::authored_route_modpack::{collision, validate_room};
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate(
    pack: &CompiledGamePack,
    town: &Town,
    contract: &Value,
    exterior: &MapModule,
    shelter: &MapModule,
) -> Result<()> {
    ensure!(
        contract["story_contract"]["Q07"]["free_access"] == true
            && contract["story_contract"]["Q07"]["paid_state_required"] == false
            && contract["admission_contract"]["status"] == "CLOSED_NO_FEE"
            && contract["admission_contract"]["money_taken"] == 0
            && contract["admission_contract"]["session_started"] == false
            && contract["admission_contract"]["pending_preserve_warp"]["enabled"] == false,
        "basecamp story access is free and expedition admission must remain closed without a fee"
    );
    let room = &town.interiors[0];
    let door = &town.doors[0];
    let arrival = room
        .passive_arrival
        .as_ref()
        .context("shelter passive arrival3")?;
    ensure!(
        door.warp_index == 1
            && door.return_warp_index == Some(2)
            && door.target == room.id
            && door.target_warp == 3
            && door.return_tile == door.arrival_tile
            && door.tile[0] == door.return_tile[0]
            && door.tile[1].checked_add(1) == Some(door.return_tile[1])
            && door.collision == "DOOR"
            && room.exits == [[8, 11], [9, 11]]
            && room.exit_collision == "WARP_CARPET_DOWN"
            && room.border_block == Some(0)
            && room.clear_arrival == Some([8, 9])
            && arrival.index == 3
            && arrival.tile == [8, 9]
            && room.items.is_empty()
            && room.keeper_tile.is_none(),
        "shelter requires two actual carpet exits, passive FLOOR arrival3 and paired exterior return2"
    );
    validate_room(pack, shelter, room)?;
    let kind = |tile| kind(pack, exterior, tile);
    let mut walk = BTreeSet::new();
    let mut surf = BTreeSet::new();
    let mut actual_doors = BTreeSet::new();
    for y in 0..48 {
        for x in 0..64 {
            let tile = [x, y];
            let collision = kind(tile)?;
            ensure!(
                collision != "CUT_TREE",
                "undeclared decorative Cut tree at{x},{y}"
            );
            let water = collision.starts_with("WATER") && collision != "WATERFALL";
            if matches!(collision, "FLOOR" | "TALL_GRASS" | "DOOR") {
                walk.insert(tile);
            }
            if matches!(collision, "FLOOR" | "TALL_GRASS" | "DOOR") || water {
                surf.insert(tile);
            }
            if collision == "DOOR" {
                actual_doors.insert(tile);
            }
            ensure!(
                !water || !(x == 0 || x == 63 || y == 0 || y == 47),
                "unconnected outer water at{x},{y} needs native buoy/rock closure"
            );
        }
    }
    ensure!(
        actual_doors == BTreeSet::from([door.tile]),
        "every actual facade DOOR needs its one native binding"
    );
    ensure!(
        kind(door.return_tile)? == "FLOOR",
        "exterior passive return must be FLOOR"
    );
    let mut bodies = BTreeSet::new();
    let mut identifiers = BTreeSet::new();
    for resident in &town.residents {
        ensure!(
            bodies.insert(resident.tile) && identifiers.insert(&resident.id),
            "overlapping/duplicate town actor {}",
            resident.id
        );
        ensure!(
            resident.service.is_none(),
            "public rest service belongs to the actual indoor medic"
        );
        ensure!(
            kind(resident.tile)? == "FLOOR",
            "town actor stands off FLOOR: {}",
            resident.id
        );
        walk.remove(&resident.tile);
        surf.remove(&resident.tile);
    }
    ensure!(
        !bodies.contains(&door.return_tile),
        "exterior passive return occupied"
    );
    let seen = reached([6, 24], &walk, &actual_doors);
    let surf_seen = reached([6, 24], &surf, &actual_doors);
    for resident in &town.residents {
        ensure!(
            adjacent(resident.tile)
                .iter()
                .any(|tile| seen.contains(tile)),
            "town actor has no free approach: {}",
            resident.id
        );
    }
    for sign in &town.signs {
        ensure!(
            kind(sign.tile)? == "WALL",
            "town sign is off native wall: {}",
            sign.id
        );
        ensure!(
            adjacent(sign.tile).iter().any(|tile| seen.contains(tile)),
            "town sign has no free approach: {}",
            sign.id
        );
    }
    ensure!(
        seen.contains(&door.tile) && seen.contains(&door.return_tile),
        "actual shelter door is not freely reachable"
    );
    let expected_ports = [
        ("MN_R12", "E", [30, 12], [28, 12], [63, 24]),
        ("MN_R13", "S", [16, 22], [16, 20], [32, 47]),
        ("MN_R20", "W", [1, 12], [3, 12], [0, 24]),
    ];
    ensure!(
        town.ports.len() == expected_ports.len(),
        "retain all three regional walking port reservations"
    );
    for (route, side, gate, landing, edge) in expected_ports {
        let port = town
            .ports
            .iter()
            .find(|port| port.route == route)
            .context("reserved town port")?;
        ensure!(
            port.side == side && port.gate_metatile == gate && port.landing_metatile == landing,
            "reserved port changed: {route}"
        );
        for tile in [gate.map(|n| n * 2), landing.map(|n| n * 2), edge] {
            ensure!(
                seen.contains(&tile),
                "free town port disconnected: {route} at{tile:?}"
            );
        }
    }
    let outer = seen
        .iter()
        .copied()
        .filter(|&[x, y]| x == 0 || x == 63 || y == 0 || y == 47)
        .collect::<BTreeSet<_>>();
    ensure!(
        outer == BTreeSet::from([[0, 24], [0, 25], [63, 24], [63, 25], [32, 47], [33, 47]]),
        "unreserved outer walking shoulder: {outer:?}"
    );
    let admission = &contract["admission_contract"];
    let pending = point(&admission["pending_preserve_warp"]["tile"])?;
    ensure!(
        !surf_seen.contains(&pending),
        "closed expedition admission has a walking/Surf bypass"
    );
    for key in ["free_approach", "early_return_safe_core"] {
        ensure!(
            seen.contains(&point(&admission[key])?),
            "free check-in approach/return disconnected"
        );
    }
    let guards = admission["physical_gate_actors"]
        .as_array()
        .context("physical gate actors")?;
    let gate_tiles = admission["gate_tiles"]
        .as_array()
        .context("physical gate tiles")?;
    ensure!(
        guards.len() == 2 && gate_tiles.len() == 2,
        "closed gate needs its actual two native guards"
    );
    for (id, tile) in guards.iter().zip(gate_tiles) {
        let id = id.as_str().context("guard identity")?;
        let resident = town
            .residents
            .iter()
            .find(|resident| resident.id == id)
            .context("actual closed guard")?;
        ensure!(
            resident.tile == point(tile)? && resident.sprite == "SPRITE_OFFICER",
            "closed guard binding changed"
        );
    }
    for id in guards.iter().chain([&admission["admission_actor"]]) {
        let actor = town
            .residents
            .iter()
            .find(|actor| Some(actor.id.as_str()) == id.as_str())
            .context("closed check-in actor")?;
        ensure!(
            json!(actor.lines)
                == json!([
                    "EXPEDITIONS CLOSED",
                    "Safari permits are",
                    "not available yet.",
                    "No fee is charged."
                ]),
            "closed ranger must truthfully refuse without taking a fee"
        );
    }
    let dock = &contract["public_dock"];
    let launch = point(&dock["launch_tile"])?;
    let water = point(&dock["water_target"])?;
    ensure!(
        seen.contains(&launch)
            && kind(launch)? == "FLOOR"
            && kind(water)? == "WATER"
            && adjacent(launch).contains(&water),
        "public landing requires a reachable native Surf launch"
    );
    ensure!(
        dock["water_seam"].is_null(),
        "public lake has no reciprocal water neighbor"
    );
    for (field, expected) in [
        ("north_rail_blocks", ["WALL", "FLOOR"]),
        ("south_rail_blocks", ["FLOOR", "WALL"]),
    ] {
        for at in dock[field].as_array().context("native dock rails")? {
            let [x, y] = point(at)?.map(|n| n * 2);
            ensure!(
                [kind([x, y])?, kind([x, y + 1])?] == expected
                    && [kind([x + 1, y])?, kind([x + 1, y + 1])?] == expected,
                "native dock rail collision changed"
            );
        }
    }
    for end in dock["closed_end_blocks"]
        .as_array()
        .context("native dock closed end")?
    {
        let [x, y] = point(&end["position"])?;
        let block =
            u16::from_str_radix(end["block"].as_str().context("dock end source block")?, 16)?;
        ensure!(
            exterior.blocks[usize::from(y) * 32 + usize::from(x)] == block
                && kind([x * 2, y * 2])? == "WALL"
                && kind([x * 2, y * 2 + 1])? == "WALL",
            "dock must retain its native closed end"
        );
    }
    let medic = room
        .residents
        .iter()
        .find(|resident| resident.id == "MN_BOUNDARY_SURVEY_MEDIC")
        .context("actual indoor free medic")?;
    ensure!(
        medic.tile == [3, 7]
            && medic.sprite == "SPRITE_NURSE"
            && matches!(medic.service, Some(ResidentService::HealParty)),
        "free medic must bind native HealParty inside real shelter"
    );
    ensure!(
        room.residents
            .iter()
            .filter(|resident| resident.service.is_some())
            .count()
            == 1,
        "only actual medic provides rest"
    );
    Ok(())
}

pub(super) fn kind<'a>(
    pack: &'a CompiledGamePack,
    module: &MapModule,
    tile: [u16; 2],
) -> Result<&'a str> {
    collision(
        pack,
        &module.blocks,
        module.attributes.width,
        tile,
        &module.attributes.tileset_name,
    )
}
fn adjacent([x, y]: [u16; 2]) -> [[u16; 2]; 4] {
    [
        [x.saturating_sub(1), y],
        [x + 1, y],
        [x, y.saturating_sub(1)],
        [x, y + 1],
    ]
}
fn reached(
    start: [u16; 2],
    allowed: &BTreeSet<[u16; 2]>,
    doors: &BTreeSet<[u16; 2]>,
) -> BTreeSet<[u16; 2]> {
    if !allowed.contains(&start) {
        return BTreeSet::new();
    }
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(tile) = queue.pop_front() {
        for next in adjacent(tile) {
            if allowed.contains(&next) && seen.insert(next) && !doors.contains(&next) {
                queue.push_back(next);
            }
        }
    }
    seen
}
