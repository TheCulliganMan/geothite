//! Declared local field fixtures; native inputs mount, encounter, catch and save.
use crystal_core::input::GameButton;
use crystal_core::models::Dv;
use crystal_core::world::movement::MovementMode;
use serde_json::Value;

use super::{checkpoint, encounter_and_capture, fixture, fixture_at_hour, verify_capture};
use crate::bevy_shell::VisibleShellController;
use crate::bevy_shell::tests::authored_route_fixture::indoor_walk;
use crate::bevy_shell::tests::interior_life::talk;
use crate::bevy_shell::tests::r10_gallery::continue_save;
use crate::bevy_shell::tests::winona::{at, press};

fn source() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../planning/minnesota/pokemon/acquisition-v01/water-caves-v01/habitats.json"
    ))
    .expect("authored habitat contract")
}

fn tile(value: &Value) -> [i16; 2] {
    serde_json::from_value(value.clone()).expect("core tile pair")
}

fn direction(a: [i16; 2], b: [i16; 2]) -> GameButton {
    match (b[0] - a[0], b[1] - a[1]) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        _ => panic!("adjacent native surface cells required"),
    }
}

fn mount(controller: &mut VisibleShellController, shore: [i16; 2], water: [i16; 2]) {
    assert_eq!(
        controller.snapshot().expect("shore").overworld.mode,
        MovementMode::Normal
    );
    press(controller, direction(shore, water));
    for _ in 0..128 {
        press(controller, GameButton::A);
        if controller.snapshot().expect("native Surf").overworld.mode == MovementMode::Surf {
            controller.wait_frames(60).expect("mount presentation");
            let arrived = controller.snapshot().expect("actual water tile");
            assert_eq!([arrived.overworld.tile.x, arrived.overworld.tile.y], water);
            return;
        }
        controller.wait_frames(16).expect("native Surf text");
    }
    panic!("native Surf did not mount from {shore:?} to {water:?}");
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with complete water/cave habitats"]
fn water_caves_all19_native_shores_mount_capture_dex_and_continue() {
    let plan = source();
    for habitat in plan["maps"].as_array().expect("map plans") {
        let water = &habitat["water"];
        if water.is_null() {
            continue;
        }
        let name = habitat["map"].as_str().expect("map name");
        let shore = tile(&water["shore"]);
        let sample = tile(&water["sample"]);
        let next = tile(&water["loop"]);
        let target = water["table"]["day"][0]["species"]
            .as_str()
            .expect("local resident");
        let (root, runtime, mut controller) = fixture(name, shore, true, false);
        checkpoint(&mut controller, &format!("{name}-water-shore"));
        mount(&mut controller, shore, sample);
        encounter_and_capture(&mut controller, &runtime, target, sample, next);
        verify_capture(&controller, &runtime, target);
        controller = continue_save(root, runtime.clone(), &mut controller);
        verify_capture(&controller, &runtime, target);
        checkpoint(&mut controller, &format!("{name}-water-caught"));
        eprintln!("verified Surf habitat {name}: {target}");
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with complete water/cave habitats"]
fn water_caves_all10_floor_loops_capture_local_residents_and_continue() {
    let plan = source();
    for habitat in plan["maps"].as_array().expect("map plans") {
        let walking = &habitat["walking"];
        if walking.is_null() {
            continue;
        }
        let name = habitat["map"].as_str().expect("map name");
        let sample = tile(&walking["sample"]);
        let next = tile(&walking["loop"]);
        let target = walking["table"]["day"][0]["species"]
            .as_str()
            .expect("local cave resident");
        let (root, runtime, mut controller) = fixture(name, sample, false, false);
        checkpoint(&mut controller, &format!("{name}-floor-start"));
        encounter_and_capture(&mut controller, &runtime, target, sample, next);
        verify_capture(&controller, &runtime, target);
        controller = continue_save(root, runtime.clone(), &mut controller);
        verify_capture(&controller, &runtime, target);
        checkpoint(&mut controller, &format!("{name}-floor-caught"));
        eprintln!("verified cave habitat {name}: {target}");
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with complete water/cave habitats"]
fn water_caves_night_burbot_sources_use_real_clock_and_native_capture() {
    let plan = source();
    for name in ["MnBemidji", "MnDuluth", "MnMoorhead"] {
        let habitat = plan["maps"]
            .as_array()
            .expect("map plans")
            .iter()
            .find(|h| h["map"] == name)
            .expect("night source");
        let water = &habitat["water"];
        let shore = tile(&water["shore"]);
        let sample = tile(&water["sample"]);
        let (root, runtime, mut controller) = fixture_at_hour(name, shore, true, false, 0);
        mount(&mut controller, shore, sample);
        encounter_and_capture(
            &mut controller,
            &runtime,
            "KEYPOUT",
            sample,
            tile(&water["loop"]),
        );
        verify_capture(&controller, &runtime, "KEYPOUT");
        controller = continue_save(root, runtime.clone(), &mut controller);
        verify_capture(&controller, &runtime, "KEYPOUT");
        checkpoint(&mut controller, &format!("{name}-night-burbot"));
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with Surf-only refuge/facility tables"]
fn water_caves_peaceful_refuge_and_facility_floors_keep_walking() {
    let plan = source();
    for name in [
        "MnR12Cave1F",
        "MnR13Cave1F",
        "MnBoundaryIntake",
        "MnIronSump",
    ] {
        let habitat = plan["maps"]
            .as_array()
            .expect("map plans")
            .iter()
            .find(|h| h["map"] == name)
            .expect("peaceful spring");
        let shore = tile(&habitat["water"]["shore"]);
        let (_, runtime, mut controller) = fixture(name, shore, false, false);
        let map = &runtime.data().maps[name];
        let next =
            crate::bevy_shell::tests::authored_route_fixture::neighbors((shore[0], shore[1]))
                .into_iter()
                .map(|(tile, _)| tile)
                .find(|&(x, y)| {
                    if x < 0
                        || y < 0
                        || x >= map.attributes.width as i16 * 2
                        || y >= map.attributes.height as i16 * 2
                    {
                        return false;
                    }
                    if map
                        .objects
                        .iter()
                        .any(|o| o.x as i16 == x && o.y as i16 == y)
                    {
                        return false;
                    }
                    let block = map.blocks
                        [y as usize / 2 * usize::from(map.attributes.width) + x as usize / 2];
                    runtime.data().tilesets[&map.attributes.tileset_name].collision
                        [&format!("{block:02x}")][y as usize % 2 * 2 + x as usize % 2]
                        == "FLOOR"
                })
                .expect("adjacent peaceful bank floor");
        let next = [next.0, next.1];
        for step in 0..128 {
            let snapshot = controller.snapshot().expect("peaceful bank");
            assert!(snapshot.battle.is_none());
            let from = [snapshot.overworld.tile.x, snapshot.overworld.tile.y];
            let to = if from == shore { next } else { shore };
            for _ in 0..4 {
                press(&mut controller, direction(from, to));
                let arrived = controller.snapshot().expect("actual dry step");
                assert!(arrived.battle.is_none(), "peaceful {name} step {step}");
                if [arrived.overworld.tile.x, arrived.overworld.tile.y] == to {
                    break;
                }
            }
            at(&mut controller, name, (to[0], to[1]));
        }
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with a Surf-only refuge spring"]
fn water_caves_sweet_scent_on_peaceful_floor_uses_native_failure_and_continue() {
    let (root, runtime) = crate::bevy_shell::tests::winona::load();
    let mut controller =
        crate::bevy_shell::tests::interior_life::fixture(&root, &runtime, "MnR13Cave1F", (7, 12));
    let partner = runtime
        .data()
        .create_pokemon("POUCHID", 10, Dv::default())
        .expect("legal level9 Sweet Scent learner");
    let pp = partner
        .moves
        .iter()
        .find(|m| m.name == "SWEET_SCENT")
        .expect("naturally learned move")
        .current_pp;
    assert!(
        controller
            .shell
            .shell
            .session_mut()
            .state_mut()
            .storage
            .party
            .add_pokemon(partner)
    );
    controller
        .shell
        .shell
        .session_mut()
        .state_mut()
        .sync_party_from_storage();
    crate::bevy_shell::mark_runtime_snapshot_dirty(&mut controller.shell);
    // Only real menu, field-move, text and Save/Continue input follows setup.
    for _ in 0..2 {
        crate::bevy_shell::tests::minnesota_field_use::use_move(
            &mut controller,
            crate::bevy_shell::PartyFieldMove::SweetScent,
        );
        let mut text = String::new();
        for _ in 0..128 {
            let snapshot = controller.snapshot().expect("Sweet Scent result");
            assert!(
                snapshot.battle.is_none(),
                "quiet floor has no land encounter"
            );
            at(&mut controller, "MnR13Cave1F", (7, 12));
            if let Some(page) = snapshot.ui.text.and_then(|page| page.asm_text) {
                text.push_str(&page);
            }
            if snapshot.phase == crystal_runtime::RuntimeShellPhase::Overworld
                && !snapshot.ui.text_window_open
                && !controller.has_pending_script_work()
                && controller.shell.field_notice.is_none()
            {
                break;
            }
            press(&mut controller, GameButton::A);
        }
        assert!(
            text.to_ascii_lowercase().contains("nothing"),
            "native failure text: {text}"
        );
        let state = controller.shell.shell.session().state();
        let partner = state.storage.party.pokemon[0]
            .as_ref()
            .expect("retained partner");
        assert_eq!(
            partner
                .moves
                .iter()
                .find(|m| m.name == "SWEET_SCENT")
                .expect("move")
                .current_pp,
            pp
        );
        controller = continue_save(root.clone(), runtime.clone(), &mut controller);
    }
}

#[test]
#[ignore = "requires CRYSTAL_WINONA_TEST_PACK with the bounded Moorhead river access"]
fn water_caves_moorhead_sign_bank_fishing_and_bounded_surf_are_real_input() {
    let (_, runtime, mut controller) = fixture("MnMoorhead", [12, 15], true, true);
    let text = talk(&mut controller, (12, 14));
    assert!(text.contains("RED RIVER ACCESS") && text.contains("SURF"));
    indoor_walk(&mut controller, &runtime, (10, 12));
    at(&mut controller, "MnMoorhead", (10, 12));
    press(&mut controller, GameButton::Left);
    let mut caught = false;
    for cast in 0..128 {
        controller
            .press(GameButton::Select)
            .expect("registered Good Rod");
        assert!(controller.shell.visible_fishing_animation.is_some());
        for _ in 0..256 {
            if controller
                .shell
                .visible_fishing_animation
                .as_ref()
                .is_some_and(|animation| {
                    animation.phase == crate::bevy_shell::VisibleFishingPhase::AwaitText
                })
            {
                break;
            }
            controller.wait_frames(1).expect("casting clock");
        }
        for _ in 0..128 {
            let snapshot = controller.snapshot().expect("fishing notice");
            assert_eq!(
                [snapshot.overworld.tile.x, snapshot.overworld.tile.y],
                [10, 12]
            );
            if snapshot.battle.is_some()
                || (controller.shell.visible_fishing_animation.is_none()
                    && controller.shell.field_notice.is_none()
                    && !controller.has_pending_script_work())
            {
                break;
            }
            press(&mut controller, GameButton::A);
        }
        if let Some(battle) = controller.snapshot().expect("actual bite").battle {
            if battle.enemy_pokemon.species.id == "SCUTLET" {
                assert!(
                    crate::bevy_shell::tests::rochester_earned::try_recruit_with_ball(
                        &mut controller,
                        &runtime,
                        "MASTER_BALL"
                    )
                );
                verify_capture(&controller, &runtime, "SCUTLET");
                caught = true;
                break;
            }
            crate::bevy_shell::tests::rochester::flee(&mut controller);
        }
        controller
            .wait_frames(17 + cast % 11)
            .expect("ordinary bank idle");
    }
    assert!(caught, "real levee Good Rod encounter and capture");
    mount(&mut controller, [10, 12], [9, 12]);
    checkpoint(&mut controller, "moorhead-river-entry");
    // Boundary cells are all visible native BUOY. This direct movement test
    // supplements the publisher's full perimeter/Surf graph checks.
    for _ in 0..32 {
        if controller.snapshot().expect("river").battle.is_some() {
            crate::bevy_shell::tests::rochester::flee(&mut controller);
        }
        press(&mut controller, GameButton::Left);
    }
    let snapshot = controller.snapshot().expect("closed river edge");
    assert_eq!(snapshot.overworld.map_name, "MnMoorhead");
    assert_eq!(snapshot.overworld.tile.x, 1);
    for _ in 0..8 {
        press(&mut controller, GameButton::Left);
    }
    assert_eq!(
        controller.snapshot().expect("buoy denial").overworld.tile.x,
        1
    );
}
