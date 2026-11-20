//! Follow the same Coin Case from the greeter to a prize and through Continue.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, B, Down, Right, Start, Up};
use crystal_core::models::Dv;
use crystal_runtime::CrystalRuntime;

use super::super::brainerd_guide::capture;
use super::super::interior_life::{fixture, load, talk, walk};
use super::super::winona::{at, continue_in_place, finish, press, travel};
use super::ROOM;
use crate::bevy_shell::{
    FieldPackAction, FieldPackPocket, FieldPackTargetMode, StartMenuOption, VisibleShellController,
    mark_runtime_snapshot_dirty, selected_vertical_menu, visible_selected_pack_item_actions,
    visible_start_menu_options,
};

fn choose(c: &mut VisibleShellController, cursor: usize, target: usize) {
    for _ in 0..cursor.abs_diff(target) {
        press(c, if cursor < target { Down } else { Up });
    }
    press(c, A);
}

fn open_pocket(c: &mut VisibleShellController, pocket: &FieldPackPocket) {
    press(c, Start);
    let s = c.snapshot().expect("Start menu");
    let pack = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("Pack");
    let cursor = c
        .shell
        .start_menu_cursor
        .as_ref()
        .expect("Start menu cursor")
        .option_index;
    choose(c, cursor, pack);
    for _ in 0..4 {
        if c.shell.field_pack_pocket.as_ref() == Some(pocket) {
            break;
        }
        press(c, Right);
    }
    assert_eq!(c.shell.field_pack_pocket.as_ref(), Some(pocket));
}

fn pack_action(
    c: &mut VisibleShellController,
    pocket: FieldPackPocket,
    id: &str,
    action: FieldPackAction,
) {
    open_pocket(c, &pocket);
    let bag = c.snapshot().expect("selected Pack pocket").bag;
    let (items, cursor) = match pocket {
        FieldPackPocket::Items => (&bag.items, &c.shell.bag_cursor),
        FieldPackPocket::KeyItems => (&bag.key_items, &c.shell.key_item_cursor),
        other => panic!("unexpected pocket {other:?}"),
    };
    let row = items
        .iter()
        .position(|item| item.item_id == id && item.quantity == 1)
        .expect("one earned item in the Pack");
    let cursor = cursor.as_ref().expect("item cursor").option_index;
    choose(c, cursor, row);
    let s = c.snapshot().expect("Pack actions");
    let row = visible_selected_pack_item_actions(&s, &c.shell, &pocket, false)
        .expect("native actions")
        .iter()
        .position(|a| *a == action)
        .expect("usable Pack action");
    let cursor = c
        .shell
        .field_pack_action_cursor
        .as_ref()
        .expect("action cursor")
        .option_index;
    choose(c, cursor, row);
}

fn open_case(c: &mut VisibleShellController, balance: u16) {
    pack_action(
        c,
        FieldPackPocket::KeyItems,
        "COIN_CASE",
        FieldPackAction::Use,
    );
    assert_eq!(c.shell.field_notice, Some(format!("Coins:\n{balance}")));
    assert_eq!(c.snapshot().expect("wallet").trainer.coins, balance);
}

fn held_claw(c: &VisibleShellController) {
    let partner = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("original partner");
    assert_eq!(partner.species.id, "POUCHID");
    assert_eq!(partner.item.as_deref(), Some("QUICK_CLAW"));
}

