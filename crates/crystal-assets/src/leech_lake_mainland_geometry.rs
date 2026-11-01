//! Mainland-only edits, a priced grass shortcut and preserved town anchors.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};

use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate(pack: &CompiledGamePack, old: &MapModule, lake: &MapModule) -> Result<()> {
    ensure!(
        lake.attributes == old.attributes && lake.blocks.len() == 32 * 24,
        "original native dimensions, palette, ports and identity required"
    );
    for (i, (&before, &after)) in old.blocks.iter().zip(&lake.blocks).enumerate() {
        let (x, y) = (i % 32, i / 32);
        ensure!(
            before == after
                || ((3..=18).contains(&x)
                    && (4..=22).contains(&y)
                    && matches!(before, 0x02 | 0x04 | 0x05)
                    && matches!(after, 0x02..=0x05 | 0x3c..=0x3f)),
            "mainland must retain every original facade, road, sign, pier, shore and island: {x},{y}"
        );
    }
    let ordinary = distances(pack, lake, [11, 14], false, false)?;
    let trail = distances(pack, lake, [11, 14], true, true)?;
    let dry_length = ordinary
        .get(&[9, 36])
        .context("ordinary public cabin path")?;
    let trail_length = trail.get(&[9, 36]).context("grassy woodland cabin link")?;
    ensure!(
        *dry_length == 54 && *trail_length <= 38 && trail_length + 16 <= *dry_length,
        "woodland link must save meaningful distance against the dry public path"
    );
    let dry_west = distances(pack, lake, [11, 14], false, true)?;
    ensure!(
        !dry_west.contains_key(&[9, 36]),
        "woodland link has a free dry bypass"
    );
    for target in [[18, 24], [18, 26]] {
        ensure!(
            trail.contains_key(&target) && !ordinary.contains_key(&target),
            "trail discovery must require its grass: {target:?}"
        );
    }
    let mut open = lake.clone();
    open.objects.retain(|o| o.script != "MnLeechLakeCheck");
    let public = distances(pack, &open, [32, 47], false, false)?;
    for at in [
        [32, 0],
        [33, 0],
        [11, 14],
        [9, 36],
        [31, 29],
        [35, 22],
        [33, 44],
    ] {
        ensure!(
            public.contains_key(&at),
            "original public approach blocked: {at:?}"
        );
    }
    for event in &old.events.bg_events {
        if event.x < 44 {
            ensure!(
                neighbors([event.x, event.y]).any(|p| public.contains_key(&p)),
                "original readable board lost its public approach"
            );
        }
    }
    for o in &lake.objects {
        ensure!(
            kind(pack, lake, [o.x, o.y])? == "FLOOR",
            "actor needs ordinary native ground"
        );
    }
    let all_access = crate::authored_town_geometry::reachable(pack, &open, [32, 47], true)?;
    for y in 0..48 {
        for x in 0..64 {
            let k = kind(pack, lake, [x, y])?;
            if matches!(k, "FLOOR" | "TALL_GRASS")
                && !open.objects.iter().any(|o| [o.x, o.y] == [x, y])
            {
                ensure!(
                    all_access.contains(&[x, y]),
                    "stranded native ground: {x},{y}"
                );
            }
            ensure!(
                k != "CUT_TREE",
                "ordinary canopy must not hide an incidental Cut gate"
            );
            if k == "TALL_GRASS" && x < 44 {
                ensure!(
                    (14..=17).contains(&x) && (20..=27).contains(&y),
                    "mainland grass must belong to the authored habitat"
                );
            }
        }
    }
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

fn distances(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: [u16; 2],
    grass: bool,
    west: bool,
) -> Result<BTreeMap<[u16; 2], u16>> {
    let mut bodies = map
        .objects
        .iter()
        .map(|o| [o.x, o.y])
        .collect::<BTreeSet<_>>();
    bodies.extend([[19, 24], [19, 26]]);
    let mut seen = BTreeMap::from([(start, 0)]);
    let mut todo = VecDeque::from([start]);
    while let Some(p) = todo.pop_front() {
        for q in neighbors(p) {
            if q[0] >= 64
                || q[1] >= 48
                || (west && q[0] >= 22)
                || bodies.contains(&q)
                || seen.contains_key(&q)
            {
                continue;
            }
            let k = kind(pack, map, q)?;
            if k == "FLOOR" || (grass && k == "TALL_GRASS") {
                seen.insert(q, seen[&p] + 1);
                todo.push_back(q);
            }
        }
    }
    Ok(seen)
}
