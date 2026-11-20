//! Read both floor displays through the normal panel interaction and scroll the full list.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, B, Down, Up};
use crystal_tui_view::{PaintedRenderer, RuntimeTextRenderer};

use super::{CAB, LOBBY, board, panel};
use crate::bevy_shell::VisibleShellController;
use crate::bevy_shell::tests::brainerd_guide::capture;
use crate::bevy_shell::tests::interior_life::{fixture, load, walk};
use crate::bevy_shell::tests::winona::{at, press, travel};

fn assert_panel(c: &mut VisibleShellController, floor: &str, rows: [&str; 4]) {
    let snapshot = c.snapshot().expect("elevator display");
    assert_eq!(
        snapshot
            .ui
            .text
            .as_ref()
            .expect("current floor and question")
            .asm_text,
        Some(format!("Which floor?\nNow on: {floor}"))
    );
    let menu = snapshot.ui.menu.as_ref().expect("floor selector");
    assert_eq!(menu.layout.vertical_menus[0].options, rows);
    let text = RuntimeTextRenderer::default().render(&snapshot);
    let buffer = PaintedRenderer::default().draw_native(&snapshot, &text, 80, 24, None, false);
    let output: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
    for line in ["Which floor?", &format!("Now on: {floor}")] {
        assert!(output.contains(line), "{line} is visible in the terminal");
    }
    let selected = rows
        .iter()
        .find_map(|row| row.strip_prefix('>'))
        .expect("one highlighted row");
    assert!(output.contains(&format!("> {selected}")));
}

#[test]
#[ignore = "requires the Lakeside Stops edition in CRYSTAL_INTERIOR_PACK"]
fn store_elevator_displays_current_floor_and_scrolls_to_the_top_destination() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnMinneapolis", (54, 36));
    let frames = std::env::var_os("MN_LAKESIDE_FRAMES").map(PathBuf::from);
    travel(&mut c, Up, LOBBY, (7, 6));
    board(&mut c, &runtime);
    panel(&mut c, &runtime);
    assert_panel(&mut c, "1F", [">1F", " 2F", " 3F", " 4F"]);
    c = capture(c, frames.as_deref(), "store-elevator-floor-list.png");
    for _ in 0..4 {
        press(&mut c, Down);
    }
    assert_panel(&mut c, "1F", [" 2F", " 3F", " 4F", ">5F"]);
    c = capture(c, frames.as_deref(), "store-elevator-floor-list-5f.png");
    press(&mut c, A);
    c.wait_frames(180).expect("finish the ride");
    at(&mut c, CAB, (3, 1));
    walk(&mut c, &runtime, (1, 3));
    at(&mut c, "MnMinneapolisDepartmentStore5F", (2, 1));
    board(&mut c, &runtime);
    panel(&mut c, &runtime);
    assert_panel(&mut c, "5F", [">1F", " 2F", " 3F", " 4F"]);
    press(&mut c, B);
    let snapshot = c.snapshot().expect("dismissed panel");
    assert!(snapshot.ui.menu.is_none());
    at(&mut c, CAB, (3, 1));
}
