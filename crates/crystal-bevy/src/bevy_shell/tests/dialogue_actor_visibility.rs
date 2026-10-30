//! An actual earned conversation must retain its visible player throughout.
use std::path::PathBuf;

use bevy::color::Alpha;
use bevy::prelude::{
    Assets, ButtonInput, Entity, Image, KeyCode, PostUpdate, Sprite, Startup, Transform,
    Visibility, Window, With,
};
use bevy::window::{PrimaryWindow, WindowResolution};

use super::super::super::{
    BevyRuntimeShell, FullscreenWorldRoot, PlayerMarker, RenderedViewport, VisibleShellController,
    setup_fullscreen_scene, sync_fullscreen_scaling, sync_fullscreen_scene_layout,
    sync_fullscreen_world_layout,
};
use super::super::{integrated_shell_test_app, winona::load};

#[test]
#[ignore = "requires an actual earned officer checkpoint in CRYSTAL_DIALOGUE_ACTOR_REVIEW_SAVE"]
fn overhaul_officer_conversation_retains_the_player_on_every_2d_frame() {
    for warmup in [0, 2] {
        officer_conversation(warmup);
    }
}

fn officer_conversation(warmup: usize) {
    use bevy::prelude::IntoScheduleConfigs;

    let (root, runtime) = load();
    let mut controller = VisibleShellController::load_save(
        root,
        runtime,
        PathBuf::from(
            std::env::var_os("CRYSTAL_DIALOGUE_ACTOR_REVIEW_SAVE")
                .expect("actual same-pack earned officer checkpoint"),
        ),
        None,
    )
    .expect("native Continue");
    let snapshot = controller.snapshot().expect("actual officer position");
    assert_eq!(snapshot.overworld.map_name, "MnBemidji");
    assert_eq!(
        (snapshot.overworld.tile.x, snapshot.overworld.tile.y),
        (21, 20)
    );
    let mut app = integrated_shell_test_app(controller.shell);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1280, 800).with_scale_factor_override(1.0),
            ..Default::default()
        },
        PrimaryWindow,
    ));
    app.add_systems(Startup, setup_fullscreen_scene)
        .add_systems(
            PostUpdate,
            (
                sync_fullscreen_scaling,
                sync_fullscreen_scene_layout,
                sync_fullscreen_world_layout,
            )
                .chain(),
        );
    for _ in 0..warmup {
        app.update();
    }
    let mut retained = (warmup > 0).then(|| {
        let world = app.world_mut();
        let (entity, transform) = world
            .query_filtered::<(Entity, &Transform), With<PlayerMarker>>()
            .single(world)
            .expect("one rendered player before talking");
        (entity, *transform)
    });
    for frame in 0..120 {
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            if frame == 0 {
                keys.press(KeyCode::KeyZ);
            }
        }
        app.update();
        let world = app.world_mut();
        let shell = world.resource::<BevyRuntimeShell>();
        assert!(
            !shell
                .shell
                .presentation_snapshot()
                .expect("native pose")
                .overworld_player_hidden
        );
        let (entity, transform, sprite) = world
            .query_filtered::<(Entity, &Transform, &Sprite), With<PlayerMarker>>()
            .single(world)
            .unwrap_or_else(|error| panic!("frame {frame}: player must remain drawn: {error}"));
        if let Some((player, before)) = retained {
            assert_eq!(
                entity, player,
                "warmup {warmup} frame {frame}: retain the actor"
            );
            assert_eq!(
                transform.translation, before.translation,
                "warmup {warmup} frame {frame}: dialogue cannot move the player"
            );
        } else {
            retained = Some((entity, *transform));
        }
        assert!(sprite.color.alpha() > 0.0, "frame {frame}: opaque actor");
        assert!(
            world
                .resource::<Assets<Image>>()
                .get(&sprite.image)
                .expect("native player image")
                .data
                .as_ref()
                .is_some_and(|pixels| pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] > 0))
        );
        assert_ne!(
            *world
                .query_filtered::<&Visibility, With<FullscreenWorldRoot>>()
                .single(world)
                .expect("field root"),
            Visibility::Hidden
        );
    }
    assert!(!app.world().resource::<RenderedViewport>().title_active);
    let mut returned = VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("same live keyboard session"),
    };
    assert_eq!(
        returned
            .snapshot()
            .expect("opened officer conversation")
            .ui
            .text
            .expect("the normal A interaction actually opened text")
            .label,
        "MnBemidjiReportAfterText"
    );
}

#[test]
#[ignore = "requires matched native 1280x800 officer captures before/after ordinary A input"]
fn overhaul_officer_dialogue_preserves_native_player_pixels() {
    let read = |name| {
        image::open(PathBuf::from(
            std::env::var_os(name).expect("explicit matched native capture"),
        ))
        .expect("native GPU capture")
        .to_rgba8()
    };
    let before = read("CRYSTAL_DIALOGUE_ACTOR_BEFORE");
    let after = read("CRYSTAL_DIALOGUE_ACTOR_AFTER");
    assert_eq!(before.dimensions(), (1280, 800));
    assert_eq!(after.dimensions(), before.dimensions());
    let actor = |image: &image::RgbaImage| {
        let mut pixels = Vec::new();
        for y in 400..448 {
            for x in 640..688 {
                pixels.push(image.get_pixel(x, y).0);
            }
        }
        pixels
    };
    let expected = actor(&before);
    assert!(
        expected
            .iter()
            .filter(|pixel| pixel[0] > 150 && pixel[1] < 100 && pixel[2] < 100)
            .count()
            > 20,
        "the actual pre-dialogue frame must contain the native red player"
    );
    assert_eq!(
        actor(&after),
        expected,
        "opening the officer's speech box cannot erase or move the player"
    );
}
