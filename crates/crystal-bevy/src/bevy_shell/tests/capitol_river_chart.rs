//! Read the library chart and bird sketch from their actual desks and shelves.
use std::path::PathBuf;

use crystal_core::input::GameButton::{A, Left, Up};

use super::super::brainerd_guide::capture;
use super::super::r10_gallery::settle;
use super::{at, fixture, load, press, talk, walk};
use crate::bevy_shell::visible_field_dialog_pages;

#[test]
#[ignore = "requires the Lakeside Stops edition in CRYSTAL_INTERIOR_PACK"]
fn capitol_library_river_chart_is_readable_from_the_aisle_and_the_foot_of_the_desk() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnStPaul", (50, 12));
    walk(&mut c, &runtime, (50, 11));
    assert_eq!(
        c.snapshot().expect("interior arrival").overworld.map_name,
        "MnStPaulCapitolLibrary"
    );
    assert_eq!(at(&mut c), (3, 10));

    walk(&mut c, &runtime, (5, 4));
    assert!(talk(&mut c, (5, 3)).contains("MN_SPX_CAPITOLLIBRARY_HOSTText"));
    let frames = std::env::var_os("MN_LAKESIDE_FRAMES").map(PathBuf::from);
    for (approach, face) in [((11, 4), Left), ((10, 6), Up)] {
        walk(&mut c, &runtime, approach);
        press(&mut c, face);
        assert_eq!(at(&mut c), approach);
        press(&mut c, A);
        let s = c
            .shell
            .shell
            .presentation_snapshot()
            .expect("opened paper chart");
        assert_eq!(
            s.ui.text.as_ref().expect("native text").label,
            "MnCapitolRiverChartText"
        );
        let pages = visible_field_dialog_pages(&s, &c.shell).expect("river chart pages");
        assert_eq!(
            pages.first().map(String::as_str),
            Some("ST. PAUL RIVER\nCanoe chart")
        );
        assert!(pages.join(" ").contains("west bridge south."));
        assert_eq!(
            pages.last().map(String::as_str),
            Some("Turn west for the\nCANOE SHOP.")
        );
        if face == Left {
            c = capture(c, frames.as_deref(), "capitol-library-river-chart.png");
        }
        settle(&mut c, "put the river chart back");
    }
    walk(&mut c, &runtime, (7, 7));
    assert!(talk(&mut c, (8, 7)).contains("MN_INTERIOR_DETAIL_CAPITOL_LENDING_SHELFText"));
    walk(&mut c, &runtime, (3, 11));
    assert_eq!(
        c.snapshot().expect("city sidewalk").overworld.map_name,
        "MnStPaul"
    );
    assert_eq!(at(&mut c), (50, 12));
}
