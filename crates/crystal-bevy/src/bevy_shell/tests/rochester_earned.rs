//! New Game to the Care Badge using earned partners and production input only.
#[path = "earned_mine_choices.rs"]
mod mine_choices;

use crystal_core::battle::damage::{calculate_type_effectiveness_multiplier, is_physical_move};
use crystal_core::battle::stats::apply_stage;
use crystal_core::battle::turn::move_priority;
use crystal_core::input::GameButton;
use crystal_core::models::Pokemon;
use crystal_core::models::pokemon::Stat;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind, RuntimeShellSnapshot};

use super::super::{VisibleShellController, VisibleTmHmDecision, carried_battle_non_ball_item_ids};
use super::authored_route_fixture::indoor_walk;
use super::northstar_earned_gyms;
use super::rochester::{delivery, flee, trail, try_trail_with};
use super::winona::{at, continue_in_place, continued, finish, press, travel};

/// A read-only attack preference, not an engine turn or RNG preview.
/// Account for physical/special stats, STAB and types so Leer is followed by a
/// physical attack rather than blindly preferring the largest printed power.
pub(super) fn best_attack(
    r: &CrystalRuntime,
    b: &crystal_runtime::RuntimeBattleSnapshot,
    p: &Pokemon,
) -> usize {
    let mut types = vec![b.enemy_pokemon.species.type1.clone()];
    if b.enemy_pokemon.species.type2 != types[0] {
        types.push(b.enemy_pokemon.species.type2.clone());
    }
    b.player_moves
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.current_pp > 0
                && b.player_disabled_move.as_deref() != Some(m.name.as_str())
                && r.data().moves[&m.name].power > 0
        })
        .max_by_key(|(_, learned)| {
            let m = &r.data().moves[&learned.name];
            let physical =
                is_physical_move(&r.data().type_categories, m).expect("native move category");
            let (attack, defense, a, d) = if physical {
                (
                    p.attack,
                    b.enemy_pokemon.defense,
                    Stat::Attack,
                    Stat::Defense,
                )
            } else {
                (
                    p.special_attack,
                    b.enemy_pokemon.special_defense,
                    Stat::SpecialAttack,
                    Stat::SpecialDefense,
                )
            };
            let attack = apply_stage(&r.data().battle_stat_multipliers, attack, p.stat_boosts[&a])
                .expect("native attack stage");
            let defense = apply_stage(
                &r.data().battle_stat_multipliers,
                defense,
                b.enemy_pokemon.stat_boosts[&d],
            )
            .expect("native defense stage");
            let effect = calculate_type_effectiveness_multiplier(
                &r.data().type_effectiveness,
                &m.move_type,
                &types,
            )
            .expect("known visible opponent types");
            let stab = if m.move_type == p.species.type1 || m.move_type == p.species.type2 {
                3_u64
            } else {
                2
            };
            let damage_score = u64::from(m.power)
                * u64::from(attack)
                * u64::from(effect.numerator)
                * stab
                * u64::from(m.accuracy.max(1))
                * 100
                / (u64::from(defense).max(1) * u64::from(effect.denominator));
            // Finish a visibly low opponent before a faster reply. This reads
            // the move's authored priority, without executing or previewing a turn.
            let priority_finish = b.enemy_pokemon.hp * 4 <= b.enemy_pokemon.max_hp
                && effect.numerator > 0
                && move_priority(m, &r.data().move_priorities).expect("authored move priority")
                    > r.data().move_priorities.base_priority;
            (priority_finish, damage_score)
        })
        .map(|(i, _)| i)
        .expect("earned damaging move with PP")
}

fn spent_move_pp(before: &RuntimeShellSnapshot, after: &RuntimeShellSnapshot) -> bool {
    after.party.slots.iter().any(|slot| {
        before.party.slots.iter().any(|old| {
            old.index == slot.index
                && slot.pokemon.moves.iter().any(|m| {
                    old.pokemon
                        .moves
                        .iter()
                        .any(|om| om.name == m.name && m.current_pp < om.current_pp)
                })
        })
    })
}

fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("journey event")
}

fn battle(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str, win: &str) {
    assert!(try_battle(c, r, id, win), "{id}: actual victory required");
}

pub(super) fn try_battle(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    id: &str,
    win: &str,
) -> bool {
    play_battle(c, r, id, Victory::Event(win), false, true)
}

