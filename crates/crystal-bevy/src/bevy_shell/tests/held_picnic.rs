//! Feed a Route 11 bakery purchase to an eel-shaped Minnesota partner through its held slot.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, B, Down, Right, Start, Up};
use crystal_core::models::{Dv, LearnedMove};

use super::interior_life::{fixture, load, walk};
use super::minnesota_fossils::{finish, talk};
use super::winona::{at, continue_in_place, press, travel};
use super::{menu_render_test_app, render_live_battle_canvas_for_test};
use crate::bevy_shell::{
    BevyRuntimeShell, FieldPackAction, FieldPackPocket, PartyAction, StartMenuOption,
    VisibleShellController, mark_runtime_snapshot_dirty, visible_party_actions,
    visible_selected_pack_item_actions, visible_start_menu_options,
};

const PIE: &str = "MN_BLUEBERRY_SLICE";
const ROAD: &str = "MnR11C00R01";

fn choose(
    c: &mut VisibleShellController,
    target: usize,
    index: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..16 {
        let selected = index(c);
        if selected == target {
            press(c, A);
            return;
        }
        press(c, if selected < target { Down } else { Up });
    }
    panic!("could not reach menu option {target}");
}

fn open_start(c: &mut VisibleShellController, option: StartMenuOption) {
    press(c, Start);
    let s = c.snapshot().expect("Start menu");
    let index = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == option)
        .expect("menu option");
    choose(c, index, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
}

fn close_menus(c: &mut VisibleShellController) {
    for _ in 0..32 {
        if c.shell.field_notice.is_some() {
            press(c, A);
        } else if c.shell.party_menu_open
            || c.shell.field_pack_pocket.is_some()
            || c.shell.start_menu_cursor.is_some()
        {
            press(c, B);
        } else {
            return;
        }
    }
    panic!("give/take menu did not close");
}

fn give_last_slice(c: &mut VisibleShellController) {
    open_start(c, StartMenuOption::Pack);
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Items) {
            break;
        }
        press(c, Right);
    }
    let s = c.snapshot().expect("Pack with bought slice");
    assert_eq!(s.bag.items[0].item_id, PIE);
    press(c, A);
    let s = c.snapshot().expect("usable and holdable slice");
    let actions = visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
        .expect("Pack actions");
    let index = actions
        .iter()
        .position(|a| *a == FieldPackAction::Give)
        .expect("Give");
    choose(c, index, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("item cursor")
            .option_index
    });
    press(c, A);
    close_menus(c);
}

fn carried(c: &VisibleShellController) -> (u16, Option<&str>) {
    let state = c.shell.shell.session().state();
    let partner = state.storage.party.pokemon[0]
        .as_ref()
        .expect("owned Keypout");
    (
        state
            .bag
            .quantity(&c.shell.shell.runtime().data().items[PIE]),
        partner.item.as_deref(),
    )
}

fn capture_recovery(
    c: VisibleShellController,
    directory: &std::path::Path,
) -> VisibleShellController {
    let mut app = menu_render_test_app(c.shell);
    app.update();
    app.update();
    std::fs::create_dir_all(directory).expect("native food capture directory");
    render_live_battle_canvas_for_test(app.world_mut(), "held pie recovery")
        .save(directory.join("keypout-held-pie.png"))
        .expect("native recovery frame");
    VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("continue after native frame"),
    }
}

fn at_battle_menu(c: &VisibleShellController) -> bool {
    c.shell.battle_action_cursor.is_some()
        && c.shell.battle_move_cursor.is_none()
        && c.shell.battle_messages.is_empty()
        && c.shell.visible_battle_transition.is_none()
        && c.shell.visible_battle_sliding_intro.is_none()
        && c.shell.visible_send_out_animation.is_none()
}

