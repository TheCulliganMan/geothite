//! Real native throws and Continue prove delivery of the preserve's caught area.

use crystal_core::input::GameButton;
use crystal_core::state::BattleMemory;
use crystal_core::systems::safari::{SafariPhase, SafariResolution};

use super::{active, continue_save, controller_fixture, encounter, press, save};

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with enabled Safari review config"]
fn safari_controller_catch_retains_native_landmark_owner_storage_and_continue() {
    let (root, runtime, mut c) = controller_fixture();
    // Setup ends before the first input. No encounter, capture, RNG or storage
    // is injected after the real ranger interaction starts.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    assert_eq!(c.shell.shell.session().state().money, 1000);
    let original_partner = c.shell.shell.session().state().storage.party.pokemon[0].clone();
    for _ in 0..30 {
        if active(&mut c).phase == SafariPhase::Exploring {
            encounter(&mut c, &runtime);
        }
        let SafariPhase::Encounter(wild) = active(&mut c).phase else {
            panic!("native wildlife required before a throw");
        };
        let landmark = u8::try_from(
            runtime
                .data()
                .pokegear_landmark_for_map(&wild.origin_map_name)
                .expect("actual Town Map landmark")
                .id,
        )
        .expect("native landmark byte");
        assert_ne!(landmark, 0);
        assert_eq!(wild.caught_location, landmark);
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::Left);
        let balls = active(&mut c).balls_remaining;
        press(&mut c, GameButton::A);
        let pending = active(&mut c);
        assert_eq!(pending.balls_remaining, balls - 1);
        let SafariPhase::Presenting { result, .. } = &pending.phase else {
            panic!("real throw must produce a saved result");
        };
        let state = c.shell.shell.session().state();
        assert_eq!(state.battle, BattleMemory::Inactive);
        assert_eq!(state.storage.party.pokemon[0], original_partner);
        if let SafariResolution::Caught(stored) = &result.resolution {
            let caught = stored
                .pokemon
                .caught_data
                .as_ref()
                .expect("native caught data");
            assert_eq!(caught.location, landmark);
            assert_eq!(caught.level, wild.pokemon.level & 0x3f);
            assert_eq!(caught.time_of_day, Some(state.time.time_of_day));
            assert_eq!(caught.original_trainer_gender, state.player_gender);
            assert_eq!(stored.pokemon.original_trainer_name, state.player_name);
            assert_eq!(stored.pokemon.original_trainer_id, state.player_id);
            assert_eq!(
                state.storage.party.pokemon[1].as_ref(),
                Some(&stored.pokemon)
            );
            assert!(state.pokedex.has_caught(&stored.pokemon.species.id));
            let storage = state.storage.clone();
            let pokedex = state.pokedex.clone();
            save(&mut c, "caught-result");
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            assert_eq!(active(&mut c), pending);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            assert_eq!(c.shell.shell.session().state().pokedex, pokedex);
            press(&mut c, GameButton::Start);
            assert_eq!(active(&mut c), pending, "pending catch cannot be rerolled");
            super::finish_catch_without_nickname(&mut c);
            assert_eq!(active(&mut c).phase, SafariPhase::Exploring);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            save(&mut c, "caught-acknowledged");
            c = continue_save(root, runtime, &mut c);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            assert_eq!(c.shell.shell.session().state().pokedex, pokedex);
            return;
        }
        press(&mut c, GameButton::A);
    }
    panic!("the native thirty-ball itinerary never delivered a capture");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with enabled Safari review config"]
fn safari_controller_full_party_delivers_to_selected_pc_box_and_continue() {
    use crystal_core::models::{CaptureStorageLocation, PARTY_SIZE};
    let (root, runtime, mut c) = controller_fixture();
    let partner = c.shell.shell.session().state().storage.party.pokemon[0]
        .as_ref()
        .expect("initial regional partner")
        .clone();
    let state = c.shell.shell.session_mut().state_mut();
    for _ in 1..PARTY_SIZE {
        assert!(state.storage.party.add_pokemon(partner.clone()));
    }
    state.sync_party_from_storage();
    state.current_pc_box = 2;
    super::mark_runtime_snapshot_dirty(&mut c.shell);
    // Full party and selected box are explicit setup; entry and every throw
    // below use production input. No capture, party or RNG injection follows.
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::A);
    let original = c.shell.shell.session().state().storage.clone();
    for _ in 0..30 {
        if active(&mut c).phase == SafariPhase::Exploring {
            encounter(&mut c, &runtime);
        }
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        let pending = active(&mut c);
        let SafariPhase::Presenting { result, .. } = &pending.phase else {
            panic!("native throw result");
        };
        if let SafariResolution::Caught(stored) = &result.resolution {
            assert_eq!(
                stored.location,
                CaptureStorageLocation::Pc {
                    box_index: 2,
                    slot: 0
                }
            );
            let state = c.shell.shell.session().state();
            assert_eq!(state.storage.party, original.party);
            assert_eq!(
                state.storage.pc_boxes[2].pokemon[0].as_ref(),
                Some(&stored.pokemon)
            );
            assert_eq!(state.storage.pc_boxes[2].count, 1);
            for (index, pc) in state.storage.pc_boxes.iter().enumerate() {
                if index != 2 {
                    assert_eq!(pc, &original.pc_boxes[index]);
                }
            }
            assert!(state.pokedex.has_caught(&stored.pokemon.species.id));
            let storage = state.storage.clone();
            save(&mut c, "pc-caught-result");
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            assert_eq!(active(&mut c), pending);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            press(&mut c, GameButton::Start);
            assert_eq!(active(&mut c), pending);
            super::finish_catch_without_nickname(&mut c);
            assert_eq!(active(&mut c).phase, SafariPhase::Exploring);
            assert_eq!(c.shell.shell.session().state().storage, storage);
            save(&mut c, "pc-caught-acknowledged");
            return;
        }
        press(&mut c, GameButton::A);
    }
    panic!("native full-party itinerary never delivered a PC capture");
}
