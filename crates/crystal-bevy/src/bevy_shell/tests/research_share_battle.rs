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

#[test]
#[ignore = "requires an external Northstar pack in CRYSTAL_WINONA_TEST_PACK"]
fn northstar_exp_share_recipient_learns_from_a_wild_victory() {
    use crystal_core::models::Dv;
    use crystal_core::systems::experience::calculate_experience;

    let (root, runtime) = super::super::winona::load();
    let mut controller =
        super::super::interior_life::fixture(&root, &runtime, "MnR01C00R00", (63, 24));
    let crystal_core::systems::learnsets::LearnsetEntry(learn_level, learned_move) =
        runtime.data().learnsets["BIRCHIT"]
            .iter()
            .find(|entry| {
                entry.0 > 5
                    && entry.0 < 16
                    && !["TACKLE", "LEER", "GROWL", "QUICK_ATTACK"].contains(&entry.1.as_str())
            })
            .expect("early native Birchit move");
    let mut lead = runtime
        .data()
        .create_pokemon("CINDORE", 40, Dv::default())
        .expect("native lead");
    let mut recipient = runtime
        .data()
        .create_pokemon("BIRCHIT", *learn_level - 1, Dv::default())
        .expect("native EXP Share recipient");
    recipient.experience = calculate_experience(
        runtime.growth_rates(),
        &recipient.species.growth_rate,
        *learn_level,
    )
    .expect("next level")
        - 1;
    recipient.item = Some("EXP_SHARE".into());
    recipient.moves = ["TACKLE", "LEER", "GROWL", "QUICK_ATTACK"]
        .into_iter()
        .map(|id| crystal_core::models::LearnedMove {
            name: id.into(),
            current_pp: runtime.data().moves[id].pp,
            pp_ups: 0,
        })
        .collect();
    {
        let state = controller.shell.shell.session_mut().state_mut();
        for pokemon in [&mut lead, &mut recipient] {
            pokemon.original_trainer_id = state.player_id;
            pokemon.original_trainer_name = state.player_name.clone();
            assert!(state.storage.party.add_pokemon(pokemon.clone()));
        }
        state.sync_party_from_storage();
    }
    super::super::mark_runtime_snapshot_dirty(&mut controller.shell);
    let map = &runtime.data().maps["MnR01C00R00"];
    let mut grass = Vec::new();
    for y in 0..i16::try_from(map.attributes.height * 2).expect("height") {
        for x in 0..i16::try_from(map.attributes.width * 2).expect("width") {
            if collision(&runtime, map, (x, y)) == Some("TALL_GRASS") {
                grass.push(((63 - x).abs() + (24 - y).abs(), (x, y)));
            }
        }
    }
    grass.sort_by_key(|entry| entry.0);
    let target = grass
        .into_iter()
        .find_map(|(_, point)| grass_walk::path(&mut controller, &runtime, point).map(|_| point))
        .expect("reachable grass");
    let mut destination = target;
    let mut neighbor = None;
    for _ in 0..1024 {
        if controller.snapshot().expect("encounter").battle.is_some() {
            break;
        }
        let path = grass_walk::path(&mut controller, &runtime, destination).expect("grass path");
        if let Some(button) = path.first() {
            press(&mut controller, *button);
        } else if destination == target {
            let point = *neighbor.get_or_insert_with(|| {
                [
                    (target.0 + 1, target.1),
                    (target.0 - 1, target.1),
                    (target.0, target.1 + 1),
                    (target.0, target.1 - 1),
                ]
                .into_iter()
                .find(|&point| {
                    grass_walk::path(&mut controller, &runtime, point)
                        .is_some_and(|path| path.len() == 1)
                })
                .expect("grass patrol")
            });
            destination = point;
        } else {
            destination = target;
        }
    }
    assert!(controller.snapshot().expect("encounter").battle.is_some());
    assert!(super::super::rochester_earned::train_wild_with_lead(
        &mut controller,
        &runtime
    ));
    let result = controller.snapshot().expect("earned reward");
    assert!(result.party.slots[1].pokemon.level >= *learn_level);
    assert!(
        result.party.slots[1]
            .pokemon
            .moves
            .iter()
            .any(|mov| mov.name == *learned_move),
        "the EXP Share recipient must learn its earned move"
    );
}
