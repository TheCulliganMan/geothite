//! Compare the displayed OAM against its pack-supplied trainer frame under real movement.
use bevy::input::InputPlugin;
use bevy::prelude::*;
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;

use super::super::{
    BevyRuntimeShell, PlayerMarker, PlayerSpriteFrames, mark_runtime_snapshot_dirty,
};
use super::badge_trails::load;
use super::interior_life::fixture;
use super::menu_render_test_app;
use super::shore_watch_bank_walk::stride;

fn changed_pixels(app: &mut App) -> Vec<(usize, usize)> {
    let world = app.world_mut();
    let mut player = world.query_filtered::<(&Sprite, &PlayerSpriteFrames), With<PlayerMarker>>();
    let (displayed, frames) = player.single(world).expect("one visible trainer");
    let textures = world.resource::<Assets<Image>>();
    let bytes = |handle: &Handle<Image>| {
        textures
            .get(handle)
            .expect("native image")
            .data
            .as_deref()
            .expect("native RGBA")
    };
    // stride() settles movement; the displayed base OAM is the standing frame.
    let native = bytes(&frames.standing);
    let actual = bytes(&displayed.image);
    assert_eq!(actual.len(), 16 * 16 * 4);
    assert_eq!(native.len(), actual.len());
    let (actual_pixels, _) = actual.as_chunks::<4>();
    let (native_pixels, _) = native.as_chunks::<4>();
    actual_pixels
        .iter()
        .zip(native_pixels)
        .enumerate()
        .filter(|(_, (drawn, original))| drawn != original)
        .map(|(i, _)| {
            (
                if displayed.flip_x {
                    15 - i % 16
                } else {
                    i % 16
                },
                i / 16,
            )
        })
        .collect()
}

#[test]
#[ignore = "requires MN_BADGE_TRAILS_PACK; also runs against either earlier Badge Trails pack"]
fn gym_walk_wading_line_follows_each_wet_foot_and_leaves_the_canoe_dry() {
    let (root, runtime) = load();
    let opted_in = runtime
        .runtime_file("data/tilesets/mn_gym_6_animations.json")
        .is_some();
    for female in [false, true] {
        let mut c = fixture(&root, &runtime, "MnBrainerdGym", (8, 4));
        c.shell.shell.session_mut().state_mut().player_gender = u8::from(female);
        mark_runtime_snapshot_dirty(&mut c.shell);
        let (walker, rider) = if female {
            ("kris", "mn_canoe_kris")
        } else {
            ("chris", "mn_canoe_chris")
        };
        let mut app = menu_render_test_app(c.shell);
        app.add_plugins(InputPlugin);
        app.update();
        app.update();
        assert!(changed_pixels(&mut app).is_empty(), "leader's sandbar");
        for y in 5..=11 {
            stride(&mut app, TilePosition::new(8, y), walker, rider, false);
            let delta = changed_pixels(&mut app);
            if !opted_in || y >= 10 {
                assert!(delta.is_empty(), "untouched feet at (8,{y})");
                continue;
            }
            assert!(!delta.is_empty(), "native waterline on {walker} at (8,{y})");
            assert!(
                delta
                    .iter()
                    .all(|&(x, row)| row >= 14 && (y != 5 || x >= 8)),
                "the western half of the mixed landing and all pixels above the ankles must stay native: {delta:?}"
            );
        }
    }
    // The preceding dry/old checks apply to original geometry too. Its deep
    // pockets have a different contour, covered by the original pack suite.
    if !opted_in {
        return;
    }
    let mut c = fixture(&root, &runtime, "MnBrainerdGym", (10, 7));
    assert!(
        c.shell
            .shell
            .session_mut()
            .state_mut()
            .bag
            .add_item(&runtime.data().items[CANOE_ITEM], 1)
            .expect("own canoe")
    );
    mark_runtime_snapshot_dirty(&mut c.shell);
    let mut app = menu_render_test_app(c.shell);
    app.add_plugins(InputPlugin);
    app.update();
    app.update();
    #[derive(Resource, Default)]
    struct LaunchFrames {
        wading: usize,
        paddling: usize,
    }
    app.init_resource::<LaunchFrames>()
        .add_systems(Last, |world: &mut World| {
            if world
                .resource::<BevyRuntimeShell>()
                .shell
                .session()
                .overworld()
                .player
                .mode
                != MovementMode::Surf
            {
                return;
            }
            let mut query =
                world.query_filtered::<(&Sprite, &PlayerSpriteFrames), With<PlayerMarker>>();
            let (sprite, frames) = query.single(world).expect("trainer crossing the launch");
            let native =
                sprite.image == frames.standing || frames.walking.as_ref() == Some(&sprite.image);
            match frames.source_id.as_ref() {
                "chris" => {
                    assert!(
                        !native,
                        "feet are still in the shoal during the committed northward launch"
                    );
                    world.resource_mut::<LaunchFrames>().wading += 1;
                }
                "mn_canoe_chris" => {
                    assert!(native, "the actual hull must keep every source pixel");
                    world.resource_mut::<LaunchFrames>().paddling += 1;
                }
                id => panic!("unexpected lake crossing sprite {id}"),
            }
        });
    stride(
        &mut app,
        TilePosition::new(10, 6),
        "chris",
        "mn_canoe_chris",
        true,
    );
    assert!(
        changed_pixels(&mut app).is_empty(),
        "native paddler and hull"
    );
    let frames = app.world().resource::<LaunchFrames>();
    assert!(
        frames.wading > 0 && frames.paddling > 0,
        "observed both sides of the physical launch"
    );
    stride(
        &mut app,
        TilePosition::new(10, 7),
        "chris",
        "mn_canoe_chris",
        false,
    );
    assert!(
        !changed_pixels(&mut app).is_empty(),
        "boots meet the water again when beaching"
    );
}
