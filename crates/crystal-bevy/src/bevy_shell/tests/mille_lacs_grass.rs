//! Explicit local fixtures followed exclusively by production controller input.
use super::*;
const PARK: &str = "MnMilleLacs";

#[test]
#[ignore = "requires reviewed Mille Lacs pack and reached-save directory"]
fn mille_lacs_grass_native_activity_crossings_and_dry_ledge() {
    let (_, r, mut c) = fixture(PARK, (17, 20), true, true, false);
    assert_eq!(terrain_path(&r, &mut c, (17, 17), false, true), 3);
    step(
        &mut c,
        GameButton::Down,
        PARK,
        (17, 18),
        "dry native lookout lip",
    );
    step(
        &mut c,
        GameButton::Down,
        PARK,
        (17, 20),
        "dry native lookout hop",
    );
    for _ in 0..3 {
        c.press(GameButton::Up).expect("reverse ledge attempt");
        c.wait_frames(16).expect("settled reverse");
        settle(&mut c);
        assert_at(&mut c, PARK, (17, 20), "native reverse remains blocked");
    }
    terrain_path(&r, &mut c, (2, 12), false, true);
    assert!(terrain_path(&r, &mut c, (39, 10), false, true) >= 2);
    assert!(terrain_path(&r, &mut c, (2, 12), false, true) >= 2);
    assert!(terrain_path(&r, &mut c, (12, 30), false, true) >= 4);
    assert!(terrain_path(&r, &mut c, (2, 12), false, true) >= 4);
    save(&mut c, "mille-lacs-activity-walk");
}

#[test]
#[ignore = "requires reviewed Mille Lacs pack and reached-save directory"]
fn mille_lacs_grass_native_wild_run_resources_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture(PARK, (11, 22), true, true, false);
    terrain_path(&r, &mut c, (11, 24), false, true);
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (12, 24))
        } else {
            (GameButton::Left, (11, 24))
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
    terrain_path(&r, &mut c, (12, 26), false, true);
    let before = c.snapshot().expect("reached state");
    let path = save(&mut c, "mille-lacs-grass");
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
    if let Some(path) = std::env::var_os("CRYSTAL_MILLE_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached Mille Lacs save");
    }
}
