//! Physical source contracts for the actual Brainerd facade/room implementation.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::brainerd_city_modpack::{Door, Room, Town};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn kind<'a>(
    pack: &'a CompiledGamePack,
    module: &MapModule,
    [x, y]: [u16; 2],
) -> Result<&'a str> {
    ensure!(
        x < module.attributes.width * 2 && y < module.attributes.height * 2,
        "Brainerd tile outside {}",
        module.id
    );
    let block = module.blocks
        [usize::from(y / 2) * usize::from(module.attributes.width) + usize::from(x / 2)];
    let collision = pack
        .data
        .tilesets
        .get(&module.attributes.tileset_name)
        .context("Brainerd native tileset")?
        .collision
        .get(&format!("{block:02x}"))
        .context("Brainerd native block collision")?;
    Ok(collision
        .get(usize::from(y % 2 * 2 + x % 2))
        .context("Brainerd collision quadrant")?)
}
fn adjacent([x, y]: [u16; 2]) -> Vec<[u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
    .collect()
}
fn walk(pack: &CompiledGamePack, module: &MapModule, water: bool) -> Result<BTreeSet<[u16; 2]>> {
    let mut cells = BTreeSet::new();
    for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            let collision = kind(pack, module, [x, y])?;
            ensure!(
                collision != "CUT_TREE",
                "Brainerd contains an incidental Cut gate"
            );
            if matches!(
                collision,
                "FLOOR" | "DOOR" | "TALL_GRASS" | "WARP_CARPET_DOWN"
            ) || (water && collision == "WATER")
            {
                cells.insert([x, y]);
            }
        }
    }
    for actor in &module.objects {
        cells.remove(&[actor.x, actor.y]);
    }
    Ok(cells)
}
fn flood(
    start: [u16; 2],
    cells: &BTreeSet<[u16; 2]>,
    doors: &BTreeSet<[u16; 2]>,
) -> BTreeSet<[u16; 2]> {
    let mut reached = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for next in adjacent(at) {
            if cells.contains(&next) && reached.insert(next) && !doors.contains(&next) {
                queue.push_back(next);
            }
        }
    }
    reached
}
fn distance(start: [u16; 2], target: [u16; 2], cells: &BTreeSet<[u16; 2]>) -> Option<usize> {
    let mut costs = BTreeMap::from([(start, 0)]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        let cost = costs[&at];
        if at == target {
            return Some(cost);
        }
        for next in adjacent(at) {
            if cells.contains(&next) && !costs.contains_key(&next) {
                costs.insert(next, cost + 1);
                queue.push_back(next);
            }
        }
    }
    None
}
fn safe_door(
    pack: &CompiledGamePack,
    module: &MapModule,
    door: &Door,
    reached: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    ensure!(
        kind(pack, module, door.tile)? == "DOOR"
            && kind(pack, module, door.return_tile)? == "FLOOR"
            && door.return_tile == [door.tile[0], door.tile[1] + 1]
            && door.arrival_tile == door.return_tile
            && reached.contains(&door.tile)
            && reached.contains(&door.return_tile),
        "Brainerd door {} or passive return unsafe",
        door.id
    );
    Ok(())
}

