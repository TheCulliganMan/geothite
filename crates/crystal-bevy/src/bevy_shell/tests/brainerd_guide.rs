//! Meet the optional guide from the entrance on four visits to Brainerd's Gym.
use std::path::Path;

use bevy::prelude::{Assets, Entity, Image, Mut, With};
use crystal_core::input::GameButton;

use super::super::{
    BevyRuntimeShell, DialogGlyphMarker, RuntimeBadgeRegion, VisibleShellController,
    mark_runtime_snapshot_dirty, visible_field_dialog_pages,
};
use super::badge_trails::load;
use super::duluth_story::press;
use super::interior_life::fixture;
use super::northstar_dialogue::canonical_pages;
use super::r10_gallery::settle;
use super::winona::{at, travel};
use super::{menu_render_test_app, pokemon_overworld_context_canvas_scaled};

const ROOM: &str = "MnBrainerdGym";

pub(super) fn capture(
    controller: VisibleShellController,
    directory: Option<&Path>,
    name: &str,
) -> VisibleShellController {
    let Some(directory) = directory else {
        return controller;
    };
    let dialogue = controller
        .shell
        .shell
        .presentation_snapshot()
        .expect("capture state")
        .ui
        .text_window_open;
    let mut app = menu_render_test_app(controller.shell);
    app.update();
    app.update();
    if dialogue {
        let mut glyphs = app
            .world_mut()
            .query_filtered::<Entity, With<DialogGlyphMarker>>();
        assert!(
            glyphs.iter(app.world()).next().is_some(),
            "visible hint characters"
        );
    }
    let lcd = app
        .world_mut()
        .resource_scope(|world, images: Mut<Assets<Image>>| {
            pokemon_overworld_context_canvas_scaled(world, &images, 4)
        });
    assert_eq!(lcd.dimensions(), (160, 144));
    std::fs::create_dir_all(directory).expect("native camera capture directory");
    lcd.save(directory.join(name)).expect("save native camera");
    VisibleShellController {
        shell: app
            .world_mut()
            .remove_resource::<BevyRuntimeShell>()
            .expect("return the rendered shell"),
    }
}

#[test]
#[ignore = "requires a Badge Battles edition in MN_BADGE_TRAILS_PACK"]
fn badge_battles_brainerd_guide_pages_follow_the_lake_journey() {
    let (root, runtime) = load();
    let output = std::env::var_os("MN_BADGE_BATTLES_GUIDE_FRAMES").map(std::path::PathBuf::from);
    let object = runtime.data().maps[ROOM]
        .objects
        .iter()
        .find(|o| o.script == "MnBrainerdGymGuide")
        .expect("gym guide");
    assert_eq!((object.x, object.y), (7, 11));

    type Visit = (
        usize,
        &'static [&'static str],
        &'static str,
        &'static [&'static str],
    );
    let scenarios: [Visit; 4] = [
        (
            5,
            &[],
            "MnBrainerdGymGuideText",
            &[
                "sand",
                "sivren",
                "canoe",
                "dark pools",
                "reedraft drains bankmaw's hp",
                "east of town",
                "day",
            ],
        ),
        (
            6,
            &["EVENT_MN_GYM_6_DEFEATED"],
            "MnBrainerdGymGuideAfterBadgeText",
            &["ranger", "badge", "fishing pier"],
        ),
        (
            6,
            &["EVENT_MN_GYM_6_DEFEATED", "EVENT_MN_BRAINERD_REPORT"],
            "MnBrainerdGymGuideAfterReportText",
            &["north", "leech lake trail"],
        ),
        (
            8,
            &[
                "EVENT_MN_GYM_6_DEFEATED",
                "EVENT_MN_BRAINERD_REPORT",
                "EVENT_MN_LEAGUE_CHAMPION",
            ],
            "MnBrainerdGymCircuitIntroText",
            &["leader", "need"],
        ),
    ];
    for (badges, events, label, required_details) in scenarios {
        let pages = canonical_pages(&runtime, ROOM, label);
        let speech = pages.join(" ").replace('\n', " ").to_ascii_lowercase();
        for detail in required_details {
            assert!(
                speech.contains(detail),
                "{label} must retain the journey detail {detail:?}: {speech}"
            );
        }
        let mut c = fixture(&root, &runtime, ROOM, (8, 12));
        for slot in 0..badges {
            c.shell
                .shell
                .award_badge(RuntimeBadgeRegion::Johto, slot)
                .expect("fixture badge");
        }
        for flag in events {
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(flag, true)
                .expect("fixture milestone");
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        let before_badge = badges == 5;
        if before_badge {
            c = capture(c, output.as_deref(), "brainerd-gym-entrance.png");
        }
        travel(&mut c, GameButton::Left, ROOM, (7, 12));
        press(&mut c, GameButton::Up);
        at(&mut c, ROOM, (7, 12));
        press(&mut c, GameButton::A);
        let raw = c
            .shell
            .shell
            .presentation_snapshot()
            .expect("guide opens text");
        let program = visible_field_dialog_pages(&raw, &c.shell).expect("guide's native pages");
        assert_eq!(
            program, pages,
            "{label} page count, line breaks and wording"
        );
        for (index, words) in pages.iter().enumerate() {
            let snapshot = c.snapshot().expect("reached a guide page");
            let text = snapshot.ui.text.expect("guide still speaking");
            assert_eq!(text.label, label);
            assert_eq!(text.asm_text.as_deref(), Some(words.as_str()));
            if before_badge && index >= 2 {
                c = capture(
                    c,
                    output.as_deref(),
                    &format!("brainerd-hint-{}.png", index - 1),
                );
            }
            press(&mut c, GameButton::A);
        }
        if badges == 8 {
            // The next real page lists an unbeaten rematch, so the Champion
            // conversation retains ownership beyond its introductory pages.
            let next = c.snapshot().expect("first rematch entry");
            assert_eq!(
                next.ui.text.expect("missing gym").label,
                "MnBrainerdGymCircuitMissing1Text"
            );
        }
        settle(&mut c, "finish the optional guide conversation");
        travel(&mut c, GameButton::Right, ROOM, (8, 12));
        assert!(
            c.snapshot()
                .expect("player back on the main path")
                .battle
                .is_none()
        );
    }
}
