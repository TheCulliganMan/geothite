//! Explicit local fixtures followed exclusively by production controller input.
use super::*;
const PARK: &str = "MnLakeItasca";

#[test]
#[ignore = "requires reviewed Itasca pack and reached-save directory"]
fn itasca_grass_native_marker_crossing_and_dry_glade_return() {
    let (_, r, mut c) = fixture(PARK, (13, 30), true, true, false);
    assert!(terrain_path(&r, &mut c, (13, 27), false, true) >= 2);
    step(
        &mut c,
        GameButton::Down,
        PARK,
        (13, 28),
        "dry native glade lip",
    );
    step(
        &mut c,
        GameButton::Down,
        PARK,
        (13, 30),
        "dry native glade return",
    );
    for _ in 0..3 {
        c.press(GameButton::Up).expect("native reverse attempt");
        c.wait_frames(16).expect("settled reverse");
        settle(&mut c);
        assert_at(&mut c, PARK, (13, 30), "native reverse remains blocked");
    }
    terrain_path(&r, &mut c, (19, 22), false, true);
    assert_eq!(kind(&r, PARK, (19, 22)), "FLOOR");
    assert!(terrain_path(&r, &mut c, (40, 24), false, true) >= 2);
    assert!(terrain_path(&r, &mut c, (19, 22), false, true) >= 2);
    terrain_path(&r, &mut c, (16, 33), false, true);
    save(&mut c, "itasca-grass-paths");
}

#[test]
#[ignore = "requires reviewed Itasca pack and reached-save directory"]
fn itasca_grass_native_wild_run_resources_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture(PARK, (35, 22), true, true, false);
    terrain_path(&r, &mut c, (32, 22), false, true);
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (33, 22))
        } else {
            (GameButton::Left, (32, 22))
        };
        assert_eq!(kind(&r, PARK, target), "TALL_GRASS");
        for _ in 0..8 {
            c.press(input).expect("native new grass step");
            c.wait_frames(16).expect("grass presentation");
            if c.snapshot().expect("wild snapshot").battle.is_some() {
                escape_wild(&mut c);
                encountered = true;
            }
            if at(&mut c) == (PARK.to_owned(), target) {
                break;
            }
        }
        assert_at(&mut c, PARK, target, "actual new grass movement");
        if encountered {
            break;
        }
    }
    assert!(encountered, "new habitat must start a native wild battle");
    terrain_path(&r, &mut c, (35, 22), false, true);
    let before = c.snapshot().expect("reached state");
    let path = save(&mut c, "itasca-grass");
    let mut loaded =
        VisibleShellController::load_save(root, r, path, None).expect("native Continue");
    let after = loaded.snapshot().expect("continued state");
    assert_eq!(before.overworld.map_name, after.overworld.map_name);
    assert_eq!(before.overworld.tile, after.overworld.tile);
    assert_eq!(before.party, after.party);
    assert_eq!(before.bag, after.bag);
    assert_eq!(before.progression.badges, after.progression.badges);
    assert_eq!(
        before.progression.active_engine_flags,
        after.progression.active_engine_flags
    );
    assert_eq!(
        before.progression.active_event_flags,
        after.progression.active_event_flags
    );
    if let Some(path) = std::env::var_os("CRYSTAL_ITASCA_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached Itasca save");
    }
}
