//! Exercise the revised ambient interactions through the visible controller.
use crystal_core::input::GameButton;

use super::interior_life::fixture;
use super::winona::{at, load, press};

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the ambient dialogue revision"]
fn minnesota_ambient_dialogue_installed_interactions() {
    let (root, runtime) = load();
    for (map, player, label, pages) in [
        (
            "MnWinona",
            (24, 16),
            "MN_W_RIVER_WATCHERText",
            &["Watching barges\ndrift downriver", "is how I relax."][..],
        ),
        (
            "MnWinona",
            (24, 21),
            "MnWStationSignText",
            &["FIELD STATION\nPROF. TAMARACK"][..],
        ),
        (
            "MnGrandRapids",
            (31, 33),
            "MN_GR_RIVER_WATCHERText",
            &[
                "I came for a quiet\nmorning by the",
                "water. The ducks\nhad other plans!",
            ][..],
        ),
    ] {
        let mut controller = fixture(&root, &runtime, map, player);
        press(&mut controller, GameButton::Up);
        at(&mut controller, map, player);
        press(&mut controller, GameButton::A);
        for expected in pages {
            let snapshot = controller.snapshot().expect("visible page");
            let text = snapshot.ui.text.expect("dialogue is open");
            assert_eq!(text.label, label);
            assert_eq!(text.asm_text.as_deref(), Some(*expected));
            press(&mut controller, GameButton::A);
        }
        assert!(
            controller
                .snapshot()
                .expect("closed text")
                .ui
                .text
                .is_none()
        );
        press(&mut controller, GameButton::Start);
        assert!(
            !super::super::visible_start_menu_entries(&controller.shell)
                .expect("Start opens")
                .is_empty()
        );
    }
}
