//! Initial local Floor/ice positions are fixtures; slides and saves are native.
use std::path::PathBuf;

use crystal_core::input::GameButton;
use crystal_core::world::movement::MovementMode;
use serde_json::{Value, json};

use super::super::VisibleShellController;
use super::minnesota_fruit_groves::fixture;
use super::{assert_at, at, load};

fn point(value: &Value) -> (i16, i16) {
    (
        value[0].as_i64().expect("x") as i16,
        value[1].as_i64().expect("y") as i16,
    )
}
fn button(value: &Value) -> GameButton {
    match point(value) {
        (1, 0) => GameButton::Right,
        (-1, 0) => GameButton::Left,
        (0, 1) => GameButton::Down,
        (0, -1) => GameButton::Up,
        other => panic!("non-cardinal frozen input {other:?}"),
    }
}
fn setup(case: &Value) -> VisibleShellController {
    fixture(
        &json!({"map":case["map"],"approach":case["start"],"item":"BERRY"}),
        0,
    )
}

#[test]
#[ignore = "requires the written frozen-water pack and its matching collision proof"]
fn frozen_lakes_native_slides_seams_crystal_rock_edges_and_continue() {
    let path = PathBuf::from(
        std::env::var_os("CRYSTAL_FROZEN_WATER_PROOF").expect("written collision proof"),
    );
    let proof: Value = serde_json::from_slice(&std::fs::read(path).expect("written proof bytes"))
        .expect("written cases");
    use sha2::{Digest, Sha256};
    let pack = PathBuf::from(
        std::env::var_os("CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK")
            .expect("published winter pack"),
    );
    assert_eq!(
        proof["pack_sha256"].as_str().expect("written pack hash"),
        format!(
            "{:x}",
            Sha256::digest(std::fs::read(pack).expect("pack bytes"))
        )
    );
    let directory = PathBuf::from(
        std::env::var_os("CRYSTAL_FROZEN_WATER_SAVES").expect("fresh retained save epoch"),
    );
    std::fs::create_dir_all(&directory).expect("retained native save directory");
    let (root, runtime) = load();
    for case in proof["cases"].as_array().expect("native ice courses") {
        let id = case["id"].as_str().expect("course id");
        let mut c = setup(case);
        let start = (
            case["map"].as_str().expect("initial map").to_owned(),
            point(&case["start"]),
        );
        let stop = (
            case["stop"]["map"].as_str().expect("stop map").to_owned(),
            point(&case["stop"]["point"]),
        );
        let save = directory.join(format!("{id}-entry.crystalsave"));
        assert!(!save.exists(), "retain earlier entry captures");
        c.save(save).expect("initial native ice approach save");
        for _ in 0..4 {
            c.press(button(&case["direction"]))
                .expect("turn and enter slippery ice");
            c.wait_frames(1).expect("first native slide frame");
            if at(&mut c) != start {
                break;
            }
        }
        assert_ne!(at(&mut c), start, "ice entry moved");
        c.wait_frames((case["path"].as_array().expect("ice path").len() + 4) * 24)
            .expect("native continuous sliding clock");
        assert_eq!(at(&mut c), stop, "exact rock stop: {id}");
        assert_eq!(
            c.snapshot().expect("native ice mode").overworld.mode,
            MovementMode::Normal
        );
        for _ in 0..4 {
            c.press(button(&case["direction"]))
                .expect("attempt to pass solid cave rock");
            c.wait_frames(24).expect("blocked step presentation");
            assert_eq!(at(&mut c), stop, "crystal cave rock blocks {id}");
        }
        let save = directory.join(format!("{id}-rock-stop.crystalsave"));
        assert!(!save.exists(), "retain earlier rock-stop captures");
        c.save(&save).expect("reached rock-stop save");
        c = VisibleShellController::load_save(root.clone(), runtime.clone(), save, None)
            .expect("native Continue on ice");
        c.press(button(&case["direction"]))
            .expect("continued rock boundary");
        c.wait_frames(24).expect("continued stop presentation");
        assert_eq!(at(&mut c), stop, "continued boundary: {id}");
        eprintln!(
            "native frozen course passed: {id}, {} tiles, {} seams",
            case["path"].as_array().expect("path").len(),
            case["seams"]
        );
    }
    for case in proof["edge_cases"].as_array().expect("outer rock edges") {
        let mut c = setup(case);
        let map = case["map"].as_str().expect("edge map");
        let start = point(&case["start"]);
        for _ in 0..4 {
            c.press(button(&case["direction"]))
                .expect("native attempt toward outer map edge");
            c.wait_frames(24).expect("blocked native edge frame");
            assert_at(
                &mut c,
                map,
                start,
                "outer Crystal-cave rock blocks ice exit",
            );
        }
        eprintln!(
            "native outer ice edge passed: {map} {:?}",
            case["direction"]
        );
    }
}
