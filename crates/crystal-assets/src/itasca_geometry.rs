//! Real dry plaque, necessary crossing and visibly closed park boundaries.
use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::authored_town_geometry::{kind, reachable};
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(pack: &CompiledGamePack, map: &MapModule, source: &Value) -> Result<()> {
    ensure!(
        source["marker_contract"]
            == json!({"tile":[19,21],"approach":[19,22],"milestone":"MS06","requires":["BADGE_7"],"sets":["EVENT_MN_HEADWATERS_READING"]}),
        "canonical dry marker anchor/prerequisite required"
    );
    ensure!(
        source["bridge_contract"]
            == json!({"walk_rows_core":[25,26],"walk_x_core":[20,25],"top_rail_core_y":24,"bottom_rail_core_y":27,"west_target_core":[19,22],"east_arrival_core":[42,24]}),
        "fixed short crossing required"
    );
    ensure!(
        kind(pack, map, [19, 21])? == "WALL" && kind(pack, map, [19, 22])? == "FLOOR",
        "marker must be a real dry-land plaque"
    );
    for x in 20..=25 {
        for y in [25, 26] {
            ensure!(
                kind(pack, map, [x, y])? == "FLOOR",
                "native bridge walking lane interrupted"
            );
        }
        for y in [24, 27] {
            ensure!(kind(pack, map, [x, y])? == "WALL", "native bridge rail gap");
        }
    }
    ensure!(
        reachable(pack, map, [42, 24], false)?.contains(&[19, 22]),
        "marker must be accessible without field moves"
    );
    let mut closed = map.clone();
    for y in 12..=13 {
        for x in 10..=12 {
            closed.blocks[y * 24 + x] = 0x35;
        }
    }
    ensure!(
        !reachable(pack, &closed, [42, 24], false)?.contains(&[19, 22]),
        "crossing has a free dry bypass"
    );
    for y in 0..48 {
        for x in 0..48 {
            ensure!(
                !matches!(kind(pack, map, [x, y])?, "DOOR" | "CUT_TREE"),
                "no false door or incidental Cut gate in public park"
            );
        }
    }
    Ok(())
}
