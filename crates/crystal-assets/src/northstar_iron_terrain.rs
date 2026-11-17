//! Small native terrain edits on the two working floors of the Iron Range mine.
use std::collections::{BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::authored_town_geometry::kind;
use crate::northstar_cave_geometry::resting_tiles;
use crate::{CompiledGamePack, MapModule};

type Tile = [u16; 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Terrain {
    schema_version: u8,
    rooms: Vec<RoomPatch>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomPatch {
    id: String,
    changes: Vec<BlockChange>,
    relocations: Vec<LandingChoices>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BlockChange {
    at: Tile,
    from: u16,
    to: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LandingChoices {
    origin: Tile,
    landings: Vec<Tile>,
}

#[derive(Serialize)]
pub(super) struct Relocation {
    map_id: String,
    from: Tile,
    landings: Vec<Tile>,
}

impl Terrain {
    pub(super) fn parse(json: &str) -> Result<Self> {
        let terrain: Self = serde_json::from_str(json)?;
        ensure!(
            terrain.schema_version == 1
                && terrain.rooms.len() == 2
                && terrain
                    .rooms
                    .iter()
                    .map(|r| r.id.as_str())
                    .collect::<BTreeSet<_>>()
                    == BTreeSet::from(["MnIronHaulage", "MnIronSump"]),
            "expected the two reviewed Iron Range floors"
        );
        Ok(terrain)
    }

    pub(super) fn apply(
        &self,
        pack: &CompiledGamePack,
        room: &mut MapModule,
    ) -> Result<Vec<Relocation>> {
        let Some(patch) = self.rooms.iter().find(|patch| patch.id == room.id) else {
            return Ok(Vec::new());
        };
        let original = pack
            .data
            .maps
            .get(&room.id)
            .context("installed mine floor")?;
        let width = room.attributes.width;
        let shutter = if room.id == "MnIronHaulage" {
            [9, 8]
        } else {
            [16, 3]
        };
        let mut edited = BTreeSet::new();
        ensure!(!patch.changes.is_empty(), "empty mine terrain revision");
        for edit in &patch.changes {
            let [x, y] = edit.at;
            ensure!(
                x < width
                    && y < room.attributes.height
                    && edit.at != shutter
                    && edited.insert(edit.at)
                    && edit.from != edit.to,
                "invalid or duplicate mine terrain edit at {:?}",
                edit.at
            );
            let block = &mut room.blocks[usize::from(y) * usize::from(width) + usize::from(x)];
            ensure!(
                *block == edit.from,
                "mine terrain preimage differs at {:?}",
                edit.at
            );
            *block = edit.to;
        }
        crate::northstar_cave_overhaul::validate_native_art(pack, room)?;
        for opened in [false, true] {
            let mut before = original.clone();
            let mut after = room.clone();
            if opened {
                let index = usize::from(shutter[1]) * usize::from(width) + usize::from(shutter[0]);
                ensure!(
                    before.blocks[index] == 0x41 && after.blocks[index] == 0x41,
                    "original pressure shutter"
                );
                before.blocks[index] = 13;
                after.blocks[index] = 13;
            }
            validate_passages(pack, &before, &after)?;
        }
        let mut moves = Vec::new();
        for [mx, my] in edited {
            for [dx, dy] in [[0, 0], [1, 0], [0, 1], [1, 1]] {
                let tile = [mx * 2 + dx, my * 2 + dy];
                let was = kind(pack, original, tile)?;
                let now = kind(pack, room, tile)?;
                ensure!(
                    was != "WATER" || now == "WATER",
                    "saved swimmer lost water at {tile:?}"
                );
                if was == "FLOOR" && now != "FLOOR" {
                    let choices = patch
                        .relocations
                        .iter()
                        .find(|r| r.origin == tile)
                        .context("saved floor needs authored landing choices")?;
                    let nearby = nearby_landings(pack, original, room, tile)?;
                    ensure!(
                        !choices.landings.is_empty()
                            && choices.landings.iter().all(|to| nearby.contains(to)),
                        "mine checkpoint alternative crosses a wall from {tile:?}"
                    );
                    moves.push(Relocation {
                        map_id: room.id.clone(),
                        from: tile,
                        landings: choices.landings.clone(),
                    });
                }
            }
        }
        ensure!(
            moves.len() == patch.relocations.len(),
            "only changed footholds require migration"
        );
        Ok(moves)
    }
}

// Check each passive arrival independently. A newly usable shortcut may join
// two paths on one side of a shutter, but must never bypass the shutter itself.
fn validate_passages(pack: &CompiledGamePack, before: &MapModule, after: &MapModule) -> Result<()> {
    let reserved = |tile: Tile| {
        before.events.warps.iter().any(|w| [w.x, w.y] == tile)
            || before.objects.iter().any(|a| [a.x, a.y] == tile)
            || before
                .events
                .coord_events
                .iter()
                .any(|e| [e.x, e.y] == tile)
            || before.events.bg_events.iter().any(|e| [e.x, e.y] == tile)
    };
    for y in 0..before.attributes.height * 2 {
        for x in 0..before.attributes.width * 2 {
            let tile = [x, y];
            ensure!(
                !reserved(tile) || kind(pack, before, tile)? == kind(pack, after, tile)?,
                "mine event lost its original footing at {tile:?}"
            );
        }
    }
    for warp in before
        .events
        .warps
        .iter()
        .filter(|warp| warp.index % 2 == 0)
    {
        let entry = [warp.x, warp.y];
        for surf in [false, true] {
            let old = resting_tiles(pack, before, entry, surf)?;
            let new = resting_tiles(pack, after, entry, surf)?;
            for port in &before.events.warps {
                let tile = [port.x, port.y];
                ensure!(
                    old.contains(&tile) == new.contains(&tile),
                    "changed ladder access from {entry:?} to {tile:?} (Surf={surf})"
                );
            }
            for target in before
                .objects
                .iter()
                .map(|a| [a.x, a.y])
                .chain(before.events.bg_events.iter().map(|e| [e.x, e.y]))
            {
                let adjacent = |set: &BTreeSet<Tile>| neighbors(target).any(|p| set.contains(&p));
                ensure!(
                    adjacent(&old) == adjacent(&new),
                    "changed access to mine control or actor at {target:?}"
                );
            }
        }
    }
    Ok(())
}

fn neighbors([x, y]: Tile) -> impl Iterator<Item = Tile> {
    [
        y.checked_sub(1).map(|y| [x, y]),
        x.checked_sub(1).map(|x| [x, y]),
        x.checked_add(1).map(|x| [x, y]),
        y.checked_add(1).map(|y| [x, y]),
    ]
    .into_iter()
    .flatten()
}

// Follow old walkable ground only. Candidates have to be safe on both sides
// of the upgrade, so a legacy checkpoint cannot skip a pressure gate.
fn nearby_landings(
    pack: &CompiledGamePack,
    old: &MapModule,
    new: &MapModule,
    from: Tile,
) -> Result<Vec<Tile>> {
    let mut seen = BTreeSet::from([from]);
    let mut todo = VecDeque::from([(from, 0)]);
    let mut result = Vec::new();
    while let Some((tile, steps)) = todo.pop_front() {
        if steps == 2 {
            continue;
        }
        for next in neighbors(tile) {
            if next[0] >= old.attributes.width * 2
                || next[1] >= old.attributes.height * 2
                || !seen.insert(next)
                || kind(pack, old, next)? != "FLOOR"
                || old.objects.iter().any(|a| [a.x, a.y] == next)
                || old.events.warps.iter().any(|w| [w.x, w.y] == next)
                || old.events.coord_events.iter().any(|e| [e.x, e.y] == next)
                || old.events.bg_events.iter().any(|e| [e.x, e.y] == next)
            {
                continue;
            }
            if kind(pack, new, next)? == "FLOOR" {
                result.push(next);
            }
            todo.push_back((next, steps + 1));
        }
    }
    ensure!(
        !result.is_empty(),
        "no nearby stable mine floor for {from:?}"
    );
    Ok(result)
}