/// Fight a visible wild opponent with the chosen training lead kept in place.
pub(super) fn train_wild(c: &mut VisibleShellController, r: &CrystalRuntime) -> bool {
    play_battle(c, r, "WILD", Victory::Wild, false, false)
}

enum Victory<'a> {
    Event(&'a str),
    League(u8),
    Wild,
}

pub(super) fn try_league_battle(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    id: &str,
    completed: u8,
) -> bool {
    play_battle(c, r, id, Victory::League(completed), false, false)
}

/// Attempt an ordinary wild capture, including throwing at full health.
/// Returns whether the native capture actually adds a party member.
pub(super) fn try_recruit(c: &mut VisibleShellController, r: &CrystalRuntime) -> bool {
    let before = c.snapshot().expect("wild recruitment");
    assert!(before.party.slots.len() < 6);
    assert!(matches!(
        before.battle.as_ref().expect("encounter to recruit").kind,
        RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. }
    ));
    assert!(before.bag.balls.iter().any(|b| b.quantity > 0));
    let _ = play_battle(c, r, "WILD", Victory::Wild, true, false);
    c.snapshot()
        .expect("actual recruitment outcome")
        .party
        .slots
        .len()
        > before.party.slots.len()
}

fn play_battle(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    id: &str,
    victory: Victory<'_>,
    recruit: bool,
    allow_training_switch: bool,
) -> bool {
    let initial = c.snapshot().expect("before earned battle");
    let mine = matches!(
        id,
        "MN_LODE_FOREMAN" | "MN_REED_MINE" | "MN_LODE_CHIEF" | "MN_REED_BOUNDARY"
    ) || id.starts_with("MN_LODE_ARC_");
    let mut saw_battle = false;
    let mut spent_pp = false;
    let mut lowered_accuracy = false;
    let mut switched_since_attack = false;
    let mut traced_turn = None;
    let capture = id == "WILD" && (initial.party.slots.len() < 2 || recruit);
    let initial_balls: u32 = initial
        .bag
        .balls
        .iter()
        .map(|b| u32::from(b.quantity))
        .sum();
    for _ in 0..1024 {
        let s = c.snapshot().expect("earned battle");
        if let Some(button) = learning_button(c, r) {
            press(c, button);
            continue;
        }
        if let Some(choice) = &c.shell.pending_name_choice {
            if choice.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                press(c, GameButton::Down);
                press(c, GameButton::A);
            }
            continue;
        }
        if c.shell.pokedex_menu_open {
            press(c, GameButton::A);
            continue;
        }
        if let Some(b) = &s.battle {
            saw_battle = true;
            lowered_accuracy |= b.enemy_pokemon.stat_boosts[&Stat::Accuracy] < 0;
            let potions = c
                .shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POTION"]);
            if (matches!(
                id,
                "MN_GYM_1" | "MN_ICE_ENFORCER" | "MN_GYM_5" | "MN_GYM_6" | "MN_LODE_FOREMAN"
            ) || matches!(&victory, Victory::League(_)))
                && traced_turn != Some(b.player_turns_taken)
            {
                traced_turn = Some(b.player_turns_taken);
                eprintln!(
                    "leader turn {} {:?}: player {} enemy {} stage {:?}, potions {}, super potions {}",
                    b.player_turns_taken,
                    b.enemy_pokemon.species.id,
                    s.party.slots[b.active_player_party_index.unwrap_or(0)]
                        .pokemon
                        .hp,
                    b.enemy_pokemon.hp,
                    b.enemy_pokemon.stat_boosts,
                    potions,
                    c.shell
                        .shell
                        .session()
                        .state()
                        .bag
                        .quantity(&r.data().items["SUPER_POTION"])
                );
            }
            if id == "WILD" {
                assert!(matches!(
                    b.kind,
                    RuntimeBattleKind::Wild { .. } | RuntimeBattleKind::StaticWild { .. }
                ));
            } else {
                assert!(
                    matches!(&b.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == id)
                );
            }
            spent_pp |= spent_move_pp(&initial, &s);
        } else if saw_battle {
            break;
        }
        let b = s.battle.as_ref();
        let active = b.and_then(|b| b.active_player_party_index).unwrap_or(0);
        let p = &s.party.slots[active].pokemon;
        let league = matches!(&victory, Victory::League(_));
        let status_cure = (p.status.as_deref() == Some("SLEEP")
            && c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["AWAKENING"])
                > 0)
        .then_some("AWAKENING");
        let status_cure = status_cure.or_else(|| {
            (league && p.status.is_some())
                .then(|| {
                    ["FULL_RESTORE", "FULL_HEAL"].into_iter().find(|id| {
                        c.shell
                            .shell
                            .session()
                            .state()
                            .bag
                            .quantity(&r.data().items[*id])
                            > 0
                    })
                })
                .flatten()
        });
        let healing_item = status_cure.or_else(|| {
            (p.hp > 0 && p.hp * 2 < p.max_hp)
                .then(|| {
                    let choices: &[&str] = if league {
                        &[
                            "FULL_RESTORE",
                            "MAX_POTION",
                            "HYPER_POTION",
                            "SUPER_POTION",
                            "POTION",
                        ]
                    } else {
                        &["HYPER_POTION", "SUPER_POTION", "POTION"]
                    };
                    choices.iter().copied().find(|id| {
                        c.shell
                            .shell
                            .session()
                            .state()
                            .bag
                            .quantity(&r.data().items[*id])
                            > 0
                    })
                })
                .flatten()
        });
        let heal = healing_item.is_some();
        let switch_training = allow_training_switch
            && id == "WILD"
            && active == 0
            && s.party.slots.iter().any(|partner| {
                partner.index != active
                    && partner.pokemon.hp > 0
                    && partner.pokemon.level >= p.level + 2
            });
        let catch = !heal
            && capture
            && (recruit || b.is_some_and(|b| b.enemy_pokemon.hp * 2 <= b.enemy_pokemon.max_hp))
            && s.bag.balls.iter().any(|b| b.quantity > 0);
        let change_prompt = s.ui.menu.as_ref().is_some_and(|m| {
            m.layout
                .vertical_menus
                .iter()
                .flat_map(|m| &m.options)
                .any(|o| o.contains("change <PKMN>"))
        });
        let mine_partner = mine.then(|| mine_choices::partner(r, &s)).flatten();
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if change_prompt {
            if mine_partner.is_some_and(|target| target != active) {
                if c.shell
                    .battle_shift_prompt_cursor
                    .as_ref()
                    .expect("native Shift cursor")
                    .option_index
                    == 0
                {
                    GameButton::A
                } else {
                    GameButton::Up
                }
            } else {
                GameButton::B
            }
        } else if c.shell.battle_pack_target_mode.is_some() {
            if c.shell.party_cursor < active {
                GameButton::Down
            } else if c.shell.party_cursor > active {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_switch_cursor {
            let target = mine_partner
                .filter(|&target| target != active)
                .or_else(|| {
                    s.party
                        .slots
                        .iter()
                        .filter(|s| s.pokemon.hp > 0 && s.index != active)
                        .max_by_key(|s| s.pokemon.level)
                        .map(|s| s.index)
                })
                .expect("living earned partner");
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_party_action_cursor {
            if cursor.option_index > 0 {
                GameButton::Up
            } else {
                switched_since_attack = true;
                GameButton::A
            }
        } else if c.shell.field_pack_action_cursor.is_some() {
            GameButton::A
        } else if c.shell.ball_cursor.is_some() {
            if heal {
                GameButton::Left
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.bag_cursor {
            if catch {
                GameButton::Right
            } else {
                let ids = carried_battle_non_ball_item_ids(&s);
                let target = ids
                    .iter()
                    .position(|id| Some(id.as_str()) == healing_item)
                    .expect("purchased healing item");
                if cursor.option_index < target {
                    GameButton::Down
                } else if cursor.option_index > target {
                    GameButton::Up
                } else {
                    GameButton::A
                }
            }
        } else if let (Some(cursor), Some(b)) = (&c.shell.battle_move_cursor, &s.battle) {
            let support = b.player_moves.iter().position(|m| {
                matches!(id, "MN_GYM_1" | "MN_ICE_ENFORCER")
                    && m.current_pp > 0
                    && ((m.name == "MUD_SLAP"
                        && id == "MN_ICE_ENFORCER"
                        && b.enemy_pokemon.stat_boosts[&Stat::Accuracy] > -3)
                        || (m.name == "SMOKESCREEN"
                            && !b.player_moves.iter().any(|m| m.name == "MUD_SLAP")
                            && b.enemy_pokemon.stat_boosts[&Stat::Accuracy] > -3)
                        || (m.name == "LEER"
                            && id == "MN_GYM_1"
                            && b.enemy_pokemon.stat_boosts[&Stat::Accuracy] <= -3
                            && b.enemy_pokemon.stat_boosts[&Stat::Defense] > -1))
            });
            let target = support.unwrap_or_else(|| best_attack(r, b, p));
            if cursor.option_index == target {
                switched_since_attack = false;
            }
            if mine && cursor.option_index == target {
                eprintln!(
                    "mine attack: {} uses {}, disabled={:?}, previous={:?}, status={:?}",
                    p.species.id,
                    b.player_moves[target].name,
                    b.player_disabled_move,
                    b.player_last_move,
                    p.status
                );
            }
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            let target = if heal || catch {
                2
            } else if switch_training
                || (!switched_since_attack && mine_partner.is_some_and(|target| target != active))
            {
                1
            } else {
                0
            };
            if cursor.option_index / 2 < target / 2 {
                GameButton::Down
            } else if cursor.option_index / 2 > target / 2 {
                GameButton::Up
            } else if cursor.option_index % 2 > target % 2 {
                GameButton::Left
            } else if cursor.option_index % 2 < target % 2 {
                GameButton::Right
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert!(saw_battle, "{id}: actual battle required");
    finish(c);
    let after = c.snapshot().expect("earned outcome");
    spent_pp |= spent_move_pp(&initial, &after);
    assert!(after.battle.is_none());
    let same_clinic_loss = c.shell.shell.session().state().battle_result & 0x3f == 1
        && match &victory {
            Victory::Event(flag) => flag.is_empty() || !event(c, flag),
            Victory::Wild | Victory::League(_) => true,
        };
    if same_clinic_loss
        || (after.overworld.map_name.ends_with("Pokecenter1F")
            && initial.overworld.map_name != after.overworld.map_name)
    {
        match victory {
            Victory::Event(flag) => {
                assert!(flag.is_empty() || !event(c, flag), "loss grants no victory");
            }
            Victory::League(_) => assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .script_runtime
                    .memory
                    .get("wMnLeagueRun")
                    .is_none_or(|value| value == "0"),
                "a native League loss resets its run"
            ),
            Victory::Wild => {}
        }
        if std::env::var_os("CRYSTAL_WINONA_SAVE_DIR").is_some() {
            let root = c.shell.asset_root.clone();
            continue_in_place(c, &root, r, &format!("campaign-{id}-native-loss"));
        }
        return false;
    }
    if id == "WILD" && c.shell.shell.session().state().battle_result & 0x3f == 2 {
        assert_eq!(after.overworld.map_name, initial.overworld.map_name);
        assert_eq!(
            after
                .party
                .slots
                .iter()
                .map(|s| s.pokemon.experience)
                .sum::<i32>(),
            initial
                .party
                .slots
                .iter()
                .map(|s| s.pokemon.experience)
                .sum::<i32>(),
            "a native escape cannot count as earned training"
        );
        eprintln!(
            "wild escape: {:?}; no training credit",
            initial.battle.as_ref().map(|b| &b.enemy_pokemon.species.id)
        );
        return false;
    }
    assert!(
        spent_pp || capture,
        "{id}: actual PP spending or capture required"
    );
    if id == "WILD" {
        assert_eq!(
            after.overworld.map_name, initial.overworld.map_name,
            "wild loss cannot pass for progress"
        );
    } else {
        assert!(
            match victory {
                Victory::Event(flag) => event(c, flag),
                Victory::League(completed) => c
                    .shell
                    .shell
                    .session()
                    .state()
                    .script_runtime
                    .memory
                    .get("wMnLeagueRun")
                    .is_some_and(|value| value.parse::<u8>() == Ok(completed)),
                Victory::Wild => false,
            },
            "{id}: actual trainer victory required; result={}, map={}, party={:?}, trainer_result={:?}, ui={:?}",
            c.shell.shell.session().state().battle_result,
            after.overworld.map_name,
            after.party,
            c.shell.battle_trainer_result,
            after.ui
        );
    }
    if id == "MN_ICE_ENFORCER"
        // A boosted review can faint the target before the secondary effect.
        && std::env::var_os("CRYSTAL_NORTHSTAR_REVIEW_LEVEL").is_none()
        && initial.party.slots[0]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "MUD_SLAP")
    {
        assert!(
            lowered_accuracy,
            "the native pack's earned Mud-Slap must lower accuracy in battle"
        );
    }
    if capture && after.party.slots.len() > initial.party.slots.len() {
        assert_eq!(
            after.party.slots.len(),
            initial.party.slots.len() + 1,
            "native capture adds one partner"
        );
        assert!(
            after
                .bag
                .balls
                .iter()
                .map(|b| u32::from(b.quantity))
                .sum::<u32>()
                < initial_balls
        );
    } else if initial.party.slots.iter().all(|s| s.pokemon.level == 100) {
        assert_eq!(
            after
                .party
                .slots
                .iter()
                .map(|s| s.pokemon.experience)
                .sum::<i32>(),
            initial
                .party
                .slots
                .iter()
                .map(|s| s.pokemon.experience)
                .sum::<i32>(),
            "the native level cap cannot award additional XP"
        );
    } else {
        assert!(
            after
                .party
                .slots
                .iter()
                .map(|s| s.pokemon.experience)
                .sum::<i32>()
                > initial
                    .party
                    .slots
                    .iter()
                    .map(|s| s.pokemon.experience)
                    .sum::<i32>()
        );
    }
    eprintln!(
        "earned {id}: level {} HP {}/{}",
        after.party.slots[0].pokemon.level,
        after.party.slots[0].pokemon.hp,
        after.party.slots[0].pokemon.max_hp
    );
    true
}

/// Choose moves through the production learning UI, preserving the useful
/// attacks and Smokescreen instead of blindly overwriting the first slot.
pub(super) fn learning_button(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
) -> Option<GameButton> {
    let snapshot = c.snapshot().expect("move-learning owner");
    let pending = snapshot.pending_move_learn.as_ref()?;
    let pokemon = &snapshot
        .party
        .slots
        .iter()
        .find(|slot| slot.index == pending.party_index)
        .expect("learning party member")
        .pokemon;
    let replacement = ["LEER", "GROWL", "FORESIGHT", "DEFENSE_CURL", "TACKLE"]
        .iter()
        .find_map(|name| pokemon.moves.iter().position(|m| m.name == *name))
        .or_else(|| {
            let new_move = &r.data().moves[&pending.learned_move.name];
            if new_move.power == 0
                || pokemon.moves.iter().any(|m| {
                    let old = &r.data().moves[&m.name];
                    old.move_type == new_move.move_type && old.power >= new_move.power
                })
            {
                return None;
            }
            pokemon
                .moves
                .iter()
                .enumerate()
                .filter(|(_, m)| {
                    let old = &r.data().moves[&m.name];
                    old.move_type == new_move.move_type
                        && old.power > 0
                        && old.power < new_move.power
                })
                .min_by_key(|(_, m)| r.data().moves[&m.name].power)
                .map(|(i, _)| i)
        });
    if !c.shell.battle_messages.is_empty() {
        return Some(GameButton::A);
    }
    if let Some(cursor) = &c.shell.move_learn_decision_cursor {
        let target = usize::from(
            replacement.is_none()
                && c.shell.move_learn_decision == Some(VisibleTmHmDecision::ForgetMove),
        );
        return Some(if cursor.option_index == target {
            GameButton::A
        } else if cursor.option_index < target {
            GameButton::Down
        } else {
            GameButton::Up
        });
    }
    if c.shell.move_learn_forget_menu_open {
        let target = replacement.unwrap_or(pokemon.moves.len());
        let cursor = c
            .shell
            .party_move_cursor
            .as_ref()
            .expect("forget-move cursor")
            .option_index;
        return Some(if cursor < target {
            GameButton::Down
        } else if cursor > target {
            GameButton::Up
        } else {
            GameButton::A
        });
    }
    Some(GameButton::A)
}

pub(super) fn heal(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (7, 5));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    let s = c.snapshot().expect("earned clinic recovery");
    assert!(s.party.slots.iter().all(|slot| {
        let p = &slot.pokemon;
        p.hp == p.max_hp
            && p.status.is_none()
            && p.moves
                .iter()
                .all(|m| m.current_pp == r.data().moves[&m.name].pp)
    }));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_ROCHESTER_POKECENTER_1F")
    );
}

pub(super) fn clinic_exit(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (10, 14));
    travel(c, GameButton::Down, "MnRochester", (15, 8));
}

pub(super) fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime, quantity: u16) {
    buy(c, r, "POTION", quantity);
}

