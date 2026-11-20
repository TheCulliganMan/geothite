//! Walk into the Minneapolis visitor desk and ask its host for directions.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, Down, Up};

use super::{at, capture_board, fixture, load, press, settle, travel};
use crate::bevy_shell::tests::minnesota_research_access::choices;
use crate::bevy_shell::{selected_vertical_menu, visible_field_dialog_pages};

#[test]
#[ignore = "requires the new South Arrival Desk directory in MN_BADGE_TRAILS_PACK"]
fn south_arrival_directory_lists_destinations_cancels_and_returns_to_the_street() {
    let (root, runtime) = load();
    let city = "MnMinneapolis";
    let room = "MnMinneapolisArrivalInfo";
    let mut c = fixture(&root, &runtime, city, (43, 60));
    travel(&mut c, Up, room, (2, 6));
    travel(&mut c, Up, room, (2, 5));
    let money = c.snapshot().expect("arriving at the desk").trainer.money;
    let frames = std::env::var_os("MN_DIRECTION_BOARD_FRAMES").map(PathBuf::from);
    for (choice, reply) in [
        (
            Some("CLINIC"),
            Some(("MnSouthDeskClinicText", "powerhouse GYM.")),
        ),
        (
            Some("GAME CORNER"),
            Some(("MnSouthDeskArcadeText", "Look for the reels")),
        ),
        (
            Some("SOUTH MARKET"),
            Some(("MnSouthDeskMarketText", "doors east of us.")),
        ),
        (Some("CANCEL"), None),
        (None, None),
    ] {
        press(&mut c, Up);
        press(&mut c, A);
        for _ in 0..6 {
            let snapshot = c.snapshot().expect("desk greeting");
            if c.shell.menu_cursor.is_some() {
                break;
            }
            assert_eq!(
                snapshot.ui.text.expect("host prompt").label,
                "MN_MP_ARRIVALINFO_HOSTText"
            );
            press(&mut c, A);
        }
        let menu = c
            .snapshot()
            .expect("public directory")
            .ui
            .menu
            .expect("open native menu");
        let menu =
            selected_vertical_menu(&menu, &c.shell.menu_cursor).expect("selectable directory");
        let rows = menu
            .options
            .iter()
            .map(|s| s.trim().trim_start_matches('>').trim())
            .collect::<Vec<_>>();
        assert_eq!(rows, ["CLINIC", "GAME CORNER", "SOUTH MARKET", "CANCEL"]);
        choices::select(&mut c, choice);
        if let Some((label, words)) = reply {
            assert_eq!(
                c.snapshot()
                    .expect("destination answer")
                    .ui
                    .text
                    .expect("native answer")
                    .label,
                label
            );
            let snapshot = c
                .shell
                .shell
                .presentation_snapshot()
                .expect("displayed directions");
            let pages = visible_field_dialog_pages(&snapshot, &c.shell).expect("direction pages");
            assert!(pages.join(" ").contains(words), "{label}: {pages:?}");
            if choice == Some("CLINIC") {
                capture_board(
                    &mut c,
                    frames.as_deref(),
                    "south-arrival-clinic-directions.png",
                );
            }
            settle(&mut c, "finish the selected directions");
        } else {
            let after = c.snapshot().expect("cancel the directory");
            assert!(
                after.ui.text.is_none() && after.ui.menu.is_none(),
                "Cancel should give back control"
            );
        }
        at(&mut c, room, (2, 5));
        assert_eq!(c.snapshot().expect("free advice").trainer.money, money);
    }
    travel(&mut c, Down, room, (2, 6));
    travel(&mut c, Down, city, (43, 60));
}
