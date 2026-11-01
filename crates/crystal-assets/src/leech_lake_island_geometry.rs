//! Island access, native drop, public-road and closed lake-boundary contracts.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Result, ensure};

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

pub(super) fn validate(
    pack: &CompiledGamePack,
    original: &MapModule,
    lake: &MapModule,
) -> Result<()> {
    ensure!(
        lake.attributes == original.attributes && lake.blocks.len() == 32 * 24,
        "original dimensions, doors and reciprocal world ports required"
    );
    for (index, (&old, &new)) in original.blocks.iter().zip(&lake.blocks).enumerate() {
        let (x, y) = (index % 32, index / 32);
        ensure!(
            new < 134
                && (old == new
                    || ((22..=28).contains(&x) && (12..=20).contains(&y))
                    || (x == 30 && (9..=10).contains(&y))),
            "island terrain changed unrelated metatile {x},{y}"
        );
    }
    let road = reachable(pack, lake, [32, 47], false)?;
    ensure!(
        road == reachable(pack, original, [32, 47], false)?,
        "original dry roads, clinic, pier, launch and cabin approaches must remain exact"
    );
    let surf = reachable(pack, lake, [32, 29], true)?;
    ensure!(
        !road.contains(&[46, 30])
            && surf.contains(&[46, 30])
            && surf.contains(&[52, 30])
            && surf.contains(&[52, 33]),
        "optional island must require Surf and have reachable trainer and reward approaches"
    );
    for &p in &surf {
        if kind(pack, lake, p)? == "WATER" {
            ensure!(
                p[0] != 0 && p[0] != 63 && p[1] != 0 && p[1] != 47,
                "unconnected lake edge needs visible rocks"
            );
        }
    }
    let mut landing_pairs = BTreeSet::new();
    for y in 24..=40 {
        for x in 44..=57 {
            let p = [x, y];
            if kind(pack, lake, p)? == "FLOOR" {
                for q in neighbors(p) {
                    if kind(pack, lake, q)? == "WATER" {
                        landing_pairs.insert((q, p));
                    }
                }
            }
        }
    }
    ensure!(
        landing_pairs == BTreeSet::from([([45, 30], [46, 30]), ([45, 31], [46, 31])]),
        "island must have only its two-tile west landing"
    );
    ensure!(
        !dry_without_grass(pack, lake, [46, 30])?.contains(&[52, 30]),
        "island discovery has a free dry grass bypass"
    );
    ensure!(
        dry_without_grass(pack, lake, [52, 36])?.contains(&[46, 30]),
        "ledge landing needs a quiet dry return to the launch"
    );
    for x in 50..=53 {
        ensure!(
            matches!(kind(pack, lake, [x, 33])?, "FLOOR" | "TALL_GRASS")
                && kind(pack, lake, [x, 34])? == "HOP_DOWN"
                && kind(pack, lake, [x, 35])? == "WALL"
                && kind(pack, lake, [x, 36])? == "FLOOR",
            "native downward lip, wall and two-tile landing required"
        );
    }
    for p in [[49, 34], [54, 34], [54, 35]] {
        ensure!(
            matches!(kind(pack, lake, p)?, "WALL" | "HEADBUTT_TREE"),
            "native ledge side must be closed permanently"
        );
    }
    for y in 0..48 {
        for x in 0..64 {
            ensure!(
                kind(pack, lake, [x, y])? != "CUT_TREE",
                "ordinary canopy must not hide an incidental Cut gate"
            );
        }
    }
    ensure!(
        kind(pack, lake, [60, 20])? == "WHIRLPOOL"
            && !surf.contains(&[60, 20])
            && kind(pack, lake, [61, 20])? == "BUOY",
        "native current and cache rock required"
    );
    let mut open = lake.clone();
    open.blocks[10 * 32 + 30] = 0x36;
    let cleared = reachable(pack, &open, [32, 29], true)?;
    ensure!(
        cleared.contains(&[60, 20]),
        "native current removal must open the cache approach"
    );
    let approaches = neighbors([61, 20])
        .filter(|p| cleared.contains(p))
        .collect::<BTreeSet<_>>();
    ensure!(
        approaches == BTreeSet::from([[60, 20]]),
        "hidden rock cache must have only the Whirlpool-controlled approach"
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

fn dry_without_grass(
    pack: &CompiledGamePack,
    map: &MapModule,
    start: [u16; 2],
) -> Result<BTreeSet<[u16; 2]>> {
    let mut seen = BTreeSet::from([start]);
    let mut todo = VecDeque::from([start]);
    while let Some(p) = todo.pop_front() {
        for q in neighbors(p) {
            if q[0] < 64
                && q[1] < 48
                && !seen.contains(&q)
                && !map.objects.iter().any(|o| [o.x, o.y] == q)
                && kind(pack, map, q)? == "FLOOR"
            {
                seen.insert(q);
                todo.push_back(q);
            }
        }
    }
    Ok(seen)
}
