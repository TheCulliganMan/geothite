//! Native shaft course, called inside the complete foreman-to-Finn story test.
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::VisibleShellController;
use super::northstar_ice_lair::{choice, event, path, save, talk, walk};
use super::winona::{finish, travel};

fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/lairs/iron-workings/network.json"
    ))
    .expect("authored mine network")
}
fn point(v: &Value) -> (i16, i16) {
    (
        v[0].as_i64().expect("x") as i16,
        v[1].as_i64().expect("y") as i16,
    )
}
fn cross(c: &mut VisibleShellController, r: &CrystalRuntime, n: &Value, id: &str, reverse: bool) {
    let link = n["links"]
        .as_array()
        .expect("links")
        .iter()
        .find(|l| l["id"] == id)
        .expect("ladder");
    let (from, to) = if reverse {
        (&link["b"], &link["a"])
    } else {
        (&link["a"], &link["b"])
    };
    assert_eq!(
        c.snapshot().expect("shaft").overworld.map_name,
        from["map"].as_str().expect("map")
    );
    let (x, y) = point(&from["active"]);
    walk(c, r, (x, y + 1));
    travel(
        c,
        GameButton::Up,
        to["map"].as_str().expect("destination"),
        point(&to["arrival"]),
    );
}

pub(super) fn enter_gallery(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    root: &crystal_assets::AssetRoot,
) {
    let n = source();
    cross(c, r, &n, "mouth", false);
    assert!(path(c, r, (28, 23)).is_none(), "haulage crosscut closed");
    walk(c, r, (5, 16));
    talk(c, GameButton::Up);
    finish(c);
    // The service loop reaches a different compartment, not the vent control.
    cross(c, r, &n, "service_loop", false);
    assert!(path(c, r, (13, 16)).is_none(), "vent crosscut closed");
    walk(c, r, (24, 15));
    talk(c, GameButton::Right);
    finish(c);
    cross(c, r, &n, "service_rise", false);
    assert!(
        path(c, r, (35, 4)).is_none(),
        "two live circuits hold sump shutter"
    );
    cross(c, r, &n, "east_drop", true);
    walk(c, r, (29, 12));
    talk(c, GameButton::Up);
    choice(c, false);
    finish(c);
    assert!(!event(c, "EVENT_MN_IRON_HAUL_PRESSURE_OFF"));
    *c = save(c, root, r, "iron-pressure-declined");
    talk(c, GameButton::Up);
    choice(c, true);
    finish(c);
    assert!(event(c, "EVENT_MN_IRON_HAUL_PRESSURE_OFF"));
    assert!(
        path(c, r, (4, 23)).is_some(),
        "haulage valve opens useful crosscut"
    );
    cross(c, r, &n, "east_drop", false);
    assert!(
        path(c, r, (35, 4)).is_none(),
        "one circuit cannot open pump access"
    );
    cross(c, r, &n, "east_drop", true);
    cross(c, r, &n, "haul_down", false);
    cross(c, r, &n, "sump_rise", false);
    walk(c, r, (13, 16));
    talk(c, GameButton::Up);
    choice(c, true);
    finish(c);
    assert!(event(c, "EVENT_MN_IRON_VENT_PRESSURE_OFF"));
    assert!(
        path(c, r, (26, 21)).is_some(),
        "vent valve opens second crosscut"
    );
    *c = save(c, root, r, "iron-both-circuits-off");
    assert!(
        path(c, r, (26, 21)).is_some(),
        "crosscut persists through Continue"
    );
    cross(c, r, &n, "upper_crossing", false);
    cross(c, r, &n, "east_drop", false);
    assert!(
        path(c, r, (35, 4)).is_some(),
        "both circuits open sump shutter on entry"
    );
    walk(c, r, (33, 8));
    *c = save(c, root, r, "iron-sump-shutter-open");
    cross(c, r, &n, "pump_access", false);
    for flag in [
        "EVENT_MN_DRILL_FEED_ISOLATED",
        "EVENT_MN_DRILL_TEST_WITNESSED",
        "EVENT_MN_DRILL_STOPPED",
        "EVENT_MN_MINE_INSTRUMENT_RECOVERED",
    ] {
        assert!(
            !event(c, flag),
            "shaft controls cannot grant later story result"
        );
    }
}

pub(super) fn leave_gallery(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let n = source();
    cross(c, r, &n, "pump_access", true);
    cross(c, r, &n, "east_drop", true);
    // Return through the pressure-opened crosscut rather than the first route.
    cross(c, r, &n, "mouth", true);
}
