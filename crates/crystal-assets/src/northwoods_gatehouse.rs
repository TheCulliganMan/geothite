//! An original Crystal gate separates the roomy trail from connected world cells.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::grand_marais_modpack::authoring::{self, command as c};
use crate::{CompiledGamePack, MapModule};

pub(super) const MAP: &str = "MnTallTaleForestGate";
pub(super) const CONSTANT: &str = "MN_TALL_TALE_FOREST_GATE";
const OUTSIDE: &str = "MnBemidjiNorthwoods";
const TRAIL: &str = "MnTallTaleTrail";
const TRAIL_CONSTANT: &str = "MN_TALL_TALE_TRAIL";

pub(super) fn bind(
    pack: &CompiledGamePack,
    outside: &mut MapModule,
    trail: &mut MapModule,
    source: &Value,
) -> Result<MapModule> {
    ensure!(
        source
            == &json!({"map": MAP, "constant": CONSTANT,
        "template":"IlexForestAzaleaGate", "outside_door":[13,11], "outside_arrival":[13,12],
        "trail_door":[3,33], "trail_arrival":[3,34], "gate_left_arrival":[1,4], "gate_right_arrival":[8,4]}),
        "reviewed original forest gate contract"
    );
    let original = &pack.data.maps["Route35"];
    ensure!(
        original.attributes.tileset_name == "johto"
            && original.blocks[11..13] == [0x08, 0x09]
            && original.blocks[21..23] == [0x1a, 0x11],
        "original National Park gate facade"
    );
    for (map, blocks) in [
        (
            &mut *outside,
            vec![
                (6, 4, 0x08),
                (7, 4, 0x09),
                (6, 5, 0x1a),
                (7, 5, 0x11),
                (6, 6, 0x01),
            ],
        ),
        (
            &mut *trail,
            vec![
                (1, 15, 0x08),
                (2, 15, 0x09),
                (1, 16, 0x1a),
                (2, 16, 0x11),
                (1, 17, 0x01),
            ],
        ),
    ] {
        for (x, y, after) in blocks {
            let index = y * usize::from(map.attributes.width) + x;
            ensure!(
                matches!(map.blocks[index], 0x02 | 0x05),
                "reviewed gate facade preimage"
            );
            for tile in [
                [x * 2, y * 2],
                [x * 2 + 1, y * 2],
                [x * 2, y * 2 + 1],
                [x * 2 + 1, y * 2 + 1],
            ] {
                ensure!(
                    !map.objects
                        .iter()
                        .any(|o| [usize::from(o.x), usize::from(o.y)] == tile)
                        && !map
                            .events
                            .bg_events
                            .iter()
                            .any(|e| [usize::from(e.x), usize::from(e.y)] == tile),
                    "occupied original gate footprint"
                );
            }
            map.blocks[index] = after;
        }
    }
    for (map, door, arrival) in [(&*outside, [13, 11], [13, 12]), (&*trail, [3, 33], [3, 34])] {
        ensure!(
            crate::authored_town_geometry::kind(pack, map, door)? == "DOOR"
                && crate::authored_town_geometry::kind(pack, map, arrival)? == "FLOOR",
            "native gate door and safe arrival"
        );
    }
    let mut gate = authoring::blank(pack, "IlexForestAzaleaGate", MAP, CONSTANT)?;
    gate.attributes.fishing_group = Some("FISHGROUP_NONE".into());
    ensure!(
        gate.attributes.width == 5
            && gate.attributes.height == 4
            && gate.attributes.environment.as_deref() == Some("GATE"),
        "original Ilex gate room"
    );
    let outside_constant = outside
        .attributes
        .map_constant
        .as_ref()
        .context("corridor constant")?
        .clone();
    authoring::warp(outside, (13, 11), CONSTANT, 5);
    authoring::warp(outside, (13, 12), &outside_constant, 2);
    authoring::warp(trail, (3, 33), CONSTANT, 6);
    authoring::warp(trail, (3, 34), TRAIL_CONSTANT, 2);
    for y in [4, 5] {
        authoring::warp(&mut gate, (0, y), &outside_constant, 2);
        authoring::warp(&mut gate, (9, y), TRAIL_CONSTANT, 2);
    }
    // Four active lane exits precede two passive floor arrivals.
    authoring::warp(&mut gate, (1, 4), CONSTANT, 5);
    authoring::warp(&mut gate, (8, 4), CONSTANT, 6);
    gate.scripts.insert(
        "MnTaleGateAttendant".into(),
        json!([c("jumptext", &["MnTaleGateHint"])]),
    );
    authoring::text(
        &mut gate,
        "MnTaleGateHint",
        &[
            "TALL TALE TRAIL",
            "is through here.",
            "Read the ground.",
            "Then ask the crew.",
        ],
    );
    authoring::npc(
        pack,
        &mut gate,
        "MnTaleGateAttendant",
        "SPRITE_GRANNY",
        5,
        2,
    )?;
    let reach = crate::authored_town_geometry::reachable(pack, &gate, [1, 4], false)?;
    ensure!(
        [[8, 4], [1, 5], [8, 5]]
            .iter()
            .all(|tile| reach.contains(tile)),
        "both native gate lanes must connect"
    );
    ensure!(
        outside.id == OUTSIDE && trail.id == TRAIL,
        "regional gate bindings"
    );
    Ok(gate)
}