fn buy(c: &mut VisibleShellController, runtime: &CrystalRuntime, clerk_x: i16, offer: &str) {
    walk(c, runtime, (clerk_x, 4));
    press(c, Up);
    press(c, A);
    for _ in 0..20 {
        let s = c.snapshot().expect("counter dialogue or menu");
        if let Some(menu) = s.ui.menu.filter(|m| m.window_open)
            && let Ok(choices) = selected_vertical_menu(&menu, &c.shell.menu_cursor)
        {
            let index = choices
                .options
                .iter()
                .position(|row| row.trim_start_matches(['>', ' ']) == offer)
                .unwrap_or_else(|| {
                    panic!("missing {offer} on the counter menu: {:?}", choices.options)
                });
            for _ in 0..index {
                press(c, Down);
            }
            press(c, A);
            for _ in 0..16 {
                let s = c.snapshot().expect("confirmation page");
                if s.ui.pending_yes_no.is_some() {
                    // The prompt appears before the last confirmation page is read.
                    // A movable cursor proves the printer released the choice.
                    press(c, Down);
                    if c.shell
                        .yes_no_cursor
                        .as_ref()
                        .expect("YES/NO cursor")
                        .option_index
                        == 1
                    {
                        press(c, Up);
                        press(c, A);
                        finish(c);
                        return;
                    }
                }
                assert!(
                    s.ui.text_window_open,
                    "missing purchase confirmation for {offer}"
                );
                press(c, A);
            }
            panic!("the clerk never offered to confirm {offer}");
        }
        press(c, A);
    }
    panic!("the counter did not open its menu for {offer}");
}

#[test]
#[ignore = "requires the Minnesota Game Corner in CRYSTAL_INTERIOR_PACK"]
fn earned_coin_case_shows_prize_debit_in_pack_and_after_continue() {
    let (root, runtime, _) = load();
    // Supply a public starting location and the price of one stack of 500 coins.
    // The regional partner starts without a held item. Coins, case and prize are all earned.
    let city = "MnMinneapolis";
    let mut c = fixture(&root, &runtime, city, (56, 52));
    let mut partner = runtime
        .data()
        .create_pokemon("POUCHID", 15, Dv::default())
        .expect("regional Pouchid");
    assert!(partner.item.is_none());
    let state = c.shell.shell.session_mut().state_mut();
    state.money = 10_000;
    partner.original_trainer_name.clone_from(&state.player_name);
    partner.original_trainer_id = state.player_id;
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    assert_eq!(c.snapshot().expect("outside arcade").trainer.coins, 0);
    travel(&mut c, Up, ROOM, (3, 12));
    walk(&mut c, &runtime, (3, 9));
    let welcome = talk(&mut c, (2, 9));
    assert!(welcome.contains("MnCornerWelcomeText"));
    open_case(&mut c, 20);
    press(&mut c, A);
    buy(&mut c, &runtime, 3, "500 COINS 10000");
    assert_eq!(c.shell.shell.session().state().money, 0);
    open_case(&mut c, 520);
    press(&mut c, A);
    buy(&mut c, &runtime, 16, "QUICK CLAW 300");
    let claw = &runtime.data().items["QUICK_CLAW"];
    assert_eq!(c.shell.shell.session().state().bag.quantity(claw), 1);
    open_case(&mut c, 220);
    let captures = std::env::var_os("MN_GAME_CORNER_WALLET_DIR").map(PathBuf::from);
    c = capture(c, captures.as_deref(), "earned-coin-case-after-prize.png");
    press(&mut c, A);
    pack_action(
        &mut c,
        FieldPackPocket::Items,
        "QUICK_CLAW",
        FieldPackAction::Give,
    );
    assert_eq!(c.shell.party_cursor, 0);
    press(&mut c, A);
    for _ in 0..8 {
        press(&mut c, B);
    }
    held_claw(&c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(claw), 0);
    walk(&mut c, &runtime, (3, 12));
    travel(&mut c, Down, city, (56, 52));
    continue_in_place(&mut c, &root, &runtime, "earned-coin-case-and-quick-claw");
    at(&mut c, city, (56, 52));
    held_claw(&c);
    assert_eq!(c.shell.shell.session().state().bag.quantity(claw), 0);
    open_case(&mut c, 220);
    press(&mut c, A);
    travel(&mut c, Up, ROOM, (3, 12));
    walk(&mut c, &runtime, (3, 9));
    assert!(talk(&mut c, (2, 9)).contains("MnCornerRulesText"));
    assert_eq!(c.snapshot().expect("repeat greeting").trainer.coins, 220);
}

