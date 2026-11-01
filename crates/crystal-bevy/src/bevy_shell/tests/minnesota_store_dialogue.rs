//! Review the installed Mart customer conversations through production input.
use crystal_core::input::GameButton;

use super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
};
use super::winona::{load, press};

#[test]
#[ignore = "requires store-dialogue review pack in CRYSTAL_WINONA_TEST_PACK"]
fn minnesota_store_customers_speak_and_release_input() {
    let cases: &[(&str, i16, &str, &[&str])] = &[
        (
            "MnBemidjiMart",
            5,
            "MN_BEMIDJI_CUSTOMERText",
            &[
                "Medicine for the\nITASCA trail, and",
                "a snack for my\npartner. The snack",
                "was their idea.",
            ],
        ),
        (
            "MnBrainerdMart",
            5,
            "MN_BRAINERD_MART_CUSTOMERText",
            &[
                "REPELs for the\nroad, and BALLS",
                "for the shore. I\nhope I left room",
                "for lunch!",
            ],
        ),
        (
            "MnGrandRapidsMart",
            6,
            "MN_GR_MART_CUSTOMERText",
            &[
                "It gets cold on\nthe road down. I",
                "make sure my\npartners have",
                "enough energy for\nthe walk back.",
            ],
        ),
        (
            "MnMankatoMart",
            6,
            "MN_MK_SHOPPERText",
            &[
                "POTIONS for the\nprairie, and room",
                "in my pack for the\nBERRIES my",
                "neighbor gave me.",
            ],
        ),
        (
            "MnRochesterMart",
            6,
            "MN_RC_SHOPPERText",
            &[
                "A few POTIONS for\nthe prairie. My",
                "partners are\nrested, so we are",
                "ready to go!",
            ],
        ),
        (
            "MnStCloudMart",
            5,
            "MN_ST_CLOUD_MART_CUSTOMERText",
            &[
                "I use REPELs on\nthe pine road when",
                "I want to watch\nthe birds in",
                "peace.",
            ],
        ),
        (
            "MnWinonaMart",
            6,
            "MN_W_CUSTOMERText",
            &[
                "My partner was\npoisoned on",
                "Driftless Trail. I\nalways carry an",
                "ANTIDOTE now.",
            ],
        ),
    ];

    let (root, runtime) = load();
    for &(map, y, label, pages) in cases {
        let shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
                map_name: map.into(),
                tile_x: 8,
                tile_y: y,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("declared customer review placement");
        let mut controller = VisibleShellController { shell };
        complete_visible_smoke_player_name_if_needed(&mut controller.shell, Some("CHRIS"))
            .expect("review player name");
        controller.set_runtime_journal_enabled(false);
        press(&mut controller, GameButton::Right);
        press(&mut controller, GameButton::A);
        for (index, expected) in pages.iter().enumerate() {
            let snapshot = controller.snapshot().expect("visible customer page");
            let text = snapshot.ui.text.expect("customer dialogue");
            assert_eq!(text.label, label, "{map} page {index}");
            assert_eq!(
                text.asm_text.as_deref(),
                Some(*expected),
                "{map} page {index}"
            );
            if let Some(directory) = std::env::var_os("CRYSTAL_STORE_REVIEW_SAVE_DIR") {
                let directory = std::path::PathBuf::from(directory);
                std::fs::create_dir_all(&directory).expect("review save directory");
                controller
                    .save(directory.join(format!("{map}-{index}.crystalsave")))
                    .expect("customer page review save");
            }
            press(&mut controller, GameButton::A);
        }
        assert!(
            controller
                .snapshot()
                .expect("completed conversation")
                .ui
                .text
                .is_none()
        );
        press(&mut controller, GameButton::Start);
        assert!(
            controller
                .snapshot()
                .expect("released input")
                .ui
                .menu
                .is_some()
        );
        assert_eq!(controller.shell.last_error, None, "{map}");
    }
}
