//! Explicit local fixtures followed exclusively by production controller input.
use super::*;
const WOODS: &str = "MnBemidjiNorthwoods";

#[test]
#[ignore = "requires reviewed Northwoods pack and reached-save directory"]
fn bemidji_northwoods_grass_native_uphill_dry_hop_and_reverse_denial() {
    let (_, r, mut c) = fixture(WOODS, (7, 16), true, true, false);
    assert_eq!(terrain_path(&r, &mut c, (7, 13), false, true), 5);
    assert_eq!(kind(&r, WOODS, (7, 13)), "FLOOR");
    assert_eq!(kind(&r, WOODS, (7, 14)), "HOP_DOWN");
    assert_eq!(kind(&r, WOODS, (7, 16)), "FLOOR");
    step(&mut c, GameButton::Down, WOODS, (7, 14), "dry ledge lip");
    step(
        &mut c,
        GameButton::Down,
        WOODS,
        (7, 16),
        "native two-tile hop",
    );
    for _ in 0..3 {
        c.press(GameButton::Up).expect("native reverse attempt");
        c.wait_frames(16).expect("reverse settles");
        settle(&mut c);
        assert_at(&mut c, WOODS, (7, 16), "reverse ledge blocked");
    }
    save(&mut c, "northwoods-hop");
}

#[test]
#[ignore = "requires reviewed Northwoods pack and reached-save directory"]
fn bemidji_northwoods_grass_native_bends_and_both_ports() {
    let (_, r, mut c) = fixture(WOODS, (8, 27), true, true, false);
    assert!(terrain_path(&r, &mut c, (8, 0), false, true) >= 11);
    let north_height = i16::try_from(r.data().maps["MnR17C00R02"].attributes.height * 2).unwrap();
    step(
        &mut c,
        GameButton::Up,
        "MnR17C00R02",
        (32, north_height - 1),
        "north native connection",
    );
    step(
        &mut c,
        GameButton::Down,
        WOODS,
        (8, 0),
        "north native return",
    );
    assert!(terrain_path(&r, &mut c, (8, 27), false, true) >= 8);
    step(
        &mut c,
        GameButton::Down,
        "MnBemidji",
        (24, 0),
        "south native connection",
    );
    step(
        &mut c,
        GameButton::Up,
        WOODS,
        (8, 27),
        "south native return",
    );
    save(&mut c, "northwoods-ports");
}

#[test]
#[ignore = "requires reviewed Northwoods pack and reached-save directory"]
fn bemidji_northwoods_grass_native_wild_run_resources_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture(WOODS, (8, 0), true, true, false);
    terrain_path(&r, &mut c, (8, 2), false, true);
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (9, 2))
        } else {
            (GameButton::Left, (8, 2))
        };
        assert_eq!(kind(&r, WOODS, target), "TALL_GRASS");
        for _ in 0..8 {
            c.press(input).expect("native new grass step");
            c.wait_frames(16).expect("grass presentation");
            if c.snapshot().expect("wild snapshot").battle.is_some() {
                escape_wild(&mut c);
                encountered = true;
            }
            if at(&mut c) == (WOODS.to_owned(), target) {
                break;
            }
        }
        assert_at(&mut c, WOODS, target, "actual new grass movement");
        if encountered {
            break;
        }
    }
    assert!(encountered, "new habitat must start a native wild battle");
    let before = c.snapshot().expect("reached state");
    let path = save(&mut c, "northwoods-grass");
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
    if let Some(path) = std::env::var_os("CRYSTAL_NORTHWOODS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached Northwoods save");
    }
}
