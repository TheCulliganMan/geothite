//! The northern pool narrows to one water square between two dry pebbled banks.
use std::path::PathBuf;

use bevy::input::InputPlugin;
use bevy::prelude::{Last, Resource, World};
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::TilePosition;
use crystal_runtime::RuntimeRtcSample;

use super::super::shore_watch_bank_walk::stride;
use super::super::{canoe, interior_life, menu_render_test_app};
use super::fen_canoe_grass::check_visible_trainer;
use super::{NativeRtcSource, assert_fen_keel_on_water, sample};

#[derive(Default, Resource)]
struct InletFrames {
    aboard: Vec<bool>,
    walked: bool,
    paddled: bool,
}

#[test]
#[ignore = "requires the Layered Fen in CRYSTAL_CANOE_TEST_PACK"]
fn fen_north_inlet_walk_launch_turn_at_the_tip_and_return() {
    let (root, runtime) = canoe::load();
    let name = "MnLayeredFen";
    let room = &runtime.data().maps[name];
    // The east tip has dry ground both north and south and a solid fir to its east.
    for (point, expected) in [
        ((17, 4), "FLOOR"),
        ((17, 5), "WATER"),
        ((17, 6), "FLOOR"),
        ((18, 5), "WALL"),
    ] {
        assert_eq!(
            interior_life::collision(&runtime, room, point),
            Some(expected)
        );
    }
    let mut fixture = interior_life::fixture(&root, &runtime, name, (17, 10));
    let state = fixture.shell.shell.session_mut().state_mut();
    state.player_gender = 1;
    assert!(
        state
            .bag
            .add_item(&runtime.data().items[CANOE_ITEM], 1)
            .expect("owned canoe")
    );
    state
        .flags
        .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
        .expect("isolate the crossing");
    let mut app = menu_render_test_app(fixture.shell);
    app.add_plugins(InputPlugin);
    app.insert_resource(NativeRtcSource::fixed(RuntimeRtcSample {
        date: GameDate::new(2000, 1, 1),
        hour: 12,
        minute: 0,
        second: 0,
    }));
    app.init_resource::<InletFrames>();
    app.update();
    app.update();
    let output = std::env::var_os("MN_SHORE_MOTION_DIR")
        .map(|base| PathBuf::from(base).join("fen-north-inlet"));
    if let Some(dir) = &output {
        std::fs::create_dir_all(dir).expect("north inlet frame directory");
    }
    let mut tick = 0;
    app.add_systems(Last, move |world: &mut World| {
        let (art, action) = sample(world, output.as_deref(), tick);
        tick += 1;
        let aboard = art == "mn_canoe_kris";
        assert!(aboard || art == "kris");
        check_visible_trainer(world);
        if aboard {
            assert_fen_keel_on_water(world);
        }
        let mut frames = world.resource_mut::<InletFrames>();
        if frames.aboard.last() != Some(&aboard) {
            frames.aboard.push(aboard);
        }
        frames.walked |= !aboard && action;
        frames.paddled |= aboard && action;
    });
    app.update();
    let outbound = [
        (17, 10, "FLOOR"),
        (17, 9, "TALL_GRASS"),
        (17, 8, "TALL_GRASS"),
        (17, 7, "FLOOR"),
        (17, 6, "FLOOR"),
        (17, 5, "WATER"),
        (16, 5, "WATER"),
    ];
    for &(x, y, role) in outbound[1..].iter().chain(outbound.iter().rev().skip(1)) {
        assert_eq!(interior_life::collision(&runtime, room, (x, y)), Some(role));
        stride(
            &mut app,
            TilePosition::new(x, y),
            "kris",
            "mn_canoe_kris",
            role == "WATER",
        );
    }
    let frames = app.world().resource::<InletFrames>();
    assert_eq!(frames.aboard, [false, true, false]);
    assert!(frames.walked && frames.paddled);
}
