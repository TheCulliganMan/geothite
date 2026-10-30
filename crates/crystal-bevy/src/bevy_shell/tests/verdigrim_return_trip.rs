//! Actual clinic exit, native Fly and camp/shelter/intake return.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use crate::bevy_shell::tests::northstar_ice_lair::walk;
use crate::bevy_shell::tests::r10_gallery::settle;
use crate::bevy_shell::tests::winona::{at, press, travel};
use crate::bevy_shell::{
    PartyAction, PartyFieldMove, StartMenuOption, VisibleShellController, active_fly_destinations,
    visible_party_actions, visible_start_menu_options,
};

pub(super) fn to_intake(c: &mut VisibleShellController, runtime: &CrystalRuntime) {
    let town = runtime.data().maps["MnRochester"]
        .attributes
        .map_constant
        .as_deref()
        .expect("Rochester identity");
    let exit = runtime.data().maps["MnRochesterPokecenter1F"]
        .events
        .warps
        .iter()
        .find(|warp| warp.target_map == town)
        .expect("native clinic street exit");
    let x = i16::try_from(exit.x).expect("clinic x");
    let y = i16::try_from(exit.y).expect("clinic y");
    walk(c, runtime, (x, y - 1));
    for _ in 0..8 {
        if c.snapshot().expect("clinic exit").overworld.map_name == "MnRochester" {
            break;
        }
        press(c, GameButton::Down);
        settle(c, "clinic street arrival");
    }
    assert_eq!(
        c.snapshot().expect("outside before Fly").overworld.map_name,
        "MnRochester"
    );
    press(c, GameButton::Start);
    for _ in 0..32 {
        let snapshot = c.snapshot().expect("Start options");
        let options = visible_start_menu_options(&c.shell, &snapshot);
        if options[c
            .shell
            .start_menu_cursor
            .as_ref()
            .expect("Start cursor")
            .option_index]
            == StartMenuOption::Pokemon
        {
            break;
        }
        press(c, GameButton::Down);
    }
    press(c, GameButton::A);
    for _ in 0..8 {
        if c.shell.party_cursor == 1 {
            break;
        }
        press(c, GameButton::Down);
    }
    assert_eq!(
        c.snapshot().expect("native Fly user").party.slots[1]
            .pokemon
            .species
            .id,
        "LOOBIT"
    );
    press(c, GameButton::A);
    for _ in 0..32 {
        let snapshot = c.snapshot().expect("party field actions");
        let actions = visible_party_actions(&snapshot, &c.shell).expect("native actions");
        if actions[c
            .shell
            .party_action_cursor
            .as_ref()
            .expect("party cursor")
            .option_index]
            == PartyAction::FieldMove(PartyFieldMove::Fly)
        {
            break;
        }
        press(c, GameButton::Down);
    }
    press(c, GameButton::A);
    assert!(c.shell.fly_cursor.is_some(), "native Fly destination menu");
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("visited Fly destinations");
        let destinations =
            active_fly_destinations(&snapshot, &c.shell.shell).expect("destinations");
        let index = c
            .shell
            .fly_cursor
            .as_ref()
            .expect("Fly cursor")
            .option_index;
        if destinations[index].flypoint_flag == "ENGINE_FLYPOINT_MN_BOUNDARY_WATERS" {
            break;
        }
        press(c, GameButton::Down);
    }
    let snapshot = c.snapshot().expect("selected camp");
    let destinations = active_fly_destinations(&snapshot, &c.shell.shell).expect("destinations");
    assert_eq!(
        destinations[c
            .shell
            .fly_cursor
            .as_ref()
            .expect("Fly cursor")
            .option_index]
            .flypoint_flag,
        "ENGINE_FLYPOINT_MN_BOUNDARY_WATERS"
    );
    press(c, GameButton::A);
    assert!(
        c.shell.visible_fly_animation.is_some(),
        "native Fly departure"
    );
    for _ in 0..512 {
        if c.snapshot().expect("Fly arrival").overworld.map_name == "MnBoundaryWaters"
            && c.shell.visible_fly_animation.is_none()
        {
            break;
        }
        c.wait_frames(16).expect("production Fly clock");
    }
    assert_eq!(
        c.snapshot()
            .expect("actual camp landing")
            .overworld
            .map_name,
        "MnBoundaryWaters"
    );
    walk(c, runtime, (43, 20));
    travel(c, GameButton::Up, "MnBoundaryWatersSurveyShelter", (8, 9));
    walk(c, runtime, (13, 7));
    travel(c, GameButton::Down, "MnBoundaryIntake", (4, 19));
    walk(c, runtime, (24, 6));
    at(c, "MnBoundaryIntake", (24, 6));
}
