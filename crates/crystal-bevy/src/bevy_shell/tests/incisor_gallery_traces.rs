//! Installed guide pages and the native view while facing each Castorock trace.
use std::path::PathBuf;

use bevy::prelude::{Assets, Image, Mut};
use crystal_core::input::GameButton;

use super::super::mark_runtime_snapshot_dirty;
use super::badge_trails::load;
use super::interior_life::{fixture, talk};
use super::r10_gallery::press;
use super::{menu_render_test_app, pokemon_overworld_context_canvas_scaled};

const ROOM: &str = "MnR03Cave1F";

#[test]
#[ignore = "requires the revised cave in MN_BADGE_TRAILS_PACK"]
fn cave_returns_incisor_guide_points_to_the_next_visible_trace() {
    let (root, runtime) = load();
    let output = std::env::var_os("MN_CAVE_TRACE_FRAMES").map(PathBuf::from);
    if let Some(directory) = output {
        std::fs::create_dir_all(&directory).expect("local cave camera output");
        for (name, point, facing) in [
            ("pool", (3, 5), GameButton::Left),
            ("pillar", (19, 4), GameButton::Right),
            ("bed", (23, 12), GameButton::Down),
        ] {
            let mut c = fixture(&root, &runtime, ROOM, point);
            press(&mut c, facing);
            let mut app = menu_render_test_app(c.shell);
            app.update();
            app.update();
            let image = app
                .world_mut()
                .resource_scope(|world, images: Mut<Assets<Image>>| {
                    pokemon_overworld_context_canvas_scaled(world, &images, 4)
                });
            assert_eq!(image.dimensions(), (160, 144));
            image
                .save(directory.join(format!("{name}.png")))
                .expect("capture native cave pixels");
        }
    }
    let progress = [
        (&[][..], "MnJawGuidePoolText"),
        (&["POOL"][..], "MnJawGuidePillarText"),
        (&["POOL", "PILLAR"][..], "MnJawGuideBedText"),
        (&["TRACKS_READ"][..], "MnJawGuideFoundText"),
        (&["TRACKS_READ", "CAUGHT"][..], "MnJawGuideCaughtText"),
    ];
    for (events, label) in progress {
        for carries_hm in [false, true] {
            let mut c = fixture(&root, &runtime, ROOM, (4, 5));
            let state = c.shell.shell.session_mut().state_mut();
            for event in events {
                state
                    .flags
                    .set_event_flag(&format!("EVENT_MN_CASTOROCK_{event}"), true)
                    .expect("known clue");
            }
            if carries_hm {
                assert!(
                    state
                        .bag
                        .add_item(&runtime.data().items["HM_STRENGTH"], 1)
                        .expect("acquired HM")
                );
            }
            mark_runtime_snapshot_dirty(&mut c.shell);
            let pages = talk(&mut c, (5, 5));
            assert!(
                pages.contains(label),
                "expected {label} from guide: {pages}"
            );
            assert_eq!(
                pages.contains("MnJawGuideStrengthText"),
                !carries_hm && !events.contains(&"TRACKS_READ")
            );
        }
    }
}
