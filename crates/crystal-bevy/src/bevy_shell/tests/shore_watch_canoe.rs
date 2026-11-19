//! Sample the real 2D player sprite on every tick of a shore crossing.
#[path = "fen_canoe_grass.rs"]
mod fen_canoe_grass;

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use crystal_core::systems::field_moves::{CANOE_ACTIVE_MEMORY, CANOE_ITEM};
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::{Direction, TilePosition};
use crystal_core::world::movement::MovementMode;

use crystal_render_api::VisualWorldLighting;
use crystal_runtime::RuntimeRtcSample;

use super::super::{
    BevyRuntimeShell, NativeRtcSource, PlayerMarker, PlayerSpriteFrames, PlayfieldBridgeTile,
    PlayfieldPriorityTile, PlayfieldTile, RenderedTilesetArt, RenderedViewport,
};
use super::{canoe, interior_life, menu_render_test_app, pokemon_overworld_context_canvas_scaled};

struct Landing {
    name: &'static str,
    map: &'static str,
    start: (i16, i16),
    outward: Direction,
    female: bool,
    hour: u8,
}

fn sample(app: &mut App, output: Option<&Path>, frame: usize) -> (String, bool) {
    let world = app.world_mut();
    let mut query =
        world.query_filtered::<(&PlayerSpriteFrames, &Sprite, &Visibility), With<PlayerMarker>>();
    let (art, sprite, visibility) = query
        .single(world)
        .expect("one player or mounted rider on every frame");
    assert_ne!(*visibility, Visibility::Hidden);
    let id = art.source_id.to_string();
    let action = art.walking.as_ref() == Some(&sprite.image) && sprite.image != art.standing;
    let image = world
        .resource::<Assets<Image>>()
        .get(&sprite.image)
        .expect("actual displayed player sprite");
    let (pixels, remainder) = image
        .data
        .as_deref()
        .expect("sprite pixels")
        .as_chunks::<4>();
    assert!(remainder.is_empty(), "expected native RGBA sprite data");
    assert!(pixels.iter().any(|rgba| rgba[3] != 0), "visible actor");
    if let Some(directory) = output {
        let canvas = world.resource_scope(|world, images: Mut<Assets<Image>>| {
            pokemon_overworld_context_canvas_scaled(world, &images, 4)
        });
        canvas
            .save(directory.join(format!("{frame:03}.png")))
            .expect("native LCD frame");
    }
    (id, action)
}

/// Read the terrain under the keel from the actual composited ground texture.
/// In particular, a valid Surf square may have been painted solid brown.
fn assert_fen_keel_on_water(world: &mut World) {
    let mut player = world.query_filtered::<(&Sprite, &Transform), With<PlayerMarker>>();
    let (rider, transform) = player.single(world).expect("displayed Fen canoe");
    let size = rider.custom_size.expect("native rider size");
    // The side-facing keel is a row higher than the bow. Sample its lowest
    // opaque center pixel in this particular frame, including a mirrored stroke.
    let hull = world
        .resource::<Assets<Image>>()
        .get(&rider.image)
        .expect("canoe image");
    assert_eq!((hull.width(), hull.height()), (16, 16));
    let rgba = hull.data.as_deref().expect("hull pixels");
    let column = if rider.flip_x { 7 } else { 8 };
    let row = (0..16)
        .rfind(|&y| rgba[4 * (y * 16 + column) + 3] != 0)
        .expect("visible keel");
    let keel = transform.translation.truncate() + size * Vec2::new(0.5, 7.5 - row as f32) / 16.0;
    let mut ground = world.query_filtered::<(&Sprite, &Transform), (
        With<PlayfieldTile>,
        Without<PlayfieldPriorityTile>,
        Without<PlayfieldBridgeTile>,
    )>();
    let (terrain, transform) = ground.single(world).expect("displayed ground layer");
    let size = terrain.custom_size.expect("native ground size");
    let position = (keel - transform.translation.truncate()) / size;
    let image = world
        .resource::<Assets<Image>>()
        .get(&terrain.image)
        .expect("ground texture");
    let x = ((0.5 + position.x) * image.width() as f32).floor() as usize;
    let y = ((0.5 - position.y) * image.height() as f32).floor() as usize;
    assert!(x < image.width() as usize && y < image.height() as usize);
    let i = 4 * (y * image.width() as usize + x);
    let pixel = &image.data.as_deref().expect("native ground pixels")[i..i + 3];
    let viewport = world.resource::<RenderedViewport>();
    let palette = world
        .resource::<RenderedTilesetArt>()
        .cache
        .iter()
        .find_map(|(key, art)| {
            (key.tileset_id == "johto"
                && VisualWorldLighting::from_source_palette(&key.time_of_day)
                    == viewport.visual_lighting)
                .then(|| art.transition_tiles[0x14].palette)
        })
        .expect("current Fen open-water palette");
    assert!(
        palette.iter().any(|rgb| rgb == pixel),
        "keel is resting on {pixel:?} at canvas ({x},{y}), outside the native water palette {palette:?}"
    );
}

