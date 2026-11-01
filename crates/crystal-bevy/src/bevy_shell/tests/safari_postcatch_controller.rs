//! Actual native Dex, nickname, transfer and binary Continue after Safari catch.

use crystal_core::input::GameButton;
use crystal_core::models::{CaptureStorageLocation, PARTY_SIZE};
use crystal_core::systems::safari::{SafariPhase, SafariResolution};
use crystal_core::systems::safari_catch_presentation::SafariCatchStage as Stage;

use super::{
    active, continue_save, controller_fixture, encounter, mark_runtime_snapshot_dirty, press, save,
};

fn stage(c: &mut super::VisibleShellController) -> Stage {
    c.snapshot()
        .expect("native completion snapshot")
        .safari_catch_presentation
        .expect("saved completion owner")
        .stage
}

#[test]
#[ignore = "requires enabled CRYSTAL_INTERIOR_PACK Safari review config"]
fn safari_postcatch_native_dex_nickname_party_pc_and_continue() {
    for pc in [false, true] {
        let (root, runtime, mut c) = controller_fixture();
        let state = c.shell.shell.session_mut().state_mut();
        state
            .flags
            .set_engine_flag("ENGINE_POKEDEX", true)
            .expect("prior owned native Pokedex");
        if pc {
            let partner = state.storage.party.pokemon[0]
                .as_ref()
                .expect("partner")
                .clone();
            for _ in 1..PARTY_SIZE {
                assert!(state.storage.party.add_pokemon(partner.clone()));
            }
            state.sync_party_from_storage();
            state.current_pc_box = 2;
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        // Setup ends. Ranger, encounters, throws, Dex and name entry all use
        // production buttons; no state/RNG/storage injection follows.
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        press(&mut c, GameButton::A);
        let prefix = if pc { "pc" } else { "party" };
        let stored = (0..30)
            .find_map(|_| {
                if active(&mut c).phase == SafariPhase::Exploring {
                    encounter(&mut c, &runtime);
                }
                press(&mut c, GameButton::Up);
                press(&mut c, GameButton::Left);
                press(&mut c, GameButton::A);
                let SafariPhase::Presenting { result, .. } = active(&mut c).phase else {
                    panic!("native throw");
                };
                if let SafariResolution::Caught(stored) = result.resolution {
                    return Some(stored);
                }
                press(&mut c, GameButton::A);
                None
            })
            .expect("actual Safari capture");
        assert_eq!(stage(&mut c), Stage::CaughtText);
        save(&mut c, &format!("{prefix}-caught"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        press(&mut c, GameButton::A);
        assert_eq!(stage(&mut c), Stage::DexNotice);
        save(&mut c, &format!("{prefix}-dex-notice"));
        press(&mut c, GameButton::A);
        assert_eq!(stage(&mut c), Stage::DexEntry);
        assert!(c.shell.pokedex_menu_open && c.shell.pokedex_scripted_entry);
        let dex = c.snapshot().expect("actual native Dex species");
        assert_eq!(
            super::super::super::selected_pokedex_catalog_species(&dex, c.shell.pokedex_cursor)
                .expect("native Dex species")
                .species_id,
            stored.pokemon.species.id
        );
        save(&mut c, &format!("{prefix}-dex-entry"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert!(c.shell.pokedex_menu_open && c.shell.pokedex_scripted_entry);
        for _ in 0..32 {
            if stage(&mut c) != Stage::DexEntry {
                break;
            }
            press(&mut c, GameButton::A);
            if c.shell.pokedex_menu_open && c.shell.pokedex_detail_page > 0 {
                let page = c.shell.pokedex_detail_page;
                c = continue_save(root.clone(), runtime.clone(), &mut c);
                assert_eq!(
                    c.shell.pokedex_detail_page, page,
                    "Continue restores actual native entry page"
                );
            }
        }
        assert_eq!(stage(&mut c), Stage::NicknamePrompt);
        assert!(!c.shell.pokedex_menu_open);
        for _ in 0..16 {
            if c.shell
                .pending_name_choice
                .as_ref()
                .expect("native nickname choice")
                .nickname_pages
                .len()
                <= 1
            {
                break;
            }
            press(&mut c, GameButton::A);
        }
        press(&mut c, GameButton::Down);
        assert!(
            !c.snapshot()
                .expect("native NO")
                .safari_catch_presentation
                .expect("owner")
                .selected_yes
        );
        save(&mut c, &format!("{prefix}-nickname-no"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(
            c.shell
                .pending_name_choice
                .as_ref()
                .expect("saved native NO choice")
                .selected,
            1
        );
        if pc {
            press(&mut c, GameButton::Up);
            press(&mut c, GameButton::A);
            assert_eq!(stage(&mut c), Stage::Naming);
            press(&mut c, GameButton::A); // Native grid A.
            press(&mut c, GameButton::Select); // Lower case.
            press(&mut c, GameButton::Right);
            press(&mut c, GameButton::A); // b.
            assert_eq!(
                c.shell
                    .pending_name_input
                    .as_ref()
                    .expect("native draft")
                    .value,
                "Ab"
            );
            save(&mut c, "pc-name-draft");
            c = continue_save(root.clone(), runtime.clone(), &mut c);
            let input = c
                .shell
                .pending_name_input
                .as_ref()
                .expect("restored native naming");
            assert_eq!(input.value, "Ab");
            assert_eq!((input.cursor_row, input.cursor_column), (0, 1));
            assert_eq!(input.case, super::super::super::NameInputCase::Lower);
            press(&mut c, GameButton::Start);
            press(&mut c, GameButton::A);
        } else {
            press(&mut c, GameButton::A);
        }
        assert_eq!(stage(&mut c), Stage::TransferText);
        let state = c.shell.shell.session().state();
        let target = match stored.location {
            CaptureStorageLocation::Party { slot } => state.storage.party.pokemon[slot]
                .as_ref()
                .expect("party catch"),
            CaptureStorageLocation::Pc { box_index, slot } => state.storage.pc_boxes[box_index]
                .pokemon[slot]
                .as_ref()
                .expect("PC catch"),
        };
        assert_eq!(
            target.nickname,
            if pc { "Ab" } else { &stored.pokemon.nickname }
        );
        assert_eq!(target.caught_data, stored.pokemon.caught_data);
        if pc {
            assert_eq!(state.storage.pc_boxes[2].nicknames[0], "Ab");
            assert_eq!(state.storage.pc_boxes[2].count, 1);
        }
        let storage = state.storage.clone();
        save(&mut c, &format!("{prefix}-transfer"));
        c = continue_save(root.clone(), runtime.clone(), &mut c);
        assert_eq!(stage(&mut c), Stage::TransferText);
        assert_eq!(c.shell.shell.session().state().storage, storage);
        press(&mut c, GameButton::A);
        assert!(
            c.snapshot()
                .expect("completion released")
                .safari_catch_presentation
                .is_none()
        );
        assert!(matches!(
            active(&mut c).phase,
            SafariPhase::Exploring | SafariPhase::Ending(_)
        ));
        assert_eq!(
            c.shell.shell.session().state().storage,
            storage,
            "completion never stores or renames twice"
        );
        save(&mut c, &format!("{prefix}-complete"));
    }
}
