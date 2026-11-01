//! Revised-room input courses for the earned Winona party; no state fixtures.
use std::path::PathBuf;
use std::sync::OnceLock;

use crystal_assets::read_minnesota_gym_design;
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::VisibleShellController;
use super::minnesota_gyms::{interact, walk_to};
use super::winona::{at, finish, press, travel};

pub(super) fn selected(r: &CrystalRuntime) -> bool {
    r.pack_identity()
        .runtime_modpack_id
        .split('+')
        .any(|id| id == "minnesota-current-eight-gyms-v1")
}

pub(super) fn design(r: &CrystalRuntime, slot: usize) -> &'static Value {
    let current = selected(r);
    let reviewed_base = r
        .pack_identity()
        .runtime_modpack_id
        .split('+')
        .any(|id| id == "minnesota-reviewed-eight-gyms-refresh-v1");
    assert!(
        current || reviewed_base,
        "reviewed gym publication required"
    );
    static CURRENT: OnceLock<Value> = OnceLock::new();
    static BASE: OnceLock<Value> = OnceLock::new();
    let cache = if current { &CURRENT } else { &BASE };
    let design = cache.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../planning/minnesota/gyms")
            .join(if current {
                "current.json"
            } else {
                "interiors.json"
            });
        serde_json::from_str(&read_minnesota_gym_design(path).expect("current gym source"))
            .expect("selected gym JSON")
    });
    let gym = &design["gyms"][slot - 1];
    let room = &r.data().maps[gym["id"].as_str().expect("native gym id")];
    assert_eq!(
        room.attributes.width,
        gym["dimensions_metatiles"][0].as_u64().expect("width") as u16
    );
    assert_eq!(
        room.attributes.height,
        gym["dimensions_metatiles"][1].as_u64().expect("height") as u16
    );
    let arrival = &room.events.warps[2];
    assert_eq!((arrival.x, arrival.y), point(&gym["arrival_core"]));
    for trainer in gym["trainers"].as_array().expect("qualifying trainers") {
        let at = point(&trainer["at"]);
        assert!(
            room.objects.iter().any(
                |o| (o.x, o.y) == at && o.script == trainer["id"].as_str().expect("trainer id")
            ),
            "published trainer differs from current source"
        );
    }
    gym
}

fn point(value: &Value) -> (u16, u16) {
    (
        value[0].as_u64().expect("x") as u16,
        value[1].as_u64().expect("y") as u16,
    )
}

pub(super) fn arrival(r: &CrystalRuntime, slot: usize) -> (i16, i16) {
    let (x, y) = point(&design(r, slot)["arrival_core"]);
    (x as i16, y as i16)
}

pub(super) fn opponent(r: &CrystalRuntime, slot: usize, id: &str) -> (u16, u16) {
    let gym = design(r, slot);
    if id == format!("MN_GYM_{slot}") {
        point(&gym["leader_at"])
    } else {
        point(
            &gym["trainers"]
                .as_array()
                .expect("trainers")
                .iter()
                .find(|t| t["id"] == id)
                .expect("native opponent")["at"],
        )
    }
}

fn event(c: &VisibleShellController, flag: &str) -> bool {
    c.shell
        .shell
        .session()
        .state()
        .flags
        .is_event_flag_set(flag)
        .expect("native gym event")
}

fn toggle(c: &mut VisibleShellController, r: &CrystalRuntime, flag: &str) {
    let routers = design(r, 1)["routers"].as_array().expect("native switches");
    let router = routers
        .iter()
        .find(|router| {
            router["modes"]
                .as_array()
                .expect("modes")
                .iter()
                .any(|mode| mode["event"] == flag)
        })
        .expect("source switch");
    for _ in 0..2 {
        if event(c, flag) {
            return;
        }
        interact(c, r, point(&router["at"]));
        assert!(c.snapshot().expect("physical switch").ui.menu.is_none());
        finish(c);
    }
    assert!(event(c, flag), "native switch must earn {flag}");
}

pub(super) fn rochester_challenge(c: &mut VisibleShellController, r: &CrystalRuntime, id: &str) {
    if id == "MN_GYM_1_TRAINER_2" {
        toggle(c, r, "EVENT_MN_GYM_1_LOWER_LEFT");
    } else if id == "MN_GYM_1" {
        if !event(c, "EVENT_MN_GYM_1_UPPER_OPEN") {
            toggle(c, r, "EVENT_MN_GYM_1_LOWER_LEFT");
            toggle(c, r, "EVENT_MN_GYM_1_UPPER_OPEN");
        }
        toggle(c, r, "EVENT_MN_GYM_1_LOWER_RIGHT");
        assert!(event(c, "EVENT_MN_GYM_1_COURT_OPEN"));
    }
    interact(c, r, opponent(r, 1, id));
}

pub(super) fn rochester_preview(c: &mut VisibleShellController, r: &CrystalRuntime) {
    let gym = design(r, 1);
    let before = c.snapshot().expect("before first gym visit");
    walk_to(c, r, (5, 10));
    press(c, GameButton::Up);
    at(c, "MnRochesterGym", (5, 10));
    assert!(!event(c, "EVENT_MN_GYM_1_LEFT_LANE_OPEN"));
    interact(c, r, point(&gym["guide_at"]));
    assert!(c.snapshot().expect("visible gym guide").ui.text.is_some());
    finish(c);
    let after = c.snapshot().expect("no progress from guide");
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert!(!event(c, "EVENT_MN_GYM_1_DEFEATED"));
}

pub(super) fn leave(
    c: &mut VisibleShellController,
    r: &CrystalRuntime,
    slot: usize,
    city: &str,
    outside: (i16, i16),
) {
    let (x, y) = point(&design(r, slot)["exit_tiles_core"][0]);
    walk_to(c, r, (x, y - 1));
    travel(c, GameButton::Down, city, outside);
}
