//! Follow held directional input across each grassy shoreline arrangement.
use std::path::PathBuf;
use std::time::Duration;

use bevy::input::{ButtonState, InputPlugin};
use bevy::prelude::{App, Entity, Transform, Vec2, With, Without, World};
use bevy::time::TimeUpdateStrategy;
use crystal_core::systems::field_moves::CANOE_ITEM;
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_runtime::RuntimeRtcSample;

use super::super::super::{
    BevyRuntimeShell, GAME_TICK_SECONDS, GrassRustleMarker, NativeRtcSource, PLAYFIELD_LEFT,
    PLAYFIELD_TOP, PlayerMarker, PlayerSpriteFrames, PlayfieldBridgeTile, PlayfieldPriorityTile,
    PlayfieldTile, RenderedViewport, RuntimeTickTimer, SOURCE_TILE_SIZE, TILE_SIZE,
};
use super::super::shore_watch_bank_walk::{keyboard_event, stride};
use super::super::{canoe, interior_life, menu_render_test_app, record_overworld_frames_every};
use super::assert_fen_keel_on_water;
use super::fen_canoe_grass::check_visible_trainer;

// On held input the controller can begin the next step on the very frame
// the boots reach the previous tile. Read their position from the camera.
fn foot_position(world: &mut World, center: Vec2) -> Vec2 {
    let ground = world
        .query_filtered::<&Transform, (
            With<PlayfieldTile>,
            Without<PlayfieldPriorityTile>,
            Without<PlayfieldBridgeTile>,
        )>()
        .single(world)
        .expect("one scrolling terrain layer")
        .translation
        .truncate();
    let (left, top) = world
        .resource::<RenderedViewport>()
        .viewport_origin
        .expect("visible map");
    let scale = TILE_SIZE / SOURCE_TILE_SIZE as f32;
    let local = center - ground;
    let x = f32::from(left) * 8.0 + (local.x - PLAYFIELD_LEFT) / scale + 0.5;
    let y = f32::from(top) * 8.0 + (PLAYFIELD_TOP - local.y) / scale + 7.5;
    Vec2::new(x, y)
}

struct Contact {
    position: Vec2,
    aboard: bool,
    rustling: bool,
}

fn sample(world: &mut World, walker: &str, rider: &str) -> Contact {
    let mut players =
        world.query_filtered::<(&PlayerSpriteFrames, &Transform), With<PlayerMarker>>();
    let (frames, transform) = players.single(world).expect("one visible actor");
    let center = transform.translation.truncate();
    let art = frames.source_id.as_ref();
    assert!(
        art == walker || art == rider,
        "unexpected shore sprite {art}"
    );
    let aboard = art == rider;
    let grass = world
        .query_filtered::<Entity, With<GrassRustleMarker>>()
        .iter(world)
        .next()
        .is_some();
    if aboard {
        assert!(!grass, "grass flashed over the canoe");
        assert_fen_keel_on_water(world);
    }
    check_visible_trainer(world);
    Contact {
        position: foot_position(world, center),
        aboard,
        rustling: grass,
    }
}

fn hold_across_bank(
    app: &mut App,
    direction: Direction,
    steps: [TilePosition; 3],
    shore: TilePosition,
    walker: &str,
    rider: &str,
) -> Vec<bool> {
    let mut previous = app
        .world()
        .resource::<BevyRuntimeShell>()
        .shell
        .session()
        .overworld()
        .player
        .tile;
    let mut visited = Vec::new();
    let mut poses = Vec::new();
    let mut stirred_grass = false;
    let launching = steps[0] == shore;
    let inland = if launching { previous } else { steps[2] };
    let mut last_sample = None;
    let mut intermediate_frames = 0;
    let interpolated = app.world().resource::<RuntimeTickTimer>().step_seconds > 0.0;
    let (dx, dy) = direction.delta();
    let one_pixel = Vec2::new(f32::from(dx), f32::from(dy));
    keyboard_event(app, direction, ButtonState::Pressed);
    for _ in 0..64 {
        app.update();
        let shell = app.world().resource::<BevyRuntimeShell>();
        assert_eq!(shell.last_error, None);
        let position = shell.shell.session().overworld().player.tile;
        let remaining = shell.player_walk_frame_ticks;
        let settled = remaining == 0;
        let subframe = app
            .world()
            .resource::<RuntimeTickTimer>()
            .presentation_subframe();
        let contact = sample(app.world_mut(), walker, rider);
        let pixel = contact.position;
        let foot = TilePosition::new(
            (pixel.x / 16.0).floor() as i16,
            (pixel.y / 16.0).floor() as i16,
        );
        let on_land = foot == shore || foot == inland;
        assert_ne!(
            contact.aboard, on_land,
            "{direction:?} at visible foot or keel {foot:?}"
        );
        if let Some((old_tile, old_remaining, old_subframe, old_pixel)) = last_sample {
            let advanced = pixel.abs_diff_eq(old_pixel + one_pixel, 0.01);
            if interpolated {
                assert!(
                    advanced || pixel.abs_diff_eq(old_pixel, 0.01),
                    "{direction:?} at {position:?}: feet or keel jumped from {old_pixel:?} to {pixel:?}"
                );
            }
            if position == old_tile
                && remaining == old_remaining
                && !settled
                && subframe > old_subframe
            {
                assert!(
                    advanced,
                    "{direction:?} at {position:?}: missing between-tick step"
                );
                intermediate_frames += 1;
            }
        }
        last_sample = Some((position, remaining, subframe, pixel));
        // Walking through two adjoining grass tiles can finish its rustle
        // before the soles leave the first. Landing should stir the bank itself.
        stirred_grass |= contact.rustling && (launching || foot == shore);
        if poses.last() != Some(&contact.aboard) {
            poses.push(contact.aboard);
        }
        if position != previous {
            visited.push(position);
            previous = position;
            if position == steps[2] {
                keyboard_event(app, direction, ButtonState::Released);
            }
        }
        if position == steps[2] && settled {
            if interpolated {
                assert!(
                    intermediate_frames >= 16,
                    "too few between-tick samples on {steps:?}"
                );
            }
            assert_eq!(
                visited, steps,
                "held {direction:?} from the bank at {shore:?}"
            );
            assert!(
                stirred_grass,
                "missing grass reaction going {direction:?} at {shore:?}"
            );
            return poses;
        }
    }
    panic!(
        "held {direction:?} from bank {shore:?} failed to settle at {:?}: {visited:?}",
        steps[2]
    );
}

