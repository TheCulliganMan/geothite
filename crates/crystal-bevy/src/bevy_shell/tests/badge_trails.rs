//! Exercise the player-facing directions on either installed Badge Trails edition.
#[path = "leech_lake_trainer_partners.rs"]
mod leech_lake_trainer_partners;
#[path = "regional_direction_boards.rs"]
mod regional_direction_boards;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::RuntimeBadgeRegion;
use std::path::PathBuf;

use super::interior_life::{fixture, talk};
use super::r10_gallery::continue_save;
use super::winona::travel;

pub(super) fn load() -> (AssetRoot, CrystalRuntime) {
    let path =
        PathBuf::from(std::env::var_os("MN_BADGE_TRAILS_PACK").expect("Badge Trails archive"));
    let root = AssetRoot::new(path.parent().expect("pack directory"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified Badge Trails archive"),
    )
    .expect("installed regional maps");
    (root, runtime)
}

#[test]
#[ignore = "requires an installed Badge Trails archive in MN_BADGE_TRAILS_PACK"]
fn badge_trails_leaders_answer_the_players_current_journey() {
    let (root, runtime) = load();
    let visit = |map: &str, badge_count: usize, events: &[&str], label: &str, phrase: &str| {
        let actor = runtime.data().maps[map]
            .objects
            .iter()
            .find(|actor| actor.script == format!("{map}Leader"))
            .expect("gym leader");
        let location = (actor.x as i16, actor.y as i16);
        let mut controller = fixture(&root, &runtime, map, (location.0 - 1, location.1));
        for slot in 0..badge_count {
            controller
                .shell
                .shell
                .award_badge(RuntimeBadgeRegion::Johto, slot)
                .expect("earned badge");
        }
        for flag in events {
            controller
                .shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(flag, true)
                .expect("journey fixture");
        }
        super::super::mark_runtime_snapshot_dirty(&mut controller.shell);
        let pages = talk(&mut controller, location);
        assert!(
            pages.contains(label) && pages.contains(phrase),
            "{map} expected {label} with {phrase}: {pages}"
        );
        assert!(
            controller
                .snapshot()
                .expect("conversation ended")
                .battle
                .is_none()
        );
    };
    visit(
        "MnStPaulGym",
        2,
        &["EVENT_MN_GYM_4_COURT_OPEN"],
        "MnStPaulGymRequirementsText",
        "AVRIN",
    );
    visit("MnStPaulGym", 3, &[], "MnStPaulGymGuideText", "either wing");
    visit(
        "MnDuluthGym",
        6,
        &["EVENT_MN_GYM_8_COURT_OPEN"],
        "MnDuluthGymRequirementsText",
        "BEMIDJI",
    );
    visit(
        "MnStPaulGym",
        4,
        &[
            "EVENT_MN_GYM_4_DEFEATED",
            "EVENT_MN_GYM_4_TM_RECEIVED",
            "EVENT_MN_DISPATCH_NOTE",
        ],
        "MnBadgeBrannicMinaText",
        "MINA",
    );
    visit(
        "MnStPaulGym",
        4,
        &[
            "EVENT_MN_GYM_4_DEFEATED",
            "EVENT_MN_GYM_4_TM_RECEIVED",
            "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        ],
        "MnBadgeBrannicWindText",
        "ROUTE 5",
    );
    visit(
        "MnStPaulGym",
        5,
        &[
            "EVENT_MN_GYM_4_DEFEATED",
            "EVENT_MN_GYM_4_TM_RECEIVED",
            "EVENT_MN_DEEP_BORE_MANIFEST_READ",
        ],
        "MnBadgeBrannicBreezeText",
        "fans on",
    );
    visit(
        "MnBrainerdGym",
        6,
        &[
            "EVENT_MN_GYM_6_DEFEATED",
            "EVENT_MN_GYM_6_TM_RECEIVED",
            "EVENT_MN_BRAINERD_REPORT",
        ],
        "MnBadgeSivrenItascaText",
        "little bridge",
    );
    visit(
        "MnBrainerdGym",
        6,
        &[
            "EVENT_MN_GYM_6_DEFEATED",
            "EVENT_MN_GYM_6_TM_RECEIVED",
            "EVENT_MN_HEADWATERS_READING",
        ],
        "MnBadgeSivrenOfficerText",
        "field officer",
    );
    visit(
        "MnBrainerdGym",
        7,
        &[
            "EVENT_MN_GYM_6_DEFEATED",
            "EVENT_MN_GYM_6_TM_RECEIVED",
            "EVENT_MN_HEADWATERS_REPORT",
        ],
        "MnBadgeSivrenWakeText",
        "TORROUT",
    );
}

#[test]
#[ignore = "requires an installed Badge Trails archive in MN_BADGE_TRAILS_PACK"]
fn badge_trails_hollow_road_exits_and_gallery_ladder_return_after_continue() {
    let (root, runtime) = load();
    const HOLLOW: &str = "MnR01Cave1F";
    const ROAD: &str = "MnR01C00R00";
    for (threshold, landing) in [((9, 11), (18, 18)), ((31, 13), (36, 20))] {
        let inland = (threshold.0, threshold.1 - 1);
        let mut c = fixture(&root, &runtime, HOLLOW, inland);
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate passage from optional cave encounters");
        travel(&mut c, GameButton::Down, ROAD, landing);
        travel(&mut c, GameButton::Up, HOLLOW, threshold);
        let mut c = continue_save(root.clone(), runtime.clone(), &mut c);
        travel(&mut c, GameButton::Up, HOLLOW, inland);
        travel(&mut c, GameButton::Down, ROAD, landing);
    }
    let mut c = fixture(&root, &runtime, HOLLOW, (10, 9));
    c.shell
        .shell
        .session_mut()
        .state_mut()
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
        .expect("isolate gallery ladder");
    travel(
        &mut c,
        GameButton::Right,
        "MnDriftlessLampGallery",
        (18, 17),
    );
    let mut c = continue_save(root, runtime, &mut c);
    travel(&mut c, GameButton::Right, HOLLOW, (10, 9));
}
