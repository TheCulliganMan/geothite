//! Explicit room/party fixtures; all traversal and interaction use production input.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use serde_json::Value;

use super::super::{VisibleShellController, mark_runtime_snapshot_dirty};
use super::interior_life::{fixture, load, resident, talk, target_name, travel, walk};

fn press(c: &mut VisibleShellController, button: GameButton) {
    c.press(button).expect("native room input");
    c.wait_frames(16).expect("settle room input");
}

fn visit_shop(c: &mut VisibleShellController, from: (i16, i16), actor: (i16, i16)) {
    let button = match (actor.0 - from.0, actor.1 - from.1) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        _ => panic!("shop approach must be adjacent"),
    };
    press(c, button);
    press(c, GameButton::A);
    for _ in 0..64 {
        if c.shell.shop_top_cursor.is_some()
            && c.shell.shop_welcome_seen
            && c.shell.shop_notice.is_none()
        {
            break;
        }
        press(c, GameButton::A);
    }
    let shop = c
        .snapshot()
        .expect("native room shop")
        .pending_shop
        .expect("original mart");
    assert!(!shop.inventory.is_empty());
    press(c, GameButton::Left);
    let s = c.snapshot().expect("shop owns movement");
    assert_eq!((s.overworld.tile.x, s.overworld.tile.y), from);
    for _ in 0..96 {
        let s = c.snapshot().expect("shop closing");
        if s.pending_shop.is_none()
            && !s.ui.text_window_open
            && s.ui.menu.is_none()
            && !c.has_pending_script_work()
            && c.shell.field_notice.is_none()
        {
            return;
        }
        let button = if s.pending_shop.is_none() || c.shell.shop_notice.is_some() {
            GameButton::A
        } else if let Some(cursor) = &c.shell.shop_top_cursor {
            if cursor.option_index == 2 {
                GameButton::A
            } else {
                GameButton::Down
            }
        } else {
            GameButton::B
        };
        press(c, button);
    }
    panic!("original room shop did not release input");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the authored room layout overlay"]
fn interior_layouts_every_resident_furniture_detail_and_return_works() {
    let (root, runtime, _) = load();
    let source = std::fs::read_to_string(
        std::env::var_os("CRYSTAL_INTERIOR_LAYOUT_SOURCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../planning/minnesota/interior-life/layouts.json")
            }),
    )
    .expect("layout contract");
    let plan: Value = serde_json::from_str(&source).expect("room layouts");
    let mut details = 0;
    for room in plan["rooms"].as_array().expect("rooms") {
        let name = room["map"].as_str().expect("room");
        let point = |v: &Value| {
            (
                v[0].as_i64().expect("x") as i16,
                v[1].as_i64().expect("y") as i16,
            )
        };
        let mut c = fixture(&root, &runtime, name, point(&room["arrival"]));
        // Actual battles are exercised by the optional city-trainer regression.
        // This fixture reads their post-match conversations and room details.
        for trainer in runtime.data().maps[name].trainer_scripts.values() {
            c.shell
                .shell
                .session_mut()
                .state_mut()
                .flags
                .set_event_flag(&trainer.event_flag, true)
                .expect("prior optional match fixture");
        }
        mark_runtime_snapshot_dirty(&mut c.shell);
        for actor in room["actor_approaches"]
            .as_array()
            .expect("resident approaches")
        {
            walk(&mut c, &runtime, point(&actor["tile"]));
            let script = actor["script"].as_str().expect("resident");
            let target = actor
                .get("counter_tile")
                .map(point)
                .unwrap_or_else(|| resident(&runtime, name, script));
            if runtime.data().maps[name]
                .script_shop_commands
                .iter()
                .any(|command| command.source_script == script)
            {
                visit_shop(&mut c, point(&actor["tile"]), target);
                continue;
            }
            let text = talk(&mut c, target);
            assert!(!text.is_empty(), "visible room resident {name}");
        }
        for exhibit in room["exhibits"].as_array().expect("furniture details") {
            walk(&mut c, &runtime, point(&exhibit["approach"]));
            let text = talk(&mut c, point(&exhibit["tile"]));
            assert!(
                text.contains(exhibit["id"].as_str().expect("detail label")),
                "native furniture text in {name}: {text}"
            );
            details += 1;
        }
        let warp = &runtime.data().maps[name].events.warps[0];
        walk(&mut c, &runtime, (warp.x as i16, warp.y as i16));
        assert_eq!(
            c.snapshot()
                .expect("original room return")
                .overworld
                .map_name,
            target_name(&runtime, &warp.target_map)
        );
    }
    let expected = plan["rooms"]
        .as_array()
        .expect("rooms")
        .iter()
        .map(|room| room["exhibits"].as_array().expect("details").len())
        .sum::<usize>();
    assert_eq!(details, expected, "all authored furniture interactions");
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the authored room layout overlay"]
fn interior_layouts_hostel_care_stairs_bed_note_and_continue_keep_the_original_service() {
    let (root, runtime, _) = load();
    let mut c = fixture(&root, &runtime, "MnDuluthHillsideHostel1F", (8, 9));
    let mut partner = runtime
        .data()
        .create_pokemon("CHIKORITA", 20, Dv::default())
        .expect("legal partner");
    partner.hp = 10;
    partner.status = Some("POISON".into());
    for m in &mut partner.moves {
        m.current_pp = 0;
    }
    let state = c.shell.shell.session_mut().state_mut();
    assert!(state.storage.party.add_pokemon(partner));
    state.sync_party_from_storage();
    state.money = 0;
    mark_runtime_snapshot_dirty(&mut c.shell);
    walk(&mut c, &runtime, (5, 6));
    talk(&mut c, (5, 5));
    let s = c.snapshot().expect("original hostel care");
    let partner = &s.party.slots[0].pokemon;
    assert_eq!(partner.hp, partner.max_hp);
    assert!(partner.status.is_none());
    assert!(partner.moves.iter().all(|m| m.current_pp > 0));
    assert_eq!(c.shell.shell.session().state().money, 0);
    travel(&mut c, &runtime, "MnDuluthHillsideHostel2F");
    walk(&mut c, &runtime, (3, 4));
    let text = talk(&mut c, (2, 4));
    assert!(text.contains("HOSTEL_COT"));
    let mut c = super::r10_gallery::continue_save(root.clone(), runtime.clone(), &mut c);
    travel(&mut c, &runtime, "MnDuluthHillsideHostel1F");
    travel(&mut c, &runtime, "MnDuluth");
    assert_eq!(
        c.snapshot().expect("hostel city return").overworld.map_name,
        "MnDuluth"
    );
}

