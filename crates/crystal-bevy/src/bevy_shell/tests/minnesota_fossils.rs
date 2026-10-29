//! Explicit access/party fixtures; every choice, gift and retry uses input.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;

use super::super::{
    VisibleShellController, mark_runtime_snapshot_dirty, visible_start_menu_entries,
};
use super::interior_life::fixture;
use super::northstar_ice_lair::choice;
use super::r10_gallery::continue_save;
use super::winona::{load, press};

fn event(c: &VisibleShellController, name: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(name)
        .expect("fossil event")
}

fn quantity(c: &VisibleShellController, item: &str) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .bag
        .quantity(&c.shell.shell.runtime().data().items[item])
}

fn finish(c: &mut VisibleShellController) {
    for _ in 0..128 {
        c.wait_frames(64).expect("fossil text presentation");
        if let Some(name) = &c.shell.pending_name_choice {
            if name.nickname_pages.len() > 1 {
                press(c, GameButton::A);
            } else {
                if name.selected != 1 {
                    press(c, GameButton::Down);
                }
                press(c, GameButton::A);
            }
            continue;
        }
        let s = c.snapshot().expect("fossil conversation");
        assert!(
            s.ui.pending_yes_no.is_none(),
            "answer fossil choices explicitly"
        );
        if !s.ui.text_window_open && !c.has_pending_script_work() {
            return;
        }
        press(c, GameButton::A);
    }
    panic!("fossil conversation did not release input");
}

