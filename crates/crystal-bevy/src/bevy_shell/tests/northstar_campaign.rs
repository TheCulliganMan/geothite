//! Continue the earned Winona party through the story without fixture mutations.
use std::path::PathBuf;

use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::authored_route_fixture::indoor_walk;
use super::super::northstar_earned_gyms;
use super::super::prairie_story::{mankato_arrival, road, try_road_with};
use super::super::rochester::flee;
use super::super::rochester_earned::{checkout, try_battle};
use super::super::winona::{at as location, continue_in_place, finish, load, press, travel};
use super::{VisibleShellController, event, interact, mankato_leave, walk_to};

fn arrival() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_EARNED_SAVE") else {
        return mankato_arrival();
    };
    // Development continuation accepts a real exported checkpoint, never a
    // synthesized state or a save migrated to another pack identity. The default
    // course above earns this state from New Game in the same invocation.
    let (root, r) = load();
    let mut c =
        VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
            .expect("actual earned checkpoint with matching pack identity");
    c.set_runtime_journal_enabled(false);
    assert!(event(&c, "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"));
    assert!(event(&c, "EVENT_MN_GYM_1_DEFEATED"));
    assert!(event(&c, "EVENT_MN_PRAIRIE_NOTEBOOK"));
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, false, false, false, false, false, false, false]
    );
    assert!(
        c.snapshot()
            .expect("earned arrival")
            .overworld
            .map_name
            .starts_with("MnMankato")
    );
    assert_eq!(c.snapshot().expect("caught partner").party.slots.len(), 2);
    (root, r, c)
}

/// Follow the native R03 north connection into the current city footprint.
/// Repaired Minneapolis is larger than its original 48-tile fixture outline.
pub(super) fn enter_minneapolis(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let source = c.snapshot().expect("R03 departure").overworld;
    assert_eq!(source.map_name, "MnR03C00R00");
    assert_eq!(source.tile.y, 0);
    let connection = r.data().maps[&source.map_name]
        .attributes
        .connections
        .iter()
        .find(|link| link.direction == "north" && link.target_map == "MnMinneapolis")
        .expect("native Minneapolis connection");
    let city = &r.data().maps["MnMinneapolis"];
    let arrival = (
        i16::try_from(i32::from(source.tile.x) - connection.offset * 2).expect("native arrival x"),
        i16::try_from(city.attributes.height * 2 - 1).expect("native arrival y"),
    );
    travel(c, GameButton::Up, "MnMinneapolis", arrival);
    travel(
        c,
        GameButton::Up,
        "MnMinneapolis",
        (arrival.0, arrival.1 - 1),
    );
}

fn depart_prairie(c: &mut VisibleShellController, r: &CrystalRuntime) {
    walk_to(c, r, (24, 4));
    travel(c, GameButton::Up, "MnR03C00R01", (18, 15));
    super::river_cities::road(c, r, (20, 0));
    enter_minneapolis(c, r);
}

fn care(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("care approach").overworld.map_name == "MnMankato" {
        indoor_walk(c, r, (31, 14));
        travel(c, GameButton::Up, "MnMankatoPokecenter1F", (10, 13));
    }
    indoor_walk(c, r, (7, 5));
    press(c, GameButton::Up);
    press(c, GameButton::A);
    finish(c);
    assert!(
        c.snapshot()
            .expect("ordinary care")
            .party
            .slots
            .iter()
            .all(|s| {
                s.pokemon.hp == s.pokemon.max_hp
                    && s.pokemon.status.is_none()
                    && s.pokemon
                        .moves
                        .iter()
                        .all(|m| m.current_pp == r.data().moves[&m.name].pp)
            })
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .last_spawn_map_constant
            .as_deref(),
        Some("MN_MANKATO_POKECENTER_1F")
    );
    indoor_walk(c, r, (10, 14));
    travel(c, GameButton::Down, "MnMankato", (31, 14));
}

