//! Explicit local placement followed only by production inputs and Continue.
use crystal_core::input::GameButton;

use super::super::VisibleShellController;
use super::authored_route_fixture::indoor_walk;
use super::duluth_story::{finish, fixture_at, save_path};

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("production cabin input");
    c.wait_frames(24).expect("presentation");
    assert_eq!(c.shell.last_error, None);
}
fn at(c: &mut VisibleShellController, map: &str, tile: (i16, i16)) {
    let s = c.snapshot().expect("native position");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), tile);
}
fn enter(c: &mut VisibleShellController) {
    for _ in 0..8 {
        if c.snapshot().expect("entrance").overworld.map_name == "MnRyansHouse" {
            at(c, "MnRyansHouse", (5, 6));
            return;
        }
        press(c, GameButton::Up);
    }
    panic!("real bank door never entered Ryan's house");
}

#[test]
#[ignore = "requires the published external Ryan's house pack"]
fn ryans_house_controller_enters_talks_reads_sign_exits_both_carpets_and_continues() {
    let (root, runtime, mut c) =
        fixture_at("MnR11C00R00", (23, 24), [false; 8], false, false, false);
    let initial = c.shell.shell.session().state().clone();
    enter(&mut c);
    indoor_walk(&mut c, &runtime, (5, 3));
    press(&mut c, GameButton::Right);
    press(&mut c, GameButton::A);
    let text = c
        .snapshot()
        .expect("Ryan speaking")
        .ui
        .text
        .expect("native dialogue");
    assert_eq!(text.label, "MN_RYANText");
    assert!(text.asm_text.expect("visible text").contains("RYAN"));
    press(&mut c, GameButton::Down);
    at(&mut c, "MnRyansHouse", (5, 3));
    finish(&mut c);
    let save = save_path("ryans-house-interior");
    c.save(&save).expect("inside save");
    c = VisibleShellController::load_save(root.clone(), runtime.clone(), save, None)
        .expect("Continue indoors");
    at(&mut c, "MnRyansHouse", (5, 3));
    for x in [5, 6] {
        indoor_walk(&mut c, &runtime, (x, 6));
        for _ in 0..8 {
            if c.snapshot().expect("exit").overworld.map_name == "MnR11C00R00" {
                break;
            }
            press(&mut c, GameButton::Down);
        }
        at(&mut c, "MnR11C00R00", (23, 22));
        if x == 5 {
            enter(&mut c);
        }
    }
    indoor_walk(&mut c, &runtime, (26, 23));
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    let text = c
        .snapshot()
        .expect("sign")
        .ui
        .text
        .expect("actual readable sign");
    assert!(text.asm_text.expect("sign text").contains("RYAN'S HOUSE"));
    finish(&mut c);
    indoor_walk(&mut c, &runtime, (23, 24));
    let save = save_path("ryans-house-porch");
    c.save(&save).expect("bank save");
    c = VisibleShellController::load_save(root, runtime, save, None).expect("Continue outdoors");
    at(&mut c, "MnR11C00R00", (23, 24));
    let state = c.shell.shell.session().state();
    // Map transitions materialize temporary false entries; compare the actual
    // event/engine values, including every key on either side of Continue.
    for flag in state
        .flags
        .event_flags
        .keys()
        .chain(initial.flags.event_flags.keys())
    {
        assert_eq!(
            state.flags.is_event_flag_set(flag),
            initial.flags.is_event_flag_set(flag),
            "event {flag}"
        );
    }
    for flag in state
        .flags
        .engine_flags
        .keys()
        .chain(initial.flags.engine_flags.keys())
    {
        assert_eq!(
            state.flags.is_engine_flag_set(flag),
            initial.flags.is_engine_flag_set(flag),
            "engine {flag}"
        );
    }
    assert_eq!(state.badges, initial.badges);
    assert_eq!(state.storage.party, initial.storage.party);
    assert_eq!(state.bag, initial.bag);
    assert_eq!(state.money, initial.money);
}
