//! Public fishing decks on Lake Bemidji and Detroit Lake.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use super::{Deck, OldBlock, Placement, install_deck, place};
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, authored_town_publication};

const PIERS: &str =
    include_str!("../../../planning/minnesota/revisions/outdoor-contours-v01/city-piers.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CityPier {
    map: String,
    tileset: String,
    metatiles: Vec<Deck>,
    placements: Vec<Placement>,
    shore: Vec<Shore>,
    actor: Angler,
    deck: Footprint,
    open_end: OpenEnd,
}

#[derive(Deserialize)]
struct Shore {
    #[serde(flatten)]
    old: OldBlock,
    to: u16,
}

#[derive(Deserialize)]
struct Angler {
    script: String,
    at: [u16; 2],
}

#[derive(Deserialize)]
struct Footprint {
    left: u16,
    top: u16,
    right: u16,
    bottom: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum OpenEnd {
    East,
    West,
}

pub(super) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for plan in serde_json::from_str::<Vec<CityPier>>(PIERS)? {
        let mut map = pack
            .data
            .maps
            .get(&plan.map)
            .context("city fishing pier")?
            .clone();
        install_deck(
            pack,
            &mut map,
            &plan.tileset,
            plan.metatiles,
            plan.placements,
        )?;
        for cell in plan.shore {
            place(&mut map, cell.old, cell.to)?;
        }
        let Footprint {
            left,
            top,
            right,
            bottom,
        } = plan.deck;
        ensure!(
            left > 0 && top > 0 && left <= right && top < bottom,
            "invalid deck bounds"
        );
        for x in left..=right {
            for y in top..=bottom {
                ensure!(
                    kind(pack, &map, [x, y])? == "FLOOR",
                    "gap in {} deck at {x},{y}",
                    plan.map
                );
            }
            for y in [top - 1, bottom + 1] {
                ensure!(
                    kind(pack, &map, [x, y])? == "WATER",
                    "blocked side of {} at {x},{y}",
                    plan.map
                );
            }
        }
        let (shore, tip) = match plan.open_end {
            OpenEnd::East => (left - 1, right + 1),
            OpenEnd::West => (right + 1, left - 1),
        };
        for y in top..=bottom {
            ensure!(
                kind(pack, &map, [shore, y])? == "FLOOR" && kind(pack, &map, [tip, y])? == "WATER",
                "{} pier must connect to town and open water",
                plan.map
            );
        }
        ensure!(
            map.objects
                .iter()
                .any(|o| o.script == plan.actor.script && [o.x, o.y] == plan.actor.at)
                && kind(pack, &map, plan.actor.at)? == "FLOOR",
            "{} public angler must retain dry footing",
            plan.map
        );
        authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}
