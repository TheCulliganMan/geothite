//! Inspect rewritten replies and approach timing through production input.
use bevy::prelude::{ButtonInput, KeyCode};
use crystal_core::input::GameButton;
use crystal_core::world::map::Direction;

use super::super::super::{BevyRuntimeShell, VisibleShellController};
use super::super::integrated_shell_test_app;
use super::super::northstar_dialogue::canonical_pages;
use super::super::winona::{continue_in_place, finish, press};
use super::fixture;

fn capture(c: &mut VisibleShellController, label: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("story scene capture directory");
        c.save(directory.join(format!("{label}.crystalsave")))
            .expect("actual reached dialogue checkpoint");
    }
}

fn read(c: &mut VisibleShellController, label: &str) {
    for (page_index, expected) in canonical_pages("", label).into_iter().enumerate() {
        let before = c.snapshot().expect("displayed authored page");
        let text = before.ui.text.expect("visible dialogue");
        assert_eq!(text.label, label);
        assert_eq!(text.asm_text, Some(expected));
        capture(c, &format!("{label}-page-{page_index}"));
        press(c, GameButton::Down);
        assert_eq!(
            c.snapshot().expect("owned input").overworld.tile,
            before.overworld.tile
        );
        press(c, GameButton::A);
    }
    assert!(c.snapshot().expect("dialogue completed").ui.text.is_none());
}

#[test]
#[ignore = "requires the rewritten external pack in CRYSTAL_WINONA_TEST_PACK"]
fn story_review_cousin_tracks_real_milestones_without_granting_progress() {
    let stages = [
        ("EVENT_MN_STARTER_RECEIVED", "MnWinonaCousinStarterText"),
        (
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "MnWinonaCousinRochesterText",
        ),
        (
            "EVENT_MN_ICE_CAPTIVE_RESCUED",
            "MnWinonaCousinMinneapolisText",
        ),
        ("EVENT_MN_COURIER_DEFEATED", "MnWinonaCousinStPaulText"),
        ("EVENT_MN_BRAINERD_REPORT", "MnWinonaCousinBrainerdText"),
        ("EVENT_MN_DEPOT_CLEARED", "MnWinonaCousinIronRangeText"),
        ("EVENT_MN_SURVEY_RESTORED", "MnWinonaCousinRestoredText"),
        ("EVENT_MN_LEAGUE_CHAMPION", "MnWinonaCousinChampionText"),
    ];
    let mut flags = vec!["EVENT_MN_HOME_INTRO"];
    for (flag, label) in stages {
        flags.push(flag);
        let (root, runtime, mut c) = fixture("MnWinonaHome2F", (2, 3), &flags);
        let before = c.shell.shell.session().state().clone();
        press(&mut c, GameButton::Left);
        press(&mut c, GameButton::A);
        capture(&mut c, label);
        read(&mut c, label);
        let after = c.shell.shell.session().state();
        assert_eq!(after.flags, before.flags);
        assert_eq!(after.bag, before.bag);
        assert_eq!(after.badges, before.badges);
        assert_eq!(after.storage.party, before.storage.party);
        continue_in_place(&mut c, &root, &runtime, label);
        press(&mut c, GameButton::A);
        read(&mut c, label);
        for _ in 0..3 {
            if c.snapshot().expect("movement released").overworld.tile.x == 3 {
                break;
            }
            press(&mut c, GameButton::Right);
        }
        assert_eq!(c.snapshot().expect("movement released").overworld.tile.x, 3);
        press(&mut c, GameButton::Start);
        assert!(c.snapshot().expect("Start released").ui.menu.is_some());
    }
}

