//! Native Pack input shared by the grotto resource regressions.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::super::super::{
    FieldPackAction, FieldPackPocket, StartMenuOption, VisibleShellController,
    visible_selected_pack_item_actions, visible_start_menu_options,
};
use super::super::super::winona::press;
use super::super::finish;

fn select(
    c: &mut VisibleShellController,
    target: usize,
    read: impl Fn(&VisibleShellController) -> usize,
) {
    for _ in 0..64 {
        let selected = read(c);
        if selected == target {
            press(c, GameButton::A);
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
    panic!("native Pack selection did not reach {target}");
}

fn use_item(c: &mut VisibleShellController, id: &str) {
    press(c, GameButton::Start);
    let s = c.snapshot().expect("Start");
    let pack = visible_start_menu_options(&c.shell, &s)
        .iter()
        .position(|o| *o == StartMenuOption::Pack)
        .expect("PACK");
    select(c, pack, |c| {
        c.shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index
    });
    for _ in 0..4 {
        if c.shell.field_pack_pocket == Some(FieldPackPocket::Items) {
            break;
        }
        press(c, GameButton::Right);
    }
    let item = c
        .snapshot()
        .expect("Items")
        .bag
        .items
        .iter()
        .position(|i| i.item_id == id && i.quantity > 0)
        .expect("actual earned resource");
    select(c, item, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("bag cursor")
            .option_index
    });
    let s = c.snapshot().expect("item actions");
    let action = visible_selected_pack_item_actions(&s, &c.shell, &FieldPackPocket::Items, false)
        .expect("actions")
        .iter()
        .position(|a| *a == FieldPackAction::Use)
        .expect("USE");
    select(c, action, |c| {
        c.shell
            .field_pack_action_cursor
            .as_ref()
            .expect("action cursor")
            .option_index
    });
}

pub(super) fn revive(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let before = c.snapshot().expect("fainted companion").party.slots[1]
        .pokemon
        .clone();
    assert_eq!(before.hp, 0);
    use_item(c, "REVIVE");
    select(c, 1, |c| c.shell.party_cursor);
    let after = c.snapshot().expect("native revival").party.slots[1]
        .pokemon
        .clone();
    assert_eq!(after.hp, (before.max_hp / 2).max(1));
    assert_eq!(after.moves, before.moves);
    assert_eq!(super::quantity(c, r, "REVIVE"), 0);
    finish(c);
}

pub(super) fn restore_move(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let before = c.snapshot().expect("depleted partner").party.slots[0]
        .pokemon
        .clone();
    assert_eq!(before.moves[0].current_pp, 0);
    use_item(c, "MAX_ETHER");
    // MOVE-scoped medicine opens the combined party/move target immediately.
    // Confirming the party first would already consume the selected move item.
    assert_eq!(c.shell.party_cursor, 0);
    select(c, 0, |c| {
        c.shell
            .party_move_cursor
            .as_ref()
            .expect("move target")
            .option_index
    });
    let after = c.snapshot().expect("native PP restored").party.slots[0]
        .pokemon
        .clone();
    assert_eq!(
        after.moves[0].current_pp,
        r.data().moves[&before.moves[0].name].pp
    );
    assert_eq!(after.hp, before.hp);
    assert_eq!(after.moves[1..], before.moves[1..]);
    finish(c);
}

pub(super) fn toss(c: &mut VisibleShellController) {
    super::super::super::northstar_progression_rewards::native_toss_first_pocket_item(
        c,
        &FieldPackPocket::Items,
    );
    for _ in 0..8 {
        press(c, GameButton::B);
    }
}
