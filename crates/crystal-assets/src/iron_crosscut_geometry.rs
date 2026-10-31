//! Body-aware walking, return-lip and native stair contracts.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Result, ensure};

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

fn neighbors([x, y]: [u16; 2]) -> impl Iterator<Item = [u16; 2]> {
    [
        x.checked_sub(1).map(|x| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_sub(1).map(|y| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

fn distances(start: [u16; 2], floor: &BTreeSet<[u16; 2]>) -> BTreeMap<[u16; 2], usize> {
    let mut seen = BTreeMap::from([(start, 0)]);
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for next in neighbors(at) {
            if floor.contains(&next) && !seen.contains_key(&next) {
                seen.insert(next, seen[&at] + 1);
                queue.push_back(next);
            }
        }
    }
    seen
}

pub(super) fn validate(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let occupied = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut ladders = BTreeSet::new();
    let mut lips = BTreeSet::new();
    for y in 0..26 {
        for x in 0..32 {
            let at = [x, y];
            let token = kind(pack, room, at)?;
            if x == 0 || y == 0 || x == 31 || y == 25 {
                ensure!(token == "WALL", "solid native crosscut perimeter required");
            }
            ensure!(
                !matches!(token, "WATER" | "PIT" | "CAVE" | "WARP_CARPET_DOWN"),
                "no unpaired terrain transitions"
            );
            if token == "FLOOR" && !occupied.contains(&at) {
                floor.insert(at);
            }
            if token == "LADDER" {
                ladders.insert(at);
            }
            if token.starts_with("HOP") {
                ensure!(
                    token == "HOP_LEFT",
                    "only the reviewed westward lip is allowed"
                );
                lips.insert(at);
            }
        }
    }
    ensure!(
        ladders == BTreeSet::from([[7, 21], [25, 5]]),
        "both original ladder quadrants required"
    );
    ensure!(
        lips == BTreeSet::from([[13, 18], [13, 19]]),
        "exactly two native westward lanes required"
    );
    for object in &room.objects {
        ensure!(
            kind(pack, room, [object.x, object.y])? == "FLOOR",
            "original actor requires dry ground"
        );
    }
    let reached = distances([6, 21], &floor);
    ensure!(
        reached.len() == floor.len(),
        "every dry floor tile must remain reachable without a jump"
    );
    for at in [
        [4, 5],
        [6, 7],
        [6, 8],
        [6, 9],
        [6, 10],
        [9, 13],
        [10, 13],
        [15, 10],
        [19, 13],
        [24, 5],
        [25, 4],
        [25, 6],
        [25, 18],
        [14, 18],
        [14, 19],
    ] {
        ensure!(reached.contains_key(&at), "dry anchor unreachable: {at:?}");
    }
    ensure!(
        room.blocks[4 * 16 + 3] == 0x30,
        "native straight bench stair required"
    );
    ensure!(
        room.blocks[9 * 16 + 6] == 0x3b
            && room.blocks[8 * 16 + 6] == 0x2e
            && room.blocks[10 * 16 + 6] == 0x2e
            && room.blocks[9 * 16 + 5] == 2
            && room.blocks[9 * 16 + 7] == 2,
        "standalone lip must not join a cliff or have a side bypass"
    );
    for y in [18, 19] {
        ensure!(
            kind(pack, room, [12, y])? == "WALL"
                && floor.contains(&[11, y])
                && floor.contains(&[14, y]),
            "safe jump and blocked reverse required"
        );
        let reverse = distances([11, y], &floor);
        ensure!(
            reverse.get(&[14, y]).is_some_and(|steps| *steps >= 24),
            "useful ordinary uphill return required"
        );
    }
    Ok(())
}
