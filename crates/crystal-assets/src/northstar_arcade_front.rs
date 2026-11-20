//! A three-bay brick front and a lighted reel sign for the Minneapolis Game Corner.

use anyhow::{Context, Result, ensure};

use crate::authored_town_geometry::kind;
use crate::grand_marais_modpack::authoring::refresh_event_section;
use crate::{CompiledGamePack, authored_town_publication, northstar_trail_caches};

const MAP: &str = "MnMinneapolis";
const TILESET: &str = "mn_minneapolis_city";
const MARQUEE: &[u8] =
    include_bytes!("../../../planning/minnesota/landmarks/game-corner/reel-marquee.json");

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let mut city = pack.data.maps.get(MAP).context("Minneapolis")?.clone();
    ensure!(
        city.attributes.tileset_name == TILESET
            && (city.attributes.width, city.attributes.height) == (40, 32),
        "unexpected Minneapolis street plan"
    );
    let door = [56, 51];
    ensure!(
        city.events.warps.iter().any(|warp| {
            [warp.x, warp.y] == door && warp.target_map_constant == "MN_MINNEAPOLIS_CORNER_HOME"
        }) && kind(pack, &city, door)? == "DOOR",
        "missing Minneapolis Game Corner entrance"
    );
    for x in [54, 55, 56] {
        ensure!(
            kind(pack, &city, [x, 52])? == "FLOOR",
            "obstructed arcade approach at ({x},52)"
        );
    }

    // Extend west into one solid pine strip. The east pines separate the
    // arcade from Cedar Home's freestanding sign and street entrance.
    for (y, expected, frontage) in [
        (24, [0x05, 0x80, 0x81], [0x80, 0x82, 0x81]),
        (25, [0x05, 0xb8, 0x9e], [0x83, 0xb9, 0x9e]),
    ] {
        let start = y * 40 + 26;
        let cells = city
            .blocks
            .get_mut(start..start + 3)
            .context("three arcade bays")?;
        ensure!(cells == expected, "unexpected Minneapolis frontage at {y}");
        cells.copy_from_slice(&frontage);
    }
    let sign = city
        .events
        .bg_events
        .iter()
        .find(|event| [event.x, event.y] == [55, 51])
        .context("Game Corner wall plaque")?;
    ensure!(
        sign.event_type == "BGEVENT_READ"
            && sign.script == "MnCornerSign"
            && !city.events.bg_events.iter().any(|e| [e.x, e.y] == [54, 51]),
        "unexpected game corner sign or left reading square"
    );
    // Both halves of the wider sign can be read from the pavement.
    let mut left_half = sign.clone();
    left_half.x = 54;
    city.events.bg_events.push(left_half);
    refresh_event_section(&mut city);

    let graphics = pack
        .runtime_files
        .get("gfx/tilesets/mn_minneapolis_city.2bpp")
        .context("Minneapolis tile atlas")?;
    ensure!(
        graphics.len() == 256 * 16 && graphics[0xfb * 16..0xff * 16].iter().all(|&b| b == 0),
        "expected four blank native arcade glyphs"
    );
    northstar_trail_caches::install_landmark_art_at(pack, MARQUEE, 0xfb)?;
    authored_town_publication::replace(pack, city)
}
