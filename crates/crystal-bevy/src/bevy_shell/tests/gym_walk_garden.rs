//! The Bemidji guide's real HM handoff and directions for the two possible keeper orders.
use super::super::mark_runtime_snapshot_dirty;
use super::badge_trails::load;
use super::interior_life::{fixture, talk};
use super::r10_gallery::continue_save;

#[test]
#[ignore = "requires an installed Gym Walk edition in MN_BADGE_TRAILS_PACK"]
fn gym_walk_garden_guide_remembers_each_keeper_and_hands_over_one_cut() {
    let (root, runtime) = load();
    let cut = &runtime.data().items["HM_CUT"];
    let gym = "MnBemidjiGym";
    let guide = (9, 17);
    let mut first = fixture(&root, &runtime, gym, (8, 17));
    assert_eq!(first.shell.shell.session().state().bag.quantity(cut), 0);
    let pages = talk(&mut first, guide);
    let gift = pages.find("MnGardenCutText").expect("actual Cut gift page");
    let fork = pages
        .find("MnGardenForkText")
        .expect("actual left/right directions");
    assert!(
        gift < fork && pages.contains("TM/HM pocket.") && pages.contains("path to ELVREN."),
        "{pages}"
    );
    assert_eq!(first.shell.shell.session().state().bag.quantity(cut), 1);
    assert!(
        first
            .shell
            .shell
            .session()
            .state()
            .flags
            .is_event_flag_set("EVENT_MN_GYM_7_CUT_HM_RECEIVED")
            .expect("HM receipt")
    );
    let mut resumed = continue_save(root.clone(), runtime.clone(), &mut first);
    let again = talk(&mut resumed, guide);
    assert!(
        again.contains("MnGardenForkText") && !again.contains("MnGardenCutText"),
        "{again}"
    );
    assert_eq!(resumed.shell.shell.session().state().bag.quantity(cut), 1);

    const MOSLEN: &str = "EVENT_MN_GYM_7_TRAINER_1_DEFEATED";
    const ELVREN: &str = "EVENT_MN_GYM_7_TRAINER_2_DEFEATED";
    const BRINDEL: &str = "EVENT_MN_GYM_7_TRAINER_3_DEFEATED";
    const ORLIS: &str = "EVENT_MN_GYM_7_DEFEATED";
    let visits: &[(&[&str], &str, &str)] = &[
        (&[BRINDEL], "MnGardenForkText", "BRINDEL only"),
        (&[MOSLEN], "MnGardenNorthText", "Just ELVREN left."),
        (&[ELVREN], "MnGardenWestText", "Just MOSLEN left."),
        (&[MOSLEN, ELVREN], "MnGardenLeaderText", "northern tree."),
        (
            &[MOSLEN, ELVREN, ORLIS],
            "MnGardenReturnText",
            "middle gate",
        ),
    ];
    for &(wins, label, words) in visits {
        let mut c = fixture(&root, &runtime, gym, (8, 17));
        {
            let state = c.shell.shell.session_mut().state_mut();
            // A visitor who learned Cut elsewhere also skips the gift.
            assert!(
                state
                    .bag
                    .add_item(cut, 1)
                    .expect("prior HM in the TM pocket")
            );
            for flag in wins {
                state
                    .flags
                    .set_event_flag(flag, true)
                    .expect("fixture gym victory");
            }
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let pages = talk(&mut c, guide);
        assert!(
            pages.contains(label) && pages.contains(words) && !pages.contains("MnGardenCutText"),
            "{wins:?}: {pages}"
        );
        assert_eq!(c.shell.shell.session().state().bag.quantity(cut), 1);
        assert!(
            !c.shell
                .shell
                .session()
                .state()
                .flags
                .is_event_flag_set("EVENT_MN_GYM_7_CUT_HM_RECEIVED")
                .expect("no duplicate handoff")
        );
    }
}
