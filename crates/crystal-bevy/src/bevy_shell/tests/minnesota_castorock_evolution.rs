//! Declared level-31 specimen; the native item menu earns its evolution.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::super::{mark_runtime_snapshot_dirty, visible_start_menu_entries};
use super::super::interior_life::fixture;
use super::super::r10_gallery::continue_save;
use super::super::winona::{load, press};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the native Minnesota roster"]
fn castorock_controller_native_level_32_evolution_and_continue() {
    let (root, runtime) = load();
    let mut c = fixture(&root, &runtime, "MnRochester", (20, 11));
    {
        let state = c.shell.shell.session_mut().state_mut();
        state.storage.party.pokemon[0] = Some(
            runtime
                .data()
                .create_pokemon("CASTOROCK", 31, Dv::default())
                .expect("declared pre-evolution specimen"),
        );
        state.sync_party_from_storage();
        assert!(
            state
                .bag
                .add_item(&runtime.data().items["RARE_CANDY"], 1)
                .expect("declared evolution item")
        );
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    // No state, party, inventory, flags, position or RNG writes below.
    press(&mut c, GameButton::Start);
    for _ in 0..8 {
        if visible_start_menu_entries(&c.shell)
            .expect("Start menu")
            .iter()
            .any(|entry| entry == ">PACK")
        {
            break;
        }
        press(&mut c, GameButton::Down);
    }
    assert!(
        visible_start_menu_entries(&c.shell)
            .expect("PACK selected")
            .iter()
            .any(|entry| entry == ">PACK")
    );
    press(&mut c, GameButton::A);
    let index = c
        .snapshot()
        .expect("native bag")
        .bag
        .items
        .iter()
        .filter(|item| item.quantity > 0)
        .position(|item| item.item_id == "RARE_CANDY")
        .expect("carried candy");
    for _ in 0..index {
        press(&mut c, GameButton::Down);
    }
    for _ in 0..3 {
        press(&mut c, GameButton::A);
    }
    let mut saw_evolution = false;
    for _ in 0..128 {
        let snapshot = c.snapshot().expect("growth presentation");
        saw_evolution |= snapshot
            .ui
            .text
            .as_ref()
            .and_then(|text| text.asm_text.as_ref())
            .is_some_and(|text| text.contains("evolved"));
        let state = c.shell.shell.session().state();
        if state.pending_move_learn.is_none()
            && state.pending_move_learn_queue.is_empty()
            && c.shell.field_notice.is_none()
            && c.shell.field_notice_queue.is_empty()
            && c.shell.field_evolution_cancellation.is_none()
            && c.shell.battle_messages.is_empty()
            && c.shell.battle_evolution_cancellations.is_empty()
            && c.shell.special_boundary.is_none()
        {
            break;
        }
        press(&mut c, GameButton::A);
    }
    let snapshot = c.snapshot().expect("actual Castoroar evolution");
    assert_eq!(snapshot.party.slots[0].pokemon.species.id, "CASTOROAR");
    assert_eq!(snapshot.party.slots[0].pokemon.level, 32);
    assert!(saw_evolution, "acknowledged native evolution dialogue");
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("CASTOROAR")
    );
    assert_eq!(
        c.shell
            .shell
            .session()
            .state()
            .bag
            .quantity(&runtime.data().items["RARE_CANDY"]),
        0
    );
    for _ in 0..8 {
        press(&mut c, GameButton::B);
    }
    c = continue_save(root, runtime, &mut c);
    let snapshot = c.snapshot().expect("continued evolved specimen");
    assert_eq!(snapshot.party.slots[0].pokemon.species.id, "CASTOROAR");
    assert_eq!(snapshot.party.slots[0].pokemon.level, 32);
    assert!(
        c.shell
            .shell
            .session()
            .state()
            .pokedex
            .has_caught("CASTOROAR")
    );
}
