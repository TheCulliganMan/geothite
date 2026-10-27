//! Production input for a battle item list and a non-leading party target.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::{VisibleShellController, prepare_visible_battle_entry};
use super::route36_overworld_shell_for_battle_render_regression;
use super::winona::press;

#[test]
fn renderer_neutral_battle_pack_navigates_items_and_heals_second_partner() {
    let mut shell = route36_overworld_shell_for_battle_render_regression();
    shell
        .shell
        .add_party_pokemon(
            "BAYLEEF",
            40,
            None,
            None,
            "CHRIS",
            1,
            Dv::from_non_hp(10, 10, 10, 10),
        )
        .expect("fixture replacement");
    shell
        .shell
        .add_bag_item("POTION", 2)
        .expect("fixture medicine");
    shell
        .shell
        .add_bag_item("ANTIDOTE", 1)
        .expect("second item row");
    // Initial fixture: the bench partner needs care. Keeping it off the
    // field isolates its exact healing from the opponent's counterattack.
    // No party, item or battle writes follow the actual battle start below.
    let state = shell.shell.session_mut().state_mut();
    let replacement = state.storage.party.pokemon[1]
        .as_mut()
        .expect("replacement");
    replacement.hp -= 20;
    state.sync_party_from_storage();
    shell
        .shell
        .start_scripted_wild_battle("Route36", "WateredWeirdTreeScript", 12)
        .expect("fixture battle");
    prepare_visible_battle_entry(&mut shell).expect("native battle entry");
    let mut c = VisibleShellController { shell };
    for _ in 0..256 {
        if c.shell.battle_action_cursor.is_some()
            && c.shell.battle_messages.is_empty()
            && c.shell.visible_battle_transition.is_none()
            && c.shell.visible_battle_sliding_intro.is_none()
            && c.shell.visible_send_out_animation.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert_eq!(
        c.snapshot()
            .expect("replacement active")
            .battle
            .expect("battle")
            .active_player_party_index,
        Some(0)
    );
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    assert_eq!(
        c.shell.bag_cursor.as_ref().expect("battle Pack").surface_id,
        "battle:bag-items"
    );
    let row = c.shell.bag_cursor.as_ref().expect("item row").option_index;
    press(&mut c, GameButton::Down);
    assert_eq!(
        c.shell
            .bag_cursor
            .as_ref()
            .expect("next item row")
            .option_index,
        row + 1
    );
    press(&mut c, GameButton::Up);
    assert_eq!(
        c.shell
            .bag_cursor
            .as_ref()
            .expect("Potion row")
            .option_index,
        row
    );
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert!(c.shell.battle_pack_target_mode.is_some());
    if c.shell.party_cursor == 1 {
        press(&mut c, GameButton::Up);
    }
    assert_eq!(c.shell.party_cursor, 0);
    press(&mut c, GameButton::Down);
    assert_eq!(c.shell.party_cursor, 1);
    assert_eq!(
        c.shell
            .bag_cursor
            .as_ref()
            .expect("retained battle items")
            .surface_id,
        "battle:bag-items"
    );
    let before = c.snapshot().expect("before actual Potion");
    let potions = c
        .shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&c.shell.runtime.data().items["POTION"]);
    press(&mut c, GameButton::A);
    let after = c.snapshot().expect("after actual Potion");
    assert_eq!(
        after.party.slots[1].pokemon.hp,
        before.party.slots[1].pokemon.hp + 20
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&c.shell.runtime.data().items["POTION"]),
        potions - 1
    );
    assert!(after.battle.is_some());
}