#[test]
#[ignore = "requires the held food edition in CRYSTAL_INTERIOR_PACK"]
fn held_pie_can_be_given_taken_saved_and_eaten_by_keypout_in_battle() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, ROAD, (31, 36));
    let mut partner = runtime
        .data()
        .create_pokemon("KEYPOUT", 85, Dv::default())
        .expect("local burbot");
    let moves = &runtime.data().pokemon["KEYPOUT"].tmhm_learnset;
    assert!(
        ["PROTECT", "SURF"]
            .iter()
            .all(|m| moves.iter().any(|id| id == m))
    );
    partner.moves = ["PROTECT", "SURF"]
        .into_iter()
        .map(|name| LearnedMove {
            name: name.into(),
            current_pp: runtime.data().moves[name].pp,
            pp_ups: 0,
        })
        .collect();
    partner.hp = 15;
    assert!(65 < partner.max_hp / 2);
    let mut reserve = runtime
        .data()
        .create_pokemon("LATCHUSK", 100, Dv::default())
        .expect("regional evolved burbot");
    assert!(
        runtime.data().pokemon["LATCHUSK"]
            .tmhm_learnset
            .iter()
            .any(|m| m == "SURF")
    );
    reserve.moves = vec![partner.moves[1].clone()];
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    assert!(state.storage.party.add_pokemon(reserve));
    state.sync_party_from_storage();
    state.money = 350;
    mark_runtime_snapshot_dirty(&mut c.shell);

    travel(&mut c, Up, "MnNorthShorePieShop", (3, 6));
    walk(&mut c, &runtime, (5, 3));
    talk(&mut c, Right, &[true]);
    assert_eq!(carried(&c), (1, None));
    assert_eq!(c.snapshot().expect("paid for pie").trainer.money, 0);
    give_last_slice(&mut c);
    assert_eq!(carried(&c), (0, Some(PIE)));
    open_start(&mut c, StartMenuOption::Pokemon);
    press(&mut c, A);
    let s = c.snapshot().expect("Keypout's party menu");
    let item = visible_party_actions(&s, &c.shell)
        .expect("party actions")
        .iter()
        .position(|a| *a == PartyAction::Item)
        .expect("ITEM");
    choose(&mut c, item, |c| {
        c.shell
            .party_action_cursor
            .as_ref()
            .expect("party action")
            .option_index
    });
    choose(&mut c, 1, |c| {
        c.shell
            .party_give_take_cursor
            .as_ref()
            .expect("GIVE/TAKE")
            .option_index
    });
    close_menus(&mut c);
    assert_eq!(carried(&c), (1, None));
    give_last_slice(&mut c);
    walk(&mut c, &runtime, (2, 7));
    at(&mut c, ROAD, (31, 36));
    continue_in_place(&mut c, &root, &runtime, "keypout-holding-pie");
    assert_eq!(carried(&c), (0, Some(PIE)));

    travel(&mut c, Down, ROAD, (31, 37));
    travel(&mut c, Right, ROAD, (37, 37));
    travel(&mut c, Up, ROAD, (37, 35));
    for _ in 0..128 {
        if at_battle_menu(&c) {
            break;
        }
        press(&mut c, A);
    }
    assert!(at_battle_menu(&c));
    let s = c.snapshot().expect("Bird Keeper Ivo fight");
    assert!(
        matches!(s.battle.expect("on-sight trainer").kind, crystal_runtime::RuntimeBattleKind::Trainer { ref trainer_id, .. } if trainer_id == "MN_R11_T06")
    );
    assert_eq!(s.party.slots[0].pokemon.hp, 15);
    press(&mut c, A);
    assert_eq!(
        c.shell
            .battle_move_cursor
            .as_ref()
            .expect("fight")
            .option_index,
        0
    );
    press(&mut c, A); // Protect isolates the 50-point recovery from incoming damage.
    let mut servings = 0;
    let output = std::env::var_os("MN_FOOD_FRAMES").map(PathBuf::from);
    for _ in 0..160 {
        if at_battle_menu(&c) {
            break;
        }
        if let Some(line) = c.shell.battle_messages.front()
            && line.contains("PIE SLICE")
        {
            assert_eq!(line, "KEYPOUT healed\nwith PIE SLICE!");
            assert_eq!(
                crate::bevy_shell::visible_battle_message_text(&c.shell, line),
                "KEYPOUT healed\nwith PIE SLICE!",
                "both the partner and the food fit the current native text page"
            );
            if servings == 0
                && let Some(dir) = output.as_deref()
            {
                c = capture_recovery(c, dir);
            }
            servings += 1;
        }
        press(&mut c, A);
    }
    assert!(at_battle_menu(&c) && servings >= 1);
    assert_eq!(
        c.snapshot().expect("consumed slice").party.slots[0]
            .pokemon
            .hp,
        65
    );
    assert_eq!(carried(&c), (0, None));

    // Another turn starts below half. The empty slot cannot give a second heal.
    press(&mut c, A);
    press(&mut c, A);
    for _ in 0..160 {
        if at_battle_menu(&c) {
            break;
        }
        assert!(
            !c.shell
                .battle_messages
                .front()
                .is_some_and(|line| line.contains("PIE SLICE"))
        );
        press(&mut c, A);
    }
    assert!(at_battle_menu(&c));
    assert!(c.snapshot().expect("next turn").party.slots[0].pokemon.hp <= 65);
    assert_eq!(carried(&c), (0, None));
    // Give the injured holder a rest. Its replacement finishes the trainer
    // battle through the same party and FIGHT menus the player sees.
    press(&mut c, Right);
    press(&mut c, A);
    choose(&mut c, 1, |c| {
        c.shell
            .battle_switch_cursor
            .as_ref()
            .expect("choose Latchusk")
            .option_index
    });
    press(&mut c, A);
    let mut used_surf = false;
    for _ in 0..256 {
        let s = c.snapshot().expect("finish Ivo's battle");
        let Some(battle) = s.battle else { break };
        let button = if !c.shell.battle_messages.is_empty() {
            assert!(
                !c.shell
                    .battle_messages
                    .front()
                    .is_some_and(|line| line.contains("PIE SLICE"))
            );
            A
        } else if c.shell.battle_shift_prompt_cursor.is_some() {
            B
        } else if let Some(cursor) = &c.shell.battle_move_cursor {
            assert_eq!(battle.active_player_party_index, Some(1));
            assert_eq!(battle.player_moves[cursor.option_index].name, "SURF");
            used_surf = true;
            A
        } else if at_battle_menu(&c)
            && c.shell
                .battle_action_cursor
                .as_ref()
                .expect("fight menu")
                .option_index
                == 1
        {
            crystal_core::input::GameButton::Left
        } else {
            A
        };
        press(&mut c, button);
    }
    assert!(used_surf && c.snapshot().expect("battle complete").battle.is_none());
    finish(&mut c);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_R11_TRAINER_06")
            .expect("Ivo victory")
    );
    continue_in_place(&mut c, &root, &runtime, "keypout-finished-pie");
    assert_eq!(carried(&c), (0, None));
}
