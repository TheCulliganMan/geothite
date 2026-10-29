//! The gray fence keeps shutdown, battle, release and evidence separate.
use std::path::PathBuf;

use crystal_core::input::GameButton;

use super::super::VisibleShellController;
use super::super::northstar_dialogue::acknowledge;
use super::super::northstar_ice_lair::{choice, page, talk};
use super::super::winona::{at, continue_in_place, finish, press, travel};
use super::fixture;

const ROOM: &str = "MnMinneapolisDispatchOffice";
const JOINED: &str = "EVENT_MN_REED_JOINED_ICE";
const POWER: &str = "EVENT_MN_ICE_HOLDING_POWER_OFF";
const ENFORCER: &str = "EVENT_MN_ICE_ENFORCER_DEFEATED";
const RESCUED: &str = "EVENT_MN_ICE_CAPTIVE_RESCUED";
const NOTE: &str = "EVENT_MN_DISPATCH_NOTE";

fn event(c: &VisibleShellController, id: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(id)
        .expect("holding event")
}

fn gate(c: &VisibleShellController) -> u16 {
    c.shell
        .shell
        .session()
        .state()
        .map_block_overrides
        .get(ROOM)
        .and_then(|blocks| blocks.get(&(6, 3)))
        .copied()
        .unwrap_or(0x38)
}

fn capture(c: &mut VisibleShellController, stage: &str) {
    let directory = PathBuf::from(
        std::env::var_os("CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR")
            .expect("isolated actual scene captures"),
    );
    std::fs::create_dir_all(&directory).expect("scene capture directory");
    c.save(directory.join(format!("holding-fence-{stage}.crystalsave")))
        .expect("actual native scene frame");
}

#[test]
#[ignore = "declared rescue prerequisites; requires the gray-fence candidate and isolated saves"]
fn overhaul_holding_fence_requires_shutdown_and_battle_then_opens_on_acknowledgment() {
    for (flags, label) in [
        (vec![JOINED, ENFORCER], "MnNICEPowerLockedText"),
        (vec![JOINED, POWER], "MnMPCellLockedText"),
    ] {
        let (_, r, mut c) = fixture(ROOM, (11, 6), &flags);
        assert_eq!(
            r.data().maps[ROOM].attributes.tileset_name,
            "mn_northstar_holding"
        );
        talk(&mut c, GameButton::Right);
        page(&mut c, label);
        assert_eq!(gate(&c), 0x38);
        assert!(!event(&c, RESCUED) && !event(&c, NOTE));
        press(&mut c, GameButton::Right);
        at(&mut c, ROOM, (11, 6));
        finish(&mut c);
        press(&mut c, GameButton::Right);
        at(&mut c, ROOM, (11, 6));
    }

    let (root, r, mut c) = fixture(ROOM, (11, 6), &[JOINED, POWER, ENFORCER]);
    let before = c.snapshot().expect("declared rescue party and bag");
    talk(&mut c, GameButton::Right);
    choice(&mut c, false);
    finish(&mut c);
    assert_eq!(gate(&c), 0x38);
    assert!(!event(&c, RESCUED) && !event(&c, NOTE));
    capture(&mut c, "closed");
    continue_in_place(&mut c, &root, &r, "overhaul-holding-gate-declined");
    assert_eq!(gate(&c), 0x38);

    talk(&mut c, GameButton::Right);
    choice(&mut c, true);
    page(&mut c, "MnMPCellReleaseText");
    assert_eq!(gate(&c), 0x38);
    assert!(!event(&c, RESCUED) && !event(&c, NOTE));
    capture(&mut c, "release-conversation");
    acknowledge(&mut c, "MS03_RESCUE", "MnMPCellReleaseText", RESCUED);
    finish(&mut c);
    assert_eq!(gate(&c), 5);
    assert!(event(&c, RESCUED) && !event(&c, NOTE));
    let after = c.snapshot().expect("Ellis visibly left by the carpet");
    assert!(
        !after
            .visible_objects
            .iter()
            .any(|o| o.object_identifier.as_deref() == Some("MN_ICE_CAPTIVE_DISPATCHER"))
    );
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
    capture(&mut c, "open");
    continue_in_place(&mut c, &root, &r, "overhaul-holding-released");
    assert_eq!(gate(&c), 5);
    assert!(event(&c, RESCUED) && !event(&c, NOTE));
    travel(&mut c, GameButton::Right, ROOM, (12, 6));
    travel(&mut c, GameButton::Right, ROOM, (13, 6));
    travel(&mut c, GameButton::Left, ROOM, (12, 6));
    travel(&mut c, GameButton::Left, ROOM, (11, 6));
    talk(&mut c, GameButton::Right);
    page(&mut c, "MnMPCellAlreadyText");
    finish(&mut c);
    assert_eq!(gate(&c), 5);
    assert!(event(&c, RESCUED) && !event(&c, NOTE));
}
