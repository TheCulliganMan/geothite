//! Explicit legal local bank/badge/party fixtures, then native input only.
use super::grand_marais_harbor::use_max_repel;
use super::{
    GameButton, MovementMode, Point, VisibleShellController, assert_at, at, fixture_with_items,
    mount, settle, step,
};

const ROAD: &str = "MnR03C00R00";
const JOIN: &str = "MnMississippiConfluenceC00R00";

fn continue_path(label: &str) -> std::path::PathBuf {
    let directory = std::env::var_os("CRYSTAL_RIVER_BEND_SAVE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&directory).expect("native river Continue evidence directory");
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("capture clock")
        .as_nanos();
    let path = directory.join(format!(
        "{label}-{}-{nonce}.crystalsave",
        std::process::id()
    ));
    assert!(!path.exists(), "retain previous Continue evidence");
    path
}

fn capture(c: &mut VisibleShellController, label: &str) {
    if let Some(directory) = std::env::var_os("CRYSTAL_RIVER_BEND_SAVE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("ignored native river capture directory");
        let path = directory.join(format!("{label}.crystalsave"));
        assert!(!path.exists(), "retain every earlier river capture Save");
        c.save(path).expect("save actual reached river state");
    }
}

fn blocked(c: &mut VisibleShellController, button: GameButton, map: &str, point: Point) {
    for _ in 0..2 {
        c.press(button).expect("native blocking-edge input");
        c.wait_frames(16).expect("blocked-edge presentation");
        settle(c);
        assert_at(c, map, point, "native river-bank/rail closure");
    }
}

#[test]
#[ignore = "requires published city bend in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn river_bend_controller_foot_bridge_both_directions_rails_sign_and_continue() {
    let (_, runtime, mut c) =
        fixture_with_items(ROAD, (20, 12), true, true, true, &[("MAX_REPEL", 2)]);
    use_max_repel(&mut c, &runtime);
    for y in (0..12).rev() {
        step(
            &mut c,
            GameButton::Up,
            ROAD,
            (20, y),
            "northbound actual bridge road",
        );
        assert_eq!(
            c.snapshot().expect("foot bridge").overworld.mode,
            MovementMode::Normal
        );
    }
    for y in 1..=12 {
        step(
            &mut c,
            GameButton::Down,
            ROAD,
            (20, y),
            "southbound actual bridge road",
        );
        if y == 5 {
            step(
                &mut c,
                GameButton::Left,
                ROAD,
                (19, y),
                "legal inner west deck",
            );
            blocked(&mut c, GameButton::Left, ROAD, (19, y));
            step(
                &mut c,
                GameButton::Right,
                ROAD,
                (20, y),
                "return from west rail",
            );
            step(&mut c, GameButton::Right, ROAD, (21, y), "inner east deck");
            step(
                &mut c,
                GameButton::Right,
                ROAD,
                (22, y),
                "legal inner east deck",
            );
            blocked(&mut c, GameButton::Right, ROAD, (22, y));
            step(
                &mut c,
                GameButton::Left,
                ROAD,
                (21, y),
                "return from east rail",
            );
            step(
                &mut c,
                GameButton::Left,
                ROAD,
                (20, y),
                "return to bridge center",
            );
            capture(&mut c, "foot-bridge-southbound");
        }
    }
    for y in (9..12).rev() {
        step(&mut c, GameButton::Up, ROAD, (20, y), "north sign path");
    }
    step(
        &mut c,
        GameButton::Right,
        ROAD,
        (21, 9),
        "sign approach west",
    );
    step(
        &mut c,
        GameButton::Right,
        ROAD,
        (22, 9),
        "sign approach south",
    );
    blocked(&mut c, GameButton::Up, ROAD, (22, 9));
    c.press(GameButton::A).expect("read relocated native sign");
    let mut seen = false;
    for _ in 0..64 {
        let snapshot = c.snapshot().expect("native north sign page");
        seen |= snapshot
            .ui
            .text
            .as_ref()
            .is_some_and(|s| s.label.contains("MN_R03_NORTH_SIGN"));
        if seen && !c.has_pending_script_work() && !snapshot.ui.text_window_open {
            break;
        }
        c.wait_frames(120).expect("print native sign page");
        c.press(GameButton::A).expect("native sign page input");
        c.wait_frames(16).expect("sign presentation");
    }
    assert!(
        seen,
        "existing north sign script must produce actual dialogue"
    );
    settle(&mut c);
    assert_at(&mut c, ROAD, (22, 9), "reading sign preserves position");
    capture(&mut c, "relocated-sign-after-pages");
    let save = continue_path("river-bend-foot");
    let before = c.snapshot().expect("foot Continue preimage");
    c.save(&save).expect("native foot Save");
    let (root, runtime) = super::load();
    c = VisibleShellController::load_save(root, runtime, save.clone(), None)
        .expect("foot Continue");
    let after = c.snapshot().expect("foot Continue");
    assert_eq!(after.overworld.map_name, before.overworld.map_name);
    assert_eq!(after.overworld.tile, before.overworld.tile);
    assert_eq!(after.overworld.facing, before.overworld.facing);
    assert_eq!(after.overworld.mode, before.overworld.mode);
    assert_eq!(after.party, before.party);
    assert_eq!(after.bag, before.bag);
}

