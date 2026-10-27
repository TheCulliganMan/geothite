//! Shore-specific promises beyond ordinary city door and boundary checks.
use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::authored_town_design::Town;
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(
    pack: &CompiledGamePack,
    town: &Town,
    map: &MapModule,
    raw: &Value,
) -> Result<()> {
    ensure!(
        town.interiors[0].id == "MnLeechLakeCareCabin"
            && town.interiors[0].native_constant == "MN_LEECH_LAKE_CARE_CABIN"
            && town.interiors[1].id == "MnLeechLakeFisherHouse"
            && town.interiors[1].native_constant == "MN_LEECH_LAKE_FISHER_HOUSE",
        "complete shore care and fisher rooms required"
    );
    ensure!(
        raw["pier_contract"]
            == json!({"walk_rows_core":[21,22],"walk_x_core":[28,40],"closed_end_core_x":41,"top_rail_core_y":20,"bottom_rail_core_y":23})
            && raw["launch_contract"]
                == json!({"approach_core":[31,29],"water_target_core":[32,29]}),
        "fixed closed pier and separate launch required"
    );
    for x in 28..41 {
        for y in [21, 22] {
            ensure!(
                kind(pack, map, [x, y])? == "FLOOR",
                "native pier walking lane interrupted"
            );
        }
        for y in [20, 23] {
            ensure!(kind(pack, map, [x, y])? == "WALL", "native pier rail gap");
        }
    }
    for y in [21, 22] {
        ensure!(
            kind(pack, map, [41, y])? == "WALL",
            "native pier end must close"
        );
    }
    ensure!(
        kind(pack, map, [31, 29])? == "FLOOR" && kind(pack, map, [32, 29])? == "WATER",
        "separate native Surf launch required"
    );
    for actor in town
        .residents
        .iter()
        .chain(town.interiors.iter().flat_map(|r| &r.residents))
    {
        ensure!(
            actor
                .lines
                .iter()
                .all(|s| !s.is_empty() && s.is_ascii() && s.len() <= 18),
            "native shore dialogue exceeds two-line page width"
        );
    }
    for sign in &town.signs {
        ensure!(
            sign.lines
                .iter()
                .all(|s| !s.is_empty() && s.is_ascii() && s.len() <= 18),
            "native shore sign exceeds page width"
        );
    }
    Ok(())
}
