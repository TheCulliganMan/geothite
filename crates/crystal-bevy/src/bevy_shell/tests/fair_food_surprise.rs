//! Disposable placement/party/DIV fixtures; purchases and exits use real input.
use crystal_core::input::GameButton;
use crystal_runtime::RuntimeBattleKind;

use super::{counter, cursor, fixture_options, flee, press, quantity};
use crate::bevy_shell::VisibleShellController;

fn pending(
    c: &VisibleShellController,
) -> Option<crystal_core::systems::fair_food::PendingFairFood> {
    crystal_core::systems::fair_food::PendingFairFood::read(c.shell.shell.session().state())
        .expect("valid saved food extension")
}

fn buy_receipt(c: &mut VisibleShellController, quantity_value: u16) {
    press(c, GameButton::A); // BUY
    cursor(c, 0, |c| {
        c.shell
            .menu_cursor
            .as_ref()
            .expect("food list")
            .option_index
    });
    press(c, GameButton::A); // quantity
    for _ in 1..quantity_value {
        press(c, GameButton::Up);
    }
    press(c, GameButton::A); // confirm prompt
    press(c, GameButton::A); // transaction
    assert!(c.shell.shop_notice.is_some(), "receipt owns input");
}

fn open_curds(c: &mut VisibleShellController, r: &crystal_runtime::CrystalRuntime) {
    counter(c, r, (67, 12));
    for _ in 0..64 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("curds vendor did not open");
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_surprise_native_purchase_pending_continue_and_flee() {
    let (root, r, mut c) = fixture_options(false, (67, 12), true, true);
    open_curds(&mut c, &r);
    let mut purchases = 0;
    for _ in 0..32 {
        let money = c.shell.shell.session().state().money;
        let before = quantity(&c, &r, "FAIR_CHEESE_CURDS");
        buy_receipt(&mut c, 2);
        purchases += 1;
        assert_eq!(quantity(&c, &r, "FAIR_CHEESE_CURDS"), before + 2);
        assert_eq!(c.shell.shell.session().state().money, money - 800);
        assert!(c.snapshot().expect("receipt").battle.is_none());
        if pending(&c).is_some() {
            break;
        }
        press(&mut c, GameButton::A); // receipt
        press(&mut c, GameButton::B); // inventory -> top
        for _ in 0..16 {
            if c.shell.shop_top_cursor.is_some() && c.shell.shop_notice.is_none() {
                break;
            }
            press(&mut c, GameButton::A);
        }
    }
    let saved_pending =
        pending(&c).expect("deterministic trace produces surprise within 32 purchases");
    let path = std::env::temp_dir().join(format!(
        "fair-food-pending-{}.crystalsave",
        std::process::id()
    ));
    c.save(&path).expect("save paid pending surprise");
    c = VisibleShellController::load_save(root, r.clone(), path.clone(), None)
        .expect("Continue paid surprise");
    std::fs::remove_file(path).expect("remove disposable save");
    assert_eq!(pending(&c), Some(saved_pending.clone()));
    for _ in 0..128 {
        if c.snapshot().expect("shop exit").battle.is_some() {
            break;
        }
        press(&mut c, GameButton::A);
    }
    let snapshot = c.snapshot().expect("food battle");
    assert!(snapshot.pending_shop.is_none(), "shop closes before battle");
    assert!(
        !c.shell
            .shell
            .session()
            .state()
            .script_runtime
            .text_window_open,
        "vendor dialogue closes before battle"
    );
    let battle = snapshot.battle.expect("normal catchable battle");
    assert!(matches!(battle.kind, RuntimeBattleKind::Wild { .. }));
    assert_eq!(battle.enemy_pokemon, saved_pending.battle.enemy_pokemon);
    assert_eq!(battle.enemy_pokemon.level, 10);
    assert!(pending(&c).is_none());
    let money = c.shell.shell.session().state().money;
    flee(&mut c, false);
    assert!(c.snapshot().expect("after RUN").battle.is_none());
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(quantity(&c, &r, "FAIR_CHEESE_CURDS"), purchases * 2);
    for _ in 0..6 {
        c.wait_frames(1).expect("idle repeated inputs");
    }
    assert!(c.snapshot().expect("no duplicate battle").battle.is_none());
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK"]
fn fair_food_surprise_native_cancel_confirmation_does_not_charge_or_roll() {
    let (_, r, mut c) = fixture_options(false, (67, 12), true, true);
    open_curds(&mut c, &r);
    let money = c.shell.shell.session().state().money;
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down); // NO
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_eq!(quantity(&c, &r, "FAIR_CHEESE_CURDS"), 0);
    assert!(pending(&c).is_none());
    assert!(
        c.snapshot()
            .expect("cancelled transaction")
            .battle
            .is_none()
    );
}
