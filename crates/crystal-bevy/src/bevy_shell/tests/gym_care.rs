//! Shared native Pack care for explicit local gym course fixtures.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, visible_selected_pack_item_actions,
    visible_start_menu_options,
};
use super::{VisibleShellController, at, press};

fn cursor(
    c: &mut VisibleShellController,
    target: usize,
    index: fn(&VisibleShellController) -> usize,
) {
    for _ in 0..32 {
        let selected = index(c);
        if selected == target {
            return;
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
    panic!("native item cursor did not reach its target");
}

pub(super) fn heal(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let stand = at(c);
    let slots = c
        .snapshot()
        .expect("party for native care")
        .party
        .slots
        .len();
    for slot in 0..slots {
        let before = c.snapshot().expect("between-battle care").party.slots[slot]
            .pokemon
            .clone();
        assert!(
            before.hp > 0,
            "Hyper Potion cannot revive a fainted partner"
        );
        if before.hp == before.max_hp {
            continue;
        }
        let medicine = &r.data().items["HYPER_POTION"];
        let quantity = c.shell.shell.session().state().bag.quantity(medicine);
        press(c, GameButton::Start);
        let snapshot = c.snapshot().expect("Start");
        let target = visible_start_menu_options(&c.shell, &snapshot)
            .iter()
            .position(|x| *x == StartMenuOption::Pack)
            .expect("PACK");
        cursor(c, target, |c| {
            c.shell
                .start_menu_cursor
                .as_ref()
                .expect("Start")
                .option_index
        });
        press(c, GameButton::A);
        let target = c
            .snapshot()
            .expect("medicine")
            .bag
            .items
            .iter()
            .filter(|x| x.quantity > 0)
            .position(|x| x.item_id == "HYPER_POTION")
            .expect("carried Hyper Potions");
        cursor(c, target, |c| {
            c.shell.bag_cursor.as_ref().expect("Pack").option_index
        });
        press(c, GameButton::A);
        let target = visible_selected_pack_item_actions(
            &c.snapshot().expect("item actions"),
            &c.shell,
            &FieldPackPocket::Items,
            false,
        )
        .expect("actions")
        .iter()
        .position(|x| *x == FieldPackAction::Use)
        .expect("USE");
        cursor(c, target, |c| {
            c.shell
                .field_pack_action_cursor
                .as_ref()
                .expect("item action")
                .option_index
        });
        press(c, GameButton::A);
        cursor(c, slot, |c| c.shell.party_cursor);
        press(c, GameButton::A);
        let after = c.snapshot().expect("native healing").party.slots[slot]
            .pokemon
            .clone();
        assert_eq!(after.hp, before.max_hp);
        assert_eq!(after.moves, before.moves, "medicine does not restore PP");
        assert_eq!(
            c.shell.shell.session().state().bag.quantity(medicine),
            quantity - 1
        );
        for _ in 0..64 {
            let s = c.snapshot().expect("close Pack");
            if c.shell.field_notice.is_some() || s.ui.pending_text_wait.is_some() {
                c.wait_frames(60).expect("medicine notice");
                press(c, GameButton::A);
            } else if s.ui.menu.is_some()
                || c.shell.field_pack_target_mode.is_some()
                || c.shell.field_pack_action_cursor.is_some()
                || c.shell.bag_cursor.is_some()
                || c.shell.field_pack_pocket.is_some()
                || c.shell.start_menu_cursor.is_some()
            {
                press(c, GameButton::B);
            } else {
                break;
            }
        }
        assert!(c.snapshot().expect("care finished").ui.menu.is_none());
        assert_eq!(at(c), stand);
    }
}
