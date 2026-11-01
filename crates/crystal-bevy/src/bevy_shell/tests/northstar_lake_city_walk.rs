//! Actual street approaches around moving residents; no skipped movement steps.
use crystal_runtime::CrystalRuntime;

use super::{CITY, VisibleShellController, finish, press};

pub(super) fn walk(c: &mut VisibleShellController, runtime: &CrystalRuntime, to: (i16, i16)) {
    for _ in 0..1024 {
        finish(c);
        let here = c.snapshot().expect("live Lake street").overworld;
        assert_eq!(
            here.map_name, CITY,
            "street walk must not use an unintended door"
        );
        if (here.tile.x, here.tile.y) == to {
            return;
        }
        let next = super::super::super::northstar_ice_lair::path(c, runtime, to)
            .and_then(|path| path.first().copied());
        if let Some(button) = next {
            press(c, button);
        } else {
            c.wait_frames(24).expect("wait for a passing city resident");
        }
    }
    let actual = c.snapshot().expect("bounded Lake street state");
    panic!(
        "Lake street did not reach {to:?}: {:?} {:?}",
        actual.overworld, actual.ui
    );
}
