//! The frost-split western face of Raven Rock on the Gunflint Trail.
use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, TilePosition};
use serde::Deserialize;

use crate::authored_town_geometry::kind;
use crate::authored_town_publication;
use crate::grand_marais_modpack::authoring;
use crate::northstar_cave_geometry::resting_tiles_avoiding;
use crate::northstar_maze_passages::{add_breakable_rock, check_rock_smash};
use crate::northstar_maze_tiles::{self, Placement};
use crate::{CompiledGamePack, MapModule};

const MAP: &str = "MnR12C00R00";
const ROCK: &str = "MN_R12_RAVEN_ROCK_FRACTURE";
const BASE: [u16; 2] = [35, 14];
const SUMMIT: [u16; 2] = [42, 9];
const LOOKOUT: [u16; 2] = [49, 8];
const CUT_CLEARING: [u16; 2] = [12, 12];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rockfall {
    terrain: Vec<Placement>,
    markings: Vec<[u16; 4]>,
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    let plan: Rockfall = serde_json::from_str(include_str!(
        "../../../planning/minnesota/revisions/wayfinding-v01/gunflint-rockfall.json"
    ))?;
    let mut road = pack.data.maps.get(MAP).context("Gunflint Trail")?.clone();
    ensure!(
        road.attributes.tileset_name == "mn_winter_gunflint"
            && kind(pack, &road, [35, 13])? == "WALL",
        "original intact bedrock face"
    );
    northstar_maze_tiles::install(pack, &mut road, &plan.terrain)?;
    crate::northstar_trail_caches::install_landmark_art_at(
        pack,
        include_bytes!(
            "../../../planning/minnesota/revisions/wayfinding-v01/gunflint-rockfall-art.json"
        ),
        0x80,
    )?;
    let width = usize::from(road.attributes.width);
    let collisions = &pack.data.tilesets[&road.attributes.tileset_name].collision;
    for [x, y, old, stamp] in plan.markings {
        let b = road
            .blocks
            .get_mut(usize::from(y) * width + usize::from(x))
            .context("Gunflint bedrock art position")?;
        ensure!(
            *b == old && collisions[&format!("{old:02x}")] == collisions[&format!("{stamp:02x}")],
            "geological stamping must preserve its native footing"
        );
        *b = stamp;
    }
    add_breakable_rock(pack, &mut road, ROCK, [35, 13])?;
    road.script_control_commands = crate::parse_script_control_commands(MAP, &road.scripts)?;
    authoring::refresh_event_section(&mut road);
    crate::northstar_cave_overhaul::validate_native_art(pack, &road)?;
    authored_town_publication::replace(pack, road)?;
    let road = &pack.data.maps[MAP];
    check_rock_smash(pack, road, TilePosition::new(35, 14), Direction::Up, ROCK)?;
    check_ridge(pack, road)
}

fn check_ridge(pack: &CompiledGamePack, road: &MapModule) -> Result<()> {
    let portals: Vec<_> = road.events.warps.iter().map(|w| [w.x, w.y]).collect();
    let sealed = resting_tiles_avoiding(pack, road, BASE, true, &portals)?;
    let reset = resting_tiles_avoiding(pack, road, SUMMIT, false, &portals)?;
    ensure!(
        sealed.contains(&LOOKOUT)
            && !sealed.contains(&SUMMIT)
            && !sealed.contains(&CUT_CLEARING)
            && reset.contains(&LOOKOUT)
            && reset.contains(&BASE),
        "the ice route, sealed outcrop, Cut clearing and return after regrowth"
    );
    // Treat every ice surface as unavailable for a player who elects to stay on rock and snow.
    let mut ice = portals;
    for y in 0..2 * road.attributes.height {
        for x in 0..2 * road.attributes.width {
            if kind(pack, road, [x, y])? == "ICE" {
                ice.push([x, y]);
            }
        }
    }
    let winter = resting_tiles_avoiding(pack, road, BASE, false, &ice)?;
    let mut open = road.clone();
    open.objects.retain(|o| o.script != ROCK);
    let ridge = resting_tiles_avoiding(pack, &open, BASE, false, &ice)?;
    ensure!(
        !winter.contains(&LOOKOUT)
            && ridge.contains(&SUMMIT)
            && ridge.contains(&[48, 6])
            && ridge.contains(&LOOKOUT)
            && !ridge.contains(&CUT_CLEARING),
        "rock passage must earn a dry crossing without opening the western Cut gate"
    );
    Ok(())
}
