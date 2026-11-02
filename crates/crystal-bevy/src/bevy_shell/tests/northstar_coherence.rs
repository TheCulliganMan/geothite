//! Exercise the revised local conversations using production controller input.
use crystal_core::input::GameButton;

use super::super::mark_runtime_snapshot_dirty;
use super::interior_life::{collision, fixture};
use super::winona::{load, press};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the coherence candidate"]
fn coherence_ambient_conversations_show_reviewed_pages_and_release_input() {
    let (root, runtime) = load();
    let cases: &[(&str, &str, &str, &[&str])] = &[
        (
            "MnR09Cave1F",
            "MN_NDRILL_NELL",
            "MnNDrillScoutQuestText",
            &[
                "NELL: FINN and his\ncrew are trapped",
                "below the drill.",
                "It cut green\ncopper ore.",
                "Runoff reached the\nlower pools.",
                "LODE kept the\ndrill running.",
                "Please help them\nget out!",
            ],
        ),
        (
            "MnMinneapolis",
            "MN_MP_ARRIVAL_GUIDE",
            "MN_MP_ARRIVAL_GUIDEText",
            &["The clinic is\nnortheast.", "Rail dispatch is\nnorthwest."],
        ),
        (
            "MnMinneapolis",
            "MN_MP_PARK_WITNESS",
            "MN_MP_PARK_WITNESSText",
            &[
                "I saw people in\npolice uniforms",
                "take a worker who\nspoke up.",
                "Their badges\nlooked odd to me.",
            ],
        ),
        (
            "MnBrainerdRangerStation",
            "MN_BRAINERD_VOLUNTEER",
            "MN_BRAINERD_VOLUNTEERText",
            &["Have a field\nreport?", "The ranger outside\ncan take it."],
        ),
        (
            "MnGrandMarais",
            "MnGrandMaraisBirdWatcher",
            "MnGrandMaraisBirdWatcherText",
            &["I walk softly.", "One quick step and\nbirds take off."],
        ),
        (
            "MnGrandRapidsPokecenter1F",
            "MN_GR_CLINIC_VISITOR",
            "MN_GR_CLINIC_VISITORText",
            &[
                "I came in for\na little rest.",
                "My POKEMON had\nthe same idea!",
            ],
        ),
        (
            "MnRochesterFossilLab",
            "MnRochesterFossilLabAssistant",
            "MnRochesterFossilLabAssistantText",
            &[
                "We study fossils.",
                "Bring one if you\nfind a specimen.",
                "Our researcher can\ntry to restore it.",
            ],
        ),
        (
            "MnRochesterGym",
            "MnRochesterGymGuide",
            "MnRochesterGymGuideText",
            &[
                "Follow the gaps\nbetween planters.",
                "Beat TAVEL, then\nELVIK. VAREL waits",
                "at the far end.",
            ],
        ),
    ];
    for (map_name, script, label, expected_pages) in cases {
        let map = &runtime.data().maps[*map_name];
        let actor = map
            .objects
            .iter()
            .find(|actor| actor.script == *script)
            .expect("actor");
        let actor_x = i16::try_from(actor.x).expect("actor x");
        let actor_y = i16::try_from(actor.y).expect("actor y");
        let (at, facing) = [
            ((actor_x, actor_y + 1), GameButton::Up),
            ((actor_x - 1, actor_y), GameButton::Right),
            ((actor_x, actor_y - 1), GameButton::Down),
            ((actor_x + 1, actor_y), GameButton::Left),
        ]
        .into_iter()
        .find(|(point, _)| {
            collision(&runtime, map, *point) == Some("FLOOR")
                && !map.objects.iter().any(|object| {
                    i16::try_from(object.x).ok() == Some(point.0)
                        && i16::try_from(object.y).ok() == Some(point.1)
                })
        })
        .expect("open adjacent tile");
        let mut controller = fixture(&root, &runtime, map_name, at);
        press(&mut controller, facing);
        press(&mut controller, GameButton::A);
        for (index, expected) in expected_pages.iter().enumerate() {
            let snapshot = controller.snapshot().expect("visible page");
            let text = snapshot.ui.text.expect("visible dialogue");
            assert_eq!(&text.label, label, "{map_name} page {index}");
            assert_eq!(
                text.asm_text.as_deref(),
                Some(*expected),
                "{map_name} page {index}"
            );
            press(&mut controller, GameButton::A);
        }
        assert!(
            !controller
                .snapshot()
                .expect("released dialogue")
                .ui
                .text_window_open,
            "{map_name}"
        );
        press(&mut controller, GameButton::Start);
        assert!(
            controller
                .snapshot()
                .expect("Start after dialogue")
                .ui
                .menu
                .is_some(),
            "{map_name}"
        );
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the coherence candidate"]
fn cousin_replies_do_not_assume_private_progress() {
    let (root, runtime) = load();
    let mut controller = fixture(&root, &runtime, "MnWinonaHome2F", (2, 3));
    for (flag, label, pages) in [
        (
            "EVENT_MN_STARTER_RECEIVED",
            "MnWinonaCousinStarterText",
            [
                "COUSIN: Hi! How's\nyour day going?",
                "Tell me all\nabout it!",
            ],
        ),
        (
            "EVENT_MN_DELIVERED_ROCHESTER_PARCEL",
            "MnWinonaCousinRochesterText",
            [
                "COUSIN: You're\nhome! Any stories?",
                "Tell me what\nyou've been up to.",
            ],
        ),
    ] {
        controller
            .shell
            .shell
            .session_mut()
            .state_mut()
            .flags
            .set_event_flag(flag, true)
            .expect("declared progress fixture");
        mark_runtime_snapshot_dirty(&mut controller.shell);
        press(&mut controller, GameButton::Left);
        press(&mut controller, GameButton::A);
        for expected in pages {
            let text = controller
                .snapshot()
                .expect("cousin page")
                .ui
                .text
                .expect("cousin text");
            assert_eq!(text.label, label);
            assert_eq!(text.asm_text.as_deref(), Some(expected));
            press(&mut controller, GameButton::A);
        }
        assert!(
            !controller
                .snapshot()
                .expect("dialogue ended")
                .ui
                .text_window_open
        );
    }
    press(&mut controller, GameButton::Start);
    assert!(
        controller
            .snapshot()
            .expect("Start after cousin")
            .ui
            .menu
            .is_some()
    );
}
