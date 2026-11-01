//! Recruit and train a carried mine counter through ordinary native encounters.
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::authored_route_fixture::{RouteGrid, RouteSpec, position};
use super::super::super::native_city_ports::city_port;
use super::super::super::rochester::flee;
use super::super::super::rochester_earned::{checkout, train_wild, try_battle, try_recruit};
use super::super::super::winona::{continue_in_place, press, travel};
use super::super::gym_eight_approach::care;
use super::super::northstar_arc_campaign::switch_lead;
use super::{VisibleShellController, event, walk_to};

const START: (i16, i16) = (0, 51);

fn lead(c: &mut VisibleShellController) -> Option<(usize, u8)> {
    c.snapshot()
        .expect("actual carried mine counter coverage")
        .party
        .slots
        .iter()
        .filter(|slot| counter_type(&slot.pokemon.species.type1, &slot.pokemon.species.type2))
        .max_by_key(|slot| slot.pokemon.level)
        .map(|slot| (slot.index, slot.pokemon.level))
}

fn counter_type(first: &str, second: &str) -> bool {
    matches!(first, "GROUND" | "WATER") || matches!(second, "GROUND" | "WATER")
}

fn grass(r: &CrystalRuntime, grid: &RouteGrid, prefix: &str, start: (i16, i16)) -> [(i16, i16); 2] {
    let mut reachable = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
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
            let map = &r.data().maps[&format!("{prefix}C{:02}R{:02}", x / 64, y / 48)];
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
        .min_by_key(|pair| pair[0].0.abs_diff(start.0) + pair[0].1.abs_diff(start.1))
        .expect("reachable native R09 grass pair")
}

/// Return physically from an actual saved R08 recruitment/income checkpoint.
pub(super) fn return_from_recruitment(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let connection = r.data().maps["MnR08C00R00"]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction.eq_ignore_ascii_case("north") && link.target_map == "MnBemidji")
        .expect("published Bemidji walking connection");
    let destination = (
        24 + i16::try_from(connection.offset * 2).expect("native R08 offset"),
        0,
    );
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR08",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start: position(c),
            doors: BTreeMap::new(),
        },
    );
    grid.walk_with_actions(c, destination, |_| {}, flee);
    travel(c, GameButton::Up, "MnBemidji", (24, 47));
    travel(c, GameButton::Up, "MnBemidji", (24, 46));
}

