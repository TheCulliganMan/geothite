//! Open Union Depot's waiting garden and finish the records hall's north wall.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};

use crate::authored_dialogue;
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, authored_town_publication};

const DEPOT: &str = "MnStPaulUnionDepot";
const CLERK: &str = "MN_ST_PAUL_DEPOT_CLERKText";
const HALL: &str = "MnStPaulCapitolHall";

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut depot = pack.data.maps.get(DEPOT).context(DEPOT)?.clone();
    ensure!(
        depot.attributes.tileset_name == "train_station"
            && (depot.attributes.width, depot.attributes.height) == (16, 12)
            && depot.blocks.get(6 * 16 + 11..6 * 16 + 13) == Some(&[0x21, 0x19])
            && depot
                .objects
                .iter()
                .any(|o| o.script == "MN_ARC_SP_REED" && (o.x, o.y) == (22, 17))
            && depot
                .events
                .bg_events
                .iter()
                .any(|e| e.script == "MN_GLACIER_LINE_TIMETABLE" && (e.x, e.y) == (21, 15)),
        "expected the narrow garden gate and the concourse timetable"
    );
    // Replace the two planters above the timetable with the adjacent lawn.
    // Reed can stand at the south gate without blocking the visitor's way in.
    depot.blocks[6 * 16 + 11] = 0x19;
    authored_dialogue::install_canonical(&mut depot, CLERK)?;
    let pages = BTreeMap::from([(CLERK.to_owned(), depot.scripts[CLERK].clone())]);
    depot
        .script_text_bodies
        .extend(crate::parse_script_text_bodies(DEPOT, &pages)?);
    authored_town_publication::replace(pack, depot)?;

    let mut hall = pack.data.maps.get(HALL).context(HALL)?.clone();
    ensure!(
        hall.attributes.tileset_name == "facility"
            && (hall.attributes.width, hall.attributes.height) == (8, 4)
            && hall.blocks.get(5..8) == Some(&[0x01, 0x1d, 0x01])
            && kind(pack, &hall, [13, 0])? == "STAIRCASE"
            && hall.events.warps.iter().all(|w| (w.x, w.y) != (13, 0)),
        "expected an unconnected staircase between two stretches of north wall"
    );
    hall.blocks[6] = 0x01;
    authored_town_publication::replace(pack, hall)
}
