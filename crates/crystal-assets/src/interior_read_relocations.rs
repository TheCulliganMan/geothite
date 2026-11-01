//! Move existing read-only details onto their reviewed native furniture.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReadRelocation {
    script: String,
    before: [u16; 2],
    tile: [u16; 2],
    approach: [u16; 2],
}

pub(crate) fn apply(
    pack: &CompiledGamePack,
    map: &mut MapModule,
    arrival: [u16; 2],
    relocations: &[ReadRelocation],
) -> Result<()> {
    if relocations.is_empty() {
        return Ok(());
    }
    let public = reachable(pack, map, arrival, false)?;
    let mut scripts = BTreeSet::new();
    for relocation in relocations {
        ensure!(
            scripts.insert(&relocation.script),
            "duplicate relocated read"
        );
        let matching = map
            .events
            .bg_events
            .iter()
            .enumerate()
            .filter(|(_, event)| event.script == relocation.script)
            .collect::<Vec<_>>();
        ensure!(matching.len() == 1, "read must have one existing event");
        let (index, event) = matching[0];
        ensure!(
            event.event_type == "BGEVENT_READ"
                && [event.x, event.y] == relocation.before
                && relocation.before != relocation.tile,
            "stale/non-read relocation: {}",
            relocation.script
        );
        let body = map
            .scripts
            .get(&relocation.script)
            .context("existing read script")?;
        ensure!(
            body.as_array().is_some_and(|commands| commands.len() == 1
                && commands[0]["command"] == "jumptext"),
            "only an ordinary read-only detail may move"
        );
        ensure!(
            matches!(
                kind(pack, map, relocation.tile)?,
                "WALL" | "BOOKSHELF" | "COUNTER" | "MART_SHELF"
            ),
            "read must target solid native furniture"
        );
        let adjacent = (relocation.tile[0] == relocation.approach[0]
            && relocation.tile[1].abs_diff(relocation.approach[1]) == 1)
            || (relocation.tile[1] == relocation.approach[1]
                && relocation.tile[0].abs_diff(relocation.approach[0]) == 1);
        ensure!(
            adjacent
                && public.contains(&relocation.approach)
                && kind(pack, map, relocation.approach)? == "FLOOR",
            "read needs a reachable adjacent floor approach"
        );
        ensure!(
            !map.events
                .bg_events
                .iter()
                .any(|e| [e.x, e.y] == relocation.tile)
                && !map
                    .events
                    .warps
                    .iter()
                    .any(|w| [w.x, w.y] == relocation.tile || [w.x, w.y] == relocation.approach)
                && !map
                    .events
                    .coord_events
                    .iter()
                    .any(|e| [e.x, e.y] == relocation.tile || [e.x, e.y] == relocation.approach)
                && !map.objects.iter().any(|o| [o.x, o.y] == relocation.tile),
            "read overlaps an existing interaction, actor or arrival/scene"
        );
        map.events.bg_events[index].x = relocation.tile[0];
        map.events.bg_events[index].y = relocation.tile[1];
    }
    Ok(())
}

#[cfg(test)]
#[path = "interior_read_relocations_tests.rs"]
mod tests;
