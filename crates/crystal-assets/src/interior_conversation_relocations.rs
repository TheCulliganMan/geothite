//! Explicit ordinary-resident moves needed by accepted cave terrain.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Relocation {
    pub map: String,
    script: String,
    sprite: String,
    before: [u16; 2],
    tile: [u16; 2],
    approach: [u16; 2],
    arrival: [u16; 2],
}

pub(crate) fn apply(
    pack: &CompiledGamePack,
    accepted_base: &MapModule,
    map: &mut MapModule,
    relocations: &[Relocation],
) -> Result<()> {
    let mut scripts = BTreeSet::new();
    for move_ in relocations.iter().filter(|r| r.map == map.id) {
        ensure!(
            scripts.insert(&move_.script),
            "duplicate resident relocation"
        );
        let index = map
            .objects
            .iter()
            .position(|o| o.script == move_.script)
            .context("newer conversation resident")?;
        let actor = &map.objects[index];
        ensure!(
            actor.sprite == move_.sprite
                && [actor.x, actor.y] == move_.before
                && actor.object_type == "OBJECTTYPE_SCRIPT"
                && !accepted_base
                    .objects
                    .iter()
                    .any(|o| o.script == actor.script),
            "only the exact newer ordinary conversation resident may move"
        );
        let body = map
            .scripts
            .get(&move_.script)
            .and_then(serde_json::Value::as_array)
            .context("ordinary conversation body")?;
        let text = ordinary_text(body).context(
            "service, gift, trainer and progression actors cannot be relocated as ordinary speech",
        )?;
        ensure!(
            text["args"].as_array().is_some_and(|args| args.len() == 1)
                && text["args"][0]
                    .as_str()
                    .is_some_and(|label| map.script_text_bodies.contains_key(label)),
            "service, gift, trainer and progression actors cannot be relocated as ordinary speech"
        );
        let [x, y] = move_.tile;
        ensure!(
            !map.objects.iter().any(|o| [o.x, o.y] == move_.tile)
                && !map.events.warps.iter().any(|w| [w.x, w.y] == move_.tile)
                && !map
                    .events
                    .coord_events
                    .iter()
                    .any(|e| [e.x, e.y] == move_.tile)
                && !map
                    .events
                    .bg_events
                    .iter()
                    .any(|e| [e.x, e.y] == move_.tile)
                && kind(pack, map, move_.tile)? == "FLOOR"
                && distance(move_.tile, move_.approach) == 1
                && kind(pack, map, move_.approach)? == "FLOOR"
                && kind(pack, map, move_.arrival)? == "FLOOR"
                && map
                    .events
                    .warps
                    .iter()
                    .any(|w| distance([w.x, w.y], move_.arrival) <= 1),
            "resident needs a clear native floor position, adjacent approach and real arrival"
        );
        map.objects[index].x = x;
        map.objects[index].y = y;
        let accessible = reachable(pack, map, move_.arrival, false)?;
        ensure!(
            accessible.contains(&move_.approach),
            "relocated resident cannot be approached from the real arrival"
        );
        for warp in &map.events.warps {
            ensure!(
                accessible.contains(&[warp.x, warp.y]),
                "relocated resident blocks a room exit"
            );
        }
    }
    if !scripts.is_empty() {
        authoring::refresh_event_section(map);
    }
    Ok(())
}

fn ordinary_text(body: &[Value]) -> Option<&Value> {
    if body.len() == 1 && body[0]["command"] == "jumptextfaceplayer" {
        return Some(&body[0]);
    }
    let expected = [
        "faceplayer",
        "opentext",
        "writetext",
        "waitbutton",
        "closetext",
        "end",
    ];
    if body
        .iter()
        .map(|c| c["command"].as_str())
        .eq(expected.into_iter().map(Some))
        && body.iter().enumerate().all(|(index, c)| {
            index == 2 || c["args"].as_array().is_some_and(|args| args.is_empty())
        })
    {
        return Some(&body[2]);
    }
    None
}

fn distance(a: [u16; 2], b: [u16; 2]) -> u32 {
    u32::from(a[0].abs_diff(b[0])) + u32::from(a[1].abs_diff(b[1]))
}
