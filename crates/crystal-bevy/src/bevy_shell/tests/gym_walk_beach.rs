//! Wade to the leader, paddle every deep pocket and see the native water move.
use std::path::Path;

use bevy::input::InputPlugin;
use bevy::prelude::*;
use crystal_core::systems::field_moves::{CANOE_ACTIVE_MEMORY, CANOE_ITEM};
use crystal_core::world::map::TilePosition;
use crystal_core::world::movement::MovementMode;

use super::super::{BevyRuntimeShell, VisibleShellController, load_tileset_art};
use super::badge_trails::load;
use super::interior_life::fixture;
use super::r10_gallery::continue_save;
use super::shore_watch_bank_walk::stride;
use super::{menu_render_test_app, pokemon_overworld_context_canvas_scaled};

const ROOM: &str = "MnBrainerdGym";

fn beach(shell: BevyRuntimeShell) -> App {
    let mut app = menu_render_test_app(shell);
    app.add_plugins(InputPlugin);
    app.update();
    app.update();
    app
}

fn record(app: &mut App, directory: &Path) {
    std::fs::create_dir_all(directory).expect("review frames directory");
    let directory = directory.to_owned();
    let mut tick = 0_u32;
    app.add_systems(Last, move |world: &mut World| {
        if tick.is_multiple_of(2) {
            let image = world.resource_scope(|world, images: Mut<Assets<Image>>| {
                pokemon_overworld_context_canvas_scaled(world, &images, 4)
            });
            image
                .save(directory.join(format!("{tick:03}.png")))
                .expect("save LCD frame");
        }
        tick += 1;
    });
}

#[test]
#[ignore = "requires a Gym Walk edition in MN_BADGE_TRAILS_PACK"]
fn gym_walk_brainerd_keeps_the_direct_wade_and_launches_in_each_pool() {
    let (root, runtime) = load();
    let evidence = std::env::var_os("MN_GYM_WALK_BEACH_FRAMES").map(std::path::PathBuf::from);
    for owns in [false, true] {
        let mut c = fixture(&root, &runtime, ROOM, (8, 12));
        if owns {
            assert!(
                c.shell
                    .shell
                    .session_mut()
                    .state_mut()
                    .bag
                    .add_item(&runtime.data().items[CANOE_ITEM], 1)
                    .expect("own canoe")
            );
        }
        let mut app = beach(c.shell);
        if owns && let Some(dir) = &evidence {
            record(&mut app, &dir.join("shoal"));
        }
        for y in (4..12).rev() {
            stride(
                &mut app,
                TilePosition::new(8, y),
                "chris",
                "mn_canoe_chris",
                false,
            );
        }
        let shell = app.world().resource::<BevyRuntimeShell>();
        let snapshot = shell.shell.session().snapshot();
        assert_eq!(snapshot.tile, TilePosition::new(8, 4));
        assert!(!shell.shell.has_active_battle() && shell.field_notice.is_none());
    }
    // Every cell in all three dark pools, plus real launch and beaching tiles.
    type Point = (i16, i16);
    let trips: [(&str, Point, &[Point]); 3] = [
        (
            "west",
            (5, 6),
            &[(4, 6), (4, 5), (5, 5), (5, 4), (5, 3), (6, 3), (7, 3)],
        ),
        (
            "float",
            (10, 7),
            &[
                (10, 6),
                (10, 5),
                (10, 4),
                (11, 4),
                (11, 3),
                (12, 3),
                (13, 3),
                (14, 3),
            ],
        ),
        (
            "south",
            (10, 9),
            &[
                (11, 9),
                (11, 10),
                (11, 9),
                (11, 8),
                (12, 8),
                (12, 9),
                (12, 8),
                (12, 7),
                (13, 7),
            ],
        ),
    ];
    for (name, start, steps) in trips {
        let mut c = fixture(&root, &runtime, ROOM, start);
        let state = c.shell.shell.session_mut().state_mut();
        state.player_gender = 1;
        assert!(
            state
                .bag
                .add_item(&runtime.data().items[CANOE_ITEM], 1)
                .expect("owned canoe")
        );
        let mut app = beach(c.shell);
        if let Some(dir) = &evidence {
            record(&mut app, &dir.join(name));
        }
        for (i, &(x, y)) in steps.iter().enumerate() {
            let afloat = i + 1 < steps.len();
            stride(
                &mut app,
                TilePosition::new(x, y),
                "kris",
                "mn_canoe_kris",
                afloat,
            );
            if i == 2 && name == "south" {
                // Resume in the middle of a pool, then finish paddling and land.
                let mut original = VisibleShellController {
                    shell: app
                        .world_mut()
                        .remove_resource::<BevyRuntimeShell>()
                        .expect("native shell"),
                };
                let resumed = continue_save(root.clone(), runtime.clone(), &mut original);
                assert_eq!(
                    resumed.shell.shell.session().snapshot().mode,
                    MovementMode::Surf
                );
                assert!(
                    resumed
                        .shell
                        .shell
                        .session()
                        .state()
                        .script_runtime
                        .memory
                        .contains_key(CANOE_ACTIVE_MEMORY)
                );
                app = beach(resumed.shell);
                if let Some(dir) = &evidence {
                    record(&mut app, &dir.join("south-after-continue"));
                }
            }
        }
    }
}

#[test]
#[ignore = "requires the gym's native wave clips in MN_BADGE_TRAILS_PACK"]
fn gym_walk_deep_and_shallow_ripples_move_while_the_sand_stays_still() {
    let (_, runtime) = load();
    let root = runtime.materialize_runtime_files().expect("native map art");
    let bank = "mn_gym_6";
    let blocks = &runtime.data().maps[ROOM].blocks;
    for phase in ["day", "nite"] {
        let mut images = Assets::<Image>::default();
        let art = load_tileset_art(
            &root,
            bank,
            phase,
            &runtime.data().tilesets[bank].palette_map,
            &mut images,
        )
        .expect("Brainerd art and animation bank");
        // Co-ordinates below are 8px native glyphs on the installed map.
        for (x, y, moves) in [
            (16, 13, true),
            (21, 13, true),
            (16, 7, false),
            (14, 23, false),
        ] {
            let block = blocks[(y / 4) * 9 + x / 4];
            let sample = |t| {
                let h = art
                    .tile_handle_at_frame(block, x % 4, y % 4, t, false)
                    .expect("visible tile");
                images
                    .get(&h)
                    .and_then(|i| i.data.as_deref())
                    .expect("native RGBA")
            };
            let a = sample(0);
            assert_eq!(a, sample(21));
            assert_eq!(a, sample(88));
            for t in [22, 44, 66] {
                if moves {
                    assert_ne!(sample(t - 22), sample(t), "wave at ({x},{y}) {phase}");
                } else {
                    assert_eq!(a, sample(t), "land at ({x},{y}) {phase}");
                }
            }
        }
    }
}
