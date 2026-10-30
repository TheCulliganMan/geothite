//! Rescue directions cannot substitute for reading the physical evidence.
use crystal_core::input::GameButton;

use super::super::native_city_ports::city_port;
use super::super::northstar_dispatch_course::{has_physical_copy, read_copy_and_return_home};
use super::super::northstar_ice_lair::{event, page, talk, walk};
use super::super::winona::{finish, load, press, travel};
use super::fixture;

const JOINED: &str = "EVENT_MN_REED_JOINED_ICE";
const POWER: &str = "EVENT_MN_ICE_HOLDING_POWER_OFF";
const ENFORCER: &str = "EVENT_MN_ICE_ENFORCER_DEFEATED";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

#[test]
#[ignore = "declared completed rescue; requires the current physical-copy candidate and isolated saves"]
fn overhaul_ellis_directs_player_to_physical_desk_copy_and_rewards_only_its_acknowledgment() {
    let (_, runtime) = load();
    let (outside, inside) = city_port(&runtime, "MnMinneapolis", "MnMinneapolisDispatcherHouse");
    let (root, r, mut c) = fixture(
        "MnMinneapolis",
        outside,
        &[JOINED, POWER, ENFORCER, RESCUED],
    );
    assert!(has_physical_copy(&r));
    // Home's presence callback must run after the declared rescue. Enter it
    // through the real doorway, as the earned campaign does.
    travel(
        &mut c,
        GameButton::Up,
        "MnMinneapolisDispatcherHouse",
        inside,
    );
    walk(&mut c, &r, (7, 6));
    let before = c.snapshot().expect("declared rescue party");
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNICEEllisQuestText");
    assert!(!event(&c, NOTE));
    press(&mut c, GameButton::Left);
    let blocked = c.snapshot().expect("Ellis owns input");
    assert_eq!(blocked.overworld.tile, before.overworld.tile);
    finish(&mut c);
    assert!(!event(&c, NOTE));
    assert_eq!(
        c.snapshot().expect("directions do not reward items").bag,
        before.bag
    );
    read_copy_and_return_home(&mut c, &root, &r);
    walk(&mut c, &r, (7, 6));
    talk(&mut c, GameButton::Up);
    page(&mut c, "MnNICEEllisAfterText");
    finish(&mut c);
    assert!(event(&c, NOTE));
}

#[test]
#[ignore = "declared incomplete rescue; requires the current physical-copy candidate"]
fn overhaul_desk_copy_requires_battle_shutdown_and_release() {
    for missing in [POWER, ENFORCER, RESCUED] {
        let flags = [JOINED, POWER, ENFORCER, RESCUED]
            .into_iter()
            .filter(|flag| *flag != missing)
            .collect::<Vec<_>>();
        let (_, _, mut c) = fixture("MnMinneapolisDispatchOffice", (10, 11), &flags);
        talk(&mut c, GameButton::Left);
        page(&mut c, "MnMPDispatchLockedText");
        finish(&mut c);
        assert!(!event(&c, NOTE), "missing {missing}");
    }
}
