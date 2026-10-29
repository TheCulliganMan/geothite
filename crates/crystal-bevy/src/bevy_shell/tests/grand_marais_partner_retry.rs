//! Declared party fixtures; partner delivery and the lighthouse use native input.
use crystal_core::input::GameButton;
use crystal_core::models::{Dv, MAX_BOX_MONS, PcBox};

use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::fixture;
use super::minnesota_fossils::talk;
use super::r10_gallery::continue_save;
use super::winona::{load, press};

const WELCOME: &str = "EVENT_MN_GRAND_MARAIS_STARTER";
const SIGNAL: &str = "EVENT_MN_GRAND_MARAIS_SIGNAL";

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the Grand Marais partner retry"]
fn marais_partner_controller_full_party_retains_offer_and_success_survives_continue() {
    let (root, runtime) = load();
    for (count, full_box) in [(1, false), (6, false), (6, true)] {
        let mut c = fixture(&root, &runtime, "MnGrandMaraisFieldStation", (5, 4));
        {
            let state = c.shell.shell.session_mut().state_mut();
            for slot in 0..count {
                state.storage.party.pokemon[slot] = Some(
                    runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("declared Minnesota partner"),
                );
            }
            if full_box {
                while state.storage.pc_boxes.len() <= state.current_pc_box {
                    state
                        .storage
                        .pc_boxes
                        .push(PcBox::new(state.storage.pc_boxes.len()));
                }
                for _ in 0..MAX_BOX_MONS {
                    let partner = runtime
                        .data()
                        .create_pokemon("POUCHID", 10, Dv::default())
                        .expect("declared full selected box");
                    assert!(state.storage.pc_boxes[state.current_pc_box].add_pokemon(partner));
                }
            }
            state.sync_party_from_storage();
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let original_storage = c.shell.shell.session().state().storage.clone();
        // No state, party, inventory, flag, position or RNG writes below.
        talk(&mut c, GameButton::Up, &[]);
        let received = !full_box;
        let party_received = received && count < 6;
        let state = c.shell.shell.session().state();
        assert_eq!(
            state.flags.is_event_flag_set(WELCOME).expect("welcome"),
            received
        );
        assert_eq!(
            state.bag.quantity(&runtime.data().items["MACHINE_PART"]),
            u16::from(received)
        );
        assert_eq!(
            state.storage.party.filled_slots(),
            count + usize::from(party_received)
        );
        if received {
            let partner = if party_received {
                state.storage.party.pokemon[count].as_ref()
            } else {
                state.storage.pc_boxes[state.current_pc_box].pokemon[0].as_ref()
            }
            .expect("actual partner in party or selected PC");
            assert_eq!(partner.species.id, "BIRCHIT");
            assert_eq!(partner.level, 8);
            assert_eq!(partner.item.as_deref(), Some("BERRY"));
            assert!(state.pokedex.has_caught("BIRCHIT"));
        } else {
            assert_eq!(
                state.storage, original_storage,
                "failed gift changed storage"
            );
            assert!(!state.pokedex.has_caught("BIRCHIT"));
        }
        let bag = state.bag.clone();
        let storage = state.storage.clone();
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        talk(&mut c, GameButton::Up, &[]);
        let state = c.shell.shell.session().state();
        assert_eq!(state.bag, bag, "repeat must not duplicate supplies");
        assert_eq!(
            state.storage, storage,
            "Continue/repeat changed the partner"
        );
        assert_eq!(
            state
                .flags
                .is_event_flag_set(WELCOME)
                .expect("saved welcome"),
            received
        );
        assert_eq!(
            state.storage.party.filled_slots(),
            count + usize::from(party_received)
        );
        if received {
            assert!(state.pokedex.has_caught("BIRCHIT"));
        }
        if party_received {
            // Walk out of the station, across town and into the lighthouse.
            super::minnesota_gyms::walk_to(&mut c, &runtime, (2, 6));
            for _ in 0..16 {
                if c.snapshot().expect("station exit").overworld.map_name == "MnGrandMarais" {
                    break;
                }
                press(&mut c, GameButton::Down);
            }
            assert_eq!(
                c.snapshot()
                    .expect("actual town arrival")
                    .overworld
                    .map_name,
                "MnGrandMarais"
            );
            super::minnesota_gyms::walk_to(&mut c, &runtime, (15, 28));
            for _ in 0..16 {
                if c.snapshot().expect("lighthouse door").overworld.map_name
                    == "MnGrandMaraisLighthouse"
                {
                    break;
                }
                press(&mut c, GameButton::Up);
            }
            assert_eq!(
                c.snapshot()
                    .expect("actual lighthouse arrival")
                    .overworld
                    .map_name,
                "MnGrandMaraisLighthouse"
            );
            super::minnesota_gyms::walk_to(&mut c, &runtime, (5, 4));
            talk(&mut c, GameButton::Up, &[]);
            let state = c.shell.shell.session().state();
            assert!(
                state
                    .flags
                    .is_event_flag_set(SIGNAL)
                    .expect("actual repair")
            );
            assert_eq!(state.bag.quantity(&runtime.data().items["MACHINE_PART"]), 0);
            assert_eq!(state.bag.quantity(&runtime.data().items["SUPER_POTION"]), 1);
            let repaired_bag = state.bag.clone();
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            talk(&mut c, GameButton::Up, &[]);
            assert_eq!(c.shell.shell.session().state().bag, repaired_bag);
            assert!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(SIGNAL)
                    .expect("saved repair")
            );
        }
    }
}
