//! Original residents and rewards remain reachable through the revised terrain.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crystal_assets::{AssetRoot, read_loaded_verified_compiled_game_pack};
use crystal_core::input::GameButton;
use crystal_runtime::CrystalRuntime;
use serde_json::Value;

use super::super::super::{
    BevyShellConfig, BevyShellStart, VisibleShellController,
    complete_visible_smoke_player_name_if_needed, initialize_bevy_runtime_shell,
    visible_field_dialog_pages,
};
use super::super::authored_route_fixture::neighbors;
use super::super::r10_gallery::{completed_visible_page, continue_save, press, settle, tile};
use super::{kind, point, walk};

fn printed_pages(c: &mut VisibleShellController) -> BTreeMap<usize, String> {
    let mut pages = BTreeMap::new();
    for _ in 0..64 {
        c.wait_frames(240).expect("print the complete native page");
        if let Some((index, page)) = completed_visible_page(c) {
            pages.insert(index, page);
        }
        let snapshot = c.snapshot().expect("dialogue ownership");
        if !snapshot.ui.text_window_open
            && snapshot.ui.pending_text_wait.is_none()
            && visible_field_dialog_pages(&snapshot, &c.shell).is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
            && c.shell.visible_field_item_notice.is_none()
        {
            return pages;
        }
        press(c, GameButton::A);
    }
    panic!("resident or reward did not release input: {pages:?}");
}

#[test]
#[ignore = "requires the published nine-cave pack; explicit local empty-party fixtures"]
fn cave_quality_controller_original_residents_rewards_once_and_continue() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_CAVE_QUALITY_PACK").expect("actual nine-cave publication"),
    )
    .canonicalize()
    .expect("local pack");
    let root = AssetRoot::new(path.parent().expect("pack parent"));
    let runtime = CrystalRuntime::from_loaded_compiled_pack(
        &root,
        read_loaded_verified_compiled_game_pack(&path).expect("verified pack"),
    )
    .expect("native runtime");
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../planning/minnesota/caves/johto-quality-v01");
    let mut room_count = 0;
    for entry in std::fs::read_dir(directory).expect("authored caves") {
        let path = entry.expect("source entry").path();
        if path.extension().is_none_or(|extension| extension != "json")
            || path.to_string_lossy().ends_with(".previous.json")
        {
            continue;
        }
        room_count += 1;
        let source: Value =
            serde_json::from_str(&std::fs::read_to_string(path).expect("authored cave source"))
                .expect("source JSON");
        let room = source["id"].as_str().expect("room identity");
        let module = &runtime.data().maps[room];
        let arrival = point(&source["clear_arrival"]);
        let shell = initialize_bevy_runtime_shell(
            root.clone(),
            runtime.clone(),
            BevyShellStart::NewGameAtRuntimeTile {
                spawn_identifier: runtime.title_new_game_spawn_identifier().expect("HOME"),
                map_name: room.into(),
                tile_x: arrival.0,
                tile_y: arrival.1,
            },
            BevyShellConfig {
                smoke_player_name: Some("CHRIS".into()),
                ..Default::default()
            },
        )
        .expect("local cave fixture");
        let mut c = VisibleShellController { shell };
        complete_visible_smoke_player_name_if_needed(&mut c.shell, Some("CHRIS"))
            .expect("fixture name");
        c.set_runtime_journal_enabled(false);
        // Empty party and cave position are setup. No subsequent game state writes.
        let bodies = module
            .objects
            .iter()
            .map(|object| (object.x as i16, object.y as i16))
            .collect::<BTreeSet<_>>();
        let mut floor = BTreeSet::new();
        for y in 0..module.attributes.height as i16 * 2 {
            for x in 0..module.attributes.width as i16 * 2 {
                if kind(&runtime, module, (x, y)) == "FLOOR" && !bodies.contains(&(x, y)) {
                    floor.insert((x, y));
                }
            }
        }
        for actor in source["residents"]
            .as_array()
            .expect("retained residents")
            .iter()
            .chain(source["items"].as_array().expect("retained items"))
        {
            let body = point(&actor["tile"]);
            let (approach, away) = neighbors(body)
                .into_iter()
                .find(|(at, _)| floor.contains(at))
                .expect("clear actor approach");
            let toward = match away {
                GameButton::Up => GameButton::Down,
                GameButton::Down => GameButton::Up,
                GameButton::Left => GameButton::Right,
                GameButton::Right => GameButton::Left,
                _ => unreachable!("orthogonal neighbor"),
            };
            walk(&mut c, module, &floor, approach);
            press(&mut c, toward);
            press(&mut c, GameButton::A);
            let snapshot = c.snapshot().expect("visible interaction");
            assert!(snapshot.ui.text_window_open, "{room}: {actor}");
            let before = tile(&mut c);
            press(&mut c, away);
            assert_eq!(tile(&mut c), before, "text owns movement");
            let pages = printed_pages(&mut c);
            assert!(
                !pages.is_empty(),
                "{room}: native pages must actually print"
            );
            if let Some(item) = actor["item"].as_str() {
                let flag = actor["flag"].as_str().expect("original pickup flag");
                let expected = actor["quantity"].as_u64().expect("quantity") as u16;
                let state = c.shell.shell.session().state();
                assert!(state.flags.is_event_flag_set(flag).expect("retained flag"));
                assert_eq!(state.bag.quantity(&runtime.data().items[item]), expected);
                c = continue_save(root.clone(), runtime.clone(), &mut c);
                press(&mut c, GameButton::A);
                settle(&mut c, "collected reward cannot repeat");
                assert_eq!(
                    c.shell
                        .shell
                        .session()
                        .state()
                        .bag
                        .quantity(&runtime.data().items[item]),
                    expected,
                    "{room}: reward remains once after Continue"
                );
            } else {
                let printed = pages
                    .values()
                    .flat_map(|page| page.split_whitespace())
                    .collect::<String>();
                for line in actor["lines"].as_array().expect("original resident lines") {
                    let expected = line
                        .as_str()
                        .expect("line")
                        .split_whitespace()
                        .collect::<String>();
                    assert!(
                        printed.contains(&expected),
                        "{room}: missing {line} in {pages:?}"
                    );
                }
            }
        }
        walk(&mut c, module, &floor, arrival);
    }
    assert_eq!(room_count, 9, "every authored cave is covered");
}
