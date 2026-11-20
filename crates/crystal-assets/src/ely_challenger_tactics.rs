//! A returning challenger can finish weakened foes, take Shift prompts, and use the Bag.
use std::collections::BTreeSet;

use crystal_core::battle::damage::Weather;
use crystal_core::battle::turn::{
    BattleAction, BattleCombatState, BattleSide, battle_action_locked_before_menu,
    preview_battle_move, preview_battle_replacement,
};
use crystal_core::models::{Move, Pokemon};
use crystal_core::state::GameState;

use super::competitive_move_slots;
use super::ely_challenge_run::treat;
use crate::GameDataSet;

#[derive(Clone, Copy, Default)]
struct Attack {
    slot: usize,
    value: i32,
    finish: bool,
}

fn strength(data: &GameDataSet, combat: &BattleCombatState, slot: usize) -> Option<Attack> {
    let known = combat.player.moves.get(slot)?;
    if known.current_pp == 0 {
        return None;
    }
    let source = &data.moves[&known.name];
    if source.power == 0
        || matches!(
            source.effect.as_str(),
            "SELFDESTRUCT" | "COUNTER" | "MIRROR_COAT"
        )
        || source.effect == "DREAM_EATER" && combat.enemy.status.as_deref() != Some("SLEEP")
    {
        return None;
    }
    // Magnitude has no single damage preview. Its native median roll is 70
    // power. This estimate is local to choosing; the actual turn rolls normally.
    let estimate: Move;
    let attack = if source.effect == "MAGNITUDE" {
        estimate = Move {
            power: 70,
            effect: "NORMAL_HIT".into(),
            ..source.clone()
        };
        &estimate
    } else {
        source
    };
    let preview = preview_battle_move(
        combat,
        BattleSide::Player,
        attack,
        &data.items,
        &data.move_priorities,
        &data.battle_stat_multipliers,
        &data.type_categories,
        &data.type_effectiveness,
        &data.weather_modifiers,
    )
    .expect("known move preview");
    let [min, max] = preview.damage_on_hit?;
    let hits = match attack.effect.as_str() {
        "DOUBLE_HIT" | "POISON_MULTI_HIT" => 2,
        "MULTI_HIT" => 3,
        _ => 1,
    };
    let charge = matches!(
        source.effect.as_str(),
        "RAZOR_WIND" | "SKULL_BASH" | "SKY_ATTACK" | "FLY"
    ) || source.effect == "SOLARBEAM" && combat.weather != Weather::Sun;
    let hp = i32::from(combat.enemy.hp.max(1));
    let damage = ((i32::from(min) + i32::from(max)) * hits / 2).min(hp);
    let finish = !charge && i32::from(min) * hits >= hp;
    let fast = preview.priority > data.move_priorities.base_priority
        || (preview.priority == data.move_priorities.base_priority
            && combat.player_loaded_stats.speed > combat.enemy_loaded_stats.speed);
    let mut value = damage * 100 / hp;
    if finish {
        value += if fast { 140 } else { 70 };
    }
    if source.effect == "LEECH_HIT" && combat.player.hp < combat.player.max_hp {
        value += (damage / 3).min(25);
    }
    value = value * i32::from(preview.accuracy.min(100)) / 100;
    if charge || (source.effect == "HYPER_BEAM" && !finish) {
        value /= 2;
    }
    Some(Attack {
        slot,
        value,
        finish,
    })
}

fn offense(data: &GameDataSet, combat: &BattleCombatState) -> Attack {
    competitive_move_slots(combat, BattleSide::Player)
        .into_iter()
        .filter_map(|s| strength(data, combat, s))
        .max_by_key(|m| (m.value, std::cmp::Reverse(m.slot)))
        .unwrap_or_else(|| {
            let slot = competitive_move_slots(combat, BattleSide::Player)[0];
            Attack {
                slot,
                ..Attack::default()
            }
        })
}

fn position(data: &GameDataSet, combat: &BattleCombatState) -> i32 {
    let p = &combat.player;
    if p.hp == 0 {
        return i32::MIN / 2;
    }
    let moves = offense(data, combat);
    // Use public species typing for unrevealed attacks, then refine the estimate
    // when the opponent has actually shown a move in this matchup.
    let observed = combat
        .enemy_last_move
        .as_ref()
        .and_then(|id| data.moves.get(id))
        .and_then(|m| {
            preview_battle_move(
                combat,
                BattleSide::Enemy,
                m,
                &data.items,
                &data.move_priorities,
                &data.battle_stat_multipliers,
                &data.type_categories,
                &data.type_effectiveness,
                &data.weather_modifiers,
            )
            .ok()?
            .damage_on_hit
        });
    let mut vulnerability = 0;
    for t in [&combat.enemy.species.type1, &combat.enemy.species.type2] {
        let mut factor = 100;
        for (i, own) in [&p.species.type1, &p.species.type2].into_iter().enumerate() {
            if i == 1 && p.species.type1 == p.species.type2 {
                continue;
            }
            let eff =
                crystal_core::battle::damage::type_effectiveness(&data.type_effectiveness, t, own)
                    .expect("public types");
            factor = factor * i32::from(eff.numerator) / i32::from(eff.denominator);
        }
        vulnerability = vulnerability.max(factor);
    }
    let danger = observed.map_or(0, |[_, high]| {
        i32::from(high.min(p.hp)) * 42 / i32::from(p.hp)
    });
    moves.value.min(250) + i32::from(p.hp) * 35 / i32::from(p.max_hp) - vulnerability / 5 - danger
        + if combat.player_loaded_stats.speed > combat.enemy_loaded_stats.speed {
            10
        } else {
            0
        }
}

