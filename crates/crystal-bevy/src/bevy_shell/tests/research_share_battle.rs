//! Earn the held reward's effect through ordinary walking and a wild victory.
#[path = "minnesota_fishing_walk.rs"]
mod grass_walk;

use crystal_core::input::GameButton;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::authored_route_fixture::indoor_walk;
use super::super::interior_life::collision;
use super::super::rochester_earned::train_wild;
use super::super::winona::{press, travel};
use super::VisibleShellController;

pub(super) fn verify_full_bench_award(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (4, 10));
    travel(c, GameButton::Down, "MnWinona", (18, 20));
    indoor_walk(c, r, (4, 18));
    travel(c, GameButton::Left, "MnR01C00R00", (63, 24));
    let before = c.snapshot().expect("unfought research partners").party;
    let held = &before.slots[1].pokemon;
    let control = &before.slots[2].pokemon;
    assert_eq!(held.species, control.species);
    assert_eq!(held.experience, control.experience);
    assert_eq!(held.item.as_deref(), Some("EXP_SHARE"));
    assert!(control.item.is_none());
    let map = &r.data().maps["MnR01C00R00"];
    let mut choices = Vec::new();
    for y in 0..i16::try_from(map.attributes.height * 2).expect("route height") {
        for x in 0..i16::try_from(map.attributes.width * 2).expect("route width") {
            if collision(r, map, (x, y)) == Some("TALL_GRASS") {
                choices.push(((63 - x).abs() + (24 - y).abs(), (x, y)));
            }
        }
    }
    choices.sort_by_key(|entry| entry.0);
    let target = choices
        .into_iter()
        .find_map(|(_, at)| grass_walk::path(c, r, at).map(|_| at))
        .expect("reachable native grass for the research partners");
    let mut won = false;
    let mut destination = target;
    let mut patrol_neighbor = None;
    for _ in 0..1024 {
        let snapshot = c.snapshot().expect("native research encounter approach");
        if let Some(battle) = snapshot.battle {
            assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
            assert_eq!(snapshot.party.active_battle_slot, Some(0));
            assert_eq!(snapshot.party.slots[0].pokemon.species.id, "CINDORE");
            assert!(train_wild(c, r), "actual wild victory");
            won = true;
            break;
        }
        let path = grass_walk::path(c, r, destination).expect("live grass approach");
        if let Some(button) = path.first() {
            press(c, *button);
        } else {
            // A changed direction can consume an input by turning in place.
            // Keep walking toward each endpoint until its position is reached.
            if destination == target {
                let neighbor = if let Some(neighbor) = patrol_neighbor {
                    neighbor
                } else {
                    let neighbor = [
                        (target.0 + 1, target.1),
                        (target.0 - 1, target.1),
                        (target.0, target.1 + 1),
                        (target.0, target.1 - 1),
                    ]
                    .into_iter()
                    .find(|&point| {
                        grass_walk::path(c, r, point).is_some_and(|path| path.len() == 1)
                    })
                    .expect("an adjacent native grass patrol endpoint");
                    patrol_neighbor = Some(neighbor);
                    neighbor
                };
                destination = neighbor;
            } else {
                destination = target;
            }
        }
    }
    let reached = c.snapshot().expect("research approach outcome");
    assert!(
        won,
        "ordinary grass steps must produce an actual encounter; at {:?}, text {:?}, repel {}",
        reached.overworld,
        reached.ui.text.as_ref().map(|text| &text.label),
        reached.progression.repel_steps_remaining,
    );
    let after = c.snapshot().expect("held and unheld native awards").party;
    let held_gain = after.slots[1].pokemon.experience - held.experience;
    let control_gain = after.slots[2].pokemon.experience - control.experience;
    assert!(control_gain > 0, "automatic bench sharing remains useful");
    assert!(
        (control_gain * 2..=control_gain * 2 + 2).contains(&held_gain),
        "held research reward must double the passive award: {held_gain} vs {control_gain}"
    );
    assert_eq!(after.slots[1].pokemon.item.as_deref(), Some("EXP_SHARE"));
}
