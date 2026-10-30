//! Continue the actual reached cargo save through the expanded HQ and evidence.
use std::path::PathBuf;

use crystal_core::input::GameButton;

use super::super::authored_route_fixture::indoor_walk;
use super::super::native_city_ports::city_port;
use super::super::northstar_dispatch_course;
use super::super::northstar_ice_lair::{event, page, save, talk, walk};
use super::super::winona::{continue_in_place, finish, load, travel};
use super::VisibleShellController;

#[test]
#[ignore = "requires the same-pack reached CRYSTAL_ICE_CARGO_CHECKPOINT and isolated native saves"]
fn ice_headquarters_reached_cargo_continues_all_floors_rescue_home_and_physical_evidence() {
    let (root, runtime) = load();
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_ICE_CARGO_CHECKPOINT")
            .expect("actual cargo checkpoint from the acted recruitment/controller course"),
    );
    let mut controller =
        VisibleShellController::load_save(root.clone(), runtime.clone(), path, None)
            .expect("same-pack Continue at the reached cargo power-off scene");
    assert_eq!(
        controller
            .snapshot()
            .expect("actual cargo Continue")
            .overworld
            .map_name,
        "MnIceCargoBasement"
    );
    for flag in ["EVENT_MN_REED_JOINED_ICE", "EVENT_MN_ICE_CARGO_FEED_OFF"] {
        assert!(event(&controller, flag));
    }
    for flag in [
        super::KEY,
        super::POWER,
        super::RESCUED,
        "EVENT_MN_DISPATCH_NOTE",
    ] {
        assert!(!event(&controller, flag));
    }
    // No party, position, flag, bag, HP, PP, storage or RNG writes. The prior
    // chapter's actual party and spent resources continue through native input.
    super::cross(
        &mut controller,
        &runtime,
        &super::source(),
        "cargo_stair",
        false,
    );
    controller = super::play_chapter(controller, &root, &runtime, false, true);
    assert!(event(&controller, super::RESCUED));
    assert!(!event(&controller, "EVENT_MN_DISPATCH_NOTE"));
    assert!(northstar_dispatch_course::has_physical_copy(&runtime));

    let office = "MnMinneapolisDispatchOffice";
    let home = "MnMinneapolisDispatcherHouse";
    let (office_outside, _) = city_port(&runtime, "MnMinneapolis", office);
    let (home_outside, home_inside) = city_port(&runtime, "MnMinneapolis", home);
    walk(&mut controller, &runtime, (10, 14));
    travel(
        &mut controller,
        GameButton::Down,
        "MnMinneapolis",
        office_outside,
    );
    indoor_walk(&mut controller, &runtime, home_outside);
    travel(&mut controller, GameButton::Up, home, home_inside);
    let witness = controller
        .snapshot()
        .expect("rescued dispatcher at home")
        .visible_objects
        .into_iter()
        .find(|object| object.object_identifier.as_deref() == Some("MN_MP_RESCUED_DISPATCHER"))
        .expect("actual rescued witness");
    let approach = (witness.x as i16, witness.y as i16 + 1);
    walk(&mut controller, &runtime, approach);
    talk(&mut controller, GameButton::Up);
    page(&mut controller, "MnNICEEllisQuestText");
    assert!(!event(&controller, "EVENT_MN_DISPATCH_NOTE"));
    finish(&mut controller);
    assert!(
        !event(&controller, "EVENT_MN_DISPATCH_NOTE"),
        "the witness gives directions; the physical desk copy is still required"
    );
    northstar_dispatch_course::read_copy_and_return_home(&mut controller, &root, &runtime);
    assert!(event(&controller, "EVENT_MN_DISPATCH_NOTE"));
    walk(&mut controller, &runtime, approach);
    controller = save(
        &mut controller,
        &root,
        &runtime,
        "current-hq-witness-and-desk-evidence",
    );
    talk(&mut controller, GameButton::Up);
    page(&mut controller, "MnNICEEllisAfterText");
    finish(&mut controller);
    continue_in_place(
        &mut controller,
        &root,
        &runtime,
        "current-hq-rescue-and-physical-copy",
    );
    assert!(event(&controller, super::RESCUED));
    assert!(event(&controller, "EVENT_MN_DISPATCH_NOTE"));
    assert!(!controller.has_pending_script_work());
}

// Separate cases let the runner retain each intended-level starter course in
// its own save directory. These declare the prior party/badges; all subsequent
// recruitment, battles, healing, supplies, travel and rescue use native input.
#[test]
#[ignore = "requires the regional HQ pack and an isolated SLIPPERINA course save directory"]
fn ice_hq_starter_balance_slipperina_completes_native_full_gauntlet() {
    super::headquarters_course(Some("SLIPPERINA"));
}

#[test]
#[ignore = "requires the regional HQ pack and an isolated RIMEPELT course save directory"]
fn ice_hq_starter_balance_rimepelt_completes_native_full_gauntlet() {
    super::headquarters_course(Some("RIMEPELT"));
}

#[test]
#[ignore = "requires the regional HQ pack and an isolated PORTABEAU course save directory"]
fn ice_hq_starter_balance_portabeau_completes_native_full_gauntlet() {
    super::headquarters_course(Some("PORTABEAU"));
}
