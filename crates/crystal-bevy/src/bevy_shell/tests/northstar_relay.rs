//! Explicit upstream access/party fixture; the chief and relay use real input.
use crystal_core::input::GameButton;

use super::{
    RESTORED, ROOM, Setup, answer, assert_at, assert_no_safari, chief, continue_save, controller,
    enter_shelter, event, finish_dialogue, load, play_chief_battle, press, prompt, receiver, walk,
};

#[test]
#[ignore = "requires native relay/species pack via CRYSTAL_BOUNDARY_BASECAMP_TEST_PACK"]
fn northstar_relay_picture_owns_input_and_console_stays_off_after_continue_and_reentry() {
    let (root, runtime) = load();
    let mut c = controller(&root, &runtime, Setup::default());
    receiver(&mut c, &runtime);
    assert!(finish_dialogue(&mut c).contains("MnBoundarySurveyReceiverLockedText"));
    chief(&mut c, &runtime);
    play_chief_battle(&mut c, false);
    finish_dialogue(&mut c);
    c = continue_save(&mut c, &root, &runtime, "relay-chief-win");
    let won = c.snapshot().expect("post-chief resources");
    let money = c.shell.shell.session().state().money;
    receiver(&mut c, &runtime);
    prompt(&mut c);
    answer(&mut c, false);
    finish_dialogue(&mut c);
    assert!(!event(&c, RESTORED));
    receiver(&mut c, &runtime);
    prompt(&mut c);
    answer(&mut c, true);
    assert!(!event(&c, RESTORED));
    let mut found = false;
    for _ in 0..96 {
        let s = c.snapshot().expect("native feed progression");
        assert!(s.battle.is_none());
        assert!(!event(&c, RESTORED), "picture must precede restoration");
        if s.ui.active_pokemon_picture.as_deref() == Some("KETTLURK") {
            found = true;
            break;
        }
        press(&mut c, GameButton::A);
    }
    assert!(
        found,
        "the actual native Kettlurk picture must be presented"
    );
    assert_at(&mut c, ROOM, (12, 4));
    let picture = c.snapshot().expect("native picture wait");
    assert!(!picture.ui.text_window_open);
    assert!(
        picture.ui.pending_text_wait.is_some(),
        "the picture wait must retain a source cursor for Continue"
    );
    press(&mut c, GameButton::Down);
    assert_at(&mut c, ROOM, (12, 4));
    assert_eq!(
        c.snapshot()
            .expect("picture owns input")
            .ui
            .active_pokemon_picture
            .as_deref(),
        Some("KETTLURK")
    );
    c = continue_save(&mut c, &root, &runtime, "relay-kettlurk-picture");
    assert_eq!(
        c.snapshot()
            .expect("continued native picture")
            .ui
            .active_pokemon_picture
            .as_deref(),
        Some("KETTLURK")
    );
    assert!(!event(&c, RESTORED));
    press(&mut c, GameButton::A);
    press(&mut c, GameButton::Down);
    assert_at(&mut c, ROOM, (12, 4));
    for _ in 0..96 {
        if c.snapshot()
            .expect("cut-link continuation")
            .ui
            .text
            .as_ref()
            .is_some_and(|t| t.label == "MnNorthstarRelayCutText")
        {
            break;
        }
        assert!(!event(&c, RESTORED));
        press(&mut c, GameButton::A);
    }
    for page in [
        "You cut the link.\nThe motors coast.",
        "The cage opens.\nThe current turns.",
    ] {
        let s = c.snapshot().expect("acknowledged cut-link page");
        assert_eq!(s.ui.text.and_then(|t| t.asm_text).as_deref(), Some(page));
        assert!(!event(&c, RESTORED));
        press(&mut c, GameButton::Down);
        assert_at(&mut c, ROOM, (12, 4));
        press(&mut c, GameButton::A);
    }
    let labels = finish_dialogue(&mut c);
    assert!(labels.contains("MnBoundarySurveyReceiverDoneText"));
    assert!(event(&c, RESTORED));
    let off = *c.shell.shell.session().state().map_block_overrides[ROOM]
        .get(&(6, 1))
        .expect("actual console power-down");
    assert_ne!(off, runtime.data().maps[ROOM].blocks[14]);
    assert!(
        c.snapshot()
            .expect("picture closed")
            .ui
            .active_pokemon_picture
            .is_none()
    );
    assert_eq!(
        c.snapshot().expect("no capture/party reward").party,
        won.party
    );
    assert_eq!(c.snapshot().expect("no item reward").bag, won.bag);
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_no_safari(&c);
    c = continue_save(&mut c, &root, &runtime, "relay-powered-down");
    walk(&mut c, &runtime, (8, 10));
    super::super::winona::travel(&mut c, GameButton::Down, "MnBoundaryWaters", (43, 20));
    enter_shelter(&mut c);
    assert_eq!(
        c.shell.shell.session().state().map_block_overrides[ROOM].get(&(6, 1)),
        Some(&off)
    );
    receiver(&mut c, &runtime);
    let labels = finish_dialogue(&mut c);
    assert!(labels.contains("MnBoundarySurveyReceiverAlreadyText"));
    assert!(!labels.contains("MnNorthstarRelayFeedText"));
    assert!(
        c.snapshot()
            .expect("no feed replay")
            .ui
            .active_pokemon_picture
            .is_none()
    );
    assert_eq!(c.shell.shell.session().state().money, money);
    assert_no_safari(&c);
}