#[test]
#[ignore = "requires the Minnesota Game Corner in CRYSTAL_INTERIOR_PACK"]
fn prize_tm_refuses_pouchid_teaches_kindlit_and_can_be_bought_again_after_continue() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, ROOM, (3, 12));
    let data = runtime.data();
    let tm = &data.items["TM_FIRE_BLAST"];
    // An established player's wallet and two ordinary regional partners.
    // Both copies of the TM must be bought with controller input.
    let state = c.shell.shell.session_mut().state_mut();
    state.coins = 9_999;
    state.money = 21_000;
    assert!(
        state
            .bag
            .add_item(&data.items["COIN_CASE"], 1)
            .expect("existing Coin Case")
    );
    for species in ["POUCHID", "KINDLIT"] {
        let partner = data
            .create_pokemon(species, 5, Dv::default())
            .expect("regional partner");
        assert!(!partner.moves.iter().any(|m| m.name == "FIRE_BLAST"));
        assert!(partner.moves.len() < 4);
        assert!(state.storage.party.add_pokemon(partner));
    }
    state.sync_party_from_storage();
    mark_runtime_snapshot_dirty(&mut c.shell);
    buy(&mut c, &runtime, 18, "FIRE BLAST 5500");
    assert_eq!(c.snapshot().expect("prize debit").trainer.coins, 4_499);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);

    open_pocket(&mut c, &FieldPackPocket::TmHm);
    let list = c.snapshot().expect("TM pocket").bag.tm_hm;
    let slot = c
        .shell
        .tmhm_cursor
        .as_ref()
        .expect("machine cursor")
        .option_index;
    assert_eq!(list[slot].item_id, "TM_FIRE_BLAST");
    for _ in 0..24 {
        if c.shell.field_pack_target_mode == Some(FieldPackTargetMode::TmHmPokemon)
            && c.shell.field_notice.is_none()
            && c.shell.pending_tmhm_text_stage.is_none()
        {
            break;
        }
        press(&mut c, A);
    }
    assert_eq!(
        c.shell.field_pack_target_mode,
        Some(FieldPackTargetMode::TmHmPokemon)
    );
    assert_eq!(c.shell.party_cursor, 0);
    press(&mut c, A);
    assert!(
        c.shell.field_notice.is_some(),
        "show the incompatible partner message"
    );
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 1);
    for _ in 0..12 {
        if c.shell.field_notice.is_none() {
            break;
        }
        press(&mut c, A);
    }
    assert_eq!(
        c.shell.field_pack_target_mode,
        Some(FieldPackTargetMode::TmHmPokemon)
    );
    press(&mut c, Down);
    assert_eq!(c.shell.party_cursor, 1);
    press(&mut c, A);
    for _ in 0..40 {
        if !crate::bevy_shell::has_visible_shell_direction_action(&mut c.shell) {
            break;
        }
        let text = c.shell.pending_tmhm_text_stage.is_some() || c.shell.field_notice.is_some();
        press(&mut c, if text { A } else { B });
    }
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 0);
    continue_in_place(&mut c, &root, &runtime, "game-corner-tm-learned");
    let s = c.snapshot().expect("continued partners and wallet");
    assert_eq!(s.trainer.coins, 4_499);
    assert_eq!(c.shell.shell.session().state().bag.quantity(tm), 0);
    assert!(
        !s.party.slots[0]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "FIRE_BLAST")
    );
    assert!(
        s.party.slots[1]
            .pokemon
            .moves
            .iter()
            .any(|m| m.name == "FIRE_BLAST")
    );
    for offer in ["500 COINS 10000", "500 COINS 10000", "50 COINS 1000"] {
        buy(&mut c, &runtime, 3, offer);
    }
    buy(&mut c, &runtime, 18, "FIRE BLAST 5500");
    let state = c.shell.shell.session().state();
    assert_eq!(
        (state.money, state.coins, state.bag.quantity(tm)),
        (0, 49, 1)
    );
}
