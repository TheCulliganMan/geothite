//! Ordinary R06 outings prepare an actually carried partner for Lake.
#[path = "northstar_lake_switch_training.rs"]
mod switch_training;

use std::collections::BTreeMap;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::authored_route_fixture::{RouteGrid, RouteSpec};
use super::super::super::rochester::flee;
use super::super::super::rochester_earned::{train_wild, try_recruit};
use super::super::super::winona::{continue_in_place, travel};
use super::super::lake_country_story::try_road_with;
use super::{CITY, CLINIC, VisibleShellController, care, indoor_walk};

fn grass(r: &CrystalRuntime) -> [(i16, i16); 2] {
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR06",
            columns: 1,
            rows: 2,
            tileset: "mn_headwaters",
            start: (42, 0),
            doors: BTreeMap::from([((9, 61), "MnR06NeedleCabin")]),
        },
    );
    let available = grid
        .floor
        .into_iter()
        .filter(|&(x, y)| {
            let map = &r.data().maps[&format!("MnR06C00R{:02}", y / 48)];
            let y = y % 48;
            let block =
                map.blocks[usize::from(map.attributes.width) * (y as usize / 2) + x as usize / 2];
            r.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
                [y as usize % 2 * 2 + x as usize % 2]
                == "TALL_GRASS"
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut pairs = available
        .iter()
        .flat_map(|&(x, y)| {
            [(x + 1, y), (x, y + 1)]
                .into_iter()
                .filter_map(|next| available.contains(&next).then_some([(x, y), next]))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    pairs.sort_by_key(|pair| pair[0].0.abs_diff(42) + pair[0].1.unsigned_abs());
    *pairs.first().expect("reachable adjacent native R06 grass")
}

fn counter_type(first: &str, second: &str) -> bool {
    matches!(first, "GRASS" | "ELECTRIC") || matches!(second, "GRASS" | "ELECTRIC")
}

fn has_counter(c: &mut VisibleShellController) -> bool {
    c.snapshot()
        .expect("carried Lake partners")
        .party
        .slots
        .iter()
        .any(|slot| counter_type(&slot.pokemon.species.type1, &slot.pokemon.species.type2))
}

fn return_from_grass(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let map = c
        .snapshot()
        .expect("actual grass return")
        .overworld
        .map_name;
    if map.starts_with("MnR06") {
        assert!(try_road_with(c, r, 6, (42, 0), flee));
        travel(c, GameButton::Up, CITY, (32, 47));
        travel(c, GameButton::Up, CITY, (32, 46));
    } else {
        assert!(map == CLINIC || map == CITY, "native recovery in {map}");
    }
}

/// Recruit a local counter using the actual clock, carried balls and wild RNG.
/// No Pokemon, money, inventory or progress is supplied by this preparation.
fn recruit_counter(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if has_counter(c) || c.snapshot().expect("party room").party.slots.len() >= 6 {
        return;
    }
    let targets = grass(r);
    let name = format!("MnR06C00R{:02}", targets[0].1 / 48);
    let period = c.shell.shell.session().state().time.time_of_day;
    let available = r.data().wild_encounters[&name]
        .grass
        .as_ref()
        .expect("installed R06 habitat")
        .slots(period)
        .iter()
        .any(|slot| {
            let species = &r.data().pokemon[&slot.species];
            counter_type(&species.type1, &species.type2)
        });
    if !available {
        return;
    }
    for outing in 0..8 {
        if has_counter(c)
            || !c
                .shell
                .shell
                .session()
                .state()
                .bag
                .balls
                .values()
                .any(|n| *n > 0)
        {
            return;
        }
        care(c, r);
        indoor_walk(c, r, (32, 46));
        travel(c, GameButton::Down, "MnR06C00R00", (42, 0));
        for step in 0..192 {
            let arrived = try_road_with(c, r, 6, targets[step % 2], |c| {
                let snapshot = c.snapshot().expect("actual R06 encounter");
                let opponent = &snapshot
                    .battle
                    .as_ref()
                    .expect("wild recruitment encounter")
                    .enemy_pokemon
                    .species;
                if counter_type(&opponent.type1, &opponent.type2)
                    && snapshot.bag.balls.iter().any(|ball| ball.quantity > 0)
                {
                    let _ = try_recruit(c, r);
                } else {
                    flee(c);
                }
            });
            if !arrived || has_counter(c) {
                break;
            }
            if !c
                .shell
                .shell
                .session()
                .state()
                .bag
                .balls
                .values()
                .any(|n| *n > 0)
            {
                break;
            }
        }
        return_from_grass(c, r);
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-lake-counter-search-{outing}"),
        );
        if has_counter(c) {
            eprintln!(
                "earned Lake counter through actual capture: {:?}",
                c.snapshot()
                    .expect("caught party")
                    .party
                    .slots
                    .iter()
                    .map(|slot| { (&slot.pokemon.species.id, slot.pokemon.level) })
                    .collect::<Vec<_>>()
            );
            return;
        }
    }
}

pub(super) fn prepare(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    level: u8,
) {
    recruit_counter(c, r, root);
    let party = c.snapshot().expect("carried Lake team").party;
    let slot = party
        .slots
        .iter()
        .enumerate()
        .max_by_key(|(_, slot)| {
            (
                matches!(slot.pokemon.species.type1.as_str(), "GRASS" | "ELECTRIC")
                    || matches!(slot.pokemon.species.type2.as_str(), "GRASS" | "ELECTRIC"),
                slot.pokemon.level,
            )
        })
        .map(|(slot, _)| slot)
        .expect("earned campaign carries at least one partner");
    super::super::northstar_arc_campaign::switch_lead(c, slot);
    let targets = grass(r);
    for outing in 0..96 {
        if c.snapshot().expect("earned training level").party.slots[0]
            .pokemon
            .level
            >= level
        {
            return;
        }
        care(c, r);
        indoor_walk(c, r, (32, 46));
        travel(c, GameButton::Down, "MnR06C00R00", (42, 0));
        let mut fought = 0;
        for step in 0..192 {
            let arrived = try_road_with(c, r, 6, targets[step % 2], |c| {
                if fought < 3
                    && c.snapshot().expect("active training partner").party.slots[0]
                        .pokemon
                        .hp
                        > 0
                {
                    // Preserve the counter as a participant by switching to an
                    // already stronger carried partner through the battle menu.
                    let _ = train_wild(c, r);
                    fought += 1;
                } else {
                    flee(c);
                }
            });
            let p = &c.snapshot().expect("actual training outcome").party.slots[0].pokemon;
            if !arrived || fought >= 3 || p.level >= level || p.hp == 0 {
                break;
            }
        }
        return_from_grass(c, r);
        eprintln!(
            "earned Lake training outing {outing}: {:?}",
            c.snapshot()
                .expect("earned team")
                .party
                .slots
                .iter()
                .map(|slot| (&slot.pokemon.species.id, slot.pokemon.level))
                .collect::<Vec<_>>()
        );
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-lake-native-training-{level}-{outing}"),
        );
    }
    panic!("bounded ordinary Lake training did not reach level {level}");
}