/// Uses the revealed opponent and our own legal move sets. Entry spikes, including
/// a knockout, are included in the choice of a replacement.
pub(super) fn best(data: &GameDataSet, combat: &BattleCombatState) -> usize {
    combat
        .player_party
        .iter()
        .enumerate()
        .filter(|(_, p)| p.hp > 0)
        .max_by_key(|(slot, _)| {
            let score = if *slot == combat.player_party_index {
                position(data, combat)
            } else {
                let entered = preview_battle_replacement(combat, BattleSide::Player, *slot)
                    .expect("able replacement");
                position(data, &entered)
            };
            (
                score,
                *slot == combat.player_party_index,
                std::cmp::Reverse(*slot),
            )
        })
        .expect("able partner")
        .0
}

pub(super) fn lead(data: &GameDataSet, party: &[Pokemon], foe: &Pokemon) -> usize {
    party
        .iter()
        .enumerate()
        .filter(|(_, p)| p.hp > 0)
        .max_by_key(|(s, p)| {
            (
                position(data, &BattleCombatState::new((*p).clone(), foe.clone())),
                std::cmp::Reverse(*s),
            )
        })
        .expect("ready lead")
        .0
}

#[derive(Default)]
pub(super) struct Tactics {
    status_tried: BTreeSet<String>,
    switched_on: BTreeSet<String>,
    previous_revive: bool,
    treated: BTreeSet<(String, usize)>,
}

impl Tactics {
    pub(super) fn choose(
        &mut self,
        data: &GameDataSet,
        state: &GameState,
        combat: &BattleCombatState,
        open_bag: bool,
        triage: bool,
    ) -> BattleAction {
        let strike = offense(data, combat);
        let attack = BattleAction::Move { slot: strike.slot };
        if battle_action_locked_before_menu(combat, BattleSide::Player) {
            return attack;
        }
        let mon = &combat.player;
        let foe = &combat.enemy;
        let has = |item: &str| open_bag && state.bag.quantity(&data.items[item]) > 0;
        let medicine = |item: &str, party_index| BattleAction::PartyItem {
            item_id: item.into(),
            party_index,
            move_slot: None,
        };
        let able = combat.player_party.iter().filter(|p| p.hp > 0).count();
        let enemy_left = combat.enemy_party.iter().filter(|p| p.hp > 0).count();
        let after_revive = std::mem::take(&mut self.previous_revive);
        if triage && has("HYPER_POTION") && !strike.finish {
            // A fragile active partner can spend its turn restoring a reserve
            // before the next forced switch. Cap treatment per opponent/partner
            // so repeated attacks cannot burn the whole receipt without reply.
            let patient = combat
                .player_party
                .iter()
                .enumerate()
                .filter(|(slot, p)| {
                    p.hp > 0
                        && p.hp * 2 <= p.max_hp
                        && !self.treated.contains(&(foe.species.id.clone(), *slot))
                        && (*slot == combat.player_party_index
                            || (able <= 3 && mon.hp * 2 < mon.max_hp))
                })
                .max_by_key(|(slot, p)| {
                    let mut restored = (*p).clone();
                    restored.hp = restored.max_hp;
                    (
                        position(data, &BattleCombatState::new(restored, foe.clone())),
                        std::cmp::Reverse(*slot),
                    )
                });
            if let Some((slot, _)) = patient {
                self.treated.insert((foe.species.id.clone(), slot));
                return medicine("HYPER_POTION", slot);
            }
        }
        if has("REVIVE")
            && able <= if triage { 3 } else { 2 }
            && (!triage || !after_revive)
            && (able == 1 || !strike.finish)
            && (enemy_left > 1 || !strike.finish)
        {
            let target = combat
                .player_party
                .iter()
                .enumerate()
                .filter(|(_, p)| p.hp == 0)
                .max_by_key(|(slot, p)| {
                    let mut revived = (*p).clone();
                    revived.hp = (revived.max_hp / 2).max(1);
                    (
                        position(data, &BattleCombatState::new(revived, foe.clone())),
                        std::cmp::Reverse(*slot),
                    )
                });
            if let Some((slot, _)) = target {
                self.previous_revive = true;
                return medicine("REVIVE", slot);
            }
        }
        if strike.finish {
            return attack;
        }
        if !triage && has("HYPER_POTION") && mon.hp * 3 < mon.max_hp && strike.value >= 30 {
            return medicine("HYPER_POTION", combat.player_party_index);
        }
        if !triage && has("SUPER_POTION") && mon.hp * 2 <= mon.max_hp && mon.max_hp - mon.hp >= 35 {
            return medicine("SUPER_POTION", combat.player_party_index);
        }
        if has("FULL_HEAL")
            && matches!(mon.status.as_deref(), Some("FREEZE" | "SLEEP"))
            && mon.hp * 2 > mon.max_hp
        {
            return medicine("FULL_HEAL", combat.player_party_index);
        }
        if !self.status_tried.contains(&foe.species.id)
            && foe.status.is_none()
            && combat.enemy_safeguard_turns == 0
            && foe.species.type1 != "GRASS"
            && foe.species.type2 != "GRASS"
            && (strike.value < 33
                || combat.player_loaded_stats.speed < combat.enemy_loaded_stats.speed)
        {
            for slot in competitive_move_slots(combat, BattleSide::Player) {
                if mon.moves.get(slot).is_some_and(|m| {
                    matches!(m.name.as_str(), "SPORE" | "SLEEP_POWDER" | "STUN_SPORE")
                }) {
                    self.status_tried.insert(foe.species.id.clone());
                    return BattleAction::Move { slot };
                }
            }
        }
        if !self.switched_on.contains(&foe.species.id)
            && combat.player_trap.is_none()
            && combat.player_escape_trap.is_none()
            && strike.value < 34
        {
            let slot = best(data, combat);
            if slot != combat.player_party_index {
                let entered = preview_battle_replacement(combat, BattleSide::Player, slot)
                    .expect("able alternative");
                if position(data, &entered) > position(data, combat) + 38 {
                    self.switched_on.insert(foe.species.id.clone());
                    return BattleAction::Switch { party_index: slot };
                }
            }
        }
        attack
    }
}

