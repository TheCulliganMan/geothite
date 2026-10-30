//! Read the desk copy with ordinary input, then return to Ellis's home.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::authored_route_fixture::indoor_walk;
use super::native_city_ports::city_port;
use super::northstar_dialogue::acknowledge;
use super::northstar_ice_lair::{event, page, talk, walk};
use super::winona::{continue_in_place, finish, travel};

const OFFICE: &str = "MnMinneapolisDispatchOffice";
const HOME: &str = "MnMinneapolisDispatcherHouse";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

pub(super) fn has_physical_copy(r: &CrystalRuntime) -> bool {
    r.data().maps[OFFICE].scripts["MnMPDispatchCopy"]
        .as_array()
        .expect("office copy program")
        .iter()
        .any(|row| row["command"] == "setevent" && row["args"][0] == NOTE)
}

pub(super) fn read_copy_and_return_home(
    c: &mut VisibleShellController,
    root: &AssetRoot,
    r: &CrystalRuntime,
) {
    assert!(!event(c, NOTE));
    let departure = c.snapshot().expect("Ellis provided directions only");
    continue_in_place(c, root, r, "campaign-ellis-directions-only");
    assert!(!event(c, NOTE));
    let (home_outside, home_inside) = city_port(r, "MnMinneapolis", HOME);
    let (office_outside, office_inside) = city_port(r, "MnMinneapolis", OFFICE);
    walk(c, r, (5, 6));
    travel(c, GameButton::Down, "MnMinneapolis", home_outside);
    indoor_walk(c, r, office_outside);
    travel(c, GameButton::Up, OFFICE, office_inside);
    walk(c, r, (10, 11));
    // Walking can apply ordinary poison damage. Isolate the conversation's
    // party invariant from the earned journey to the physical desk.
    let before = c.snapshot().expect("standing beside the office desk");
    talk(c, GameButton::Left);
    page(c, "MnMPDispatchReadText");
    assert!(!event(c, NOTE));
    acknowledge(c, "MS03_RESCUE", "MnMPDispatchReadText", NOTE);
    finish(c);
    assert!(event(c, NOTE));
    let after = c.snapshot().expect("copied shipment details");
    assert_eq!(before.party, after.party);
    assert_eq!(departure.bag, after.bag);
    continue_in_place(c, root, r, "campaign-office-copy-acknowledged");
    talk(c, GameButton::Left);
    page(c, "MnMPDispatchDoneText");
    finish(c);
    assert!(event(c, NOTE));
    assert_eq!(c.snapshot().expect("repeat copy").bag, departure.bag);
    walk(c, r, (10, 14));
    travel(c, GameButton::Down, "MnMinneapolis", office_outside);
    indoor_walk(c, r, home_outside);
    travel(c, GameButton::Up, HOME, home_inside);
}