#[test]
#[ignore = "requires a Shore Watch edition in CRYSTAL_CANOE_TEST_PACK"]
fn shore_watch_walk_launch_paddle_and_land_in_native_frames() {
    check_landings(&[
        Landing {
            name: "mainland",
            map: "MnR18C00R00",
            start: (56, 19),
            outward: Direction::Down,
            female: false,
            hour: 12,
        },
        Landing {
            name: "island",
            map: "MnR18C00R00",
            start: (58, 35),
            outward: Direction::Down,
            female: true,
            hour: 12,
        },
        Landing {
            name: "east",
            map: "MnR18C01R00",
            start: (6, 19),
            outward: Direction::Down,
            female: false,
            hour: 21,
        },
        Landing {
            name: "cove",
            map: "MnMilleLacs",
            start: (14, 30),
            outward: Direction::Right,
            female: true,
            hour: 12,
        },
    ]);
}

#[test]
#[ignore = "requires the Layered Fen pools in CRYSTAL_CANOE_TEST_PACK"]
fn fen_south_lake_walk_launch_paddle_and_land_in_native_frames() {
    let fen = "MnLayeredFen";
    check_landings(&[
        Landing {
            name: "fen-south-bank",
            map: fen,
            start: (24, 23),
            outward: Direction::Up,
            female: false,
            hour: 12,
        },
        Landing {
            name: "fen-north-bank",
            map: fen,
            start: (24, 15),
            outward: Direction::Down,
            female: true,
            hour: 8,
        },
        Landing {
            name: "fen-west-cove",
            map: fen,
            start: (9, 19),
            outward: Direction::Right,
            female: true,
            hour: 12,
        },
        Landing {
            name: "fen-east-bank",
            map: fen,
            start: (37, 18),
            outward: Direction::Left,
            female: false,
            hour: 21,
        },
    ]);
}