#[test]
#[ignore = "requires the Layered Fen in CRYSTAL_CANOE_TEST_PACK"]
fn fen_grassy_banks_keep_the_trainer_and_canoe_visible_under_held_input() {
    check_grassy_banks(None);
}

#[test]
#[ignore = "requires the Layered Fen in CRYSTAL_CANOE_TEST_PACK"]
fn fen_grassy_banks_walk_launch_and_land_between_game_ticks() {
    check_grassy_banks(Some(Duration::from_secs_f64(
        f64::from(GAME_TICK_SECONDS) * 0.5,
    )));
}

fn check_grassy_banks(host_frame: Option<Duration>) {
    use Direction::{Down, Left, Right, Up};
    let (root, runtime) = canoe::load();
    let map = "MnLayeredFen";
    let room = &runtime.data().maps[map];
    // The approach begins on ordinary floor and walks into the patch.
    for (name, shore, out, back, female, approach) in [
        (
            "grass-west",
            (34, 20),
            Left,
            Right,
            true,
            &[(36, 20), (35, 20)][..],
        ),
        (
            "grass-east",
            (14, 22),
            Right,
            Left,
            false,
            &[(12, 21), (12, 22), (13, 22)],
        ),
        (
            "grass-north",
            (13, 22),
            Up,
            Down,
            false,
            &[(11, 23), (12, 23), (13, 23)],
        ),
        (
            "grass-south",
            (19, 15),
            Down,
            Up,
            true,
            &[(17, 14), (18, 14), (19, 14)],
        ),
    ] {
        let shore = TilePosition::new(shore.0, shore.1);
        let inland = shore.moved(back);
        let water = shore.moved(out);
        let end = water.moved(out);
        assert_eq!(approach.last(), Some(&(inland.x, inland.y)));
        for (tile, expected) in [
            (inland, "TALL_GRASS"),
            (shore, "TALL_GRASS"),
            (water, "WATER"),
            (end, "WATER"),
        ] {
            assert_eq!(
                interior_life::collision(&runtime, room, (tile.x, tile.y)),
                Some(expected),
                "{name} {tile:?}"
            );
        }
        let (walker, rider) = if female {
            ("kris", "mn_canoe_kris")
        } else {
            ("chris", "mn_canoe_chris")
        };
        let mut fixture = interior_life::fixture(&root, &runtime, map, approach[0]);
        let state = fixture.shell.shell.session_mut().state_mut();
        state.player_gender = u8::from(female);
        assert!(
            state
                .bag
                .add_item(&runtime.data().items[CANOE_ITEM], 1)
                .expect("own canoe")
        );
        state
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate visual footing");
        let mut app = menu_render_test_app(fixture.shell);
        app.add_plugins(InputPlugin);
        app.insert_resource(NativeRtcSource::fixed(RuntimeRtcSample {
            date: GameDate::new(2000, 1, 1),
            hour: 12,
            minute: 0,
            second: 0,
        }));
        app.update();
        app.update();
        if let Some(duration) = host_frame {
            app.insert_resource(RuntimeTickTimer::new(2.0 * duration.as_secs_f64()));
            app.insert_resource(TimeUpdateStrategy::ManualDuration(duration));
        }
        if let Some(directory) = std::env::var_os("MN_SHORE_MOTION_DIR") {
            let mut directory = PathBuf::from(directory);
            if host_frame.is_some() {
                directory.push("between-ticks");
            }
            let interval = if host_frame.is_some() { 1 } else { 2 };
            record_overworld_frames_every(&mut app, &directory.join(name), interval);
        }
        for &(x, y) in &approach[1..] {
            stride(&mut app, TilePosition::new(x, y), walker, rider, false);
        }
        assert_eq!(
            hold_across_bank(&mut app, out, [shore, water, end], shore, walker, rider),
            [false, true],
            "{name} launch"
        );
        assert_eq!(
            hold_across_bank(&mut app, back, [water, shore, inland], shore, walker, rider),
            [true, false],
            "{name} landing"
        );
    }
}
