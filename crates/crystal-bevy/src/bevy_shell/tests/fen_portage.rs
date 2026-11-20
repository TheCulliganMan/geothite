//! Carry the same canoe from the north inlet to the large southern lake.
use std::collections::BTreeSet;
use std::path::PathBuf;

use bevy::input::InputPlugin;
use bevy::prelude::*;
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::TilePosition;
use crystal_runtime::RuntimeRtcSample;

use super::super::super::{GrassRustleMarker, PLAYFIELD_LEFT, PLAYFIELD_TOP};
use super::super::shore_watch_bank_walk::stride;
use super::super::{canoe, interior_life, menu_render_test_app, record_overworld_frames};
use super::fen_canoe_grass::check_visible_trainer;
use super::{
    NativeRtcSource, PlayerMarker, assert_fen_keel_on_water,
    pokemon_overworld_context_canvas_scaled, sample,
};

#[derive(Default, Resource)]
struct PortagePoses(Vec<bool>);

/// Sample the same small patch of open water and dry bank from the stationary game camera.
fn parked_ripples(app: &mut App, dry_offset: (i32, i32)) {
    let mut wet = BTreeSet::new();
    let mut dry = BTreeSet::new();
    for tick in 0..=70 {
        if tick % 4 == 0 {
            let world = app.world_mut();
            let mut player = world.query_filtered::<(&Sprite, &Transform), With<PlayerMarker>>();
            let (sprite, transform) = player.single(world).expect("parked rider");
            let size = sprite.custom_size.expect("native rider size");
            let x =
                ((transform.translation.x - size.x / 2.0 - PLAYFIELD_LEFT) / 4.0).round() as i32;
            let y = ((PLAYFIELD_TOP - transform.translation.y - size.y / 2.0) / 4.0).round() as i32;
            let (water, bank) = world.resource_scope(|world, images: Mut<Assets<Image>>| {
                let lcd = pokemon_overworld_context_canvas_scaled(world, &images, 4);
                let patch = |(dx, dy): (i32, i32)| {
                    let x = u32::try_from(x + 16 * dx + 4).expect("visible sample column");
                    let y = u32::try_from(y + 16 * dy + 4).expect("visible sample row");
                    image::imageops::crop_imm(&lcd, x, y, 8, 8)
                        .to_image()
                        .into_raw()
                };
                (patch((-1, 0)), patch(dry_offset))
            });
            wet.insert(water);
            dry.insert(bank);
        }
        app.update();
    }
    assert_eq!(
        wet.len(),
        3,
        "the three distinct native Johto ripple images"
    );
    assert_eq!(dry.len(), 1, "the pebble bank must remain still");
}

#[test]
#[ignore = "requires both Layered Fen pools in CRYSTAL_CANOE_TEST_PACK"]
fn fen_portage_between_both_lakes_preserves_footing_and_water_motion() {
    let (root, runtime) = canoe::load();
    let name = "MnLayeredFen";
    let room = &runtime.data().maps[name];
    let route = [
        (17, 5, "WATER"),
        (16, 5, "WATER"),
        (16, 6, "WATER"),
        (16, 7, "FLOOR"),
        (16, 8, "TALL_GRASS"),
        (16, 9, "TALL_GRASS"),
        (16, 10, "FLOOR"),
        (16, 11, "FLOOR"),
        (16, 12, "FLOOR"),
        (16, 13, "FLOOR"),
        (16, 14, "FLOOR"),
        (16, 15, "FLOOR"),
        (17, 15, "FLOOR"),
        (18, 15, "TALL_GRASS"),
        (19, 15, "TALL_GRASS"),
        (19, 16, "WATER"),
        (19, 17, "WATER"),
        (18, 17, "WATER"),
        (18, 16, "WATER"),
        (18, 15, "TALL_GRASS"),
    ];
    for (x, y, role) in route {
        assert_eq!(interior_life::collision(&runtime, room, (x, y)), Some(role));
    }
    for (gender, walker, rider, hour) in [
        (0, "chris", "mn_canoe_chris", 12),
        (1, "kris", "mn_canoe_kris", 21),
    ] {
        let mut fixture = interior_life::fixture(&root, &runtime, name, (17, 6));
        let state = fixture.shell.shell.session_mut().state_mut();
        state.player_gender = gender;
        assert!(
            state
                .bag
                .add_item(&runtime.data().items[CANOE_ITEM], 1)
                .expect("own canoe")
        );
        state
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate the portage");
        let mut app = menu_render_test_app(fixture.shell);
        app.add_plugins(InputPlugin);
        app.insert_resource(NativeRtcSource::fixed(RuntimeRtcSample {
            date: GameDate::new(2000, 1, 1),
            hour,
            minute: 0,
            second: 0,
        }));
        app.init_resource::<PortagePoses>();
        app.update();
        app.update();
        if let Some(base) = std::env::var_os("MN_SHORE_MOTION_DIR") {
            record_overworld_frames(
                &mut app,
                &PathBuf::from(base).join(format!("fen-portage-{walker}")),
            );
        }
        app.add_systems(Last, move |world: &mut World| {
            let (art, _) = sample(world, None, 0);
            assert!(art == walker || art == rider);
            let aboard = art == rider;
            check_visible_trainer(world);
            if aboard {
                assert_fen_keel_on_water(world);
                assert_eq!(
                    world
                        .query_filtered::<Entity, With<GrassRustleMarker>>()
                        .iter(world)
                        .count(),
                    0
                );
            }
            let mut poses = world.resource_mut::<PortagePoses>();
            if poses.0.last() != Some(&aboard) {
                poses.0.push(aboard);
            }
        });
        app.update();
        for (x, y, role) in route {
            stride(
                &mut app,
                TilePosition::new(x, y),
                walker,
                rider,
                role == "WATER",
            );
            match (x, y) {
                (17, 5) => parked_ripples(&mut app, (0, 1)),
                (19, 17) => parked_ripples(&mut app, (-2, -2)),
                (16, 10) => {
                    let (hidden, visible) = check_visible_trainer(app.world_mut());
                    assert_eq!(hidden, 0, "the walking boots clear the grass on the trail");
                    assert!(visible > 0);
                }
                _ => {}
            }
        }
        assert_eq!(
            app.world().resource::<PortagePoses>().0,
            [false, true, false, true, false]
        );
    }
}