fn check_landings(sites: &[Landing]) {
    let (root, runtime) = canoe::load();
    let out = std::env::var_os("MN_SHORE_MOTION_DIR").map(PathBuf::from);
    for site in sites {
        let (dx, dy) = site.outward.delta();
        let (outward, inward) = match site.outward {
            Direction::Up => (KeyCode::ArrowUp, KeyCode::ArrowDown),
            Direction::Down => (KeyCode::ArrowDown, KeyCode::ArrowUp),
            Direction::Left => (KeyCode::ArrowLeft, KeyCode::ArrowRight),
            Direction::Right => (KeyCode::ArrowRight, KeyCode::ArrowLeft),
        };
        let mut c = interior_life::fixture(&root, &runtime, site.map, site.start);
        let state = c.shell.shell.session_mut().state_mut();
        state.player_gender = u8::from(site.female);
        assert!(
            state
                .bag
                .add_item(&runtime.data().items[CANOE_ITEM], 1)
                .expect("own canoe")
        );
        state
            .flags
            .set_engine_flag("STATUSFLAGS_NO_WILD_ENCOUNTERS_F", true)
            .expect("isolate shoreline animation");
        let output = out.as_ref().map(|p| p.join(site.name));
        if let Some(dir) = &output {
            std::fs::create_dir_all(dir).expect("capture directory");
        }
        let mut app = menu_render_test_app(c.shell);
        app.insert_resource(NativeRtcSource::fixed(RuntimeRtcSample {
            date: GameDate::new(2000, 1, 1),
            hour: site.hour,
            minute: 0,
            second: 0,
        }));
        app.update();
        app.update();
        let walker = if site.female { "kris" } else { "chris" };
        let rider = if site.female {
            "mn_canoe_kris"
        } else {
            "mn_canoe_chris"
        };
        assert_eq!(sample(&mut app, output.as_deref(), 0).0, walker);
        let mut frame = 1;
        let mut walk_action = false;
        let mut paddle_action = false;
        let mut log = String::from("tick,step,x,y,remaining,total,art,stroke\n");
        for (step, distance) in [1, 2, 3, 4, 3, 2, 1, 0].into_iter().enumerate() {
            let tile =
                TilePosition::new(site.start.0 + dx * distance, site.start.1 + dy * distance);
            let afloat = distance >= 3;
            let kind = interior_life::collision(
                &runtime,
                &runtime.data().maps[site.map],
                (tile.x, tile.y),
            );
            assert_eq!(
                kind,
                Some(if afloat { "WATER" } else { "FLOOR" }),
                "{} at {tile:?}",
                site.name
            );
            let key = if step < 4 { outward } else { inward };
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            let mut arrived = false;
            for _ in 0..40 {
                app.update();
                let (position, remaining, total, mounted, mode) = {
                    let shell = app.world().resource::<BevyRuntimeShell>();
                    assert_eq!(shell.last_error, None);
                    let session = shell.shell.session();
                    let snapshot = session.snapshot();
                    (
                        snapshot.tile,
                        shell.player_walk_frame_ticks,
                        shell.player_walk_total_ticks,
                        session
                            .state()
                            .script_runtime
                            .memory
                            .contains_key(CANOE_ACTIVE_MEMORY),
                        snapshot.mode,
                    )
                };
                let (art, action) = sample(&mut app, output.as_deref(), frame);
                if site.map == "MnLayeredFen" && art == rider {
                    assert_fen_keel_on_water(app.world_mut());
                }
                walk_action |= art == walker && action;
                paddle_action |= art == rider && action;
                use std::fmt::Write as _;
                writeln!(
                    log,
                    "{frame},{step},{},{},{remaining},{total},{art},{action}",
                    position.x, position.y
                )
                .expect("frame log");
                frame += 1;
                assert!(
                    art == walker || art == rider,
                    "{} lost or substituted rider at {position:?}",
                    site.name
                );
                if position == tile {
                    app.world_mut()
                        .resource_mut::<ButtonInput<KeyCode>>()
                        .reset(key);
                    if step == 2 || step == 5 {
                        // Check which physical square contains the center of the
                        // visible soles/keel. The logical step already committed.
                        let traveled = 16 * i16::from(total - remaining) / i16::from(total);
                        let forward = if step < 4 { 1 } else { -1 };
                        let contact_x = 8 + forward * dx * traveled;
                        let contact_y = 15 + forward * dy * traveled;
                        let on_departure =
                            (0..16).contains(&contact_x) && (0..16).contains(&contact_y);
                        let should_paddle = if on_departure { !afloat } else { afloat };
                        assert_eq!(
                            art,
                            if should_paddle { rider } else { walker },
                            "{} at {tile:?}, {remaining}/{total} ticks left",
                            site.name
                        );
                    }
                    assert_eq!(mounted, afloat);
                    assert_eq!(
                        mode,
                        if afloat {
                            MovementMode::Surf
                        } else {
                            MovementMode::Normal
                        }
                    );
                    if remaining == 0 {
                        assert_eq!(art, if afloat { rider } else { walker });
                        arrived = true;
                        break;
                    }
                }
            }
            assert!(
                arrived,
                "{} did not reach {tile:?} in step {step}\n{log}",
                site.name
            );
        }
        assert!(
            walk_action && paddle_action,
            "{} should show both a walking stride and a paddle stroke",
            site.name
        );
        if let Some(dir) = output {
            std::fs::write(dir.join("frames.csv"), log).expect("frame timing");
        }
    }
}