pub(super) fn talk(c: &mut VisibleShellController, facing: GameButton, answers: &[bool]) {
    press(c, facing);
    press(c, GameButton::A);
    for answer in answers {
        choice(c, *answer);
    }
    finish(c);
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with Minnesota fossil research"]
fn fossils_controller_both_choices_decline_persist_and_post_league_exchange() {
    let (root, r) = load();
    for lens in [true, false] {
        let mut c = fixture(&root, &r, "MnR01C00R00", (40, 21));
        talk(&mut c, GameButton::Left, &[false]);
        assert!(!event(&c, "EVENT_MN_FOSSIL_CHOSEN"));
        talk(&mut c, GameButton::Left, &[true, lens, false]);
        assert!(!event(&c, "EVENT_MN_FOSSIL_CHOSEN"));
        talk(&mut c, GameButton::Left, &[true, lens, true]);
        let (selected, other) = if lens {
            ("LENS_FOSSIL", "SPIRE_FOSSIL")
        } else {
            ("SPIRE_FOSSIL", "LENS_FOSSIL")
        };
        assert!(event(&c, "EVENT_MN_FOSSIL_CHOSEN"));
        assert_eq!(event(&c, "EVENT_MN_FOSSIL_CHOSE_LENS"), lens);
        assert_eq!(quantity(&c, selected), 1);
        assert_eq!(quantity(&c, other), 0);
        c = continue_save(root.clone(), r.clone(), &mut c);
        talk(&mut c, GameButton::Left, &[]);
        assert_eq!(quantity(&c, selected), 1);
        assert_eq!(quantity(&c, other), 0);
    }
    // Separate declared post-League fixtures, not an earned Champion claim.
    for lens in [true, false] {
        let mut c = fixture(&root, &r, "MnR01C00R00", (40, 21));
        {
            let state = c.shell.shell.session_mut().state_mut();
            for flag in ["EVENT_MN_FOSSIL_CHOSEN", "EVENT_MN_LEAGUE_CHAMPION"] {
                state
                    .flags
                    .set_event_flag(flag, true)
                    .expect("post-League fixture");
            }
            state
                .flags
                .set_event_flag("EVENT_MN_FOSSIL_CHOSE_LENS", lens)
                .expect("choice fixture");
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let other = if lens { "SPIRE_FOSSIL" } else { "LENS_FOSSIL" };
        talk(&mut c, GameButton::Left, &[false]);
        assert!(!event(&c, "EVENT_MN_FOSSIL_EXCHANGED"));
        talk(&mut c, GameButton::Left, &[true]);
        assert!(event(&c, "EVENT_MN_FOSSIL_EXCHANGED"));
        assert_eq!(quantity(&c, other), 1);
        c = continue_save(root.clone(), r.clone(), &mut c);
        talk(&mut c, GameButton::Left, &[]);
        assert_eq!(quantity(&c, other), 1);
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with Minnesota fossil research"]
fn fossils_controller_badge_full_party_and_actual_restoration_continue() {
    let (root, r) = load();
    for (species, item) in [("TRILUME", "LENS_FOSSIL"), ("SIPHLET", "SPIRE_FOSSIL")] {
        for (badge, party_count) in [(false, 0), (true, 6), (true, 5)] {
            let mut c = fixture(&root, &r, "MnRochester", (20, 11));
            {
                let state = c.shell.shell.session_mut().state_mut();
                state.badges.johto[0] = badge;
                state
                    .flags
                    .set_engine_flag("ENGINE_ZEPHYRBADGE", badge)
                    .expect("declared badge fixture");
                assert!(
                    state
                        .bag
                        .add_item(&r.data().items[item], 1)
                        .expect("declared specimen")
                );
                for slot in 0..party_count {
                    state.storage.party.pokemon[slot] = Some(
                        r.data()
                            .create_pokemon("POUCHID", 10, Dv::default())
                            .expect("native party fixture"),
                    );
                }
                state.sync_party_from_storage();
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
            // No state, inventory, flags, party, position or RNG writes below.
            talk(&mut c, GameButton::Up, if badge { &[true] } else { &[] });
            let restored = format!("EVENT_MN_{species}_RESTORED");
            if !badge || party_count == 6 {
                assert_eq!(quantity(&c, item), 1);
                assert!(!event(&c, &restored));
                assert_eq!(
                    c.snapshot().expect("retained party").party.slots.len(),
                    party_count
                );
            } else {
                assert_eq!(quantity(&c, item), 0);
                assert!(event(&c, &restored));
                let s = c.snapshot().expect("native restored creature");
                let gift = &s.party.slots[party_count].pokemon;
                assert_eq!(gift.species.id, species);
                assert_eq!(gift.level, 10);
                assert!(c.shell.shell.session().state().pokedex.has_caught(species));
            }
            c = continue_save(root.clone(), r.clone(), &mut c);
            if badge && party_count == 5 {
                talk(&mut c, GameButton::Up, &[]);
                assert_eq!(
                    c.snapshot().expect("no duplicate gift").party.slots.len(),
                    6
                );
                assert_eq!(quantity(&c, item), 0);
            } else {
                assert_eq!(quantity(&c, item), 1);
                assert!(!event(&c, &restored));
            }
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with Minnesota fossil research"]
fn fossils_controller_native_level_34_evolutions_and_continue() {
    let (root, r) = load();
    for (species, evolved) in [("TRILUME", "PANOPLITE"), ("SIPHLET", "OBELURK")] {
        let mut c = fixture(&root, &r, "MnRochester", (20, 11));
        {
            let state = c.shell.shell.session_mut().state_mut();
            state.storage.party.pokemon[0] = Some(
                r.data()
                    .create_pokemon(species, 33, Dv::default())
                    .expect("declared pre-evolution party"),
            );
            state.sync_party_from_storage();
            assert!(
                state
                    .bag
                    .add_item(&r.data().items["RARE_CANDY"], 1)
                    .expect("declared evolution item")
            );
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        // The fixture supplies level33. Actual PACK use must earn level34 and
        // run the native evolution, consume the item and survive Continue.
        press(&mut c, GameButton::Start);
        for _ in 0..8 {
            if visible_start_menu_entries(&c.shell)
                .expect("Start menu")
                .iter()
                .any(|s| s == ">PACK")
            {
                break;
            }
            press(&mut c, GameButton::Down);
        }
        assert!(
            visible_start_menu_entries(&c.shell)
                .expect("PACK selection")
                .iter()
                .any(|s| s == ">PACK")
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
        let mut visible_evolution = false;
        for _ in 0..128 {
            let s = c.snapshot().expect("visible growth and evolution");
            visible_evolution |=
                s.ui.text
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
        let s = c.snapshot().expect("native fossil evolution");
        assert_eq!(s.party.slots[0].pokemon.level, 34);
        assert_eq!(quantity(&c, "RARE_CANDY"), 0);
        assert_eq!(s.party.slots[0].pokemon.species.id, evolved);
        assert!(visible_evolution, "native evolution dialogue must complete");
        assert!(c.shell.shell.session().state().pokedex.has_caught(evolved));
        for _ in 0..8 {
            press(&mut c, GameButton::B);
        }
        c = continue_save(root.clone(), r.clone(), &mut c);
        assert_eq!(
            c.snapshot()
                .expect("continued fossil evolution")
                .party
                .slots[0]
                .pokemon
                .species
                .id,
            evolved
        );
    }
}
