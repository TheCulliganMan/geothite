//! Dry refuge circuits, native stair sides and water-only pillar approaches.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Result, ensure};

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate(pack: &CompiledGamePack, room: &MapModule) -> Result<()> {
    let bodies = room
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    let mut floor = BTreeSet::new();
    let mut water = BTreeSet::new();
    let mut mouths = BTreeSet::new();
    for y in 0..28 {
        for x in 0..40 {
            let at = [x, y];
            let k = kind(pack, room, at)?;
            ensure!(
                !k.contains("HOP") && k != "PIT",
                "no cliff-adjacent drop or unpaired pit"
            );
            if k == "FLOOR" && !bodies.contains(&at) {
                floor.insert(at);
            }
            if k == "WATER" {
                ensure!(
                    x > 0 && x < 39 && y > 0 && y < 27,
                    "unconnected spring edge"
                );
                water.insert(at);
            }
            if matches!(k, "LADDER" | "WARP_CARPET_DOWN" | "CAVE") {
                mouths.insert(at);
            }
        }
    }
    ensure!(
        flood([17, 22], &floor) == floor,
        "all ordinary cave floor must be reachable"
    );
    ensure!(
        mouths == BTreeSet::from([[17, 23]]) && floor.contains(&[17, 22]),
        "original active mouth and clear step required"
    );
    for at in [
        [6, 5],
        [6, 8],
        [6, 9],
        [6, 10],
        [34, 3],
        [34, 8],
        [34, 9],
        [34, 10],
        [21, 19],
        [36, 18],
        [28, 24],
    ] {
        ensure!(floor.contains(&at), "dry cave approach blocked: {at:?}");
    }
    ensure!(
        room.blocks[4 * 20 + 3] == 0x30
            && room.blocks[4 * 20 + 17] == 0x36
            && kind(pack, room, [35, 9])? == "WALL",
        "correct native30/36 climb and closed corner side required"
    );
    for (start, end, allowed) in [
        (
            [13, 18],
            [13, 10],
            floor.iter().copied().filter(|p| p[0] <= 13).collect(),
        ),
        (
            [13, 18],
            [13, 10],
            floor.iter().copied().filter(|p| p[0] >= 13).collect(),
        ),
        (
            [21, 19],
            [36, 18],
            floor.iter().copied().filter(|p| p[1] <= 19).collect(),
        ),
        (
            [21, 19],
            [36, 18],
            floor.iter().copied().filter(|p| p[1] >= 18).collect(),
        ),
    ] {
        ensure!(
            flood(start, &allowed).contains(&end),
            "both dry fin and spring circuits required"
        );
    }
    ensure!(
        water.contains(&[22, 19]) && water.contains(&[23, 19]) && flood([22, 19], &water) == water,
        "original launch and one contained spring required"
    );
    for at in [[28, 16], [29, 16], [28, 17], [29, 17]] {
        ensure!(
            kind(pack, room, at)? == "WALL",
            "native spring pillar must block movement"
        );
    }
    let approaches = neighbors([28, 16])
        .filter(|p| floor.contains(p) || water.contains(p))
        .collect::<BTreeSet<_>>();
    ensure!(
        approaches == BTreeSet::from([[27, 16], [28, 15]])
            && approaches.iter().all(|p| water.contains(p)),
        "hidden pillar item must have only water approaches"
    );
    Ok(())
}

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

fn flood(start: [u16; 2], allowed: &BTreeSet<[u16; 2]>) -> BTreeSet<[u16; 2]> {
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some(p) = todo.pop_front() {
        for q in neighbors(p) {
            if allowed.contains(&q) && seen.insert(q) {
                todo.push_back(q);
            }
        }
    }
    seen
}