pub(super) fn buy(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str, quantity: u16) {
    indoor_walk(c, r, (29, 10));
    travel(c, GameButton::Up, "MnRochesterMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, id, quantity);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, "MnRochester", (29, 10));
}

pub(super) fn checkout(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    id: &str,
    quantity: u16,
) {
    for _ in 0..32 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let before = c.shell.shell.session().state().clone();
    let target = c
        .snapshot()
        .expect("native mart stock")
        .pending_shop
        .expect("shop")
        .inventory
        .iter()
        .position(|s| s == id)
        .unwrap_or_else(|| panic!("native mart does not stock {id}"));
    press(c, GameButton::A);
    for _ in 0..32 {
        let index = c
            .shell
            .menu_cursor
            .as_ref()
            .expect("native buy cursor")
            .option_index;
        if index == target {
            break;
        }
        press(
            c,
            if index < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    press(c, GameButton::A);
    assert_eq!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("purchase quantity")
            .item_id,
        id
    );
    for _ in 1..quantity {
        press(c, GameButton::Up);
    }
    press(c, GameButton::A);
    press(c, GameButton::A);
    let item = &r.data().items[id];
    let state = c.shell.shell.session().state();
    assert_eq!(
        state.bag.quantity(item),
        before.bag.quantity(item) + quantity
    );
    assert_eq!(
        state.money,
        before.money - u32::from(quantity) * u32::from(item.price)
    );
    assert_eq!(state.flags, before.flags);
    for _ in 0..96 {
        let s = c.snapshot().expect("shop departure");
        if s.pending_shop.is_none()
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            break;
        }
        let button = if s.pending_shop.is_none() || c.shell.shop_notice.is_some() {
            GameButton::A
        } else if c.shell.shop_quantity.is_some() {
            GameButton::B
        } else if let Some(cursor) = &c.shell.shop_top_cursor {
            if cursor.option_index == 2 {
                GameButton::A
            } else {
                GameButton::Down
            }
        } else {
            GameButton::B
        };
        press(c, button);
    }
    assert!(c.snapshot().expect("closed shop").pending_shop.is_none());
}

fn recruit(c: &mut VisibleShellController, r: &CrystalRuntime, root: &crystal_assets::AssetRoot) {
    indoor_walk(c, r, (15, 8));
    travel(c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
    heal(c, r);
    clinic_exit(c, r);
    indoor_walk(c, r, (38, 18));
    travel(c, GameButton::Right, "MnR01C00R00", (0, 24));
    // Look for a useful second partner after the two gym qualifications.
    // These are the pack's ordinary scaled encounters, with no RNG/level edits.
    for step in 0..256 {
        if c.snapshot().expect("recruited party").party.slots.len() == 2 {
            break;
        }
        let tile = if step % 2 == 0 { (54, 24) } else { (57, 24) };
        let arrived = try_trail_with(c, r, tile, |c| {
            let s = c.snapshot().expect("visible encounter");
            let b = s.battle.as_ref().expect("wild partner");
            if s.party.slots.len() < 2
                && b.enemy_pokemon.level >= 8
                && b.enemy_pokemon.level <= s.party.slots[0].pokemon.level + 1
            {
                let _ = try_battle(c, r, "WILD", "");
            } else {
                flee(c);
            }
        });
        if !arrived {
            at(c, "MnRochesterPokecenter1F", (10, 13));
            continue_in_place(c, root, r, "fresh-recruit-blackout");
            heal(c, r);
            clinic_exit(c, r);
            if c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POKE_BALL"])
                == 0
            {
                let quantity = (c.shell.shell.session().state().money
                    / u32::from(r.data().items["POKE_BALL"].price))
                .min(3) as u16;
                assert!(
                    quantity > 0,
                    "catching supplies require actual earned money"
                );
                buy(c, r, "POKE_BALL", quantity);
            }
            indoor_walk(c, r, (38, 18));
            travel(c, GameButton::Right, "MnR01C00R00", (0, 24));
        }
    }
    assert_eq!(
        c.snapshot().expect("native recruitment").party.slots.len(),
        2
    );
    trail(c, r, (0, 24));
    travel(c, GameButton::Left, "MnRochester", (39, 18));
}

fn train_for_care(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &crystal_assets::AssetRoot,
    level: u8,
) {
    for outing in 0..24 {
        if c.snapshot().expect("earned starter level").party.slots[0]
            .pokemon
            .level
            >= level
        {
            return;
        }
        if c.snapshot().expect("training recovery").overworld.map_name != "MnRochesterPokecenter1F"
        {
            indoor_walk(c, r, (15, 8));
            travel(c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
        }
        heal(c, r);
        clinic_exit(c, r);
        indoor_walk(c, r, (38, 18));
        travel(c, GameButton::Right, "MnR01C00R00", (0, 24));
        let mut fought = false;
        for step in 0..64 {
            let target = if step % 2 == 0 { (54, 24) } else { (57, 24) };
            let arrived = try_trail_with(c, r, target, |c| {
                if fought {
                    flee(c);
                    return;
                }
                let s = c.snapshot().expect("visible training encounter");
                let b = s.battle.as_ref().expect("wild training");
                if b.enemy_pokemon.level <= s.party.slots[0].pokemon.level + 1
                    && !["WATER", "GROUND", "ROCK"]
                        .contains(&b.enemy_pokemon.species.type1.as_str())
                    && !["WATER", "GROUND", "ROCK"]
                        .contains(&b.enemy_pokemon.species.type2.as_str())
                {
                    let _ = try_battle(c, r, "WILD", "");
                    fought = true;
                } else {
                    flee(c);
                }
            });
            if !arrived || fought {
                break;
            }
        }
        if c.snapshot().expect("training result").overworld.map_name == "MnR01C00R00" {
            trail(c, r, (0, 24));
            travel(c, GameButton::Left, "MnRochester", (39, 18));
        } else {
            at(c, "MnRochesterPokecenter1F", (10, 13));
            continue_in_place(c, root, r, &format!("fresh-training-loss-{outing}"));
        }
    }
    assert!(
        c.snapshot().expect("native earned training").party.slots[0]
            .pokemon
            .level
            >= level,
        "bounded ordinary wild training must prepare Care"
    );
}

pub(super) fn care_badge() -> (
    crystal_assets::AssetRoot,
    CrystalRuntime,
    VisibleShellController,
) {
    let (root, r, mut c) = delivery();
    heal(&mut c, &r);
    clinic_exit(&mut c, &r);
    supplies(&mut c, &r, 8);
    indoor_walk(&mut c, &r, (38, 18));
    travel(&mut c, GameButton::Right, "MnR01C00R00", (0, 24));
    for (tile, facing, id, win) in [
        (
            (44, 27),
            GameButton::Right,
            "MN_R01_T01",
            "EVENT_MN_R01_TRAINER_01",
        ),
        (
            (25, 28),
            GameButton::Down,
            "MN_R01_T02",
            "EVENT_MN_R01_TRAINER_02",
        ),
        (
            (33, 14),
            GameButton::Up,
            "MN_R01_T03",
            "EVENT_MN_R01_TRAINER_03",
        ),
    ] {
        trail(&mut c, &r, tile);
        press(&mut c, facing);
        press(&mut c, GameButton::A);
        battle(&mut c, &r, id, win);
        assert!(!c.shell.shell.session().state().badges.johto[0]);
    }
    trail(&mut c, &r, (19, 12));
    let potions = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["POTION"]);
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_R01_ITEM_01"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["POTION"]),
        potions + 1
    );
    trail(&mut c, &r, (0, 24));
    travel(&mut c, GameButton::Left, "MnRochester", (39, 18));
    for (tile, facing, id, win) in [
        (
            (6, 16),
            GameButton::Left,
            "MN_GYM_1_TRAINER_1",
            "EVENT_MN_GYM_1_TRAINER_1_DEFEATED",
        ),
        (
            (6, 8),
            GameButton::Left,
            "MN_GYM_1_TRAINER_2",
            "EVENT_MN_GYM_1_TRAINER_2_DEFEATED",
        ),
        (
            (6, 4),
            GameButton::Up,
            "MN_GYM_1",
            "EVENT_MN_GYM_1_DEFEATED",
        ),
    ] {
        if id == "MN_GYM_1"
            && c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POTION"])
                <= 3
        {
            supplies(&mut c, &r, 2);
        }
        if id == "MN_GYM_1" {
            recruit(&mut c, &r, &root);
            train_for_care(&mut c, &r, &root, 12);
            if r.data().maps.contains_key("MnBoundaryIntake")
                && !event(&c, "EVENT_MN_REED_ROCHESTER_DEFEATED")
            {
                if c.snapshot().expect("early rival return").overworld.map_name
                    != "MnRochesterPokecenter1F"
                {
                    indoor_walk(&mut c, &r, (15, 8));
                    travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
                }
                heal(&mut c, &r);
                indoor_walk(&mut c, &r, (8, 10));
                super::northstar_ice_lair::talk(&mut c, GameButton::Right);
                super::northstar_ice_lair::choice(&mut c, true);
                assert!(try_battle(
                    &mut c,
                    &r,
                    "MN_REED_ROCHESTER",
                    "EVENT_MN_REED_ROCHESTER_DEFEATED"
                ));
                finish(&mut c);
                heal(&mut c, &r);
                clinic_exit(&mut c, &r);
                continue_in_place(&mut c, &root, &r, "campaign-rochester-rival-earned");
            }
            if c.snapshot()
                .expect("post-training location")
                .overworld
                .map_name
                == "MnRochesterPokecenter1F"
            {
                heal(&mut c, &r);
                clinic_exit(&mut c, &r);
            }
            let owned = c
                .shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POTION"]);
            let available = (c.shell.shell.session().state().money
                / u32::from(r.data().items["POTION"].price))
            .min(u32::from(6_u16.saturating_sub(owned))) as u16;
            if available > 0 {
                supplies(&mut c, &r, available);
            }
        }
        for attempt in 0..4 {
            if c.snapshot().expect("recovery or street").overworld.map_name
                != "MnRochesterPokecenter1F"
            {
                indoor_walk(&mut c, &r, (15, 8));
                travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
            }
            heal(&mut c, &r);
            clinic_exit(&mut c, &r);
            indoor_walk(&mut c, &r, (30, 24));
            let selected = northstar_earned_gyms::selected(&r);
            let arrival = if selected {
                northstar_earned_gyms::arrival(&r, 1)
            } else {
                (6, 17)
            };
            travel(&mut c, GameButton::Up, "MnRochesterGym", arrival);
            if selected {
                northstar_earned_gyms::rochester_challenge(&mut c, &r, id);
            } else {
                indoor_walk(&mut c, &r, tile);
                press(&mut c, facing);
                press(&mut c, GameButton::A);
            }
            if try_battle(&mut c, &r, id, win) {
                break;
            }
            at(&mut c, "MnRochesterPokecenter1F", (10, 13));
            c = continued(c, &root, &r, &format!("fresh-{id}-loss-{attempt}"));
            if id == "MN_GYM_1" && attempt < 3 {
                train_for_care(&mut c, &r, &root, 13 + attempt);
            }
        }
        assert!(
            event(&c, win),
            "{id}: bounded played retry must earn victory"
        );
        if northstar_earned_gyms::selected(&r) {
            northstar_earned_gyms::leave(&mut c, &r, 1, "MnRochester", (30, 24));
        } else {
            indoor_walk(&mut c, &r, (6, 18));
            travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
        }
        c = continued(c, &root, &r, id);
        assert!(event(&c, win));
    }
    let s = c.snapshot().expect("earned Care Badge");
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, false, false, false, false, false, false, false]
    );
    assert!(event(&c, "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"));
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_HEADBUTT"]),
        1
    );
    assert_eq!(s.party.slots.len(), 2);
    assert!(matches!(
        s.party.slots[0].pokemon.species.id.as_str(),
        "CYNDAQUIL" | "QUILAVA" | "KINDLIT" | "RIMEPELT"
    ));
    c = continued(c, &root, &r, "rochester-earned-care-badge");
    assert!(event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    at(&mut c, "MnRochester", (30, 24));
    (root, r, c)
}

#[test]
#[ignore = "requires connected Rochester via CRYSTAL_WINONA_TEST_PACK"]
fn rochester_earned_starter_route_trainers_first_badge_and_continue() {
    let _ = care_badge();
}
