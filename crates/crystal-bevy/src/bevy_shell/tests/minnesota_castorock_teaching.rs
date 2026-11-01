//! Native PACK teaching for the declared, compatible cave-access fixture.
use crystal_core::input::GameButton;

use super::super::super::{
    VisibleShellController, has_visible_shell_direction_action, visible_start_menu_entries,
};
use super::super::winona::press;

pub(super) fn teach_strength(c: &mut VisibleShellController) {
    assert!(!knows_strength(c));
    press(c, GameButton::Start);
    for _ in 0..8 {
        if visible_start_menu_entries(&c.shell)
            .expect("production Start menu")
            .iter()
            .any(|entry| entry == ">PACK")
        {
            break;
        }
        press(c, GameButton::Down);
    }
    assert!(
        visible_start_menu_entries(&c.shell)
            .expect("PACK selected")
            .iter()
            .any(|entry| entry == ">PACK")
    );
    press(c, GameButton::A);
    for _ in 0..4 {
        if c.shell.tmhm_cursor.is_some() {
            break;
        }
        press(c, GameButton::Right);
    }
    let target = c
        .snapshot()
        .expect("machine pocket")
        .bag
        .tm_hm
        .iter()
        .position(|item| item.item_id == "HM_STRENGTH")
        .expect("declared carried HM");
    for _ in 0..64 {
        let selected = c
            .shell
            .tmhm_cursor
            .as_ref()
            .expect("native HM cursor")
            .option_index;
        if selected == target {
            break;
        }
        press(
            c,
            if selected < target {
                GameButton::Down
            } else {
                GameButton::Up
            },
        );
    }
    assert_eq!(
        c.shell
            .tmhm_cursor
            .as_ref()
            .expect("selected HM")
            .option_index,
        target
    );
    // There is one partner with an open move slot. Every confirmation and the
    // party selection is an ordinary A press through the native teaching flow.
    for _ in 0..128 {
        if knows_strength(c) {
            break;
        }
        press(c, GameButton::A);
    }
    assert!(
        knows_strength(c),
        "native HM teaching must add the real move"
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&c.shell.shell.runtime().data().items["HM_STRENGTH"]),
        1,
        "HM remains reusable"
    );
    for _ in 0..128 {
        if !has_visible_shell_direction_action(&mut c.shell) {
            return;
        }
        let text = c.shell.pending_tmhm_text_stage.is_some() || c.shell.field_notice.is_some();
        press(c, if text { GameButton::A } else { GameButton::B });
    }
    panic!("teaching did not return control to the cave");
}

fn knows_strength(c: &mut VisibleShellController) -> bool {
    c.snapshot().expect("actual party moves").party.slots[0]
        .pokemon
        .moves
        .iter()
        .any(|m| m.name == "STRENGTH")
}
