//! Local road-port fixtures; public approaches and Winona's wardens use input.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::Value;

use super::{
    VisibleShellController, continued, event, load, location, mark_runtime_snapshot_dirty, name,
    point, press, visited,
};

fn checkpoint_move(c: &mut VisibleShellController, button: GameButton, to: (i16, i16)) {
    let city = location(c).0;
    for _ in 0..8 {
        let at = location(c);
        assert_eq!(at.0, city, "checkpoint stays in its city");
        if at.1 == to {
            return;
        }
        press(c, button);
    }
    assert_eq!(location(c), (city, to), "native checkpoint movement");
}

#[test]
#[ignore = "requires the current service in CRYSTAL_INTERIOR_PACK and its city-access.json"]
fn glacier_line_public_roads_reach_all_five_halls_and_return_through_original_checkpoint() {
    let (root, runtime, stops) = load();
    let source = std::env::var_os("CRYSTAL_GLACIER_CITY_ACCESS")
        .map(PathBuf::from)
        .expect("current native city-port contracts");
    let design: Value =
        serde_json::from_str(&std::fs::read_to_string(source).expect("public access contracts"))
            .expect("native public access JSON");
    let paths = design["paths"].as_array().expect("five paths");
    assert_eq!(paths.len(), 5, "every station needs its public approach");
    assert_eq!(stops.len(), paths.len());
    for (stop, access) in stops.iter().zip(paths) {
        assert_eq!(stop["id"], access["id"]);
        assert_eq!(stop["map"], access["station"]);
        let city = access["city"].as_str().expect("public city");
        assert!(
            runtime.data().maps[city]
                .attributes
                .connections
                .iter()
                .any(|connection| {
                    serde_json::to_value(connection).expect("native connection")
                        == access["native_connection"]
                })
        );
        let port = point(&access["city_port"]);
        let mut c = super::super::fixture(&root, &runtime, city, port);
        let state = c.shell.shell.session_mut().state_mut();
        assert!(
            state.storage.party.add_pokemon(
                runtime
                    .data()
                    .create_pokemon("RIMEPELT", 40, Dv::default())
                    .expect("legal local passenger partner")
            )
        );
        state.sync_party_from_storage();
        if let Some(flag) = access["required_initial_event"].as_str() {
            state
                .flags
                .set_event_flag(flag, true)
                .expect("declared original starter prerequisite");
        }
        // Initial road position, legal party and Winona's prerequisite above
        // are local setup. All movement, checkpoints and visits below use input.
        mark_runtime_snapshot_dirty(&mut c.shell);
        if access["checkpoint"].is_object() {
            // The x1 coordinate event finishes its retained two-step inward
            // movement at x3 during this production press.
            checkpoint_move(
                &mut c,
                GameButton::Right,
                point(&access["checkpoint"]["inside"]),
            );
            assert_eq!(
                location(&mut c),
                (city.into(), point(&access["checkpoint"]["inside"]))
            );
        }
        super::super::walk(&mut c, &runtime, point(&access["public_door"]));
        assert_eq!(
            location(&mut c),
            (name(stop).into(), point(&stop["public_arrival_core"]))
        );
        for other in &stops {
            assert_eq!(event(&c, visited(other)), name(other) == name(stop));
        }
        assert!(!event(&c, super::SERVICE));
        continued(
            &mut c,
            &root,
            &runtime,
            &format!("public-road-{}", stop["id"].as_str().expect("stop id")),
        );
        let exit = &runtime.data().maps[name(stop)].events.warps[0];
        super::super::walk(
            &mut c,
            &runtime,
            (
                i16::try_from(exit.x).expect("exit x"),
                i16::try_from(exit.y).expect("exit y"),
            ),
        );
        assert_eq!(
            location(&mut c),
            (city.into(), point(&access["public_porch"]))
        );
        if access["checkpoint"].is_object() {
            super::super::walk(&mut c, &runtime, point(&access["checkpoint"]["inside"]));
            // Qualified reconciliation hides both warden bodies. The outward
            // x1 event must not replay the inward movement.
            for x in (port.0..3).rev() {
                checkpoint_move(&mut c, GameButton::Left, (x, port.1));
                assert_eq!(location(&mut c), (city.into(), (x, port.1)));
            }
        }
        super::super::walk(&mut c, &runtime, port);
        assert_eq!(location(&mut c), (city.into(), port));
        assert!(!event(&c, super::SERVICE));
    }
}
