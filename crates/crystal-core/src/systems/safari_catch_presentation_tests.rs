//! Real core captures retain native completion and storage across binary saves.

use crate::models::{CaptureStorageLocation, PARTY_SIZE};
use crate::systems::safari::{self, SafariAction, SafariEndReason, SafariPhase, SafariResolution};
use crate::systems::safari_catch_presentation::{
    self as screens, SafariCatchCommand as Command, SafariCatchStage as Stage, SafariNameDraft,
};

use super::{Draws, action, active, binary_continue, encounter, exploring, wildlife};

#[test]
fn safari_capture_screens_rename_party_and_pc_atomically_and_defer_last_ball_return() {
    for pc in [false, true] {
        let mut state = exploring();
        if pc {
            for _ in 0..PARTY_SIZE {
                assert!(state.storage.party.add_pokemon(wildlife()));
            }
            state.sync_party_from_storage();
            state.current_pc_box = 2;
        }
        let mut permit = active(&state);
        permit.balls_remaining = 1;
        safari::set_session(&mut state, &permit).expect("last-ball setup");
        safari::begin_encounter(&mut state, encounter()).expect("actual core wildlife");
        action(&mut state, SafariAction::Ball, &mut Draws::new(&[0])).expect("actual core capture");
        screens::begin(&mut state, true, false).expect("native completion owner");
        let original_storage = state.storage.clone();
        let before = state.clone();
        assert!(
            safari::acknowledge_result(&mut state).is_err(),
            "raw acknowledgement cannot skip owned capture screens"
        );
        assert_eq!(state, before);
        for expected in [Stage::DexNotice, Stage::DexEntry, Stage::NicknamePrompt] {
            screens::apply(&mut state, Command::Advance).expect("one owned boundary");
            assert_eq!(
                screens::presentation(&state)
                    .expect("valid")
                    .expect("owner")
                    .stage,
                expected
            );
            assert_eq!(state.storage, original_storage);
            state = binary_continue(state);
        }
        screens::apply(
            &mut state,
            Command::SelectNickname {
                selected_yes: false,
            },
        )
        .expect("native NO");
        state = binary_continue(state);
        assert!(
            !screens::presentation(&state)
                .expect("valid")
                .expect("owner")
                .selected_yes
        );
        screens::apply(&mut state, Command::SelectNickname { selected_yes: true })
            .expect("native YES");
        screens::apply(&mut state, Command::ConfirmNickname).expect("name entry");
        let draft = SafariNameDraft {
            value: "PINE".into(),
            cursor_column: 3,
            cursor_row: 2,
            uppercase: false,
        };
        screens::apply(
            &mut state,
            Command::UpdateNameDraft {
                draft: draft.clone(),
            },
        )
        .expect("native name input");
        state = binary_continue(state);
        assert_eq!(
            screens::presentation(&state)
                .expect("valid")
                .expect("owner")
                .name_draft,
            draft
        );
        let before = state.clone();
        assert!(
            screens::apply(
                &mut state,
                Command::UpdateNameDraft {
                    draft: SafariNameDraft {
                        value: "TOO LONG NAME".into(),
                        ..draft
                    }
                }
            )
            .is_err()
        );
        assert_eq!(
            state, before,
            "invalid draft rejects without touching capture or storage"
        );
        screens::apply(&mut state, Command::FinishName).expect("atomic stored/result rename");
        let SafariPhase::Presenting { result, .. } = active(&state).phase else {
            panic!("pending catch");
        };
        let SafariResolution::Caught(stored) = result.resolution else {
            panic!("stored catch");
        };
        assert_eq!(stored.pokemon.nickname, "PINE");
        match stored.location {
            CaptureStorageLocation::Pc { box_index, slot } => {
                assert_eq!((box_index, slot), (2, 0));
                assert_eq!(state.storage.pc_boxes[2].nicknames[0], "PINE");
                assert_eq!(
                    state.storage.pc_boxes[2].pokemon[0].as_ref(),
                    Some(&stored.pokemon)
                );
                assert_eq!(state.storage.party, original_storage.party);
            }
            CaptureStorageLocation::Party { slot } => assert_eq!(
                state.storage.party.pokemon[slot].as_ref(),
                Some(&stored.pokemon)
            ),
        }
        safari::validate_saved_session(&state).expect("strict renamed catch/storage validation");
        state = binary_continue(state);
        assert_eq!(
            screens::presentation(&state)
                .expect("valid")
                .expect("owner")
                .stage,
            Stage::TransferText
        );
        assert_eq!(
            active(&state).pending_end,
            Some(SafariEndReason::BallsExhausted)
        );
        screens::apply(&mut state, Command::Advance).expect("finish transfer notice once");
        assert!(screens::presentation(&state).expect("valid").is_none());
        assert_eq!(
            active(&state).phase,
            SafariPhase::Ending(SafariEndReason::BallsExhausted)
        );
        assert!(screens::apply(&mut state, Command::Advance).is_err());
    }
}

#[test]
fn safari_capture_decline_and_required_nickname_follow_native_policy() {
    for required in [false, true] {
        let mut state = exploring();
        safari::begin_encounter(&mut state, encounter()).expect("wildlife");
        action(&mut state, SafariAction::Ball, &mut Draws::new(&[0])).expect("capture");
        screens::begin(&mut state, false, required).expect("native policy");
        screens::apply(&mut state, Command::Advance).expect("no owned Dex screen");
        screens::apply(
            &mut state,
            Command::SelectNickname {
                selected_yes: false,
            },
        )
        .expect("NO");
        screens::apply(&mut state, Command::ConfirmNickname).expect("native policy on NO");
        let stage = screens::presentation(&state)
            .expect("valid")
            .expect("owner")
            .stage;
        assert_eq!(
            stage,
            if required {
                Stage::Naming
            } else {
                Stage::TransferText
            }
        );
        if required {
            screens::apply(&mut state, Command::FinishName)
                .expect("empty entry keeps native species name");
        }
        screens::apply(&mut state, Command::Advance).expect("finish transfer");
        assert_eq!(active(&state).phase, SafariPhase::Exploring);
    }
}
