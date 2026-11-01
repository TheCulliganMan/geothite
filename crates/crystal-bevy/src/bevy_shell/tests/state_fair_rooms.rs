//! Native room stories, every carpet return and remembered fair stamps.
use std::collections::BTreeMap;
use std::path::PathBuf;

use crystal_core::input::GameButton;
use serde_json::Value;

use super::{
    FAIR, STAMPS, at, continue_save, counter, enter, event, fixture_options, leave, press, walk,
};

fn point(value: &Value) -> (u16, u16) {
    (
        u16::try_from(value[0].as_u64().expect("authored x")).expect("core x"),
        u16::try_from(value[1].as_u64().expect("authored y")).expect("core y"),
    )
}

fn printed_story(c: &mut super::VisibleShellController) -> String {
    let mut pages = BTreeMap::new();
    for _ in 0..96 {
        c.wait_frames(240).expect("native story printing");
        if let Some((index, page)) =
            crate::bevy_shell::tests::r10_gallery::completed_visible_page(c)
        {
            pages.insert(index, page);
        }
        let snapshot = c.snapshot().expect("native room story");
        if !snapshot.ui.text_window_open && !c.has_pending_script_work() {
            assert!(
                !pages.is_empty(),
                "room resident must print an actual story"
            );
            return pages.into_values().collect::<Vec<_>>().join(" ");
        }
        let before = at(c);
        press(c, GameButton::Right);
        assert_eq!(at(c), before, "the printed story owns movement input");
        press(c, GameButton::A);
    }
    panic!("room story did not release native input");
}

#[test]
#[ignore = "requires CRYSTAL_FAIR_TEST_PACK and the matching CRYSTAL_FAIR_SOURCE"]
fn state_fair_controller_all_room_stories_every_carpet_and_continue() {
    let source =
        PathBuf::from(std::env::var_os("CRYSTAL_FAIR_SOURCE").expect("matching fair dossier"));
    let design: Value =
        serde_json::from_str(&std::fs::read_to_string(source).expect("fair dossier"))
            .expect("native fair source");
    let (root, runtime, mut c) = fixture_options(false, (93, 65), false, true);
    let town = &design["native_map"];
    let mut residents = 0;
    let mut returns = 0;
    for room in town["interiors"].as_array().expect("three fair rooms") {
        let name = room["id"].as_str().expect("native room");
        let door = town["doors"]
            .as_array()
            .expect("real doors")
            .iter()
            .find(|door| door["target"] == name)
            .expect("paired exterior entrance");
        let outside = point(&door["return_tile"]);
        enter(&mut c, &runtime, outside, name);
        assert_eq!(at(&mut c), point(&room["passive_arrival"]["tile"]));
        for actor in room["residents"].as_array().expect("local residents") {
            if actor["counter_tile"].is_array() {
                counter(&mut c, &runtime, point(&actor["approach"]));
            } else {
                super::super::interact(&mut c, &runtime, point(&actor["tile"]));
            }
            let story = printed_story(&mut c);
            let expected = match actor["id"].as_str().expect("resident identity") {
                "MnFairShowHost" | "MnFairFolkPerformer" => "BEMIDJI",
                "MnFairRangeChorus" => "IRON RANGE",
                "MnFairBarnVolunteer" => "livestock",
                "MnFairBarnMoomoo" => "OATLET",
                "MnFairDairyDemonstrator" | "MnFairButterArtist" => "BUTTERMUSE",
                other => panic!("unreviewed fair room resident: {other}"),
            };
            assert!(story.contains(expected), "{name}: {story}");
            residents += 1;
        }
        for exit in room["exits"].as_array().expect("all native carpets") {
            let carpet = point(&exit["tile"]);
            leave(&mut c, &runtime, (carpet.0, carpet.1 - 1));
            assert_eq!(at(&mut c), outside);
            assert_eq!(
                c.snapshot().expect("actual room return").overworld.map_name,
                FAIR
            );
            returns += 1;
            enter(&mut c, &runtime, outside, name);
            assert_eq!(at(&mut c), point(&room["passive_arrival"]["tile"]));
        }
        let carpet = point(&room["exits"][0]["tile"]);
        leave(&mut c, &runtime, (carpet.0, carpet.1 - 1));
    }
    assert_eq!((residents, returns), (7, 6));
    walk(&mut c, &runtime, (93, 65));
    c = continue_save(&mut c, &root, &runtime);
    assert_eq!(at(&mut c), (93, 65));
    assert!(event(&c, STAMPS[0]) && event(&c, STAMPS[1]));
    assert!(
        !event(&c, STAMPS[2]),
        "visiting halls cannot invent the midway quiz stamp"
    );
}
