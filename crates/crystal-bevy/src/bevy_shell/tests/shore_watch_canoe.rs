//! Sample the real 2D player sprite on every tick of a shore crossing.
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use crystal_core::systems::field_moves::{CANOE_ACTIVE_MEMORY, CANOE_ITEM};
use crystal_core::systems::time::GameDate;
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;

use crystal_runtime::RuntimeRtcSample;

use super::super::{BevyRuntimeShell, NativeRtcSource, PlayerMarker, PlayerSpriteFrames};
use super::{canoe, interior_life, menu_render_test_app, pokemon_overworld_context_canvas_scaled};

struct Landing {
    name: &'static str,
    map: &'static str,
    start: (i16, i16),
    outward: KeyCode,
    delta: (i16, i16),
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

#[test]
#[ignore = "requires a Shore Watch edition in CRYSTAL_CANOE_TEST_PACK"]
fn shore_watch_walk_launch_paddle_and_land_in_native_frames() {
    let (root, runtime) = canoe::load();
    let out = std::env::var_os("MN_SHORE_MOTION_DIR").map(PathBuf::from);
    for site in [
        Landing {
            name: "mainland",
            map: "MnR18C00R00",
            start: (56, 19),
            outward: KeyCode::ArrowDown,
            delta: (0, 1),
            female: false,
            hour: 12,
        },
        Landing {
            name: "island",
            map: "MnR18C00R00",
            start: (58, 35),
            outward: KeyCode::ArrowDown,
            delta: (0, 1),
            female: true,
            hour: 12,
        },
        Landing {
            name: "east",
            map: "MnR18C01R00",
            start: (6, 19),
            outward: KeyCode::ArrowDown,
            delta: (0, 1),
            female: false,
            hour: 21,
        },
        Landing {
            name: "cove",
            map: "MnMilleLacs",
            start: (14, 30),
            outward: KeyCode::ArrowRight,
            delta: (1, 0),
            female: true,
            hour: 12,
        },
    ] {
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
            let tile = TilePosition::new(
                site.start.0 + site.delta.0 * distance,
                site.start.1 + site.delta.1 * distance,
            );
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
            let key = if step < 4 {
                site.outward
            } else if site.outward == KeyCode::ArrowRight {
                KeyCode::ArrowLeft
            } else {
                KeyCode::ArrowUp
            };
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
                        let contact_x = 8 + forward * site.delta.0 * traveled;
                        let contact_y = 15 + forward * site.delta.1 * traveled;
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
