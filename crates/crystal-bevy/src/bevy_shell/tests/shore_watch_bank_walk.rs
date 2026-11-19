//! A player can walk beside a lake and turn to look at it while carrying the canoe.
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::{ButtonState, InputPlugin};
use bevy::prelude::*;
use crystal_core::systems::field_moves::{CANOE_ACTIVE_MEMORY, CANOE_ITEM};
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use super::super::{BevyRuntimeShell, PlayerMarker, PlayerSpriteFrames};
use super::{canoe, interior_life, menu_render_test_app};

struct Bank {
    map: &'static str,
    trail: &'static [(i16, i16)],
    launch: (i16, i16),
    toward_water: Direction,
    female: bool,
}

struct Frame {
    at: TilePosition,
    facing: Direction,
    remaining: u8,
    mounted: bool,
    art: String,
}

fn frame(app: &mut App) -> Frame {
    let world = app.world_mut();
    let shell = world.resource::<BevyRuntimeShell>();
    assert_eq!(shell.last_error, None);
    let session = shell.shell.session();
    let snapshot = session.snapshot();
    let mounted = session
        .state()
        .script_runtime
        .memory
        .contains_key(CANOE_ACTIVE_MEMORY);
    assert_eq!(
        snapshot.mode,
        if mounted {
            MovementMode::Surf
        } else {
            MovementMode::Normal
        }
    );
    let remaining = shell.player_walk_frame_ticks;
    let mut query =
        world.query_filtered::<(&PlayerSpriteFrames, &Sprite, &Visibility), With<PlayerMarker>>();
    let (art, sprite, visibility) = query.single(world).expect("one native player sprite");
    assert_ne!(*visibility, Visibility::Hidden);
    let pixels = world
        .resource::<Assets<Image>>()
        .get(&sprite.image)
        .and_then(|image| image.data.as_ref())
        .expect("loaded native player image");
    let (rgba, remainder) = pixels.as_chunks::<4>();
    assert!(remainder.is_empty(), "expected native RGBA pixels");
    assert!(
        rgba.iter().any(|pixel| pixel[3] != 0),
        "invisible player frame"
    );
    Frame {
        at: snapshot.tile,
        facing: snapshot.facing,
        remaining,
        mounted,
        art: art.source_id.to_string(),
    }
}

fn keyboard_event(app: &mut App, direction: Direction, state: ButtonState) {
    let (key_code, logical_key) = match direction {
        Direction::Up => (KeyCode::ArrowUp, Key::ArrowUp),
        Direction::Down => (KeyCode::ArrowDown, Key::ArrowDown),
        Direction::Left => (KeyCode::ArrowLeft, Key::ArrowLeft),
        Direction::Right => (KeyCode::ArrowRight, Key::ArrowRight),
    };
    app.world_mut()
        .write_message(KeyboardInput {
            key_code,
            logical_key,
            state,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        })
        .expect("installed keyboard input");
}

pub(super) fn stride(
    app: &mut App,
    dest: TilePosition,
    walker: &str,
    rider: &str,
    will_mount: bool,
) {
    let start = frame(app);
    let direction = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ]
    .into_iter()
    .find(|d| start.at.moved(*d) == dest)
    .expect("one adjacent square");
    keyboard_event(app, direction, ButtonState::Pressed);
    let mut released = false;
    for _ in 0..40 {
        app.update();
        let now = frame(app);
        assert!(
            now.at == start.at || now.at == dest,
            "walking from {:?} toward {dest:?} reached {:?}",
            start.at,
            now.at
        );
        assert_eq!(
            now.mounted,
            if now.at == dest {
                will_mount
            } else {
                start.mounted
            }
        );
        if start.mounted == will_mount {
            assert_eq!(
                now.art,
                if will_mount { rider } else { walker },
                "wrong art while approaching {dest:?}"
            );
        }
        if now.at == dest {
            if !released {
                keyboard_event(app, direction, ButtonState::Released);
                released = true;
            } else if now.remaining == 0 {
                assert_eq!(now.art, if will_mount { rider } else { walker });
                return;
            }
        }
    }
    panic!("could not reach {dest:?} from {:?}", start.at);
}