pub(crate) fn validate(
    pack: &CompiledGamePack,
    town: &Town,
    modules: &[(MapModule, [u16; 2])],
) -> Result<()> {
    let exterior = &modules[0].0;
    let expected = [
        ("MN_R06", "S", [16, 22], [16, 20]),
        ("MN_R07", "N", [16, 1], [16, 3]),
        ("MN_R15", "W", [1, 12], [3, 12]),
        ("MN_R18", "E", [30, 12], [28, 12]),
    ];
    ensure!(
        town.ports.len() == 4
            && town.doors.len() == 5
            && town.interiors.len() == 5
            && modules.len() == 5,
        "complete Brainerd port/door/room roster"
    );
    let doors = town.doors.iter().map(|d| d.tile).collect::<BTreeSet<_>>();
    ensure!(doors.len() == 5, "Brainerd overlapping facade doors");
    let cells = walk(pack, exterior, false)?;
    let reached = flood([32, 44], &cells, &doors);
    let surf = flood([32, 44], &walk(pack, exterior, true)?, &doors);
    for (port, (route, side, gate, landing)) in town.ports.iter().zip(expected) {
        ensure!(
            port.route == route
                && port.side == side
                && port.gate_metatile == gate
                && port.landing_metatile == landing,
            "reserved Brainerd port changed"
        );
        for [x, y] in [gate, landing] {
            ensure!(
                reached.contains(&[x * 2, y * 2]),
                "Brainerd reserved port disconnected"
            );
        }
    }
    let perimeter = |[x, y]: [u16; 2]| x == 0 || x == 63 || y == 0 || y == 47;
    ensure!(
        reached
            .iter()
            .copied()
            .filter(|p| perimeter(*p))
            .collect::<BTreeSet<_>>()
            == BTreeSet::from([
                [0, 24],
                [0, 25],
                [63, 24],
                [63, 25],
                [32, 0],
                [33, 0],
                [32, 47],
                [33, 47]
            ]),
        "unexpected Brainerd walking perimeter opening"
    );
    let mut actual_doors = BTreeSet::new();
    for y in 0..48 {
        for x in 0..64 {
            match kind(pack, exterior, [x, y])? {
                "DOOR" => {
                    actual_doors.insert([x, y]);
                }
                "WATER" => ensure!(
                    !perimeter([x, y]) || !surf.contains(&[x, y]),
                    "unconnected Brainerd Surf edge"
                ),
                _ => {}
            }
        }
    }
    ensure!(actual_doors == doors, "Brainerd false/unbound facade door");
    for door in &town.doors {
        safe_door(pack, exterior, door, &reached)?;
    }
    for resident in &town.residents {
        ensure!(
            kind(pack, exterior, resident.tile)? == "FLOOR"
                && adjacent(resident.tile).iter().any(|p| reached.contains(p)),
            "Brainerd resident inaccessible"
        );
    }
    for sign in &town.signs {
        ensure!(
            kind(pack, exterior, sign.tile)? == "WALL"
                && adjacent(sign.tile).iter().any(|p| reached.contains(p)),
            "Brainerd sign inaccessible"
        );
    }
    let bodies = exterior
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        bodies.len() == exterior.objects.len(),
        "Brainerd actor overlap"
    );
    for y in 7..10 {
        for cy in [y * 2, y * 2 + 1] {
            ensure!(
                kind(pack, exterior, [46, cy])? == "WALL"
                    && kind(pack, exterior, [49, cy])? == "WALL",
                "Brainerd pier side rail gap"
            );
        }
    }
    for x in 46..50 {
        ensure!(
            kind(pack, exterior, [x, 12])? == "WALL",
            "Brainerd pier head gap"
        );
    }
    ensure!(
        reached.contains(&[44, 20])
            && kind(pack, exterior, [44, 20])? == "FLOOR"
            && kind(pack, exterior, [44, 19])? == "WATER",
        "Brainerd public lake launch"
    );
    ensure!(
        town.one_way_ledges.len() == 1,
        "Brainerd needs its one authored useful return ledge"
    );
    let ledge = &town.one_way_ledges[0];
    ensure!(
        ledge.direction == "east"
            && ledge.upper_approach == [23, 38]
            && ledge.hop_from == [24, 38]
            && ledge.landing == [26, 38]
            && kind(pack, exterior, ledge.hop_from)? == "HOP_RIGHT"
            && kind(pack, exterior, ledge.reverse_blocked)? == "WALL"
            && ledge
                .side_barriers
                .iter()
                .all(|p| kind(pack, exterior, *p).is_ok_and(|k| k == "WALL")),
        "Brainerd ledge direction/reverse/side seal"
    );
    ensure!(
        reached.contains(&ledge.upper_approach)
            && reached.contains(&ledge.landing)
            && distance(ledge.landing, ledge.upper_approach, &cells)
                .is_some_and(|d| d >= usize::from(ledge.minimum_return_steps) && d >= 9),
        "Brainerd ordinary ledge return"
    );
    let mut no_ramp = cells.clone();
    let [x0, y0, x1, y1] = ledge.return_passage_core;
    ensure!(
        [x0, y0, x1, y1] == [20, 40, 22, 42],
        "Brainerd native ramp passage changed"
    );
    for y in y0..y1 {
        for x in x0..x1 {
            no_ramp.remove(&[x, y]);
        }
    }
    ensure!(
        distance(ledge.landing, ledge.upper_approach, &no_ramp).is_none(),
        "Brainerd ledge has an uphill bypass"
    );
    for (module, arrival) in modules.iter().skip(1) {
        let source = town
            .interiors
            .iter()
            .find(|r| r.id == module.id)
            .context("Brainerd room source")?;
        room(pack, module, source, *arrival)?;
    }
    Ok(())
}

