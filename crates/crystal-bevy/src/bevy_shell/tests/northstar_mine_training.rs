//! Earn the carried Ground partner's evolution in actual R09 encounters.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::authored_route_fixture::{RouteGrid, RouteSpec};
use super::super::super::rochester::flee;
use super::super::super::rochester_earned::train_wild;
use super::super::super::winona::{continue_in_place, travel};
use super::super::gym_eight_approach::care;
use super::super::northstar_arc_campaign::switch_lead;
use super::{VisibleShellController, walk_to};

const START: (i16, i16) = (0, 51);

fn lead(c: &mut VisibleShellController) -> Option<(usize, u8)> {
    c.snapshot()
        .expect("actual carried Ground coverage")
        .party
        .slots
        .iter()
        .filter(|slot| matches!(slot.pokemon.species.id.as_str(), "CACHEEK" | "FURROWBANK"))
        .max_by_key(|slot| slot.pokemon.level)
        .map(|slot| (slot.index, slot.pokemon.level))
}

fn grass(r: &CrystalRuntime, grid: &RouteGrid) -> [(i16, i16); 2] {
    let mut reachable = BTreeSet::from([START]);
    let mut queue = VecDeque::from([START]);
    while let Some((x, y)) = queue.pop_front() {
        for next in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if grid.floor.contains(&next)
                && !grid.blocked.contains(&next)
                && !grid.hazards.contains(&next)
                && reachable.insert(next)
            {
                queue.push_back(next);
            }
        }
    }
    let available = reachable
        .into_iter()
        .filter(|&(x, y)| {
            let map = &r.data().maps[&format!("MnR09C{:02}R{:02}", x / 64, y / 48)];
            let (x, y) = (x as usize % 64, y as usize % 48);
            let block = map.blocks[y / 2 * usize::from(map.attributes.width) + x / 2];
            r.data().tilesets[&map.attributes.tileset_name].collision[&format!("{block:02x}")]
                [y % 2 * 2 + x % 2]
                == "TALL_GRASS"
        })
        .collect::<BTreeSet<_>>();
    let available_ref = &available;
    available
        .iter()
        .flat_map(|&(x, y)| {
            [(x + 1, y), (x, y + 1)]
                .into_iter()
                .filter_map(move |next| available_ref.contains(&next).then_some([(x, y), next]))
        })
        .min_by_key(|pair| pair[0].0.abs_diff(START.0) + pair[0].1.abs_diff(START.1))
        .expect("reachable native R09 grass pair")
}

pub(super) fn prepare(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if lead(c).is_none_or(|(_, level)| level >= 36) {
        return;
    }
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR09",
            columns: 2,
            rows: 2,
            tileset: "mn_iron_range",
            start: START,
            doors: BTreeMap::from([((40, 41), "MnR09Cave1F"), ((40, 42), "MnR09Cave1F")]),
        },
    );
    let targets = grass(r, &grid);
    for outing in 0..48 {
        if lead(c).expect("same carried CACHEEK family").1 >= 36 {
            return;
        }
        care(c, r, "MnBemidji", (31, 10));
        let slot = lead(c).expect("same carried CACHEEK family").0;
        switch_lead(c, slot);
        walk_to(c, r, (43, 25));
        travel(c, GameButton::Right, "MnR09C00R01", (0, 3));
        let mut fought = 0;
        for step in 0..192 {
            grid.walk_with_actions(
                c,
                targets[step % 2],
                |_| {},
                |c| {
                    let s = c.snapshot().expect("visible training encounter");
                    let b = s.battle.as_ref().expect("native wild encounter");
                    let p = &s.party.slots[0].pokemon;
                    if fought < 3
                        && p.hp * 2 > p.max_hp
                        && matches!(b.enemy_pokemon.species.id.as_str(), "AGATOT" | "BANDRAKE")
                    {
                        let _ = train_wild(c, r);
                        fought += 1;
                    } else {
                        flee(c);
                    }
                },
            );
            let p = &c.snapshot().expect("earned training progress").party.slots[0].pokemon;
            if fought >= 3 || p.level >= 36 || p.hp * 2 <= p.max_hp {
                break;
            }
        }
        grid.walk_with_actions(c, START, |_| {}, flee);
        travel(c, GameButton::Left, "MnBemidji", (43, 25));
        eprintln!(
            "earned mine training outing {outing}: {:?}",
            c.snapshot()
                .expect("actual team")
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
            &format!("campaign-mine-native-training-36-{outing}"),
        );
    }
    panic!("bounded native R09 training did not reach the carried partner's evolution");
}