#[test]
#[ignore = "requires CRYSTAL_INTERIOR_PACK with the harbor-care room overlay"]
fn harbor_care_clinic_counter_and_beach_rest_restore_resources_after_continue() {
    let (root, runtime, _) = load();
    for (name, arrival, approach, target) in [
        ("MnDuluthFieldClinic", (8, 9), (5, 7), (5, 6)),
        ("MnDuluthBeachShelter", (6, 9), (5, 6), (5, 5)),
    ] {
        let mut c = fixture(&root, &runtime, name, arrival);
        let partner = runtime
            .data()
            .create_pokemon("CHIKORITA", 20, Dv::default())
            .expect("legal care partner");
        let state = c.shell.shell.session_mut().state_mut();
        assert!(state.storage.party.add_pokemon(partner));
        state.sync_party_from_storage();
        state.money = 0;
        for visit in 0..2 {
            let state = c.shell.shell.session_mut().state_mut();
            let partner = state.storage.party.pokemon[0]
                .as_mut()
                .expect("care partner");
            partner.hp = 10;
            partner.status = Some("POISON".into());
            for learned in &mut partner.moves {
                learned.current_pp = 0;
            }
            state.sync_party_from_storage();
            mark_runtime_snapshot_dirty(&mut c.shell);
            walk(&mut c, &runtime, approach);
            talk(&mut c, target);
            let s = c.snapshot().expect("completed free care");
            let partner = &s.party.slots[0].pokemon;
            assert_eq!(partner.hp, partner.max_hp, "{name} visit {visit}");
            assert!(partner.status.is_none());
            assert!(
                partner
                    .moves
                    .iter()
                    .all(|m| { m.current_pp == runtime.data().moves[&m.name].pp })
            );
            assert_eq!(c.shell.shell.session().state().money, 0);
            if visit == 0 {
                c = super::r10_gallery::continue_save(root.clone(), runtime.clone(), &mut c);
                let s = c.snapshot().expect("continued care room");
                assert_eq!(s.overworld.map_name, name);
                assert_eq!((s.overworld.tile.x, s.overworld.tile.y), approach);
                travel(&mut c, &runtime, "MnDuluth");
                travel(&mut c, &runtime, name);
            }
        }
    }
}