fn supplies(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let money = c.shell.shell.session().state().money;
    let id = if money >= u32::from(r.data().items["SUPER_POTION"].price) {
        "SUPER_POTION"
    } else {
        "POTION"
    };
    let item = &r.data().items[id];
    let owned = c.shell.shell.session().state().bag.quantity(item);
    let quantity =
        (money / u32::from(item.price)).min(u32::from(3_u16.saturating_sub(owned))) as u16;
    if quantity == 0 {
        return;
    }
    indoor_walk(c, r, (37, 28));
    travel(c, GameButton::Up, "MnMankatoMart", (6, 5));
    indoor_walk(c, r, (3, 3));
    press(c, GameButton::Left);
    press(c, GameButton::A);
    checkout(c, r, id, quantity);
    indoor_walk(c, r, (6, 6));
    travel(c, GameButton::Down, "MnMankato", (37, 28));
}

pub(super) fn learn_tm(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    item: &str,
    learned: &str,
    forgotten: &str,
) {
    learn_machine_on(c, r, item, learned, forgotten, 0);
}

/// Teach the actual carried machine through Pack and the selected party slot.
pub(super) fn learn_machine_on(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    item: &str,
    learned: &str,
    forgotten: &str,
    target_slot: usize,
) {
    let tm = &r.data().items[item];
    let remaining = u16::from(item.starts_with("HM_"));
    assert!(
        c.snapshot()
            .expect("legal native machine recipient")
            .party
            .slots[target_slot]
            .pokemon
            .species
            .tmhm_learnset
            .iter()
            .any(|m| m == learned)
    );
    if c.snapshot().expect("continued learned move").party.slots[target_slot]
        .pokemon
        .moves
        .iter()
        .any(|m| m.name == learned)
    {
        assert_eq!(
            c.shell.shell.session().state().bag.quantity(tm),
            remaining,
            "native machine consumption persists across Continue"
        );
        return;
    }
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    let target_move = c.snapshot().expect("known earned moves").party.slots[target_slot]
        .pokemon
        .moves
        .iter()
        .position(|m| m.name == forgotten)
        .or_else(|| {
            c.snapshot().expect("regional move replacement").party.slots[target_slot]
                .pokemon
                .moves
                .iter()
                .position(|m| r.data().moves[&m.name].power == 0)
        })
        .or_else(|| {
            c.snapshot()
                .expect("deliberate attack replacement")
                .party
                .slots[target_slot]
                .pokemon
                .moves
                .iter()
                .enumerate()
                .filter(|(_, m)| !matches!(m.name.as_str(), "CUT" | "SURF"))
                .min_by_key(|(_, m)| r.data().moves[&m.name].power)
                .map(|(i, _)| i)
        })
        .expect("deliberately selected move to replace");
    press(c, GameButton::Start);
    let entries = super::super::super::visible_start_menu_entries(&c.shell).expect("native Start");
    let target = entries
        .iter()
        .position(|s| s.trim_start_matches('>').trim() == "PACK")
        .expect("PACK");
    for _ in 0..16 {
        let index = c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
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
    for _ in 0..4 {
        if c.shell.tmhm_cursor.is_some() {
            break;
        }
        press(c, GameButton::Right);
    }
    assert!(c.shell.tmhm_cursor.is_some());
    let machine_index = c
        .snapshot()
        .expect("carried machine pocket")
        .bag
        .tm_hm
        .iter()
        .position(|m| m.item_id == item)
        .expect("actual earned machine");
    for _ in 0..128 {
        let index = c
            .shell
            .tmhm_cursor
            .as_ref()
            .expect("native machine cursor")
            .option_index;
        if index == machine_index {
            break;
        }
        press(
            c,
            if index < machine_index {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    for _ in 0..128 {
        if c.snapshot().expect("native machine knowledge").party.slots[target_slot]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == learned)
        {
            break;
        }
        // Read the native pages, select a compatible partner and choose a move.
        let button = if c.shell.pending_tmhm_text_stage.is_some() || c.shell.field_notice.is_some()
        {
            GameButton::A
        } else if c.shell.field_pack_target_mode
            == Some(super::super::super::FieldPackTargetMode::TmHmPokemon)
        {
            if c.shell.party_cursor < target_slot {
                GameButton::Down
            } else if c.shell.party_cursor > target_slot {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else if c.shell.tmhm_forget_menu_open
            && c.shell.pending_tmhm_text_stage.is_none()
            && c.shell.field_notice.is_none()
        {
            let index = c
                .shell
                .party_move_cursor
                .as_ref()
                .expect("native TM forget cursor")
                .option_index;
            if index < target_move {
                GameButton::Down
            } else if index > target_move {
                GameButton::Up
            } else {
                GameButton::A
            }
        } else {
            GameButton::A
        };
        press(c, button);
    }
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), remaining);
    assert!(
        c.snapshot().expect("native TM learning").party.slots[target_slot]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == learned)
    );
    for _ in 0..128 {
        if !super::super::super::has_visible_shell_direction_action(&mut c.shell) {
            break;
        }
        let text = c.shell.pending_tmhm_text_stage.is_some() || c.shell.field_notice.is_some();
        press(c, if text { GameButton::A } else { GameButton::B });
    }
    assert!(!super::super::super::has_visible_shell_direction_action(
        &mut c.shell
    ));
}

fn enter_road(c: &mut VisibleShellController, r: &CrystalRuntime) {
    indoor_walk(c, r, (46, 18));
    travel(c, GameButton::Right, "MnR02C00R00", (0, 30));
}

fn leave_road(c: &mut VisibleShellController, r: &CrystalRuntime) {
    if c.snapshot().expect("road outcome").overworld.map_name == "MnR02C00R00" {
        road(c, r, (0, 30));
        travel(c, GameButton::Left, "MnMankato", (47, 18));
        travel(c, GameButton::Left, "MnMankato", (46, 18));
    } else {
        location(c, "MnMankatoPokecenter1F", (10, 13));
    }
}

fn train(c: &mut VisibleShellController, r: &CrystalRuntime, root: &AssetRoot, level: u8) {
    for outing in 0..32 {
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
            let target = if step % 2 == 0 { (36, 18) } else { (39, 18) };
            let arrived = try_road_with(c, r, target, |c| {
                let s = c.snapshot().expect("training encounter");
                let b = s.battle.as_ref().expect("ordinary wild fight");
                if !fought && b.enemy_pokemon.level <= s.party.slots[0].pokemon.level + 1 {
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
        leave_road(c, r);
        continue_in_place(c, root, r, &format!("campaign-training-{level}-{outing}"));
    }
    assert!(
        c.snapshot().expect("bounded ordinary training").party.slots[0]
            .pokemon
            .level
            >= level
    );
}

#[test]
#[ignore = "live-RNG earned campaign; optional CRYSTAL_NORTHSTAR_EARNED_SAVE must be a genuine Mankato checkpoint"]
fn northstar_campaign_earned_party_wins_prairie_and_reaches_minneapolis() {
    let _ = prairie();
}

pub(super) fn prairie() -> (AssetRoot, CrystalRuntime, VisibleShellController) {
    if let Some(path) = std::env::var_os("CRYSTAL_NORTHSTAR_PRAIRIE_EXIT_SAVE") {
        let (root, r) = load();
        let mut c =
            VisibleShellController::load_save(root.clone(), r.clone(), PathBuf::from(path), None)
                .expect("actual same-pack earned Prairie victory checkpoint");
        c.set_runtime_journal_enabled(false);
        location(&mut c, "MnMankato", (10, 12));
        assert_eq!(
            c.shell.shell.session().state().badges.johto,
            [true, true, false, false, false, false, false, false]
        );
        for flag in [
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "EVENT_MN_GYM_1_DEFEATED",
            "EVENT_MN_REED_ROCHESTER_DEFEATED",
            "EVENT_MN_PRAIRIE_NOTEBOOK",
            "EVENT_MN_GYM_2_DEFEATED",
        ] {
            assert!(event(&c, flag), "played checkpoint prerequisite {flag}");
        }
        // This resumes a retained, genuinely played checkpoint. No state,
        // inventory, level, position, flags or RNG are rewritten after load.
        depart_prairie(&mut c, &r);
        continue_in_place(&mut c, &root, &r, "campaign-earned-minneapolis-arrival");
        return (root, r, c);
    }
    let (root, r, mut c) = arrival();
    care(&mut c, &r);
    learn_tm(&mut c, &r, "TM_HEADBUTT", "HEADBUTT", "TACKLE");
    for (tile, facing, id, flag) in [
        (
            (52, 21),
            GameButton::Left,
            "MN_R02_T01",
            "EVENT_MN_R02_TRAINER_01",
        ),
        (
            (24, 19),
            GameButton::Left,
            "MN_R02_T02",
            "EVENT_MN_R02_TRAINER_02",
        ),
        (
            (46, 37),
            GameButton::Right,
            "MN_R02_T03",
            "EVENT_MN_R02_TRAINER_03",
        ),
    ] {
        for attempt in 0..4 {
            if event(&c, flag) {
                break;
            }
            care(&mut c, &r);
            enter_road(&mut c, &r);
            road(&mut c, &r, tile);
            press(&mut c, facing);
            press(&mut c, GameButton::A);
            let won = try_battle(&mut c, &r, id, flag);
            leave_road(&mut c, &r);
            continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-{attempt}"));
            if won {
                break;
            }
        }
        assert!(event(&c, flag));
    }
    for (tile, id) in [
        ((4, 17), "MN_GYM_2_TRAINER_1"),
        ((11, 11), "MN_GYM_2_TRAINER_2"),
        ((11, 3), "MN_GYM_2"),
    ] {
        if id == "MN_GYM_2" {
            train(
                &mut c,
                &r,
                &root,
                if r.data().maps.contains_key("MnBoundaryIntake") {
                    16
                } else {
                    14
                },
            );
            assert!(matches!(
                c.snapshot().expect("native starter evolution").party.slots[0]
                    .pokemon
                    .species
                    .id
                    .as_str(),
                "QUILAVA" | "RIMEPELT"
            ));
        }
        let flag = format!("EVENT_{id}_DEFEATED");
        for attempt in 0..4 {
            care(&mut c, &r);
            supplies(&mut c, &r);
            indoor_walk(&mut c, &r, (10, 12));
            travel(&mut c, GameButton::Up, "MnMankatoGym", (6, 17));
            interact(&mut c, &r, tile);
            if try_battle(&mut c, &r, id, &flag) {
                if northstar_earned_gyms::selected(&r) {
                    northstar_earned_gyms::leave(&mut c, &r, 2, "MnMankato", (10, 12));
                } else {
                    mankato_leave(&mut c, &r, (6, 19), (10, 12));
                }
                continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-win"));
                break;
            }
            location(&mut c, "MnMankatoPokecenter1F", (10, 13));
            assert!(
                !c.shell.shell.session().state().badges.johto[1],
                "a loss cannot award Prairie"
            );
            continue_in_place(&mut c, &root, &r, &format!("campaign-{id}-loss-{attempt}"));
            train(&mut c, &r, &root, 14 + attempt);
        }
        assert!(event(&c, &flag));
    }
    assert_eq!(
        c.shell.shell.session().state().badges.johto,
        [true, true, false, false, false, false, false, false]
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&r.data().items["TM_MUD_SLAP"]),
        1
    );
    depart_prairie(&mut c, &r);
    continue_in_place(&mut c, &root, &r, "campaign-earned-minneapolis-arrival");
    assert!(!event(&c, "EVENT_MN_ICE_ENFORCER_DEFEATED"));
    (root, r, c)
}
