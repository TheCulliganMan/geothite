//! Explicit local fixtures followed exclusively by production controller input.
use super::*;
const PASS: &str = "MnElyShorePass";

#[test]
#[ignore = "requires reviewed Ely pass pack and reached-save directory"]
fn ely_shore_pass_grass_native_bends_and_both_ports() {
    let (_, r, mut c) = fixture(PASS, (0, 14), true, true, false);
    assert_eq!(terrain_path(&r, &mut c, (13, 10), false, true), 5);
    step(
        &mut c,
        GameButton::Right,
        "MnR11C00R02",
        (0, 30),
        "native warm-coast connection",
    );
    step(
        &mut c,
        GameButton::Left,
        PASS,
        (13, 10),
        "native pass return",
    );
    assert_eq!(terrain_path(&r, &mut c, (0, 14), false, true), 5);
    let width = i16::try_from(r.data().maps["MnEly"].attributes.width * 2).unwrap();
    step(
        &mut c,
        GameButton::Left,
        "MnEly",
        (width - 1, 14),
        "native inland connection",
    );
    step(
        &mut c,
        GameButton::Right,
        PASS,
        (0, 14),
        "native inland return",
    );
    terrain_path(&r, &mut c, (9, 14), false, true);
    let text = super::super::super::interior_life::talk(&mut c, (8, 14));
    assert!(
        !text.is_empty(),
        "native trail guide remains reachable and readable"
    );
    save(&mut c, "ely-pass-ports-and-guide");
}

#[test]
#[ignore = "requires reviewed Ely pass pack and reached-save directory"]
fn ely_shore_pass_grass_native_wild_run_resources_and_continue() {
    use super::super::super::authored_route_fixture::escape_wild;
    let (root, r, mut c) = fixture(PASS, (13, 10), true, true, false);
    terrain_path(&r, &mut c, (10, 10), false, true);
    let mut encountered = false;
    for index in 0..256 {
        let (input, target) = if index % 2 == 0 {
            (GameButton::Right, (11, 10))
        } else {
            (GameButton::Left, (10, 10))
        };
        assert_eq!(kind(&r, PASS, target), "TALL_GRASS");
        for _ in 0..8 {
            c.press(input).expect("native new grass step");
            c.wait_frames(16).expect("grass presentation");
            if c.snapshot().expect("wild snapshot").battle.is_some() {
                escape_wild(&mut c);
                encountered = true;
            }
            if at(&mut c) == (PASS.to_owned(), target) {
                break;
            }
        }
        assert_at(&mut c, PASS, target, "actual new grass movement");
        if encountered {
            break;
        }
    }
    assert!(encountered, "new habitat must start a native wild battle");
    terrain_path(&r, &mut c, (12, 10), false, true);
    let before = c.snapshot().expect("reached state");
    let path = save(&mut c, "ely-pass-grass");
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
    if let Some(path) = std::env::var_os("CRYSTAL_ELY_PASS_GRASS_REACHED_SAVE") {
        loaded
            .save(PathBuf::from(path))
            .expect("retain reached Ely pass save");
    }
}