/// Use free turns between matches. Earlier in the circuit a Hyper Potion
/// stays in the Bag until at least half the recipient's HP is missing.
pub(super) fn regroup(data: &GameDataSet, state: &mut GameState, champion_next: bool) {
    let mut slots: Vec<_> = (0..6).collect();
    for &slot in &slots {
        let member = state.storage.party.pokemon[slot]
            .as_ref()
            .expect("same six partners");
        if member.hp == 0 {
            treat(data, state, slot, "REVIVE");
        }
        let member = state.storage.party.pokemon[slot]
            .as_ref()
            .expect("treated partner");
        if member.hp > 0 && member.status.is_some() {
            treat(data, state, slot, "FULL_HEAL");
        }
    }
    slots.sort_by_key(|&slot| {
        let p = state.storage.party.pokemon[slot]
            .as_ref()
            .expect("candidate");
        std::cmp::Reverse(p.max_hp - p.hp)
    });
    for slot in slots {
        let p = state.storage.party.pokemon[slot]
            .as_ref()
            .expect("candidate");
        if p.hp > 0
            && if champion_next {
                p.hp * 4 < p.max_hp * 3
            } else {
                p.hp * 2 <= p.max_hp
            }
        {
            treat(data, state, slot, "HYPER_POTION");
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_MINNESOTA_BATTLE_TEST_PACK and the actual Ely teams"]
fn ely_tactical_prairondo_uses_its_paid_ground_move_against_musketunge() {
    let path = std::env::var_os("CRYSTAL_MINNESOTA_BATTLE_TEST_PACK").expect("Ely edition");
    let pack = crate::read_verified_compiled_game_pack(path).expect("verified archive");
    let data = pack.data();
    let team = super::ely_challenger_parties::party(data, 0);
    let bird = team
        .into_iter()
        .find(|p| p.species.id == "PRAIRONDO")
        .expect("trained partner");
    let opponents = crystal_core::battle::start::materialize_trainer_party(
        &data.trainers.trainers["MN_CHAMPION"],
        &data.pokemon,
        &data.learnsets,
        &data.moves,
        &data.growth_rates,
    )
    .expect("Casey's team");
    let fish = opponents
        .into_iter()
        .find(|p| p.species.id == "MUSKETUNGE")
        .expect("Casey's Water/Steel partner");
    let battle = BattleCombatState::new(bird, fish);
    let BattleAction::Move { slot } = super::attack(data, &battle) else {
        panic!("original attack choice")
    };
    assert_ne!(battle.player.moves[slot].name, "MAGNITUDE");
    assert_eq!(
        battle.player.moves[offense(data, &battle).slot].name,
        "MAGNITUDE"
    );
}
