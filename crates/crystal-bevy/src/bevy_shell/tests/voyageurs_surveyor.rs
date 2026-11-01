//! A local winter story uses the existing dry landing and original ladder loop.
use crystal_core::input::GameButton;

use super::super::interior_life::{fixture, load, talk};
use super::{
    LOWER, UPPER, completed_visible_page, continue_save, kind, map, press, settle, tile, travel,
};

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the Voyageurs surveyor overlay"]
fn voyageurs_surveyor_talks_on_dry_landing_owns_input_and_returns_after_continue() {
    let (root, runtime, _) = load();
    let room = &runtime.data().maps[LOWER];
    let resident = room
        .objects
        .iter()
        .find(|o| o.script == "MN_INTERIOR_VOYAGEURS_SURVEYOR")
        .expect("canoe guide on the lower landing");
    assert_eq!((resident.x, resident.y), (3, 20));
    assert_eq!(kind(&runtime, room, (3, 20)), "FLOOR");
    assert_eq!(kind(&runtime, room, (4, 20)), "FLOOR");
    let mut controller = fixture(&root, &runtime, LOWER, (5, 19));
    travel(&mut controller, &runtime, (4, 20), "");
    press(&mut controller, GameButton::Left);
    press(&mut controller, GameButton::A);
    controller.wait_frames(240).expect("first surveyor page");
    let first = completed_visible_page(&mut controller).expect("visible surveyor story");
    let stand = tile(&mut controller);
    press(&mut controller, GameButton::Up);
    assert_eq!(
        tile(&mut controller),
        stand,
        "dialogue owns a legal floor step"
    );
    press(&mut controller, GameButton::A);
    controller.wait_frames(240).expect("next surveyor page");
    let second = completed_visible_page(&mut controller).expect("next visible story page");
    assert_ne!(first, second, "A advances the actual conversation");
    settle(&mut controller, "surveyor releases input");
    controller = continue_save(root, runtime.clone(), &mut controller);
    let story = talk(&mut controller, (3, 20));
    for local_detail in ["Voyageurs", "grandson", "dancing loon"] {
        assert!(
            story.contains(local_detail),
            "missing local detail: {story}"
        );
    }
    travel(&mut controller, &runtime, (5, 21), UPPER);
    assert_eq!(map(&mut controller), UPPER);
    assert_eq!(tile(&mut controller), (23, 6));
}
