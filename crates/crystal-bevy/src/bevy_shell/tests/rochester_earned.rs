//! New Game to the Care Badge using earned partners and production input only.
use crystal_core::input::GameButton;
use crystal_core::models::pokemon::Stat;
use crystal_runtime::{CrystalRuntime, RuntimeBattleKind};

use super::super::{VisibleShellController, carried_battle_non_ball_item_ids};
use super::authored_route_fixture::indoor_walk;
use super::rochester::{delivery, flee, trail, trail_with};
use super::winona::{at, continued, finish, press, travel};

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
    let initial = c.snapshot().expect("before earned battle");
    let mut saw_battle = false;
    let mut spent_pp = false;
    let mut traced_turn = None;
    let capture = id == "WILD" && initial.party.slots.len() < 2;
    let initial_balls = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&r.data().items["POKE_BALL"]);
    for _ in 0..1024 {
        let s = c.snapshot().expect("earned battle");
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
            let potions = c
                .shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POTION"]);
            if id == "MN_GYM_1" && traced_turn != Some(b.player_turns_taken) {
                traced_turn = Some(b.player_turns_taken);
                eprintln!(
                    "leader turn {} {:?}: player {} enemy {} stage {:?}, potions {}",
                    b.player_turns_taken,
                    b.enemy_pokemon.species.id,
                    s.party.slots[b.active_player_party_index.unwrap_or(0)]
                        .pokemon
                        .hp,
                    b.enemy_pokemon.hp,
                    b.enemy_pokemon.stat_boosts,
                    potions
                );
            }
            if id == "WILD" {
                assert!(matches!(b.kind, RuntimeBattleKind::Wild { .. }));
            } else {
                assert!(
                    matches!(&b.kind, RuntimeBattleKind::Trainer { trainer_id, .. } if trainer_id == id)
                );
            }
            spent_pp |= s.party.slots.iter().any(|slot| {
                initial.party.slots.iter().any(|old| {
                    old.index == slot.index
                        && slot.pokemon.moves.iter().any(|m| {
                            old.pokemon
                                .moves
                                .iter()
                                .any(|om| om.name == m.name && m.current_pp < om.current_pp)
                        })
                })
            });
        } else if saw_battle {
            break;
        }
        let b = s.battle.as_ref();
        let active = b.and_then(|b| b.active_player_party_index).unwrap_or(0);
        let p = &s.party.slots[active].pokemon;
        let heal = p.hp > 0
            && p.hp * 2 < p.max_hp
            && c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POTION"])
                > 0;
        let catch = !heal
            && capture
            && b.is_some_and(|b| b.enemy_pokemon.hp * 2 <= b.enemy_pokemon.max_hp)
            && c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POKE_BALL"])
                > 0;
        let change_prompt = s.ui.menu.as_ref().is_some_and(|m| {
            m.layout
                .vertical_menus
                .iter()
                .flat_map(|m| &m.options)
                .any(|o| o.contains("change <PKMN>"))
        });
        let button = if s.ui.menu.is_none() && s.ui.text.is_some() {
            GameButton::A
        } else if change_prompt {
            GameButton::B
        } else if c.shell.battle_pack_target_mode.is_some() {
            if c.shell.party_cursor < active {
                GameButton::Down
            } else if c.shell.party_cursor > active {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_switch_cursor {
            let target = s
                .party
                .slots
                .iter()
                .position(|s| s.pokemon.hp > 0 && s.index != active)
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
                    .position(|id| id == "POTION")
                    .expect("purchased Potion");
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
                id.starts_with("MN_GYM_")
                    && m.current_pp > 0
                    && ((m.name == "SMOKESCREEN"
                        && b.enemy_pokemon.stat_boosts[&Stat::Accuracy]
                            > if id == "MN_GYM_1" { -6 } else { -3 })
                        || (m.name == "LEER"
                            && b.enemy_pokemon.stat_boosts[&Stat::Accuracy]
                                <= if id == "MN_GYM_1" { -6 } else { -3 }
                            && b.enemy_pokemon.stat_boosts[&Stat::Defense]
                                > if id == "MN_GYM_1" { -3 } else { -1 }))
            });
            let target = support.unwrap_or_else(|| {
                b.player_moves
                    .iter()
                    .enumerate()
                    .filter(|(_, m)| m.current_pp > 0 && r.data().moves[&m.name].power > 0)
                    .max_by_key(|(_, m)| r.data().moves[&m.name].power)
                    .map(|(i, _)| i)
                    .expect("earned damaging move with PP")
            });
            if cursor.option_index < target {
                GameButton::Down
            } else if cursor.option_index > target {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if let Some(cursor) = &c.shell.battle_action_cursor {
            let target = if heal || catch { 2 } else { 0 };
            if cursor.option_index / 2 < target / 2 {
                GameButton::Down
            } else if cursor.option_index / 2 > target / 2 {
                GameButton::Up
            } else if cursor.option_index % 2 > target % 2 {
                GameButton::Left
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert!(
        saw_battle && spent_pp,
        "{id}: actual battle and PP spending"
    );
    finish(c);
    let after = c.snapshot().expect("earned outcome");
    assert!(after.battle.is_none());
    if id == "WILD" {
        assert_eq!(
            after.overworld.map_name, initial.overworld.map_name,
            "wild loss cannot pass for progress"
        );
    } else {
        assert!(
            event(c, win),
            "{id}: actual trainer victory required; result={}, map={}, party={:?}, trainer_result={:?}, ui={:?}",
            c.shell.shell.session().state().battle_result,
            after.overworld.map_name,
            after.party,
            c.shell.battle_trainer_result,
            after.ui
        );
    }
    if capture && after.party.slots.len() > initial.party.slots.len() {
        assert_eq!(
            after.party.slots.len(),
            initial.party.slots.len() + 1,
            "native capture adds one partner"
        );
        assert!(
            c.shell
                .shell
                .session()
                .state()
                .bag
                .quantity(&r.data().items["POKE_BALL"])
                < initial_balls
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
}

fn heal(c: &mut VisibleShellController, r: &CrystalRuntime) {
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

fn clinic_exit(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (10, 14));
    travel(c, GameButton::Down, "MnRochester", (15, 8));
}

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime, quantity: u16) {
    indoor_walk(c, r, (29, 10));
    travel(c, GameButton::Up, "MnRochesterMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
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
    press(c, GameButton::A);
    press(c, GameButton::Down);
    press(c, GameButton::A);
    assert_eq!(
        c.shell
            .shop_quantity
            .as_ref()
            .expect("purchase quantity")
            .item_id,
        "POTION"
    );
    for _ in 1..quantity {
        press(c, GameButton::Up);
    }
    press(c, GameButton::A);
    press(c, GameButton::A);
    let item = &r.data().items["POTION"];
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
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, "MnRochester", (29, 10));
}

fn recruit(c: &mut VisibleShellController, r: &CrystalRuntime) {
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
        trail_with(c, r, tile, |c| {
            let s = c.snapshot().expect("visible encounter");
            let b = s.battle.as_ref().expect("wild partner");
            if s.party.slots.len() < 2
                && b.enemy_pokemon.level >= 8
                && matches!(
                    b.enemy_pokemon.species.id.as_str(),
                    "RATTATA" | "SENTRET" | "HOOTHOOT"
                )
            {
                battle(c, r, "WILD", "");
            } else {
                flee(c);
            }
        });
    }
    assert_eq!(
        c.snapshot().expect("native recruitment").party.slots.len(),
        2
    );
    trail(c, r, (0, 24));
    travel(c, GameButton::Left, "MnRochester", (39, 18));
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
            recruit(&mut c, &r);
        }
        indoor_walk(&mut c, &r, (15, 8));
        travel(&mut c, GameButton::Up, "MnRochesterPokecenter1F", (10, 13));
        heal(&mut c, &r);
        clinic_exit(&mut c, &r);
        indoor_walk(&mut c, &r, (30, 24));
        travel(&mut c, GameButton::Up, "MnRochesterGym", (6, 17));
        indoor_walk(&mut c, &r, tile);
        press(&mut c, facing);
        press(&mut c, GameButton::A);
        battle(&mut c, &r, id, win);
        indoor_walk(&mut c, &r, (6, 18));
        travel(&mut c, GameButton::Down, "MnRochester", (30, 24));
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
    assert_eq!(s.party.slots[0].pokemon.species.id, "CYNDAQUIL");
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
