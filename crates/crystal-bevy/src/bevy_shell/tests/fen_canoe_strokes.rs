//! Compare visible upright paddling strokes with the canoe's native frames.
use bevy::input::InputPlugin;
use bevy::prelude::{App, Assets, Image, Last, Resource, Sprite, With, World};
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::world::map::TilePosition;

use super::super::shore_watch_bank_walk::stride;
use super::super::{canoe, interior_life, menu_render_test_app};
use super::{BevyRuntimeShell, PlayerMarker, PlayerSpriteFrames};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pose {
    Rest,
    Action,
    MirroredAction,
}

#[derive(Default, Resource)]
struct StrokeSample(Option<(TilePosition, Pose)>);

fn sample_midstep(world: &mut World) {
    let shell = world.resource::<BevyRuntimeShell>();
    if shell.player_walk_frame_ticks == 0
        || shell.player_walk_frame_ticks * 2 != shell.player_walk_total_ticks
    {
        return;
    }
    let position = shell.shell.session().overworld().player.tile;
    let mut player = world.query_filtered::<(&Sprite, &PlayerSpriteFrames), With<PlayerMarker>>();
    let (sprite, frames) = player.single(world).expect("one visible rider or walker");
    let images = world.resource::<Assets<Image>>();
    let pixels = |handle| {
        let image = images.get(handle).expect("loaded directional actor frame");
        assert_eq!((image.width(), image.height()), (16, 16));
        image.data.as_deref().expect("native RGBA pixels")
    };
    let standing = pixels(frames.standing.id());
    let stroke = pixels(frames.walking.as_ref().expect("authored action frame").id());
    let displayed = pixels(sprite.image.id());
    let pose = if sprite.image == frames.standing {
        Pose::Rest
    } else if sprite.flip_x {
        Pose::MirroredAction
    } else {
        Pose::Action
    };
    let mounted = frames.source_id.starts_with("mn_canoe_");
    let mut differs_from_rest = false;
    for y in 0..16 {
        for x in 0..16 {
            let offset = |flipped| 4 * (16 * y + if flipped { 15 - x } else { x });
            let dst = offset(false);
            let src = offset(sprite.flip_x);
            let actual = &displayed[src..src + 4];
            differs_from_rest |= actual != &standing[dst..dst + 4];
            // Both paddling beats have an authored upright stroke. Only a
            // walking trainer alternates the action artwork between feet.
            let expected = if pose == Pose::Rest {
                &standing[dst..dst + 4]
            } else {
                let source = offset(!mounted && sprite.flip_x);
                &stroke[source..source + 4]
            };
            assert_eq!(
                actual, expected,
                "{} {pose:?} at {position:?} pixel ({x},{y})",
                frames.source_id
            );
        }
    }
    if mounted && pose != Pose::Rest {
        assert!(differs_from_rest, "invisible paddle stroke at {position:?}");
    }
    world.resource_mut::<StrokeSample>().0 = Some((position, pose));
}

fn take_step(app: &mut App, y: i16, walker: &str, rider: &str, afloat: bool) -> Pose {
    let destination = TilePosition::new(17, y);
    app.world_mut().resource_mut::<StrokeSample>().0 = None;
    stride(app, destination, walker, rider, afloat);
    let (sampled, pose) = app
        .world()
        .resource::<StrokeSample>()
        .0
        .expect("visible midpoint of the step");
    assert_eq!(sampled, destination);
    pose
}

#[test]
#[ignore = "requires the Layered Fen in CRYSTAL_CANOE_TEST_PACK"]
fn fen_south_lake_vertical_strokes_stay_visible_on_both_beats() {
    let (root, runtime) = canoe::load();
    for (gender, walker, rider) in [(0, "chris", "mn_canoe_chris"), (1, "kris", "mn_canoe_kris")] {
        let mut fixture = interior_life::fixture(&root, &runtime, "MnLayeredFen", (17, 11));
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
            .expect("isolate paddle animation");
        let mut app = menu_render_test_app(fixture.shell);
        app.add_plugins(InputPlugin);
        app.init_resource::<StrokeSample>();
        app.add_systems(Last, sample_midstep);
        app.update();
        app.update();
        if let Some(directory) = std::env::var_os("MN_SHORE_MOTION_DIR") {
            super::super::record_overworld_frames(
                &mut app,
                &std::path::Path::new(&directory).join(format!("paddle-{walker}")),
            );
        }

        let gait: Vec<_> = (12..=15)
            .map(|y| take_step(&mut app, y, walker, rider, false))
            .collect();
        assert!(
            gait.contains(&Pose::Action) && gait.contains(&Pose::MirroredAction),
            "{walker} dry gait {gait:?}"
        );
        take_step(&mut app, 16, walker, rider, true);
        for leg in [[17, 18, 19, 20, 21], [20, 19, 18, 17, 16]] {
            let strokes = leg.map(|y| take_step(&mut app, y, walker, rider, true));
            assert!(
                strokes.iter().filter(|&&pose| pose == Pose::Action).count() >= 2,
                "{rider} on {leg:?} showed {strokes:?}"
            );
            assert!(
                strokes.windows(2).all(|pair| pair[0] != pair[1]),
                "{rider} failed to alternate rest and action on {leg:?}: {strokes:?}"
            );
        }
    }
}