fn glance(app: &mut App, at: TilePosition, direction: Direction, walker: &str) {
    // Let the preceding press and interpolation finish before the single turn.
    for _ in 0..5 {
        app.update();
    }
    let start = frame(app);
    assert_eq!(start.at, at);
    assert_eq!(start.remaining, 0);
    assert_ne!(start.facing, direction);
    keyboard_event(app, direction, ButtonState::Pressed);
    app.update();
    let turned = frame(app);
    assert_eq!((turned.at, turned.facing), (at, direction));
    assert!(!turned.mounted && turned.art == walker);
    keyboard_event(app, direction, ButtonState::Released);
    for _ in 0..12 {
        app.update();
        let keyboard = app.world().resource::<ButtonInput<KeyCode>>();
        assert_eq!(keyboard.get_pressed().count(), 0);
        assert_eq!(keyboard.get_just_pressed().count(), 0);
        let now = frame(app);
        assert_eq!((now.at, now.facing), (at, direction));
        assert!(
            !now.mounted && now.art == walker,
            "the trainer only looked toward the water"
        );
    }
}

#[test]
#[ignore = "requires Badge Trails or Shore Watch in CRYSTAL_CANOE_TEST_PACK"]
fn shore_watch_bank_walk_and_one_tick_water_glance_keep_the_player_on_foot() {
    let (root, runtime) = canoe::load();
    let banks = [
        Bank {
            map: "MnR18C00R00",
            trail: &[
                (52, 19),
                (53, 19),
                (53, 20),
                (54, 20),
                (55, 20),
                (56, 20),
                (56, 21),
                (57, 21),
                (57, 20),
                (58, 20),
                (58, 19),
                (59, 19),
                (60, 19),
            ],
            launch: (57, 21),
            toward_water: Direction::Down,
            female: false,
        },
        Bank {
            map: "MnMilleLacs",
            trail: &[
                (16, 29),
                (16, 30),
                (16, 31),
                (16, 32),
                (17, 32),
                (17, 33),
                (18, 33),
                (18, 34),
                (19, 34),
                (19, 35),
                (18, 35),
            ],
            launch: (19, 34),
            toward_water: Direction::Up,
            female: true,
        },
    ];
    for bank in banks {
        let room = &runtime.data().maps[bank.map];
        let shore = TilePosition::new(bank.launch.0, bank.launch.1);
        let water = shore.moved(bank.toward_water);
        assert_eq!(
            interior_life::collision(&runtime, room, (water.x, water.y)),
            Some("WATER")
        );
        for &point in bank.trail {
            assert_eq!(
                interior_life::collision(&runtime, room, point),
                Some("FLOOR")
            );
        }
        for owns in [false, true] {
            let mut fixture = interior_life::fixture(&root, &runtime, bank.map, bank.trail[0]);
            let state = fixture.shell.shell.session_mut().state_mut();
            state.player_gender = u8::from(bank.female);
            if owns {
                assert!(
                    state
                        .bag
                        .add_item(&runtime.data().items[CANOE_ITEM], 1)
                        .expect("owned canoe")
                );
            }
            state
                .flags
                .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
                .expect("isolate the shore walk");
            let mut app = menu_render_test_app(fixture.shell);
            // The minimal render harness supplies no keyboard event processing.
            app.add_plugins(InputPlugin);
            app.update();
            app.update();
            let (walker, rider) = if bank.female {
                ("kris", "mn_canoe_kris")
            } else {
                ("chris", "mn_canoe_chris")
            };
            for &(x, y) in &bank.trail[1..] {
                let dest = TilePosition::new(x, y);
                stride(&mut app, dest, walker, rider, false);
                if dest != shore {
                    continue;
                }
                glance(&mut app, shore, bank.toward_water, walker);
                if owns {
                    stride(&mut app, water, walker, rider, true);
                    stride(&mut app, shore, walker, rider, false);
                } else {
                    keyboard_event(&mut app, bank.toward_water, ButtonState::Pressed);
                    for _ in 0..12 {
                        app.update();
                        let blocked = frame(&mut app);
                        assert_eq!(blocked.at, shore);
                        assert!(!blocked.mounted && blocked.art == walker);
                    }
                    keyboard_event(&mut app, bank.toward_water, ButtonState::Released);
                    app.update();
                }
            }
        }
    }
}