/// Earn ordinary route prizes when carried balls are gone, then buy replacements.
fn restock(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &AssetRoot,
    grid: &RouteGrid,
    start: (i16, i16),
) {
    let shop = r.data().maps["MnBemidjiMart"]
        .script_shop_commands
        .first()
        .expect("installed native Mart binding");
    let inventory = r
        .data()
        .marts
        .inventory_ids(&shop.mart_id)
        .expect("installed native Mart stock");
    let ball_id = ["POKE_BALL", "GREAT_BALL", "ULTRA_BALL"]
        .into_iter()
        .filter(|id| inventory.iter().any(|stocked| stocked == id))
        .min_by_key(|id| r.data().items[*id].price)
        .expect("native Mart sells an ordinary capture ball");
    let ball = &r.data().items[ball_id];
    if c.shell
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
    for (id, flag) in [
        ("MN_R08_T07", "EVENT_MN_R08_TRAINER_07"),
        ("MN_R08_T05", "EVENT_MN_R08_TRAINER_05"),
        ("MN_R08_T02", "EVENT_MN_R08_TRAINER_02"),
        ("MN_R08_T06", "EVENT_MN_R08_TRAINER_06"),
        ("MN_R08_T01", "EVENT_MN_R08_TRAINER_01"),
        ("MN_R08_T08", "EVENT_MN_R08_TRAINER_08"),
        ("MN_R08_T03", "EVENT_MN_R08_TRAINER_03"),
        ("MN_R08_T04", "EVENT_MN_R08_TRAINER_04"),
    ] {
        if c.shell.shell.session().state().money >= u32::from(ball.price) * 3 {
            break;
        }
        if event(c, flag) {
            continue;
        }
        let mut actor = None;
        for row in 0..2 {
            for column in 0..2 {
                let map = &r.data().maps[&format!("MnR08C{column:02}R{row:02}")];
                if let Some(object) = map
                    .objects
                    .iter()
                    .find(|object| object.script.starts_with(id))
                {
                    actor = Some((
                        column * 64 + i16::try_from(object.x).expect("native actor x"),
                        row * 48 + i16::try_from(object.y).expect("native actor y"),
                    ));
                }
            }
        }
        let (x, y) = actor.expect("installed R08 trainer actor");
        // A trainer's front tile is deliberately excluded as a sight hazard.
        // Approach a reachable side/back tile, then use ordinary A interaction.
        let Some((tile, facing)) = [
            ((x - 1, y), GameButton::Right),
            ((x + 1, y), GameButton::Left),
            ((x, y - 1), GameButton::Down),
            ((x, y + 1), GameButton::Up),
        ]
        .into_iter()
        .filter(|(tile, _)| {
            grid.floor.contains(tile)
                && !grid.blocked.contains(tile)
                && !grid.hazards.contains(tile)
        })
        .min_by_key(|(tile, _)| tile.0.abs_diff(start.0) + tile.1.abs_diff(start.1)) else {
            continue;
        };
        care(c, r, "MnBemidji", (31, 10));
        if let Some(slot) = c
            .snapshot()
            .expect("carried route team")
            .party
            .slots
            .iter()
            .filter(|slot| {
                slot.pokemon.species.type1 == "FIRE" || slot.pokemon.species.type2 == "FIRE"
            })
            .max_by_key(|slot| slot.pokemon.level)
            .map(|slot| slot.index)
        {
            switch_lead(c, slot);
        }
        walk_to(c, r, (24, 46));
        travel(c, GameButton::Down, "MnR08C00R00", start);
        grid.walk_with_actions(
            c,
            tile,
            |_| {},
            |c| {
                let snapshot = c.snapshot().expect("native route encounter");
                if matches!(
                    snapshot.battle.expect("encounter").kind,
                    crystal_runtime::RuntimeBattleKind::Wild { .. }
                        | crystal_runtime::RuntimeBattleKind::StaticWild { .. }
                ) {
                    flee(c);
                } else {
                    assert!(
                        try_battle(c, r, id, flag),
                        "actual route prize requires victory"
                    );
                }
            },
        );
        if !event(c, flag) {
            press(c, facing);
            press(c, GameButton::A);
            assert!(
                try_battle(c, r, id, flag),
                "actual route prize requires victory"
            );
        }
        continue_in_place(c, root, r, &format!("campaign-mine-income-{id}"));
        grid.walk_with_actions(c, start, |_| {}, flee);
        travel(c, GameButton::Up, "MnBemidji", (24, 47));
        travel(c, GameButton::Up, "MnBemidji", (24, 46));
    }
    let state = c.shell.shell.session().state();
    let quantity = (state.money / u32::from(ball.price)).min(3) as u16;
    assert!(
        quantity > 0,
        "recruitment needs earned money for real balls"
    );
    let (door, arrival) = city_port(r, "MnBemidji", "MnBemidjiMart");
    walk_to(
        c,
        r,
        (
            u16::try_from(door.0).expect("native Mart x"),
            u16::try_from(door.1).expect("native Mart y"),
        ),
    );
    travel(c, GameButton::Up, "MnBemidjiMart", arrival);
    walk_to(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, ball_id, quantity);
    walk_to(c, r, (6, 6));
    travel(c, GameButton::Down, "MnBemidji", door);
    continue_in_place(c, root, r, "campaign-mine-balls-purchased");
}

