//! Bounded ordinary wild outings after the earned R05 trainers; no state writes.
use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_runtime::CrystalRuntime;

use super::super::super::authored_route_fixture::{RouteGrid, RouteSpec};
use super::super::super::rochester::flee;
use super::super::super::rochester_earned::{try_battle, try_recruit};
use super::super::super::winona::continue_in_place;
use super::super::lake_country_story::try_road_with;
use super::{VisibleShellController, care, enter_road, return_to_city, supplies};

fn grass(r: &CrystalRuntime) -> [(i16, i16); 2] {
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR05",
            columns: 1,
            rows: 3,
            tileset: "mn_headwaters",
            start: (24, 0),
            doors: BTreeMap::from([((16, 77), "MnR05RockCut"), ((30, 27), "MnR05RockCut")]),
        },
    );
    let mut available = grid
        .floor
        .into_iter()
        .filter(|&(x, y)| {
            let name = format!("MnR05C00R{:02}", y / 48);
            let map = &r.data().maps[&name];
            let y = y % 48;
            let block =
                map.blocks[usize::from(map.attributes.width) * (y as usize / 2) + x as usize / 2];
            r.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
                [y as usize % 2 * 2 + x as usize % 2]
                == "TALL_GRASS"
        })
        .collect::<Vec<_>>();
    available.sort_by_key(|&(x, y)| (x.abs_diff(24) + y.unsigned_abs(), x, y));
    [available[0], available[1]]
}

/// Search the actual time-dependent table and catch a local Electric/Psychic
/// partner. Every ball, encounter, failed throw and recovery is authoritative.
pub(super) fn recruit(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    const PARTNERS: &[&str] = &[
        "MAREEP",
        "FLAAFFY",
        "AMPHAROS",
        "DROWZEE",
        "HYPNO",
        "GIRAFARIG",
        "SLOWPOKE",
        "SLOWBRO",
        "SLOWKING",
    ];
    let recruited = |c: &mut VisibleShellController| {
        c.snapshot()
            .expect("earned local partner")
            .party
            .slots
            .iter()
            .any(|slot| PARTNERS.contains(&slot.pokemon.species.id.as_str()))
    };
    let targets = grass(r);
    for outing in 0..16 {
        if recruited(c) {
            return;
        }
        care(c, r);
        supplies::balls(c, r);
        enter_road(c, r);
        let mut attempted = false;
        for step in 0..64 {
            let arrived = try_road_with(c, r, 5, targets[step % 2], |c| {
                let wanted = c
                    .snapshot()
                    .expect("visible wild candidate")
                    .battle
                    .as_ref()
                    .is_some_and(|b| PARTNERS.contains(&b.enemy_pokemon.species.id.as_str()));
                if wanted && !attempted {
                    let _ = try_recruit(c, r);
                    attempted = true;
                } else {
                    flee(c);
                }
            });
            if attempted || !arrived {
                break;
            }
        }
        return_to_city(c, r);
        continue_in_place(c, root, r, &format!("campaign-wind-recruit-{outing}"));
    }
    assert!(
        recruited(c),
        "bounded ordinary recruitment must actually catch a partner"
    );
}

pub(super) fn train(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    level: u8,
) {
    let targets = grass(r);
    for outing in 0..48 {
        if c.snapshot().expect("earned level").party.slots[0]
            .pokemon
            .level
            >= level
        {
            return;
        }
        care(c, r);
        enter_road(c, r);
        let mut fought = false;
        for step in 0..64 {
            let arrived = try_road_with(c, r, 5, targets[step % 2], |c| {
                if fought {
                    flee(c);
                } else {
                    let _ = try_battle(c, r, "WILD", "");
                    fought = true;
                }
            });
            if fought || !arrived {
                break;
            }
        }
        return_to_city(c, r);
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-wind-training-{level}-{outing}"),
        );
    }
    assert!(
        c.snapshot().expect("bounded native training").party.slots[0]
            .pokemon
            .level
            >= level
    );
}