#[test]
#[ignore = "declared chapter fixtures; requires rewritten pack in CRYSTAL_WINONA_TEST_PACK"]
fn story_review_tamarack_returns_show_the_current_lead_after_continue() {
    for (event, label) in [
        ("EVENT_MN_ICE_CAPTIVE_RESCUED", "MnWTamarackGuideStPaulText"),
        ("EVENT_MN_COURIER_DEFEATED", "MnWTamarackGuideManifestText"),
        (
            "EVENT_MN_DEEP_BORE_MANIFEST_READ",
            "MnWTamarackGuideBrainerdText",
        ),
        ("EVENT_MN_BRAINERD_REPORT", "MnWTamarackGuideNorthText"),
        ("EVENT_MN_HEADWATERS_REPORT", "MnWTamarackGuideMineText"),
        (
            "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
            "MnWTamarackGuideDepotText",
        ),
        ("EVENT_MN_DEPOT_CLEARED", "MnWTamarackGuideDuluthText"),
        ("EVENT_MN_HARBOR_INSPECTED", "MnWTamarackGuideBoundaryText"),
        ("EVENT_MN_ARC_LINK_SENT", "MnWTamarackGuideIntakeText"),
        ("EVENT_MN_SURVEY_RESTORED", "MnWTamarackGuideRestoredText"),
        ("EVENT_MN_LEAGUE_CHAMPION", "MnWTamarackGuideChampionText"),
    ] {
        let (root, runtime, mut c) = fixture(
            "MnWinonaFieldStation",
            (4, 2),
            &[
                "EVENT_MN_HOME_INTRO",
                "EVENT_MN_TAMARACK_INTRO",
                "EVENT_MN_STARTER_RECEIVED",
                "EVENT_MN_R01_PARCEL_RECEIVED",
                "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
                "EVENT_MN_TAMARACK_EGG_RECEIVED",
                event,
            ],
        );
        continue_in_place(&mut c, &root, &runtime, label);
        press(&mut c, GameButton::Right);
        press(&mut c, GameButton::A);
        if c.snapshot()
            .expect("return interaction")
            .ui
            .text
            .as_ref()
            .is_some_and(|text| text.label == "MnRewardMasterIntroText")
        {
            assert_eq!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .bag
                    .quantity(&runtime.data().items["MASTER_BALL"]),
                0
            );
            finish(&mut c);
            assert_eq!(
                c.shell
                    .shell
                    .session()
                    .state()
                    .bag
                    .quantity(&runtime.data().items["MASTER_BALL"]),
                1
            );
            press(&mut c, GameButton::A);
        }
        capture(&mut c, label);
        read(&mut c, label);
        press(&mut c, GameButton::Start);
        assert!(
            c.snapshot()
                .expect("Start after guidance")
                .ui
                .menu
                .is_some()
        );
    }
}

#[test]
#[ignore = "declared delivery fixture; requires revised external pack and native 2D schedule"]
fn story_review_reed_reaches_each_column_before_his_first_page() {
    for (column, from_south) in [
        (8, true),
        (9, false),
        (9, true),
        (10, false),
        (10, true),
        (11, false),
        (11, true),
    ] {
        let (_, _, c) = fixture(
            "MnRochesterPokecenter1F",
            (column, if from_south { 13 } else { 11 }),
            &[
                "EVENT_MN_HOME_INTRO",
                "EVENT_MN_TAMARACK_INTRO",
                "EVENT_MN_STARTER_RECEIVED",
                "EVENT_MN_R01_PARCEL_RECEIVED",
                "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            ],
        );
        let mut app = integrated_shell_test_app(c.shell);
        let mut approached = false;
        let mut speaking = false;
        for frame in 0..512 {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            if frame == 0 || frame == 8 {
                keys.press(if from_south {
                    KeyCode::ArrowUp
                } else {
                    KeyCode::ArrowDown
                });
            }
            app.update();
            let shell = app.world().resource::<BevyRuntimeShell>();
            assert_eq!(shell.last_error, None);
            approached |= shell.visible_script_movement.is_some();
            let snapshot = shell
                .shell
                .presentation_snapshot()
                .expect("native scene frame");
            if snapshot.ui.text_window_open {
                assert!(approached, "Reed must visibly approach");
                assert!(
                    shell.visible_script_movement.is_none(),
                    "speech must wait for movement"
                );
                let reed = snapshot.visible_object_runtime_tiles["MN_ARC_RC_REED"];
                assert_eq!(
                    (reed.x, reed.y),
                    if column == 8 { (9, 12) } else { (column, 11) }
                );
                assert_eq!(
                    (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
                    (column, 12)
                );
                assert_eq!(
                    snapshot.visible_object_facings["MN_ARC_RC_REED"],
                    if column == 8 {
                        Direction::Left
                    } else {
                        Direction::Down
                    }
                );
                assert_eq!(
                    snapshot.overworld.facing,
                    if column == 8 {
                        Direction::Right
                    } else {
                        Direction::Up
                    }
                );
                assert_eq!(
                    snapshot.ui.text.expect("first speaker").label,
                    "MnArcRCPathsText"
                );
                speaking = true;
                break;
            }
        }
        assert!(
            speaking,
            "the automatic encounter must start in column {column}"
        );
    }
}