fn room(
    pack: &CompiledGamePack,
    module: &MapModule,
    source: &Room,
    arrival: [u16; 2],
) -> Result<()> {
    let cells = walk(pack, module, false)?;
    let reached = flood(arrival, &cells, &BTreeSet::new());
    ensure!(
        source.passive_arrival.index == 3
            && kind(pack, module, arrival)? == "FLOOR"
            && cells.contains(&arrival),
        "Brainerd passive room arrival"
    );
    let expected = source.exits.iter().map(|e| e.tile).collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            let collision = kind(pack, module, [x, y])?;
            ensure!(
                !matches!(collision, "DOOR" | "PIT" | "LADDER" | "STAIRCASE"),
                "unbound Brainerd room floor trigger"
            );
            if collision == "WARP_CARPET_DOWN" {
                actual.insert([x, y]);
            }
            if x == 0
                || y == 0
                || x == module.attributes.width * 2 - 1
                || y == module.attributes.height * 2 - 1
            {
                ensure!(
                    !matches!(
                        collision,
                        "FLOOR" | "TALL_GRASS" | "WATER" | "WATERFALL" | "WARP_CARPET_DOWN"
                    ) || expected.contains(&[x, y]),
                    "Brainerd room perimeter leak"
                );
            }
        }
    }
    ensure!(
        expected.len() == 2 && actual == expected && expected.is_subset(&reached),
        "Brainerd room carpet exits unreachable"
    );
    let bodies = module
        .objects
        .iter()
        .map(|a| [a.x, a.y])
        .collect::<BTreeSet<_>>();
    ensure!(
        bodies.len() == module.objects.len(),
        "Brainerd room actor overlap"
    );
    for resident in &source.residents {
        ensure!(
            kind(pack, module, resident.tile)? == "FLOOR",
            "Brainerd room actor off Floor"
        );
        if let Some(counter) = resident.counter_tile {
            ensure!(
                kind(pack, module, counter)? == "COUNTER"
                    && resident.approach.is_some_and(|p| reached.contains(&p)),
                "Brainerd native counter service inaccessible"
            );
        } else {
            ensure!(
                adjacent(resident.tile).iter().any(|p| reached.contains(p)),
                "Brainerd room actor inaccessible"
            );
        }
    }
    for sign in &source.signs {
        ensure!(
            matches!(
                kind(pack, module, sign.tile)?,
                "WALL" | "WINDOW" | "BOOKSHELF"
            ) && sign.approach.is_some_and(|p| reached.contains(&p)),
            "Brainerd room sign inaccessible"
        );
    }
    if let Some(pc) = &source.pc {
        ensure!(
            kind(pack, module, pc.tile)? == "PC" && reached.contains(&pc.approach),
            "Brainerd actual PC quadrant inaccessible"
        );
    }
    Ok(())
}
