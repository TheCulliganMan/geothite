//! Speak to the research aide before choosing, before receiving the readings, and after delivery.
use crystal_core::input::GameButton::{A, Left};
use crystal_core::models::Dv;
use crystal_runtime::RuntimeShellPhase;

use super::super::brainerd_guide::capture;
use super::super::interior_life::fixture;
use super::super::minnesota_research_access::choices;
use super::super::northstar_ice_lair::{choice, event};
use super::super::winona::{load, press};
use super::mark_runtime_snapshot_dirty;

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the revised aide"]
fn winona_aide_uses_the_players_actual_stage_of_ivys_errand() {
    const STARTER: &str = "EVENT_MN_STARTER_RECEIVED";
    const READINGS: &str = "EVENT_MN_R01_PARCEL_RECEIVED";
    const DELIVERED: &str = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL";
    const BALLS: &str = "EVENT_MN_WINONA_BALLS_RECEIVED";
    let (root, runtime) = load();
    let frames = std::env::var_os("MN_MARAIS_PARTNER_FRAMES").map(std::path::PathBuf::from);
    let modern_choices = choices::installed(&runtime);
    for (index, expected) in [
        "MnWinonaAideMeetText",
        "MnWinonaAideReadingsText",
        "MnEarlyPartnerWaitText",
        if modern_choices {
            "MnEarlyPartnerBIRCHITText"
        } else {
            "MnEarlyPartnerPOUCHIDText"
        },
    ]
    .into_iter()
    .enumerate()
    {
        let mut c = fixture(&root, &runtime, "MnWinonaFieldStation", (3, 9));
        let state = c.shell.shell.session_mut().state_mut();
        if index > 0 {
            let partner = runtime
                .data()
                .create_pokemon("KINDLIT", 5, Dv::default())
                .expect("known lab partner");
            assert!(state.storage.party.add_pokemon(partner));
            state.sync_party_from_storage();
            for flag in [STARTER, "EVENT_MN_WINONA_CYNDAQUIL"] {
                state
                    .flags
                    .set_event_flag(flag, true)
                    .expect("starter selected");
            }
        }
        state
            .flags
            .set_event_flag(READINGS, index >= 2)
            .expect("reading receipt");
        state
            .flags
            .set_event_flag(DELIVERED, index == 3)
            .expect("clinic receipt");
        mark_runtime_snapshot_dirty(&mut c.shell);
        press(&mut c, Left);
        assert_eq!(
            c.shell
                .shell
                .current_overworld_interaction_checked()
                .expect("aide interaction")
                .expect("facing aide")
                .script,
            "MnKindlitResearchAide"
        );
        press(&mut c, A);
        let mut labels = Vec::new();
        for _ in 0..96 {
            c.wait_frames(128).expect("printed aide dialogue");
            let snap = c.snapshot().expect("aide visit");
            if let Some(text) = snap.ui.text.as_ref()
                && labels.last() != Some(&text.label)
            {
                labels.push(text.label.clone());
                if index == 0 && text.label == expected {
                    c = capture(c, frames.as_deref(), "winona-research-aide.png");
                }
            }
            if index == 3
                && modern_choices
                && c.shell.menu_cursor.is_some()
                && snap.ui.menu.as_ref().is_some_and(|menu| menu.window_open)
            {
                choices::select(&mut c, Some("BIRCHIT"));
            } else if snap.ui.pending_yes_no.is_some() {
                assert_eq!(index, 3);
                choice(&mut c, false);
            } else if snap.phase == RuntimeShellPhase::Overworld && !c.has_pending_script_work() {
                break;
            } else {
                press(&mut c, A);
            }
        }
        assert!(
            labels.iter().any(|s| s == expected),
            "stage {index}: {labels:?}"
        );
        assert!(!labels.iter().any(|s| s == "MnWChooseText"));
        assert_eq!(
            labels.iter().any(|s| s == "MnEarlyPartnerWaitText"),
            index == 2
        );
        if index == 0 {
            assert_eq!(labels, [expected]);
        }
        assert_eq!(event(&c, STARTER), index > 0);
        assert_eq!(event(&c, BALLS), index > 0);
        assert_eq!(event(&c, READINGS), index >= 2);
        assert_eq!(event(&c, DELIVERED), index == 3);
        let state = c.shell.shell.session().state();
        assert_eq!(state.storage.party.filled_slots(), usize::from(index > 0));
        assert_eq!(
            state.bag.quantity(&runtime.data().items["POKE_BALL"]),
            if index > 0 { 5 } else { 0 }
        );
        assert!(!c.has_pending_script_work());
    }
}
