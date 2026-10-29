//! Explicit local fixtures followed exclusively by production controller input.
use super::*;
const APPROACH: &str = "MnDetroitLakesApproach";

#[test]
#[ignore = "requires reviewed Detroit approach pack and reached-save directory"]
fn detroit_approach_grass_native_uphill_dry_hop_and_reverse_denial() {
    let (_, r, mut c) = fixture(APPROACH, (15, 20), true, true, false);
    assert_eq!(terrain_path(&r, &mut c, (15, 17), false, true), 3);
    assert_eq!(kind(&r, APPROACH, (15, 17)), "FLOOR");
    assert_eq!(kind(&r, APPROACH, (15, 18)), "HOP_DOWN");
    assert_eq!(kind(&r, APPROACH, (15, 20)), "FLOOR");
    step(
        &mut c,
        GameButton::Down,
        APPROACH,
        (15, 18),
        "dry ledge lip",
    );
    step(
        &mut c,
        GameButton::Down,
        APPROACH,
        (15, 20),
        "native two-tile hop",
    );
    for _ in 0..3 {
        c.press(GameButton::Up).expect("native reverse attempt");
        c.wait_frames(16).expect("reverse settles");
        settle(&mut c);
        assert_at(&mut c, APPROACH, (15, 20), "reverse ledge blocked");
    }
    save(&mut c, "detroit-approach-hop");
}

#[test]
#[ignore = "requires reviewed Detroit approach pack and reached-save directory"]
fn detroit_approach_grass_native_bends_and_both_ports() {
    let (_, r, mut c) = fixture(APPROACH, (0, 18), true, true, false);
    assert!(terrain_path(&r, &mut c, (47, 18), false, true) >= 11);
    step(
        &mut c,
        GameButton::Right,
        "MnDetroitLakes",
        (0, 18),
        "east native connection",
    );
    step(
        &mut c,
        GameButton::Left,
        APPROACH,
        (47, 18),
        "east native return",
    );
    assert!(terrain_path(&r, &mut c, (0, 18), false, true) >= 13);
    let width = i16::try_from(r.data().maps["MnR16C01R00"].attributes.width * 2).unwrap();
    step(
        &mut c,
        GameButton::Left,
        "MnR16C01R00",
        (width - 1, 24),
        "west native connection",
    );
    step(
        &mut c,
        GameButton::Right,
        APPROACH,
        (0, 18),
        "west native return",
    );
    save(&mut c, "detroit-approach-ports");
}

#[test]
#[ignore = "requires reviewed Detroit approach pack and reached-save directory"]
fn detroit_approach_grass_native_wild_run_resources_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture(APPROACH, (0, 18), true, true, false);
    terrain_path(&r, &mut c, (4, 18), false, true);
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (5, 18))
        } else {
            (GameButton::Left, (4, 18))
        };
        assert_eq!(kind(&r, APPROACH, target), "TALL_GRASS");
        for _ in 0..8 {
            c.press(input).expect("native new grass step");
            c.wait_frames(16).expect("grass presentation");
            if c.snapshot().expect("wild snapshot").battle.is_some() {
                escape_wild(&mut c);
                encountered = true;
            }
            if at(&mut c) == (APPROACH.to_owned(), target) {
                break;
            }
        }
        assert_at(&mut c, APPROACH, target, "actual new grass movement");
        if encountered {
            break;
        }
    }
    assert!(encountered, "new habitat must start a native wild battle");
    let before = c.snapshot().expect("reached state");
    let path = save(&mut c, "detroit-approach-grass");
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
    if let Some(path) = std::env::var_os("CRYSTAL_DETROIT_APPROACH_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached Detroit approach save");
    }
}