#[test]
#[ignore = "requires published city bend in CRYSTAL_MINNEAPOLIS_FALLS_CONTROLLER_PACK"]
fn river_bend_controller_six_surf_rows_both_ways_seven_seam_lanes_and_continue() {
    let (root, runtime, mut c) =
        fixture_with_items(ROAD, (20, 12), true, true, true, &[("MAX_REPEL", 2)]);
    use_max_repel(&mut c, &runtime);
    for y in (9..12).rev() {
        step(
            &mut c,
            GameButton::Up,
            ROAD,
            (20, y),
            "ordinary road to grassy launch approach",
        );
    }
    for x in (17..20).rev() {
        step(
            &mut c,
            GameButton::Left,
            ROAD,
            (x, 9),
            "ordinary grassy bank approach",
        );
    }
    step(
        &mut c,
        GameButton::Up,
        ROAD,
        (17, 8),
        "actual grassy Surf launch bank",
    );
    mount(&mut c, GameButton::Up, ROAD, (17, 7));
    let mut continued = false;
    for y in (2..=7).rev() {
        if at(&mut c).1.1 != y {
            step(
                &mut c,
                GameButton::Up,
                ROAD,
                (17, y),
                "outside-bridge row change",
            );
        }
        for x in 18..=25 {
            step(
                &mut c,
                GameButton::Right,
                ROAD,
                (x, y),
                "eastbound under bridge",
            );
            if x == 20 {
                blocked(&mut c, GameButton::Up, ROAD, (x, y));
                blocked(&mut c, GameButton::Down, ROAD, (x, y));
                if y == 4 {
                    capture(&mut c, "surf-under-widened-bridge");
                    let save = continue_path("river-bend-surf");
                    let before = c.snapshot().expect("underpass Save preimage");
                    c.save(&save).expect("production Surf Save");
                    c = VisibleShellController::load_save(
                        root.clone(),
                        runtime.clone(),
                        save.clone(),
                        None,
                    )
                    .expect("Surf Continue under bridge");
                    let after = c.snapshot().expect("Surf Continue");
                    assert_eq!(after.overworld.map_name, before.overworld.map_name);
                    assert_eq!(after.overworld.tile, before.overworld.tile);
                    assert_eq!(after.overworld.facing, before.overworld.facing);
                    assert_eq!(after.overworld.mode, before.overworld.mode);
                    assert_eq!(after.party, before.party);
                    assert_eq!(after.bag, before.bag);
                    continued = true;
                }
            }
            assert_eq!(
                c.snapshot().expect("eastbound Surf").overworld.mode,
                MovementMode::Surf
            );
        }
        for x in (17..25).rev() {
            step(
                &mut c,
                GameButton::Left,
                ROAD,
                (x, y),
                "westbound under bridge",
            );
            assert_eq!(
                c.snapshot().expect("westbound Surf").overworld.mode,
                MovementMode::Surf
            );
        }
    }
    assert!(continued);
    for x in 18..=46 {
        step(
            &mut c,
            GameButton::Right,
            ROAD,
            (x, 2),
            "widened river approach to actual east seam",
        );
    }
    for y in 2..=8 {
        if y > 2 {
            step(
                &mut c,
                GameButton::Down,
                ROAD,
                (46, y),
                "river seam lane change",
            );
        }
        step(&mut c, GameButton::Right, ROAD, (47, y), "east river edge");
        step(
            &mut c,
            GameButton::Right,
            JOIN,
            (0, y),
            "actual east water connection",
        );
        step(
            &mut c,
            GameButton::Right,
            JOIN,
            (1, y),
            "neighbor water lane",
        );
        step(
            &mut c,
            GameButton::Left,
            JOIN,
            (0, y),
            "neighbor return edge",
        );
        step(
            &mut c,
            GameButton::Left,
            ROAD,
            (47, y),
            "actual west return connection",
        );
        step(&mut c, GameButton::Left, ROAD, (46, y), "west water lane");
        assert_eq!(
            c.snapshot().expect("connected Surf seam").overworld.mode,
            MovementMode::Surf
        );
    }
    blocked(&mut c, GameButton::Down, ROAD, (46, 8));
    capture(&mut c, "south-bank-at-widened-water-seam");
}
