//! Place Brainerd's lake ranger at the landing and point the Route 7 birder toward the stairs.
use anyhow::{Context, Result, ensure};
use serde_json::json;

use crate::CompiledGamePack;
use crate::authored_dialogue::install_canonical;
use crate::authored_town_geometry::kind;
use crate::authored_town_publication::replace;
use crate::grand_marais_modpack::authoring::refresh_event_section;

const RANGER: &str = "MN_BRAINERD_RANGER";
const BIRDER: &str = "MN_R07_BIRDERText";

const SAPLING: [u8; 16] = [5, 5, 5, 5, 5, 5, 5, 5, 0xc0, 0x1f, 5, 5, 0xc1, 0xc2, 5, 5];

pub(crate) fn apply(pack: &mut CompiledGamePack, timber: u16) -> Result<()> {
    restore_lake_piers(pack, timber)?;
    let mut town = pack
        .data
        .maps
        .get("MnBrainerd")
        .context("Brainerd")?
        .clone();
    for (at, surface) in [
        ([49, 20], "FLOOR"),
        ([49, 21], "FLOOR"),
        ([49, 19], "WATER"),
        ([47, 20], "FLOOR"),
        ([48, 20], "FLOOR"),
    ] {
        ensure!(
            kind(pack, &town, at)? == surface,
            "Brainerd pier changed at {at:?}"
        );
    }
    ensure!(
        town.objects.iter().all(|o| [o.x, o.y] != [49, 20]),
        "Brainerd ranger's shore position is occupied"
    );
    let ranger = town
        .objects
        .iter_mut()
        .find(|o| o.script == RANGER)
        .context("Brainerd ranger")?;
    ensure!(
        ranger.object_identifier.as_deref() == Some(RANGER) && [ranger.x, ranger.y] == [25, 19],
        "unexpected Brainerd ranger identity or position"
    );
    (ranger.x, ranger.y) = (49, 20);
    refresh_event_section(&mut town);
    let label = town
        .attributes
        .map_events_label
        .as_ref()
        .context("Brainerd events")?;
    if let Some(raw) = town.scripts.get_mut(label) {
        *raw = town
            .map_event_section_commands
            .iter()
            .map(|row| json!({"command":row.command,"args":row.args}))
            .collect();
    }
    replace(pack, town)?;

    let mut route = pack
        .data
        .maps
        .get("MnR07C00R00")
        .context("Route 7 sandbank")?
        .clone();
    let old = route.scripts.get(BIRDER).context("Route 7 birder")?.clone();
    install_canonical(&mut route, BIRDER)?;
    ensure!(
        route.scripts[BIRDER] != old,
        "Route 7 birder still describes the old ledge"
    );
    route.script_text_bodies = crate::parse_script_text_bodies(&route.id, &route.scripts)?;
    replace(pack, route)
}

fn restore_lake_piers(pack: &mut CompiledGamePack, donor: u16) -> Result<()> {
    const BANK: &str = "mn_headwaters";
    let file = format!("data/tilesets/{BANK}_metatiles.bin");
    let layout = pack
        .runtime_files
        .get(&file)
        .context("Headwaters metatiles")?;
    let collision = &pack
        .data
        .tilesets
        .get(BANK)
        .context("Headwaters permissions")?
        .collision;
    if donor != 0x88 {
        ensure!(
            layout.get(0x88 * 16..0x89 * 16) == Some(SAPLING.as_slice())
                && collision
                    .get("88")
                    .is_some_and(|cells| cells == &["FLOOR", "FLOOR", "HEADBUTT_TREE", "FLOOR"]),
            "unrecognized former Headwaters pier tile"
        );
    }
    for (name, columns, y) in [("MnLeechLake", 17..=20, 10), ("MnMilleLacs", 10..=14, 11)] {
        let mut room = pack
            .data
            .maps
            .get(name)
            .with_context(|| format!("{name} pier"))?
            .clone();
        ensure!(
            room.attributes.tileset_name == BANK,
            "{name} pier bank changed"
        );
        for x in columns {
            let index = usize::from(room.attributes.width) * y + x;
            let block = room
                .blocks
                .get_mut(index)
                .context("pier outside lake map")?;
            ensure!(*block == 0x88, "unexpected {name} pier tile at {x},{y}");
            *block = donor;
        }
        if donor != 0x88 {
            replace(pack, room)?;
        }
    }
    Ok(())
}
