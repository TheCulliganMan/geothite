//! Read Saven's first-match tip after opening the upper crossing at the far bank.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, Left, Up};
use crystal_core::models::Dv;
use crystal_runtime::RuntimeBattleKind;

use super::super::brainerd_guide::capture;
use super::super::duluth_story::press;
use super::{FANS, at, choose_fan, event, expect_page, fixture, load, set_earned, settle, travel};
use crate::bevy_shell::{
    RuntimeBadgeRegion, mark_runtime_snapshot_dirty, visible_field_dialog_pages,
};

const COURT: &str = "EVENT_MN_GYM_5_COURT_OPEN";
const SEEN: &str = "MnStCloudGymLeaderSeenText";

#[test]
#[ignore = "requires Saven's revised court conversation in MN_BADGE_TRAILS_PACK"]
fn saven_court_teaches_the_dig_switch_before_the_first_battle() {
    let (root, runtime) = load();
    // Start beside Saven solely to exercise her guard when the fans are unqualified.
    let mut blocked = fixture(&root, &runtime, FANS, (4, 3));
    blocked
        .shell
        .shell
        .award_badge(RuntimeBadgeRegion::Johto, 3)
        .expect("Capitol Badge");
    mark_runtime_snapshot_dirty(&mut blocked.shell);
    assert!(!event(&blocked, COURT));
    press(&mut blocked, Left);
    press(&mut blocked, A);
    let denial = blocked
        .snapshot()
        .expect("unqualified audience")
        .ui
        .text
        .expect("Saven's directions");
    assert_eq!(denial.label, "MnStCloudGymRequirementsText");
    assert_eq!(
        denial.asm_text.as_deref(),
        Some("Show your CAPITOL\nBADGE, then beat")
    );
    settle(&mut blocked, "leader leaves the match closed");

    // Wyrel is beaten and the lower fan was turned east to reach this bank.
    // Operate the upper dial, ride it back west and speak to Saven without fighting Hallen.
    let mut c = fixture(&root, &runtime, FANS, (15, 4));
    for slot in 0..4 {
        c.shell
            .shell
            .award_badge(RuntimeBadgeRegion::Johto, slot)
            .expect("four prior badges");
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(
        state.storage.party.add_pokemon(
            runtime
                .data()
                .create_pokemon("BRISTUB", 24, Dv::default())
                .expect("regional Route 5 partner")
        )
    );
    state.sync_party_from_storage();
    set_earned(
        &mut c,
        &[
            "EVENT_MN_GYM_5_TRAINER_1_DEFEATED",
            "EVENT_MN_GYM_5_V2_RELAY_EAST",
        ],
    );
    let (state, overworld) = c.shell.shell.session_mut().state_and_overworld_mut();
    runtime
        .data()
        .apply_map_setup_callbacks(state, overworld, FANS, "MAPSETUP_DOOR")
        .expect("restore eastbound fans");
    c.wait_frames(2).expect("arrive on the far bank");
    assert!(!event(&c, COURT));
    choose_fan(&mut c, true);
    expect_page(
        &mut c,
        "MnStCloudGymUPPER_EAST_BANKRoutingWESTSuccessText",
        "Both fans: WEST.\nFollow the arrows.",
    );
    assert!(event(&c, COURT));
    assert!(!event(&c, "EVENT_MN_GYM_5_TRAINER_2_DEFEATED"));
    travel(&mut c, Left, FANS, (12, 4));
    travel(&mut c, Up, FANS, (12, 3));
    press(&mut c, Left);
    if c.snapshot()
        .expect("face the westbound fan")
        .overworld
        .tile
        .x
        == 12
    {
        press(&mut c, Left);
    }
    at(&mut c, FANS, (7, 3));
    travel(&mut c, Left, FANS, (4, 3));
    press(&mut c, Left);
    at(&mut c, FANS, (4, 3));
    assert!(
        c.snapshot()
            .expect("the leader waits to be addressed")
            .battle
            .is_none()
    );
    press(&mut c, A);
    let expected = [
        "SAVEN: Feel that?\nThe wind turned.",
        "If BANKWING digs,\nsend in a flier.",
        "Let me see how\nquickly you adapt!",
    ];
    let opening = c.shell.shell.presentation_snapshot().expect("Saven's text");
    assert_eq!(
        visible_field_dialog_pages(&opening, &c.shell).expect("native pages"),
        expected
    );
    let review = std::env::var_os("MN_SAVEN_REVIEW_DIR").map(PathBuf::from);
    for (index, words) in expected.iter().enumerate() {
        let snapshot = c.snapshot().expect("current page");
        assert!(snapshot.battle.is_none());
        let text = snapshot.ui.text.expect("Saven speaks");
        assert_eq!(text.label, SEEN);
        assert_eq!(text.asm_text.as_deref(), Some(*words));
        if index == 1 {
            c = capture(c, review.as_deref(), "saven-bankwing-tip.png");
        }
        press(&mut c, A);
    }
    for _ in 0..12 {
        if let Some(battle) = c.snapshot().expect("start the match").battle {
            assert!(
                matches!(&battle.kind, RuntimeBattleKind::Trainer { trainer_class, trainer_id, .. } if trainer_class == "MN_LEADER_SAVEN" && trainer_id == "MN_GYM_5")
            );
            assert_eq!(battle.enemy_pokemon.species.id, "DEEPULSE");
            let team: Vec<_> = battle
                .enemy_party
                .iter()
                .map(|p| (p.species.id.as_str(), p.level))
                .collect();
            assert_eq!(team, [("DEEPULSE", 28), ("BANKWING", 29), ("SILOHAWK", 30)]);
            return;
        }
        press(&mut c, A);
    }
    panic!("the first battle did not follow the third page");
}
