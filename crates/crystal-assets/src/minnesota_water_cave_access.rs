//! Native bounded river entry for the previously sealed Moorhead levee.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;
use crate::grand_marais_modpack::authoring::command as c;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Access {
    cells: Vec<Cell>,
    sign: [u16; 2],
    sign_approach: [u16; 2],
    shore: [u16; 2],
    sample: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cell {
    metatile: [u16; 2],
    before: String,
    after: String,
}

pub(super) fn install(pack: &mut CompiledGamePack, access: &Access) -> Result<()> {
    let mut map = pack
        .data
        .maps
        .get("MnMoorhead")
        .context("Moorhead")?
        .clone();
    ensure!(
        map.attributes.width == 16
            && map.attributes.height == 12
            && map.attributes.tileset_name == "johto_modern"
            && access.cells.len() == 23,
        "current Red River levee required"
    );
    let mut seen = BTreeSet::new();
    for cell in &access.cells {
        let [x, y] = cell.metatile;
        ensure!(
            x < 16 && y < 12 && seen.insert([x, y]),
            "unique bounded levee cell required"
        );
        ensure!(
            !map.objects.iter().any(|o| [o.x / 2, o.y / 2] == [x, y])
                && !map
                    .events
                    .warps
                    .iter()
                    .any(|w| [w.x / 2, w.y / 2] == [x, y])
                && !map
                    .events
                    .bg_events
                    .iter()
                    .any(|e| [e.x / 2, e.y / 2] == [x, y]),
            "levee patch overlaps an original event"
        );
        let index = usize::from(y) * 16 + usize::from(x);
        ensure!(
            map.blocks[index] == u16::from_str_radix(&cell.before, 16)?,
            "levee preimage differs"
        );
        map.blocks[index] = u16::from_str_radix(&cell.after, 16)?;
    }
    for (tile, expected) in [
        (access.shore, "FLOOR"),
        (access.sample, "WATER"),
        (access.sign, "WALL"),
        (access.sign_approach, "FLOOR"),
    ] {
        ensure!(
            crate::authored_town_geometry::kind(pack, &map, tile)? == expected,
            "native levee collision differs at {tile:?}"
        );
    }
    // Moorhead has only its eastern land connection. Every river edge must be
    // visibly closed before opening this bank to Surf.
    for y in 0..24 {
        for x in 0..32 {
            if x == 0 || x == 31 || y == 0 || y == 23 {
                ensure!(
                    crate::authored_town_geometry::kind(pack, &map, [x, y])? != "WATER",
                    "river reaches an unconnected map edge"
                );
            }
        }
    }
    authoring::text(
        &mut map,
        "MnMoorheadRiverAccessText",
        &[
            "RED RIVER ACCESS",
            "Cast from the bank.",
            "SURF with a",
            "trusted partner.",
        ],
    );
    map.scripts.insert(
        "MnMoorheadRiverAccess".into(),
        serde_json::json!([
            c("opentext", &[]),
            c("writetext", &["MnMoorheadRiverAccessText"]),
            c("waitbutton", &[]),
            c("closetext", &[]),
            c("end", &[]),
        ]),
    );
    map.events.bg_events.push(BackgroundEvent {
        x: access.sign[0],
        y: access.sign[1],
        event_type: "BGEVENT_READ".into(),
        script: "MnMoorheadRiverAccess".into(),
    });
    crate::northstar_rival_arc::publish(pack, map)
}
