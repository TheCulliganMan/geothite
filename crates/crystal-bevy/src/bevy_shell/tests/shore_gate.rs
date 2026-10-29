//! Initial approach placement is setup; doors, conversations and returns use input.
use std::collections::BTreeMap;

use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;

use super::super::VisibleShellController;
use super::interior_life::{fixture, load, talk, walk};
use super::r10_gallery::{continue_save, press};

const ROOM: &str = "MnGrandMaraisShoreGate";
const ROAD: &str = "MnMaraisCedarApproach";

fn at(c: &mut VisibleShellController, map: &str, tile: (i16, i16)) {
    let s = c.snapshot().expect("native location");
    assert_eq!(s.overworld.map_name, map);
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), tile);
}
fn cross(c: &mut VisibleShellController, button: GameButton, map: &str, tile: (i16, i16)) {
    for _ in 0..8 {
        if c.snapshot().expect("native crossing").overworld.map_name == map {
            break;
        }
        press(c, button);
    }
    at(c, map, tile);
}
fn step(c: &mut VisibleShellController, button: GameButton, map: &str, tile: (i16, i16)) {
    for _ in 0..4 {
        let visible = c.snapshot().expect("native return approach");
        if visible.overworld.map_name == map
            && (visible.overworld.tile.x, visible.overworld.tile.y) == tile
        {
            break;
        }
        press(c, button);
    }
    at(c, map, tile);
}
fn flags(c: &VisibleShellController, runtime: &CrystalRuntime) -> BTreeMap<String, bool> {
    runtime
        .data()
        .story_event_script_constants
        .global
        .keys()
        .filter(|name| name.starts_with("EVENT_MN_"))
        .map(|name| {
            (
                name.clone(),
                c.shell
                    .shell
                    .session()
                    .state()
                    .flags
                    .is_event_flag_set(name)
                    .expect("native event"),
            )
        })
        .collect()
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing the Cedar shore gate"]
fn shore_gate_controller_both_mouths_lanes_stories_and_continue() {
    let (root, r, _) = load();
    for lane in 0..2 {
        let mut c = fixture(&root, &r, ROAD, (18, 14 + lane));
        let before = flags(&c, &r);
        let money = c.shell.shell.session().state().money;
        let bag = c.shell.shell.session().state().bag.clone();
        cross(&mut c, GameButton::Right, ROOM, (1, 4 + lane));
        c = continue_save(root.clone(), r.clone(), &mut c);
        at(&mut c, ROOM, (1, 4 + lane));
        walk(&mut c, &r, (5, 4));
        // The actual counter redirects A to the attendant two tiles away.
        let words = talk(&mut c, (5, 3));
        assert!(words.contains("grandpa") && words.contains("GRAND MARAIS"));
        walk(&mut c, &r, (7, 4));
        assert!(talk(&mut c, (7, 5)).contains("agates"));
        press(&mut c, GameButton::Start);
        assert!(c.shell.start_menu_cursor.is_some());
        press(&mut c, GameButton::B);
        walk(&mut c, &r, (8, 4 + lane));
        cross(&mut c, GameButton::Right, ROAD, (25, 14 + lane));
        step(&mut c, GameButton::Right, ROAD, (26, 14 + lane));
        step(&mut c, GameButton::Left, ROAD, (25, 14 + lane));
        cross(&mut c, GameButton::Left, ROOM, (8, 4 + lane));
        c = continue_save(root.clone(), r.clone(), &mut c);
        at(&mut c, ROOM, (8, 4 + lane));
        walk(&mut c, &r, (1, 4 + lane));
        cross(&mut c, GameButton::Left, ROAD, (18, 14 + lane));
        assert_eq!(
            flags(&c, &r),
            before,
            "shelter cannot grant regional progression"
        );
        assert_eq!(c.shell.shell.session().state().money, money);
        assert_eq!(c.shell.shell.session().state().bag, bag);
        step(&mut c, GameButton::Left, ROAD, (17, 14 + lane));
    }
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK containing the Cedar shore gate"]
fn shore_gate_controller_retains_both_actual_road_seams() {
    let (root, r, _) = load();
    for lane in 0..2 {
        let mut c = fixture(&root, &r, "MnR12C00R01", (63, 6 + lane));
        cross(&mut c, GameButton::Right, ROAD, (0, 14 + lane));
        cross(&mut c, GameButton::Left, "MnR12C00R01", (63, 6 + lane));
        let mut c = fixture(&root, &r, "MnGrandMarais", (18 + lane, 0));
        cross(&mut c, GameButton::Up, ROAD, (26 + lane, 23));
        cross(&mut c, GameButton::Down, "MnGrandMarais", (18 + lane, 0));
    }
}
