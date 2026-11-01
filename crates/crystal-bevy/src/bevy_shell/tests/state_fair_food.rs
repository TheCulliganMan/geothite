//! Purchased beer uses native Pack targeting and authoritative battle turns.
use crystal_core::input::GameButton;

use super::{
    battle, buy, continue_save, cursor, eat, event, finish, fixture, press, quantity, walk,
};

#[test]
#[ignore = "requires the published fair foods pack"]
fn state_fair_beer_purchase_field_continue_battle_and_confusion() {
    let (root, r, mut c) = fixture(false);
    buy(&mut c, &r, (71, 12), "FAIR_BEER");
    buy(&mut c, &r, (71, 12), "FAIR_BEER");
    eat(&mut c, &r, "FAIR_BEER", 100);
    assert_eq!(quantity(&c, &r, "FAIR_BEER"), 1);
    c = continue_save(&mut c, &root, &r);
    let target = &c.shell.shell.session().state().storage.party.pokemon[0];
    assert_eq!(
        target.as_ref().expect("field beer target").confusion_turns,
        4
    );
    walk(&mut c, &r, (35, 52));
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
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
    let before = c.snapshot().expect("actual junior battle");
    assert!(before.battle.is_some());
    assert_eq!(
        before.party.slots[0].pokemon.confusion_turns, 4,
        "field beer reaches the battle without a lost volatile counter"
    );
    let hp = before.party.slots[0].pokemon.hp;
    // The battle action menu is a 2x2 grid; PACK is below FIGHT.
    press(&mut c, GameButton::Down);
    press(&mut c, GameButton::A);
    let row = before
        .bag
        .items
        .iter()
        .filter(|i| i.quantity > 0)
        .position(|i| i.item_id == "FAIR_BEER")
        .expect("purchased beer row");
    cursor(&mut c, row, |c| {
        c.shell
            .bag_cursor
            .as_ref()
            .expect("battle bag")
            .option_index
    });
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert!(c.shell.battle_pack_target_mode.is_some());
    cursor(&mut c, 0, |c| c.shell.party_cursor);
    press(&mut c, GameButton::A);
    let after = c.snapshot().expect("beer and enemy response");
    assert_eq!(quantity(&c, &r, "FAIR_BEER"), 0);
    assert!(after.party.slots[0].pokemon.hp > hp);
    assert!(after.party.slots[0].pokemon.hp <= hp + 100);
    assert_eq!(after.party.slots[0].pokemon.confusion_turns, 4);
    assert!(
        after.battle.expect("battle continues").enemy_turns_taken
            > before.battle.expect("before battle").enemy_turns_taken,
        "using beer spends a real turn with an enemy response"
    );
    assert!(
        c.shell
            .battle_messages
            .iter()
            .any(|s| s.contains("became confused"))
    );
    battle(&mut c, false);
    finish(&mut c);
    assert!(event(&c, "EVENT_MN_FAIR_JUNIOR_DEFEATED"));
    assert_eq!(
        c.snapshot().expect("battle ended").party.slots[0]
            .pokemon
            .confusion_turns,
        0,
        "confusion clears on the normal battle boundary"
    );
}
