//! Walking inputs at the actual Rochester and Mankato stair shoulders.
use crystal_assets::AssetRoot;
use crystal_core::input::GameButton::{self, Down, Left, Right, Up};
use crystal_runtime::CrystalRuntime;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::badge_trails::load;
use super::interior_life::fixture;
use super::winona::{at, press, travel};

fn enter(
    root: &AssetRoot,
    runtime: &CrystalRuntime,
    city: &str,
    doorstep: (i16, i16),
    arrival: (i16, i16),
    wins: &[&str],
) -> VisibleShellController {
    let mut c = fixture(root, runtime, city, doorstep);
    // Supply the student wins outside. The public door and the normal map
    // callback must install the earned stair before these movement checks.
    for flag in wins {
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("existing trainer victory");
    }
    mark_runtime_snapshot_dirty(&mut c.shell);
    travel(&mut c, Up, &format!("{city}Gym"), arrival);
    c
}

fn blocked(c: &mut VisibleShellController, direction: GameButton, map: &str, position: (i16, i16)) {
    press(c, direction);
    press(c, direction);
    at(c, map, position);
}

#[test]
#[ignore = "requires a Gym Walk edition in MN_BADGE_TRAILS_PACK"]
fn gym_walk_court_and_quarry_require_the_visible_stairs() {
    let (root, runtime) = load();
    let mut c = enter(
        &root,
        &runtime,
        "MnRochester",
        (26, 18),
        (12, 15),
        &[
            "EVENT_MN_GYM_1_TRAINER_1_DEFEATED",
            "EVENT_MN_GYM_1_TRAINER_2_DEFEATED",
        ],
    );
    let room = "MnRochesterGym";
    travel(&mut c, Right, room, (13, 15));
    travel(&mut c, Up, room, (13, 12));
    travel(&mut c, Right, room, (15, 12));
    travel(&mut c, Up, room, (15, 7));
    blocked(&mut c, Up, room, (15, 7));
    travel(&mut c, Left, room, (14, 7));
    travel(&mut c, Up, room, (14, 6));
    travel(&mut c, Left, room, (13, 6));

    let mut c = enter(
        &root,
        &runtime,
        "MnMankato",
        (10, 12),
        (6, 17),
        &[
            "EVENT_MN_GYM_2_TRAINER_1_DEFEATED",
            "EVENT_MN_GYM_2_TRAINER_2_DEFEATED",
        ],
    );
    let room = "MnMankatoGym";
    travel(&mut c, Left, room, (3, 17));
    travel(&mut c, Up, room, (3, 13));
    travel(&mut c, Up, room, (3, 9));
    travel(&mut c, Left, room, (1, 9));
    travel(&mut c, Up, room, (1, 8));
    blocked(&mut c, Up, room, (1, 8));
    travel(&mut c, Right, room, (2, 8));
    travel(&mut c, Up, room, (2, 5));
    travel(&mut c, Right, room, (6, 5));
    travel(&mut c, Down, room, (6, 6));
    travel(&mut c, Down, room, (6, 8));
    travel(&mut c, Down, room, (6, 12));
    travel(&mut c, Right, room, (9, 12));
    travel(&mut c, Right, room, (12, 12));
    travel(&mut c, Up, room, (12, 7));
    blocked(&mut c, Left, room, (12, 7));
    travel(&mut c, Up, room, (12, 6));
    travel(&mut c, Left, room, (11, 6));
    travel(&mut c, Up, room, (11, 4));
}