/// Backtrack to R08 for a current-time Water/Ground partner using carried balls.
/// Every capture, return, heal and checkpoint uses the production controller.
fn recruit(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    if lead(c).is_some() || c.snapshot().expect("actual party room").party.slots.len() >= 6 {
        return;
    }
    let connection = r.data().maps["MnR08C00R00"]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction.eq_ignore_ascii_case("north") && link.target_map == "MnBemidji")
        .expect("published R08 walking connection");
    let start = (
        24 + i16::try_from(connection.offset * 2).expect("native R08 offset"),
        0,
    );
    let grid = RouteGrid::new_with_grass(
        r,
        RouteSpec {
            prefix: "MnR08",
            columns: 2,
            rows: 2,
            tileset: "mn_headwaters",
            start,
            doors: BTreeMap::new(),
        },
    );
    let targets = grass(r, &grid, "MnR08", start);
    let name = format!("MnR08C{:02}R{:02}", targets[0].0 / 64, targets[0].1 / 48);
    let period = c.shell.shell.session().state().time.time_of_day;
    if !r.data().wild_encounters[&name]
        .grass
        .as_ref()
        .expect("installed R08 grass")
        .slots(period)
        .iter()
        .any(|slot| {
            let species = &r.data().pokemon[&slot.species];
            counter_type(&species.type1, &species.type2)
        })
    {
        eprintln!("No counter in the current-time R08 grass; clock unchanged");
        return;
    }
    for outing in 0..8 {
        if lead(c).is_some() {
            return;
        }
        restock(c, r, root, &grid, start);
        care(c, r, "MnBemidji", (31, 10));
        // The visible night targets use Water and Psychic attacks. Avoid the
        // carried Fire and Poison partners while trying to weaken one.
        if let Some(slot) = c
            .snapshot()
            .expect("carried capture lead")
            .party
            .slots
            .iter()
            .filter(|slot| {
                ![
                    slot.pokemon.species.type1.as_str(),
                    slot.pokemon.species.type2.as_str(),
                ]
                .iter()
                .any(|kind| matches!(*kind, "FIRE" | "POISON"))
            })
            .max_by_key(|slot| slot.pokemon.level)
            .map(|slot| slot.index)
        {
            switch_lead(c, slot);
        }
        walk_to(c, r, (24, 46));
        travel(c, GameButton::Down, "MnR08C00R00", start);
        for step in 0..192 {
            grid.walk_with_actions(
                c,
                targets[step % 2],
                |_| {},
                |c| {
                    let snapshot = c.snapshot().expect("actual mine recruitment encounter");
                    let enemy = &snapshot
                        .battle
                        .as_ref()
                        .expect("wild encounter")
                        .enemy_pokemon
                        .species;
                    if counter_type(&enemy.type1, &enemy.type2)
                        && snapshot.bag.balls.iter().any(|ball| ball.quantity > 0)
                    {
                        let _ = try_recruit(c, r);
                    } else {
                        flee(c);
                    }
                },
            );
            let snapshot = c.snapshot().expect("actual capture outing health");
            let active = &snapshot.party.slots[0].pokemon;
            if lead(c).is_some()
                || active.hp * 2 <= active.max_hp
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
                break;
            }
        }
        grid.walk_with_actions(c, start, |_| {}, flee);
        travel(c, GameButton::Up, "MnBemidji", (24, 47));
        travel(c, GameButton::Up, "MnBemidji", (24, 46));
        continue_in_place(
            c,
            root,
            r,
            &format!("campaign-mine-counter-search-{outing}"),
        );
    }
}

pub(super) fn prepare(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot) {
    recruit(c, r, root);
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
    let targets = grass(r, &grid, "MnR09", START);
    for outing in 0..48 {
        if lead(c).expect("same carried mine counter partner").1 >= 36 {
            return;
        }
        care(c, r, "MnBemidji", (31, 10));
        let slot = lead(c).expect("same carried mine counter partner").0;
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
    panic!(
        "bounded native R09 training did not reach the carried mine counter partner's training level"
    );
}
